//! Modigliani–Modigliani M² (Risk-Adjusted Performance, "RAP", 1997).
//!
//! Answers: "What would the portfolio's return have been if it had been
//! levered/de-levered to match the benchmark's volatility?"
//!
//!   M² = Rf + Sharpe_portfolio · σ_benchmark
//!
//! Equivalently:
//!
//!   M² = Rf + (R_p − Rf) · (σ_b / σ_p)
//!
//! Interpretation: a directly-comparable return number (in the same
//! units as the benchmark's), so M² − R_b is the volatility-adjusted
//! excess return. Sharpe ranks the same portfolios, but M² makes the
//! gap *interpretable as basis points*.
//!
//! Pure compute. All inputs in the same period (daily, monthly, ann).
//! Caller annualizes if desired.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct M2Report {
    pub portfolio_return: f64,
    pub portfolio_volatility: f64,
    pub benchmark_return: f64,
    pub benchmark_volatility: f64,
    pub risk_free_rate: f64,
    pub sharpe_portfolio: f64,
    pub m2_return: f64,
    /// M² − R_b: volatility-adjusted excess return over benchmark.
    pub m2_alpha: f64,
}

pub fn compute(portfolio: &[f64], benchmark: &[f64], risk_free_rate: f64) -> Option<M2Report> {
    if portfolio.len() < 2 || benchmark.len() < 2 || !risk_free_rate.is_finite() {
        return None;
    }
    let (rp_mean, rp_var) = mean_var(portfolio)?;
    let (rb_mean, rb_var) = mean_var(benchmark)?;
    let rp_sd = rp_var.max(0.0).sqrt();
    let rb_sd = rb_var.max(0.0).sqrt();
    if rp_sd <= 0.0 {
        return None;
    }
    let sharpe = (rp_mean - risk_free_rate) / rp_sd;
    let m2 = risk_free_rate + sharpe * rb_sd;
    Some(M2Report {
        portfolio_return: rp_mean,
        portfolio_volatility: rp_sd,
        benchmark_return: rb_mean,
        benchmark_volatility: rb_sd,
        risk_free_rate,
        sharpe_portfolio: sharpe,
        m2_return: m2,
        m2_alpha: m2 - rb_mean,
    })
}

fn mean_var(xs: &[f64]) -> Option<(f64, f64)> {
    if xs.iter().any(|x| !x.is_finite()) {
        return None;
    }
    // Reject truly flat input: float round-off can yield a tiny nonzero
    // variance from identical values, which then poisons Sharpe.
    let (mn, mx) = xs
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), x| {
            (a.min(*x), b.max(*x))
        });
    if mx - mn <= 0.0 {
        return None;
    }
    let n = xs.len() as f64;
    let mean: f64 = xs.iter().sum::<f64>() / n;
    let var: f64 = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    Some((mean, var))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01], &[0.01, 0.02], 0.0).is_none());
        assert!(compute(&[0.01, 0.02], &[0.01], 0.0).is_none());
    }

    #[test]
    fn nan_inputs_return_none() {
        let bad = vec![0.01, f64::NAN, 0.02];
        let ok = vec![0.01, 0.02, 0.03];
        assert!(compute(&bad, &ok, 0.0).is_none());
        assert!(compute(&ok, &bad, 0.0).is_none());
        assert!(compute(&ok, &ok, f64::NAN).is_none());
    }

    #[test]
    fn flat_portfolio_returns_none() {
        let flat = vec![0.01_f64; 20];
        let varying = vec![0.01, 0.02, 0.005, 0.03];
        assert!(compute(&flat, &varying, 0.0).is_none());
    }

    #[test]
    fn portfolio_equals_benchmark_yields_zero_alpha() {
        let r = vec![0.01, 0.02, -0.01, 0.03, -0.02, 0.015];
        let report = compute(&r, &r, 0.0).unwrap();
        assert!(report.m2_alpha.abs() < 1e-12);
        assert!((report.m2_return - report.benchmark_return).abs() < 1e-12);
    }

    #[test]
    fn higher_sharpe_yields_higher_m2() {
        // Two portfolios with same mean but different vol; lower vol →
        // higher Sharpe → higher M² when scaled to benchmark vol.
        let bench = vec![0.02, -0.01, 0.03, -0.02, 0.01, -0.005];
        let low_vol = vec![0.012, 0.008, 0.011, 0.009, 0.013, 0.007];
        let high_vol = vec![0.05, -0.04, 0.06, -0.05, 0.04, -0.03];
        let r_low = compute(&low_vol, &bench, 0.0).unwrap();
        let r_high = compute(&high_vol, &bench, 0.0).unwrap();
        assert!(
            r_low.m2_return > r_high.m2_return,
            "low-vol M² ({}) should exceed high-vol M² ({})",
            r_low.m2_return,
            r_high.m2_return
        );
    }

    #[test]
    fn m2_uses_benchmark_vol_for_scaling() {
        let port = vec![0.02, 0.04, 0.01, 0.03];
        let bench = vec![0.01, 0.02, 0.005, 0.015];
        let r = compute(&port, &bench, 0.0).unwrap();
        // Recompute manually: M² = Rf + Sharpe · σ_b.
        let expected = 0.0 + r.sharpe_portfolio * r.benchmark_volatility;
        assert!((r.m2_return - expected).abs() < 1e-12);
    }

    #[test]
    fn risk_free_offset_propagates() {
        let port = vec![0.02, 0.03, 0.01, 0.04];
        let bench = vec![0.01, 0.02, 0.005, 0.015];
        let r_zero = compute(&port, &bench, 0.0).unwrap();
        let r_rf = compute(&port, &bench, 0.005).unwrap();
        // M² depends on Rf via both the additive base and the Sharpe numerator.
        assert!(r_rf.m2_return != r_zero.m2_return);
    }
}
