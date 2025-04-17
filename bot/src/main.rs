use anyhow::Result;

use alloy::providers::{Provider, ProviderBuilder};
use bot_config::Config;
use bot_db::DB;
use dex_common::DEX;
use futures_util::StreamExt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    bot_logger::init_logger(tracing::Level::INFO);

    let config = Config::load("./config.json".into())?;

    let provider = Arc::new(ProviderBuilder::default().connect(&config.rpc_url).await?);
    let database = DB::new(config).await?;

    let uniswap_v2 = uniswap_v2::UniswapV2::new(database.clone(), provider.clone()).await?;

    let mut stream = provider.subscribe_blocks().await?.into_stream();

    while let Some(header) = stream.next().await {
        tracing::info!("⚡️ new block {}", header.number);
        uniswap_v2.on_block(header).await?;
    }

    return Ok(());
}
