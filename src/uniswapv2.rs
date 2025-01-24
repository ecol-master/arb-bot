use alloy::primitives::{Address, U256};
use std::{collections::HashMap, hash::Hash};

pub struct ERC20 {
    address: Address,
    supply: U256,
}

pub struct UniswapV2Pair {
    token0: ERC20,
    token1: ERC20,
}


