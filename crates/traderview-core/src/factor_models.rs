//! Factor regressions — Fama-French 3-factor (Mkt-Rf, SMB, HML) and
//! Carhart 4-factor (FF3 + WML momentum).
//!
//! Regression form:
//!   r_p − r_f = α + β_mkt·(r_m − r_f) + β_smb·SMB + β_hml·HML
//!             + β_wml·WML (4-factor only) + ε
//!
//! Returns coefficients with SEs + R² + t-stats for each loading.
//! Loadings are diagnostic: e.g. positive HML = value tilt, positive
//! SMB = small-cap tilt, positive WML = momentum tilt, α > 0 with
//! t-stat > 2 = skill above factor exposure.
//!
//! Pure compute. Uses the same Gauss-Jordan OLS with SE routine as
//! `treynor_mazuy`, `henriksson_merton`, and `cointegration`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ff3Report {
    pub alpha: f64,
    pub beta_mkt: f64,
    pub beta_smb: f64,
    pub beta_hml: f64,
    pub alpha_se: f64,
    pub beta_mkt_se: f64,
    pub beta_smb_se: f64,
    pub beta_hml_se: f64,
    pub alpha_tstat: f64,
    pub r_squared: f64,
    pub n_observations: usize,
}

#[derive(Debug, Clone)]
pub struct Ff3Inputs<'a> {
    pub portfolio_returns: &'a [f64],
    pub market_excess: &'a [f64],
    pub smb: &'a [f64],
    pub hml: &'a [f64],
    pub risk_free: &'a [f64],
}

