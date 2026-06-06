//! Rental balcony inspection plus Exterior Elevated Element (EEE) safety
//! framework — covers California SB 721 (rental) plus SB 326 (HOA / condo)
//! mandatory periodic inspection regime enacted in response to the June 16,
//! 2015 Berkeley balcony collapse that killed six students at the Library
//! Gardens apartments. AB 2579 (signed 2024) extended the SB 721 first-cycle
//! deadline from January 1, 2025 to January 1, 2026.
//!
//! Distinct from sibling `rental_basement_water_intrusion_disclosure`
//! (basement water disclosure), `rental_window_guard_installation` (fall
//! protection from windows), `tenant_window_air_conditioner_install_right`
//! (iter ... — falling AC bracket safety), [[rental_storage_unit_lease_
//! disclosure]] (iter 509 storage disclosure).
//!
//! Trader-landlord critical because (1) **SB 721 first-cycle inspection
//! deadline January 1, 2026 (post-AB 2579 extension)** — any covered
//! building (3+ residential dwelling units with weather-exposed wood-framed
//! EEEs) that has NOT obtained inspection by deadline faces local-code-
//! enforcement civil penalties ($100-$500 per day until cured) plus civil
//! liability exposure on EEE-failure injury claims; (2) inspector must be
//! licensed architect, licensed civil/structural engineer, building
//! contractor with A/B/C-5 license + 5 years experience, or certified
//! building inspector per Cal. Health & Safety Code § 17973(d); (3)
//! minimum 15% of each EEE type must be inspected per § 17973(c) (for 40
//! balconies → 6 must be inspected); (4) post-2026 cycle: SB 721 every 6
//! years, SB 326 every 9 years (HOA condo); (5) SB 326 EEEs must be
//! inspected by licensed structural engineer specifically; (6) failure
//! to repair "immediate threat" identified in inspection triggers 120-day
//! repair window plus permitting requirement; (7) Cal. Civ. Code § 1942.4
//! "tenantability" claim attaches if EEE is unsafe and tenant is exposed.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    /// California SB 721 / SB 326 framework.
    California,
    /// New York City Local Law 11 (FISP) — Façade Inspection Safety
    /// Program — 5-year cycle inspections for 6+ story buildings.
    NewYorkCityFisp,
    /// Default — common-law habitability plus state-specific.
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingType {
    /// Multifamily rental 3+ units with EEEs — SB 721 applies.
    Multifamily3PlusRentalSb721,
    /// HOA / condominium common-interest development with EEEs — SB 326
    /// applies.
    HoaCondominiumSb326,
    /// Single-family or duplex rental — SB 721 inapplicable; common-law
    /// habitability still attaches.
    SingleFamilyOrDuplex,
    /// Building with NO wood-framed EEEs (concrete/steel construction
    /// only) — both SB 721 and SB 326 inapplicable.
    NoWoodFramedEees,
    /// New York City 6+ story building — NYC Local Law 11 FISP applies.
    NycSixPlusStoryFisp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectorQualification {
    /// Licensed architect — SB 721 qualified.
    LicensedArchitect,
    /// Licensed civil or structural engineer — SB 721 + SB 326 qualified.
    LicensedCivilOrStructuralEngineer,
    /// Building contractor with A, B, or C-5 license + 5+ years
    /// experience — SB 721 qualified only.
    ContractorWithABorC5LicenseAndFiveYears,
    /// Certified building inspector — SB 721 qualified only.
    CertifiedBuildingInspector,
    /// Not qualified.
    NotQualified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantWithFirstCycleInspection,
    FirstCycleInspectionPastDeadlineViolation,
    InspectorNotQualified,
    InspectionSampleBelow15Pct,
    ImmediateThreatRepairOverdue120Day,
    HoaSb326UsedNonStructuralEngineerInvalid,
    TenantOccupiedUnsafeEeeHabitabilityBreach,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub building_type: BuildingType,
    pub inspector_qualification: InspectorQualification,
    pub total_eee_count: u32,
    pub inspected_eee_count: u32,
    pub inspection_completed_by_2026_01_01: bool,
    pub immediate_threat_identified: bool,
    pub days_since_immediate_threat_identified: u32,
    pub tenant_currently_using_unsafe_eee: bool,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub minimum_inspection_count_required: u32,
    pub daily_civil_penalty_min_cents: u64,
    pub daily_civil_penalty_max_cents: u64,
    pub annual_rent_at_risk_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const SB_721_FIRST_CYCLE_DEADLINE_YEAR: i32 = 2026;
pub const SB_721_FIRST_CYCLE_ORIGINAL_DEADLINE_YEAR: i32 = 2025;
pub const SB_721_SUBSEQUENT_CYCLE_YEARS: u32 = 6;
pub const SB_326_INSPECTION_CYCLE_YEARS: u32 = 9;
pub const SB_721_MINIMUM_INSPECTION_PCT_BPS: u32 = 1_500;
pub const SB_721_IMMEDIATE_THREAT_REPAIR_DAYS: u32 = 120;
pub const SB_721_CONTRACTOR_EXPERIENCE_YEARS: u32 = 5;
pub const SB_721_DAILY_PENALTY_MIN_CENTS: u64 = 10_000;
pub const SB_721_DAILY_PENALTY_MAX_CENTS: u64 = 50_000;
pub const BERKELEY_BALCONY_COLLAPSE_DATE: &str = "2015-06-16";

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(
        input.building_type,
        BuildingType::NoWoodFramedEees | BuildingType::SingleFamilyOrDuplex
    ) {
        notes.push(
            "Building has NO weather-exposed wood-framed Exterior Elevated Elements (EEEs) \
             OR is single-family / duplex outside SB 721 scope. SB 721 / SB 326 inapplicable. \
             Common-law habitability doctrine still attaches; if EEEs exist on a 1-2 unit \
             property, voluntary inspection is best practice but not statutorily required."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            minimum_inspection_count_required: 0,
            daily_civil_penalty_min_cents: 0,
            daily_civil_penalty_max_cents: 0,
            annual_rent_at_risk_cents: 0,
            citation: "n/a (outside SB 721/326 scope)",
            notes,
        };
    }

    let minimum_required: u32 = (u64::from(input.total_eee_count)
        * u64::from(SB_721_MINIMUM_INSPECTION_PCT_BPS)
        / 10_000) as u32;
    let minimum_required = minimum_required.max(1);

    if input.tenant_currently_using_unsafe_eee && input.immediate_threat_identified {
        severity = Severity::TenantOccupiedUnsafeEeeHabitabilityBreach;
        actions.push(
            "IMMEDIATE THREAT identified AND tenant currently using unsafe EEE — habitability \
             breach under Cal. Civ. Code § 1942.4. Restrict tenant access to unsafe EEE \
             pending repair; temporarily relocate per [[mid_tenancy_temporary_relocation]] \
             sibling if EEE is the sole egress path. Notify tenants in writing of identified \
             threat; engage permitting per Cal. Health & Safety Code § 17973(f) within \
             accelerated window."
                .to_string(),
        );
    } else if matches!(input.building_type, BuildingType::HoaCondominiumSb326)
        && !matches!(
            input.inspector_qualification,
            InspectorQualification::LicensedCivilOrStructuralEngineer
        )
    {
        severity = Severity::HoaSb326UsedNonStructuralEngineerInvalid;
        actions.push(
            "SB 326 HOA / condominium inspection REQUIRES licensed structural engineer per \
             Cal. Civ. Code § 5551 — architect plus contractor inspections not sufficient. \
             Re-engage qualified structural engineer; prior inspection report not valid for \
             HOA / condo compliance even if SB 721 qualifications met."
                .to_string(),
        );
    } else if matches!(
        input.inspector_qualification,
        InspectorQualification::NotQualified
    ) {
        severity = Severity::InspectorNotQualified;
        actions.push(format!(
            "Inspector not qualified under Cal. Health & Safety Code § 17973(d). Qualifying \
             credentials: (1) licensed architect, (2) licensed civil or structural engineer, \
             (3) building contractor with A, B, or C-5 license plus at least {} years \
             experience, (4) certified building inspector. SB 326 (HOA) additionally requires \
             licensed structural engineer specifically.",
            SB_721_CONTRACTOR_EXPERIENCE_YEARS
        ));
    } else if !input.inspection_completed_by_2026_01_01 {
        severity = Severity::FirstCycleInspectionPastDeadlineViolation;
        actions.push(format!(
            "First-cycle SB 721 inspection NOT completed by January 1, {} deadline (post-AB \
             2579 one-year extension from original January 1, {} deadline). Local code \
             enforcement may impose civil penalties of ${} to ${} per day until inspection \
             completed and any identified hazards cured. ENGAGE qualified inspector \
             immediately; document inspection report retention plus permit any 'immediate \
             threat' repairs within {}-day accelerated repair window.",
            SB_721_FIRST_CYCLE_DEADLINE_YEAR,
            SB_721_FIRST_CYCLE_ORIGINAL_DEADLINE_YEAR,
            SB_721_DAILY_PENALTY_MIN_CENTS / 100,
            SB_721_DAILY_PENALTY_MAX_CENTS / 100,
            SB_721_IMMEDIATE_THREAT_REPAIR_DAYS
        ));
    } else if input.inspected_eee_count < minimum_required {
        severity = Severity::InspectionSampleBelow15Pct;
        actions.push(format!(
            "Inspection sample {} EEEs falls below 15% minimum required by Cal. Health & \
             Safety Code § 17973(c). For {} total EEEs of this type, at least {} must be \
             inspected (minimum 1 per type). Engage inspector to expand sample plus update \
             written report.",
            input.inspected_eee_count, input.total_eee_count, minimum_required
        ));
    } else if input.immediate_threat_identified
        && input.days_since_immediate_threat_identified > SB_721_IMMEDIATE_THREAT_REPAIR_DAYS
    {
        severity = Severity::ImmediateThreatRepairOverdue120Day;
        actions.push(format!(
            "Immediate-threat repair OVERDUE: {} days since identification; statutory \
             accelerated repair window is {} days per Cal. Health & Safety Code § 17973(f). \
             Civil-penalty risk plus tenant-injury liability accelerating; secure permits \
             and begin repair work immediately. Document barricade-or-restrict-access \
             measures pending completion.",
            input.days_since_immediate_threat_identified, SB_721_IMMEDIATE_THREAT_REPAIR_DAYS
        ));
    } else {
        severity = Severity::CompliantWithFirstCycleInspection;
        actions.push(format!(
            "Compliant: first-cycle inspection completed by January 1, {} deadline; sample \
             count {} EEEs meets 15% minimum (required {} EEEs for {} total). Maintain \
             inspection report for {}-year inspection cycle (SB 721) or {}-year (SB 326). \
             Calendar next cycle inspection plus annual landlord walk-through for early \
             detection.",
            SB_721_FIRST_CYCLE_DEADLINE_YEAR,
            input.inspected_eee_count,
            minimum_required,
            input.total_eee_count,
            SB_721_SUBSEQUENT_CYCLE_YEARS,
            SB_326_INSPECTION_CYCLE_YEARS
        ));
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            notes.push(format!(
                "Cal. Health & Safety Code § 17973 (SB 721, signed September 17, 2018, \
                 effective January 1, 2019) requires 9-year inspection cycle initially; \
                 first cycle deadline ORIGINALLY January 1, {} extended to January 1, {} \
                 by AB 2579 (signed 2024). Cal. Civ. Code § 5551 (SB 326, signed August 30, \
                 2019) parallel HOA / condominium regime with 9-year cycle requiring \
                 licensed structural engineer. Berkeley balcony collapse {} killed six \
                 students at Library Gardens; legislative response in 2018 SB 721 plus \
                 2019 SB 326. Cal. Civ. Code § 1942.4 tenantability claim attaches for \
                 unsafe EEE exposure.",
                SB_721_FIRST_CYCLE_ORIGINAL_DEADLINE_YEAR,
                SB_721_FIRST_CYCLE_DEADLINE_YEAR,
                BERKELEY_BALCONY_COLLAPSE_DATE
            ));
        }
        Jurisdiction::NewYorkCityFisp => {
            notes.push(
                "NYC Local Law 11 of 1998 (Façade Inspection Safety Program — FISP) requires \
                 façade inspection of buildings 6+ stories every 5 years; reports filed with \
                 NYC Department of Buildings. NYC AC § 28-302.1 governs. Separate from SB \
                 721 / SB 326 California regime; FISP focuses on façade rather than wood-\
                 framed EEEs. NYC adopted RCNY 1 § 103-04 procedural rules implementing \
                 LL11."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "Outside California and NYC, no statewide mandatory periodic balcony / EEE \
                 inspection regime — common-law habitability + premises-liability tort \
                 framework governs. Best practice: voluntary 6-9 year inspection cycle \
                 following SB 721 / SB 326 methodology to mitigate catastrophic-injury \
                 liability exposure. Florida + Hawaii pending legislation post-2021 \
                 Champlain Towers collapse but as of 2026 no state-mandated periodic \
                 inspection."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[rental_basement_water_intrusion_disclosure]] (water-damage \
         cause of wood EEE deterioration cross-reference), [[rental_window_guard_\
         installation]] (parallel fall-protection framework — distinct exposure pathway), \
         [[tenant_window_air_conditioner_install_right]] (window-frame safety analog), \
         [[mid_tenancy_temporary_relocation]] (when tenant must vacate during major EEE \
         repair), [[tenant_emotional_distress_damages]] (IIED claim for post-collapse \
         psychological injury), [[rental_storage_unit_lease_disclosure]] (iter 509 — \
         storage area may include exterior wood-framed deck subject to inspection)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::TenantOccupiedUnsafeEeeHabitabilityBreach => input.annual_rent_cents,
        Severity::ImmediateThreatRepairOverdue120Day
        | Severity::FirstCycleInspectionPastDeadlineViolation
        | Severity::HoaSb326UsedNonStructuralEngineerInvalid => input.annual_rent_cents,
        Severity::InspectorNotQualified | Severity::InspectionSampleBelow15Pct => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        minimum_inspection_count_required: minimum_required,
        daily_civil_penalty_min_cents: SB_721_DAILY_PENALTY_MIN_CENTS,
        daily_civil_penalty_max_cents: SB_721_DAILY_PENALTY_MAX_CENTS,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        citation: match input.jurisdiction {
            Jurisdiction::California => {
                "Cal. Health & Safety Code § 17973 SB 721 + Cal. Civ. Code § 5551 SB 326 + AB 2579 + § 1942.4"
            }
            Jurisdiction::NewYorkCityFisp => "NYC LL 11 of 1998 + NYC AC § 28-302.1 + RCNY 1 § 103-04",
            Jurisdiction::Default => "Common-law habitability + premises-liability tort",
        },
        notes,
    }
}

pub type RentalBalconyInspectionSeismicSafetyInput = Input;
pub type RentalBalconyInspectionSeismicSafetyResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            building_type: BuildingType::Multifamily3PlusRentalSb721,
            inspector_qualification: InspectorQualification::LicensedCivilOrStructuralEngineer,
            total_eee_count: 40,
            inspected_eee_count: 6,
            inspection_completed_by_2026_01_01: true,
            immediate_threat_identified: false,
            days_since_immediate_threat_identified: 0,
            tenant_currently_using_unsafe_eee: false,
            annual_rent_cents: 360_000_00,
        }
    }

    #[test]
    fn no_wood_framed_eees_not_applicable() {
        let mut i = baseline();
        i.building_type = BuildingType::NoWoodFramedEees;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn single_family_or_duplex_not_applicable() {
        let mut i = baseline();
        i.building_type = BuildingType::SingleFamilyOrDuplex;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
    }

    #[test]
    fn compliant_first_cycle_inspection_passes() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithFirstCycleInspection
        ));
        assert_eq!(r.minimum_inspection_count_required, 6);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn first_cycle_overdue_violation_full_rent() {
        let mut i = baseline();
        i.inspection_completed_by_2026_01_01 = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::FirstCycleInspectionPastDeadlineViolation
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("AB 2579")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("January 1, 2026")));
    }

    #[test]
    fn inspector_not_qualified_half_rent() {
        let mut i = baseline();
        i.inspector_qualification = InspectorQualification::NotQualified;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::InspectorNotQualified));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 17973(d)")));
    }

    #[test]
    fn inspector_architect_qualified_for_sb_721() {
        let mut i = baseline();
        i.inspector_qualification = InspectorQualification::LicensedArchitect;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithFirstCycleInspection
        ));
    }

    #[test]
    fn inspector_contractor_qualified_for_sb_721() {
        let mut i = baseline();
        i.inspector_qualification = InspectorQualification::ContractorWithABorC5LicenseAndFiveYears;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithFirstCycleInspection
        ));
    }

    #[test]
    fn inspector_certified_building_inspector_qualified_for_sb_721() {
        let mut i = baseline();
        i.inspector_qualification = InspectorQualification::CertifiedBuildingInspector;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithFirstCycleInspection
        ));
    }

    #[test]
    fn hoa_sb_326_architect_invalid_must_be_structural_engineer() {
        let mut i = baseline();
        i.building_type = BuildingType::HoaCondominiumSb326;
        i.inspector_qualification = InspectorQualification::LicensedArchitect;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::HoaSb326UsedNonStructuralEngineerInvalid
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Cal. Civ. Code § 5551")));
    }

    #[test]
    fn hoa_sb_326_structural_engineer_compliant() {
        let mut i = baseline();
        i.building_type = BuildingType::HoaCondominiumSb326;
        i.inspector_qualification = InspectorQualification::LicensedCivilOrStructuralEngineer;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithFirstCycleInspection
        ));
    }

    #[test]
    fn inspection_sample_below_15_pct_violation() {
        let mut i = baseline();
        i.total_eee_count = 40;
        i.inspected_eee_count = 4;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::InspectionSampleBelow15Pct));
        assert_eq!(r.minimum_inspection_count_required, 6);
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
    }

    #[test]
    fn inspection_sample_at_exactly_15_pct_compliant() {
        let mut i = baseline();
        i.total_eee_count = 40;
        i.inspected_eee_count = 6;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithFirstCycleInspection
        ));
    }

    #[test]
    fn immediate_threat_overdue_120_day_full_rent() {
        let mut i = baseline();
        i.immediate_threat_identified = true;
        i.days_since_immediate_threat_identified = 150;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::ImmediateThreatRepairOverdue120Day
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("120 days")));
    }

    #[test]
    fn immediate_threat_at_exactly_120_days_compliant() {
        let mut i = baseline();
        i.immediate_threat_identified = true;
        i.days_since_immediate_threat_identified = 120;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithFirstCycleInspection
        ));
    }

    #[test]
    fn tenant_using_unsafe_eee_habitability_breach() {
        let mut i = baseline();
        i.immediate_threat_identified = true;
        i.tenant_currently_using_unsafe_eee = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::TenantOccupiedUnsafeEeeHabitabilityBreach
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 1942.4")));
    }

    #[test]
    fn minimum_inspection_count_pinned_at_least_one() {
        let mut i = baseline();
        i.total_eee_count = 1;
        i.inspected_eee_count = 1;
        let r = check(&i);
        assert_eq!(r.minimum_inspection_count_required, 1);
    }

    #[test]
    fn ca_jurisdiction_pins_berkeley_collapse_and_ab_2579() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Berkeley")));
        assert!(r.notes.iter().any(|n| n.contains("2015-06-16")));
        assert!(r.notes.iter().any(|n| n.contains("AB 2579")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cal. Health & Safety Code § 17973")));
        assert!(r.notes.iter().any(|n| n.contains("Cal. Civ. Code § 5551")));
    }

    #[test]
    fn nyc_jurisdiction_pins_ll_11_fisp() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYorkCityFisp;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Local Law 11 of 1998")));
        assert!(r.notes.iter().any(|n| n.contains("FISP")));
        assert!(r.notes.iter().any(|n| n.contains("NYC AC § 28-302.1")));
    }

    #[test]
    fn default_jurisdiction_pins_champlain_towers_post_2021() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Champlain Towers")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("common-law habitability")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_basement_water_intrusion_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_window_guard_installation")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("mid_tenancy_temporary_relocation")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_storage_unit_lease_disclosure")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::California,
            Jurisdiction::NewYorkCityFisp,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("rental_basement_water_intrusion_disclosure")),
                "coordination missing for {j:?}"
            );
        }
    }

    #[test]
    fn sb_721_first_cycle_deadline_year_pins_2026() {
        assert_eq!(SB_721_FIRST_CYCLE_DEADLINE_YEAR, 2026);
    }

    #[test]
    fn sb_721_original_deadline_year_pins_2025() {
        assert_eq!(SB_721_FIRST_CYCLE_ORIGINAL_DEADLINE_YEAR, 2025);
    }

    #[test]
    fn sb_721_subsequent_cycle_pins_6_years() {
        assert_eq!(SB_721_SUBSEQUENT_CYCLE_YEARS, 6);
    }

    #[test]
    fn sb_326_inspection_cycle_pins_9_years() {
        assert_eq!(SB_326_INSPECTION_CYCLE_YEARS, 9);
    }

    #[test]
    fn sb_721_minimum_inspection_pct_pins_15_pct() {
        assert_eq!(SB_721_MINIMUM_INSPECTION_PCT_BPS, 1_500);
    }

    #[test]
    fn sb_721_immediate_threat_repair_days_pins_120() {
        assert_eq!(SB_721_IMMEDIATE_THREAT_REPAIR_DAYS, 120);
    }

    #[test]
    fn berkeley_balcony_collapse_date_pins_2015_06_16() {
        assert_eq!(BERKELEY_BALCONY_COLLAPSE_DATE, "2015-06-16");
    }

    #[test]
    fn daily_penalty_constants_pin_100_to_500() {
        assert_eq!(SB_721_DAILY_PENALTY_MIN_CENTS, 10_000);
        assert_eq!(SB_721_DAILY_PENALTY_MAX_CENTS, 50_000);
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let ca = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::California;
            i
        });
        let nyc = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::NewYorkCityFisp;
            i
        });
        let de = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Default;
            i
        });
        assert!(ca.citation.contains("SB 721"));
        assert!(ca.citation.contains("SB 326"));
        assert!(ca.citation.contains("AB 2579"));
        assert!(nyc.citation.contains("LL 11 of 1998"));
        assert!(de.citation.contains("Common-law"));
    }

    #[test]
    fn severity_priority_tenant_unsafe_overrides_all() {
        let mut i = baseline();
        i.immediate_threat_identified = true;
        i.tenant_currently_using_unsafe_eee = true;
        i.inspection_completed_by_2026_01_01 = false;
        i.inspector_qualification = InspectorQualification::NotQualified;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::TenantOccupiedUnsafeEeeHabitabilityBreach
        ));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.inspection_completed_by_2026_01_01 = false;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }
}
