//! IRC §104 — Compensation for injuries or sickness.
//!
//! Universal for any trader receiving litigation settlement or
//! judgment proceeds. §104(a)(2) EXCLUDES from gross income "the
//! amount of any damages (other than punitive damages) received
//! (whether by suit or agreement and whether as lump sums or as
//! periodic payments) on account of personal PHYSICAL injuries or
//! physical sickness." Several categories of damages within a
//! settlement get different treatment, making the precise
//! categorization in the settlement agreement load-bearing.
//!
//! **§104(a)(2) physical-vs-non-physical distinction** (added by
//! Small Business Job Protection Act of 1996, codifying decades of
//! Tax Court precedent):
//!
//! - Damages on account of PHYSICAL injury or PHYSICAL sickness:
//!   EXCLUDED (compensatory damages, pain and suffering, lost
//!   wages, and emotional distress that ORIGINATES from the
//!   physical injury all qualify).
//! - Damages for emotional distress NOT originating from physical
//!   injury: INCLUDED in gross income — except for the amount
//!   actually paid for medical care attributable to the emotional
//!   distress (which is excluded to that extent).
//!
//! **Punitive damages**: INCLUDED in gross income with one narrow
//! exception — § 104(c) wrongful death actions in states whose
//! wrongful death statute provides ONLY for punitive damages (a
//! handful of states historically). The module surfaces this carve-out
//! via the input flag.
//!
//! **Lost wages and economic damages**: tax treatment follows the
//! ORIGIN of the claim. If lost wages were caused by physical
//! injury → excluded. If from non-physical employment
//! discrimination or contract claims → included.
//!
//! **Interest on award**: ALWAYS INCLUDED. Pre-judgment and
//! post-judgment interest is treated as ordinary income regardless
//! of the underlying claim's tax character.
//!
//! **§ 104(a) flush sentence — tax benefit recapture**: the
//! exclusion does NOT apply to amounts attributable to medical
//! deductions allowed under § 213 in any prior taxable year. If a
//! taxpayer deducted medical expenses and later recovers them via
//! settlement, the recovered amount is INCLUDED to the extent it
//! produced a tax benefit in the deduction year.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section104Input {
    /// Compensatory damages for direct physical injury or sickness
    /// (e.g., car-accident bodily injury award component).
    pub physical_injury_compensatory_dollars: i64,
    /// Pain and suffering arising from physical injury.
    pub pain_suffering_physical_origin_dollars: i64,
    /// Lost wages caused by physical injury.
    pub lost_wages_physical_origin_dollars: i64,
    /// Emotional distress damages where distress ORIGINATES from a
    /// physical injury.
    pub emotional_distress_physical_origin_dollars: i64,
    /// Emotional distress damages NOT arising from physical injury
    /// (e.g., defamation, employment discrimination).
    pub emotional_distress_non_physical_dollars: i64,
    /// Amount of the non-physical emotional distress award actually
    /// paid for medical care for that distress — excluded to that
    /// extent under § 104(a) flush language.
    pub medical_care_for_emotional_distress_dollars: i64,
    pub punitive_damages_dollars: i64,
    /// True if the punitive damages are from a wrongful-death action
    /// in a state whose wrongful death statute provides ONLY for
    /// punitive damages (§ 104(c) exception).
    pub punitive_damages_wrongful_death_only_punitives_state: bool,
    pub interest_on_award_dollars: i64,
    /// Amount the taxpayer previously deducted under § 213 medical
    /// expense for the same injury and from which the taxpayer
    /// realized a tax benefit.
    pub previously_deducted_medical_with_tax_benefit_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section104Result {
    pub physical_injury_excluded_dollars: i64,
    pub emotional_distress_physical_excluded_dollars: i64,
    pub emotional_distress_non_physical_excluded_dollars: i64,
    pub punitive_excluded_dollars: i64,
    pub tax_benefit_recapture_dollars: i64,
    pub total_excluded_from_gross_income_dollars: i64,
    pub total_included_in_gross_income_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section104Input) -> Section104Result {
    // Physical-injury pillar: compensatory + pain/suffering + lost
    // wages + emotional distress from physical injury, all excluded.
    let physical_excluded = input.physical_injury_compensatory_dollars
        + input.pain_suffering_physical_origin_dollars
        + input.lost_wages_physical_origin_dollars
        + input.emotional_distress_physical_origin_dollars;

    let emot_phys_excluded = input.emotional_distress_physical_origin_dollars;

    // Non-physical emotional distress: included EXCEPT to the extent
    // of medical care expenses for that distress.
    let emot_nonphys_excluded = input
        .medical_care_for_emotional_distress_dollars
        .min(input.emotional_distress_non_physical_dollars);
    let emot_nonphys_included =
        input.emotional_distress_non_physical_dollars - emot_nonphys_excluded;

    // Punitives: included except the wrongful-death-only-punitives
    // state carveout under § 104(c).
    let punitive_excluded = if input.punitive_damages_wrongful_death_only_punitives_state {
        input.punitive_damages_dollars
    } else {
        0
    };
    let punitive_included = input.punitive_damages_dollars - punitive_excluded;

    // Interest always included.
    let interest_included = input.interest_on_award_dollars;

    // Tax benefit recapture: previously deducted medical reduces the
    // physical-injury exclusion (added to income to the extent of
    // prior tax benefit).
    let recapture = input
        .previously_deducted_medical_with_tax_benefit_dollars
        .min(physical_excluded);

    let total_excluded = physical_excluded + emot_nonphys_excluded + punitive_excluded - recapture;
    let total_included = emot_nonphys_included + punitive_included + interest_included + recapture;

    let note = format!(
        "§104(a)(2) damages classification — excluded: ${} physical-injury pillar + ${} emotional-distress (non-physical) medical-care carveout + ${} punitive § 104(c) wrongful-death exception − ${} § 104(a) flush prior-§213 tax benefit recapture = ${} total excluded. Included: ${} non-physical emotional distress + ${} punitive damages + ${} interest = ${} total included.",
        physical_excluded,
        emot_nonphys_excluded,
        punitive_excluded,
        recapture,
        total_excluded,
        emot_nonphys_included,
        punitive_included,
        interest_included,
        total_included,
    );

    Section104Result {
        physical_injury_excluded_dollars: physical_excluded,
        emotional_distress_physical_excluded_dollars: emot_phys_excluded,
        emotional_distress_non_physical_excluded_dollars: emot_nonphys_excluded,
        punitive_excluded_dollars: punitive_excluded,
        tax_benefit_recapture_dollars: recapture,
        total_excluded_from_gross_income_dollars: total_excluded,
        total_included_in_gross_income_dollars: total_included,
        citation:
            "IRC §104(a)(2) exclusion for damages on account of personal physical injury/sickness (Small Business Job Protection Act of 1996); §104(a) flush sentence prior-§213 medical deduction tax benefit recapture; §104(c) wrongful-death only-punitives state exception; Treas. Reg. §1.104-1; interest on awards excluded from § 104"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_input() -> Section104Input {
        Section104Input {
            physical_injury_compensatory_dollars: 0,
            pain_suffering_physical_origin_dollars: 0,
            lost_wages_physical_origin_dollars: 0,
            emotional_distress_physical_origin_dollars: 0,
            emotional_distress_non_physical_dollars: 0,
            medical_care_for_emotional_distress_dollars: 0,
            punitive_damages_dollars: 0,
            punitive_damages_wrongful_death_only_punitives_state: false,
            interest_on_award_dollars: 0,
            previously_deducted_medical_with_tax_benefit_dollars: 0,
        }
    }

    // Physical injury exclusion.

    #[test]
    fn physical_injury_compensatory_excluded() {
        let mut i = zero_input();
        i.physical_injury_compensatory_dollars = 500_000;
        let r = compute(&i);
        assert_eq!(r.physical_injury_excluded_dollars, 500_000);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 500_000);
        assert_eq!(r.total_included_in_gross_income_dollars, 0);
    }

    #[test]
    fn pain_suffering_physical_origin_excluded() {
        let mut i = zero_input();
        i.pain_suffering_physical_origin_dollars = 200_000;
        let r = compute(&i);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 200_000);
    }

    #[test]
    fn lost_wages_from_physical_injury_excluded() {
        let mut i = zero_input();
        i.lost_wages_physical_origin_dollars = 150_000;
        let r = compute(&i);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 150_000);
    }

    #[test]
    fn emotional_distress_from_physical_injury_excluded() {
        let mut i = zero_input();
        i.emotional_distress_physical_origin_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.emotional_distress_physical_excluded_dollars, 100_000);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 100_000);
    }

    // Non-physical emotional distress.

    #[test]
    fn emotional_distress_non_physical_fully_included_when_no_medical_care() {
        let mut i = zero_input();
        i.emotional_distress_non_physical_dollars = 80_000;
        let r = compute(&i);
        assert_eq!(r.emotional_distress_non_physical_excluded_dollars, 0);
        assert_eq!(r.total_included_in_gross_income_dollars, 80_000);
    }

    #[test]
    fn emotional_distress_non_physical_partial_exclusion_with_medical_care() {
        // $80k non-physical emotional distress + $10k actual medical
        // care for that distress → $10k excluded, $70k included.
        let mut i = zero_input();
        i.emotional_distress_non_physical_dollars = 80_000;
        i.medical_care_for_emotional_distress_dollars = 10_000;
        let r = compute(&i);
        assert_eq!(r.emotional_distress_non_physical_excluded_dollars, 10_000);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 10_000);
        assert_eq!(r.total_included_in_gross_income_dollars, 70_000);
    }

    #[test]
    fn medical_care_capped_at_emotional_distress_amount() {
        // Medical care can't exceed the emotional distress award itself.
        let mut i = zero_input();
        i.emotional_distress_non_physical_dollars = 5_000;
        i.medical_care_for_emotional_distress_dollars = 20_000;
        let r = compute(&i);
        assert_eq!(r.emotional_distress_non_physical_excluded_dollars, 5_000);
    }

    // Punitive damages.

    #[test]
    fn punitive_damages_included_by_default() {
        let mut i = zero_input();
        i.punitive_damages_dollars = 1_000_000;
        let r = compute(&i);
        assert_eq!(r.punitive_excluded_dollars, 0);
        assert_eq!(r.total_included_in_gross_income_dollars, 1_000_000);
    }

    #[test]
    fn punitive_damages_wrongful_death_state_carveout() {
        // § 104(c): only-punitives state wrongful death → excluded.
        let mut i = zero_input();
        i.punitive_damages_dollars = 500_000;
        i.punitive_damages_wrongful_death_only_punitives_state = true;
        let r = compute(&i);
        assert_eq!(r.punitive_excluded_dollars, 500_000);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 500_000);
        assert_eq!(r.total_included_in_gross_income_dollars, 0);
    }

    // Interest.

    #[test]
    fn interest_on_award_always_included() {
        // Even with physical injury (excluded), interest is included.
        let mut i = zero_input();
        i.physical_injury_compensatory_dollars = 1_000_000;
        i.interest_on_award_dollars = 50_000;
        let r = compute(&i);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 1_000_000);
        assert_eq!(r.total_included_in_gross_income_dollars, 50_000);
    }

    // Tax benefit recapture.

    #[test]
    fn previously_deducted_medical_reduces_exclusion() {
        // $200k physical + $50k prior medical deduction = $200k − $50k = $150k excluded;
        // $50k included.
        let mut i = zero_input();
        i.physical_injury_compensatory_dollars = 200_000;
        i.previously_deducted_medical_with_tax_benefit_dollars = 50_000;
        let r = compute(&i);
        assert_eq!(r.tax_benefit_recapture_dollars, 50_000);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 150_000);
        assert_eq!(r.total_included_in_gross_income_dollars, 50_000);
    }

    #[test]
    fn recapture_capped_at_physical_excluded_amount() {
        // Recapture can't exceed the physical exclusion.
        let mut i = zero_input();
        i.physical_injury_compensatory_dollars = 30_000;
        i.previously_deducted_medical_with_tax_benefit_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.tax_benefit_recapture_dollars, 30_000);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 0);
    }

    // Combined scenarios.

    #[test]
    fn complex_settlement_full_classification() {
        // Real-world style mixed award:
        // - $500k compensatory physical
        // - $200k pain & suffering
        // - $150k lost wages from injury
        // - $100k emotional distress from physical injury
        // - $80k non-physical emotional distress with $10k medical
        // - $300k punitive (not wrongful death)
        // - $25k interest
        // - $20k previously deducted medical
        let i = Section104Input {
            physical_injury_compensatory_dollars: 500_000,
            pain_suffering_physical_origin_dollars: 200_000,
            lost_wages_physical_origin_dollars: 150_000,
            emotional_distress_physical_origin_dollars: 100_000,
            emotional_distress_non_physical_dollars: 80_000,
            medical_care_for_emotional_distress_dollars: 10_000,
            punitive_damages_dollars: 300_000,
            punitive_damages_wrongful_death_only_punitives_state: false,
            interest_on_award_dollars: 25_000,
            previously_deducted_medical_with_tax_benefit_dollars: 20_000,
        };
        let r = compute(&i);
        // Physical excluded: 500k + 200k + 150k + 100k = 950k
        // − recapture 20k = 930k physical net
        // + 10k emotional medical = 940k total excluded.
        assert_eq!(r.physical_injury_excluded_dollars, 950_000);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 940_000);
        // Included: 70k non-physical emotional + 300k punitive + 25k interest + 20k recapture = 415k.
        assert_eq!(r.total_included_in_gross_income_dollars, 415_000);
    }

    #[test]
    fn pure_punitive_award_fully_included() {
        let mut i = zero_input();
        i.punitive_damages_dollars = 5_000_000;
        let r = compute(&i);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 0);
        assert_eq!(r.total_included_in_gross_income_dollars, 5_000_000);
    }

    #[test]
    fn pure_physical_injury_award_fully_excluded() {
        let mut i = zero_input();
        i.physical_injury_compensatory_dollars = 2_000_000;
        let r = compute(&i);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 2_000_000);
        assert_eq!(r.total_included_in_gross_income_dollars, 0);
    }

    #[test]
    fn zero_damages_zero_inclusion_zero_exclusion() {
        let r = compute(&zero_input());
        assert_eq!(r.total_excluded_from_gross_income_dollars, 0);
        assert_eq!(r.total_included_in_gross_income_dollars, 0);
    }

    // Notes / citations.

    #[test]
    fn note_describes_physical_pillar_with_amounts() {
        let mut i = zero_input();
        i.physical_injury_compensatory_dollars = 100_000;
        let r = compute(&i);
        assert!(r.note.contains("§104(a)(2)"));
        assert!(r.note.contains("excluded"));
    }

    #[test]
    fn note_describes_recapture_when_present() {
        let mut i = zero_input();
        i.physical_injury_compensatory_dollars = 100_000;
        i.previously_deducted_medical_with_tax_benefit_dollars = 20_000;
        let r = compute(&i);
        assert!(r.note.contains("§213 tax benefit recapture"));
    }

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&zero_input());
        assert!(r.citation.contains("§104(a)(2)"));
        assert!(r.citation.contains("§104(a) flush"));
        assert!(r.citation.contains("§104(c)"));
        assert!(r.citation.contains("§1.104-1"));
        assert!(r.citation.contains("1996"));
    }

    // Boundary / precision.

    #[test]
    fn very_large_award_precision() {
        let mut i = zero_input();
        i.physical_injury_compensatory_dollars = 100_000_000_000;
        let r = compute(&i);
        assert_eq!(r.total_excluded_from_gross_income_dollars, 100_000_000_000);
    }
}
