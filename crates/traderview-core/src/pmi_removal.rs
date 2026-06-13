//! PMI removal timeline — when scheduled amortization drops private mortgage
//! insurance off a conventional loan.
//!
//! PMI is tied to loan-to-value against the *original* home value: a borrower
//! may request cancellation at 80% LTV, and the servicer must cancel
//! automatically at 78% (Homeowners Protection Act). This finds the month the
//! amortizing balance first reaches each threshold.
//!
//! Solving `B_k = P(1+i)^k − pmt·((1+i)^k − 1)/i = target` for k:
//!
//! ```text
//! (1+i)^k = (target − pmt/i) / (P − pmt/i)
//! k = ln(that) / ln(1+i)
//! ```

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PmiInput {
    pub original_home_value_usd: f64,
    pub original_loan_usd: f64,
    pub annual_rate_pct: f64,
    pub loan_term_months: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PmiResult {
    pub monthly_payment_usd: f64,
    /// Loan-to-value at origination, percent.
    pub original_ltv_pct: f64,
    /// Balance threshold for borrower-requested cancellation (80% of value).
    pub target_80_balance_usd: f64,
    /// Balance threshold for automatic termination (78% of value).
    pub target_78_balance_usd: f64,
    /// Months until the balance reaches 80% LTV (request); `None` if no PMI.
    pub months_to_80: Option<f64>,
    /// Months until the balance reaches 78% LTV (automatic); `None` if no PMI.
    pub months_to_78: Option<f64>,
}

fn payment(p: f64, i: f64, n: f64) -> f64 {
    if p <= 0.0 || n <= 0.0 {
        return 0.0;
    }
    if i.abs() < 1e-12 {
        p / n
    } else {
        let f = (1.0 + i).powf(n);
        p * i * f / (f - 1.0)
    }
}

/// Whole months until the amortizing balance first reaches `target`.
fn months_until(p: f64, i: f64, pmt: f64, target: f64) -> Option<f64> {
    if p <= target {
        return Some(0.0); // already at or below the threshold
    }
    if i.abs() < 1e-12 {
        // Linear paydown: principal portion is pmt each month.
        if pmt <= 0.0 {
            return None;
        }
        return Some(((p - target) / pmt).ceil());
    }
    let denom = p - pmt / i;
    if denom == 0.0 {
        return None;
    }
    let x = (target - pmt / i) / denom;
    if x <= 0.0 {
        return None;
    }
    Some((x.ln() / (1.0 + i).ln()).ceil())
}

pub fn analyze(input: &PmiInput) -> PmiResult {
    let i = input.annual_rate_pct / 100.0 / 12.0;
    let pmt = payment(input.original_loan_usd, i, input.loan_term_months);

    let t80 = 0.80 * input.original_home_value_usd;
    let t78 = 0.78 * input.original_home_value_usd;

    let ltv = if input.original_home_value_usd > 0.0 {
        input.original_loan_usd / input.original_home_value_usd * 100.0
    } else {
        0.0
    };

    PmiResult {
        monthly_payment_usd: pmt,
        original_ltv_pct: ltv,
        target_80_balance_usd: t80,
        target_78_balance_usd: t78,
        months_to_80: months_until(input.original_loan_usd, i, pmt, t80),
        months_to_78: months_until(input.original_loan_usd, i, pmt, t78),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> PmiInput {
        PmiInput {
            original_home_value_usd: 400_000.0,
            original_loan_usd: 360_000.0,
            annual_rate_pct: 6.0,
            loan_term_months: 360.0,
        }
    }

    #[test]
    fn payment_and_ltv() {
        let r = analyze(&base());
        assert!(close(r.monthly_payment_usd, 2158.381891));
        assert!(close(r.original_ltv_pct, 90.0));
    }

    #[test]
    fn thresholds() {
        let r = analyze(&base());
        assert!(close(r.target_80_balance_usd, 320_000.0));
        assert!(close(r.target_78_balance_usd, 312_000.0));
    }

    #[test]
    fn months_to_eighty() {
        // Balance hits 320k at month 89 (88.91 → ceil).
        assert!(close(analyze(&base()).months_to_80.unwrap(), 89.0));
    }

    #[test]
    fn months_to_seventy_eight() {
        assert!(close(analyze(&base()).months_to_78.unwrap(), 103.0));
    }

    #[test]
    fn auto_termination_after_request() {
        let r = analyze(&base());
        // 78% is reached later than 80%.
        assert!(r.months_to_78.unwrap() > r.months_to_80.unwrap());
    }

    #[test]
    fn no_pmi_when_under_80_at_origination() {
        // 20% down → 80% LTV → at the request threshold immediately.
        let r = analyze(&PmiInput {
            original_home_value_usd: 400_000.0,
            original_loan_usd: 320_000.0,
            annual_rate_pct: 6.0,
            loan_term_months: 360.0,
        });
        assert!(close(r.months_to_80.unwrap(), 0.0));
    }

    #[test]
    fn zero_rate_linear_paydown() {
        // 360k loan, 0%, 360 months → $1000/mo principal. To 320k: 40k/1000 = 40.
        let r = analyze(&PmiInput {
            original_home_value_usd: 400_000.0,
            original_loan_usd: 360_000.0,
            annual_rate_pct: 0.0,
            loan_term_months: 360.0,
        });
        assert!(close(r.months_to_80.unwrap(), 40.0));
    }

    #[test]
    fn lower_down_payment_delays_removal() {
        let small_down = analyze(&PmiInput {
            original_home_value_usd: 400_000.0,
            original_loan_usd: 380_000.0,
            annual_rate_pct: 6.0,
            loan_term_months: 360.0,
        });
        assert!(small_down.months_to_80.unwrap() > analyze(&base()).months_to_80.unwrap());
    }
}
