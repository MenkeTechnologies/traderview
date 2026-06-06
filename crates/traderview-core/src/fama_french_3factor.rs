//! Fama-French 3-Factor Model — Eugene Fama & Kenneth French (1993).
//!
//! Regresses an asset's excess returns on three explanatory factors:
//!
//!   r_t − rf_t = α + β_MKT · (r_M − rf) + β_SMB · SMB + β_HML · HML + ε_t
//!
//! where:
//!   - MKT = market excess return (R_M − R_f)
//!   - SMB = "Small Minus Big" (small-cap excess return over big-cap)
//!   - HML = "High Minus Low" (value/high-B-M minus growth/low-B-M)
//!
//! Outputs:
//!   - α (Jensen-style intercept, "abnormal" return)
//!   - β_MKT, β_SMB, β_HML loadings
//!   - R², adjusted R², residual std error
//!   - per-coefficient t-statistics
//!
//! Pure compute. OLS via normal equations on a 4-column design matrix.
//! Companion to `factor_models`, `henriksson_merton`, `treynor_mazuy`,
//! `beta_shrinkage`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FamaFrenchReport {
    pub alpha: f64,
    pub beta_market: f64,
    pub beta_smb: f64,
    pub beta_hml: f64,
    pub r_squared: f64,
    pub adjusted_r_squared: f64,
    pub residual_std_error: f64,
    pub t_alpha: f64,
    pub t_beta_market: f64,
    pub t_beta_smb: f64,
    pub t_beta_hml: f64,
    pub n_observations: usize,
}

