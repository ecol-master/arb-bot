use anyhow::Result;
use futures_util::StreamExt;
use math::find_triangular_arbitrage;

use alloy::{
    dyn_abi::abi::token,
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    rpc::types::Filter,
    sol_types::SolEvent,
};
use bot_config::Config;
use bot_db::{
    tables::{Pair, DEX},
    DB,
};
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
    let database = DB::new(config).await?;
    //let database = Arc::new(RwLock::new(MemDB::new(config).await?));

    if let Err(e) = run(database, provider).await {
        tracing::error!("{e:?}");
    }

    return Ok(());
}

const DEX_ID: i32 = 1;

type P = Arc<RootProvider>;
async fn run(database: DB, provider: P) -> Result<()> {
    let filter = Filter::new().event_signature(IUniswapV2Pair::Sync::SIGNATURE_HASH);
    let mut stream = provider.subscribe_blocks().await?.into_stream();

    while let Some(header) = stream.next().await {
        tracing::info!("⚡ block {:?}", header.number);
        let f = filter.clone().from_block(header.number);

        let mut updated_tokens = Vec::new();

        for log in provider.get_logs(&f).await? {
            let sync = IUniswapV2Pair::Sync::decode_log(&log.inner, false)?;

            tracing::info!("update pair: {:?}", sync.address);
            let (token0, token1) = match database.pair_tokens(DEX_ID, &sync.address).await {
                Ok(r) => (r.0, r.1),
                Err(_) => {
                    // TODO: add new pair to postgres
                    let instance = IUniswapV2Pair::new(sync.address.clone(), provider.clone());
                    let token0 = instance.token0().call().await?._0;
                    let token1 = instance.token1().call().await?._0;

                    let pair = Pair {
                        address: sync.address.clone(),
                        dex_id: DEX_ID,
                        token0: token0.clone(),
                        token1: token1.clone(),
                    };
                    database.add_pair(pair).await?;
                    (token0, token1)
                }
            };

            database
                .update_reserves(DEX_ID, &token0, &token1, sync.reserve0, sync.reserve1)
                .await?;

            updated_tokens.push(token0);
            updated_tokens.push(token1);
        }

        let paths =
            find_triangular_arbitrage(&updated_tokens, database.clone(), provider.clone()).await?;
        tracing::info!("arbitrage paths: {:?}", paths);
    }

    Ok(())
}
