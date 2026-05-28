//! Engle-Granger cointegration test for pairs trading.
//!
//! Two-step procedure:
//!   1. Regress  y_t = α + β · x_t + ε_t        (OLS)
//!   2. Test the residual series ε_t for stationarity with an
//!      Augmented-Dickey-Fuller (ADF) regression:
//!
//!        Δε_t = γ · ε_{t−1} + Σ φ_i · Δε_{t−i} + u_t
//!
//!      The test statistic is t-stat on γ. The null hypothesis is
//!      "ε has a unit root" (not cointegrated). Reject when t < critical.
//!
//! Returns the OLS hedge ratio β (used to size the pair trade) plus
//! the ADF statistic, p-value bin (1% / 5% / 10% / insignificant),
//! and half-life of mean reversion if applicable.
//!
//! Critical values are MacKinnon (1991) for residual-based ADF with
//! intercept — embedded as constants since they're stationary
//! distribution quantiles, not data-derived.
//!
//! Pure compute. No lag selection — caller supplies `adf_lags`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignificanceLevel { Pct1, Pct5, Pct10, Insignificant }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CointegrationReport {
    pub hedge_ratio_beta: f64,
    pub intercept_alpha: f64,
    pub residuals: Vec<f64>,
    pub adf_statistic: f64,
    pub significance: SignificanceLevel,
    pub mean_reversion_half_life: Option<f64>,
}

pub fn test(y: &[f64], x: &[f64], adf_lags: usize) -> Option<CointegrationReport> {
    let n = y.len();
    if n != x.len() || n < 20 + adf_lags + 2 {
        return None;
    }
    if y.iter().any(|v| !v.is_finite()) || x.iter().any(|v| !v.is_finite()) {
        return None;
    }
    // Step 1: OLS y = α + βx.
    let n_f = n as f64;
    let mean_x = x.iter().sum::<f64>() / n_f;
    let mean_y = y.iter().sum::<f64>() / n_f;
    let mut sxy = 0.0_f64;
    let mut sxx = 0.0_f64;
    for (xi, yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        sxy += dx * dy;
        sxx += dx * dx;
    }
    if sxx <= 0.0 { return None; }
    let beta = sxy / sxx;
    let alpha = mean_y - beta * mean_x;
    let residuals: Vec<f64> = y.iter().zip(x.iter()).map(|(yi, xi)| yi - alpha - beta * xi).collect();
    // Step 2: ADF on residuals.
    let adf_stat = adf_t_stat(&residuals, adf_lags)?;
    // MacKinnon residual-based ADF critical values (intercept, no trend).
    // For ~250 obs: 1% ≈ −3.90, 5% ≈ −3.34, 10% ≈ −3.04.
    let sig = if adf_stat < -3.90 { SignificanceLevel::Pct1 }
        else if adf_stat < -3.34 { SignificanceLevel::Pct5 }
        else if adf_stat < -3.04 { SignificanceLevel::Pct10 }
        else { SignificanceLevel::Insignificant };
    // Mean-reversion half-life from OU fit: ε_t = ρ · ε_{t−1} + ν.
    let half_life = ou_half_life(&residuals);
    Some(CointegrationReport {
        hedge_ratio_beta: beta,
        intercept_alpha: alpha,
        residuals,
        adf_statistic: adf_stat,
        significance: sig,
        mean_reversion_half_life: half_life,
    })
}

fn adf_t_stat(series: &[f64], lags: usize) -> Option<f64> {
    let n = series.len();
    if n < lags + 4 { return None; }
    // Δy_t = γ · y_{t−1} + Σ φ_i · Δy_{t−i} + intercept + u_t
    // Build design matrix manually for the simple case.
    // For lags=0 it reduces to a 2-column OLS: intercept + y_{t−1}.
    let mut diffs = vec![0.0_f64; n];
    for i in 1..n { diffs[i] = series[i] - series[i - 1]; }
    let start = 1 + lags;
    if n <= start { return None; }
    let m = n - start;
    if m < 2 { return None; }
    // X columns: [1, y_{t−1}, Δy_{t−1}, …, Δy_{t−lags}], y: Δy_t.
    let p = 2 + lags;
    let mut x: Vec<Vec<f64>> = (0..p).map(|_| Vec::with_capacity(m)).collect();
    let mut y_vec: Vec<f64> = Vec::with_capacity(m);
    for i in start..n {
        x[0].push(1.0);
        x[1].push(series[i - 1]);
        for k in 0..lags {
            x[2 + k].push(diffs[i - 1 - k]);
        }
        y_vec.push(diffs[i]);
    }
    let (beta, se) = ols_with_se(&x, &y_vec)?;
    let gamma = beta[1];
    let se_gamma = se[1];
    if se_gamma <= 0.0 { return None; }
    Some(gamma / se_gamma)
}

