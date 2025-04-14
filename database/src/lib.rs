use std::ops::Deref;

use anyhow::Result;
use bot_config::Config;

pub mod postgres;
pub mod redis;
pub mod tables;

pub use postgres::*;

#[derive(Clone)]
pub struct DB {
    redis: redis::RedisDB,
    postgres: postgres::PostgresDB,
}

impl DB {
    pub async fn new(config: Config) -> Result<DB> {
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

    pub fn postgres(&self) -> &PostgresDB {
        &self.postgres
    }
}

impl Deref for DB {
    type Target = redis::RedisDB;
    fn deref(&self) -> &Self::Target {
        &self.redis
    }
}
