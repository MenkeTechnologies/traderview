//! Cornish-Fisher expansion VaR — parametric VaR adjusted for the
//! sample skewness and (excess) kurtosis of returns.
//!
//! Standard parametric VaR assumes Gaussian returns. The Cornish-Fisher
//! correction shifts the z-quantile to account for higher-order moments:
//!
//!   z_cf = z + (z² − 1)·s/6 + (z³ − 3z)·k/24 − (2z³ − 5z)·s²/36
//!   VaR_cf = −(μ + σ · z_cf)
//!
//! where s = skewness, k = excess kurtosis (kurtosis − 3), z is the
//! standard-Gaussian α-quantile. The expansion is valid for "moderate"
//! deviations from normality (|s| ≲ 1, |k| ≲ 4); beyond that the
//! transformation can become non-monotonic. We flag that case.
//!
//! Pure compute. Reports both the unadjusted (Gaussian) and adjusted
//! VaR + the validity flag.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CornishFisherReport {
    pub mean: f64,
    pub stdev: f64,
    pub skewness: f64,
    pub excess_kurtosis: f64,
    pub z_alpha: f64,
    pub z_alpha_cf: f64,
    pub var_gaussian: f64,
    pub var_cornish_fisher: f64,
    pub n_observations: usize,
    /// True when the Cornish-Fisher transform stays monotonic across
    /// the relevant z range (no quantile crossing); false flags caller
    /// to fall back to historical VaR.
    pub valid: bool,
}

pub fn compute(returns: &[f64], alpha: f64) -> Option<CornishFisherReport> {
    if returns.is_empty() || !alpha.is_finite() || !(0.0..1.0).contains(&alpha) || alpha == 0.0 {
        return None;
    }
    let clean: Vec<f64> = returns.iter().copied().filter(|x| x.is_finite()).collect();
    let n = clean.len();
    if n < 4 {
        return None;
    }
    let n_f = n as f64;
    let mean = clean.iter().sum::<f64>() / n_f;
    let m2 = clean.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n_f;
    // Use a relative tolerance — float-accumulation of identical inputs
    // can leave a tiny positive m2 instead of exact zero. Treat any
    // standard-deviation below 10 ULP of |mean| as effectively flat.
    let stdev = m2.max(0.0).sqrt();
    let flat_threshold = mean.abs() * 1e-12 + f64::EPSILON;
    if stdev <= flat_threshold {
        return None;
    }
    // Sample skew & kurtosis (population moments).
    let m3 = clean.iter().map(|x| (x - mean).powi(3)).sum::<f64>() / n_f;
    let m4 = clean.iter().map(|x| (x - mean).powi(4)).sum::<f64>() / n_f;
    let skew = m3 / m2.powf(1.5);
    let kurt_excess = m4 / (m2 * m2) - 3.0;
    let z_alpha = inv_norm_cdf(alpha);
    let z_cf = z_alpha
        + (z_alpha.powi(2) - 1.0) * skew / 6.0
        + (z_alpha.powi(3) - 3.0 * z_alpha) * kurt_excess / 24.0
        - (2.0 * z_alpha.powi(3) - 5.0 * z_alpha) * skew * skew / 36.0;
    // Monotonicity check: a sufficient (not necessary) condition is that
    // the derivative dz_cf/dz > 0 over [−3, 3] — the practical-use range.
    let valid = monotone_check(skew, kurt_excess);
    Some(CornishFisherReport {
        mean,
        stdev,
        skewness: skew,
        excess_kurtosis: kurt_excess,
        z_alpha,
        z_alpha_cf: z_cf,
        var_gaussian: -(mean + stdev * z_alpha),
        var_cornish_fisher: -(mean + stdev * z_cf),
        n_observations: n,
        valid,
    })
}

fn monotone_check(skew: f64, kurt_excess: f64) -> bool {
    // dz_cf/dz = 1 + z·s/3 + (3z²−3)·k/24 − (6z²−5)·s²/36
    // Sample at z = −3, −2, 2, 3 — if any goes ≤ 0, flag invalid.
    let derivative = |z: f64| {
        1.0 + z * skew / 3.0 + (3.0 * z * z - 3.0) * kurt_excess / 24.0
            - (6.0 * z * z - 5.0) * skew * skew / 36.0
    };
    [-3.0_f64, -2.0, 2.0, 3.0]
        .iter()
        .all(|z| derivative(*z) > 0.0)
}

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
        assert!(compute(&[], 0.05).is_none());
        assert!(compute(&[0.01; 10], 0.0).is_none());
        assert!(compute(&[0.01; 10], 1.0).is_none());
        assert!(compute(&[0.01; 10], f64::NAN).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01; 3], 0.05).is_none());
    }

    #[test]
    fn flat_series_returns_none() {
        assert!(compute(&[0.01; 50], 0.05).is_none());
    }

    #[test]
    fn gaussian_returns_yield_cf_approximately_equal_to_gaussian() {
        // Symmetric, kurt-3 series → s ≈ 0, k_excess ≈ 0 → z_cf ≈ z → VaRs match.
        let mut state: u64 = 12345;
        let mut r = Vec::with_capacity(2_000);
        for _ in 0..1_000 {
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
            r.push(0.01 * z1);
            r.push(0.01 * z2);
        }
        let rep = compute(&r, 0.05).unwrap();
        let rel_err = (rep.var_cornish_fisher - rep.var_gaussian).abs() / rep.var_gaussian.abs();
        assert!(
            rel_err < 0.15,
            "Gaussian draws should give CF ≈ Gaussian VaR: cf={} gauss={}",
            rep.var_cornish_fisher,
            rep.var_gaussian
        );
    }

    #[test]
    fn negative_skew_shifts_z_cf_more_negative() {
        // Most direct verification: with negative skew, the CF-adjusted
        // 5th-percentile quantile (z_cf) sits LOWER than the Gaussian
        // z_alpha — that's the structural property the expansion exists
        // to express. Whether the resulting VaR is bigger or smaller in
        // absolute terms also depends on kurtosis, mean, and stdev; the
        // shift in z_cf is the test that actually pins down "skew matters".
        let mut state: u64 = 9999;
        let mut r = Vec::with_capacity(400);
        for _ in 0..400 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = (state >> 32) as f64 / u32::MAX as f64;
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u3 = (state >> 32) as f64 / u32::MAX as f64;
            let z = (-2.0 * u2.ln()).sqrt() * (2.0 * std::f64::consts::PI * u3).cos();
            r.push(if u < 0.10 {
                -0.03 + 0.005 * z
            } else {
                0.01 * z
            });
        }
        let rep = compute(&r, 0.05).unwrap();
        assert!(
            rep.skewness < -0.1,
            "expected negative skew, got {}",
            rep.skewness
        );
        assert!(
            rep.z_alpha_cf < rep.z_alpha,
            "negative-skew CF expansion should push z_cf below z_alpha: z_cf={} z={}",
            rep.z_alpha_cf,
            rep.z_alpha
        );
    }

    #[test]
    fn extreme_kurtosis_flags_invalid() {
        // Single huge outlier in otherwise zero series → enormous kurtosis,
        // breaking the monotone CF approximation.
        let mut r = vec![0.0_f64; 50];
        r[25] = 100.0;
        let rep = compute(&r, 0.05).unwrap();
        assert!(rep.excess_kurtosis > 10.0);
        assert!(!rep.valid, "huge kurtosis should flag invalid");
    }

    #[test]
    fn nan_returns_skipped_safely() {
        let mut r: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        r[10] = f64::NAN;
        let rep = compute(&r, 0.05).unwrap();
        assert_eq!(rep.n_observations, 99);
    }
}
