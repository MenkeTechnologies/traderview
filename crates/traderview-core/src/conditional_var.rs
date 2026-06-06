//! Conditional Value-at-Risk (CVaR) / Expected Shortfall (ES).
//!
//! Where standard VaR(α) gives the threshold loss exceeded with
//! probability α, CVaR(α) is the *expected* loss conditional on being
//! beyond that threshold:
//!
//!   VaR(α)  = −quantile(returns, α)
//!   CVaR(α) = −E[ R | R ≤ quantile(returns, α) ]
//!
//! CVaR is a coherent risk measure (Artzner et al. 1999) — VaR is not.
//! CVaR is the basis of regulatory capital under Basel-III FRTB and
//! the standard portfolio-optimization risk constraint in the post-
//! Rockafellar-Uryasev literature.
//!
//! Returns both historical (empirical-quantile) and parametric-Gaussian
//! flavors. Parametric uses analytical Gaussian ES:
//!
//!   ES_Gauss(α) = −(μ − σ · φ(z_α) / α)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct CvarReport {
    pub var_historical: f64,
    pub cvar_historical: f64,
    pub var_parametric: f64,
    pub cvar_parametric: f64,
    pub mean: f64,
    pub stdev: f64,
    pub n_observations: usize,
    pub n_tail_observations: usize,
}

pub fn compute(returns: &[f64], alpha: f64) -> Option<CvarReport> {
    if returns.is_empty() || !alpha.is_finite() || !(0.0..1.0).contains(&alpha) || alpha == 0.0 {
        return None;
    }
    // Filter to finite returns.
    let mut clean: Vec<f64> = returns.iter().copied().filter(|x| x.is_finite()).collect();
    let n = clean.len();
    if n < 2 {
        return None;
    }
    // Sort ascending (worst losses first).
    clean.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    // Historical VaR: −quantile at α.
    let k = ((n as f64) * alpha).ceil().max(1.0) as usize;
    let k = k.min(n); // safety
    let var_hist = -clean[k - 1];
    // Historical CVaR: mean of worst-k losses (signed: returns are losses
    // when negative, so CVaR is the negation of the mean tail return).
    let tail_sum: f64 = clean[..k].iter().sum();
    let cvar_hist = -tail_sum / k as f64;
    // Parametric Gaussian.
    let n_f = n as f64;
    let mean = clean.iter().sum::<f64>() / n_f;
    let var = clean.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_f - 1.0);
    let stdev = var.max(0.0).sqrt();
    let z_alpha = inv_norm_cdf(alpha);
    let pdf_z = (-0.5 * z_alpha * z_alpha).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let var_param = -(mean + stdev * z_alpha);
    let cvar_param = -(mean - stdev * pdf_z / alpha);
    Some(CvarReport {
        var_historical: var_hist,
        cvar_historical: cvar_hist,
        var_parametric: var_param,
        cvar_parametric: cvar_param,
        mean,
        stdev,
        n_observations: n,
        n_tail_observations: k,
    })
}

fn inv_norm_cdf(p: f64) -> f64 {
    // Acklam (max abs err ~1.15e-9 in body). For tail uses we accept it.
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
        assert!(compute(&[], 0.05).is_none());
        assert!(compute(&[0.01; 10], 0.0).is_none());
        assert!(compute(&[0.01; 10], 1.0).is_none());
        assert!(compute(&[0.01; 10], -0.05).is_none());
        assert!(compute(&[0.01; 10], f64::NAN).is_none());
    }

    #[test]
    fn all_nan_returns_none() {
        let r = vec![f64::NAN; 10];
        assert!(compute(&r, 0.05).is_none());
    }

    #[test]
    fn historical_cvar_is_mean_of_worst_k_losses() {
        // Known returns: sorted [-10, -8, -6, -4, -2, 0, 2, 4, 6, 8]
        // α = 0.20 → k = ceil(2.0) = 2 → worst 2 returns = [-10, -8].
        // VaR = -(-8) = 8.  CVaR = -((-10 + -8) / 2) = 9.
        let r = vec![-10.0, -8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0, 8.0];
        let report = compute(&r, 0.20).unwrap();
        assert!((report.var_historical - 8.0).abs() < 1e-9);
        assert!((report.cvar_historical - 9.0).abs() < 1e-9);
        assert_eq!(report.n_tail_observations, 2);
    }

    #[test]
    fn cvar_greater_than_or_equal_to_var() {
        // CVaR ≥ VaR always — by definition (mean of tail ≥ tail threshold).
        let r: Vec<f64> = (-50..50).map(|i| i as f64 * 0.01).collect();
        for alpha in [0.01, 0.05, 0.10, 0.25] {
            let report = compute(&r, alpha).unwrap();
            assert!(
                report.cvar_historical >= report.var_historical - 1e-9,
                "CVaR ({}) should be >= VaR ({}) at α={}",
                report.cvar_historical,
                report.var_historical,
                alpha
            );
            assert!(report.cvar_parametric >= report.var_parametric - 1e-9);
        }
    }

    #[test]
    fn gaussian_returns_yield_historical_close_to_parametric() {
        // Generate 5_000 pseudo-Gaussian via Box-Muller from LCG.
        let mut state: u64 = 42;
        let mut r = Vec::with_capacity(5_000);
        for _ in 0..2_500 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (state >> 32) as f64 / u32::MAX as f64;
            let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let z2 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).sin();
            r.push(0.0005 + 0.01 * z1);
            r.push(0.0005 + 0.01 * z2);
        }
        let report = compute(&r, 0.05).unwrap();
        // Historical and parametric should agree to within ~15% on Gaussian draws.
        let rel =
            (report.cvar_historical - report.cvar_parametric).abs() / report.cvar_parametric.abs();
        assert!(
            rel < 0.15,
            "CVaR_hist={} CVaR_param={} rel_err={}",
            report.cvar_historical,
            report.cvar_parametric,
            rel
        );
    }

    #[test]
    fn pure_gains_yield_negative_var() {
        let r = vec![0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.09, 0.10];
        let report = compute(&r, 0.10).unwrap();
        // No losses → VaR is negative (i.e. you "lose" nothing).
        assert!(report.var_historical < 0.0);
    }

    #[test]
    fn nan_returns_skipped_safely() {
        let r = vec![0.01, f64::NAN, -0.02, 0.03, -0.05];
        let report = compute(&r, 0.50).unwrap();
        assert_eq!(report.n_observations, 4);
    }
}
