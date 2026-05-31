//! IRC §163(j) — Limitation on business interest expense.
//!
//! Active traders who elect §475(f) trader-in-securities status are
//! engaged in a "trade or business" — making their margin interest
//! **business interest expense**, subject to §163(j). The deduction
//! for the year is capped at:
//!
//!   limit = business_interest_income
//!         + floor_plan_financing_interest
//!         + 0.30 × adjusted_taxable_income
//!
//! Anything above the limit carries forward indefinitely under
//! §163(j)(2). The carryforward is treated as paid-or-accrued in the
//! succeeding year, so big interest years stack until ATI catches up.
//!
//! **Adjusted Taxable Income** (§163(j)(8)) is taxable income computed
//! WITHOUT regard to:
//!   * Any item not properly allocable to a trade or business.
//!   * Business interest expense or income.
//!   * Net operating loss deduction.
//!   * §199A qualified business income deduction.
//!   * For tax years beginning before 2022 only: depreciation, amortization,
//!     depletion. (After 2021, these are NOT added back — the TCJA
//!     transition makes the cap meaningfully tighter post-2021.)
//!
//! **Small business taxpayer exception** (§163(j)(3)): the cap does
//! NOT apply when the taxpayer's average annual gross receipts for
//! the prior 3 years are at or below the §448(c) threshold. The
//! threshold is annually indexed for inflation:
//!
//!   * 2020 — $26M
//!   * 2021 — $26M
//!   * 2022 — $27M
//!   * 2023 — $29M
//!   * 2024 — $30M
//!   * 2025 — $31M (caller can pass override)
//!
//! Note: traders almost always blow past this — gross receipts =
//! gross proceeds from all sales — so the exception rarely helps
//! active traders. But day-1 traders may briefly qualify.
//!
//! Pure compute. Caller supplies the dollar amounts; we compute the
//! cap and split current-year deduction vs carryforward.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section163jInput {
    pub tax_year: i32,
    /// Total business interest expense this year (margin interest +
    /// any other interest properly allocable to the trade or business).
    pub business_interest_expense: Decimal,
    /// Interest income from the trade or business (e.g. T-bill interest
    /// held as working capital for the trading account).
    pub business_interest_income: Decimal,
    /// Floor plan financing interest — rare for traders, zero usually.
    pub floor_plan_financing_interest: Decimal,
    /// Adjusted Taxable Income per §163(j)(8). Caller computes this
    /// (taxable income before interest, NOL, §199A — see module docs).
    pub adjusted_taxable_income: Decimal,
    /// Indefinite carryforward of disallowed business interest from
    /// prior years under §163(j)(2). Added to current-year expense.
    pub prior_year_carryforward: Decimal,
    /// Average annual gross receipts for prior 3 years. Triggers the
    /// §163(j)(3) small-business exception when ≤ threshold for the year.
    pub avg_3yr_gross_receipts: Decimal,
    /// Override the §448(c) threshold (in case the caller has a more
    /// current published figure than the table embedded here).
    pub small_business_threshold_override: Option<Decimal>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section163jResult {
    pub small_business_exempt: bool,
    pub small_business_threshold_applied: Decimal,
    pub adjusted_taxable_income: Decimal,
    /// 30% × ATI (floor at zero).
    pub thirty_pct_of_ati: Decimal,
    /// Combined cap = 30% × ATI + business interest income + floor plan.
    pub deduction_limit: Decimal,
    /// Total expense to apply this year = current + prior carryforward.
    pub total_expense_available: Decimal,
    pub deductible_this_year: Decimal,
    pub carryforward_to_next_year: Decimal,
    pub note: String,
}

fn small_business_threshold(year: i32) -> Decimal {
    let d = |s: &str| Decimal::from_str(s).unwrap();
    match year {
        ..=2020 => d("26000000"),
        2021 => d("26000000"),
        2022 => d("27000000"),
        2023 => d("29000000"),
        2024 => d("30000000"),
        2025 => d("31000000"),
        _ => d("31000000"), // caller can override for 2026+
    }
}

fn thirty_pct() -> Decimal {
    Decimal::from_str("0.30").unwrap()
}

