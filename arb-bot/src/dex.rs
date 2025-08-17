use anyhow::Result;
use ethers::abi::Address;
use ethers::prelude::*;
// remove: use std::sync::Arc; // not needed if you use `.into()`

abigen!(
    IUniswapV2Router02,
    r#"[
        function getAmountsOut(uint256 amountIn, address[] calldata path) external view returns (uint256[] memory amounts)
    ]"#
);

#[derive(Clone)]
pub struct Dex {
    pub name: String,
    pub router: IUniswapV2Router02<Provider<Http>>,
}

impl Dex {
    pub fn new(name: &str, router_addr: Address, provider: Provider<Http>) -> Self {
        Self {
            name: name.to_string(),
            router: IUniswapV2Router02::new(router_addr, provider.into()), // <-- convert to Arc<_>
        }
    }

    pub async fn get_amount_out(&self, amount_in: U256, path: Vec<Address>) -> Result<U256> {
        let amounts: Vec<U256> = self.router.get_amounts_out(amount_in, path).call().await?;
        Ok(amounts
            .last()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("empty amounts"))?)
    }
}

pub fn parse_addr(s: &str) -> Address {
    s.parse::<Address>().expect("invalid address in config")
}

pub fn mk_provider(rpc_url: &str) -> Provider<Http> {
    Provider::<Http>::try_from(rpc_url)
        .expect("bad RPC URL")
        .interval(std::time::Duration::from_millis(300))
}
