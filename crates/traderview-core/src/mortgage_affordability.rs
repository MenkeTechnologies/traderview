//! Mortgage affordability — the most house you can buy under the 28/36 rule.
//!
//! Lenders cap housing cost (PITI) at ~28% of gross monthly income (front-end)
//! and total debt at ~36% (back-end). The tighter cap sets the max PITI; since
//! property tax and the loan payment both scale with price, solve for it:
//!
//! ```text
//! max PITI = (price − down)·pf + price·(tax rate/12) + insurance/12
//! price = (max PITI − insurance/12 + down·pf) / (pf + tax rate/12)
//! ```
//!
//! where `pf` is the per-dollar mortgage payment factor.

use serde::{Deserialize, Serialize};

fn d_front() -> f64 {
    28.0
}
fn d_back() -> f64 {
    36.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct AffordabilityInput {
    pub annual_income_usd: f64,
    /// Other monthly debt payments (cards, auto, student loans).
    #[serde(default)]
    pub monthly_debts_usd: f64,
    pub down_payment_usd: f64,
    pub annual_rate_pct: f64,
    pub term_months: f64,
    /// Annual property tax as a percent of home value.
    #[serde(default)]
    pub property_tax_rate_pct: f64,
    /// Annual homeowners insurance (dollars).
    #[serde(default)]
    pub annual_insurance_usd: f64,
    #[serde(default = "d_front")]
    pub front_end_pct: f64,
    #[serde(default = "d_back")]
    pub back_end_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AffordabilityResult {
    pub monthly_income_usd: f64,
    /// Front-end cap (housing only).
    pub front_end_max_usd: f64,
    /// Back-end cap less other debts (housing room within total-debt limit).
    pub back_end_max_usd: f64,
    /// The binding monthly housing budget (lesser of the two).
    pub max_piti_usd: f64,
    /// Largest loan the budget supports.
    pub max_loan_usd: f64,
    /// Largest home price (loan + down payment).
    pub max_home_price_usd: f64,
    /// Which cap binds: "front" or "back".
    pub binding_constraint: String,
}

pub fn analyze(input: &AffordabilityInput) -> AffordabilityResult {
    let monthly_income = input.annual_income_usd / 12.0;
    let front_max = input.front_end_pct / 100.0 * monthly_income;
    let back_max = (input.back_end_pct / 100.0 * monthly_income - input.monthly_debts_usd).max(0.0);

    let (max_piti, binding) = if front_max <= back_max {
        (front_max, "front")
    } else {
        (back_max, "back")
    };

    let i = input.annual_rate_pct / 100.0 / 12.0;
    let n = input.term_months;
    let pf = if i.abs() < 1e-12 {
        if n > 0.0 { 1.0 / n } else { 0.0 }
    } else {
        let f = (1.0 + i).powf(n);
        i * f / (f - 1.0)
    };

    let tax_monthly_factor = input.property_tax_rate_pct / 100.0 / 12.0;
    let insurance_monthly = input.annual_insurance_usd / 12.0;

    let denom = pf + tax_monthly_factor;
    let price = if denom > 0.0 {
        ((max_piti - insurance_monthly + input.down_payment_usd * pf) / denom).max(0.0)
    } else {
        0.0
    };
    let loan = (price - input.down_payment_usd).max(0.0);

    AffordabilityResult {
        monthly_income_usd: monthly_income,
        front_end_max_usd: front_max,
        back_end_max_usd: back_max,
        max_piti_usd: max_piti,
        max_loan_usd: loan,
        max_home_price_usd: price,
        binding_constraint: binding.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> AffordabilityInput {
        AffordabilityInput {
            annual_income_usd: 100_000.0,
            monthly_debts_usd: 500.0,
            down_payment_usd: 50_000.0,
            annual_rate_pct: 6.5,
            term_months: 360.0,
            property_tax_rate_pct: 1.2,
            annual_insurance_usd: 1_500.0,
            front_end_pct: 28.0,
            back_end_pct: 36.0,
        }
    }

    #[test]
    fn income_and_caps() {
        let r = analyze(&base());
        assert!(close(r.monthly_income_usd, 8_333.3333));
        assert!(close(r.front_end_max_usd, 2_333.3333));
        assert!(close(r.back_end_max_usd, 2_500.0));
    }

    #[test]
    fn front_end_binds_here() {
        let r = analyze(&base());
        assert_eq!(r.binding_constraint, "front");
        assert!(close(r.max_piti_usd, 2_333.3333));
    }

    #[test]
    fn max_price_and_loan() {
        let r = analyze(&base());
        assert!(close(r.max_home_price_usd, 344_826.8828));
        assert!(close(r.max_loan_usd, 294_826.8828));
    }

    #[test]
    fn back_end_binds_with_high_debts() {
        let r = analyze(&AffordabilityInput {
            monthly_debts_usd: 1_500.0,
            ..base()
        });
        // Back-end room = 3000 − 1500 = 1500 < front 2333 → back binds.
        assert_eq!(r.binding_constraint, "back");
        assert!(close(r.max_piti_usd, 1_500.0));
    }

    #[test]
    fn higher_income_affords_more() {
        let low = analyze(&base());
        let high = analyze(&AffordabilityInput {
            annual_income_usd: 200_000.0,
            ..base()
        });
        assert!(high.max_home_price_usd > low.max_home_price_usd);
    }

    #[test]
    fn larger_down_payment_raises_price() {
        let small = analyze(&base());
        let big = analyze(&AffordabilityInput {
            down_payment_usd: 100_000.0,
            ..base()
        });
        assert!(big.max_home_price_usd > small.max_home_price_usd);
    }

    #[test]
    fn higher_rate_lowers_price() {
        let low = analyze(&base());
        let high = analyze(&AffordabilityInput {
            annual_rate_pct: 9.0,
            ..base()
        });
        assert!(high.max_home_price_usd < low.max_home_price_usd);
    }

    #[test]
    fn loan_plus_down_equals_price() {
        let r = analyze(&base());
        assert!(close(r.max_loan_usd + 50_000.0, r.max_home_price_usd));
    }
}
