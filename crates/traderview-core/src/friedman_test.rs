//! Friedman Non-Parametric Repeated-Measures Test (Friedman 1937).
//!
//! Tests whether k treatments applied to the same n subjects produce
//! different rank distributions. Ranks each row (subject) across
//! treatments, then compares column-wise rank sums.
//!
//!   χ²_F = (12 / (n·k·(k+1))) · Σ_j R_j² − 3·n·(k+1)
//!
//! Under H0 (treatments interchangeable), χ²_F ~ χ²(k − 1).
//!
//! Use cases:
//!   - Compare k forecasting models on the same n forecast dates
//!   - Compare k strategies' returns on the same n trading days
//!   - Distribution-free alternative to repeated-measures ANOVA
//!
//! Pure compute. Companion to `mann_whitney_u`, `wilcoxon_signed_rank`,
//! `kolmogorov_smirnov_2sample`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FriedmanReport {
    pub chi_squared_statistic: f64,
    pub degrees_of_freedom: f64,
    pub p_value: f64,
    pub rank_sums: Vec<f64>,
    pub n_subjects: usize,
    pub n_treatments: usize,
    pub reject_at_5pct: bool,
}

/// `data[i][j]` is subject i's value under treatment j.
pub fn test(data: &[Vec<f64>]) -> Option<FriedmanReport> {
    let n = data.len();
    if n < 3 { return None; }
    let k = data[0].len();
    if k < 2 { return None; }
    if data.iter().any(|row| row.len() != k
        || row.iter().any(|x| !x.is_finite())) { return None; }
    // Per-subject rank with mid-rank tie correction.
    let mut rank_sums = vec![0.0_f64; k];
    for row in data {
        let mut idx: Vec<usize> = (0..k).collect();
        idx.sort_by(|a, b| row[*a].partial_cmp(&row[*b]).unwrap_or(std::cmp::Ordering::Equal));
        let mut ranks = vec![0.0_f64; k];
        let mut i = 0;
        while i < k {
            let mut j = i;
            while j + 1 < k && row[idx[j + 1]] == row[idx[i]] { j += 1; }
            let mid = (i + j) as f64 / 2.0 + 1.0;
            for r in i..=j { ranks[idx[r]] = mid; }
            i = j + 1;
        }
        for (j, slot) in rank_sums.iter_mut().enumerate() { *slot += ranks[j]; }
    }
    let n_f = n as f64;
    let k_f = k as f64;
    let sum_sq: f64 = rank_sums.iter().map(|r| r * r).sum();
    let chi_sq = (12.0 / (n_f * k_f * (k_f + 1.0))) * sum_sq - 3.0 * n_f * (k_f + 1.0);
    let dof = k_f - 1.0;
    let p_value = chi_squared_upper_tail(chi_sq, dof);
    let crit = chi_squared_5pct_critical(k - 1);
    Some(FriedmanReport {
        chi_squared_statistic: chi_sq,
        degrees_of_freedom: dof,
        p_value,
        rank_sums,
        n_subjects: n,
        n_treatments: k,
        reject_at_5pct: chi_sq > crit,
    })
}

fn chi_squared_upper_tail(x: f64, k: f64) -> f64 {
    if x <= 0.0 || k <= 0.0 { return 1.0; }
    let z = ((x / k).powf(1.0 / 3.0) - (1.0 - 2.0 / (9.0 * k))) / (2.0 / (9.0 * k)).sqrt();
    1.0 - standard_normal_cdf(z)
}

fn chi_squared_5pct_critical(k: usize) -> f64 {
    match k {
        1 => 3.841, 2 => 5.991, 3 => 7.815, 4 => 9.488, 5 => 11.070,
        _ => k as f64 + 2.0 * (2.0 * k as f64).sqrt(),
    }
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
    fn too_few_subjects_or_treatments_return_none() {
        assert!(test(&[vec![1.0, 2.0, 3.0]]).is_none());
        assert!(test(&[vec![1.0], vec![2.0], vec![3.0]]).is_none());
    }

    #[test]
    fn ragged_or_nan_returns_none() {
        let bad = vec![vec![1.0, 2.0], vec![1.0]];
        assert!(test(&bad).is_none());
        let nan = vec![vec![1.0, f64::NAN], vec![2.0, 3.0], vec![4.0, 5.0]];
        assert!(test(&nan).is_none());
    }

    #[test]
    fn identical_treatments_do_not_reject() {
        // Each subject gives same value to all treatments → all ranks tied.
        let data = vec![vec![5.0; 3]; 10];
        let r = test(&data).unwrap();
        assert!(!r.reject_at_5pct);
    }

    #[test]
    fn one_treatment_consistently_best_rejects() {
        // Treatment 2 (index 2) always best for every subject.
        let data: Vec<Vec<f64>> = (1..=20).map(|i| {
            vec![i as f64, i as f64 + 0.5, i as f64 + 100.0]
        }).collect();
        let r = test(&data).unwrap();
        assert!(r.reject_at_5pct,
            "consistent winner should reject, χ² = {}", r.chi_squared_statistic);
        // Treatment 2 has highest rank-sum.
        let max_idx = r.rank_sums.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
        assert_eq!(max_idx, 2);
    }

    #[test]
    fn rank_sums_sum_to_n_times_k_plus_1_over_2() {
        let data: Vec<Vec<f64>> = (1..=10).map(|i| vec![i as f64, i as f64 * 2.0, i as f64 * 3.0]).collect();
        let r = test(&data).unwrap();
        let total: f64 = r.rank_sums.iter().sum();
        // Each row contributes ranks 1+2+3 = 6. n rows → 6n.
        assert!((total - 60.0).abs() < 1e-9);
    }

    #[test]
    fn p_value_in_unit_range() {
        let data: Vec<Vec<f64>> = (0..15).map(|i| {
            vec![(i as f64).sin(), (i as f64).cos(), (i as f64).tan()]
        }).collect();
        let r = test(&data).unwrap();
        assert!((0.0..=1.0).contains(&r.p_value));
    }
}
