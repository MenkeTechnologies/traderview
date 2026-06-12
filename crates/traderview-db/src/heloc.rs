//! Home Equity Line of Credit (HELOC) calculator.
//!
//! Standard HELOC structure: a revolving credit line tied to home
//! equity, typically variable-rate (prime + margin), with two phases:
//!
//!   1. DRAW PERIOD     — typically 10 years. Borrower can draw up to
//!      the credit limit; minimum payment is usually
//!      interest-only (or 1-2% of balance).
//!   2. REPAYMENT PERIOD — typically 20 years. No more draws; balance
//!      amortizes like a standard fixed-rate loan
//!      at the current variable rate.
//!
//! Inputs:
//!   - line_size_usd                  — credit line cap
//!   - current_balance_usd             — currently drawn
//!   - variable_apr_pct                — current annual rate
//!   - draw_period_months              — typically 120
//!   - repayment_period_months         — typically 240
//!   - draw_phase_min_pct              — minimum draw-phase payment as
//!     % of balance (0 = interest-only,
//!     1.0 = 1% of balance + interest)
//!   - monthly_voluntary_principal_usd — extra principal paid each month
//!     during draw phase
//!
//! Compute returns:
//!   - utilization_pct = current_balance / line_size × 100
//!   - draw_phase_monthly_interest_usd  (balance × APR/12)
//!   - draw_phase_min_payment_usd       (interest + balance × min%)
//!   - draw_phase_total_payment_usd     (min + voluntary principal)
//!   - repayment_phase_balance_usd      — projected balance at end of
//!     draw period (if only paying
//!     min + voluntary)
//!   - repayment_phase_monthly_pi_usd   — amortizing payment on that
//!     balance at current APR for
//!     repayment period
//!   - total_lifetime_interest_usd      — sum over both phases
//!   - status — "interest_only" / "principal_reducing" /
//!     "underutilized" / "maxed"
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct HelocInput {
    pub line_size_usd: f64,
    #[serde(default)]
    pub current_balance_usd: f64,
    pub variable_apr_pct: f64,
    #[serde(default = "default_draw")]
    pub draw_period_months: u32,
    #[serde(default = "default_repay")]
    pub repayment_period_months: u32,
    #[serde(default)]
    pub draw_phase_min_pct: f64,
    #[serde(default)]
    pub monthly_voluntary_principal_usd: f64,
}

fn default_draw() -> u32 { 120 }
fn default_repay() -> u32 { 240 }

#[derive(Debug, Clone, Serialize)]
pub struct HelocReport {
    pub utilization_pct: f64,
    pub draw_phase_monthly_interest_usd: f64,
    pub draw_phase_min_payment_usd: f64,
    pub draw_phase_total_payment_usd: f64,
    pub draw_phase_total_interest_usd: f64,
    pub repayment_phase_balance_usd: f64,
    pub repayment_phase_monthly_pi_usd: f64,
    pub repayment_phase_total_interest_usd: f64,
    pub total_lifetime_interest_usd: f64,
    pub status: &'static str,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

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

/// Project balance at end of draw phase given starting balance,
/// monthly voluntary principal payment, and a `min_pct` minimum
/// principal payment (% of starting balance).
pub fn project_draw_balance(
    balance: f64,
    apr_pct: f64,
    months: u32,
    min_pct: f64,
    voluntary: f64,
) -> (f64, f64) {
    let r = apr_pct / 100.0 / 12.0;
    let mut bal = balance;
    let mut interest_acc = 0.0;
    for _ in 0..months {
        if bal <= 0.0 { break; }
        let interest = bal * r;
        interest_acc += interest;
        // Recalculate min on the CURRENT balance each month. Real HELOCs
        // amortize the minimum off the outstanding principal, not the
        // initial draw — the previous fixed `principal_min` overstated
        // paydown as `bal` shrank and underreported total interest.
        let principal_min = bal * min_pct / 100.0;
        let pay_principal = (principal_min + voluntary).min(bal);
        bal -= pay_principal;
        if bal < 0.005 { bal = 0.0; }
    }
    (bal, interest_acc)
}

pub fn compute(input: &HelocInput) -> HelocReport {
    let util = if input.line_size_usd > 0.0 {
        input.current_balance_usd / input.line_size_usd * 100.0
    } else { 0.0 };
    let monthly_r = input.variable_apr_pct / 100.0 / 12.0;
    let interest_only = input.current_balance_usd * monthly_r;
    let min_principal = input.current_balance_usd * input.draw_phase_min_pct / 100.0;
    let draw_min = interest_only + min_principal;
    let draw_total = draw_min + input.monthly_voluntary_principal_usd;

    let (end_draw_balance, draw_interest) = project_draw_balance(
        input.current_balance_usd,
        input.variable_apr_pct,
        input.draw_period_months,
        input.draw_phase_min_pct,
        input.monthly_voluntary_principal_usd,
    );

    let repay_pi = monthly_payment(end_draw_balance, input.variable_apr_pct, input.repayment_period_months);
    let repay_total_payments = repay_pi * input.repayment_period_months as f64;
    let repay_interest = (repay_total_payments - end_draw_balance).max(0.0);
    let lifetime_interest = draw_interest + repay_interest;

    let status: &'static str = if util > 90.0 {
        "maxed"
    } else if util < 10.0 {
        "underutilized"
    } else if input.monthly_voluntary_principal_usd > 0.0 || input.draw_phase_min_pct > 0.0 {
        "principal_reducing"
    } else {
        "interest_only"
    };

