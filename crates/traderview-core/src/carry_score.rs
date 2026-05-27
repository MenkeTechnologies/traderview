//! Carry-trade attractiveness score.
//!
//! For a long-currency vs funding-currency pair, the carry trade
//! profitability depends on:
//!   1. Interest-rate differential (positive carry).
//!   2. FX volatility (carry trades blow up in volatile periods).
//!
//! Naive but useful score:
//!   carry_score = (rate_long - rate_funding) / annualized_vol
//!
//! Analogous to the Sharpe ratio for FX carry. Higher = more attractive.
//! Standard rule of thumb: > 1.0 = strong, 0.5-1.0 = okay, < 0.5 = poor.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CarryReport {
    pub long_rate: f64,
    pub funding_rate: f64,
    pub rate_differential: f64,
    pub annualized_vol: f64,
    pub carry_score: f64,
    pub tier: CarryTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CarryTier {
    Strong,
    Okay,
    #[default]
    Poor,
    Negative,
}

pub fn score(long_rate: f64, funding_rate: f64, annualized_vol: f64) -> CarryReport {
    let diff = long_rate - funding_rate;
    let score = if annualized_vol > 0.0 { diff / annualized_vol } else { 0.0 };
    let tier = if diff < 0.0 { CarryTier::Negative }
        else if score >= 1.0 { CarryTier::Strong }
        else if score >= 0.5 { CarryTier::Okay }
        else                  { CarryTier::Poor };
    CarryReport {
        long_rate,
        funding_rate,
        rate_differential: diff,
        annualized_vol,
        carry_score: score,
        tier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_diff_low_vol_strong_carry() {
        // 5% long, 1% funding, 3% vol → diff=4%, score = 0.04/0.03 = 1.33.
        let r = score(0.05, 0.01, 0.03);
        assert!(r.carry_score > 1.0);
        assert_eq!(r.tier, CarryTier::Strong);
    }

    #[test]
    fn positive_diff_high_vol_poor_carry() {
        // 5% long, 1% funding, 20% vol → diff=4%, score=0.04/0.20=0.20 → poor.
        let r = score(0.05, 0.01, 0.20);
        assert!(r.carry_score < 0.5);
        assert_eq!(r.tier, CarryTier::Poor);
    }

    #[test]
    fn middling_score_okay_tier() {
        // diff 3%, vol 4% → score 0.75 → okay.
        let r = score(0.04, 0.01, 0.04);
        assert_eq!(r.tier, CarryTier::Okay);
    }

    #[test]
    fn negative_differential_negative_tier() {
        // Long rate LOWER than funding — anti-carry.
        let r = score(0.01, 0.05, 0.10);
        assert!(r.rate_differential < 0.0);
        assert_eq!(r.tier, CarryTier::Negative);
    }

    #[test]
    fn zero_vol_zero_score_no_panic() {
        let r = score(0.05, 0.01, 0.0);
        assert_eq!(r.carry_score, 0.0);
    }

    #[test]
    fn exactly_one_score_at_strong_boundary() {
        let r = score(0.05, 0.0, 0.05);    // diff/vol = 1.0
        assert!((r.carry_score - 1.0).abs() < 1e-12);
        assert_eq!(r.tier, CarryTier::Strong);
    }

    #[test]
    fn rate_differential_correct() {
        let r = score(0.07, 0.02, 0.10);
        assert!((r.rate_differential - 0.05).abs() < 1e-12);
    }
}
