//! IRC §213 — Medical, dental, etc. expenses deduction.
//!
//! Universal for any itemizing taxpayer. Allows deduction of qualified
//! medical expenses to the extent they exceed **7.5% of Adjusted Gross
//! Income**. The 7.5% floor was made **permanent** by Section 103 of
//! the Taxpayer Certainty and Disaster Tax Relief Act of 2019 (enacted
//! as part of the Further Consolidated Appropriations Act of 2020,
//! P.L. 116-94), amending §213(f) — previously it had been scheduled
//! to revert to 10%.
//!
//! **Qualifying medical expenses (§213(d))**: amounts paid for the
//! diagnosis, cure, mitigation, treatment, or prevention of disease,
//! or for the purpose of affecting any structure or function of the
//! body. Includes:
//! - Doctor visits, dental, vision, mental-health care
//! - Prescription drugs (insulin treated as prescription)
//! - Hospital and clinic charges
//! - Long-term care services (chronic illness, ADL impairment)
//! - Transportation to medical care (mileage rate / actual cost)
//! - Lodging up to $50/night while away from home for medical care
//! - Health insurance premiums (subject to limits for self-employed
//!   via §162(l))
//! - **Long-term care insurance premiums** — capped per §213(d)(10)
//!   age-tiered limits indexed annually by IRS Rev. Proc.
//!
//! **HSA / FSA / HRA reimbursement double-deduction prevention**: any
//! amount paid or reimbursed under an HSA, FSA, Archer MSA, or HRA
//! CANNOT also be deducted under §213. The module subtracts these
//! reimbursements before applying the 7.5% AGI floor.
//!
//! **§213(d)(10) eligible long-term care premium age-tiered caps**
//! (2025 — IRS Rev. Proc. 2024-40):
//!
//! | Insured's attained age at year end | 2025 cap |
//! |------------------------------------|----------|
//! | ≤ 40                               | $480     |
//! | 41-50                              | $900     |
//! | 51-60                              | $1,800   |
//! | 61-70                              | $4,810   |
//! | 71+                                | $6,020   |
//!
//! For 2026 (IRS Rev. Proc. 2025-32): the 71+ cap rises to $6,200,
//! ≤40 to $500, 41-50 to $930, 51-60 to $1,860 (approximately 3%
//! across the board).
//!
//! Itemization required (Schedule A) — the standard deduction takes
//! precedence if larger. Module surfaces `requires_itemization: true`
//! always.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section213Input {
    pub tax_year: i32,
    pub adjusted_gross_income: Decimal,
    /// Qualified medical expenses OTHER than long-term-care premiums
    /// (doctors, prescriptions, hospital, transportation, lodging,
    /// health-insurance premiums, etc.).
    pub qualified_medical_expenses_other_than_ltc_premiums: Decimal,
    /// LTC insurance premiums paid; subject to §213(d)(10) age cap.
    pub ltc_premiums_paid: Decimal,
    /// Insured's attained age at year end for LTC cap lookup.
    pub insured_age_at_year_end: u32,
    /// Amounts reimbursed by HSA, FSA, Archer MSA, HRA, or insurance —
    /// not separately deductible.
    pub hsa_fsa_hra_reimbursements: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section213Result {
    pub agi_75pct_floor: Decimal,
    pub ltc_premium_cap: Decimal,
    pub ltc_premium_allowed: Decimal,
    pub ltc_premium_excess: Decimal,
    pub total_qualified_expenses_after_reimbursements: Decimal,
    pub deductible_medical_expense: Decimal,
    pub requires_itemization: bool,
    pub citation: String,
    pub note: String,
}

