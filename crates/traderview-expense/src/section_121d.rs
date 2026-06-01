//! IRC §121(d)(2) + §121(d)(3) — Divorce special rules for the
//! § 121 principal-residence-sale exclusion.
//!
//! Companion to [`section_121`] (the general exclusion). When a
//! married couple divorces and one ex-spouse acquires or remains in
//! the principal residence, two §121(d) special rules let BOTH
//! spouses (or just one) preserve §121 exclusion eligibility
//! ([Cornell LII 26 U.S.C. § 121](https://www.law.cornell.edu/uscode/text/26/121),
//! [LegalClarity — §1041 + §121 in Divorce](https://legalclarity.org/the-tax-rules-for-property-transfers-under-irc-section-1041/)).
//!
//! **§121(a) baseline test** — exclusion requires the taxpayer to
//! have OWNED and USED the residence as a principal residence for
//! periods aggregating **2 years or more** out of the 5-year period
//! ending on the date of sale. Single filers exclude up to $250k
//! gain; joint filers exclude up to $500k.
//!
//! **§121(d)(2) holding-period tacking** — when a residence is
//! transferred under § 1041(a) (spouse-to-spouse incident to
//! divorce), the transferee's OWNERSHIP period for § 121 purposes
//! includes the transferor's prior ownership period. So a spouse
//! who receives the home in divorce can immediately count the years
//! when the other spouse owned it.
//!
//! **§121(d)(3)(A) use attribution via former-spouse occupation** —
//! solely for § 121 purposes, an individual is treated as USING
//! the property as their principal residence during any period of
//! ownership while the individual's spouse or FORMER SPOUSE is
//! granted use of the property under a divorce or separation
//! instrument. This lets the ex-spouse who moved out continue
//! accumulating use-period months as long as the other ex-spouse
//! occupies the home pursuant to the divorce decree.
//!
//! **Divorce or separation instrument** = decree of divorce, decree
//! of separate maintenance, or written instrument incident to such
//! a decree (including spousal-support orders).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section121dInput {
    /// True if the residence was transferred to the taxpayer in a
    /// § 1041(a) transaction (incident to divorce). Triggers
    /// §121(d)(2) holding-period tacking.
    pub residence_received_under_section_1041: bool,
    pub transferor_prior_ownership_months: u32,
    pub transferor_prior_use_months: u32,
    pub transferee_own_ownership_months_post_transfer: u32,
    pub transferee_own_use_months_post_transfer: u32,
    /// True if a divorce or separation instrument grants the former
    /// spouse use of the property. Triggers §121(d)(3)(A) use
    /// attribution.
    pub former_spouse_use_under_divorce_instrument: bool,
    /// Months during which the former spouse occupied the residence
    /// pursuant to the divorce/separation instrument while taxpayer
    /// retained ownership.
    pub former_spouse_occupation_months: u32,
    /// Filing status at sale — affects §121(b) exclusion limit.
    pub joint_filer_at_sale: bool,
    pub gain_realized_on_sale_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section121dResult {
    pub total_ownership_months: u32,
    pub total_use_months: u32,
    pub meets_2_year_ownership_test: bool,
    pub meets_2_year_use_test: bool,
    pub section_121d2_tacking_applies: bool,
    pub section_121d3_use_attribution_applies: bool,
    pub section_121_exclusion_available: bool,
    pub exclusion_limit_dollars: i64,
    pub excluded_gain_dollars: i64,
    pub recognized_gain_dollars: i64,
    pub citation: String,
    pub note: String,
}

const TWO_YEAR_MONTHS: u32 = 24;
const SINGLE_EXCLUSION_LIMIT: i64 = 250_000;
const JOINT_EXCLUSION_LIMIT: i64 = 500_000;

