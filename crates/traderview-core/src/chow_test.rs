//! Chow Test for Structural Break (Chow 1960).
//!
//! Tests whether a linear regression has the same coefficients across
//! two sub-samples (split at a candidate break date):
//!
//!   F = ((SSR_pooled − SSR_1 − SSR_2) / k)  /  ((SSR_1 + SSR_2) / (n − 2k))
//!
//! where:
//!   - SSR_pooled = sum of squared residuals from regression on full sample
//!   - SSR_1, SSR_2 = SSR from separate regressions on each sub-sample
//!   - k = number of parameters (intercept + slope, so 2 for univariate)
//!   - n = total observations
//!
//! Under H0 (no break): F ~ F(k, n − 2k).
//!
//! Use cases:
//!   - Detect regime change in beta (e.g. before / after a macro event)
//!   - Validate model stability over time
//!   - Test "after-event" excess return persistence
//!
//! Pure compute. Companion to `cusum`, `cointegration`, `vector_autoregression`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChowTestReport {
    pub f_statistic: f64,
    pub ssr_pooled: f64,
    pub ssr_segment_1: f64,
    pub ssr_segment_2: f64,
    pub degrees_of_freedom_numerator: f64,
    pub degrees_of_freedom_denominator: f64,
    pub n_segment_1: usize,
    pub n_segment_2: usize,
    pub reject_at_5pct: bool,
}

/// Univariate Chow test: y = α + β · x split at `break_index`.
/// `break_index` is the first index of the SECOND segment (so segment 1
/// is [0, break_index), segment 2 is [break_index, n)).
pub fn univariate(
    x: &[f64],
    y: &[f64],
    break_index: usize,
) -> Option<ChowTestReport> {
    let n = x.len();
    if n < 8 || y.len() != n { return None; }
    if break_index < 4 || break_index > n - 4 { return None; }
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let pooled_ssr = ols_ssr(x, y)?;
    let seg1_ssr = ols_ssr(&x[..break_index], &y[..break_index])?;
    let seg2_ssr = ols_ssr(&x[break_index..], &y[break_index..])?;
    let k = 2.0_f64;
    let n_f = n as f64;
    let num = (pooled_ssr - seg1_ssr - seg2_ssr) / k;
    let den = (seg1_ssr + seg2_ssr) / (n_f - 2.0 * k);
    if den <= 0.0 { return None; }
    let f_stat = num / den;
    // Critical value for F(2, n-4) at 5%; use χ²(2)/2 = 3.0 large-sample
    // approximation when dof_denom > 30.
    let crit_5pct = if n_f - 2.0 * k >= 30.0 { 3.00 } else { 3.89 };
    Some(ChowTestReport {
        f_statistic: f_stat,
        ssr_pooled: pooled_ssr,
        ssr_segment_1: seg1_ssr,
        ssr_segment_2: seg2_ssr,
        degrees_of_freedom_numerator: k,
        degrees_of_freedom_denominator: n_f - 2.0 * k,
        n_segment_1: break_index,
        n_segment_2: n - break_index,
        reject_at_5pct: f_stat > crit_5pct,
    })
}

fn ols_ssr(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len() as f64;
    if n < 3.0 { return None; }
    let x_mean: f64 = x.iter().sum::<f64>() / n;
    let y_mean: f64 = y.iter().sum::<f64>() / n;
    let mut sxx = 0.0_f64;
    let mut sxy = 0.0_f64;
    for i in 0..x.len() {
        let dx = x[i] - x_mean;
        sxx += dx * dx;
        sxy += dx * (y[i] - y_mean);
    }
    if sxx <= 0.0 { return None; }
    let beta = sxy / sxx;
    let alpha = y_mean - beta * x_mean;
    let mut ssr = 0.0_f64;
    for i in 0..x.len() {
        let resid = y[i] - alpha - beta * x[i];
        ssr += resid * resid;
    }
    Some(ssr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..20).map(|i| 2.0 * i as f64).collect();
        assert!(univariate(&x[..5], &y[..5], 2).is_none());
        assert!(univariate(&x, &y[..10], 5).is_none());
        assert!(univariate(&x, &y, 2).is_none());     // break too early
        assert!(univariate(&x, &y, 18).is_none());    // break too late
    }

    #[test]
    fn nan_input_returns_none() {
        let mut x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..20).map(|i| 2.0 * i as f64).collect();
        x[5] = f64::NAN;
        assert!(univariate(&x, &y, 10).is_none());
    }

    #[test]
    fn no_break_does_not_reject() {
        // Constant linear relation y = 2x. F should be very small.
        let mut state: u64 = 42;
        let x: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            2.0 * xi + eps
        }).collect();
        let r = univariate(&x, &y, 100).unwrap();
        assert!(!r.reject_at_5pct,
            "stable relation shouldn't reject, F={}", r.f_statistic);
    }

    #[test]
    fn structural_break_rejects() {
        // y = 2x for first half, y = 5x for second half.
        let mut state: u64 = 11;
        let x: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..200).map(|i| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            let xi = i as f64;
            let slope = if i < 100 { 2.0 } else { 5.0 };
            slope * xi + eps
        }).collect();
        let r = univariate(&x, &y, 100).unwrap();
        assert!(r.reject_at_5pct,
            "slope change should reject, F={}", r.f_statistic);
    }

    #[test]
    fn segment_sizes_reported_correctly() {
        // Use noisy y so OLS has nonzero SSR (otherwise denominator = 0).
        let mut state: u64 = 7;
        let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            2.0 * xi + eps
        }).collect();
        let r = univariate(&x, &y, 12).unwrap();
        assert_eq!(r.n_segment_1, 12);
        assert_eq!(r.n_segment_2, 18);
    }

    #[test]
    fn flat_predictor_in_segment_returns_none() {
        // Segment with no x variation makes OLS singular.
        let mut x = vec![1.0_f64; 12];
        x.extend((12..24).map(|i| i as f64));
        let y: Vec<f64> = x.iter().enumerate().map(|(i, _)| {
            if i < 12 { 5.0 } else { 2.0 * i as f64 }
        }).collect();
        assert!(univariate(&x, &y, 12).is_none());
    }

    #[test]
    fn ssr_pooled_at_least_as_large_as_sum_of_segment_ssr() {
        let mut state: u64 = 11;
        let x: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..200).map(|i| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            let slope = if i < 100 { 2.0 } else { 5.0 };
            slope * (i as f64) + eps
        }).collect();
        let r = univariate(&x, &y, 100).unwrap();
        let split_sum = r.ssr_segment_1 + r.ssr_segment_2;
        assert!(r.ssr_pooled >= split_sum - 1e-6,
            "pooled SSR {} should be >= split SSR sum {}", r.ssr_pooled, split_sum);
    }
}
