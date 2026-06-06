//! IRC §263A — Uniform Capitalization (UNICAP) rules.
//!
//! §263A(a)(1) requires certain costs to be **capitalized** into the
//! basis of property produced or acquired for resale, rather than
//! currently deducted. Direct costs (materials, labor) plus an
//! allocable share of indirect costs (overhead, facilities,
//! interest under §263A(f) for long-period construction) get
//! capitalized.
//!
//! The trader vs dealer distinction is **load-bearing** for active
//! traders:
//!
//!   * A **dealer in securities** under §475(c)(1) — a person who
//!     regularly purchases securities from or sells securities to
//!     customers in the ordinary course of trade or business — holds
//!     securities as INVENTORY. UNICAP applies: direct acquisition
//!     costs + indirect costs allocable to inventory must be
//!     capitalized into the basis of the inventory.
//!
//!   * A **trader in securities** (proprietary trading for own
//!     account; not buying/selling to customers) holds securities as
//!     INVESTMENT property. Under §263A(c)(3) + §475(f), trader
//!     positions are NOT inventory and §263A does NOT apply. Costs
//!     stay currently deductible as §162 ordinary business expenses.
//!
//! **§263A(b)(2)(B) small business taxpayer exception** under §448(c):
//! taxpayer is exempt from UNICAP entirely if average annual gross
//! receipts for the prior 3 years are at or below the §448(c)
//! threshold. Threshold table is shared with iter 16's `section_163j`:
//!
//!   * 2020 — $26M
//!   * 2021 — $26M
//!   * 2022 — $27M
//!   * 2023 — $29M
//!   * 2024 — $30M
//!   * 2025 — $31M
//!
//! Active securities dealers almost always blow past this — gross
//! receipts include gross proceeds from every sale — so the
//! exception rarely helps the average prop desk but day-1 startups
//! may briefly qualify.
//!
//! Pure compute. Caller asserts trader-vs-dealer classification +
//! supplies cost data + gross receipts; we determine UNICAP
//! applicability and split current-year deduction vs capitalized
//! amount.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradingClassification {
    /// §475(c)(1) dealer in securities: buys + sells to customers in
    /// ordinary course. Subject to UNICAP.
    Dealer,
    /// Trader in securities: proprietary trading for own account, no
    /// customers. NOT subject to UNICAP under §263A(c)(3) + §475(f).
    Trader,
    /// Investor (passive holder). Not engaged in trade or business;
    /// §263A doesn't apply in the dealer sense, but acquisition
    /// costs may still be capitalized into basis on a per-security
    /// basis (handled outside this module).
    Investor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section263AInput {
    pub tax_year: i32,
    pub classification: TradingClassification,
    /// Direct costs of acquiring inventory securities (broker
    /// commissions, transaction fees on dealer purchases).
    pub direct_costs: Decimal,
    /// Indirect costs allocable to inventory under §263A(a)(1)(B) +
    /// Reg. §1.263A-1(e) (facilities, support staff, IT systems
    /// share, depreciation on trading infrastructure).
    pub indirect_costs_allocable_to_inventory: Decimal,
    /// Average annual gross receipts for prior 3 tax years.
    pub avg_3yr_gross_receipts: Decimal,
    /// Override the §448(c) threshold. Defaults to the published
    /// table for the tax year.
    pub small_business_threshold_override: Option<Decimal>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section263AResult {
    pub subject_to_unicap: bool,
    pub small_business_threshold_applied: Decimal,
    pub small_business_exempt: bool,
    pub trader_exempt: bool,
    pub investor_exempt: bool,
    pub direct_costs_capitalized: Decimal,
    pub indirect_costs_capitalized: Decimal,
    pub total_costs_capitalized: Decimal,
    pub costs_currently_deductible: Decimal,
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
        _ => d("31000000"),
    }
}

