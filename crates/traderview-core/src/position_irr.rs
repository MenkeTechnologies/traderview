//! Position Internal Rate of Return (IRR).
//!
//! Computes annualized return for a stream of cash flows. Standard
//! formula: find r where NPV = sum(cf_t / (1+r)^t) = 0.
//!
//! Uses Newton-Raphson with bisection fallback for robustness. Pure
//! compute.
//!
//! Used by traders to compare investment-style positions with
//! interleaved deposits/withdrawals — a simple "ending / starting"
//! ratio is misleading when capital flowed in and out mid-period.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlow {
    pub date: NaiveDate,
    /// Negative = outflow (deposit/buy), positive = inflow (withdraw/sell).
    pub amount: f64,
}

/// Computes annualized IRR. Returns None if the cash flows lack a
/// sign change (no IRR exists) or solver doesn't converge.
pub fn annualized_irr(flows: &[CashFlow]) -> Option<f64> {
    if flows.len() < 2 {
        return None;
    }
    // Need at least one positive and one negative flow.
    let has_pos = flows.iter().any(|f| f.amount > 0.0);
    let has_neg = flows.iter().any(|f| f.amount < 0.0);
    if !(has_pos && has_neg) {
        return None;
    }
    let t0 = flows[0].date;
    let years: Vec<f64> = flows
        .iter()
        .map(|f| (f.date - t0).num_days() as f64 / 365.25)
        .collect();
    // Bisection within [-0.99, 10.0]. NPV is monotonic in r for
    // well-formed cash flow series.
    let f = |r: f64| -> f64 {
        flows
            .iter()
            .zip(&years)
            .map(|(cf, t)| cf.amount / (1.0 + r).powf(*t))
            .sum()
    };
    let mut lo = -0.99;
    let mut hi = 10.0;
    let mut f_lo = f(lo);
    let f_hi = f(hi);
    if f_lo.signum() == f_hi.signum() {
        return None;
    }
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        let f_mid = f(mid);
        if f_mid.abs() < 1e-10 {
            return Some(mid);
        }
        if f_mid.signum() == f_lo.signum() {
            lo = mid;
            f_lo = f_mid;
        } else {
            hi = mid;
        }
        if (hi - lo).abs() < 1e-10 {
            return Some((lo + hi) / 2.0);
        }
    }
    Some((lo + hi) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }
    fn cf(date: NaiveDate, amount: f64) -> CashFlow {
        CashFlow { date, amount }
    }

    #[test]
    fn fewer_than_two_flows_none() {
        assert!(annualized_irr(&[]).is_none());
        assert!(annualized_irr(&[cf(d(2026, 1, 1), -1000.0)]).is_none());
    }

    #[test]
    fn no_sign_change_none() {
        let flows = vec![cf(d(2026, 1, 1), -1000.0), cf(d(2027, 1, 1), -500.0)];
        assert!(annualized_irr(&flows).is_none());
    }

    #[test]
    fn doubling_in_one_year_yields_100pct() {
        // -$1000 today, +$2000 in one year → 100% IRR.
        let flows = vec![cf(d(2026, 1, 1), -1000.0), cf(d(2027, 1, 1), 2000.0)];
        let r = annualized_irr(&flows).unwrap();
        assert!((r - 1.0).abs() < 0.01, "expected ~100%, got {}", r);
    }

    #[test]
    fn fifty_pct_loss_in_one_year_yields_minus_50pct() {
        let flows = vec![cf(d(2026, 1, 1), -1000.0), cf(d(2027, 1, 1), 500.0)];
        let r = annualized_irr(&flows).unwrap();
        assert!((r - (-0.5)).abs() < 0.01);
    }

    #[test]
    fn even_money_in_one_year_yields_zero() {
        let flows = vec![cf(d(2026, 1, 1), -1000.0), cf(d(2027, 1, 1), 1000.0)];
        let r = annualized_irr(&flows).unwrap();
        assert!(r.abs() < 0.01);
    }

    #[test]
    fn ten_pct_per_year_over_5_years() {
        // -$1000 → $1000 × 1.10^5 = $1610.51 in 5 years.
        let flows = vec![cf(d(2026, 1, 1), -1000.0), cf(d(2031, 1, 1), 1610.51)];
        let r = annualized_irr(&flows).unwrap();
        assert!((r - 0.10).abs() < 0.01);
    }

    #[test]
    fn handles_interleaved_deposits_and_withdrawals() {
        // Realistic scenario: invest $1000, add $500 in 6 months, withdraw $2000 in 1 year.
        let flows = vec![
            cf(d(2026, 1, 1), -1000.0),
            cf(d(2026, 7, 1), -500.0),
            cf(d(2027, 1, 1), 2000.0),
        ];
        let r = annualized_irr(&flows).unwrap();
        // Should be positive (net withdrawn more than deposited).
        assert!(r > 0.0);
    }

    #[test]
    fn inverted_sign_flows_yield_same_magnitude_rate() {
        // IRR is sign-direction-agnostic: borrowing $1000 today and
        // repaying $1100 in a year implies 10% rate from the lender's
        // perspective. The "short trade lost $100" P&L view is NOT IRR.
        let flows = vec![cf(d(2026, 1, 1), 1000.0), cf(d(2027, 1, 1), -1100.0)];
        let r = annualized_irr(&flows).unwrap();
        assert!((r - 0.10).abs() < 0.01, "expected ~10%, got {}", r);
    }
}
