use alloy::{
    primitives::{Address, Uint},
    providers::RootProvider,
};
use anyhow::Result;
use ethereum_abi::IUniswapV2Pair;
use std::sync::Arc;

pub async fn get_reserves(
    pair_adr: &Address,
    provider: Arc<RootProvider>,
) -> Result<(Uint<112, 2>, Uint<112, 2>)> {
    let pair_instance = IUniswapV2Pair::new(*pair_adr, provider);
    let reserves = pair_instance.getReserves().call().await?;

    Ok((reserves.reserve0, reserves.reserve1))
}