pub fn compute(input: &Section263AInput) -> Section263AResult {
    let mut r = Section263AResult {
        small_business_threshold_applied: input
            .small_business_threshold_override
            .unwrap_or_else(|| small_business_threshold(input.tax_year)),
        ..Section263AResult::default()
    };

    let total_costs = input.direct_costs + input.indirect_costs_allocable_to_inventory;

    // Classification short-circuits.
    match input.classification {
        TradingClassification::Trader => {
            r.trader_exempt = true;
            r.subject_to_unicap = false;
            r.costs_currently_deductible = total_costs;
            r.note = "§475(f) trader: positions held as investment, not inventory — §263A does not apply. All costs currently deductible as §162 ordinary business expense.".into();
            return r;
        }
        TradingClassification::Investor => {
            r.investor_exempt = true;
            r.subject_to_unicap = false;
            r.costs_currently_deductible = Decimal::ZERO;
            r.direct_costs_capitalized = input.direct_costs; // basis capitalization
            r.total_costs_capitalized = input.direct_costs;
            r.note = "investor (not in trade or business): direct acquisition costs capitalized into security basis per §1012; indirect costs nondeductible (§212 investment expenses are §67(g) limited).".into();
            return r;
        }
        TradingClassification::Dealer => {}
    }

    // Dealer path: check §263A(b)(2)(B) small business exception.
    if input.avg_3yr_gross_receipts <= r.small_business_threshold_applied {
        r.small_business_exempt = true;
        r.subject_to_unicap = false;
        r.costs_currently_deductible = total_costs;
        r.note = format!(
            "§263A(b)(2)(B) small-business exception: avg gross receipts ${} ≤ ${} threshold ({}). All costs currently deductible.",
            input.avg_3yr_gross_receipts,
            r.small_business_threshold_applied,
            input.tax_year
        );
        return r;
    }

    // Subject to UNICAP — capitalize direct + indirect costs into
    // inventory basis.
    r.subject_to_unicap = true;
    r.direct_costs_capitalized = input.direct_costs.max(Decimal::ZERO);
    r.indirect_costs_capitalized = input
        .indirect_costs_allocable_to_inventory
        .max(Decimal::ZERO);
    r.total_costs_capitalized = r.direct_costs_capitalized + r.indirect_costs_capitalized;
    r.costs_currently_deductible = Decimal::ZERO;
    r.note = format!(
        "§263A applies: dealer with avg gross receipts ${} > ${} threshold. ${} direct + ${} indirect = ${} capitalized into inventory basis.",
        input.avg_3yr_gross_receipts,
        r.small_business_threshold_applied,
        r.direct_costs_capitalized,
        r.indirect_costs_capitalized,
        r.total_costs_capitalized,
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section263AInput {
        Section263AInput {
            tax_year: 2024,
            classification: TradingClassification::Dealer,
            direct_costs: dec!(100000),
            indirect_costs_allocable_to_inventory: dec!(50000),
            avg_3yr_gross_receipts: dec!(50000000), // way over threshold
            small_business_threshold_override: None,
        }
    }

    #[test]
    fn dealer_above_threshold_subject_to_unicap() {
        let r = compute(&base());
        assert!(r.subject_to_unicap);
        assert_eq!(r.total_costs_capitalized, dec!(150000));
        assert_eq!(r.costs_currently_deductible, Decimal::ZERO);
    }

    #[test]
    fn dealer_below_threshold_exempt_currently_deductible() {
        let mut i = base();
        i.avg_3yr_gross_receipts = dec!(10000000);
        let r = compute(&i);
        assert!(r.small_business_exempt);
        assert!(!r.subject_to_unicap);
        assert_eq!(r.costs_currently_deductible, dec!(150000));
        assert_eq!(r.total_costs_capitalized, Decimal::ZERO);
    }

    #[test]
    fn dealer_at_threshold_exact_still_exempt() {
        let mut i = base();
        i.avg_3yr_gross_receipts = dec!(30000000); // exactly threshold
        let r = compute(&i);
        assert!(r.small_business_exempt);
    }

    #[test]
    fn dealer_one_dollar_over_threshold_subject() {
        let mut i = base();
        i.avg_3yr_gross_receipts = dec!(30000001);
        let r = compute(&i);
        assert!(r.subject_to_unicap);
        assert!(!r.small_business_exempt);
    }

    #[test]
    fn trader_exempt_regardless_of_receipts() {
        let mut i = base();
        i.classification = TradingClassification::Trader;
        // Even with $100M gross receipts.
        i.avg_3yr_gross_receipts = dec!(100000000);
        let r = compute(&i);
        assert!(r.trader_exempt);
        assert!(!r.subject_to_unicap);
        assert_eq!(r.costs_currently_deductible, dec!(150000));
        assert!(r.note.contains("§475(f) trader"));
    }

    #[test]
    fn investor_costs_capitalized_to_basis_not_currently_deductible() {
        let mut i = base();
        i.classification = TradingClassification::Investor;
        let r = compute(&i);
        assert!(r.investor_exempt);
        assert!(!r.subject_to_unicap);
        // Direct costs capitalized to basis; indirect costs investor
        // expenses limited by §67(g) (not deductible 2018-2025 TCJA).
        assert_eq!(r.direct_costs_capitalized, dec!(100000));
        assert_eq!(r.costs_currently_deductible, Decimal::ZERO);
    }

    #[test]
    fn threshold_table_each_year_2020_through_2025() {
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
            i.avg_3yr_gross_receipts = expected;
            let r = compute(&i);
            assert_eq!(r.small_business_threshold_applied, expected, "year {year}");
            assert!(r.small_business_exempt);
        }
    }

    #[test]
    fn override_threshold_overrides_table() {
        let mut i = base();
        i.small_business_threshold_override = Some(dec!(40000000));
        i.avg_3yr_gross_receipts = dec!(35000000);
        let r = compute(&i);
        assert_eq!(r.small_business_threshold_applied, dec!(40000000));
        assert!(r.small_business_exempt);
    }

    #[test]
    fn zero_costs_dealer_subject_but_nothing_to_capitalize() {
        let mut i = base();
        i.direct_costs = Decimal::ZERO;
        i.indirect_costs_allocable_to_inventory = Decimal::ZERO;
        let r = compute(&i);
        assert!(r.subject_to_unicap);
        assert_eq!(r.total_costs_capitalized, Decimal::ZERO);
    }

    #[test]
    fn only_direct_costs_only_those_capitalized() {
        let mut i = base();
        i.indirect_costs_allocable_to_inventory = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.direct_costs_capitalized, dec!(100000));
        assert_eq!(r.indirect_costs_capitalized, Decimal::ZERO);
        assert_eq!(r.total_costs_capitalized, dec!(100000));
    }

    #[test]
    fn only_indirect_costs_only_those_capitalized() {
        let mut i = base();
        i.direct_costs = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.direct_costs_capitalized, Decimal::ZERO);
        assert_eq!(r.indirect_costs_capitalized, dec!(50000));
        assert_eq!(r.total_costs_capitalized, dec!(50000));
    }

    #[test]
    fn trader_note_distinguishes_from_dealer_path() {
        let mut t = base();
        t.classification = TradingClassification::Trader;
        let trader = compute(&t);
        assert!(trader.note.contains("§475(f) trader"));

        let dealer = compute(&base());
        assert!(dealer.note.contains("§263A applies"));
    }

    #[test]
    fn investor_note_describes_basis_capitalization_path() {
        let mut i = base();
        i.classification = TradingClassification::Investor;
        let r = compute(&i);
        assert!(r.note.contains("investor"));
        assert!(r.note.contains("§1012"));
    }

    #[test]
    fn small_business_exception_with_huge_costs_still_currently_deductible() {
        let mut i = base();
        i.avg_3yr_gross_receipts = dec!(5000000);
        i.direct_costs = dec!(2000000);
        i.indirect_costs_allocable_to_inventory = dec!(500000);
        let r = compute(&i);
        assert!(r.small_business_exempt);
        assert_eq!(r.costs_currently_deductible, dec!(2500000));
    }

    #[test]
    fn dealer_subject_path_total_capitalized_equals_sum_of_buckets() {
        let r = compute(&base());
        assert_eq!(
            r.total_costs_capitalized,
            r.direct_costs_capitalized + r.indirect_costs_capitalized
        );
    }

    #[test]
    fn trader_short_circuits_threshold_check() {
        // Trader classification runs FIRST; threshold check never fires.
        let mut i = base();
        i.classification = TradingClassification::Trader;
        i.avg_3yr_gross_receipts = dec!(100000000); // would fail threshold
        let r = compute(&i);
        assert!(r.trader_exempt);
        assert!(!r.small_business_exempt);
    }
}
