//! IRC §174 — Research and experimental (R&E) expenditure
//! capitalization, post-TCJA amendment.
//!
//! The Tax Cuts and Jobs Act amended §174 effective tax years
//! beginning after December 31, 2021. Before TCJA, R&E expenditures
//! could be either **expensed currently** OR **capitalized + amortized
//! over 5+ years** at taxpayer's election. After TCJA, current
//! expensing is GONE — all R&E expenditures must be capitalized
//! and amortized over:
//!
//!   * **Domestic R&E** — 5 years (60 months) per §174(a)(2)(A).
//!   * **Foreign R&E** — 15 years (180 months) per §174(a)(2)(B).
//!
//! **Half-year convention** per §174(a)(2): amortization begins at
//! the midpoint of the tax year of expenditure. Year 1 gets half a
//! year (1/10 of 5-year domestic, or 1/30 of 15-year foreign); the
//! final stub year picks up the other half. So a $100k domestic R&E
//! in 2024:
//!
//!   * 2024 (year 1, half): $10,000
//!   * 2025-2028 (years 2-5, full): $20,000 each
//!   * 2029 (year 6, stub half): $10,000
//!
//! For 5-year domestic, total amortization touches 6 calendar years.
//! For 15-year foreign, total touches 16 calendar years.
//!
//! **§174 covers software development** per Rev. Proc. 2000-50 +
//! TCJA committee report. This is THE rule that hit algorithmic
//! traders writing internal trading software — previously expensed
//! immediately under §174(a)(1); now must capitalize the full cost
//! and recover over 5 years.
//!
//! Excluded from §174 (still currently deductible under §162):
//! routine business operations, market research, advertising, sales
//! promotion, ordinary testing of prototypes. The line between R&E
//! and §162 ordinary expense is fact-intensive; caller responsible
//! for the classification.
//!
//! Pure compute. Caller passes the R&E amount + expenditure year +
//! domestic/foreign + current tax year; we compute the annual
//! amortization deduction + the cumulative-to-date + remaining
//! capitalized basis.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RDLocation {
    /// §174(a)(2)(A) — 5-year amortization.
    Domestic,
    /// §174(a)(2)(B) — 15-year amortization for R&E performed outside US.
    Foreign,
}

