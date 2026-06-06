//! Mandatory landlord-paid disclosure of LEAD IN DRINKING WATER
//! to tenant — when a public water system notifies a landlord
//! that drinking water serving the building exceeds the EPA lead
//! action level (15 µg/L), what disclosure obligation does the
//! landlord owe to tenants? Distinct from `lead_disclosure`
//! (which addresses federal Title X lead-based PAINT disclosure),
//! `flood_disclosure`, `radon_disclosure`, `asbestos_disclosure`,
//! and `mold_disclosure`. Trader-landlord operational concern in
//! NJ, MI (Flint legacy), and other states with explicit
//! lead-in-drinking-water statutes.
//!
//! Failure to distribute the notice exposes landlord to state
//! enforcement (NJ DEP) + private right of action under state
//! Lead Hazard statutes + common-law negligence per se for
//! pediatric lead poisoning claims.
//!
//! **Three regimes**:
//!
//! **New Jersey — N.J.S.A. 58:12A-12.4 et seq. (Lead in Drinking
//! Water Notification Act)**. Most explicit statutory framework.
//! Public water system MUST notify landlords when sampling shows
//! lead exceeding action level (15 µg/L). Landlord then MUST
//! distribute the notice (1) to EVERY TENANT within THREE
//! BUSINESS DAYS of receipt, AND (2) POST the notice in a
//! PROMINENT LOCATION accessible to tenants. P.L. 2021, c. 82
//! and P.L. 2021, c. 183 amended the framework to include
//! private right of action + civil penalties. NJ DEP enforces.
//!
//! **Michigan — Mich. Comp. Laws § 325.1001 et seq. (Safe
//! Drinking Water Act post-Flint amendments)**. Post-Flint
//! framework. Landlord must distribute Lead Action Level
//! Exceedance Notice when public water supply reports lead above
//! action level. Michigan Lead and Copper Rule (effective 2018)
//! sets state action level at 12 µg/L (below federal 15 µg/L).
//! State enforcement + private right of action.
//!
//! **Default — Federal SDWA + EPA Lead and Copper Rule (40 CFR
//! Part 141.85)**. Federal floor only. Public water system MUST
//! deliver Consumer Confidence Report (CCR) annually + issue
//! Lead Public Education Statements when action level exceeded.
//! NO statutory landlord-tenant distribution mandate at federal
//! level — landlord's obligation is governed by state-specific
//! lead disclosure / habitability framework. State UDAP may
//! reach landlord failure to disclose known lead in drinking
//! water.
//!
//! Citations: N.J.S.A. 58:12A-12.4 et seq. (NJ Lead in Drinking
//! Water Notification Act); P.L. 2021, c. 82; P.L. 2021, c. 183;
//! N.J.A.C. 7:10 (NJ Safe Drinking Water Act regulations); Mich.
//! Comp. Laws § 325.1001 et seq. (MI Safe Drinking Water Act);
//! Michigan Lead and Copper Rule (effective 2018, MAC R
//! 325.10101 et seq.); 42 U.S.C. § 300f et seq. (federal Safe
//! Drinking Water Act); 40 CFR Part 141.85 (EPA Lead and Copper
//! Rule); 40 CFR Part 141 Subpart O (Consumer Confidence Reports).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewJersey,
    Michigan,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LeadInDrinkingWaterInput {
    pub regime: Regime,
    /// Whether the building is served by a public water system
    /// (private well buildings are subject to separate testing
    /// framework).
    pub public_water_system_serves_building: bool,
    /// Whether the public water system has notified the landlord
    /// of lead exceeding action level.
    pub landlord_received_lead_notice: bool,
    /// Lead concentration in micrograms per liter (µg/L) as
    /// reported. Federal action level = 15 µg/L; Michigan state
    /// action level = 12 µg/L.
    pub lead_concentration_micrograms_per_liter: i64,
    /// Whether the landlord distributed the notice to EVERY
    /// tenant within THREE BUSINESS DAYS of receipt (NJ).
    pub distributed_to_tenants_within_3_business_days: bool,
    /// Whether the landlord posted the notice in a PROMINENT
    /// LOCATION accessible to tenants.
    pub posted_in_prominent_location: bool,
    /// Whether the public water system delivered the federal
    /// Consumer Confidence Report (CCR) annually (40 CFR Part
    /// 141 Subpart O).
    pub ccr_delivered_annually: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LeadInDrinkingWaterResult {
    pub compliant: bool,
    pub disclosure_obligation_engaged: bool,
    /// Whether the lead concentration exceeds the federal action
    /// level (15 µg/L).
    pub federal_action_level_exceeded: bool,
    /// Whether the lead concentration exceeds the Michigan state
    /// action level (12 µg/L).
    pub michigan_state_action_level_exceeded: bool,
    pub private_right_of_action_available: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &LeadInDrinkingWaterInput) -> LeadInDrinkingWaterResult {
    let federal_action_level = 15i64;
    let michigan_action_level = 12i64;

    let federal_exceeded = input.lead_concentration_micrograms_per_liter > federal_action_level;
    let michigan_exceeded = input.lead_concentration_micrograms_per_liter > michigan_action_level;

    match input.regime {
        Regime::NewJersey => check_new_jersey(input, federal_exceeded, michigan_exceeded),
        Regime::Michigan => check_michigan(input, federal_exceeded, michigan_exceeded),
        Regime::Default => check_default(input, federal_exceeded, michigan_exceeded),
    }
}

fn check_new_jersey(
    input: &LeadInDrinkingWaterInput,
    federal_exceeded: bool,
    michigan_exceeded: bool,
) -> LeadInDrinkingWaterResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "N.J.S.A. 58:12A-12.4 et seq. (NJ Lead in Drinking Water Notification Act) — public water system MUST notify landlords when sampling shows lead exceeding action level (15 µg/L); landlord then MUST distribute notice to tenants + post in prominent location"
            .to_string(),
        "P.L. 2021, c. 82 + P.L. 2021, c. 183 amended NJ framework to include private right of action + civil penalties; NJ Department of Environmental Protection (NJ DEP) enforces"
            .to_string(),
    ];

    let disclosure_engaged =
        input.public_water_system_serves_building && input.landlord_received_lead_notice;

    if disclosure_engaged {
        if !input.distributed_to_tenants_within_3_business_days {
            violations.push(
                "N.J.S.A. 58:12A-12.4 — landlord MUST distribute lead-in-drinking-water notice to EVERY tenant within THREE BUSINESS DAYS of receipt from public water system"
                    .to_string(),
            );
        }

        if !input.posted_in_prominent_location {
            violations.push(
                "N.J.S.A. 58:12A-12.4 — landlord MUST post the notice in a PROMINENT LOCATION accessible to tenants in addition to direct distribution"
                    .to_string(),
            );
        }
    }

    let compliant = violations.is_empty();
    LeadInDrinkingWaterResult {
        compliant,
        disclosure_obligation_engaged: disclosure_engaged,
        federal_action_level_exceeded: federal_exceeded,
        michigan_state_action_level_exceeded: michigan_exceeded,
        private_right_of_action_available: true,
        violations,
        citation:
            "N.J.S.A. 58:12A-12.4 et seq.; P.L. 2021, c. 82; P.L. 2021, c. 183; N.J.A.C. 7:10",
        notes,
    }
}

