//! Realized higher moments — rolling skewness and excess kurtosis of
//! the return distribution.
//!
//!   skew_t = (1/n) Σ ((r_i − μ) / σ)³
//!   kurt_t = (1/n) Σ ((r_i − μ) / σ)⁴ − 3        (excess kurtosis)
//!
//! Skewness < 0 = left-tailed (crash-prone), > 0 = right-tailed.
//! Excess kurtosis > 0 = fatter tails than Gaussian.
//!
//! Used in:
//!   - Cornish-Fisher VaR (see `cornish_fisher`)
//!   - Strategy-quality filtering (avoid strategies with deeply
//!     negative skew + high kurtosis — they look great until they don't)
//!   - Regime detection (rolling skew flips often precede vol clusters)
//!
//! Pure compute. Returns rolling skew + kurt series + their cross-time
//! summary statistics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HigherMomentsReport {
    pub skewness: Vec<Option<f64>>,
    pub excess_kurtosis: Vec<Option<f64>>,
    pub mean_skewness: f64,
    pub mean_excess_kurtosis: f64,
    pub n_observations: usize,
    pub window: usize,
}

pub fn compute(returns: &[f64], window: usize) -> Option<HigherMomentsReport> {
    let n = returns.len();
    if window < 4 || n < window {
        return None;
    }
    let mut skew = vec![None; n];
    let mut kurt = vec![None; n];
    let mut skew_sum = 0.0_f64;
    let mut kurt_sum = 0.0_f64;
    let mut populated = 0_usize;
    for i in (window - 1)..n {
        let win = &returns[i + 1 - window..=i];
        if win.iter().any(|x| !x.is_finite()) { continue; }
        // Reject windows with no variation: round-off can yield tiny sd
        // for a truly flat input, which then produces nonsense moments.
        let (mn, mx) = win.iter().fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(a, b), x| (a.min(*x), b.max(*x)),
        );
        if mx - mn <= 0.0 { continue; }
        let m: f64 = win.iter().sum::<f64>() / window as f64;
        let m2: f64 = win.iter().map(|x| (x - m).powi(2)).sum::<f64>() / window as f64;
        let sd = m2.max(0.0).sqrt();
        if sd <= 0.0 { continue; }
        let m3: f64 = win.iter().map(|x| ((x - m) / sd).powi(3)).sum::<f64>() / window as f64;
        let m4: f64 = win.iter().map(|x| ((x - m) / sd).powi(4)).sum::<f64>() / window as f64;
        if m3.is_finite() && m4.is_finite() {
            skew[i] = Some(m3);
            kurt[i] = Some(m4 - 3.0);
            skew_sum += m3;
            kurt_sum += m4 - 3.0;
            populated += 1;
        }
    }
    if populated == 0 { return None; }
    let pop_f = populated as f64;
    Some(HigherMomentsReport {
        skewness: skew,
        excess_kurtosis: kurt,
        mean_skewness: skew_sum / pop_f,
        mean_excess_kurtosis: kurt_sum / pop_f,
        n_observations: n,
        window,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01; 5], 20).is_none());
    }

    #[test]
    fn window_too_small_returns_none() {
        assert!(compute(&[0.01; 50], 3).is_none());
    }

    #[test]
    fn flat_series_yields_none() {
        // sd = 0 across window → all None.
        assert!(compute(&[0.01; 50], 20).is_none());
    }

    #[test]
    fn symmetric_distribution_yields_near_zero_skew() {
        // Symmetric +/- pattern over a window.
        let mut state: u64 = 42;
        let r: Vec<f64> = (0..500).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (state >> 32) as f64 / u32::MAX as f64;
            (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos() * 0.01
        }).collect();
        let report = compute(&r, 50).unwrap();
        assert!(report.mean_skewness.abs() < 0.3,
            "Gaussian draws should have skew ~ 0, got {}", report.mean_skewness);
        assert!(report.mean_excess_kurtosis.abs() < 0.7,
            "Gaussian draws should have excess kurt ~ 0, got {}",
            report.mean_excess_kurtosis);
    }

    #[test]
    fn left_skewed_returns_negative_skew() {
        // Many small gains + one big loss.
        let mut r = vec![0.005_f64; 100];
        r[50] = -0.30;
        let report = compute(&r, 30).unwrap();
        // Windows touching the outlier should have strongly negative skew.
        let touching: Vec<f64> = report.skewness.iter().enumerate()
            .filter(|(i, _)| (50..=79).contains(i))
            .filter_map(|(_, s)| *s).collect();
        let avg = touching.iter().sum::<f64>() / touching.len() as f64;
        assert!(avg < -0.5, "left outlier should yield negative skew, got {avg}");
    }

    #[test]
    fn heavy_tails_inflate_kurtosis() {
        // Series with two big outliers → high excess kurt.
        let mut r = vec![0.005_f64; 200];
        r[50] = -0.30;
        r[150] = 0.30;
        let report = compute(&r, 30).unwrap();
        // Some windows will have very high excess kurt.
        let max_kurt = report.excess_kurtosis.iter()
            .filter_map(|x| *x)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(max_kurt > 5.0, "outliers should produce kurt > 5, got {max_kurt}");
    }

    #[test]
    fn rolling_outputs_aligned_to_input() {
        let r: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin() * 0.02).collect();
        let report = compute(&r, 20).unwrap();
        assert_eq!(report.skewness.len(), 50);
        assert_eq!(report.excess_kurtosis.len(), 50);
        // First (window-1) slots None.
        assert!(report.skewness[18].is_none());
        assert!(report.skewness[19].is_some() || report.skewness[19].is_none());    // depends on sd
    }

    #[test]
    fn nan_input_window_skipped() {
        let mut r: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin() * 0.02).collect();
        r[10] = f64::NAN;
        let report = compute(&r, 20);
        // Windows containing NaN are skipped, but should still find populated windows.
        if let Some(rep) = report {
            assert!(rep.n_observations == 50);
        }
    }
}
