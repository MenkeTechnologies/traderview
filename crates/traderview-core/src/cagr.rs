//! CAGR — Compound Annual Growth Rate.
//!
//! Simple form:
//!   CAGR = (ending / beginning)^(1/years) - 1
//!
//! Plus a rolling-period helper: compute N-year CAGRs at each anchor
//! point so the user can see the distribution (best/worst/median rolling
//! 3yr return).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

pub fn simple(beginning: f64, ending: f64, years: f64) -> Option<f64> {
    if beginning <= 0.0 || ending <= 0.0 || years <= 0.0 { return None; }
    Some((ending / beginning).powf(1.0 / years) - 1.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    /// Years since first observation (0.0 = first point, 1.0 = a year later).
    pub years: f64,
    pub equity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RollingReport {
    pub windows_count: usize,
    pub best_cagr: f64,
    pub worst_cagr: f64,
    pub median_cagr: f64,
    pub mean_cagr: f64,
    pub all_cagrs: Vec<f64>,
}

/// Compute CAGR over every rolling `period_years` window in the series.
pub fn rolling(equity: &[EquityPoint], period_years: f64) -> RollingReport {
    let mut report = RollingReport::default();
    if equity.len() < 2 || period_years <= 0.0 { return report; }
    let mut cagrs = Vec::new();
    let mut j = 0;
    for i in 0..equity.len() {
        // Find smallest j > i where years_diff >= period_years.
        if j <= i { j = i + 1; }
        while j < equity.len() && equity[j].years - equity[i].years < period_years {
            j += 1;
        }
        if j >= equity.len() { break; }
        let years = equity[j].years - equity[i].years;
        if let Some(c) = simple(equity[i].equity, equity[j].equity, years) {
            cagrs.push(c);
        }
    }
    if cagrs.is_empty() { return report; }
    let mut sorted = cagrs.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mean = cagrs.iter().sum::<f64>() / cagrs.len() as f64;
    report.windows_count = cagrs.len();
    report.best_cagr = sorted[sorted.len() - 1];
    report.worst_cagr = sorted[0];
    report.median_cagr = sorted[sorted.len() / 2];
    report.mean_cagr = mean;
    report.all_cagrs = cagrs;
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(y: f64, e: f64) -> EquityPoint {
        EquityPoint { years: y, equity: e }
    }

    // ─── simple ────────────────────────────────────────────────────────

    #[test]
    fn cagr_double_in_one_year_is_100pct() {
        let r = simple(100.0, 200.0, 1.0).unwrap();
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cagr_zero_growth_returns_zero() {
        let r = simple(100.0, 100.0, 5.0).unwrap();
        assert!(r.abs() < 1e-9);
    }

    #[test]
    fn cagr_negative_loss() {
        // 100 → 50 over 1 year = -50%.
        let r = simple(100.0, 50.0, 1.0).unwrap();
        assert!((r + 0.5).abs() < 1e-9);
    }

    #[test]
    fn cagr_returns_none_on_zero_or_negative_inputs() {
        assert!(simple(0.0, 100.0, 1.0).is_none());
        assert!(simple(100.0, -50.0, 1.0).is_none());
        assert!(simple(100.0, 200.0, 0.0).is_none());
    }

    #[test]
    fn cagr_compounds_correctly() {
        // 100 → 144 over 2 years should be 20% CAGR (1.20^2 = 1.44).
        let r = simple(100.0, 144.0, 2.0).unwrap();
        assert!((r - 0.20).abs() < 1e-9);
    }

    // ─── rolling ───────────────────────────────────────────────────────

    #[test]
    fn empty_returns_default() {
        let r = rolling(&[], 1.0);
        assert_eq!(r.windows_count, 0);
    }

    #[test]
    fn rolling_one_year_emits_one_window_per_eligible_anchor() {
        let equity = vec![
            pt(0.0, 100.0),
            pt(1.0, 110.0),
            pt(2.0, 121.0),
            pt(3.0, 133.1),
        ];
        let r = rolling(&equity, 1.0);
        // 3 rolling 1-year windows.
        assert_eq!(r.windows_count, 3);
        // 10% growth → all CAGRs ≈ 10%.
        for c in &r.all_cagrs {
            assert!((c - 0.10).abs() < 1e-9);
        }
    }

    #[test]
    fn rolling_period_larger_than_history_returns_empty() {
        let equity = vec![pt(0.0, 100.0), pt(1.0, 110.0)];
        let r = rolling(&equity, 5.0);
        assert_eq!(r.windows_count, 0);
    }

    #[test]
    fn rolling_best_and_worst_extracted() {
        let equity = vec![
            pt(0.0, 100.0),
            pt(1.0, 110.0),     // +10%
            pt(2.0, 88.0),      // -20% from prior
            pt(3.0, 132.0),     // +50% from prior
        ];
        let r = rolling(&equity, 1.0);
        assert!(r.best_cagr > 0.40, "best 1yr should be ~50% got {}", r.best_cagr);
        assert!(r.worst_cagr < -0.10);
    }

    #[test]
    fn rolling_mean_matches_arithmetic_average_of_cagrs() {
        let equity = vec![
            pt(0.0, 100.0),
            pt(1.0, 110.0),
            pt(2.0, 121.0),
        ];
        let r = rolling(&equity, 1.0);
        let expected = r.all_cagrs.iter().sum::<f64>() / r.all_cagrs.len() as f64;
        assert!((r.mean_cagr - expected).abs() < 1e-12);
    }
}
