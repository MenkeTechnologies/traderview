//! Levene's Test for Equality of Variances (1960).
//!
//! Tests H0: all k groups have equal variance. Less sensitive to
//! departures from normality than Bartlett's test, making it the
//! preferred choice for finance-style heavy-tailed data.
//!
//! Procedure (Brown-Forsythe modification, using median centering for
//! extra robustness):
//!
//!   z_{ij} = | x_{ij} − median_i |
//!   W = ((N − k) / (k − 1)) · Σ_i n_i · (z̄_i. − z̄..)²
//!                                 / Σ_i Σ_j (z_{ij} − z̄_i.)²
//!
//! Under H0, W ~ F(k − 1, N − k).
//!
//! Use cases:
//!   - Validate equal-variance assumption for ANOVA / t-test
//!   - Detect regime change in volatility across periods
//!   - Compare risk profiles of multiple strategies
//!
//! Pure compute. Companion to `mann_whitney_u`, `wilcoxon_signed_rank`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LeveneReport {
    pub w_statistic: f64,
    pub degrees_of_freedom_numerator: f64,
    pub degrees_of_freedom_denominator: f64,
    pub p_value: f64,
    pub n_groups: usize,
    pub n_total: usize,
    pub reject_at_5pct: bool,
}

pub fn test(groups: &[Vec<f64>]) -> Option<LeveneReport> {
    let k = groups.len();
    if k < 2 {
        return None;
    }
    if groups
        .iter()
        .any(|g| g.len() < 3 || g.iter().any(|x| !x.is_finite()))
    {
        return None;
    }
    let n_total: usize = groups.iter().map(|g| g.len()).sum();
    if n_total <= k {
        return None;
    }
    // Median-centered absolute deviations (Brown-Forsythe variant).
    let medians: Vec<f64> = groups.iter().map(|g| median(g)).collect();
    let z_groups: Vec<Vec<f64>> = groups
        .iter()
        .zip(medians.iter())
        .map(|(g, m)| g.iter().map(|x| (x - m).abs()).collect())
        .collect();
    let z_group_means: Vec<f64> = z_groups
        .iter()
        .map(|z| z.iter().sum::<f64>() / z.len() as f64)
        .collect();
    let z_overall_mean: f64 = z_groups.iter().flatten().sum::<f64>() / n_total as f64;
    let numerator: f64 = z_groups
        .iter()
        .zip(z_group_means.iter())
        .map(|(z, m)| z.len() as f64 * (m - z_overall_mean).powi(2))
        .sum();
    let denominator: f64 = z_groups
        .iter()
        .zip(z_group_means.iter())
        .map(|(z, m)| z.iter().map(|zi| (zi - m).powi(2)).sum::<f64>())
        .sum();
    if denominator <= 0.0 {
        return None;
    }
    let dof_num = (k - 1) as f64;
    let dof_den = (n_total - k) as f64;
    let w = (dof_den / dof_num) * (numerator / denominator);
    let p_value = f_distribution_upper_tail(w, dof_num, dof_den);
    // 5% critical value of F(k-1, N-k). Use rough approximation: for
    // moderate dofs, F crit ≈ 3.0 for (1, 30), 2.0 for (2, 30), etc.
    let crit = f_5pct_critical(dof_num, dof_den);
    Some(LeveneReport {
        w_statistic: w,
        degrees_of_freedom_numerator: dof_num,
        degrees_of_freedom_denominator: dof_den,
        p_value,
        n_groups: k,
        n_total,
        reject_at_5pct: w > crit,
    })
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    if n.is_multiple_of(2_usize) {
        0.5 * (sorted[n / 2 - 1] + sorted[n / 2])
    } else {
        sorted[n / 2]
    }
}

/// F upper-tail via χ²-approximation: F(d1, d2) ≈ χ²(d1) / d1 for
/// large d2; we use Wilson-Hilferty on χ²(d1·F) for a cheap p-value.
fn f_distribution_upper_tail(f: f64, d1: f64, d2: f64) -> f64 {
    if f <= 0.0 || d1 <= 0.0 || d2 <= 0.0 {
        return 1.0;
    }
    let chi_sq = d1 * f;
    let k = d1;
    let z = ((chi_sq / k).powf(1.0 / 3.0) - (1.0 - 2.0 / (9.0 * k))) / (2.0 / (9.0 * k)).sqrt();
    let _ = d2;
    1.0 - standard_normal_cdf(z)
}

fn f_5pct_critical(d1: f64, d2: f64) -> f64 {
    // Lookup-style approximations for common dof pairs; falls back to
    // an asymptotic formula for large dof.
    if d2 > 60.0 {
        // F crit ≈ χ²(d1) / d1 at 5% (large d2 limit).
        match d1 as usize {
            1 => 3.84,
            2 => 3.00,
            3 => 2.60,
            4 => 2.37,
            5 => 2.21,
            _ => 2.10,
        }
    } else if d1 == 1.0 {
        4.20
    } else if d1 == 2.0 {
        3.35
    } else {
        3.00
    }
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

    fn box_muller(n: usize, seed: u64, scale: f64, mean: f64) -> Vec<f64> {
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
                mean + scale * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
            })
            .collect()
    }

    #[test]
    fn too_few_groups_returns_none() {
        let g = vec![vec![1.0, 2.0, 3.0, 4.0]];
        assert!(test(&g).is_none());
    }

    #[test]
    fn small_group_returns_none() {
        let g = vec![vec![1.0, 2.0], vec![3.0, 4.0, 5.0]];
        assert!(test(&g).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let g = vec![
            vec![1.0, f64::NAN, 3.0, 4.0, 5.0],
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
        ];
        assert!(test(&g).is_none());
    }

    #[test]
    fn equal_variance_does_not_reject() {
        let g1 = box_muller(200, 42, 1.0, 0.0);
        let g2 = box_muller(200, 13, 1.0, 5.0); // different mean, same vol
        let r = test(&[g1, g2]).unwrap();
        assert!(
            !r.reject_at_5pct,
            "equal-variance groups shouldn't reject, W = {}",
            r.w_statistic
        );
    }

    #[test]
    fn different_variance_rejects() {
        let g1 = box_muller(200, 42, 1.0, 0.0);
        let g2 = box_muller(200, 13, 5.0, 0.0); // same mean, 5× variance
        let r = test(&[g1, g2]).unwrap();
        assert!(
            r.reject_at_5pct,
            "5x variance difference should reject, W = {}",
            r.w_statistic
        );
    }

    #[test]
    fn three_groups_supported() {
        let g1 = box_muller(100, 1, 1.0, 0.0);
        let g2 = box_muller(100, 2, 1.0, 0.0);
        let g3 = box_muller(100, 3, 1.0, 0.0);
        let r = test(&[g1, g2, g3]).unwrap();
        assert_eq!(r.n_groups, 3);
        assert_eq!(r.n_total, 300);
    }

    #[test]
    fn p_value_in_unit_range() {
        let g1 = box_muller(50, 1, 1.0, 0.0);
        let g2 = box_muller(50, 2, 1.0, 0.0);
        let r = test(&[g1, g2]).unwrap();
        assert!((0.0..=1.0).contains(&r.p_value));
    }
}
