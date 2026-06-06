//! Anderson-Darling Test for Normality (Stephens 1986).
//!
//! Like Jarque-Bera, tests whether a sample comes from a normal
//! distribution — but uses the full empirical CDF (vs JB's first 4
//! moments only), making it more powerful in the tails.
//!
//!   Z_i = Φ((x_{(i)} − x̄) / s)            i = 1..n (sorted)
//!   A² = −n − (1/n) · Σ (2i − 1) · [ln(Z_i) + ln(1 − Z_{n+1-i})]
//!
//! Stephens' small-sample correction (for sample-estimated mean+sd):
//!
//!   A²* = A² · (1 + 0.75/n + 2.25/n²)
//!
//! Critical values for normality (Stephens 1986):
//!   α = 0.10 → A²* > 0.631
//!   α = 0.05 → A²* > 0.752
//!   α = 0.025 → A²* > 0.873
//!   α = 0.01 → A²* > 1.035
//!
//! Use cases:
//!   - Sharper tail-weight detection than Jarque-Bera
//!   - Validate normality assumption for parametric VaR
//!   - Residual diagnostics
//!
//! Pure compute. Companion to `jarque_bera`, `kolmogorov_smirnov_2sample`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AndersonDarlingReport {
    pub a_squared: f64,
    pub a_squared_adjusted: f64,
    pub reject_at_5pct: bool,
    pub reject_at_1pct: bool,
    pub n_observations: usize,
}

pub fn test(sample: &[f64]) -> Option<AndersonDarlingReport> {
    let n = sample.len();
    if n < 8 {
        return None;
    }
    if sample.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let mut sorted = sample.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n_f = n as f64;
    let mean: f64 = sorted.iter().sum::<f64>() / n_f;
    let var: f64 = sorted.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_f - 1.0);
    if var <= 0.0 {
        return None;
    }
    let sd = var.sqrt();
    // Compute Φ((x_{(i)} − μ̂) / σ̂) for each.
    let phi: Vec<f64> = sorted
        .iter()
        .map(|x| standard_normal_cdf((x - mean) / sd).clamp(1e-12, 1.0 - 1e-12))
        .collect();
    let mut acc = 0.0_f64;
    for i in 0..n {
        let i_f = (i + 1) as f64;
        let z_i = phi[i];
        let z_ni = phi[n - 1 - i];
        acc += (2.0 * i_f - 1.0) * (z_i.ln() + (1.0 - z_ni).ln());
    }
    let a_sq = -n_f - acc / n_f;
    let a_sq_adj = a_sq * (1.0 + 0.75 / n_f + 2.25 / (n_f * n_f));
    Some(AndersonDarlingReport {
        a_squared: a_sq,
        a_squared_adjusted: a_sq_adj,
        reject_at_5pct: a_sq_adj > 0.752,
        reject_at_1pct: a_sq_adj > 1.035,
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
    fn too_short_returns_none() {
        assert!(test(&[0.0; 5]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(test(&[0.0, f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).is_none());
    }

    #[test]
    fn flat_returns_none() {
        assert!(test(&[1.0; 50]).is_none());
    }

    #[test]
    fn normal_sample_does_not_reject() {
        let s = box_muller(2000, 42);
        let r = test(&s).unwrap();
        assert!(
            !r.reject_at_1pct,
            "Gaussian shouldn't reject at 1%, A²* = {}",
            r.a_squared_adjusted
        );
    }

    #[test]
    fn skewed_sample_rejects() {
        // |z|: half-normal — strongly right-skewed.
        let s: Vec<f64> = box_muller(500, 7).into_iter().map(|z| z.abs()).collect();
        let r = test(&s).unwrap();
        assert!(
            r.reject_at_5pct,
            "skewed sample should reject, A²* = {}",
            r.a_squared_adjusted
        );
    }

    #[test]
    fn heavy_mixture_rejects() {
        // Gaussian mixture (90% N(0,1), 10% N(0,25)) → heavy tails.
        let mut state: u64 = 11;
        let s: Vec<f64> = (0..2000)
            .map(|_| {
                let u = {
                    state = state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    (state >> 32) as f64 / u32::MAX as f64
                };
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u2 = (state >> 32) as f64 / u32::MAX as f64;
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                if u < 0.1 {
                    z * 5.0
                } else {
                    z
                }
            })
            .collect();
        let r = test(&s).unwrap();
        assert!(
            r.reject_at_1pct,
            "heavy-tail mixture should reject at 1%, A²* = {}",
            r.a_squared_adjusted
        );
    }

    #[test]
    fn adjustment_factor_inflates_statistic_for_small_n() {
        // For n=20, the adjustment factor is (1 + 0.75/20 + 2.25/400) ≈ 1.043.
        let s = box_muller(20, 99);
        let r = test(&s).unwrap();
        assert!(r.a_squared_adjusted >= r.a_squared);
    }

    #[test]
    fn n_reported_correctly() {
        let s = box_muller(100, 3);
        let r = test(&s).unwrap();
        assert_eq!(r.n_observations, 100);
    }
}
