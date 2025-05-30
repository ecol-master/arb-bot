use crate::tables::Pair;
use std::collections::HashSet;

use alloy::primitives::Address;
use anyhow::Result;
use kronos_common::Reserves;
use kronos_config::Config;

pub mod postgres;
pub mod redis;
pub mod tables;

pub use postgres::*;

pub struct UpdateReservesData {
    pub token0: Address,
    pub token1: Address,
    pub reserves: Reserves,
}

// Traits
#[async_trait::async_trait]
pub trait PricesStorage {
    async fn reserves(&self, dex_id: i32, token0: &Address, token1: &Address) -> Result<Reserves>;

    async fn update_reserves(&self, dex_id: i32, data: UpdateReservesData) -> Result<()>;
}

#[async_trait::async_trait]
pub trait TokensGraphStorage {
    async fn add_pair(&self, pair: Pair) -> Result<()>;

    async fn adjacent_tokens(&self, dex_id: i32, token: &Address) -> Result<HashSet<Address>>;

    async fn pair_by_tokens(&self, dex_id: i32, pair_adr: &Address) -> Result<(Address, Address)>;

    async fn pair_adr(&self, dex_id: i32, token0: &Address, token1: &Address) -> Result<Address>;
}

/// `DB`
#[derive(Clone)]
pub struct DB {
    redis: redis::RedisDB,
    postgres: postgres::PostgresDB,
}

impl DB {
    pub async fn from_config(config: &Config) -> Result<DB> {
        let postgres = postgres::PostgresDB::connect(&config.postgres).await?;
        let redis = redis::RedisDB::connect(&config.redis).await?;

        // pre initialization
        let pairs = postgres.select_pairs().await?;
        tracing::info!("📦 Load {} pairs from Postgres", pairs.len());

        for pair in pairs {
            redis.add_pair(pair).await?;
        }

        Ok(Self { redis, postgres })
    }

    pub fn postgres(&self) -> PostgresDB {
        self.postgres.clone()
    }
}

// Impl DB traits
#[async_trait::async_trait]
impl PricesStorage for DB {
    async fn reserves(&self, dex_id: i32, token0: &Address, token1: &Address) -> Result<Reserves> {
        self.redis.reserves(dex_id, token0, token1).await
    }

    async fn update_reserves(&self, dex_id: i32, data: UpdateReservesData) -> Result<()> {
        self.redis
            .update_reserves(
                dex_id,
                &data.token0,
                &data.token1,
                data.reserves.0,
                data.reserves.1,
            )
            .await
    }
}

#[async_trait::async_trait]
impl TokensGraphStorage for DB {
    async fn add_pair(&self, pair: Pair) -> Result<()> {
        self.redis.add_pair(pair.clone()).await?;
        self.postgres.insert_pair(pair).await
    }

    async fn adjacent_tokens(&self, dex_id: i32, token: &Address) -> Result<HashSet<Address>> {
        self.redis.adjacent(dex_id, token).await
    }

    async fn pair_by_tokens(&self, dex_id: i32, pair_adr: &Address) -> Result<(Address, Address)> {
        self.redis.pair_by_tokens(dex_id, pair_adr).await
    }

    async fn pair_adr(&self, dex_id: i32, token0: &Address, token1: &Address) -> Result<Address> {
        self.redis.pair_adr(dex_id, token0, token1).await
    }
}
