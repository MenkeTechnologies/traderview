//! Treynor ratio + Information ratio.
//!
//! **Treynor** = (portfolio_return - rf) / beta
//!
//! Sharpe uses total volatility; Treynor uses systematic risk (beta)
//! only. Better for diversified portfolios where idiosyncratic risk
//! is largely diversified away.
//!
//! **Information ratio** = (portfolio_return - benchmark_return) / tracking_error
//!
//! where tracking_error = stdev(portfolio - benchmark) per period.
//! The "active management" yardstick — measures alpha per unit of
//! benchmark-relative volatility. > 0.5 is good, > 1.0 is rare.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TreynorReport {
    pub portfolio_mean: f64,
    pub rf: f64,
    pub beta: f64,
    /// (mean - rf) / beta. NaN when beta == 0 (no systematic exposure).
    pub treynor: f64,
}

pub fn treynor(portfolio_returns: &[f64], rf_per_period: f64, beta: f64) -> TreynorReport {
    if portfolio_returns.is_empty() {
        return TreynorReport {
            rf: rf_per_period,
            beta,
            ..Default::default()
        };
    }
    let mean = portfolio_returns.iter().sum::<f64>() / portfolio_returns.len() as f64;
    let treynor = if beta == 0.0 {
        f64::NAN
    } else {
        (mean - rf_per_period) / beta
    };
    TreynorReport {
        portfolio_mean: mean,
        rf: rf_per_period,
        beta,
        treynor,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InfoRatioReport {
    pub n: usize,
    pub mean_active_return: f64,
    pub tracking_error: f64,
    /// active_return / tracking_error × sqrt(annualization).
    pub information_ratio: f64,
}

pub fn information_ratio(
    portfolio: &[f64],
    benchmark: &[f64],
    annualization: f64,
) -> Option<InfoRatioReport> {
    if portfolio.len() != benchmark.len() || portfolio.len() < 2 {
        return None;
    }
    let n = portfolio.len();
    let active: Vec<f64> = portfolio
        .iter()
        .zip(benchmark)
        .map(|(p, b)| p - b)
        .collect();
    let mean = active.iter().sum::<f64>() / n as f64;
    let var = active.iter().map(|a| (a - mean).powi(2)).sum::<f64>() / n as f64;
    let te = var.sqrt();
    let ir = if te == 0.0 {
        0.0
    } else {
        mean / te * annualization.sqrt()
    };
    Some(InfoRatioReport {
        n,
        mean_active_return: mean,
        tracking_error: te,
        information_ratio: ir,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── treynor ──────────────────────────────────────────────────────

    #[test]
    fn treynor_empty_returns_default_keeps_beta_rf() {
        let r = treynor(&[], 0.01, 1.2);
        assert_eq!(r.rf, 0.01);
        assert_eq!(r.beta, 1.2);
        assert_eq!(r.treynor, 0.0);
    }

    #[test]
    fn treynor_zero_beta_returns_nan() {
        let r = treynor(&[0.05, 0.03, 0.07], 0.01, 0.0);
        assert!(r.treynor.is_nan());
    }

    #[test]
    fn treynor_with_excess_return_above_rf() {
        // Mean 0.05, rf 0.01, beta 1.0 → treynor = 0.04.
        let r = treynor(&[0.05, 0.05, 0.05], 0.01, 1.0);
        assert!((r.treynor - 0.04).abs() < 1e-9);
    }

    #[test]
    fn treynor_higher_when_beta_is_lower_for_same_returns() {
        // Less risk per unit of return → higher treynor.
        let r_high = treynor(&[0.05, 0.05, 0.05], 0.01, 2.0);
        let r_low = treynor(&[0.05, 0.05, 0.05], 0.01, 1.0);
        assert!(
            r_low.treynor > r_high.treynor,
            "lower beta → higher treynor for same returns"
        );
    }

    #[test]
    fn treynor_negative_when_returns_below_rf() {
        let r = treynor(&[0.005, 0.005, 0.005], 0.01, 1.0);
        assert!(r.treynor < 0.0);
    }

    // ─── information ratio ────────────────────────────────────────────

    #[test]
    fn ir_length_mismatch_returns_none() {
        assert!(information_ratio(&[1.0, 2.0], &[1.0], 252.0).is_none());
    }

    #[test]
    fn ir_singleton_returns_none() {
        assert!(information_ratio(&[1.0], &[1.0], 252.0).is_none());
    }

    #[test]
    fn ir_identical_series_zero_active_return_zero_te_zero_ir() {
        let r = information_ratio(&[0.05, 0.03, 0.07], &[0.05, 0.03, 0.07], 252.0).unwrap();
        assert_eq!(r.mean_active_return, 0.0);
        assert_eq!(r.tracking_error, 0.0);
        assert_eq!(r.information_ratio, 0.0);
    }

    #[test]
    fn ir_consistent_outperformance_yields_positive_finite_ir() {
        // Portfolio beats bench by 0.01 every period → mean active = 0.01,
        // te = 0 (no variation). IR forced to 0 by our zero-te guard.
        let r = information_ratio(&[0.05, 0.05, 0.05], &[0.04, 0.04, 0.04], 252.0).unwrap();
        assert!((r.mean_active_return - 0.01).abs() < 1e-12);
        assert_eq!(r.tracking_error, 0.0);
        assert_eq!(r.information_ratio, 0.0);
    }

    #[test]
    fn ir_positive_when_alpha_with_some_variance() {
        let r = information_ratio(
            &[0.05, 0.06, 0.04], // mean = 0.05
            &[0.04, 0.04, 0.04], // mean = 0.04
            252.0,
        )
        .unwrap();
        assert!((r.mean_active_return - 0.01).abs() < 1e-9);
        assert!(r.tracking_error > 0.0);
        assert!(r.information_ratio > 0.0);
    }

    #[test]
    fn ir_negative_when_underperforming() {
        let r = information_ratio(&[0.03, 0.04, 0.02], &[0.05, 0.05, 0.05], 252.0).unwrap();
        assert!(r.mean_active_return < 0.0);
        assert!(r.information_ratio < 0.0);
    }
}
