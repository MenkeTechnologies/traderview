//! EPA Lead Renovation, Repair, and Painting (RRP) Rule — 40 CFR
//! Part 745, Subpart E. Landlord compliance check for any renovation
//! work performed in target housing (pre-1978) or a child-occupied
//! facility. Critical for any trader-landlord renovating older
//! multifamily property — TSCA § 16(a) penalties are per-day-per-
//! violation and have triggered EPA's largest-ever lead-paint
//! enforcement actions.
//!
//! Distinct from `lead_disclosure` (which addresses TSCA § 1018
//! / 40 CFR Part 745 Subpart F — initial lead-based paint
//! disclosure upon SALE or LEASE), `asbestos_disclosure`, and
//! `bedbug_extermination_cost`. This module addresses the RENOVATION
//! WORK PRACTICES pathway when work disturbs lead-based paint.
//!
//! Two jurisdictional regimes:
//!
//! **EPA Federal — 40 CFR Part 745, Subpart E**. Federal rule
//! applies in states that have not been delegated state primacy
//! by EPA under TSCA § 404. As of late 2025, ~35 states + DC are
//! under federal EPA enforcement. Firm registration via online
//! EPA application; individual renovator certification via EPA-
//! accredited 8-hour training course.
//!
//! **State-Authorized — TSCA § 404 delegated programs**. EPA has
//! delegated RRP authority to 15 states (in order of adoption):
//! Wisconsin, Iowa, North Carolina, Mississippi, Kansas, Rhode
//! Island, Utah, Oregon, Massachusetts, Alabama, Washington,
//! Georgia, Oklahoma, Delaware, Vermont. State law governs but
//! must be AT LEAST AS PROTECTIVE as federal. Firm registration
//! and renovator certification processed through the state lead
//! program.
//!
//! Eight compliance elements (universal):
//!
//!   1. Renovation in TARGET HOUSING (pre-1978) OR CHILD-OCCUPIED
//!      FACILITY (40 CFR § 745.83 definitions).
//!   2. Work DISTURBS PAINT above de minimis threshold (>6 sq ft
//!      interior per room / >20 sq ft exterior / window
//!      replacement / demolition of any size painted surface;
//!      40 CFR § 745.83 "minor repair and maintenance" exemption).
//!   3. Firm EPA-CERTIFIED (or state-authorized equivalent).
//!   4. Individual renovator trained via EPA-accredited 8-hour
//!      course (renovator certification valid 5 years; refresher
//!      4 hours).
//!   5. "Renovate Right" pamphlet (or state equivalent) provided
//!      to OWNER and OCCUPANT before work begins; signed
//!      acknowledgment or certificate of mailing retained.
//!   6. CONTAINMENT used during work (plastic sheeting, signage,
//!      etc.) and prohibited work practices avoided (no open-
//!      flame burning, no machine sanding/grinding without HEPA
//!      attachment, no use of heat guns above 1100°F).
//!   7. CLEANUP VERIFICATION completed (visual inspection, dust
//!      cleanup verification cards, or clearance testing).
//!   8. RECORDS retained for 3 YEARS post-renovation
//!      (40 CFR § 745.86).
//!
//! Penalty exposure: TSCA § 16(a) civil penalty up to $37,500 per
//! day per violation (15 USC § 2615(a)(1)) at statutory base.
//! Inflation-adjusted maximum under 40 CFR § 19.4 is materially
//! higher (mid-$40K range as of 2025 publication; 2026 adjustment
//! canceled by OMB so 2025 figure persists). EPA's largest-ever
//! TSCA RRP penalty was multi-million-dollar consent decree.
//! Criminal penalties under TSCA § 16(b) (15 USC § 2615(b))
//! available for knowing or willful violations — up to $25,000
//! per day plus imprisonment.
//!
//! Citations: 40 CFR Part 745, Subpart E (RRP rule); 40 CFR
//! § 745.81 (purpose and scope); § 745.82 (applicability);
//! § 745.83 (definitions — target housing, child-occupied
//! facility, minor repair and maintenance threshold); § 745.84
//! (information distribution requirements — "Renovate Right");
//! § 745.85 (work practice standards — containment, prohibited
//! practices, cleanup verification); § 745.86 (recordkeeping
//! 3-year retention); § 745.89 (firm certification); § 745.90
//! (renovator individual certification); TSCA § 16(a) (15 USC
//! § 2615(a)(1)) civil penalty $37,500/day; TSCA § 404 (15 USC
//! § 2684) state primacy delegation; 40 CFR § 19.4 (inflation
//! adjustment table).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    EpaFederal,
    StateAuthorized,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RrpInput {
    pub jurisdiction: Jurisdiction,
    /// Element 1 — target housing (pre-1978) OR child-occupied
    /// facility (e.g., pre-1978 daycare). If false, RRP does not
    /// apply.
    pub target_housing_or_child_occupied_facility: bool,
    /// Element 2 — work disturbs paint above the § 745.83 "minor
    /// repair and maintenance" threshold (> 6 sq ft interior per
    /// room / > 20 sq ft exterior / window replacement / any
    /// demolition).
    pub work_disturbs_paint_above_de_minimis: bool,
    /// Element 3 — firm has current EPA or state-authorized
    /// certification (§§ 745.89, 745.90).
    pub firm_epa_or_state_certified: bool,
    /// Element 4 — individual renovator completed EPA-accredited
    /// 8-hour training (5-year validity; 4-hour refresher).
    pub renovator_individually_trained: bool,
    /// Element 5 — "Renovate Right" pamphlet provided to owner and
    /// occupant before work begins (§ 745.84).
    pub renovate_right_pamphlet_provided: bool,
    /// Element 5 supporting — signed acknowledgment or certificate
    /// of mailing retained for the pamphlet delivery.
    pub pamphlet_acknowledgment_retained: bool,
    /// Element 6a — containment used during work (plastic sheeting,
    /// signage, etc.; § 745.85(a)(1)).
    pub containment_used: bool,
    /// Element 6b — prohibited work practices avoided (no open-
    /// flame burning, no machine sanding/grinding without HEPA, no
    /// heat guns above 1100°F; § 745.85(a)(3)).
    pub prohibited_work_practices_avoided: bool,
    /// Element 7 — cleanup verification completed (visual
    /// inspection plus dust-cleanup verification cards or clearance
    /// testing; § 745.85(b)).
    pub cleanup_verification_completed: bool,
    /// Element 8 — records retained for 3 years post-renovation
    /// (§ 745.86).
    pub records_retained_3_years: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RrpResult {
    pub rule_applies: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RrpInput) -> RrpResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.target_housing_or_child_occupied_facility {
        notes.push(
            "RRP does not apply — building is NOT target housing (pre-1978) or child-occupied facility per § 745.83"
                .to_string(),
        );
        return RrpResult {
            rule_applies: false,
            compliant: true,
            violations,
            citation: citation_for(input.jurisdiction),
            notes,
        };
    }

    if !input.work_disturbs_paint_above_de_minimis {
        notes.push(
            "RRP does not apply — work falls within § 745.83 minor repair and maintenance exemption (≤ 6 sq ft interior per room / ≤ 20 sq ft exterior / no window replacement / no demolition)"
                .to_string(),
        );
        return RrpResult {
            rule_applies: false,
            compliant: true,
            violations,
            citation: citation_for(input.jurisdiction),
            notes,
        };
    }

    if !input.firm_epa_or_state_certified {
        violations.push(
            "§§ 745.89, 745.90 — firm not currently EPA or state-authorized certified"
                .to_string(),
        );
    }

    if !input.renovator_individually_trained {
        violations.push(
            "§ 745.90 — individual renovator lacks current EPA-accredited 8-hour training (5-year validity; 4-hour refresher required)"
                .to_string(),
        );
    }

    if !input.renovate_right_pamphlet_provided {
        violations.push(
            "§ 745.84 — \"Renovate Right\" pamphlet (or state equivalent) not provided to owner and occupant before work begins"
                .to_string(),
        );
    }

    if input.renovate_right_pamphlet_provided && !input.pamphlet_acknowledgment_retained {
        violations.push(
            "§ 745.84 — signed acknowledgment or certificate of mailing for pamphlet delivery not retained"
                .to_string(),
        );
    }

    if !input.containment_used {
        violations.push(
            "§ 745.85(a)(1) — containment (plastic sheeting, signage) not used during renovation"
                .to_string(),
        );
    }

    if !input.prohibited_work_practices_avoided {
        violations.push(
            "§ 745.85(a)(3) — prohibited work practices used (open-flame burning, machine sanding/grinding without HEPA, or heat guns above 1100°F)"
                .to_string(),
        );
    }

    if !input.cleanup_verification_completed {
        violations.push(
            "§ 745.85(b) — cleanup verification not completed (visual inspection plus dust-cleanup verification cards or clearance testing)"
                .to_string(),
        );
    }

    if !input.records_retained_3_years {
        violations.push(
            "§ 745.86 — records not retained for 3 years post-renovation"
                .to_string(),
        );
    }

    notes.push(
        "TSCA § 16(a) civil penalty up to $37,500 per day per violation (15 USC § 2615(a)(1)); inflation-adjusted maximum under 40 CFR § 19.4 is materially higher"
            .to_string(),
    );

    match input.jurisdiction {
        Jurisdiction::EpaFederal => {
            notes.push(
                "EPA Federal jurisdiction — firm registration via online EPA application"
                    .to_string(),
            );
        }
        Jurisdiction::StateAuthorized => {
            notes.push(
                "State-Authorized jurisdiction — TSCA § 404 delegated state program (WI, IA, NC, MS, KS, RI, UT, OR, MA, AL, WA, GA, OK, DE, VT); firm registration via state lead program"
                    .to_string(),
            );
        }
    }

    RrpResult {
        rule_applies: true,
        compliant: violations.is_empty(),
        violations,
        citation: citation_for(input.jurisdiction),
        notes,
    }
}

