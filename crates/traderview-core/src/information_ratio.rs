//! Information Ratio = active return / tracking error.
//!
//! Active return = portfolio return − benchmark return per period.
//! Tracking error = stdev of active returns.
//!
//!   IR = mean(active) / stdev(active)
//!
//! Distinct from Sharpe in that it uses the BENCHMARK as the riskless
//! reference rather than the risk-free rate — measures skill at picking
//! over-/under-weights vs the benchmark, not absolute performance.
//!
//! Returns the mean active return, tracking error, and IR plus
//! optional annualized variants (caller supplies periods_per_year).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct InformationReport {
    pub mean_active_return: f64,
    pub tracking_error: f64,
    pub information_ratio: f64,
    pub n_observations: usize,
    pub annualized_active_return: f64,
    pub annualized_tracking_error: f64,
    pub annualized_information_ratio: f64,
}

pub fn compute(
    portfolio_returns: &[f64],
    benchmark_returns: &[f64],
    periods_per_year: f64,
) -> Option<InformationReport> {
    if portfolio_returns.len() != benchmark_returns.len()
        || portfolio_returns.len() < 2
        || !periods_per_year.is_finite()
        || periods_per_year <= 0.0
    {
        return None;
    }
    let mut active = Vec::with_capacity(portfolio_returns.len());
    for (p, b) in portfolio_returns.iter().zip(benchmark_returns.iter()) {
        if !p.is_finite() || !b.is_finite() {
            continue;
        }
        active.push(p - b);
    }
    let n = active.len();
    if n < 2 {
        return None;
    }
    let n_f = n as f64;
    let mean = active.iter().sum::<f64>() / n_f;
    let var = active.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_f - 1.0);
    let te = var.max(0.0).sqrt();
    let ir = if te > 0.0 {
        mean / te
    } else if mean == 0.0 {
        0.0
    } else {
        f64::INFINITY
    };
    let sqrt_py = periods_per_year.sqrt();
    Some(InformationReport {
        mean_active_return: mean,
        tracking_error: te,
        information_ratio: ir,
        n_observations: n,
        annualized_active_return: mean * periods_per_year,
        annualized_tracking_error: te * sqrt_py,
        annualized_information_ratio: if te > 0.0 { (mean / te) * sqrt_py } else { ir },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_mismatch_returns_none() {
        assert!(compute(&[0.01; 5], &[0.01; 10], 252.0).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01], &[0.01], 252.0).is_none());
    }

    #[test]
    fn invalid_periods_returns_none() {
        let p = vec![0.01; 10];
        let b = vec![0.005; 10];
        assert!(compute(&p, &b, 0.0).is_none());
        assert!(compute(&p, &b, f64::NAN).is_none());
        assert!(compute(&p, &b, -1.0).is_none());
    }

    #[test]
    fn perfect_replication_yields_zero_active_and_zero_ir() {
        let p = vec![0.01, 0.02, -0.01, 0.005];
        let r = compute(&p, &p, 252.0).unwrap();
        assert_eq!(r.mean_active_return, 0.0);
        assert_eq!(r.tracking_error, 0.0);
        assert_eq!(r.information_ratio, 0.0);
    }

    #[test]
    fn constant_outperformance_yields_infinity() {
        // Every period the portfolio beats by exactly 1bp → tracking error = 0.
        let p = vec![0.011, 0.011, 0.011, 0.011];
        let b = vec![0.010, 0.010, 0.010, 0.010];
        let r = compute(&p, &b, 252.0).unwrap();
        assert!(r.information_ratio.is_infinite() && r.information_ratio > 0.0);
        assert_eq!(r.tracking_error, 0.0);
    }

    #[test]
    fn positive_active_return_yields_positive_ir() {
        let p = vec![0.02, 0.015, 0.025, 0.018, 0.020];
        let b = vec![0.01, 0.012, 0.011, 0.009, 0.013];
        let r = compute(&p, &b, 252.0).unwrap();
        assert!(r.mean_active_return > 0.0);
        assert!(r.information_ratio > 0.0);
        assert!(
            r.annualized_information_ratio > r.information_ratio,
            "annualized IR should scale up by √252"
        );
    }

    #[test]
    fn nan_pairs_skipped_safely() {
        let p = vec![0.01, f64::NAN, 0.02, 0.015];
        let b = vec![0.005, 0.01, f64::NAN, 0.008];
        let r = compute(&p, &b, 252.0).unwrap();
        // 2 valid pairs after NaN filtering.
        assert_eq!(r.n_observations, 2);
    }

    #[test]
    fn annualization_scales_correctly() {
        let p = vec![0.01, 0.02, -0.01, 0.005, 0.008];
        let b = vec![0.005, 0.015, -0.005, 0.001, 0.007];
        let r = compute(&p, &b, 252.0).unwrap();
        assert!((r.annualized_active_return - r.mean_active_return * 252.0).abs() < 1e-12);
        assert!((r.annualized_tracking_error - r.tracking_error * 252.0_f64.sqrt()).abs() < 1e-12);
    }
}
