//! Rent affordability — the most rent you can comfortably afford.
//!
//! Two common rules:
//! * The **30% rule** caps rent at 30% of gross monthly income (the same as the
//!   landlord "40× annual income" rule, since 30%/12 = 1/40).
//! * A **debt-adjusted** cap keeps rent + other debts within a back-end limit
//!   (commonly 40% of income).
//!
//! ```text
//! 30% cap        = 0.30 × monthly income
//! debt-adjusted  = back-end % × monthly income − monthly debts
//! recommended    = the lower of the two
//! ```

use serde::{Deserialize, Serialize};

fn d_rent() -> f64 {
    30.0
}
fn d_back() -> f64 {
    40.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentAffordInput {
    pub annual_income_usd: f64,
    #[serde(default)]
    pub monthly_debts_usd: f64,
    #[serde(default = "d_rent")]
    pub rent_pct: f64,
    #[serde(default = "d_back")]
    pub back_end_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RentAffordResult {
    pub monthly_income_usd: f64,
    /// Rent under the income (30%) rule.
    pub max_rent_income_rule_usd: f64,
    /// Rent under the debt-adjusted (back-end) rule.
    pub max_rent_debt_adjusted_usd: f64,
    /// The lower of the two — the recommended ceiling.
    pub recommended_max_rent_usd: f64,
    /// Which rule binds: "income" or "debts".
    pub binding_constraint: String,
}

pub fn analyze(input: &RentAffordInput) -> RentAffordResult {
    let monthly_income = input.annual_income_usd / 12.0;
    let by_income = input.rent_pct / 100.0 * monthly_income;
    let by_debts = (input.back_end_pct / 100.0 * monthly_income - input.monthly_debts_usd).max(0.0);

    let (recommended, binding) = if by_income <= by_debts {
        (by_income, "income")
    } else {
        (by_debts, "debts")
    };

    RentAffordResult {
        monthly_income_usd: monthly_income,
        max_rent_income_rule_usd: by_income,
        max_rent_debt_adjusted_usd: by_debts,
        recommended_max_rent_usd: recommended,
        binding_constraint: binding.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn base() -> RentAffordInput {
        RentAffordInput {
            annual_income_usd: 60_000.0,
            monthly_debts_usd: 300.0,
            rent_pct: 30.0,
            back_end_pct: 40.0,
        }
    }

    #[test]
    fn monthly_income() {
        assert!(close(analyze(&base()).monthly_income_usd, 5_000.0));
    }

    #[test]
    fn income_rule_equals_40x() {
        // 30% × 5,000 = 1,500 = 60,000 / 40.
        let r = analyze(&base());
        assert!(close(r.max_rent_income_rule_usd, 1_500.0));
        assert!(close(r.max_rent_income_rule_usd, 60_000.0 / 40.0));
    }

    #[test]
    fn debt_adjusted_rule() {
        // 40% × 5,000 − 300 = 1,700.
        assert!(close(analyze(&base()).max_rent_debt_adjusted_usd, 1_700.0));
    }

    #[test]
    fn income_binds_with_low_debts() {
        let r = analyze(&base());
        assert!(close(r.recommended_max_rent_usd, 1_500.0));
        assert_eq!(r.binding_constraint, "income");
    }

    #[test]
    fn debts_bind_when_high() {
        let r = analyze(&RentAffordInput {
            monthly_debts_usd: 1_000.0,
            ..base()
        });
        // 40% × 5,000 − 1,000 = 1,000 < 1,500.
        assert!(close(r.recommended_max_rent_usd, 1_000.0));
        assert_eq!(r.binding_constraint, "debts");
    }

    #[test]
    fn higher_income_affords_more() {
        let low = analyze(&base());
        let high = analyze(&RentAffordInput {
            annual_income_usd: 120_000.0,
            ..base()
        });
        assert!(high.recommended_max_rent_usd > low.recommended_max_rent_usd);
    }

    #[test]
    fn crushing_debts_floor_at_zero() {
        let r = analyze(&RentAffordInput {
            monthly_debts_usd: 5_000.0,
            ..base()
        });
        assert!(close(r.max_rent_debt_adjusted_usd, 0.0));
        assert!(close(r.recommended_max_rent_usd, 0.0));
    }

    #[test]
    fn custom_rent_pct() {
        let r = analyze(&RentAffordInput {
            rent_pct: 25.0,
            ..base()
        });
        assert!(close(r.max_rent_income_rule_usd, 1_250.0));
    }
}
