//! Auto loan amortization calculator.
//!
//! Standard fixed-rate amortization closed form for a vehicle loan:
//!
//!   monthly_payment = P × r / (1 − (1+r)^(−n))
//!
//! where P = principal financed, r = APR/12, n = term in months.
//! Plus a month-by-month amortization schedule showing the
//! split between principal and interest each month, with running
//! balance.
//!
//! Inputs: vehicle_price + down_payment + trade_in (both subtract from
//! financed amount) + sales_tax_pct (added to financed amount unless
//! `tax_paid_at_signing = true`) + apr_pct + term_months. Compute
//! returns:
//!
//!   - principal_financed_usd
//!   - monthly_payment_usd        (P&I only)
//!   - total_payments_usd          = monthly × term
//!   - total_interest_usd          = total_payments − principal
//!   - schedule                    = Vec<MonthRow>
//!
//! Pure compute — no DB I/O, no clock reads.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AutoLoanInput {
    pub vehicle_price_usd: f64,
    #[serde(default)]
    pub down_payment_usd: f64,
    #[serde(default)]
    pub trade_in_credit_usd: f64,
    #[serde(default)]
    pub sales_tax_pct: f64,
    #[serde(default)]
    pub tax_paid_at_signing: bool,
    pub apr_pct: f64,
    pub term_months: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonthRow {
    pub month: u32,
    pub payment_usd: f64,
    pub principal_usd: f64,
    pub interest_usd: f64,
    pub balance_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AutoLoanReport {
    pub principal_financed_usd: f64,
    pub monthly_payment_usd: f64,
    pub total_payments_usd: f64,
    pub total_interest_usd: f64,
    pub schedule: Vec<MonthRow>,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn financed_amount(input: &AutoLoanInput) -> f64 {
    let sales_tax = input.vehicle_price_usd * input.sales_tax_pct / 100.0;
    let cash_price = input.vehicle_price_usd + if input.tax_paid_at_signing { 0.0 } else { sales_tax };
    (cash_price - input.down_payment_usd - input.trade_in_credit_usd).max(0.0)
}

/// Standard amortizing-loan payment.
pub fn monthly_payment(principal: f64, apr_pct: f64, term_months: u32) -> f64 {
    if term_months == 0 || principal <= 0.0 {
        return 0.0;
    }
    let r = apr_pct / 100.0 / 12.0;
    if r.abs() < 1e-12 {
        return principal / term_months as f64;
    }
    let n = term_months as f64;
    principal * r / (1.0 - (1.0 + r).powf(-n))
}

pub fn schedule(principal: f64, apr_pct: f64, term_months: u32) -> Vec<MonthRow> {
    if term_months == 0 || principal <= 0.0 {
        return Vec::new();
    }
    let r = apr_pct / 100.0 / 12.0;
    let payment = monthly_payment(principal, apr_pct, term_months);
    let mut balance = principal;
    let mut out: Vec<MonthRow> = Vec::with_capacity(term_months as usize);
    for m in 1..=term_months {
        let interest = balance * r;
        let mut principal_portion = (payment - interest).max(0.0);
        if principal_portion > balance {
            principal_portion = balance;
        }
        balance -= principal_portion;
        if balance < 0.005 {
            balance = 0.0;
        }
        let actual_payment = interest + principal_portion;
        out.push(MonthRow {
            month: m,
            payment_usd: actual_payment,
            principal_usd: principal_portion,
            interest_usd: interest,
            balance_usd: balance,
        });
        if balance <= 0.005 { break; }
    }
    out
}

pub fn compute(input: &AutoLoanInput) -> AutoLoanReport {
    let principal = financed_amount(input);
    let payment = monthly_payment(principal, input.apr_pct, input.term_months);
    let total_payments = payment * input.term_months as f64;
    let total_interest = (total_payments - principal).max(0.0);
    let schedule_rows = schedule(principal, input.apr_pct, input.term_months);
    AutoLoanReport {
        principal_financed_usd: principal,
        monthly_payment_usd: payment,
        total_payments_usd: total_payments,
        total_interest_usd: total_interest,
        schedule: schedule_rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn financed_amount_basic() {
        let f = financed_amount(&AutoLoanInput {
            vehicle_price_usd: 30_000.0,
            down_payment_usd: 5_000.0,
            trade_in_credit_usd: 2_000.0,
            sales_tax_pct: 0.0,
            tax_paid_at_signing: false,
            apr_pct: 6.0,
            term_months: 60,
        });
        assert_eq!(f, 23_000.0);
    }

    #[test]
    fn financed_amount_includes_tax_rolled() {
        let f = financed_amount(&AutoLoanInput {
            vehicle_price_usd: 30_000.0,
            down_payment_usd: 0.0,
            trade_in_credit_usd: 0.0,
            sales_tax_pct: 8.0,
            tax_paid_at_signing: false,
            apr_pct: 6.0,
            term_months: 60,
        });
        assert_eq!(f, 32_400.0);
    }

    #[test]
    fn financed_amount_excludes_tax_when_paid_at_signing() {
        let f = financed_amount(&AutoLoanInput {
            vehicle_price_usd: 30_000.0,
            down_payment_usd: 0.0,
            trade_in_credit_usd: 0.0,
            sales_tax_pct: 8.0,
            tax_paid_at_signing: true,
            apr_pct: 6.0,
            term_months: 60,
        });
        assert_eq!(f, 30_000.0);
    }

    #[test]
    fn financed_amount_clamps_to_zero_when_down_exceeds_price() {
        let f = financed_amount(&AutoLoanInput {
            vehicle_price_usd: 10_000.0,
            down_payment_usd: 15_000.0,
            trade_in_credit_usd: 0.0,
            sales_tax_pct: 0.0,
            tax_paid_at_signing: false,
            apr_pct: 6.0,
            term_months: 60,
        });
        assert_eq!(f, 0.0);
    }

    #[test]
    fn monthly_payment_zero_apr_linear() {
        assert_eq!(monthly_payment(12_000.0, 0.0, 60), 200.0);
    }

    #[test]
    fn monthly_payment_zero_term_returns_zero() {
        assert_eq!(monthly_payment(10_000.0, 6.0, 0), 0.0);
    }

    #[test]
    fn monthly_payment_known_amount() {
        // $25k @ 6% APR / 60 months = $483.32 standard published.
        let p = monthly_payment(25_000.0, 6.0, 60);
        assert!((p - 483.32).abs() < 0.5, "got {p}");
    }

    #[test]
    fn schedule_length_matches_term() {
        let s = schedule(10_000.0, 6.0, 36);
        assert_eq!(s.len(), 36);
        assert_eq!(s.last().unwrap().month, 36);
    }

    #[test]
    fn schedule_final_balance_zero() {
        let s = schedule(10_000.0, 6.0, 36);
        assert!(s.last().unwrap().balance_usd <= 0.005);
    }

    #[test]
    fn schedule_principal_sum_equals_loan() {
        let s = schedule(10_000.0, 6.0, 36);
        let principal_sum: f64 = s.iter().map(|m| m.principal_usd).sum();
        assert!((principal_sum - 10_000.0).abs() < 0.05, "got {principal_sum}");
    }

    #[test]
    fn schedule_interest_decreases_over_time() {
        let s = schedule(20_000.0, 8.0, 60);
        // First month should have more interest than last month.
        assert!(s[0].interest_usd > s.last().unwrap().interest_usd);
    }

    #[test]
    fn compute_full_report() {
        let r = compute(&AutoLoanInput {
            vehicle_price_usd: 30_000.0,
            down_payment_usd: 5_000.0,
            trade_in_credit_usd: 0.0,
            sales_tax_pct: 0.0,
            tax_paid_at_signing: true,
            apr_pct: 6.0,
            term_months: 60,
        });
        assert_eq!(r.principal_financed_usd, 25_000.0);
        assert!((r.monthly_payment_usd - 483.32).abs() < 0.5);
        assert!(r.total_interest_usd > 0.0);
        assert_eq!(r.schedule.len(), 60);
    }

    #[test]
    fn compute_zero_principal_zero_payment() {
        let r = compute(&AutoLoanInput {
            vehicle_price_usd: 0.0,
            down_payment_usd: 0.0,
            trade_in_credit_usd: 0.0,
            sales_tax_pct: 0.0,
            tax_paid_at_signing: false,
            apr_pct: 6.0,
            term_months: 60,
        });
        assert_eq!(r.principal_financed_usd, 0.0);
        assert_eq!(r.monthly_payment_usd, 0.0);
        assert_eq!(r.total_interest_usd, 0.0);
    }
}
