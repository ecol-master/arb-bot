use alloy::sol;

// Generate IERC20 contract from its abi
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    IERC20,
    "../../abi/IERC20.json",
);

// V2
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IUniswapV2Pair,
    "../../abi/IUniswapV2Pair.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IUniswapV2Factory,
    "../../abi/IUniswapV2Factory.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ArbBot,
    "../../out/ArbBot.sol/ArbBot.json"
);

// V3
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IUniswapV3Pool,
    "../../abi/IUniswapV3Pool.json"
);


// Router02 Swap Functions
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
