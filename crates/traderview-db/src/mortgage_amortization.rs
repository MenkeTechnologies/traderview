//! Mortgage amortization calculator (fixed-rate, PITI + extra-payment).
//!
//! Standard fixed-rate amortization closed form for principal & interest:
//!
//!   monthly_pi = P × r / (1 − (1+r)^(−n))
//!
//! Plus PMI when LTV > 80%, property tax escrow, homeowners insurance,
//! HOA dues. Total monthly = PITIA. Optionally an `extra_principal_usd`
//! per month accelerates payoff and shrinks interest paid; reports both
//! the baseline schedule (no extra) and the extra-payment schedule with
//! the months saved and interest saved.
//!
//! Inputs:
//!   - home_price_usd, down_payment_usd, apr_pct, term_months
//!   - annual_property_tax_usd, annual_insurance_usd, monthly_hoa_usd
//!   - pmi_annual_rate_pct (e.g. 0.5–1.0% of loan amount until 80% LTV)
//!   - extra_principal_usd (extra applied each month after the regular P+I)
//!
//! Compute returns:
//!   - principal_financed_usd, ltv_pct
//!   - monthly_pi_usd, monthly_pmi_usd, monthly_tax_usd,
//!     monthly_insurance_usd, monthly_hoa_usd, total_monthly_usd
//!   - baseline_total_interest_usd, baseline_term_months
//!   - extra_total_interest_usd, extra_term_months
//!   - months_saved, interest_saved_usd
//!   - (a small head of the extra-payment amortization schedule for charting)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

const MAX_AMORT_MONTHS: u32 = 600;

