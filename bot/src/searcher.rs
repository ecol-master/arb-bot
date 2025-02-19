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
    ops::Add,
    sync::Arc,
};
use tracing::info;
use tracing_appender::rolling::Rotation;

pub async fn triangular_swap(
    reserves: StorageReserves,
) -> Result<Vec<(Address, Address, Address)>, Box<dyn std::error::Error>> {
    info!("start triangular swap find");
    let mut paths = Vec::new();

    for token_0 in reserves.keys() {
        let pair_variants = reserves.get(token_0).unwrap();
        for token_1 in pair_variants.keys() {
            for token_2 in reserves.get(token_1).unwrap().keys() {
                if pair_variants.contains_key(token_2) {
                    let is_loop =
                        check_tokens_for_triangular_swap(&reserves, token_0, token_1, token_2);
                    if is_loop {
                        paths.push((*token_0, *token_1, *token_2));
                        info!("");
                    }
                }
            }
        }
    }
    Ok(paths)
}

fn check_tokens_for_triangular_swap(
    reserves: &StorageReserves,
    token0: &Address,
    token1: &Address,
    token2: &Address,
) -> bool {
    let (reserve_0, reserve_1) = get_reserves(reserves, token0, token1);
    let p_ij = calc_token0_price_log(reserve_0, reserve_1);

    let (reserve_1, reserve_2) = get_reserves(reserves, token1, token2);
    let p_jk = calc_token0_price_log(reserve_1, reserve_2);

    let (reserve_2, reserve_0) = get_reserves(reserves, token2, token0);
    let p_ki = calc_token0_price_log(reserve_2, reserve_0);
    //info!("p_ij = {p_ij:?}, p_jk = {p_jk:?}, p_ki = {p_ki:?}");

    p_ij + p_jk + p_ki > 0
}

pub fn get_reserves(
    data: &StorageReserves,
    token0: &Address,
    token1: &Address,
) -> (Uint<112, 2>, Uint<112, 2>) {
    let reserve0 = data.get(token0).unwrap().get(token1).unwrap();
    let reserve1 = data.get(token1).unwrap().get(token0).unwrap();

    (*reserve0, *reserve1)
}

fn calc_token0_price_log(reserve0: Uint<112, 2>, reserve1: Uint<112, 2>) -> i128 {
    (reserve1.saturating_mul(Uint::from(997)).log2() as i128)
        - (reserve0.saturating_mul(Uint::from(1000)).log2() as i128)
}

pub fn calc_out(
    reserve_x: Uint<112, 2>,
    reserve_y: Uint<112, 2>,
    amount_in_x: Uint<256, 4>,
) -> Uint<256, 4> {
    //info!("amount_in_x: {amount_in_x:?}, reserve_x: {reserve_x:?}, reserve_y: {reserve_y:?}");
    let reserve_x: Uint<256, 4> = Uint::from(reserve_x);
    let reserve_y: Uint<256, 4> = Uint::from(reserve_y);
    // info!("amount_in_x: {amount_in_x:?}, reserve_x: {reserve_x:?}, reserve_y: {reserve_y:?}");

    let (k, is_overflow) = reserve_x.overflowing_mul(reserve_y);
    info!("calc k = rx * ry is_overflow: {is_overflow:?}");

    let (k, is_overflow) = k.overflowing_mul(Uint::from(1000));
    info!("calc k = 1000 * k is_overflow: {is_overflow:?}");

    let amount_in_effective = amount_in_x * Uint::from(997);
    // info!("amount_in_effective: {:?}", amount_in_effective);
    let new_reserve0 = reserve_x.saturating_mul(Uint::from(1000)) + amount_in_effective;

    reserve_y - (k / new_reserve0)
}
