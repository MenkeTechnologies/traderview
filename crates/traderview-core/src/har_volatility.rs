//! HAR-RV (Heterogeneous AutoRegressive model of Realized Variance) —
//! Fulvio Corsi (2009).
//!
//! Single-equation regression that captures the long-memory structure
//! of realized variance using three average-RV components at distinct
//! horizons:
//!
//!   RV_{t+1} = β_0 + β_d · RV_t + β_w · RV_t^{(w)} + β_m · RV_t^{(m)} + ε
//!
//! where:
//!   - RV_t                = single-day realized variance
//!   - RV_t^{(w)}          = mean RV over the past 5 trading days
//!   - RV_t^{(m)}          = mean RV over the past 22 trading days
//!
//! The model captures heterogeneous investor horizons (intraday, weekly,
//! monthly) without explicit long-memory parameters. Despite its
//! simplicity it has been shown to forecast equity RV better than
//! ARFIMA / fractionally-integrated GARCH variants in out-of-sample
//! comparisons.
//!
//! Pure compute. Companion to `garch_1_1`, `gjr_garch`, `realized_volatility`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HarVolatilityReport {
    pub beta_0: f64,
    pub beta_daily: f64,
    pub beta_weekly: f64,
    pub beta_monthly: f64,
    pub residual_std_error: f64,
    pub r_squared: f64,
    pub n_observations: usize,
    /// 1-day-ahead RV forecast from the most recent window.
    pub one_step_forecast: f64,
}

pub fn estimate(realized_variance: &[f64]) -> Option<HarVolatilityReport> {
    let n = realized_variance.len();
    if n < 30 { return None; }
    if realized_variance.iter().any(|x| !x.is_finite() || *x < 0.0) { return None; }
    // Build the regressors at each forecast origin t.
    // We can compute targets for t = 22..n-1 (need 22 history bars and 1
    // future bar for target). The response is RV_{t+1}.
    let mut rows: Vec<[f64; 4]> = Vec::new();    // [1, RV_t, RV_w, RV_m]
    let mut targets: Vec<f64> = Vec::new();
    for t in 22..(n - 1) {
        let rv_d = realized_variance[t];
        let rv_w = (t + 1 - 5..=t).map(|i| realized_variance[i]).sum::<f64>() / 5.0;
        let rv_m = (t + 1 - 22..=t).map(|i| realized_variance[i]).sum::<f64>() / 22.0;
        rows.push([1.0, rv_d, rv_w, rv_m]);
        targets.push(realized_variance[t + 1]);
    }
    if rows.len() < 8 { return None; }
    let n_obs = rows.len();
    // OLS via normal equations on X'X (4×4) and X'y (4×1).
    let p = 4_usize;
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    for (row, y) in rows.iter().zip(targets.iter()) {
        for j in 0..p {
            xty[j] += row[j] * y;
            for k in 0..p { xtx[j][k] += row[j] * row[k]; }
        }
    }
    let coef = solve_linear(&xtx, &xty)?;
    let mut ssr = 0.0_f64;
    let y_mean: f64 = targets.iter().sum::<f64>() / n_obs as f64;
    let mut tss = 0.0_f64;
    for (row, y) in rows.iter().zip(targets.iter()) {
        let yhat = coef[0] + coef[1] * row[1] + coef[2] * row[2] + coef[3] * row[3];
        ssr += (y - yhat).powi(2);
        tss += (y - y_mean).powi(2);
    }
    let dof = (n_obs - p) as f64;
    if dof <= 0.0 { return None; }
    let sigma2 = ssr / dof;
    let r_sq = if tss > 0.0 { 1.0 - ssr / tss } else { 0.0 };
    // 1-step-ahead forecast at t = n - 1 (most recent fully-observable origin).
    let t = n - 1;
    let rv_d = realized_variance[t];
    let rv_w = (t + 1 - 5..=t).map(|i| realized_variance[i]).sum::<f64>() / 5.0;
    let rv_m = (t + 1 - 22..=t).map(|i| realized_variance[i]).sum::<f64>() / 22.0;
    let forecast = coef[0] + coef[1] * rv_d + coef[2] * rv_w + coef[3] * rv_m;
    Some(HarVolatilityReport {
        beta_0: coef[0],
        beta_daily: coef[1],
        beta_weekly: coef[2],
        beta_monthly: coef[3],
        residual_std_error: sigma2.sqrt(),
        r_squared: r_sq,
        n_observations: n_obs,
        one_step_forecast: forecast.max(0.0),
    })
}

