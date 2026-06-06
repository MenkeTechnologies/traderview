//! Treynor-Mazuy (1966) market-timing skill test.
//!
//! Regression:
//!   r_p − r_f = α + β · (r_m − r_f) + γ · (r_m − r_f)² + ε
//!
//! γ > 0 (and statistically significant) = market-timing skill: the
//! manager increases beta when the market rallies and reduces it on
//! downturns (convex payoff vs market). γ < 0 = anti-timing.
//!
//! Returns the three coefficients, their standard errors, and the
//! t-statistic for γ (the timing parameter — the headline reading).
//!
//! Pure compute. OLS via the same Gauss-Jordan + cov(β) routine used
//! elsewhere; no external linear-algebra dependency.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmReport {
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
    pub alpha_se: f64,
    pub beta_se: f64,
    pub gamma_se: f64,
    pub gamma_tstat: f64,
    pub r_squared: f64,
    pub n_observations: usize,
}

pub fn analyze(
    portfolio_returns: &[f64],
    market_returns: &[f64],
    risk_free_returns: &[f64],
) -> Option<TmReport> {
    let n = portfolio_returns.len();
    if market_returns.len() != n || risk_free_returns.len() != n || n < 10 {
        return None;
    }
    let mut y = Vec::with_capacity(n);
    let mut xm = Vec::with_capacity(n);
    let mut xm2 = Vec::with_capacity(n);
    for i in 0..n {
        let p = portfolio_returns[i];
        let m = market_returns[i];
        let rf = risk_free_returns[i];
        if !p.is_finite() || !m.is_finite() || !rf.is_finite() {
            continue;
        }
        let mer = m - rf;
        y.push(p - rf);
        xm.push(mer);
        xm2.push(mer * mer);
    }
    let n_obs = y.len();
    if n_obs < 10 {
        return None;
    }
    let ones: Vec<f64> = vec![1.0; n_obs];
    let cols = vec![ones, xm, xm2];
    let (beta, se) = ols_with_se(&cols, &y)?;
    if beta.len() != 3 || se.len() != 3 {
        return None;
    }
    let alpha = beta[0];
    let beta_coef = beta[1];
    let gamma = beta[2];
    let alpha_se = se[0];
    let beta_se = se[1];
    let gamma_se = se[2];
    let tstat = if gamma_se > 0.0 {
        gamma / gamma_se
    } else {
        0.0
    };
    // R²: 1 − SS_res / SS_tot.
    let y_mean: f64 = y.iter().sum::<f64>() / n_obs as f64;
    let mut ss_tot = 0.0;
    let mut ss_res = 0.0;
    for k in 0..n_obs {
        let pred = alpha + beta_coef * cols[1][k] + gamma * cols[2][k];
        ss_tot += (y[k] - y_mean).powi(2);
        ss_res += (y[k] - pred).powi(2);
    }
    let r2 = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };
    Some(TmReport {
        alpha,
        beta: beta_coef,
        gamma,
        alpha_se,
        beta_se,
        gamma_se,
        gamma_tstat: tstat,
        r_squared: r2,
        n_observations: n_obs,
    })
}

fn ols_with_se(x: &[Vec<f64>], y: &[f64]) -> Option<(Vec<f64>, Vec<f64>)> {
    let p = x.len();
    let n = y.len();
    if p == 0 || n == 0 || x.iter().any(|c| c.len() != n) {
        return None;
    }
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
        for r in 0..p {
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
    fn dim_mismatch_returns_none() {
        let p = vec![0.01; 50];
        let m = vec![0.01; 50];
        let rf = vec![0.001; 25];
        assert!(analyze(&p, &m, &rf).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        let p = vec![0.01; 5];
        let m = vec![0.01; 5];
        let rf = vec![0.001; 5];
        assert!(analyze(&p, &m, &rf).is_none());
    }

    #[test]
    fn perfect_market_tracker_yields_zero_gamma() {
        // Portfolio = market exactly → α = 0, β = 1, γ = 0.
        let m: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin() * 0.02).collect();
        let rf = vec![0.0001; 100];
        let p = m.clone();
        let report = analyze(&p, &m, &rf).unwrap();
        assert!(
            (report.beta - 1.0).abs() < 0.01,
            "beta should be ≈ 1.0, got {}",
            report.beta
        );
        assert!(
            report.gamma.abs() < 0.05,
            "gamma should be ≈ 0 for pure tracker, got {}",
            report.gamma
        );
    }

    #[test]
    fn convex_timing_strategy_yields_positive_gamma() {
        // Synthetic timer: r_p − rf = (r_m − rf) + 5 · (r_m − rf)² → big positive γ.
        let m: Vec<f64> = (0..200).map(|i| (i as f64 * 0.07).sin() * 0.03).collect();
        let rf = vec![0.00005; 200];
        let p: Vec<f64> = m
            .iter()
            .zip(rf.iter())
            .map(|(mi, rfi)| {
                let mer = mi - rfi;
                rfi + mer + 5.0 * mer * mer
            })
            .collect();
        let report = analyze(&p, &m, &rf).unwrap();
        assert!(
            report.gamma > 1.0,
            "expected large positive gamma, got {}",
            report.gamma
        );
        assert!(
            report.gamma_tstat > 2.0,
            "expected significant t-stat, got {}",
            report.gamma_tstat
        );
    }

    #[test]
    fn anti_timing_strategy_yields_negative_gamma() {
        let m: Vec<f64> = (0..200).map(|i| (i as f64 * 0.07).sin() * 0.03).collect();
        let rf = vec![0.00005; 200];
        let p: Vec<f64> = m
            .iter()
            .zip(rf.iter())
            .map(|(mi, rfi)| {
                let mer = mi - rfi;
                rfi + mer - 5.0 * mer * mer
            })
            .collect();
        let report = analyze(&p, &m, &rf).unwrap();
        assert!(
            report.gamma < -1.0,
            "expected negative gamma, got {}",
            report.gamma
        );
    }

    #[test]
    fn nan_observations_skipped() {
        let mut p: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        let mut m = p.clone();
        let rf = vec![0.0; 100];
        p[50] = f64::NAN;
        m[51] = f64::NAN;
        let report = analyze(&p, &m, &rf).unwrap();
        assert!(report.n_observations < 100);
        assert!(report.beta.is_finite());
    }
}
