use anyhow::{anyhow, Result};
use dotenvy::dotenv;
use serde::Deserialize;
use std::env;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub rpc_url: String,

    pub usdc: String,
    pub weth: String,

    pub quickswap_router: String,
    pub sushiswap_router: String,

    pub trade_size_usdc: f64,
    pub min_profit_usdc: f64,
    pub gas_cost_usdc: f64,
    pub check_interval: Duration,

    pub db_path: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv().ok();

        let rpc_url = match env::var("RPC_URL") {
    Ok(v) => v,
    Err(_) => {
        let key = env::var("ALCHEMY_KEY")?; // bubbles VarError into anyhow
        format!("https://polygon-mainnet.g.alchemy.com/v2/{key}")
    }
};


        let usdc = env::var("USDC")?;
        let weth = env::var("WETH")?;

        let quickswap_router = env::var("QUICKSWAP_ROUTER")?;
        let sushiswap_router = env::var("SUSHISWAP_ROUTER")?;

        let trade_size_usdc = env::var("TRADE_SIZE_USDC")?.parse::<f64>()?;
        let min_profit_usdc = env::var("MIN_PROFIT_USDC")?.parse::<f64>()?;
        let gas_cost_usdc = env::var("GAS_COST_USDC")?.parse::<f64>()?;
        let check_interval =
            Duration::from_secs(env::var("CHECK_INTERVAL_SECS")?.parse::<u64>()?);

        let db_path = env::var("DB_PATH").unwrap_or_else(|_| "arb.db".to_string());

        if trade_size_usdc <= 0.0 {
            return Err(anyhow!("TRADE_SIZE_USDC must be > 0"));
        }

        Ok(Self {
            rpc_url,
            usdc,
            weth,
            quickswap_router,
            sushiswap_router,
            trade_size_usdc,
            min_profit_usdc,
            gas_cost_usdc,
            check_interval,
            db_path,
        })
    }
}
