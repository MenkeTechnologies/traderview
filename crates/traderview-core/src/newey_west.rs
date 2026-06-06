//! Newey-West HAC (Heteroskedasticity- and Autocorrelation-Consistent)
//! Standard Errors (Newey & West 1987).
//!
//! For an OLS regression y = α + β·x + ε with potentially serially
//! correlated and heteroskedastic residuals, the HAC variance of β̂ is:
//!
//!   V̂_β = (X'X)⁻¹ · Ω̂ · (X'X)⁻¹
//!
//!   Ω̂ = Σ_t x_t · x_t' · ê_t² + Σ_{l=1..L} w(l, L) · (Γ_l + Γ_l')
//!
//! where w(l, L) = 1 − l / (L + 1) is the Bartlett kernel weight and
//! Γ_l = Σ_t x_t · x_{t-l}' · ê_t · ê_{t-l}.
//!
//! The Bartlett kernel guarantees the resulting Ω̂ is positive
//! semi-definite. Default lag L = ⌊4·(n/100)^(2/9)⌋ per Newey-West's
//! original automatic-bandwidth rule.
//!
//! Use cases:
//!   - Robust inference on β with serially correlated returns
//!     (overlapping samples, momentum/long-horizon regressions)
//!   - Standard-error reporting in macro / asset-pricing models
//!
//! Univariate predictor only. Pure compute.
//!
//! Companion to `breusch_godfrey`, `breusch_pagan_test`,
//! `white_robust_se`, `factor_models`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NeweyWestReport {
    pub alpha: f64,
    pub beta: f64,
    pub se_alpha_hac: f64,
    pub se_beta_hac: f64,
    pub t_stat_beta_hac: f64,
    pub se_alpha_ols: f64,
    pub se_beta_ols: f64,
    pub lag_truncation: usize,
    pub n_observations: usize,
}

