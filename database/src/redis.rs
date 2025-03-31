use std::ops::Add;

use crate::tables::{Pair, PairRaw};
use alloy::{
    dyn_abi::abi::token,
    primitives::{Address, Uint},
};
use anyhow::{anyhow, Result};
use bb8_redis::RedisConnectionManager;
use bot_config::RedisConfig;
use redis::AsyncCommands;
use std::collections::HashSet;

const BYTES: usize = Uint::<112, 2>::BYTES;

#[derive(Clone, Debug)]
pub struct RedisDB {
    pool: bb8::Pool<RedisConnectionManager>,
}

impl RedisDB {
    pub async fn connect(config: &RedisConfig) -> Result<Self> {
        let manager = RedisConnectionManager::new(config.into_connection())?;
        let pool = bb8::Pool::builder().build(manager).await?;
        Ok(Self { pool })
    }
}

/// Implementation for key functions
impl RedisDB {
    /// key: "adjacent:{dex_id}:{token}"
    pub fn key_adjacent_tokens(dex_id: i32, token: &Address) -> String {
        format!("adjacent:{dex_id}:{token:?}")
    }

    /// key : "reserves:{dex_id}:{token0}:{token1}"
    /// Returns reserves for token0 in pair with token1
    pub fn key_token_reserves(dex_id: i32, token0: &Address, token1: &Address) -> String {
        format!("reserves:{dex_id}:{token0:?}:{token1:?}")
    }

    /// key: "tokens:{dex_id}:{pair_address}"
    pub fn key_tokens(dex_id: i32, pair_address: &Address) -> String {
        format!("tokens:{dex_id}:{pair_address}")
    }

    /// key: "pair:{dex_id}:{token0}:{token1}"
    pub fn key_pair(dex_id: i32, token0: &Address, token1: &Address) -> String {
        // Change to correct order
        if *token0 < *token1 {
            format!("pair:{dex_id}:{token0}:{token1}")
        } else {
            format!("pair:{dex_id}:{token1}:{token0}")
        }
    }
}

impl RedisDB {
    // Adding to redis three things:
    // 1. mapping from pair to its tokens
    // 2. mapping from tokens to pair address
    // 3. setting adjacent tokens
    pub async fn add_pair(&self, pair: Pair) -> Result<()> {
        let mut conn = self.pool.get().await?;

        // mapping from `pair`: `token0+token1`
        let key_tokens = Self::key_tokens(pair.dex_id, &pair.address);
        let mut addresses = [0u8; 40];
        addresses[0..20].copy_from_slice(&pair.token0.as_slice());
        addresses[20..40].copy_from_slice(&pair.token1.as_slice());
        let _: () = conn.set(key_tokens, &addresses).await?;

        // mapping from `tokens` to `pair` address
        let key_pair = Self::key_pair(pair.dex_id, &pair.token0, &pair.token1);
        let _: () = conn.set(key_pair, &pair.address.as_slice()).await?;

        // addind adjacent tokens
        let key_token0_adjacent = Self::key_adjacent_tokens(pair.dex_id, &pair.token0);
        let key_token1_adjacent = Self::key_adjacent_tokens(pair.dex_id, &pair.token1);
        let _: () = conn
            .sadd(key_token0_adjacent, &pair.token1.as_slice())
            .await?;
        let _: () = conn
            .sadd(key_token1_adjacent, &pair.token0.as_slice())
            .await?;
        Ok(())
    }

    /// Returns (token0, token1) addresses for concrete pair
    pub async fn pair_tokens(&self, dex_id: i32, pair_adr: &Address) -> Result<(Address, Address)> {
        let mut conn = self.pool.get().await?;
        let key = Self::key_tokens(dex_id, pair_adr);

        match conn.get::<String, [u8; 40]>(key).await {
            Ok(addresses) => {
                return Ok((
                    Address::from_slice(&addresses[0..20]),
                    Address::from_slice(&addresses[20..40]),
                ))
            }
            Err(_) => return Err(anyhow!("think about fetching data")),
        }

        let addresses: [u8; 40] = conn.get(key).await?;

        Ok((
            Address::from_slice(&addresses[0..20]),
            Address::from_slice(&addresses[20..40]),
        ))
    }

    pub async fn pair_adr(
        &self,
        dex_id: i32,
        token0: &Address,
        token1: &Address,
    ) -> Result<Address> {
        let mut conn = self.pool.get().await?;
        let key = Self::key_pair(dex_id, token0, token1);

        let bytes: [u8; 20] = conn.get(key).await?;
        Ok(Address::from_slice(&bytes))
    }

    pub async fn adjacent(&self, dex_id: i32, token: &Address) -> Result<HashSet<Address>> {
        let mut conn = self.pool.get().await?;
        let key = Self::key_adjacent_tokens(dex_id, token);

        let adjacent: HashSet<[u8; 20]> = conn.smembers(key).await?;
        Ok(adjacent
            .iter()
            .map(|bytes| Address::from_slice(bytes))
            .collect())
    }

    /// NOTE: reserves convert to big-endian bytes
    pub async fn update_reserves(
        &self,
        dex_id: i32,
        token0: &Address,
        token1: &Address,
        reserve0: Uint<112, 2>,
        reserve1: Uint<112, 2>,
    ) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let key_token0 = Self::key_token_reserves(dex_id, token0, token1);
        let key_token1 = Self::key_token_reserves(dex_id, token1, token0);

        let reserve0_be: [u8; BYTES] = reserve0.to_be_bytes();
        let reserve1_be: [u8; BYTES] = reserve1.to_be_bytes();

        let _: () = conn.set(key_token0, &reserve0_be).await?;
        let _: () = conn.set(key_token1, &reserve1_be).await?;
        Ok(())
    }

    pub async fn reserves(
        &self,
        dex_id: i32,
        token0: &Address,
        token1: &Address,
    ) -> Result<(Uint<112, 2>, Uint<112, 2>)> {
        let mut conn = self.pool.get().await?;
        let key_token0 = Self::key_token_reserves(dex_id, token0, token1);
        let key_token1 = Self::key_token_reserves(dex_id, token1, token0);

        let reserve0_bytes: [u8; BYTES] = conn.get(key_token0).await?;
        let reserve1_bytes: [u8; BYTES] = conn.get(key_token1).await?;

        Ok((
            Uint::<112, 2>::from_be_bytes(reserve0_bytes),
            Uint::<112, 2>::from_be_bytes(reserve1_bytes),
        ))
    }
}
