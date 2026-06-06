//! Seattle Just Cause Eviction Ordinance (SMC 22.206.160.C)
//! Compliance Module — first US local just-cause eviction
//! ordinance ever enacted (1980), preceding every other
//! municipal just-cause regime in the country.
//!
//! Pure-compute check for landlord compliance with the Seattle
//! Just Cause Eviction Ordinance (JCEO) codified at Seattle
//! Municipal Code (SMC) § 22.206.160.C. The JCEO prohibits a
//! landlord from terminating a month-to-month tenancy OR
//! refusing to renew a lease on residential property within
//! the City of Seattle unless one of the enumerated just
//! causes applies. SMC 22.206.160.C also lays out specific
//! notice-period requirements (14-day pay-or-vacate, 10-day
//! comply-or-vacate, 20-day termination, 60-day rent increase,
//! 90-day owner-move-in / single-family-sale) and ties the
//! demolition / substantial-rehabilitation / use-change just
//! causes to obtaining a tenant relocation license under the
//! Tenant Relocation Assistance Ordinance (TRAO) from the
//! Seattle Department of Construction and Inspections (SDCI).
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: Seattle's Just Cause Eviction Ordinance
//!   was adopted by the Seattle City Council in **1980**,
//!   making it the FIRST local just-cause eviction ordinance
//!   in the United States ([Seattle SDCI — Just Cause Eviction
//!   Ordinance](https://www.seattle.gov/sdci/codes/codes-we-enforce-(a-z)/just-cause-eviction-ordinance);
//!   [Tenants Union of Washington — Just Cause Eviction
//!   Protection](https://tenantsunion.org/rights/just-cause-eviction-protection)).
//! - **Codification**: SMC § 22.206.160.C ("Duties of Owners
//!   and Tenants — Termination of Tenancy") under SMC Title 22
//!   ("Building and Construction Codes"), Subtitle II
//!   ("Housing Code"), Chapter 22.206 ("Habitable Buildings"),
//!   Subchapter VI ([Seattle Municipal Code 22.206.160](https://library.municode.com/wa/seattle/codes/municipal_code?nodeId=TIT22BUCOCO_SUBTITLE_IIHOCO_CH22.206HABU_SUBCHAPTER_VIDUOWTE_22.206.160DUOW)).
//! - **Scope**: Applies to month-to-month tenancies AND lease
//!   non-renewals on residential property within Seattle.
//!   Exempts: transient lodging (hotels, motels, certain
//!   short-term housing), housing provided by educational
//!   institutions to students/faculty, occupancy under medical-
//!   care or correctional facilities, and live-aboard vessels.
//! - **Enumerated Just Causes** (SMC 22.206.160.C — sources
//!   describe between 16 and 18 enumerated causes as the
//!   ordinance has been amended since 1980, most recently by
//!   the 2021 amendments):
//!   1. **Non-payment of rent** after 14-day pay-or-vacate
//!      notice (RCW 59.18.057);
//!   2. **Material noncompliance** with lease after 10-day
//!      comply-or-vacate notice;
//!   3. **Chronic late rent** — tenant received **4 OR MORE
//!      14-day pay-or-vacate notices** in a 12-month period;
//!      requires 20-day termination notice;
//!   4. **Habitual rule violations** — tenant received
//!      **3 OR MORE 10-day comply-or-vacate notices** in a
//!      12-month period; requires 20-day termination notice;
//!   5. **Owner or immediate family occupancy** — landlord
//!      or immediate family will occupy unit as principal
//!      residence and no comparable unit is available; 90-day
//!      written notice; landlord must complete 60 consecutive
//!      days of occupancy within 90 days of tenant vacating;
//!      violation subjects landlord to tenant damages claim
//!      up to **$2,000**;
//!   6. **Sale of single-family dwelling** — owner elects to
//!      sell single-family unit; 90-day written notice; owner
//!      must list / show property within 30 days; failure
//!      triggers $2,000 tenant damages;
//!   7. **Substantial rehabilitation, demolition, or change
//!      of use** — landlord MUST first obtain Tenant
//!      Relocation Assistance Ordinance (TRAO) license from
//!      SDCI BEFORE serving termination notice;
//!   8. **Conversion to condominium or cooperative** —
//!      120-day notice required under RCW 64.34.440;
//!   9. **Tenant refused to sign new lease** with terms
//!      substantially identical to expiring lease;
//!  10. **Criminal activity** — must be recorded with the
//!      city and substantially affects other tenants' use or
//!      landlord's property interest;
//!  11. **Owner quitting shared occupancy** with tenant in
//!      same dwelling;
//!  12. **Transfer to comparable subsidized or rent-restricted
//!      unit** ([Seattle Tenants Union — Just Cause Eviction
//!      Protection](https://tenantsunion.org/rights/just-cause-eviction-protection);
//!      [How to Evict a Tenant in Seattle —
//!      Brink Law](https://www.brinkatlaw.com/seattle-eviction-guide/);
//!      [Rental Housing Changes in Seattle —
//!      Law HG](https://www.lawhg.net/news-and-insights/2021/8/5/b71bnjmu3itkqgkc0rbwa63st2cu8n-9NqPA)).
//! - **Owner Move-In Good Faith Rebuttable Presumption**: if
//!   owner fails to occupy unit for at least **60 consecutive
//!   days within 90 days of tenant vacating**, statutory
//!   rebuttable presumption that owner did NOT act in good
//!   faith; tenant entitled to damages up to **$2,000** plus
//!   actual damages, reasonable attorney fees, and court costs.
//! - **TRAO Linkage**: SMC § 22.210 (Tenant Relocation
//!   Assistance Ordinance) requires landlord to obtain a
//!   tenant relocation license BEFORE serving 20-day
//!   termination notice for demolition / substantial
//!   rehabilitation / change of use. Eligible low-income
//!   tenants receive **$3,667** as of 2024 (50 % SDCI / 50 %
//!   landlord) per the TRAO; statute updates the figure
//!   periodically; this module does NOT hardcode the TRAO
//!   payment amount.
//! - **Enforcement**: SDCI investigates JCEO complaints. A
//!   landlord who terminates without a qualifying just cause
//!   is subject to civil penalty + tenant private right of
//!   action for unlawful detainer defense, statutory damages,
//!   and reasonable attorney fees.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SEATTLE_JCEO_ENACTMENT_YEAR: u32 = 1980;
pub const SEATTLE_JCEO_CHRONIC_LATE_RENT_NOTICE_THRESHOLD_PER_12_MONTHS: u32 = 4;
pub const SEATTLE_JCEO_HABITUAL_RULE_VIOLATION_NOTICE_THRESHOLD_PER_12_MONTHS: u32 = 3;
pub const SEATTLE_JCEO_NON_PAYMENT_OF_RENT_NOTICE_DAYS_REQUIRED: u32 = 14;
pub const SEATTLE_JCEO_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS_REQUIRED: u32 = 10;
pub const SEATTLE_JCEO_TERMINATION_NOTICE_DAYS_RULE_VIOLATIONS: u32 = 20;
pub const SEATTLE_JCEO_OWNER_MOVE_IN_NOTICE_DAYS_REQUIRED: u32 = 90;
pub const SEATTLE_JCEO_SINGLE_FAMILY_SALE_NOTICE_DAYS_REQUIRED: u32 = 90;
pub const SEATTLE_JCEO_CONDOMINIUM_CONVERSION_NOTICE_DAYS_REQUIRED: u32 = 120;
pub const SEATTLE_JCEO_OWNER_MOVE_IN_OCCUPANCY_DAYS_REQUIRED: u32 = 60;
pub const SEATTLE_JCEO_OWNER_MOVE_IN_OCCUPANCY_DEADLINE_DAYS_AFTER_VACATE: u32 = 90;
pub const SEATTLE_JCEO_OWNER_MOVE_IN_VIOLATION_DAMAGES_DOLLARS: u64 = 2_000;
pub const SEATTLE_JCEO_SINGLE_FAMILY_SALE_VIOLATION_DAMAGES_DOLLARS: u64 = 2_000;
pub const SEATTLE_JCEO_LOOKBACK_MONTHS_NOTICE_COUNTING: u32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    WithinSeattleCityLimits,
    OutsideSeattleCityLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitType {
    ResidentialMultiFamily,
    ResidentialSingleFamilyDwelling,
    TransientLodgingHotelMotelExempt,
    EducationalInstitutionHousingExempt,
    MedicalOrCorrectionalOccupancyExempt,
    LiveAboardVesselExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JustCauseAsserted {
    NonPaymentOfRentAfter14DayNotice,
    MaterialLeaseNoncomplianceAfter10DayNotice,
    ChronicLateRent4PlusPayOrVacateNoticesIn12Months,
    HabitualRuleViolations3PlusComplyOrVacateNoticesIn12Months,
    OwnerOrImmediateFamilyOccupancy,
    SaleOfSingleFamilyDwelling,
    SubstantialRehabilitationOrDemolitionOrUseChange,
    ConversionToCondominiumOrCooperative,
    TenantRefusedToSignNewLeaseSubstantiallyIdenticalTerms,
    CriminalActivityRecordedWithCity,
    OwnerQuittingSharedOccupancyWithTenant,
    TransferToComparableSubsidizedOrRentRestrictedUnit,
    NoJustCauseAsserted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerMoveInOccupancyStatus {
    OccupancyCompletedAtOrAbove60ConsecutiveDaysWithin90DaysOfVacate,
    OccupancyNotCompletedAsRequiredRebuttablePresumptionOfBadFaith,
    NotApplicableToThisJustCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraoLicenseStatus {
    TraoLicenseObtainedFromSdciPriorToTerminationNotice,
    TraoLicenseNotObtained,
    NotApplicableToThisJustCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SeattleJceoMode {
    NotApplicablePropertyOutsideSeattle,
    NotApplicableTransientLodgingExempt,
    NotApplicableEducationalInstitutionHousingExempt,
    NotApplicableMedicalOrCorrectionalOccupancyExempt,
    NotApplicableLiveAboardVesselExempt,
    CompliantNonPaymentOfRentAfter14DayPayOrVacateNotice,
    CompliantMaterialLeaseNoncomplianceAfter10DayComplyOrVacateNotice,
    CompliantChronicLateRent4PlusPayOrVacateNoticesIn12MonthsWith20DayTermination,
    CompliantHabitualRuleViolations3PlusComplyOrVacateNoticesIn12MonthsWith20DayTermination,
    CompliantOwnerOrImmediateFamilyOccupancyWith90DayNoticeAndGoodFaithOccupancy,
    CompliantSaleOfSingleFamilyDwellingWith90DayNotice,
    CompliantSubstantialRehabilitationOrDemolitionOrUseChangeWithTraoLicense,
    CompliantConversionToCondominiumOrCooperativeWith120DayNotice,
    CompliantTenantRefusedToSignNewLeaseSubstantiallyIdenticalTerms,
    CompliantCriminalActivityRecordedWithCity,
    CompliantOwnerQuittingSharedOccupancyWithTenant,
    CompliantTransferToComparableSubsidizedOrRentRestrictedUnit,
    ViolationNoticeOfTerminationWithoutAssertingAnyJustCause,
    ViolationChronicLateRentInsufficientPriorPayOrVacateNoticesUnder4In12Months,
    ViolationHabitualRuleViolationsInsufficientPriorComplyOrVacateNoticesUnder3In12Months,
    ViolationOwnerOrFamilyOccupancyWithoutRequired90DayNotice,
    ViolationOwnerMoveInFailureToOccupyTriggeringRebuttablePresumptionAndTenantDamages,
    ViolationSingleFamilyDwellingSaleWithoutRequired90DayNotice,
    ViolationSubstantialRehabilitationOrDemolitionOrUseChangeWithoutTraoLicense,
    ViolationConversionToCondominiumOrCooperativeWithoutRequired120DayNotice,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub unit_type: UnitType,
    pub just_cause_asserted: JustCauseAsserted,
    pub prior_pay_or_vacate_notices_last_12_months: u32,
    pub prior_comply_or_vacate_notices_last_12_months: u32,
    pub owner_move_in_occupancy_status: OwnerMoveInOccupancyStatus,
    pub trao_license_status: TraoLicenseStatus,
    pub notice_days_provided: u32,
    pub monthly_rent_cents: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: SeattleJceoMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub owner_move_in_violation_damages_cents: u64,
}

pub type RentalSeattleSmc22206160JustCauseEvictionInput = Input;
pub type RentalSeattleSmc22206160JustCauseEvictionOutput = Output;
pub type RentalSeattleSmc22206160JustCauseEvictionResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "SMC 22.206.160.C — Seattle Just Cause Eviction Ordinance (JCEO); enacted 1980 as first US municipal just-cause eviction ordinance; codified under SMC Title 22 (Building and Construction Codes), Subtitle II (Housing Code), Chapter 22.206 (Habitable Buildings), Subchapter VI (Duties of Owners and Tenants)".to_string(),
        "Scope — applies to all month-to-month tenancies and lease non-renewals on residential property within Seattle; exempts transient lodging (hotels/motels), housing provided by educational institutions, medical-care/correctional occupancy, and live-aboard vessels".to_string(),
        "Enumerated Just Causes — between 16 and 18 enumerated causes (ordinance amended periodically; 2021 amendments most recent major package): non-payment of rent (14-day notice); material noncompliance (10-day notice); chronic late rent (4+ 14-day notices in 12 months + 20-day termination); habitual rule violations (3+ 10-day notices in 12 months + 20-day termination); owner/family occupancy (90-day notice + 60-day good-faith occupancy); single-family-dwelling sale (90-day notice); demolition/substantial rehab/use change (TRAO license required); condominium/cooperative conversion (120-day notice under RCW 64.34.440); tenant refused new lease with substantially identical terms; criminal activity recorded with city; owner quitting shared occupancy; transfer to comparable subsidized unit".to_string(),
        "Owner Move-In Good Faith Rebuttable Presumption — owner must occupy unit for at least 60 consecutive days within 90 days of tenant vacating; failure triggers statutory rebuttable presumption of bad faith; tenant entitled to up to $2,000 statutory damages + actual damages + reasonable attorney fees + court costs".to_string(),
        "Single-Family Sale — owner must provide 90-day written notice prior to date set for vacating; failure to list / show unit triggers $2,000 tenant damages claim".to_string(),
        "TRAO Linkage — SMC § 22.210 Tenant Relocation Assistance Ordinance requires landlord to obtain a tenant relocation license from SDCI BEFORE serving 20-day termination notice for demolition / substantial rehabilitation / change of use; relocation payment per eligible low-income tenant is statutorily updated (50 % SDCI / 50 % landlord)".to_string(),
        "Condominium Conversion — RCW 64.34.440 requires 120-day written notice for condominium / cooperative conversion; JCEO incorporates the state requirement by reference".to_string(),
        "Enforcement — SDCI investigates JCEO complaints; landlord who terminates without qualifying just cause subject to civil penalty + tenant private right of action for unlawful detainer defense, statutory damages, and reasonable attorney fees".to_string(),
        "Historical Significance — Seattle JCEO (1980) is the FIRST municipal just-cause eviction ordinance in the United States, preceding California AB 1482 (2019), Oregon SB 608 (2019), Colorado HB 24-1098 (2024), Washington HB 1217 (2025), and every other US local/state just-cause regime".to_string(),
        "Seattle SDCI — Just Cause Eviction Ordinance program page (primary enforcement agency)".to_string(),
        "Tenants Union of Washington — Just Cause Eviction Protection (tenant-facing summary)".to_string(),
        "Brink Law — How to Evict a Tenant in Seattle (practitioner guide)".to_string(),
        "Law HG — Rental Housing Changes in the City of Seattle (2021 amendments analysis)".to_string(),
        "Washington Supreme Court — review history under JCEO (opinions concerning constitutional / preemption challenges)".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::OutsideSeattleCityLimits {
        return Output {
            mode: SeattleJceoMode::NotApplicablePropertyOutsideSeattle,
            statutory_basis: "Property outside Seattle city limits; SMC 22.206.160.C inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside Seattle city limits; SMC 22.206.160.C just cause eviction ordinance inapplicable.".to_string(),
            citations,
            owner_move_in_violation_damages_cents: 0,
        };
    }

    match input.unit_type {
        UnitType::TransientLodgingHotelMotelExempt => {
            return Output {
                mode: SeattleJceoMode::NotApplicableTransientLodgingExempt,
                statutory_basis: "SMC 22.206.160 — transient lodging exemption".to_string(),
                notes: "NOT APPLICABLE: transient lodging (hotel/motel/short-term housing); SMC 22.206.160.C just cause eviction ordinance does not apply.".to_string(),
                citations,
                owner_move_in_violation_damages_cents: 0,
            };
        }
        UnitType::EducationalInstitutionHousingExempt => {
            return Output {
                mode: SeattleJceoMode::NotApplicableEducationalInstitutionHousingExempt,
                statutory_basis: "SMC 22.206.160 — educational institution housing exemption".to_string(),
                notes: "NOT APPLICABLE: housing provided by educational institution to student/faculty; SMC 22.206.160.C just cause eviction ordinance does not apply.".to_string(),
                citations,
                owner_move_in_violation_damages_cents: 0,
            };
        }
        UnitType::MedicalOrCorrectionalOccupancyExempt => {
            return Output {
                mode: SeattleJceoMode::NotApplicableMedicalOrCorrectionalOccupancyExempt,
                statutory_basis: "SMC 22.206.160 — medical or correctional facility occupancy exemption".to_string(),
                notes: "NOT APPLICABLE: occupancy under medical-care or correctional facility; SMC 22.206.160.C just cause eviction ordinance does not apply.".to_string(),
                citations,
                owner_move_in_violation_damages_cents: 0,
            };
        }
        UnitType::LiveAboardVesselExempt => {
            return Output {
                mode: SeattleJceoMode::NotApplicableLiveAboardVesselExempt,
                statutory_basis: "SMC 22.206.160 — live-aboard vessel exemption".to_string(),
                notes: "NOT APPLICABLE: live-aboard vessel; SMC 22.206.160.C just cause eviction ordinance does not apply.".to_string(),
                citations,
                owner_move_in_violation_damages_cents: 0,
            };
        }
        UnitType::ResidentialMultiFamily | UnitType::ResidentialSingleFamilyDwelling => {}
    }

    match input.just_cause_asserted {
        JustCauseAsserted::NoJustCauseAsserted => {
            Output {
                mode: SeattleJceoMode::ViolationNoticeOfTerminationWithoutAssertingAnyJustCause,
                statutory_basis: "SMC 22.206.160.C — termination prohibited without qualifying just cause".to_string(),
                notes: "VIOLATION: landlord served notice of termination or refused to renew lease without asserting any of the enumerated SMC 22.206.160.C just causes; ordinance prohibits no-cause termination; tenant may assert as affirmative defense to unlawful detainer and seek statutory damages and attorney fees.".to_string(),
                citations,
                owner_move_in_violation_damages_cents: 0,
            }
        }
        JustCauseAsserted::NonPaymentOfRentAfter14DayNotice => {
            Output {
                mode: SeattleJceoMode::CompliantNonPaymentOfRentAfter14DayPayOrVacateNotice,
                statutory_basis: "SMC 22.206.160.C — non-payment of rent after 14-day pay-or-vacate notice (RCW 59.18.057)".to_string(),
                notes: "COMPLIANT: just cause asserted is non-payment of rent following statutory 14-day pay-or-vacate notice under RCW 59.18.057.".to_string(),
                citations,
                owner_move_in_violation_damages_cents: 0,
            }
        }
        JustCauseAsserted::MaterialLeaseNoncomplianceAfter10DayNotice => {
            Output {
                mode: SeattleJceoMode::CompliantMaterialLeaseNoncomplianceAfter10DayComplyOrVacateNotice,
                statutory_basis: "SMC 22.206.160.C — material lease noncompliance after 10-day comply-or-vacate notice".to_string(),
                notes: "COMPLIANT: just cause asserted is material lease noncompliance following 10-day comply-or-vacate notice.".to_string(),
                citations,
                owner_move_in_violation_damages_cents: 0,
            }
        }
        JustCauseAsserted::ChronicLateRent4PlusPayOrVacateNoticesIn12Months => {
            if input.prior_pay_or_vacate_notices_last_12_months
                < SEATTLE_JCEO_CHRONIC_LATE_RENT_NOTICE_THRESHOLD_PER_12_MONTHS
            {
                Output {
                    mode: SeattleJceoMode::ViolationChronicLateRentInsufficientPriorPayOrVacateNoticesUnder4In12Months,
                    statutory_basis: "SMC 22.206.160.C — chronic late rent requires 4+ 14-day pay-or-vacate notices in 12 months".to_string(),
                    notes: format!(
                        "VIOLATION: chronic late rent just cause requires at least 4 prior 14-day pay-or-vacate notices within 12 months; record shows only {} such notices; ordinance threshold not met; 20-day termination notice unenforceable.",
                        input.prior_pay_or_vacate_notices_last_12_months
                    ),
                    citations,
                    owner_move_in_violation_damages_cents: 0,
                }
            } else {
                Output {
                    mode: SeattleJceoMode::CompliantChronicLateRent4PlusPayOrVacateNoticesIn12MonthsWith20DayTermination,
                    statutory_basis: "SMC 22.206.160.C — chronic late rent (4+ 14-day pay-or-vacate notices in 12 months + 20-day termination)".to_string(),
                    notes: format!(
                        "COMPLIANT: chronic late rent just cause supported by {} prior 14-day pay-or-vacate notices within 12 months (≥ 4 threshold); 20-day termination notice may be served.",
                        input.prior_pay_or_vacate_notices_last_12_months
                    ),
                    citations,
                    owner_move_in_violation_damages_cents: 0,
                }
            }
        }
        JustCauseAsserted::HabitualRuleViolations3PlusComplyOrVacateNoticesIn12Months => {
            if input.prior_comply_or_vacate_notices_last_12_months
                < SEATTLE_JCEO_HABITUAL_RULE_VIOLATION_NOTICE_THRESHOLD_PER_12_MONTHS
            {
                Output {
                    mode: SeattleJceoMode::ViolationHabitualRuleViolationsInsufficientPriorComplyOrVacateNoticesUnder3In12Months,
                    statutory_basis: "SMC 22.206.160.C — habitual rule violations require 3+ 10-day comply-or-vacate notices in 12 months".to_string(),
                    notes: format!(
                        "VIOLATION: habitual rule violations just cause requires at least 3 prior 10-day comply-or-vacate notices within 12 months; record shows only {} such notices; ordinance threshold not met; 20-day termination notice unenforceable.",
                        input.prior_comply_or_vacate_notices_last_12_months
                    ),
                    citations,
                    owner_move_in_violation_damages_cents: 0,
                }
            } else {
                Output {
                    mode: SeattleJceoMode::CompliantHabitualRuleViolations3PlusComplyOrVacateNoticesIn12MonthsWith20DayTermination,
                    statutory_basis: "SMC 22.206.160.C — habitual rule violations (3+ 10-day comply-or-vacate notices in 12 months + 20-day termination)".to_string(),
                    notes: format!(
                        "COMPLIANT: habitual rule violations just cause supported by {} prior 10-day comply-or-vacate notices within 12 months (≥ 3 threshold); 20-day termination notice may be served.",
                        input.prior_comply_or_vacate_notices_last_12_months
                    ),
                    citations,
                    owner_move_in_violation_damages_cents: 0,
                }
            }
        }
        JustCauseAsserted::OwnerOrImmediateFamilyOccupancy => {
            if input.notice_days_provided < SEATTLE_JCEO_OWNER_MOVE_IN_NOTICE_DAYS_REQUIRED {
                return Output {
                    mode: SeattleJceoMode::ViolationOwnerOrFamilyOccupancyWithoutRequired90DayNotice,
                    statutory_basis: "SMC 22.206.160.C — owner / immediate-family occupancy requires 90-day written notice".to_string(),
                    notes: format!(
                        "VIOLATION: owner / immediate-family occupancy just cause requires 90-day advance written notice; landlord provided only {} days; termination unenforceable.",
                        input.notice_days_provided
                    ),
                    citations,
                    owner_move_in_violation_damages_cents: 0,
                };
            }
            if input.owner_move_in_occupancy_status
                == OwnerMoveInOccupancyStatus::OccupancyNotCompletedAsRequiredRebuttablePresumptionOfBadFaith
            {
                let damages_dollars = SEATTLE_JCEO_OWNER_MOVE_IN_VIOLATION_DAMAGES_DOLLARS;
                let damages_cents = damages_dollars.saturating_mul(100);
                return Output {
                    mode: SeattleJceoMode::ViolationOwnerMoveInFailureToOccupyTriggeringRebuttablePresumptionAndTenantDamages,
                    statutory_basis: "SMC 22.206.160.C — owner-move-in good-faith rebuttable presumption: at least 60 consecutive days within 90 days of vacate".to_string(),
                    notes: format!(
                        "VIOLATION: owner-move-in just cause asserted but owner did NOT occupy unit for at least 60 consecutive days within 90 days of tenant vacating; statutory rebuttable presumption of bad faith triggered; tenant entitled to up to ${} statutory damages plus actual damages, reasonable attorney fees, and court costs.",
                        damages_dollars
                    ),
                    citations,
                    owner_move_in_violation_damages_cents: damages_cents,
                };
            }
            Output {
                mode: SeattleJceoMode::CompliantOwnerOrImmediateFamilyOccupancyWith90DayNoticeAndGoodFaithOccupancy,
                statutory_basis: "SMC 22.206.160.C — owner / immediate-family occupancy with 90-day notice and good-faith 60-day occupancy".to_string(),
                notes: format!(
                    "COMPLIANT: owner / immediate-family occupancy just cause supported by {}-day notice (≥ 90 required) and good-faith 60-consecutive-day occupancy within 90 days of vacate.",
                    input.notice_days_provided
                ),
                citations,
                owner_move_in_violation_damages_cents: 0,
            }
        }
        JustCauseAsserted::SaleOfSingleFamilyDwelling => {
            if input.notice_days_provided < SEATTLE_JCEO_SINGLE_FAMILY_SALE_NOTICE_DAYS_REQUIRED {
                Output {
                    mode: SeattleJceoMode::ViolationSingleFamilyDwellingSaleWithoutRequired90DayNotice,
                    statutory_basis: "SMC 22.206.160.C — sale of single-family dwelling requires 90-day written notice".to_string(),
                    notes: format!(
                        "VIOLATION: sale of single-family dwelling just cause requires 90-day written notice prior to vacate date; landlord provided only {} days; failure also subjects landlord to ${} tenant damages claim.",
                        input.notice_days_provided,
                        SEATTLE_JCEO_SINGLE_FAMILY_SALE_VIOLATION_DAMAGES_DOLLARS
                    ),
                    citations,
                    owner_move_in_violation_damages_cents: 0,
                }
            } else {
                Output {
                    mode: SeattleJceoMode::CompliantSaleOfSingleFamilyDwellingWith90DayNotice,
                    statutory_basis: "SMC 22.206.160.C — sale of single-family dwelling with 90-day notice".to_string(),
                    notes: format!(
                        "COMPLIANT: sale of single-family dwelling just cause supported by {}-day notice (≥ 90 required); owner must list / show unit within 30 days to maintain good faith.",
                        input.notice_days_provided
                    ),
                    citations,
                    owner_move_in_violation_damages_cents: 0,
                }
            }
        }
        JustCauseAsserted::SubstantialRehabilitationOrDemolitionOrUseChange => {
            if input.trao_license_status
                == TraoLicenseStatus::TraoLicenseObtainedFromSdciPriorToTerminationNotice
            {
                Output {
                    mode: SeattleJceoMode::CompliantSubstantialRehabilitationOrDemolitionOrUseChangeWithTraoLicense,
                    statutory_basis: "SMC 22.206.160.C + SMC 22.210 (TRAO) — substantial rehab / demolition / use change with tenant relocation license".to_string(),
                    notes: "COMPLIANT: substantial rehabilitation / demolition / use change just cause supported by tenant relocation assistance license obtained from SDCI prior to serving 20-day termination notice.".to_string(),
                    citations,
                    owner_move_in_violation_damages_cents: 0,
                }
            } else {
                Output {
                    mode: SeattleJceoMode::ViolationSubstantialRehabilitationOrDemolitionOrUseChangeWithoutTraoLicense,
                    statutory_basis: "SMC 22.206.160.C + SMC 22.210 (TRAO) — TRAO license required before termination notice".to_string(),
                    notes: "VIOLATION: substantial rehabilitation / demolition / use change just cause asserted without first obtaining required tenant relocation assistance license from SDCI; termination notice unenforceable.".to_string(),
                    citations,
                    owner_move_in_violation_damages_cents: 0,
                }
            }
        }
        JustCauseAsserted::ConversionToCondominiumOrCooperative => {
            if input.notice_days_provided < SEATTLE_JCEO_CONDOMINIUM_CONVERSION_NOTICE_DAYS_REQUIRED
            {
                Output {
                    mode: SeattleJceoMode::ViolationConversionToCondominiumOrCooperativeWithoutRequired120DayNotice,
                    statutory_basis: "SMC 22.206.160.C + RCW 64.34.440 — condominium / cooperative conversion requires 120-day written notice".to_string(),
                    notes: format!(
                        "VIOLATION: condominium / cooperative conversion just cause requires 120-day written notice under RCW 64.34.440 (state statute incorporated by JCEO reference); landlord provided only {} days; termination unenforceable.",
                        input.notice_days_provided
                    ),
                    citations,
                    owner_move_in_violation_damages_cents: 0,
                }
            } else {
                Output {
                    mode: SeattleJceoMode::CompliantConversionToCondominiumOrCooperativeWith120DayNotice,
                    statutory_basis: "SMC 22.206.160.C + RCW 64.34.440 — condominium / cooperative conversion with 120-day notice".to_string(),
                    notes: format!(
                        "COMPLIANT: condominium / cooperative conversion just cause supported by {}-day notice (≥ 120 required under RCW 64.34.440).",
                        input.notice_days_provided
                    ),
                    citations,
                    owner_move_in_violation_damages_cents: 0,
                }
            }
        }
        JustCauseAsserted::TenantRefusedToSignNewLeaseSubstantiallyIdenticalTerms => {
            Output {
                mode: SeattleJceoMode::CompliantTenantRefusedToSignNewLeaseSubstantiallyIdenticalTerms,
                statutory_basis: "SMC 22.206.160.C — tenant refused to sign new lease with substantially identical terms".to_string(),
                notes: "COMPLIANT: just cause asserted is tenant's refusal to sign a new lease with terms substantially identical to the expiring lease.".to_string(),
                citations,
                owner_move_in_violation_damages_cents: 0,
            }
        }
        JustCauseAsserted::CriminalActivityRecordedWithCity => {
            Output {
                mode: SeattleJceoMode::CompliantCriminalActivityRecordedWithCity,
                statutory_basis: "SMC 22.206.160.C — criminal activity recorded with the city substantially affecting other tenants or property".to_string(),
                notes: "COMPLIANT: just cause asserted is criminal activity recorded with the City of Seattle that substantially affects use and enjoyment of other tenants or landlord's property interest.".to_string(),
                citations,
                owner_move_in_violation_damages_cents: 0,
            }
        }
        JustCauseAsserted::OwnerQuittingSharedOccupancyWithTenant => {
            Output {
                mode: SeattleJceoMode::CompliantOwnerQuittingSharedOccupancyWithTenant,
                statutory_basis: "SMC 22.206.160.C — owner quitting shared occupancy with tenant in same dwelling".to_string(),
                notes: "COMPLIANT: just cause asserted is owner's election to terminate shared occupancy with tenant in the same dwelling.".to_string(),
                citations,
                owner_move_in_violation_damages_cents: 0,
            }
        }
        JustCauseAsserted::TransferToComparableSubsidizedOrRentRestrictedUnit => {
            Output {
                mode: SeattleJceoMode::CompliantTransferToComparableSubsidizedOrRentRestrictedUnit,
                statutory_basis: "SMC 22.206.160.C — transfer to comparable subsidized or rent-restricted unit".to_string(),
                notes: "COMPLIANT: just cause asserted is required transfer of tenant to comparable subsidized or rent-restricted unit consistent with subsidy program rules.".to_string(),
                citations,
                owner_move_in_violation_damages_cents: 0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::WithinSeattleCityLimits,
            unit_type: UnitType::ResidentialMultiFamily,
            just_cause_asserted: JustCauseAsserted::NonPaymentOfRentAfter14DayNotice,
            prior_pay_or_vacate_notices_last_12_months: 0,
            prior_comply_or_vacate_notices_last_12_months: 0,
            owner_move_in_occupancy_status:
                OwnerMoveInOccupancyStatus::NotApplicableToThisJustCause,
            trao_license_status: TraoLicenseStatus::NotApplicableToThisJustCause,
            notice_days_provided: 0,
            monthly_rent_cents: 250_000,
        }
    }

    #[test]
    fn property_outside_seattle_not_applicable() {
        let mut input = baseline_input();
        input.property_jurisdiction = PropertyJurisdiction::OutsideSeattleCityLimits;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::NotApplicablePropertyOutsideSeattle
        );
        assert_eq!(output.owner_move_in_violation_damages_cents, 0);
    }

    #[test]
    fn transient_lodging_exempt() {
        let mut input = baseline_input();
        input.unit_type = UnitType::TransientLodgingHotelMotelExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::NotApplicableTransientLodgingExempt
        );
    }

    #[test]
    fn educational_housing_exempt() {
        let mut input = baseline_input();
        input.unit_type = UnitType::EducationalInstitutionHousingExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::NotApplicableEducationalInstitutionHousingExempt
        );
    }

    #[test]
    fn live_aboard_vessel_exempt() {
        let mut input = baseline_input();
        input.unit_type = UnitType::LiveAboardVesselExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::NotApplicableLiveAboardVesselExempt
        );
    }

    #[test]
    fn no_just_cause_asserted_violation() {
        let mut input = baseline_input();
        input.just_cause_asserted = JustCauseAsserted::NoJustCauseAsserted;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::ViolationNoticeOfTerminationWithoutAssertingAnyJustCause
        );
        assert!(output.notes.contains("VIOLATION"));
    }

    #[test]
    fn non_payment_of_rent_after_14_day_notice_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantNonPaymentOfRentAfter14DayPayOrVacateNotice
        );
    }

    #[test]
    fn material_noncompliance_after_10_day_notice_compliant() {
        let mut input = baseline_input();
        input.just_cause_asserted = JustCauseAsserted::MaterialLeaseNoncomplianceAfter10DayNotice;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantMaterialLeaseNoncomplianceAfter10DayComplyOrVacateNotice
        );
    }

    #[test]
    fn chronic_late_rent_with_4_prior_notices_compliant() {
        let mut input = baseline_input();
        input.just_cause_asserted =
            JustCauseAsserted::ChronicLateRent4PlusPayOrVacateNoticesIn12Months;
        input.prior_pay_or_vacate_notices_last_12_months = 4;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantChronicLateRent4PlusPayOrVacateNoticesIn12MonthsWith20DayTermination
        );
    }

    #[test]
    fn chronic_late_rent_with_3_prior_notices_violation() {
        let mut input = baseline_input();
        input.just_cause_asserted =
            JustCauseAsserted::ChronicLateRent4PlusPayOrVacateNoticesIn12Months;
        input.prior_pay_or_vacate_notices_last_12_months = 3;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::ViolationChronicLateRentInsufficientPriorPayOrVacateNoticesUnder4In12Months
        );
        assert!(output.notes.contains("only 3"));
    }

    #[test]
    fn habitual_rule_violations_with_3_prior_notices_compliant() {
        let mut input = baseline_input();
        input.just_cause_asserted =
            JustCauseAsserted::HabitualRuleViolations3PlusComplyOrVacateNoticesIn12Months;
        input.prior_comply_or_vacate_notices_last_12_months = 3;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantHabitualRuleViolations3PlusComplyOrVacateNoticesIn12MonthsWith20DayTermination
        );
    }

    #[test]
    fn habitual_rule_violations_with_2_prior_notices_violation() {
        let mut input = baseline_input();
        input.just_cause_asserted =
            JustCauseAsserted::HabitualRuleViolations3PlusComplyOrVacateNoticesIn12Months;
        input.prior_comply_or_vacate_notices_last_12_months = 2;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::ViolationHabitualRuleViolationsInsufficientPriorComplyOrVacateNoticesUnder3In12Months
        );
        assert!(output.notes.contains("only 2"));
    }

    #[test]
    fn owner_move_in_with_90_day_notice_and_good_faith_compliant() {
        let mut input = baseline_input();
        input.just_cause_asserted = JustCauseAsserted::OwnerOrImmediateFamilyOccupancy;
        input.notice_days_provided = 90;
        input.owner_move_in_occupancy_status =
            OwnerMoveInOccupancyStatus::OccupancyCompletedAtOrAbove60ConsecutiveDaysWithin90DaysOfVacate;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantOwnerOrImmediateFamilyOccupancyWith90DayNoticeAndGoodFaithOccupancy
        );
        assert_eq!(output.owner_move_in_violation_damages_cents, 0);
    }

    #[test]
    fn owner_move_in_without_90_day_notice_violation() {
        let mut input = baseline_input();
        input.just_cause_asserted = JustCauseAsserted::OwnerOrImmediateFamilyOccupancy;
        input.notice_days_provided = 60;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::ViolationOwnerOrFamilyOccupancyWithoutRequired90DayNotice
        );
    }

    #[test]
    fn owner_move_in_failure_to_occupy_triggers_2000_dollar_damages() {
        let mut input = baseline_input();
        input.just_cause_asserted = JustCauseAsserted::OwnerOrImmediateFamilyOccupancy;
        input.notice_days_provided = 90;
        input.owner_move_in_occupancy_status =
            OwnerMoveInOccupancyStatus::OccupancyNotCompletedAsRequiredRebuttablePresumptionOfBadFaith;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::ViolationOwnerMoveInFailureToOccupyTriggeringRebuttablePresumptionAndTenantDamages
        );
        assert_eq!(output.owner_move_in_violation_damages_cents, 200_000);
        assert!(output.notes.contains("$2000"));
    }

    #[test]
    fn single_family_sale_with_90_day_notice_compliant() {
        let mut input = baseline_input();
        input.unit_type = UnitType::ResidentialSingleFamilyDwelling;
        input.just_cause_asserted = JustCauseAsserted::SaleOfSingleFamilyDwelling;
        input.notice_days_provided = 90;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantSaleOfSingleFamilyDwellingWith90DayNotice
        );
    }

    #[test]
    fn single_family_sale_without_90_day_notice_violation() {
        let mut input = baseline_input();
        input.unit_type = UnitType::ResidentialSingleFamilyDwelling;
        input.just_cause_asserted = JustCauseAsserted::SaleOfSingleFamilyDwelling;
        input.notice_days_provided = 60;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::ViolationSingleFamilyDwellingSaleWithoutRequired90DayNotice
        );
    }

    #[test]
    fn substantial_rehab_with_trao_license_compliant() {
        let mut input = baseline_input();
        input.just_cause_asserted =
            JustCauseAsserted::SubstantialRehabilitationOrDemolitionOrUseChange;
        input.trao_license_status =
            TraoLicenseStatus::TraoLicenseObtainedFromSdciPriorToTerminationNotice;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantSubstantialRehabilitationOrDemolitionOrUseChangeWithTraoLicense
        );
    }

    #[test]
    fn substantial_rehab_without_trao_license_violation() {
        let mut input = baseline_input();
        input.just_cause_asserted =
            JustCauseAsserted::SubstantialRehabilitationOrDemolitionOrUseChange;
        input.trao_license_status = TraoLicenseStatus::TraoLicenseNotObtained;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::ViolationSubstantialRehabilitationOrDemolitionOrUseChangeWithoutTraoLicense
        );
    }

    #[test]
    fn condominium_conversion_with_120_day_notice_compliant() {
        let mut input = baseline_input();
        input.just_cause_asserted = JustCauseAsserted::ConversionToCondominiumOrCooperative;
        input.notice_days_provided = 120;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantConversionToCondominiumOrCooperativeWith120DayNotice
        );
    }

    #[test]
    fn condominium_conversion_without_120_day_notice_violation() {
        let mut input = baseline_input();
        input.just_cause_asserted = JustCauseAsserted::ConversionToCondominiumOrCooperative;
        input.notice_days_provided = 90;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::ViolationConversionToCondominiumOrCooperativeWithoutRequired120DayNotice
        );
    }

    #[test]
    fn tenant_refused_new_lease_compliant() {
        let mut input = baseline_input();
        input.just_cause_asserted =
            JustCauseAsserted::TenantRefusedToSignNewLeaseSubstantiallyIdenticalTerms;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantTenantRefusedToSignNewLeaseSubstantiallyIdenticalTerms
        );
    }

    #[test]
    fn criminal_activity_recorded_compliant() {
        let mut input = baseline_input();
        input.just_cause_asserted = JustCauseAsserted::CriminalActivityRecordedWithCity;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantCriminalActivityRecordedWithCity
        );
    }

    #[test]
    fn owner_quitting_shared_occupancy_compliant() {
        let mut input = baseline_input();
        input.just_cause_asserted = JustCauseAsserted::OwnerQuittingSharedOccupancyWithTenant;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantOwnerQuittingSharedOccupancyWithTenant
        );
    }

    #[test]
    fn transfer_to_subsidized_unit_compliant() {
        let mut input = baseline_input();
        input.just_cause_asserted =
            JustCauseAsserted::TransferToComparableSubsidizedOrRentRestrictedUnit;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SeattleJceoMode::CompliantTransferToComparableSubsidizedOrRentRestrictedUnit
        );
    }

    #[test]
    fn constants_pin_statutory_thresholds() {
        assert_eq!(SEATTLE_JCEO_ENACTMENT_YEAR, 1980);
        assert_eq!(
            SEATTLE_JCEO_CHRONIC_LATE_RENT_NOTICE_THRESHOLD_PER_12_MONTHS,
            4
        );
        assert_eq!(
            SEATTLE_JCEO_HABITUAL_RULE_VIOLATION_NOTICE_THRESHOLD_PER_12_MONTHS,
            3
        );
        assert_eq!(SEATTLE_JCEO_NON_PAYMENT_OF_RENT_NOTICE_DAYS_REQUIRED, 14);
        assert_eq!(SEATTLE_JCEO_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS_REQUIRED, 10);
        assert_eq!(SEATTLE_JCEO_TERMINATION_NOTICE_DAYS_RULE_VIOLATIONS, 20);
        assert_eq!(SEATTLE_JCEO_OWNER_MOVE_IN_NOTICE_DAYS_REQUIRED, 90);
        assert_eq!(SEATTLE_JCEO_SINGLE_FAMILY_SALE_NOTICE_DAYS_REQUIRED, 90);
        assert_eq!(
            SEATTLE_JCEO_CONDOMINIUM_CONVERSION_NOTICE_DAYS_REQUIRED,
            120
        );
        assert_eq!(SEATTLE_JCEO_OWNER_MOVE_IN_OCCUPANCY_DAYS_REQUIRED, 60);
        assert_eq!(
            SEATTLE_JCEO_OWNER_MOVE_IN_OCCUPANCY_DEADLINE_DAYS_AFTER_VACATE,
            90
        );
        assert_eq!(SEATTLE_JCEO_OWNER_MOVE_IN_VIOLATION_DAMAGES_DOLLARS, 2_000);
        assert_eq!(
            SEATTLE_JCEO_SINGLE_FAMILY_SALE_VIOLATION_DAMAGES_DOLLARS,
            2_000
        );
        assert_eq!(SEATTLE_JCEO_LOOKBACK_MONTHS_NOTICE_COUNTING, 12);
    }

    #[test]
    fn citation_contains_smc_22_206_160() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("SMC 22.206.160"));
        assert!(joined.contains("1980"));
        assert!(joined.contains("first US municipal just-cause"));
    }

    #[test]
    fn owner_move_in_damages_saturating_overflow_defense() {
        let damages_dollars = SEATTLE_JCEO_OWNER_MOVE_IN_VIOLATION_DAMAGES_DOLLARS;
        let damages_cents = damages_dollars.saturating_mul(100);
        assert_eq!(damages_cents, 200_000);
        let max_dollars: u64 = u64::MAX;
        let max_cents = max_dollars.saturating_mul(100);
        assert_eq!(max_cents, u64::MAX);
    }
}
