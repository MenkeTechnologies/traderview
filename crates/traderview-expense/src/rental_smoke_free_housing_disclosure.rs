//! Multi-jurisdictional residential rental smoke-free
//! housing disclosure framework. Trader-landlord critical
//! because (1) modern shift toward smoke-free properties
//! for insurance discounts + tenant health protection has
//! made smoke-free policies a routine landlord choice; (2)
//! failure to disclose smoking policy in lease can expose
//! landlord to constructive-eviction claims from
//! secondhand-smoke-affected tenants; (3) misclassifying
//! tobacco-product coverage (cigarettes only vs. cigars
//! and pipes and e-cigarettes) creates lease enforcement
//! disputes; (4) HUD public housing smoke-free rule
//! (effective July 31, 2018) applies to all federally-
//! funded units, including LIHTC trader-landlord
//! investments.
//!
//! Companion to rental_carbon_monoxide_detector,
//! rental_pesticide_application_notification,
//! tenant_data_privacy,
//! landlord_emergency_entry_notice.
//!
//! **California Civ. Code § 1947.5** (SB 332 of 2011,
//! **effective January 1, 2012**) — landlord MAY prohibit
//! smoking of cigarettes or other tobacco products on the
//! property OR in any building or portion of the building,
//! INCLUDING any dwelling unit, other interior or exterior
//! areas, OR the premises on which it is located.
//!
//! § 1947.5(b) **lease disclosure requirement** — every
//! lease or rental agreement entered into on or after
//! January 1, 2012, for a residential dwelling unit on
//! property where landlord has prohibited smoking MUST
//! INCLUDE A PROVISION that specifies the areas on the
//! property where smoking is prohibited, IF the lessee has
//! NOT previously occupied the dwelling unit.
//!
//! § 1947.5(c) **change-of-terms procedure for pre-2012
//! leases** — for leases EXECUTED BEFORE January 1, 2012,
//! a prohibition against smoking in any portion of the
//! property in which smoking was previously permitted
//! constitutes a **CHANGE OF TERMS OF TENANCY** requiring
//! ADEQUATE NOTICE IN WRITING (per Cal. Civ. Code § 827).
//!
//! § 1947.5 covers cigarettes + cigars + pipes + other
//! "tobacco products" as defined in Cal. Bus. & Prof. Code
//! § 22950.5; e-cigarettes added effective 2017 per SB 5.
//!
//! **HUD 24 CFR § 965.653 + § 966 Subpart G — Smoke-Free
//! Public Housing Rule** (final rule November 30, 2016 at
//! 81 Fed. Reg. 87430; **mandatory implementation deadline
//! July 31, 2018**):
//!
//! § 965.653(a) PHAs MUST design and implement a policy
//! prohibiting use of prohibited tobacco products in ALL:
//! 1. **Public housing living units**;
//! 2. **Interior areas** (including hallways, rental and
//!    administrative offices, community centers, daycare
//!    centers, laundry centers, and similar structures);
//! 3. **Outdoor areas within 25 FEET** of public housing
//!    and administrative office buildings.
//!
//! § 965.653(b) PHAs MAY (1) limit smoking to designated
//! smoking areas outside restricted 25-foot zone; OR (2)
//! create additional smoke-free areas; OR (3) make the
//! ENTIRE grounds smoke-free.
//!
//! "Prohibited tobacco products" defined in § 965.651:
//! cigarettes + cigars + pipes (e-cigarettes/ENDS NOT
//! mandated by federal rule but PHAs may add by local
//! policy).
//!
//! § 966 Subpart G enforcement guidance — single incident
//! of smoking in violation of policy is NOT GROUNDS FOR
//! EVICTION or termination of assistance; "compassionate
//! enforcement" approach required.
//!
//! **New York Multiple Dwelling Law § 17 + § 17-101 +
//! § 17-179** — buildings of **3+ DWELLING UNITS** must
//! adopt and disclose a written smoking policy specifying:
//! 1. Whether smoking is permitted, prohibited, or
//!    restricted to specific areas;
//! 2. Common areas where smoking is prohibited
//!    (mandatory under § 1399-n of NY Public Health Law);
//! 3. Outdoor areas where smoking is prohibited.
//!
//! Policy must be DISCLOSED IN LEASE OR PROVIDED IN
//! WRITING to tenant before tenancy begins; failure
//! creates per-unit civil penalty + tenant constructive-
//! eviction remedy.
//!
//! **Massachusetts G.L. c. 270 § 22 + § 22A — Smoke-Free
//! Workplace Law** — common areas of multifamily buildings
//! that constitute workplaces (lobbies, hallways,
//! maintenance areas) are smoke-free; landlord must post
//! signs. Boston Public Health Commission Tobacco Control
//! Regulations add municipal overlays.
//!
//! **Default — common-law nuisance + breach of warranty
//! of habitability** — tenant subjected to severe
//! secondhand smoke may bring common-law nuisance or
//! warranty-of-habitability claim regardless of statutory
//! disclosure regime.
//!
//! Citations: Cal. Civ. Code § 1947.5 (SB 332 of 2011);
//! Cal. Bus. & Prof. Code § 22950.5; Cal. Civ. Code § 827;
//! HUD 24 CFR § 965.651 to § 965.655; HUD 24 CFR Part 965
//! Subpart G; HUD 24 CFR Part 966 Subpart G; 81 Fed. Reg.
//! 87430 (December 5, 2016); NY MDL § 17; NY Public Health
//! Law § 1399-n; Mass. G.L. c. 270 § 22 and § 22A.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    HudPublicHousing,
    NewYork,
    Massachusetts,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalSmokeFreeHousingDisclosureInput {
    pub jurisdiction: Jurisdiction,
    /// Whether landlord has adopted a smoke-free policy
    /// (or restricted-smoking policy).
    pub smoke_free_policy_adopted: bool,
    /// Lease execution year (CA: ≥ 2012 requires
    /// in-lease disclosure; < 2012 requires § 827
    /// change-of-terms notice).
    pub lease_year: u32,
    /// Lease execution month.
    pub lease_month: u32,
    /// Whether lessee previously occupied the dwelling
    /// unit (CA § 1947.5(b) carveout).
    pub lessee_previously_occupied: bool,
    /// Whether lease/rental agreement contains a
    /// provision specifying areas where smoking is
    /// prohibited.
    pub lease_specifies_prohibited_areas: bool,
    /// Whether written change-of-terms notice was
    /// provided per Cal. Civ. Code § 827 (pre-2012
    /// leases).
    pub written_change_of_terms_notice_provided: bool,
    /// Number of dwelling units in building (NY MDL 3+
    /// unit threshold).
    pub dwelling_unit_count: u32,
    /// Whether HUD-funded public housing unit (engages
    /// 24 CFR § 965.653 mandatory framework).
    pub hud_funded_public_housing: bool,
    /// Whether interior/common-area smoking prohibition
    /// implemented per HUD § 965.653(a).
    pub interior_common_area_prohibition_implemented: bool,
    /// Whether 25-foot outdoor perimeter restriction
    /// implemented per HUD § 965.653(a)(3).
    pub twenty_five_foot_perimeter_restriction_implemented: bool,
    /// Whether NY MDL § 17 written smoking policy
    /// disclosed in lease.
    pub ny_mdl_written_policy_disclosed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalSmokeFreeHousingDisclosureResult {
    pub jurisdiction: Jurisdiction,
    pub disclosure_obligation_triggered: bool,
    pub disclosure_compliant: bool,
    pub hud_twenty_five_foot_perimeter_engaged: bool,
    pub change_of_terms_procedure_required: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentalSmokeFreeHousingDisclosureInput,
) -> RentalSmokeFreeHousingDisclosureResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let (
        disclosure_obligation_triggered,
        disclosure_compliant,
        hud_twenty_five_foot_perimeter_engaged,
        change_of_terms_procedure_required,
    ) = match input.jurisdiction {
        Jurisdiction::California => {
            let post_2012 = input.lease_year > 2012
                || (input.lease_year == 2012 && input.lease_month >= 1);
            let obligation_triggered = input.smoke_free_policy_adopted
                && post_2012
                && !input.lessee_previously_occupied;
            let mut compliant = true;
            if obligation_triggered && !input.lease_specifies_prohibited_areas {
                compliant = false;
                failure_reasons.push(
                    "Cal. Civ. Code § 1947.5(b) — every residential lease entered into on or after January 1, 2012, on property where landlord has prohibited smoking MUST INCLUDE A PROVISION specifying the areas where smoking is prohibited (carveout: lessee previously occupied the dwelling unit)".to_string(),
                );
            }
            let change_required = input.smoke_free_policy_adopted && !post_2012;
            if change_required && !input.written_change_of_terms_notice_provided {
                compliant = false;
                failure_reasons.push(
                    "Cal. Civ. Code § 1947.5(c) + § 827 — for leases EXECUTED BEFORE January 1, 2012, prohibition against smoking in property where smoking was previously permitted constitutes CHANGE OF TERMS OF TENANCY requiring ADEQUATE NOTICE IN WRITING".to_string(),
                );
            }
            (obligation_triggered, compliant, false, change_required)
        }
        Jurisdiction::HudPublicHousing => {
            let obligation_triggered = input.hud_funded_public_housing;
            let mut compliant = true;
            if obligation_triggered && !input.interior_common_area_prohibition_implemented {
                compliant = false;
                failure_reasons.push(
                    "HUD 24 CFR § 965.653(a) — PHAs MUST design and implement policy prohibiting prohibited tobacco products in ALL public housing living units AND interior areas (hallways, rental/admin offices, community centers, daycare, laundry)".to_string(),
                );
            }
            let perimeter_engaged = obligation_triggered
                && input.twenty_five_foot_perimeter_restriction_implemented;
            if obligation_triggered && !input.twenty_five_foot_perimeter_restriction_implemented {
                compliant = false;
                failure_reasons.push(
                    "HUD 24 CFR § 965.653(a)(3) — PHAs MUST prohibit smoking in OUTDOOR AREAS WITHIN 25 FEET of public housing and administrative office buildings; 25-foot perimeter prevents secondhand smoke from entering open windows in lower-level units and exposing tenants on lower-floor balconies/porches".to_string(),
                );
            }
            (obligation_triggered, compliant, perimeter_engaged, false)
        }
        Jurisdiction::NewYork => {
            let obligation_triggered = input.dwelling_unit_count >= 3;
            let mut compliant = true;
            if obligation_triggered && !input.ny_mdl_written_policy_disclosed {
                compliant = false;
                failure_reasons.push(
                    "NY Multiple Dwelling Law § 17 + § 17-101 + § 17-179 — buildings with 3+ DWELLING UNITS must adopt and DISCLOSE a written smoking policy specifying (1) whether smoking is permitted/prohibited/restricted; (2) common areas where smoking is prohibited per NY Public Health Law § 1399-n; (3) outdoor areas where smoking is prohibited".to_string(),
                );
            }
            (obligation_triggered, compliant, false, false)
        }
        Jurisdiction::Massachusetts => {
            let obligation_triggered = input.dwelling_unit_count >= 2;
            let compliant = !obligation_triggered || input.lease_specifies_prohibited_areas;
            if obligation_triggered && !compliant {
                failure_reasons.push(
                    "Mass. G.L. c. 270 § 22 + § 22A — Smoke-Free Workplace Law: common areas of multifamily buildings constituting workplaces (lobbies, hallways, maintenance areas) are smoke-free; landlord must POST SIGNS in common areas; Boston Public Health Commission Tobacco Control Regulations add municipal overlays".to_string(),
                );
            }
            (obligation_triggered, compliant, false, false)
        }
        Jurisdiction::Default => {
            let obligation_triggered = false;
            let compliant = true;
            (obligation_triggered, compliant, false, false)
        }
    };

    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1947.5(a) (SB 332 of 2011, effective January 1, 2012) — landlord MAY prohibit smoking of cigarettes or other tobacco products on the property OR in any building or portion of the building, INCLUDING any dwelling unit, other interior or exterior areas, OR the premises on which it is located".to_string(),
        "Cal. Civ. Code § 1947.5(b) — every residential lease entered into on or after January 1, 2012, on property where landlord has prohibited smoking MUST INCLUDE A PROVISION specifying the areas where smoking is prohibited; carveout if lessee previously occupied the dwelling unit".to_string(),
        "Cal. Civ. Code § 1947.5(c) + Cal. Civ. Code § 827 — for leases EXECUTED BEFORE January 1, 2012, prohibition against smoking in property where smoking was previously permitted constitutes CHANGE OF TERMS OF TENANCY requiring ADEQUATE NOTICE IN WRITING".to_string(),
        "Cal. Civ. Code § 1947.5 covers cigarettes + cigars + pipes + other tobacco products as defined in Cal. Bus. & Prof. Code § 22950.5; e-cigarettes added effective 2017 per SB 5".to_string(),
        "HUD 24 CFR § 965.653(a) Smoke-Free Public Housing Rule (final rule November 30, 2016 at 81 Fed. Reg. 87430; mandatory implementation deadline July 31, 2018) — PHAs MUST design and implement policy prohibiting use of prohibited tobacco products in ALL (1) public housing living units; (2) interior areas; (3) outdoor areas within 25 FEET of buildings".to_string(),
        "HUD 24 CFR § 965.653(a)(3) — 25-FOOT OUTDOOR PERIMETER restriction prevents secondhand smoke from entering open windows in lower-level units and exposing tenants on lower-floor balconies/porches".to_string(),
        "HUD 24 CFR § 965.653(b) — PHAs MAY (1) limit smoking to designated smoking areas outside the restricted 25-foot zone; OR (2) create additional smoke-free areas; OR (3) make the ENTIRE grounds smoke-free".to_string(),
        "HUD 24 CFR § 965.651 'prohibited tobacco products' = cigarettes + cigars + pipes (e-cigarettes/ENDS NOT mandated by federal rule but PHAs may add by local policy)".to_string(),
        "HUD 24 CFR Part 966 Subpart G — single incident of smoking in violation of policy is NOT GROUNDS FOR EVICTION or termination of assistance; 'compassionate enforcement' approach required per HUD Guidebook".to_string(),
        "NY Multiple Dwelling Law § 17 + § 17-101 + § 17-179 — buildings of 3+ DWELLING UNITS must adopt and DISCLOSE a written smoking policy specifying (1) whether smoking is permitted/prohibited/restricted to specific areas; (2) common areas where smoking is prohibited under NY Public Health Law § 1399-n; (3) outdoor areas where smoking is prohibited".to_string(),
        "Mass. G.L. c. 270 § 22 + § 22A — Smoke-Free Workplace Law: common areas of multifamily buildings constituting workplaces (lobbies, hallways, maintenance areas) are smoke-free; landlord must POST SIGNS in common areas; Boston Public Health Commission Tobacco Control Regulations add municipal overlays".to_string(),
        "Default — common-law nuisance + breach of warranty of habitability; tenant subjected to severe secondhand smoke may bring common-law nuisance OR warranty-of-habitability claim regardless of statutory disclosure regime".to_string(),
    ];

    RentalSmokeFreeHousingDisclosureResult {
        jurisdiction: input.jurisdiction,
        disclosure_obligation_triggered,
        disclosure_compliant,
        hud_twenty_five_foot_perimeter_engaged,
        change_of_terms_procedure_required,
        failure_reasons,
        citation: "Cal. Civ. Code § 1947.5 (SB 332 of 2011, effective January 1, 2012); Cal. Bus. & Prof. Code § 22950.5; Cal. Civ. Code § 827; HUD 24 CFR § 965.651 to § 965.655; HUD 24 CFR Part 965 Subpart G; HUD 24 CFR Part 966 Subpart G; 81 Fed. Reg. 87430 (December 5, 2016); NY MDL § 17 and § 17-101 and § 17-179; NY Public Health Law § 1399-n; Mass. G.L. c. 270 § 22 and § 22A",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> RentalSmokeFreeHousingDisclosureInput {
        RentalSmokeFreeHousingDisclosureInput {
            jurisdiction: Jurisdiction::California,
            smoke_free_policy_adopted: true,
            lease_year: 2026,
            lease_month: 6,
            lessee_previously_occupied: false,
            lease_specifies_prohibited_areas: true,
            written_change_of_terms_notice_provided: false,
            dwelling_unit_count: 8,
            hud_funded_public_housing: false,
            interior_common_area_prohibition_implemented: false,
            twenty_five_foot_perimeter_restriction_implemented: false,
            ny_mdl_written_policy_disclosed: false,
        }
    }

    #[test]
    fn ca_post_2012_smoke_free_with_lease_provision_compliant() {
        let r = check(&ca_compliant());
        assert!(r.disclosure_obligation_triggered);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn ca_post_2012_smoke_free_no_lease_provision_violation() {
        let mut i = ca_compliant();
        i.lease_specifies_prohibited_areas = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1947.5(b)")
            && f.contains("January 1, 2012")));
    }

    #[test]
    fn ca_lessee_previously_occupied_no_obligation() {
        let mut i = ca_compliant();
        i.lessee_previously_occupied = true;
        i.lease_specifies_prohibited_areas = false;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn ca_pre_2012_lease_requires_change_of_terms_notice() {
        let mut i = ca_compliant();
        i.lease_year = 2011;
        i.lease_month = 12;
        i.written_change_of_terms_notice_provided = false;
        let r = check(&i);
        assert!(r.change_of_terms_procedure_required);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1947.5(c)")
            && f.contains("§ 827")
            && f.contains("CHANGE OF TERMS OF TENANCY")));
    }

    #[test]
    fn ca_pre_2012_lease_with_change_of_terms_notice_compliant() {
        let mut i = ca_compliant();
        i.lease_year = 2011;
        i.lease_month = 12;
        i.written_change_of_terms_notice_provided = true;
        let r = check(&i);
        assert!(r.change_of_terms_procedure_required);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn ca_no_policy_no_obligation() {
        let mut i = ca_compliant();
        i.smoke_free_policy_adopted = false;
        i.lease_specifies_prohibited_areas = false;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
    }

    #[test]
    fn hud_public_housing_all_three_prongs_compliant() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::HudPublicHousing;
        i.hud_funded_public_housing = true;
        i.interior_common_area_prohibition_implemented = true;
        i.twenty_five_foot_perimeter_restriction_implemented = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(r.hud_twenty_five_foot_perimeter_engaged);
    }

    #[test]
    fn hud_missing_25_foot_perimeter_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::HudPublicHousing;
        i.hud_funded_public_housing = true;
        i.interior_common_area_prohibition_implemented = true;
        i.twenty_five_foot_perimeter_restriction_implemented = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 965.653(a)(3)")
            && f.contains("25 FEET")));
    }

    #[test]
    fn hud_missing_interior_prohibition_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::HudPublicHousing;
        i.hud_funded_public_housing = true;
        i.interior_common_area_prohibition_implemented = false;
        i.twenty_five_foot_perimeter_restriction_implemented = true;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 965.653(a)")
            && f.contains("interior areas")));
    }

    #[test]
    fn hud_non_hud_unit_no_obligation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::HudPublicHousing;
        i.hud_funded_public_housing = false;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
    }

    #[test]
    fn ny_3_unit_building_triggers_obligation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.dwelling_unit_count = 3;
        i.ny_mdl_written_policy_disclosed = true;
        let r = check(&i);
        assert!(r.disclosure_obligation_triggered);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn ny_2_unit_building_no_obligation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.dwelling_unit_count = 2;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
    }

    #[test]
    fn ny_no_written_policy_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.dwelling_unit_count = 10;
        i.ny_mdl_written_policy_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("NY Multiple Dwelling Law § 17")
            && f.contains("3+ DWELLING UNITS")
            && f.contains("§ 1399-n")));
    }

    #[test]
    fn ma_2_unit_building_engages_workplace_law() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.dwelling_unit_count = 2;
        i.lease_specifies_prohibited_areas = true;
        let r = check(&i);
        assert!(r.disclosure_obligation_triggered);
    }

    #[test]
    fn ma_single_family_no_obligation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.dwelling_unit_count = 1;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
    }

    #[test]
    fn ma_no_common_area_signs_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.dwelling_unit_count = 4;
        i.lease_specifies_prohibited_areas = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("c. 270 § 22")
            && f.contains("Smoke-Free Workplace Law")
            && f.contains("POST SIGNS")));
    }

    #[test]
    fn default_jurisdiction_no_obligation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(!r.disclosure_obligation_triggered);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn jurisdiction_truth_table_five_cells() {
        for jur in [
            Jurisdiction::California,
            Jurisdiction::HudPublicHousing,
            Jurisdiction::NewYork,
            Jurisdiction::Massachusetts,
            Jurisdiction::Default,
        ] {
            let mut i = ca_compliant();
            i.jurisdiction = jur;
            let r = check(&i);
            assert_eq!(r.jurisdiction, jur);
        }
    }

    #[test]
    fn hud_uniquely_engages_25_foot_perimeter_invariant() {
        let mut hud = ca_compliant();
        hud.jurisdiction = Jurisdiction::HudPublicHousing;
        hud.hud_funded_public_housing = true;
        hud.interior_common_area_prohibition_implemented = true;
        hud.twenty_five_foot_perimeter_restriction_implemented = true;
        let r_hud = check(&hud);
        assert!(r_hud.hud_twenty_five_foot_perimeter_engaged);

        for jur in [
            Jurisdiction::California,
            Jurisdiction::NewYork,
            Jurisdiction::Massachusetts,
            Jurisdiction::Default,
        ] {
            let mut i = ca_compliant();
            i.jurisdiction = jur;
            let r = check(&i);
            assert!(
                !r.hud_twenty_five_foot_perimeter_engaged,
                "jur={:?}",
                jur
            );
        }
    }

    #[test]
    fn ca_uniquely_engages_change_of_terms_procedure_invariant() {
        let mut ca_pre = ca_compliant();
        ca_pre.lease_year = 2011;
        let r_pre = check(&ca_pre);
        assert!(r_pre.change_of_terms_procedure_required);

        for jur in [
            Jurisdiction::HudPublicHousing,
            Jurisdiction::NewYork,
            Jurisdiction::Massachusetts,
            Jurisdiction::Default,
        ] {
            let mut i = ca_compliant();
            i.jurisdiction = jur;
            i.lease_year = 2011;
            let r = check(&i);
            assert!(!r.change_of_terms_procedure_required, "jur={:?}", jur);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("Cal. Civ. Code § 1947.5"));
        assert!(r.citation.contains("SB 332 of 2011"));
        assert!(r.citation.contains("January 1, 2012"));
        assert!(r.citation.contains("Cal. Bus. & Prof. Code § 22950.5"));
        assert!(r.citation.contains("Cal. Civ. Code § 827"));
        assert!(r.citation.contains("HUD 24 CFR § 965.651 to § 965.655"));
        assert!(r.citation.contains("Part 965 Subpart G"));
        assert!(r.citation.contains("Part 966 Subpart G"));
        assert!(r.citation.contains("81 Fed. Reg. 87430"));
        assert!(r.citation.contains("NY MDL § 17 and § 17-101 and § 17-179"));
        assert!(r.citation.contains("NY Public Health Law § 1399-n"));
        assert!(r.citation.contains("Mass. G.L. c. 270 § 22 and § 22A"));
    }

    #[test]
    fn note_pins_ca_landlord_authority() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.5(a)")
            && n.contains("SB 332 of 2011")
            && n.contains("January 1, 2012")
            && n.contains("dwelling unit")));
    }

    #[test]
    fn note_pins_ca_lease_disclosure() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.5(b)")
            && n.contains("January 1, 2012")
            && n.contains("MUST INCLUDE A PROVISION")
            && n.contains("carveout if lessee previously occupied")));
    }

    #[test]
    fn note_pins_ca_change_of_terms() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.5(c)")
            && n.contains("§ 827")
            && n.contains("CHANGE OF TERMS OF TENANCY")));
    }

    #[test]
    fn note_pins_ca_e_cigarette_2017_sb5() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.5")
            && n.contains("e-cigarettes added effective 2017 per SB 5")));
    }

    #[test]
    fn note_pins_hud_three_prong_framework() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 965.653(a)")
            && n.contains("July 31, 2018")
            && n.contains("public housing living units")
            && n.contains("interior areas")
            && n.contains("25 FEET")));
    }

    #[test]
    fn note_pins_hud_25_foot_perimeter_rationale() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 965.653(a)(3)")
            && n.contains("25-FOOT OUTDOOR PERIMETER")
            && n.contains("lower-level units")));
    }

    #[test]
    fn note_pins_hud_pha_flexibility() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 965.653(b)")
            && n.contains("designated smoking areas")
            && n.contains("ENTIRE grounds smoke-free")));
    }

    #[test]
    fn note_pins_hud_prohibited_tobacco_definition() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 965.651")
            && n.contains("cigarettes + cigars + pipes")
            && n.contains("e-cigarettes/ENDS NOT mandated")));
    }

    #[test]
    fn note_pins_hud_compassionate_enforcement() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Part 966 Subpart G")
            && n.contains("NOT GROUNDS FOR EVICTION")
            && n.contains("compassionate enforcement")));
    }

    #[test]
    fn note_pins_ny_mdl_3_unit_threshold() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("NY Multiple Dwelling Law § 17")
            && n.contains("3+ DWELLING UNITS")
            && n.contains("written smoking policy")
            && n.contains("§ 1399-n")));
    }

    #[test]
    fn note_pins_ma_smoke_free_workplace_law() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("c. 270 § 22")
            && n.contains("Smoke-Free Workplace Law")
            && n.contains("POST SIGNS")
            && n.contains("Boston Public Health Commission")));
    }

    #[test]
    fn note_pins_default_nuisance_habitability() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Default")
            && n.contains("common-law nuisance")
            && n.contains("warranty of habitability")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::HudPublicHousing;
        i.hud_funded_public_housing = true;
        i.interior_common_area_prohibition_implemented = false;
        i.twenty_five_foot_perimeter_restriction_implemented = false;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 2);
    }
}
