//! Boiler and gas-piping inspection compliance framework for residential rentals.
//!
//! Boilers and gas-piping systems in residential rentals are subject to periodic
//! inspection requirements imposed by federal, state, and municipal codes. The
//! ASME Boiler and Pressure Vessel Code (BPVC) Section IV (heating boilers) +
//! Section VI (recommended rules for low-pressure boiler care) provides the
//! foundational engineering standard adopted by 49 states + DC. Municipal codes
//! layer additional inspection cycles, particularly NYC Local Law 152 of 2016
//! (gas-piping inspections every 4 years).
//!
//! Jurisdictional grid:
//!
//! - NYC LOCAL LAW 152 of 2016: gas-piping systems in all buildings (except
//!   one- and two-family homes and other R-3 occupancies) must be inspected by
//!   a Licensed Master Plumber (LMP) at least once every 4 years on a
//!   community-district-based schedule. Inspection report due within 30 days;
//!   GPS1 Certification filing with DOB due within 60 days. NYC Admin. Code
//!   § 28-318.
//! - NY STATE INDUSTRIAL CODE RULE 4: annual boiler inspection for high-pressure
//!   steam boilers; biennial inspection for low-pressure boilers.
//! - CA CAL/OSHA TITLE 8 SUBCHAPTER 1 §§ 750-784: boiler operating permit + annual
//!   inspection for boilers above 15 psi steam or 160 psi water (Pressure Vessel
//!   Unit jurisdiction).
//! - IL 430 ILCS 75 BOILER AND PRESSURE VESSEL SAFETY ACT: state Boiler Inspection
//!   Section; annual inspection + 6-year internal inspection cycle.
//! - MA GEN. L. ch. 146 § 46: annual external + internal inspection by Massachusetts
//!   Department of Public Safety, Office of Public Safety and Inspections.
//! - TX HEALTH & SAFETY CODE § 755: boiler inspection program by Texas Department
//!   of Licensing and Regulation (TDLR); annual / biennial inspection cycle.
//! - DEFAULT (ASME BPVC Section IV adopted): annual external inspection +
//!   periodic internal inspection per state-of-jurisdiction adoption of ASME BPVC.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - nyc.gov/site/buildings/property-or-business-owner/gas-piping-inspections.page
//! - jtgmp.com/local-law-152
//! - energo.com/blog/nyc-local-law-152-new-rules-2026-deadlines-how-to-stay-compliant/
//! - skybriz.com/insights/local-law-152-gas-piping-inspection-nyc/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NycLocalLaw152,
    NewYorkStateIcr4,
    California,
    Illinois,
    Massachusetts,
    Texas,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingType {
    /// Multi-family > 2 units (subject to NYC LL 152).
    MultiFamilyMoreThanTwoUnits,
    /// One- or two-family home (R-3 occupancy — exempt from NYC LL 152 gas-piping).
    OneOrTwoFamilyHomeR3OccupancyExempt,
    /// Commercial / mixed-use building.
    CommercialOrMixedUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionStatus {
    /// Inspection performed by qualified inspector + report filed timely.
    InspectionPerformedAndReportFiledTimely,
    /// Inspection performed but report not yet filed within 60 days.
    InspectionPerformedReportNotFiledWithinSixtyDays,
    /// Inspection performed by unqualified individual (not LMP for NYC LL 152).
    InspectionPerformedByUnqualifiedIndividualNotLicensedMasterPlumber,
    /// Inspection not performed by required cycle deadline.
    InspectionNotPerformedByCycleDeadline,
    /// Required corrections from prior inspection not addressed.
    CorrectionsFromPriorInspectionNotAddressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    OneOrTwoFamilyExemptNoInspectionRequired,
    CompliantInspectionAndReportFiled,
    LateFilingWithinThirtyDayCureWindow,
    NycLocalLaw152UnqualifiedInspectorViolation,
    InspectionMissedDeadlineDobCivilPenaltyExposure,
    UnaddressedCorrectionsClassCViolationEscalatedEnforcement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub building_type: BuildingType,
    pub inspection_status: InspectionStatus,
    pub days_after_inspection_report_filed: u32,
    pub days_past_cycle_deadline: u32,
}

pub type RentalBoilerInspectionComplianceInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub inspection_cycle_years: u32,
    pub filing_window_days: u32,
    pub estimated_civil_penalty_cents: u64,
    pub note: String,
}

pub type RentalBoilerInspectionComplianceOutput = Output;
pub type RentalBoilerInspectionComplianceResult = Output;

