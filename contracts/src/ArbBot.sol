// SPDX-License-Identifier: SEE LICENSE IN LICENSE
pragma solidity >=0.5.0;

import {IUniswapV2Pair} from "../lib/uniswap_v2/IUniswapV2Pair.sol";

contract ArbBot {
    function swapOnPair(
        address pair_adr,
        uint amount0Out,
        uint amount1Out,
        address to,
        bytes calldata data
    ) external {
        IUniswapV2Pair(pair_adr).swap(amount0Out, amount1Out, to, data);
    }
}
