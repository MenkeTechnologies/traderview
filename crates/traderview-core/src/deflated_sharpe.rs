//! Deflated Sharpe Ratio — Bailey & López de Prado (2014).
//!
//! Adjusts the observed Sharpe ratio for two well-documented biases:
//!   1. **Skewness and kurtosis of returns** (the Mertens 2002 SE).
//!   2. **Multiple testing** (the maximum-of-N trials effect from
//!      backtest overfitting — N strategies tried, best one reported).
//!
//! Probability that the strategy's *true* Sharpe is positive given the
//! observed sample SR, n_observations, skew, kurtosis, and the number
//! of independent trials (N_trials) used to find this strategy:
//!
//!   PSR_★ = Φ(z)
//!
//! where
//!   z = (SR_obs − SR_★) · √(n − 1) / √(1 − γ_3·SR_obs + (γ_4 − 1)/4·SR_obs²)
//!   SR_★ ≈ √(2 · ln(N_trials))  (expected max-of-N noise floor)
//!
//! Returns the deflated SR threshold (SR_★), the variance of the
//! observed SR, the z-score, and the probability that the true SR
//! exceeds SR_★ (the canonical "deflated p-value").
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DsrReport {
    pub observed_sharpe: f64,
    pub deflated_threshold_sharpe: f64,
    pub sharpe_variance: f64,
    pub z_score: f64,
    pub probability_true_sr_above_threshold: f64,
}

pub fn compute(
    observed_sharpe: f64,
    n_observations: usize,
    skewness: f64,
    kurtosis: f64,
    n_trials: usize,
) -> Option<DsrReport> {
    if !observed_sharpe.is_finite()
        || !skewness.is_finite()
        || !kurtosis.is_finite()
        || n_observations < 4
        || n_trials < 1
    {
        return None;
    }
    // Mertens SE for the observed Sharpe ratio.
    let n = n_observations as f64;
    let denom_inner = 1.0 - skewness * observed_sharpe
        + ((kurtosis - 1.0) / 4.0) * observed_sharpe * observed_sharpe;
    if !denom_inner.is_finite() || denom_inner <= 0.0 {
        return None;
    }
    let sr_var = denom_inner / (n - 1.0);
    let sr_se = sr_var.sqrt();
    // Expected max of N independent Sharpe estimates (Bailey & López de
    // Prado 2014 eq. 12) using the Euler-Mascheroni-augmented approx.
    let sr_star = if n_trials == 1 {
        0.0
    } else {
        let gamma_em = 0.577_215_664_901_532_8_f64;
        let ln_n = (n_trials as f64).ln();
        // (1 − γ)·Φ⁻¹(1 − 1/N) + γ·Φ⁻¹(1 − 1/(N·e)).
        let z1 = inv_norm_cdf(1.0 - 1.0 / n_trials as f64);
        let z2 = inv_norm_cdf(1.0 - 1.0 / (n_trials as f64 * std::f64::consts::E));
        // Conservative fallback: simple √(2 ln N) bound.
        let approx = (1.0 - gamma_em) * z1 + gamma_em * z2;
        if approx.is_finite() && approx > 0.0 {
            approx
        } else {
            (2.0 * ln_n).sqrt()
        }
    };
    let z = (observed_sharpe - sr_star) / sr_se;
    let prob = norm_cdf(z);
    Some(DsrReport {
        observed_sharpe,
        deflated_threshold_sharpe: sr_star,
        sharpe_variance: sr_var,
        z_score: z,
        probability_true_sr_above_threshold: prob,
    })
}

fn norm_cdf(x: f64) -> f64 {
    let a1 = 0.254829592_f64;
    let a2 = -0.284496736_f64;
    let a3 = 1.421413741_f64;
    let a4 = -1.453152027_f64;
    let a5 = 1.061405429_f64;
    let p = 0.3275911_f64;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let xa = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + p * xa);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-xa * xa).exp();
    0.5 * (1.0 + sign * y)
}

