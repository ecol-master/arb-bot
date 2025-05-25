use anyhow::Result;

use alloy::providers::{Provider, ProviderBuilder};
use bot_config::Config;
use bot_db::DB;
use bot_executor::ArbitrageExecutor;
use crossbeam::channel::unbounded;
use dex_common::{Arbitrage, DEX};
use futures_util::StreamExt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    bot_logger::init_logger(tracing::Level::INFO);
    tracing::info!("i am started");

    let config = Config::load("./config.yml".into())?;
    tracing::info!("config: {config:?}");

    let provider = Arc::new(ProviderBuilder::default().connect(&config.rpc_url).await?);
    let database = DB::new(config).await?;

    let (s, r) = unbounded::<Arbitrage>();
    let uniswap_v2 = uniswap_v2::UniswapV2::new(s, database.clone(), provider.clone()).await?;
    let executor = ArbitrageExecutor::new(r, database.clone(), provider.clone());

    let executor_handle = tokio::spawn(async move {
        if let Err(err) = executor.run().await {
            tracing::error!("Executor error: {err:?}")
        }
    });

    let block_handle = tokio::spawn(async move {
        let mut stream = provider
            .subscribe_blocks()
            .await
            .expect("Failed to create stream")
            .into_stream();

        while let Some(header) = stream.next().await {
            tracing::info!("⚡️ new block {}", header.number);
            if let Err(err) = uniswap_v2.on_block(header).await {
                tracing::error!("uniswap-v2 error: {err:?}");
            };
        }
    });

    executor_handle.await;
    block_handle.await;

    return Ok(());
}