const NYC_LL152_INSPECTION_CYCLE_YEARS: u32 = 4;
const NYC_LL152_REPORT_FILING_WINDOW_DAYS: u32 = 60;
const NYC_LL152_INSPECTION_REPORT_DELIVERY_DAYS: u32 = 30;
const NY_STATE_HIGH_PRESSURE_BOILER_CYCLE_YEARS: u32 = 1;
#[allow(dead_code)]
const NY_STATE_LOW_PRESSURE_BOILER_CYCLE_YEARS: u32 = 2;
const CA_CALOSHA_INSPECTION_CYCLE_YEARS: u32 = 1;
const IL_EXTERNAL_INSPECTION_CYCLE_YEARS: u32 = 1;
#[allow(dead_code)]
const IL_INTERNAL_INSPECTION_CYCLE_YEARS: u32 = 6;
const MA_INSPECTION_CYCLE_YEARS: u32 = 1;
const DEFAULT_INSPECTION_CYCLE_YEARS: u32 = 1;
const NYC_LL152_LATE_FILING_PENALTY_CENTS: u64 = 100_000;
const NYC_LL152_NO_INSPECTION_PENALTY_CENTS: u64 = 1_000_000;
const NYC_LL152_UNQUALIFIED_INSPECTOR_PENALTY_CENTS: u64 = 500_000;
const NYC_CLASS_C_VIOLATION_DAILY_PENALTY_CENTS: u64 = 25_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.building_type,
        BuildingType::OneOrTwoFamilyHomeR3OccupancyExempt
    ) && matches!(input.jurisdiction, Jurisdiction::NycLocalLaw152)
    {
        return Output {
            severity: Severity::OneOrTwoFamilyExemptNoInspectionRequired,
            inspection_cycle_years: 0,
            filing_window_days: 0,
            estimated_civil_penalty_cents: 0,
            note: "NYC LL 152 EXEMPTION: one- and two-family homes (R-3 occupancy) are \
                   EXEMPT from the gas-piping inspection requirement. NYC Admin. Code \
                   § 28-318. State-law boiler inspection requirements still apply (NY State \
                   Industrial Code Rule 4 + ASME BPVC adoption). Maintain manufacturer's \
                   service records + carbon-monoxide detector batteries."
                .to_string(),
        };
    }

    let (cycle_years, filing_window) = jurisdiction_cycle(input.jurisdiction);

    if matches!(
        input.inspection_status,
        InspectionStatus::InspectionPerformedByUnqualifiedIndividualNotLicensedMasterPlumber
    ) && matches!(input.jurisdiction, Jurisdiction::NycLocalLaw152)
    {
        return Output {
            severity: Severity::NycLocalLaw152UnqualifiedInspectorViolation,
            inspection_cycle_years: cycle_years,
            filing_window_days: filing_window,
            estimated_civil_penalty_cents: NYC_LL152_UNQUALIFIED_INSPECTOR_PENALTY_CENTS,
            note: format!(
                "NYC Local Law 152 VIOLATION. Gas-piping inspection must be performed by a \
                 Licensed Master Plumber (LMP) or a qualified individual working under an \
                 LMP's license. Inspection by an unqualified individual is non-compliant; \
                 the certification cannot be filed. Estimated DOB civil penalty: ${} + \
                 re-inspection cost + Class B / C HPD violations if conditions identified \
                 remain unaddressed.",
                NYC_LL152_UNQUALIFIED_INSPECTOR_PENALTY_CENTS / 100
            ),
        };
    }

    if matches!(
        input.inspection_status,
        InspectionStatus::InspectionNotPerformedByCycleDeadline
    ) {
        let days_late_penalty = u64::from(input.days_past_cycle_deadline)
            .saturating_mul(NYC_CLASS_C_VIOLATION_DAILY_PENALTY_CENTS)
            .min(NYC_LL152_NO_INSPECTION_PENALTY_CENTS.saturating_mul(10));
        let estimated_penalty = NYC_LL152_NO_INSPECTION_PENALTY_CENTS
            .saturating_add(days_late_penalty);
        return Output {
            severity: Severity::InspectionMissedDeadlineDobCivilPenaltyExposure,
            inspection_cycle_years: cycle_years,
            filing_window_days: filing_window,
            estimated_civil_penalty_cents: estimated_penalty,
            note: format!(
                "Missed inspection cycle deadline by {} days. Estimated civil penalty ${} \
                 reflects base statutory penalty (${}) + ${}/day Class-C-equivalent \
                 escalation. NYC LL 152 enforcement is significantly stricter in Cycle 2 than \
                 Cycle 1. NYC DOB may issue stop-use order; FDNY may shut off gas service \
                 if hazardous conditions identified. Tenants exposed to gas leak risk; \
                 personal-injury / wrongful-death exposure independent of statutory penalty.",
                input.days_past_cycle_deadline,
                estimated_penalty / 100,
                NYC_LL152_NO_INSPECTION_PENALTY_CENTS / 100,
                NYC_CLASS_C_VIOLATION_DAILY_PENALTY_CENTS / 100
            ),
        };
    }

    if matches!(
        input.inspection_status,
        InspectionStatus::CorrectionsFromPriorInspectionNotAddressed
    ) {
        return Output {
            severity: Severity::UnaddressedCorrectionsClassCViolationEscalatedEnforcement,
            inspection_cycle_years: cycle_years,
            filing_window_days: filing_window,
            estimated_civil_penalty_cents: NYC_LL152_NO_INSPECTION_PENALTY_CENTS,
            note: format!(
                "Required corrections from prior inspection NOT addressed. Conditions \
                 identified by LMP must be repaired and re-inspected within the timeframe \
                 specified in the inspection report. Unaddressed Class B / Class C conditions \
                 escalate to HPD class-C immediately-hazardous enforcement; daily civil \
                 penalty + FDNY emergency shutoff authority + personal-injury tort exposure. \
                 Estimated penalty ${} excludes daily escalation and tort exposure.",
                NYC_LL152_NO_INSPECTION_PENALTY_CENTS / 100
            ),
        };
    }

    if matches!(
        input.inspection_status,
        InspectionStatus::InspectionPerformedReportNotFiledWithinSixtyDays
    ) {
        if input.days_after_inspection_report_filed
            <= filing_window + NYC_LL152_INSPECTION_REPORT_DELIVERY_DAYS
        {
            return Output {
                severity: Severity::LateFilingWithinThirtyDayCureWindow,
                inspection_cycle_years: cycle_years,
                filing_window_days: filing_window,
                estimated_civil_penalty_cents: NYC_LL152_LATE_FILING_PENALTY_CENTS,
                note: format!(
                    "Late filing within 30-day cure window. LMP report received but not filed \
                     with DOB within {filing_window} days of inspection. Estimated late- \
                     filing penalty ${} per occurrence. Cure by filing immediately; document \
                     the LMP delivery delay if applicable.",
                    NYC_LL152_LATE_FILING_PENALTY_CENTS / 100
                ),
            };
        }
        return Output {
            severity: Severity::InspectionMissedDeadlineDobCivilPenaltyExposure,
            inspection_cycle_years: cycle_years,
            filing_window_days: filing_window,
            estimated_civil_penalty_cents: NYC_LL152_NO_INSPECTION_PENALTY_CENTS,
            note: format!(
                "Filing window EXCEEDED by more than 30 days. Estimated civil penalty ${}. \
                 The 30-day cure window has elapsed; the certification is treated as not \
                 timely filed. Re-engage LMP and file immediately to mitigate further \
                 penalty escalation.",
                NYC_LL152_NO_INSPECTION_PENALTY_CENTS / 100
            ),
        };
    }

    Output {
        severity: Severity::CompliantInspectionAndReportFiled,
        inspection_cycle_years: cycle_years,
        filing_window_days: filing_window,
        estimated_civil_penalty_cents: 0,
        note: format!(
            "Compliant: inspection performed by qualified inspector and certification filed \
             timely. {} Retain LMP inspection report and DOB filing receipt for {}-year cycle \
             (longer of statute of limitations or next inspection cycle).",
            jurisdiction_citation(input.jurisdiction),
            cycle_years
        ),
    }
}

