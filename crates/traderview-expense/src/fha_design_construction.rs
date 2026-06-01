//! Fair Housing Act design and construction requirements for
//! covered multifamily dwellings — 24 CFR § 100.205. Federal
//! accessibility requirements for new-construction multifamily
//! buildings first occupied after March 13, 1991.
//!
//! Trader-landlord operational concern when acquiring, developing,
//! or substantially renovating a covered multifamily dwelling. The
//! FHA design and construction requirements are a FEDERAL FLOOR;
//! state codes (CA Title 24, MA AAB, NY HRL) may add stricter
//! requirements but cannot reduce the federal baseline.
//!
//! Distinct from `reasonable_accommodation_modification` (which
//! addresses INDIVIDUAL TENANT requests for accommodations and
//! modifications under FHA § 3604(f)(3)), `service_animal`
//! (ADA/FHA service-animal framework), and `emotional_support_
//! animal_documentation` (ESA reliability framework). This module
//! addresses ONLY the NEW-CONSTRUCTION DESIGN-LEVEL requirements.
//!
//! § 100.205(a) APPLICABILITY — covered multifamily dwellings
//! designed and constructed for first occupancy after March 13,
//! 1991 must be designed and constructed for accessibility.
//! "Covered multifamily dwellings" = buildings consisting of 4 or
//! more dwelling units IF the building has one or more elevators
//! (all floors must be accessible) OR ground-floor units in
//! buildings without an elevator.
//!
//! § 100.205(a) BURDEN OF PROOF — first occupancy presumed
//! achieved by March 13, 1991 if the dwelling was occupied by that
//! date OR the last building permit / renewal was issued by a
//! state, county, or local government on or before June 15, 1990.
//!
//! § 100.205(c) SEVEN DESIGN REQUIREMENTS:
//!
//!   1. ACCESSIBLE BUILDING ENTRANCE on an accessible route
//!      (unless impractical due to terrain or unusual site
//!      characteristics — burden on designer/builder)
//!   2. ACCESSIBLE AND USABLE public and common use areas
//!      (lobbies, mailbox areas, parking, recreation facilities)
//!   3. USABLE DOORS — sufficient width for wheelchair passage
//!      (typically 32" clear opening)
//!   4. ACCESSIBLE ROUTE into and through the covered dwelling
//!      unit
//!   5. LIGHT SWITCHES, ELECTRICAL OUTLETS, THERMOSTATS and other
//!      environmental controls in ACCESSIBLE LOCATIONS
//!   6. REINFORCED WALLS for later installation of grab bars in
//!      bathrooms
//!   7. USABLE KITCHENS AND BATHROOMS such that a wheelchair user
//!      can maneuver
//!
//! Failure to comply with ANY of the seven requirements
//! constitutes a violation of 42 U.S.C. § 3604(f)(3)(C) of the
//! Fair Housing Act, exposing the designer/builder/owner to
//! private suit, HUD administrative enforcement, and DOJ pattern-
//! or-practice litigation. § 3613(c) damages: actual + punitive +
//! attorney fees. HUD § 3612 charge: civil penalty up to $25,597
//! first violation (inflation-adjusted 2025); $63,991 prior
//! violation within 5 years.
//!
//! Citations: 42 U.S.C. § 3604(f)(3)(C) (Fair Housing Act design
//! and construction requirements); 24 CFR § 100.205(a) (March 13,
//! 1991 cutoff + impracticality defense); § 100.205(c)(1)-(7)
//! (seven requirements); 24 CFR Part 100 Subpart D (Fair Housing
//! Accessibility Guidelines); 42 U.S.C. § 3613 (private
//! enforcement); 42 U.S.C. § 3612 (HUD administrative enforcement);
//! Fair Housing Act Design Manual (HUD).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct FhaDesignConstructionInput {
    /// Whether the building was first occupied AFTER March 13,
    /// 1991. Buildings first occupied on or before this date are
    /// grandfathered and not subject to § 100.205 requirements.
    pub first_occupancy_after_march_13_1991: bool,
    /// Whether the building is a "covered multifamily dwelling" —
    /// 4+ units with an elevator (all units covered) OR ground-
    /// floor units in non-elevator buildings.
    pub covered_multifamily_dwelling: bool,
    /// Requirement 1 — accessible building entrance on accessible
    /// route (or terrain-impracticality defense documented).
    pub accessible_entrance_on_accessible_route: bool,
    /// Terrain or unusual site characteristics make Requirement 1
    /// impractical. Burden on builder per § 100.205(a).
    pub terrain_impracticality_documented: bool,
    /// Requirement 2 — accessible and usable public/common-use
    /// areas (lobbies, mailbox, parking, recreation).
    pub accessible_public_common_use_areas: bool,
    /// Requirement 3 — usable doors (32" clear opening minimum
    /// for wheelchair passage).
    pub usable_doors_for_wheelchair: bool,
    /// Requirement 4 — accessible route INTO AND THROUGH the
    /// covered dwelling unit.
    pub accessible_route_through_dwelling: bool,
    /// Requirement 5 — light switches, electrical outlets,
    /// thermostats, environmental controls in accessible
    /// locations.
    pub environmental_controls_accessible_locations: bool,
    /// Requirement 6 — reinforced walls in bathrooms for later
    /// grab-bar installation.
    pub reinforced_walls_for_grab_bars: bool,
    /// Requirement 7 — usable kitchens and bathrooms permitting
    /// wheelchair maneuverability.
    pub usable_kitchens_and_bathrooms: bool,
    /// Whether this is a repeat violation within 5 years
    /// (drives § 3612 civil penalty tier).
    pub prior_violation_within_5_years: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FhaDesignConstructionResult {
    pub rule_applies: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub max_hud_civil_penalty_dollars: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &FhaDesignConstructionInput) -> FhaDesignConstructionResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.first_occupancy_after_march_13_1991 {
        notes.push(
            "24 CFR § 100.205(a) — dwelling first occupied on or before March 13, 1991 (or last building permit issued on or before June 15, 1990); GRANDFATHERED — § 100.205 design requirements do NOT apply"
                .to_string(),
        );
        return FhaDesignConstructionResult {
            rule_applies: false,
            compliant: true,
            violations,
            max_hud_civil_penalty_dollars: 0,
            citation: citation(),
            notes,
        };
    }

    if !input.covered_multifamily_dwelling {
        notes.push(
            "building does NOT qualify as covered multifamily dwelling — must be 4+ units with elevator (all units) OR ground-floor units in non-elevator buildings; § 100.205 design requirements do not apply"
                .to_string(),
        );
        return FhaDesignConstructionResult {
            rule_applies: false,
            compliant: true,
            violations,
            max_hud_civil_penalty_dollars: 0,
            citation: citation(),
            notes,
        };
    }

    if !input.accessible_entrance_on_accessible_route {
        if input.terrain_impracticality_documented {
            notes.push(
                "§ 100.205(c)(1) — accessible entrance unavailable but terrain/unusual-site impracticality defense documented; burden of proof on designer/builder per § 100.205(a)"
                    .to_string(),
            );
        } else {
            violations.push(
                "§ 100.205(c)(1) Requirement 1 — accessible building entrance on accessible route missing without documented terrain-impracticality defense"
                    .to_string(),
            );
        }
    }

    if !input.accessible_public_common_use_areas {
        violations.push(
            "§ 100.205(c)(2) Requirement 2 — public and common use areas not accessible and usable (lobbies, mailbox areas, parking, recreation facilities)"
                .to_string(),
        );
    }

    if !input.usable_doors_for_wheelchair {
        violations.push(
            "§ 100.205(c)(3) Requirement 3 — doors do not provide sufficient clear opening (typically 32\") for wheelchair passage"
                .to_string(),
        );
    }

    if !input.accessible_route_through_dwelling {
        violations.push(
            "§ 100.205(c)(4) Requirement 4 — no accessible route into and through covered dwelling unit"
                .to_string(),
        );
    }

    if !input.environmental_controls_accessible_locations {
        violations.push(
            "§ 100.205(c)(5) Requirement 5 — light switches, electrical outlets, thermostats, or environmental controls not in accessible locations"
                .to_string(),
        );
    }

    if !input.reinforced_walls_for_grab_bars {
        violations.push(
            "§ 100.205(c)(6) Requirement 6 — bathroom walls lack reinforcement for later grab-bar installation"
                .to_string(),
        );
    }

    if !input.usable_kitchens_and_bathrooms {
        violations.push(
            "§ 100.205(c)(7) Requirement 7 — kitchens or bathrooms lack wheelchair maneuverability"
                .to_string(),
        );
    }

    let civil_penalty = if input.prior_violation_within_5_years {
        63_991
    } else {
        25_597
    };

    if !violations.is_empty() {
        notes.push(format!(
            "42 U.S.C. § 3612 HUD civil penalty up to ${} ({}); 42 U.S.C. § 3613(c) private suit damages — actual + punitive + attorney fees",
            civil_penalty,
            if input.prior_violation_within_5_years {
                "prior violation within 5 years"
            } else {
                "first violation"
            }
        ));
    }

    notes.push(
        "FHA design and construction is a FEDERAL FLOOR — state codes (CA Title 24, MA AAB, NY HRL, etc.) may add stricter requirements but cannot reduce baseline"
            .to_string(),
    );

    let violations_empty = violations.is_empty();
    FhaDesignConstructionResult {
        rule_applies: true,
        compliant: violations_empty,
        violations,
        max_hud_civil_penalty_dollars: if violations_empty { 0 } else { civil_penalty },
        citation: citation(),
        notes,
    }
}

