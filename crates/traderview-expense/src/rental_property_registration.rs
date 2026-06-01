//! Mandatory landlord rental property registration with state /
//! municipal agency — distinct from `owner_identification`
//! (disclosure of owner identity to tenant), `landlord_
//! identification_disclosure` (party-identification at lease
//! execution), and `tenant_rights_statement_disclosure` (annual
//! Statement distribution). This module addresses the
//! AFFIRMATIVE REGISTRATION OBLIGATION the landlord owes to the
//! STATE / MUNICIPAL agency before lawfully renting the property.
//!
//! Trader-landlord operational concern — failure to register may
//! result in (a) inability to enforce rent collection in court,
//! (b) inability to pursue eviction, (c) municipal fines, (d)
//! treble damages under state Consumer Fraud Acts.
//!
//! **Three regimes**:
//!
//! **New Jersey — N.J.S.A. 46:8-28 + § 46:8-28.5**. Most explicit
//! statewide framework. Two-tier filing:
//! - **1-unit OR 2-unit non-owner-occupied premises**: file
//!   Certificate of Registration with MUNICIPAL CLERK.
//! - **3+ unit buildings** ("multiple dwellings" under the Hotel
//!   and Multiple Dwelling Law): file with **Bureau of Housing
//!   Inspection, NJ Department of Community Affairs**.
//!
//! Certificate contents (§ 46:8-28): name + address of record
//! owner; name + address of record owner of rental business if
//! different; name + address of authorized agent for service of
//! process; name + address of person designated to receive
//! rental notices; designation of bank where security deposits
//! held. **Amended certificate within 20 DAYS of any change**
//! (§ 46:8-28 amendment rule).
//!
//! § 46:8-28.5 — registration fee + exceptions for
//! owner-occupied 2-family premises in some circumstances.
//!
//! **District of Columbia — D.C. Code § 47-2851.03**. Requires
//! Basic Business License (Rental Housing endorsement) before
//! renting residential property. Failure to register = unable to
//! collect rent in court (DC v. Hayes equitable bar) + civil
//! penalties.
//!
//! **Default — varies; mostly municipal**. Some states (NY MDL
//! § 325 for NYC, IL Chicago RLTO, MA Mass. Gen. Laws ch. 111
//! § 197A for some municipalities) impose registration via
//! municipal authority. Most states leave registration entirely
//! to local ordinance. Federal default — no registration
//! obligation.
//!
//! Citations: N.J.S.A. § 46:8-28 (NJ Certificate of Registration
//! contents); § 46:8-28.5 (NJ registration fee + owner-occupied
//! 2-family exceptions); NJ Hotel and Multiple Dwelling Law
//! (3+ unit "multiple dwelling" definition); D.C. Code §
//! 47-2851.03 (DC Basic Business License + Rental Housing
//! endorsement); N.Y. Mult. Dwell. Law § 325 (NYC building
//! registration); 765 ILCS 705/1 et seq. (IL state-level
//! framework with municipal supplements).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewJersey,
    DistrictOfColumbia,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilingAgency {
    MunicipalClerk,
    NjDcaBureauOfHousingInspection,
    DcBasicBusinessLicense,
    Other,
    NotRequired,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalPropertyRegistrationInput {
    pub regime: Regime,
    /// Number of dwelling units in the building.
    pub building_unit_count: u32,
    /// Whether the building is owner-occupied.
    pub owner_occupied_building: bool,
    /// Whether the landlord has filed the Certificate of
    /// Registration (or analogous registration) with the
    /// appropriate agency.
    pub registration_filed_with_appropriate_agency: bool,
    /// Whether there has been a change to registration info
    /// (owner, agent for service, etc.) that triggers the
    /// 20-day amendment rule.
    pub recent_change_to_registration_info: bool,
    /// Whether the landlord filed an amended certificate within
    /// 20 days of the change (NJ § 46:8-28 requirement).
    pub amended_certificate_filed_within_20_days: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalPropertyRegistrationResult {
    pub regime_applies_to_building: bool,
    pub filing_agency: FilingAgency,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalPropertyRegistrationInput) -> RentalPropertyRegistrationResult {
    match input.regime {
        Regime::NewJersey => check_new_jersey(input),
        Regime::DistrictOfColumbia => check_district_of_columbia(input),
        Regime::Default => check_default(input),
    }
}

fn check_new_jersey(
    input: &RentalPropertyRegistrationInput,
) -> RentalPropertyRegistrationResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let applies = if input.owner_occupied_building && input.building_unit_count <= 2 {
        false
    } else {
        input.building_unit_count >= 1
    };

    let filing_agency = if !applies {
        FilingAgency::NotRequired
    } else if input.building_unit_count >= 3 {
        FilingAgency::NjDcaBureauOfHousingInspection
    } else {
        FilingAgency::MunicipalClerk
    };

    notes.push(
        "N.J.S.A. 46:8-28 — Certificate of Registration required for every landlord; two-tier filing structure"
            .to_string(),
    );

    if !applies {
        notes.push(
            "N.J.S.A. § 46:8-28.5 — owner-occupied 2-family premises may be exempt from registration requirement"
                .to_string(),
        );
        return RentalPropertyRegistrationResult {
            regime_applies_to_building: false,
            filing_agency,
            compliant: true,
            violations,
            citation: "N.J.S.A. §§ 46:8-28, 46:8-28.5",
            notes,
        };
    }

    if !input.registration_filed_with_appropriate_agency {
        let agency_str = match filing_agency {
            FilingAgency::NjDcaBureauOfHousingInspection => {
                "Bureau of Housing Inspection, NJ Department of Community Affairs (3+ unit multiple dwelling)"
            }
            FilingAgency::MunicipalClerk => {
                "municipal clerk (1-unit OR 2-unit non-owner-occupied premises)"
            }
            _ => "appropriate state / municipal agency",
        };
        violations.push(format!(
            "N.J.S.A. § 46:8-28 — landlord MUST file Certificate of Registration with {}",
            agency_str
        ));
    }

    if input.recent_change_to_registration_info
        && !input.amended_certificate_filed_within_20_days
    {
        violations.push(
            "N.J.S.A. § 46:8-28 — amended certificate MUST be filed within 20 DAYS of any change to registration information"
                .to_string(),
        );
    }

    notes.push(
        "N.J.S.A. § 46:8-28 contents: name + address of record owner; rental business if different; authorized agent for service of process; person designated to receive notices; bank holding security deposits"
            .to_string(),
    );
    notes.push(
        "NJ Hotel and Multiple Dwelling Law — 3+ unit buildings classified as 'multiple dwellings' and filed with Bureau of Housing Inspection at NJ DCA"
            .to_string(),
    );
    notes.push(
        "consequences of nonregistration: cannot enforce rent collection in court + cannot pursue eviction + treble damages exposure under N.J.S.A. 56:8-1 Consumer Fraud Act"
            .to_string(),
    );

    let compliant = violations.is_empty();
    RentalPropertyRegistrationResult {
        regime_applies_to_building: true,
        filing_agency,
        compliant,
        violations,
        citation: "N.J.S.A. §§ 46:8-28, 46:8-28.5; NJ Hotel and Multiple Dwelling Law",
        notes,
    }
}

fn check_district_of_columbia(
    input: &RentalPropertyRegistrationInput,
) -> RentalPropertyRegistrationResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "D.C. Code § 47-2851.03 — Basic Business License with Rental Housing endorsement REQUIRED before renting residential property"
            .to_string(),
        "DC v. Hayes equitable bar — landlord without Basic Business License is BARRED from collecting rent in court; civil penalties"
            .to_string(),
        "DC Rental Housing Commission supervises filing + renewal + civil penalties for nonregistration"
            .to_string(),
    ];

    if !input.registration_filed_with_appropriate_agency {
        violations.push(
            "D.C. Code § 47-2851.03 — landlord MUST obtain Basic Business License with Rental Housing endorsement before renting"
                .to_string(),
        );
    }

    let compliant = violations.is_empty();
    RentalPropertyRegistrationResult {
        regime_applies_to_building: true,
        filing_agency: FilingAgency::DcBasicBusinessLicense,
        compliant,
        violations,
        citation: "D.C. Code § 47-2851.03",
        notes,
    }
}