pub fn estimate(x: &[f64], y: &[f64], lag: Option<usize>) -> Option<NeweyWestReport> {
    let n = x.len();
    if n < 10 || y.len() != n {
        return None;
    }
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let n_f = n as f64;
    let x_mean: f64 = x.iter().sum::<f64>() / n_f;
    let y_mean: f64 = y.iter().sum::<f64>() / n_f;
    let mut sxx = 0.0_f64;
    let mut sxy = 0.0_f64;
    for i in 0..n {
        sxx += (x[i] - x_mean).powi(2);
        sxy += (x[i] - x_mean) * (y[i] - y_mean);
    }
    if sxx <= 0.0 {
        return None;
    }
    let beta = sxy / sxx;
    let alpha = y_mean - beta * x_mean;
    let resid: Vec<f64> = (0..n).map(|i| y[i] - alpha - beta * x[i]).collect();
    // OLS classical SEs.
    let dof = (n - 2) as f64;
    let sigma2 = resid.iter().map(|r| r * r).sum::<f64>() / dof;
    let se_beta_ols = (sigma2 / sxx).sqrt();
    let se_alpha_ols = (sigma2 * (1.0 / n_f + x_mean * x_mean / sxx)).sqrt();
    // HAC variance via Newey-West Bartlett kernel.
    // Use mean-deviation z_t = (x_t − x̄) · ê_t (slope-equation moment).
    let z: Vec<f64> = (0..n).map(|i| (x[i] - x_mean) * resid[i]).collect();
    let l = lag
        .unwrap_or_else(|| (4.0 * (n_f / 100.0).powf(2.0 / 9.0)).floor() as usize)
        .min(n - 1);
    let mut omega: f64 = z.iter().map(|zi| zi * zi).sum();
    for k in 1..=l {
        let w = 1.0 - k as f64 / (l as f64 + 1.0);
        let cov_k: f64 = (k..n).map(|t| z[t] * z[t - k]).sum();
        omega += 2.0 * w * cov_k;
    }
    let var_beta_hac = omega / (sxx * sxx);
    let se_beta_hac = var_beta_hac.max(0.0).sqrt();
    // For α, use the same z-summation framework with z_α = ê (constant column).
    let z_alpha: Vec<f64> = resid.clone();
    let mut omega_alpha: f64 = z_alpha.iter().map(|zi| zi * zi).sum();
    for k in 1..=l {
        let w = 1.0 - k as f64 / (l as f64 + 1.0);
        let cov_k: f64 = (k..n).map(|t| z_alpha[t] * z_alpha[t - k]).sum();
        omega_alpha += 2.0 * w * cov_k;
    }
    let var_alpha_hac = omega_alpha * (1.0 / n_f + x_mean * x_mean / sxx).powi(2);
    let se_alpha_hac = var_alpha_hac.max(0.0).sqrt();
    let t_beta = if se_beta_hac > 0.0 {
        beta / se_beta_hac
    } else {
        0.0
    };
    Some(NeweyWestReport {
        alpha,
        beta,
        se_alpha_hac,
        se_beta_hac,
        t_stat_beta_hac: t_beta,
        se_alpha_ols,
        se_beta_ols,
        lag_truncation: l,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(estimate(&[1.0; 5], &[1.0; 5], None).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let x = vec![1.0_f64; 30];
        let mut y = vec![1.0_f64; 30];
        y[5] = f64::NAN;
        assert!(estimate(&x, &y, None).is_none());
    }

    #[test]
    fn flat_predictor_returns_none() {
        let x = vec![1.0_f64; 30];
        let y: Vec<f64> = (0..30).map(|i| i as f64).collect();
        assert!(estimate(&x, &y, None).is_none());
    }

    #[test]
    fn iid_residuals_hac_equals_ols_at_lag_zero() {
        let mut state: u64 = 42;
        let x: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|xi| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let eps = (state >> 32) as f64 / u32::MAX as f64 - 0.5;
                2.0 * xi + eps
            })
            .collect();
        let r = estimate(&x, &y, Some(0)).unwrap();
        let rel = (r.se_beta_hac - r.se_beta_ols).abs() / r.se_beta_ols;
        assert!(
            rel < 0.1,
            "HAC(0) should track OLS: HAC={}, OLS={}",
            r.se_beta_hac,
            r.se_beta_ols
        );
    }

    #[test]
    fn ar1_residuals_inflate_hac_se() {
        // Build residuals with strong positive AR(1) → HAC SE > OLS SE.
        let mut state: u64 = 11;
        let n = 500_usize;
        let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let mut e = vec![0.0_f64; n];
        for i in 1..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eta = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0;
            e[i] = 0.85 * e[i - 1] + eta;
        }
        let y: Vec<f64> = x
            .iter()
            .zip(e.iter())
            .map(|(xi, ei)| 2.0 * xi + ei)
            .collect();
        let r = estimate(&x, &y, Some(20)).unwrap();
        assert!(
            r.se_beta_hac > r.se_beta_ols,
            "AR(1) residuals: HAC SE {} should exceed OLS SE {}",
            r.se_beta_hac,
            r.se_beta_ols
        );
    }

    #[test]
    fn coefficients_match_ols() {
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi + 1.0).collect();
        let r = estimate(&x, &y, Some(2)).unwrap();
        assert!((r.beta - 2.0).abs() < 1e-9);
        assert!((r.alpha - 1.0).abs() < 1e-9);
    }

    #[test]
    fn t_statistic_finite() {
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi).collect();
        let r = estimate(&x, &y, None);
        // Perfect fit → SE = 0 → t = 0 by our convention (not NaN).
        if let Some(rep) = r {
            assert!(rep.t_stat_beta_hac.is_finite());
        }
    }

    #[test]
    fn lag_truncation_reported() {
        let x: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi).collect();
        let r = estimate(&x, &y, Some(7)).unwrap();
        assert_eq!(r.lag_truncation, 7);
    }
}
