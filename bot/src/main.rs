use anyhow::Result;
use futures_util::StreamExt;
use math::find_triangular_arbitrage;

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    rpc::types::Filter,
    sol_types::SolEvent,
};
use arbbot_config::Config;
use arbbot_storage::{tables::PairV2, MemDB, PairV2Data};
use ethereum_abi::IUniswapV2Pair;
use std::sync::Arc;
use tokio::sync::RwLock;

mod logger;
mod math;
mod mempool_searchers;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    logger::init_logger(tracing::Level::INFO);

    let config = Config::load("./config.json".into())?;

    let provider = Arc::new(ProviderBuilder::default().connect(&config.rpc_url).await?);
    let database = Arc::new(RwLock::new(MemDB::new(config).await?));

    if let Err(e) = run(database, provider).await {
        tracing::error!("{e:?}");
    }

    return Ok(());
}

type P = Arc<RootProvider>;
async fn run(mut database: Arc<RwLock<MemDB>>, provider: P) -> Result<()> {
    let filter = Filter::new().event_signature(IUniswapV2Pair::Sync::SIGNATURE_HASH);
    let mut stream = provider.subscribe_blocks().await?.into_stream();

    while let Some(header) = stream.next().await {
        tracing::info!("⚡ block {:?}", header.number);
        let f = filter.clone().from_block(header.number);

        let mut updated_tokens = Vec::new();

        for log in provider.get_logs(&f).await? {
            let sync = IUniswapV2Pair::Sync::decode_log(&log.inner, false)?;
            let pair_instance = IUniswapV2Pair::new(sync.address.clone(), provider.clone());

            let k = pair_instance.kLast().call().await?._0;
            let token0 = pair_instance.token0().call().await?._0;
            let token1 = pair_instance.token1().call().await?._0;

            if !database.read().await.pair_exists(&sync.address) {
                database
                    .write()
                    .await
                    .add_pair_v2(PairV2 {
                        address: sync.address.clone(),
                        token0: token0.clone(),
                        token1: token1.clone(),
                    })
                    .await?;
            }

            database
                .write()
                .await
                .update_reserves(&sync.address, sync.reserve0, sync.reserve1)?;

            updated_tokens.push(token0);
            updated_tokens.push(token1);
        }

        let paths =
            find_triangular_arbitrage(&updated_tokens, database.clone(), provider.clone()).await?;
        tracing::info!("arbitrage paths: {:?}", paths);
    }

    Ok(())
}