fn jurisdiction_cycle(jurisdiction: Jurisdiction) -> (u32, u32) {
    match jurisdiction {
        Jurisdiction::NycLocalLaw152 => (
            NYC_LL152_INSPECTION_CYCLE_YEARS,
            NYC_LL152_REPORT_FILING_WINDOW_DAYS,
        ),
        Jurisdiction::NewYorkStateIcr4 => (NY_STATE_HIGH_PRESSURE_BOILER_CYCLE_YEARS, 30),
        Jurisdiction::California => (CA_CALOSHA_INSPECTION_CYCLE_YEARS, 30),
        Jurisdiction::Illinois => (IL_EXTERNAL_INSPECTION_CYCLE_YEARS, 30),
        Jurisdiction::Massachusetts => (MA_INSPECTION_CYCLE_YEARS, 30),
        Jurisdiction::Texas => (1, 30),
        Jurisdiction::Default => (DEFAULT_INSPECTION_CYCLE_YEARS, 30),
    }
}

fn jurisdiction_citation(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::NycLocalLaw152 => {
            "NYC Local Law 152 of 2016 + NYC Admin. Code § 28-318: gas-piping inspection by \
             LMP every 4 years on community-district-based schedule."
        }
        Jurisdiction::NewYorkStateIcr4 => {
            "NY State Industrial Code Rule 4: annual high-pressure boiler / biennial \
             low-pressure boiler inspection."
        }
        Jurisdiction::California => {
            "California Cal/OSHA Title 8 Subchapter 1 §§ 750-784: Pressure Vessel Unit \
             jurisdiction; annual inspection for boilers above 15 psi steam or 160 psi water."
        }
        Jurisdiction::Illinois => {
            "Illinois 430 ILCS 75 Boiler and Pressure Vessel Safety Act: annual external \
             + 6-year internal inspection cycle."
        }
        Jurisdiction::Massachusetts => {
            "MA Gen. L. ch. 146 § 46: annual external + internal inspection by Department \
             of Public Safety, Office of Public Safety and Inspections."
        }
        Jurisdiction::Texas => {
            "Texas Health & Safety Code § 755 + Texas Department of Licensing and Regulation \
             (TDLR) boiler inspection program."
        }
        Jurisdiction::Default => {
            "ASME Boiler & Pressure Vessel Code Section IV adopted by state of jurisdiction; \
             annual external inspection + periodic internal inspection."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_nyc() -> Input {
        Input {
            jurisdiction: Jurisdiction::NycLocalLaw152,
            building_type: BuildingType::MultiFamilyMoreThanTwoUnits,
            inspection_status: InspectionStatus::InspectionPerformedAndReportFiledTimely,
            days_after_inspection_report_filed: 45,
            days_past_cycle_deadline: 0,
        }
    }

    #[test]
    fn nyc_one_or_two_family_exempt() {
        let mut input = base_nyc();
        input.building_type = BuildingType::OneOrTwoFamilyHomeR3OccupancyExempt;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::OneOrTwoFamilyExemptNoInspectionRequired
        );
        assert!(output.note.contains("R-3"));
        assert!(output.note.contains("§ 28-318"));
    }

    #[test]
    fn nyc_compliant_inspection_and_filing() {
        let input = base_nyc();
        let output = check(&input);
        assert_eq!(output.severity, Severity::CompliantInspectionAndReportFiled);
        assert_eq!(output.inspection_cycle_years, 4);
        assert_eq!(output.filing_window_days, 60);
        assert!(output.note.contains("Local Law 152"));
    }

    #[test]
    fn nyc_unqualified_inspector_violation() {
        let mut input = base_nyc();
        input.inspection_status =
            InspectionStatus::InspectionPerformedByUnqualifiedIndividualNotLicensedMasterPlumber;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NycLocalLaw152UnqualifiedInspectorViolation
        );
        assert_eq!(output.estimated_civil_penalty_cents, 5_000_00);
        assert!(output.note.contains("LMP"));
    }

    #[test]
    fn nyc_missed_cycle_deadline_civil_penalty() {
        let mut input = base_nyc();
        input.inspection_status = InspectionStatus::InspectionNotPerformedByCycleDeadline;
        input.days_past_cycle_deadline = 30;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::InspectionMissedDeadlineDobCivilPenaltyExposure
        );
        // $10K base + 30 × $250 = $10,000 + $7,500 = $17,500
        assert_eq!(output.estimated_civil_penalty_cents, 17_500_00);
        assert!(output.note.contains("Class-C"));
    }

    #[test]
    fn nyc_unaddressed_corrections_escalated_enforcement() {
        let mut input = base_nyc();
        input.inspection_status =
            InspectionStatus::CorrectionsFromPriorInspectionNotAddressed;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::UnaddressedCorrectionsClassCViolationEscalatedEnforcement
        );
        assert!(output.note.contains("Class C"));
        assert!(output.note.contains("FDNY"));
    }

    #[test]
    fn nyc_late_filing_within_cure_window() {
        let mut input = base_nyc();
        input.inspection_status =
            InspectionStatus::InspectionPerformedReportNotFiledWithinSixtyDays;
        input.days_after_inspection_report_filed = 75;
        let output = check(&input);
        // 75 < 60 + 30 = 90 → within cure window
        assert_eq!(output.severity, Severity::LateFilingWithinThirtyDayCureWindow);
        assert_eq!(output.estimated_civil_penalty_cents, 1_000_00);
    }

    #[test]
    fn nyc_late_filing_outside_cure_window() {
        let mut input = base_nyc();
        input.inspection_status =
            InspectionStatus::InspectionPerformedReportNotFiledWithinSixtyDays;
        input.days_after_inspection_report_filed = 95;
        let output = check(&input);
        // 95 > 90 → outside cure window → no-inspection penalty
        assert_eq!(
            output.severity,
            Severity::InspectionMissedDeadlineDobCivilPenaltyExposure
        );
    }

    #[test]
    fn new_york_state_icr4_annual_cycle() {
        let mut input = base_nyc();
        input.jurisdiction = Jurisdiction::NewYorkStateIcr4;
        let output = check(&input);
        assert_eq!(output.inspection_cycle_years, 1);
        assert!(output.note.contains("Industrial Code Rule 4"));
    }

    #[test]
    fn california_calosha_annual_cycle() {
        let mut input = base_nyc();
        input.jurisdiction = Jurisdiction::California;
        let output = check(&input);
        assert_eq!(output.inspection_cycle_years, 1);
        assert!(output.note.contains("Cal/OSHA"));
        assert!(output.note.contains("§§ 750-784"));
    }

    #[test]
    fn illinois_430_ilcs_75_annual_external() {
        let mut input = base_nyc();
        input.jurisdiction = Jurisdiction::Illinois;
        let output = check(&input);
        assert!(output.note.contains("430 ILCS 75"));
    }

    #[test]
    fn massachusetts_ch_146_section_46() {
        let mut input = base_nyc();
        input.jurisdiction = Jurisdiction::Massachusetts;
        let output = check(&input);
        assert!(output.note.contains("ch. 146 § 46"));
    }

    #[test]
    fn texas_health_safety_755_tdlr() {
        let mut input = base_nyc();
        input.jurisdiction = Jurisdiction::Texas;
        let output = check(&input);
        assert!(output.note.contains("§ 755"));
        assert!(output.note.contains("TDLR"));
    }

    #[test]
    fn default_jurisdiction_asme_bpvc() {
        let mut input = base_nyc();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert!(output.note.contains("ASME"));
        assert!(output.note.contains("Section IV"));
    }

    #[test]
    fn nyc_ll152_cycle_constant_pins_4_years() {
        assert_eq!(NYC_LL152_INSPECTION_CYCLE_YEARS, 4);
    }

    #[test]
    fn nyc_ll152_filing_window_constant_pins_60_days() {
        assert_eq!(NYC_LL152_REPORT_FILING_WINDOW_DAYS, 60);
    }

    #[test]
    fn nyc_ll152_report_delivery_constant_pins_30_days() {
        assert_eq!(NYC_LL152_INSPECTION_REPORT_DELIVERY_DAYS, 30);
    }

    #[test]
    fn ny_state_high_pressure_cycle_constant_pins_1_year() {
        assert_eq!(NY_STATE_HIGH_PRESSURE_BOILER_CYCLE_YEARS, 1);
    }

    #[test]
    fn ny_state_low_pressure_cycle_constant_pins_2_years() {
        assert_eq!(NY_STATE_LOW_PRESSURE_BOILER_CYCLE_YEARS, 2);
    }

    #[test]
    fn il_external_cycle_constant_pins_1_year() {
        assert_eq!(IL_EXTERNAL_INSPECTION_CYCLE_YEARS, 1);
    }

    #[test]
    fn il_internal_cycle_constant_pins_6_years() {
        assert_eq!(IL_INTERNAL_INSPECTION_CYCLE_YEARS, 6);
    }

    #[test]
    fn nyc_ll152_no_inspection_penalty_constant_pins_10000() {
        assert_eq!(NYC_LL152_NO_INSPECTION_PENALTY_CENTS, 1_000_000);
    }

    #[test]
    fn nyc_class_c_daily_penalty_constant_pins_250() {
        assert_eq!(NYC_CLASS_C_VIOLATION_DAILY_PENALTY_CENTS, 25_000);
    }

    #[test]
    fn very_large_days_past_deadline_no_overflow() {
        let mut input = base_nyc();
        input.inspection_status = InspectionStatus::InspectionNotPerformedByCycleDeadline;
        input.days_past_cycle_deadline = u32::MAX;
        let output = check(&input);
        // saturating_mul + .min() defense prevents overflow
        assert!(output.estimated_civil_penalty_cents > 0);
    }

    #[test]
    fn zero_days_late_no_panic() {
        let mut input = base_nyc();
        input.inspection_status = InspectionStatus::InspectionNotPerformedByCycleDeadline;
        input.days_past_cycle_deadline = 0;
        let output = check(&input);
        // Base penalty only, no escalation
        assert_eq!(output.estimated_civil_penalty_cents, 10_000_00);
    }
}
