//! Indiana Code Title 32, Article 31 — Landlord-Tenant Relations
//! Compliance Module — pure-compute check for landlord
//! statutory compliance with Indiana's non-URLTA hybrid
//! landlord-tenant regime. Indiana did NOT adopt the
//! Uniform Residential Landlord and Tenant Act (URLTA); its
//! landlord-tenant law is codified across multiple chapters
//! of Title 32, Article 31 of the Indiana Code, including
//! Chapter 3 (Security Deposits), Chapter 7 (Tenant
//! Obligations), and Chapter 8 (Landlord Obligations Under
//! a Rental Agreement).
//!
//! **Distinctive Indiana feature**: Indiana imposes **NO
//! STATUTORY CAP** on the amount of security deposit a
//! landlord may charge (one of only a handful of US states
//! without a statutory cap). Indiana law instead focuses on
//! return-deadline + itemized-accounting procedural
//! safeguards under **IC § 32-31-3-12** (45-day return
//! deadline) and **IC § 32-31-3-14** (itemized notice of
//! damages).
//!
//! Web research (verified 2026-06-03):
//! - **IC § 32-31-3 Security Deposits Chapter**: governs landlord retention and return of residential security deposits ([Indiana General Assembly — 2024 Indiana Code Title 32 Article 31](https://iga.in.gov/laws/2024/ic/titles/32/articles/31); [Justia — 2025 Indiana Code Title 32 Article 31](https://law.justia.com/codes/indiana/title-32/article-31/); [Justia — IC § 32-31-3-12 Return of Deposits; Deductions; Liability](https://law.justia.com/codes/indiana/title-32/article-31/chapter-3/section-32-31-3-12/); [FindLaw — IC § 32-31-3-12](https://codes.findlaw.com/in/title-32-property/in-code-sect-32-31-3-12/); [Justia — IC § 32-31-3-14 Notice of Damages](https://law.justia.com/codes/indiana/title-32/article-31/chapter-3/section-32-31-3-14/); [Rentable — Indiana Security Deposit Laws](https://www.rentable.com/blog/indiana-security-deposit-laws-a-complete-guide-for-landlords-tenants/); [Innago — Indiana Landlord Tenant Rental Laws](https://innago.com/indiana-landlord-tenant-laws/); [Mann Law — Landlords and Tenants Beware](https://www.rmannlawoffice.com/blog/2020/07/landlords-and-tenants-beware/)).
//! - **IC § 32-31-3-12 45-Day Deposit Return**: upon termination of a rental agreement, landlord must return security deposit minus any amount applied to permissible deductions, all as **ITEMIZED by the landlord with the amount due in a written notice that is delivered to the tenant NOT MORE THAN FORTY-FIVE (45) DAYS** after termination of the rental agreement and delivery of possession.
//! - **IC § 32-31-3-13 Permissible Deductions**: security deposit may be used to reimburse the landlord for (i) actual damages to the rental unit or any ancillary facility that are NOT THE RESULT OF ORDINARY WEAR AND TEAR, (ii) all rent in arrearage under the rental agreement, AND (iii) rent due for premature termination of the rental agreement by the tenant.
//! - **IC § 32-31-3-9 Tenant Mailing Address Required**: landlord is NOT LIABLE under IC § 32-31-3-12 until tenant supplies the landlord IN WRITING with a mailing address to which to deliver the notice and amount; failure of tenant to provide written mailing address tolls the 45-day deadline.
//! - **IC § 32-31-3 No Statutory Deposit Cap**: Indiana is one of only a handful of US states with **NO STATUTORY CAP** on security deposit amount; landlords may charge any amount mutually agreed to in the rental agreement.
//! - **Tenant Remedy for Landlord Failure**: if a landlord fails to comply with IC § 32-31-3-12 procedures, tenant may recover all of the security deposit due AND **REASONABLE ATTORNEY'S FEES**.
//! - **IC § 32-31-8 Landlord Obligations Under a Rental Agreement**: codifies landlord's statutory duty to maintain residential rental property in safe, clean, and habitable condition ([Justia — IC § 32-31-8-5 Landlord Obligations](https://law.justia.com/codes/indiana/title-32/article-31/chapter-8/section-32-31-8-5/); [LawServer — IC § 32-31-8-5](https://www.lawserver.com/law/state/indiana/in-code/indiana_code_32-31-8-5); [McNeely Law — What is Your Landlord Obligated to Do for You?](https://www.mcneelylaw.com/what-is-your-landlord-obligated-to-do-for-you/); [In.gov Vermillion County Health — IC 32-31-7 Tenant Obligations PDF](https://www.in.gov/localhealth/vermillioncounty/files/Indiana-Code-for-Tenant-and-Landlord-Rights.pdf); [Indiana Bar Foundation — Habitability Enforcement in Indiana 2024](https://www.inbarfoundation.org/wp-content/uploads/2024/10/HABITA_.pdf)).
//! - **IC § 32-31-8-5 Landlord Obligations**: landlord MUST (i) deliver the rental premises to a tenant in compliance with the rental agreement, and in a **SAFE, CLEAN, AND HABITABLE CONDITION**; (ii) comply with all **HEALTH AND HOUSING CODES** applicable to the rental premises; (iii) make all reasonable efforts to keep **COMMON AREAS** of a rental premises in a clean and proper condition; (iv) **PROVIDE AND MAINTAIN PLUMBING SYSTEMS** sufficient to accommodate a reasonable supply of **HOT AND COLD RUNNING WATER AT ALL TIMES**; (v) provide and maintain electrical, gas, mechanical (heating, ventilating, air conditioning), and sanitary systems and appliances supplied or required to be supplied.
//! - **IC § 32-31-8-6 Tenant Enforcement Right with Notice Prerequisites**: a tenant may bring an action in a court with jurisdiction to enforce an obligation of a landlord under this chapter, but **ONLY IF**: (i) the tenant gives the landlord **NOTICE** of the landlord's noncompliance; (ii) the landlord has been given a **REASONABLE AMOUNT OF TIME** to make repairs or provide a remedy; AND (iii) the landlord **FAILS OR REFUSES** to repair or remedy the condition described in the tenant's notice.
//! - **IC § 32-31-7-5 Tenant Obligations**: tenants must (i) comply with all obligations imposed primarily on a tenant by applicable provisions of **HEALTH AND HOUSING CODES**; (ii) keep the areas of the rental premises occupied or used by the tenant **REASONABLY CLEAN**; (iii) refrain from **DEFACING, DAMAGING, DESTROYING, IMPAIRING, or REMOVING** any part of the rental premises; (iv) ensure that each **SMOKE DETECTOR** installed in the tenant's rental unit remains **FUNCTIONAL AND IS NOT DISABLED**; AND (v) if the smoke detector is **BATTERY OPERATED**, the tenant shall **REPLACE BATTERIES** as necessary.
//! - **IC § 32-31-1-6 Ten-Day Pay or Quit for Nonpayment**: Indiana requires a **10-DAY PAY OR QUIT NOTICE** before filing an eviction action for nonpayment of rent; tenant has the entire 10-day period to cure by paying full rent owed.
//! - **IC § 32-31-2.9 Methamphetamine Contamination Disclosure**: separate Indiana statute requires landlord to disclose to prospective tenants if rental property has been used for methamphetamine manufacturing or was previously found contaminated and has not been remediated.
//! - **NOTE on URLTA**: Indiana did NOT adopt the Uniform Residential Landlord and Tenant Act; Indiana's hybrid regime under IC § 32-31 operates independently of URLTA.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IN_LANDLORD_TENANT_ARTICLE_NUMBER: u32 = 31;
pub const IN_LANDLORD_TENANT_TITLE_NUMBER: u32 = 32;
pub const IN_LANDLORD_TENANT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 45;
pub const IN_LANDLORD_TENANT_PAY_OR_QUIT_DEMAND_DAYS: u32 = 10;
pub const IN_LANDLORD_TENANT_HAS_STATUTORY_DEPOSIT_CAP: bool = false;
pub const IN_LANDLORD_TENANT_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromArticle31,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantMailingAddressStatus {
    WrittenMailingAddressProvidedByTenant,
    WrittenMailingAddressNotProvidedByTenant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeAndReasonableTimeStatus {
    TenantGaveNoticeAndLandlordHadReasonableTime,
    TenantDidNotGiveNotice,
    TenantGaveNoticeButLandlordNotGivenReasonableTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositReturnUnderSection32_31_3_12,
    PermissibleDeductionsUnderSection32_31_3_13,
    TenantMailingAddressRequirementUnderSection32_31_3_9,
    LandlordObligationToMaintainHabitabilityUnderSection32_31_8_5,
    LandlordPlumbingAndHotWaterObligationUnderSection32_31_8_5,
    TenantEnforcementNoticePrerequisiteUnderSection32_31_8_6,
    TenantObligationsUnderSection32_31_7_5,
    TenDayPayOrQuitNonpaymentNoticeUnderSection32_31_1_6,
    MethamphetamineContaminationDisclosureUnderSection32_31_2_9,
    NoStatutoryDepositCapAcknowledgement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InLandlordTenantMode {
    NotApplicableTenancyExemptFromArticle31,
    CompliantDepositReturnedWithItemizedNoticeWithin45Days,
    CompliantDepositRetentionForPermissibleDeductionsOnly,
    CompliantTenantProvidedWrittenMailingAddress,
    CompliantLandlordMaintainsSafeCleanHabitableCondition,
    CompliantLandlordProvidesHotAndColdRunningWaterAtAllTimes,
    CompliantTenantGaveNoticeAndLandlordHadReasonableTimeToRepair,
    CompliantTenantMeetsCleanlinessAndSmokeDetectorObligations,
    CompliantTenDayPayOrQuitNoticeProperlyServed,
    CompliantMethamphetamineContaminationDisclosureProvided,
    CompliantNoStatutoryDepositCapAcknowledged,
    ViolationDepositReturnedPast45DayDeadline,
    ViolationDepositRetainedForOrdinaryWearAndTear,
    ViolationLandlordFailedToMaintainSafeCleanHabitableCondition,
    ViolationLandlordFailedToProvideHotAndColdRunningWater,
    ViolationTenantEnforcementWithoutNoticeOrReasonableTime,
    ViolationTenantBreachesCleanlinessOrSmokeDetectorObligation,
    ViolationTenDayPayOrQuitNoticeShorterThanTenDays,
    ViolationMethamphetamineContaminationDisclosureOmitted,
    NotLiableTenantDidNotProvideWrittenMailingAddress,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub tenant_mailing_address_status: TenantMailingAddressStatus,
    pub notice_and_reasonable_time_status: NoticeAndReasonableTimeStatus,
    pub compliance_aspect: ComplianceAspect,
    pub days_to_return_deposit: u32,
    pub deposit_withheld_for_ordinary_wear_and_tear: bool,
    pub landlord_maintains_safe_clean_habitable_condition: bool,
    pub landlord_provides_hot_and_cold_running_water_at_all_times: bool,
    pub tenant_meets_cleanliness_and_smoke_detector_obligations: bool,
    pub pay_or_quit_notice_days_given: u32,
    pub methamphetamine_disclosure_provided: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: InLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type InLandlordTenantInput = Input;
pub type InLandlordTenantOutput = Output;
pub type InLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Indiana Code Title 32, Article 31 (Landlord-Tenant Relations) — non-URLTA hybrid Indiana landlord-tenant law; Indiana did NOT adopt the Uniform Residential Landlord and Tenant Act; Article 31 covers IC §§ 32-31-1 through 32-31-9".to_string(),
        "IC § 32-31-3-12 Return of Deposits — landlord shall return security deposit minus any amount applied to permissible deductions, all as ITEMIZED by the landlord with the amount due in a written notice that is delivered to the tenant NOT MORE THAN FORTY-FIVE (45) DAYS after termination of the rental agreement and delivery of possession".to_string(),
        "IC § 32-31-3-13 Permissible Deductions — security deposit may be used to reimburse the landlord for (i) actual damages to the rental unit or any ancillary facility that are NOT THE RESULT OF ORDINARY WEAR AND TEAR, (ii) all rent in arrearage under the rental agreement, AND (iii) rent due for premature termination of the rental agreement by the tenant".to_string(),
        "IC § 32-31-3-9 Tenant Mailing Address Required — landlord is NOT LIABLE under IC § 32-31-3-12 until tenant supplies the landlord IN WRITING with a mailing address to which to deliver the notice and amount; failure of tenant to provide written mailing address tolls the 45-day deadline".to_string(),
        "IC § 32-31-3 No Statutory Deposit Cap — Indiana is one of only a handful of US states with NO STATUTORY CAP on security deposit amount; landlords may charge any amount mutually agreed to in the rental agreement".to_string(),
        "IC § 32-31-3 Tenant Remedy for Landlord Failure — if a landlord fails to comply with IC § 32-31-3-12 procedures, tenant may recover all of the security deposit due AND REASONABLE ATTORNEY'S FEES".to_string(),
        "IC § 32-31-8-5 Landlord Obligations — landlord MUST (i) deliver the rental premises in a SAFE, CLEAN, AND HABITABLE CONDITION; (ii) comply with all HEALTH AND HOUSING CODES; (iii) keep COMMON AREAS in a clean and proper condition; (iv) PROVIDE AND MAINTAIN PLUMBING SYSTEMS sufficient for HOT AND COLD RUNNING WATER AT ALL TIMES; (v) provide and maintain electrical, gas, mechanical, and sanitary systems and appliances".to_string(),
        "IC § 32-31-8-6 Tenant Enforcement Right with Notice Prerequisites — tenant may bring action to enforce landlord obligation ONLY IF (i) tenant gives the landlord NOTICE of noncompliance; (ii) landlord has been given a REASONABLE AMOUNT OF TIME to make repairs or provide a remedy; AND (iii) landlord FAILS OR REFUSES to repair or remedy the condition".to_string(),
        "IC § 32-31-7-5 Tenant Obligations — tenants must (i) comply with HEALTH AND HOUSING CODES; (ii) keep rental premises REASONABLY CLEAN; (iii) refrain from DEFACING, DAMAGING, DESTROYING, IMPAIRING, or REMOVING any part of the rental premises; (iv) ensure each SMOKE DETECTOR remains FUNCTIONAL AND IS NOT DISABLED; (v) if BATTERY OPERATED, the tenant shall REPLACE BATTERIES as necessary".to_string(),
        "IC § 32-31-1-6 Ten-Day Pay or Quit for Nonpayment — Indiana requires a 10-DAY PAY OR QUIT NOTICE before filing an eviction action for nonpayment of rent".to_string(),
        "IC § 32-31-2.9 Methamphetamine Contamination Disclosure — landlord must disclose to prospective tenants if rental property has been used for methamphetamine manufacturing or was previously found contaminated and has not been remediated".to_string(),
        "Indiana General Assembly + Justia + FindLaw + LawServer + Rentable + Innago + McNeely Law + Mann Law + Indiana Bar Foundation Habitability Enforcement Conference 2024 — practitioner overviews of Indiana Code Title 32 Article 31".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromArticle31 {
        return Output {
            mode: InLandlordTenantMode::NotApplicableTenancyExemptFromArticle31,
            statutory_basis: "Indiana Code Title 32, Article 31 jurisdiction — tenancy exempt from Article 31 coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from Indiana Code Title 32, Article 31; landlord-tenant obligations under Article 31 unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositReturnUnderSection32_31_3_12 => {
            if input.tenant_mailing_address_status
                == TenantMailingAddressStatus::WrittenMailingAddressNotProvidedByTenant
            {
                return Output {
                    mode: InLandlordTenantMode::NotLiableTenantDidNotProvideWrittenMailingAddress,
                    statutory_basis: "IC § 32-31-3-9 — landlord not liable until tenant provides written mailing address".to_string(),
                    notes: "NOT LIABLE: tenant did not provide written mailing address; 45-day return deadline tolled until written address provided.".to_string(),
                    citations,
                };
            }
            if input.days_to_return_deposit
                <= IN_LANDLORD_TENANT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS
            {
                Output {
                    mode: InLandlordTenantMode::CompliantDepositReturnedWithItemizedNoticeWithin45Days,
                    statutory_basis: "IC § 32-31-3-12 — security deposit returned with itemized notice within 45 days of termination + delivery of possession".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit returned with itemized written notice within {d} days (statutory deadline is 45 days under IC § 32-31-3-12).",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: InLandlordTenantMode::ViolationDepositReturnedPast45DayDeadline,
                    statutory_basis: "IC § 32-31-3-12 — security deposit return exceeded 45-day deadline".to_string(),
                    notes: format!(
                        "VIOLATION: deposit returned at {d} days exceeds 45-day statutory deadline under IC § 32-31-3-12; tenant may recover full deposit + reasonable attorney's fees.",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::PermissibleDeductionsUnderSection32_31_3_13 => {
            if input.deposit_withheld_for_ordinary_wear_and_tear {
                Output {
                    mode: InLandlordTenantMode::ViolationDepositRetainedForOrdinaryWearAndTear,
                    statutory_basis: "IC § 32-31-3-13 — security deposit may NOT be retained for ordinary wear and tear".to_string(),
                    notes: "VIOLATION: deposit retained for ordinary wear and tear; only actual damages NOT result of ordinary wear and tear permitted under IC § 32-31-3-13.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: InLandlordTenantMode::CompliantDepositRetentionForPermissibleDeductionsOnly,
                    statutory_basis: "IC § 32-31-3-13 — security deposit retention limited to actual damages (NOT ordinary wear and tear), rent in arrearage, and rent due for premature termination".to_string(),
                    notes: "COMPLIANT: deposit retention limited to permissible deductions under IC § 32-31-3-13 (actual damages NOT ordinary wear and tear + rent in arrearage + premature-termination rent).".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::TenantMailingAddressRequirementUnderSection32_31_3_9 => {
            if input.tenant_mailing_address_status
                == TenantMailingAddressStatus::WrittenMailingAddressProvidedByTenant
            {
                Output {
                    mode: InLandlordTenantMode::CompliantTenantProvidedWrittenMailingAddress,
                    statutory_basis: "IC § 32-31-3-9 — tenant provided written mailing address to landlord".to_string(),
                    notes: "COMPLIANT: tenant provided written mailing address; landlord's 45-day deadline under IC § 32-31-3-12 begins on date of termination + delivery of possession.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: InLandlordTenantMode::NotLiableTenantDidNotProvideWrittenMailingAddress,
                    statutory_basis: "IC § 32-31-3-9 — landlord not liable under IC § 32-31-3-12 without written tenant mailing address".to_string(),
                    notes: "NOT LIABLE: tenant did not provide written mailing address; IC § 32-31-3-12 45-day deadline tolled.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordObligationToMaintainHabitabilityUnderSection32_31_8_5 => {
            if input.landlord_maintains_safe_clean_habitable_condition {
                Output {
                    mode: InLandlordTenantMode::CompliantLandlordMaintainsSafeCleanHabitableCondition,
                    statutory_basis: "IC § 32-31-8-5 — landlord maintains safe, clean, and habitable condition + health and housing code compliance".to_string(),
                    notes: "COMPLIANT: landlord delivers and maintains rental premises in safe, clean, and habitable condition per IC § 32-31-8-5.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: InLandlordTenantMode::ViolationLandlordFailedToMaintainSafeCleanHabitableCondition,
                    statutory_basis: "IC § 32-31-8-5 — landlord failed to maintain safe, clean, and habitable condition".to_string(),
                    notes: "VIOLATION: landlord failed to maintain rental premises in safe, clean, and habitable condition required by IC § 32-31-8-5.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordPlumbingAndHotWaterObligationUnderSection32_31_8_5 => {
            if input.landlord_provides_hot_and_cold_running_water_at_all_times {
                Output {
                    mode: InLandlordTenantMode::CompliantLandlordProvidesHotAndColdRunningWaterAtAllTimes,
                    statutory_basis: "IC § 32-31-8-5 — landlord provides and maintains plumbing for hot and cold running water at all times".to_string(),
                    notes: "COMPLIANT: landlord provides and maintains plumbing systems sufficient to accommodate a reasonable supply of hot and cold running water at all times per IC § 32-31-8-5.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: InLandlordTenantMode::ViolationLandlordFailedToProvideHotAndColdRunningWater,
                    statutory_basis: "IC § 32-31-8-5 — landlord failed to provide hot and cold running water at all times".to_string(),
                    notes: "VIOLATION: landlord failed to provide and maintain plumbing systems for hot and cold running water at all times under IC § 32-31-8-5.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::TenantEnforcementNoticePrerequisiteUnderSection32_31_8_6 => {
            match input.notice_and_reasonable_time_status {
                NoticeAndReasonableTimeStatus::TenantGaveNoticeAndLandlordHadReasonableTime => {
                    Output {
                        mode: InLandlordTenantMode::CompliantTenantGaveNoticeAndLandlordHadReasonableTimeToRepair,
                        statutory_basis: "IC § 32-31-8-6 — tenant gave notice and landlord had reasonable time to repair".to_string(),
                        notes: "COMPLIANT: tenant satisfied notice + reasonable time prerequisites under IC § 32-31-8-6; tenant may bring enforcement action if landlord failed or refused to remedy.".to_string(),
                        citations,
                    }
                }
                _ => Output {
                    mode: InLandlordTenantMode::ViolationTenantEnforcementWithoutNoticeOrReasonableTime,
                    statutory_basis: "IC § 32-31-8-6 — tenant enforcement requires notice + reasonable time prerequisites".to_string(),
                    notes: "VIOLATION: tenant attempted enforcement without satisfying IC § 32-31-8-6 prerequisites (written notice of noncompliance + reasonable time for landlord to repair).".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::TenantObligationsUnderSection32_31_7_5 => {
            if input.tenant_meets_cleanliness_and_smoke_detector_obligations {
                Output {
                    mode: InLandlordTenantMode::CompliantTenantMeetsCleanlinessAndSmokeDetectorObligations,
                    statutory_basis: "IC § 32-31-7-5 — tenant meets cleanliness, no-damage, and smoke-detector functionality obligations".to_string(),
                    notes: "COMPLIANT: tenant complies with health/housing codes, keeps premises reasonably clean, refrains from damage, and maintains functional smoke detectors + batteries per IC § 32-31-7-5.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: InLandlordTenantMode::ViolationTenantBreachesCleanlinessOrSmokeDetectorObligation,
                    statutory_basis: "IC § 32-31-7-5 — tenant breaches cleanliness or smoke detector obligation".to_string(),
                    notes: "VIOLATION: tenant breached cleanliness / no-damage / smoke detector functionality obligation under IC § 32-31-7-5.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::TenDayPayOrQuitNonpaymentNoticeUnderSection32_31_1_6 => {
            if input.pay_or_quit_notice_days_given >= IN_LANDLORD_TENANT_PAY_OR_QUIT_DEMAND_DAYS {
                Output {
                    mode: InLandlordTenantMode::CompliantTenDayPayOrQuitNoticeProperlyServed,
                    statutory_basis: "IC § 32-31-1-6 — 10-day pay or quit notice for nonpayment properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day pay or quit notice satisfies 10-day statutory minimum under IC § 32-31-1-6.",
                        d = input.pay_or_quit_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: InLandlordTenantMode::ViolationTenDayPayOrQuitNoticeShorterThanTenDays,
                    statutory_basis: "IC § 32-31-1-6 — pay or quit notice shorter than statutory 10-day minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day pay or quit notice shorter than 10-day statutory minimum under IC § 32-31-1-6.",
                        d = input.pay_or_quit_notice_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::MethamphetamineContaminationDisclosureUnderSection32_31_2_9 => {
            if input.methamphetamine_disclosure_provided {
                Output {
                    mode: InLandlordTenantMode::CompliantMethamphetamineContaminationDisclosureProvided,
                    statutory_basis: "IC § 32-31-2.9 — methamphetamine contamination disclosure provided to prospective tenants".to_string(),
                    notes: "COMPLIANT: landlord provided methamphetamine contamination disclosure to prospective tenants per IC § 32-31-2.9.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: InLandlordTenantMode::ViolationMethamphetamineContaminationDisclosureOmitted,
                    statutory_basis: "IC § 32-31-2.9 — methamphetamine contamination disclosure required but omitted".to_string(),
                    notes: "VIOLATION: methamphetamine contamination disclosure omitted under IC § 32-31-2.9; required for rental properties used for methamphetamine manufacturing and not remediated.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::NoStatutoryDepositCapAcknowledgement => Output {
            mode: InLandlordTenantMode::CompliantNoStatutoryDepositCapAcknowledged,
            statutory_basis: "IC § 32-31-3 — Indiana imposes NO statutory cap on security deposit amount".to_string(),
            notes: "COMPLIANT: Indiana is one of only a handful of US states with NO statutory cap on security deposit amount; landlords may charge any mutually-agreed-to amount under the rental agreement.".to_string(),
            citations,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            tenancy_coverage: TenancyCoverage::CoveredResidentialTenancy,
            tenant_mailing_address_status:
                TenantMailingAddressStatus::WrittenMailingAddressProvidedByTenant,
            notice_and_reasonable_time_status:
                NoticeAndReasonableTimeStatus::TenantGaveNoticeAndLandlordHadReasonableTime,
            compliance_aspect: ComplianceAspect::SecurityDepositReturnUnderSection32_31_3_12,
            days_to_return_deposit: 30,
            deposit_withheld_for_ordinary_wear_and_tear: false,
            landlord_maintains_safe_clean_habitable_condition: true,
            landlord_provides_hot_and_cold_running_water_at_all_times: true,
            tenant_meets_cleanliness_and_smoke_detector_obligations: true,
            pay_or_quit_notice_days_given: 10,
            methamphetamine_disclosure_provided: true,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromArticle31;
        let out = check(&input);
        assert_eq!(out.mode, InLandlordTenantMode::NotApplicableTenancyExemptFromArticle31);
    }

    #[test]
    fn deposit_returned_within_45_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection32_31_3_12;
        input.days_to_return_deposit = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::CompliantDepositReturnedWithItemizedNoticeWithin45Days
        );
    }

    #[test]
    fn deposit_returned_at_45_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection32_31_3_12;
        input.days_to_return_deposit = 45;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::CompliantDepositReturnedWithItemizedNoticeWithin45Days
        );
    }

    #[test]
    fn deposit_returned_at_46_day_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection32_31_3_12;
        input.days_to_return_deposit = 46;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::ViolationDepositReturnedPast45DayDeadline
        );
    }

    #[test]
    fn tenant_did_not_provide_written_mailing_address_tolls_deadline() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection32_31_3_12;
        input.tenant_mailing_address_status =
            TenantMailingAddressStatus::WrittenMailingAddressNotProvidedByTenant;
        input.days_to_return_deposit = 100;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::NotLiableTenantDidNotProvideWrittenMailingAddress
        );
    }

    #[test]
    fn tenant_provided_written_mailing_address_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TenantMailingAddressRequirementUnderSection32_31_3_9;
        input.tenant_mailing_address_status =
            TenantMailingAddressStatus::WrittenMailingAddressProvidedByTenant;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::CompliantTenantProvidedWrittenMailingAddress
        );
    }

    #[test]
    fn permissible_deductions_only_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PermissibleDeductionsUnderSection32_31_3_13;
        input.deposit_withheld_for_ordinary_wear_and_tear = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::CompliantDepositRetentionForPermissibleDeductionsOnly
        );
    }

    #[test]
    fn deposit_retained_for_ordinary_wear_and_tear_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PermissibleDeductionsUnderSection32_31_3_13;
        input.deposit_withheld_for_ordinary_wear_and_tear = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::ViolationDepositRetainedForOrdinaryWearAndTear
        );
    }

    #[test]
    fn landlord_maintains_safe_clean_habitable_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordObligationToMaintainHabitabilityUnderSection32_31_8_5;
        input.landlord_maintains_safe_clean_habitable_condition = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::CompliantLandlordMaintainsSafeCleanHabitableCondition
        );
    }

    #[test]
    fn landlord_failed_to_maintain_habitability_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordObligationToMaintainHabitabilityUnderSection32_31_8_5;
        input.landlord_maintains_safe_clean_habitable_condition = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::ViolationLandlordFailedToMaintainSafeCleanHabitableCondition
        );
    }

    #[test]
    fn landlord_provides_hot_and_cold_running_water_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordPlumbingAndHotWaterObligationUnderSection32_31_8_5;
        input.landlord_provides_hot_and_cold_running_water_at_all_times = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::CompliantLandlordProvidesHotAndColdRunningWaterAtAllTimes
        );
    }

    #[test]
    fn landlord_failed_to_provide_hot_water_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordPlumbingAndHotWaterObligationUnderSection32_31_8_5;
        input.landlord_provides_hot_and_cold_running_water_at_all_times = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::ViolationLandlordFailedToProvideHotAndColdRunningWater
        );
    }

    #[test]
    fn tenant_gave_notice_and_landlord_had_reasonable_time_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TenantEnforcementNoticePrerequisiteUnderSection32_31_8_6;
        input.notice_and_reasonable_time_status =
            NoticeAndReasonableTimeStatus::TenantGaveNoticeAndLandlordHadReasonableTime;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::CompliantTenantGaveNoticeAndLandlordHadReasonableTimeToRepair
        );
    }

    #[test]
    fn tenant_enforcement_without_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TenantEnforcementNoticePrerequisiteUnderSection32_31_8_6;
        input.notice_and_reasonable_time_status =
            NoticeAndReasonableTimeStatus::TenantDidNotGiveNotice;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::ViolationTenantEnforcementWithoutNoticeOrReasonableTime
        );
    }

    #[test]
    fn tenant_enforcement_without_reasonable_time_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TenantEnforcementNoticePrerequisiteUnderSection32_31_8_6;
        input.notice_and_reasonable_time_status =
            NoticeAndReasonableTimeStatus::TenantGaveNoticeButLandlordNotGivenReasonableTime;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::ViolationTenantEnforcementWithoutNoticeOrReasonableTime
        );
    }

    #[test]
    fn tenant_obligations_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenantObligationsUnderSection32_31_7_5;
        input.tenant_meets_cleanliness_and_smoke_detector_obligations = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::CompliantTenantMeetsCleanlinessAndSmokeDetectorObligations
        );
    }

    #[test]
    fn tenant_breaches_cleanliness_or_smoke_detector_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenantObligationsUnderSection32_31_7_5;
        input.tenant_meets_cleanliness_and_smoke_detector_obligations = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::ViolationTenantBreachesCleanlinessOrSmokeDetectorObligation
        );
    }

    #[test]
    fn ten_day_pay_or_quit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TenDayPayOrQuitNonpaymentNoticeUnderSection32_31_1_6;
        input.pay_or_quit_notice_days_given = 10;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::CompliantTenDayPayOrQuitNoticeProperlyServed
        );
    }

    #[test]
    fn nine_day_pay_or_quit_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TenDayPayOrQuitNonpaymentNoticeUnderSection32_31_1_6;
        input.pay_or_quit_notice_days_given = 9;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::ViolationTenDayPayOrQuitNoticeShorterThanTenDays
        );
    }

    #[test]
    fn methamphetamine_disclosure_provided_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MethamphetamineContaminationDisclosureUnderSection32_31_2_9;
        input.methamphetamine_disclosure_provided = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::CompliantMethamphetamineContaminationDisclosureProvided
        );
    }

    #[test]
    fn methamphetamine_disclosure_omitted_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MethamphetamineContaminationDisclosureUnderSection32_31_2_9;
        input.methamphetamine_disclosure_provided = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::ViolationMethamphetamineContaminationDisclosureOmitted
        );
    }

    #[test]
    fn no_statutory_deposit_cap_acknowledged_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NoStatutoryDepositCapAcknowledgement;
        let out = check(&input);
        assert_eq!(
            out.mode,
            InLandlordTenantMode::CompliantNoStatutoryDepositCapAcknowledged
        );
    }

    #[test]
    fn constants_pin_indiana_landlord_tenant_statutory_thresholds() {
        assert_eq!(IN_LANDLORD_TENANT_ARTICLE_NUMBER, 31);
        assert_eq!(IN_LANDLORD_TENANT_TITLE_NUMBER, 32);
        assert_eq!(IN_LANDLORD_TENANT_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 45);
        assert_eq!(IN_LANDLORD_TENANT_PAY_OR_QUIT_DEMAND_DAYS, 10);
        assert!(!IN_LANDLORD_TENANT_HAS_STATUTORY_DEPOSIT_CAP);
        assert_eq!(IN_LANDLORD_TENANT_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_indiana_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Indiana Code Title 32, Article 31"));
        assert!(joined.contains("IC § 32-31-3-12"));
        assert!(joined.contains("FORTY-FIVE (45) DAYS"));
        assert!(joined.contains("IC § 32-31-3-13"));
        assert!(joined.contains("NOT THE RESULT OF ORDINARY WEAR AND TEAR"));
        assert!(joined.contains("IC § 32-31-3-9"));
        assert!(joined.contains("written mailing address"));
        assert!(joined.contains("NO STATUTORY CAP"));
        assert!(joined.contains("REASONABLE ATTORNEY'S FEES"));
        assert!(joined.contains("IC § 32-31-8-5"));
        assert!(joined.contains("SAFE, CLEAN, AND HABITABLE CONDITION"));
        assert!(joined.contains("HEALTH AND HOUSING CODES"));
        assert!(joined.contains("HOT AND COLD RUNNING WATER AT ALL TIMES"));
        assert!(joined.contains("IC § 32-31-8-6"));
        assert!(joined.contains("REASONABLE AMOUNT OF TIME"));
        assert!(joined.contains("IC § 32-31-7-5"));
        assert!(joined.contains("SMOKE DETECTOR"));
        assert!(joined.contains("REPLACE BATTERIES"));
        assert!(joined.contains("IC § 32-31-1-6"));
        assert!(joined.contains("10-DAY PAY OR QUIT NOTICE"));
        assert!(joined.contains("IC § 32-31-2.9"));
        assert!(joined.contains("methamphetamine"));
        assert!(joined.contains("did NOT adopt"));
    }
}
