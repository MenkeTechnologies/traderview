//! Nelson-Siegel-Svensson (NSS) Yield Curve Fit.
//!
//! Extends the 4-parameter Nelson-Siegel model with a 2nd hump term,
//! giving 6 parameters total:
//!
//!   y(t) = β₀ + β₁ · (1 − e^(−t/τ₁)) / (t/τ₁)
//!        + β₂ · ((1 − e^(−t/τ₁)) / (t/τ₁) − e^(−t/τ₁))
//!        + β₃ · ((1 − e^(−t/τ₂)) / (t/τ₂) − e^(−t/τ₂))
//!
//! where:
//!   β₀ = long-run level
//!   β₁ = short-run vs long-run spread
//!   β₂ = first hump magnitude
//!   β₃ = second hump magnitude
//!   τ₁, τ₂ = decay parameters for the two humps (τ₂ > τ₁)
//!
//! This module fits β₀..β₃ via OLS given fixed τ₁, τ₂ (typical
//! practice: caller pre-selects τ values from a grid or uses a
//! non-linear outer loop).
//!
//! Pure compute. Companion to `nelson_siegel`, `yield_curve_bootstrap`,
//! `key_rate_duration`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CurvePoint {
    pub time_years: f64,
    pub yield_decimal: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NssFitReport {
    pub beta_0: f64,
    pub beta_1: f64,
    pub beta_2: f64,
    pub beta_3: f64,
    pub tau_1: f64,
    pub tau_2: f64,
    pub r_squared: f64,
    pub residual_std_error: f64,
    pub n_points: usize,
}

pub fn fit(points: &[CurvePoint], tau_1: f64, tau_2: f64) -> Option<NssFitReport> {
    let n = points.len();
    if n < 6 || !tau_1.is_finite() || tau_1 <= 0.0 || !tau_2.is_finite() || tau_2 <= tau_1 {
        return None;
    }
    if points
        .iter()
        .any(|p| !p.time_years.is_finite() || p.time_years <= 0.0 || !p.yield_decimal.is_finite())
    {
        return None;
    }
    // Design matrix columns:
    //   c_0 = 1
    //   c_1 = (1 − exp(−t/τ₁)) / (t/τ₁)
    //   c_2 = c_1 − exp(−t/τ₁)
    //   c_3 = (1 − exp(−t/τ₂)) / (t/τ₂) − exp(−t/τ₂)
    let p = 4_usize;
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    let mut sum_y = 0.0_f64;
    let mut sum_y_sq = 0.0_f64;
    for pt in points {
        let t = pt.time_years;
        let u1 = t / tau_1;
        let u2 = t / tau_2;
        let exp_u1 = (-u1).exp();
        let exp_u2 = (-u2).exp();
        let row = [
            1.0,
            (1.0 - exp_u1) / u1,
            (1.0 - exp_u1) / u1 - exp_u1,
            (1.0 - exp_u2) / u2 - exp_u2,
        ];
        let y = pt.yield_decimal;
        sum_y += y;
        sum_y_sq += y * y;
        for j in 0..p {
            xty[j] += row[j] * y;
            for k in 0..p {
                xtx[j][k] += row[j] * row[k];
            }
        }
    }
    let coef = solve_linear(&xtx, &xty)?;
    let n_f = n as f64;
    let y_mean = sum_y / n_f;
    let tss = sum_y_sq - n_f * y_mean.powi(2);
    let mut ssr = 0.0_f64;
    for pt in points {
        let t = pt.time_years;
        let u1 = t / tau_1;
        let u2 = t / tau_2;
        let exp_u1 = (-u1).exp();
        let exp_u2 = (-u2).exp();
        let yhat = coef[0]
            + coef[1] * (1.0 - exp_u1) / u1
            + coef[2] * ((1.0 - exp_u1) / u1 - exp_u1)
            + coef[3] * ((1.0 - exp_u2) / u2 - exp_u2);
        ssr += (pt.yield_decimal - yhat).powi(2);
    }
    let dof = (n - p) as f64;
    if dof <= 0.0 {
        return None;
    }
    let r_sq = if tss > 1e-18 { 1.0 - ssr / tss } else { 0.0 };
    let sigma = (ssr / dof).sqrt();
    Some(NssFitReport {
        beta_0: coef[0],
        beta_1: coef[1],
        beta_2: coef[2],
        beta_3: coef[3],
        tau_1,
        tau_2,
        r_squared: r_sq,
        residual_std_error: sigma,
        n_points: n,
    })
}

