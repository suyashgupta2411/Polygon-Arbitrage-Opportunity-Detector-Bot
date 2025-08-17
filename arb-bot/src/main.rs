mod arb;
mod config;
mod db;
mod dex;

use crate::arb::{mk_pair, simulate_roundtrip, ArbParams};
use crate::config::Config;
use crate::db::Db;
use crate::dex::{mk_provider, parse_addr, Dex};
use anyhow::Result;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let cfg = Config::load()?;
    info!("Loaded config");

    // DB
    let db = Db::new(&cfg.db_path)?;
    info!("DB ready at {}", cfg.db_path);

    // Provider
    let provider = mk_provider(&cfg.rpc_url);

    // DEXes
    let dex_quick = Dex::new(
        "QuickSwap",
        parse_addr(&cfg.quickswap_router),
        provider.clone(),
    );
    let dex_sushi = Dex::new(
        "SushiSwap",
        parse_addr(&cfg.sushiswap_router),
        provider.clone(),
    );

    // Pair: USDC <-> WETH
    let pair = mk_pair(&cfg.usdc, &cfg.weth);

    let params = ArbParams {
        trade_size_usdc: cfg.trade_size_usdc,
        min_profit_usdc: cfg.min_profit_usdc,
        gas_cost_usdc: cfg.gas_cost_usdc,
    };

    let mut interval = tokio::time::interval(cfg.check_interval);

    info!(
        "Starting loop: every {:?}, trade {:.2} USDC, min profit {:.2} USDC (gas {:.2})",
        cfg.check_interval, cfg.trade_size_usdc, cfg.min_profit_usdc, cfg.gas_cost_usdc
    );

    loop {
        interval.tick().await;

        // Check both directions:
        // 1) Buy on QuickSwap, sell on Sushi
        // 2) Buy on Sushi, sell on QuickSwap
        for (buy, sell) in [(&dex_quick, &dex_sushi), (&dex_sushi, &dex_quick)] {
            match simulate_roundtrip(buy, sell, &pair, &params).await {
                Ok(res) => {
                    let direction = format!("{} -> {}", res.dex_buy, res.dex_sell);
                    if res.net_profit >= params.min_profit_usdc {
                        info!(
                            "🔥 Opportunity {} | buy {:.6} WETH @ {:.4} USDC, sell @ {:.4} USDC | gross {:.4} USDC, net {:.4} USDC",
                            direction, res.weth_acquired, res.price_buy, res.price_sell, res.gross_profit, res.net_profit
                        );
                        if let Err(e) = db.insert_opportunity(
                            &res.dex_buy,
                            &res.dex_sell,
                            "USDC",
                            "WETH",
                            &format!("{:.6}", res.usdc_in),
                            &format!("{:.6}", res.usdc_out),
                            res.price_buy,
                            res.price_sell,
                            res.gross_profit,
                            res.net_profit,
                        ) {
                            error!("DB insert failed: {e:?}");
                        }
                    } else {
                        info!(
                            "No-op {} | net profit {:.4} < min {:.4}",
                            direction, res.net_profit, params.min_profit_usdc
                        );
                    }
                }
                Err(e) => error!("Simulation error on {} -> {}: {e:?}", buy.name, sell.name),
            }
        }
    }
}
