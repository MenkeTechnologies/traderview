//! Rental property tenant accessible parking accommodation
//! right — when must a trader-landlord (1) provide accessible
//! parking spaces as a matter of design and construction
//! requirements under FHA + ADA + state law, AND (2) grant
//! a reasonable accommodation request from a disabled tenant
//! for reserved/accessible parking? Trader-landlord critical
//! for any multifamily property owner: refusal to grant
//! reasonable parking accommodation exposes owner to FHA
//! enforcement + HUD complaints + private right of action +
//! significant civil penalties.
//!
//! Distinct from siblings `emotional_support_animal_
//! documentation` (ESA accommodation framework), `service_
//! animal` (ADA service animal rules), `fha_design_
//! construction` (FHA § 3604(f)(3)(C) design and construction
//! generally), and `fair_chance_housing` (criminal-background
//! screening).
//!
//! **Three regimes**:
//!
//! **Federal FHA Reasonable Accommodation — 42 USC § 3604(f)
//! + 24 CFR § 100.204**:
//! - Applies to ALL multifamily rental housing (no covered-
//!   dwelling exception for reasonable accommodation).
//! - Landlord must grant reasonable accommodation request
//!   from disabled tenant for reserved or accessible parking
//!   space, e.g., space closest to unit entrance.
//! - Reasonable accommodation only required if: (a) tenant
//!   has FHA disability, (b) accommodation necessary for
//!   equal opportunity to use and enjoy dwelling, (c)
//!   accommodation is reasonable (not unduly burdensome).
//!
//! **FHA Design and Construction — 24 CFR § 100.205(c)**:
//! - Applies only to "covered multifamily dwellings" first
//!   occupied AFTER March 13, 1991.
//! - Minimum **2% of parking spaces** must be accessible.
//! - Spaces must be located on an accessible route.
//! - Sufficient number of EACH TYPE (surface, garage,
//!   covered) must be accessible.
//! - Exempts multifamily dwellings with fewer than 4 units;
//!   multifamily townhouses without elevator.
//!
//! **California FEHA + Disabled Persons Act**:
//! - Cal. Gov. Code §§ 12955(c) + 12927(c) — FEHA extends
//!   FHA reasonable accommodation requirements with broader
//!   coverage.
//! - Cal. Civ. Code § 54.1 — Disabled Persons Act establishes
//!   equal right to enjoy housing.
//! - Stronger remedies including statutory damages of at
//!   least $4,000 + attorney's fees.
//!
//! **Default — federal FHA only**. ADA Title III is NOT
//! generally applicable to private residential housing.
//! Title II applies to public housing only.
//!
//! Citations: 42 USC § 3604(f) (FHA); 24 CFR §§ 100.204
//! (reasonable accommodation), 100.205(c) (design and
//! construction parking); Cal. Gov. Code §§ 12955(c),
//! 12927(c) (FEHA); Cal. Civ. Code § 54.1 (Disabled Persons
//! Act).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    FederalFhaOnly,
    California,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    /// Reasonable accommodation analysis (individual disabled
    /// tenant request).
    ReasonableAccommodation,
    /// Design and construction parking spaces (covered
    /// multifamily new construction).
    DesignAndConstruction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantAccessibleParkingInput {
    pub regime: Regime,
    pub analysis_type: AnalysisType,
    /// Whether the tenant has an FHA-recognized disability
    /// (for reasonable accommodation analysis).
    pub tenant_has_disability: bool,
    /// Whether the requested accommodation is necessary for
    /// equal opportunity to use and enjoy dwelling.
    pub accommodation_necessary: bool,
    /// Whether the requested accommodation is reasonable
    /// (not unduly burdensome + no fundamental alteration of
    /// services).
    pub accommodation_reasonable: bool,
    /// Whether landlord granted accommodation request.
    pub accommodation_granted: bool,
    /// Number of parking spaces serving covered dwelling
    /// units (for 2% calculation).
    pub total_parking_spaces: u32,
    /// Number of accessible parking spaces provided.
    pub accessible_parking_spaces: u32,
    /// Whether building has 4+ units (FHA design and
    /// construction threshold).
    pub building_has_4_plus_units: bool,
    /// Whether building was first occupied after March 13,
    /// 1991 (FHA design and construction effective date).
    pub first_occupied_after_march_13_1991: bool,
    /// Whether building is multifamily townhouse without
    /// elevator (FHA design and construction exemption).
    pub multifamily_townhouse_without_elevator: bool,
    /// Whether accessible spaces are located on accessible
    /// route.
    pub on_accessible_route: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantAccessibleParkingResult {
    pub compliant: bool,
    pub design_construction_engaged: bool,
    pub accommodation_must_be_granted: bool,
    pub minimum_accessible_spaces_required: u32,
    pub state_law_extension_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TenantAccessibleParkingInput) -> TenantAccessibleParkingResult {
    let mut violations: Vec<String> = Vec::new();
    let mut accommodation_must_be_granted = false;
    let mut design_construction_engaged = false;
    let mut state_law_extension = false;

    let mut notes: Vec<String> = vec![
        "42 USC § 3604(f) + 24 CFR § 100.204 — FHA requires landlord to grant reasonable accommodation request from disabled tenant for reserved or accessible parking space, e.g., space closest to unit entrance"
            .to_string(),
        "24 CFR § 100.205(c) FHA design and construction — minimum 2% of parking spaces serving covered multifamily dwellings must be accessible AND located on accessible route AND sufficient number of each type (surface + garage + covered)"
            .to_string(),
        "FHA design and construction exemptions — multifamily dwellings with fewer than 4 units; multifamily townhouses without elevator (24 CFR § 100.205)"
            .to_string(),
        "Three-prong reasonable accommodation test under § 100.204: (1) tenant has FHA disability; (2) accommodation necessary for equal opportunity to use and enjoy dwelling; (3) accommodation is reasonable (not unduly burdensome + no fundamental alteration of services)"
            .to_string(),
    ];

    if matches!(input.regime, Regime::California) {
        notes.push(
            "Cal. Gov. Code §§ 12955(c) + 12927(c) FEHA — extends FHA reasonable accommodation requirements; Cal. Civ. Code § 54.1 Disabled Persons Act establishes equal right with statutory damages of at least $4,000 + attorney's fees"
                .to_string(),
        );
        state_law_extension = true;
    }

    match input.analysis_type {
        AnalysisType::ReasonableAccommodation => {
            let three_prong_satisfied = input.tenant_has_disability
                && input.accommodation_necessary
                && input.accommodation_reasonable;
            accommodation_must_be_granted = three_prong_satisfied;

            if three_prong_satisfied && !input.accommodation_granted {
                violations.push(
                    "42 USC § 3604(f) + 24 CFR § 100.204 — landlord must grant reasonable accommodation when three-prong test satisfied (disability + necessity + reasonableness)".to_string(),
                );
            }
        }
        AnalysisType::DesignAndConstruction => {
            let covered_multifamily = input.building_has_4_plus_units
                && input.first_occupied_after_march_13_1991
                && !input.multifamily_townhouse_without_elevator;
            design_construction_engaged = covered_multifamily;

            if covered_multifamily {
                let two_percent_required = (input.total_parking_spaces * 2).div_ceil(100);
                let two_percent_min = two_percent_required.max(1);

                if input.accessible_parking_spaces < two_percent_min {
                    violations.push(format!(
                        "24 CFR § 100.205(c) — only {} of {} parking spaces accessible; minimum 2% ({} spaces) required",
                        input.accessible_parking_spaces, input.total_parking_spaces, two_percent_min
                    ));
                }

                if !input.on_accessible_route {
                    violations.push(
                        "24 CFR § 100.205(c) — accessible parking spaces must be located on accessible route".to_string(),
                    );
                }
            }
        }
    }

    let minimum_required = if matches!(input.analysis_type, AnalysisType::DesignAndConstruction)
        && input.building_has_4_plus_units
        && input.first_occupied_after_march_13_1991
        && !input.multifamily_townhouse_without_elevator
    {
        let two_percent = (input.total_parking_spaces * 2).div_ceil(100);
        two_percent.max(1)
    } else {
        0
    };

    let citation = match input.regime {
        Regime::California => {
            "42 USC § 3604(f); 24 CFR §§ 100.204, 100.205(c); Cal. Gov. Code §§ 12955(c), 12927(c); Cal. Civ. Code § 54.1"
        }
        _ => "42 USC § 3604(f); 24 CFR §§ 100.204, 100.205(c)",
    };

    TenantAccessibleParkingResult {
        compliant: violations.is_empty(),
        design_construction_engaged,
        accommodation_must_be_granted,
        minimum_accessible_spaces_required: minimum_required,
        state_law_extension_engaged: state_law_extension,
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ra_compliant() -> TenantAccessibleParkingInput {
        TenantAccessibleParkingInput {
            regime: Regime::FederalFhaOnly,
            analysis_type: AnalysisType::ReasonableAccommodation,
            tenant_has_disability: true,
            accommodation_necessary: true,
            accommodation_reasonable: true,
            accommodation_granted: true,
            total_parking_spaces: 0,
            accessible_parking_spaces: 0,
            building_has_4_plus_units: true,
            first_occupied_after_march_13_1991: true,
            multifamily_townhouse_without_elevator: false,
            on_accessible_route: true,
        }
    }

    fn dc_compliant() -> TenantAccessibleParkingInput {
        TenantAccessibleParkingInput {
            regime: Regime::FederalFhaOnly,
            analysis_type: AnalysisType::DesignAndConstruction,
            tenant_has_disability: false,
            accommodation_necessary: false,
            accommodation_reasonable: false,
            accommodation_granted: false,
            total_parking_spaces: 50,
            accessible_parking_spaces: 1,
            building_has_4_plus_units: true,
            first_occupied_after_march_13_1991: true,
            multifamily_townhouse_without_elevator: false,
            on_accessible_route: true,
        }
    }

    #[test]
    fn ra_three_prong_satisfied_grant_compliant() {
        let r = check(&ra_compliant());
        assert!(r.compliant);
        assert!(r.accommodation_must_be_granted);
    }

    #[test]
    fn ra_three_prong_satisfied_refusal_violates() {
        let mut i = ra_compliant();
        i.accommodation_granted = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 3604(f)") && v.contains("three-prong test")));
    }

    #[test]
    fn ra_no_disability_no_obligation() {
        let mut i = ra_compliant();
        i.tenant_has_disability = false;
        i.accommodation_granted = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.accommodation_must_be_granted);
    }

    #[test]
    fn ra_accommodation_not_necessary_no_obligation() {
        let mut i = ra_compliant();
        i.accommodation_necessary = false;
        i.accommodation_granted = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.accommodation_must_be_granted);
    }

    #[test]
    fn ra_unreasonable_request_no_obligation() {
        let mut i = ra_compliant();
        i.accommodation_reasonable = false;
        i.accommodation_granted = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.accommodation_must_be_granted);
    }

    #[test]
    fn dc_50_spaces_1_accessible_compliant() {
        let r = check(&dc_compliant());
        assert!(r.compliant);
        assert!(r.design_construction_engaged);
        assert_eq!(r.minimum_accessible_spaces_required, 1);
    }

    #[test]
    fn dc_100_spaces_2_accessible_compliant() {
        let mut i = dc_compliant();
        i.total_parking_spaces = 100;
        i.accessible_parking_spaces = 2;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.minimum_accessible_spaces_required, 2);
    }

    #[test]
    fn dc_100_spaces_1_accessible_violates() {
        let mut i = dc_compliant();
        i.total_parking_spaces = 100;
        i.accessible_parking_spaces = 1;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 100.205(c)") && v.contains("1 of 100")));
    }

    #[test]
    fn dc_under_4_units_not_covered() {
        let mut i = dc_compliant();
        i.building_has_4_plus_units = false;
        i.accessible_parking_spaces = 0;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.design_construction_engaged);
    }

    #[test]
    fn dc_pre_march_13_1991_not_covered() {
        let mut i = dc_compliant();
        i.first_occupied_after_march_13_1991 = false;
        i.accessible_parking_spaces = 0;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.design_construction_engaged);
    }

    #[test]
    fn dc_townhouse_without_elevator_not_covered() {
        let mut i = dc_compliant();
        i.multifamily_townhouse_without_elevator = true;
        i.accessible_parking_spaces = 0;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.design_construction_engaged);
    }

    #[test]
    fn dc_not_on_accessible_route_violates() {
        let mut i = dc_compliant();
        i.on_accessible_route = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("accessible route")));
    }

    #[test]
    fn ca_state_law_extension_engaged() {
        let mut i = ra_compliant();
        i.regime = Regime::California;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.state_law_extension_engaged);
        assert!(r.notes.iter().any(|n| n.contains("FEHA")
            && n.contains("§ 12955(c)")
            && n.contains("$4,000")
            && n.contains("Disabled Persons Act")));
    }

    #[test]
    fn ca_citation_pins_feha_dpa() {
        let mut i = ra_compliant();
        i.regime = Regime::California;
        let r = check(&i);
        assert!(r.citation.contains("§§ 12955(c), 12927(c)"));
        assert!(r.citation.contains("§ 54.1"));
    }

    #[test]
    fn fha_only_no_state_extension() {
        let r = check(&ra_compliant());
        assert!(!r.state_law_extension_engaged);
    }

    #[test]
    fn default_no_state_extension() {
        let mut i = ra_compliant();
        i.regime = Regime::Default;
        let r = check(&i);
        assert!(!r.state_law_extension_engaged);
    }

    #[test]
    fn dc_two_violations_stack_low_count_no_route() {
        let mut i = dc_compliant();
        i.total_parking_spaces = 100;
        i.accessible_parking_spaces = 0;
        i.on_accessible_route = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 2);
    }

    #[test]
    fn ra_three_prong_truth_table() {
        for (disability, necessary, reasonable, exp_grant_required) in [
            (false, false, false, false),
            (true, false, false, false),
            (true, true, false, false),
            (true, true, true, true),
            (false, true, true, false),
            (true, false, true, false),
        ] {
            let mut i = ra_compliant();
            i.tenant_has_disability = disability;
            i.accommodation_necessary = necessary;
            i.accommodation_reasonable = reasonable;
            i.accommodation_granted = exp_grant_required;
            let r = check(&i);
            assert_eq!(r.accommodation_must_be_granted, exp_grant_required);
        }
    }

    #[test]
    fn note_pins_fha_reasonable_accommodation() {
        let r = check(&ra_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 3604(f)")
            && n.contains("§ 100.204")
            && n.contains("reasonable accommodation")));
    }

    #[test]
    fn note_pins_2_percent_design_construction() {
        let r = check(&ra_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 100.205(c)")
            && n.contains("2%")
            && n.contains("accessible route")));
    }

    #[test]
    fn note_pins_exemptions() {
        let r = check(&ra_compliant());
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("fewer than 4 units")
                    && n.contains("townhouses without elevator"))
        );
    }

    #[test]
    fn note_pins_three_prong_test() {
        let r = check(&ra_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Three-prong")
            && n.contains("FHA disability")
            && n.contains("necessary")
            && n.contains("reasonable")));
    }

    #[test]
    fn dc_minimum_calculation_truth_table() {
        for (total, exp_min) in [
            (0_u32, 1_u32),
            (1, 1),
            (49, 1),
            (50, 1),
            (51, 2),
            (100, 2),
            (101, 3),
            (200, 4),
        ] {
            let mut i = dc_compliant();
            i.total_parking_spaces = total;
            i.accessible_parking_spaces = exp_min;
            let r = check(&i);
            assert_eq!(r.minimum_accessible_spaces_required, exp_min);
            assert!(r.compliant);
        }
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::FederalFhaOnly, Regime::California, Regime::Default] {
            let mut i = ra_compliant();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn ca_uniquely_engages_state_extension_invariant() {
        let mut i_ca = ra_compliant();
        i_ca.regime = Regime::California;
        let r_ca = check(&i_ca);
        assert!(r_ca.state_law_extension_engaged);

        let mut i_fha = ra_compliant();
        i_fha.regime = Regime::FederalFhaOnly;
        let r_fha = check(&i_fha);
        assert!(!r_fha.state_law_extension_engaged);

        let mut i_default = ra_compliant();
        i_default.regime = Regime::Default;
        let r_default = check(&i_default);
        assert!(!r_default.state_law_extension_engaged);
    }

    #[test]
    fn dc_design_construction_engagement_truth_table() {
        for (has_4, post_1991, townhouse, exp_engaged) in [
            (false, false, false, false),
            (true, false, false, false),
            (false, true, false, false),
            (true, true, false, true),
            (true, true, true, false),
            (false, false, true, false),
        ] {
            let mut i = dc_compliant();
            i.building_has_4_plus_units = has_4;
            i.first_occupied_after_march_13_1991 = post_1991;
            i.multifamily_townhouse_without_elevator = townhouse;
            i.accessible_parking_spaces = if exp_engaged { 1 } else { 0 };
            let r = check(&i);
            assert_eq!(r.design_construction_engaged, exp_engaged);
        }
    }
}
