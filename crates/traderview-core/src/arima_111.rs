//! ARIMA(1, 1, 1) — Box-Jenkins integrated ARMA model.
//!
//! Model in differenced series Δy_t = y_t − y_{t−1}:
//!
//!   Δy_t = c + φ · Δy_{t−1} + θ · ε_{t−1} + ε_t
//!
//! Returns the (c, φ, θ) coefficients, residual variance, and one-step
//! ahead forecast in LEVEL space (y_{T+1} = y_T + forecast(Δy_{T+1})).
//!
//! Estimation: conditional least-squares on (Δy_t) regressed on
//! (Δy_{t-1}, ε_{t-1}), where ε_t is built iteratively from past
//! residuals. Simpler than maximum-likelihood (no Kalman filter
//! required) but adequate for short-horizon forecasting.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArimaReport {
    pub constant: f64,
    pub phi: f64,
    pub theta: f64,
    pub residual_variance: f64,
    pub one_step_forecast_level: f64,
    pub one_step_forecast_diff: f64,
    pub n_observations: usize,
}

pub fn fit(series: &[f64]) -> Option<ArimaReport> {
    let n = series.len();
    if n < 10 { return None; }
    if series.iter().any(|x| !x.is_finite()) { return None; }
    // First-difference.
    let mut diffs = Vec::with_capacity(n - 1);
    for i in 1..n {
        diffs.push(series[i] - series[i - 1]);
    }
    let nd = diffs.len();
    if nd < 5 { return None; }
    // Iteration 0: AR(1)-only fit. Residuals start as all-zero, so adding
    // the MA column on the first pass makes it collinear with the
    // intercept and XᵀX singular. Run pure AR(1), build residuals, THEN
    // bring in θ on subsequent passes. Also handle the constant-diff
    // edge case (flat series → AR1 column collinear with intercept):
    // fall back to a pure intercept fit (drift = mean(diffs)).
    let mean_diff: f64 = diffs.iter().sum::<f64>() / nd as f64;
    let diff_var: f64 = diffs.iter().map(|d| (d - mean_diff).powi(2)).sum::<f64>() / nd as f64;
    let (mut c, mut phi) = if diff_var < 1e-18 {
        (mean_diff, 0.0)    // degenerate: constant diffs → pure-drift model
    } else {
        let mut x_intercept0 = Vec::with_capacity(nd - 1);
        let mut x_phi0 = Vec::with_capacity(nd - 1);
        let mut y0 = Vec::with_capacity(nd - 1);
        for t in 1..nd {
            x_intercept0.push(1.0);
            x_phi0.push(diffs[t - 1]);
            y0.push(diffs[t]);
        }
        let beta0 = ols(&[x_intercept0, x_phi0], &y0)?;
        (beta0[0], beta0[1])
    };
    let mut theta = 0.0_f64;
    let mut residuals = vec![0.0_f64; nd];
    for t in 1..nd {
        residuals[t] = diffs[t] - c - phi * diffs[t - 1];
    }
    // Iterations 1..max: include MA column once residuals are non-trivial.
    let max_iters = 20;
    for _ in 0..max_iters {
        // If residuals are still effectively zero (flat-input edge case),
        // keep the AR(1) fit and stop iterating — model collapses to MA=0.
        let res_norm: f64 = residuals.iter().map(|r| r * r).sum::<f64>().sqrt();
        if res_norm < 1e-12 { break; }
        let mut x_intercept = Vec::with_capacity(nd - 1);
        let mut x_phi = Vec::with_capacity(nd - 1);
        let mut x_theta = Vec::with_capacity(nd - 1);
        let mut y = Vec::with_capacity(nd - 1);
        for t in 1..nd {
            x_intercept.push(1.0);
            x_phi.push(diffs[t - 1]);
            x_theta.push(residuals[t - 1]);
            y.push(diffs[t]);
        }
        let new_beta = match ols(&[x_intercept, x_phi, x_theta], &y) {
            Some(b) => b,
            None => break,    // singular at this iteration — keep prior fit
        };
        let new_c = new_beta[0];
        let new_phi = new_beta[1];
        let new_theta = new_beta[2];
        let mut new_res = vec![0.0_f64; nd];
        for t in 1..nd {
            let predicted = new_c + new_phi * diffs[t - 1] + new_theta * new_res[t - 1];
            new_res[t] = diffs[t] - predicted;
        }
        let converged = (new_phi - phi).abs() < 1e-9 && (new_theta - theta).abs() < 1e-9;
        c = new_c; phi = new_phi; theta = new_theta;
        residuals = new_res;
        if converged { break; }
    }
    // Residual variance.
    let n_eff = (nd - 1).max(1);
    let res_var: f64 = residuals.iter().skip(1).map(|r| r * r).sum::<f64>() / n_eff as f64;
    // One-step-ahead forecast.
    let last_diff = diffs[nd - 1];
    let last_res = residuals[nd - 1];
    let forecast_diff = c + phi * last_diff + theta * last_res;
    let forecast_level = series[n - 1] + forecast_diff;
    Some(ArimaReport {
        constant: c,
        phi,
        theta,
        residual_variance: res_var,
        one_step_forecast_level: forecast_level,
        one_step_forecast_diff: forecast_diff,
        n_observations: n,
    })
}

