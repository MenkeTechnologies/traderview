//! Realized Kernel — Barndorff-Nielsen, Hansen, Lunde, Shephard (2008).
//!
//! Estimates integrated variance from noisy high-frequency returns
//! via a weighted sum of return autocovariances:
//!
//!   RK = γ_0 + Σ_{h=1..H} k(h / (H + 1)) · (γ_h + γ_{−h})
//!
//! where γ_h = Σ_t r_t · r_{t−h} is the h-th return autocovariance and
//! k(·) is a kernel weight function (Bartlett k(x) = 1 − x by default).
//!
//! The bandwidth H controls noise robustness vs efficiency:
//!   - Small H → close to plain RV (more biased by noise)
//!   - Large H → fully noise-robust but higher variance
//!
//! Default H = floor(0.3 · n^(2/3)) per Barndorff-Nielsen et al. when
//! not specified.
//!
//! Pure compute. Companion to `two_scales_realized_variance`,
//! `realized_volatility`, `bipower_variation`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelKind {
    Bartlett,
    Parzen,
    Tukey,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealizedKernelReport {
    pub realized_kernel: f64,
    pub bandwidth_h: usize,
    pub gamma_0: f64,
    pub n_returns: usize,
}

pub fn compute(
    returns: &[f64],
    bandwidth: Option<usize>,
    kernel: KernelKind,
) -> Option<RealizedKernelReport> {
    let n = returns.len();
    if n < 30 { return None; }
    if returns.iter().any(|x| !x.is_finite()) { return None; }
    let n_f = n as f64;
    let h = bandwidth.unwrap_or_else(|| {
        (0.3 * n_f.powf(2.0 / 3.0)).floor() as usize
    }).max(1).min(n - 1);
    let gamma_0: f64 = returns.iter().map(|r| r * r).sum();
    let mut rk = gamma_0;
    for lag in 1..=h {
        let weight = match kernel {
            KernelKind::Bartlett => bartlett_weight(lag as f64 / (h as f64 + 1.0)),
            KernelKind::Parzen => parzen_weight(lag as f64 / (h as f64 + 1.0)),
            KernelKind::Tukey => tukey_weight(lag as f64 / (h as f64 + 1.0)),
        };
        let gamma_h: f64 = (lag..n).map(|t| returns[t] * returns[t - lag]).sum();
        // Symmetric: γ_h = γ_{−h} for real-valued series → multiply by 2.
        rk += 2.0 * weight * gamma_h;
    }
    Some(RealizedKernelReport {
        realized_kernel: rk.max(0.0),
        bandwidth_h: h,
        gamma_0,
        n_returns: n,
    })
}

fn bartlett_weight(x: f64) -> f64 {
    (1.0 - x).max(0.0)
}

fn parzen_weight(x: f64) -> f64 {
    if x <= 0.5 {
        1.0 - 6.0 * x * x + 6.0 * x.abs().powi(3)
    } else if x <= 1.0 {
        2.0 * (1.0 - x.abs()).powi(3)
    } else { 0.0 }
}

fn tukey_weight(x: f64) -> f64 {
    if x.abs() < 1.0 {
        0.5 * (1.0 + (std::f64::consts::PI * x).cos())
    } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_muller(n: usize, seed: u64, scale: f64) -> Vec<f64> {
        let mut state = seed;
        (0..n).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (state >> 32) as f64 / u32::MAX as f64;
            scale * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        }).collect()
    }

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01; 20], None, KernelKind::Bartlett).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut r = vec![0.01_f64; 100];
        r[5] = f64::NAN;
        assert!(compute(&r, None, KernelKind::Bartlett).is_none());
    }

    #[test]
    fn clean_returns_kernel_near_rv() {
        // No noise → kernel should be close to RV (γ_0 dominates,
        // off-diagonal autocovariances near zero).
        let r = box_muller(500, 42, 0.01);
        let result = compute(&r, Some(10), KernelKind::Bartlett).unwrap();
        let rv: f64 = r.iter().map(|x| x * x).sum();
        let rel_diff = (result.realized_kernel - rv).abs() / rv;
        assert!(rel_diff < 0.5, "RK = {} vs RV = {}, rel diff {:.2}",
            result.realized_kernel, rv, rel_diff);
    }

    #[test]
    fn bartlett_kernel_weight_decreases_linearly() {
        assert!((bartlett_weight(0.0) - 1.0).abs() < 1e-12);
        assert!((bartlett_weight(0.5) - 0.5).abs() < 1e-12);
        assert!((bartlett_weight(1.0)).abs() < 1e-12);
    }

    #[test]
    fn parzen_kernel_positive_on_unit_interval() {
        for x in 0..=100 {
            let xf = x as f64 / 100.0;
            assert!(parzen_weight(xf) >= 0.0);
        }
    }

    #[test]
    fn tukey_kernel_zero_at_boundary() {
        assert!((tukey_weight(1.0)).abs() < 1e-12);
        assert!((tukey_weight(0.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn output_metadata_correct() {
        let r = box_muller(150, 7, 0.01);
        let result = compute(&r, Some(5), KernelKind::Bartlett).unwrap();
        assert_eq!(result.n_returns, 150);
        assert_eq!(result.bandwidth_h, 5);
    }

    #[test]
    fn parzen_and_bartlett_give_similar_results_on_clean_data() {
        let r = box_muller(500, 42, 0.01);
        let bartlett = compute(&r, Some(10), KernelKind::Bartlett).unwrap();
        let parzen = compute(&r, Some(10), KernelKind::Parzen).unwrap();
        let rel = (bartlett.realized_kernel - parzen.realized_kernel).abs()
            / bartlett.realized_kernel;
        assert!(rel < 0.5, "kernels disagree too much: {} vs {}",
            bartlett.realized_kernel, parzen.realized_kernel);
    }

    #[test]
    fn realized_kernel_floored_at_zero() {
        let r = vec![0.0_f64; 50];
        let result = compute(&r, Some(5), KernelKind::Bartlett).unwrap();
        assert_eq!(result.realized_kernel, 0.0);
    }
}
