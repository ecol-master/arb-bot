use alloy::{primitives::Address, providers::RootProvider};
use anyhow::Result;
use arbbot_storage::PairV2Data;
use ethereum_abi::IUniswapV2Pair;
use std::sync::Arc;

pub async fn get_pair_v2_data(
    pair_adr: &Address,
    provider: Arc<RootProvider>,
) -> Result<PairV2Data> {
    let pair_instance = IUniswapV2Pair::new(*pair_adr, provider);
    let reserves = pair_instance.getReserves().call().await?;
    let k = pair_instance.kLast().call().await?._0;

    Ok(PairV2Data {
        reserve0: reserves.reserve0,
        reserve1: reserves.reserve1,
        k,
    })
}