fn citation() -> &'static str {
    "42 U.S.C. § 3604(f)(3)(C); 24 CFR § 100.205(a)/(c)(1)-(7); 24 CFR Part 100 Subpart D; 42 U.S.C. § 3613(c); 42 U.S.C. § 3612; Fair Housing Act Design Manual (HUD)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_compliance() -> FhaDesignConstructionInput {
        FhaDesignConstructionInput {
            first_occupancy_after_march_13_1991: true,
            covered_multifamily_dwelling: true,
            accessible_entrance_on_accessible_route: true,
            terrain_impracticality_documented: false,
            accessible_public_common_use_areas: true,
            usable_doors_for_wheelchair: true,
            accessible_route_through_dwelling: true,
            environmental_controls_accessible_locations: true,
            reinforced_walls_for_grab_bars: true,
            usable_kitchens_and_bathrooms: true,
            prior_violation_within_5_years: false,
        }
    }

    #[test]
    fn full_compliance_passes() {
        let r = check(&full_compliance());
        assert!(r.rule_applies);
        assert!(r.compliant);
        assert!(r.violations.is_empty());
        assert_eq!(r.max_hud_civil_penalty_dollars, 0);
    }

    #[test]
    fn pre_march_13_1991_grandfathered() {
        let mut i = full_compliance();
        i.first_occupancy_after_march_13_1991 = false;
        i.accessible_entrance_on_accessible_route = false;
        let r = check(&i);
        assert!(!r.rule_applies);
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("GRANDFATHERED")));
    }

    #[test]
    fn non_covered_multifamily_dwelling_rule_does_not_apply() {
        let mut i = full_compliance();
        i.covered_multifamily_dwelling = false;
        let r = check(&i);
        assert!(!r.rule_applies);
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("does NOT qualify as covered multifamily")));
    }

    #[test]
    fn missing_requirement_1_without_impracticality_violation() {
        let mut i = full_compliance();
        i.accessible_entrance_on_accessible_route = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 100.205(c)(1)")));
    }

    #[test]
    fn missing_requirement_1_with_terrain_impracticality_no_violation() {
        let mut i = full_compliance();
        i.accessible_entrance_on_accessible_route = false;
        i.terrain_impracticality_documented = true;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("terrain/unusual-site impracticality defense")));
    }

    #[test]
    fn missing_requirement_2_public_common_areas_violation() {
        let mut i = full_compliance();
        i.accessible_public_common_use_areas = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 100.205(c)(2)") && v.contains("Requirement 2")));
    }

    #[test]
    fn missing_requirement_3_usable_doors_violation() {
        let mut i = full_compliance();
        i.usable_doors_for_wheelchair = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 100.205(c)(3)") && v.contains("wheelchair passage")));
    }

    #[test]
    fn missing_requirement_4_accessible_route_violation() {
        let mut i = full_compliance();
        i.accessible_route_through_dwelling = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 100.205(c)(4)")));
    }

    #[test]
    fn missing_requirement_5_environmental_controls_violation() {
        let mut i = full_compliance();
        i.environmental_controls_accessible_locations = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 100.205(c)(5)") && v.contains("thermostats")));
    }

    #[test]
    fn missing_requirement_6_reinforced_walls_violation() {
        let mut i = full_compliance();
        i.reinforced_walls_for_grab_bars = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 100.205(c)(6)") && v.contains("grab-bar")));
    }

    #[test]
    fn missing_requirement_7_kitchens_bathrooms_violation() {
        let mut i = full_compliance();
        i.usable_kitchens_and_bathrooms = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 100.205(c)(7)") && v.contains("wheelchair maneuverability")));
    }

    #[test]
    fn first_violation_penalty_25597() {
        let mut i = full_compliance();
        i.usable_doors_for_wheelchair = false;
        i.prior_violation_within_5_years = false;
        let r = check(&i);
        assert_eq!(r.max_hud_civil_penalty_dollars, 25_597);
    }

    #[test]
    fn prior_violation_within_5_years_penalty_63991() {
        let mut i = full_compliance();
        i.usable_doors_for_wheelchair = false;
        i.prior_violation_within_5_years = true;
        let r = check(&i);
        assert_eq!(r.max_hud_civil_penalty_dollars, 63_991);
    }

    #[test]
    fn private_suit_remedies_in_violation_note() {
        let mut i = full_compliance();
        i.usable_doors_for_wheelchair = false;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("42 U.S.C. § 3613(c)") && n.contains("actual + punitive + attorney fees")));
    }

    #[test]
    fn all_seven_requirements_must_be_satisfied_invariant() {
        let base = full_compliance();
        let breakers: [&dyn Fn(&mut FhaDesignConstructionInput); 7] = [
            &|i| i.accessible_entrance_on_accessible_route = false,
            &|i| i.accessible_public_common_use_areas = false,
            &|i| i.usable_doors_for_wheelchair = false,
            &|i| i.accessible_route_through_dwelling = false,
            &|i| i.environmental_controls_accessible_locations = false,
            &|i| i.reinforced_walls_for_grab_bars = false,
            &|i| i.usable_kitchens_and_bathrooms = false,
        ];
        for break_fn in breakers.iter() {
            let mut i = base.clone();
            break_fn(&mut i);
            let r = check(&i);
            assert!(!r.compliant, "single requirement gap should violate compliance");
        }
    }

    #[test]
    fn multiple_violations_accumulate() {
        let mut i = full_compliance();
        i.usable_doors_for_wheelchair = false;
        i.reinforced_walls_for_grab_bars = false;
        i.usable_kitchens_and_bathrooms = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 3);
    }

    #[test]
    fn federal_floor_note_always_present() {
        let r = check(&full_compliance());
        assert!(r.notes.iter().any(|n| n.contains("FEDERAL FLOOR") && n.contains("state codes")));
    }

    #[test]
    fn citation_pins_subsections_and_authorities() {
        let r = check(&full_compliance());
        assert!(r.citation.contains("42 U.S.C. § 3604(f)(3)(C)"));
        assert!(r.citation.contains("24 CFR § 100.205(a)"));
        assert!(r.citation.contains("(c)(1)-(7)"));
        assert!(r.citation.contains("24 CFR Part 100 Subpart D"));
        assert!(r.citation.contains("42 U.S.C. § 3613(c)"));
        assert!(r.citation.contains("42 U.S.C. § 3612"));
        assert!(r.citation.contains("Fair Housing Act Design Manual"));
    }

    #[test]
    fn rule_applies_only_when_post_march_13_1991_and_covered() {
        let mut grandfathered = full_compliance();
        grandfathered.first_occupancy_after_march_13_1991 = false;
        assert!(!check(&grandfathered).rule_applies);

        let mut not_covered = full_compliance();
        not_covered.covered_multifamily_dwelling = false;
        assert!(!check(&not_covered).rule_applies);

        let r_compliant = check(&full_compliance());
        assert!(r_compliant.rule_applies);
    }

    #[test]
    fn note_describes_civil_penalty_for_first_violation() {
        let mut i = full_compliance();
        i.usable_doors_for_wheelchair = false;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("$25597") || n.contains("$25,597")));
    }

    #[test]
    fn note_describes_civil_penalty_for_prior_violation() {
        let mut i = full_compliance();
        i.usable_doors_for_wheelchair = false;
        i.prior_violation_within_5_years = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("$63991") || n.contains("$63,991")));
    }

    #[test]
    fn no_civil_penalty_when_compliant() {
        let r = check(&full_compliance());
        assert_eq!(r.max_hud_civil_penalty_dollars, 0);
    }

    #[test]
    fn impracticality_defense_does_not_extend_to_other_requirements() {
        let mut i = full_compliance();
        i.accessible_entrance_on_accessible_route = false;
        i.terrain_impracticality_documented = true;
        i.usable_doors_for_wheelchair = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 1);
        assert!(r.violations.iter().any(|v| v.contains("§ 100.205(c)(3)")));
    }

    #[test]
    fn all_seven_violations_accumulate_when_nothing_satisfied() {
        let i = FhaDesignConstructionInput {
            first_occupancy_after_march_13_1991: true,
            covered_multifamily_dwelling: true,
            accessible_entrance_on_accessible_route: false,
            terrain_impracticality_documented: false,
            accessible_public_common_use_areas: false,
            usable_doors_for_wheelchair: false,
            accessible_route_through_dwelling: false,
            environmental_controls_accessible_locations: false,
            reinforced_walls_for_grab_bars: false,
            usable_kitchens_and_bathrooms: false,
            prior_violation_within_5_years: false,
        };
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 7);
    }

    #[test]
    fn impracticality_carveout_only_applies_to_requirement_1() {
        let mut i = full_compliance();
        i.usable_doors_for_wheelchair = false;
        i.terrain_impracticality_documented = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 100.205(c)(3)")));
    }

    #[test]
    fn pre_march_13_1991_with_full_violations_remains_compliant() {
        let i = FhaDesignConstructionInput {
            first_occupancy_after_march_13_1991: false,
            covered_multifamily_dwelling: true,
            accessible_entrance_on_accessible_route: false,
            terrain_impracticality_documented: false,
            accessible_public_common_use_areas: false,
            usable_doors_for_wheelchair: false,
            accessible_route_through_dwelling: false,
            environmental_controls_accessible_locations: false,
            reinforced_walls_for_grab_bars: false,
            usable_kitchens_and_bathrooms: false,
            prior_violation_within_5_years: false,
        };
        let r = check(&i);
        assert!(r.compliant, "grandfathered building immune from compliance check");
        assert!(r.violations.is_empty());
    }
}

