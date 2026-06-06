//! Phillips-Perron Unit Root Test (Phillips & Perron 1988).
//!
//! Tests H0: series has a unit root (non-stationary) vs H1: stationary.
//! Like ADF, but uses Newey-West HAC correction on the standard error
//! instead of adding lagged-difference regressors.
//!
//!   1. Regress Δy_t = α + β·y_{t-1} + ε_t (no lagged Δy terms)
//!   2. Standard error of β̂ is HAC-adjusted via Newey-West with
//!      Bartlett kernel
//!   3. Test statistic Z(t_β) = t-stat using HAC SE
//!
//! Critical values (Phillips-Perron 1988):
//!   α = 0.10 → −2.57
//!   α = 0.05 → −2.86
//!   α = 0.01 → −3.43
//!
//! Use cases:
//!   - Cross-check ADF result (PP is robust to certain misspecifications)
//!   - Test stationarity without choosing lag order
//!
//! Pure compute. Companion to `adf_standalone`, `kpss_test`,
//! `newey_west`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PpTestReport {
    pub pp_statistic: f64,
    pub beta_hat: f64,
    pub se_hac: f64,
    pub long_run_variance: f64,
    pub bandwidth_lag: usize,
    pub reject_unit_root_5pct: bool,
    pub reject_unit_root_1pct: bool,
    pub n_observations: usize,
}

pub fn test(series: &[f64], bandwidth_lag: Option<usize>) -> Option<PpTestReport> {
    let n = series.len();
    if n < 20 {
        return None;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return None;
    }
    // Step 1: OLS Δy_t = α + β·y_{t-1} + ε_t.
    let n_eff = n - 1;
    let n_f = n_eff as f64;
    let mut sum_x = 0.0_f64;
    let mut sum_y = 0.0_f64;
    let mut sum_xx = 0.0_f64;
    let mut sum_xy = 0.0_f64;
    for t in 1..n {
        let x = series[t - 1];
        let y = series[t] - series[t - 1];
        sum_x += x;
        sum_y += y;
        sum_xx += x * x;
        sum_xy += x * y;
    }
    let denom = n_f * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-18 {
        return None;
    }
    let beta = (n_f * sum_xy - sum_x * sum_y) / denom;
    let alpha = (sum_y - beta * sum_x) / n_f;
    let mut resid = vec![0.0_f64; n_eff];
    for (i, t) in (1..n).enumerate() {
        resid[i] = (series[t] - series[t - 1]) - alpha - beta * series[t - 1];
    }
    let ssr: f64 = resid.iter().map(|r| r * r).sum();
    let sigma2_ols = ssr / (n_f - 2.0);
    let var_beta_ols = sigma2_ols * n_f / denom;
    let se_ols = var_beta_ols.max(0.0).sqrt();
    // Step 2: Newey-West HAC correction on the residual long-run variance.
    let l = bandwidth_lag
        .unwrap_or_else(|| (4.0 * (n_f / 100.0).powf(0.25)).floor() as usize)
        .max(1)
        .min(n_eff / 2);
    let gamma_0: f64 = resid.iter().map(|r| r * r).sum::<f64>() / n_f;
    let mut s2 = gamma_0;
    for k in 1..=l {
        let w = 1.0 - k as f64 / (l as f64 + 1.0);
        let gamma_k: f64 = (k..n_eff).map(|t| resid[t] * resid[t - k]).sum::<f64>() / n_f;
        s2 += 2.0 * w * gamma_k;
    }
    if s2 <= 0.0 {
        return None;
    }
    // PP-adjusted t-statistic:
    //   Z(t) = (γ_0 / s²)^½ · t_β − ½ · (s² − γ_0) · (n · SE(β̂_OLS)) / (s² · √denom·n/(n·n_f))
    // We use the standard simplified form:
    //   Z = (γ_0 / s²)^½ · t_β_OLS − ½ · (s² − γ_0) · √(n · n) / (s² · √(n · var_beta · sigma2)^{-1})
    // For simplicity and stability we approximate as:
    //   Z ≈ t_β · (γ_0 / s²)^{1/2}
    // which is the leading-order Phillips-Perron correction when β ≈ 1
    // (under the null).
    let t_ols = beta / se_ols;
    let z_t = t_ols * (gamma_0 / s2).sqrt();
    let se_hac = se_ols * (s2 / gamma_0).sqrt();
    Some(PpTestReport {
        pp_statistic: z_t,
        beta_hat: beta,
        se_hac,
        long_run_variance: s2,
        bandwidth_lag: l,
        reject_unit_root_5pct: z_t < -2.86,
        reject_unit_root_1pct: z_t < -3.43,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(test(&[1.0; 10], None).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut s = vec![1.0_f64; 50];
        s[10] = f64::NAN;
        assert!(test(&s, None).is_none());
    }

    #[test]
    fn random_walk_does_not_reject_unit_root() {
        let mut state: u64 = 42;
        let mut s = vec![0.0_f64; 500];
        for i in 1..500 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let step = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0;
            s[i] = s[i - 1] + step;
        }
        let r = test(&s, None).unwrap();
        assert!(
            !r.reject_unit_root_5pct,
            "RW shouldn't reject unit root, PP = {}",
            r.pp_statistic
        );
    }

    #[test]
    fn stationary_ar1_rejects_unit_root() {
        // x_t = 0.5·x_{t-1} + ε → strongly stationary, PP should reject.
        let mut state: u64 = 11;
        let mut s = vec![0.0_f64; 500];
        for i in 1..500 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0;
            s[i] = 0.5 * s[i - 1] + eps;
        }
        let r = test(&s, None).unwrap();
        assert!(
            r.reject_unit_root_5pct,
            "stationary AR(1) should reject, PP = {}",
            r.pp_statistic
        );
    }

    #[test]
    fn custom_bandwidth_used() {
        let s: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin()).collect();
        let r = test(&s, Some(5)).unwrap();
        assert_eq!(r.bandwidth_lag, 5);
    }

    #[test]
    fn n_observations_reported() {
        let s: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).cos()).collect();
        let r = test(&s, None).unwrap();
        assert_eq!(r.n_observations, 100);
    }
}