fn check_michigan(
    input: &LeadInDrinkingWaterInput,
    federal_exceeded: bool,
    michigan_exceeded: bool,
) -> LeadInDrinkingWaterResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = vec![
        "Mich. Comp. Laws § 325.1001 et seq. (MI Safe Drinking Water Act post-Flint amendments) — Michigan Lead and Copper Rule (effective 2018) sets STATE action level at 12 µg/L (BELOW federal 15 µg/L)"
            .to_string(),
        "MAC R 325.10101 et seq. — landlord must distribute Lead Action Level Exceedance Notice when public water supply reports lead above action level; state enforcement + private right of action"
            .to_string(),
    ];

    let disclosure_engaged =
        input.public_water_system_serves_building && input.landlord_received_lead_notice;

    if disclosure_engaged
        && !input.distributed_to_tenants_within_3_business_days
        && !input.posted_in_prominent_location
    {
        violations.push(
            "Mich. Comp. Laws § 325.1001 et seq. — landlord must distribute Lead Action Level Exceedance Notice to tenants when public water supply reports lead above action level"
                .to_string(),
        );
    }

    if michigan_exceeded {
        notes.push(format!(
            "Michigan state action level (12 µg/L) EXCEEDED — reported concentration {} µg/L",
            input.lead_concentration_micrograms_per_liter
        ));
    }

    let compliant = violations.is_empty();
    LeadInDrinkingWaterResult {
        compliant,
        disclosure_obligation_engaged: disclosure_engaged,
        federal_action_level_exceeded: federal_exceeded,
        michigan_state_action_level_exceeded: michigan_exceeded,
        private_right_of_action_available: true,
        violations,
        citation: "Mich. Comp. Laws § 325.1001 et seq.; MAC R 325.10101 et seq. (Michigan Lead and Copper Rule)",
        notes,
    }
}