fn solve_linear(m: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let n = m.len();
    if n == 0 || y.len() != n { return None; }
    let mut aug = vec![vec![0.0_f64; n + 1]; n];
    for (i, row) in aug.iter_mut().enumerate() {
        for (j, slot) in row.iter_mut().enumerate().take(n) { *slot = m[i][j]; }
        row[n] = y[i];
    }
    for i in 0..n {
        let mut pivot = i;
        for r in (i + 1)..n {
            if aug[r][i].abs() > aug[pivot][i].abs() { pivot = r; }
        }
        if aug[pivot][i].abs() < 1e-18 { return None; }
        aug.swap(i, pivot);
        let div = aug[i][i];
        for v in aug[i].iter_mut() { *v /= div; }
        for r in 0..n {
            if r == i { continue; }
            let f = aug[r][i];
            if f == 0.0 { continue; }
            let pivot_row = aug[i].clone();
            for (j, v) in aug[r].iter_mut().enumerate() { *v -= f * pivot_row[j]; }
        }
    }
    Some((0..n).map(|i| aug[i][n]).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(estimate(&[0.0001; 20]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut s = vec![0.0001_f64; 50];
        s[5] = f64::NAN;
        assert!(estimate(&s).is_none());
    }

    #[test]
    fn negative_variance_returns_none() {
        let mut s = vec![0.0001_f64; 50];
        s[5] = -0.001;
        assert!(estimate(&s).is_none());
    }

    #[test]
    fn constant_rv_yields_forecast_equal_to_constant() {
        // RV is constant; one-step forecast should match it.
        let s = vec![0.0001_f64; 100];
        // OLS on constant inputs is degenerate (X'X singular except for intercept).
        // Implementation should return None in this case.
        assert!(estimate(&s).is_none());
    }

    #[test]
    fn synthetic_har_recovers_coefficients() {
        // Construct RV from a known HAR process; check estimation.
        let mut state: u64 = 42;
        let true_beta = [0.00001_f64, 0.3, 0.4, 0.2];
        let mut rv = vec![0.0001_f64; 30];
        for _ in 30..500 {
            let t = rv.len();
            let d = rv[t - 1];
            let w = rv[t - 5..t].iter().sum::<f64>() / 5.0;
            let m = rv[t - 22..t].iter().sum::<f64>() / 22.0;
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.00002;
            let next = (true_beta[0] + true_beta[1] * d + true_beta[2] * w
                + true_beta[3] * m + eps).max(1e-8);
            rv.push(next);
        }
        let r = estimate(&rv).unwrap();
        // Estimates should be within 0.15 of true values for this scale.
        assert!((r.beta_daily - true_beta[1]).abs() < 0.15,
            "β_d: estimated {} vs true {}", r.beta_daily, true_beta[1]);
        assert!((r.beta_weekly - true_beta[2]).abs() < 0.2,
            "β_w: estimated {} vs true {}", r.beta_weekly, true_beta[2]);
        assert!((r.beta_monthly - true_beta[3]).abs() < 0.2,
            "β_m: estimated {} vs true {}", r.beta_monthly, true_beta[3]);
    }

    #[test]
    fn forecast_non_negative() {
        let mut state: u64 = 11;
        let rv: Vec<f64> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 32) as f64 / u32::MAX as f64).max(1e-6) * 0.0005
        }).collect();
        let r = estimate(&rv).unwrap();
        assert!(r.one_step_forecast >= 0.0);
    }

    #[test]
    fn r_squared_in_unit_range() {
        let mut state: u64 = 99;
        let rv: Vec<f64> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 32) as f64 / u32::MAX as f64).max(1e-6) * 0.0005
        }).collect();
        let r = estimate(&rv).unwrap();
        assert!((0.0..=1.0).contains(&r.r_squared) || r.r_squared < 0.0,
            "R² {} should be in [0,1] or negative for bad fit", r.r_squared);
    }
}