#[derive(Debug, Clone, Deserialize)]
pub struct MortgageInput {
    pub home_price_usd: f64,
    #[serde(default)]
    pub down_payment_usd: f64,
    pub apr_pct: f64,
    pub term_months: u32,
    #[serde(default)]
    pub annual_property_tax_usd: f64,
    #[serde(default)]
    pub annual_insurance_usd: f64,
    #[serde(default)]
    pub monthly_hoa_usd: f64,
    /// PMI annual rate as % of remaining loan principal while LTV > 80%.
    /// Common range 0.3 – 1.5%.
    #[serde(default)]
    pub pmi_annual_rate_pct: f64,
    /// Extra dollars applied to principal each month, on top of regular P+I.
    #[serde(default)]
    pub extra_principal_usd: f64,
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
pub struct MortgageReport {
    pub principal_financed_usd: f64,
    pub ltv_pct: f64,
    pub monthly_pi_usd: f64,
    pub monthly_pmi_usd: f64,
    pub monthly_tax_usd: f64,
    pub monthly_insurance_usd: f64,
    pub monthly_hoa_usd: f64,
    pub total_monthly_usd: f64,
    pub baseline_total_interest_usd: f64,
    pub baseline_term_months: u32,
    pub extra_total_interest_usd: f64,
    pub extra_term_months: u32,
    pub months_saved: u32,
    pub interest_saved_usd: f64,
    pub schedule_head: Vec<MonthRow>,
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

/// Simulate amortization. Returns (total_interest_paid, months_used).
/// If `extra_principal` > 0 the loan pays off early.
pub fn simulate(principal: f64, apr_pct: f64, term_months: u32, extra_principal: f64) -> (f64, u32) {
    if principal <= 0.0 || term_months == 0 {
        return (0.0, 0);
    }
    let r = apr_pct / 100.0 / 12.0;
    let pi = monthly_payment(principal, apr_pct, term_months);
    let mut balance = principal;
    let mut total_interest = 0.0;
    let mut m: u32 = 0;
    while m < MAX_AMORT_MONTHS && balance > 0.005 {
        m += 1;
        let interest = balance * r;
        total_interest += interest;
        let mut pay_principal = (pi - interest).max(0.0) + extra_principal;
        if pay_principal > balance {
            pay_principal = balance;
        }
        balance -= pay_principal;
        if balance < 0.005 { balance = 0.0; }
    }
    (total_interest, m)
}

pub fn schedule_head(principal: f64, apr_pct: f64, term_months: u32, extra_principal: f64, n_rows: usize) -> Vec<MonthRow> {
    if principal <= 0.0 || term_months == 0 {
        return Vec::new();
    }
    let r = apr_pct / 100.0 / 12.0;
    let pi = monthly_payment(principal, apr_pct, term_months);
    let mut balance = principal;
    let mut out: Vec<MonthRow> = Vec::with_capacity(n_rows);
    let mut m: u32 = 0;
    while m < MAX_AMORT_MONTHS && balance > 0.005 && out.len() < n_rows {
        m += 1;
        let interest = balance * r;
        let mut pay_principal = (pi - interest).max(0.0) + extra_principal;
        if pay_principal > balance {
            pay_principal = balance;
        }
        balance -= pay_principal;
        if balance < 0.005 { balance = 0.0; }
        let payment = interest + pay_principal;
        out.push(MonthRow {
            month: m,
            payment_usd: payment,
            principal_usd: pay_principal,
            interest_usd: interest,
            balance_usd: balance,
        });
    }
    out
}

pub fn compute(input: &MortgageInput) -> MortgageReport {
    let principal = (input.home_price_usd - input.down_payment_usd).max(0.0);
    let ltv = if input.home_price_usd > 0.0 {
        principal / input.home_price_usd * 100.0
    } else { 0.0 };
    let pi = monthly_payment(principal, input.apr_pct, input.term_months);
    let pmi = if ltv > 80.0 {
        principal * input.pmi_annual_rate_pct / 100.0 / 12.0
    } else { 0.0 };
    let tax = input.annual_property_tax_usd / 12.0;
    let ins = input.annual_insurance_usd / 12.0;
    let hoa = input.monthly_hoa_usd;
    let total_monthly = pi + pmi + tax + ins + hoa + input.extra_principal_usd;

    let (baseline_interest, baseline_months) = simulate(principal, input.apr_pct, input.term_months, 0.0);
    let (extra_interest, extra_months) = simulate(principal, input.apr_pct, input.term_months, input.extra_principal_usd);
    let months_saved = baseline_months.saturating_sub(extra_months);
    let interest_saved = (baseline_interest - extra_interest).max(0.0);

    let head = schedule_head(principal, input.apr_pct, input.term_months, input.extra_principal_usd, 24);

    MortgageReport {
        principal_financed_usd: principal,
        ltv_pct: ltv,
        monthly_pi_usd: pi,
        monthly_pmi_usd: pmi,
        monthly_tax_usd: tax,
        monthly_insurance_usd: ins,
        monthly_hoa_usd: hoa,
        total_monthly_usd: total_monthly,
        baseline_total_interest_usd: baseline_interest,
        baseline_term_months: baseline_months,
        extra_total_interest_usd: extra_interest,
        extra_term_months: extra_months,
        months_saved,
        interest_saved_usd: interest_saved,
        schedule_head: head,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monthly_payment_known() {
        // $300k @ 6.5% / 360mo ≈ $1896.20 standard.
        let p = monthly_payment(300_000.0, 6.5, 360);
        assert!((p - 1_896.20).abs() < 1.0, "got {p}");
    }

    #[test]
    fn monthly_payment_zero_apr_linear() {
        assert_eq!(monthly_payment(360_000.0, 0.0, 360), 1_000.0);
    }

    #[test]
    fn simulate_finishes_in_term_with_no_extra() {
        let (_interest, months) = simulate(300_000.0, 6.5, 360, 0.0);
        assert!(months >= 359 && months <= 360, "got {months}");
    }

    #[test]
    fn simulate_extra_principal_speeds_payoff() {
        let (interest0, months0) = simulate(300_000.0, 6.5, 360, 0.0);
        let (interest1, months1) = simulate(300_000.0, 6.5, 360, 200.0);
        assert!(months1 < months0);
        assert!(interest1 < interest0);
    }

    #[test]
    fn compute_ltv_basic() {
        let r = compute(&MortgageInput {
            home_price_usd: 500_000.0,
            down_payment_usd: 100_000.0,
            apr_pct: 6.5,
            term_months: 360,
            annual_property_tax_usd: 0.0,
            annual_insurance_usd: 0.0,
            monthly_hoa_usd: 0.0,
            pmi_annual_rate_pct: 0.0,
            extra_principal_usd: 0.0,
        });
        assert_eq!(r.principal_financed_usd, 400_000.0);
        assert_eq!(r.ltv_pct, 80.0);
    }

    #[test]
    fn compute_pmi_only_when_ltv_above_80() {
        let r_under = compute(&MortgageInput {
            home_price_usd: 500_000.0,
            down_payment_usd: 100_000.0,  // 80% LTV exactly — no PMI
            apr_pct: 6.5,
            term_months: 360,
            annual_property_tax_usd: 0.0,
            annual_insurance_usd: 0.0,
            monthly_hoa_usd: 0.0,
            pmi_annual_rate_pct: 0.8,
            extra_principal_usd: 0.0,
        });
        assert_eq!(r_under.monthly_pmi_usd, 0.0);

        let r_over = compute(&MortgageInput {
            home_price_usd: 500_000.0,
            down_payment_usd: 50_000.0,  // 90% LTV — PMI applies
            apr_pct: 6.5,
            term_months: 360,
            annual_property_tax_usd: 0.0,
            annual_insurance_usd: 0.0,
            monthly_hoa_usd: 0.0,
            pmi_annual_rate_pct: 0.8,
            extra_principal_usd: 0.0,
        });
        assert!(r_over.monthly_pmi_usd > 0.0);
        assert_eq!(r_over.monthly_pmi_usd, 450_000.0 * 0.008 / 12.0);
    }

    #[test]
    fn compute_total_monthly_sums_all_components() {
        let r = compute(&MortgageInput {
            home_price_usd: 500_000.0,
            down_payment_usd: 50_000.0,
            apr_pct: 6.5,
            term_months: 360,
            annual_property_tax_usd: 6_000.0,
            annual_insurance_usd: 1_200.0,
            monthly_hoa_usd: 100.0,
            pmi_annual_rate_pct: 0.8,
            extra_principal_usd: 0.0,
        });
        let sum = r.monthly_pi_usd + r.monthly_pmi_usd + r.monthly_tax_usd + r.monthly_insurance_usd + r.monthly_hoa_usd;
        assert!((r.total_monthly_usd - sum).abs() < 0.01);
        assert!((r.monthly_tax_usd - 500.0).abs() < 0.01);
        assert!((r.monthly_insurance_usd - 100.0).abs() < 0.01);
    }

    #[test]
    fn compute_extra_payment_saves_months_and_interest() {
        let r = compute(&MortgageInput {
            home_price_usd: 400_000.0,
            down_payment_usd: 80_000.0,
            apr_pct: 6.5,
            term_months: 360,
            annual_property_tax_usd: 0.0,
            annual_insurance_usd: 0.0,
            monthly_hoa_usd: 0.0,
            pmi_annual_rate_pct: 0.0,
            extra_principal_usd: 200.0,
        });
        assert!(r.months_saved > 0);
        assert!(r.interest_saved_usd > 0.0);
    }

    #[test]
    fn compute_zero_principal_zero_pi() {
        let r = compute(&MortgageInput {
            home_price_usd: 0.0,
            down_payment_usd: 0.0,
            apr_pct: 6.5,
            term_months: 360,
            annual_property_tax_usd: 0.0,
            annual_insurance_usd: 0.0,
            monthly_hoa_usd: 0.0,
            pmi_annual_rate_pct: 0.0,
            extra_principal_usd: 0.0,
        });
        assert_eq!(r.principal_financed_usd, 0.0);
        assert_eq!(r.monthly_pi_usd, 0.0);
    }

    #[test]
    fn schedule_head_default_24_rows() {
        let r = compute(&MortgageInput {
            home_price_usd: 400_000.0,
            down_payment_usd: 80_000.0,
            apr_pct: 6.5,
            term_months: 360,
            annual_property_tax_usd: 0.0,
            annual_insurance_usd: 0.0,
            monthly_hoa_usd: 0.0,
            pmi_annual_rate_pct: 0.0,
            extra_principal_usd: 0.0,
        });
        assert_eq!(r.schedule_head.len(), 24);
    }

    #[test]
    fn ltv_zero_safe_on_zero_price() {
        let r = compute(&MortgageInput {
            home_price_usd: 0.0,
            down_payment_usd: 0.0,
            apr_pct: 6.5,
            term_months: 360,
            annual_property_tax_usd: 0.0,
            annual_insurance_usd: 0.0,
            monthly_hoa_usd: 0.0,
            pmi_annual_rate_pct: 0.0,
            extra_principal_usd: 0.0,
        });
        assert_eq!(r.ltv_pct, 0.0);
    }
}