fn ols(x: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let p = x.len();
    let n = y.len();
    if p == 0 || n == 0 || x.iter().any(|c| c.len() != n) { return None; }
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    for i in 0..p {
        for j in 0..p {
            xtx[i][j] = x[i].iter().zip(x[j].iter()).map(|(a, b)| a * b).sum();
        }
        xty[i] = x[i].iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    }
    let mut aug = vec![vec![0.0_f64; p + 1]; p];
    for i in 0..p {
        for j in 0..p { aug[i][j] = xtx[i][j]; }
        aug[i][p] = xty[i];
    }
    for i in 0..p {
        let mut pivot = i;
        for r in (i + 1)..p {
            if aug[r][i].abs() > aug[pivot][i].abs() { pivot = r; }
        }
        if aug[pivot][i].abs() < 1e-18 { return None; }
        aug.swap(i, pivot);
        let div = aug[i][i];
        for v in aug[i].iter_mut() { *v /= div; }
        for r in 0..p {
            if r == i { continue; }
            let f = aug[r][i];
            if f == 0.0 { continue; }
            let pivot_row = aug[i].clone();
            for (j, v) in aug[r].iter_mut().enumerate() { *v -= f * pivot_row[j]; }
        }
    }
    Some((0..p).map(|i| aug[i][p]).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(fit(&[1.0; 5]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut s = vec![1.0; 20];
        s[5] = f64::NAN;
        assert!(fit(&s).is_none());
    }

    #[test]
    fn flat_series_yields_zero_forecast_diff() {
        let s = vec![100.0; 30];
        let r = fit(&s).unwrap();
        assert!(r.one_step_forecast_diff.abs() < 1e-9);
        assert!((r.one_step_forecast_level - 100.0).abs() < 1e-9);
    }

    #[test]
    fn linear_trend_recovers_constant_drift() {
        // y_t = 100 + 2·t → Δy_t = 2 constant → c ≈ 2, φ ≈ 0, θ ≈ 0.
        let s: Vec<f64> = (0..50).map(|i| 100.0 + 2.0 * i as f64).collect();
        let r = fit(&s).unwrap();
        assert!((r.constant - 2.0).abs() < 0.1);
        // Forecast = 100 + 2·49 + Δ_predicted ≈ 200.
        assert!((r.one_step_forecast_level - 200.0).abs() < 0.5);
    }

    #[test]
    fn forecast_dimensions_correct() {
        let s: Vec<f64> = (0..30).map(|i| (i as f64 * 0.1).sin() * 10.0 + 100.0).collect();
        let r = fit(&s).unwrap();
        assert_eq!(r.n_observations, 30);
        assert!(r.one_step_forecast_level.is_finite());
        assert!(r.one_step_forecast_diff.is_finite());
        assert!(r.residual_variance >= 0.0);
    }

    #[test]
    fn ar1_process_recovers_phi() {
        // Δy_t = 0.5 · Δy_{t-1} + noise → estimated φ ≈ 0.5.
        let n = 1_000;
        let mut state: u64 = 42;
        let mut diffs = vec![0.0_f64; n];
        for i in 1..n {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = (state >> 32) as f64 / u32::MAX as f64 - 0.5;
            diffs[i] = 0.5 * diffs[i - 1] + 0.1 * u;
        }
        let mut s = vec![100.0_f64];
        for d in &diffs { s.push(s.last().unwrap() + d); }
        let r = fit(&s).unwrap();
        // Iterative-CLS for ARMA(1,1) has a known small-sample bias toward
        // larger φ when the true process is pure AR(1) (θ has nothing
        // to fit, so the algorithm pushes its weight to φ). Tolerance
        // 0.2 is reasonable for a 1000-sample CLS estimate.
        assert!((r.phi - 0.5).abs() < 0.2,
            "φ should be ≈ 0.5, got {}", r.phi);
    }
}