fn solve_linear(m: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let n = m.len();
    if n == 0 || y.len() != n {
        return None;
    }
    let mut aug = vec![vec![0.0_f64; n + 1]; n];
    for (i, row) in aug.iter_mut().enumerate() {
        for (j, slot) in row.iter_mut().enumerate().take(n) {
            *slot = m[i][j];
        }
        row[n] = y[i];
    }
    for i in 0..n {
        let mut pivot = i;
        for r in (i + 1)..n {
            if aug[r][i].abs() > aug[pivot][i].abs() {
                pivot = r;
            }
        }
        if aug[pivot][i].abs() < 1e-18 {
            return None;
        }
        aug.swap(i, pivot);
        let div = aug[i][i];
        for v in aug[i].iter_mut() {
            *v /= div;
        }
        for r in 0..n {
            if r == i {
                continue;
            }
            let f = aug[r][i];
            if f == 0.0 {
                continue;
            }
            let pivot_row = aug[i].clone();
            for (j, v) in aug[r].iter_mut().enumerate() {
                *v -= f * pivot_row[j];
            }
        }
    }
    Some((0..n).map(|i| aug[i][n]).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(t: f64, y: f64) -> CurvePoint {
        CurvePoint {
            time_years: t,
            yield_decimal: y,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        let pts = vec![p(0.5, 0.04); 6];
        assert!(fit(&pts, 0.0, 2.0).is_none());
        assert!(fit(&pts, 1.0, 1.0).is_none());
        assert!(fit(&pts, 1.0, 0.5).is_none());
        assert!(fit(&pts, f64::NAN, 2.0).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let bad = vec![
            p(0.5, 0.04),
            p(1.0, 0.045),
            p(2.0, 0.05),
            p(3.0, f64::NAN),
            p(5.0, 0.058),
            p(10.0, 0.06),
        ];
        assert!(fit(&bad, 1.0, 5.0).is_none());
    }

    #[test]
    fn too_few_points_returns_none() {
        let pts = vec![p(0.5, 0.04), p(1.0, 0.045)];
        assert!(fit(&pts, 1.0, 5.0).is_none());
    }

    #[test]
    fn nonpositive_time_returns_none() {
        let pts = vec![
            p(0.0, 0.04),
            p(1.0, 0.045),
            p(2.0, 0.05),
            p(3.0, 0.055),
            p(5.0, 0.058),
            p(10.0, 0.06),
        ];
        assert!(fit(&pts, 1.0, 5.0).is_none());
    }

    #[test]
    fn flat_yield_curve_yields_beta_0_only() {
        let pts: Vec<_> = vec![0.5, 1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 20.0, 30.0]
            .into_iter()
            .map(|t| p(t, 0.04))
            .collect();
        let r = fit(&pts, 1.0, 5.0).unwrap();
        assert!((r.beta_0 - 0.04).abs() < 0.005);
        // Other betas should be ~0 (or small).
        assert!(r.beta_1.abs() < 0.01);
        assert!(r.beta_2.abs() < 0.01);
        assert!(r.beta_3.abs() < 0.01);
    }

    #[test]
    fn upward_sloping_curve_yields_negative_beta_1() {
        // Yields ~3% short, ~6% long → β₁ should be negative (since
        // β₀ = long-run level and β₁ = short - long).
        let pts: Vec<_> = vec![
            (0.25, 0.030),
            (0.5, 0.032),
            (1.0, 0.035),
            (2.0, 0.040),
            (3.0, 0.045),
            (5.0, 0.050),
            (10.0, 0.055),
            (20.0, 0.060),
            (30.0, 0.062),
        ]
        .into_iter()
        .map(|(t, y)| p(t, y))
        .collect();
        let r = fit(&pts, 1.0, 5.0).unwrap();
        assert!(r.beta_1 < 0.0);
    }

    #[test]
    fn r_squared_high_on_smooth_curve() {
        let pts: Vec<_> = vec![
            (0.5, 0.030),
            (1.0, 0.033),
            (2.0, 0.038),
            (3.0, 0.042),
            (5.0, 0.048),
            (10.0, 0.055),
            (20.0, 0.058),
            (30.0, 0.060),
        ]
        .into_iter()
        .map(|(t, y)| p(t, y))
        .collect();
        let r = fit(&pts, 1.0, 5.0).unwrap();
        assert!(r.r_squared > 0.95);
    }

    #[test]
    fn n_points_reported() {
        let pts: Vec<_> = (1..=10)
            .map(|i| p(i as f64, 0.04 + i as f64 * 0.001))
            .collect();
        let r = fit(&pts, 1.0, 5.0).unwrap();
        assert_eq!(r.n_points, 10);
    }
}
