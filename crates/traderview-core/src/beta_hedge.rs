//! Beta-adjusted hedge calculator.
//!
//! For a long stock position, compute the number of SPY (or other
//! benchmark) shares to short to neutralize market beta exposure:
//!
//!   hedge_shares = position_notional × beta / benchmark_price
//!
//! Caller can also compute a partial hedge (e.g. 50% market-neutral)
//! by scaling.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HedgeReport {
    pub position_notional: f64,
    pub beta: f64,
    pub benchmark_price: f64,
    /// Negative = short the benchmark to hedge a long stock.
    pub hedge_shares: f64,
    pub hedge_notional: f64,
    pub partial_pct: f64,
}

/// `partial_pct` = 1.0 for full hedge, 0.5 for half-hedge, 0.0 for none.
pub fn compute(position_notional: f64, beta: f64, benchmark_price: f64, partial_pct: f64)
    -> HedgeReport
{
    if benchmark_price <= 0.0 {
        return HedgeReport { position_notional, beta, benchmark_price, partial_pct, ..Default::default() };
    }
    let full_hedge_notional = position_notional * beta;
    let scaled = full_hedge_notional * partial_pct;
    let shares = -scaled / benchmark_price;    // negative = short
    HedgeReport {
        position_notional,
        beta,
        benchmark_price,
        hedge_shares: shares,
        hedge_notional: scaled,
        partial_pct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_benchmark_price_returns_zero_hedge() {
        let r = compute(100_000.0, 1.2, 0.0, 1.0);
        assert_eq!(r.hedge_shares, 0.0);
    }

    #[test]
    fn beta_one_full_hedge_matches_position_dollar_for_dollar() {
        // $100k long AAPL with beta 1 vs SPY at $500 → short 200 SPY.
        let r = compute(100_000.0, 1.0, 500.0, 1.0);
        assert_eq!(r.hedge_shares, -200.0);
        assert_eq!(r.hedge_notional, 100_000.0);
    }

    #[test]
    fn beta_2_doubles_required_hedge() {
        // High-beta stock: $100k × 2.0 / $500 = 400 SPY short.
        let r = compute(100_000.0, 2.0, 500.0, 1.0);
        assert_eq!(r.hedge_shares, -400.0);
    }

    #[test]
    fn low_beta_under_one_smaller_hedge() {
        // $100k utility stock with beta 0.5 → only 100 SPY short.
        let r = compute(100_000.0, 0.5, 500.0, 1.0);
        assert_eq!(r.hedge_shares, -100.0);
    }

    #[test]
    fn partial_hedge_scales_proportionally() {
        let full = compute(100_000.0, 1.0, 500.0, 1.0);
        let half = compute(100_000.0, 1.0, 500.0, 0.5);
        assert!((half.hedge_shares - full.hedge_shares / 2.0).abs() < 1e-9);
    }

    #[test]
    fn zero_partial_pct_no_hedge() {
        let r = compute(100_000.0, 1.0, 500.0, 0.0);
        assert_eq!(r.hedge_shares, 0.0);
    }

    #[test]
    fn negative_position_short_long_benchmark_to_hedge() {
        // Short $100k of AAPL → need LONG SPY to hedge (positive shares).
        let r = compute(-100_000.0, 1.0, 500.0, 1.0);
        assert_eq!(r.hedge_shares, 200.0);
    }
}
