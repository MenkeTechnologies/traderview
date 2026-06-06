//! Vasicek Bayesian Beta Shrinkage (Vasicek 1973, "A Note on the
//! Cross-Sectional Information").
//!
//! Raw OLS beta from a short return series is noisy. Vasicek showed
//! the Bayes-optimal estimator shrinks toward the cross-sectional
//! prior beta β̄ (typically 1.0 for equity universes):
//!
//!   β̂ = w · β_OLS + (1 − w) · β̄
//!
//!   w = σ²_cs / (σ²_cs + se²_OLS)
//!
//! where:
//!   - β_OLS is the per-asset OLS slope vs the market
//!   - se_OLS is the standard error of β_OLS
//!   - β̄ is the cross-sectional mean of OLS betas (≈ 1.0 by construction
//!     for cap-weighted universes)
//!   - σ²_cs is the cross-sectional variance of OLS betas
//!
//! Interpretation: assets with high OLS standard error get pulled
//! strongly toward β̄; assets with low se stay near their OLS slope.
//! Reduces estimation error and improves out-of-sample forecast.
//!
//! Pure compute. Returns per-asset shrunk beta + shrinkage weight.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetReturns {
    pub symbol: String,
    pub asset_returns: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShrunkBeta {
    pub symbol: String,
    pub beta_ols: f64,
    pub standard_error: f64,
    pub shrinkage_weight: f64,
    pub beta_shrunk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShrinkageReport {
    pub prior_beta: f64,
    pub cross_sectional_variance: f64,
    pub assets: Vec<ShrunkBeta>,
}

pub fn shrink(assets: &[AssetReturns], market_returns: &[f64]) -> Option<ShrinkageReport> {
    if assets.is_empty() || market_returns.len() < 5 {
        return None;
    }
    if market_returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n_obs = market_returns.len() as f64;
    let mut ols: Vec<(String, f64, f64)> = Vec::with_capacity(assets.len());
    for a in assets {
        if a.asset_returns.len() != market_returns.len() {
            continue;
        }
        if a.asset_returns.iter().any(|x| !x.is_finite()) {
            continue;
        }
        if let Some((beta, se)) = ols_beta(&a.asset_returns, market_returns) {
            ols.push((a.symbol.clone(), beta, se));
        }
    }
    if ols.is_empty() {
        return None;
    }
    // Cross-sectional prior + variance (across assets, not across time).
    let prior_beta: f64 = ols.iter().map(|(_, b, _)| *b).sum::<f64>() / ols.len() as f64;
    let cs_var: f64 = if ols.len() > 1 {
        ols.iter()
            .map(|(_, b, _)| (b - prior_beta).powi(2))
            .sum::<f64>()
            / (ols.len() - 1) as f64
    } else {
        0.0
    };
    let mut report = Vec::with_capacity(ols.len());
    for (sym, beta, se) in ols {
        let var_ols = se * se;
        let denom = cs_var + var_ols;
        let w = if denom > 0.0 { cs_var / denom } else { 0.0 };
        let shrunk = w * beta + (1.0 - w) * prior_beta;
        report.push(ShrunkBeta {
            symbol: sym,
            beta_ols: beta,
            standard_error: se,
            shrinkage_weight: w,
            beta_shrunk: shrunk,
        });
    }
    let _ = n_obs;
    Some(ShrinkageReport {
        prior_beta,
        cross_sectional_variance: cs_var,
        assets: report,
    })
}

fn ols_beta(y: &[f64], x: &[f64]) -> Option<(f64, f64)> {
    let n = y.len();
    if n < 5 || x.len() != n {
        return None;
    }
    let n_f = n as f64;
    let x_mean: f64 = x.iter().sum::<f64>() / n_f;
    let y_mean: f64 = y.iter().sum::<f64>() / n_f;
    let mut s_xy = 0.0;
    let mut s_xx = 0.0;
    for i in 0..n {
        let dx = x[i] - x_mean;
        let dy = y[i] - y_mean;
        s_xy += dx * dy;
        s_xx += dx * dx;
    }
    if s_xx <= 0.0 {
        return None;
    }
    let beta = s_xy / s_xx;
    let alpha = y_mean - beta * x_mean;
    // Residual variance → SE(β).
    let mut ssr = 0.0;
    for i in 0..n {
        let resid = y[i] - alpha - beta * x[i];
        ssr += resid * resid;
    }
    let dof = (n - 2) as f64;
    if dof <= 0.0 {
        return None;
    }
    let sigma2 = ssr / dof;
    let se = (sigma2 / s_xx).max(0.0).sqrt();
    Some((beta, se))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(sym: &str, r: Vec<f64>) -> AssetReturns {
        AssetReturns {
            symbol: sym.into(),
            asset_returns: r,
        }
    }

    #[test]
    fn empty_returns_none() {
        let m = vec![0.01; 10];
        assert!(shrink(&[], &m).is_none());
        let one = vec![a("X", vec![0.01; 10])];
        assert!(shrink(&one, &[0.01; 3]).is_none());
    }

    #[test]
    fn nan_market_returns_none() {
        let m = vec![0.01, f64::NAN, 0.02, 0.01, 0.02];
        let one = vec![a("X", vec![0.01, 0.02, 0.01, 0.02, 0.01])];
        assert!(shrink(&one, &m).is_none());
    }

    #[test]
    fn shrinkage_weight_in_unit_range() {
        let m: Vec<f64> = (0..30).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        let assets = vec![
            a("A", m.iter().map(|x| x * 1.2 + 0.001).collect()),
            a("B", m.iter().map(|x| x * 0.8 + 0.0005).collect()),
            a("C", m.iter().map(|x| x * 1.0 + 0.002).collect()),
        ];
        let r = shrink(&assets, &m).unwrap();
        for s in &r.assets {
            assert!((0.0..=1.0).contains(&s.shrinkage_weight));
        }
    }

    #[test]
    fn high_standard_error_shrinks_more() {
        // Build market and two assets: one tracks market tightly (low se),
        // one is mostly noise (high se).
        let mut state: u64 = 7;
        let mut rand = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 32) as f64 / u32::MAX as f64) - 0.5
        };
        let m: Vec<f64> = (0..100).map(|_| rand() * 0.02).collect();
        let tight: Vec<f64> = m.iter().map(|x| x * 1.5 + rand() * 0.0005).collect();
        let noisy: Vec<f64> = m.iter().map(|x| x * 1.5 + rand() * 0.05).collect();
        let assets = vec![a("TIGHT", tight), a("NOISY", noisy)];
        let r = shrink(&assets, &m).unwrap();
        let tight_w = r
            .assets
            .iter()
            .find(|s| s.symbol == "TIGHT")
            .unwrap()
            .shrinkage_weight;
        let noisy_w = r
            .assets
            .iter()
            .find(|s| s.symbol == "NOISY")
            .unwrap()
            .shrinkage_weight;
        assert!(
            tight_w > noisy_w,
            "tight-fit asset should retain more OLS weight ({tight_w}) than noisy ({noisy_w})"
        );
    }

    #[test]
    fn beta_shrunk_between_ols_and_prior() {
        let m: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        let assets = vec![
            a("HIGH", m.iter().map(|x| x * 2.0).collect()),
            a("LOW", m.iter().map(|x| x * 0.5).collect()),
            a("MID", m.iter().map(|x| x * 1.0).collect()),
        ];
        let r = shrink(&assets, &m).unwrap();
        for s in &r.assets {
            let lo = s.beta_ols.min(r.prior_beta);
            let hi = s.beta_ols.max(r.prior_beta);
            assert!(
                s.beta_shrunk >= lo - 1e-9 && s.beta_shrunk <= hi + 1e-9,
                "{} shrunk {} outside [{}, {}]",
                s.symbol,
                s.beta_shrunk,
                lo,
                hi
            );
        }
    }

    #[test]
    fn mismatched_lengths_skipped() {
        let m = vec![0.01_f64; 20];
        let good = a("OK", vec![0.01; 20]);
        let bad = a("BAD", vec![0.01; 10]);
        let r = shrink(&[good, bad], &m).unwrap();
        assert_eq!(r.assets.len(), 1);
    }

    #[test]
    fn flat_market_returns_none() {
        let m = vec![0.0_f64; 20];
        let one = vec![a("X", vec![0.01; 20])];
        // OLS slope undefined when market has no variance.
        assert!(shrink(&one, &m).is_none());
    }
}