impl RDLocation {
    pub fn amortization_years(self) -> u32 {
        match self {
            RDLocation::Domestic => 5,
            RDLocation::Foreign => 15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section174Input {
    /// Total R&E expenditure dollars in the year incurred.
    pub r_and_d_amount: Decimal,
    pub expenditure_year: i32,
    pub location: RDLocation,
    pub current_tax_year: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section174Result {
    pub amortization_years: u32,
    pub year_of_life: u32,
    pub amortization_for_current_year: Decimal,
    pub cumulative_amortization_through_current_year: Decimal,
    pub remaining_capitalized_basis: Decimal,
    pub fully_amortized: bool,
    /// Per-year schedule across the full recovery period (years 1
    /// through life+1) so callers can show the user the multi-year
    /// deduction curve.
    pub annual_schedule: Vec<AnnualAmortization>,
    pub pre_tcja_expensing_available: bool,
    pub note: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnnualAmortization {
    pub tax_year: i32,
    pub amount: Decimal,
}

fn one_half() -> Decimal {
    Decimal::from_str("0.5").unwrap()
}

pub fn compute(input: &Section174Input) -> Section174Result {
    let mut r = Section174Result {
        amortization_years: input.location.amortization_years(),
        ..Section174Result::default()
    };

    // Pre-TCJA expensing path: §174(a)(1) before 2022 allowed current
    // deduction. We compute the schedule but flag that the taxpayer
    // could have expensed it instead.
    r.pre_tcja_expensing_available = input.expenditure_year < 2022;

    if input.r_and_d_amount <= Decimal::ZERO {
        r.note = "no R&D expenditure".into();
        return r;
    }

    let years = r.amortization_years;
    let annual_full = (input.r_and_d_amount / Decimal::from(years)).round_dp(2);
    let annual_half = (annual_full * one_half()).round_dp(2);

    // Build full schedule: years 1 through life+1 (extra stub year at
    // the end picks up the year-1 half).
    for n in 0..=(years as i32) {
        let year = input.expenditure_year + n;
        let amount = if n == 0 || n == years as i32 {
            annual_half
        } else {
            annual_full
        };
        r.annual_schedule.push(AnnualAmortization {
            tax_year: year,
            amount,
        });
    }

    // Compute current-year amount + cumulative.
    let mut current_year_amt = Decimal::ZERO;
    let mut cumulative = Decimal::ZERO;
    for entry in &r.annual_schedule {
        if entry.tax_year < input.current_tax_year {
            cumulative += entry.amount;
        } else if entry.tax_year == input.current_tax_year {
            current_year_amt = entry.amount;
            cumulative += entry.amount;
        }
    }
    r.amortization_for_current_year = current_year_amt;
    r.cumulative_amortization_through_current_year = cumulative;
    r.year_of_life = (input.current_tax_year - input.expenditure_year + 1).max(0) as u32;
    r.remaining_capitalized_basis = (input.r_and_d_amount - cumulative).max(Decimal::ZERO);
    r.fully_amortized = r.remaining_capitalized_basis == Decimal::ZERO
        || input.current_tax_year > input.expenditure_year + years as i32;

    r.note = if r.pre_tcja_expensing_available {
        format!(
            "§174 pre-TCJA: ${} R&D could have been expensed in {} OR amortized over {} years. Post-2022 amendment forces capitalization for new expenditures.",
            input.r_and_d_amount, input.expenditure_year, years
        )
    } else {
        format!(
            "§174 post-TCJA: ${} R&D capitalized over {} years ({:?}). Current year {} amortization ${} ({}/{} year of life); ${} remaining basis.",
            input.r_and_d_amount,
            years,
            input.location,
            input.current_tax_year,
            r.amortization_for_current_year,
            r.year_of_life,
            years + 1,
            r.remaining_capitalized_basis,
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section174Input {
        Section174Input {
            r_and_d_amount: dec!(100000),
            expenditure_year: 2024,
            location: RDLocation::Domestic,
            current_tax_year: 2024,
        }
    }

    #[test]
    fn domestic_5_year_year_1_half_year_convention() {
        // $100k / 5 = $20k full year; year 1 half = $10k.
        let r = compute(&base());
        assert_eq!(r.amortization_years, 5);
        assert_eq!(r.amortization_for_current_year, dec!(10000));
        assert_eq!(r.cumulative_amortization_through_current_year, dec!(10000));
        assert_eq!(r.remaining_capitalized_basis, dec!(90000));
        assert_eq!(r.year_of_life, 1);
    }

    #[test]
    fn domestic_5_year_year_2_full_year() {
        let mut i = base();
        i.current_tax_year = 2025;
        let r = compute(&i);
        assert_eq!(r.amortization_for_current_year, dec!(20000));
        // Cumulative = year 1 ($10k) + year 2 ($20k) = $30k.
        assert_eq!(r.cumulative_amortization_through_current_year, dec!(30000));
        assert_eq!(r.remaining_capitalized_basis, dec!(70000));
    }

    #[test]
    fn domestic_5_year_final_stub_year_6() {
        // Year 6 = stub half = $10k.
        let mut i = base();
        i.current_tax_year = 2029;
        let r = compute(&i);
        assert_eq!(r.amortization_for_current_year, dec!(10000));
        assert_eq!(r.cumulative_amortization_through_current_year, dec!(100000));
        assert_eq!(r.remaining_capitalized_basis, Decimal::ZERO);
        assert!(r.fully_amortized);
    }

    #[test]
    fn domestic_year_7_post_recovery_no_deduction() {
        let mut i = base();
        i.current_tax_year = 2030;
        let r = compute(&i);
        assert_eq!(r.amortization_for_current_year, Decimal::ZERO);
        assert!(r.fully_amortized);
    }

    #[test]
    fn domestic_schedule_has_6_entries_for_5_year_recovery() {
        let r = compute(&base());
        assert_eq!(r.annual_schedule.len(), 6);
        // Schedule years 2024 through 2029.
        assert_eq!(r.annual_schedule[0].tax_year, 2024);
        assert_eq!(r.annual_schedule[5].tax_year, 2029);
    }

    #[test]
    fn domestic_schedule_amounts_correctly_distributed() {
        let r = compute(&base());
        assert_eq!(r.annual_schedule[0].amount, dec!(10000)); // year 1 half
        assert_eq!(r.annual_schedule[1].amount, dec!(20000)); // year 2 full
        assert_eq!(r.annual_schedule[2].amount, dec!(20000)); // year 3 full
        assert_eq!(r.annual_schedule[3].amount, dec!(20000)); // year 4 full
        assert_eq!(r.annual_schedule[4].amount, dec!(20000)); // year 5 full
        assert_eq!(r.annual_schedule[5].amount, dec!(10000)); // year 6 stub half
                                                              // Sum = $100k total.
        let sum: Decimal = r.annual_schedule.iter().map(|x| x.amount).sum();
        assert_eq!(sum, dec!(100000));
    }

    #[test]
    fn foreign_15_year_year_1_half_year_convention() {
        // $100k / 15 = $6,666.67 full; year 1 half = $3,333.34 (rounded).
        let mut i = base();
        i.location = RDLocation::Foreign;
        let r = compute(&i);
        assert_eq!(r.amortization_years, 15);
        // (100000 / 15).round_dp(2) = 6666.67; * 0.5 = 3333.335 → 3333.34
        assert_eq!(r.amortization_for_current_year, dec!(3333.34));
    }

    #[test]
    fn foreign_15_year_schedule_has_16_entries() {
        let mut i = base();
        i.location = RDLocation::Foreign;
        let r = compute(&i);
        assert_eq!(r.annual_schedule.len(), 16);
    }

    #[test]
    fn pre_2022_expenditure_flags_expensing_available() {
        let mut i = base();
        i.expenditure_year = 2021;
        i.current_tax_year = 2021;
        let r = compute(&i);
        assert!(r.pre_tcja_expensing_available);
        assert!(r.note.contains("could have been expensed"));
    }

    #[test]
    fn post_2022_no_expensing_option() {
        let r = compute(&base());
        assert!(!r.pre_tcja_expensing_available);
        assert!(r.note.contains("post-TCJA"));
    }

    #[test]
    fn zero_amount_no_op() {
        let mut i = base();
        i.r_and_d_amount = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.amortization_for_current_year, Decimal::ZERO);
        assert!(r.note.contains("no R&D"));
    }

    #[test]
    fn before_expenditure_year_zero_deduction() {
        let mut i = base();
        i.current_tax_year = 2023; // before 2024 expenditure
        let r = compute(&i);
        assert_eq!(r.amortization_for_current_year, Decimal::ZERO);
        assert_eq!(
            r.cumulative_amortization_through_current_year,
            Decimal::ZERO
        );
    }

    #[test]
    fn domestic_cumulative_grows_predictably_across_recovery_period() {
        let expected = [
            (2024, dec!(10000)),
            (2025, dec!(30000)),
            (2026, dec!(50000)),
            (2027, dec!(70000)),
            (2028, dec!(90000)),
            (2029, dec!(100000)),
        ];
        for (year, cum) in expected {
            let mut i = base();
            i.current_tax_year = year;
            let r = compute(&i);
            assert_eq!(
                r.cumulative_amortization_through_current_year, cum,
                "year {year}: expected ${cum} cumulative, got ${}",
                r.cumulative_amortization_through_current_year
            );
        }
    }

    #[test]
    fn rd_location_helper_returns_correct_years() {
        assert_eq!(RDLocation::Domestic.amortization_years(), 5);
        assert_eq!(RDLocation::Foreign.amortization_years(), 15);
    }

    #[test]
    fn algorithmic_trader_software_dev_100k_year_1_only_10k_deductible() {
        // The TCJA hit: algorithmic trader spends $100k on internal
        // software dev in 2024. Pre-TCJA could have expensed the full
        // $100k. Post-TCJA: $10k deductible year 1.
        let r = compute(&base());
        assert_eq!(r.amortization_for_current_year, dec!(10000));
        // Cash impact: $90k of basis sits on the balance sheet, deducted
        // over the next 5 calendar years.
    }

    #[test]
    fn five_year_recovery_sum_ties_to_full_amount() {
        // Sanity invariant: schedule total ties to original amount.
        let mut i = base();
        i.r_and_d_amount = dec!(250000); // try a non-round amount
        let r = compute(&i);
        let sum: Decimal = r.annual_schedule.iter().map(|x| x.amount).sum();
        assert_eq!(sum, dec!(250000));
    }
}
