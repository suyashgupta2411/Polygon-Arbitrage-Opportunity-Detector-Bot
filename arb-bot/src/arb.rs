use crate::dex::{parse_addr, Dex};
use anyhow::Result;
use ethers::abi::Address;
use ethers::types::U256;

#[derive(Clone)]
pub struct Pair {
    pub token_in: Address,   // USDC when buying WETH (USDC -> WETH)
    pub token_out: Address,  // WETH then sold back to USDC
}

pub struct ArbParams {
    pub trade_size_usdc: f64,
    pub min_profit_usdc: f64,
    pub gas_cost_usdc: f64,
}

pub struct ArbResult {
    pub dex_buy: String,
    pub dex_sell: String,
    pub usdc_in: f64,
    pub weth_acquired: f64,
    pub usdc_out: f64,
    pub price_buy: f64,   // USDC per WETH implied on buy DEX
    pub price_sell: f64,  // USDC per WETH implied on sell DEX
    pub gross_profit: f64,
    pub net_profit: f64,
}

const USDC_DECIMALS: u32 = 6;
const WETH_DECIMALS: u32 = 18;

/// Simulate buy on A (USDC->WETH) and sell on B (WETH->USDC).
pub async fn simulate_roundtrip(
    dex_buy: &Dex,
    dex_sell: &Dex,
    pair: &Pair,
    params: &ArbParams,
) -> Result<ArbResult> {
    let usdc_in_u256 = to_units(params.trade_size_usdc, USDC_DECIMALS);

    // Buy WETH with USDC on dex_buy
    let path_buy = vec![pair.token_in, pair.token_out];
    let weth_out_u256 = dex_buy.get_amount_out(usdc_in_u256, path_buy).await?;

    // Sell WETH back to USDC on dex_sell
    let path_sell = vec![pair.token_out, pair.token_in];
    let usdc_out_u256 = dex_sell.get_amount_out(weth_out_u256, path_sell).await?;

    let usdc_in = from_units(usdc_in_u256, USDC_DECIMALS);
    let weth_acquired = from_units(weth_out_u256, WETH_DECIMALS);
    let usdc_out = from_units(usdc_out_u256, USDC_DECIMALS);

    // Prices implied by the two legs
    let price_buy = usdc_in / weth_acquired.max(1e-18);  // USDC per WETH paid
    let price_sell = usdc_out / weth_acquired.max(1e-18);// USDC per WETH received

    let gross_profit = usdc_out - usdc_in;
    let net_profit = gross_profit - params.gas_cost_usdc;

    Ok(ArbResult {
        dex_buy: dex_buy.name.clone(),
        dex_sell: dex_sell.name.clone(),
        usdc_in,
        weth_acquired,
        usdc_out,
        price_buy,
        price_sell,
        gross_profit,
        net_profit,
    })
}

/// Basic helpers for decimal math
pub fn to_units(amount: f64, decimals: u32) -> U256 {
    let scale = 10u128.pow(decimals);
    let value = (amount * scale as f64).round() as u128;
    U256::from(value)
}


pub fn from_units(x: U256, decimals: u32) -> f64 {
    
    let scale = 10u128.pow(decimals) as f64;
    let hi = x / U256::exp10(18);  
    let lo = x % U256::exp10(18);
    (hi.as_u128() as f64) * 1e18 / scale + (lo.as_u128() as f64) / scale
}

pub fn mk_pair(usdc: &str, weth: &str) -> Pair {
    Pair {
        token_in: parse_addr(usdc),
        token_out: parse_addr(weth),
    }
}
