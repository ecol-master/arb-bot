use crate::{storage::StorageReserves, types::IUniswapV2Pair::IUniswapV2PairInstance};
use alloy::{
    primitives::{address, Address, Uint, U256},
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
};
use anyhow::anyhow;
use enum_iterator::Sequence;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    hash::Hash,
    sync::Arc,
};
use tracing::info;
use tracing_appender::rolling::Rotation;

// true -> can do erbitrage
pub async fn floyd_warshall_search(
    data: StorageReserves,
) -> Result<Option<Vec<Address>>, Box<dyn std::error::Error>> {
    info!("start floyd warshall");

    Ok(None)
}

pub async fn dfs_search(
    data: StorageReserves,
) -> Result<Option<Vec<Address>>, Box<dyn std::error::Error>> {
    info!("start dfs search");

    let tokens_max = data.len();

    // let start_token = address!("0xdAC17F958D2ee523a2206206994597C13D831ec7");

    let amount_in: Uint<112, 2> = Uint::from(100_000_000);

    for start_token in data.keys() {
        let mut current_path: Vec<Address> = vec![start_token.clone()];
        for sub_token in data.get(start_token).unwrap().keys() {
            let (reserve0, reserve1) = get_reserves(&data, start_token, sub_token);
            if dfs(
                &mut current_path,
                start_token,
                sub_token,
                calc_out(reserve0, reserve1, amount_in),
            ) {
                return Ok(Some(current_path));
            }
        }
    }

    return Ok(None);
}

// returns if cycle found
// cycle stores in current_path
fn dfs(
    current_path: &mut Vec<Address>,
    start_token: &Address,
    current_token: &Address,
    current_token_amount: Uint<112, 2>,
) -> bool {
    todo!()
}

fn get_reserves(
    data: &StorageReserves,
    token0: &Address,
    token1: &Address,
) -> (Uint<112, 2>, Uint<112, 2>) {
    let reserve0 = data.get(token0).unwrap().get(token1).unwrap();
    let reserve1 = data.get(token1).unwrap().get(token0).unwrap();

    (*reserve0, *reserve1)
}

fn calc_out(
    reserve0: Uint<112, 2>,
    reserve1: Uint<112, 2>,
    amount_in: Uint<112, 2>,
) -> Uint<112, 2> {
    let k = reserve0 * reserve1;

    let amount_in_effective = amount_in * Uint::from(997) / Uint::from(1000);
    let new_reserve0 = reserve0 + amount_in_effective;

    reserve1 - (k / new_reserve0)
}
