//! Peaks Over Threshold (POT) — Davison & Smith (1990).
//!
//! Classic EVT methodology for tail estimation:
//!   1. Choose threshold u (e.g. 95th or 99th percentile of |returns|)
//!   2. Identify exceedances {y_i = x_i − u | x_i > u}
//!   3. Fit GPD to {y_i} via `gpd_tail_fit::fit`
//!   4. Use fitted GPD to extrapolate beyond observed data
//!
//! Threshold selection heuristics shipped here:
//!   - **Quantile-based**: u = q-th percentile of absolute losses
//!   - **Mean Residual Life plot input** (returned for user to inspect)
//!
//! Outputs the fitted GPD parameters + diagnostic mean-residual-life
//! sequence (mean exceedance vs threshold) to support visual threshold
//! selection in the trader's UI.
//!
//! Pure compute. Companion to `gpd_tail_fit`, `evt_value_at_risk`,
//! `hill_estimator`.

use serde::{Deserialize, Serialize};

use crate::gpd_tail_fit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdDiagnostic {
    pub threshold: f64,
    pub n_exceedances: usize,
    pub mean_excess: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PotReport {
    pub gpd: gpd_tail_fit::GpdFitReport,
    pub mean_residual_life: Vec<ThresholdDiagnostic>,
    pub quantile_used: f64,
}

pub fn run(
    losses: &[f64],
    quantile: f64,
    mrl_grid: &[f64],
) -> Option<PotReport> {
    if losses.len() < 50 || !quantile.is_finite() || !(0.5..1.0).contains(&quantile) {
        return None;
    }
    let mut clean: Vec<f64> = losses.iter().copied().filter(|x| x.is_finite()).collect();
    if clean.len() < 50 { return None; }
    clean.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let threshold = clean[((quantile * clean.len() as f64).floor() as usize).min(clean.len() - 1)];
    let gpd = gpd_tail_fit::fit(losses, threshold)?;
    let mut mrl = Vec::with_capacity(mrl_grid.len());
    for &t in mrl_grid {
        if !t.is_finite() { continue; }
        let exc: Vec<f64> = losses.iter().copied()
            .filter(|x| x.is_finite() && *x > t).map(|x| x - t).collect();
        if exc.is_empty() {
            mrl.push(ThresholdDiagnostic { threshold: t, n_exceedances: 0, mean_excess: 0.0 });
            continue;
        }
        let mean: f64 = exc.iter().sum::<f64>() / exc.len() as f64;
        mrl.push(ThresholdDiagnostic {
            threshold: t,
            n_exceedances: exc.len(),
            mean_excess: mean,
        });
    }
    Some(PotReport {
        gpd,
        mean_residual_life: mrl,
        quantile_used: quantile,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sim(n: usize, xi: f64, beta: f64, seed: u64) -> Vec<f64> {
        let mut state = seed;
        (0..n).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            if xi.abs() < 1e-9 {
                -beta * (1.0 - u).ln()
            } else {
                beta / xi * ((1.0 - u).powf(-xi) - 1.0)
            }
        }).collect()
    }

    #[test]
    fn too_short_or_invalid_quantile_returns_none() {
        assert!(run(&[1.0; 10], 0.95, &[1.0]).is_none());
        let s = sim(200, 0.2, 1.0, 1);
        assert!(run(&s, 0.4, &[1.0]).is_none());
        assert!(run(&s, 1.0, &[1.0]).is_none());
    }

    #[test]
    fn recovers_gpd_shape_above_quantile() {
        // Heavy-tail simulation; POT at 90th percentile recovers ξ.
        let s = sim(2000, 0.30, 1.0, 42);
        let r = run(&s, 0.90, &[]).unwrap();
        assert!(r.gpd.shape_xi > 0.1, "shape {} too small", r.gpd.shape_xi);
    }

    #[test]
    fn mrl_grid_populated() {
        let s = sim(1000, 0.25, 1.0, 7);
        let grid = vec![0.5, 1.0, 2.0, 3.0];
        let r = run(&s, 0.90, &grid).unwrap();
        assert_eq!(r.mean_residual_life.len(), 4);
        for d in &r.mean_residual_life {
            assert!(grid.contains(&d.threshold));
        }
    }

    #[test]
    fn higher_threshold_in_grid_yields_fewer_exceedances() {
        let s = sim(1000, 0.25, 1.0, 11);
        let r = run(&s, 0.90, &[1.0, 2.0, 3.0]).unwrap();
        // Monotone non-increasing exceedance count.
        for w in r.mean_residual_life.windows(2) {
            assert!(w[1].n_exceedances <= w[0].n_exceedances);
        }
    }

    #[test]
    fn quantile_recorded() {
        let s = sim(500, 0.2, 1.0, 99);
        let r = run(&s, 0.95, &[]).unwrap();
        assert_eq!(r.quantile_used, 0.95);
    }
}
