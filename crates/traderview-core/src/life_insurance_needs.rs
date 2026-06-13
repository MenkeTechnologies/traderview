//! Life-insurance needs analysis (DIME-style) — how much coverage replaces lost
//! income and clears the family's obligations, net of what's already in place.
//!
//! ```text
//! income replacement = annual income × years to replace
//! total need = income replacement + mortgage + other debts
//!              + education + final expenses
//! coverage gap = total need − existing coverage − liquid savings   (floored at 0)
//! ```
//!
//! "DIME" = Debt, Income, Mortgage, Education — the obligations a policy should
//! cover so survivors aren't forced to sell assets or take on debt.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LifeInsuranceInput {
    pub annual_income_usd: f64,
    /// Years of income to replace.
    pub years_to_replace: f64,
    #[serde(default)]
    pub mortgage_balance_usd: f64,
    #[serde(default)]
    pub other_debts_usd: f64,
    #[serde(default)]
    pub education_costs_usd: f64,
    #[serde(default)]
    pub final_expenses_usd: f64,
    #[serde(default)]
    pub existing_coverage_usd: f64,
    #[serde(default)]
    pub existing_savings_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LifeInsuranceResult {
    /// annual income × years.
    pub income_replacement_usd: f64,
    /// Sum of every obligation the policy should cover.
    pub total_need_usd: f64,
    /// Existing coverage + liquid savings.
    pub total_offsets_usd: f64,
    /// Additional coverage to buy (floored at 0).
    pub coverage_gap_usd: f64,
    /// Existing resources beyond the need (0 if there's a gap).
    pub surplus_usd: f64,
}

pub fn analyze(input: &LifeInsuranceInput) -> LifeInsuranceResult {
    let income_replacement = input.annual_income_usd * input.years_to_replace;
    let total_need = income_replacement
        + input.mortgage_balance_usd
        + input.other_debts_usd
        + input.education_costs_usd
        + input.final_expenses_usd;
    let offsets = input.existing_coverage_usd + input.existing_savings_usd;
    let net = total_need - offsets;

    LifeInsuranceResult {
        income_replacement_usd: income_replacement,
        total_need_usd: total_need,
        total_offsets_usd: offsets,
        coverage_gap_usd: net.max(0.0),
        surplus_usd: (-net).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn base() -> LifeInsuranceInput {
        LifeInsuranceInput {
            annual_income_usd: 80_000.0,
            years_to_replace: 10.0,
            mortgage_balance_usd: 250_000.0,
            other_debts_usd: 20_000.0,
            education_costs_usd: 100_000.0,
            final_expenses_usd: 15_000.0,
            existing_coverage_usd: 200_000.0,
            existing_savings_usd: 50_000.0,
        }
    }

    #[test]
    fn income_replacement() {
        assert!(close(analyze(&base()).income_replacement_usd, 800_000.0));
    }

    #[test]
    fn total_need() {
        // 800k + 250k + 20k + 100k + 15k = 1,185,000.
        assert!(close(analyze(&base()).total_need_usd, 1_185_000.0));
    }

    #[test]
    fn offsets() {
        assert!(close(analyze(&base()).total_offsets_usd, 250_000.0));
    }

    #[test]
    fn coverage_gap() {
        assert!(close(analyze(&base()).coverage_gap_usd, 935_000.0));
        assert!(close(analyze(&base()).surplus_usd, 0.0));
    }

    #[test]
    fn over_insured_shows_surplus() {
        let r = analyze(&LifeInsuranceInput {
            existing_coverage_usd: 2_000_000.0,
            ..base()
        });
        assert!(close(r.coverage_gap_usd, 0.0));
        assert!(r.surplus_usd > 0.0);
    }

    #[test]
    fn existing_coverage_reduces_gap() {
        let less = analyze(&LifeInsuranceInput {
            existing_coverage_usd: 100_000.0,
            ..base()
        });
        let more = analyze(&LifeInsuranceInput {
            existing_coverage_usd: 400_000.0,
            ..base()
        });
        assert!(more.coverage_gap_usd < less.coverage_gap_usd);
    }

    #[test]
    fn no_obligations_no_gap() {
        let r = analyze(&LifeInsuranceInput {
            annual_income_usd: 0.0,
            years_to_replace: 0.0,
            mortgage_balance_usd: 0.0,
            other_debts_usd: 0.0,
            education_costs_usd: 0.0,
            final_expenses_usd: 0.0,
            existing_coverage_usd: 0.0,
            existing_savings_usd: 0.0,
        });
        assert!(close(r.coverage_gap_usd, 0.0));
    }

    #[test]
    fn more_years_widens_gap() {
        let short = analyze(&LifeInsuranceInput {
            years_to_replace: 5.0,
            ..base()
        });
        let long = analyze(&LifeInsuranceInput {
            years_to_replace: 20.0,
            ..base()
        });
        assert!(long.coverage_gap_usd > short.coverage_gap_usd);
    }
}
