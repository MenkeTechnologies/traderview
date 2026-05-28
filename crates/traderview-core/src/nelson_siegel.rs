//! Nelson-Siegel-Svensson yield-curve parametric fit.
//!
//! Four-factor parametric form (β₀, β₁, β₂, τ) — the classic Nelson-
//! Siegel (1987), plus optional Svensson (1994) extension with a fifth
//! factor (β₃, τ₂):
//!
//!   y(t) = β₀ + β₁ · [(1 − e^{−t/τ})/(t/τ)]
//!             + β₂ · [(1 − e^{−t/τ})/(t/τ) − e^{−t/τ}]
//!             + β₃ · [(1 − e^{−t/τ₂})/(t/τ₂) − e^{−t/τ₂}]   (Svensson)
//!
//! Interpretation:
//!   β₀ = long-rate level
//!   β₁ = short-vs-long slope (negative = steep curve)
//!   β₂ = curvature / hump
//!   β₃ = second hump (Svensson)
//!   τ, τ₂ = decay constants pinning hump locations
//!
//! Fit method: given tenors `t_i` and observed zero rates `y_i` plus
//! fixed τ (and τ₂), solve linear least-squares for (β₀, β₁, β₂[, β₃]).
//! Non-linear τ optimization is out-of-scope — caller supplies τ from
//! market convention (typical: τ = 1.0–2.0, τ₂ = 5.0 for Svensson).
//!
//! Pure compute. Companion to `yield_curve_bootstrap` (which builds the
//! raw discount curve point-by-point); this module produces a smooth
//! parametric overlay for extrapolation and risk-factor analysis.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NsParams {
    pub beta0: f64,
    pub beta1: f64,
    pub beta2: f64,
    pub tau: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NssParams {
    pub beta0: f64,
    pub beta1: f64,
    pub beta2: f64,
    pub beta3: f64,
    pub tau1: f64,
    pub tau2: f64,
}

pub fn nelson_siegel_yield(tenor: f64, p: &NsParams) -> f64 {
    if tenor <= 0.0 || p.tau <= 0.0 || !p.tau.is_finite() {
        return f64::NAN;
    }
    let x = tenor / p.tau;
    let factor1 = (1.0 - (-x).exp()) / x;
    let factor2 = factor1 - (-x).exp();
    p.beta0 + p.beta1 * factor1 + p.beta2 * factor2
}

pub fn svensson_yield(tenor: f64, p: &NssParams) -> f64 {
    if tenor <= 0.0 || p.tau1 <= 0.0 || p.tau2 <= 0.0
        || !p.tau1.is_finite() || !p.tau2.is_finite()
    {
        return f64::NAN;
    }
    let x1 = tenor / p.tau1;
    let x2 = tenor / p.tau2;
    let f1 = (1.0 - (-x1).exp()) / x1;
    let f2 = f1 - (-x1).exp();
    let f3 = (1.0 - (-x2).exp()) / x2 - (-x2).exp();
    p.beta0 + p.beta1 * f1 + p.beta2 * f2 + p.beta3 * f3
}

pub fn fit_nelson_siegel(tenors: &[f64], yields: &[f64], tau: f64) -> Option<NsParams> {
    let n = tenors.len();
    if yields.len() != n || n < 3
        || !tau.is_finite() || tau <= 0.0
        || tenors.iter().any(|t| !t.is_finite() || *t <= 0.0)
        || yields.iter().any(|y| !y.is_finite())
    {
        return None;
    }
    // Linear LS: y = β₀·1 + β₁·f1 + β₂·f2.
    let mut col0 = Vec::with_capacity(n);
    let mut col1 = Vec::with_capacity(n);
    let mut col2 = Vec::with_capacity(n);
    for &t in tenors {
        let x = t / tau;
        let f1 = (1.0 - (-x).exp()) / x;
        let f2 = f1 - (-x).exp();
        col0.push(1.0);
        col1.push(f1);
        col2.push(f2);
    }
    let (b, _) = ols(&[col0, col1, col2], yields)?;
    if b.len() != 3 || b.iter().any(|x| !x.is_finite()) { return None; }
    Some(NsParams { beta0: b[0], beta1: b[1], beta2: b[2], tau })
}

pub fn fit_svensson(tenors: &[f64], yields: &[f64], tau1: f64, tau2: f64) -> Option<NssParams> {
    let n = tenors.len();
    if yields.len() != n || n < 4
        || !tau1.is_finite() || tau1 <= 0.0
        || !tau2.is_finite() || tau2 <= 0.0
        || (tau1 - tau2).abs() < 1e-9
        || tenors.iter().any(|t| !t.is_finite() || *t <= 0.0)
        || yields.iter().any(|y| !y.is_finite())
    {
        return None;
    }
    let mut col0 = Vec::with_capacity(n);
    let mut col1 = Vec::with_capacity(n);
    let mut col2 = Vec::with_capacity(n);
    let mut col3 = Vec::with_capacity(n);
    for &t in tenors {
        let x1 = t / tau1;
        let x2 = t / tau2;
        let f1 = (1.0 - (-x1).exp()) / x1;
        let f2 = f1 - (-x1).exp();
        let f3 = (1.0 - (-x2).exp()) / x2 - (-x2).exp();
        col0.push(1.0);
        col1.push(f1);
        col2.push(f2);
        col3.push(f3);
    }
    let (b, _) = ols(&[col0, col1, col2, col3], yields)?;
    if b.len() != 4 || b.iter().any(|x| !x.is_finite()) { return None; }
    Some(NssParams { beta0: b[0], beta1: b[1], beta2: b[2], beta3: b[3], tau1, tau2 })
}