pub fn compute(input: &Section163jInput) -> Section163jResult {
    let mut r = Section163jResult {
        adjusted_taxable_income: input.adjusted_taxable_income,
        small_business_threshold_applied: input
            .small_business_threshold_override
            .unwrap_or_else(|| small_business_threshold(input.tax_year)),
        ..Section163jResult::default()
    };

    r.total_expense_available = input.business_interest_expense + input.prior_year_carryforward;

    if r.total_expense_available <= Decimal::ZERO {
        r.note = "no business interest to limit".into();
        return r;
    }

    // §163(j)(3) small-business exception: avg gross receipts ≤ §448(c)
    // threshold means §163(j) doesn't apply at all.
    if input.avg_3yr_gross_receipts <= r.small_business_threshold_applied {
        r.small_business_exempt = true;
        r.deductible_this_year = r.total_expense_available;
        r.note = format!(
            "§163(j)(3) small-business exception: avg gross receipts ${} ≤ threshold ${} ({})",
            input.avg_3yr_gross_receipts, r.small_business_threshold_applied, input.tax_year
        );
        return r;
    }

    // Standard path: 30% × ATI + business interest income + floor plan.
    r.thirty_pct_of_ati = (input.adjusted_taxable_income.max(Decimal::ZERO) * thirty_pct())
        .round_dp(2);
    r.deduction_limit = r.thirty_pct_of_ati
        + input.business_interest_income
        + input.floor_plan_financing_interest;

    r.deductible_this_year = r.total_expense_available.min(r.deduction_limit);
    r.carryforward_to_next_year = (r.total_expense_available - r.deductible_this_year)
        .max(Decimal::ZERO);

    r.note = if r.carryforward_to_next_year > Decimal::ZERO {
        format!(
            "§163(j): ${} deducted (cap ${}), ${} carries to next year",
            r.deductible_this_year, r.deduction_limit, r.carryforward_to_next_year
        )
    } else {
        format!(
            "§163(j): full ${} deducted within ${} cap",
            r.deductible_this_year, r.deduction_limit
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section163jInput {
        // $50k margin interest, $100k ATI, no carryforward, no other
        // business interest income, big-trader (no SB exemption).
        Section163jInput {
            tax_year: 2024,
            business_interest_expense: dec!(50000),
            business_interest_income: Decimal::ZERO,
            floor_plan_financing_interest: Decimal::ZERO,
            adjusted_taxable_income: dec!(100000),
            prior_year_carryforward: Decimal::ZERO,
            avg_3yr_gross_receipts: dec!(50000000), // way over threshold
            small_business_threshold_override: None,
        }
    }

    #[test]
    fn standard_30pct_cap_partial_deduction() {
        // 30% × $100k = $30k cap. $50k expense → $30k deducted, $20k carries.
        let r = compute(&base());
        assert_eq!(r.thirty_pct_of_ati, dec!(30000));
        assert_eq!(r.deduction_limit, dec!(30000));
        assert_eq!(r.deductible_this_year, dec!(30000));
        assert_eq!(r.carryforward_to_next_year, dec!(20000));
    }

    #[test]
    fn expense_below_cap_fully_deducted() {
        let mut i = base();
        i.business_interest_expense = dec!(20000);
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, dec!(20000));
        assert_eq!(r.carryforward_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn business_interest_income_raises_cap_dollar_for_dollar() {
        // ATI $100k → 30% cap = $30k. Plus $10k T-bill interest income.
        // Effective cap = $40k. $50k expense → $40k deductible, $10k carry.
        let mut i = base();
        i.business_interest_income = dec!(10000);
        let r = compute(&i);
        assert_eq!(r.deduction_limit, dec!(40000));
        assert_eq!(r.deductible_this_year, dec!(40000));
        assert_eq!(r.carryforward_to_next_year, dec!(10000));
    }

    #[test]
    fn prior_carryforward_stacks_with_current_year() {
        // $50k current + $20k prior = $70k available. $30k cap → $40k carry.
        let mut i = base();
        i.prior_year_carryforward = dec!(20000);
        let r = compute(&i);
        assert_eq!(r.total_expense_available, dec!(70000));
        assert_eq!(r.deductible_this_year, dec!(30000));
        assert_eq!(r.carryforward_to_next_year, dec!(40000));
    }

    #[test]
    fn small_business_under_threshold_fully_exempt() {
        // Avg gross receipts $10M (under $30M 2024) → exempt, full deduction.
        let mut i = base();
        i.avg_3yr_gross_receipts = dec!(10000000);
        let r = compute(&i);
        assert!(r.small_business_exempt);
        assert_eq!(r.deductible_this_year, dec!(50000));
        assert_eq!(r.carryforward_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn small_business_at_threshold_exact_still_exempt() {
        // §448(c) is ≤, not <. At threshold is still exempt.
        let mut i = base();
        i.avg_3yr_gross_receipts = dec!(30000000);
        let r = compute(&i);
        assert!(r.small_business_exempt);
    }

    #[test]
    fn small_business_one_dollar_over_threshold_loses_exemption() {
        let mut i = base();
        i.avg_3yr_gross_receipts = dec!(30000001);
        let r = compute(&i);
        assert!(!r.small_business_exempt);
        assert_eq!(r.deductible_this_year, dec!(30000));
    }

    #[test]
    fn threshold_table_2020_through_2025() {
        for (year, expected) in [
            (2020, dec!(26000000)),
            (2021, dec!(26000000)),
            (2022, dec!(27000000)),
            (2023, dec!(29000000)),
            (2024, dec!(30000000)),
            (2025, dec!(31000000)),
        ] {
            let mut i = base();
            i.tax_year = year;
            // Push gross receipts to exactly threshold to verify lookup.
            i.avg_3yr_gross_receipts = expected;
            let r = compute(&i);
            assert_eq!(r.small_business_threshold_applied, expected, "{year}");
            assert!(r.small_business_exempt);
        }
    }

    #[test]
    fn caller_threshold_override_overrides_table() {
        let mut i = base();
        i.small_business_threshold_override = Some(dec!(40000000));
        i.avg_3yr_gross_receipts = dec!(35000000);
        let r = compute(&i);
        assert_eq!(r.small_business_threshold_applied, dec!(40000000));
        assert!(r.small_business_exempt);
    }

    #[test]
    fn negative_ati_caps_30pct_at_zero() {
        // Loss year: ATI negative. 30% × max(0, ATI) = 0. Only BI income
        // + floor plan in the cap.
        let mut i = base();
        i.adjusted_taxable_income = dec!(-50000);
        i.business_interest_income = dec!(5000);
        let r = compute(&i);
        assert_eq!(r.thirty_pct_of_ati, Decimal::ZERO);
        assert_eq!(r.deduction_limit, dec!(5000));
        assert_eq!(r.deductible_this_year, dec!(5000));
        assert_eq!(r.carryforward_to_next_year, dec!(45000));
    }

    #[test]
    fn no_expense_no_op() {
        let mut i = base();
        i.business_interest_expense = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, Decimal::ZERO);
        assert!(r.note.contains("no business interest"));
    }

    #[test]
    fn floor_plan_financing_adds_to_cap() {
        // Auto dealer trader (rare but real) — floor plan interest is
        // always fully deductible per §163(j)(1)(C).
        let mut i = base();
        i.floor_plan_financing_interest = dec!(15000);
        let r = compute(&i);
        // Cap = $30k 30%-ATI + $0 BI income + $15k floor plan = $45k.
        assert_eq!(r.deduction_limit, dec!(45000));
        assert_eq!(r.deductible_this_year, dec!(45000));
        assert_eq!(r.carryforward_to_next_year, dec!(5000));
    }

    #[test]
    fn multi_year_chain_carryforward_eventually_absorbs() {
        // Year 1: $50k expense, $30k cap, $20k carry.
        // Year 2: $0 new expense, ATI rises to $150k → $45k cap.
        //   Available = $20k carry. Deduct $20k, carry zero.
        let y1 = compute(&base());
        assert_eq!(y1.carryforward_to_next_year, dec!(20000));

        let y2 = compute(&Section163jInput {
            tax_year: 2025,
            business_interest_expense: Decimal::ZERO,
            business_interest_income: Decimal::ZERO,
            floor_plan_financing_interest: Decimal::ZERO,
            adjusted_taxable_income: dec!(150000),
            prior_year_carryforward: y1.carryforward_to_next_year,
            avg_3yr_gross_receipts: dec!(50000000),
            small_business_threshold_override: None,
        });
        assert_eq!(y2.deductible_this_year, dec!(20000));
        assert_eq!(y2.carryforward_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn full_deduction_note_when_under_cap() {
        let mut i = base();
        i.business_interest_expense = dec!(20000);
        let r = compute(&i);
        assert!(r.note.contains("full"));
    }

    #[test]
    fn carryforward_note_when_capped() {
        let r = compute(&base());
        assert!(r.note.contains("carries"));
    }
}
