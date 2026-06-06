//! Lo-MacKinlay Variance Ratio Test (1988).
//!
//! Tests the null that a series follows a random walk by comparing the
//! variance of k-period returns to k times the variance of single-
//! period returns:
//!
//!   VR(k) = Var(r_t + r_{t−1} + … + r_{t−k+1}) / (k · Var(r_t))
//!
//! Under H0 (random walk), VR(k) = 1 for all k.
//!
//! Lo-MacKinlay z-statistic (heteroskedasticity-robust variant):
//!
//!   z(k) = (VR(k) − 1) / √φ(k)
//!   φ(k) = Σ_{j=1..k−1}  (2·(k − j)/k)² · δ(j)
//!   δ(j) = T · Σ (r_t − μ)²(r_{t−j} − μ)² / (Σ (r_t − μ)²)²
//!
//! Under H0, z(k) ~ N(0, 1).
//!
//! Interpretation:
//!   - VR(k) > 1: positive serial correlation (momentum)
//!   - VR(k) < 1: negative serial correlation (mean reversion)
//!   - |z(k)| > 1.96 → reject random walk at 5%
//!
//! Pure compute. Companion to `adf_standalone`, `ljung_box`,
//! `hurst_exponent`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VarianceRatioReport {
    pub k: usize,
    pub variance_ratio: f64,
    pub z_statistic_robust: f64,
    pub p_value_two_sided: f64,
    pub reject_at_5pct: bool,
    pub n_observations: usize,
}

pub fn test(returns: &[f64], k: usize) -> Option<VarianceRatioReport> {
    let n = returns.len();
    if k < 2 || n < k + 4 {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    // Reject truly flat input: float roundoff can produce tiny sample
    // variance even when all values are identical.
    let (mn, mx) = returns
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), x| {
            (a.min(*x), b.max(*x))
        });
    if mx - mn <= 0.0 {
        return None;
    }
    let n_f = n as f64;
    let mu: f64 = returns.iter().sum::<f64>() / n_f;
    let var_1: f64 = returns.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / (n_f - 1.0);
    if var_1 <= 0.0 {
        return None;
    }
    // k-period overlapping returns: r^k_t = Σ_{i=0..k-1} r_{t-i}.
    let mut sum_sq = 0.0_f64;
    let mut count = 0_usize;
    for t in (k - 1)..n {
        let sum_k: f64 = (0..k).map(|i| returns[t - i]).sum();
        sum_sq += (sum_k - k as f64 * mu).powi(2);
        count += 1;
    }
    let var_k = sum_sq / count as f64;
    let vr = var_k / (k as f64 * var_1);
    // Heteroskedasticity-robust φ(k).
    let denom_sq: f64 = returns
        .iter()
        .map(|x| (x - mu).powi(2))
        .sum::<f64>()
        .powi(2);
    let mut phi = 0.0_f64;
    for j in 1..k {
        let w = (2.0 * (k - j) as f64 / k as f64).powi(2);
        let mut num = 0.0_f64;
        for t in j..n {
            num += (returns[t] - mu).powi(2) * (returns[t - j] - mu).powi(2);
        }
        let delta = n_f * num / denom_sq;
        phi += w * delta;
    }
    if phi <= 0.0 {
        return None;
    }
    let z = (vr - 1.0) / phi.sqrt();
    let p_two = 2.0 * (1.0 - standard_normal_cdf(z.abs())).clamp(0.0, 1.0);
    Some(VarianceRatioReport {
        k,
        variance_ratio: vr,
        z_statistic_robust: z,
        p_value_two_sided: p_two,
        reject_at_5pct: z.abs() > 1.96,
        n_observations: n,
    })
}

fn standard_normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let y = 1.0
        - (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736) * t
            + 0.254_829_592)
            * t
            * (-x * x).exp();
    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_muller(n: usize, seed: u64) -> Vec<f64> {
        let mut state = seed;
        (0..n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u2 = (state >> 32) as f64 / u32::MAX as f64;
                (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
            })
            .collect()
    }

    #[test]
    fn too_short_or_invalid_k_returns_none() {
        let r = vec![0.01_f64; 30];
        assert!(test(&r, 1).is_none());
        assert!(test(&[0.01_f64; 5], 4).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let mut r = vec![0.01_f64; 30];
        r[5] = f64::NAN;
        assert!(test(&r, 2).is_none());
    }

    #[test]
    fn flat_returns_none() {
        assert!(test(&[0.01_f64; 100], 4).is_none());
    }

    #[test]
    fn random_walk_yields_vr_near_one() {
        let r = box_muller(2000, 42);
        let result = test(&r, 4).unwrap();
        // VR(k) should be close to 1.
        assert!(
            (result.variance_ratio - 1.0).abs() < 0.20,
            "RW VR = {}, expected near 1",
            result.variance_ratio
        );
        assert!(
            !result.reject_at_5pct,
            "shouldn't reject RW, z = {}",
            result.z_statistic_robust
        );
    }

    #[test]
    fn positive_serial_correlation_vr_above_one() {
        // AR(1) with positive φ produces VR > 1.
        let mut state: u64 = 11;
        let phi = 0.4_f64;
        let mut r = vec![0.0_f64; 1000];
        for i in 1..1000 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0;
            r[i] = phi * r[i - 1] + eps;
        }
        let result = test(&r, 4).unwrap();
        assert!(
            result.variance_ratio > 1.0,
            "AR(1) with positive φ should have VR > 1, got {}",
            result.variance_ratio
        );
    }

    #[test]
    fn negative_serial_correlation_vr_below_one() {
        let mut state: u64 = 7;
        let phi = -0.4_f64;
        let mut r = vec![0.0_f64; 1000];
        for i in 1..1000 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0;
            r[i] = phi * r[i - 1] + eps;
        }
        let result = test(&r, 4).unwrap();
        assert!(
            result.variance_ratio < 1.0,
            "AR(1) with negative φ should have VR < 1, got {}",
            result.variance_ratio
        );
    }

    #[test]
    fn p_value_in_unit_range() {
        let r = box_muller(500, 99);
        let result = test(&r, 4).unwrap();
        assert!((0.0..=1.0).contains(&result.p_value_two_sided));
    }

    #[test]
    fn k_reported_correctly() {
        let r = box_muller(500, 3);
        let result = test(&r, 8).unwrap();
        assert_eq!(result.k, 8);
    }
}