/// Inverse normal CDF via Acklam's rational approximation
/// (max abs error ≈ 1.15e-9 for p ∈ (0, 1) excluding tails).
fn inv_norm_cdf(p: f64) -> f64 {
    if !(0.0..=1.0).contains(&p) || !p.is_finite() {
        return f64::NAN;
    }
    if p == 0.0 {
        return f64::NEG_INFINITY;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }
    let plow = 0.02425;
    let phigh = 1.0 - plow;
    let a = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_153_46,
    ];
    let b = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    let c = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    let d = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    if p < plow {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= phigh {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(f64::NAN, 100, 0.0, 3.0, 1).is_none());
        assert!(compute(1.0, 3, 0.0, 3.0, 1).is_none());
        assert!(compute(1.0, 100, 0.0, 3.0, 0).is_none());
        assert!(compute(1.0, 100, f64::NAN, 3.0, 1).is_none());
    }

    #[test]
    fn single_trial_threshold_is_zero() {
        // With N_trials = 1, SR_★ = 0 (no max-of-N inflation).
        let r = compute(0.5, 252, 0.0, 3.0, 1).unwrap();
        assert_eq!(r.deflated_threshold_sharpe, 0.0);
    }

    #[test]
    fn multiple_trials_threshold_positive_and_increasing() {
        let r10 = compute(2.0, 252, 0.0, 3.0, 10).unwrap();
        let r100 = compute(2.0, 252, 0.0, 3.0, 100).unwrap();
        let r1000 = compute(2.0, 252, 0.0, 3.0, 1_000).unwrap();
        assert!(r10.deflated_threshold_sharpe > 0.0);
        assert!(r100.deflated_threshold_sharpe > r10.deflated_threshold_sharpe);
        assert!(r1000.deflated_threshold_sharpe > r100.deflated_threshold_sharpe);
    }

    #[test]
    fn high_observed_sharpe_yields_high_probability() {
        // SR_obs = 3.0 (impressive), Gaussian returns, 252 obs, single trial.
        let r = compute(3.0, 252, 0.0, 3.0, 1).unwrap();
        assert!(
            r.probability_true_sr_above_threshold > 0.99,
            "high SR should yield high p, got {}",
            r.probability_true_sr_above_threshold
        );
    }

    #[test]
    fn many_trials_can_deflate_high_sharpe_below_significance() {
        // SR_obs = 1.5, 252 obs, but 1_000_000 trials → SR_★ very large.
        let r = compute(1.5, 252, 0.0, 3.0, 1_000_000).unwrap();
        // After massive multi-test correction, the probability should drop.
        assert!(
            r.probability_true_sr_above_threshold < 0.20,
            "1M trials should deflate, got prob={}",
            r.probability_true_sr_above_threshold
        );
    }

    #[test]
    fn negative_skew_decreases_psr() {
        // Same SR, but a portfolio with negative skewness should be penalized
        // (left-tail risk inflates the SR's standard error).
        //
        // At large SR (≥ 1) the Mertens-SE adjustment for skew sign saturates
        // because z = SR/SE >> 3 in both cases, making Φ(z) ≈ 1 for either
        // sign. Use a small SR + short sample to keep z in a discriminating
        // range where Φ(·) hasn't bottomed/topped out.
        let r_pos = compute(0.3, 30, 0.5, 3.0, 1).unwrap();
        let r_neg = compute(0.3, 30, -0.5, 3.0, 1).unwrap();
        assert!(
            r_neg.probability_true_sr_above_threshold < r_pos.probability_true_sr_above_threshold,
            "negative skew should lower PSR: pos={} neg={}",
            r_pos.probability_true_sr_above_threshold,
            r_neg.probability_true_sr_above_threshold
        );
    }

    #[test]
    fn fat_tails_increase_sr_variance() {
        let r_gauss = compute(1.5, 252, 0.0, 3.0, 1).unwrap();
        let r_fat = compute(1.5, 252, 0.0, 8.0, 1).unwrap();
        assert!(r_fat.sharpe_variance > r_gauss.sharpe_variance);
    }

    #[test]
    fn inv_norm_basic() {
        assert!((inv_norm_cdf(0.5)).abs() < 1e-8);
        assert!((inv_norm_cdf(0.975) - 1.959964).abs() < 1e-4);
    }
}
