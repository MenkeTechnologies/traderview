//! Debt yield & loan sizing — the commercial-RE lender constraints.
//!
//! A lender sizes a commercial real-estate loan against three ceilings, and
//! the smallest wins:
//!
//!   * **Debt yield** = NOI / loan. A rate- and amortization-independent
//!     measure of risk; lenders want it above a floor (often 8–10%). Max loan
//!     = NOI / floor.
//!   * **LTV** = loan / appraised value. Max loan = value × max LTV.
//!   * **LTC** = loan / total project cost. Max loan = cost × max LTC.
//!
//! Reports each ratio for a proposed loan, the max loan each ceiling allows,
//! the binding constraint, and whether the proposed loan fits. Complements
//! cap rate (NOI/value) and DSCR (NOI/debt-service). Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DebtYieldInput {
    pub noi_usd: f64,
    pub property_value_usd: f64,
    /// Total project cost (purchase + rehab) for the LTC test.
    pub total_project_cost_usd: f64,
    /// Proposed loan amount to evaluate.
    pub loan_amount_usd: f64,
    /// Lender's minimum debt yield (e.g. 10).
    pub min_debt_yield_pct: f64,
    /// Lender's maximum LTV (e.g. 75).
    pub max_ltv_pct: f64,
    /// Lender's maximum LTC (e.g. 80).
    pub max_ltc_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DebtYieldResult {
    pub debt_yield_pct: f64,
    pub ltv_pct: f64,
    pub ltc_pct: f64,
    pub max_loan_by_debt_yield_usd: f64,
    pub max_loan_by_ltv_usd: f64,
    pub max_loan_by_ltc_usd: f64,
    /// The smallest of the three ceilings — the actual max loan.
    pub max_loan_usd: f64,
    /// Which ceiling binds: "debt_yield" / "ltv" / "ltc".
    pub binding_constraint: String,
    /// True when the proposed loan is within the max.
    pub loan_fits: bool,
}

pub fn analyze(i: &DebtYieldInput) -> DebtYieldResult {
    let loan = i.loan_amount_usd.max(0.0);

    let debt_yield = if loan > 0.0 { i.noi_usd / loan * 100.0 } else { 0.0 };
    let ltv = if i.property_value_usd > 0.0 { loan / i.property_value_usd * 100.0 } else { 0.0 };
    let ltc = if i.total_project_cost_usd > 0.0 { loan / i.total_project_cost_usd * 100.0 } else { 0.0 };

    let max_by_dy = if i.min_debt_yield_pct > 0.0 { i.noi_usd / (i.min_debt_yield_pct / 100.0) } else { f64::INFINITY };
    let max_by_ltv = i.property_value_usd * i.max_ltv_pct / 100.0;
    let max_by_ltc = i.total_project_cost_usd * i.max_ltc_pct / 100.0;

    // The binding ceiling is the smallest.
    let mut max_loan = max_by_dy;
    let mut binding = "debt_yield";
    if max_by_ltv < max_loan {
        max_loan = max_by_ltv;
        binding = "ltv";
    }
    if max_by_ltc < max_loan {
        max_loan = max_by_ltc;
        binding = "ltc";
    }

    DebtYieldResult {
        debt_yield_pct: debt_yield,
        ltv_pct: ltv,
        ltc_pct: ltc,
        max_loan_by_debt_yield_usd: if max_by_dy.is_finite() { max_by_dy } else { 0.0 },
        max_loan_by_ltv_usd: max_by_ltv,
        max_loan_by_ltc_usd: max_by_ltc,
        max_loan_usd: if max_loan.is_finite() { max_loan } else { 0.0 },
        binding_constraint: binding.to_string(),
        loan_fits: max_loan.is_finite() && loan <= max_loan,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> DebtYieldInput {
        DebtYieldInput {
            noi_usd: 100_000.0,
            property_value_usd: 1_400_000.0,
            total_project_cost_usd: 1_350_000.0,
            loan_amount_usd: 1_000_000.0,
            min_debt_yield_pct: 10.0,
            max_ltv_pct: 75.0,
            max_ltc_pct: 80.0,
        }
    }

    #[test]
    fn debt_yield_is_noi_over_loan() {
        // 100k / 1M = 10%.
        let r = analyze(&base());
        assert!((r.debt_yield_pct - 10.0).abs() < 1e-9);
    }

    #[test]
    fn ltv_and_ltc() {
        let r = analyze(&base());
        assert!((r.ltv_pct - 1_000_000.0 / 1_400_000.0 * 100.0).abs() < 1e-9);
        assert!((r.ltc_pct - 1_000_000.0 / 1_350_000.0 * 100.0).abs() < 1e-9);
    }

    #[test]
    fn max_loan_by_debt_yield() {
        // NOI 100k / 10% = 1,000,000.
        let r = analyze(&base());
        assert!((r.max_loan_by_debt_yield_usd - 1_000_000.0).abs() < 1e-6);
    }

    #[test]
    fn max_loan_by_ltv_and_ltc() {
        let r = analyze(&base());
        assert!((r.max_loan_by_ltv_usd - 1_050_000.0).abs() < 1e-6); // 1.4M × 75%
        assert!((r.max_loan_by_ltc_usd - 1_080_000.0).abs() < 1e-6); // 1.35M × 80%
    }

    #[test]
    fn binding_constraint_is_the_smallest() {
        // DY 1.0M < LTV 1.05M < LTC 1.08M → debt yield binds.
        let r = analyze(&base());
        assert_eq!(r.binding_constraint, "debt_yield");
        assert!((r.max_loan_usd - 1_000_000.0).abs() < 1e-6);
    }

    #[test]
    fn ltv_binds_when_value_is_low() {
        // Low value makes LTV ceiling the smallest.
        let r = analyze(&DebtYieldInput { property_value_usd: 1_000_000.0, ..base() });
        // LTV max = 750k, DY max = 1.0M, LTC max = 1.08M → LTV binds.
        assert_eq!(r.binding_constraint, "ltv");
        assert!((r.max_loan_usd - 750_000.0).abs() < 1e-6);
    }

    #[test]
    fn loan_fits_within_max() {
        // Proposed 1.0M ≤ max 1.0M → fits (boundary).
        assert!(analyze(&base()).loan_fits);
        // Over the max → doesn't fit.
        let over = analyze(&DebtYieldInput { loan_amount_usd: 1_200_000.0, ..base() });
        assert!(!over.loan_fits);
    }

    #[test]
    fn zero_loan_guards_ratios() {
        let r = analyze(&DebtYieldInput { loan_amount_usd: 0.0, ..base() });
        assert!(r.debt_yield_pct.abs() < 1e-9);
        assert!(r.ltv_pct.abs() < 1e-9);
        assert!(r.loan_fits); // 0 ≤ any max
    }
}
