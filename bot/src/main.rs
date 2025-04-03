use anyhow::Result;

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

    let uniswap_v2 = uniswap_v2::UniswapV2::new(database.clone(), provider.clone()).await?;

    if let Err(e) = uniswap_v2.run().await {
        tracing::error!("{e:?}");
    }

    return Ok(());
}
