//! Ulcer Index — drawdown-weighted volatility.
//!
//! Peter Martin's volatility measure that penalizes DEPTH and DURATION
//! of drawdowns rather than dispersion of returns. Distinct from standard
//! deviation (which treats up and down moves symmetrically) — the Ulcer
//! Index only cares about downside pain.
//!
//! Formula: `UI = √(mean(drawdown_pct²))` where `drawdown_pct` is the
//! percent decline from the highest prior peak.
//!
//! Companion metric: **Ulcer Performance Index (UPI)** = excess return /
//! Ulcer Index. Higher = better risk-adjusted performance (analogous to
//! Sharpe but using ulcer rather than stdev).
//!
//! Pure compute. Caller supplies the equity curve.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UlcerReport {
    pub ulcer_index: f64,
    /// Highest absolute drawdown seen during the curve.
    pub max_drawdown_pct: f64,
    /// Bar count of the longest drawdown duration.
    pub max_dd_duration: usize,
    /// UPI = (excess_return / ulcer_index). Only populated when
    /// `risk_free_rate` is supplied.
    pub upi: Option<f64>,
}

pub fn compute(equity: &[f64], risk_free_rate: Option<f64>) -> UlcerReport {
    let n = equity.len();
    if n < 2 { return UlcerReport::default(); }
    let mut peak = equity[0];
    let mut sum_dd_sq = 0.0_f64;
    let mut max_dd_pct = 0.0_f64;
    let mut current_dd_len = 0usize;
    let mut max_dd_len = 0usize;
    let mut count = 0usize;
    for &v in equity.iter() {
        if v > peak { peak = v; current_dd_len = 0; }
        if peak > 0.0 {
            let dd_pct = (v - peak) / peak * 100.0;
            sum_dd_sq += dd_pct * dd_pct;
            if dd_pct.abs() > max_dd_pct { max_dd_pct = dd_pct.abs(); }
            if v < peak {
                current_dd_len += 1;
                if current_dd_len > max_dd_len { max_dd_len = current_dd_len; }
            }
            count += 1;
        }
    }
    let ulcer_index = if count > 0 {
        (sum_dd_sq / count as f64).sqrt()
    } else { 0.0 };
    let upi = risk_free_rate.and_then(|rf| {
        if equity[0] <= 0.0 || ulcer_index == 0.0 { return None; }
        let total_return = (equity[n - 1] / equity[0] - 1.0) * 100.0;
        Some((total_return - rf) / ulcer_index)
    });
    UlcerReport {
        ulcer_index, max_drawdown_pct: max_dd_pct, max_dd_duration: max_dd_len, upi,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_single_point_returns_default() {
        assert_eq!(compute(&[], None).ulcer_index, 0.0);
        assert_eq!(compute(&[100.0], None).ulcer_index, 0.0);
    }

    #[test]
    fn monotonic_curve_has_zero_ulcer() {
        let r = compute(&[100.0, 105.0, 110.0, 115.0], None);
        assert_eq!(r.ulcer_index, 0.0, "no drawdowns → UI = 0");
        assert_eq!(r.max_drawdown_pct, 0.0);
    }

    #[test]
    fn single_drawdown_produces_positive_ulcer() {
        // Peak at 110, dips to 100 (−9.09%), recovers.
        let r = compute(&[100.0, 110.0, 100.0, 110.0, 120.0], None);
        assert!(r.ulcer_index > 0.0);
        assert!((r.max_drawdown_pct - 100.0 / 110.0 * 100.0 * 0.0909).abs() < 1.0);    // ~9.09%
    }

    #[test]
    fn longer_drawdowns_score_higher_ulcer() {
        // Same depth but A recovers immediately, B stays drawn down.
        let a = compute(&[100.0, 110.0, 100.0, 110.0], None);    // short drawdown
        let b = compute(&[100.0, 110.0, 100.0, 100.0, 100.0, 100.0, 110.0], None);    // long drawdown
        assert!(b.ulcer_index > a.ulcer_index,
            "longer drawdown should produce larger UI: a={} b={}", a.ulcer_index, b.ulcer_index);
    }

    #[test]
    fn max_dd_duration_tracked() {
        // Drawdown from peak at idx 1 (110) lasts 5 bars until recovery at idx 6.
        let r = compute(&[100.0, 110.0, 100.0, 100.0, 100.0, 100.0, 100.0, 110.0], None);
        assert!(r.max_dd_duration >= 5, "got {}", r.max_dd_duration);
    }

    #[test]
    fn upi_computed_when_risk_free_supplied() {
        // Steady uptrend with no drawdowns → UI = 0 → UPI undefined.
        let r0 = compute(&[100.0, 110.0, 120.0], Some(5.0));
        assert!(r0.upi.is_none(), "UI=0 → UPI undefined");
        // Drawdown present → UPI defined.
        let r1 = compute(&[100.0, 110.0, 95.0, 105.0, 115.0], Some(2.0));
        assert!(r1.upi.is_some());
        assert!(r1.upi.unwrap() > 0.0, "positive excess return / positive UI");
    }
}