fn ols_with_se(x: &[Vec<f64>], y: &[f64]) -> Option<(Vec<f64>, Vec<f64>)> {
    let p = x.len();
    let n = y.len();
    if p == 0 || n == 0 { return None; }
    if x.iter().any(|c| c.len() != n) { return None; }
    // Normal equations: (XᵀX) β = Xᵀy. Solve with Gauss-Jordan
    // (in-place; only sensible for small p — ADF has p ≤ ~10 typically).
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    for i in 0..p {
        for j in 0..p {
            xtx[i][j] = x[i].iter().zip(x[j].iter()).map(|(a, b)| a * b).sum();
        }
        xty[i] = x[i].iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    }
    // Augmented [XᵀX | I | Xᵀy] for inverse + solve in one pass.
    let mut aug = vec![vec![0.0_f64; 2 * p + 1]; p];
    for i in 0..p {
        for j in 0..p {
            aug[i][j] = xtx[i][j];
            aug[i][p + j] = if i == j { 1.0 } else { 0.0 };
        }
        aug[i][2 * p] = xty[i];
    }
    for i in 0..p {
        // Partial pivot.
        let mut pivot_row = i;
        for r in (i + 1)..p {
            if aug[r][i].abs() > aug[pivot_row][i].abs() { pivot_row = r; }
        }
        if aug[pivot_row][i].abs() < 1e-18 { return None; }
        aug.swap(i, pivot_row);
        let div = aug[i][i];
        for v in aug[i].iter_mut() { *v /= div; }
        for r in 0..p {
            if r == i { continue; }
            let factor = aug[r][i];
            if factor == 0.0 { continue; }
            let pivot_row = aug[i].clone();
            for (j, v) in aug[r].iter_mut().enumerate() {
                *v -= factor * pivot_row[j];
            }
        }
    }
    let beta: Vec<f64> = (0..p).map(|i| aug[i][2 * p]).collect();
    // Residuals → σ² → cov(β) = σ² · (XᵀX)⁻¹.
    let mut residuals = Vec::with_capacity(n);
    for k in 0..n {
        let yhat: f64 = (0..p).map(|i| beta[i] * x[i][k]).sum();
        residuals.push(y[k] - yhat);
    }
    let ss_res: f64 = residuals.iter().map(|r| r * r).sum();
    let dof = (n as isize - p as isize).max(1) as f64;
    let sigma2 = ss_res / dof;
    let mut se = vec![0.0_f64; p];
    for i in 0..p {
        let var = sigma2 * aug[i][p + i];
        se[i] = if var > 0.0 { var.sqrt() } else { 0.0 };
    }
    Some((beta, se))
}

fn ou_half_life(series: &[f64]) -> Option<f64> {
    let n = series.len();
    if n < 4 { return None; }
    let mut x = Vec::with_capacity(n - 1);
    let mut y = Vec::with_capacity(n - 1);
    for i in 1..n {
        x.push(series[i - 1]);
        y.push(series[i] - series[i - 1]);
    }
    // OLS Δε = (ρ − 1) · ε_{t−1} + drift.
    let m = x.len() as f64;
    let mx = x.iter().sum::<f64>() / m;
    let my = y.iter().sum::<f64>() / m;
    let mut sxy = 0.0; let mut sxx = 0.0;
    for (xi, yi) in x.iter().zip(y.iter()) {
        sxy += (xi - mx) * (yi - my);
        sxx += (xi - mx).powi(2);
    }
    if sxx <= 0.0 { return None; }
    let theta = sxy / sxx;        // = ρ − 1 (negative when mean-reverting)
    if theta >= 0.0 { return None; }
    Some(-(2.0_f64.ln()) / theta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_mismatch_returns_none() {
        let y = vec![1.0; 50];
        let x = vec![1.0; 20];
        assert!(test(&y, &x, 1).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        let y = vec![1.0; 10];
        let x = vec![1.0; 10];
        assert!(test(&y, &x, 1).is_none());
    }

    #[test]
    fn nan_inputs_return_none() {
        let mut y = vec![1.0; 100];
        let x = vec![1.0; 100];
        y[5] = f64::NAN;
        assert!(test(&y, &x, 1).is_none());
    }

    #[test]
    fn perfectly_cointegrated_pair_significant() {
        // y = 2·x + small mean-reverting noise — should be cointegrated.
        let n = 250;
        let mut x = vec![0.0_f64; n];
        let mut state: u64 = 12345;
        for i in 1..n {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let inc = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.05;
            x[i] = x[i - 1] + inc;    // random walk
        }
        let mut noise = vec![0.0_f64; n];
        // Generate AR(1) residual: ε_t = 0.5 · ε_{t-1} + u_t — mean-reverting.
        for i in 1..n {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let u = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.02;
            noise[i] = 0.5 * noise[i - 1] + u;
        }
        let y: Vec<f64> = x.iter().zip(noise.iter()).map(|(xi, ei)| 2.0 * xi + ei).collect();
        let report = test(&y, &x, 2).expect("populated");
        assert!((report.hedge_ratio_beta - 2.0).abs() < 0.1,
            "hedge ratio should ≈ 2, got {}", report.hedge_ratio_beta);
        // ADF stat should be quite negative (mean-reverting noise).
        assert!(report.adf_statistic < -2.0,
            "ADF should be significantly negative, got {}", report.adf_statistic);
    }

    #[test]
    fn unrelated_random_walks_not_cointegrated() {
        let n = 250;
        let mut x = vec![0.0_f64; n];
        let mut y = vec![0.0_f64; n];
        let mut state: u64 = 999;
        for i in 1..n {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let u = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.05;
            x[i] = x[i - 1] + u;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let v = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.05;
            y[i] = y[i - 1] + v;
        }
        let report = test(&y, &x, 2).expect("populated");
        // Spurious regression: ADF should NOT be highly negative.
        assert!(matches!(report.significance, SignificanceLevel::Insignificant | SignificanceLevel::Pct10),
            "unrelated walks should not show strong cointegration, got {:?} (adf={})",
            report.significance, report.adf_statistic);
    }

    #[test]
    fn collinear_x_returns_none() {
        let x = vec![5.0; 100];
        let y = vec![10.0; 100];
        assert!(test(&y, &x, 1).is_none());
    }
}
