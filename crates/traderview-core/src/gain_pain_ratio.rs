//! Jack Schwager's Gain-to-Pain Ratio.
//!
//!   GPR = Σ positive_returns / |Σ negative_returns|
//!
//! Schwager prefers this to Sharpe because it (a) only penalizes
//! downside volatility (asymmetric like Sortino), (b) is naturally
//! invariant to scale/units, and (c) is intuitive: GPR > 1 means total
//! gains exceeded total losses by that ratio. Typical equity portfolios
//! score 0.5–1.5; > 2 over a multi-year window is considered very good
//! by Schwager's own (oft-cited) hedge-fund rankings.
//!
//! Returns the ratio plus the explicit sums for caller diagnostics.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct GpReport {
    pub ratio: f64,
    pub total_gain: f64,
    pub total_pain: f64,
    pub n_gains: usize,
    pub n_losses: usize,
}

pub fn compute(returns: &[f64]) -> Option<GpReport> {
    if returns.is_empty() {
        return None;
    }
    let mut gain = 0.0_f64;
    let mut pain = 0.0_f64;
    let mut n_gain = 0_usize;
    let mut n_loss = 0_usize;
    let mut any_valid = false;
    for r in returns {
        if !r.is_finite() {
            continue;
        }
        any_valid = true;
        if *r > 0.0 {
            gain += r;
            n_gain += 1;
        } else if *r < 0.0 {
            pain += -r;
            n_loss += 1;
        }
    }
    if !any_valid {
        return None;
    }
    let ratio = if pain == 0.0 {
        if gain == 0.0 {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        gain / pain
    };
    Some(GpReport {
        ratio,
        total_gain: gain,
        total_pain: pain,
        n_gains: n_gain,
        n_losses: n_loss,
    })
}

/// Rolling-window variant: returns per-bar GPR over `window` past returns.
pub fn rolling(returns: &[f64], window: usize) -> Vec<Option<f64>> {
    let n = returns.len();
    let mut out = vec![None; n];
    if window < 2 || n < window {
        return out;
    }
    for (i, slot) in out.iter_mut().enumerate().skip(window - 1) {
        let lo = i + 1 - window;
        if let Some(report) = compute(&returns[lo..=i]) {
            *slot = Some(report.ratio);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[]).is_none());
    }

    #[test]
    fn all_nan_returns_none() {
        assert!(compute(&[f64::NAN; 5]).is_none());
    }

    #[test]
    fn all_gains_yields_infinity() {
        let r = vec![0.01, 0.02, 0.03];
        let report = compute(&r).unwrap();
        assert!(report.ratio.is_infinite());
        assert_eq!(report.n_losses, 0);
    }

    #[test]
    fn all_losses_yields_zero() {
        let r = vec![-0.01, -0.02, -0.03];
        let report = compute(&r).unwrap();
        assert_eq!(report.ratio, 0.0);
        assert_eq!(report.n_gains, 0);
    }

    #[test]
    fn balanced_symmetric_returns_yields_one() {
        let r = vec![0.01, -0.01, 0.02, -0.02];
        let report = compute(&r).unwrap();
        assert!((report.ratio - 1.0).abs() < 1e-9);
    }

    #[test]
    fn nan_observations_skipped_safely() {
        let r = vec![0.01, f64::NAN, -0.005, 0.02];
        let report = compute(&r).unwrap();
        assert_eq!(report.n_gains, 2);
        assert_eq!(report.n_losses, 1);
        assert!((report.ratio - 0.03 / 0.005).abs() < 1e-9);
    }

    #[test]
    fn zero_returns_neither_gain_nor_loss() {
        let r = vec![0.0, 0.0, 0.0];
        let report = compute(&r).unwrap();
        assert_eq!(report.n_gains, 0);
        assert_eq!(report.n_losses, 0);
        assert_eq!(report.ratio, 0.0);
    }

    #[test]
    fn rolling_window_returns_consistent_values() {
        // 100 alternating ±1% returns → GPR ≈ 1 in each window.
        let r: Vec<f64> = (0_usize..100)
            .map(|i| if i.is_multiple_of(2) { 0.01 } else { -0.01 })
            .collect();
        let out = rolling(&r, 20);
        for v in out.iter().flatten() {
            assert!((v - 1.0).abs() < 1e-9, "expected GPR ≈ 1, got {v}");
        }
    }

    #[test]
    fn rolling_window_too_small_returns_all_none() {
        assert!(rolling(&[0.01; 5], 0).iter().all(|x| x.is_none()));
        assert!(rolling(&[0.01; 5], 1).iter().all(|x| x.is_none()));
    }

    #[test]
    fn rolling_window_too_large_returns_all_none() {
        assert!(rolling(&[0.01; 5], 10).iter().all(|x| x.is_none()));
    }
}
