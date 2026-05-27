//! Earnings expected-move estimator.
//!
//! Reads the at-the-money (ATM) straddle price for the front expiry
//! that covers earnings and converts to expected ±% move.
//!
//! Two methods:
//!
//! 1. **Straddle method** (most common): expected move ≈ ATM straddle.
//!    Quick & dirty — assumes earnings is the only event embedded.
//!
//! 2. **Implied volatility method**: extract IV from the front expiry,
//!    annualize-back to the days-to-expiry, multiply by underlying.
//!    Better when other events (FOMC, etc) also embed.
//!
//! Output: dollar move + percent move + upper/lower band prices.
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExpectedMove {
    pub dollar_move: f64,
    pub pct_move: f64,
    pub upper_target: f64,
    pub lower_target: f64,
}

/// Straddle method: simplest. expected_move ≈ ATM call + ATM put.
pub fn from_straddle(underlying: f64, atm_call: f64, atm_put: f64) -> ExpectedMove {
    let straddle = atm_call + atm_put;
    ExpectedMove {
        dollar_move: straddle,
        pct_move: if underlying > 0.0 { straddle / underlying * 100.0 } else { 0.0 },
        upper_target: underlying + straddle,
        lower_target: underlying - straddle,
    }
}

/// IV method: expected_move = S × IV × √(T/365).
pub fn from_iv(underlying: f64, implied_vol: f64, days_to_expiry: f64) -> ExpectedMove {
    if days_to_expiry <= 0.0 || underlying <= 0.0 {
        return ExpectedMove::default();
    }
    let dollar = underlying * implied_vol * (days_to_expiry / 365.0).sqrt();
    ExpectedMove {
        dollar_move: dollar,
        pct_move: dollar / underlying * 100.0,
        upper_target: underlying + dollar,
        lower_target: underlying - dollar,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── straddle method ──────────────────────────────────────────────

    #[test]
    fn straddle_sum_of_call_and_put() {
        // Underlying 100, ATM straddle = $5 call + $5 put = $10.
        let m = from_straddle(100.0, 5.0, 5.0);
        assert_eq!(m.dollar_move, 10.0);
        assert_eq!(m.pct_move, 10.0);
        assert_eq!(m.upper_target, 110.0);
        assert_eq!(m.lower_target, 90.0);
    }

    #[test]
    fn straddle_pct_relative_to_underlying() {
        // Same $10 straddle on $200 underlying = 5%.
        let m = from_straddle(200.0, 5.0, 5.0);
        assert_eq!(m.pct_move, 5.0);
    }

    #[test]
    fn straddle_zero_underlying_returns_zero_pct() {
        let m = from_straddle(0.0, 5.0, 5.0);
        assert_eq!(m.pct_move, 0.0);
    }

    // ─── IV method ────────────────────────────────────────────────────

    #[test]
    fn iv_method_zero_days_returns_default() {
        let m = from_iv(100.0, 0.30, 0.0);
        assert_eq!(m.dollar_move, 0.0);
    }

    #[test]
    fn iv_method_one_day_yields_small_move() {
        // S=100, IV=30%, T=1day → move = 100 × 0.30 × √(1/365) ≈ 1.57.
        let m = from_iv(100.0, 0.30, 1.0);
        assert!((m.dollar_move - 100.0 * 0.30 * (1.0_f64/365.0).sqrt()).abs() < 1e-9);
    }

    #[test]
    fn iv_method_one_year_yields_iv_times_spot() {
        // T = 365 days → move = S × IV × 1 = $30.
        let m = from_iv(100.0, 0.30, 365.0);
        assert!((m.dollar_move - 30.0).abs() < 1e-9);
    }

    #[test]
    fn iv_method_scales_with_sqrt_time() {
        let four_days = from_iv(100.0, 0.30, 4.0);
        let one_day = from_iv(100.0, 0.30, 1.0);
        // √4 / √1 = 2 → four-day move = 2× one-day move.
        assert!((four_days.dollar_move / one_day.dollar_move - 2.0).abs() < 1e-9);
    }

    #[test]
    fn iv_method_negative_underlying_returns_default() {
        let m = from_iv(-50.0, 0.30, 10.0);
        assert_eq!(m.dollar_move, 0.0);
    }

    #[test]
    fn targets_symmetric_around_underlying() {
        let m = from_iv(150.0, 0.40, 30.0);
        assert!(((m.upper_target + m.lower_target) / 2.0 - 150.0).abs() < 1e-9);
    }
}
