use anyhow::Result;

use alloy::primitives::address;
use alloy::providers::{Provider, ProviderBuilder, RootProvider, WsConnect};
use bot_config::Config;
use bot_db::DB;
use dex_common::DEX;
use std::sync::Arc;

mod mempool_searchers;

#[tokio::main]
async fn main() -> Result<()> {
    bot_logger::init_logger(tracing::Level::INFO);

    let config = Config::load("./config.json".into())?;

    let provider = Arc::new(ProviderBuilder::default().connect(&config.rpc_url).await?);
    let database = DB::new(config).await?;

    let usdc = address!("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    let instance = ethereum_abi::IUniswapV2Pair::new(usdc, provider.clone());
    tracing::info!("usdc decimals: {}", instance.decimals().call().await?._0);

    let uniswap_v2 = uniswap_v2::UniswapV2::new(database.clone(), provider.clone()).await?;

    if let Err(e) = uniswap_v2.run().await {
        tracing::error!("{e:?}");
    }

    return Ok(());
}
