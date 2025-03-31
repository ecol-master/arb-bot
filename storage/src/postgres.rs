use crate::tables::{PairV2, PAIR_V2_TABLE};
use alloy::primitives::Address;
use anyhow::Result;
use arbbot_config::PostgresConfig;
use tokio_postgres::NoTls;

pub struct PostgresDB {
    client: tokio_postgres::Client,
}

impl PostgresDB {
    pub async fn connect(config: &PostgresConfig) -> Result<Self> {
        let conn_data = config.into_connection(); 
        let (client, connection) = tokio_postgres::connect(&conn_data, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                panic!("Connect to Postgres: {e:?}");
            }
        });

        Ok(Self { client })
    }

    pub async fn select_pairs_v2(&self) -> Result<Vec<PairV2>> {
        let query = format!("SELECT * FROM {PAIR_V2_TABLE}");
        let rows = self.client.query(&query, &[]).await?;

        Ok(rows
            .iter()
            .map(|row| PairV2 {
                address: Address::from_slice(row.get::<&str, &[u8]>("address")),
                token0: Address::from_slice(row.get::<&str, &[u8]>("token0")),
                token1: Address::from_slice(row.get::<&str, &[u8]>("token1")),
            })
            .collect())
    }

    pub async fn insert_pair_v2(&self, pair: &PairV2) -> Result<()> {
        let query =
            format!("INSERT INTO {PAIR_V2_TABLE} (address, token0, token1) VALUES ($1, $2, $3)");
        let rows_modified = self
            .client
            .execute(
                &query,
                &[
                    &pair.address.as_slice(),
                    &pair.token0.as_slice(),
                    &pair.token1.as_slice(),
                ],
            )
            .await?;

        debug_assert!(rows_modified == 1, "PostgresDB don't insert PairV2");
        tracing::info!("💵 Inserted new pair: {:?}", pair.address);
        Ok(())
    }
}
