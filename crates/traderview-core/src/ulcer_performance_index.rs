//! Ulcer Performance Index (UPI) — Peter Martin & Byron McCann
//! ("The Investor's Guide to Fidelity Funds", 1989).
//!
//! Drawdown-adjusted return metric: excess return per unit of "ulcer"
//! (RMS drawdown depth). Analogous to Sharpe but penalizes only
//! downside excursions, not upside volatility:
//!
//!   period_return = (final / initial) - 1
//!   annualized_return = (1 + period_return)^(periods_per_year / n) - 1
//!   excess = annualized_return - risk_free_rate
//!   UPI = excess / ulcer_index
//!
//! Where `ulcer_index` is the root-mean-square of percent drawdowns
//! from the rolling all-time-high.
//!
//! UPI > 1 is widely considered "investable" by Martin's framework.
//!
//! Pure compute. Default periods_per_year = 252 (daily). Companion to
//! `ulcer_index`, `sharpe` (sortino, calmar variants), `risk_adjusted_ratios`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UlcerPerformanceReport {
    pub upi: f64,
    pub ulcer_index: f64,
    pub period_return: f64,
    pub annualized_return: f64,
    pub n_observations: usize,
}

pub fn compute(
    equity_curve: &[f64],
    risk_free_rate: f64,
    periods_per_year: f64,
) -> Option<UlcerPerformanceReport> {
    let n = equity_curve.len();
    if n < 2
        || !risk_free_rate.is_finite()
        || !periods_per_year.is_finite()
        || periods_per_year <= 0.0
    {
        return None;
    }
    if equity_curve.iter().any(|x| !x.is_finite() || *x <= 0.0) {
        return None;
    }
    let initial = equity_curve[0];
    let final_v = equity_curve[n - 1];
    let period_return = final_v / initial - 1.0;
    let years = (n - 1) as f64 / periods_per_year;
    if years <= 0.0 {
        return None;
    }
    let annualized = (1.0 + period_return).powf(1.0 / years) - 1.0;
    let excess = annualized - risk_free_rate;
    let ulcer = ulcer_index_value(equity_curve);
    if ulcer <= 0.0 {
        // Zero ulcer means no drawdowns; report UPI as infinity-equivalent
        // signal: if excess is positive return f64::INFINITY, else 0.
        let upi = if excess > 0.0 {
            f64::INFINITY
        } else if excess < 0.0 {
            f64::NEG_INFINITY
        } else {
            0.0
        };
        return Some(UlcerPerformanceReport {
            upi,
            ulcer_index: 0.0,
            period_return,
            annualized_return: annualized,
            n_observations: n,
        });
    }
    Some(UlcerPerformanceReport {
        upi: excess / ulcer,
        ulcer_index: ulcer,
        period_return,
        annualized_return: annualized,
        n_observations: n,
    })
}

fn ulcer_index_value(equity_curve: &[f64]) -> f64 {
    let n = equity_curve.len();
    if n == 0 {
        return 0.0;
    }
    let mut peak = equity_curve[0];
    let mut sum_sq = 0.0_f64;
    for v in equity_curve {
        if *v > peak {
            peak = *v;
        }
        let dd_pct = if peak > 0.0 {
            (v - peak) / peak * 100.0
        } else {
            0.0
        };
        sum_sq += dd_pct * dd_pct;
    }
    (sum_sq / n as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[100.0], 0.0, 252.0).is_none());
        assert!(compute(&[100.0, 110.0], 0.0, 0.0).is_none());
        assert!(compute(&[100.0, f64::NAN], 0.0, 252.0).is_none());
        assert!(compute(&[100.0, -10.0], 0.0, 252.0).is_none());
    }

    #[test]
    fn monotone_uptrend_yields_infinite_upi_with_positive_return() {
        // No drawdowns → ulcer = 0; excess > 0 → UPI = +∞.
        let eq: Vec<f64> = (0..252).map(|i| 100.0 + i as f64).collect();
        let r = compute(&eq, 0.0, 252.0).unwrap();
        assert!(r.ulcer_index.abs() < 1e-9);
        assert!(r.upi.is_infinite() && r.upi > 0.0);
    }

    #[test]
    fn drawdown_curve_yields_finite_positive_upi_when_return_positive() {
        let mut eq = vec![100.0_f64];
        for _ in 0..100 {
            eq.push(eq.last().unwrap() + 1.0);
        }
        for _ in 0..30 {
            eq.push(eq.last().unwrap() - 0.5);
        }
        for _ in 0..100 {
            eq.push(eq.last().unwrap() + 1.0);
        }
        let r = compute(&eq, 0.0, 252.0).unwrap();
        assert!(r.ulcer_index > 0.0);
        assert!(r.upi > 0.0 && r.upi.is_finite());
    }

    #[test]
    fn declining_curve_yields_negative_upi() {
        let eq: Vec<f64> = (0..252).map(|i| 200.0 - i as f64 * 0.3).collect();
        let r = compute(&eq, 0.0, 252.0).unwrap();
        assert!(r.upi < 0.0);
    }

    #[test]
    fn high_risk_free_rate_flips_upi_sign() {
        // Monotone uptrend → ulcer = 0. Excess depends entirely on
        // (annualized - rf). With rf above annualized, excess < 0
        // and UPI returns -∞ per our zero-ulcer branch.
        let eq: Vec<f64> = (0..252).map(|i| 100.0 + i as f64 * 0.01).collect();
        let r_zero = compute(&eq, 0.0, 252.0).unwrap();
        let r_high = compute(&eq, 1.0, 252.0).unwrap();
        // Same annualized return both calls (curve unchanged).
        assert!((r_zero.annualized_return - r_high.annualized_return).abs() < 1e-12);
        // Zero rf: positive excess, ulcer=0 → +∞.
        assert!(r_zero.upi.is_infinite() && r_zero.upi > 0.0);
        // 100% rf: negative excess, ulcer=0 → -∞.
        assert!(r_high.upi.is_infinite() && r_high.upi < 0.0);
    }

    #[test]
    fn n_observations_reported() {
        let eq = vec![100.0_f64; 30];
        let r = compute(&eq, 0.0, 252.0).unwrap();
        assert_eq!(r.n_observations, 30);
    }
}
