//! Wilcoxon Signed-Rank Test (1945).
//!
//! Non-parametric test for whether the median of the differences
//! between paired observations is zero. Used when:
//!   - Two measurements per subject (before/after treatment)
//!   - Two strategies' P&L on the same days
//!   - Asset returns vs. benchmark returns aligned by date
//!
//! Procedure:
//!   1. d_i = x_i − y_i
//!   2. Drop zeros (Wilcoxon convention).
//!   3. Rank |d_i| with mid-ranks for ties.
//!   4. T_+ = Σ ranks of positive differences
//!   5. T_− = Σ ranks of negative differences
//!   6. Test statistic W = min(T_+, T_−)
//!
//! Normal approximation (for n ≥ 10 after dropping zeros):
//!   μ = n·(n+1)/4
//!   σ² = n·(n+1)·(2n+1)/24    (with tie correction)
//!   z = (W − μ) / σ
//!
//! Pure compute. Companion to `mann_whitney_u`, `kolmogorov_smirnov_2sample`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WilcoxonReport {
    pub w_statistic: f64,
    pub t_plus: f64,
    pub t_minus: f64,
    pub z_statistic: f64,
    pub p_value_two_sided: f64,
    pub n_pairs_used: usize,
    pub reject_at_5pct: bool,
}

pub fn test(sample_x: &[f64], sample_y: &[f64]) -> Option<WilcoxonReport> {
    if sample_x.is_empty() || sample_x.len() != sample_y.len() { return None; }
    if sample_x.iter().any(|v| !v.is_finite())
        || sample_y.iter().any(|v| !v.is_finite()) { return None; }
    // Pairwise differences; drop zeros.
    let diffs: Vec<f64> = sample_x.iter().zip(sample_y.iter())
        .map(|(x, y)| x - y).filter(|d| *d != 0.0).collect();
    let n = diffs.len();
    if n < 6 { return None; }
    // Rank |d| with mid-ranks for ties.
    let abs_diffs: Vec<f64> = diffs.iter().map(|d| d.abs()).collect();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|a, b| abs_diffs[*a].partial_cmp(&abs_diffs[*b])
        .unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0_f64; n];
    let mut tie_correction = 0.0_f64;
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && abs_diffs[idx[j + 1]] == abs_diffs[idx[i]] { j += 1; }
        let mid_rank = (i + j) as f64 / 2.0 + 1.0;
        for k in i..=j { ranks[idx[k]] = mid_rank; }
        let tie_size = (j - i + 1) as f64;
        if tie_size > 1.0 { tie_correction += tie_size.powi(3) - tie_size; }
        i = j + 1;
    }
    let mut t_plus = 0.0_f64;
    let mut t_minus = 0.0_f64;
    for k in 0..n {
        if diffs[k] > 0.0 { t_plus += ranks[k]; }
        else { t_minus += ranks[k]; }
    }
    let w = t_plus.min(t_minus);
    // Normal approximation with tie correction.
    let n_f = n as f64;
    let mu = n_f * (n_f + 1.0) / 4.0;
    let var = n_f * (n_f + 1.0) * (2.0 * n_f + 1.0) / 24.0 - tie_correction / 48.0;
    if var <= 0.0 { return None; }
    let sigma = var.sqrt();
    let z = (w - mu) / sigma;
    let p_two = 2.0 * standard_normal_cdf(z).min(1.0 - standard_normal_cdf(z));
    let p_two = p_two.clamp(0.0, 1.0);
    Some(WilcoxonReport {
        w_statistic: w,
        t_plus,
        t_minus,
        z_statistic: z,
        p_value_two_sided: p_two,
        n_pairs_used: n,
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
    fn empty_or_mismatched_returns_none() {
        assert!(test(&[], &[]).is_none());
        assert!(test(&[1.0, 2.0], &[3.0]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(test(&[1.0, f64::NAN, 3.0], &[1.0, 2.0, 3.0]).is_none());
    }

    #[test]
    fn all_zeros_after_diff_returns_none() {
        let s = vec![1.0_f64; 20];
        assert!(test(&s, &s).is_none());
    }

    #[test]
    fn no_systematic_difference_does_not_reject() {
        let x = box_muller(200, 42, 1.0, 0.0);
        let y = box_muller(200, 13, 1.0, 0.0);
        let r = test(&x, &y).unwrap();
        assert!(!r.reject_at_5pct,
            "no shift shouldn't reject, z = {}", r.z_statistic);
    }

    #[test]
    fn systematic_positive_shift_rejects() {
        let x = box_muller(200, 42, 1.0, 1.0);
        let y = box_muller(200, 13, 1.0, 0.0);
        let r = test(&x, &y).unwrap();
        assert!(r.reject_at_5pct,
            "x > y consistently → should reject, z = {}", r.z_statistic);
    }

    #[test]
    fn ties_handled_with_zeros_dropped() {
        // 12 pairs: 3 zero-pairs (dropped), 9 non-zero with several |d|
        // ties to exercise the tie-correction code path.
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let y = vec![1.0, 1.5, 3.5, 4.0, 4.5, 6.5, 7.5, 8.5, 9.5, 10.0, 11.5, 12.5];
        let r = test(&x, &y).unwrap();
        assert!(r.n_pairs_used >= 6);
        assert!(r.z_statistic.is_finite());
    }

    #[test]
    fn p_value_in_unit_range() {
        let x = box_muller(50, 1, 1.0, 0.0);
        let y = box_muller(50, 2, 1.0, 0.0);
        let r = test(&x, &y).unwrap();
        assert!((0.0..=1.0).contains(&r.p_value_two_sided));
    }
}
