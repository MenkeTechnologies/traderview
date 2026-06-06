//! Hansen-Lunde Noise-to-Signal Ratio (Hansen & Lunde 2006).
//!
//! Estimates the magnitude of microstructure noise in high-frequency
//! return data. The first-order return autocovariance under the
//! noise-plus-signal model is:
//!
//!   γ_1 = −σ_η²
//!
//! where σ_η² is the variance of the noise component. The noise-to-
//! signal ratio is:
//!
//!   NSR = σ_η² / IV ≈ −γ_1 / (γ_0 + 2·γ_1)
//!
//! where γ_0 = Σ r² is plain RV and γ_1 = Σ r_t · r_{t−1}. The
//! denominator is a coarse first-order correction for the noise-
//! induced bias in γ_0.
//!
//! Interpretation:
//!   - NSR ≈ 0 → noise-free high-frequency data (rare in practice)
//!   - NSR ≈ 0.05 → typical liquid-equity tick data
//!   - NSR > 0.20 → very noisy, use noise-robust estimators (TSRV,
//!     realized kernel, pre-averaging)
//!
//! Pure compute. Companion to `two_scales_realized_variance`,
//! `realized_kernel`, `realized_volatility`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoiseToSignalReport {
    pub noise_to_signal: f64,
    pub noise_variance_estimate: f64,
    pub integrated_variance_estimate: f64,
    pub gamma_0: f64,
    pub gamma_1: f64,
    pub n_returns: usize,
}

pub fn compute(returns: &[f64]) -> Option<NoiseToSignalReport> {
    let n = returns.len();
    if n < 30 {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let gamma_0: f64 = returns.iter().map(|r| r * r).sum();
    let gamma_1: f64 = (1..n).map(|t| returns[t] * returns[t - 1]).sum();
    let noise_var = (-gamma_1).max(0.0);
    let iv_estimate = (gamma_0 + 2.0 * gamma_1).max(0.0);
    if iv_estimate <= 0.0 {
        // All variance attributable to noise (γ_0 ≈ -2γ_1).
        return Some(NoiseToSignalReport {
            noise_to_signal: f64::INFINITY,
            noise_variance_estimate: noise_var,
            integrated_variance_estimate: 0.0,
            gamma_0,
            gamma_1,
            n_returns: n,
        });
    }
    let nsr = noise_var / iv_estimate;
    Some(NoiseToSignalReport {
        noise_to_signal: nsr,
        noise_variance_estimate: noise_var,
        integrated_variance_estimate: iv_estimate,
        gamma_0,
        gamma_1,
        n_returns: n,
    })
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
    fn too_short_returns_none() {
        assert!(compute(&[0.01; 20]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut r = vec![0.01_f64; 100];
        r[5] = f64::NAN;
        assert!(compute(&r).is_none());
    }

    #[test]
    fn clean_returns_low_nsr() {
        // Pure signal: NSR should be ~0 (γ_1 ≈ 0).
        let r = box_muller(2000, 42, 0.01);
        let result = compute(&r).unwrap();
        assert!(
            result.noise_to_signal < 0.05,
            "clean signal NSR should be small, got {}",
            result.noise_to_signal
        );
    }

    #[test]
    fn noisy_returns_high_nsr() {
        // Add MA(1) noise: observed = signal + (η_t - η_{t-1}).
        let mut state: u64 = 0xCAFE_BABE_DEAD_BEEF;
        let n = 2000_usize;
        let true_returns = box_muller(n, 42, 0.01);
        let noise: Vec<f64> = (0..=n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u2 = (state >> 32) as f64 / u32::MAX as f64;
                0.005 * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
            })
            .collect();
        let observed: Vec<f64> = true_returns
            .iter()
            .enumerate()
            .map(|(i, r)| r + noise[i + 1] - noise[i])
            .collect();
        let result = compute(&observed).unwrap();
        assert!(
            result.noise_to_signal > 0.05,
            "noisy signal NSR should be elevated, got {}",
            result.noise_to_signal
        );
    }

    #[test]
    fn gamma_1_negative_under_noise() {
        // Standard MA(1) noise → γ_1 < 0.
        let mut state: u64 = 0xCAFE_BABE_DEAD_BEEF;
        let n = 2000_usize;
        let true_returns = box_muller(n, 42, 0.01);
        let noise: Vec<f64> = (0..=n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u2 = (state >> 32) as f64 / u32::MAX as f64;
                0.005 * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
            })
            .collect();
        let observed: Vec<f64> = true_returns
            .iter()
            .enumerate()
            .map(|(i, r)| r + noise[i + 1] - noise[i])
            .collect();
        let result = compute(&observed).unwrap();
        assert!(result.gamma_1 < 0.0);
    }

    #[test]
    fn output_metadata_correct() {
        let r = box_muller(100, 7, 0.01);
        let result = compute(&r).unwrap();
        assert_eq!(result.n_returns, 100);
    }
}
