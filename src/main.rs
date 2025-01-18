mod config;
mod contracts;

use crate::{
    config::Config,
    contracts::{IUniswapV3Pool, IERC20},
};

use envconfig::Envconfig;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use alloy::{
    dyn_abi::abi::token,
    primitives::{address, U256},
    providers::{Provider, ProviderBuilder, WsConnect},
    sol,
};
use futures_util::{stream, StreamExt};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = Config::init_from_env()?;
    info!("Started listen rpc: {}", config.rpc_url);

    let provider = ProviderBuilder::new()
        .on_ws(WsConnect::new(config.rpc_url))
        .await?;
    let provider = Arc::new(provider);

    let eth_usdc_adr = address!("0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");
    let eth_usdc = IUniswapV3Pool::new(eth_usdc_adr, provider.clone());

    // ETH/USDC Pool returns as token0 - USDC address, and token1 - ETH
    let usdc_adr = eth_usdc.token0().call().await?._0;
    let usdc = IERC20::new(usdc_adr, provider.clone());
    let mut usdc_supply = usdc.totalSupply().call().await?._0;

    let eth_adr = eth_usdc.token1().call().await?._0;
    let eth = IERC20::new(eth_adr, provider.clone());
    let mut eth_supply = eth.totalSupply().call().await?._0;

    let subscription = provider.subscribe_blocks().await?;
    let mut stream = subscription.into_stream();

    while let Some(block) = stream.next().await {
        info!("NEW BLOCK: {}", block.number);

        let usdc_current_supply = usdc.totalSupply().call().await?._0;
        let usdc_diff = usdc_current_supply.abs_diff(usdc_supply);
        info!("USDC supply diff: {}", usdc_diff);

        let eth_current_supply = eth.totalSupply().call().await?._0;
        let eth_diff = eth_current_supply.abs_diff(eth_supply);
        info!("ETH supply diff: {}", eth_diff);

        let precision: u32 = 6;
        let usdc_price_diff = (usdc_diff * U256::from(10u32.pow(precision + 2))) / usdc_supply;
        let eth_price_diff = (eth_diff * U256::from(10u32.pow(precision + 2))) / eth_supply;
        info!("USDC price diff(%): {} / 10^{}", usdc_price_diff, precision);
        info!("ETH price diff(%): {} / 10^{}", eth_price_diff, precision);

        usdc_supply = usdc_current_supply;
        eth_supply = eth_current_supply;
    }

    Ok(())
}
