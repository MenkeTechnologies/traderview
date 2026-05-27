//! Meals & Entertainment 50% deduction calculator (IRC §274(n)).
//!
//! Business meals are 50% deductible (with documented business purpose +
//! present-with-client). Office snacks for employees are 50% (through
//! 2025). Office meals provided FOR EMPLOYER CONVENIENCE were temporarily
//! 100% under TCJA 2018-2025; revert to 50% starting 2026.
//!
//! Pure compute. Given a list of meal expenses with categories, returns
//! per-category totals + the deduction split (deductible / non-deductible).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MealCategory {
    /// Standard business meal with client present + documented purpose.
    /// 50% deductible (the default trader-with-mentor lunch case).
    BusinessClient,
    /// Office snacks provided to staff. 50% through 2025, reverts to 0%
    /// starting 2026 under the same TCJA sunset.
    OfficeSnacks,
    /// Meals provided for employer convenience (working dinner, on-site
    /// catering during late hours). 100% deductible 2018-2025 (TCJA
    /// temporary), 50% from 2026.
    EmployerConvenience,
    /// Pure entertainment — football tickets, concerts. 0% deductible
    /// since TCJA 2018. Pinned to surface the policy.
    Entertainment,
    /// Personal — not deductible at all. Caller marks splits like
    /// "I paid for myself + client" by entering the personal half here
    /// and the client half as BusinessClient.
    Personal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealExpense {
    pub date: NaiveDate,
    pub amount: Decimal,        // positive, in account currency
    pub category: MealCategory,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MealsReport {
    pub total_gross: Decimal,
    pub total_deductible: Decimal,
    pub total_non_deductible: Decimal,
    /// Per-category gross spend.
    pub by_category_gross: Vec<(MealCategory, Decimal)>,
    /// Per-category deductible amount.
    pub by_category_deductible: Vec<(MealCategory, Decimal)>,
}

/// Returns the deductible fraction (0.0..=1.0) for a category in the
/// given tax year. Encodes the TCJA sunset.
pub fn deductible_fraction(category: MealCategory, tax_year: i32) -> Decimal {
    let half = Decimal::from_str("0.5").unwrap();
    let one = Decimal::ONE;
    let zero = Decimal::ZERO;
    match category {
        MealCategory::BusinessClient => half,
        MealCategory::OfficeSnacks => {
            // 50% through 2025, 0% starting 2026.
            if tax_year <= 2025 { half } else { zero }
        }
        MealCategory::EmployerConvenience => {
            // 100% under TCJA 2018-2025, 50% from 2026.
            if tax_year <= 2025 { one } else { half }
        }
        MealCategory::Entertainment => zero,
        MealCategory::Personal      => zero,
    }
}

pub fn report(meals: &[MealExpense]) -> MealsReport {
    use std::collections::HashMap;
    let mut gross: HashMap<MealCategory, Decimal> = HashMap::new();
    let mut deductible: HashMap<MealCategory, Decimal> = HashMap::new();
    let mut total_gross = Decimal::ZERO;
    let mut total_deductible = Decimal::ZERO;
    for m in meals {
        let frac = deductible_fraction(m.category, m.date.format("%Y").to_string().parse().unwrap_or(2026));
        let ded = m.amount * frac;
        *gross.entry(m.category).or_insert(Decimal::ZERO) += m.amount;
        *deductible.entry(m.category).or_insert(Decimal::ZERO) += ded;
        total_gross += m.amount;
        total_deductible += ded;
    }
    let total_non_deductible = total_gross - total_deductible;
    let mut by_category_gross: Vec<_> = gross.into_iter().collect();
    let mut by_category_deductible: Vec<_> = deductible.into_iter().collect();
    by_category_gross.sort_by(|a, b| b.1.cmp(&a.1));
    by_category_deductible.sort_by(|a, b| b.1.cmp(&a.1));
    MealsReport {
        total_gross,
        total_deductible,
        total_non_deductible,
        by_category_gross,
        by_category_deductible,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }
    fn date(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn business_meal_is_always_50_percent() {
        for year in [2018, 2025, 2026, 2030] {
            assert_eq!(deductible_fraction(MealCategory::BusinessClient, year),
                d("0.5"), "BusinessClient must be 50% in {year}");
        }
    }

    #[test]
    fn entertainment_is_zero_post_tcja() {
        for year in [2018, 2024, 2026] {
            assert_eq!(deductible_fraction(MealCategory::Entertainment, year),
                Decimal::ZERO);
        }
    }

    #[test]
    fn personal_is_always_zero() {
        assert_eq!(deductible_fraction(MealCategory::Personal, 2024), Decimal::ZERO);
    }

    #[test]
    fn office_snacks_50_pct_through_2025_then_zero() {
        assert_eq!(deductible_fraction(MealCategory::OfficeSnacks, 2024), d("0.5"));
        assert_eq!(deductible_fraction(MealCategory::OfficeSnacks, 2025), d("0.5"));
        assert_eq!(deductible_fraction(MealCategory::OfficeSnacks, 2026), Decimal::ZERO,
            "TCJA sunset — office snacks lose deductibility in 2026");
    }

    #[test]
    fn employer_convenience_100_pct_through_2025_then_50() {
        assert_eq!(deductible_fraction(MealCategory::EmployerConvenience, 2024), Decimal::ONE);
        assert_eq!(deductible_fraction(MealCategory::EmployerConvenience, 2025), Decimal::ONE);
        assert_eq!(deductible_fraction(MealCategory::EmployerConvenience, 2026), d("0.5"),
            "TCJA sunset — employer-convenience drops from 100% to 50% in 2026");
    }

    #[test]
    fn report_aggregates_and_splits_correctly() {
        let meals = vec![
            MealExpense { date: date(2025, 6, 1), amount: d("100"),
                          category: MealCategory::BusinessClient, note: "".into() },
            MealExpense { date: date(2025, 6, 1), amount: d("50"),
                          category: MealCategory::OfficeSnacks, note: "".into() },
            MealExpense { date: date(2025, 6, 1), amount: d("200"),
                          category: MealCategory::EmployerConvenience, note: "".into() },
            MealExpense { date: date(2025, 6, 1), amount: d("80"),
                          category: MealCategory::Entertainment, note: "".into() },
        ];
        let r = report(&meals);
        // 100×0.5 + 50×0.5 + 200×1.0 + 80×0.0 = 50 + 25 + 200 + 0 = 275
        assert_eq!(r.total_gross, d("430"));
        assert_eq!(r.total_deductible, d("275.0"));
        assert_eq!(r.total_non_deductible, d("155.0"));
    }

    #[test]
    fn report_handles_empty_input() {
        let r = report(&[]);
        assert_eq!(r.total_gross, Decimal::ZERO);
        assert_eq!(r.total_deductible, Decimal::ZERO);
        assert!(r.by_category_gross.is_empty());
    }

    #[test]
    fn personal_share_of_split_bill_is_zero_deductible() {
        // "I paid $80, $40 of it was for me, $40 for the client mentor."
        let meals = vec![
            MealExpense { date: date(2025, 6, 1), amount: d("40"),
                          category: MealCategory::Personal, note: "my half".into() },
            MealExpense { date: date(2025, 6, 1), amount: d("40"),
                          category: MealCategory::BusinessClient, note: "client half".into() },
        ];
        let r = report(&meals);
        assert_eq!(r.total_gross, d("80"));
        // Only client half × 50% = $20.
        assert_eq!(r.total_deductible, d("20.0"));
    }

    #[test]
    fn report_categories_sort_by_gross_descending() {
        let meals = vec![
            MealExpense { date: date(2025, 1, 1), amount: d("10"),
                          category: MealCategory::Personal, note: "".into() },
            MealExpense { date: date(2025, 1, 1), amount: d("100"),
                          category: MealCategory::BusinessClient, note: "".into() },
            MealExpense { date: date(2025, 1, 1), amount: d("50"),
                          category: MealCategory::Entertainment, note: "".into() },
        ];
        let r = report(&meals);
        // First entry must be the biggest gross.
        assert_eq!(r.by_category_gross[0].0, MealCategory::BusinessClient);
        assert_eq!(r.by_category_gross[0].1, d("100"));
    }
}
