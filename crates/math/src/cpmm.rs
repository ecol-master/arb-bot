use alloy::primitives::{Address, Uint};
use anyhow::Result;
// use dex_common::{DexError, Reserves, DEX};
use kronos_common::{DexError, Reserves};
use kronos_db::{PricesStorage, TokensGraphStorage, DB};

#[derive(Clone, Debug)]
pub struct ArbitrageData {
    pub reserves: Reserves,
    pub fee: Uint<112, 2>,
}

pub async fn find_triangular_arbitrage(
    start_tokens: &[Address],
    db: DB,
    dex_id: i32,
) -> Result<Vec<Vec<(Address, Address)>>> {
    let mut paths = vec![];

    for token0 in start_tokens {
        let adjacent_for_token0 = db.adjacent_tokens(dex_id, token0).await?;

        for token1 in adjacent_for_token0.iter() {
            'iter: for token2 in db.adjacent_tokens(dex_id, token1).await?.iter() {
                if adjacent_for_token0.contains(token2) {
                    if token0 == token1 || token0 == token2 || token1 == token2 {
                        continue;
                    }

                    let mut reserves = Vec::<Reserves>::new();

                    let tokens = vec![(*token0, *token1), (*token1, *token2), (*token2, *token0)];
                    for (t0, t1) in tokens.iter() {
                        let token_reserves = match db.reserves(dex_id, t0, t1).await {
                            Ok(reserves) => reserves,
                            Err(err) => {
                                if let Some(DexError::BlockRpcLimitExceed) =
                                    err.downcast_ref::<DexError>()
                                {
                                    continue 'iter;
                                }
                                return Err(err);
                            }
                        };
                        reserves.push(token_reserves);
                    }

                    let fee = Uint::from(3);
                    if arbitrage_exists(fee, &reserves) {
                        paths.push(tokens);
                    }
                }
            }
        }
    }

    Ok(paths)
}

pub fn arbitrage_exists(fee: Uint<112, 2>, reserves: &[Reserves]) -> bool {
    let mut log_sum = 0f64;

    for r in reserves.iter() {
        log_sum += price_log(fee, r);
    }

    log_sum > 0.0
}

// p = (1 - fee) * r_i/r_j - price of `j` in terms of `i`
pub fn price_log(fee: Uint<112, 2>, reserves: &Reserves) -> f64 {
    let base = Uint::from(1000);
    reserves.0.saturating_mul(base - fee).approx_log2()
        - reserves.1.saturating_mul(base).approx_log2()
}

// dy = y - k / (x + 0.997 * dx)
// dy = y - 1000* k / (1000x + 997*dx)
pub fn calculate_dy(data: &ArbitrageData, amount_in: Uint<256, 4>) -> Uint<256, 4> {
    let base = Uint::from(1000);

    let new_reserve0 = Uint::<256, 4>::from(data.reserves.0).saturating_mul(base)
        + amount_in * (base - Uint::<256, 4>::from(data.fee));

    let k_last = Uint::<256, 4>::from(data.reserves.0) * Uint::<256, 4>::from(data.reserves.1);

    Uint::<256, 4>::from(data.reserves.1) - (k_last * Uint::from(base) / new_reserve0)
}

// Profit: (optimal_amount_in, max_profit)
type Profit = (Uint<256, 4>, Uint<256, 4>);

pub fn find_profit(data: &[ArbitrageData]) -> Option<Profit> {
    let mut amount_in = Uint::<256, 4>::from(1);

    let mut best_amount_in = None;
    let mut best_profit = None;

    for _ in 0..100 {
        let mut amount_out = amount_in;

        for d in data {
            amount_out = calculate_dy(d, amount_in);
        }

        if amount_out > amount_in {
            let profit = amount_out - amount_in;
            if best_profit.is_none() || profit > best_profit.unwrap() {
                best_amount_in = Some(amount_in);
                best_profit = Some(profit);
            }
        }

        amount_in *= Uint::from(2);
    }

    best_amount_in.zip(best_profit)
}

