//! Kolmogorov-Smirnov Two-Sample Test.
//!
//! Tests the null that two independent samples were drawn from the
//! same continuous distribution. Test statistic:
//!
//!   D_{m,n} = sup_x |F̂_m(x) − Ĝ_n(x)|
//!
//! where F̂_m and Ĝ_n are the empirical CDFs of the two samples.
//!
//! Asymptotic p-value (Kolmogorov 1933 / Smirnov 1948):
//!
//!   z = D · √(m · n / (m + n))
//!   p ≈ 2 · Σ_{k=1}^∞ (−1)^{k−1} exp(−2 k² z²)
//!
//! The series converges quickly; ~6 terms gives machine precision.
//!
//! Use cases:
//!   - Test if out-of-sample returns come from the same distribution
//!     as in-sample (overfit detector)
//!   - Compare model-generated vs empirical return distributions
//!   - Compare two strategies' P&L distributions
//!
//! Pure compute. Companion to `jarque_bera`, `anderson_darling_normality`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ks2SampleReport {
    pub d_statistic: f64,
    pub p_value: f64,
    pub n_sample_a: usize,
    pub n_sample_b: usize,
    pub reject_at_5pct: bool,
}

pub fn test(sample_a: &[f64], sample_b: &[f64]) -> Option<Ks2SampleReport> {
    let m = sample_a.len();
    let n = sample_b.len();
    if m < 3 || n < 3 {
        return None;
    }
    if sample_a.iter().any(|x| !x.is_finite()) || sample_b.iter().any(|x| !x.is_finite()) {
        return None;
    }
    // Build the sorted union and walk both ECDFs in parallel.
    let mut a = sample_a.to_vec();
    let mut b = sample_b.to_vec();
    a.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    b.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    let mut i = 0_usize;
    let mut j = 0_usize;
    let mut d_max = 0.0_f64;
    let m_f = m as f64;
    let n_f = n as f64;
    while i < m && j < n {
        let ai = a[i];
        let bj = b[j];
        if ai <= bj {
            i += 1;
        }
        if bj <= ai {
            j += 1;
        }
        let f_a = i as f64 / m_f;
        let f_b = j as f64 / n_f;
        let d = (f_a - f_b).abs();
        if d > d_max {
            d_max = d;
        }
    }
    let z = d_max * (m_f * n_f / (m_f + n_f)).sqrt();
    let p_value = ks_p_value(z);
    let crit_5pct = 1.358 * ((m_f + n_f) / (m_f * n_f)).sqrt();
    Some(Ks2SampleReport {
        d_statistic: d_max,
        p_value,
        n_sample_a: m,
        n_sample_b: n,
        reject_at_5pct: d_max > crit_5pct,
    })
}

fn ks_p_value(z: f64) -> f64 {
    if z <= 0.0 {
        return 1.0;
    }
    // 2·Σ_{k=1..N} (-1)^{k-1} · exp(-2·k²·z²). 50 terms is overkill but cheap.
    let mut sum = 0.0_f64;
    let mut sign = 1.0_f64;
    for k in 1..=50 {
        let term = sign * (-2.0 * (k as f64).powi(2) * z * z).exp();
        sum += term;
        sign = -sign;
        if term.abs() < 1e-18 {
            break;
        }
    }
    (2.0 * sum).clamp(0.0, 1.0)
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
    fn too_short_returns_none() {
        assert!(test(&[1.0, 2.0], &[1.0, 2.0, 3.0]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(test(&[1.0, f64::NAN, 3.0], &[1.0, 2.0, 3.0]).is_none());
    }

    #[test]
    fn identical_distributions_do_not_reject() {
        let a = box_muller(500, 42, 1.0, 0.0);
        let b = box_muller(500, 13, 1.0, 0.0);
        let r = test(&a, &b).unwrap();
        assert!(
            !r.reject_at_5pct,
            "same distribution shouldn't reject, D = {}, p = {}",
            r.d_statistic, r.p_value
        );
    }

    #[test]
    fn shifted_distributions_reject() {
        let a = box_muller(500, 42, 1.0, 0.0);
        let b = box_muller(500, 13, 1.0, 1.5); // shifted by 1.5σ
        let r = test(&a, &b).unwrap();
        assert!(
            r.reject_at_5pct,
            "shifted distribution should reject, D = {}, p = {}",
            r.d_statistic, r.p_value
        );
    }

    #[test]
    fn different_scales_reject() {
        let a = box_muller(500, 42, 1.0, 0.0);
        let b = box_muller(500, 13, 3.0, 0.0); // 3× wider
        let r = test(&a, &b).unwrap();
        assert!(
            r.reject_at_5pct,
            "different scales should reject, D = {}",
            r.d_statistic
        );
    }

    #[test]
    fn d_statistic_in_unit_range() {
        let a = box_muller(100, 1, 1.0, 0.0);
        let b = box_muller(100, 2, 1.0, 5.0); // very different
        let r = test(&a, &b).unwrap();
        assert!(r.d_statistic >= 0.0 && r.d_statistic <= 1.0);
    }

    #[test]
    fn p_value_in_unit_range() {
        let a = box_muller(100, 1, 1.0, 0.0);
        let b = box_muller(100, 2, 1.0, 0.0);
        let r = test(&a, &b).unwrap();
        assert!((0.0..=1.0).contains(&r.p_value));
    }

    #[test]
    fn sample_sizes_reported_correctly() {
        let a = box_muller(50, 1, 1.0, 0.0);
        let b = box_muller(80, 2, 1.0, 0.0);
        let r = test(&a, &b).unwrap();
        assert_eq!(r.n_sample_a, 50);
        assert_eq!(r.n_sample_b, 80);
    }
}
