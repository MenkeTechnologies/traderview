//! Cash-out refinance — how much equity a homeowner can pull out and what the
//! new loan costs.
//!
//! The new loan is capped at the lender's max LTV against the home's value; the
//! cash out is what's left after paying off the old balance and closing costs:
//!
//! ```text
//! max new loan  = max LTV × home value
//! cash out      = max new loan − current balance − closing costs
//! new payment   = amortize(max new loan, new rate, new term)
//! ```
//!
//! Investment-deal returns live in `brrrr`; this is the consumer equity pull.

use serde::{Deserialize, Serialize};

fn d_max_ltv() -> f64 {
    80.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct CashOutInput {
    pub home_value_usd: f64,
    pub current_balance_usd: f64,
    /// Lender's maximum loan-to-value, percent (cash-out caps are often 80).
    #[serde(default = "d_max_ltv")]
    pub max_ltv_pct: f64,
    pub new_rate_pct: f64,
    pub new_term_months: f64,
    #[serde(default)]
    pub closing_costs_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CashOutResult {
    /// Largest loan allowed at the max LTV.
    pub max_new_loan_usd: f64,
    /// New loan − current balance (before costs).
    pub gross_cash_out_usd: f64,
    /// Cash in hand after closing costs.
    pub net_cash_out_usd: f64,
    /// Payment on the new loan.
    pub new_monthly_payment_usd: f64,
    /// Current loan-to-value, percent.
    pub current_ltv_pct: f64,
    /// LTV after the refinance, percent.
    pub new_ltv_pct: f64,
    /// Equity left after the cash-out (home value − new loan).
    pub equity_remaining_usd: f64,
}

fn payment(principal: f64, annual_rate_pct: f64, n: f64) -> f64 {
    if principal <= 0.0 || n <= 0.0 {
        return 0.0;
    }
    let i = annual_rate_pct / 100.0 / 12.0;
    if i.abs() < 1e-12 {
        principal / n
    } else {
        let f = (1.0 + i).powf(n);
        principal * i * f / (f - 1.0)
    }
}

pub fn analyze(input: &CashOutInput) -> CashOutResult {
    let max_new_loan = (input.max_ltv_pct / 100.0 * input.home_value_usd).max(0.0);
    let gross = (max_new_loan - input.current_balance_usd).max(0.0);
    let net = gross - input.closing_costs_usd;

    let ltv = |loan: f64| {
        if input.home_value_usd > 0.0 {
            loan / input.home_value_usd * 100.0
        } else {
            0.0
        }
    };

    CashOutResult {
        max_new_loan_usd: max_new_loan,
        gross_cash_out_usd: gross,
        net_cash_out_usd: net,
        new_monthly_payment_usd: payment(max_new_loan, input.new_rate_pct, input.new_term_months),
        current_ltv_pct: ltv(input.current_balance_usd),
        new_ltv_pct: ltv(max_new_loan),
        equity_remaining_usd: input.home_value_usd - max_new_loan,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> CashOutInput {
        CashOutInput {
            home_value_usd: 500_000.0,
            current_balance_usd: 250_000.0,
            max_ltv_pct: 80.0,
            new_rate_pct: 6.5,
            new_term_months: 360.0,
            closing_costs_usd: 6_000.0,
        }
    }

    #[test]
    fn max_loan_at_ltv() {
        assert!(close(analyze(&base()).max_new_loan_usd, 400_000.0));
    }

    #[test]
    fn gross_and_net_cash_out() {
        let r = analyze(&base());
        assert!(close(r.gross_cash_out_usd, 150_000.0));
        assert!(close(r.net_cash_out_usd, 144_000.0));
    }

    #[test]
    fn new_payment() {
        assert!(close(analyze(&base()).new_monthly_payment_usd, 2528.272094));
    }

    #[test]
    fn ltvs() {
        let r = analyze(&base());
        assert!(close(r.current_ltv_pct, 50.0));
        assert!(close(r.new_ltv_pct, 80.0));
    }

    #[test]
    fn equity_remaining() {
        assert!(close(analyze(&base()).equity_remaining_usd, 100_000.0));
    }

    #[test]
    fn no_cash_out_when_balance_above_max_loan() {
        // Owe more than 80% LTV → no equity to pull.
        let r = analyze(&CashOutInput {
            current_balance_usd: 420_000.0,
            ..base()
        });
        assert!(close(r.gross_cash_out_usd, 0.0));
        assert!(r.net_cash_out_usd < 0.0); // costs with no cash out
    }

    #[test]
    fn higher_ltv_pulls_more() {
        let low = analyze(&base());
        let high = analyze(&CashOutInput {
            max_ltv_pct: 90.0,
            ..base()
        });
        assert!(high.gross_cash_out_usd > low.gross_cash_out_usd);
    }

    #[test]
    fn closing_costs_reduce_net() {
        let r = analyze(&base());
        assert!(close(r.net_cash_out_usd, r.gross_cash_out_usd - 6_000.0));
    }
}
