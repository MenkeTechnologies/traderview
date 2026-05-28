//! Rolling Beta vs Benchmark.
//!
//! For each bar with sufficient lookback, computes the OLS slope of
//! asset returns on benchmark returns:
//!
//!   beta_t = cov(r_asset, r_bench) / var(r_bench)
//!
//! over the trailing `window` observations.
//!
//! Companion outputs:
//!   - rolling alpha (intercept from same regression, annualized)
//!   - rolling R² (% of variance explained by market)
//!
//! Pure compute. Companion to `factor_models`, `rolling_sharpe`,
//! `beta_shrinkage`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RollingBetaReport {
    pub rolling_beta: Vec<Option<f64>>,
    pub rolling_alpha_annualized: Vec<Option<f64>>,
    pub rolling_r_squared: Vec<Option<f64>>,
}

pub fn compute(
    asset_returns: &[f64],
    benchmark_returns: &[f64],
    window: usize,
    periods_per_year: f64,
) -> Option<RollingBetaReport> {
    let n = asset_returns.len();
    if n < window || benchmark_returns.len() != n
        || window < 5
        || !periods_per_year.is_finite() || periods_per_year <= 0.0 {
        return None;
    }
    if asset_returns.iter().any(|x| !x.is_finite())
        || benchmark_returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let mut beta = vec![None; n];
    let mut alpha_ann = vec![None; n];
    let mut r_sq = vec![None; n];
    let n_f = window as f64;
    for i in (window - 1)..n {
        let a = &asset_returns[i + 1 - window..=i];
        let b = &benchmark_returns[i + 1 - window..=i];
        let a_mean: f64 = a.iter().sum::<f64>() / n_f;
        let b_mean: f64 = b.iter().sum::<f64>() / n_f;
        let mut sxx = 0.0_f64;
        let mut sxy = 0.0_f64;
        let mut syy = 0.0_f64;
        for j in 0..window {
            let dx = b[j] - b_mean;
            let dy = a[j] - a_mean;
            sxx += dx * dx;
            sxy += dx * dy;
            syy += dy * dy;
        }
        if sxx <= 0.0 { continue; }
        let bb = sxy / sxx;
        let aa = a_mean - bb * b_mean;
        let r2 = if syy > 0.0 { (bb * bb * sxx / syy).clamp(0.0, 1.0) } else { 0.0 };
        beta[i] = Some(bb);
        alpha_ann[i] = Some(aa * periods_per_year);
        r_sq[i] = Some(r2);
    }
    Some(RollingBetaReport {
        rolling_beta: beta,
        rolling_alpha_annualized: alpha_ann,
        rolling_r_squared: r_sq,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_or_invalid_returns_none() {
        let r = vec![0.01_f64; 10];
        assert!(compute(&r, &r, 20, 252.0).is_none());
        assert!(compute(&r, &[0.01_f64; 5], 5, 252.0).is_none());
        assert!(compute(&r, &r, 1, 252.0).is_none());
        assert!(compute(&r, &r, 5, 0.0).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut a = vec![0.01_f64; 30];
        a[5] = f64::NAN;
        let b = vec![0.01_f64; 30];
        assert!(compute(&a, &b, 20, 252.0).is_none());
    }

    #[test]
    fn asset_equals_benchmark_yields_beta_one() {
        let mut state: u64 = 42;
        let b: Vec<f64> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.02
        }).collect();
        let r = compute(&b, &b, 60, 252.0).unwrap();
        for v in r.rolling_beta.iter().skip(59).flatten() {
            assert!((v - 1.0).abs() < 1e-9);
        }
        for v in r.rolling_r_squared.iter().skip(59).flatten() {
            assert!((v - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn high_leverage_asset_yields_beta_above_one() {
        let mut state: u64 = 11;
        let b: Vec<f64> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.02
        }).collect();
        let a: Vec<f64> = b.iter().map(|x| 2.0 * x).collect();
        let r = compute(&a, &b, 60, 252.0).unwrap();
        let last_beta = r.rolling_beta[199].unwrap();
        assert!((last_beta - 2.0).abs() < 1e-9);
    }

    #[test]
    fn output_lengths_match_input() {
        let a: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        let b = a.clone();
        let r = compute(&a, &b, 20, 252.0).unwrap();
        assert_eq!(r.rolling_beta.len(), 50);
        assert!(r.rolling_beta[18].is_none());
        assert!(r.rolling_beta[19].is_some());
    }
}