fn optimal_amount_in_bin_search(
    _pair_reserves: &[(Uint<112, 2>, Uint<112, 2>)],
) -> Option<Uint<256, 4>> {
    let mut _amount_in = Uint::<256, 4>::from(1);
    None
}

// mod tests {

//     #[tokio::test]
//     async fn test_price_log_correctness() -> Result<()> {
//         kronos_logger::init_logger(tracing::Level::INFO);

//         let config = Config::load("../config.json".into())?;
//         let provider = Arc::new(
//             ProviderBuilder::new()
//                 .on_ws(WsConnect::new(config.rpc_url))
//                 .await?,
//         );

//         let usdc_eth = address!("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc");
//         let pair = IUniswapV2Pair::new(usdc_eth, provider.clone());
//         let reserves = pair.getReserves().call().await?;

//         let (reserve0, reserve1) = (reserves.reserve0, reserves.reserve1);
//         let approx_log = reserve0.approx_log2();
//         tracing::info!("Uint<112, 2> reserve0.approx_log2() = {}", approx_log);

//         let reserve_f64: f64 = reserve0.to_string().parse()?;
//         let reserve_f64_log = reserve_f64.log2();
//         tracing::info!("f64: reserve_f64.log2() = {}", reserve_f64_log);

//         ///////////////////////////////////////////////////////////////
//         tracing::info!("////////////////////////////////////////////////////");
//         ///////////////////////////////////////////////////////////////
//         let approx_log = reserve1.approx_log2();
//         tracing::info!("Uint<112, 2> reserve1.approx_log2() = {}", approx_log);

//         let reserve_f64: f64 = reserve1.to_string().parse()?;
//         let reserve_f64_log = reserve_f64.log2();
//         tracing::info!("f64: reserve1_f64.log2() = {}", reserve_f64_log);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_choose_amount_in() -> Result<()> {
//         kronos_logger::init_logger(Level::INFO);
//         // let token0 = address!("0xA2b4C0Af19cC16a6CfAcCe81F192B024d625817D");
//         // let token1 = address!("0x514cdb9cd8A2fb2BdCf7A3b8DDd098CaF466E548");
//         // let token2 = address!("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");

//         let reserve_ij_0 = Uint::<112, 2>::from(186207702687816381u128);
//         let reserve_ij_1 = Uint::<112, 2>::from(2782290017905555178812751u128);

//         let reserve_jk_0 = Uint::<112, 2>::from(15417613504024212838975381u128);
//         let reserve_jk_1 = Uint::<112, 2>::from(1939484985u128);

//         let reserve_ki_0 = Uint::<112, 2>::from(122843973192u128);
//         let reserve_ki_1 = Uint::<112, 2>::from(64043414140367650753u128);

//         //token0-token1
//         // let reserve_ij_0 = Uint::<112, 2>::from(40231970157230793u128);
//         // let reserve_ij_1 = Uint::<112, 2>::from(477843027700932911383u128);
//         let p_ij = price_log(Uint::from(3), &Reserves(reserve_ij_0, reserve_ij_1));
//         tracing::info!("p_ij = {}", p_ij);

//         //token1-token2
//         // let reserve_jk_0 = Uint::<112, 2>::from(300142426723603695424046u128);
//         // let reserve_jk_1 = Uint::<112, 2>::from(2243233282602387u128);
//         let p_jk = price_log(Uint::from(3), &Reserves(reserve_jk_0, reserve_jk_1));
//         tracing::info!("p_jk = {}", p_jk);

//         //token0-token2
//         // let reserve_ki_0 = Uint::<112, 2>::from(293433654763848772092u128);
//         // let reserve_ki_1 = Uint::<112, 2>::from(2907164345878467241383433u128);
//         let p_ki = price_log(Uint::from(3), &Reserves(reserve_ki_0, reserve_ki_1));
//         tracing::info!("p_ki = {}", p_ki);

//         tracing::info!("p_ij + p_jk + p_ki = {}", p_ij + p_jk + p_ki);
//         assert!(p_ij + p_jk + p_ki > 0f64);

//         /////////////////////////////////////////////////////////////////////
//         /////////////////////////////////////////////////////////////////////