pub fn ff3(inputs: &Ff3Inputs<'_>) -> Option<Ff3Report> {
    let n = inputs.portfolio_returns.len();
    if inputs.market_excess.len() != n
        || inputs.smb.len() != n
        || inputs.hml.len() != n
        || inputs.risk_free.len() != n
        || n < 10
    {
        return None;
    }
    let mut y = Vec::with_capacity(n);
    let mut x_mkt = Vec::with_capacity(n);
    let mut x_smb = Vec::with_capacity(n);
    let mut x_hml = Vec::with_capacity(n);
    for i in 0..n {
        let p = inputs.portfolio_returns[i];
        let m = inputs.market_excess[i];
        let s = inputs.smb[i];
        let h = inputs.hml[i];
        let rf = inputs.risk_free[i];
        if !p.is_finite() || !m.is_finite() || !s.is_finite() || !h.is_finite() || !rf.is_finite() {
            continue;
        }
        y.push(p - rf);
        x_mkt.push(m);
        x_smb.push(s);
        x_hml.push(h);
    }
    let n_obs = y.len();
    if n_obs < 10 { return None; }
    let cols = vec![vec![1.0; n_obs], x_mkt, x_smb, x_hml];
    let (b, se) = ols_with_se(&cols, &y)?;
    if b.len() != 4 || se.len() != 4 { return None; }
    let y_mean: f64 = y.iter().sum::<f64>() / n_obs as f64;
    let mut ss_tot = 0.0; let mut ss_res = 0.0;
    for k in 0..n_obs {
        let pred = b[0] + b[1] * cols[1][k] + b[2] * cols[2][k] + b[3] * cols[3][k];
        ss_tot += (y[k] - y_mean).powi(2);
        ss_res += (y[k] - pred).powi(2);
    }
    let r2 = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };
    Some(Ff3Report {
        alpha: b[0], beta_mkt: b[1], beta_smb: b[2], beta_hml: b[3],
        alpha_se: se[0], beta_mkt_se: se[1], beta_smb_se: se[2], beta_hml_se: se[3],
        alpha_tstat: if se[0] > 0.0 { b[0] / se[0] } else { 0.0 },
        r_squared: r2,
        n_observations: n_obs,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Carhart4Report {
    pub alpha: f64,
    pub beta_mkt: f64,
    pub beta_smb: f64,
    pub beta_hml: f64,
    pub beta_wml: f64,
    pub alpha_se: f64,
    pub beta_mkt_se: f64,
    pub beta_smb_se: f64,
    pub beta_hml_se: f64,
    pub beta_wml_se: f64,
    pub alpha_tstat: f64,
    pub r_squared: f64,
    pub n_observations: usize,
}

#[derive(Debug, Clone)]
pub struct Carhart4Inputs<'a> {
    pub portfolio_returns: &'a [f64],
    pub market_excess: &'a [f64],
    pub smb: &'a [f64],
    pub hml: &'a [f64],
    pub wml: &'a [f64],
    pub risk_free: &'a [f64],
}

pub fn carhart4(inputs: &Carhart4Inputs<'_>) -> Option<Carhart4Report> {
    let n = inputs.portfolio_returns.len();
    if inputs.market_excess.len() != n
        || inputs.smb.len() != n
        || inputs.hml.len() != n
        || inputs.wml.len() != n
        || inputs.risk_free.len() != n
        || n < 10
    {
        return None;
    }
    let mut y = Vec::with_capacity(n);
    let mut x_mkt = Vec::with_capacity(n);
    let mut x_smb = Vec::with_capacity(n);
    let mut x_hml = Vec::with_capacity(n);
    let mut x_wml = Vec::with_capacity(n);
    for i in 0..n {
        let p = inputs.portfolio_returns[i];
        let m = inputs.market_excess[i];
        let s = inputs.smb[i];
        let h = inputs.hml[i];
        let w = inputs.wml[i];
        let rf = inputs.risk_free[i];
        if !p.is_finite() || !m.is_finite() || !s.is_finite() || !h.is_finite()
            || !w.is_finite() || !rf.is_finite() {
            continue;
        }
        y.push(p - rf);
        x_mkt.push(m);
        x_smb.push(s);
        x_hml.push(h);
        x_wml.push(w);
    }
    let n_obs = y.len();
    if n_obs < 10 { return None; }
    let cols = vec![vec![1.0; n_obs], x_mkt, x_smb, x_hml, x_wml];
    let (b, se) = ols_with_se(&cols, &y)?;
    if b.len() != 5 || se.len() != 5 { return None; }
    let y_mean: f64 = y.iter().sum::<f64>() / n_obs as f64;
    let mut ss_tot = 0.0; let mut ss_res = 0.0;
    for k in 0..n_obs {
        let pred = b[0] + b[1] * cols[1][k] + b[2] * cols[2][k] + b[3] * cols[3][k] + b[4] * cols[4][k];
        ss_tot += (y[k] - y_mean).powi(2);
        ss_res += (y[k] - pred).powi(2);
    }
    let r2 = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };
    Some(Carhart4Report {
        alpha: b[0], beta_mkt: b[1], beta_smb: b[2], beta_hml: b[3], beta_wml: b[4],
        alpha_se: se[0], beta_mkt_se: se[1], beta_smb_se: se[2], beta_hml_se: se[3],
        beta_wml_se: se[4],
        alpha_tstat: if se[0] > 0.0 { b[0] / se[0] } else { 0.0 },
        r_squared: r2,
        n_observations: n_obs,
    })
}

