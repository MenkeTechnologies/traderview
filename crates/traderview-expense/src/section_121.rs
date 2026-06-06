//! IRC §121 — Exclusion of gain from sale of principal residence.
//!
//! Up to **$250,000 single / $500,000 MFJ** of gain on the sale of a
//! taxpayer's principal residence is excluded from gross income when
//! all of these are met:
//!
//!   * **§121(a) ownership test** — owned the home for at least 2 of
//!     the 5 years ending on the sale date.
//!   * **§121(a) use test** — used the home as principal residence for
//!     at least 2 of the 5 years ending on the sale date.
//!   * **§121(b)(3) once-every-2-years rule** — exclusion is unavailable
//!     if the taxpayer used §121 on another sale within the prior 2
//!     years.
//!
//! Three carve-outs / haircuts:
//!
//!   * **§121(b)(4) reduced maximum exclusion** — if the failure to
//!     meet either 2-year test was due to (a) change in employment,
//!     (b) health, or (c) unforeseen circumstances, the cap is
//!     pro-rated by `(months_used / 24)`. Lets a 12-month qualifier
//!     exclude $125k single / $250k MFJ.
//!
//!   * **§121(b)(5) non-qualified use** — applicable to dispositions
//!     after 2008. Any period AFTER 2008 during which the property was
//!     NOT the taxpayer's principal residence reduces the eligible
//!     gain proportionally: `eligible_gain = realized_gain ×
//!     (qualified_use_days_post_2008 / total_ownership_days_post_2008)`.
//!     §121(b)(5)(C) carves out periods of temporary absence (≤ 2 yrs
//!     for health, employment, unforeseen) and any period after the
//!     last date of qualified use — caller asserts those via input.
//!
//!   * **§121(d)(6) depreciation recapture** — any post-1997 depreciation
//!     (from a prior business / rental use of the property) is NOT
//!     eligible for exclusion. It's recaptured as §1250 unrecaptured
//!     gain at the 25% rate. We surface this as `unrecaptured_section_1250`
//!     in the result.
//!
//! Pure compute. Caller passes sale details + the 2-year test outcomes plus non-qualified-use day counts plus post-1997 depreciation; we compute the exclusion, the taxable LTCG, and the §1250 recapture.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    MarriedFilingSeparately,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReducedExclusionReason {
    /// Sale failed 2-year tests but qualified for §121(b)(4) reduced cap.
    ChangeInEmployment,
    Health,
    UnforeseenCircumstances,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section121Input {
    pub sale_price: Decimal,
    pub selling_costs: Decimal,
    pub adjusted_basis: Decimal,
    pub filing_status: FilingStatus,
    pub sale_date: NaiveDate,
    /// Months of ownership during the 5-year window ending on sale_date.
    /// Used for the 2-year (24-month) test AND the §121(b)(4) pro-rata.
    pub months_owned_in_5yr_window: u32,
    /// Months of principal-residence use in the 5-year window.
    pub months_used_as_residence_in_5yr_window: u32,
    /// True if taxpayer used §121 on a different sale within prior 2 yrs.
    /// §121(b)(3): per-spouse rule — if either spouse used it, joint
    /// return can still use the OTHER spouse's $250k.
    pub used_section_121_within_prior_2_years: bool,
    /// If the 2-year tests fail, caller may assert a §121(b)(4)
    /// hardship reason — we then pro-rate the cap by months/24.
    pub reduced_exclusion_reason: Option<ReducedExclusionReason>,
    /// §121(b)(5) day counts (post-2008). If zero / zero, the rule
    /// is skipped (sale of a property never used as rental post-2008).
    pub non_qualified_use_days_post_2008: u32,
    pub total_ownership_days_post_2008: u32,
    /// §121(d)(6) — depreciation deductions claimed on the property
    /// for periods after May 6, 1997 (e.g. when it was a home office
    /// or short-term rental). Recaptured as §1250 unrecaptured gain.
    pub depreciation_post_1997: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section121Result {
    pub amount_realized: Decimal,
    pub realized_gain: Decimal,
    /// Standard cap: $250k / $500k by filing status.
    pub standard_cap: Decimal,
    /// Cap actually applied (standard, halved for MFS, or §121(b)(4)
    /// pro-rated for hardship).
    pub exclusion_cap_applied: Decimal,
    /// Eligible gain after §121(b)(5) non-qualified-use reduction.
    pub eligible_gain_after_nqu_reduction: Decimal,
    /// Gain attributable to non-qualified use (taxable LTCG).
    pub gain_attributable_to_nqu: Decimal,
    /// §121(d)(6) depreciation recapture — not eligible for exclusion.
    pub unrecaptured_section_1250: Decimal,
    pub gain_excluded: Decimal,
    /// Post-exclusion LTCG portion (over-cap + NQU). Does NOT include
    /// the §1250 recapture — that's a separate rate bucket.
    pub taxable_long_term_gain: Decimal,
    /// Sum of every taxable bucket — LTCG + §1250 recapture. What the
    /// IRS gets paid on, summed across rate brackets.
    pub total_taxable_gain: Decimal,
    pub disqualified: bool,
    pub disqualification_reasons: Vec<String>,
    pub note: String,
}

fn standard_cap(fs: FilingStatus) -> Decimal {
    match fs {
        FilingStatus::MarriedFilingJointly => Decimal::from_str("500000").unwrap(),
        FilingStatus::MarriedFilingSeparately => Decimal::from_str("250000").unwrap(),
        _ => Decimal::from_str("250000").unwrap(),
    }
}

pub fn compute(input: &Section121Input) -> Section121Result {
    let mut r = Section121Result {
        standard_cap: standard_cap(input.filing_status),
        ..Section121Result::default()
    };

    r.amount_realized = input.sale_price - input.selling_costs;
    r.realized_gain = r.amount_realized - input.adjusted_basis;

    if r.realized_gain <= Decimal::ZERO {
        r.note = "loss on sale — §121 not applicable (no gain to exclude)".into();
        r.taxable_long_term_gain = r.realized_gain;
        r.total_taxable_gain = r.realized_gain;
        return r;
    }

    // Test for ownership + use + once-every-2-years (24 months).
    let ownership_passes = input.months_owned_in_5yr_window >= 24;
    let use_passes = input.months_used_as_residence_in_5yr_window >= 24;
    let recent_use_blocks = input.used_section_121_within_prior_2_years;

    let full_qualifies = ownership_passes && use_passes && !recent_use_blocks;

    // Determine the exclusion cap.
    if full_qualifies {
        r.exclusion_cap_applied = r.standard_cap;
    } else if let Some(reason) = input.reduced_exclusion_reason {
        // §121(b)(4): pro-rate cap by months / 24. Use the LESSER of
        // months_owned and months_used as the qualifying months.
        let qualifying_months = input
            .months_owned_in_5yr_window
            .min(input.months_used_as_residence_in_5yr_window)
            .min(24);
        let ratio = Decimal::from(qualifying_months) / Decimal::from(24);
        r.exclusion_cap_applied = (r.standard_cap * ratio).round_dp(2);
        let _ = reason; // reason just unlocks the pro-rata path
    } else {
        r.disqualified = true;
        if !ownership_passes {
            r.disqualification_reasons.push(format!(
                "failed §121(a) ownership test: {} of 24 months",
                input.months_owned_in_5yr_window
            ));
        }
        if !use_passes {
            r.disqualification_reasons.push(format!(
                "failed §121(a) use test: {} of 24 months",
                input.months_used_as_residence_in_5yr_window
            ));
        }
        if recent_use_blocks {
            r.disqualification_reasons
                .push("§121(b)(3): used exclusion on another sale within prior 2 years".into());
        }
        r.taxable_long_term_gain = r.realized_gain;
        r.total_taxable_gain = r.realized_gain;
        r.note = format!(
            "§121 disqualified: {}",
            r.disqualification_reasons.join("; ")
        );
        return r;
    }

    // §121(d)(6) depreciation recapture is recaptured BEFORE exclusion.
    // It's not eligible for §121, period — it's §1250 unrecaptured.
    r.unrecaptured_section_1250 = input
        .depreciation_post_1997
        .min(r.realized_gain)
        .max(Decimal::ZERO);
    let gain_after_recapture = r.realized_gain - r.unrecaptured_section_1250;

    // §121(b)(5) non-qualified use reduction. Skip when both day counts
    // are zero (no post-2008 activity to apportion).
    let nqu_ratio = if input.total_ownership_days_post_2008 > 0 {
        Decimal::from(input.non_qualified_use_days_post_2008)
            / Decimal::from(input.total_ownership_days_post_2008)
    } else {
        Decimal::ZERO
    };
    r.gain_attributable_to_nqu = (gain_after_recapture * nqu_ratio).round_dp(2);
    r.eligible_gain_after_nqu_reduction = gain_after_recapture - r.gain_attributable_to_nqu;

    // Apply exclusion cap to eligible portion.
    r.gain_excluded = r
        .eligible_gain_after_nqu_reduction
        .min(r.exclusion_cap_applied);
    let eligible_over_cap =
        (r.eligible_gain_after_nqu_reduction - r.gain_excluded).max(Decimal::ZERO);
    r.taxable_long_term_gain = eligible_over_cap + r.gain_attributable_to_nqu;
    r.total_taxable_gain = r.taxable_long_term_gain + r.unrecaptured_section_1250;

    r.note = if r.unrecaptured_section_1250 > Decimal::ZERO
        || r.gain_attributable_to_nqu > Decimal::ZERO
    {
        format!(
            "§121: ${} excluded (cap ${}); ${} §1250 recapture + ${} NQU + ${} over-cap = ${} taxable LTCG",
            r.gain_excluded, r.exclusion_cap_applied,
            r.unrecaptured_section_1250, r.gain_attributable_to_nqu, eligible_over_cap,
            r.taxable_long_term_gain,
        )
    } else if eligible_over_cap > Decimal::ZERO {
        format!(
            "§121: ${} excluded at cap; ${} over-cap taxable LTCG",
            r.gain_excluded, eligible_over_cap
        )
    } else {
        format!("§121: full ${} gain excluded", r.gain_excluded)
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section121Input {
        // Single, $200k gain, full qualifier, no NQU, no depreciation.
        Section121Input {
            sale_price: dec!(700000),
            selling_costs: dec!(50000),
            adjusted_basis: dec!(450000),
            filing_status: FilingStatus::Single,
            sale_date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            months_owned_in_5yr_window: 60,
            months_used_as_residence_in_5yr_window: 60,
            used_section_121_within_prior_2_years: false,
            reduced_exclusion_reason: None,
            non_qualified_use_days_post_2008: 0,
            total_ownership_days_post_2008: 0,
            depreciation_post_1997: Decimal::ZERO,
        }
    }

    #[test]
    fn single_full_qualifier_under_cap_full_exclusion() {
        // $700k - $50k - $450k = $200k gain. Single cap $250k. Full exclusion.
        let r = compute(&base());
        assert_eq!(r.realized_gain, dec!(200000));
        assert_eq!(r.exclusion_cap_applied, dec!(250000));
        assert_eq!(r.gain_excluded, dec!(200000));
        assert_eq!(r.taxable_long_term_gain, Decimal::ZERO);
    }

    #[test]
    fn mfj_cap_is_500k() {
        let mut i = base();
        i.filing_status = FilingStatus::MarriedFilingJointly;
        let r = compute(&i);
        assert_eq!(r.standard_cap, dec!(500000));
    }

    #[test]
    fn over_cap_portion_taxable_ltcg() {
        // Single, $400k gain, $250k cap → $150k taxable.
        let mut i = base();
        i.sale_price = dec!(900000);
        let r = compute(&i);
        assert_eq!(r.realized_gain, dec!(400000));
        assert_eq!(r.gain_excluded, dec!(250000));
        assert_eq!(r.taxable_long_term_gain, dec!(150000));
    }

    #[test]
    fn failed_2_year_test_no_reason_disqualified() {
        let mut i = base();
        i.months_owned_in_5yr_window = 18;
        i.months_used_as_residence_in_5yr_window = 18;
        let r = compute(&i);
        assert!(r.disqualified);
        assert_eq!(r.gain_excluded, Decimal::ZERO);
        assert_eq!(r.taxable_long_term_gain, dec!(200000));
        assert!(r
            .disqualification_reasons
            .iter()
            .any(|s| s.contains("ownership")));
        assert!(r.disqualification_reasons.iter().any(|s| s.contains("use")));
    }

    #[test]
    fn section_121_b_4_health_pro_rates_cap_at_12_months() {
        // 12 months qualifying / 24 → cap halved to $125k single.
        let mut i = base();
        i.months_owned_in_5yr_window = 12;
        i.months_used_as_residence_in_5yr_window = 12;
        i.reduced_exclusion_reason = Some(ReducedExclusionReason::Health);
        let r = compute(&i);
        assert_eq!(r.exclusion_cap_applied, dec!(125000));
        // Gain $200k > $125k cap → $125k excluded, $75k taxable.
        assert_eq!(r.gain_excluded, dec!(125000));
        assert_eq!(r.taxable_long_term_gain, dec!(75000));
    }

    #[test]
    fn section_121_b_4_job_move_pro_rates_correctly() {
        let mut i = base();
        i.months_owned_in_5yr_window = 18;
        i.months_used_as_residence_in_5yr_window = 18;
        i.reduced_exclusion_reason = Some(ReducedExclusionReason::ChangeInEmployment);
        let r = compute(&i);
        // 18/24 = 0.75. Cap = 0.75 × $250k = $187,500.
        assert_eq!(r.exclusion_cap_applied, dec!(187500));
    }

    #[test]
    fn section_121_b_4_uses_lesser_of_owned_and_used_months() {
        // Owned 24, used 12 — qualifying months = 12 (lesser).
        let mut i = base();
        i.months_used_as_residence_in_5yr_window = 12;
        i.reduced_exclusion_reason = Some(ReducedExclusionReason::UnforeseenCircumstances);
        let r = compute(&i);
        assert_eq!(r.exclusion_cap_applied, dec!(125000));
    }

    #[test]
    fn once_every_2_years_blocks_full_exclusion() {
        let mut i = base();
        i.used_section_121_within_prior_2_years = true;
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r
            .disqualification_reasons
            .iter()
            .any(|s| s.contains("prior 2 years")));
    }

    #[test]
    fn section_121_b_5_non_qualified_use_proportional() {
        // 10 years post-2008 ownership; 5 years rental + 5 years residence.
        // 10 years = 3653 days; 5 years = 1826 days.
        // NQU ratio = 1826/3653 ≈ 0.5.
        // Gain $200k → NQU portion $100k taxable, eligible $100k excluded.
        let mut i = base();
        i.non_qualified_use_days_post_2008 = 1826;
        i.total_ownership_days_post_2008 = 3653;
        let r = compute(&i);
        let exp_nqu_ratio = Decimal::from(1826) / Decimal::from(3653);
        let exp_nqu = (dec!(200000) * exp_nqu_ratio).round_dp(2);
        assert_eq!(r.gain_attributable_to_nqu, exp_nqu);
        assert_eq!(r.gain_excluded, dec!(200000) - exp_nqu);
        assert_eq!(r.taxable_long_term_gain, exp_nqu);
    }

    #[test]
    fn section_121_d_6_depreciation_recapture_before_exclusion() {
        // $30k post-1997 depreciation (home office) on $200k gain.
        // Recapture $30k as §1250 unrecaptured. Remaining $170k eligible.
        let mut i = base();
        i.depreciation_post_1997 = dec!(30000);
        let r = compute(&i);
        assert_eq!(r.unrecaptured_section_1250, dec!(30000));
        assert_eq!(r.gain_excluded, dec!(170000));
        // LTCG bucket is zero (all eligible gain was excluded).
        assert_eq!(r.taxable_long_term_gain, Decimal::ZERO);
        // But total taxable includes the §1250 recapture.
        assert_eq!(r.total_taxable_gain, dec!(30000));
    }

    #[test]
    fn loss_on_sale_not_excludable_but_recognized() {
        let mut i = base();
        i.sale_price = dec!(350000);
        let r = compute(&i);
        assert!(r.realized_gain < Decimal::ZERO);
        assert_eq!(r.gain_excluded, Decimal::ZERO);
        // §121 only excludes gains; losses on personal residence are
        // NOT deductible per §165(c) but we don't model that here.
        assert!(r.note.contains("loss"));
    }

    #[test]
    fn mfs_uses_250k_not_half_of_mfj() {
        let mut i = base();
        i.filing_status = FilingStatus::MarriedFilingSeparately;
        let r = compute(&i);
        // MFS spouse files separately — gets the single $250k, not
        // half of the $500k joint cap.
        assert_eq!(r.standard_cap, dec!(250000));
    }

    #[test]
    fn combined_recapture_nqu_and_over_cap_all_taxable() {
        // Big gain, all the haircuts at once.
        // $1M sale, $50k costs, $200k basis → $750k gain.
        // $50k depreciation recapture → $700k after recapture.
        // 25% NQU days → $175k NQU taxable, $525k eligible.
        // Single $250k cap → $275k over-cap taxable.
        // Total taxable = $50k + $175k + $275k = $500k. Excluded = $250k.
        let mut i = base();
        i.sale_price = dec!(1000000);
        i.adjusted_basis = dec!(200000);
        i.depreciation_post_1997 = dec!(50000);
        i.non_qualified_use_days_post_2008 = 1;
        i.total_ownership_days_post_2008 = 4;
        let r = compute(&i);
        assert_eq!(r.realized_gain, dec!(750000));
        assert_eq!(r.unrecaptured_section_1250, dec!(50000));
        assert_eq!(r.gain_attributable_to_nqu, dec!(175000));
        assert_eq!(r.gain_excluded, dec!(250000));
        // LTCG bucket = $175k NQU + $275k over-cap = $450k.
        assert_eq!(r.taxable_long_term_gain, dec!(450000));
        // Total = $450k LTCG + $50k §1250 recapture = $500k.
        assert_eq!(r.total_taxable_gain, dec!(500000));
    }

    #[test]
    fn zero_post_2008_ownership_skips_nqu_reduction() {
        // Pre-2008 acquisition, no post-2008 NQU computation.
        let mut i = base();
        i.total_ownership_days_post_2008 = 0;
        i.non_qualified_use_days_post_2008 = 0;
        let r = compute(&i);
        assert_eq!(r.gain_attributable_to_nqu, Decimal::ZERO);
        assert_eq!(r.gain_excluded, dec!(200000));
    }

    #[test]
    fn full_exclusion_note_when_fully_under_cap() {
        let r = compute(&base());
        assert!(r.note.contains("full") && r.note.contains("excluded"));
    }
}
