mod config;
mod mempool_searchers;
mod mempool_subscribers;
mod searcher;

use crate::{config::Config, mempool_subscribers::subscribe_sync_in_block, searcher::listen_swaps};
use tokio::task::JoinHandle;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use alloy::{
    primitives::{address, Address, FixedBytes, Log},
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    transports::http::{Client, Http},
};
use crossbeam::channel::unbounded;
use dotenv::dotenv;
use std::sync::Arc;
use storage::Storage;

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

    let (log_s, log_r) = unbounded::<Log>();

    let rpc = Config::new()?.infura_rpc_url;
    let provider = Arc::new(ProviderBuilder::new().on_ws(WsConnect::new(rpc)).await?);
    let storage = Storage::new(provider.clone()).await?;

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
