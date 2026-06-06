//! Expected Calibration Error (ECE) — Naeini, Cooper, Hauskrecht (2015).
//!
//! Diagnostic for binary probabilistic forecasts measuring the average
//! gap between predicted probability and empirical frequency within
//! confidence bins:
//!
//!   ECE = Σ_k (n_k / N) · | p̄_k − ō_k |
//!
//! where p̄_k is the mean forecast probability in bin k and ō_k is
//! the empirical fraction of positive outcomes in that bin.
//!
//! Also reports:
//!   - **MCE** (Maximum Calibration Error): max bin-wise gap (worst bin)
//!   - Per-bin diagnostic: predicted vs observed, sample size
//!
//! Range [0, 1]; 0 = perfect calibration.
//!
//! Distinct from `brier_score` (which measures both calibration AND
//! resolution); ECE/MCE measure calibration only.
//!
//! Use cases:
//!   - Validate ML classifier probability outputs
//!   - Verify implied-probability calibration from option markets
//!   - Diagnose over-confident / under-confident forecasts
//!
//! Pure compute. Companion to `brier_score`,
//! `continuous_ranked_probability_score`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinDiagnostic {
    pub bin_index: usize,
    pub mean_predicted_probability: f64,
    pub empirical_positive_rate: f64,
    pub n_observations: usize,
    pub calibration_gap: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EceReport {
    pub expected_calibration_error: f64,
    pub maximum_calibration_error: f64,
    pub bin_diagnostics: Vec<BinDiagnostic>,
    pub n_observations: usize,
    pub n_bins: usize,
}

pub fn compute(probabilities: &[f64], outcomes: &[u8], n_bins: usize) -> Option<EceReport> {
    let n = probabilities.len();
    if n == 0 || outcomes.len() != n || n_bins == 0 {
        return None;
    }
    if probabilities
        .iter()
        .any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
    {
        return None;
    }
    if outcomes.iter().any(|o| *o > 1) {
        return None;
    }
    let mut bin_p_sum = vec![0.0_f64; n_bins];
    let mut bin_y_sum = vec![0.0_f64; n_bins];
    let mut bin_n = vec![0_usize; n_bins];
    for (p, y) in probabilities.iter().zip(outcomes.iter()) {
        let bin = ((p * n_bins as f64).floor() as usize).min(n_bins - 1);
        bin_p_sum[bin] += p;
        bin_y_sum[bin] += *y as f64;
        bin_n[bin] += 1;
    }
    let n_f = n as f64;
    let mut ece = 0.0_f64;
    let mut mce = 0.0_f64;
    let mut diagnostics = Vec::with_capacity(n_bins);
    for k in 0..n_bins {
        if bin_n[k] == 0 {
            diagnostics.push(BinDiagnostic {
                bin_index: k,
                mean_predicted_probability: 0.0,
                empirical_positive_rate: 0.0,
                n_observations: 0,
                calibration_gap: 0.0,
            });
            continue;
        }
        let nk = bin_n[k] as f64;
        let p_bar = bin_p_sum[k] / nk;
        let o_bar = bin_y_sum[k] / nk;
        let gap = (p_bar - o_bar).abs();
        ece += (nk / n_f) * gap;
        if gap > mce {
            mce = gap;
        }
        diagnostics.push(BinDiagnostic {
            bin_index: k,
            mean_predicted_probability: p_bar,
            empirical_positive_rate: o_bar,
            n_observations: bin_n[k],
            calibration_gap: gap,
        });
    }
    Some(EceReport {
        expected_calibration_error: ece,
        maximum_calibration_error: mce,
        bin_diagnostics: diagnostics,
        n_observations: n,
        n_bins,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[], &[], 10).is_none());
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[0.5, 0.5], &[1], 10).is_none());
        assert!(compute(&[1.5, 0.5], &[1, 0], 10).is_none());
        assert!(compute(&[0.5, 0.5], &[1, 2], 10).is_none());
        assert!(compute(&[0.5, 0.5], &[1, 0], 0).is_none());
    }

    #[test]
    fn perfect_calibration_yields_zero_ece() {
        // 10 forecasts at 1.0 with all-positive outcomes; ECE = 0.
        let probs = vec![1.0_f64; 10];
        let outcomes = vec![1_u8; 10];
        let r = compute(&probs, &outcomes, 10).unwrap();
        assert_eq!(r.expected_calibration_error, 0.0);
        assert_eq!(r.maximum_calibration_error, 0.0);
    }

    #[test]
    fn miscalibrated_high_yields_positive_ece() {
        // All forecasts say 0.9 but only 30% of outcomes are positive.
        let probs = vec![0.9_f64; 100];
        let mut outcomes = vec![0_u8; 100];
        for slot in outcomes.iter_mut().take(30) {
            *slot = 1;
        }
        let r = compute(&probs, &outcomes, 10).unwrap();
        // Bin = floor(0.9 · 10) = 9. p_bar = 0.9, o_bar = 0.3 → gap 0.6.
        assert!((r.expected_calibration_error - 0.6).abs() < 1e-9);
        assert!((r.maximum_calibration_error - 0.6).abs() < 1e-9);
    }

    #[test]
    fn ece_bounded_in_unit_range() {
        let probs = vec![0.5_f64; 50];
        let outcomes: Vec<u8> = (0..50).map(|i| (i % 3 == 0) as u8).collect();
        let r = compute(&probs, &outcomes, 10).unwrap();
        assert!((0.0..=1.0).contains(&r.expected_calibration_error));
        assert!((0.0..=1.0).contains(&r.maximum_calibration_error));
    }

    #[test]
    fn mce_at_least_as_large_as_ece() {
        let probs = vec![0.1, 0.3, 0.5, 0.7, 0.9, 0.2, 0.4, 0.6, 0.8, 0.5];
        let outcomes = vec![0_u8, 0, 1, 1, 1, 0, 0, 1, 1, 1];
        let r = compute(&probs, &outcomes, 5).unwrap();
        assert!(r.maximum_calibration_error >= r.expected_calibration_error);
    }

    #[test]
    fn empty_bins_handled() {
        let probs = vec![0.9_f64; 5];
        let outcomes = vec![1_u8; 5];
        let r = compute(&probs, &outcomes, 10).unwrap();
        // Most bins should be empty.
        let empty_count = r
            .bin_diagnostics
            .iter()
            .filter(|b| b.n_observations == 0)
            .count();
        assert!(empty_count >= 5);
    }

    #[test]
    fn n_observations_reported() {
        let probs = vec![0.5_f64; 17];
        let outcomes = vec![1_u8; 17];
        let r = compute(&probs, &outcomes, 5).unwrap();
        assert_eq!(r.n_observations, 17);
        assert_eq!(r.n_bins, 5);
    }
}
