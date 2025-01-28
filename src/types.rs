use alloy::sol;

// Generate IERC20 contract from its abi
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    IERC20,
    "abi/IERC20.json",
);

// Generate IUniswapV3Pool contract from its abi
// See solidity/iuniswap_v3_pool/Readme.md
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IUniswapV3Pool,
    "abi/IUniswapV3Pool.json"
);