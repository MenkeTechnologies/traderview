//! Bipower Variation — Barndorff-Nielsen & Shephard (2004, 2006).
//!
//! Robust estimator of the continuous (jump-free) integrated variance:
//!
//!   BPV = (π/2) · Σ_{i=2..n} |r_i| · |r_{i−1}|
//!
//! Key property: in the presence of finite-activity jumps, RV inflates
//! but BPV stays consistent for the continuous component. The
//! difference (RV − BPV) is an estimate of the jump-variation
//! contribution.
//!
//! Jump test (Huang-Tauchen 2005, Andersen-Bollerslev-Diebold 2007):
//!
//!   z = √n · (RV − BPV) / (BPV · √(θ · max(1, TQ/BPV²)))
//!
//! Under H0 (no jumps), z ~ N(0, 1); large positive z = jump detected.
//! TQ (tripower quarticity) is the robust IQ estimator used to scale.
//!
//! Pure compute. Companion to `realized_volatility`,
//! `realized_semivariance`, `realized_higher_moments`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BipowerReport {
    pub realized_variance: f64,
    pub bipower_variation: f64,
    /// RV − BPV truncated at zero; estimate of jump variation contribution.
    pub jump_variation: f64,
    pub tripower_quarticity: f64,
    /// Huang-Tauchen z statistic; under H0 (no jumps) ~ N(0, 1).
    pub jump_test_z: f64,
    /// One-sided p-value (upper tail) of the jump test.
    pub jump_test_p_value: f64,
    pub n_observations: usize,
}

const MU1: f64 = 0.797_884_560_802_865_4; // √(2/π)
const THETA: f64 = std::f64::consts::PI * std::f64::consts::PI / 4.0 + std::f64::consts::PI - 5.0;

pub fn compute(returns: &[f64]) -> Option<BipowerReport> {
    if returns.len() < 4 {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n = returns.len();
    let n_f = n as f64;
    let rv: f64 = returns.iter().map(|x| x * x).sum();
    // BPV = μ₁⁻² · Σ |r_i| · |r_{i-1}|; μ₁⁻² = π/2.
    let bpv: f64 = (1.0 / (MU1 * MU1))
        * (1..n)
            .map(|i| returns[i].abs() * returns[i - 1].abs())
            .sum::<f64>();
    let jump = (rv - bpv).max(0.0);
    // Tripower Quarticity: TQ = n · μ_{4/3}⁻³ · Σ |r|^{4/3} · |r|^{4/3} · |r|^{4/3}
    // (with the standard 3-period overlapping product). Used as robust IQ.
    let mu_43 = 2.0_f64.powf(2.0 / 3.0) * gamma_7_6() / gamma_1_2();
    let mu_43_cubed_inv = 1.0 / mu_43.powi(3);
    let tq_sum: f64 = (2..n)
        .map(|i| {
            returns[i].abs().powf(4.0 / 3.0)
                * returns[i - 1].abs().powf(4.0 / 3.0)
                * returns[i - 2].abs().powf(4.0 / 3.0)
        })
        .sum();
    let tq = n_f * mu_43_cubed_inv * tq_sum;
    // Huang-Tauchen z statistic with TQ-based scaling.
    let scale_denom = (THETA * (1.0_f64).max(tq / (bpv * bpv))).max(0.0).sqrt();
    let z = if bpv > 0.0 && scale_denom > 0.0 {
        n_f.sqrt() * (rv - bpv) / bpv / scale_denom
    } else {
        0.0
    };
    let p_value = 1.0 - standard_normal_cdf(z);
    Some(BipowerReport {
        realized_variance: rv,
        bipower_variation: bpv,
        jump_variation: jump,
        tripower_quarticity: tq,
        jump_test_z: z,
        jump_test_p_value: p_value,
        n_observations: n,
    })
}

fn gamma_7_6() -> f64 {
    0.927_553_793_283_388_2
}
fn gamma_1_2() -> f64 {
    std::f64::consts::PI.sqrt()
}

fn standard_normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

// Abramowitz & Stegun 7.1.26 series approximation, ~1.5e-7 max error.
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

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01, -0.02]).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        assert!(compute(&[0.01, f64::NAN, 0.02, 0.01]).is_none());
    }

    #[test]
    fn no_jump_yields_bpv_close_to_rv() {
        // Smooth-ish path: small symmetric returns; BPV should track RV.
        let mut state: u64 = 12345;
        let r: Vec<f64> = (0..500)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u2 = (state >> 32) as f64 / u32::MAX as f64;
                (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos() * 0.01
            })
            .collect();
        let rep = compute(&r).unwrap();
        let rel_diff =
            (rep.realized_variance - rep.bipower_variation).abs() / rep.realized_variance;
        assert!(
            rel_diff < 0.30,
            "BPV should track RV in no-jump regime, rel diff = {rel_diff}"
        );
    }

    #[test]
    fn big_jump_inflates_rv_above_bpv() {
        // Constant small returns + one massive single-point jump.
        let mut r = vec![0.001_f64; 200];
        r[100] = 0.50;
        let rep = compute(&r).unwrap();
        assert!(rep.jump_variation > 0.0);
        assert!(
            rep.realized_variance > rep.bipower_variation,
            "RV ({}) should exceed BPV ({}) with jump",
            rep.realized_variance,
            rep.bipower_variation
        );
    }

    #[test]
    fn jump_test_z_positive_for_jumpy_path() {
        // Series with clear jump should yield positive Huang-Tauchen z.
        let mut r = vec![0.001_f64; 200];
        r[100] = 0.50;
        let rep = compute(&r).unwrap();
        assert!(rep.jump_test_z > 0.0);
    }

    #[test]
    fn flat_series_yields_zero_components() {
        let r = vec![0.0_f64; 100];
        let rep = compute(&r).unwrap();
        assert_eq!(rep.realized_variance, 0.0);
        assert_eq!(rep.bipower_variation, 0.0);
        assert_eq!(rep.jump_variation, 0.0);
    }

    #[test]
    fn p_value_in_unit_range() {
        let r: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        let rep = compute(&r).unwrap();
        assert!((0.0..=1.0).contains(&rep.jump_test_p_value));
    }
}
