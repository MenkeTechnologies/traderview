//! Debt-Service Coverage Ratio (DSCR) — the core rental/commercial loan-sizing
//! metric. A lender divides a property's net operating income by its annual
//! debt service; a ratio at or above the lender's minimum (commonly 1.20–1.25)
//! means the income covers the loan payments with a cushion.
//!
//! ```text
//! DSCR = NOI / annual_debt_service
//! ```
//!
//! Beyond the ratio on the proposed loan, this sizes the largest loan that
//! still clears a target DSCR: cap the annual debt service at NOI / target,
//! then present-value that payment stream back to a loan balance.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DscrInput {
    /// Annual net operating income (rent + other income − operating expenses,
    /// before debt service).
    pub noi_usd: f64,
    /// Proposed loan amount.
    pub loan_amount_usd: f64,
    /// Annual interest rate, percent.
    pub interest_rate_pct: f64,
    /// Amortization term in years.
    pub amortization_years: f64,
    /// Payments per year (12 for a monthly mortgage).
    pub payments_per_year: f64,
    /// Lender's minimum DSCR (e.g. 1.25).
    pub target_dscr: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DscrResult {
    /// One periodic loan payment (principal + interest).
    pub periodic_payment_usd: f64,
    /// Total debt service over a year.
    pub annual_debt_service_usd: f64,
    /// NOI / annual debt service.
    pub dscr: f64,
    /// Whether the DSCR meets or beats the target.
    pub meets_target: bool,
    /// NOI − annual debt service (the cushion in dollars).
    pub annual_cash_flow_usd: f64,
    /// Largest annual debt service that still clears the target.
    pub max_annual_debt_service_usd: f64,
    /// Largest loan that still clears the target DSCR.
    pub max_loan_at_target_usd: f64,
    /// Fraction of NOI that may be lost before DSCR falls to the target
    /// (the break-even income cushion), 0–1.
    pub noi_cushion_fraction: f64,
}

/// Level periodic payment to amortize `principal` over `n` payments at periodic
/// rate `i` (the standard mortgage formula).
fn level_payment(principal: f64, i: f64, n: f64) -> f64 {
    if principal <= 0.0 || n <= 0.0 {
        return 0.0;
    }
    if i.abs() < 1e-12 {
        principal / n
    } else {
        let f = (1.0 + i).powf(n);
        principal * i * f / (f - 1.0)
    }
}

/// Loan balance a level payment `pmt` will amortize over `n` periods at `i`
/// (present value of the payment stream) — the inverse of `level_payment`.
fn loan_from_payment(pmt: f64, i: f64, n: f64) -> f64 {
    if pmt <= 0.0 || n <= 0.0 {
        return 0.0;
    }
    if i.abs() < 1e-12 {
        pmt * n
    } else {
        let f = (1.0 + i).powf(n);
        pmt * (f - 1.0) / (i * f)
    }
}

pub fn analyze(input: &DscrInput) -> DscrResult {
    let freq = if input.payments_per_year > 0.0 {
        input.payments_per_year
    } else {
        12.0
    };
    let i = input.interest_rate_pct / 100.0 / freq;
    let n = input.amortization_years * freq;

    let payment = level_payment(input.loan_amount_usd, i, n);
    let annual_ds = payment * freq;

    let dscr = if annual_ds > 0.0 {
        input.noi_usd / annual_ds
    } else {
        0.0
    };

    let target = input.target_dscr.max(0.0);
    // Largest debt service that still clears the target, and the loan it implies.
    let max_annual_ds = if target > 0.0 {
        input.noi_usd / target
    } else {
        0.0
    };
    let max_payment = max_annual_ds / freq;
    let max_loan = loan_from_payment(max_payment, i, n);

    // How much NOI can erode before DSCR hits the target.
    let noi_cushion = if input.noi_usd > 0.0 && dscr > 0.0 && target > 0.0 {
        (1.0 - target / dscr).max(0.0)
    } else {
        0.0
    };

    DscrResult {
        periodic_payment_usd: payment,
        annual_debt_service_usd: annual_ds,
        dscr,
        meets_target: dscr >= target && annual_ds > 0.0,
        annual_cash_flow_usd: input.noi_usd - annual_ds,
        max_annual_debt_service_usd: max_annual_ds,
        max_loan_at_target_usd: max_loan,
        noi_cushion_fraction: noi_cushion,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(noi: f64, loan: f64, rate: f64, years: f64, target: f64) -> DscrResult {
        analyze(&DscrInput {
            noi_usd: noi,
            loan_amount_usd: loan,
            interest_rate_pct: rate,
            amortization_years: years,
            payments_per_year: 12.0,
            target_dscr: target,
        })
    }

    #[test]
    fn payment_matches_amortization_formula() {
        // 600k @ 7% / 30yr monthly → $3,991.81/mo (independently checked).
        let r = run(60_000.0, 600_000.0, 7.0, 30.0, 1.25);
        assert!(close(r.periodic_payment_usd, 3991.814971));
        assert!(close(r.annual_debt_service_usd, 3991.814971 * 12.0));
    }

    #[test]
    fn dscr_is_noi_over_debt_service() {
        let r = run(60_000.0, 600_000.0, 7.0, 30.0, 1.25);
        assert!(close(r.dscr, 60_000.0 / (3991.814971 * 12.0)));
    }

    #[test]
    fn below_target_fails() {
        // 1.252 DSCR here clears, drop NOI so it fails 1.25.
        let pass = run(60_000.0, 600_000.0, 7.0, 30.0, 1.25);
        assert!(pass.meets_target);
        let fail = run(45_000.0, 600_000.0, 7.0, 30.0, 1.25);
        assert!(!fail.meets_target);
        assert!(fail.dscr < 1.25);
    }

    #[test]
    fn annual_cash_flow_is_surplus() {
        let r = run(60_000.0, 600_000.0, 7.0, 30.0, 1.25);
        assert!(close(r.annual_cash_flow_usd, 60_000.0 - r.annual_debt_service_usd));
    }

    #[test]
    fn max_loan_clears_target_exactly() {
        // Feed the max loan back in; its DSCR should equal the target.
        let r = run(60_000.0, 600_000.0, 7.0, 30.0, 1.25);
        let check = run(60_000.0, r.max_loan_at_target_usd, 7.0, 30.0, 1.25);
        assert!(close(check.dscr, 1.25));
    }

    #[test]
    fn higher_target_shrinks_max_loan() {
        let lenient = run(60_000.0, 600_000.0, 7.0, 30.0, 1.20);
        let strict = run(60_000.0, 600_000.0, 7.0, 30.0, 1.35);
        assert!(strict.max_loan_at_target_usd < lenient.max_loan_at_target_usd);
    }

    #[test]
    fn noi_cushion_matches_definition() {
        let r = run(60_000.0, 600_000.0, 7.0, 30.0, 1.25);
        assert!(close(r.noi_cushion_fraction, (1.0 - 1.25 / r.dscr).max(0.0)));
    }

    #[test]
    fn zero_rate_amortizes_linearly() {
        // 360k @ 0% / 30yr monthly → 360k/360 = $1,000/mo.
        let r = run(20_000.0, 360_000.0, 0.0, 30.0, 1.2);
        assert!(close(r.periodic_payment_usd, 1000.0));
    }
}
