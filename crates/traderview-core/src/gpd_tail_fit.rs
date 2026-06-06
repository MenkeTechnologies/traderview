//! Generalized Pareto Distribution (GPD) tail fit via Probability-
//! Weighted Moments (PWM) — Hosking & Wallis (1987).
//!
//! For exceedances y_i = (x_i − u) > 0 over threshold u, the GPD has
//! CDF:
//!
//!   F(y; ξ, β) = 1 − (1 + ξ·y/β)^(−1/ξ)        ξ ≠ 0
//!   F(y; 0, β) = 1 − exp(−y/β)                  ξ = 0
//!
//! PWM estimators (closed-form, more robust than MLE on small samples):
//!
//!   ξ̂ = 2 − ā_0 / (ā_0 − 2·ā_1)
//!   β̂ = 2·ā_0·ā_1 / (ā_0 − 2·ā_1)
//!
//! where ā_0 = mean(y) and ā_1 = mean(y · (1 − F̂_n(y))).
//!
//! Shape ξ interpretation:
//!   - ξ > 0 → heavy-tailed (Fréchet domain, e.g. equity losses)
//!   - ξ = 0 → exponential tail (Gumbel domain)
//!   - ξ < 0 → bounded tail (Weibull domain)
//!
//! Pure compute. Companion to `hill_estimator`, `peaks_over_threshold`,
//! `evt_value_at_risk`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpdFitReport {
    pub shape_xi: f64,
    pub scale_beta: f64,
    pub threshold: f64,
    pub n_exceedances: usize,
    pub mean_exceedance: f64,
}

pub fn fit(losses: &[f64], threshold: f64) -> Option<GpdFitReport> {
    if !threshold.is_finite() {
        return None;
    }
    let exceedances: Vec<f64> = losses
        .iter()
        .copied()
        .filter(|x| x.is_finite() && *x > threshold)
        .map(|x| x - threshold)
        .collect();
    let n = exceedances.len();
    if n < 10 {
        return None;
    }
    let n_f = n as f64;
    let mut sorted = exceedances.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let a0: f64 = sorted.iter().sum::<f64>() / n_f;
    // a_1 = (1/n) · Σ y_i · (n − rank_i) / (n − 1)
    //     where rank_i is 1-based ASCENDING rank.
    let a1: f64 = sorted
        .iter()
        .enumerate()
        .map(|(i, y)| {
            let rank = (i + 1) as f64;
            y * (n_f - rank) / (n_f - 1.0)
        })
        .sum::<f64>()
        / n_f;
    let denom = a0 - 2.0 * a1;
    if denom.abs() < 1e-18 {
        return None;
    }
    let xi = 2.0 - a0 / denom;
    let beta = 2.0 * a0 * a1 / denom;
    if !xi.is_finite() || !beta.is_finite() || beta <= 0.0 {
        return None;
    }
    Some(GpdFitReport {
        shape_xi: xi,
        scale_beta: beta,
        threshold,
        n_exceedances: n,
        mean_exceedance: a0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simulate_gpd(n: usize, xi: f64, beta: f64, seed: u64) -> Vec<f64> {
        let mut state = seed;
        (0..n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
                if xi.abs() < 1e-9 {
                    -beta * (1.0 - u).ln()
                } else {
                    beta / xi * ((1.0 - u).powf(-xi) - 1.0)
                }
            })
            .collect()
    }

    #[test]
    fn too_few_exceedances_returns_none() {
        let losses = vec![0.5_f64; 100];
        assert!(fit(&losses, 1.0).is_none());
    }

    #[test]
    fn nan_threshold_returns_none() {
        let losses = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!(fit(&losses, f64::NAN).is_none());
    }

    #[test]
    fn nan_inputs_filtered() {
        let mut losses = simulate_gpd(200, 0.3, 1.0, 42);
        losses.extend([f64::NAN, f64::INFINITY]);
        let r = fit(&losses, 0.0);
        assert!(r.is_some());
    }

    #[test]
    fn heavy_tail_recovers_positive_shape() {
        // Simulate GPD with ξ = 0.3 → fit should recover positive shape.
        let losses = simulate_gpd(2000, 0.3, 1.0, 42);
        let r = fit(&losses, 0.0).unwrap();
        assert!(r.shape_xi > 0.1, "expected ξ ≈ 0.3, got {}", r.shape_xi);
        assert!(r.scale_beta > 0.0);
    }

    #[test]
    fn exponential_tail_recovers_near_zero_shape() {
        // ξ = 0 → exponential tail; fit should recover ξ ≈ 0.
        let losses = simulate_gpd(2000, 0.0, 1.5, 7);
        let r = fit(&losses, 0.0).unwrap();
        assert!(
            r.shape_xi.abs() < 0.15,
            "expected ξ ≈ 0, got {}",
            r.shape_xi
        );
    }

    #[test]
    fn threshold_increases_n_exceedances_decreases() {
        let losses = simulate_gpd(1000, 0.3, 1.0, 11);
        let r_low = fit(&losses, 0.0).unwrap();
        let r_high = fit(&losses, 1.0).unwrap();
        assert!(r_high.n_exceedances < r_low.n_exceedances);
    }

    #[test]
    fn mean_exceedance_reported() {
        let losses = vec![
            0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5,
        ];
        let r = fit(&losses, 0.05);
        if let Some(rep) = r {
            assert!(rep.mean_exceedance > 0.0);
            assert_eq!(rep.threshold, 0.05);
        }
    }
}
