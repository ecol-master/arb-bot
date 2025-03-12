mod math;
mod mempool_searchers;

use anyhow::Result;
use futures_util::StreamExt;
use math::triangular_swap;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        //.with_writer(rolling::daily("logs", "processed_tx.log"))
        //.with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = Config::load("./config.json".into())?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .on_ws(WsConnect::new(&config.rpc_url))
            .await?,
    );
    let storage = Storage::new(config, provider.clone()).await?;

    if let Err(e) = run(storage, provider).await {
        tracing::error!("{e:?}");
    }

    return Ok(());
}

type P = Arc<RootProvider<PubSubFrontend>>;
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