fn check_default(
    input: &LeadInDrinkingWaterInput,
    federal_exceeded: bool,
    michigan_exceeded: bool,
) -> LeadInDrinkingWaterResult {
    let mut notes: Vec<String> = vec![
        "default rule — federal Safe Drinking Water Act (42 U.S.C. § 300f et seq.) + EPA Lead and Copper Rule (40 CFR Part 141.85) floor only; federal action level 15 µg/L"
            .to_string(),
        "default rule — NO statutory landlord-tenant distribution mandate at federal level; public water system delivers Consumer Confidence Report (CCR) annually + issues Lead Public Education Statements when action level exceeded; landlord disclosure governed by state-specific lead / habitability framework"
            .to_string(),
        "default rule — state UDAP may reach landlord failure to disclose known lead in drinking water; common-law negligence per se for pediatric lead poisoning claims"
            .to_string(),
    ];

    if input.public_water_system_serves_building && !input.ccr_delivered_annually {
        notes.push(
            "40 CFR Part 141 Subpart O — federal SDWA requires public water system to deliver Consumer Confidence Report annually; failure is system-side violation (NOT landlord)"
                .to_string(),
        );
    }

    LeadInDrinkingWaterResult {
        compliant: true,
        disclosure_obligation_engaged: false,
        federal_action_level_exceeded: federal_exceeded,
        michigan_state_action_level_exceeded: michigan_exceeded,
        private_right_of_action_available: false,
        violations: Vec::new(),
        citation: "42 U.S.C. § 300f et seq. (federal SDWA); 40 CFR Part 141.85 (EPA Lead and Copper Rule); 40 CFR Part 141 Subpart O (CCRs)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nj_clean() -> LeadInDrinkingWaterInput {
        LeadInDrinkingWaterInput {
            regime: Regime::NewJersey,
            public_water_system_serves_building: true,
            landlord_received_lead_notice: true,
            lead_concentration_micrograms_per_liter: 20,
            distributed_to_tenants_within_3_business_days: true,
            posted_in_prominent_location: true,
            ccr_delivered_annually: true,
        }
    }

    fn mi_clean() -> LeadInDrinkingWaterInput {
        let mut i = nj_clean();
        i.regime = Regime::Michigan;
        i
    }

    fn default_clean() -> LeadInDrinkingWaterInput {
        let mut i = nj_clean();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn nj_clean_compliance_passes() {
        let r = check(&nj_clean());
        assert!(r.compliant);
        assert!(r.disclosure_obligation_engaged);
    }

    #[test]
    fn nj_missing_distribution_violates() {
        let mut i = nj_clean();
        i.distributed_to_tenants_within_3_business_days = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("58:12A-12.4") && v.contains("THREE BUSINESS DAYS")));
    }

    #[test]
    fn nj_missing_posting_violates() {
        let mut i = nj_clean();
        i.posted_in_prominent_location = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("58:12A-12.4") && v.contains("PROMINENT LOCATION")));
    }

    #[test]
    fn nj_private_water_no_disclosure_obligation() {
        let mut i = nj_clean();
        i.public_water_system_serves_building = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.disclosure_obligation_engaged);
    }

    #[test]
    fn nj_no_lead_notice_no_obligation() {
        let mut i = nj_clean();
        i.landlord_received_lead_notice = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.disclosure_obligation_engaged);
    }

    #[test]
    fn nj_federal_action_level_15_exceeded() {
        let mut i = nj_clean();
        i.lead_concentration_micrograms_per_liter = 16;
        let r = check(&i);
        assert!(r.federal_action_level_exceeded);
    }

    #[test]
    fn nj_at_15_boundary_not_exceeded() {
        let mut i = nj_clean();
        i.lead_concentration_micrograms_per_liter = 15;
        let r = check(&i);
        assert!(!r.federal_action_level_exceeded);
    }

    #[test]
    fn nj_p_l_2021_amendments_note_present() {
        let r = check(&nj_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("P.L. 2021, c. 82") && n.contains("P.L. 2021, c. 183")));
    }

    #[test]
    fn nj_private_right_of_action_available() {
        let r = check(&nj_clean());
        assert!(r.private_right_of_action_available);
    }

    #[test]
    fn nj_citation_pins_njsa_and_amendments() {
        let r = check(&nj_clean());
        assert!(r.citation.contains("58:12A-12.4"));
        assert!(r.citation.contains("P.L. 2021, c. 82"));
        assert!(r.citation.contains("P.L. 2021, c. 183"));
        assert!(r.citation.contains("N.J.A.C. 7:10"));
    }

    #[test]
    fn mi_clean_compliance_passes() {
        let r = check(&mi_clean());
        assert!(r.compliant);
    }

    #[test]
    fn mi_action_level_12_below_federal_note() {
        let r = check(&mi_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("12 µg/L") && n.contains("BELOW federal 15 µg/L")));
    }

    #[test]
    fn mi_at_13_state_exceeded_federal_not() {
        let mut i = mi_clean();
        i.lead_concentration_micrograms_per_liter = 13;
        let r = check(&i);
        assert!(r.michigan_state_action_level_exceeded);
        assert!(!r.federal_action_level_exceeded);
    }

    #[test]
    fn mi_at_12_boundary_not_exceeded() {
        let mut i = mi_clean();
        i.lead_concentration_micrograms_per_liter = 12;
        let r = check(&i);
        assert!(!r.michigan_state_action_level_exceeded);
    }

    #[test]
    fn mi_at_16_both_exceeded() {
        let mut i = mi_clean();
        i.lead_concentration_micrograms_per_liter = 16;
        let r = check(&i);
        assert!(r.michigan_state_action_level_exceeded);
        assert!(r.federal_action_level_exceeded);
    }

    #[test]
    fn mi_missing_both_distribution_and_posting_violates() {
        let mut i = mi_clean();
        i.distributed_to_tenants_within_3_business_days = false;
        i.posted_in_prominent_location = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn mi_citation_pins_state_lead_and_copper_rule() {
        let r = check(&mi_clean());
        assert!(r.citation.contains("§ 325.1001"));
        assert!(r.citation.contains("325.10101"));
        assert!(r.citation.contains("Michigan Lead and Copper Rule"));
    }

    #[test]
    fn default_compliant_always() {
        let r = check(&default_clean());
        assert!(r.compliant);
    }

    #[test]
    fn default_no_landlord_distribution_mandate_note() {
        let r = check(&default_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n
                .contains("NO statutory landlord-tenant distribution mandate at federal level")));
    }

    #[test]
    fn default_ccr_note_when_missing_ccr() {
        let mut i = default_clean();
        i.ccr_delivered_annually = false;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("40 CFR Part 141 Subpart O")
                && n.contains("Consumer Confidence Report")));
    }

    #[test]
    fn default_state_udap_and_negligence_note() {
        let r = check(&default_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("UDAP") && n.contains("negligence per se")));
    }

    #[test]
    fn default_no_private_right_of_action() {
        let r = check(&default_clean());
        assert!(!r.private_right_of_action_available);
    }

    #[test]
    fn default_citation_references_federal_sdwa_and_lcr() {
        let r = check(&default_clean());
        assert!(r.citation.contains("42 U.S.C. § 300f"));
        assert!(r.citation.contains("40 CFR Part 141.85"));
        assert!(r.citation.contains("Lead and Copper Rule"));
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::NewJersey, Regime::Michigan, Regime::Default] {
            let mut i = nj_clean();
            i.regime = regime;
            let r = check(&i);
            let _ = r.compliant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn nj_three_business_day_unique_invariant() {
        let mut i_nj = nj_clean();
        i_nj.distributed_to_tenants_within_3_business_days = false;
        let r_nj = check(&i_nj);
        assert!(!r_nj.compliant);
        assert!(r_nj
            .violations
            .iter()
            .any(|v| v.contains("THREE BUSINESS DAYS")));

        let mut i_default = default_clean();
        i_default.distributed_to_tenants_within_3_business_days = false;
        let r_default = check(&i_default);
        assert!(r_default.compliant);
    }

    #[test]
    fn nj_unique_private_right_of_action_invariant() {
        let r_nj = check(&nj_clean());
        let r_mi = check(&mi_clean());
        let r_default = check(&default_clean());
        assert!(r_nj.private_right_of_action_available);
        assert!(r_mi.private_right_of_action_available);
        assert!(!r_default.private_right_of_action_available);
    }

    #[test]
    fn michigan_uniquely_lower_action_level_invariant() {
        let mut i_mi = mi_clean();
        i_mi.lead_concentration_micrograms_per_liter = 13;
        let r_mi = check(&i_mi);
        assert!(r_mi.michigan_state_action_level_exceeded);
        assert!(!r_mi.federal_action_level_exceeded);

        let mut i_nj = nj_clean();
        i_nj.lead_concentration_micrograms_per_liter = 13;
        let r_nj = check(&i_nj);
        assert!(!r_nj.federal_action_level_exceeded);
    }

    #[test]
    fn nj_both_violations_simultaneous() {
        let mut i = nj_clean();
        i.distributed_to_tenants_within_3_business_days = false;
        i.posted_in_prominent_location = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 2);
    }

    #[test]
    fn federal_action_level_boundary_truth_table() {
        for concentration in [13i64, 14, 15, 16, 17] {
            let mut i = nj_clean();
            i.lead_concentration_micrograms_per_liter = concentration;
            let r = check(&i);
            assert_eq!(r.federal_action_level_exceeded, concentration > 15);
        }
    }

    #[test]
    fn michigan_action_level_boundary_truth_table() {
        for concentration in [11i64, 12, 13, 14, 15] {
            let mut i = mi_clean();
            i.lead_concentration_micrograms_per_liter = concentration;
            let r = check(&i);
            assert_eq!(r.michigan_state_action_level_exceeded, concentration > 12);
        }
    }

    #[test]
    fn nj_clean_no_violations() {
        let r = check(&nj_clean());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn mi_clean_no_violations() {
        let r = check(&mi_clean());
        assert!(r.violations.is_empty());
    }
}