pub fn compute(input: &Section121dInput) -> Section121dResult {
    // §121(d)(2) holding-period tacking.
    let tacking_applies = input.residence_received_under_section_1041;
    let total_ownership = if tacking_applies {
        input.transferor_prior_ownership_months
            + input.transferee_own_ownership_months_post_transfer
    } else {
        input.transferee_own_ownership_months_post_transfer
    };

    // §121(d)(3)(A) use attribution: pre-transfer transferor use also
    // tacks under §121(d)(2). Then post-transfer, former-spouse
    // occupation under divorce decree adds to use months.
    let attribution_applies = input.former_spouse_use_under_divorce_instrument;
    let attribution_months = if attribution_applies {
        input.former_spouse_occupation_months
    } else {
        0
    };
    let total_use = if tacking_applies {
        input.transferor_prior_use_months
            + input.transferee_own_use_months_post_transfer
            + attribution_months
    } else {
        input.transferee_own_use_months_post_transfer + attribution_months
    };

    let meets_ownership = total_ownership >= TWO_YEAR_MONTHS;
    let meets_use = total_use >= TWO_YEAR_MONTHS;
    let exclusion_available = meets_ownership && meets_use;

    let limit = if input.joint_filer_at_sale {
        JOINT_EXCLUSION_LIMIT
    } else {
        SINGLE_EXCLUSION_LIMIT
    };

    let excluded = if exclusion_available {
        input.gain_realized_on_sale_dollars.min(limit).max(0)
    } else {
        0
    };
    let recognized = input.gain_realized_on_sale_dollars - excluded;

    let mut note_parts: Vec<String> = Vec::new();
    if tacking_applies {
        note_parts.push(format!(
            "§121(d)(2) tacking: {} months prior + {} months post-transfer = {} months ownership; {} months prior + {} months post-transfer use",
            input.transferor_prior_ownership_months,
            input.transferee_own_ownership_months_post_transfer,
            total_ownership,
            input.transferor_prior_use_months,
            input.transferee_own_use_months_post_transfer,
        ));
    }
    if attribution_applies {
        note_parts.push(format!(
            "§121(d)(3)(A) use attribution: + {} months former-spouse occupation under divorce/separation instrument",
            attribution_months,
        ));
    }
    note_parts.push(format!(
        "Total ownership {} months ({} years); total use {} months ({} years); 2-year tests: ownership {}, use {}",
        total_ownership,
        total_ownership / 12,
        total_use,
        total_use / 12,
        if meets_ownership { "SATISFIED" } else { "NOT MET" },
        if meets_use { "SATISFIED" } else { "NOT MET" },
    ));
    if exclusion_available {
        note_parts.push(format!(
            "§121 exclusion AVAILABLE: limit ${}; ${} excluded; ${} recognized",
            limit, excluded, recognized,
        ));
    } else {
        note_parts.push(format!(
            "§121 exclusion NOT AVAILABLE; full ${} gain recognized",
            input.gain_realized_on_sale_dollars,
        ));
    }

    Section121dResult {
        total_ownership_months: total_ownership,
        total_use_months: total_use,
        meets_2_year_ownership_test: meets_ownership,
        meets_2_year_use_test: meets_use,
        section_121d2_tacking_applies: tacking_applies,
        section_121d3_use_attribution_applies: attribution_applies,
        section_121_exclusion_available: exclusion_available,
        exclusion_limit_dollars: limit,
        excluded_gain_dollars: excluded,
        recognized_gain_dollars: recognized,
        citation:
            "IRC §121(a) 2-year ownership + 2-year use baseline test; §121(b) $250k single / $500k joint exclusion limits; §121(d)(2) §1041(a) holding-period tacking from transferor spouse; §121(d)(3)(A) use attribution via former-spouse occupation under divorce/separation instrument; Treas. Reg. §1.121-1 et seq."
                .to_string(),
        note: note_parts.join(" "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section121dInput {
        Section121dInput {
            residence_received_under_section_1041: false,
            transferor_prior_ownership_months: 0,
            transferor_prior_use_months: 0,
            transferee_own_ownership_months_post_transfer: 36,
            transferee_own_use_months_post_transfer: 36,
            former_spouse_use_under_divorce_instrument: false,
            former_spouse_occupation_months: 0,
            joint_filer_at_sale: false,
            gain_realized_on_sale_dollars: 200_000,
        }
    }

    // Baseline § 121 satisfied without divorce special rules.

    #[test]
    fn baseline_3_year_ownership_and_use_qualifies() {
        let r = compute(&base());
        assert!(r.meets_2_year_ownership_test);
        assert!(r.meets_2_year_use_test);
        assert!(r.section_121_exclusion_available);
        assert_eq!(r.excluded_gain_dollars, 200_000);
    }

    // § 121(d)(2) holding-period tacking.

    #[test]
    fn tacking_lets_transferee_meet_2_year_ownership() {
        // Transferee owned for only 12 months post-transfer, but
        // transferor owned 24 months prior. Tacked total = 36 months ≥ 24.
        let mut i = base();
        i.residence_received_under_section_1041 = true;
        i.transferor_prior_ownership_months = 24;
        i.transferor_prior_use_months = 24;
        i.transferee_own_ownership_months_post_transfer = 12;
        i.transferee_own_use_months_post_transfer = 12;
        let r = compute(&i);
        assert!(r.section_121d2_tacking_applies);
        assert_eq!(r.total_ownership_months, 36);
        assert!(r.meets_2_year_ownership_test);
        assert!(r.section_121_exclusion_available);
    }

    #[test]
    fn no_tacking_without_1041_transfer() {
        let mut i = base();
        i.residence_received_under_section_1041 = false;
        i.transferor_prior_ownership_months = 100; // Should not count
        i.transferee_own_ownership_months_post_transfer = 12;
        let r = compute(&i);
        assert!(!r.section_121d2_tacking_applies);
        assert_eq!(r.total_ownership_months, 12);
    }

    // § 121(d)(3)(A) use attribution.

    #[test]
    fn use_attribution_through_former_spouse_occupation() {
        // Taxpayer (ex-spouse) moved out of home 18 months ago.
        // Ex-spouse occupied for 18 months under divorce decree.
        // Taxpayer's own use 12 months before move-out + 18 attribution
        // = 30 months use ≥ 24.
        let mut i = base();
        i.residence_received_under_section_1041 = false;
        i.transferee_own_ownership_months_post_transfer = 36;
        i.transferee_own_use_months_post_transfer = 12;
        i.former_spouse_use_under_divorce_instrument = true;
        i.former_spouse_occupation_months = 18;
        let r = compute(&i);
        assert!(r.section_121d3_use_attribution_applies);
        assert_eq!(r.total_use_months, 30);
        assert!(r.meets_2_year_use_test);
    }

    #[test]
    fn use_attribution_does_not_apply_without_divorce_instrument() {
        let mut i = base();
        i.former_spouse_use_under_divorce_instrument = false;
        i.former_spouse_occupation_months = 24; // Ignored without instrument
        let r = compute(&i);
        assert!(!r.section_121d3_use_attribution_applies);
        assert_eq!(r.total_use_months, 36); // Just transferee's own use
    }

    #[test]
    fn use_attribution_at_18_months_alone_not_enough() {
        // Use attribution adds 18 months, but transferee's own use is 0.
        // Total 18 months < 24-month threshold.
        let mut i = base();
        i.transferee_own_use_months_post_transfer = 0;
        i.former_spouse_use_under_divorce_instrument = true;
        i.former_spouse_occupation_months = 18;
        let r = compute(&i);
        assert_eq!(r.total_use_months, 18);
        assert!(!r.meets_2_year_use_test);
        assert!(!r.section_121_exclusion_available);
    }

    // Combined tacking + attribution.

    #[test]
    fn tacking_plus_attribution_chains_correctly() {
        // Transferor prior: 12 months ownership + 12 months use.
        // Transferee post: 6 months ownership + 0 months use (immediately moved out).
        // Former-spouse occupation: 18 months under decree.
        // Tacked ownership: 12 + 6 = 18 — DOES NOT meet 24 threshold.
        let mut i = base();
        i.residence_received_under_section_1041 = true;
        i.transferor_prior_ownership_months = 12;
        i.transferor_prior_use_months = 12;
        i.transferee_own_ownership_months_post_transfer = 6;
        i.transferee_own_use_months_post_transfer = 0;
        i.former_spouse_use_under_divorce_instrument = true;
        i.former_spouse_occupation_months = 18;
        let r = compute(&i);
        assert_eq!(r.total_ownership_months, 18);
        assert!(!r.meets_2_year_ownership_test);
        // Use = 12 prior + 0 post + 18 attribution = 30 → meets use
        assert_eq!(r.total_use_months, 30);
        assert!(r.meets_2_year_use_test);
        // But ownership not met → no exclusion.
        assert!(!r.section_121_exclusion_available);
    }

    #[test]
    fn tacking_plus_attribution_both_satisfy_exclusion_available() {
        let mut i = base();
        i.residence_received_under_section_1041 = true;
        i.transferor_prior_ownership_months = 24;
        i.transferor_prior_use_months = 12;
        i.transferee_own_ownership_months_post_transfer = 12;
        i.transferee_own_use_months_post_transfer = 0;
        i.former_spouse_use_under_divorce_instrument = true;
        i.former_spouse_occupation_months = 12;
        let r = compute(&i);
        // Ownership: 24 + 12 = 36 ✓
        // Use: 12 + 0 + 12 = 24 ✓
        assert!(r.section_121d2_tacking_applies);
        assert!(r.section_121d3_use_attribution_applies);
        assert!(r.meets_2_year_ownership_test);
        assert!(r.meets_2_year_use_test);
        assert!(r.section_121_exclusion_available);
    }

    // Exclusion limits.

    #[test]
    fn single_filer_250k_limit() {
        let mut i = base();
        i.joint_filer_at_sale = false;
        i.gain_realized_on_sale_dollars = 300_000;
        let r = compute(&i);
        assert_eq!(r.exclusion_limit_dollars, 250_000);
        assert_eq!(r.excluded_gain_dollars, 250_000);
        assert_eq!(r.recognized_gain_dollars, 50_000);
    }

    #[test]
    fn joint_filer_500k_limit() {
        let mut i = base();
        i.joint_filer_at_sale = true;
        i.gain_realized_on_sale_dollars = 600_000;
        let r = compute(&i);
        assert_eq!(r.exclusion_limit_dollars, 500_000);
        assert_eq!(r.excluded_gain_dollars, 500_000);
        assert_eq!(r.recognized_gain_dollars, 100_000);
    }

    #[test]
    fn gain_below_limit_fully_excluded() {
        let mut i = base();
        i.gain_realized_on_sale_dollars = 150_000;
        let r = compute(&i);
        assert_eq!(r.excluded_gain_dollars, 150_000);
        assert_eq!(r.recognized_gain_dollars, 0);
    }

    // 2-year threshold boundaries.

    #[test]
    fn twenty_three_months_ownership_does_not_meet_test() {
        let mut i = base();
        i.transferee_own_ownership_months_post_transfer = 23;
        let r = compute(&i);
        assert!(!r.meets_2_year_ownership_test);
        assert!(!r.section_121_exclusion_available);
    }

    #[test]
    fn twenty_four_months_exact_ownership_meets_test() {
        let mut i = base();
        i.transferee_own_ownership_months_post_transfer = 24;
        i.transferee_own_use_months_post_transfer = 24;
        let r = compute(&i);
        assert!(r.meets_2_year_ownership_test);
        assert!(r.meets_2_year_use_test);
        assert!(r.section_121_exclusion_available);
    }

    // No exclusion path.

    #[test]
    fn no_exclusion_full_gain_recognized() {
        let mut i = base();
        i.transferee_own_ownership_months_post_transfer = 12;
        let r = compute(&i);
        assert!(!r.section_121_exclusion_available);
        assert_eq!(r.recognized_gain_dollars, 200_000);
    }

    // Notes / citations.

    #[test]
    fn note_describes_tacking_when_applies() {
        let mut i = base();
        i.residence_received_under_section_1041 = true;
        i.transferor_prior_ownership_months = 24;
        let r = compute(&i);
        assert!(r.note.contains("§121(d)(2) tacking"));
    }

    #[test]
    fn note_describes_use_attribution_when_applies() {
        let mut i = base();
        i.former_spouse_use_under_divorce_instrument = true;
        i.former_spouse_occupation_months = 12;
        let r = compute(&i);
        assert!(r.note.contains("§121(d)(3)(A) use attribution"));
    }

    #[test]
    fn note_describes_exclusion_available_path() {
        let r = compute(&base());
        assert!(r.note.contains("§121 exclusion AVAILABLE"));
    }

    #[test]
    fn note_describes_no_exclusion_path() {
        let mut i = base();
        i.transferee_own_ownership_months_post_transfer = 12;
        let r = compute(&i);
        assert!(r.note.contains("NOT AVAILABLE"));
    }

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§121(a)"));
        assert!(r.citation.contains("§121(b)"));
        assert!(r.citation.contains("§121(d)(2)"));
        assert!(r.citation.contains("§121(d)(3)(A)"));
        assert!(r.citation.contains("§1.121-1"));
    }
}
