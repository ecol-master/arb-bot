use anyhow::Result;
use futures_util::StreamExt;
use math::triangular_swap;

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    rpc::types::Filter,
    sol_types::SolEvent,
};
use arbbot_config::Config;
use arbbot_storage::Storage;
use ethereum_abi::IUniswapV2Pair;
use std::sync::Arc;

mod logger;
mod math;
mod mempool_searchers;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    logger::init_logger(tracing::Level::INFO);

    let config = Config::load("./config.json".into())?;

    let provider = Arc::new(ProviderBuilder::default().connect(&config.rpc_url).await?);
    let storage = Storage::new(config, provider.clone()).await?;

    if let Err(e) = run(storage, provider).await {
        tracing::error!("{e:?}");
    }

    return Ok(());
}

type P = Arc<RootProvider>;
async fn run(mut storage: Storage, provider: P) -> Result<()> {
    let filter = Filter::new().event_signature(IUniswapV2Pair::Sync::SIGNATURE_HASH);
    let mut stream = provider.subscribe_blocks().await?.into_stream();

    while let Some(header) = stream.next().await {
        let f = filter.clone().from_block(header.number);
        let logs = provider.get_logs(&f).await?;

        for log in logs {
            let sync = IUniswapV2Pair::Sync::decode_log(&log.inner, false)?;
            storage
                .update_reserves(&sync.address, sync.reserve0, sync.reserve1)
                .await?;
        }

        let reserves = storage.reserves();
        let paths = triangular_swap(reserves).await?;
        tracing::info!("arbitrage paths: {:?}", paths);
    }

    Ok(())
}
