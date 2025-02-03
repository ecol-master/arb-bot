mod config;
mod pool;
mod searcher;
mod storage;
mod types;

use crate::{
    config::Config, pool::subscribe_pool, searcher::floyd_warshall_search, storage::Storage,
    types::IUniswapV2Pair,
};

use envconfig::Envconfig;
use tokio::task::JoinHandle;
use tracing::{info, Level};
use tracing_appender::rolling;
use tracing_subscriber::FmtSubscriber;

use alloy::{
    consensus::{transaction, Transaction},
    primitives::{address, Address, TxNumber, U256},
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    signers::k256::elliptic_curve::weierstrass::add,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        //.with_writer(rolling::daily("logs", "router_02.log"))
        //.with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = Config::init_from_env()?;
    info!("Started listen rpc: {}", config.rpc_url);

    let provider = Arc::new(
        ProviderBuilder::new()
            .on_ws(WsConnect::new(config.rpc_url))
            .await?,
    );

    let uniswap_v2_pairs = vec![
        IUniswapV2Pair::new(
            address!("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"),
            provider.clone(),
        ), // USDC/ETH
        IUniswapV2Pair::new(
            address!("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852"),
            provider.clone(),
        ), // ETH/USDT
        IUniswapV2Pair::new(
            address!("0x811beEd0119b4AfCE20D2583EB608C6F7AF1954f"),
            provider.clone(),
        ), // SHIB/ETH
        IUniswapV2Pair::new(
            address!("0x881d5c98866a08f90A6F60E3F94f0e461093D049"),
            provider.clone(),
        ), // SHIB/USDC
        IUniswapV2Pair::new(
            address!("0x3041CbD36888bECc7bbCBc0045E3B1f144466f5f"),
            provider.clone(),
        ), // USDC/USDT
    ];

    let storage = Arc::new(Storage::new(&uniswap_v2_pairs).await?);

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for pair in uniswap_v2_pairs {
        let provider = provider.clone();
        let storage = storage.clone();
        let handle = tokio::spawn(async move {
            let pair_adr = pair.address().clone();
            if let Err(e) = subscribe_pool(pair, storage, provider).await {
                eprintln!("subscribe_pool failed for {:?}: {:?}", pair_adr, e);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
    //let router03_adr = address!("0xE592427A0AEce92De3Edee1F18E0157C05861564");

    return Ok(());
}
