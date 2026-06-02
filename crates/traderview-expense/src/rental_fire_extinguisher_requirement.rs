//! Multi-jurisdictional rental property FIRE
//! EXTINGUISHER installation, inspection, and maintenance
//! compliance framework. When must a landlord provide
//! portable fire extinguishers to tenants, what travel-
//! distance and visibility standards apply, what
//! inspection cycle is required, and what failure-mode
//! liabilities expose landlord after a fire event?
//!
//! Distinct from sibling modules: rental_chimney_fireplace_
//! inspection_disclosure (iter 471), rental_carbon_monoxide_
//! detector, tenant_fire_safety_plan_disclosure, rental_
//! bedroom_egress_window, rental_window_blind_cord_safety
//! (iter 469), rental_swimming_pool_drain_safety.
//!
//! Four-jurisdiction framework:
//!
//! 1. TEXAS (most specific landlord duty) — Tex. Prop.
//!    Code § 92.252 + § 92.255 require landlord to
//!    inspect any installed 1A10BC residential fire
//!    extinguisher (or non-rechargeable extinguisher) at
//!    BEGINNING OF TENANT'S OCCUPANCY and within
//!    REASONABLE TIME after receiving tenant's WRITTEN
//!    REQUEST; landlord must repair or replace any non-
//!    functioning extinguisher, any extinguisher with
//!    incorrect pressure per manufacturer recommendations,
//!    or any extinguisher used by tenant for legitimate
//!    purpose. Texas does NOT impose a freestanding
//!    statewide installation mandate — duty triggers when
//!    extinguisher is provided.
//! 2. MASSACHUSETTS — 527 CMR 1.00 (Massachusetts
//!    Comprehensive Fire Safety Code) adopts NFPA 10 by
//!    reference; multi-family buildings (3+ units) require
//!    portable extinguishers in common areas, on each
//!    floor, near exits per local fire-marshal authority;
//!    M.G.L. c. 148 § 26G fire safety + M.G.L. c. 148
//!    § 28A applies to certain occupancies; no statewide
//!    in-unit mandate but municipal ordinances common
//!    (Boston, Worcester, Springfield).
//! 3. NEW JERSEY — N.J.A.C. 5:70-3 (NJ Uniform Fire Code
//!    adopting NFPA 10 by reference); landlords NOT
//!    legally required to provide fire extinguishers in
//!    units unless stated in lease contract or local
//!    ordinance, BUT any installed unit MUST be
//!    maintained, inspected, and tagged per NFPA 10 and
//!    local code; N.J.S.A. 46:8-39 to -50 hotels and
//!    multiple dwellings registration triggers fire-safety
//!    inspections that include extinguishers in common
//!    areas.
//! 4. DEFAULT — NFPA 10 (Standard for Portable Fire
//!    Extinguishers, current edition) is a VOLUNTARY
//!    NATIONAL STANDARD adopted by reference in most
//!    state fire codes; multi-family > 2 units typically
//!    requires extinguishers in common areas; common-law
//!    implied warranty of habitability per Hilder v. St.
//!    Peter, 478 A.2d 202 (Vt. 1984); Green v. Superior
//!    Court, 10 Cal. 3d 616 (1974); landlord tort
//!    negligence + premises liability for fire injury
//!    where extinguisher availability could have
//!    prevented or limited damage.
//!
//! NFPA 10 five-cycle inspection framework:
//! 1. MONTHLY VISUAL INSPECTION — verify gauge in green,
//!    no obvious damage, tamper seal intact, label legible,
//!    weight check
//! 2. ANNUAL MAINTENANCE — certified technician
//!    inspection including hose/nozzle/discharge mechanism
//!    + recharge if needed
//! 3. SIX-YEAR INTERNAL EXAMINATION — required for dry
//!    chemical units; complete disassembly + recharge
//! 4. TWELVE-YEAR HYDROSTATIC TEST — pressure-vessel
//!    integrity test
//! 5. RECHARGE AFTER EVERY USE — including partial
//!    discharge; landlord duty under Tex. Prop. Code
//!    § 92.255 explicitly
//!
//! NFPA 10 placement requirements:
//! - Class A (ordinary combustibles): maximum 75-ft
//!   travel distance
//! - Class B (flammable liquids): 30-ft or 50-ft based on
//!   hazard
//! - Mounting height: max 5 ft (handle from floor) for
//!   extinguishers over 40 lbs; max 3 ft 6 in for
//!   extinguishers > 40 lbs
//! - Visibility: must be conspicuously located + clearly
//!   marked; signage required if not visible
//!
//! Universal failure-mode liability framework:
//! 1. Provided extinguisher non-functional during fire →
//!    tort negligence + premises liability + breach of
//!    implied warranty
//! 2. No extinguisher provided in multi-family building
//!    where required by local code → fire code violation
//!    + civil penalties + tenant rescission
//! 3. Extinguisher used by tenant + landlord refuses
//!    recharge → Tex. Prop. Code § 92.255 explicit
//!    violation; constructive eviction risk
//! 4. Tenant complaint regarding gauge / pressure /
//!    expiration ignored → habitability breach + retaliation
//!    exposure (see landlord_retaliation_damages)
//! 5. Fire injury during tenancy with non-compliant
//!    extinguisher → tort negligence + wrongful death +
//!    IIED parallel to tenant_emotional_distress_damages
//!    iter 453
//!
//! Trader-landlord critical because (1) Texas trader-
//! landlord has explicit statutory duty under § 92.252 +
//! § 92.255 to inspect at occupancy and respond to
//! tenant written request, with no statewide installation
//! mandate but strict inspection-after-provision duty;
//! (2) Massachusetts trader operating multifamily in
//! Boston/Worcester must comply with NFPA 10 in common
//! areas per 527 CMR 1.00 + local ordinance; (3) New
//! Jersey trader-landlord avoiding contractual
//! extinguisher commitment limits statutory exposure but
//! must still maintain any installed unit per NFPA 10;
//! (4) annual inspection cost $5-$25 per extinguisher
//! vs. tort liability exposure exceeding $1M per fire
//! injury; (5) 6-year internal examination + 12-year
//! hydrostatic test are key date-tracking compliance
//! obligations easily missed in portfolio management.
//!
//! Authority: NFPA 10 (Standard for Portable Fire
//! Extinguishers, current edition); Tex. Prop. Code
//! § 92.252; Tex. Prop. Code § 92.255; 527 C.M.R. 1.00
//! Massachusetts Comprehensive Fire Safety Code; M.G.L.
//! c. 148 § 26G; M.G.L. c. 148 § 28A; N.J.A.C. 5:70-3
//! NJ Uniform Fire Code; N.J.S.A. 46:8-39 to -50;
//! Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984); Green
//! v. Superior Court, 10 Cal. 3d 616 (1974); Cal. Civ.
//! Code § 1941.1 implied warranty.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    Texas,
    Massachusetts,
    NewJersey,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingType {
    SingleFamily,
    DuplexTwoUnit,
    MultifamilyThreeToFive,
    MultifamilyOverFive,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub building_type: BuildingType,
    pub extinguisher_provided_by_landlord: bool,
    pub last_annual_inspection_months_ago: u32,
    pub last_six_year_internal_exam_years_ago: u32,
    pub last_twelve_year_hydrostatic_test_years_ago: u32,
    pub tenant_written_request_received: bool,
    pub tenant_written_request_addressed: bool,
    pub gauge_in_green: bool,
    pub used_by_tenant_legitimate_purpose: bool,
    pub recharge_completed_after_use: bool,
    pub max_travel_distance_to_extinguisher_feet: u32,
    pub fire_injury_event_reported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    InspectionOverdue,
    DefectObserved,
    PlacementViolation,
    StatutoryDutyBreach,
    FireInjuryEvent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub const ANNUAL_INSPECTION_MAX_MONTHS: u32 = 12;
pub const SIX_YEAR_INTERNAL_EXAM_MAX_YEARS: u32 = 6;
pub const TWELVE_YEAR_HYDROSTATIC_MAX_YEARS: u32 = 12;
pub const NFPA_10_CLASS_A_MAX_TRAVEL_FEET: u32 = 75;

pub type RentalFireExtinguisherRequirementInput = Input;
pub type RentalFireExtinguisherRequirementResult = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: Texas (most specific landlord duty — Tex. Prop. Code § 92.252 + § 92.255 require inspection at occupancy and after tenant written request; recharge/repair/replace duty for any landlord-provided 1A10BC residential extinguisher); Massachusetts (527 C.M.R. 1.00 Comprehensive Fire Safety Code adopting NFPA 10 by reference; M.G.L. c. 148 § 26G + § 28A; multifamily 3+ unit common-area extinguishers per local fire marshal); New Jersey (N.J.A.C. 5:70-3 NJ Uniform Fire Code adopting NFPA 10; no statutory in-unit mandate absent lease contract; N.J.S.A. 46:8-39 to -50 hotel/multiple-dwelling registration); Default (NFPA 10 voluntary national standard + common-law habitability per Hilder v. St. Peter 478 A.2d 202 (Vt. 1984) + Cal. Civ. Code § 1941.1).".to_string(),
        "NFPA 10 five-cycle inspection framework: (1) MONTHLY visual inspection (gauge green + tamper seal intact + label legible + weight check); (2) ANNUAL maintenance by certified technician (hose/nozzle/discharge mechanism + recharge as needed); (3) SIX-YEAR internal examination for dry-chemical units; (4) TWELVE-YEAR hydrostatic test for pressure-vessel integrity; (5) RECHARGE AFTER EVERY USE including partial discharge (Tex. Prop. Code § 92.255 explicit landlord duty).".to_string(),
        "NFPA 10 placement requirements: Class A maximum 75-ft travel distance; Class B 30-ft or 50-ft based on hazard; mounting height max 5 ft (handle from floor) for extinguishers up to 40 lbs, max 3 ft 6 in for over 40 lbs; conspicuously located with signage if not visible.".to_string(),
        "Five universal failure-mode liabilities: (1) provided extinguisher non-functional during fire → tort negligence + premises liability + implied-warranty breach; (2) no extinguisher in multi-family where required by local code → fire code violation + civil penalties + tenant rescission; (3) extinguisher used by tenant + landlord refuses recharge → Tex. Prop. Code § 92.255 explicit violation + constructive eviction; (4) tenant complaint about gauge/pressure/expiration ignored → habitability breach + retaliation (landlord_retaliation_damages); (5) fire injury with non-compliant extinguisher → tort negligence + wrongful death + IIED parallel to tenant_emotional_distress_damages iter 453.".to_string(),
        "Companion modules: rental_chimney_fireplace_inspection_disclosure (iter 471), rental_carbon_monoxide_detector, tenant_fire_safety_plan_disclosure, rental_bedroom_egress_window, rental_window_blind_cord_safety (iter 469), rental_swimming_pool_drain_safety, landlord_retaliation_damages, tenant_emotional_distress_damages.".to_string(),
    ];
    let mut actions: Vec<String> = Vec::new();

    if input.fire_injury_event_reported {
        actions.push("Fire injury event reported: engage emergency services + counsel; preserve evidence including extinguisher unit; tort negligence + wrongful death + IIED exposure parallel to tenant_emotional_distress_damages iter 453.".to_string());
    }

    if !input.extinguisher_provided_by_landlord {
        // Texas: no statewide installation mandate but multifamily common-area may apply
        // NJ: no statutory mandate absent contract
        // MA: multifamily common-area required
        match input.jurisdiction {
            Jurisdiction::Massachusetts if !matches!(input.building_type, BuildingType::SingleFamily | BuildingType::DuplexTwoUnit) => {
                actions.push("Massachusetts multifamily (3+ unit): 527 C.M.R. 1.00 Comprehensive Fire Safety Code adopting NFPA 10 + local fire-marshal authority typically requires portable extinguishers in common areas, on each floor, near exits; M.G.L. c. 148 § 26G + § 28A.".to_string());
            }
            _ => {
                actions.push(
                    "No landlord-provided extinguisher: no statutory in-unit mandate triggered absent local ordinance or lease contract. Common-area extinguishers may still be required in multifamily buildings — verify with local fire marshal."
                        .to_string(),
                );
            }
        }

        let severity = if input.fire_injury_event_reported {
            Severity::FireInjuryEvent
        } else if matches!(input.jurisdiction, Jurisdiction::Massachusetts)
            && !matches!(input.building_type, BuildingType::SingleFamily | BuildingType::DuplexTwoUnit)
        {
            Severity::StatutoryDutyBreach
        } else {
            Severity::Compliant
        };

        return Output {
            severity,
            jurisdiction_specific_actions: actions,
            notes,
        };
    }

    let mut defect_observed = false;
    let mut inspection_overdue = false;
    let mut placement_violation = false;
    let mut statutory_duty_breach = false;

    if !input.gauge_in_green {
        defect_observed = true;
        actions.push("Extinguisher gauge not in green: low pressure or over-pressurized; landlord repair/replace duty triggered.".to_string());
    }

    if input.last_annual_inspection_months_ago > ANNUAL_INSPECTION_MAX_MONTHS {
        inspection_overdue = true;
        actions.push(format!(
            "Annual NFPA 10 inspection overdue: {} months since last inspection exceeds 12-month maximum; engage NFPA 10 certified technician.",
            input.last_annual_inspection_months_ago
        ));
    }

    if input.last_six_year_internal_exam_years_ago > SIX_YEAR_INTERNAL_EXAM_MAX_YEARS {
        inspection_overdue = true;
        actions.push(format!(
            "Six-year NFPA 10 internal examination overdue: {} years since last internal exam exceeds 6-year cycle for dry-chemical units; complete disassembly and recharge required.",
            input.last_six_year_internal_exam_years_ago
        ));
    }

    if input.last_twelve_year_hydrostatic_test_years_ago > TWELVE_YEAR_HYDROSTATIC_MAX_YEARS {
        inspection_overdue = true;
        actions.push(format!(
            "Twelve-year NFPA 10 hydrostatic test overdue: {} years since last hydrostatic test exceeds 12-year cycle; pressure-vessel integrity test required.",
            input.last_twelve_year_hydrostatic_test_years_ago
        ));
    }

    if input.max_travel_distance_to_extinguisher_feet > NFPA_10_CLASS_A_MAX_TRAVEL_FEET {
        placement_violation = true;
        actions.push(format!(
            "NFPA 10 Class A travel-distance violation: maximum {} ft exceeds 75-ft cap; add extinguishers to reduce travel distance.",
            input.max_travel_distance_to_extinguisher_feet
        ));
    }

    if input.used_by_tenant_legitimate_purpose && !input.recharge_completed_after_use {
        statutory_duty_breach = true;
        let citation = match input.jurisdiction {
            Jurisdiction::Texas => "Tex. Prop. Code § 92.255 explicit duty",
            Jurisdiction::Massachusetts => "NFPA 10 recharge-after-use (adopted via 527 C.M.R. 1.00)",
            Jurisdiction::NewJersey => "NFPA 10 recharge-after-use (adopted via N.J.A.C. 5:70-3)",
            Jurisdiction::Default => "NFPA 10 recharge-after-use",
        };
        actions.push(format!(
            "Extinguisher used by tenant for legitimate purpose but not recharged: violates {}; landlord recharge/repair/replace duty triggered.",
            citation
        ));
    }

    if input.tenant_written_request_received && !input.tenant_written_request_addressed {
        statutory_duty_breach = true;
        let citation = match input.jurisdiction {
            Jurisdiction::Texas => "Tex. Prop. Code § 92.255 reasonable-time response duty",
            _ => "NFPA 10 + landlord_retaliation_damages exposure",
        };
        actions.push(format!(
            "Tenant written request unaddressed: violates {}; constructive eviction + retaliation risk.",
            citation
        ));
    }

    match input.jurisdiction {
        Jurisdiction::Texas => {
            actions.push("Texas: Tex. Prop. Code § 92.252 + § 92.255 — landlord must inspect any installed 1A10BC residential extinguisher at occupancy + after tenant written request; repair/replace duty for non-functional + incorrect pressure + tenant-used units.".to_string());
        }
        Jurisdiction::Massachusetts => {
            actions.push("Massachusetts: 527 C.M.R. 1.00 Comprehensive Fire Safety Code adopting NFPA 10 by reference; M.G.L. c. 148 § 26G + § 28A; multifamily common-area extinguishers per local fire marshal authority.".to_string());
        }
        Jurisdiction::NewJersey => {
            actions.push("New Jersey: N.J.A.C. 5:70-3 NJ Uniform Fire Code adopting NFPA 10 by reference; no in-unit statutory mandate absent lease contract or local ordinance; N.J.S.A. 46:8-39 to -50 hotel/multiple-dwelling registration triggers inspections.".to_string());
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: NFPA 10 voluntary national standard adopted by reference in most state fire codes; common-law implied warranty of habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Cal. Civ. Code § 1941.1.".to_string());
        }
    }

    let severity = if input.fire_injury_event_reported {
        Severity::FireInjuryEvent
    } else if statutory_duty_breach {
        Severity::StatutoryDutyBreach
    } else if placement_violation {
        Severity::PlacementViolation
    } else if defect_observed {
        Severity::DefectObserved
    } else if inspection_overdue {
        Severity::InspectionOverdue
    } else {
        Severity::Compliant
    };

    Output {
        severity,
        jurisdiction_specific_actions: actions,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::Texas,
            building_type: BuildingType::SingleFamily,
            extinguisher_provided_by_landlord: true,
            last_annual_inspection_months_ago: 6,
            last_six_year_internal_exam_years_ago: 2,
            last_twelve_year_hydrostatic_test_years_ago: 5,
            tenant_written_request_received: false,
            tenant_written_request_addressed: false,
            gauge_in_green: true,
            used_by_tenant_legitimate_purpose: false,
            recharge_completed_after_use: false,
            max_travel_distance_to_extinguisher_feet: 50,
            fire_injury_event_reported: false,
        }
    }

    #[test]
    fn texas_compliant_baseline() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn no_extinguisher_single_family_compliant() {
        let mut i = baseline();
        i.extinguisher_provided_by_landlord = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn no_extinguisher_ma_multifamily_breach() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.building_type = BuildingType::MultifamilyThreeToFive;
        i.extinguisher_provided_by_landlord = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::StatutoryDutyBreach);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Massachusetts multifamily"));
        assert!(joined.contains("527 C.M.R. 1.00"));
        assert!(joined.contains("§ 26G"));
    }

    #[test]
    fn no_extinguisher_nj_multifamily_no_breach() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        i.building_type = BuildingType::MultifamilyOverFive;
        i.extinguisher_provided_by_landlord = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn annual_inspection_overdue_over_12_months() {
        let mut i = baseline();
        i.last_annual_inspection_months_ago = 18;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("12-month"));
        assert!(joined.contains("certified technician"));
    }

    #[test]
    fn annual_inspection_exactly_12_months_compliant() {
        let mut i = baseline();
        i.last_annual_inspection_months_ago = 12;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn six_year_internal_exam_overdue() {
        let mut i = baseline();
        i.last_six_year_internal_exam_years_ago = 7;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("6-year cycle"));
        assert!(joined.contains("dry-chemical"));
    }

    #[test]
    fn twelve_year_hydrostatic_overdue() {
        let mut i = baseline();
        i.last_twelve_year_hydrostatic_test_years_ago = 13;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("12-year"));
        assert!(joined.contains("hydrostatic"));
    }

    #[test]
    fn gauge_not_green_defect_observed() {
        let mut i = baseline();
        i.gauge_in_green = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("gauge not in green"));
    }

    #[test]
    fn travel_distance_over_75_feet_placement_violation() {
        let mut i = baseline();
        i.max_travel_distance_to_extinguisher_feet = 90;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PlacementViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("75-ft cap"));
        assert!(joined.contains("Class A"));
    }

    #[test]
    fn travel_distance_exactly_75_feet_compliant() {
        let mut i = baseline();
        i.max_travel_distance_to_extinguisher_feet = 75;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn used_by_tenant_not_recharged_statutory_duty_breach_texas() {
        let mut i = baseline();
        i.used_by_tenant_legitimate_purpose = true;
        i.recharge_completed_after_use = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::StatutoryDutyBreach);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Tex. Prop. Code § 92.255"));
    }

    #[test]
    fn used_by_tenant_recharged_compliant() {
        let mut i = baseline();
        i.used_by_tenant_legitimate_purpose = true;
        i.recharge_completed_after_use = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn tenant_written_request_unaddressed_breach_texas() {
        let mut i = baseline();
        i.tenant_written_request_received = true;
        i.tenant_written_request_addressed = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::StatutoryDutyBreach);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Tex. Prop. Code § 92.255 reasonable-time"));
    }

    #[test]
    fn tenant_written_request_addressed_compliant() {
        let mut i = baseline();
        i.tenant_written_request_received = true;
        i.tenant_written_request_addressed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn fire_injury_event_top_severity() {
        let mut i = baseline();
        i.fire_injury_event_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FireInjuryEvent);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Fire injury event"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
    }

    #[test]
    fn ma_jurisdiction_cites_527_cmr_and_148() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("527 C.M.R. 1.00"));
        assert!(joined.contains("§ 26G"));
        assert!(joined.contains("§ 28A"));
    }

    #[test]
    fn nj_jurisdiction_cites_5_70_3_and_46_8_39() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("N.J.A.C. 5:70-3"));
        assert!(joined.contains("N.J.S.A. 46:8-39"));
    }

    #[test]
    fn default_jurisdiction_common_law_habitability() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("voluntary national standard"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("§ 1941.1"));
    }

    #[test]
    fn severity_priority_injury_above_statutory_above_placement_above_defect_above_inspection() {
        let mut i = baseline();
        i.fire_injury_event_reported = true;
        i.used_by_tenant_legitimate_purpose = true;
        i.recharge_completed_after_use = false;
        i.max_travel_distance_to_extinguisher_feet = 100;
        i.gauge_in_green = false;
        i.last_annual_inspection_months_ago = 24;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FireInjuryEvent);
    }

    #[test]
    fn severity_statutory_above_placement_above_defect_above_inspection() {
        let mut i = baseline();
        i.used_by_tenant_legitimate_purpose = true;
        i.recharge_completed_after_use = false;
        i.max_travel_distance_to_extinguisher_feet = 100;
        i.gauge_in_green = false;
        i.last_annual_inspection_months_ago = 24;
        let out = check(&i);
        assert_eq!(out.severity, Severity::StatutoryDutyBreach);
    }

    #[test]
    fn severity_placement_above_defect_above_inspection() {
        let mut i = baseline();
        i.max_travel_distance_to_extinguisher_feet = 100;
        i.gauge_in_green = false;
        i.last_annual_inspection_months_ago = 24;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PlacementViolation);
    }

    #[test]
    fn severity_defect_above_inspection() {
        let mut i = baseline();
        i.gauge_in_green = false;
        i.last_annual_inspection_months_ago = 24;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefectObserved);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("NFPA 10"));
        assert!(joined.contains("Tex. Prop. Code § 92.252"));
        assert!(joined.contains("Tex. Prop. Code § 92.255"));
        assert!(joined.contains("527 C.M.R. 1.00"));
        assert!(joined.contains("§ 26G"));
        assert!(joined.contains("§ 28A"));
        assert!(joined.contains("N.J.A.C. 5:70-3"));
        assert!(joined.contains("N.J.S.A. 46:8-39"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("§ 1941.1"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Texas (most specific"));
        assert!(joined.contains("Massachusetts"));
        assert!(joined.contains("New Jersey"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_five_cycle_inspection_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("MONTHLY visual"));
        assert!(joined.contains("ANNUAL maintenance"));
        assert!(joined.contains("SIX-YEAR"));
        assert!(joined.contains("TWELVE-YEAR"));
        assert!(joined.contains("RECHARGE AFTER EVERY USE"));
    }

    #[test]
    fn note_pins_nfpa_10_placement_requirements() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("75-ft travel distance"));
        assert!(joined.contains("Class A"));
        assert!(joined.contains("Class B"));
        assert!(joined.contains("mounting height"));
    }

    #[test]
    fn note_pins_five_failure_modes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("non-functional during fire"));
        assert!(joined.contains("no extinguisher"));
        assert!(joined.contains("refuses recharge"));
        assert!(joined.contains("complaint about gauge"));
        assert!(joined.contains("fire injury"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("rental_chimney_fireplace_inspection_disclosure"));
        assert!(joined.contains("rental_carbon_monoxide_detector"));
        assert!(joined.contains("tenant_fire_safety_plan_disclosure"));
        assert!(joined.contains("rental_window_blind_cord_safety"));
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        let tx = check(&Input {
            jurisdiction: Jurisdiction::Texas,
            ..baseline()
        });
        let ma = check(&Input {
            jurisdiction: Jurisdiction::Massachusetts,
            ..baseline()
        });
        let nj = check(&Input {
            jurisdiction: Jurisdiction::NewJersey,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            ..baseline()
        });
        assert_eq!(tx.severity, Severity::Compliant);
        assert_eq!(ma.severity, Severity::Compliant);
        assert_eq!(nj.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn texas_uniquely_strictest_inspection_duty_invariant() {
        // Same fact: tenant written request unaddressed
        let tx = check(&Input {
            jurisdiction: Jurisdiction::Texas,
            tenant_written_request_received: true,
            tenant_written_request_addressed: false,
            ..baseline()
        });
        // Texas explicitly cites § 92.255
        let joined_tx = tx.jurisdiction_specific_actions.join(" ");
        assert!(joined_tx.contains("Tex. Prop. Code § 92.255 reasonable-time"));
    }

    #[test]
    fn multiple_problems_stack_in_actions() {
        let mut i = baseline();
        i.gauge_in_green = false;
        i.last_annual_inspection_months_ago = 18;
        i.last_six_year_internal_exam_years_ago = 7;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("gauge not in green"));
        assert!(joined.contains("12-month"));
        assert!(joined.contains("6-year cycle"));
    }
}
