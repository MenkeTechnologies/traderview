//! Colorado HB 24-1098 of 2024 Just Cause Eviction Compliance
//! Module — first-ever statewide just-cause eviction law in
//! Colorado history.
//!
//! Pure-compute check for landlord compliance with Colorado's
//! HB 24-1098 ("Cause Required for Eviction of Residential
//! Tenant"), signed by Governor Jared Polis on April 19, 2024
//! and effective immediately due to legislative safety clause.
//! Codified at Colo. Rev. Stat. § 38-12-1301 et seq.
//!
//! Web research (verified 2026-06-03):
//! - **Colorado HB 24-1098 of 2024** (Mabrey/Duran/Bacon/Brown
//!   et al.) — signed by Governor **Jared Polis on April 19,
//!   2024**; effective immediately due to legislative safety
//!   clause ([Colorado General Assembly HB24-1098](https://leg.colorado.gov/bills/hb24-1098);
//!   [Colorado HB24-1098 Signed Bill Text](https://content.leg.colorado.gov/sites/default/files/2024a_1098_signed.pdf);
//!   [Brownstein — Shifting Dynamics: Changes to Landlord Eviction
//!   Rights in Colorado Under HB1098](https://www.bhfs.com/insight/shifting-dynamics-changes-to-landlord-eviction-rights-in-colorado-under-hb1098/)).
//! - **For-Cause Eviction Grounds** (permitted without
//!   relocation assistance): non-payment of rent; material
//!   lease violations; substantial property damage; criminal
//!   activity; non-curable lease violations.
//! - **No-Fault Eviction Grounds** (require 90-day notice +
//!   relocation assistance): (1) demolition or conversion of
//!   residential premises; (2) substantial repairs or
//!   renovations; (3) owner or owner family-member occupancy
//!   assumption; (4) withdrawal of premises from rental
//!   market for sale; (5) tenant refuses to sign new lease
//!   with reasonable terms; (6) tenant has history of non-
//!   payment of rent.
//! - **90-Day Notice Requirement**: landlord must provide
//!   90-day written notice for any no-fault eviction action.
//! - **Relocation Assistance**: no-fault eviction triggers
//!   **2 months' rent** baseline relocation assistance, PLUS
//!   **1 additional month** if any resident is (a) under
//!   age **18**, (b) at least age **60**, (c) household income
//!   **≤ 80 % of area median income (AMI)**, OR (d) an
//!   individual with a **disability** — yielding **3 months'
//!   rent total** vulnerable-resident relocation.
//! - **Exemptions**: short-term rental properties; certain
//!   owner-occupied units (typically 4 or fewer units with
//!   owner-occupied); employer-provided housing agreements;
//!   tenants residing in unit for less than **12 months** AND
//!   not known by landlord as such ([Lyons Gaddis — Navigating
//!   Colorado's HB24-1098 For Cause Eviction Law](https://www.lyonsgaddis.com/navigating-colorados-hb24-1098-for-cause-eviciton-law-by-brian-allard-lyons-gaddis/)).
//! - **Tenant Remedies**: if landlord proceeds with eviction
//!   without cause, tenant may seek relief under existing
//!   unlawful-removal-of-tenant statutes AND may assert the
//!   landlord's violation as an **affirmative defense** to the
//!   eviction proceeding.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CO_HB_24_1098_SIGNED_DATE_YEAR: u32 = 2024;
pub const CO_HB_24_1098_SIGNED_DATE_MONTH: u32 = 4;
pub const CO_HB_24_1098_SIGNED_DATE_DAY: u32 = 19;
pub const CO_HB_24_1098_NOTICE_DAYS_REQUIRED_NO_FAULT: u32 = 90;
pub const CO_HB_24_1098_BASE_RELOCATION_MONTHS_RENT: u32 = 2;
pub const CO_HB_24_1098_VULNERABLE_RESIDENT_RELOCATION_MONTHS_RENT: u32 = 3;
pub const CO_HB_24_1098_MINOR_AGE_THRESHOLD: u32 = 18;
pub const CO_HB_24_1098_SENIOR_AGE_THRESHOLD: u32 = 60;
pub const CO_HB_24_1098_AMI_THRESHOLD_BASIS_POINTS: u64 = 8_000;
pub const CO_HB_24_1098_AMI_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const CO_HB_24_1098_MIN_TENANCY_MONTHS_FOR_COVERAGE: u32 = 12;
pub const CO_HB_24_1098_OWNER_OCCUPIED_MAX_UNITS_EXEMPT: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionGround {
    ForCauseNonPaymentOfRent,
    ForCauseMaterialLeaseViolation,
    ForCauseSubstantialPropertyDamage,
    ForCauseCriminalActivity,
    ForCauseNonCurableLeaseViolation,
    NoFaultDemolitionOrConversion,
    NoFaultSubstantialRepairsOrRenovations,
    NoFaultOwnerOrFamilyMemberOccupancy,
    NoFaultWithdrawalFromRentalMarketForSale,
    NoFaultTenantRefusedNewLeaseReasonableTerms,
    NoFaultTenantHistoryOfNonpaymentOfRent,
    NoCauseAttempted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyExemptionStatus {
    NotExemptFullyCoveredByHb241098,
    ExemptShortTermRentalProperty,
    ExemptOwnerOccupiedUnitsLessOrEqual4,
    ExemptEmployerProvidedHousing,
    ExemptTenantUnder12MonthsAndUnknownToLandlord,
    PropertyOutsideColorado,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VulnerableResidentStatus {
    HouseholdHasNoVulnerableResident,
    HouseholdHasMinorUnder18,
    HouseholdHasSeniorAt60OrAbove,
    HouseholdIncomeAtOrBelow80PctAmi,
    HouseholdHasDisabledIndividual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeProvided {
    NoticeProvidedAtOrAbove90Days,
    NoticeProvidedLessThan90Days,
    NoNoticeProvided,
    NoticeNotRequiredForCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ColoradoHb241098Mode {
    NotApplicablePropertyOutsideColorado,
    NotApplicableShortTermRentalExempt,
    NotApplicableOwnerOccupied4OrFewerUnitsExempt,
    NotApplicableEmployerProvidedHousingExempt,
    NotApplicableTenantUnder12MonthsAndUnknownExempt,
    CompliantForCauseEvictionGroundProperlyAsserted,
    CompliantNoFaultEvictionWith90DayNoticeAndBaseRelocationAssistance,
    CompliantNoFaultEvictionWithVulnerableResident3MonthRelocationAssistance,
    ViolationNoCauseEvictionAttemptedWithoutQualifyingGround,
    ViolationNoFaultEvictionWithoutRequired90DayNotice,
    ViolationNoFaultEvictionWithoutRequiredRelocationAssistance,
    ViolationNoFaultEvictionVulnerableResidentRelocationAssistanceUnderpaid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_exemption_status: PropertyExemptionStatus,
    pub eviction_ground: EvictionGround,
    pub vulnerable_resident_status: VulnerableResidentStatus,
    pub notice_provided: NoticeProvided,
    pub monthly_rent_cents: u64,
    pub relocation_assistance_paid_cents: u64,
    pub tenancy_duration_months: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: ColoradoHb241098Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub required_relocation_assistance_cents: u64,
}

pub type RentalColoradoHb241098JustCauseEvictionInput = Input;
pub type RentalColoradoHb241098JustCauseEvictionOutput = Output;
pub type RentalColoradoHb241098JustCauseEvictionResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Colorado HB 24-1098 of 2024 (Mabrey/Duran/Bacon/Brown) — 'Cause Required for Eviction of Residential Tenant'; signed by Governor Jared Polis on April 19, 2024; effective immediately due to legislative safety clause".to_string(),
        "Colo. Rev. Stat. § 38-12-1301 et seq. — first-ever statewide just-cause eviction law in Colorado history".to_string(),
        "For-Cause Eviction Grounds — non-payment of rent, material lease violations, substantial property damage, criminal activity, non-curable lease violations (no notice/relocation assistance required)".to_string(),
        "No-Fault Eviction Grounds — (1) demolition or conversion of residential premises; (2) substantial repairs or renovations; (3) owner or family-member occupancy assumption; (4) withdrawal from rental market for sale; (5) tenant refuses to sign new lease with reasonable terms; (6) tenant has history of nonpayment of rent".to_string(),
        "90-Day Notice Requirement — landlord must provide 90-day written notice for any no-fault eviction action".to_string(),
        "Relocation Assistance — 2 months' rent baseline for no-fault eviction; 3 months' rent (1 additional month) if any resident is under 18, at least 60, household income ≤ 80 % AMI, or disabled individual".to_string(),
        "Exemptions — short-term rental properties; owner-occupied units (typically 4 or fewer units with owner-occupied); employer-provided housing agreements; tenants residing < 12 months AND not known by landlord as such".to_string(),
        "Tenant Remedies — tenant may seek relief under existing unlawful-removal-of-tenant statutes AND may assert landlord's violation as affirmative defense to eviction proceeding".to_string(),
        "Brownstein Hyatt Farber Schreck — Shifting Dynamics: Changes to Landlord Eviction Rights in Colorado Under HB1098 — practitioner guide".to_string(),
        "Lyons Gaddis — Navigating Colorado's HB24-1098 For Cause Eviction Law — practitioner analysis".to_string(),
        "ACLU of Colorado — HB24-1098 legislative summary".to_string(),
        "Colorado General Assembly HB24-1098 — primary bill text and history".to_string(),
    ];

    if input.property_exemption_status == PropertyExemptionStatus::PropertyOutsideColorado {
        return Output {
            mode: ColoradoHb241098Mode::NotApplicablePropertyOutsideColorado,
            statutory_basis: "Property outside Colorado; HB 24-1098 inapplicable".to_string(),
            notes: "Property outside Colorado; Colorado HB 24-1098 just cause eviction law inapplicable.".to_string(),
            citations,
            required_relocation_assistance_cents: 0,
        };
    }

    if input.property_exemption_status == PropertyExemptionStatus::ExemptShortTermRentalProperty {
        return Output {
            mode: ColoradoHb241098Mode::NotApplicableShortTermRentalExempt,
            statutory_basis: "Colo. Rev. Stat. § 38-12-1301 et seq. — short-term rental property exemption".to_string(),
            notes: "NOT APPLICABLE: short-term rental property; HB 24-1098 just cause eviction requirements do not apply.".to_string(),
            citations,
            required_relocation_assistance_cents: 0,
        };
    }

    if input.property_exemption_status
        == PropertyExemptionStatus::ExemptOwnerOccupiedUnitsLessOrEqual4
    {
        return Output {
            mode: ColoradoHb241098Mode::NotApplicableOwnerOccupied4OrFewerUnitsExempt,
            statutory_basis: "Colo. Rev. Stat. § 38-12-1301 et seq. — owner-occupied 4-or-fewer-units exemption".to_string(),
            notes: "NOT APPLICABLE: owner-occupied property with 4 or fewer units; HB 24-1098 just cause eviction requirements do not apply.".to_string(),
            citations,
            required_relocation_assistance_cents: 0,
        };
    }

    if input.property_exemption_status == PropertyExemptionStatus::ExemptEmployerProvidedHousing {
        return Output {
            mode: ColoradoHb241098Mode::NotApplicableEmployerProvidedHousingExempt,
            statutory_basis: "Colo. Rev. Stat. § 38-12-1301 et seq. — employer-provided housing exemption".to_string(),
            notes: "NOT APPLICABLE: employer-provided housing agreement; HB 24-1098 just cause eviction requirements do not apply.".to_string(),
            citations,
            required_relocation_assistance_cents: 0,
        };
    }

    if input.property_exemption_status
        == PropertyExemptionStatus::ExemptTenantUnder12MonthsAndUnknownToLandlord
        && input.tenancy_duration_months < CO_HB_24_1098_MIN_TENANCY_MONTHS_FOR_COVERAGE
    {
        return Output {
            mode: ColoradoHb241098Mode::NotApplicableTenantUnder12MonthsAndUnknownExempt,
            statutory_basis: "Colo. Rev. Stat. § 38-12-1301 et seq. — tenant under 12 months and unknown to landlord exemption".to_string(),
            notes: format!(
                "NOT APPLICABLE: tenant has resided in unit only {} months (< 12-month statutory threshold) AND is unknown to landlord as such; HB 24-1098 just cause eviction requirements do not apply.",
                input.tenancy_duration_months
            ),
            citations,
            required_relocation_assistance_cents: 0,
        };
    }

    if input.eviction_ground == EvictionGround::NoCauseAttempted {
        return Output {
            mode: ColoradoHb241098Mode::ViolationNoCauseEvictionAttemptedWithoutQualifyingGround,
            statutory_basis: "Colo. Rev. Stat. § 38-12-1301 et seq. — eviction without qualifying for-cause or no-fault ground prohibited".to_string(),
            notes: "VIOLATION: eviction attempted without asserting any for-cause or no-fault qualifying ground; HB 24-1098 prohibits no-cause evictions of covered residential tenants; tenant may assert as affirmative defense.".to_string(),
            citations,
            required_relocation_assistance_cents: 0,
        };
    }

    let is_for_cause = matches!(
        input.eviction_ground,
        EvictionGround::ForCauseNonPaymentOfRent
            | EvictionGround::ForCauseMaterialLeaseViolation
            | EvictionGround::ForCauseSubstantialPropertyDamage
            | EvictionGround::ForCauseCriminalActivity
            | EvictionGround::ForCauseNonCurableLeaseViolation
    );

    if is_for_cause {
        return Output {
            mode: ColoradoHb241098Mode::CompliantForCauseEvictionGroundProperlyAsserted,
            statutory_basis: "Colo. Rev. Stat. § 38-12-1301 et seq. — for-cause eviction ground".to_string(),
            notes: format!(
                "COMPLIANT: for-cause eviction ground asserted ({:?}); no 90-day notice or relocation assistance required for for-cause evictions under HB 24-1098.",
                input.eviction_ground
            ),
            citations,
            required_relocation_assistance_cents: 0,
        };
    }

    if !matches!(
        input.notice_provided,
        NoticeProvided::NoticeProvidedAtOrAbove90Days
    ) {
        return Output {
            mode: ColoradoHb241098Mode::ViolationNoFaultEvictionWithoutRequired90DayNotice,
            statutory_basis: "Colo. Rev. Stat. § 38-12-1301 et seq. — 90-day written notice required for no-fault eviction".to_string(),
            notes: format!(
                "VIOLATION: no-fault eviction attempted without required 90-day written notice; notice classification: {:?}.",
                input.notice_provided
            ),
            citations,
            required_relocation_assistance_cents: 0,
        };
    }

    let has_vulnerable_resident = !matches!(
        input.vulnerable_resident_status,
        VulnerableResidentStatus::HouseholdHasNoVulnerableResident
    );

    let required_relocation_cents = if has_vulnerable_resident {
        input.monthly_rent_cents.saturating_mul(u64::from(
            CO_HB_24_1098_VULNERABLE_RESIDENT_RELOCATION_MONTHS_RENT,
        ))
    } else {
        input
            .monthly_rent_cents
            .saturating_mul(u64::from(CO_HB_24_1098_BASE_RELOCATION_MONTHS_RENT))
    };

    if input.relocation_assistance_paid_cents < required_relocation_cents {
        if has_vulnerable_resident {
            return Output {
                mode: ColoradoHb241098Mode::ViolationNoFaultEvictionVulnerableResidentRelocationAssistanceUnderpaid,
                statutory_basis: "Colo. Rev. Stat. § 38-12-1301 et seq. — 3 months' rent relocation for vulnerable resident".to_string(),
                notes: format!(
                    "VIOLATION: no-fault eviction with vulnerable resident ({:?}) requires 3 months' rent relocation assistance ({} cents); landlord paid only {} cents.",
                    input.vulnerable_resident_status,
                    required_relocation_cents,
                    input.relocation_assistance_paid_cents
                ),
                citations,
                required_relocation_assistance_cents: required_relocation_cents,
            };
        }
        return Output {
            mode: ColoradoHb241098Mode::ViolationNoFaultEvictionWithoutRequiredRelocationAssistance,
            statutory_basis: "Colo. Rev. Stat. § 38-12-1301 et seq. — 2 months' rent relocation assistance required".to_string(),
            notes: format!(
                "VIOLATION: no-fault eviction requires 2 months' rent relocation assistance ({} cents); landlord paid only {} cents.",
                required_relocation_cents, input.relocation_assistance_paid_cents
            ),
            citations,
            required_relocation_assistance_cents: required_relocation_cents,
        };
    }

    if has_vulnerable_resident {
        return Output {
            mode: ColoradoHb241098Mode::CompliantNoFaultEvictionWithVulnerableResident3MonthRelocationAssistance,
            statutory_basis: "Colo. Rev. Stat. § 38-12-1301 et seq. — vulnerable resident 3 months' rent relocation".to_string(),
            notes: format!(
                "COMPLIANT: no-fault eviction with vulnerable resident ({:?}); 90-day notice provided; 3 months' rent relocation assistance ({} cents) paid (≥ required {} cents).",
                input.vulnerable_resident_status,
                input.relocation_assistance_paid_cents,
                required_relocation_cents
            ),
            citations,
            required_relocation_assistance_cents: required_relocation_cents,
        };
    }
    Output {
        mode: ColoradoHb241098Mode::CompliantNoFaultEvictionWith90DayNoticeAndBaseRelocationAssistance,
        statutory_basis: "Colo. Rev. Stat. § 38-12-1301 et seq. — 90-day notice + 2 months' rent relocation".to_string(),
        notes: format!(
            "COMPLIANT: no-fault eviction with 90-day notice provided; 2 months' rent relocation assistance ({} cents) paid (≥ required {} cents).",
            input.relocation_assistance_paid_cents, required_relocation_cents
        ),
        citations,
        required_relocation_assistance_cents: required_relocation_cents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_no_fault_compliant() -> Input {
        Input {
            property_exemption_status: PropertyExemptionStatus::NotExemptFullyCoveredByHb241098,
            eviction_ground: EvictionGround::NoFaultOwnerOrFamilyMemberOccupancy,
            vulnerable_resident_status: VulnerableResidentStatus::HouseholdHasNoVulnerableResident,
            notice_provided: NoticeProvided::NoticeProvidedAtOrAbove90Days,
            monthly_rent_cents: 250_000,
            relocation_assistance_paid_cents: 500_000,
            tenancy_duration_months: 18,
        }
    }

    #[test]
    fn property_outside_colorado_not_applicable() {
        let input = Input {
            property_exemption_status: PropertyExemptionStatus::PropertyOutsideColorado,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::NotApplicablePropertyOutsideColorado
        );
    }

    #[test]
    fn short_term_rental_exempt_not_applicable() {
        let input = Input {
            property_exemption_status: PropertyExemptionStatus::ExemptShortTermRentalProperty,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::NotApplicableShortTermRentalExempt
        );
    }

    #[test]
    fn owner_occupied_4_or_fewer_exempt() {
        let input = Input {
            property_exemption_status:
                PropertyExemptionStatus::ExemptOwnerOccupiedUnitsLessOrEqual4,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::NotApplicableOwnerOccupied4OrFewerUnitsExempt
        );
    }

    #[test]
    fn employer_provided_housing_exempt() {
        let input = Input {
            property_exemption_status: PropertyExemptionStatus::ExemptEmployerProvidedHousing,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::NotApplicableEmployerProvidedHousingExempt
        );
    }

    #[test]
    fn tenant_under_12_months_and_unknown_exempt() {
        let input = Input {
            property_exemption_status:
                PropertyExemptionStatus::ExemptTenantUnder12MonthsAndUnknownToLandlord,
            tenancy_duration_months: 6,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::NotApplicableTenantUnder12MonthsAndUnknownExempt
        );
    }

    #[test]
    fn no_cause_attempted_violation() {
        let input = Input {
            eviction_ground: EvictionGround::NoCauseAttempted,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::ViolationNoCauseEvictionAttemptedWithoutQualifyingGround
        );
    }

    #[test]
    fn for_cause_non_payment_compliant() {
        let input = Input {
            eviction_ground: EvictionGround::ForCauseNonPaymentOfRent,
            notice_provided: NoticeProvided::NoticeNotRequiredForCause,
            relocation_assistance_paid_cents: 0,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::CompliantForCauseEvictionGroundProperlyAsserted
        );
    }

    #[test]
    fn for_cause_criminal_activity_compliant() {
        let input = Input {
            eviction_ground: EvictionGround::ForCauseCriminalActivity,
            notice_provided: NoticeProvided::NoticeNotRequiredForCause,
            relocation_assistance_paid_cents: 0,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::CompliantForCauseEvictionGroundProperlyAsserted
        );
    }

    #[test]
    fn no_fault_owner_move_in_compliant() {
        let result = check(&baseline_no_fault_compliant());
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::CompliantNoFaultEvictionWith90DayNoticeAndBaseRelocationAssistance
        );
        assert_eq!(result.required_relocation_assistance_cents, 500_000);
    }

    #[test]
    fn no_fault_notice_below_90_days_violation() {
        let input = Input {
            notice_provided: NoticeProvided::NoticeProvidedLessThan90Days,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::ViolationNoFaultEvictionWithoutRequired90DayNotice
        );
    }

    #[test]
    fn no_fault_no_notice_violation() {
        let input = Input {
            notice_provided: NoticeProvided::NoNoticeProvided,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::ViolationNoFaultEvictionWithoutRequired90DayNotice
        );
    }

    #[test]
    fn no_fault_relocation_underpaid_violation() {
        let input = Input {
            relocation_assistance_paid_cents: 100_000,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::ViolationNoFaultEvictionWithoutRequiredRelocationAssistance
        );
        assert_eq!(result.required_relocation_assistance_cents, 500_000);
    }

    #[test]
    fn no_fault_minor_under_18_3_month_relocation_compliant() {
        let input = Input {
            vulnerable_resident_status: VulnerableResidentStatus::HouseholdHasMinorUnder18,
            relocation_assistance_paid_cents: 750_000,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::CompliantNoFaultEvictionWithVulnerableResident3MonthRelocationAssistance
        );
        assert_eq!(result.required_relocation_assistance_cents, 750_000);
    }

    #[test]
    fn no_fault_senior_60_plus_3_month_relocation_compliant() {
        let input = Input {
            vulnerable_resident_status: VulnerableResidentStatus::HouseholdHasSeniorAt60OrAbove,
            relocation_assistance_paid_cents: 750_000,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::CompliantNoFaultEvictionWithVulnerableResident3MonthRelocationAssistance
        );
    }

    #[test]
    fn no_fault_low_income_80_pct_ami_3_month_relocation_compliant() {
        let input = Input {
            vulnerable_resident_status: VulnerableResidentStatus::HouseholdIncomeAtOrBelow80PctAmi,
            relocation_assistance_paid_cents: 750_000,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::CompliantNoFaultEvictionWithVulnerableResident3MonthRelocationAssistance
        );
    }

    #[test]
    fn no_fault_disabled_individual_3_month_relocation_compliant() {
        let input = Input {
            vulnerable_resident_status: VulnerableResidentStatus::HouseholdHasDisabledIndividual,
            relocation_assistance_paid_cents: 750_000,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::CompliantNoFaultEvictionWithVulnerableResident3MonthRelocationAssistance
        );
    }

    #[test]
    fn no_fault_vulnerable_resident_relocation_underpaid_violation() {
        let input = Input {
            vulnerable_resident_status: VulnerableResidentStatus::HouseholdHasMinorUnder18,
            relocation_assistance_paid_cents: 500_000,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::ViolationNoFaultEvictionVulnerableResidentRelocationAssistanceUnderpaid
        );
        assert_eq!(result.required_relocation_assistance_cents, 750_000);
    }

    #[test]
    fn no_fault_at_exactly_2_months_compliant() {
        let input = Input {
            relocation_assistance_paid_cents: 500_000,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::CompliantNoFaultEvictionWith90DayNoticeAndBaseRelocationAssistance
        );
    }

    #[test]
    fn no_fault_at_1_cent_below_2_months_violation() {
        let input = Input {
            relocation_assistance_paid_cents: 499_999,
            ..baseline_no_fault_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ColoradoHb241098Mode::ViolationNoFaultEvictionWithoutRequiredRelocationAssistance
        );
    }

    #[test]
    fn citations_pin_hb_24_1098_signing_and_grounds() {
        let result = check(&baseline_no_fault_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Colorado HB 24-1098 of 2024"));
        assert!(joined.contains("Governor Jared Polis"));
        assert!(joined.contains("April 19, 2024"));
        assert!(joined.contains("Colo. Rev. Stat. § 38-12-1301"));
        assert!(joined.contains("For-Cause"));
        assert!(joined.contains("non-payment of rent"));
        assert!(joined.contains("criminal activity"));
        assert!(joined.contains("No-Fault"));
        assert!(joined.contains("demolition or conversion"));
        assert!(joined.contains("owner or family-member occupancy"));
        assert!(joined.contains("withdrawal from rental market for sale"));
        assert!(joined.contains("90-day"));
        assert!(joined.contains("2 months' rent baseline"));
        assert!(joined.contains("3 months' rent"));
        assert!(joined.contains("under 18"));
        assert!(joined.contains("at least 60"));
        assert!(joined.contains("80 % AMI"));
        assert!(joined.contains("disabled"));
        assert!(joined.contains("short-term rental"));
        assert!(joined.contains("owner-occupied units"));
        assert!(joined.contains("employer-provided housing"));
        assert!(joined.contains("12 months"));
        assert!(joined.contains("affirmative defense"));
        assert!(joined.contains("Brownstein"));
        assert!(joined.contains("Lyons Gaddis"));
        assert!(joined.contains("ACLU of Colorado"));
    }

    #[test]
    fn constant_pin_dates_thresholds_and_relocation() {
        assert_eq!(CO_HB_24_1098_SIGNED_DATE_YEAR, 2024);
        assert_eq!(CO_HB_24_1098_SIGNED_DATE_MONTH, 4);
        assert_eq!(CO_HB_24_1098_SIGNED_DATE_DAY, 19);
        assert_eq!(CO_HB_24_1098_NOTICE_DAYS_REQUIRED_NO_FAULT, 90);
        assert_eq!(CO_HB_24_1098_BASE_RELOCATION_MONTHS_RENT, 2);
        assert_eq!(CO_HB_24_1098_VULNERABLE_RESIDENT_RELOCATION_MONTHS_RENT, 3);
        assert_eq!(CO_HB_24_1098_MINOR_AGE_THRESHOLD, 18);
        assert_eq!(CO_HB_24_1098_SENIOR_AGE_THRESHOLD, 60);
        assert_eq!(CO_HB_24_1098_AMI_THRESHOLD_BASIS_POINTS, 8_000);
        assert_eq!(CO_HB_24_1098_AMI_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(CO_HB_24_1098_MIN_TENANCY_MONTHS_FOR_COVERAGE, 12);
        assert_eq!(CO_HB_24_1098_OWNER_OCCUPIED_MAX_UNITS_EXEMPT, 4);
    }
}
