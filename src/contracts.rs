use alloy::{sol, sol_types::SolCall};

// Generate IERC20 contract from its abi
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
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
