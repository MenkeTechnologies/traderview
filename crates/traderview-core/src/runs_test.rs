//! Wald-Wolfowitz Runs Test for Randomness.
//!
//! Given a binary or sign-coded sequence, counts the number of "runs"
//! (maximal subsequences of the same sign) and compares to the
//! expected distribution under H0 of iid randomness.
//!
//! For n_+ positives and n_− negatives (n = n_+ + n_−):
//!
//!   μ = (2·n_+·n_−) / n + 1
//!   σ² = (μ − 1) · (μ − 2) / (n − 1)
//!   z = (R − μ) / σ
//!
//! Under H0, z ~ N(0, 1) asymptotically.
//!
//! Sign coding: positive values count as +1, negative as −1, zero
//! values are dropped (Wald-Wolfowitz convention). Caller can pre-
//! transform (e.g. compare returns to median to test sign-balance).
//!
//! Use cases:
//!   - Test trading-rule signal generation for randomness
//!   - Test regression residual signs for structure
//!   - Independence test on simulation outputs
//!
//! Pure compute. Companion to `cusum`, `ljung_box`, `variance_ratio_test`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunsTestReport {
    pub n_positive: usize,
    pub n_negative: usize,
    pub n_runs: usize,
    pub expected_runs: f64,
    pub variance: f64,
    pub z_statistic: f64,
    pub p_value_two_sided: f64,
    pub reject_at_5pct: bool,
}

pub fn test(values: &[f64], threshold: f64) -> Option<RunsTestReport> {
    if values.is_empty() || !threshold.is_finite() { return None; }
    let signs: Vec<i8> = values.iter().filter_map(|v| {
        if !v.is_finite() { return None; }
        let d = v - threshold;
        if d > 0.0 { Some(1) }
        else if d < 0.0 { Some(-1) }
        else { None }
    }).collect();
    let n = signs.len();
    if n < 6 { return None; }
    let n_pos = signs.iter().filter(|s| **s == 1).count();
    let n_neg = n - n_pos;
    if n_pos == 0 || n_neg == 0 { return None; }
    let mut runs = 1_usize;
    for i in 1..n {
        if signs[i] != signs[i - 1] { runs += 1; }
    }
    let n_f = n as f64;
    let np_f = n_pos as f64;
    let nn_f = n_neg as f64;
    let mu = (2.0 * np_f * nn_f) / n_f + 1.0;
    let var = (mu - 1.0) * (mu - 2.0) / (n_f - 1.0);
    if var <= 0.0 { return None; }
    let z = (runs as f64 - mu) / var.sqrt();
    let p_two = 2.0 * (1.0 - standard_normal_cdf(z.abs())).clamp(0.0, 1.0);
    Some(RunsTestReport {
        n_positive: n_pos,
        n_negative: n_neg,
        n_runs: runs,
        expected_runs: mu,
        variance: var,
        z_statistic: z,
        p_value_two_sided: p_two,
        reject_at_5pct: z.abs() > 1.96,
    })
}

fn standard_normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let y = 1.0 - (((((1.061_405_429 * t - 1.453_152_027) * t)
        + 1.421_413_741) * t - 0.284_496_736) * t + 0.254_829_592) * t * (-x * x).exp();
    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(test(&[], 0.0).is_none());
    }

    #[test]
    fn nan_threshold_returns_none() {
        assert!(test(&[1.0, -1.0, 1.0], f64::NAN).is_none());
    }

    #[test]
    fn all_same_sign_returns_none() {
        let vals = vec![1.0_f64; 20];
        assert!(test(&vals, 0.0).is_none());
    }

    #[test]
    fn too_few_after_zero_drop_returns_none() {
        // All zeros → 0 signs after filter.
        let vals = vec![0.0_f64; 20];
        assert!(test(&vals, 0.0).is_none());
    }

    #[test]
    fn alternating_pattern_yields_low_p_value() {
        // +1, -1, +1, -1, ... → max number of runs → strongly anti-
        // random → small p-value (likely reject).
        let vals: Vec<f64> = (0..40).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
        let r = test(&vals, 0.0).unwrap();
        assert!(r.reject_at_5pct,
            "alternating pattern should reject randomness, z = {}", r.z_statistic);
        assert_eq!(r.n_runs, 40);
    }

    #[test]
    fn clustered_pattern_yields_low_p_value() {
        // First half +, second half - → only 2 runs → very few runs → reject.
        let mut vals = vec![1.0_f64; 30];
        vals.extend(vec![-1.0_f64; 30]);
        let r = test(&vals, 0.0).unwrap();
        assert_eq!(r.n_runs, 2);
        assert!(r.reject_at_5pct,
            "clustered pattern should reject randomness, z = {}", r.z_statistic);
    }

    #[test]
    fn random_sequence_does_not_reject() {
        let mut state: u64 = 42;
        let vals: Vec<f64> = (0..2000).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 32) as f64 / u32::MAX as f64) - 0.5
        }).collect();
        let r = test(&vals, 0.0).unwrap();
        assert!(!r.reject_at_5pct,
            "iid sequence shouldn't reject, z = {}", r.z_statistic);
    }

    #[test]
    fn zero_values_dropped_per_wald_wolfowitz() {
        let mut vals = vec![1.0_f64; 5];
        vals.extend([0.0; 10]);    // dropped
        vals.extend([-1.0_f64; 5]);
        let r = test(&vals, 0.0).unwrap();
        assert_eq!(r.n_positive, 5);
        assert_eq!(r.n_negative, 5);
        assert_eq!(r.n_runs, 2);
    }
}
