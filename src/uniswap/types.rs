use alloy::sol;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IUniswapV2Pair,
    "abi/IUniswapV2Pair.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IUniswaV2Factory,
    "abi/IUniswapV2Factory.json"
);