fn citation_for(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::EpaFederal => {
            "40 CFR Part 745 Subpart E (§§ 745.81-745.90); TSCA § 16(a) (15 USC § 2615(a)(1)); 40 CFR § 19.4"
        }
        Jurisdiction::StateAuthorized => {
            "TSCA § 404 (15 USC § 2684); 40 CFR Part 745 Subpart E floor; state lead program implementing rules"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_compliance(jurisdiction: Jurisdiction) -> RrpInput {
        RrpInput {
            jurisdiction,
            target_housing_or_child_occupied_facility: true,
            work_disturbs_paint_above_de_minimis: true,
            firm_epa_or_state_certified: true,
            renovator_individually_trained: true,
            renovate_right_pamphlet_provided: true,
            pamphlet_acknowledgment_retained: true,
            containment_used: true,
            prohibited_work_practices_avoided: true,
            cleanup_verification_completed: true,
            records_retained_3_years: true,
        }
    }

    #[test]
    fn full_compliance_federal_passes() {
        let r = check(&full_compliance(Jurisdiction::EpaFederal));
        assert!(r.rule_applies);
        assert!(r.compliant);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn full_compliance_state_authorized_passes() {
        let r = check(&full_compliance(Jurisdiction::StateAuthorized));
        assert!(r.rule_applies);
        assert!(r.compliant);
    }

    #[test]
    fn post_1978_housing_rule_does_not_apply() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.target_housing_or_child_occupied_facility = false;
        let r = check(&i);
        assert!(!r.rule_applies);
        assert!(r.compliant);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("NOT target housing")));
    }

    #[test]
    fn minor_repair_below_de_minimis_rule_does_not_apply() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.work_disturbs_paint_above_de_minimis = false;
        let r = check(&i);
        assert!(!r.rule_applies);
        assert!(r.compliant);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("minor repair and maintenance")));
    }

    #[test]
    fn missing_firm_certification_violation() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.firm_epa_or_state_certified = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§§ 745.89, 745.90")));
    }

    #[test]
    fn missing_individual_training_violation() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.renovator_individually_trained = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 745.90") && v.contains("8-hour training")));
    }

    #[test]
    fn missing_pamphlet_distribution_violation() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.renovate_right_pamphlet_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 745.84") && v.contains("Renovate Right")));
    }

    #[test]
    fn missing_pamphlet_acknowledgment_violation() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.pamphlet_acknowledgment_retained = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("signed acknowledgment")));
    }

    #[test]
    fn pamphlet_not_provided_skips_acknowledgment_check() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.renovate_right_pamphlet_provided = false;
        i.pamphlet_acknowledgment_retained = false;
        let r = check(&i);
        let acknowledgment_violations: Vec<_> = r
            .violations
            .iter()
            .filter(|v| v.contains("signed acknowledgment"))
            .collect();
        assert!(acknowledgment_violations.is_empty(), "acknowledgment check skipped when pamphlet not provided");
    }

    #[test]
    fn missing_containment_violation() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.containment_used = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 745.85(a)(1)") && v.contains("plastic sheeting")));
    }

    #[test]
    fn prohibited_work_practices_violation() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.prohibited_work_practices_avoided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 745.85(a)(3)") && v.contains("open-flame burning")));
    }

    #[test]
    fn missing_cleanup_verification_violation() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.cleanup_verification_completed = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 745.85(b)") && v.contains("clearance testing")));
    }

    #[test]
    fn missing_records_retention_violation() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.records_retained_3_years = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 745.86") && v.contains("3 years")));
    }

    #[test]
    fn note_pins_tsca_section_16_penalty() {
        let r = check(&full_compliance(Jurisdiction::EpaFederal));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("TSCA § 16(a)") && n.contains("$37,500")));
    }

    #[test]
    fn federal_jurisdiction_note_describes_epa_application() {
        let r = check(&full_compliance(Jurisdiction::EpaFederal));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("EPA Federal jurisdiction") && n.contains("EPA application")));
    }

    #[test]
    fn state_authorized_note_lists_15_states() {
        let r = check(&full_compliance(Jurisdiction::StateAuthorized));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("State-Authorized jurisdiction") && n.contains("WI, IA, NC, MS, KS, RI, UT, OR, MA, AL, WA, GA, OK, DE, VT")));
    }

    #[test]
    fn citation_federal_pins_part_745_and_tsca_16() {
        let r = check(&full_compliance(Jurisdiction::EpaFederal));
        assert!(r.citation.contains("40 CFR Part 745 Subpart E"));
        assert!(r.citation.contains("TSCA § 16(a)"));
        assert!(r.citation.contains("15 USC § 2615(a)(1)"));
        assert!(r.citation.contains("40 CFR § 19.4"));
    }

    #[test]
    fn citation_state_authorized_pins_tsca_404_and_state_floor() {
        let r = check(&full_compliance(Jurisdiction::StateAuthorized));
        assert!(r.citation.contains("TSCA § 404"));
        assert!(r.citation.contains("15 USC § 2684"));
        assert!(r.citation.contains("40 CFR Part 745 Subpart E floor"));
    }

    #[test]
    fn all_eight_elements_must_be_satisfied_invariant() {
        let mut base = full_compliance(Jurisdiction::EpaFederal);
        let fields: [&dyn Fn(&mut RrpInput); 8] = [
            &|i: &mut RrpInput| i.firm_epa_or_state_certified = false,
            &|i: &mut RrpInput| i.renovator_individually_trained = false,
            &|i: &mut RrpInput| i.renovate_right_pamphlet_provided = false,
            &|i: &mut RrpInput| i.pamphlet_acknowledgment_retained = false,
            &|i: &mut RrpInput| i.containment_used = false,
            &|i: &mut RrpInput| i.prohibited_work_practices_avoided = false,
            &|i: &mut RrpInput| i.cleanup_verification_completed = false,
            &|i: &mut RrpInput| i.records_retained_3_years = false,
        ];
        for break_fn in fields.iter() {
            let mut i = base.clone();
            break_fn(&mut i);
            let r = check(&i);
            assert!(!r.compliant, "one element gap should violate compliance");
        }
        let _ = &mut base;
    }

    #[test]
    fn multiple_violations_accumulate_all_listed() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.firm_epa_or_state_certified = false;
        i.renovator_individually_trained = false;
        i.cleanup_verification_completed = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.len() >= 3);
    }

    #[test]
    fn rule_applies_when_target_housing_and_above_de_minimis() {
        let r = check(&full_compliance(Jurisdiction::EpaFederal));
        assert!(r.rule_applies);
    }

    #[test]
    fn neither_target_housing_nor_above_de_minimis_rule_does_not_apply() {
        let mut i = full_compliance(Jurisdiction::EpaFederal);
        i.target_housing_or_child_occupied_facility = false;
        i.work_disturbs_paint_above_de_minimis = false;
        let r = check(&i);
        assert!(!r.rule_applies);
        assert!(r.compliant);
    }
}

