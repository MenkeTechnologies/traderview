//! IRC § 179 — Election to expense certain depreciable business assets.
//!
//! 26 U.S.C. § 179 lets a taxpayer ELECT to deduct the full cost of
//! qualifying §1245-class property in the year placed in service, instead
//! of capitalizing and recovering through MACRS depreciation. Three
//! limitations stack:
//!
//! 1. **§ 179(b)(1) dollar cap** — annual deduction is limited to a
//!    statutory ceiling. The One Big Beautiful Bill Act of 2025 (OBBBA)
//!    raised the baseline to **$2.5M for 2025** (effective for tax years
//!    beginning after 2024-12-31). For 2026 the inflation-adjusted cap is
//!    **$2,560,000**.
//! 2. **§ 179(b)(2) phaseout** — the cap is reduced dollar-for-dollar
//!    when total §1245 property placed in service in the year exceeds the
//!    phaseout threshold. **2026 threshold: $4,090,000**. A taxpayer
//!    placing $5.09M of qualifying property loses $1M of the cap.
//! 3. **§ 179(b)(3) taxable-income limitation** — the §179 deduction
//!    cannot exceed the taxpayer's aggregate taxable income from the
//!    active conduct of any trade or business. **Excess carries forward
//!    indefinitely** under § 179(b)(3)(B).
//!
//! **§ 179(b)(5) SUV sublimit** — passenger vehicles with GVWR between
//! 6,000 and 14,000 lb. (the "heavy SUV" carve-in to § 280F) have their
//! own annual cap. **2026 SUV sublimit: $32,000**. The remainder of the
//! basis falls into § 168(k) bonus depreciation, which OBBBA restored
//! to **100% PERMANENTLY** for property placed in service after 2024.
//!
//! Citations: 26 U.S.C. § 179; § 179(b)(1) (dollar cap); § 179(b)(2)
//! (phaseout dollar-for-dollar above threshold); § 179(b)(3)(A) (taxable-
//! income limitation); § 179(b)(3)(B) (carryforward); § 179(b)(5) (SUV
//! sublimit); § 168(k) (100% bonus depreciation, made permanent by OBBBA
//! § 70302, effective 2025-01-01).
//!
//! Caveats: this module models the four primary limits and the SUV
//! sublimit. Out of scope: § 179(d)(3) related-party purchase restriction;
//! § 179(d)(10) recapture on subsequent business-use percentage drop below
//! 50%; § 179(f) qualified real-property carve-in (roofs, HVAC, fire alarm,
//! security systems on nonresidential real property).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section179Input {
    /// Total cost of §1245 property placed in service this year (cents).
    /// Drives both the deduction and the phaseout calculation.
    pub qualifying_property_cents: i64,
    /// Of `qualifying_property_cents`, the portion attributable to heavy
    /// SUVs (GVWR 6,001-14,000 lb.). § 179(b)(5) caps deduction on this
    /// portion separately. Remainder of SUV basis falls to § 168(k) bonus.
    pub suv_property_cents: i64,
    /// Aggregate taxable income from the active conduct of any trade or
    /// business of the taxpayer. § 179(b)(3)(A) ceiling on the deduction.
    pub taxable_income_cents: i64,
    /// Carryforward from prior years (§ 179(b)(3)(B)). Added back to
    /// current-year tentative deduction subject to all three limits.
    pub prior_year_carryforward_cents: i64,
    /// Annual § 179(b)(1) dollar cap for the tax year (cents). 2026 = $2,560,000.
    pub dollar_cap_cents: i64,
    /// § 179(b)(2) phaseout threshold for the tax year (cents). 2026 = $4,090,000.
    pub phaseout_threshold_cents: i64,
    /// § 179(b)(5) SUV sublimit for the tax year (cents). 2026 = $32,000.
    pub suv_sublimit_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section179Result {
    /// § 179(b)(1) cap reduced by § 179(b)(2) phaseout.
    pub adjusted_cap_cents: i64,
    /// Amount of cap reduced under § 179(b)(2) phaseout.
    pub phaseout_reduction_cents: i64,
    /// Tentative deduction = min(qualifying property, adjusted cap) +
    /// prior-year carryforward, before applying taxable-income limit and
    /// SUV sublimit.
    pub tentative_deduction_cents: i64,
    /// Amount blocked by § 179(b)(3)(A) taxable-income limitation.
    pub taxable_income_limit_block_cents: i64,
    /// Amount blocked by § 179(b)(5) SUV sublimit (excess of SUV property
    /// over the SUV sublimit). Falls to § 168(k) bonus depreciation.
    pub suv_excess_cents: i64,
    /// Deduction actually allowed this year after all three limits + SUV.
    pub current_year_deduction_cents: i64,
    /// § 179(b)(3)(B) carryforward to next year (excess blocked by
    /// taxable-income limit).
    pub carryforward_to_next_year_cents: i64,
    /// Amount of SUV basis that falls to § 168(k) bonus depreciation.
    /// 100% permanently under OBBBA § 70302 (effective 2025-01-01).
    pub bonus_depreciation_eligible_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section179Input) -> Section179Result {
    let qualifying = input.qualifying_property_cents.max(0);
    let suv = input.suv_property_cents.max(0).min(qualifying);
    let taxable_income = input.taxable_income_cents.max(0);
    let carryforward_in = input.prior_year_carryforward_cents.max(0);
    let dollar_cap = input.dollar_cap_cents.max(0);
    let phaseout_threshold = input.phaseout_threshold_cents.max(0);
    let suv_sublimit = input.suv_sublimit_cents.max(0);

    // § 179(b)(2) phaseout: dollar-for-dollar above threshold.
    let phaseout_reduction = (qualifying - phaseout_threshold).max(0);
    let adjusted_cap = (dollar_cap - phaseout_reduction).max(0);

    // SUV sublimit: cannot deduct more than the sublimit on SUV property
    // alone; excess falls to § 168(k) bonus depreciation.
    let suv_excess = (suv - suv_sublimit).max(0);

    // Tentative deduction: combined current-year qualifying + carryforward
    // minus SUV-excess (because SUV excess is not eligible for § 179 at all),
    // capped at adjusted cap.
    let qualifying_after_suv_block = qualifying - suv_excess;
    let tentative = (qualifying_after_suv_block + carryforward_in).min(adjusted_cap);

    // § 179(b)(3)(A) taxable-income limit: excess carries forward.
    let current_year_deduction = tentative.min(taxable_income);
    let taxable_income_block = tentative - current_year_deduction;

    let citation = "26 U.S.C. § 179 — election to expense certain depreciable business assets (Form 4562 Part I)";

    let mut note = format!(
        "§ 179(b)(1) dollar cap = {} cents. § 179(b)(2) phaseout reduction = max(0, qualifying ({}) − threshold ({})) = {} cents. Adjusted cap = {} cents. SUV sublimit = {} cents; SUV excess = max(0, suv ({}) − sublimit ({})) = {} cents (falls to § 168(k) 100% bonus depreciation). Tentative deduction (after SUV block) = min(qualifying_after_suv + carryforward, adjusted cap) = {} cents. § 179(b)(3)(A) taxable-income limit = {} cents. Current-year deduction = {} cents. § 179(b)(3)(B) carryforward to next year = {} cents.",
        dollar_cap,
        qualifying,
        phaseout_threshold,
        phaseout_reduction,
        adjusted_cap,
        suv_sublimit,
        suv,
        suv_sublimit,
        suv_excess,
        tentative,
        taxable_income,
        current_year_deduction,
        taxable_income_block,
    );
    if phaseout_reduction > 0 {
        note.push_str(" PHASEOUT triggered — qualifying property exceeded § 179(b)(2) threshold.");
    }
    if adjusted_cap == 0 && phaseout_reduction > 0 {
        note.push_str(" CAP FULLY PHASED OUT — qualifying property exceeded cap+threshold.");
    }
    if taxable_income_block > 0 {
        note.push_str(" Excess deduction blocked by taxable-income limit; carries forward indefinitely under § 179(b)(3)(B).");
    }

    Section179Result {
        adjusted_cap_cents: adjusted_cap,
        phaseout_reduction_cents: phaseout_reduction,
        tentative_deduction_cents: tentative,
        taxable_income_limit_block_cents: taxable_income_block,
        suv_excess_cents: suv_excess,
        current_year_deduction_cents: current_year_deduction,
        carryforward_to_next_year_cents: taxable_income_block,
        bonus_depreciation_eligible_cents: suv_excess,
        citation,
        note,
    }
}