fn ols(x: &[Vec<f64>], y: &[f64]) -> Option<(Vec<f64>, Vec<f64>)> {
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
    let mut aug = vec![vec![0.0_f64; 2 * p + 1]; p];
    for i in 0..p {
        for j in 0..p {
            aug[i][j] = xtx[i][j];
            aug[i][p + j] = if i == j { 1.0 } else { 0.0 };
        }
        aug[i][2 * p] = xty[i];
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
    let beta: Vec<f64> = (0..p).map(|i| aug[i][2 * p]).collect();
    let mut ss_res = 0.0_f64;
    for k in 0..n {
        let yh: f64 = (0..p).map(|i| beta[i] * x[i][k]).sum();
        ss_res += (y[k] - yh).powi(2);
    }
    let dof = (n as isize - p as isize).max(1) as f64;
    let sigma2 = ss_res / dof;
    let mut se = vec![0.0_f64; p];
    for i in 0..p {
        let var = sigma2 * aug[i][p + i];
        se[i] = if var > 0.0 { var.sqrt() } else { 0.0 };
    }
    Some((beta, se))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ns_yield_at_zero_tenor_undefined() {
        let p = NsParams { beta0: 0.05, beta1: -0.02, beta2: 0.01, tau: 2.0 };
        assert!(nelson_siegel_yield(0.0, &p).is_nan());
        assert!(nelson_siegel_yield(-1.0, &p).is_nan());
    }

    #[test]
    fn ns_invalid_tau_returns_nan() {
        let p = NsParams { beta0: 0.05, beta1: -0.02, beta2: 0.01, tau: 0.0 };
        assert!(nelson_siegel_yield(1.0, &p).is_nan());
        let p = NsParams { beta0: 0.05, beta1: -0.02, beta2: 0.01, tau: f64::NAN };
        assert!(nelson_siegel_yield(1.0, &p).is_nan());
    }

    #[test]
    fn ns_long_tenor_approaches_beta0() {
        // As t → ∞, the loadings on β₁ and β₂ both → 0 → y(t) → β₀.
        let p = NsParams { beta0: 0.05, beta1: -0.02, beta2: 0.01, tau: 2.0 };
        let y_30 = nelson_siegel_yield(30.0, &p);
        let y_50 = nelson_siegel_yield(50.0, &p);
        assert!((y_50 - 0.05).abs() < 0.005);
        assert!((y_30 - 0.05).abs() > (y_50 - 0.05).abs());
    }

    #[test]
    fn ns_short_tenor_approaches_beta0_plus_beta1() {
        // As t → 0⁺, the loading on β₁ → 1 and on β₂ → 0 → y → β₀ + β₁.
        let p = NsParams { beta0: 0.05, beta1: -0.02, beta2: 0.01, tau: 2.0 };
        let y_small = nelson_siegel_yield(0.001, &p);
        assert!((y_small - (0.05 - 0.02)).abs() < 0.001);
    }

    #[test]
    fn fit_recovers_known_ns_params() {
        // Synthetic curve generated from known params + sample at tenors.
        let true_p = NsParams { beta0: 0.045, beta1: -0.015, beta2: 0.020, tau: 1.5 };
        let tenors: Vec<f64> = vec![0.25, 0.5, 1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 20.0, 30.0];
        let yields: Vec<f64> = tenors.iter().map(|t| nelson_siegel_yield(*t, &true_p)).collect();
        let est = fit_nelson_siegel(&tenors, &yields, 1.5).unwrap();
        assert!((est.beta0 - true_p.beta0).abs() < 1e-9);
        assert!((est.beta1 - true_p.beta1).abs() < 1e-9);
        assert!((est.beta2 - true_p.beta2).abs() < 1e-9);
    }

    #[test]
    fn fit_too_few_points_returns_none() {
        assert!(fit_nelson_siegel(&[1.0, 2.0], &[0.05, 0.04], 1.5).is_none());
    }

    #[test]
    fn fit_dim_mismatch_returns_none() {
        assert!(fit_nelson_siegel(&[1.0, 2.0, 3.0], &[0.05, 0.04], 1.5).is_none());
    }

    #[test]
    fn fit_invalid_tau_returns_none() {
        assert!(fit_nelson_siegel(&[1.0, 2.0, 3.0], &[0.05, 0.04, 0.045], 0.0).is_none());
        assert!(fit_nelson_siegel(&[1.0, 2.0, 3.0], &[0.05, 0.04, 0.045], -1.0).is_none());
    }

    #[test]
    fn svensson_fit_recovers_known_params() {
        let true_p = NssParams {
            beta0: 0.045, beta1: -0.015, beta2: 0.020, beta3: -0.010,
            tau1: 1.5, tau2: 5.0,
        };
        let tenors: Vec<f64> = vec![0.25, 0.5, 1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 20.0, 30.0];
        let yields: Vec<f64> = tenors.iter().map(|t| svensson_yield(*t, &true_p)).collect();
        let est = fit_svensson(&tenors, &yields, 1.5, 5.0).unwrap();
        assert!((est.beta0 - true_p.beta0).abs() < 1e-9);
        assert!((est.beta1 - true_p.beta1).abs() < 1e-9);
        assert!((est.beta2 - true_p.beta2).abs() < 1e-9);
        assert!((est.beta3 - true_p.beta3).abs() < 1e-9);
    }

    #[test]
    fn svensson_rejects_collinear_taus() {
        assert!(fit_svensson(&[1.0; 5], &[0.05; 5], 1.5, 1.5).is_none());
    }
}
