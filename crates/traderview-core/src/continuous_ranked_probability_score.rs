//! Continuous Ranked Probability Score (CRPS) — Matheson & Winkler (1976).
//!
//! Strictly proper scoring rule for probabilistic forecasts of a
//! continuous variable. For a predictive CDF F and a realized
//! observation y:
//!
//!   CRPS(F, y) = ∫ (F(x) − 1{x ≥ y})² dx
//!
//! Empirical (sample-CDF) approximation when F is represented by an
//! ensemble of M samples {z_1, …, z_M}:
//!
//!   CRPS(z, y) = (1/M) · Σ |z_i − y|  −  (1/(2 M²)) · Σ_i Σ_j |z_i − z_j|
//!
//! Lower CRPS = better forecast. Reduces to MAE for point forecasts
//! (single-sample ensemble) and to a variance-aware metric for spread.
//!
//! Use cases:
//!   - Validate Monte-Carlo / scenario-based return forecasts
//!   - Compare two probabilistic forecasting models
//!   - Diagnostic for tail-risk model calibration
//!
//! Pure compute. Companion to `brier_score`, `expected_calibration_error`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrpsReport {
    pub per_event_crps: Vec<f64>,
    pub mean_crps: f64,
    pub n_events: usize,
}

pub fn ensemble(
    ensembles: &[Vec<f64>],
    observations: &[f64],
) -> Option<CrpsReport> {
    let n = observations.len();
    if n == 0 || ensembles.len() != n { return None; }
    if observations.iter().any(|y| !y.is_finite()) { return None; }
    if ensembles.iter().any(|e| e.is_empty() || e.iter().any(|x| !x.is_finite())) {
        return None;
    }
    let mut per_event = Vec::with_capacity(n);
    for (e, y) in ensembles.iter().zip(observations.iter()) {
        per_event.push(crps_single(e, *y));
    }
    let mean: f64 = per_event.iter().sum::<f64>() / n as f64;
    Some(CrpsReport {
        per_event_crps: per_event,
        mean_crps: mean,
        n_events: n,
    })
}

fn crps_single(ensemble: &[f64], y: f64) -> f64 {
    let m = ensemble.len() as f64;
    let mae: f64 = ensemble.iter().map(|z| (z - y).abs()).sum::<f64>() / m;
    // Sorted-ensemble closed form for the pairwise mean absolute distance:
    //   (1 / M²) Σ_i Σ_j |z_i − z_j| = (2 / M²) · Σ_i (2i − M − 1) · z_(i)
    // (using 1-based index i, sorted ascending).
    let mut sorted = ensemble.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut weighted = 0.0_f64;
    for (i_zero, z) in sorted.iter().enumerate() {
        let i = (i_zero + 1) as f64;
        weighted += (2.0 * i - m - 1.0) * z;
    }
    let pairwise_mean = (2.0 / (m * m)) * weighted;
    mae - 0.5 * pairwise_mean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(ensemble(&[], &[]).is_none());
    }

    #[test]
    fn mismatched_lengths_returns_none() {
        let e = vec![vec![1.0, 2.0]];
        let o = vec![1.0, 2.0];
        assert!(ensemble(&e, &o).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let e = vec![vec![1.0, 2.0]];
        let o = vec![f64::NAN];
        assert!(ensemble(&e, &o).is_none());
        let e_bad = vec![vec![1.0, f64::NAN]];
        let o = vec![1.0];
        assert!(ensemble(&e_bad, &o).is_none());
    }

    #[test]
    fn empty_inner_ensemble_returns_none() {
        let e = vec![vec![]];
        let o = vec![1.0];
        assert!(ensemble(&e, &o).is_none());
    }

    #[test]
    fn point_forecast_reduces_to_absolute_error() {
        // Single-sample ensemble = point forecast → CRPS = |z - y|.
        let e = vec![vec![5.0], vec![10.0]];
        let o = vec![3.0, 12.0];
        let r = ensemble(&e, &o).unwrap();
        assert!((r.per_event_crps[0] - 2.0).abs() < 1e-9);
        assert!((r.per_event_crps[1] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn perfect_calibration_yields_zero_crps() {
        // Ensemble = point mass at the realization → CRPS = 0.
        let e = vec![vec![5.0; 100]];
        let o = vec![5.0];
        let r = ensemble(&e, &o).unwrap();
        assert!(r.mean_crps.abs() < 1e-9);
    }

    #[test]
    fn wider_ensemble_higher_crps_when_off_target() {
        let tight = [vec![10.0_f64; 20]];
        let mut wide = Vec::new();
        for i in -10..=10 { wide.push(10.0 + i as f64); }
        let e_tight = vec![tight[0].clone()];
        let e_wide = vec![wide];
        let o = vec![10.0];
        let r_tight = ensemble(&e_tight, &o).unwrap();
        let r_wide = ensemble(&e_wide, &o).unwrap();
        // Tight = 0 vs wide > 0 (penalized for unnecessary spread).
        assert!(r_wide.mean_crps > r_tight.mean_crps);
    }

    #[test]
    fn mean_crps_is_average_of_per_event() {
        let e = vec![vec![1.0, 2.0, 3.0], vec![10.0, 20.0, 30.0]];
        let o = vec![2.0, 20.0];
        let r = ensemble(&e, &o).unwrap();
        let avg: f64 = r.per_event_crps.iter().sum::<f64>() / r.per_event_crps.len() as f64;
        assert!((r.mean_crps - avg).abs() < 1e-12);
    }

    #[test]
    fn n_events_reported() {
        let e = vec![vec![1.0]; 5];
        let o = vec![1.0; 5];
        let r = ensemble(&e, &o).unwrap();
        assert_eq!(r.n_events, 5);
    }
}
