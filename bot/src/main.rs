mod config;
mod mempool_searchers;
mod pool;
mod searcher;
mod storage;
mod types;

use crate::{
    config::Config,
    mempool_searchers::run_mempool_searches,
    pool::{subscribe_pool, SubcribePoolContext},
    storage::Storage,
    types::{ArbBot, IUniswapV2Pair, IUniswapV2Pair::Swap},
};

use tokio::{fs, task::JoinHandle};
use tracing::{info, Level};
use tracing_appender::rolling;
use tracing_subscriber::FmtSubscriber;

use alloy::{
    consensus::{transaction, Transaction},
    node_bindings::Anvil,
    primitives::{address, Address, FixedBytes, TxNumber, Uint, U256},
    providers::{ext::AnvilApi, Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    rpc::types::anvil,
    signers::k256::elliptic_curve::weierstrass::add,
    transports::http::{Client, Http},
};
use crossbeam::channel::{unbounded, Sender};
use dotenv::dotenv;
use futures_util::StreamExt;
use std::sync::Arc;
use types::IUniswaV2Factory;

type TxHash = FixedBytes<32>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    dotenv().expect("Failed to load .env variables");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(rolling::daily("logs", "processed_tx.log"))
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let (tx_s, tx_r) = unbounded::<TxHash>();
    let (event_s, _) = unbounded::<Swap>();

    let subscriber_handle = tokio::spawn(async move {
        if let Err(e) = run_mempool_subscriber(tx_s).await {
            info!("Error during run_mempool_subscriber: {e:?}")
        }
    });

    let runners_handle = tokio::spawn(async move {
        if let Err(e) = run_mempool_searches(tx_r, event_s).await {
            info!("Error during run_mempool_searchers: {e:?}");
        }
    });

    subscriber_handle.await;
    runners_handle.await;

    return Ok(());
}

async fn run_mempool_subscriber(s: Sender<TxHash>) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .on_ws(WsConnect::new(config.infura_rpc_url.clone()))
            .await?,
    );
    //info!("Create provider");

    let mut stream = provider
        .subscribe_pending_transactions()
        .await?
        .into_stream();

    info!("Start subscribing mempool on: {:?}", config.infura_rpc_url);
    while let Some(tx_hash) = stream.next().await {
        //info!("Send new tx_hash: {tx_hash:?}");
        s.send(tx_hash)?;
    }
    Ok(())
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

/*
async fn get_pairs_v2(
    provider: Arc<RootProvider<PubSubFrontend>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let factory_adr = address!("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");
    let factory = IUniswaV2Factory::new(factory_adr, provider);

    let pairs_amount = factory.allPairsLength().call().await?._0;
    info!("UniswapV2 total pairs amount: {pairs_amount}");

    let mut pair_addresses = Vec::new();

    let mut current_idx: Uint<256, 4> = Uint::ZERO;

    while current_idx < pairs_amount {
        match factory.allPairs(current_idx).call().await {
            Ok(result) => pair_addresses.push(result.pair),
            Err(e) => {
                info!("Got error: {e:?}");
                break;
            }
        }
        info!("get pair: {current_idx:?}");
        current_idx += Uint::from(1);
    }

    let content = serde_json::to_string(&pair_addresses)?;
    fs::write("uniswap_v2_pairs.json", content);
    Ok(())
}*/
