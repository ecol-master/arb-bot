use crate::tables::{Pair, PairRaw, PAIRS_TABLE};
use alloy::primitives::Address;
use anyhow::Result;
use bot_config::PostgresConfig;
use sqlx::{Executor, Pool, Postgres};
use tokio_postgres::NoTls;

#[derive(Clone)]
pub struct PostgresDB {
    pool: Pool<Postgres>,
}

impl PostgresDB {
    pub async fn connect(config: &PostgresConfig) -> Result<Self> {
        let conn_data = config.sqlx_connection();
        tracing::info!("postgres connection data: {conn_data:?}");
        let pool = sqlx::PgPool::connect(&conn_data).await?;

        Ok(Self { pool })
    }

    pub async fn select_pairs(&self) -> Result<Vec<Pair>> {
        let query = format!("SELECT * FROM {PAIRS_TABLE}");
        let pairs_v2: Vec<PairRaw> = sqlx::query_as(&query).fetch_all(&self.pool).await?;
        Ok(pairs_v2
            .iter()
            .map(|pair_raw| Pair {
                address: Address::from_slice(&pair_raw.address),
                token0: Address::from_slice(&pair_raw.token0),
                token1: Address::from_slice(&pair_raw.token1),
                dex_id: pair_raw.dex_id,
            })
            .collect())
    }

    pub async fn insert_pair(&self, pair: &Pair) -> Result<()> {
        let query = format!(
            "INSERT INTO {PAIRS_TABLE} (address, dex_id, token0, token1) VALUES ($1, $2, $3, $4)"
        );

        let rows_affected = sqlx::query(&query)
            .bind(&pair.address.as_slice())
            .bind(pair.dex_id)
            .bind(&pair.token0.as_slice())
            .bind(&pair.token1.as_slice())
            .execute(&self.pool)
            .await?
            .rows_affected();

        debug_assert!(rows_affected == 1, "PostgresDB don't insert PairV2");
        tracing::info!("💵 Inserted new pair: {:?}", pair.address);
        Ok(())
    }
}