pub fn estimate(
    excess_returns: &[f64],
    mkt: &[f64],
    smb: &[f64],
    hml: &[f64],
) -> Option<FamaFrenchReport> {
    let n = excess_returns.len();
    if n < 8 || mkt.len() != n || smb.len() != n || hml.len() != n {
        return None;
    }
    if excess_returns.iter().any(|x| !x.is_finite())
        || mkt.iter().any(|x| !x.is_finite())
        || smb.iter().any(|x| !x.is_finite())
        || hml.iter().any(|x| !x.is_finite())
    {
        return None;
    }
    // X = [1, MKT, SMB, HML], Y = excess_returns. Solve (X'X) β = X'y.
    let p = 4_usize;
    let mut xtx = vec![vec![0.0_f64; p]; p];
    let mut xty = vec![0.0_f64; p];
    for i in 0..n {
        let row = [1.0, mkt[i], smb[i], hml[i]];
        for j in 0..p {
            for k in 0..p {
                xtx[j][k] += row[j] * row[k];
            }
            xty[j] += row[j] * excess_returns[i];
        }
    }
    let coef = solve_linear(&xtx, &xty)?;
    let alpha = coef[0];
    let beta_m = coef[1];
    let beta_smb = coef[2];
    let beta_hml = coef[3];
    // Fitted values + residuals.
    let mut ssr = 0.0_f64;
    let mut tss = 0.0_f64;
    let y_mean = excess_returns.iter().sum::<f64>() / n as f64;
    for i in 0..n {
        let yhat = alpha + beta_m * mkt[i] + beta_smb * smb[i] + beta_hml * hml[i];
        let resid = excess_returns[i] - yhat;
        ssr += resid * resid;
        tss += (excess_returns[i] - y_mean).powi(2);
    }
    let dof = (n - p) as f64;
    if dof <= 0.0 {
        return None;
    }
    let sigma2 = ssr / dof;
    let se = sigma2.sqrt();
    let r_sq = if tss > 0.0 { 1.0 - ssr / tss } else { 0.0 };
    let adj_r_sq = if dof > 0.0 && tss > 0.0 {
        1.0 - (1.0 - r_sq) * (n - 1) as f64 / dof
    } else {
        0.0
    };
    // Standard errors from σ² · (X'X)⁻¹ diagonal.
    let xtx_inv = invert_4x4(&xtx)?;
    let se_alpha = (sigma2 * xtx_inv[0][0]).max(0.0).sqrt();
    let se_beta_m = (sigma2 * xtx_inv[1][1]).max(0.0).sqrt();
    let se_beta_smb = (sigma2 * xtx_inv[2][2]).max(0.0).sqrt();
    let se_beta_hml = (sigma2 * xtx_inv[3][3]).max(0.0).sqrt();
    let t_div = |coef: f64, se: f64| if se > 0.0 { coef / se } else { 0.0 };
    Some(FamaFrenchReport {
        alpha,
        beta_market: beta_m,
        beta_smb,
        beta_hml,
        r_squared: r_sq,
        adjusted_r_squared: adj_r_sq,
        residual_std_error: se,
        t_alpha: t_div(alpha, se_alpha),
        t_beta_market: t_div(beta_m, se_beta_m),
        t_beta_smb: t_div(beta_smb, se_beta_smb),
        t_beta_hml: t_div(beta_hml, se_beta_hml),
        n_observations: n,
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

fn invert_4x4(m: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = 4;
    let mut aug = vec![vec![0.0_f64; 2 * n]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = m[i][j];
        }
        aug[i][n + i] = 1.0;
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
    let mut inv = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            inv[i][j] = aug[i][n + j];
        }
    }
    Some(inv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        let v = vec![0.01_f64; 5];
        assert!(estimate(&v, &v, &v, &v).is_none());
    }

    #[test]
    fn mismatched_lengths_return_none() {
        let v = vec![0.01_f64; 20];
        let short = vec![0.01_f64; 10];
        assert!(estimate(&v, &short, &v, &v).is_none());
    }

    #[test]
    fn nan_inputs_return_none() {
        let v = vec![0.01_f64; 20];
        let mut bad = vec![0.01_f64; 20];
        bad[5] = f64::NAN;
        assert!(estimate(&v, &bad, &v, &v).is_none());
    }

    #[test]
    fn synthetic_data_recovers_known_alpha_and_betas() {
        // True model: r = 0.001 + 0.8·MKT + 0.3·SMB − 0.2·HML + ε
        let mut state: u64 = 11;
        let n = 200;
        let mkt: Vec<f64> = (0..n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.04
            })
            .collect();
        let smb: Vec<f64> = (0..n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.02
            })
            .collect();
        let hml: Vec<f64> = (0..n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.02
            })
            .collect();
        let eps: Vec<f64> = (0..n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.005
            })
            .collect();
        let ret: Vec<f64> = (0..n)
            .map(|i| 0.001 + 0.8 * mkt[i] + 0.3 * smb[i] - 0.2 * hml[i] + eps[i])
            .collect();
        let r = estimate(&ret, &mkt, &smb, &hml).unwrap();
        assert!((r.alpha - 0.001).abs() < 0.001, "alpha {}", r.alpha);
        assert!(
            (r.beta_market - 0.8).abs() < 0.1,
            "beta_mkt {}",
            r.beta_market
        );
        assert!((r.beta_smb - 0.3).abs() < 0.2, "beta_smb {}", r.beta_smb);
        assert!((r.beta_hml + 0.2).abs() < 0.2, "beta_hml {}", r.beta_hml);
        assert!(r.r_squared > 0.5, "r² {}", r.r_squared);
    }

    #[test]
    fn perfect_fit_yields_r_squared_one() {
        let mkt = vec![0.01, 0.02, -0.01, 0.03, -0.02, 0.015, 0.0, -0.005];
        let smb = vec![0.005, -0.01, 0.0, 0.02, -0.015, 0.01, 0.005, -0.01];
        let hml = vec![-0.005, 0.01, 0.005, -0.01, 0.02, -0.01, 0.0, 0.005];
        let ret: Vec<f64> = mkt
            .iter()
            .zip(smb.iter())
            .zip(hml.iter())
            .map(|((m, s), h)| 0.5 * m + 0.3 * s - 0.2 * h)
            .collect();
        let r = estimate(&ret, &mkt, &smb, &hml).unwrap();
        assert!((r.r_squared - 1.0).abs() < 1e-9);
        assert!(r.residual_std_error < 1e-9);
    }

    #[test]
    fn r_squared_in_unit_range() {
        let v: Vec<f64> = (0..30).map(|i| (i as f64).sin() * 0.01).collect();
        let m: Vec<f64> = (0..30).map(|i| (i as f64).cos() * 0.02).collect();
        let s: Vec<f64> = (0..30).map(|i| (i as f64 * 0.5).sin() * 0.01).collect();
        let h: Vec<f64> = (0..30).map(|i| (i as f64 * 0.3).cos() * 0.01).collect();
        let r = estimate(&v, &m, &s, &h).unwrap();
        assert!((0.0..=1.0).contains(&r.r_squared));
    }

    #[test]
    fn flat_market_factor_returns_none() {
        let r = vec![0.01_f64; 20];
        let flat = vec![0.0_f64; 20];
        // X'X is singular when any column is identically zero.
        assert!(estimate(&r, &flat, &flat, &flat).is_none());
    }
}