    HelocReport {
        utilization_pct: util,
        draw_phase_monthly_interest_usd: interest_only,
        draw_phase_min_payment_usd: draw_min,
        draw_phase_total_payment_usd: draw_total,
        draw_phase_total_interest_usd: draw_interest,
        repayment_phase_balance_usd: end_draw_balance,
        repayment_phase_monthly_pi_usd: repay_pi,
        repayment_phase_total_interest_usd: repay_interest,
        total_lifetime_interest_usd: lifetime_interest,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> HelocInput {
        HelocInput {
            line_size_usd: 100_000.0,
            current_balance_usd: 50_000.0,
            variable_apr_pct: 8.5,
            draw_period_months: 120,
            repayment_period_months: 240,
            draw_phase_min_pct: 0.0,
            monthly_voluntary_principal_usd: 0.0,
        }
    }

    #[test]
    fn monthly_payment_known() {
        let p = monthly_payment(50_000.0, 8.5, 240);
        assert!((p - 433.93).abs() < 5.0, "got {p}");
    }

    #[test]
    fn project_draw_balance_interest_only() {
        let (bal, _) = project_draw_balance(50_000.0, 8.5, 120, 0.0, 0.0);
        // Interest-only: balance unchanged.
        assert_eq!(bal, 50_000.0);
    }

    #[test]
    fn project_draw_balance_with_voluntary_principal() {
        let (bal, _) = project_draw_balance(50_000.0, 8.5, 120, 0.0, 500.0);
        assert!(bal < 50_000.0);
    }

    #[test]
    fn project_draw_balance_full_paydown() {
        // High voluntary → balance hits 0 before end of draw.
        let (bal, _) = project_draw_balance(10_000.0, 8.5, 120, 0.0, 2000.0);
        assert_eq!(bal, 0.0);
    }

    #[test]
    fn compute_basic_util() {
        let r = compute(&input());
        assert_eq!(r.utilization_pct, 50.0);
    }

    #[test]
    fn compute_interest_only_status_default() {
        let r = compute(&input());
        assert_eq!(r.status, "interest_only");
    }

    #[test]
    fn compute_maxed_status_above_90() {
        let mut i = input();
        i.current_balance_usd = 95_000.0;
        let r = compute(&i);
        assert_eq!(r.status, "maxed");
    }

    #[test]
    fn compute_underutilized_below_10() {
        let mut i = input();
        i.current_balance_usd = 5_000.0;
        let r = compute(&i);
        assert_eq!(r.status, "underutilized");
    }

    #[test]
    fn compute_principal_reducing_with_voluntary() {
        let mut i = input();
        i.monthly_voluntary_principal_usd = 500.0;
        let r = compute(&i);
        assert_eq!(r.status, "principal_reducing");
    }

    #[test]
    fn compute_draw_phase_interest_basic() {
        // $50k @ 8.5% / 12 = $354.17/mo interest-only payment.
        let r = compute(&input());
        assert!((r.draw_phase_monthly_interest_usd - 354.17).abs() < 0.5);
    }

    #[test]
    fn compute_full_paydown_via_voluntary_zero_repay() {
        let mut i = input();
        i.monthly_voluntary_principal_usd = 5_000.0;  // wipes it out fast
        let r = compute(&i);
        assert_eq!(r.repayment_phase_balance_usd, 0.0);
        assert_eq!(r.repayment_phase_monthly_pi_usd, 0.0);
        assert_eq!(r.repayment_phase_total_interest_usd, 0.0);
    }

    #[test]
    fn compute_lifetime_interest_sums_phases() {
        let r = compute(&input());
        assert!((r.total_lifetime_interest_usd
            - (r.draw_phase_total_interest_usd + r.repayment_phase_total_interest_usd))
            .abs() < 0.01);
    }

    #[test]
    fn compute_zero_line_size_safe() {
        let mut i = input();
        i.line_size_usd = 0.0;
        i.current_balance_usd = 0.0;
        let r = compute(&i);
        assert_eq!(r.utilization_pct, 0.0);
        assert_eq!(r.draw_phase_monthly_interest_usd, 0.0);
    }
}
