use crate::types::{IUniswaV2Factory, IUniswapV2Pair, IERC20};
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
use IUniswaV2Factory::IUniswaV2FactoryInstance;

sol!(
    #[allow(missing_docs)]
    function swapExactTokensForTokens(
        uint256 amountIn,
        uint256 amountOutMin,
        address[] calldata path,
        address to,
        uint256 deadline
      ) external returns (uint256[] memory amounts);
);

sol!(
    #[allow(missing_docs)]
    function swapExactETHForTokens(
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external payable returns (uint[] memory amounts);
);

sol!(
    #[allow(missing_docs)]
    function swapTokensForExactETH(
        uint amountOut,
        uint amountInMax,
        address[] calldata path,
        address to,
        uint deadline
    ) external returns (uint[] memory amounts);
);

pub async fn process_swap_exact_tokens_for_tokens(
    call_data: swapExactTokensForTokensCall,
    pair_adr: Address,
    provider: Arc<RootProvider<PubSubFrontend>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pair = IUniswapV2Pair::new(pair_adr, provider.clone());
    let reserves = pair.getReserves().call().await.unwrap();

    let token0_adr = pair.token0().call().await.unwrap()._0;
    let token0 = IERC20::new(token0_adr, provider.clone());

    let token1_adr = pair.token1().call().await.unwrap()._0;
    let token1 = IERC20::new(token1_adr, provider.clone());

    let token0_symbol = token0.symbol().call().await?._0;
    let token1_symbol = token1.symbol().call().await?._0;
    info!("Pair: {:?}/{:?}", token0_symbol, token1_symbol);

    // get reserves for token0 and token1
    // change the order of reserves if token0 is not first token in pair
    let (reserve_0, reserve_1) = if call_data.path[0] < call_data.path[1] {
        (reserves.reserve0, reserves.reserve1)
    } else {
        (reserves.reserve1, reserves.reserve0)
    };

    // dy = reserve_1 * 0.997 * dx / (reserve_0 + 0.997 + dx)
    let dy = U256::from(call_data.amountIn) * U256::from(reserve_1) * U256::from(997)
        / (U256::from(reserve_0) * U256::from(1000)
            + U256::from(call_data.amountIn) * U256::from(997));

    let token_in_symbol = if call_data.path[0] == token0_adr {
        token0_symbol
    } else {
        token1_symbol
    };
    info!("Token In: {:?}", token_in_symbol);
    info!("AmountIn: {:?}", call_data.amountIn);
    info!("dy: {:?}", dy);
    Ok(())
}
