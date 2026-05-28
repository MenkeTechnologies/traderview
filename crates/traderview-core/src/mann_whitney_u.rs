//! Mann-Whitney U Test (Wilcoxon rank-sum test, 1947).
//!
//! Non-parametric test that two independent samples come from
//! populations with the same distribution (specifically, the same
//! median / stochastic equivalence). Robust to outliers and does not
//! require normality (unlike Student's t-test).
//!
//! Test statistic:
//!
//!   U_1 = R_1 − n_1·(n_1 + 1)/2
//!   U_2 = R_2 − n_2·(n_2 + 1)/2 = n_1·n_2 − U_1
//!
//! where R_i is the rank-sum of group i in the merged ranking.
//!
//! Under H0 with no ties:
//!   μ = n_1 · n_2 / 2
//!   σ² = n_1 · n_2 · (n_1 + n_2 + 1) / 12
//!   z = (U − μ) / σ
//!
//! For ties, σ² is corrected per Mann-Whitney's tie-adjusted formula.
//!
//! Use cases:
//!   - Compare two strategies' P&L distributions without assuming normality
//!   - Test win-rate / Sharpe equality across two regimes
//!   - Robust alternative to t-test on small / noisy samples
//!
//! Pure compute. Companion to `kolmogorov_smirnov_2sample`,
//! `wilcoxon_signed_rank`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MannWhitneyReport {
    pub u_statistic: f64,
    pub z_statistic: f64,
    pub p_value_two_sided: f64,
    pub n_sample_a: usize,
    pub n_sample_b: usize,
    pub reject_at_5pct: bool,
}

pub fn test(sample_a: &[f64], sample_b: &[f64]) -> Option<MannWhitneyReport> {
    let n1 = sample_a.len();
    let n2 = sample_b.len();
    if n1 < 4 || n2 < 4 { return None; }
    if sample_a.iter().any(|x| !x.is_finite())
        || sample_b.iter().any(|x| !x.is_finite()) { return None; }
    // Merge and rank with mid-rank tie handling.
    let mut combined: Vec<(f64, u8)> = sample_a.iter().map(|x| (*x, 0_u8)).collect();
    combined.extend(sample_b.iter().map(|x| (*x, 1_u8)));
    combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let n_total = combined.len();
    let mut ranks = vec![0.0_f64; n_total];
    let mut tie_correction = 0.0_f64;
    let mut i = 0;
    while i < n_total {
        let mut j = i;
        while j + 1 < n_total && combined[j + 1].0 == combined[i].0 { j += 1; }
        let mid_rank = (i + j) as f64 / 2.0 + 1.0;
        for slot in ranks.iter_mut().take(j + 1).skip(i) { *slot = mid_rank; }
        let tie_size = (j - i + 1) as f64;
        if tie_size > 1.0 {
            tie_correction += tie_size.powi(3) - tie_size;
        }
        i = j + 1;
    }
    let mut r1 = 0.0_f64;
    for k in 0..n_total {
        if combined[k].1 == 0 { r1 += ranks[k]; }
    }
    let n1_f = n1 as f64;
    let n2_f = n2 as f64;
    let u1 = r1 - n1_f * (n1_f + 1.0) / 2.0;
    let u_min = u1.min(n1_f * n2_f - u1);
    // Normal approximation with tie correction.
    let n_t = n1_f + n2_f;
    let mu = n1_f * n2_f / 2.0;
    let var_no_tie = n1_f * n2_f * (n_t + 1.0) / 12.0;
    let var = if tie_correction > 0.0 {
        let factor = 1.0 - tie_correction / (n_t * (n_t * n_t - 1.0));
        var_no_tie * factor
    } else { var_no_tie };
    if var <= 0.0 { return None; }
    let sigma = var.sqrt();
    let z = (u_min - mu) / sigma;
    let p_two = 2.0 * standard_normal_cdf(z).min(1.0 - standard_normal_cdf(z));
    let p_two = p_two.clamp(0.0, 1.0);
    Some(MannWhitneyReport {
        u_statistic: u_min,
        z_statistic: z,
        p_value_two_sided: p_two,
        n_sample_a: n1,
        n_sample_b: n2,
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

    fn box_muller(n: usize, seed: u64, scale: f64, mean: f64) -> Vec<f64> {
        let mut state = seed;
        (0..n).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (state >> 32) as f64 / u32::MAX as f64;
            mean + scale * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        }).collect()
    }

    #[test]
    fn too_short_returns_none() {
        assert!(test(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(test(&[1.0, f64::NAN, 3.0, 4.0], &[1.0, 2.0, 3.0, 4.0]).is_none());
    }

    #[test]
    fn identical_distributions_do_not_reject() {
        let a = box_muller(200, 42, 1.0, 0.0);
        let b = box_muller(200, 13, 1.0, 0.0);
        let r = test(&a, &b).unwrap();
        assert!(!r.reject_at_5pct,
            "same distribution shouldn't reject, z = {}, p = {}", r.z_statistic, r.p_value_two_sided);
    }

    #[test]
    fn shifted_distributions_reject() {
        let a = box_muller(200, 42, 1.0, 0.0);
        let b = box_muller(200, 13, 1.0, 1.0);
        let r = test(&a, &b).unwrap();
        assert!(r.reject_at_5pct,
            "shifted distribution should reject, z = {}", r.z_statistic);
    }

    #[test]
    fn ties_handled_correctly() {
        let a = vec![1.0, 2.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![2.0, 3.0, 3.0, 4.0, 4.0, 5.0];
        let r = test(&a, &b).unwrap();
        assert!(r.z_statistic.is_finite());
    }

    #[test]
    fn p_value_in_unit_range() {
        let a = box_muller(50, 1, 1.0, 0.0);
        let b = box_muller(50, 2, 1.0, 0.0);
        let r = test(&a, &b).unwrap();
        assert!((0.0..=1.0).contains(&r.p_value_two_sided));
    }

    #[test]
    fn sample_sizes_reported() {
        let a = box_muller(30, 1, 1.0, 0.0);
        let b = box_muller(50, 2, 1.0, 0.0);
        let r = test(&a, &b).unwrap();
        assert_eq!(r.n_sample_a, 30);
        assert_eq!(r.n_sample_b, 50);
    }
}
