//! Unusual Options Activity (UOA) Scanner.
//!
//! Flags option contracts where intraday volume diverges sharply from
//! historical open interest — the prototypical signal for "smart
//! money" positioning. Each contract is scored on multiple dimensions
//! and emitted only if it crosses configurable thresholds.
//!
//! Per-contract scoring:
//!   - **vol_oi_ratio** = volume / open_interest                  ; > 1 = unusual
//!   - **premium_paid** = volume · last_price · 100               ; dollars at risk
//!   - **direction**    = "above_ask" / "below_bid" / "midpoint"  ; aggressor side
//!
//! Emit only contracts meeting ALL of:
//!   vol_oi_ratio ≥ min_vol_oi_ratio
//!   volume ≥ min_volume
//!   premium_paid ≥ min_premium_paid
//!
//! Output sorted by premium_paid descending so the biggest "money in"
//! prints surface first.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionContract {
    pub symbol: String,
    pub strike: f64,
    pub expiry: String,
    /// "call" or "put"
    pub option_type: String,
    pub volume: f64,
    pub open_interest: f64,
    pub last_price: f64,
    pub bid: f64,
    pub ask: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FillSide { AboveAsk, BelowBid, Midpoint }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UoaHit {
    pub symbol: String,
    pub strike: f64,
    pub expiry: String,
    pub option_type: String,
    pub volume: f64,
    pub open_interest: f64,
    pub vol_oi_ratio: f64,
    pub premium_paid: f64,
    pub fill_side: FillSide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub min_vol_oi_ratio: f64,
    pub min_volume: f64,
    pub min_premium_paid: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self { min_vol_oi_ratio: 2.0, min_volume: 500.0, min_premium_paid: 100_000.0 }
    }
}

pub fn scan(contracts: &[OptionContract], cfg: &Config) -> Vec<UoaHit> {
    let mut hits: Vec<UoaHit> = contracts.iter()
        .filter(|c| c.volume.is_finite() && c.open_interest.is_finite()
            && c.last_price.is_finite() && c.bid.is_finite() && c.ask.is_finite()
            && c.volume >= 0.0 && c.open_interest >= 0.0 && c.last_price >= 0.0
            && c.bid >= 0.0 && c.ask >= c.bid)
        .filter_map(|c| {
            let oi = c.open_interest.max(1.0);    // floor to 1 to avoid div by zero
            let vol_oi = c.volume / oi;
            let premium = c.volume * c.last_price * 100.0;
            if vol_oi < cfg.min_vol_oi_ratio
                || c.volume < cfg.min_volume
                || premium < cfg.min_premium_paid { return None; }
            let side = if c.last_price >= c.ask { FillSide::AboveAsk }
                else if c.last_price <= c.bid { FillSide::BelowBid }
                else { FillSide::Midpoint };
            Some(UoaHit {
                symbol: c.symbol.clone(),
                strike: c.strike,
                expiry: c.expiry.clone(),
                option_type: c.option_type.clone(),
                volume: c.volume,
                open_interest: c.open_interest,
                vol_oi_ratio: vol_oi,
                premium_paid: premium,
                fill_side: side,
            })
        })
        .collect();
    hits.sort_by(|a, b| b.premium_paid.partial_cmp(&a.premium_paid)
        .unwrap_or(std::cmp::Ordering::Equal));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(sym: &str, vol: f64, oi: f64, last: f64, bid: f64, ask: f64) -> OptionContract {
        OptionContract {
            symbol: sym.into(),
            strike: 100.0,
            expiry: "2026-06-19".into(),
            option_type: "call".into(),
            volume: vol,
            open_interest: oi,
            last_price: last,
            bid,
            ask,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(scan(&[], &Config::default()).is_empty());
    }

    #[test]
    fn below_thresholds_filtered_out() {
        // Premium = 100 · 2.50 · 100 = $25,000 < $100K threshold.
        let contracts = vec![c("AAPL", 100.0, 10.0, 2.50, 2.45, 2.55)];
        assert!(scan(&contracts, &Config::default()).is_empty());
    }

    #[test]
    fn low_vol_oi_filtered_out() {
        // Vol 200, OI 1000 → ratio 0.2 < 2.0 threshold.
        let contracts = vec![c("AAPL", 200.0, 1000.0, 5.00, 4.95, 5.05)];
        assert!(scan(&contracts, &Config::default()).is_empty());
    }

    #[test]
    fn meets_all_thresholds_emitted() {
        // vol=2000, oi=500, ratio=4, premium = 2000·5·100 = $1M.
        let contracts = vec![c("TSLA", 2000.0, 500.0, 5.00, 4.95, 5.05)];
        let hits = scan(&contracts, &Config::default());
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].symbol, "TSLA");
        assert!((hits[0].vol_oi_ratio - 4.0).abs() < 1e-9);
        assert!((hits[0].premium_paid - 1_000_000.0).abs() < 1e-9);
    }

    #[test]
    fn fill_above_ask_marked_aggressive() {
        let contracts = vec![c("NVDA", 2000.0, 500.0, 5.10, 4.95, 5.05)];
        let hits = scan(&contracts, &Config::default());
        assert_eq!(hits[0].fill_side, FillSide::AboveAsk);
    }

    #[test]
    fn fill_below_bid_marked_aggressive() {
        let contracts = vec![c("NVDA", 2000.0, 500.0, 4.90, 4.95, 5.05)];
        let hits = scan(&contracts, &Config::default());
        assert_eq!(hits[0].fill_side, FillSide::BelowBid);
    }

    #[test]
    fn sorted_by_premium_descending() {
        let contracts = vec![
            c("AAA", 1000.0, 100.0, 1.00, 0.95, 1.05),    // premium $100K
            c("BBB", 2000.0, 100.0, 5.00, 4.95, 5.05),    // premium $1M
            c("CCC", 1500.0, 100.0, 2.00, 1.95, 2.05),    // premium $300K
        ];
        let hits = scan(&contracts, &Config::default());
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].symbol, "BBB");
        assert_eq!(hits[1].symbol, "CCC");
        assert_eq!(hits[2].symbol, "AAA");
    }

    #[test]
    fn nan_or_negative_fields_filtered() {
        let bad_vol = c("X", f64::NAN, 100.0, 5.0, 4.9, 5.1);
        let neg_oi = c("Y", 1000.0, -1.0, 5.0, 4.9, 5.1);
        let crossed = c("Z", 1000.0, 100.0, 5.0, 5.1, 4.9);    // bid > ask
        let hits = scan(&[bad_vol, neg_oi, crossed], &Config::default());
        assert!(hits.is_empty());
    }
}
