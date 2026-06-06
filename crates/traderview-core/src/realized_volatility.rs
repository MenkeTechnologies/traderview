//! Realized Volatility & Bipower Variation (Andersen, Bollerslev,
//! Diebold, Labys; Barndorff-Nielsen & Shephard).
//!
//! For an intraday return series r_1..r_M aggregating to one day:
//!   - **Realized Variance**    RV = Σ r_i²
//!   - **Bipower Variation**    BV = (π/2) Σ |r_i| |r_{i+1}|
//!   - **Jump Component**       J  = max(RV − BV, 0)
//!   - **Continuous Variance**  C  = BV
//!   - **Realized Volatility**  RVol = √RV
//!
//! BV (Barndorff-Nielsen & Shephard 2004) is robust to jumps because
//! consecutive returns are unlikely to both jump in the same window;
//! using |r_i||r_{i+1}| dampens single-bar outliers. Jumps are then
//! identified as the difference RV − BV.
//!
//! Per-window outputs: caller provides intraday log-returns segmented
//! into daily blocks (or rolling windows). Each window produces one
//! RV / BV / J / RVol triple.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindowMetrics {
    pub realized_variance: f64,
    pub bipower_variation: f64,
    pub jump_component: f64,
    pub realized_volatility: f64,
}

pub fn compute(intraday_returns: &[Vec<f64>]) -> Vec<Option<WindowMetrics>> {
    intraday_returns.iter().map(|w| single_window(w)).collect()
}

fn single_window(returns: &[f64]) -> Option<WindowMetrics> {
    if returns.is_empty() {
        return None;
    }
    let mut rv = 0.0_f64;
    let mut ok = true;
    for r in returns {
        if !r.is_finite() {
            ok = false;
            break;
        }
        rv += r * r;
    }
    if !ok || !rv.is_finite() || rv < 0.0 {
        return None;
    }
    let bv = if returns.len() >= 2 {
        let mut sum = 0.0_f64;
        let mut bv_ok = true;
        for w in returns.windows(2) {
            if !w[0].is_finite() || !w[1].is_finite() {
                bv_ok = false;
                break;
            }
            sum += w[0].abs() * w[1].abs();
        }
        if !bv_ok {
            return None;
        }
        (std::f64::consts::PI / 2.0) * sum
    } else {
        0.0
    };
    let jump = (rv - bv).max(0.0);
    let rvol = rv.sqrt();
    Some(WindowMetrics {
        realized_variance: rv,
        bipower_variation: bv,
        jump_component: jump,
        realized_volatility: rvol,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[]).is_empty());
    }

    #[test]
    fn empty_window_yields_none() {
        let out = compute(&[Vec::<f64>::new()]);
        assert!(out[0].is_none());
    }

    #[test]
    fn nan_return_skips_window() {
        let out = compute(&[vec![0.01, f64::NAN, 0.02]]);
        assert!(out[0].is_none());
    }

    #[test]
    fn flat_zero_returns_yield_zero_metrics() {
        let out = compute(&[vec![0.0; 100]]);
        let m = out[0].expect("populated");
        assert_eq!(m.realized_variance, 0.0);
        assert_eq!(m.bipower_variation, 0.0);
        assert_eq!(m.jump_component, 0.0);
        assert_eq!(m.realized_volatility, 0.0);
    }

    #[test]
    fn smooth_diffusion_rv_approximately_equals_bv() {
        // Returns sampled from a smooth process (small constants) →
        // RV ≈ BV → jump should be ≈ 0 (no extreme outliers).
        let returns: Vec<f64> = (0..100).map(|i| 0.001 * ((i as f64).sin())).collect();
        let out = compute(&[returns]);
        let m = out[0].unwrap();
        // |RV − BV| should be tiny vs RV magnitude.
        if m.realized_variance > 1e-12 {
            assert!(
                m.jump_component / m.realized_variance < 0.5,
                "smooth process should have small jump fraction: jump={} RV={}",
                m.jump_component,
                m.realized_variance
            );
        }
    }

    #[test]
    fn single_large_jump_inflates_rv_relative_to_bv() {
        // All small returns + one huge spike → RV dominated by spike,
        // BV barely sees it (only paired with neighbors).
        let mut returns = vec![0.0001; 100];
        returns[50] = 0.5; // huge jump
        let out = compute(&[returns]);
        let m = out[0].unwrap();
        // Jump component should be the dominant share of RV.
        assert!(
            m.jump_component > 0.5 * m.realized_variance,
            "jump should dominate RV: jump={} RV={}",
            m.jump_component,
            m.realized_variance
        );
    }

    #[test]
    fn single_observation_bv_is_zero() {
        let out = compute(&[vec![0.01]]);
        let m = out[0].unwrap();
        assert!((m.realized_variance - 0.0001).abs() < 1e-12);
        assert_eq!(m.bipower_variation, 0.0);
        assert!((m.jump_component - 0.0001).abs() < 1e-12);
    }

    #[test]
    fn multiple_windows_processed_independently() {
        let out = compute(&[vec![0.01, 0.02, 0.01], vec![0.05, 0.05, 0.05], vec![]]);
        assert!(out[0].is_some());
        assert!(out[1].is_some());
        assert!(out[2].is_none());
        // Bigger returns → bigger RV.
        assert!(out[1].unwrap().realized_variance > out[0].unwrap().realized_variance);
    }
}
