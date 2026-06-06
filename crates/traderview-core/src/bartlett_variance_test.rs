//! Bartlett's Test for Equality of Variances (1937).
//!
//! Classical test for H0: all k groups have equal variance. More
//! powerful than Levene when the data IS normal, but very sensitive
//! to departures from normality — use `levene_test` for finance data
//! with heavy tails.
//!
//!   χ² = ((N − k) · ln(σ²_pooled) − Σ_i (n_i − 1) · ln(σ²_i))
//!        / (1 + (1/(3·(k − 1))) · (Σ_i 1/(n_i − 1) − 1/(N − k)))
//!
//! Under H0, χ² ~ χ²(k − 1).
//!
//! Pure compute. Companion to `levene_test`, `breusch_pagan_test`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BartlettReport {
    pub chi_squared_statistic: f64,
    pub degrees_of_freedom: f64,
    pub p_value: f64,
    pub pooled_variance: f64,
    pub n_groups: usize,
    pub n_total: usize,
    pub reject_at_5pct: bool,
}

pub fn test(groups: &[Vec<f64>]) -> Option<BartlettReport> {
    let k = groups.len();
    if k < 2 {
        return None;
    }
    if groups
        .iter()
        .any(|g| g.len() < 2 || g.iter().any(|x| !x.is_finite()))
    {
        return None;
    }
    let n_total: usize = groups.iter().map(|g| g.len()).sum();
    if n_total <= k {
        return None;
    }
    // Per-group sample variance (Bessel-corrected).
    let mut variances = Vec::with_capacity(k);
    for g in groups {
        let n_g = g.len() as f64;
        let mean: f64 = g.iter().sum::<f64>() / n_g;
        let var: f64 = g.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_g - 1.0);
        if var <= 0.0 {
            return None;
        }
        variances.push(var);
    }
    let n_total_f = n_total as f64;
    let k_f = k as f64;
    let pooled: f64 = groups
        .iter()
        .zip(variances.iter())
        .map(|(g, v)| (g.len() - 1) as f64 * v)
        .sum::<f64>()
        / (n_total_f - k_f);
    if pooled <= 0.0 {
        return None;
    }
    let numerator: f64 = (n_total_f - k_f) * pooled.ln()
        - groups
            .iter()
            .zip(variances.iter())
            .map(|(g, v)| (g.len() - 1) as f64 * v.ln())
            .sum::<f64>();
    let inv_sum: f64 = groups.iter().map(|g| 1.0 / (g.len() - 1) as f64).sum();
    let correction = 1.0 + (1.0 / (3.0 * (k_f - 1.0))) * (inv_sum - 1.0 / (n_total_f - k_f));
    let chi_sq = numerator / correction;
    let dof = k_f - 1.0;
    let p_value = chi_squared_upper_tail(chi_sq, dof);
    let crit_5pct = chi_squared_5pct_critical(k - 1);
    Some(BartlettReport {
        chi_squared_statistic: chi_sq,
        degrees_of_freedom: dof,
        p_value,
        pooled_variance: pooled,
        n_groups: k,
        n_total,
        reject_at_5pct: chi_sq > crit_5pct,
    })
}

fn chi_squared_upper_tail(x: f64, k: f64) -> f64 {
    if x <= 0.0 || k <= 0.0 {
        return 1.0;
    }
    let z = ((x / k).powf(1.0 / 3.0) - (1.0 - 2.0 / (9.0 * k))) / (2.0 / (9.0 * k)).sqrt();
    1.0 - standard_normal_cdf(z)
}

fn chi_squared_5pct_critical(k: usize) -> f64 {
    match k {
        1 => 3.841,
        2 => 5.991,
        3 => 7.815,
        4 => 9.488,
        5 => 11.070,
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

    fn box_muller(n: usize, seed: u64, scale: f64) -> Vec<f64> {
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
                scale * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
            })
            .collect()
    }

    #[test]
    fn too_few_or_small_groups_return_none() {
        assert!(test(&[vec![1.0, 2.0, 3.0]]).is_none());
        assert!(test(&[vec![1.0], vec![2.0, 3.0]]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let g = vec![vec![1.0, f64::NAN, 3.0], vec![1.0, 2.0, 3.0]];
        assert!(test(&g).is_none());
    }

    #[test]
    fn equal_variance_does_not_reject() {
        let g1 = box_muller(200, 42, 1.0);
        let g2 = box_muller(200, 13, 1.0);
        let r = test(&[g1, g2]).unwrap();
        assert!(
            !r.reject_at_5pct,
            "equal variances shouldn't reject, χ² = {}",
            r.chi_squared_statistic
        );
    }

    #[test]
    fn different_variance_rejects() {
        let g1 = box_muller(200, 42, 1.0);
        let g2 = box_muller(200, 13, 5.0);
        let r = test(&[g1, g2]).unwrap();
        assert!(
            r.reject_at_5pct,
            "5x variance difference should reject, χ² = {}",
            r.chi_squared_statistic
        );
    }

    #[test]
    fn three_groups_supported() {
        let g1 = box_muller(100, 1, 1.0);
        let g2 = box_muller(100, 2, 1.0);
        let g3 = box_muller(100, 3, 1.0);
        let r = test(&[g1, g2, g3]).unwrap();
        assert_eq!(r.n_groups, 3);
        assert_eq!(r.n_total, 300);
        assert_eq!(r.degrees_of_freedom, 2.0);
    }

    #[test]
    fn p_value_in_unit_range() {
        let g1 = box_muller(50, 1, 1.0);
        let g2 = box_muller(50, 2, 1.0);
        let r = test(&[g1, g2]).unwrap();
        assert!((0.0..=1.0).contains(&r.p_value));
    }
}
