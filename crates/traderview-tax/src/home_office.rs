//! Home office deduction — IRC § 280A, simplified-method safe harbor
//! per Rev. Proc. 2013-13.
//!
//! Two methods to claim the deduction (taxpayer picks per year, can
//! switch between years):
//!
//! ### Simplified method (Rev. Proc. 2013-13)
//!
//! * $5 per square foot of qualified home-office space, **maximum 300
//!   square feet → $1,500 cap**.
//! * No depreciation recapture on sale.
//! * Itemized deductions for the home (mortgage interest, real-estate
//!   tax) stay 100% on Schedule A — they aren't pro-rated to the
//!   office's business-use %.
//! * No carryover of disallowed amount.
//!
//! ### Actual-expense method (Form 8829)
//!
//! * Allocate total home expenses by business-use % = office sqft /
//!   total home sqft (or rooms, if rooms are similar size).
//! * Deductible components: utilities, repairs, insurance, rent (if
//!   renting), depreciation (if owned), security system, HOA fees.
//! * Mortgage interest + RE tax: business portion allowed here AND
//!   personal portion stays on Schedule A.
//! * Depreciation on sale triggers § 1250 recapture at 25%.
//! * Carryover of disallowed amount (over net business income limit)
//!   to future years.
//!
//! ### Income limitation (both methods)
//!
//! Home office deduction cannot create or increase a net loss from
//! the business — capped at gross business income minus other
//! business expenses. Excess carries forward (actual method) or
//! disappears (simplified method).
//!
//! Sources:
//!   * IRC § 280A
//!   * Rev. Proc. 2013-13 (simplified method)
//!   * Form 8829 instructions
//!   * IRS Pub 587 (Business Use of Your Home)

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// $5/sqft per Rev. Proc. 2013-13 § 4.01. NOT inflation-adjusted.
pub const SIMPLIFIED_RATE_PER_SQFT: &str = "5.00";
/// 300 sqft cap per Rev. Proc. 2013-13 § 4.02.
pub const SIMPLIFIED_MAX_SQFT: u32 = 300;
/// Resulting maximum deduction: 300 × $5 = $1,500.
pub const SIMPLIFIED_MAX_DEDUCTION: i64 = 1_500;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HomeOfficeInput {
    /// Square footage of the dedicated office space.
    pub office_sqft: u32,
    /// Total square footage of the home (used for actual-method % allocation).
    pub home_sqft: u32,
    /// Total annual home expenses eligible under the actual method —
    /// utilities + insurance + rent + repairs + depreciation + HOA.
    /// Mortgage interest and RE tax are passed separately because they
    /// get a different treatment (kept on Schedule A in full under
    /// simplified, split under actual).
    pub allocable_home_expenses: Decimal,
    /// Gross income from the business minus other business expenses.
    /// Home-office deduction can't exceed this — § 280A(c)(5) income cap.
    pub business_income_after_other_expenses: Decimal,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct HomeOfficeResult {
    /// Sqft used for the simplified calc (capped at 300).
    pub simplified_qualified_sqft: u32,
    /// Pre-cap simplified amount: qualified_sqft × $5.
    pub simplified_pre_income_cap: Decimal,
    /// Simplified amount after income cap.
    pub simplified_deduction: Decimal,
    /// Allocation percent for actual method = office / home (in [0, 1]).
    pub actual_business_use_pct: Decimal,
    /// Pre-cap actual amount: allocable expenses × business-use %.
    pub actual_pre_income_cap: Decimal,
    /// Actual amount after income cap.
    pub actual_deduction: Decimal,
    /// "simplified" or "actual" — whichever yields more.
    pub better_method: &'static str,
    /// Higher of the two final deductions.
    pub max_deduction: Decimal,
}

pub fn compute(input: HomeOfficeInput) -> HomeOfficeResult {
    let income_cap = input
        .business_income_after_other_expenses
        .max(Decimal::ZERO);

    // Simplified method.
    let qualified_sqft = input.office_sqft.min(SIMPLIFIED_MAX_SQFT);
    let rate: Decimal = SIMPLIFIED_RATE_PER_SQFT.parse().unwrap();
    let simplified_pre = Decimal::from(qualified_sqft) * rate;
    let simplified_max = Decimal::from(SIMPLIFIED_MAX_DEDUCTION);
    let simplified_pre_capped = simplified_pre.min(simplified_max);
    let simplified_deduction = simplified_pre_capped.min(income_cap);

    // Actual method.
    let actual_business_use_pct = if input.home_sqft == 0 {
        Decimal::ZERO
    } else {
        let office_d = Decimal::from(input.office_sqft);
        let home_d = Decimal::from(input.home_sqft);
        (office_d / home_d)
            .max(Decimal::ZERO)
            .min(Decimal::ONE)
            .round_dp(6)
    };
    let actual_pre = (input.allocable_home_expenses * actual_business_use_pct).round_dp(2);
    let actual_deduction = actual_pre.min(income_cap).max(Decimal::ZERO);

    let (better_method, max_deduction) = if simplified_deduction >= actual_deduction {
        ("simplified", simplified_deduction)
    } else {
        ("actual", actual_deduction)
    };

    HomeOfficeResult {
        simplified_qualified_sqft: qualified_sqft,
        simplified_pre_income_cap: simplified_pre_capped,
        simplified_deduction,
        actual_business_use_pct,
        actual_pre_income_cap: actual_pre,
        actual_deduction,
        better_method,
        max_deduction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: i64) -> Decimal {
        Decimal::from(n)
    }
    fn dc(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    #[test]
    fn simplified_max_deduction_at_300_sqft() {
        let r = compute(HomeOfficeInput {
            office_sqft: 300,
            home_sqft: 2_000,
            allocable_home_expenses: d(20_000),
            business_income_after_other_expenses: d(50_000),
        });
        assert_eq!(r.simplified_pre_income_cap, d(1_500));
        assert_eq!(r.simplified_deduction, d(1_500));
    }

    #[test]
    fn simplified_caps_at_300_sqft_even_with_400_sqft_office() {
        let r = compute(HomeOfficeInput {
            office_sqft: 400,
            home_sqft: 2_000,
            allocable_home_expenses: d(20_000),
            business_income_after_other_expenses: d(50_000),
        });
        assert_eq!(r.simplified_qualified_sqft, 300);
        assert_eq!(r.simplified_deduction, d(1_500));
    }

    #[test]
    fn simplified_low_sqft_lower_deduction() {
        // 100 sqft × $5 = $500.
        let r = compute(HomeOfficeInput {
            office_sqft: 100,
            home_sqft: 2_000,
            allocable_home_expenses: d(20_000),
            business_income_after_other_expenses: d(50_000),
        });
        assert_eq!(r.simplified_deduction, d(500));
    }

    #[test]
    fn actual_method_uses_sqft_allocation() {
        // Office 200 / home 2000 = 10%. Expenses $20k × 10% = $2,000.
        let r = compute(HomeOfficeInput {
            office_sqft: 200,
            home_sqft: 2_000,
            allocable_home_expenses: d(20_000),
            business_income_after_other_expenses: d(50_000),
        });
        assert_eq!(r.actual_business_use_pct, dc("0.1"));
        assert_eq!(r.actual_deduction, d(2_000));
    }

    #[test]
    fn actual_method_wins_when_high_expenses_high_pct() {
        // 250 / 1000 = 25%. $30k × 25% = $7,500. Simplified at 250 sqft = $1,250.
        let r = compute(HomeOfficeInput {
            office_sqft: 250,
            home_sqft: 1_000,
            allocable_home_expenses: d(30_000),
            business_income_after_other_expenses: d(100_000),
        });
        assert_eq!(r.actual_deduction, d(7_500));
        assert_eq!(r.simplified_deduction, d(1_250));
        assert_eq!(r.better_method, "actual");
        assert_eq!(r.max_deduction, d(7_500));
    }

    #[test]
    fn simplified_wins_when_actual_pct_is_tiny() {
        // 100 sqft office / 5000 sqft home = 2%. $20k × 2% = $400.
        // Simplified at 100 sqft = $500.
        let r = compute(HomeOfficeInput {
            office_sqft: 100,
            home_sqft: 5_000,
            allocable_home_expenses: d(20_000),
            business_income_after_other_expenses: d(50_000),
        });
        assert_eq!(r.actual_deduction, d(400));
        assert_eq!(r.simplified_deduction, d(500));
        assert_eq!(r.better_method, "simplified");
    }

    #[test]
    fn income_cap_clamps_both_methods() {
        // Big office, big expenses, but business income only $200.
        let r = compute(HomeOfficeInput {
            office_sqft: 300,
            home_sqft: 1_000,
            allocable_home_expenses: d(10_000),
            business_income_after_other_expenses: d(200),
        });
        assert_eq!(r.simplified_pre_income_cap, d(1_500));
        assert_eq!(r.simplified_deduction, d(200)); // capped at income
        assert_eq!(r.actual_pre_income_cap, d(3_000));
        assert_eq!(r.actual_deduction, d(200));
    }

    #[test]
    fn negative_business_income_yields_zero() {
        // § 280A(c)(5): deduction can't create a loss.
        let r = compute(HomeOfficeInput {
            office_sqft: 200,
            home_sqft: 1_000,
            allocable_home_expenses: d(10_000),
            business_income_after_other_expenses: d(-5_000),
        });
        assert_eq!(r.simplified_deduction, Decimal::ZERO);
        assert_eq!(r.actual_deduction, Decimal::ZERO);
    }

    #[test]
    fn zero_home_sqft_doesnt_div_by_zero() {
        let r = compute(HomeOfficeInput {
            office_sqft: 200,
            home_sqft: 0,
            allocable_home_expenses: d(10_000),
            business_income_after_other_expenses: d(50_000),
        });
        assert_eq!(r.actual_business_use_pct, Decimal::ZERO);
        assert_eq!(r.actual_deduction, Decimal::ZERO);
    }

    #[test]
    fn zero_office_sqft_zero_deduction() {
        let r = compute(HomeOfficeInput {
            office_sqft: 0,
            home_sqft: 2_000,
            allocable_home_expenses: d(20_000),
            business_income_after_other_expenses: d(50_000),
        });
        assert_eq!(r.simplified_deduction, Decimal::ZERO);
        assert_eq!(r.actual_deduction, Decimal::ZERO);
    }

    #[test]
    fn office_larger_than_home_clamps_pct_at_1() {
        // Defensive: bad input where office > home.
        let r = compute(HomeOfficeInput {
            office_sqft: 5_000,
            home_sqft: 1_000,
            allocable_home_expenses: d(10_000),
            business_income_after_other_expenses: d(50_000),
        });
        assert_eq!(r.actual_business_use_pct, Decimal::ONE);
    }
}
