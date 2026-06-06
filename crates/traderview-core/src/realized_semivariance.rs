//! Realized semivariance — Barndorff-Nielsen / Kinnebrock / Shephard
//! (2008), Patton-Sheppard (2009).
//!
//! Decomposes realized variance into downside (RS⁻) and upside (RS⁺)
//! components:
//!
//!   RS⁻ = Σ r_i² · 1{r_i < 0}
//!   RS⁺ = Σ r_i² · 1{r_i > 0}
//!   RV  = RS⁻ + RS⁺
//!
//! Skewness signal:
//!   - RS⁻ ≫ RS⁺ → crash regime (downside risk dominates)
//!   - RS⁺ ≫ RS⁻ → rally regime
//!   - Patton-Sheppard showed RS⁻ has stronger predictive power for
//!     future volatility than RV (downside surprises more persistent).
//!
//! Pure compute. Companion to `realized_volatility`,
//! `realized_correlation`, `realized_higher_moments`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SemivarianceReport {
    pub realized_variance: f64,
    pub realized_semivariance_negative: f64,
    pub realized_semivariance_positive: f64,
    /// Annualized downside vol (assumes `periods_per_year` provided).
    pub downside_volatility_annualized: f64,
    pub upside_volatility_annualized: f64,
    /// RS⁻ / RV — fraction of variance from downside moves.
    pub downside_share: f64,
    pub n_observations: usize,
}

pub fn compute(returns: &[f64], periods_per_year: f64) -> Option<SemivarianceReport> {
    if returns.is_empty() || !periods_per_year.is_finite() || periods_per_year <= 0.0 {
        return None;
    }
    let mut neg = 0.0_f64;
    let mut pos = 0.0_f64;
    let mut n = 0_usize;
    for r in returns {
        if !r.is_finite() {
            continue;
        }
        let sq = r * r;
        if *r < 0.0 {
            neg += sq;
        } else if *r > 0.0 {
            pos += sq;
        }
        n += 1;
    }
    if n == 0 {
        return None;
    }
    let rv = neg + pos;
    let downside_share = if rv > 0.0 { neg / rv } else { 0.0 };
    let n_f = n as f64;
    let downside_vol_ann = (neg / n_f * periods_per_year).max(0.0).sqrt();
    let upside_vol_ann = (pos / n_f * periods_per_year).max(0.0).sqrt();
    Some(SemivarianceReport {
        realized_variance: rv,
        realized_semivariance_negative: neg,
        realized_semivariance_positive: pos,
        downside_volatility_annualized: downside_vol_ann,
        upside_volatility_annualized: upside_vol_ann,
        downside_share,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[], 252.0).is_none());
    }

    #[test]
    fn invalid_freq_returns_none() {
        assert!(compute(&[0.01, -0.02], 0.0).is_none());
        assert!(compute(&[0.01, -0.02], f64::NAN).is_none());
    }

    #[test]
    fn all_nan_returns_none() {
        assert!(compute(&[f64::NAN, f64::NAN], 252.0).is_none());
    }

    #[test]
    fn symmetric_returns_yield_balanced_decomposition() {
        let returns = vec![0.01, -0.01, 0.02, -0.02, 0.005, -0.005];
        let r = compute(&returns, 252.0).unwrap();
        assert!(
            (r.realized_semivariance_negative - r.realized_semivariance_positive).abs() < 1e-12
        );
        assert!((r.downside_share - 0.5).abs() < 1e-12);
    }

    #[test]
    fn all_negative_yields_zero_upside() {
        let returns = vec![-0.01, -0.02, -0.005];
        let r = compute(&returns, 252.0).unwrap();
        assert_eq!(r.realized_semivariance_positive, 0.0);
        assert!(r.realized_semivariance_negative > 0.0);
        assert_eq!(r.downside_share, 1.0);
    }

    #[test]
    fn all_positive_yields_zero_downside() {
        let returns = vec![0.01, 0.02, 0.005];
        let r = compute(&returns, 252.0).unwrap();
        assert_eq!(r.realized_semivariance_negative, 0.0);
        assert!(r.realized_semivariance_positive > 0.0);
        assert_eq!(r.downside_share, 0.0);
    }

    #[test]
    fn zero_returns_contribute_nothing() {
        let returns = vec![0.0, 0.0, 0.01, -0.01];
        let r = compute(&returns, 252.0).unwrap();
        assert!((r.realized_semivariance_negative - 0.0001).abs() < 1e-15);
        assert!((r.realized_semivariance_positive - 0.0001).abs() < 1e-15);
        assert_eq!(r.n_observations, 4);
    }

    #[test]
    fn annualized_vol_correct_for_uniform_negatives() {
        // 252 daily returns of -1% each: downside var per day = 0.0001;
        // annualized = 0.0001 · 252; vol = sqrt of that.
        let returns = vec![-0.01_f64; 252];
        let r = compute(&returns, 252.0).unwrap();
        let expected = (0.0001_f64 * 252.0).sqrt();
        assert!((r.downside_volatility_annualized - expected).abs() < 1e-12);
    }
}
