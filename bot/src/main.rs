use anyhow::Result;

use alloy::providers::{Provider, ProviderBuilder};
use futures_util::StreamExt;
use kronos_config::Config;
use kronos_db::DB;
use kronos_dexes::uniswap_v2::UniswapV2;
use kronos_executor::Executor;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    kronos_logger::init_logger(tracing::Level::INFO);

    let config = Config::load("./config.yml".into())?;

    let database = DB::from_config(&config).await?;
    let provider = Arc::new(ProviderBuilder::default().connect(&config.rpc_url).await?);

    let (blocks_tx, blocks_rx) = tokio::sync::mpsc::unbounded_channel();
    let (arbitrage_tx, arbitrage_rx) = tokio::sync::mpsc::unbounded_channel();

    let uniswap_v2 =
        UniswapV2::new(database.clone(), provider.clone(), blocks_rx, arbitrage_tx).await?;

    let executor = Executor::new(database.clone(), provider.clone(), arbitrage_rx);

    // Create handle to start bot
    let uniswapv2_handle = tokio::spawn(async move { uniswap_v2.start().await.unwrap() });

    let executor_handle = tokio::spawn(async move { executor.start().await.unwrap() });

    let blocks_handle = tokio::spawn(async move {
        let mut stream = provider.subscribe_blocks().await.unwrap().into_stream();
        while let Some(block) = stream.next().await {
            tracing::info!("📦 block: {}", block.number);
            blocks_tx.send(block).unwrap();
        }
    });

    uniswapv2_handle.await?;
    executor_handle.await?;
    blocks_handle.await?;

    return Ok(());
}
