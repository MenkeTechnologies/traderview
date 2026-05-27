//! Implied-volatility rank + percentile.
//!
//! Two canonical "is this option cheap or expensive" metrics:
//!
//! **IV Rank**:
//!   (current_iv - 52w_low) / (52w_high - 52w_low) × 100
//!
//! Linear distance from 52w low to 52w high. 0 = at 52w low (cheap),
//! 100 = at 52w high (expensive).
//!
//! **IV Percentile**:
//!   fraction of trading days in the lookback when IV ≤ current_iv × 100
//!
//! Distribution-aware — better for skewed IV series where rank is
//! pulled toward extremes by a single spike (e.g. an earnings event).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IvRankReport {
    pub current_iv: f64,
    pub low_52w: f64,
    pub high_52w: f64,
    /// IV rank, 0..=100. Returns 0 when 52w range is degenerate.
    pub iv_rank: f64,
    /// IV percentile, 0..=100. Fraction of observations <= current.
    pub iv_percentile: f64,
    pub observations: usize,
}

pub fn compute(current_iv: f64, history: &[f64]) -> IvRankReport {
    if history.is_empty() {
        return IvRankReport {
            current_iv,
            ..Default::default()
        };
    }
    let low = history.iter().cloned().fold(f64::INFINITY, f64::min);
    let high = history.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = high - low;
    let iv_rank = if range > 0.0 {
        ((current_iv - low) / range * 100.0).clamp(0.0, 100.0)
    } else { 0.0 };
    let le_count = history.iter().filter(|v| **v <= current_iv).count();
    let iv_percentile = le_count as f64 / history.len() as f64 * 100.0;
    IvRankReport {
        current_iv,
        low_52w: low,
        high_52w: high,
        iv_rank,
        iv_percentile,
        observations: history.len(),
    }
}

/// Classify the IV environment for a quick UI badge. Standard trader
/// conventions:
///   - IV rank < 25: low (favor net-debit strategies, long premium)
///   - IV rank 25-75: normal
///   - IV rank > 75: high (favor net-credit strategies, sell premium)
pub fn classify(rank: f64) -> IvEnvironment {
    if rank < 25.0 { IvEnvironment::Low }
    else if rank < 75.0 { IvEnvironment::Normal }
    else { IvEnvironment::High }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IvEnvironment { Low, Normal, High }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_history_returns_zero_metrics() {
        let r = compute(0.30, &[]);
        assert_eq!(r.iv_rank, 0.0);
        assert_eq!(r.iv_percentile, 0.0);
    }

    #[test]
    fn current_at_52w_low_rank_zero() {
        let r = compute(0.10, &[0.10, 0.20, 0.30, 0.40]);
        assert_eq!(r.iv_rank, 0.0);
        assert!(r.iv_percentile > 0.0, "percentile reflects the obs at current");
    }

    #[test]
    fn current_at_52w_high_rank_100() {
        let r = compute(0.40, &[0.10, 0.20, 0.30, 0.40]);
        assert_eq!(r.iv_rank, 100.0);
        assert_eq!(r.iv_percentile, 100.0);
    }

    #[test]
    fn current_at_midpoint_rank_50() {
        let r = compute(0.25, &[0.10, 0.20, 0.30, 0.40]);
        // (0.25 - 0.10) / (0.40 - 0.10) = 0.5.
        assert!((r.iv_rank - 50.0).abs() < 1e-9);
    }

    #[test]
    fn rank_clamps_when_current_above_52w_high() {
        // Current > history high → clamps at 100.
        let r = compute(0.50, &[0.10, 0.20, 0.30]);
        assert_eq!(r.iv_rank, 100.0);
    }

    #[test]
    fn rank_clamps_when_current_below_52w_low() {
        let r = compute(0.05, &[0.10, 0.20, 0.30]);
        assert_eq!(r.iv_rank, 0.0);
    }

    #[test]
    fn percentile_inclusive_at_current() {
        // Current = 0.25 in history [0.10, 0.20, 0.25, 0.30, 0.40].
        // ≤ 0.25 → 3 obs / 5 = 60%.
        let r = compute(0.25, &[0.10, 0.20, 0.25, 0.30, 0.40]);
        assert_eq!(r.iv_percentile, 60.0);
    }

    #[test]
    fn degenerate_history_all_same_value() {
        // Range = 0 → rank = 0 (degenerate).
        let r = compute(0.20, &[0.20, 0.20, 0.20]);
        assert_eq!(r.iv_rank, 0.0);
        assert_eq!(r.iv_percentile, 100.0);
    }

    // ─── classification ───────────────────────────────────────────────

    #[test]
    fn classify_below_25_is_low() {
        assert_eq!(classify(10.0), IvEnvironment::Low);
        assert_eq!(classify(24.9), IvEnvironment::Low);
    }

    #[test]
    fn classify_25_to_75_is_normal() {
        assert_eq!(classify(25.0), IvEnvironment::Normal);
        assert_eq!(classify(50.0), IvEnvironment::Normal);
        assert_eq!(classify(74.9), IvEnvironment::Normal);
    }

    #[test]
    fn classify_75_and_above_is_high() {
        assert_eq!(classify(75.0), IvEnvironment::High);
        assert_eq!(classify(99.0), IvEnvironment::High);
    }
}
