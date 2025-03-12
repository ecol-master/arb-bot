mod mempool_searchers;
mod mempool_subscribers;
mod searcher;

use crate::{mempool_subscribers::subscribe_sync_in_block, searcher::listen_swaps};
use tokio::task::JoinHandle;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use alloy::{
    primitives::{address, Address, FixedBytes, Log},
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    transports::http::{Client, Http},
};
use arbbot_config::Config;
use arbbot_storage::Storage;
use crossbeam::channel::unbounded;
use dotenv::dotenv;
use std::sync::Arc;

type TxHash = FixedBytes<32>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        //.with_writer(rolling::daily("logs", "processed_tx.log"))
        //.with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = Config::load("./config.json".into())?;

    let (log_s, log_r) = unbounded::<Log>();

    let provider = Arc::new(
        ProviderBuilder::new()
            .on_ws(WsConnect::new(&config.rpc_url))
            .await?,
    );
    let storage = Storage::new(config, provider.clone()).await?;

    let listener_handle = tokio::spawn(async move {
        if let Err(e) = listen_swaps(log_r, storage).await {
            tracing::error!("Error during listen_swaps: {e:?}");
        }
    });

    let subscriber_handle = tokio::spawn(async move {
        if let Err(e) = subscribe_sync_in_block(log_s, provider).await {
            tracing::error!("Error during subscribe_swaps_in_block: {e:?}");
        }
    });

    listener_handle.await?;
    subscriber_handle.await?;

    return Ok(());
}

const UNISWAP_V2_PAIR_ADDRESSES: [Address; 1] = [
    address!("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"), // USDC/ETH
];