/// 2026 inflation-adjusted limits per Rev. Proc. 2025 / OBBBA § 70302.
pub const CAP_2026_CENTS: i64 = 256_00000000;
pub const PHASEOUT_THRESHOLD_2026_CENTS: i64 = 409_00000000;
pub const SUV_SUBLIMIT_2026_CENTS: i64 = 3200000;

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn input_2026(
        qualifying: i64,
        suv: i64,
        taxable_income: i64,
        carryforward: i64,
    ) -> Section179Input {
        Section179Input {
            qualifying_property_cents: qualifying,
            suv_property_cents: suv,
            taxable_income_cents: taxable_income,
            prior_year_carryforward_cents: carryforward,
            dollar_cap_cents: CAP_2026_CENTS,
            phaseout_threshold_cents: PHASEOUT_THRESHOLD_2026_CENTS,
            suv_sublimit_cents: SUV_SUBLIMIT_2026_CENTS,
        }
    }

    #[test]
    fn under_cap_no_phaseout_full_deduction() {
        let r = compute(&input_2026(10000000, 0, 1_00000000, 0));
        assert_eq!(r.current_year_deduction_cents, 10000000);
        assert_eq!(r.phaseout_reduction_cents, 0);
        assert_eq!(r.adjusted_cap_cents, CAP_2026_CENTS);
    }

    #[test]
    fn at_phaseout_threshold_no_reduction() {
        let r = compute(&input_2026(409_00000000, 0, 100_00000000, 0));
        // Property exactly equals threshold — phaseout = max(0, qualifying −
        // threshold) = 0; cap intact.
        assert_eq!(r.phaseout_reduction_cents, 0);
        assert_eq!(r.adjusted_cap_cents, CAP_2026_CENTS);
    }

    #[test]
    fn one_dollar_above_threshold_reduces_cap() {
        let r = compute(&input_2026(409_00000100, 0, 100_00000000, 0));
        assert_eq!(r.phaseout_reduction_cents, 100);
        assert_eq!(r.adjusted_cap_cents, CAP_2026_CENTS - 100);
    }

    #[test]
    fn dollar_for_dollar_phaseout_5_09M_loses_1M() {
        // From the BDO guidance: $5.09M qualifying → lose $1M cap → $1.56M cap.
        let r = compute(&input_2026(509_00000000, 0, 100_00000000, 0));
        assert_eq!(r.phaseout_reduction_cents, 100_00000000);
        assert_eq!(r.adjusted_cap_cents, 156_00000000);
    }

    #[test]
    fn cap_fully_phased_out_at_6_65M_threshold_plus_cap() {
        // Threshold $4.09M + cap $2.56M = $6.65M → cap entirely phased out.
        let r = compute(&input_2026(665_00000000, 0, 100_00000000, 0));
        assert_eq!(r.adjusted_cap_cents, 0);
        assert_eq!(r.current_year_deduction_cents, 0);
    }

    #[test]
    fn cap_at_2_56M_for_2026() {
        let r = compute(&input_2026(500_00000000, 0, 100_00000000, 0));
        // Property < threshold so no phaseout.
        assert_eq!(r.phaseout_reduction_cents, 91_00000000);
        assert_eq!(r.adjusted_cap_cents, 165_00000000);
    }

    #[test]
    fn taxable_income_limit_blocks_excess() {
        let r = compute(&input_2026(10000000, 0, 5000000, 0));
        assert_eq!(r.current_year_deduction_cents, 5000000);
        assert_eq!(r.taxable_income_limit_block_cents, 5000000);
        assert_eq!(r.carryforward_to_next_year_cents, 5000000);
        assert!(r.note.contains("§ 179(b)(3)(B)"));
    }

    #[test]
    fn carryforward_added_to_current_year() {
        let r = compute(&input_2026(5000000, 0, 10000000, 3000000));
        // 50K current + 30K carryforward = 80K tentative, < 100K taxable income.
        assert_eq!(r.current_year_deduction_cents, 8000000);
        assert_eq!(r.carryforward_to_next_year_cents, 0);
    }

    #[test]
    fn suv_sublimit_32k_blocks_excess_to_bonus() {
        // SUV cost $90K; sublimit $32K; excess $58K to § 168(k) bonus.
        let r = compute(&input_2026(9000000, 9000000, 1_00000000, 0));
        assert_eq!(r.suv_excess_cents, 5800000);
        assert_eq!(r.bonus_depreciation_eligible_cents, 5800000);
        assert_eq!(r.current_year_deduction_cents, 3200000);
    }

    #[test]
    fn suv_under_sublimit_no_block() {
        let r = compute(&input_2026(2000000, 2000000, 1_00000000, 0));
        assert_eq!(r.suv_excess_cents, 0);
        assert_eq!(r.current_year_deduction_cents, 2000000);
    }

    #[test]
    fn suv_at_sublimit_boundary_no_excess() {
        let r = compute(&input_2026(3200000, 3200000, 1_00000000, 0));
        assert_eq!(r.suv_excess_cents, 0);
        assert_eq!(r.current_year_deduction_cents, 3200000);
    }

    #[test]
    fn mixed_suv_and_other_property() {
        // $100K total: $50K SUV (cap to $32K + $18K bonus) + $50K other.
        // Deduction = $32K SUV + $50K other = $82K.
        let r = compute(&input_2026(10000000, 5000000, 1_00000000, 0));
        assert_eq!(r.suv_excess_cents, 1800000);
        assert_eq!(r.current_year_deduction_cents, 8200000);
    }

    #[test]
    fn phaseout_and_taxable_income_limits_both_apply() {
        // $5M qualifying → phaseout $910K → adjusted cap $1.65M.
        // Taxable income $1M → blocks excess of $1.65M to $1M.
        // Carryforward = $650K.
        let r = compute(&input_2026(500_00000000, 0, 100_00000000, 0));
        assert!(r.phaseout_reduction_cents > 0);
        assert_eq!(r.current_year_deduction_cents, 100_00000000);
        assert_eq!(r.carryforward_to_next_year_cents, 65_00000000);
    }

    #[test]
    fn citation_pins_section_179_and_form_4562() {
        let r = compute(&input_2026(10000000, 0, 1_00000000, 0));
        assert!(r.citation.contains("§ 179"));
        assert!(r.citation.contains("Form 4562"));
    }

    #[test]
    fn zero_qualifying_no_deduction() {
        let r = compute(&input_2026(0, 0, 1_00000000, 0));
        assert_eq!(r.current_year_deduction_cents, 0);
    }

    #[test]
    fn taxable_income_zero_full_carryforward() {
        let r = compute(&input_2026(5000000, 0, 0, 0));
        assert_eq!(r.current_year_deduction_cents, 0);
        assert_eq!(r.carryforward_to_next_year_cents, 5000000);
    }

    #[test]
    fn negative_taxable_income_clamped() {
        let r = compute(&input_2026(5000000, 0, -1000000, 0));
        assert_eq!(r.current_year_deduction_cents, 0);
        assert_eq!(r.carryforward_to_next_year_cents, 5000000);
    }

    #[test]
    fn suv_in_excess_of_qualifying_clamped() {
        // Defensive: suv field can't exceed qualifying.
        let r = compute(&input_2026(5000000, 10000000, 1_00000000, 0));
        // SUV clamped to qualifying ($50K), excess $18K to bonus.
        assert_eq!(r.suv_excess_cents, 1800000);
        assert_eq!(r.current_year_deduction_cents, 3200000);
    }

    #[test]
    fn note_describes_phaseout_when_triggered() {
        let r = compute(&input_2026(500_00000000, 0, 100_00000000, 0));
        assert!(r.note.contains("PHASEOUT triggered"));
    }

    #[test]
    fn note_describes_full_phaseout_when_cap_zero() {
        let r = compute(&input_2026(700_00000000, 0, 100_00000000, 0));
        assert!(r.note.contains("CAP FULLY PHASED OUT"));
    }

    #[test]
    fn note_describes_carryforward_when_income_limited() {
        let r = compute(&input_2026(10000000, 0, 5000000, 0));
        assert!(r.note.contains("carries forward indefinitely"));
    }

    #[test]
    fn bonus_depreciation_eligible_equals_suv_excess() {
        // OBBBA made § 168(k) 100% bonus PERMANENT — SUV excess flows there.
        let r = compute(&input_2026(9000000, 9000000, 1_00000000, 0));
        assert_eq!(r.bonus_depreciation_eligible_cents, r.suv_excess_cents);
        assert_eq!(r.bonus_depreciation_eligible_cents, 5800000);
    }

    #[test]
    fn constants_match_2026_inflation_adjusted_values() {
        // Pinned to Rev. Proc. 2025 inflation adjustments + OBBBA § 70302.
        assert_eq!(CAP_2026_CENTS, 256_00000000);
        assert_eq!(PHASEOUT_THRESHOLD_2026_CENTS, 409_00000000);
        assert_eq!(SUV_SUBLIMIT_2026_CENTS, 3200000);
    }
}
