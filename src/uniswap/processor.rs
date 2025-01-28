use crate::{
    types::IERC20,
    uniswap::{
        router::{swapExactETHForTokensCall, swapExactTokensForTokensCall},
        types::IUniswapV2Pair,
    },
};
use alloy::{
    dyn_abi::abi::token,
    primitives::{Address, Uint, U256},
    providers::RootProvider,
    pubsub::PubSubFrontend,
    sol,
    sol_types::SolCall,
};
use std::sync::Arc;
use tracing::info;

pub struct SwapContext {
    pub token0_adr: Address,
    pub token1_adr: Address,
    pub pair:
        IUniswapV2Pair::IUniswapV2PairInstance<PubSubFrontend, Arc<RootProvider<PubSubFrontend>>>,
}

pub async fn process_swap_exact_tokens_for_tokens(
    call_data: swapExactTokensForTokensCall,
    ctx: SwapContext,
    provider: Arc<RootProvider<PubSubFrontend>>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("SwapExactTokensForTokens");
    let reserves = ctx.pair.getReserves().call().await.unwrap();

    let token0 = IERC20::new(ctx.token0_adr, provider.clone());
    let token1 = IERC20::new(ctx.token1_adr, provider.clone());

    let token0_symbol = token0.symbol().call().await?._0;
    let token1_symbol = token1.symbol().call().await?._0;
    info!("Pair: {:?}/{:?}", token0_symbol, token1_symbol);

    let dy = calculate_dy(
        call_data.amountOutMin,
        reserves.reserve0,
        reserves.reserve1,
        call_data.path[0] < call_data.path[1],
    );

    if call_data.path[0] < call_data.path[1] {
        info!("AmountIn: {:?} {:?}", call_data.amountIn, token0_symbol);
        info!("dy: {:?} {:?}", dy, token1_symbol);
    } else {
        info!("AmountIn: {:?} {:?}", call_data.amountIn, token1_symbol);
        info!("dy: {:?} {:?}", dy, token0_symbol);
    };
    info!("----------------------");
    Ok(())
}

pub async fn process_swap_exact_eth_for_tokens(
    call_data: swapExactETHForTokensCall,
    ctx: SwapContext,
    provider: Arc<RootProvider<PubSubFrontend>>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("SwapExactETHForTokens");
    let reserves = ctx.pair.getReserves().call().await.unwrap();

    let token0 = IERC20::new(ctx.token0_adr, provider.clone());
    let token1 = IERC20::new(ctx.token1_adr, provider.clone());

    let token0_symbol = token0.symbol().call().await?._0;
    let token1_symbol = token1.symbol().call().await?._0;
    info!("Pair: {:?}/{:?}", token0_symbol, token1_symbol);

    // get reserves for token0 and token1
    // change the order of reserves if token0 is not first token in pair

    // dy = reserve_1 * 0.997 * dx / (reserve_0 + 0.997 + dx)
    let dy = calculate_dy(
        call_data.amountOutMin,
        reserves.reserve0,
        reserves.reserve1,
        call_data.path[0] < call_data.path[1],
    );

    let token_in_symbol = if call_data.path[0] == ctx.token0_adr {
        token0_symbol
    } else {
        token1_symbol
    };
    info!("Token In: {:?}", token_in_symbol);
    info!("AmountOutMin: {:?}", call_data.amountOutMin);
    info!("dy: {:?}", dy);
    info!("----------------------");
    Ok(())
}

// dy = (y * dx * 0.997) / (x + 0.997 * dx)
// or
// dy = (y * dx * 997) / (1000 * x + 997 * dx)
fn calculate_dy(
    dx: Uint<256, 4>,
    reserve_x: Uint<112, 2>,
    reserve_y: Uint<112, 2>,
    order: bool,
) -> Uint<256, 4> {
    let (reserve_x, reserve_y) = if order {
        (reserve_x, reserve_y)
    } else {
        (reserve_y, reserve_x)
    };

    U256::from(reserve_y) * U256::from(dx) * U256::from(997)
        / (U256::from(reserve_x) * U256::from(1000) + U256::from(dx) * U256::from(997))
}
