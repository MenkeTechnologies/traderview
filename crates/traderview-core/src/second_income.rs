//! Second-income worth-it analysis — what a household actually keeps from a
//! second earner's income after taxes, childcare, and work-related costs.
//!
//! The second income is taxed at the household's marginal rate (it stacks on
//! top of the first), then childcare and commuting/work costs come out:
//!
//! ```text
//! after-tax  = income × (1 − marginal rate)
//! net benefit = after-tax − childcare − commute − other work costs
//! keep rate   = net benefit / gross income
//! ```

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SecondIncomeInput {
    pub second_annual_income_usd: f64,
    /// Combined marginal rate (federal + state + FICA) on the second income.
    pub marginal_tax_rate_pct: f64,
    #[serde(default)]
    pub annual_childcare_usd: f64,
    #[serde(default)]
    pub annual_commute_usd: f64,
    #[serde(default)]
    pub annual_other_work_expenses_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SecondIncomeResult {
    pub taxes_usd: f64,
    pub after_tax_income_usd: f64,
    /// Childcare + commute + other work expenses.
    pub total_costs_usd: f64,
    /// After-tax income less all costs (what the household actually gains).
    pub net_benefit_usd: f64,
    /// Net benefit as a percent of gross second income.
    pub keep_rate_pct: f64,
    /// Whether the second income leaves the household ahead.
    pub worth_it: bool,
}

pub fn analyze(input: &SecondIncomeInput) -> SecondIncomeResult {
    let taxes = input.second_annual_income_usd * input.marginal_tax_rate_pct / 100.0;
    let after_tax = input.second_annual_income_usd - taxes;
    let costs =
        input.annual_childcare_usd + input.annual_commute_usd + input.annual_other_work_expenses_usd;
    let net = after_tax - costs;

    SecondIncomeResult {
        taxes_usd: taxes,
        after_tax_income_usd: after_tax,
        total_costs_usd: costs,
        net_benefit_usd: net,
        keep_rate_pct: if input.second_annual_income_usd > 0.0 {
            net / input.second_annual_income_usd * 100.0
        } else {
            0.0
        },
        worth_it: net > 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn base() -> SecondIncomeInput {
        SecondIncomeInput {
            second_annual_income_usd: 50_000.0,
            marginal_tax_rate_pct: 30.0,
            annual_childcare_usd: 18_000.0,
            annual_commute_usd: 3_000.0,
            annual_other_work_expenses_usd: 2_000.0,
        }
    }

    #[test]
    fn taxes_and_after_tax() {
        let r = analyze(&base());
        assert!(close(r.taxes_usd, 15_000.0));
        assert!(close(r.after_tax_income_usd, 35_000.0));
    }

    #[test]
    fn total_costs() {
        assert!(close(analyze(&base()).total_costs_usd, 23_000.0));
    }

    #[test]
    fn net_benefit() {
        // 35,000 − 23,000 = 12,000.
        assert!(close(analyze(&base()).net_benefit_usd, 12_000.0));
    }

    #[test]
    fn keep_rate() {
        // 12,000 / 50,000 = 24%.
        assert!(close(analyze(&base()).keep_rate_pct, 24.0));
    }

    #[test]
    fn worth_it_when_positive() {
        assert!(analyze(&base()).worth_it);
    }

    #[test]
    fn high_childcare_makes_it_not_worth_it() {
        let r = analyze(&SecondIncomeInput {
            annual_childcare_usd: 35_000.0,
            ..base()
        });
        assert!(r.net_benefit_usd < 0.0);
        assert!(!r.worth_it);
    }

    #[test]
    fn higher_tax_lowers_benefit() {
        let low = analyze(&base());
        let high = analyze(&SecondIncomeInput {
            marginal_tax_rate_pct: 45.0,
            ..base()
        });
        assert!(high.net_benefit_usd < low.net_benefit_usd);
    }

    #[test]
    fn no_costs_keeps_full_after_tax() {
        let r = analyze(&SecondIncomeInput {
            annual_childcare_usd: 0.0,
            annual_commute_usd: 0.0,
            annual_other_work_expenses_usd: 0.0,
            ..base()
        });
        assert!(close(r.net_benefit_usd, r.after_tax_income_usd));
        assert!(close(r.keep_rate_pct, 70.0));
    }
}
