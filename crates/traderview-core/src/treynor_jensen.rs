//! Treynor ratio, Jensen's alpha, and Modigliani (M²) — the classic
//! systematic-risk-adjusted performance trio. Distinct from Treynor-
//! Mazuy (timing test) and the Sharpe family (total-risk-adjusted).
//!
//! Inputs are aligned portfolio / market / risk-free return series. We
//! first regress excess portfolio returns on excess market returns to
//! get α and β (via the usual closed-form OLS), then:
//!
//!   treynor_ratio   = (mean_excess_p) / β               (return per unit β)
//!   jensen_alpha    = α                                  (CAPM intercept)
//!   m2              = mean_excess_p · (σ_m / σ_p) + r_f  ("M²-equivalent")
//!
//! where σ_m and σ_p are the sample stdevs of market and portfolio
//! returns. M² re-scales the portfolio so its volatility matches the
//! benchmark, then reports the equivalent return.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub treynor_ratio: f64,
    pub jensen_alpha: f64,
    pub m_squared: f64,
    pub beta: f64,
    pub mean_excess_portfolio: f64,
    pub mean_excess_market: f64,
    pub stdev_portfolio: f64,
    pub stdev_market: f64,
    pub n_observations: usize,
}

pub fn compute(
    portfolio_returns: &[f64],
    market_returns: &[f64],
    risk_free_returns: &[f64],
) -> Option<PerformanceReport> {
    let n = portfolio_returns.len();
    if market_returns.len() != n || risk_free_returns.len() != n || n < 4 {
        return None;
    }
    let mut ep = Vec::with_capacity(n);
    let mut em = Vec::with_capacity(n);
    let mut p_raw = Vec::with_capacity(n);
    let mut m_raw = Vec::with_capacity(n);
    let mut rf_acc = 0.0_f64;
    let mut rf_count = 0_usize;
    for i in 0..n {
        let p = portfolio_returns[i];
        let m = market_returns[i];
        let rf = risk_free_returns[i];
        if !p.is_finite() || !m.is_finite() || !rf.is_finite() {
            continue;
        }
        ep.push(p - rf);
        em.push(m - rf);
        p_raw.push(p);
        m_raw.push(m);
        rf_acc += rf;
        rf_count += 1;
    }
    let n_obs = ep.len();
    if n_obs < 4 {
        return None;
    }
    let n_f = n_obs as f64;
    let mean_ep = ep.iter().sum::<f64>() / n_f;
    let mean_em = em.iter().sum::<f64>() / n_f;
    let mean_rf = rf_acc / rf_count.max(1) as f64;
    // β = Cov(ep, em) / Var(em).
    let cov: f64 = ep
        .iter()
        .zip(em.iter())
        .map(|(a, b)| (a - mean_ep) * (b - mean_em))
        .sum::<f64>()
        / (n_f - 1.0);
    let var_em: f64 = em.iter().map(|x| (x - mean_em).powi(2)).sum::<f64>() / (n_f - 1.0);
    // Relative tolerance: identical-input float accumulation can leave
    // a residual variance below ~1e-30 even for "constant" series.
    let flat_threshold = mean_em.abs() * mean_em.abs() * 1e-20 + f64::EPSILON;
    if var_em <= flat_threshold {
        return None;
    }
    let beta = cov / var_em;
    if !beta.is_finite() {
        return None;
    }
    let alpha = mean_ep - beta * mean_em;
    let stdev_p = (p_raw
        .iter()
        .map(|x| {
            let m = p_raw.iter().sum::<f64>() / n_f;
            (x - m).powi(2)
        })
        .sum::<f64>()
        / (n_f - 1.0))
        .max(0.0)
        .sqrt();
    let stdev_m = (m_raw
        .iter()
        .map(|x| {
            let m = m_raw.iter().sum::<f64>() / n_f;
            (x - m).powi(2)
        })
        .sum::<f64>()
        / (n_f - 1.0))
        .max(0.0)
        .sqrt();
    let treynor = if beta != 0.0 {
        mean_ep / beta
    } else {
        f64::NAN
    };
    let m_sq = if stdev_p > 0.0 {
        mean_ep * (stdev_m / stdev_p) + mean_rf
    } else {
        f64::NAN
    };
    Some(PerformanceReport {
        treynor_ratio: treynor,
        jensen_alpha: alpha,
        m_squared: m_sq,
        beta,
        mean_excess_portfolio: mean_ep,
        mean_excess_market: mean_em,
        stdev_portfolio: stdev_p,
        stdev_market: stdev_m,
        n_observations: n_obs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_mismatch_returns_none() {
        let p = vec![0.01; 50];
        let m = vec![0.01; 25];
        let rf = vec![0.001; 25];
        assert!(compute(&p, &m, &rf).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        let p = vec![0.01; 3];
        assert!(compute(&p, &p, &p).is_none());
    }

    #[test]
    fn flat_market_returns_none() {
        let p: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        let m = vec![0.005; 50];
        let rf = vec![0.001; 50];
        // var_em = 0 → None.
        assert!(compute(&p, &m, &rf).is_none());
    }

    #[test]
    fn portfolio_identical_to_market_yields_beta_one_and_zero_alpha() {
        let m: Vec<f64> = (0..200).map(|i| (i as f64 * 0.07).sin() * 0.02).collect();
        let rf = vec![0.0001; 200];
        let p = m.clone();
        let r = compute(&p, &m, &rf).unwrap();
        assert!((r.beta - 1.0).abs() < 1e-9);
        assert!(r.jensen_alpha.abs() < 1e-9);
    }

    #[test]
    fn levered_portfolio_yields_beta_above_one() {
        let m: Vec<f64> = (0..200).map(|i| (i as f64 * 0.07).sin() * 0.02).collect();
        let rf = vec![0.0001; 200];
        let p: Vec<f64> = m.iter().map(|x| 2.0 * x).collect();
        let r = compute(&p, &m, &rf).unwrap();
        assert!((r.beta - 2.0).abs() < 0.01);
    }

    #[test]
    fn positive_alpha_strategy_yields_positive_jensen() {
        let m: Vec<f64> = (0..200).map(|i| (i as f64 * 0.07).sin() * 0.02).collect();
        let rf = vec![0.0001; 200];
        // Portfolio = market + constant 1bp/period alpha.
        let p: Vec<f64> = m.iter().map(|x| x + 0.0001).collect();
        let r = compute(&p, &m, &rf).unwrap();
        assert!(r.jensen_alpha > 0.0);
    }

    #[test]
    fn nan_observations_skipped() {
        let mut p: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        let m = p.clone();
        let rf = vec![0.0; 100];
        p[50] = f64::NAN;
        let r = compute(&p, &m, &rf).unwrap();
        assert!(r.n_observations < 100);
    }

    #[test]
    fn m_squared_finite_for_well_defined_inputs() {
        let m: Vec<f64> = (0..100).map(|i| (i as f64 * 0.07).sin() * 0.02).collect();
        let rf = vec![0.0001; 100];
        let p = m.clone();
        let r = compute(&p, &m, &rf).unwrap();
        assert!(r.m_squared.is_finite());
    }
}
