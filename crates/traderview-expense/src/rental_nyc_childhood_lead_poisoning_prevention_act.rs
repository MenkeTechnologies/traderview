//! NYC Local Law 1 of 2004 Childhood Lead Poisoning Prevention Act
//! compliance for trader-landlords with NYC multifamily inventory.
//!
//! Enacted August 2, 2004 as one of the most comprehensive lead-
//! based paint laws in the United States. Replaced the earlier
//! Local Law 38 of 1999 (which had been struck down by NY courts
//! as inadequate). Codified at NYC Admin Code § 27-2056 et seq.
//! plus 28 RCNY Subchapter K (HPD implementing regulations).
//!
//! **Coverage** (NYC Admin Code § 27-2056.2):
//!
//! - Multiple dwellings (3+ units) built BEFORE 1960; OR
//! - Buildings constructed 1960-1977 if lead-based paint is known
//!   to be present; AND
//! - Child under 6 years old "resides" in unit, where "resides"
//!   means living in or routinely spending 10 or more hours per
//!   week in the dwelling unit.
//!
//! **Six core landlord obligations**:
//!
//! 1. **Annual Notice** (28 RCNY § 11-02): each year between
//!    January 1 and February 15, landlord must distribute notice
//!    to all tenants asking whether a child under 6 resides or
//!    spends 10+ hours per week in the unit. Tenant response due
//!    by February 15.
//!
//! 2. **Annual Investigation** (NYC Admin Code § 27-2056.4(d)): if
//!    a child under 6 resides, landlord must perform a visual
//!    inspection of the unit and common areas for lead-based paint
//!    hazards (peeling paint, friction surfaces, etc.).
//!
//! 3. **Turnover Inspection** (28 RCNY § 11-04): when a unit is
//!    vacated and a new tenancy commences with a child under 6
//!    residing, landlord must perform a turnover inspection AND
//!    have any lead-based paint hazards remediated before
//!    occupancy.
//!
//! 4. **Lead Hazard Remediation** (NYC Admin Code § 27-2056.4(g)):
//!    21 days to correct identified lead hazards or HPD may
//!    complete emergency repair and charge owner under § 27-2125
//!    administrative-lien process.
//!
//! 5. **EPA-Certified Renovator** (NYC Admin Code § 27-2056.11):
//!    EPA Lead Renovation, Repair, and Painting (RRP) certified
//!    firm required for any work disturbing more than 100 sqft of
//!    lead-based paint, replacing windows, or fixing HPD lead
//!    violations.
//!
//! 6. **Recordkeeping** (NYC Admin Code § 27-2056.14): maintain
//!    inspection records, remediation records, and tenant notice
//!    records for at least 10 years.
//!
//! **Local Law 31 of 2020 — XRF Testing Requirement**:
//! all dwelling units where a child under 6 resides must have
//! X-ray fluorescence (XRF) lead-paint testing performed by EPA-
//! certified inspector/risk assessor by **August 9, 2025**. Failure
//! to comply is Class "C" immediately hazardous violation carrying
//! civil penalty up to $1,500.
//!
//! **Local Law 66 of 2019 — Lead-Based Paint Threshold**: NYC
//! lowered the lead-paint threshold from the federal HUD 1.0 mg/cm²
//! to **0.5 mg/cm²** (stricter NYC standard), effective December 1,
//! 2021. NYC pre-1960 buildings are presumed to contain lead-based
//! paint at the 0.5 mg/cm² threshold absent professional XRF
//! testing showing otherwise.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const LL1_ENACTMENT_YEAR: u32 = 2004;
#[allow(dead_code)]
pub const PRE_1960_BUILDING_THRESHOLD_YEAR: u32 = 1960;
#[allow(dead_code)]
pub const POST_1978_LEAD_FREE_THRESHOLD_YEAR: u32 = 1978;
#[allow(dead_code)]
pub const CHILD_AGE_THRESHOLD_YEARS: u32 = 6;
#[allow(dead_code)]
pub const RESIDING_HOURS_PER_WEEK_THRESHOLD: u32 = 10;
#[allow(dead_code)]
pub const LEAD_HAZARD_REMEDIATION_DAYS: u32 = 21;
#[allow(dead_code)]
pub const LL31_XRF_TESTING_DEADLINE_YEAR: u32 = 2025;
#[allow(dead_code)]
pub const LL31_XRF_TESTING_DEADLINE_MONTH: u32 = 8;
#[allow(dead_code)]
pub const LL31_XRF_TESTING_DEADLINE_DAY: u32 = 9;
#[allow(dead_code)]
pub const LL31_CIVIL_PENALTY_MAX_CENTS: u64 = 150_000;
#[allow(dead_code)]
pub const LL66_LEAD_THRESHOLD_MG_CM2_X_10: u32 = 5;
#[allow(dead_code)]
pub const HUD_PRIOR_LEAD_THRESHOLD_MG_CM2_X_10: u32 = 10;
#[allow(dead_code)]
pub const EPA_CERTIFIED_RRP_DISTURBANCE_THRESHOLD_SQFT: u32 = 100;
#[allow(dead_code)]
pub const HPD_CLASS_C_HAZARDOUS_CORRECTION_DAYS: u32 = 21;
#[allow(dead_code)]
pub const RECORDKEEPING_RETENTION_YEARS: u32 = 10;
#[allow(dead_code)]
pub const MULTIPLE_DWELLING_UNIT_THRESHOLD: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExemptBuildingPost1960NoKnownLead,
    ExemptUnder3UnitsNotMultipleDwelling,
    ExemptNoChildUnder6Residing,
    CompliantAllObligationsMet,
    ViolationAnnualNoticeNotSentJanFeb,
    ViolationAnnualInvestigationNotPerformed,
    ViolationXrfTestingMissedAug9_2025Deadline,
    ViolationLeadHazardNotRemediatedWithin21Days,
    ViolationEpaCertifiedRrpRenovatorNotUsedForDisturbanceOver100Sqft,
    ViolationRecordkeepingRetentionUnder10Years,
    AggravatedViolationHpdEmergencyRepairChargeback,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub building_year_built: u32,
    pub multiple_dwelling_unit_count: u32,
    pub lead_based_paint_known_present_1960_to_1977: bool,
    pub child_under_6_residing_or_10_plus_hours_weekly: bool,
    pub annual_notice_sent_to_tenants_jan_to_feb_15: bool,
    pub annual_investigation_performed_for_units_with_child: bool,
    pub xrf_testing_completed_by_aug_9_2025: bool,
    pub current_year: u32,
    pub lead_hazard_identified: bool,
    pub days_to_remediate_lead_hazard: u32,
    pub disturbance_over_100_sqft_or_window_replacement: bool,
    pub epa_certified_rrp_renovator_used: bool,
    pub recordkeeping_10_year_complete: bool,
    pub hpd_emergency_repair_triggered: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub total_penalty_cents: u64,
    pub xrf_testing_required: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type NycChildhoodLeadPoisoningPreventionActInput = Input;
