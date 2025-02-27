mod config;
mod mempool_searchers;
mod mempool_subscribers;
mod pool;
mod searcher;
mod storage;
mod types;

use crate::{
    config::Config,
    mempool_searchers::run_mempool_searches,
    mempool_subscribers::run_mempool_subscribers,
    pool::{subscribe_pool, SubcribePoolContext},
    storage::Storage,
    types::IUniswapV2Pair::Swap,
};

use tokio::task::JoinHandle;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use alloy::{
    primitives::{address, Address, FixedBytes},
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    transports::http::{Client, Http},
};
use crossbeam::channel::unbounded;
use dotenv::dotenv;
use std::sync::Arc;

type TxHash = FixedBytes<32>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    dotenv().expect("Failed to load .env variables");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        //.with_writer(rolling::daily("logs", "processed_tx.log"))
        //.with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let (tx_s, tx_r) = unbounded::<TxHash>();
    let (event_s, _) = unbounded::<Swap>();

    let rpc = Config::new()?.alchemy_rpc_url;
    let provider = Arc::new(ProviderBuilder::new().on_ws(WsConnect::new(rpc)).await?);

    let p = provider.clone();
    let subscriber_handle = tokio::spawn(async move {
        if let Err(e) = run_mempool_subscribers(tx_s, p).await {
            info!("Error during run_mempool_subscriber: {e:?}")
        }
    });

    let runners_handle = tokio::spawn(async move {
        if let Err(e) = run_mempool_searches(tx_r, event_s, provider).await {
            info!("Error during run_mempool_searchers: {e:?}");
        }
    });

    subscriber_handle.await;
    runners_handle.await;

    return Ok(());
}

const UNISWAP_V2_PAIR_ADDRESSES_PREV: [Address; 5] = [
    address!("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"), // USDC/ETH
    address!("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852"), // ETH/USDT
    address!("0x811beEd0119b4AfCE20D2583EB608C6F7AF1954f"), // SHIB/ETH
    address!("0x881d5c98866a08f90A6F60E3F94f0e461093D049"), // SHIB/USDC
    address!("0x3041CbD36888bECc7bbCBc0045E3B1f144466f5f"), // USDC/USDT
];

const UNISWAP_V2_PAIR_ADDRESSES: [Address; 1] = [
    address!("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"), // USDC/ETH
];

/// Run subscribers for UniswapV2Pair
async fn run_subscribers(
    storage: Arc<Storage>,
    provider: Arc<RootProvider<PubSubFrontend>>,
    anvil_provider: Arc<RootProvider<Http<Client>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for pair_adr in UNISWAP_V2_PAIR_ADDRESSES {
        let ctx = SubcribePoolContext {
            pair_adr,
            storage: storage.clone(),
            provider: provider.clone(),
            anvil_provider: anvil_provider.clone(),
        };

        let handle = tokio::spawn(async move {
            if let Err(e) = subscribe_pool(ctx).await {
                eprintln!("subscribe_pool failed for {:?}: {:?}", pair_adr, e);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
    Ok(())
}
