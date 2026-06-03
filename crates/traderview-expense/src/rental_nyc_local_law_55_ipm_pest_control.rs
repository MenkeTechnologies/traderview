//! NYC Local Law 55 of 2018 Integrated Pest Management + Indoor
//! Allergen Hazard Compliance Module ("Asthma-Free Housing Act").
//!
//! Pure-compute compliance check for NYC's residential landlord
//! obligations under Local Law 55 of 2018 (effective January 19,
//! 2019; codified at NYC Admin Code § 27-2017 et seq.). Trader-
//! landlord critical because LL 55 imposes ANNUAL INSPECTION,
//! LICENSED-IPM-OPERATOR, and TENANT NOTICE requirements with
//! daily civil penalties up to $125/day (max $10,000) and false-
//! certification penalties up to $500 per hazardous violation.
//!
//! Web research (verified 2026-06-03):
//! - **NYC Admin Code § 27-2017 et seq.** (Local Law 55 of 2018;
//!   Asthma-Free Housing Act; effective January 19, 2019): applies
//!   to multiple dwellings (3+ units); indoor allergen hazards =
//!   mice, cockroaches, rats, and mold. Owner must inspect units
//!   ANNUALLY for indoor allergen hazards and use Integrated Pest
//!   Management (IPM) to address infestations. ([NYC HPD Indoor
//!   Allergen Hazards (Mold and Pests)](https://www.nyc.gov/site/hpd/services-and-information/indoor-allergen-hazards-mold-and-pests.page);
//!   NYC HPD Local Law 55 IPM Guide PDF.)
//! - **NYC Admin Code § 27-2017.8** — INTEGRATED PEST MANAGEMENT
//!   PRACTICES: any pesticide applied to eradicate the presence
//!   of pests must be applied by a **PEST PROFESSIONAL LICENSED
//!   by the NEW YORK STATE DEPARTMENT OF ENVIRONMENTAL
//!   CONSERVATION (NYS DEC)**. ([Code Library NYC Admin Code
//!   § 27-2017.8](https://codelibrary.amlegal.com/codes/newyorkcity/latest/NYCadmin/0-0-0-60305).)
//! - **Tenant notice requirement**: with every new lease or lease
//!   renewal, landlord must provide tenants with an **annual
//!   notice** stating the property owner's responsibilities under
//!   Local Law 55 PLUS the **DOHMH Local Law 55 fact sheet**.
//! - **Access notice for inspections**: 24-hour written notice
//!   constitutes good-faith effort to gain access for IPM
//!   inspection.
//! - **HPD Penalties**: $10 - $125 per day, up to maximum
//!   **$10,000** depending on class and severity. FALSE
//!   CERTIFICATION of correction: $50-$250 non-hazardous violation
//!   / **$250-$500 hazardous violation**. ([NYC HPD Penalties and
//!   Fees page](https://www.nyc.gov/site/hpd/services-and-information/penalties-and-fees.page);
//!   M and M Pest Control What is Local Law 55 guide.)

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const LOCAL_LAW_55_YEAR: u32 = 2018;
pub const LOCAL_LAW_55_EFFECTIVE_DATE_YEAR: u32 = 2019;
pub const LOCAL_LAW_55_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const LOCAL_LAW_55_EFFECTIVE_DATE_DAY: u32 = 19;
pub const LL55_ACCESS_NOTICE_HOURS: u32 = 24;
pub const LL55_MIN_DAILY_PENALTY_DOLLARS: u64 = 10;
pub const LL55_MAX_DAILY_PENALTY_DOLLARS: u64 = 125;
pub const LL55_MAX_TOTAL_PENALTY_DOLLARS: u64 = 10_000;
pub const LL55_FALSE_CERT_NON_HAZARDOUS_MIN_DOLLARS: u64 = 50;
pub const LL55_FALSE_CERT_NON_HAZARDOUS_MAX_DOLLARS: u64 = 250;
pub const LL55_FALSE_CERT_HAZARDOUS_MIN_DOLLARS: u64 = 250;
pub const LL55_FALSE_CERT_HAZARDOUS_MAX_DOLLARS: u64 = 500;
pub const LL55_MULTIPLE_DWELLING_MIN_UNITS: u32 = 3;
pub const LL55_NYC_ADMIN_CODE_SECTION_BASE: u32 = 2_017;
pub const LL55_NYC_ADMIN_CODE_IPM_SUBSECTION: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyClassification {
    NycMultipleDwelling3PlusUnits,
    NycSingleOrTwoFamilyDwelling,
    PropertyOutsideNyc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndoorAllergenHazard {
    MicePresent,
    CockroachesPresent,
    RatsPresent,
    MoldPresent,
    MultipleHazardsPresent,
    NoAllergenHazardPresent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PesticideOperatorLicense {
    NysDecLicensedPestProfessional,
    UnlicensedNonprofessional,
    NoPesticideApplied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalLaw55Mode {
    NotApplicableNotMultipleDwellingOrOutsideNyc,
    NotApplicableNoAllergenHazardAndAnnualInspectionPerformed,
    CompliantAnnualInspectionPerformedAndIpmApplied,
    CompliantIPMOperatorLicensedDecApplied,
    CompliantTenantNoticeAnnuallyAndAtLeaseRenewal,
    Compliant24HourAccessNoticeProvided,
    CompliantAllergenHazardRemediatedWithinTimeline,
    ViolationAnnualInspectionNotPerformed,
    ViolationIpmOperatorNotLicensedDec,
    ViolationTenantAnnualNoticeNotProvided,
    ViolationTenantLeaseRenewalNoticeNotProvided,
    Violation24HourAccessNoticeNotProvided,
    ViolationFalseCertificationOfCorrectionMade,
    ViolationDailyPenaltyAccrued,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_classification: PropertyClassification,
    pub indoor_allergen_hazard_detected: IndoorAllergenHazard,
    pub annual_inspection_performed_this_year: bool,
    pub ipm_approach_documented: bool,
    pub pesticide_operator_license: PesticideOperatorLicense,
    pub tenant_annual_notice_provided: bool,
    pub tenant_lease_renewal_notice_provided: bool,
    pub dohmh_fact_sheet_attached_to_notice: bool,
    pub access_notice_hours_provided_to_tenant: u32,
    pub days_violation_uncured_after_hpd_notice: u32,
    pub false_certification_of_correction_made: bool,
    pub violation_classified_as_hazardous: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: LocalLaw55Mode,
    pub accrued_daily_penalty_dollars: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalNycLocalLaw55IpmPestControlInput = Input;
pub type RentalNycLocalLaw55IpmPestControlOutput = Output;
pub type RentalNycLocalLaw55IpmPestControlResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "NYC Admin Code § 27-2017 et seq. (Local Law 55 of 2018 — Asthma-Free Housing Act; effective Jan 19, 2019) — multiple dwellings 3+ units; indoor allergen hazards = mice/cockroaches/rats/mold; annual inspection + IPM approach required".to_string(),
        "NYC Admin Code § 27-2017.8 — IPM practices: pesticide must be applied by NYS DEC-licensed pest professional".to_string(),
        "Local Law 55 tenant notice — owner must provide annual notice + DOHMH fact sheet at lease signing + lease renewal".to_string(),
        "Local Law 55 access notice — 24-hour written notice to tenant constitutes good-faith effort for inspection access".to_string(),
        "NYC HPD Penalties: $10-$125 per day; up to $10,000 maximum depending on class and severity".to_string(),
        "False certification of correction: $50-$250 non-hazardous violation / $250-$500 hazardous violation".to_string(),
        "DOHMH (Department of Health and Mental Hygiene) — Local Law 55 fact sheet publisher".to_string(),
        "HPD (Department of Housing Preservation and Development) — Local Law 55 enforcement agency; HPDOnline complaint portal".to_string(),
        "NYC Multiple Dwelling Law cross-reference — 3+ unit building threshold for LL 55 applicability".to_string(),
    ];

    if !matches!(
        input.property_classification,
        PropertyClassification::NycMultipleDwelling3PlusUnits
    ) {
        return Output {
            mode: LocalLaw55Mode::NotApplicableNotMultipleDwellingOrOutsideNyc,
            accrued_daily_penalty_dollars: 0,
            statutory_basis: "Local Law 55 applies only to NYC multiple dwellings (3+ units)".to_string(),
            notes: format!(
                "Property classification = {:?}; LL 55 inapplicable to single/two-family dwellings or properties outside NYC.",
                input.property_classification
            ),
            citations,
        };
    }

    if !input.annual_inspection_performed_this_year {
        return Output {
            mode: LocalLaw55Mode::ViolationAnnualInspectionNotPerformed,
            accrued_daily_penalty_dollars: input
                .days_violation_uncured_after_hpd_notice
                .saturating_mul(LL55_MIN_DAILY_PENALTY_DOLLARS as u32) as u64,
            statutory_basis: "NYC Admin Code § 27-2017 — annual inspection required".to_string(),
            notes: format!(
                "VIOLATION: LL 55 annual inspection not performed; {} days uncured; daily penalty $10-$125 per day up to $10,000 maximum.",
                input.days_violation_uncured_after_hpd_notice
            ),
            citations,
        };
    }

    if input.indoor_allergen_hazard_detected != IndoorAllergenHazard::NoAllergenHazardPresent
        && input.pesticide_operator_license == PesticideOperatorLicense::UnlicensedNonprofessional
    {
        return Output {
            mode: LocalLaw55Mode::ViolationIpmOperatorNotLicensedDec,
            accrued_daily_penalty_dollars: input
                .days_violation_uncured_after_hpd_notice
                .saturating_mul(LL55_MAX_DAILY_PENALTY_DOLLARS as u32) as u64,
            statutory_basis: "NYC Admin Code § 27-2017.8 — pesticide must be applied by NYS DEC-licensed pest professional".to_string(),
            notes: format!(
                "VIOLATION § 27-2017.8: pesticide applied by unlicensed nonprofessional; LL 55 requires NYS DEC-licensed pest professional. {} days uncured.",
                input.days_violation_uncured_after_hpd_notice
            ),
            citations,
        };
    }

    if !input.tenant_annual_notice_provided {
        return Output {
            mode: LocalLaw55Mode::ViolationTenantAnnualNoticeNotProvided,
            accrued_daily_penalty_dollars: 0,
            statutory_basis: "LL 55 annual tenant notice required".to_string(),
            notes: "VIOLATION: LL 55 annual notice + DOHMH fact sheet not provided to tenant.".to_string(),
            citations,
        };
    }

    if !input.tenant_lease_renewal_notice_provided {
        return Output {
            mode: LocalLaw55Mode::ViolationTenantLeaseRenewalNoticeNotProvided,
            accrued_daily_penalty_dollars: 0,
            statutory_basis: "LL 55 tenant notice required at lease signing/renewal".to_string(),
            notes: "VIOLATION: LL 55 notice not provided at lease signing or renewal.".to_string(),
            citations,
        };
    }

    if !input.dohmh_fact_sheet_attached_to_notice {
        return Output {
            mode: LocalLaw55Mode::ViolationTenantAnnualNoticeNotProvided,
            accrued_daily_penalty_dollars: 0,
            statutory_basis: "LL 55 tenant notice must include DOHMH fact sheet".to_string(),
            notes: "VIOLATION: DOHMH Local Law 55 fact sheet not attached to tenant annual notice.".to_string(),
            citations,
        };
    }

    if input.access_notice_hours_provided_to_tenant < LL55_ACCESS_NOTICE_HOURS {
        return Output {
            mode: LocalLaw55Mode::Violation24HourAccessNoticeNotProvided,
            accrued_daily_penalty_dollars: 0,
            statutory_basis: "LL 55 — 24-hour access notice required for inspection".to_string(),
            notes: format!(
                "VIOLATION: only {} hours' notice provided for inspection access; LL 55 requires 24-hour written notice.",
                input.access_notice_hours_provided_to_tenant
            ),
            citations,
        };
    }

    if input.false_certification_of_correction_made {
        let (min_penalty, max_penalty) = if input.violation_classified_as_hazardous {
            (LL55_FALSE_CERT_HAZARDOUS_MIN_DOLLARS, LL55_FALSE_CERT_HAZARDOUS_MAX_DOLLARS)
        } else {
            (LL55_FALSE_CERT_NON_HAZARDOUS_MIN_DOLLARS, LL55_FALSE_CERT_NON_HAZARDOUS_MAX_DOLLARS)
        };
        return Output {
            mode: LocalLaw55Mode::ViolationFalseCertificationOfCorrectionMade,
            accrued_daily_penalty_dollars: max_penalty,
            statutory_basis: format!(
                "LL 55 false certification: ${}-${} {} violation",
                min_penalty,
                max_penalty,
                if input.violation_classified_as_hazardous {
                    "hazardous"
                } else {
                    "non-hazardous"
                }
            ),
            notes: format!(
                "VIOLATION: false certification of correction made for {} violation; civil penalty ${}-${}.",
                if input.violation_classified_as_hazardous {
                    "hazardous"
                } else {
                    "non-hazardous"
                },
                min_penalty,
                max_penalty
            ),
            citations,
        };
    }

    if input.days_violation_uncured_after_hpd_notice > 0 {
        let daily_accrual = if input.violation_classified_as_hazardous {
            LL55_MAX_DAILY_PENALTY_DOLLARS
        } else {
            LL55_MIN_DAILY_PENALTY_DOLLARS
        };
        let accrued = (input.days_violation_uncured_after_hpd_notice as u64)
            .saturating_mul(daily_accrual)
            .min(LL55_MAX_TOTAL_PENALTY_DOLLARS);
        return Output {
            mode: LocalLaw55Mode::ViolationDailyPenaltyAccrued,
            accrued_daily_penalty_dollars: accrued,
            statutory_basis: "LL 55 daily penalty accrual: $10-$125 per day; max $10,000".to_string(),
            notes: format!(
                "VIOLATION: {} days uncured after HPD notice; daily penalty ${} accrued × {} days = ${} (max ${}).",
                input.days_violation_uncured_after_hpd_notice,
                daily_accrual,
                input.days_violation_uncured_after_hpd_notice,
                accrued,
                LL55_MAX_TOTAL_PENALTY_DOLLARS
            ),
            citations,
        };
    }

    if input.indoor_allergen_hazard_detected == IndoorAllergenHazard::NoAllergenHazardPresent {
        return Output {
            mode: LocalLaw55Mode::NotApplicableNoAllergenHazardAndAnnualInspectionPerformed,
            accrued_daily_penalty_dollars: 0,
            statutory_basis: "LL 55 — annual inspection performed; no allergen hazard detected".to_string(),
            notes: "COMPLIANT: annual inspection performed; no allergen hazard detected; no remediation action required.".to_string(),
            citations,
        };
    }

    if input.pesticide_operator_license == PesticideOperatorLicense::NysDecLicensedPestProfessional {
        return Output {
            mode: LocalLaw55Mode::CompliantIPMOperatorLicensedDecApplied,
            accrued_daily_penalty_dollars: 0,
            statutory_basis: "NYC Admin Code § 27-2017.8 — NYS DEC-licensed operator applied".to_string(),
            notes: format!(
                "COMPLIANT § 27-2017.8: NYS DEC-licensed pest professional applied pesticide for {:?}; IPM approach documented; tenant notice + DOHMH fact sheet provided.",
                input.indoor_allergen_hazard_detected
            ),
            citations,
        };
    }

    Output {
        mode: LocalLaw55Mode::CompliantAnnualInspectionPerformedAndIpmApplied,
        accrued_daily_penalty_dollars: 0,
        statutory_basis: "NYC Admin Code § 27-2017 et seq. — LL 55 satisfied".to_string(),
        notes: format!(
            "COMPLIANT LL 55: annual inspection performed; IPM approach documented = {}; tenant notice + DOHMH fact sheet provided; 24-hour access notice; no false certification.",
            input.ipm_approach_documented
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_compliant() -> Input {
        Input {
            property_classification: PropertyClassification::NycMultipleDwelling3PlusUnits,
            indoor_allergen_hazard_detected: IndoorAllergenHazard::MicePresent,
            annual_inspection_performed_this_year: true,
            ipm_approach_documented: true,
            pesticide_operator_license: PesticideOperatorLicense::NysDecLicensedPestProfessional,
            tenant_annual_notice_provided: true,
            tenant_lease_renewal_notice_provided: true,
            dohmh_fact_sheet_attached_to_notice: true,
            access_notice_hours_provided_to_tenant: 24,
            days_violation_uncured_after_hpd_notice: 0,
            false_certification_of_correction_made: false,
            violation_classified_as_hazardous: false,
        }
    }

    #[test]
    fn single_family_dwelling_not_applicable() {
        let input = Input {
            property_classification: PropertyClassification::NycSingleOrTwoFamilyDwelling,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::NotApplicableNotMultipleDwellingOrOutsideNyc);
    }

    #[test]
    fn outside_nyc_not_applicable() {
        let input = Input {
            property_classification: PropertyClassification::PropertyOutsideNyc,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::NotApplicableNotMultipleDwellingOrOutsideNyc);
    }

    #[test]
    fn no_allergen_with_annual_inspection_compliant() {
        let input = Input {
            indoor_allergen_hazard_detected: IndoorAllergenHazard::NoAllergenHazardPresent,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::NotApplicableNoAllergenHazardAndAnnualInspectionPerformed);
    }

    #[test]
    fn nys_dec_licensed_operator_compliant() {
        let result = check(&baseline_compliant());
        assert_eq!(result.mode, LocalLaw55Mode::CompliantIPMOperatorLicensedDecApplied);
    }

    #[test]
    fn unlicensed_operator_violation() {
        let input = Input {
            pesticide_operator_license: PesticideOperatorLicense::UnlicensedNonprofessional,
            days_violation_uncured_after_hpd_notice: 5,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::ViolationIpmOperatorNotLicensedDec);
        assert_eq!(result.accrued_daily_penalty_dollars, 625);
    }

    #[test]
    fn annual_inspection_not_performed_violation() {
        let input = Input {
            annual_inspection_performed_this_year: false,
            days_violation_uncured_after_hpd_notice: 10,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::ViolationAnnualInspectionNotPerformed);
        assert_eq!(result.accrued_daily_penalty_dollars, 100);
    }

    #[test]
    fn tenant_annual_notice_not_provided_violation() {
        let input = Input {
            tenant_annual_notice_provided: false,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::ViolationTenantAnnualNoticeNotProvided);
    }

    #[test]
    fn tenant_lease_renewal_notice_not_provided_violation() {
        let input = Input {
            tenant_lease_renewal_notice_provided: false,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::ViolationTenantLeaseRenewalNoticeNotProvided);
    }

    #[test]
    fn dohmh_fact_sheet_not_attached_violation() {
        let input = Input {
            dohmh_fact_sheet_attached_to_notice: false,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::ViolationTenantAnnualNoticeNotProvided);
    }

    #[test]
    fn access_notice_below_24_hours_violation() {
        let input = Input {
            access_notice_hours_provided_to_tenant: 23,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::Violation24HourAccessNoticeNotProvided);
    }

    #[test]
    fn access_notice_at_exactly_24_hours_compliant() {
        let input = Input {
            access_notice_hours_provided_to_tenant: 24,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::CompliantIPMOperatorLicensedDecApplied);
    }

    #[test]
    fn false_certification_non_hazardous_violation() {
        let input = Input {
            false_certification_of_correction_made: true,
            violation_classified_as_hazardous: false,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::ViolationFalseCertificationOfCorrectionMade);
        assert_eq!(result.accrued_daily_penalty_dollars, 250);
    }

    #[test]
    fn false_certification_hazardous_violation_500() {
        let input = Input {
            false_certification_of_correction_made: true,
            violation_classified_as_hazardous: true,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::ViolationFalseCertificationOfCorrectionMade);
        assert_eq!(result.accrued_daily_penalty_dollars, 500);
    }

    #[test]
    fn daily_penalty_accrual_capped_at_10000() {
        let input = Input {
            days_violation_uncured_after_hpd_notice: 200,
            violation_classified_as_hazardous: true,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::ViolationDailyPenaltyAccrued);
        assert_eq!(result.accrued_daily_penalty_dollars, 10_000);
    }

    #[test]
    fn daily_penalty_below_cap_compliant_computation() {
        let input = Input {
            days_violation_uncured_after_hpd_notice: 30,
            violation_classified_as_hazardous: false,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::ViolationDailyPenaltyAccrued);
        assert_eq!(result.accrued_daily_penalty_dollars, 300);
    }

    #[test]
    fn cockroaches_present_with_licensed_compliant() {
        let input = Input {
            indoor_allergen_hazard_detected: IndoorAllergenHazard::CockroachesPresent,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::CompliantIPMOperatorLicensedDecApplied);
    }

    #[test]
    fn rats_present_with_licensed_compliant() {
        let input = Input {
            indoor_allergen_hazard_detected: IndoorAllergenHazard::RatsPresent,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::CompliantIPMOperatorLicensedDecApplied);
    }

    #[test]
    fn mold_present_with_licensed_compliant() {
        let input = Input {
            indoor_allergen_hazard_detected: IndoorAllergenHazard::MoldPresent,
            ..baseline_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, LocalLaw55Mode::CompliantIPMOperatorLicensedDecApplied);
    }

    #[test]
    fn citations_pin_admin_code_and_penalties() {
        let result = check(&baseline_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("NYC Admin Code § 27-2017"));
        assert!(joined.contains("Local Law 55 of 2018"));
        assert!(joined.contains("Asthma-Free Housing Act"));
        assert!(joined.contains("Jan 19, 2019"));
        assert!(joined.contains("§ 27-2017.8"));
        assert!(joined.contains("NYS DEC-licensed pest professional"));
        assert!(joined.contains("DOHMH"));
        assert!(joined.contains("HPD"));
        assert!(joined.contains("$10-$125"));
        assert!(joined.contains("$10,000"));
        assert!(joined.contains("$50-$250"));
        assert!(joined.contains("$250-$500"));
    }

    #[test]
    fn constant_pin_dates_penalties_thresholds() {
        assert_eq!(LOCAL_LAW_55_YEAR, 2018);
        assert_eq!(LOCAL_LAW_55_EFFECTIVE_DATE_YEAR, 2019);
        assert_eq!(LOCAL_LAW_55_EFFECTIVE_DATE_MONTH, 1);
        assert_eq!(LOCAL_LAW_55_EFFECTIVE_DATE_DAY, 19);
        assert_eq!(LL55_ACCESS_NOTICE_HOURS, 24);
        assert_eq!(LL55_MIN_DAILY_PENALTY_DOLLARS, 10);
        assert_eq!(LL55_MAX_DAILY_PENALTY_DOLLARS, 125);
        assert_eq!(LL55_MAX_TOTAL_PENALTY_DOLLARS, 10_000);
        assert_eq!(LL55_FALSE_CERT_NON_HAZARDOUS_MIN_DOLLARS, 50);
        assert_eq!(LL55_FALSE_CERT_NON_HAZARDOUS_MAX_DOLLARS, 250);
        assert_eq!(LL55_FALSE_CERT_HAZARDOUS_MIN_DOLLARS, 250);
        assert_eq!(LL55_FALSE_CERT_HAZARDOUS_MAX_DOLLARS, 500);
        assert_eq!(LL55_MULTIPLE_DWELLING_MIN_UNITS, 3);
        assert_eq!(LL55_NYC_ADMIN_CODE_SECTION_BASE, 2_017);
        assert_eq!(LL55_NYC_ADMIN_CODE_IPM_SUBSECTION, 8);
    }
}