fn ols_with_se(x: &[Vec<f64>], y: &[f64]) -> Option<(Vec<f64>, Vec<f64>)> {
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
    fn ff3_dim_mismatch_returns_none() {
        let p = vec![0.01; 30];
        let m = vec![0.01; 30];
        let s = vec![0.01; 30];
        let h = vec![0.01; 30];
        let rf = vec![0.0; 15];
        let inputs = Ff3Inputs {
            portfolio_returns: &p, market_excess: &m, smb: &s, hml: &h, risk_free: &rf,
        };
        assert!(ff3(&inputs).is_none());
    }

    #[test]
    fn ff3_too_short_returns_none() {
        let p = vec![0.01; 5];
        let inputs = Ff3Inputs {
            portfolio_returns: &p,
            market_excess: &p,
            smb: &p,
            hml: &p,
            risk_free: &p,
        };
        assert!(ff3(&inputs).is_none());
    }

    #[test]
    fn ff3_recovers_synthetic_betas() {
        // Portfolio = α + 1.0·M + 0.3·SMB − 0.5·HML.
        let n = 500;
        let mut state: u64 = 1234;
        let mut m = Vec::with_capacity(n);
        let mut s = Vec::with_capacity(n);
        let mut h = Vec::with_capacity(n);
        let mut rf = Vec::with_capacity(n);
        let mut p = Vec::with_capacity(n);
        for _ in 0..n {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let m_i = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.04;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let s_i = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.03;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let h_i = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.025;
            let rf_i = 0.00005;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.005;
            let p_i = rf_i + 0.001 + 1.0 * m_i + 0.3 * s_i - 0.5 * h_i + eps;
            m.push(m_i); s.push(s_i); h.push(h_i); rf.push(rf_i); p.push(p_i);
        }
        let inputs = Ff3Inputs {
            portfolio_returns: &p, market_excess: &m, smb: &s, hml: &h, risk_free: &rf,
        };
        let report = ff3(&inputs).unwrap();
        assert!((report.beta_mkt - 1.0).abs() < 0.1, "β_mkt: got {}", report.beta_mkt);
        assert!((report.beta_smb - 0.3).abs() < 0.1, "β_smb: got {}", report.beta_smb);
        assert!((report.beta_hml - (-0.5)).abs() < 0.1, "β_hml: got {}", report.beta_hml);
        assert!(report.r_squared > 0.5);
    }

    #[test]
    fn carhart4_picks_up_momentum_loading() {
        let n = 500;
        let mut state: u64 = 5678;
        let mut m = Vec::with_capacity(n);
        let mut s = Vec::with_capacity(n);
        let mut h = Vec::with_capacity(n);
        let mut w = Vec::with_capacity(n);
        let mut rf = Vec::with_capacity(n);
        let mut p = Vec::with_capacity(n);
        for _ in 0..n {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let m_i = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.04;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let s_i = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.03;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let h_i = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.025;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let w_i = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.02;
            let rf_i = 0.00005;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let eps = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.005;
            // Momentum-tilted: β_wml = 0.8.
            let p_i = rf_i + 0.001 + 0.9 * m_i + 0.0 + 0.0 + 0.8 * w_i + eps;
            m.push(m_i); s.push(s_i); h.push(h_i); w.push(w_i); rf.push(rf_i); p.push(p_i);
        }
        let inputs = Carhart4Inputs {
            portfolio_returns: &p, market_excess: &m, smb: &s, hml: &h, wml: &w, risk_free: &rf,
        };
        let report = carhart4(&inputs).unwrap();
        assert!((report.beta_wml - 0.8).abs() < 0.1,
            "β_wml: got {}", report.beta_wml);
    }

    #[test]
    fn nan_observations_skipped_safely() {
        // Use varying SMB/HML to avoid collinearity with the intercept
        // (constant SMB → singular XᵀX → regression fails to invert).
        let mut p: Vec<f64> = (0..100).map(|i| (i as f64 * 0.07).sin() * 0.01).collect();
        let m: Vec<f64> = (0..100).map(|i| (i as f64 * 0.07).cos() * 0.01).collect();
        let s: Vec<f64> = (0..100).map(|i| (i as f64 * 0.13).sin() * 0.005).collect();
        let h: Vec<f64> = (0..100).map(|i| (i as f64 * 0.11).cos() * 0.005).collect();
        let rf = vec![0.0; 100];
        p[20] = f64::NAN;
        let inputs = Ff3Inputs {
            portfolio_returns: &p, market_excess: &m, smb: &s, hml: &h, risk_free: &rf,
        };
        let report = ff3(&inputs).unwrap();
        assert!(report.n_observations < 100);
    }
}
