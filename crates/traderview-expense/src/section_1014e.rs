//! IRC §1014(e) — Appreciated property acquired by decedent by gift
//! within 1 year of death.
//!
//! Anti-abuse companion to [`section_1014`] (general death basis
//! step-up). Closes the "deathbed gift" loophole: without §1014(e), a
//! healthy person could gift appreciated property to a terminally ill
//! relative, who would then bequeath it back, generating a basis
//! step-up under §1014(a) that washes away the embedded capital gain.
//!
//! **§1014(e) applies when ALL three conditions are met**:
//!
//! 1. The decedent acquired the property BY GIFT (carryover basis
//!    from the donor under §1015).
//! 2. The gift occurred during the **1-year period ending on the
//!    date of the decedent's death**.
//! 3. The property is acquired from the decedent by (or passes from
//!    the decedent to) **the donor of the property OR the spouse of
//!    such donor**.
//!
//! **Effect when all 3 conditions are met**: basis IN THE HANDS OF THE
//! DONOR (or spouse) = decedent's adjusted basis IMMEDIATELY BEFORE
//! DEATH. The normal §1014(a) date-of-death FMV step-up is DENIED.
//!
//! **Only applies to APPRECIATED property**: §1014(e) requires that
//! FMV at gift exceed the donor's adjusted basis at gift. If the
//! property was at a loss or break-even at gift, §1014(e) does not
//! apply (no step-up to be denied).
//!
//! **§ 1014(e) NOT triggered when property passes to a TRUST with
//! multiple beneficiaries** including third parties (e.g., a credit
//! shelter trust naming spouse + children as discretionary
//! beneficiaries). Trust structure breaks the direct "back to donor"
//! linkage that §1014(e) targets. The module surfaces this via the
//! `passes_to_trust_with_other_beneficiaries` input flag.
//!
//! Sources: [Cornell LII 26 U.S.C. § 1014](https://www.law.cornell.edu/uscode/text/26/1014),
//! [Greenleaf Trust — §1014(e) Limitation](https://greenleaftrust.com/missives/basis-step-up-on-death-the-irc-1014e-limitation/),
//! [NAEPC Journal — Understanding §1014(e)](https://www.naepcjournal.org/journal/issue17j.pdf).

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1014eInput {
    pub gift_date: NaiveDate,
    pub decedent_date_of_death: NaiveDate,
    pub donor_adjusted_basis_at_gift_dollars: i64,
    pub fmv_at_gift_dollars: i64,
    pub decedent_adjusted_basis_immediately_before_death_dollars: i64,
    pub fmv_at_death_dollars: i64,
    /// True if the property passes FROM the decedent to the original
    /// donor of the gift, OR to the donor's spouse.
    pub passes_back_to_donor_or_donor_spouse: bool,
    /// True if the property passes to a TRUST that has beneficiaries
    /// other than just the donor and donor's spouse (e.g., credit
    /// shelter trust naming spouse + children). Breaks the direct
    /// pass-back linkage under §1014(e).
    pub passes_to_trust_with_other_beneficiaries: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1014eResult {
    pub days_between_gift_and_death: i64,
    pub within_1_year_window: bool,
    pub property_was_appreciated_at_gift: bool,
    pub trust_structure_breaks_passback: bool,
    pub section_1014e_applies: bool,
    pub basis_step_up_denied: bool,
    pub effective_basis_dollars: i64,
    pub step_up_avoided_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section1014eInput) -> Section1014eResult {
    let days = (input.decedent_date_of_death - input.gift_date).num_days();
    let within_year = days >= 0 && days <= Duration::days(365).num_days();
    let appreciated = input.fmv_at_gift_dollars > input.donor_adjusted_basis_at_gift_dollars;
    let trust_breaks = input.passes_to_trust_with_other_beneficiaries;

    // §1014(e) applies only when ALL conditions met AND trust does
    // not break the pass-back.
    let applies = within_year
        && appreciated
        && input.passes_back_to_donor_or_donor_spouse
        && !trust_breaks;

    let (effective_basis, step_up_avoided) = if applies {
        // §1014(e): basis = decedent's adjusted basis immediately
        // before death. Step-up DENIED.
        let stepped_up_basis = input.fmv_at_death_dollars;
        let carryover_basis = input.decedent_adjusted_basis_immediately_before_death_dollars;
        let avoided = (stepped_up_basis - carryover_basis).max(0);
        (carryover_basis, avoided)
    } else {
        // Normal §1014(a) step-up to date-of-death FMV.
        (input.fmv_at_death_dollars, 0)
    };

    let mut failure_reasons: Vec<String> = Vec::new();
    if !within_year {
        failure_reasons.push(format!(
            "gift-to-death window {} days exceeds 1-year (365 days) threshold",
            days
        ));
    }
    if !appreciated {
        failure_reasons.push(
            "property was NOT appreciated at gift (FMV ≤ donor's adjusted basis); no step-up to deny"
                .to_string(),
        );
    }
    if !input.passes_back_to_donor_or_donor_spouse && !trust_breaks {
        failure_reasons.push("property does not pass back to original donor or donor's spouse".to_string());
    }
    if trust_breaks {
        failure_reasons.push(
            "property passes to trust with other beneficiaries (e.g., credit shelter trust with children); breaks §1014(e) direct pass-back"
                .to_string(),
        );
    }

    let note = if applies {
        format!(
            "§1014(e) APPLIES: {} days from gift to death; appreciated property passes back to donor/spouse. Basis step-up DENIED; basis = ${} carryover (decedent's adjusted basis immediately before death). Step-up of ${} avoided.",
            days,
            effective_basis,
            step_up_avoided,
        )
    } else {
        format!(
            "§1014(e) DOES NOT APPLY ({}). Normal §1014(a) basis step-up to ${} FMV at death.",
            failure_reasons.join("; "),
            effective_basis,
        )
    };

    Section1014eResult {
        days_between_gift_and_death: days,
        within_1_year_window: within_year,
        property_was_appreciated_at_gift: appreciated,
        trust_structure_breaks_passback: trust_breaks,
        section_1014e_applies: applies,
        basis_step_up_denied: applies,
        effective_basis_dollars: effective_basis,
        step_up_avoided_dollars: step_up_avoided,
        citation:
            "IRC §1014(e) appreciated-property-by-gift-within-1-year-of-death basis denial; §1014(a) general date-of-death FMV step-up; §1015 donor carryover basis to donee; Treas. Reg. §1.1014-1; credit-shelter-trust workaround per NAEPC Journal analysis"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn base() -> Section1014eInput {
        Section1014eInput {
            gift_date: d(2025, 6, 1),
            decedent_date_of_death: d(2025, 12, 1), // ~6 months after gift
            donor_adjusted_basis_at_gift_dollars: 100_000,
            fmv_at_gift_dollars: 500_000,
            decedent_adjusted_basis_immediately_before_death_dollars: 100_000,
            fmv_at_death_dollars: 600_000,
            passes_back_to_donor_or_donor_spouse: true,
            passes_to_trust_with_other_beneficiaries: false,
        }
    }

    // §1014(e) full applicability.

    #[test]
    fn baseline_all_3_conditions_met_step_up_denied() {
        let r = compute(&base());
        assert!(r.within_1_year_window);
        assert!(r.property_was_appreciated_at_gift);
        assert!(r.section_1014e_applies);
        assert!(r.basis_step_up_denied);
        assert_eq!(r.effective_basis_dollars, 100_000);
        assert_eq!(r.step_up_avoided_dollars, 500_000);
    }

    #[test]
    fn note_describes_applies_path() {
        let r = compute(&base());
        assert!(r.note.contains("§1014(e) APPLIES"));
        assert!(r.note.contains("DENIED"));
    }

    // 1-year window boundary.

    #[test]
    fn gift_exactly_365_days_before_death_applies() {
        let mut i = base();
        i.gift_date = d(2024, 12, 1);
        i.decedent_date_of_death = d(2025, 12, 1); // 365 days
        let r = compute(&i);
        assert_eq!(r.days_between_gift_and_death, 365);
        assert!(r.within_1_year_window);
        assert!(r.section_1014e_applies);
    }

    #[test]
    fn gift_366_days_before_death_does_not_apply() {
        let mut i = base();
        i.gift_date = d(2024, 11, 30);
        i.decedent_date_of_death = d(2025, 12, 1); // 366 days
        let r = compute(&i);
        assert!(!r.within_1_year_window);
        assert!(!r.section_1014e_applies);
        assert_eq!(r.effective_basis_dollars, 600_000); // Step-up allowed
    }

    #[test]
    fn gift_after_death_clamps_or_invalidates() {
        // Pathological case: gift date AFTER death date.
        let mut i = base();
        i.gift_date = d(2026, 1, 1);
        i.decedent_date_of_death = d(2025, 12, 1);
        let r = compute(&i);
        assert!(!r.within_1_year_window);
        assert!(!r.section_1014e_applies);
    }

    // Appreciation requirement.

    #[test]
    fn property_at_break_even_does_not_trigger_1014e() {
        let mut i = base();
        i.donor_adjusted_basis_at_gift_dollars = 500_000;
        i.fmv_at_gift_dollars = 500_000;
        let r = compute(&i);
        assert!(!r.property_was_appreciated_at_gift);
        assert!(!r.section_1014e_applies);
    }

    #[test]
    fn property_at_loss_does_not_trigger_1014e() {
        let mut i = base();
        i.donor_adjusted_basis_at_gift_dollars = 800_000;
        i.fmv_at_gift_dollars = 500_000;
        let r = compute(&i);
        assert!(!r.property_was_appreciated_at_gift);
        assert!(!r.section_1014e_applies);
    }

    // Pass-back recipient.

    #[test]
    fn property_passes_to_unrelated_party_no_1014e() {
        let mut i = base();
        i.passes_back_to_donor_or_donor_spouse = false;
        let r = compute(&i);
        assert!(!r.section_1014e_applies);
        assert!(r.note.contains("does not pass back"));
    }

    // Credit shelter trust workaround.

    #[test]
    fn credit_shelter_trust_with_other_beneficiaries_breaks_passback() {
        // Even with all other conditions met, trust with children as
        // beneficiaries breaks §1014(e).
        let mut i = base();
        i.passes_to_trust_with_other_beneficiaries = true;
        let r = compute(&i);
        assert!(r.trust_structure_breaks_passback);
        assert!(!r.section_1014e_applies);
        assert_eq!(r.effective_basis_dollars, 600_000);
        assert!(r.note.contains("credit shelter trust"));
    }

    // Effective basis computation.

    #[test]
    fn normal_step_up_when_1014e_inapplicable() {
        let mut i = base();
        i.passes_back_to_donor_or_donor_spouse = false;
        let r = compute(&i);
        assert_eq!(r.effective_basis_dollars, 600_000);
        assert_eq!(r.step_up_avoided_dollars, 0);
    }

    #[test]
    fn step_up_avoided_equals_fmv_minus_carryover_basis() {
        // $600k FMV − $100k carryover = $500k step-up avoided.
        let r = compute(&base());
        assert_eq!(r.step_up_avoided_dollars, 500_000);
    }

    // Multiple disqualifications.

    #[test]
    fn multiple_disqualifications_all_listed_in_note() {
        let mut i = base();
        i.gift_date = d(2023, 1, 1); // > 1 year before death
        i.donor_adjusted_basis_at_gift_dollars = 500_000;
        i.fmv_at_gift_dollars = 500_000; // No appreciation
        let r = compute(&i);
        assert!(!r.section_1014e_applies);
        assert!(r.note.contains("exceeds 1-year"));
        assert!(r.note.contains("NOT appreciated"));
    }

    // Citation.

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§1014(e)"));
        assert!(r.citation.contains("§1014(a)"));
        assert!(r.citation.contains("§1015"));
        assert!(r.citation.contains("credit-shelter-trust"));
    }

    // Precision / large.

    #[test]
    fn very_large_step_up_avoided_precision() {
        let mut i = base();
        i.decedent_adjusted_basis_immediately_before_death_dollars = 1_000_000;
        i.fmv_at_death_dollars = 1_000_000_000;
        let r = compute(&i);
        assert_eq!(r.effective_basis_dollars, 1_000_000);
        assert_eq!(r.step_up_avoided_dollars, 999_000_000);
    }

    // Specific day boundary.

    #[test]
    fn gift_180_days_before_death_within_window() {
        let mut i = base();
        i.gift_date = d(2025, 6, 4);
        i.decedent_date_of_death = d(2025, 12, 1);
        let r = compute(&i);
        assert_eq!(r.days_between_gift_and_death, 180);
        assert!(r.within_1_year_window);
        assert!(r.section_1014e_applies);
    }

    #[test]
    fn gift_zero_days_before_death_same_day_applies() {
        // Gift made on day of death — still within 1-year window.
        let mut i = base();
        i.gift_date = d(2025, 12, 1);
        i.decedent_date_of_death = d(2025, 12, 1);
        let r = compute(&i);
        assert_eq!(r.days_between_gift_and_death, 0);
        assert!(r.within_1_year_window);
        assert!(r.section_1014e_applies);
    }

    // Notes.

    #[test]
    fn note_describes_inapplicable_path_with_reasons() {
        let mut i = base();
        i.gift_date = d(2023, 1, 1);
        let r = compute(&i);
        assert!(r.note.contains("DOES NOT APPLY"));
        assert!(r.note.contains("Normal §1014(a)"));
    }
}
