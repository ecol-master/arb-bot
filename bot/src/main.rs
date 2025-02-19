mod config;
mod pool;
mod searcher;
mod storage;
mod types;

use crate::{
    config::Config,
    pool::{subscribe_pool, SubcribePoolContext},
    storage::Storage,
    types::IUniswapV2Pair,
};

use envconfig::Envconfig;
use tokio::{fs, task::JoinHandle};
use tracing::{info, Level};
use tracing_appender::rolling;
use tracing_subscriber::FmtSubscriber;

use alloy::{
    consensus::{transaction, Transaction},
    node_bindings::Anvil,
    primitives::{address, Address, TxNumber, Uint, U256},
    providers::{ext::AnvilApi, Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    rpc::types::anvil,
    signers::k256::elliptic_curve::weierstrass::add,
    transports::http::{Client, Http},
};
use std::sync::Arc;
use types::IUniswaV2Factory;

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
            .on_ws(WsConnect::new(config.rpc_url.clone()))
            .await?,
    );

    let anvil = Anvil::new().fork(config.rpc_url).try_spawn()?;
    let anvil_provider = Arc::new(ProviderBuilder::new().on_http(anvil.endpoint_url()));

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

    run_subscribers(
        Arc::new(Storage::new()),
        provider.clone(),
        anvil_provider.clone(),
    )
    .await?;

    return Ok(());
}

const UNISWAP_V2_PAIR_ADDRESSES: [Address; 5] = [
    address!("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"), // USDC/ETH
    address!("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852"), // ETH/USDT
    address!("0x811beEd0119b4AfCE20D2583EB608C6F7AF1954f"), // SHIB/ETH
    address!("0x881d5c98866a08f90A6F60E3F94f0e461093D049"), // SHIB/USDC
    address!("0x3041CbD36888bECc7bbCBc0045E3B1f144466f5f"), // USDC/USDT
];

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