pub type NycChildhoodLeadPoisoningPreventionActOutput = Output;
pub type NycChildhoodLeadPoisoningPreventionActResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NYC Local Law 1 of 2004 (Childhood Lead Poisoning Prevention Act)".to_string(),
        "NYC Admin Code § 27-2056 et seq. (statutory codification)".to_string(),
        "NYC Admin Code § 27-2056.2 (coverage definitions)".to_string(),
        "NYC Admin Code § 27-2056.4 (landlord investigation duties)".to_string(),
        "NYC Admin Code § 27-2056.4(g) (21-day remediation deadline)".to_string(),
        "NYC Admin Code § 27-2056.11 (EPA RRP certification requirement)".to_string(),
        "NYC Admin Code § 27-2056.14 (10-year recordkeeping)".to_string(),
        "NYC Admin Code § 27-2125 (HPD emergency repair administrative lien)".to_string(),
        "28 RCNY Subchapter K (HPD implementing regulations)".to_string(),
        "NYC Local Law 31 of 2020 (XRF testing requirement by August 9, 2025)".to_string(),
        "NYC Local Law 66 of 2019 (0.5 mg/cm² lead-paint threshold eff. Dec 1, 2021)".to_string(),
        "NYC Local Law 38 of 1999 (predecessor — struck down)".to_string(),
        "EPA RRP Rule (40 C.F.R. Part 745 Subpart E — RRP certification)".to_string(),
        "Federal Title X § 1018 / 42 U.S.C. § 4852d (federal disclosure floor — see rental_lead_paint_disclosure)".to_string(),
    ];

    let coverage_year = input.building_year_built < PRE_1960_BUILDING_THRESHOLD_YEAR
        || (input.building_year_built < POST_1978_LEAD_FREE_THRESHOLD_YEAR
            && input.lead_based_paint_known_present_1960_to_1977);

    if !coverage_year {
        notes.push(format!(
            "Building year {} ≥ {} threshold and no known lead-based paint — outside Local Law 1 coverage.",
            input.building_year_built, PRE_1960_BUILDING_THRESHOLD_YEAR
        ));
        return Output {
            severity: Severity::ExemptBuildingPost1960NoKnownLead,
            compliant: true,
            total_penalty_cents: 0,
            xrf_testing_required: false,
            notes,
            citations,
        };
    }

    if input.multiple_dwelling_unit_count < MULTIPLE_DWELLING_UNIT_THRESHOLD {
        notes.push(format!(
            "Building {} units < {} multiple-dwelling threshold — outside Local Law 1 coverage.",
            input.multiple_dwelling_unit_count, MULTIPLE_DWELLING_UNIT_THRESHOLD
        ));
        return Output {
            severity: Severity::ExemptUnder3UnitsNotMultipleDwelling,
            compliant: true,
            total_penalty_cents: 0,
            xrf_testing_required: false,
            notes,
            citations,
        };
    }

    if input.hpd_emergency_repair_triggered {
        notes.push("HPD performed emergency repair under § 27-2125 administrative-lien chargeback — aggravated violation status; landlord faces full repair-cost recovery plus civil penalty.".to_string());
        return Output {
            severity: Severity::AggravatedViolationHpdEmergencyRepairChargeback,
            compliant: false,
            total_penalty_cents: LL31_CIVIL_PENALTY_MAX_CENTS,
            xrf_testing_required: true,
            notes,
            citations,
        };
    }

    if !input.child_under_6_residing_or_10_plus_hours_weekly {
        notes.push(format!(
            "No child under {} years old residing or spending {}+ hours per week — Local Law 1 inspection/testing obligations not triggered (annual notice still required).",
            CHILD_AGE_THRESHOLD_YEARS, RESIDING_HOURS_PER_WEEK_THRESHOLD
        ));
        if !input.annual_notice_sent_to_tenants_jan_to_feb_15 {
            return Output {
                severity: Severity::ViolationAnnualNoticeNotSentJanFeb,
                compliant: false,
                total_penalty_cents: LL31_CIVIL_PENALTY_MAX_CENTS,
                xrf_testing_required: false,
                notes,
                citations,
            };
        }
        return Output {
            severity: Severity::ExemptNoChildUnder6Residing,
            compliant: true,
            total_penalty_cents: 0,
            xrf_testing_required: false,
            notes,
            citations,
        };
    }

    if !input.annual_notice_sent_to_tenants_jan_to_feb_15 {
        notes.push("Annual notice not distributed to tenants between January 1 and February 15 — 28 RCNY § 11-02 violation.".to_string());
        return Output {
            severity: Severity::ViolationAnnualNoticeNotSentJanFeb,
            compliant: false,
            total_penalty_cents: LL31_CIVIL_PENALTY_MAX_CENTS,
            xrf_testing_required: true,
            notes,
            citations,
        };
    }

    if !input.annual_investigation_performed_for_units_with_child {
        notes.push("Annual visual investigation not performed for unit with child under 6 — NYC Admin Code § 27-2056.4(d) violation.".to_string());
        return Output {
            severity: Severity::ViolationAnnualInvestigationNotPerformed,
            compliant: false,
            total_penalty_cents: LL31_CIVIL_PENALTY_MAX_CENTS,
            xrf_testing_required: true,
            notes,
            citations,
        };
    }

    if input.current_year >= LL31_XRF_TESTING_DEADLINE_YEAR
        && !input.xrf_testing_completed_by_aug_9_2025
    {
        notes.push(format!(
            "XRF lead-paint testing not completed by August 9, {} Local Law 31 deadline — Class \"C\" immediately hazardous violation; civil penalty up to ${}.",
            LL31_XRF_TESTING_DEADLINE_YEAR,
            LL31_CIVIL_PENALTY_MAX_CENTS / 100
        ));
        return Output {
            severity: Severity::ViolationXrfTestingMissedAug9_2025Deadline,
            compliant: false,
            total_penalty_cents: LL31_CIVIL_PENALTY_MAX_CENTS,
            xrf_testing_required: true,
            notes,
            citations,
        };
    }

    if input.lead_hazard_identified
        && input.days_to_remediate_lead_hazard > LEAD_HAZARD_REMEDIATION_DAYS
    {
        notes.push(format!(
            "Lead hazard identified but not remediated within {} days ({} days elapsed) — NYC Admin Code § 27-2056.4(g) violation; HPD may trigger § 27-2125 emergency repair.",
            LEAD_HAZARD_REMEDIATION_DAYS, input.days_to_remediate_lead_hazard
        ));
        return Output {
            severity: Severity::ViolationLeadHazardNotRemediatedWithin21Days,
            compliant: false,
            total_penalty_cents: LL31_CIVIL_PENALTY_MAX_CENTS,
            xrf_testing_required: true,
            notes,
            citations,
        };
    }

    if input.disturbance_over_100_sqft_or_window_replacement
        && !input.epa_certified_rrp_renovator_used
    {
        notes.push(format!(
            "Renovation work disturbing > {} sqft of lead-based paint or replacing windows performed without EPA-certified RRP renovator — § 27-2056.11 violation.",
            EPA_CERTIFIED_RRP_DISTURBANCE_THRESHOLD_SQFT
        ));
        return Output {
            severity: Severity::ViolationEpaCertifiedRrpRenovatorNotUsedForDisturbanceOver100Sqft,
            compliant: false,
            total_penalty_cents: LL31_CIVIL_PENALTY_MAX_CENTS,
            xrf_testing_required: true,
            notes,
            citations,
        };
    }

    if !input.recordkeeping_10_year_complete {
        notes.push(format!(
            "Inspection and remediation records not retained for full {} years — NYC Admin Code § 27-2056.14 violation.",
            RECORDKEEPING_RETENTION_YEARS
        ));
        return Output {
            severity: Severity::ViolationRecordkeepingRetentionUnder10Years,
            compliant: false,
            total_penalty_cents: LL31_CIVIL_PENALTY_MAX_CENTS,
            xrf_testing_required: true,
            notes,
            citations,
        };
    }

    notes.push("Full NYC Local Law 1 of 2004 compliance: annual notice distributed Jan-Feb 15, annual investigation performed for child-under-6 units, XRF testing per LL 31, 21-day remediation observed, EPA RRP renovator used when required, 10-year records retained.".to_string());
    Output {
        severity: Severity::CompliantAllObligationsMet,
        compliant: true,
        total_penalty_cents: 0,
        xrf_testing_required: true,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_compliant_pre_1960() -> Input {
        Input {
            building_year_built: 1925,
            multiple_dwelling_unit_count: 12,
            lead_based_paint_known_present_1960_to_1977: false,
            child_under_6_residing_or_10_plus_hours_weekly: true,
            annual_notice_sent_to_tenants_jan_to_feb_15: true,
            annual_investigation_performed_for_units_with_child: true,
            xrf_testing_completed_by_aug_9_2025: true,
            current_year: 2026,
            lead_hazard_identified: false,
            days_to_remediate_lead_hazard: 0,
            disturbance_over_100_sqft_or_window_replacement: false,
            epa_certified_rrp_renovator_used: false,
            recordkeeping_10_year_complete: true,
            hpd_emergency_repair_triggered: false,
        }
    }

    #[test]
    fn fully_compliant_baseline() {
        let out = check(&base_compliant_pre_1960());
        assert_eq!(out.severity, Severity::CompliantAllObligationsMet);
        assert!(out.compliant);
        assert_eq!(out.total_penalty_cents, 0);
    }

    #[test]
    fn building_post_1960_no_known_lead_exempt() {
        let mut i = base_compliant_pre_1960();
        i.building_year_built = 1965;
        i.lead_based_paint_known_present_1960_to_1977 = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptBuildingPost1960NoKnownLead);
    }

    #[test]
    fn building_1960_to_1977_with_known_lead_covered() {
        let mut i = base_compliant_pre_1960();
        i.building_year_built = 1965;
        i.lead_based_paint_known_present_1960_to_1977 = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantAllObligationsMet);
    }

    #[test]
    fn under_3_units_not_multiple_dwelling_exempt() {
        let mut i = base_compliant_pre_1960();
        i.multiple_dwelling_unit_count = 2;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptUnder3UnitsNotMultipleDwelling);
    }

    #[test]
    fn no_child_under_6_exempt_if_notice_sent() {
        let mut i = base_compliant_pre_1960();
        i.child_under_6_residing_or_10_plus_hours_weekly = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptNoChildUnder6Residing);
    }

    #[test]
    fn no_child_under_6_but_no_notice_still_violation() {
        let mut i = base_compliant_pre_1960();
        i.child_under_6_residing_or_10_plus_hours_weekly = false;
        i.annual_notice_sent_to_tenants_jan_to_feb_15 = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationAnnualNoticeNotSentJanFeb);
    }

    #[test]
    fn annual_notice_not_sent_violation() {
        let mut i = base_compliant_pre_1960();
        i.annual_notice_sent_to_tenants_jan_to_feb_15 = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationAnnualNoticeNotSentJanFeb);
        assert_eq!(out.total_penalty_cents, 150_000);
    }

    #[test]
    fn annual_investigation_not_performed_violation() {
        let mut i = base_compliant_pre_1960();
        i.annual_investigation_performed_for_units_with_child = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationAnnualInvestigationNotPerformed
        );
    }

    #[test]
    fn xrf_testing_missed_aug_9_2025_deadline_violation() {
        let mut i = base_compliant_pre_1960();
        i.current_year = 2026;
        i.xrf_testing_completed_by_aug_9_2025 = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationXrfTestingMissedAug9_2025Deadline
        );
        assert_eq!(out.total_penalty_cents, 150_000);
    }

    #[test]
    fn xrf_testing_not_required_pre_2025() {
        let mut i = base_compliant_pre_1960();
        i.current_year = 2024;
        i.xrf_testing_completed_by_aug_9_2025 = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantAllObligationsMet);
    }

    #[test]
    fn lead_hazard_22_days_unremediated_violation() {
        let mut i = base_compliant_pre_1960();
        i.lead_hazard_identified = true;
        i.days_to_remediate_lead_hazard = 22;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationLeadHazardNotRemediatedWithin21Days
        );
    }

    #[test]
    fn lead_hazard_at_exactly_21_days_compliant() {
        let mut i = base_compliant_pre_1960();
        i.lead_hazard_identified = true;
        i.days_to_remediate_lead_hazard = 21;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantAllObligationsMet);
    }

    #[test]
    fn epa_rrp_not_used_for_100_sqft_disturbance_violation() {
        let mut i = base_compliant_pre_1960();
        i.disturbance_over_100_sqft_or_window_replacement = true;
        i.epa_certified_rrp_renovator_used = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationEpaCertifiedRrpRenovatorNotUsedForDisturbanceOver100Sqft
        );
    }

    #[test]
    fn recordkeeping_under_10_years_violation() {
        let mut i = base_compliant_pre_1960();
        i.recordkeeping_10_year_complete = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationRecordkeepingRetentionUnder10Years
        );
    }

    #[test]
    fn hpd_emergency_repair_aggravated_violation() {
        let mut i = base_compliant_pre_1960();
        i.hpd_emergency_repair_triggered = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::AggravatedViolationHpdEmergencyRepairChargeback
        );
    }

    #[test]
    fn citations_pin_ll1_admin_code_subsections() {
        let out = check(&base_compliant_pre_1960());
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Local Law 1 of 2004")));
        assert!(out.citations.iter().any(|c| c.contains("§ 27-2056")));
        assert!(out.citations.iter().any(|c| c.contains("§ 27-2056.4")));
        assert!(out.citations.iter().any(|c| c.contains("§ 27-2056.11")));
        assert!(out.citations.iter().any(|c| c.contains("§ 27-2056.14")));
        assert!(out.citations.iter().any(|c| c.contains("§ 27-2125")));
    }

    #[test]
    fn citations_pin_ll31_2020_xrf_and_ll66_2019_threshold() {
        let out = check(&base_compliant_pre_1960());
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Local Law 31 of 2020")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Local Law 66 of 2019")));
        assert!(out.citations.iter().any(|c| c.contains("0.5 mg/cm²")));
    }

    #[test]
    fn citations_pin_28_rcny_and_epa_rrp() {
        let out = check(&base_compliant_pre_1960());
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("28 RCNY Subchapter K")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("40 C.F.R. Part 745 Subpart E")));
    }

    #[test]
    fn constant_pin_pre_1960_threshold() {
        assert_eq!(PRE_1960_BUILDING_THRESHOLD_YEAR, 1960);
    }

    #[test]
    fn constant_pin_child_age_6_threshold() {
        assert_eq!(CHILD_AGE_THRESHOLD_YEARS, 6);
    }

    #[test]
    fn constant_pin_10_hours_per_week_residing() {
        assert_eq!(RESIDING_HOURS_PER_WEEK_THRESHOLD, 10);
    }

    #[test]
    fn constant_pin_21_day_remediation() {
        assert_eq!(LEAD_HAZARD_REMEDIATION_DAYS, 21);
    }

    #[test]
    fn constant_pin_xrf_2025_deadline() {
        assert_eq!(LL31_XRF_TESTING_DEADLINE_YEAR, 2025);
    }

    #[test]
    fn constant_pin_1500_civil_penalty() {
        assert_eq!(LL31_CIVIL_PENALTY_MAX_CENTS, 150_000);
    }

    #[test]
    fn constant_pin_05_mg_cm2_threshold_x_10() {
        assert_eq!(LL66_LEAD_THRESHOLD_MG_CM2_X_10, 5);
    }

    #[test]
    fn constant_pin_3_unit_multiple_dwelling_threshold() {
        assert_eq!(MULTIPLE_DWELLING_UNIT_THRESHOLD, 3);
    }

    #[test]
    fn constant_pin_100_sqft_epa_rrp_threshold() {
        assert_eq!(EPA_CERTIFIED_RRP_DISTURBANCE_THRESHOLD_SQFT, 100);
    }

    #[test]
    fn constant_pin_10_year_recordkeeping() {
        assert_eq!(RECORDKEEPING_RETENTION_YEARS, 10);
    }
}
