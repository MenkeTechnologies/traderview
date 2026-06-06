//! Rental property tenant satellite dish installation right
//! compliance — when a trader-landlord may or may not
//! restrict tenant installation of a satellite dish or
//! over-the-air antenna for video programming reception.
//! Trader-landlord operational concern: federal FCC OTARD
//! Rule (47 CFR § 1.4000) PREEMPTS state law, local
//! ordinances, building codes, HOA rules, AND lease
//! provisions that impair installation on areas under
//! tenant's EXCLUSIVE USE OR CONTROL; per-violation FCC
//! Petition for Declaratory Ruling exposure + tenant
//! injunctive relief. Distinct from siblings `rental_
//! broadband_mte_rules` (cable/broadband building access —
//! already shipped), `rental_carbon_monoxide_detector`,
//! `tenant_data_privacy`.
//!
//! **§ 1.4000(a)(1)(i)-(iii) Covered antennas** — federal
//! preemption applies to three classes of OTARD antennas:
//! 1. **Direct broadcast satellite (DBS) dish antennas**
//!    1 meter or less in diameter (or any size in Alaska)
//!    designed to receive video programming via DBS
//!    service.
//! 2. **Broadband radio service (BRS, formerly MMDS)
//!    antennas** 1 meter or less in diameter.
//! 3. **Television broadcast antennas** designed to receive
//!    local television broadcasts (no size limit).
//!
//! **Exclusive use or control gate** — preemption ONLY
//! applies to installation on property within tenant's
//! **EXCLUSIVE USE OR CONTROL** (balcony, patio, garden
//! area assigned to tenant). Landlord MAY prohibit
//! installation on **COMMON or RESTRICTED areas** —
//! exterior walls, roof, common-area parking, shared
//! corridors. Critical doctrinal line between protected
//! and unprotected installation locations.
//!
//! **§ 1.4000(a)(3) Permissible restrictions** — narrowly
//! tailored restrictions permitted ONLY if **NECESSARY
//! to**:
//! 1. **Accomplish a clearly defined, legitimate safety
//!    objective**; OR
//! 2. **Preserve a historic district designated under
//!    federal, state, or local law**;
//!
//! AND no more burdensome than necessary; AND landlord
//! shows reasonable factual basis for restriction.
//!
//! **§ 1.4000(d) Federal preemption** — preempts state
//! statutes, local zoning codes, building codes, HOA
//! covenants, condominium declarations, AND lease
//! provisions. Preemption applies retroactively to existing
//! restrictions.
//!
//! **§ 1.4000(f) Cost-impairment doctrine** — restriction
//! impairs installation if it (1) unreasonably delays or
//! prevents installation, maintenance, or use; (2)
//! unreasonably increases cost; OR (3) precludes acceptable
//! quality reception.
//!
//! **Enforcement** — tenant may file FCC **Petition for
//! Declaratory Ruling** challenging restriction OR pursue
//! private action in federal or state court of competent
//! jurisdiction. FCC declaratory ruling has res judicata
//! effect on landlord.
//!
//! Citations: 47 CFR § 1.4000(a)-(f) (FCC OTARD Rule, eff.
//! October 1996); FCC Public Notice DA 96-1755; FCC FAQ on
//! Over-the-Air Reception Devices Rule; Telecommunications
//! Act of 1996 § 207 (statutory authority); 47 USC § 303
//! (FCC antenna authority).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AntennaType {
    /// § 1.4000(a)(1)(i) — Direct broadcast satellite (DBS)
    /// dish antenna 1 meter or less in diameter.
    DbsDishUnderOneMeter,
    /// § 1.4000(a)(1)(ii) — Broadband radio service (BRS,
    /// formerly MMDS) antenna 1 meter or less.
    BrsAntennaUnderOneMeter,
    /// § 1.4000(a)(1)(iii) — TV broadcast antenna (no size
    /// limit).
    TvBroadcastAntenna,
    /// DBS dish over 1 meter (NOT covered outside Alaska).
    DbsDishOverOneMeter,
    /// Antenna not covered by OTARD Rule (e.g., amateur
    /// radio antenna without OTARD scope).
    NonOtardAntenna,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstallationLocation {
    /// Tenant's balcony (exclusive use).
    TenantBalcony,
    /// Tenant's patio (exclusive use).
    TenantPatio,
    /// Tenant's exclusive-use yard or garden.
    TenantExclusiveYard,
    /// Exterior building wall (common area — landlord
    /// control).
    ExteriorWall,
    /// Building roof (common area — landlord control).
    Roof,
    /// Shared corridor or common-area parking (landlord
    /// control).
    CommonArea,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RestrictionBasis {
    /// § 1.4000(a)(3) — clearly defined, legitimate safety
    /// objective.
    LegitimateSafetyObjective,
    /// § 1.4000(a)(3) — historic district preservation.
    HistoricDistrictPreservation,
    /// Aesthetic preference (NOT permissible).
    AestheticPreference,
    /// Landlord exclusive-marketing or revenue-share
    /// agreement (NOT permissible — federal preemption
    /// strikes anti-competitive bundling).
    LandlordExclusiveMarketingAgreement,
    /// No restriction imposed.
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalSatelliteDishInstallationRightInput {
    pub antenna_type: AntennaType,
    pub installation_location: InstallationLocation,
    pub restriction_basis: RestrictionBasis,
    /// Whether restriction is narrowly tailored (no more
    /// burdensome than necessary).
    pub restriction_narrowly_tailored: bool,
    /// Whether landlord can show reasonable factual basis
    /// for restriction.
    pub landlord_reasonable_factual_basis: bool,
    /// Whether installation in Alaska (no DBS dish size
    /// limit applies).
    pub installation_in_alaska: bool,
    /// Whether restriction unreasonably delays or prevents
    /// installation (§ 1.4000(f) impairment).
    pub unreasonable_delay_or_prevention: bool,
    /// Whether restriction unreasonably increases cost.
    pub unreasonable_cost_increase: bool,
    /// Whether restriction precludes acceptable quality
    /// reception.
    pub precludes_acceptable_quality_reception: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalSatelliteDishInstallationRightResult {
    pub otard_preemption_engaged: bool,
    pub installation_protected: bool,
    pub exclusive_use_or_control_satisfied: bool,
    pub antenna_type_covered: bool,
    pub permissible_restriction_satisfied: bool,
    pub impairment_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentalSatelliteDishInstallationRightInput,
) -> RentalSatelliteDishInstallationRightResult {
    let mut violations: Vec<String> = Vec::new();

    let antenna_covered = match input.antenna_type {
        AntennaType::DbsDishUnderOneMeter => true,
        AntennaType::BrsAntennaUnderOneMeter => true,
        AntennaType::TvBroadcastAntenna => true,
        AntennaType::DbsDishOverOneMeter => input.installation_in_alaska,
        AntennaType::NonOtardAntenna => false,
    };

    let exclusive_use = matches!(
        input.installation_location,
        InstallationLocation::TenantBalcony
            | InstallationLocation::TenantPatio
            | InstallationLocation::TenantExclusiveYard
    );

    let preemption = antenna_covered && exclusive_use;

    if !antenna_covered && !matches!(input.antenna_type, AntennaType::NonOtardAntenna) {
        violations.push(
            "47 CFR § 1.4000(a)(1) — DBS dish over 1 meter NOT covered outside Alaska; OTARD federal preemption does not engage".to_string(),
        );
    }

    let permissible_restriction = match input.restriction_basis {
        RestrictionBasis::LegitimateSafetyObjective
        | RestrictionBasis::HistoricDistrictPreservation => {
            input.restriction_narrowly_tailored && input.landlord_reasonable_factual_basis
        }
        RestrictionBasis::None => true,
        _ => false,
    };

    if preemption
        && !matches!(input.restriction_basis, RestrictionBasis::None)
        && !permissible_restriction
    {
        match input.restriction_basis {
            RestrictionBasis::AestheticPreference => violations.push(
                "47 CFR § 1.4000(d) — aesthetic preference NOT permissible basis for restriction on tenant exclusive-use area; federal preemption strikes lease provisions, HOA covenants, zoning codes that impair installation".to_string(),
            ),
            RestrictionBasis::LandlordExclusiveMarketingAgreement => violations.push(
                "47 CFR § 1.4000(d) — landlord exclusive-marketing or revenue-share agreement with cable/satellite provider NOT permissible basis for restriction; federal preemption strikes anti-competitive bundling".to_string(),
            ),
            RestrictionBasis::LegitimateSafetyObjective => violations.push(
                "47 CFR § 1.4000(a)(3) — safety-objective restriction permissible ONLY if narrowly tailored AND landlord shows reasonable factual basis".to_string(),
            ),
            RestrictionBasis::HistoricDistrictPreservation => violations.push(
                "47 CFR § 1.4000(a)(3) — historic-district restriction permissible ONLY if narrowly tailored AND landlord shows reasonable factual basis".to_string(),
            ),
            RestrictionBasis::None => {}
        }
    }

    let impairment = input.unreasonable_delay_or_prevention
        || input.unreasonable_cost_increase
        || input.precludes_acceptable_quality_reception;

    if preemption && impairment {
        violations.push(
            "47 CFR § 1.4000(f) — restriction unlawfully impairs installation: unreasonably delays/prevents installation OR unreasonably increases cost OR precludes acceptable quality reception".to_string(),
        );
    }

    let installation_protected = preemption
        && (matches!(input.restriction_basis, RestrictionBasis::None) || permissible_restriction)
        && !impairment;

    let notes: Vec<String> = vec![
        "47 CFR § 1.4000(a)(1) covered antennas: (i) DBS dish 1 meter or less (any size in Alaska); (ii) BRS antenna 1 meter or less; (iii) TV broadcast antenna no size limit".to_string(),
        "47 CFR § 1.4000(a)(2) — preemption applies ONLY to installation on property within tenant's EXCLUSIVE USE OR CONTROL (balcony + patio + exclusive-use yard); landlord MAY prohibit installation on common areas (exterior walls + roof + shared corridors + common-area parking)".to_string(),
        "47 CFR § 1.4000(a)(3) permissible restrictions — narrowly tailored restrictions permitted ONLY if necessary to (1) accomplish clearly defined legitimate safety objective OR (2) preserve historic district designated under federal/state/local law; landlord must show reasonable factual basis".to_string(),
        "47 CFR § 1.4000(d) federal preemption — preempts state statutes + local zoning codes + building codes + HOA covenants + condominium declarations + lease provisions; preemption applies retroactively to existing restrictions".to_string(),
        "47 CFR § 1.4000(f) cost-impairment doctrine — restriction impairs installation if it (1) unreasonably delays or prevents installation maintenance or use; (2) unreasonably increases cost; OR (3) precludes acceptable quality reception".to_string(),
        "Tenant enforcement — FCC Petition for Declaratory Ruling OR private action in federal or state court of competent jurisdiction; FCC declaratory ruling has res judicata effect on landlord".to_string(),
        "Statutory authority — Telecommunications Act of 1996 § 207 + 47 USC § 303 FCC antenna authority; FCC OTARD Rule effective October 1996".to_string(),
    ];

    RentalSatelliteDishInstallationRightResult {
        otard_preemption_engaged: preemption,
        installation_protected,
        exclusive_use_or_control_satisfied: exclusive_use,
        antenna_type_covered: antenna_covered,
        permissible_restriction_satisfied: permissible_restriction,
        impairment_engaged: impairment,
        violations,
        citation: "47 CFR § 1.4000(a)-(f) (FCC OTARD Rule, eff. October 1996); FCC Public Notice DA 96-1755; Telecommunications Act of 1996 § 207; 47 USC § 303",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dbs_balcony_no_restriction() -> RentalSatelliteDishInstallationRightInput {
        RentalSatelliteDishInstallationRightInput {
            antenna_type: AntennaType::DbsDishUnderOneMeter,
            installation_location: InstallationLocation::TenantBalcony,
            restriction_basis: RestrictionBasis::None,
            restriction_narrowly_tailored: false,
            landlord_reasonable_factual_basis: false,
            installation_in_alaska: false,
            unreasonable_delay_or_prevention: false,
            unreasonable_cost_increase: false,
            precludes_acceptable_quality_reception: false,
        }
    }

    #[test]
    fn dbs_dish_balcony_no_restriction_protected() {
        let r = check(&dbs_balcony_no_restriction());
        assert!(r.otard_preemption_engaged);
        assert!(r.installation_protected);
        assert!(r.antenna_type_covered);
        assert!(r.exclusive_use_or_control_satisfied);
    }

    #[test]
    fn brs_antenna_balcony_covered() {
        let mut i = dbs_balcony_no_restriction();
        i.antenna_type = AntennaType::BrsAntennaUnderOneMeter;
        let r = check(&i);
        assert!(r.otard_preemption_engaged);
    }

    #[test]
    fn tv_broadcast_antenna_balcony_covered() {
        let mut i = dbs_balcony_no_restriction();
        i.antenna_type = AntennaType::TvBroadcastAntenna;
        let r = check(&i);
        assert!(r.otard_preemption_engaged);
    }

    #[test]
    fn dbs_dish_over_1_meter_outside_alaska_not_covered() {
        let mut i = dbs_balcony_no_restriction();
        i.antenna_type = AntennaType::DbsDishOverOneMeter;
        i.installation_in_alaska = false;
        let r = check(&i);
        assert!(!r.antenna_type_covered);
        assert!(!r.otard_preemption_engaged);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("1 meter") && v.contains("Alaska")));
    }

    #[test]
    fn dbs_dish_over_1_meter_in_alaska_covered() {
        let mut i = dbs_balcony_no_restriction();
        i.antenna_type = AntennaType::DbsDishOverOneMeter;
        i.installation_in_alaska = true;
        let r = check(&i);
        assert!(r.antenna_type_covered);
        assert!(r.otard_preemption_engaged);
    }

    #[test]
    fn non_otard_antenna_not_covered() {
        let mut i = dbs_balcony_no_restriction();
        i.antenna_type = AntennaType::NonOtardAntenna;
        let r = check(&i);
        assert!(!r.antenna_type_covered);
        assert!(!r.otard_preemption_engaged);
    }

    #[test]
    fn installation_on_exterior_wall_not_protected() {
        let mut i = dbs_balcony_no_restriction();
        i.installation_location = InstallationLocation::ExteriorWall;
        let r = check(&i);
        assert!(!r.exclusive_use_or_control_satisfied);
        assert!(!r.installation_protected);
    }

    #[test]
    fn installation_on_roof_not_protected() {
        let mut i = dbs_balcony_no_restriction();
        i.installation_location = InstallationLocation::Roof;
        let r = check(&i);
        assert!(!r.exclusive_use_or_control_satisfied);
        assert!(!r.installation_protected);
    }

    #[test]
    fn installation_on_common_area_not_protected() {
        let mut i = dbs_balcony_no_restriction();
        i.installation_location = InstallationLocation::CommonArea;
        let r = check(&i);
        assert!(!r.exclusive_use_or_control_satisfied);
    }

    #[test]
    fn tenant_patio_exclusive_use_protected() {
        let mut i = dbs_balcony_no_restriction();
        i.installation_location = InstallationLocation::TenantPatio;
        let r = check(&i);
        assert!(r.exclusive_use_or_control_satisfied);
        assert!(r.installation_protected);
    }

    #[test]
    fn aesthetic_preference_restriction_not_permissible() {
        let mut i = dbs_balcony_no_restriction();
        i.restriction_basis = RestrictionBasis::AestheticPreference;
        let r = check(&i);
        assert!(!r.installation_protected);
        assert!(!r.permissible_restriction_satisfied);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("aesthetic preference") && v.contains("§ 1.4000(d)")));
    }

    #[test]
    fn landlord_exclusive_marketing_agreement_not_permissible() {
        let mut i = dbs_balcony_no_restriction();
        i.restriction_basis = RestrictionBasis::LandlordExclusiveMarketingAgreement;
        let r = check(&i);
        assert!(!r.installation_protected);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("exclusive-marketing") && v.contains("anti-competitive bundling")));
    }

    #[test]
    fn safety_objective_restriction_with_narrow_tailoring_permissible() {
        let mut i = dbs_balcony_no_restriction();
        i.restriction_basis = RestrictionBasis::LegitimateSafetyObjective;
        i.restriction_narrowly_tailored = true;
        i.landlord_reasonable_factual_basis = true;
        let r = check(&i);
        assert!(r.permissible_restriction_satisfied);
        assert!(r.installation_protected);
    }

    #[test]
    fn safety_objective_restriction_without_narrow_tailoring_violation() {
        let mut i = dbs_balcony_no_restriction();
        i.restriction_basis = RestrictionBasis::LegitimateSafetyObjective;
        i.restriction_narrowly_tailored = false;
        let r = check(&i);
        assert!(!r.permissible_restriction_satisfied);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1.4000(a)(3)") && v.contains("narrowly tailored")));
    }

    #[test]
    fn historic_district_preservation_with_narrow_tailoring_permissible() {
        let mut i = dbs_balcony_no_restriction();
        i.restriction_basis = RestrictionBasis::HistoricDistrictPreservation;
        i.restriction_narrowly_tailored = true;
        i.landlord_reasonable_factual_basis = true;
        let r = check(&i);
        assert!(r.permissible_restriction_satisfied);
    }

    #[test]
    fn unreasonable_delay_engages_impairment_violation() {
        let mut i = dbs_balcony_no_restriction();
        i.unreasonable_delay_or_prevention = true;
        let r = check(&i);
        assert!(r.impairment_engaged);
        assert!(!r.installation_protected);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1.4000(f)") && v.contains("impairs installation")));
    }

    #[test]
    fn unreasonable_cost_increase_engages_impairment() {
        let mut i = dbs_balcony_no_restriction();
        i.unreasonable_cost_increase = true;
        let r = check(&i);
        assert!(r.impairment_engaged);
    }

    #[test]
    fn precludes_acceptable_quality_engages_impairment() {
        let mut i = dbs_balcony_no_restriction();
        i.precludes_acceptable_quality_reception = true;
        let r = check(&i);
        assert!(r.impairment_engaged);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&dbs_balcony_no_restriction());
        assert!(r.citation.contains("§ 1.4000(a)-(f)"));
        assert!(r.citation.contains("OTARD"));
        assert!(r.citation.contains("October 1996"));
        assert!(r.citation.contains("DA 96-1755"));
        assert!(r.citation.contains("§ 207"));
        assert!(r.citation.contains("47 USC § 303"));
    }

    #[test]
    fn note_pins_three_covered_antenna_classes() {
        let r = check(&dbs_balcony_no_restriction());
        assert!(r.notes.iter().any(|n| n.contains("§ 1.4000(a)(1)")
            && n.contains("DBS dish 1 meter")
            && n.contains("BRS antenna 1 meter")
            && n.contains("TV broadcast antenna")));
    }

    #[test]
    fn note_pins_exclusive_use_or_control_doctrine() {
        let r = check(&dbs_balcony_no_restriction());
        assert!(r.notes.iter().any(|n| n.contains("§ 1.4000(a)(2)")
            && n.contains("EXCLUSIVE USE OR CONTROL")
            && n.contains("balcony")
            && n.contains("common areas")));
    }

    #[test]
    fn note_pins_two_permissible_restriction_grounds() {
        let r = check(&dbs_balcony_no_restriction());
        assert!(r.notes.iter().any(|n| n.contains("§ 1.4000(a)(3)")
            && n.contains("safety objective")
            && n.contains("historic district")
            && n.contains("narrowly tailored")));
    }

    #[test]
    fn note_pins_federal_preemption_scope() {
        let r = check(&dbs_balcony_no_restriction());
        assert!(r.notes.iter().any(|n| n.contains("§ 1.4000(d)")
            && n.contains("state statutes")
            && n.contains("HOA covenants")
            && n.contains("lease provisions")
            && n.contains("retroactively")));
    }

    #[test]
    fn note_pins_three_prong_impairment_test() {
        let r = check(&dbs_balcony_no_restriction());
        assert!(r.notes.iter().any(|n| n.contains("§ 1.4000(f)")
            && n.contains("unreasonably delays")
            && n.contains("unreasonably increases cost")
            && n.contains("acceptable quality reception")));
    }

    #[test]
    fn note_pins_tenant_enforcement_petition_and_court() {
        let r = check(&dbs_balcony_no_restriction());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Petition for Declaratory Ruling")
                && n.contains("private action")
                && n.contains("res judicata")));
    }

    #[test]
    fn note_pins_statutory_authority_1996_telecom_act() {
        let r = check(&dbs_balcony_no_restriction());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Telecommunications Act of 1996 § 207")
                && n.contains("47 USC § 303")));
    }

    #[test]
    fn antenna_type_truth_table_five_cells() {
        for (antenna, in_alaska, exp_covered) in [
            (AntennaType::DbsDishUnderOneMeter, false, true),
            (AntennaType::BrsAntennaUnderOneMeter, false, true),
            (AntennaType::TvBroadcastAntenna, false, true),
            (AntennaType::DbsDishOverOneMeter, false, false),
            (AntennaType::DbsDishOverOneMeter, true, true),
            (AntennaType::NonOtardAntenna, false, false),
        ] {
            let mut i = dbs_balcony_no_restriction();
            i.antenna_type = antenna;
            i.installation_in_alaska = in_alaska;
            let r = check(&i);
            assert_eq!(
                r.antenna_type_covered, exp_covered,
                "antenna={:?} alaska={} expected covered={}",
                antenna, in_alaska, exp_covered
            );
        }
    }

    #[test]
    fn installation_location_truth_table_six_cells() {
        for (location, exp_exclusive) in [
            (InstallationLocation::TenantBalcony, true),
            (InstallationLocation::TenantPatio, true),
            (InstallationLocation::TenantExclusiveYard, true),
            (InstallationLocation::ExteriorWall, false),
            (InstallationLocation::Roof, false),
            (InstallationLocation::CommonArea, false),
        ] {
            let mut i = dbs_balcony_no_restriction();
            i.installation_location = location;
            let r = check(&i);
            assert_eq!(
                r.exclusive_use_or_control_satisfied, exp_exclusive,
                "location={:?} expected exclusive={}",
                location, exp_exclusive
            );
        }
    }

    #[test]
    fn restriction_basis_truth_table_five_cells() {
        for (basis, narrow, factual, exp_permissible) in [
            (RestrictionBasis::None, false, false, true),
            (
                RestrictionBasis::LegitimateSafetyObjective,
                true,
                true,
                true,
            ),
            (
                RestrictionBasis::LegitimateSafetyObjective,
                false,
                true,
                false,
            ),
            (
                RestrictionBasis::HistoricDistrictPreservation,
                true,
                true,
                true,
            ),
            (RestrictionBasis::AestheticPreference, true, true, false),
            (
                RestrictionBasis::LandlordExclusiveMarketingAgreement,
                true,
                true,
                false,
            ),
        ] {
            let mut i = dbs_balcony_no_restriction();
            i.restriction_basis = basis;
            i.restriction_narrowly_tailored = narrow;
            i.landlord_reasonable_factual_basis = factual;
            let r = check(&i);
            assert_eq!(
                r.permissible_restriction_satisfied, exp_permissible,
                "basis={:?} narrow={} factual={} expected permissible={}",
                basis, narrow, factual, exp_permissible
            );
        }
    }

    #[test]
    fn impairment_only_violation_when_preemption_engaged_invariant() {
        let mut i_protected = dbs_balcony_no_restriction();
        i_protected.unreasonable_delay_or_prevention = true;
        let r_protected = check(&i_protected);
        assert!(!r_protected.installation_protected);

        let mut i_not_preempted = dbs_balcony_no_restriction();
        i_not_preempted.installation_location = InstallationLocation::Roof;
        i_not_preempted.unreasonable_delay_or_prevention = true;
        let r_not_preempted = check(&i_not_preempted);
        assert!(!r_not_preempted.otard_preemption_engaged);
    }

    #[test]
    fn dbs_dish_size_threshold_invariant_alaska_vs_other_states() {
        let mut i_alaska = dbs_balcony_no_restriction();
        i_alaska.antenna_type = AntennaType::DbsDishOverOneMeter;
        i_alaska.installation_in_alaska = true;
        let r_alaska = check(&i_alaska);
        assert!(r_alaska.antenna_type_covered);

        let mut i_other = dbs_balcony_no_restriction();
        i_other.antenna_type = AntennaType::DbsDishOverOneMeter;
        i_other.installation_in_alaska = false;
        let r_other = check(&i_other);
        assert!(!r_other.antenna_type_covered);
    }

    #[test]
    fn multiple_violations_stack() {
        let mut i = dbs_balcony_no_restriction();
        i.restriction_basis = RestrictionBasis::AestheticPreference;
        i.unreasonable_delay_or_prevention = true;
        i.unreasonable_cost_increase = true;
        let r = check(&i);
        assert_eq!(r.violations.len(), 2);
    }
}