/// Look up the §213(d)(10) age-tiered LTC premium cap for a given
/// year and attained age. Falls back to 2025 caps for any year not
/// explicitly modeled.
fn ltc_cap_for_age(tax_year: i32, age: u32) -> i64 {
    // Confirmed 2026 caps per IRS Rev. Proc. 2025-32; mid-tier 61-70
    // not separately confirmed in this conversation — defaults to 2025
    // value rather than estimating.
    match (tax_year, age) {
        (2026, a) if a <= 40 => 500,
        (2026, a) if a <= 50 => 930,
        (2026, a) if a <= 60 => 1_860,
        (2026, a) if a <= 70 => 4_810, // 2025 fallback — 2026 not confirmed
        (2026, _) => 6_200,
        // 2025 (and default fallback for any other year).
        (_, a) if a <= 40 => 480,
        (_, a) if a <= 50 => 900,
        (_, a) if a <= 60 => 1_800,
        (_, a) if a <= 70 => 4_810,
        (_, _) => 6_020,
    }
}

pub fn compute(input: &Section213Input) -> Section213Result {
    let floor = input.adjusted_gross_income * Decimal::new(75, 3); // × 0.075
    let ltc_cap = Decimal::from(ltc_cap_for_age(
        input.tax_year,
        input.insured_age_at_year_end,
    ));
    let ltc_allowed = input.ltc_premiums_paid.min(ltc_cap);
    let ltc_excess = (input.ltc_premiums_paid - ltc_cap).max(Decimal::ZERO);

    let total_qualified_gross = input.qualified_medical_expenses_other_than_ltc_premiums
        + ltc_allowed;
    let total_after_reimb = (total_qualified_gross - input.hsa_fsa_hra_reimbursements)
        .max(Decimal::ZERO);
    let deductible = (total_after_reimb - floor).max(Decimal::ZERO);

    let note = format!(
        "§213 medical deduction: AGI ${} × 7.5% floor = ${}; qualified expenses ${} − HSA/FSA/HRA reimbursements ${} = ${}; deductible above floor = ${}. LTC premiums ${} capped at age-{} tier ${} ({}{}). Requires Schedule A itemization.",
        input.adjusted_gross_income.round_dp(2),
        floor.round_dp(2),
        total_qualified_gross.round_dp(2),
        input.hsa_fsa_hra_reimbursements.round_dp(2),
        total_after_reimb.round_dp(2),
        deductible.round_dp(2),
        input.ltc_premiums_paid.round_dp(2),
        input.insured_age_at_year_end,
        ltc_cap,
        input.tax_year,
        if ltc_excess > Decimal::ZERO {
            format!(", ${} excess premium NOT deductible", ltc_excess.round_dp(2))
        } else {
            String::new()
        },
    );

    Section213Result {
        agi_75pct_floor: floor,
        ltc_premium_cap: ltc_cap,
        ltc_premium_allowed: ltc_allowed,
        ltc_premium_excess: ltc_excess,
        total_qualified_expenses_after_reimbursements: total_after_reimb,
        deductible_medical_expense: deductible,
        requires_itemization: true,
        citation:
            "IRC §213(a) 7.5% AGI floor (CAA 2020 §103 made permanent); §213(d) qualified medical care definition; §213(d)(10) LTC premium age-tiered cap; IRS Rev. Proc. 2024-40 (2025) / Rev. Proc. 2025-32 (2026) annual indexed limits"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section213Input {
        Section213Input {
            tax_year: 2025,
            adjusted_gross_income: dec!(100_000),
            qualified_medical_expenses_other_than_ltc_premiums: dec!(15_000),
            ltc_premiums_paid: Decimal::ZERO,
            insured_age_at_year_end: 50,
            hsa_fsa_hra_reimbursements: Decimal::ZERO,
        }
    }

    #[test]
    fn standard_deduction_above_floor() {
        // AGI $100k × 7.5% = $7,500 floor. $15k expenses − $7,500 = $7,500 deductible.
        let r = compute(&base());
        assert_eq!(r.agi_75pct_floor, dec!(7_500));
        assert_eq!(r.deductible_medical_expense, dec!(7_500));
        assert!(r.requires_itemization);
    }

    #[test]
    fn expenses_below_floor_no_deduction() {
        let mut i = base();
        i.qualified_medical_expenses_other_than_ltc_premiums = dec!(5_000);
        let r = compute(&i);
        assert_eq!(r.deductible_medical_expense, Decimal::ZERO);
    }

    #[test]
    fn expenses_exactly_at_floor_no_deduction() {
        let mut i = base();
        i.qualified_medical_expenses_other_than_ltc_premiums = dec!(7_500);
        let r = compute(&i);
        assert_eq!(r.deductible_medical_expense, Decimal::ZERO);
    }

    #[test]
    fn one_dollar_above_floor_deductible() {
        let mut i = base();
        i.qualified_medical_expenses_other_than_ltc_premiums = dec!(7_501);
        let r = compute(&i);
        assert_eq!(r.deductible_medical_expense, dec!(1));
    }

    #[test]
    fn hsa_reimbursements_reduce_deduction() {
        // $15k expenses − $5k HSA reimbursement = $10k qualified; floor $7.5k; deduct $2.5k.
        let mut i = base();
        i.hsa_fsa_hra_reimbursements = dec!(5_000);
        let r = compute(&i);
        assert_eq!(r.total_qualified_expenses_after_reimbursements, dec!(10_000));
        assert_eq!(r.deductible_medical_expense, dec!(2_500));
    }

    #[test]
    fn reimbursements_exceed_expenses_clamp_to_zero() {
        let mut i = base();
        i.qualified_medical_expenses_other_than_ltc_premiums = dec!(5_000);
        i.hsa_fsa_hra_reimbursements = dec!(10_000);
        let r = compute(&i);
        assert_eq!(r.total_qualified_expenses_after_reimbursements, Decimal::ZERO);
        assert_eq!(r.deductible_medical_expense, Decimal::ZERO);
    }

    // LTC age-tier caps (2025).

    #[test]
    fn ltc_2025_age_40_or_less_480_cap() {
        let mut i = base();
        i.insured_age_at_year_end = 40;
        i.ltc_premiums_paid = dec!(1_000);
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(480));
        assert_eq!(r.ltc_premium_allowed, dec!(480));
        assert_eq!(r.ltc_premium_excess, dec!(520));
    }

    #[test]
    fn ltc_2025_age_50_900_cap() {
        let mut i = base();
        i.insured_age_at_year_end = 50;
        i.ltc_premiums_paid = dec!(1_500);
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(900));
        assert_eq!(r.ltc_premium_excess, dec!(600));
    }

    #[test]
    fn ltc_2025_age_60_1800_cap() {
        let mut i = base();
        i.insured_age_at_year_end = 60;
        i.ltc_premiums_paid = dec!(2_000);
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(1_800));
    }

    #[test]
    fn ltc_2025_age_70_4810_cap() {
        let mut i = base();
        i.insured_age_at_year_end = 70;
        i.ltc_premiums_paid = dec!(5_000);
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(4_810));
        assert_eq!(r.ltc_premium_excess, dec!(190));
    }

    #[test]
    fn ltc_2025_age_71_plus_6020_cap() {
        let mut i = base();
        i.insured_age_at_year_end = 75;
        i.ltc_premiums_paid = dec!(8_000);
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(6_020));
        assert_eq!(r.ltc_premium_excess, dec!(1_980));
    }

    #[test]
    fn ltc_age_boundary_40_in_first_tier() {
        let mut i = base();
        i.insured_age_at_year_end = 40;
        i.ltc_premiums_paid = dec!(1_000);
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(480));
    }

    #[test]
    fn ltc_age_boundary_41_in_second_tier() {
        let mut i = base();
        i.insured_age_at_year_end = 41;
        i.ltc_premiums_paid = dec!(1_000);
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(900));
    }

    // LTC 2026 caps.

    #[test]
    fn ltc_2026_age_40_500_cap() {
        let mut i = base();
        i.tax_year = 2026;
        i.insured_age_at_year_end = 40;
        i.ltc_premiums_paid = dec!(1_000);
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(500));
    }

    #[test]
    fn ltc_2026_age_50_930_cap() {
        let mut i = base();
        i.tax_year = 2026;
        i.insured_age_at_year_end = 50;
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(930));
    }

    #[test]
    fn ltc_2026_age_60_1860_cap() {
        let mut i = base();
        i.tax_year = 2026;
        i.insured_age_at_year_end = 60;
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(1_860));
    }

    #[test]
    fn ltc_2026_age_71_plus_6200_cap() {
        let mut i = base();
        i.tax_year = 2026;
        i.insured_age_at_year_end = 75;
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(6_200));
    }

    #[test]
    fn ltc_within_cap_full_allowed() {
        let mut i = base();
        i.insured_age_at_year_end = 50;
        i.ltc_premiums_paid = dec!(700);
        let r = compute(&i);
        assert_eq!(r.ltc_premium_allowed, dec!(700));
        assert_eq!(r.ltc_premium_excess, Decimal::ZERO);
    }

    #[test]
    fn ltc_premium_added_to_total_qualified() {
        // $15k other + $900 LTC capped = $15.9k qualified.
        let mut i = base();
        i.insured_age_at_year_end = 50;
        i.ltc_premiums_paid = dec!(900);
        let r = compute(&i);
        assert_eq!(r.total_qualified_expenses_after_reimbursements, dec!(15_900));
        assert_eq!(r.deductible_medical_expense, dec!(8_400));
    }

    // High-income / low-expense scenarios.

    #[test]
    fn high_agi_high_floor_no_deduction() {
        let mut i = base();
        i.adjusted_gross_income = dec!(1_000_000);
        let r = compute(&i);
        assert_eq!(r.agi_75pct_floor, dec!(75_000));
        assert_eq!(r.deductible_medical_expense, Decimal::ZERO);
    }

    #[test]
    fn high_agi_with_high_expenses_full_floor_subtraction() {
        let mut i = base();
        i.adjusted_gross_income = dec!(1_000_000);
        i.qualified_medical_expenses_other_than_ltc_premiums = dec!(200_000);
        let r = compute(&i);
        assert_eq!(r.deductible_medical_expense, dec!(125_000));
    }

    #[test]
    fn zero_agi_zero_floor_full_expenses_deductible() {
        let mut i = base();
        i.adjusted_gross_income = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.agi_75pct_floor, Decimal::ZERO);
        assert_eq!(r.deductible_medical_expense, dec!(15_000));
    }

    #[test]
    fn zero_expenses_zero_deduction() {
        let mut i = base();
        i.qualified_medical_expenses_other_than_ltc_premiums = Decimal::ZERO;
        i.ltc_premiums_paid = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.deductible_medical_expense, Decimal::ZERO);
    }

    #[test]
    fn unknown_year_falls_back_to_2025_caps() {
        let mut i = base();
        i.tax_year = 2030;
        i.insured_age_at_year_end = 75;
        i.ltc_premiums_paid = dec!(10_000);
        let r = compute(&i);
        assert_eq!(r.ltc_premium_cap, dec!(6_020));
    }

    #[test]
    fn note_describes_floor_and_deductible() {
        let r = compute(&base());
        assert!(r.note.contains("7.5% floor"));
        assert!(r.note.contains("Schedule A"));
    }

    #[test]
    fn note_describes_ltc_excess_when_capped() {
        let mut i = base();
        i.insured_age_at_year_end = 50;
        i.ltc_premiums_paid = dec!(2_000);
        let r = compute(&i);
        assert!(r.note.contains("excess premium NOT deductible"));
    }

    #[test]
    fn requires_itemization_always_true() {
        let r = compute(&base());
        assert!(r.requires_itemization);
    }

    #[test]
    fn citation_mentions_caa_2020_and_revproc() {
        let r = compute(&base());
        assert!(r.citation.contains("CAA 2020"));
        assert!(r.citation.contains("Rev. Proc."));
    }
}
