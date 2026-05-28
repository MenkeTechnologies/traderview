//! Diebold-Mariano Forecast Accuracy Comparison Test (1995).
//!
//! Tests the null that two competing forecasts have equal expected
//! loss against the alternative that their losses differ:
//!
//!   d_t = L(e_{1,t}) − L(e_{2,t})
//!   DM = mean(d) / SE_HAC(d)
//!
//! where L is typically the squared-error loss L(e) = e². Under H0,
//! DM ~ N(0, 1) asymptotically (or Student-t for small samples).
//!
//! Use cases:
//!   - Compare two volatility forecasts
//!   - Compare ML vs benchmark VaR forecasts
//!   - Choose between competing factor models in OOS evaluation
//!
//! For h-step-ahead forecasts, the standard error uses h-1 Newey-West
//! lags. Default h = 1 (one-step-ahead).
//!
//! Harvey-Leybourne-Newbold (1997) small-sample correction:
//!
//!   DM* = DM · √((n + 1 − 2h + h(h−1)/n) / n)
//!
//! Pure compute. Companion to `newey_west`, `model_confidence_set`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LossFunction { SquaredError, AbsoluteError }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DieboldMarianoReport {
    pub dm_statistic: f64,
    pub dm_harvey_leybourne_newbold: f64,
    pub p_value_two_sided: f64,
    pub mean_loss_differential: f64,
    pub n_forecasts: usize,
    pub forecast_horizon: usize,
    pub favors_forecast_one: bool,
}

pub fn test(
    forecast_errors_1: &[f64],
    forecast_errors_2: &[f64],
    loss: LossFunction,
    horizon: usize,
) -> Option<DieboldMarianoReport> {
    let n = forecast_errors_1.len();
    if n < 10 || forecast_errors_2.len() != n || horizon == 0 { return None; }
    if forecast_errors_1.iter().any(|x| !x.is_finite())
        || forecast_errors_2.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let d: Vec<f64> = forecast_errors_1.iter().zip(forecast_errors_2.iter()).map(|(e1, e2)| {
        match loss {
            LossFunction::SquaredError => e1.powi(2) - e2.powi(2),
            LossFunction::AbsoluteError => e1.abs() - e2.abs(),
        }
    }).collect();
    let n_f = n as f64;
    let mean_d: f64 = d.iter().sum::<f64>() / n_f;
    // HAC variance estimate with Bartlett kernel; lag = h - 1 (no
    // autocorrelation expected past forecast horizon).
    let lag = horizon - 1;
    let gamma_0: f64 = d.iter().map(|x| (x - mean_d).powi(2)).sum::<f64>() / n_f;
    let mut var_hac = gamma_0;
    for k in 1..=lag {
        let w = 1.0 - k as f64 / (lag as f64 + 1.0);
        let gamma_k: f64 = (k..n).map(|t| (d[t] - mean_d) * (d[t - k] - mean_d))
            .sum::<f64>() / n_f;
        var_hac += 2.0 * w * gamma_k;
    }
    // Degenerate case: zero loss differential AND zero variance — the
    // two forecasts are identical. Report DM = 0 instead of None.
    if var_hac <= 0.0 {
        if mean_d.abs() < 1e-18 {
            return Some(DieboldMarianoReport {
                dm_statistic: 0.0,
                dm_harvey_leybourne_newbold: 0.0,
                p_value_two_sided: 1.0,
                mean_loss_differential: 0.0,
                n_forecasts: n,
                forecast_horizon: horizon,
                favors_forecast_one: false,
            });
        }
        return None;
    }
    let se = (var_hac / n_f).sqrt();
    let dm = mean_d / se;
    // Harvey-Leybourne-Newbold small-sample correction.
    let h = horizon as f64;
    let correction = ((n_f + 1.0 - 2.0 * h + h * (h - 1.0) / n_f) / n_f).max(0.0).sqrt();
    let dm_hln = dm * correction;
    let p_two = 2.0 * (1.0 - standard_normal_cdf(dm_hln.abs())).clamp(0.0, 1.0);
    Some(DieboldMarianoReport {
        dm_statistic: dm,
        dm_harvey_leybourne_newbold: dm_hln,
        p_value_two_sided: p_two,
        mean_loss_differential: mean_d,
        n_forecasts: n,
        forecast_horizon: horizon,
        favors_forecast_one: mean_d < 0.0,
    })
}

fn standard_normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let y = 1.0 - (((((1.061_405_429 * t - 1.453_152_027) * t)
        + 1.421_413_741) * t - 0.284_496_736) * t + 0.254_829_592) * t * (-x * x).exp();
    sign * y
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
        let e1 = vec![0.01; 5];
        let e2 = vec![0.02; 5];
        assert!(test(&e1, &e2, LossFunction::SquaredError, 1).is_none());
    }

    #[test]
    fn invalid_horizon_returns_none() {
        let e1 = vec![0.01; 50];
        let e2 = vec![0.02; 50];
        assert!(test(&e1, &e2, LossFunction::SquaredError, 0).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut e1 = vec![0.01_f64; 50];
        e1[10] = f64::NAN;
        let e2 = vec![0.02; 50];
        assert!(test(&e1, &e2, LossFunction::SquaredError, 1).is_none());
    }

    #[test]
    fn identical_forecasts_yield_zero_dm() {
        let e = box_muller(200, 42, 0.01);
        let r = test(&e, &e, LossFunction::SquaredError, 1).unwrap();
        assert!(r.dm_statistic.abs() < 1e-9);
    }

    #[test]
    fn worse_forecast_one_yields_positive_dm() {
        // Forecast 1 has higher errors → mean_d > 0 → DM > 0.
        let e1 = box_muller(300, 42, 0.05);
        let e2 = box_muller(300, 13, 0.01);
        let r = test(&e1, &e2, LossFunction::SquaredError, 1).unwrap();
        assert!(r.dm_statistic > 0.0);
        assert!(!r.favors_forecast_one);
    }

    #[test]
    fn worse_forecast_two_yields_negative_dm() {
        let e1 = box_muller(300, 42, 0.01);
        let e2 = box_muller(300, 13, 0.05);
        let r = test(&e1, &e2, LossFunction::SquaredError, 1).unwrap();
        assert!(r.dm_statistic < 0.0);
        assert!(r.favors_forecast_one);
    }

    #[test]
    fn absolute_error_loss_supported() {
        let e1 = box_muller(100, 1, 0.01);
        let e2 = box_muller(100, 2, 0.01);
        let r = test(&e1, &e2, LossFunction::AbsoluteError, 1).unwrap();
        assert!(r.dm_statistic.is_finite());
    }

    #[test]
    fn hln_correction_smaller_than_raw_dm() {
        let e1 = box_muller(50, 1, 0.05);
        let e2 = box_muller(50, 2, 0.01);
        let r = test(&e1, &e2, LossFunction::SquaredError, 5).unwrap();
        // HLN correction for small n with horizon > 1 should shrink DM.
        assert!(r.dm_harvey_leybourne_newbold.abs() <= r.dm_statistic.abs());
    }

    #[test]
    fn p_value_in_unit_range() {
        let e1 = box_muller(50, 1, 0.01);
        let e2 = box_muller(50, 2, 0.01);
        let r = test(&e1, &e2, LossFunction::SquaredError, 1).unwrap();
        assert!((0.0..=1.0).contains(&r.p_value_two_sided));
    }
}
