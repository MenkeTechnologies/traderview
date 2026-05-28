//! Engle's ARCH-LM test for conditional heteroscedasticity.
//!
//! Tests H₀: returns have constant (homoscedastic) variance vs
//! H₁: returns exhibit time-varying variance (ARCH effects).
//!
//! Procedure:
//!   1. Demean the return series → e_t.
//!   2. Regress e_t² on a constant + (e_{t-1}², …, e_{t-q}²).
//!   3. LM statistic = (n − q) · R² ~ χ²(q) under the null.
//!
//! High LM (and small p-value) rejects homoscedasticity, motivating a
//! GARCH model (see `garch_1_1`).
//!
//! Pure compute. Returns the test statistic and the R² of the auxiliary
//! regression; caller can compare LM against tabulated χ²(q) critical
//! values (q=5 at 5% → 11.07).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ArchLmReport {
    pub lm_statistic: f64,
    pub r_squared: f64,
    pub lags: usize,
    pub n_observations: usize,
}

pub fn test(returns: &[f64], lags: usize) -> Option<ArchLmReport> {
    let n = returns.len();
    if n < 3 * lags + 2 || lags == 0 { return None; }
    if returns.iter().any(|r| !r.is_finite()) { return None; }
    // Demean.
    let mean = returns.iter().sum::<f64>() / n as f64;
    let e_sq: Vec<f64> = returns.iter().map(|r| (r - mean).powi(2)).collect();
    let start = lags;
    let m = n - start;
    if m < lags + 2 { return None; }
    let mut x_intercept = Vec::with_capacity(m);
    let mut x_lags: Vec<Vec<f64>> = (0..lags).map(|_| Vec::with_capacity(m)).collect();
    let mut y = Vec::with_capacity(m);
    for t in start..n {
        x_intercept.push(1.0);
        for i in 0..lags { x_lags[i].push(e_sq[t - 1 - i]); }
        y.push(e_sq[t]);
    }
    let mut cols = vec![x_intercept];
    cols.extend(x_lags);
    let beta = ols(&cols, &y)?;
    let mut ss_res = 0.0_f64;
    for k in 0..m {
        let yh: f64 = (0..cols.len()).map(|i| beta[i] * cols[i][k]).sum();
        ss_res += (y[k] - yh).powi(2);
    }
    let y_mean: f64 = y.iter().sum::<f64>() / m as f64;
    let ss_tot: f64 = y.iter().map(|yv| (yv - y_mean).powi(2)).sum();
    if ss_tot <= 0.0 { return None; }
    let r2 = 1.0 - ss_res / ss_tot;
    let lm = (m as f64) * r2;
    Some(ArchLmReport {
        lm_statistic: lm,
        r_squared: r2,
        lags,
        n_observations: m,
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
        assert!(test(&[0.01; 5], 5).is_none());
    }

    #[test]
    fn zero_lags_returns_none() {
        assert!(test(&[0.01; 50], 0).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut r = vec![0.01; 50];
        r[10] = f64::NAN;
        assert!(test(&r, 2).is_none());
    }

    #[test]
    fn arch_process_yields_large_lm() {
        // Generate ARCH(1) series: r_t = √(0.01 + 0.8·r_{t-1}²) · z_t.
        let n = 1_000;
        let mut state: u64 = 42;
        let mut r = vec![0.0_f64; n];
        for t in 1..n {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (state >> 32) as f64 / u32::MAX as f64;
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let var = 0.01 + 0.8 * r[t - 1].powi(2);
            r[t] = var.sqrt() * z;
        }
        let report = test(&r, 5).unwrap();
        // Strong ARCH effects → LM well above 5% χ²(5) critical (11.07).
        assert!(report.lm_statistic > 20.0,
            "ARCH(1) series should reject homoscedasticity, got LM={}",
            report.lm_statistic);
    }

    #[test]
    fn iid_gaussian_yields_small_lm() {
        let n = 1_000;
        let mut state: u64 = 999;
        let mut r = Vec::with_capacity(n);
        for _ in 0..n / 2 {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u1 = ((state >> 32) as f64 / u32::MAX as f64).max(1e-12);
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u2 = (state >> 32) as f64 / u32::MAX as f64;
            let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let z2 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).sin();
            r.push(0.01 * z1);
            r.push(0.01 * z2);
        }
        let report = test(&r, 5).unwrap();
        // Under the null, LM has mean = lags = 5. Allow generous band.
        assert!(report.lm_statistic < 25.0,
            "iid Gaussian should NOT show strong ARCH, got LM={}",
            report.lm_statistic);
    }

    #[test]
    fn r_squared_in_unit_range() {
        let r: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin() * 0.02).collect();
        let report = test(&r, 3).unwrap();
        assert!((-1.0..=1.0).contains(&report.r_squared));
    }
}
