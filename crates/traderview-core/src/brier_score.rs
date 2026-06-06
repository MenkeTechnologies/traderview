//! Brier Score — strictly proper scoring rule for binary probabilistic
//! forecasts (Brier 1950).
//!
//!   BS = (1/N) · Σ (p_i − y_i)²
//!
//! where p_i ∈ [0, 1] is the predicted probability of the positive
//! outcome and y_i ∈ {0, 1} is the realized outcome.
//!
//! Range [0, 1]: 0 = perfect forecast; 0.25 = uninformative 50/50
//! coin flip on a balanced sample.
//!
//! Decomposition (Murphy 1973):
//!
//!   BS = Reliability − Resolution + Uncertainty
//!
//! computed by binning forecasts into K bins:
//!   Reliability  = Σ_k (n_k / N) · (p̄_k − ō_k)²    (calibration)
//!   Resolution   = Σ_k (n_k / N) · (ō_k − ō)²        (discrimination)
//!   Uncertainty  = ō · (1 − ō)                        (base-rate variance)
//!
//! Pure compute. Companion to `continuous_ranked_probability_score`,
//! `expected_calibration_error`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrierReport {
    pub brier_score: f64,
    pub reliability: f64,
    pub resolution: f64,
    pub uncertainty: f64,
    pub base_rate: f64,
    pub n_observations: usize,
}

pub fn compute(probabilities: &[f64], outcomes: &[u8], n_bins: usize) -> Option<BrierReport> {
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
    let n_f = n as f64;
    let brier: f64 = probabilities
        .iter()
        .zip(outcomes.iter())
        .map(|(p, y)| (p - *y as f64).powi(2))
        .sum::<f64>()
        / n_f;
    let base_rate: f64 = outcomes.iter().map(|y| *y as f64).sum::<f64>() / n_f;
    let uncertainty = base_rate * (1.0 - base_rate);
    // Reliability + resolution decomposition.
    let mut bin_p_sum = vec![0.0_f64; n_bins];
    let mut bin_y_sum = vec![0.0_f64; n_bins];
    let mut bin_n = vec![0_usize; n_bins];
    for (p, y) in probabilities.iter().zip(outcomes.iter()) {
        let bin = ((p * n_bins as f64).floor() as usize).min(n_bins - 1);
        bin_p_sum[bin] += p;
        bin_y_sum[bin] += *y as f64;
        bin_n[bin] += 1;
    }
    let mut reliability = 0.0_f64;
    let mut resolution = 0.0_f64;
    for k in 0..n_bins {
        if bin_n[k] == 0 {
            continue;
        }
        let nk = bin_n[k] as f64;
        let p_bar = bin_p_sum[k] / nk;
        let o_bar = bin_y_sum[k] / nk;
        reliability += (nk / n_f) * (p_bar - o_bar).powi(2);
        resolution += (nk / n_f) * (o_bar - base_rate).powi(2);
    }
    Some(BrierReport {
        brier_score: brier,
        reliability,
        resolution,
        uncertainty,
        base_rate,
        n_observations: n,
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
    fn mismatched_lengths_return_none() {
        assert!(compute(&[0.5, 0.5], &[1], 10).is_none());
    }

    #[test]
    fn invalid_probability_returns_none() {
        assert!(compute(&[1.1, 0.5], &[1, 0], 10).is_none());
        assert!(compute(&[-0.1, 0.5], &[1, 0], 10).is_none());
        assert!(compute(&[f64::NAN, 0.5], &[1, 0], 10).is_none());
    }

    #[test]
    fn invalid_outcome_returns_none() {
        assert!(compute(&[0.5, 0.5], &[1, 2], 10).is_none());
    }

    #[test]
    fn perfect_forecast_yields_zero_brier() {
        let probs = vec![1.0, 0.0, 1.0, 0.0, 1.0];
        let outcomes = vec![1_u8, 0, 1, 0, 1];
        let r = compute(&probs, &outcomes, 10).unwrap();
        assert!(r.brier_score.abs() < 1e-12);
    }

    #[test]
    fn random_forecast_brier_near_uncertainty() {
        // 50/50 forecasts on balanced sample → BS = uncertainty = 0.25.
        let probs = vec![0.5_f64; 10];
        let outcomes = vec![1_u8, 0, 1, 0, 1, 0, 1, 0, 1, 0];
        let r = compute(&probs, &outcomes, 10).unwrap();
        assert!((r.brier_score - 0.25).abs() < 1e-12);
        assert!((r.uncertainty - 0.25).abs() < 1e-12);
    }

    #[test]
    fn brier_equals_reliability_minus_resolution_plus_uncertainty() {
        // Murphy decomposition is only exact when each bin contains
        // forecasts of identical probability. Use 10 forecasts at
        // distinct deciles binned into 10 bins → one forecast per bin.
        let probs: Vec<f64> = (0..10).map(|i| 0.05 + i as f64 * 0.1).collect();
        let outcomes = vec![0_u8, 0, 1, 0, 1, 1, 0, 1, 1, 1];
        let r = compute(&probs, &outcomes, 10).unwrap();
        let recomposed = r.reliability - r.resolution + r.uncertainty;
        assert!(
            (r.brier_score - recomposed).abs() < 1e-9,
            "Murphy decomposition: BS={}, recomposed={}",
            r.brier_score,
            recomposed
        );
    }

    #[test]
    fn base_rate_computed_correctly() {
        let probs = vec![0.5_f64; 10];
        let outcomes = vec![1_u8, 1, 1, 0, 0, 0, 0, 0, 0, 0];
        let r = compute(&probs, &outcomes, 10).unwrap();
        assert!((r.base_rate - 0.3).abs() < 1e-12);
    }

    #[test]
    fn n_observations_reported() {
        let probs = vec![0.5_f64; 7];
        let outcomes = vec![1_u8; 7];
        let r = compute(&probs, &outcomes, 5).unwrap();
        assert_eq!(r.n_observations, 7);
    }
}
