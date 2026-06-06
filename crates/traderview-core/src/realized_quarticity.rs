//! Realized Quarticity — Barndorff-Nielsen & Shephard (2002).
//!
//! Fourth-moment realized statistic used as the variance of the
//! realized variance estimator:
//!
//!   RQ = (n / 3) · Σ r_i^4
//!
//! Under a continuous Itô semimartingale, n · (RV − IV) →_p √2 · IQ
//! where IQ = ∫ σ⁴ dt is the integrated quarticity. RQ is the
//! sample analogue of IQ.
//!
//! Use cases:
//!   - Construct standard errors / confidence intervals on RV
//!   - Input to the Huang-Tauchen jump-test statistic (see
//!     `bipower_variation`)
//!   - Build feasible HAR-RV forecasts with weights inversely
//!     proportional to RQ
//!
//! Tripower variant (jump-robust) for comparison:
//!
//!   TQ = n · μ_{4/3}^{-3} · Σ |r_{i-2}|^{4/3} · |r_{i-1}|^{4/3} · |r_i|^{4/3}
//!
//! Pure compute. Companion to `bipower_variation`,
//! `realized_volatility`, `realized_higher_moments`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealizedQuarticityReport {
    pub realized_quarticity: f64,
    pub tripower_quarticity: f64,
    pub realized_variance: f64,
    /// Approximate standard error of RV: √(2·RQ / n).
    pub rv_standard_error: f64,
    pub n_returns: usize,
}

pub fn compute(returns: &[f64]) -> Option<RealizedQuarticityReport> {
    let n = returns.len();
    if n < 5 {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n_f = n as f64;
    let rv: f64 = returns.iter().map(|r| r * r).sum();
    let rq = (n_f / 3.0) * returns.iter().map(|r| r.powi(4)).sum::<f64>();
    // Tripower with 3-bar product window.
    let mu_43 = 2.0_f64.powf(2.0 / 3.0) * gamma_7_6() / std::f64::consts::PI.sqrt();
    let mu_43_cubed_inv = 1.0 / mu_43.powi(3);
    let tq_sum: f64 = (2..n)
        .map(|i| {
            returns[i].abs().powf(4.0 / 3.0)
                * returns[i - 1].abs().powf(4.0 / 3.0)
                * returns[i - 2].abs().powf(4.0 / 3.0)
        })
        .sum();
    let tq = n_f * mu_43_cubed_inv * tq_sum;
    let rv_se = (2.0 * rq / n_f).max(0.0).sqrt();
    Some(RealizedQuarticityReport {
        realized_quarticity: rq,
        tripower_quarticity: tq,
        realized_variance: rv,
        rv_standard_error: rv_se,
        n_returns: n,
    })
}

fn gamma_7_6() -> f64 {
    0.927_553_793_283_388_2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01, -0.01]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(compute(&[0.01, f64::NAN, 0.02, 0.01, 0.005]).is_none());
    }

    #[test]
    fn flat_returns_yield_zero_rq() {
        let r = compute(&[0.0; 50]).unwrap();
        assert_eq!(r.realized_quarticity, 0.0);
        assert_eq!(r.rv_standard_error, 0.0);
    }

    #[test]
    fn rq_proportional_to_sum_of_fourth_powers() {
        let returns = vec![0.01, 0.02, 0.005, 0.015, 0.01];
        let r = compute(&returns).unwrap();
        let sum_r4: f64 = returns.iter().map(|x| x.powi(4)).sum();
        let n_f = returns.len() as f64;
        let expected_rq = (n_f / 3.0) * sum_r4;
        assert!((r.realized_quarticity - expected_rq).abs() < 1e-15);
    }

    #[test]
    fn jump_inflates_rq_more_than_tq() {
        // Add a big single-period jump; RQ inflates more strongly than TQ
        // because TQ uses |r|^{4/3} (sublinear in |r|), so a large isolated
        // |r| has dampened impact.
        let mut r = vec![0.001_f64; 200];
        r[100] = 0.5;
        let result = compute(&r).unwrap();
        // TQ should be smaller than RQ when there's a jump.
        assert!(
            result.tripower_quarticity < result.realized_quarticity,
            "jump should depress TQ vs RQ: TQ = {}, RQ = {}",
            result.tripower_quarticity,
            result.realized_quarticity
        );
    }

    #[test]
    fn rv_standard_error_scales_with_quarticity() {
        let returns = vec![0.01_f64; 50];
        let r = compute(&returns).unwrap();
        let expected_se = (2.0 * r.realized_quarticity / 50.0).sqrt();
        assert!((r.rv_standard_error - expected_se).abs() < 1e-15);
    }

    #[test]
    fn n_reported_correctly() {
        let returns = vec![0.01_f64; 100];
        let r = compute(&returns).unwrap();
        assert_eq!(r.n_returns, 100);
    }
}
