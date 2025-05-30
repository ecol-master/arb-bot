use alloy::primitives::{address, Address, Uint};
use anyhow::{anyhow, Result};
use kronos_db::{PricesStorage, TokensGraphStorage, DB};
use kronos_common::{Reserves};

pub mod cpmm;

const USDT: Address = address!("0xdAC17F958D2ee523a2206206994597C13D831ec7");
const USDC: Address = address!("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
const DAI: Address = address!("0x6B175474E89094C44Da98b954EedeAC495271d0F");

const STABLE_COINS: [Address; 3] = [DAI, USDC, USDT];

pub async fn price_to_usd(
    db: DB,
    dex_id: i32,
    token: &Address,
    amount: Uint<256, 4>,
) -> Result<f64> {
    let adjacent = db.adjacent_tokens(dex_id, token).await?;

    for stable in STABLE_COINS.iter() {
        if !adjacent.contains(stable) {
            continue;
        }
        let Reserves(r_token, r_stable) = db.reserves(dex_id, token, stable).await?;

        let stable_amount = amount * Uint::<256, 4>::from(r_stable) / Uint::<256, 4>::from(r_token);
        let usd: f64 = stable_amount.to_string().parse()?;
        if *stable == DAI {
            return Ok(usd / 1_000_000_000_000_000_000.0);
        }
        // for USDt and USDc the same digits = 6
        return Ok(usd / 1_000_000.0);
    }

    Err(anyhow!("Not found pool with stable for {token:?}"))
}
