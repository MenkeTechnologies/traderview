//! Standard mileage deduction calculator — IRC § 274(d), Rev. Proc.
//! 2019-46 (substantiation), and the annual IRS mileage-rate notices.
//!
//! ### 2025 rates (IRS Notice 2024-79)
//!
//!   * **Business**:   $0.70/mile  (Schedule C / Form 2106)
//!   * **Medical**:    $0.21/mile  (Schedule A line 1, subject to 7.5% AGI floor)
//!   * **Charitable**: $0.14/mile  (Schedule A line 12, NOT inflation-adjusted —
//!     set by statute at IRC § 170(i))
//!   * **Moving (active-duty military only)**: $0.21/mile (Form 3903)
//!
//! ### What counts
//!
//! * Business: Travel between job sites, client visits, business
//!   errands. The commute from home to a regular workplace is NOT
//!   deductible (Comm'r v. Flowers, 326 U.S. 465).
//! * Medical: Travel to/from medical care. Lodging incidental to
//!   medical care is $50/night max (separate from mileage).
//! * Charitable: Volunteer driving for a qualified § 501(c)(3) org.
//!
//! ### Standard vs Actual
//!
//! Standard mileage is one of two methods (the other is actual
//! expenses — gas + maintenance + depreciation + insurance, allocated
//! by business-use %). Taxpayer chooses per vehicle in year 1. If
//! they start with standard, they may switch to actual; if they start
//! with actual using accelerated depreciation, they're locked into
//! actual for that vehicle's life.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 2025 IRS standard mileage rates ($/mile) — Notice 2024-79.
pub mod rates_2025 {
    pub const BUSINESS: &str = "0.70";
    pub const MEDICAL: &str = "0.21";
    /// Set by statute (IRC § 170(i)), not inflation-adjusted.
    pub const CHARITABLE: &str = "0.14";
    pub const MOVING_ACTIVE_DUTY: &str = "0.21";
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MileageInput {
    /// Business miles driven (Schedule C line 9 via Form 2106).
    pub business_miles: u32,
    /// Medical miles driven (Schedule A line 1 component).
    pub medical_miles: u32,
    /// Charitable miles driven (Schedule A line 12 component).
    pub charitable_miles: u32,
    /// Moving miles — ONLY deductible for active-duty military post-TCJA.
    pub active_duty_moving_miles: u32,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct MileageResult {
    /// Business mileage × $0.70.
    pub business_deduction: Decimal,
    /// Medical mileage × $0.21. Subject to 7.5% AGI floor on Schedule A.
    pub medical_deduction: Decimal,
    /// Charitable mileage × $0.14.
    pub charitable_deduction: Decimal,
    /// Active-duty military moving mileage × $0.21.
    pub moving_deduction: Decimal,
    /// Total dollars from all four categories.
    pub total: Decimal,
}

pub fn compute(input: MileageInput) -> MileageResult {
    let biz_rate: Decimal = rates_2025::BUSINESS.parse().unwrap();
    let med_rate: Decimal = rates_2025::MEDICAL.parse().unwrap();
    let char_rate: Decimal = rates_2025::CHARITABLE.parse().unwrap();
    let mov_rate: Decimal = rates_2025::MOVING_ACTIVE_DUTY.parse().unwrap();

    let business = (Decimal::from(input.business_miles) * biz_rate).round_dp(2);
    let medical = (Decimal::from(input.medical_miles) * med_rate).round_dp(2);
    let charitable = (Decimal::from(input.charitable_miles) * char_rate).round_dp(2);
    let moving = (Decimal::from(input.active_duty_moving_miles) * mov_rate).round_dp(2);
    let total = business + medical + charitable + moving;

    MileageResult {
        business_deduction: business,
        medical_deduction: medical,
        charitable_deduction: charitable,
        moving_deduction: moving,
        total,
    }
}

/// Comparison helper: compare standard-mileage vs actual-expense methods
/// to surface the higher deduction for a single vehicle.
///
/// `actual_total_expenses` is the sum of gas, maintenance, insurance,
/// registration, lease payments / depreciation × business-use %.
/// `business_use_pct` is in [0.0, 1.0]. The 0.575 in IRS Pub 463 example
/// would be passed as `0.575`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MethodComparisonInput {
    pub business_miles: u32,
    pub actual_total_expenses: Decimal,
    pub business_use_pct: Decimal,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct MethodComparisonResult {
    pub standard_method_deduction: Decimal,
    pub actual_method_deduction: Decimal,
    /// "standard" when standard ≥ actual, else "actual".
    pub better_method: &'static str,
    /// Higher of the two.
    pub max_deduction: Decimal,
    /// Dollar advantage of `better_method` over the loser.
    pub advantage: Decimal,
}

pub fn compare_methods(input: MethodComparisonInput) -> MethodComparisonResult {
    let biz_rate: Decimal = rates_2025::BUSINESS.parse().unwrap();
    let standard = (Decimal::from(input.business_miles) * biz_rate).round_dp(2);
    let pct = input.business_use_pct.max(Decimal::ZERO).min(Decimal::ONE);
    let actual = (input.actual_total_expenses * pct).round_dp(2);

    let (better_method, max_deduction) = if standard >= actual {
        ("standard", standard)
    } else {
        ("actual", actual)
    };
    let advantage = (standard - actual).abs();

    MethodComparisonResult {
        standard_method_deduction: standard,
        actual_method_deduction: actual,
        better_method,
        max_deduction,
        advantage,
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
    fn ten_thousand_business_miles_at_70_cents() {
        let r = compute(MileageInput {
            business_miles: 10_000,
            medical_miles: 0,
            charitable_miles: 0,
            active_duty_moving_miles: 0,
        });
        assert_eq!(r.business_deduction, dc("7000"));
        assert_eq!(r.total, dc("7000"));
    }

    #[test]
    fn all_four_categories_sum_correctly() {
        let r = compute(MileageInput {
            business_miles: 1_000,         // $700
            medical_miles: 500,            // $105
            charitable_miles: 200,         // $28
            active_duty_moving_miles: 100, // $21
        });
        assert_eq!(r.business_deduction, d(700));
        assert_eq!(r.medical_deduction, d(105));
        assert_eq!(r.charitable_deduction, d(28));
        assert_eq!(r.moving_deduction, d(21));
        assert_eq!(r.total, d(854));
    }

    #[test]
    fn zero_miles_zero_deduction() {
        let r = compute(MileageInput {
            business_miles: 0,
            medical_miles: 0,
            charitable_miles: 0,
            active_duty_moving_miles: 0,
        });
        assert_eq!(r.total, Decimal::ZERO);
    }

    #[test]
    fn charitable_rate_is_14_cents_per_statute() {
        // IRC § 170(i) — Congress-set, not inflation-adjusted.
        let r = compute(MileageInput {
            business_miles: 0,
            medical_miles: 0,
            charitable_miles: 100,
            active_duty_moving_miles: 0,
        });
        assert_eq!(r.charitable_deduction, d(14));
    }

    #[test]
    fn rounding_at_cents() {
        // 1 mile × $0.70 = $0.70 (clean). 3 miles × $0.21 = $0.63.
        let r = compute(MileageInput {
            business_miles: 1,
            medical_miles: 3,
            charitable_miles: 0,
            active_duty_moving_miles: 0,
        });
        assert_eq!(r.business_deduction, dc("0.70"));
        assert_eq!(r.medical_deduction, dc("0.63"));
    }

    #[test]
    fn standard_method_wins_when_few_actual_expenses() {
        // 15,000 biz miles standard = $10,500. Actual: $8k × 0.80 = $6,400.
        let r = compare_methods(MethodComparisonInput {
            business_miles: 15_000,
            actual_total_expenses: d(8_000),
            business_use_pct: dc("0.80"),
        });
        assert_eq!(r.standard_method_deduction, dc("10500"));
        assert_eq!(r.actual_method_deduction, dc("6400"));
        assert_eq!(r.better_method, "standard");
        assert_eq!(r.max_deduction, dc("10500"));
        assert_eq!(r.advantage, dc("4100"));
    }

    #[test]
    fn actual_method_wins_when_high_expenses_low_miles() {
        // 5,000 biz miles standard = $3,500. Actual $12k × 0.90 = $10,800.
        let r = compare_methods(MethodComparisonInput {
            business_miles: 5_000,
            actual_total_expenses: d(12_000),
            business_use_pct: dc("0.90"),
        });
        assert_eq!(r.standard_method_deduction, dc("3500"));
        assert_eq!(r.actual_method_deduction, dc("10800"));
        assert_eq!(r.better_method, "actual");
        assert_eq!(r.advantage, dc("7300"));
    }

    #[test]
    fn business_use_pct_clamped_to_zero_one() {
        // Bad input: 1.5 should clamp to 1.0.
        let r = compare_methods(MethodComparisonInput {
            business_miles: 1_000,
            actual_total_expenses: d(10_000),
            business_use_pct: dc("1.5"),
        });
        assert_eq!(r.actual_method_deduction, d(10_000));
    }

    #[test]
    fn negative_use_pct_treated_as_zero() {
        let r = compare_methods(MethodComparisonInput {
            business_miles: 1_000,
            actual_total_expenses: d(10_000),
            business_use_pct: dc("-0.5"),
        });
        assert_eq!(r.actual_method_deduction, Decimal::ZERO);
    }
}