//         // let reserves: [(Uint<112, 2>, Uint<112, 2>); 3] = [
//         //     (reserve_ij_0, reserve_ij_1),
//         //     (reserve_jk_0, reserve_jk_1),
//         //     (reserve_ki_0, reserve_ki_1),
//         // ];

//         // let best_out = optimal_amount_in_bin_search(&reserves);
//         // tracing::info!("optimal out: {:?}", best_out);

//         // Ok(())
//     }
// }

/*
2025-03-15T08:30:39.770462Z  INFO bot::math: token0: 0xA2b4C0Af19cC16a6CfAcCe81F192B024d625817D, token1: 0x514cdb9cd8A2fb2BdCf7A3b8DDd098CaF466E548, token2: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
2025-03-15T08:30:39.770602Z  INFO bot::math: token0-token1: reserve0: 40231970157230793, reserve1: 477843027700932911383
2025-03-15T08:30:39.770625Z  INFO bot::math: token1-token2: reserve0: 300142426723603695424046, reserve1: 2243233282602387
2025-03-15T08:30:39.770645Z  INFO bot::math: token0-token2: reserve0: 293433654763848772092, reserve1: 2907164345878467241383433
2025-03-15T08:30:39.773897Z  INFO bot::math: token0: 0x956F47F50A910163D8BF957Cf5846D573E7f87CA, token1: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token2: 0xc7283b66Eb1EB5FB86327f08e1B5816b0720212B
2025-03-15T08:30:39.774045Z  INFO bot::math: token0-token1: reserve0: 1113351787774514693547, reserve1: 576862833721677128
2025-03-15T08:30:39.774062Z  INFO bot::math: token1-token2: reserve0: 1653290990084274937, reserve1: 9081800306320123112293
2025-03-15T08:30:39.774078Z  INFO bot::math: token0-token2: reserve0: 1661813785629847859156879, reserve1: 577734324468183558940369
2025-03-15T08:30:39.774674Z  INFO bot::math: token0: 0xdAC17F958D2ee523a2206206994597C13D831ec7, token1: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token2: 0x0488401c3F535193Fa8Df029d9fFe615A06E74E6
2025-03-15T08:30:39.774822Z  INFO bot::math: token0-token1: reserve0: 1, reserve1: 22824391
2025-03-15T08:30:39.774837Z  INFO bot::math: token1-token2: reserve0: 15977931080918888120, reserve1: 297308588752440190304033179
2025-03-15T08:30:39.774853Z  INFO bot::math: token0-token2: reserve0: 49820356760677272256891881, reserve1: 5148961452
2025-03-15T08:30:39.774919Z  INFO bot::math: token0: 0xdAC17F958D2ee523a2206206994597C13D831ec7, token1: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token2: 0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE
2025-03-15T08:30:39.775053Z  INFO bot::math: token0-token1: reserve0: 1, reserve1: 22824391
2025-03-15T08:30:39.775069Z  INFO bot::math: token1-token2: reserve0: 1035394677544698125443, reserve1: 156027372504498882236814815576
2025-03-15T08:30:39.775085Z  INFO bot::math: token0-token2: reserve0: 2464330697636548010891304920, reserve1: 31427691625
2025-03-15T08:30:39.775114Z  INFO bot::math: token0: 0xdAC17F958D2ee523a2206206994597C13D831ec7, token1: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token2: 0x4206931337dc273a630d328dA6441786BfaD668f
2025-03-15T08:30:39.775198Z  INFO bot::math: token0-token1: reserve0: 1, reserve1: 22824391
2025-03-15T08:30:39.775206Z  INFO bot::math: token1-token2: reserve0: 930151931666016236882, reserve1: 1038238658063426
2025-03-15T08:30:39.775217Z  INFO bot::math: token0-token2: reserve0: 1137751417547996, reserve1: 1954329603114
2025-03-15T08:30:39.775290Z  INFO bot::math: token0: 0xdAC17F958D2ee523a2206206994597C13D831ec7, token1: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token2: 0x2b591e99afE9f32eAA6214f7B7629768c40Eeb39
2025-03-15T08:30:39.775368Z  INFO bot::math: token0-token1: reserve0: 1, reserve1: 22824391
2025-03-15T08:30:39.775377Z  INFO bot::math: token1-token2: reserve0: 125752380946074511549, reserve1: 10260034085067538
2025-03-15T08:30:39.775386Z  INFO bot::math: token0-token2: reserve0: 114480601232053, reserve1: 2702521494
2025-03-15T08:30:39.775401Z  INFO bot::math: token0: 0xdAC17F958D2ee523a2206206994597C13D831ec7, token1: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token2: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
2025-03-15T08:30:39.775476Z  INFO bot::math: token0-token1: reserve0: 1, reserve1: 22824391
2025-03-15T08:30:39.775484Z  INFO bot::math: token1-token2: reserve0: 64043414140367650753, reserve1: 122843973192
2025-03-15T08:30:39.775493Z  INFO bot::math: token0-token2: reserve0: 2358854568558, reserve1: 2363357144493
2025-03-15T08:30:39.775510Z  INFO bot::math: token0: 0xdAC17F958D2ee523a2206206994597C13D831ec7, token1: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token2: 0x667102BD3413bFEaa3Dffb48fa8288819E480a88
2025-03-15T08:30:39.775621Z  INFO bot::math: token0-token1: reserve0: 1, reserve1: 22824391
2025-03-15T08:30:39.775633Z  INFO bot::math: token1-token2: reserve0: 216898955979556837443, reserve1: 1546118306095
2025-03-15T08:30:39.775647Z  INFO bot::math: token0-token2: reserve0: 1158871955464, reserve1: 311367340047
2025-03-15T08:30:39.781166Z  INFO bot::math: token0: 0xc7283b66Eb1EB5FB86327f08e1B5816b0720212B, token1: 0x956F47F50A910163D8BF957Cf5846D573E7f87CA, token2: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
2025-03-15T08:30:39.781253Z  INFO bot::math: token0-token1: reserve0: 1661813785629847859156879, reserve1: 577734324468183558940369
2025-03-15T08:30:39.781262Z  INFO bot::math: token1-token2: reserve0: 1113351787774514693547, reserve1: 576862833721677128
2025-03-15T08:30:39.781271Z  INFO bot::math: token0-token2: reserve0: 1653290990084274937, reserve1: 9081800306320123112293
2025-03-15T08:30:39.784640Z  INFO bot::math: token0: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token1: 0x3845badAde8e6dFF049820680d1F14bD3903a5d0, token2: 0xe53EC727dbDEB9E2d5456c3be40cFF031AB40A55
2025-03-15T08:30:39.784742Z  INFO bot::math: token0-token1: reserve0: 203831529295350718892, reserve1: 1388428483632562506874340
2025-03-15T08:30:39.784753Z  INFO bot::math: token1-token2: reserve0: 165997723369937598048763, reserve1: 102907880135256195713695
2025-03-15T08:30:39.784764Z  INFO bot::math: token0-token2: reserve0: 5124444895502033875098061, reserve1: 1201876058017965609200
2025-03-15T08:30:39.784838Z  INFO bot::math: token0: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token1: 0x0488401c3F535193Fa8Df029d9fFe615A06E74E6, token2: 0xdAC17F958D2ee523a2206206994597C13D831ec7
2025-03-15T08:30:39.784937Z  INFO bot::math: token0-token1: reserve0: 15977931080918888120, reserve1: 297308588752440190304033179
2025-03-15T08:30:39.784948Z  INFO bot::math: token1-token2: reserve0: 49820356760677272256891881, reserve1: 5148961452
2025-03-15T08:30:39.784960Z  INFO bot::math: token0-token2: reserve0: 1, reserve1: 22824391
2025-03-15T08:30:39.784998Z  INFO bot::math: token0: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token1: 0xFbD5fD3f85e9f4c5E8B40EEC9F8B8ab1cAAa146b, token2: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
2025-03-15T08:30:39.785095Z  INFO bot::math: token0-token1: reserve0: 186207702687816381, reserve1: 2782290017905555178812751
2025-03-15T08:30:39.785107Z  INFO bot::math: token1-token2: reserve0: 15417613504024212838975381, reserve1: 1939484985
2025-03-15T08:30:39.785117Z  INFO bot::math: token0-token2: reserve0: 122843973192, reserve1: 64043414140367650753
2025-03-15T08:30:39.785185Z  INFO bot::math: token0: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token1: 0xc7283b66Eb1EB5FB86327f08e1B5816b0720212B, token2: 0x956F47F50A910163D8BF957Cf5846D573E7f87CA
2025-03-15T08:30:39.785286Z  INFO bot::math: token0-token1: reserve0: 1653290990084274937, reserve1: 9081800306320123112293
2025-03-15T08:30:39.785297Z  INFO bot::math: token1-token2: reserve0: 1661813785629847859156879, reserve1: 577734324468183558940369
2025-03-15T08:30:39.785308Z  INFO bot::math: token0-token2: reserve0: 1113351787774514693547, reserve1: 576862833721677128
2025-03-15T08:30:39.785360Z  INFO bot::math: token0: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token1: 0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE, token2: 0xdAC17F958D2ee523a2206206994597C13D831ec7
2025-03-15T08:30:39.785461Z  INFO bot::math: token0-token1: reserve0: 1035394677544698125443, reserve1: 156027372504498882236814815576
2025-03-15T08:30:39.785472Z  INFO bot::math: token1-token2: reserve0: 2464330697636548010891304920, reserve1: 31427691625
2025-03-15T08:30:39.785485Z  INFO bot::math: token0-token2: reserve0: 1, reserve1: 22824391
2025-03-15T08:30:39.785534Z  INFO bot::math: token0: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token1: 0x4206931337dc273a630d328dA6441786BfaD668f, token2: 0xdAC17F958D2ee523a2206206994597C13D831ec7
2025-03-15T08:30:39.785634Z  INFO bot::math: token0-token1: reserve0: 930151931666016236882, reserve1: 1038238658063426
2025-03-15T08:30:39.785645Z  INFO bot::math: token1-token2: reserve0: 1137751417547996, reserve1: 1954329603114
2025-03-15T08:30:39.785675Z  INFO bot::math: token0-token2: reserve0: 1, reserve1: 22824391
2025-03-15T08:30:39.785867Z  INFO bot::math: token0: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token1: 0x2b591e99afE9f32eAA6214f7B7629768c40Eeb39, token2: 0xdAC17F958D2ee523a2206206994597C13D831ec7
2025-03-15T08:30:39.785956Z  INFO bot::math: token0-token1: reserve0: 125752380946074511549, reserve1: 10260034085067538
2025-03-15T08:30:39.785971Z  INFO bot::math: token1-token2: reserve0: 114480601232053, reserve1: 2702521494
2025-03-15T08:30:39.785980Z  INFO bot::math: token0-token2: reserve0: 1, reserve1: 22824391
2025-03-15T08:30:39.786028Z  INFO bot::math: token0: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token1: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48, token2: 0xdAC17F958D2ee523a2206206994597C13D831ec7
2025-03-15T08:30:39.786105Z  INFO bot::math: token0-token1: reserve0: 64043414140367650753, reserve1: 122843973192
2025-03-15T08:30:39.786114Z  INFO bot::math: token1-token2: reserve0: 2358854568558, reserve1: 2363357144493
2025-03-15T08:30:39.786123Z  INFO bot::math: token0-token2: reserve0: 1, reserve1: 22824391
2025-03-15T08:30:39.786173Z  INFO bot::math: token0: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2, token1: 0x667102BD3413bFEaa3Dffb48fa8288819E480a88, token2: 0xdAC17F958D2ee523a2206206994597C13D831ec7
2025-03-15T08:30:39.786250Z  INFO bot::math: token0-token1: reserve0: 216898955979556837443, reserve1: 1546118306095
2025-03-15T08:30:39.786259Z  INFO bot::math: token1-token2: reserve0: 1158871955464, reserve1: 311367340047
2025-03-15T08:30:39.786268Z  INFO bot::math: token0-token2: reserve0: 1, reserve1: 22824391
*/
