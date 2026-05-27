//! Beta-weighted portfolio exposure + hedge ratio calculator.
//!
//! Two positions of equal dollar size aren't equally risky if their
//! betas differ. AAPL (β≈1.2) is roughly 20% more market-sensitive than
//! SPY (β=1.0). To express portfolio exposure honestly, weight every
//! position by its beta and sum.
//!
//! Output: per-position beta-weighted notional + total portfolio beta
//! exposure (in SPY-equivalent dollars) + hedge size that flattens it.
//!
//! Pure compute. Caller supplies the beta map (typically pulled from a
//! reference data source like the existing market_data module).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    /// Signed dollar notional. Negative = short.
    pub notional: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedPosition {
    pub symbol: String,
    pub notional: Decimal,
    /// β used for the weight (defaults to 1.0 if unknown).
    pub beta: Decimal,
    /// notional × beta.
    pub beta_weighted_notional: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HedgeReport {
    pub positions: Vec<WeightedPosition>,
    /// Sum of beta-weighted notionals. Positive = net long market;
    /// negative = net short.
    pub total_beta_exposure: Decimal,
    /// SPY-equivalent dollars to SHORT to flatten the exposure
    /// (or LONG, if you're net short — sign matches what you'd put
    /// in the order, i.e. opposite of total_beta_exposure).
    pub hedge_notional: Decimal,
    /// At an SPY price, how many shares to short/long to hedge.
    /// None if spy_price is None or zero.
    pub hedge_shares: Option<Decimal>,
}

pub fn compute(
    positions: &[Position],
    beta_by_symbol: &HashMap<String, Decimal>,
    spy_price: Option<Decimal>,
) -> HedgeReport {
    let default_beta = Decimal::ONE;
    let weighted: Vec<WeightedPosition> = positions
        .iter()
        .map(|p| {
            let beta = beta_by_symbol
                .get(&p.symbol)
                .copied()
                .unwrap_or(default_beta);
            WeightedPosition {
                symbol: p.symbol.clone(),
                notional: p.notional,
                beta,
                beta_weighted_notional: p.notional * beta,
            }
        })
        .collect();
    let total: Decimal = weighted.iter().map(|w| w.beta_weighted_notional).sum();
    let hedge_notional = -total; // short net long, long net short
    let hedge_shares = spy_price.and_then(|p| {
        if p.is_zero() {
            None
        } else {
            Some(hedge_notional / p)
        }
    });
    HedgeReport {
        positions: weighted,
        total_beta_exposure: total,
        hedge_notional,
        hedge_shares,
    }
}

/// Sample standard beta table for testing + common defaults. Real callers
/// override with a fresh market-data lookup.
pub fn default_betas() -> HashMap<String, Decimal> {
    let d = |s: &str| Decimal::from_str(s).unwrap();
    let mut m = HashMap::new();
    for (sym, b) in [
        ("SPY", "1.00"),
        ("QQQ", "1.15"),
        ("IWM", "1.20"),
        ("AAPL", "1.20"),
        ("MSFT", "1.05"),
        ("NVDA", "1.65"),
        ("TSLA", "2.20"),
        ("GME", "1.50"),
    ] {
        m.insert(sym.into(), d(b));
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn empty_positions_zero_exposure() {
        let r = compute(&[], &HashMap::new(), Some(d("400")));
        assert_eq!(r.total_beta_exposure, Decimal::ZERO);
        assert_eq!(r.hedge_shares, Some(Decimal::ZERO));
    }

    #[test]
    fn unknown_symbol_defaults_to_beta_one() {
        let r = compute(
            &[Position {
                symbol: "RANDO".into(),
                notional: d("10000"),
            }],
            &HashMap::new(),
            None,
        );
        assert_eq!(r.positions[0].beta, Decimal::ONE);
        assert_eq!(r.total_beta_exposure, d("10000"));
    }

    #[test]
    fn higher_beta_inflates_weighted_exposure() {
        let betas = default_betas();
        let r = compute(
            &[Position {
                symbol: "TSLA".into(),
                notional: d("10000"),
            }],
            &betas,
            None,
        );
        // TSLA β=2.20 → weighted = 22000.
        assert_eq!(r.total_beta_exposure, d("22000.00"));
    }

    #[test]
    fn short_positions_subtract_from_exposure() {
        let betas = default_betas();
        let r = compute(
            &[
                Position {
                    symbol: "AAPL".into(),
                    notional: d("10000"),
                }, // 1.20 × 10k = 12k
                Position {
                    symbol: "SPY".into(),
                    notional: d("-5000"),
                }, // 1.00 × -5k = -5k
            ],
            &betas,
            None,
        );
        assert_eq!(r.total_beta_exposure, d("7000.00"));
    }

    #[test]
    fn hedge_size_is_negative_of_net_exposure() {
        let betas = default_betas();
        let r = compute(
            &[
                Position {
                    symbol: "QQQ".into(),
                    notional: d("100000"),
                }, // 1.15 × 100k = 115k
            ],
            &betas,
            Some(d("500")),
        );
        assert_eq!(r.total_beta_exposure, d("115000.00"));
        assert_eq!(r.hedge_notional, d("-115000.00"));
        // Hedge shares = -115000 / 500 = -230 SPY (short 230 shares).
        assert_eq!(r.hedge_shares, Some(d("-230.00000000")));
    }

    #[test]
    fn flat_portfolio_zero_hedge() {
        let betas = default_betas();
        let r = compute(
            &[
                Position {
                    symbol: "SPY".into(),
                    notional: d("10000"),
                },
                Position {
                    symbol: "SPY".into(),
                    notional: d("-10000"),
                },
            ],
            &betas,
            Some(d("400")),
        );
        assert_eq!(r.total_beta_exposure, Decimal::ZERO);
        assert_eq!(r.hedge_notional, Decimal::ZERO);
    }

    #[test]
    fn zero_spy_price_yields_no_share_count() {
        let betas = default_betas();
        let r = compute(
            &[Position {
                symbol: "AAPL".into(),
                notional: d("10000"),
            }],
            &betas,
            Some(Decimal::ZERO),
        );
        assert!(r.hedge_shares.is_none());
    }

    #[test]
    fn no_spy_price_yields_no_share_count() {
        let betas = default_betas();
        let r = compute(
            &[Position {
                symbol: "AAPL".into(),
                notional: d("10000"),
            }],
            &betas,
            None,
        );
        assert!(r.hedge_shares.is_none());
    }
}