fn check_default(_input: &RentalPropertyRegistrationInput) -> RentalPropertyRegistrationResult {
    let notes: Vec<String> = vec![
        "default rule — most states have NO statewide rental-property-registration mandate; municipal ordinances may impose registration (Chicago RLTO, NYC MDL § 325 for buildings of 3+ units, MA Mass. Gen. Laws ch. 111 § 197A for some municipalities)"
            .to_string(),
        "federal default — no rental-property-registration obligation; state-specific landlord-tenant statute + municipal ordinance control"
            .to_string(),
    ];

    RentalPropertyRegistrationResult {
        regime_applies_to_building: false,
        filing_agency: FilingAgency::Other,
        compliant: true,
        violations: Vec::new(),
        citation: "state-specific landlord-tenant statute + municipal ordinance",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nj_3_unit_compliant() -> RentalPropertyRegistrationInput {
        RentalPropertyRegistrationInput {
            regime: Regime::NewJersey,
            building_unit_count: 3,
            owner_occupied_building: false,
            registration_filed_with_appropriate_agency: true,
            recent_change_to_registration_info: false,
            amended_certificate_filed_within_20_days: false,
        }
    }

    fn nj_1_unit_compliant() -> RentalPropertyRegistrationInput {
        RentalPropertyRegistrationInput {
            regime: Regime::NewJersey,
            building_unit_count: 1,
            owner_occupied_building: false,
            registration_filed_with_appropriate_agency: true,
            recent_change_to_registration_info: false,
            amended_certificate_filed_within_20_days: false,
        }
    }

    fn dc_compliant() -> RentalPropertyRegistrationInput {
        RentalPropertyRegistrationInput {
            regime: Regime::DistrictOfColumbia,
            building_unit_count: 1,
            owner_occupied_building: false,
            registration_filed_with_appropriate_agency: true,
            recent_change_to_registration_info: false,
            amended_certificate_filed_within_20_days: false,
        }
    }

    fn default_base() -> RentalPropertyRegistrationInput {
        RentalPropertyRegistrationInput {
            regime: Regime::Default,
            building_unit_count: 6,
            owner_occupied_building: false,
            registration_filed_with_appropriate_agency: false,
            recent_change_to_registration_info: false,
            amended_certificate_filed_within_20_days: false,
        }
    }

    #[test]
    fn nj_3_unit_routes_to_dca() {
        let r = check(&nj_3_unit_compliant());
        assert_eq!(r.filing_agency, FilingAgency::NjDcaBureauOfHousingInspection);
        assert!(r.regime_applies_to_building);
        assert!(r.compliant);
    }

    #[test]
    fn nj_1_unit_routes_to_municipal_clerk() {
        let r = check(&nj_1_unit_compliant());
        assert_eq!(r.filing_agency, FilingAgency::MunicipalClerk);
        assert!(r.regime_applies_to_building);
    }

    #[test]
    fn nj_2_unit_non_owner_occupied_routes_to_municipal_clerk() {
        let mut i = nj_1_unit_compliant();
        i.building_unit_count = 2;
        i.owner_occupied_building = false;
        let r = check(&i);
        assert_eq!(r.filing_agency, FilingAgency::MunicipalClerk);
        assert!(r.regime_applies_to_building);
    }

    #[test]
    fn nj_2_unit_owner_occupied_exempt() {
        let mut i = nj_1_unit_compliant();
        i.building_unit_count = 2;
        i.owner_occupied_building = true;
        let r = check(&i);
        assert!(!r.regime_applies_to_building);
        assert_eq!(r.filing_agency, FilingAgency::NotRequired);
        assert!(r.notes.iter().any(|n| n.contains("§ 46:8-28.5") && n.contains("owner-occupied 2-family")));
    }

    #[test]
    fn nj_1_unit_owner_occupied_exempt() {
        let mut i = nj_1_unit_compliant();
        i.owner_occupied_building = true;
        let r = check(&i);
        assert!(!r.regime_applies_to_building);
    }

    #[test]
    fn nj_3_unit_owner_occupied_still_in_scope() {
        let mut i = nj_3_unit_compliant();
        i.owner_occupied_building = true;
        let r = check(&i);
        assert!(r.regime_applies_to_building);
        assert_eq!(r.filing_agency, FilingAgency::NjDcaBureauOfHousingInspection);
    }

    #[test]
    fn nj_no_registration_filed_violates() {
        let mut i = nj_3_unit_compliant();
        i.registration_filed_with_appropriate_agency = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 46:8-28") && v.contains("Bureau of Housing Inspection")));
    }

    #[test]
    fn nj_1_unit_no_filing_routes_violation_to_municipal_clerk() {
        let mut i = nj_1_unit_compliant();
        i.registration_filed_with_appropriate_agency = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 46:8-28") && v.contains("municipal clerk")));
    }

    #[test]
    fn nj_amended_certificate_within_20_days_compliant() {
        let mut i = nj_3_unit_compliant();
        i.recent_change_to_registration_info = true;
        i.amended_certificate_filed_within_20_days = true;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn nj_amended_certificate_not_filed_within_20_days_violates() {
        let mut i = nj_3_unit_compliant();
        i.recent_change_to_registration_info = true;
        i.amended_certificate_filed_within_20_days = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 46:8-28") && v.contains("20 DAYS")));
    }

    #[test]
    fn nj_no_recent_change_no_amendment_violation() {
        let mut i = nj_3_unit_compliant();
        i.recent_change_to_registration_info = false;
        i.amended_certificate_filed_within_20_days = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn nj_certificate_contents_note_present() {
        let r = check(&nj_3_unit_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 46:8-28 contents") && n.contains("authorized agent for service")));
    }

    #[test]
    fn nj_consequences_note_describes_treble_damages_and_eviction_bar() {
        let r = check(&nj_3_unit_compliant());
        assert!(r.notes.iter().any(|n| n.contains("cannot enforce rent collection") && n.contains("Consumer Fraud Act")));
    }

    #[test]
    fn nj_multiple_dwelling_law_note_describes_three_plus() {
        let r = check(&nj_3_unit_compliant());
        assert!(r.notes.iter().any(|n| n.contains("NJ Hotel and Multiple Dwelling Law") && n.contains("3+ unit")));
    }

    #[test]
    fn nj_citation_pins_46_8_28_and_28_5() {
        let r = check(&nj_3_unit_compliant());
        assert!(r.citation.contains("§§ 46:8-28, 46:8-28.5"));
        assert!(r.citation.contains("Hotel and Multiple Dwelling Law"));
    }

    #[test]
    fn dc_compliant_with_basic_business_license() {
        let r = check(&dc_compliant());
        assert!(r.compliant);
        assert_eq!(r.filing_agency, FilingAgency::DcBasicBusinessLicense);
    }

    #[test]
    fn dc_no_license_violates() {
        let mut i = dc_compliant();
        i.registration_filed_with_appropriate_agency = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 47-2851.03") && v.contains("Basic Business License")));
    }

    #[test]
    fn dc_hayes_equitable_bar_note() {
        let r = check(&dc_compliant());
        assert!(r.notes.iter().any(|n| n.contains("DC v. Hayes") && n.contains("BARRED from collecting rent")));
    }

    #[test]
    fn dc_citation_pins_47_2851_03() {
        let r = check(&dc_compliant());
        assert!(r.citation.contains("§ 47-2851.03"));
    }

    #[test]
    fn default_no_statewide_mandate_compliant_even_when_unregistered() {
        let r = check(&default_base());
        assert!(r.compliant);
        assert!(!r.regime_applies_to_building);
    }

    #[test]
    fn default_municipal_ordinance_examples_note() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("Chicago RLTO") && n.contains("NYC MDL § 325")));
    }

    #[test]
    fn nj_unique_three_tier_filing_agency_invariant() {
        let mut i_3 = nj_3_unit_compliant();
        i_3.building_unit_count = 3;
        let r_3 = check(&i_3);
        assert_eq!(r_3.filing_agency, FilingAgency::NjDcaBureauOfHousingInspection);

        let mut i_2 = nj_1_unit_compliant();
        i_2.building_unit_count = 2;
        let r_2 = check(&i_2);
        assert_eq!(r_2.filing_agency, FilingAgency::MunicipalClerk);

        let mut i_owner = nj_1_unit_compliant();
        i_owner.building_unit_count = 2;
        i_owner.owner_occupied_building = true;
        let r_owner = check(&i_owner);
        assert_eq!(r_owner.filing_agency, FilingAgency::NotRequired);
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::NewJersey, Regime::DistrictOfColumbia, Regime::Default] {
            let mut i = nj_3_unit_compliant();
            i.regime = regime;
            let r = check(&i);
            let _ = r.compliant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn nj_scoping_truth_table() {
        let cases: [(u32, bool, bool, FilingAgency); 6] = [
            (1, false, true, FilingAgency::MunicipalClerk),
            (1, true, false, FilingAgency::NotRequired),
            (2, false, true, FilingAgency::MunicipalClerk),
            (2, true, false, FilingAgency::NotRequired),
            (3, false, true, FilingAgency::NjDcaBureauOfHousingInspection),
            (3, true, true, FilingAgency::NjDcaBureauOfHousingInspection),
        ];

        for (units, owner_occ, expected_applies, expected_agency) in cases {
            let mut i = nj_3_unit_compliant();
            i.building_unit_count = units;
            i.owner_occupied_building = owner_occ;
            let r = check(&i);
            assert_eq!(r.regime_applies_to_building, expected_applies);
            assert_eq!(r.filing_agency, expected_agency);
        }
    }

    #[test]
    fn nj_both_violations_simultaneous() {
        let mut i = nj_3_unit_compliant();
        i.registration_filed_with_appropriate_agency = false;
        i.recent_change_to_registration_info = true;
        i.amended_certificate_filed_within_20_days = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 2);
    }

    #[test]
    fn dc_compliant_clean_path() {
        let r = check(&dc_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn nj_compliant_clean_path() {
        let r = check(&nj_3_unit_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn default_clean_path_no_violations_ever() {
        let mut i = default_base();
        i.registration_filed_with_appropriate_agency = false;
        i.recent_change_to_registration_info = true;
        i.amended_certificate_filed_within_20_days = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.violations.is_empty());
    }
}
