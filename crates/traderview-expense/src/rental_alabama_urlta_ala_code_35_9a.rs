//! Alabama Uniform Residential Landlord and
//! Tenant Act (AURLTA) — Ala. Code §§ 35-9A-101
//! through 35-9A-603 Compliance Module.
//!
//! Pure-compute check for landlord statutory compliance
//! with Alabama's URLTA-based regime (adopted 2006).
//!
//! **Most distinctive Alabama feature**: **60-DAY DEPOSIT
//! RETURN** under § 35-9A-201 — **LONGEST URLTA WINDOW
//! AMONG THE UNITED STATES** (cf. most URLTA states 30
//! days: KY, OH, SC, TN, IA, NC); twice the standard
//! URLTA timeframe.
//!
//! **Other distinctive Alabama features**: **DOUBLE
//! DEPOSIT PENALTY** for failure to return within 60-day
//! window under § 35-9A-201 (UNUSUAL — most URLTA states
//! use 2x deposit, 2x monthly rent, or 3x penalties; AL
//! uses 2x ORIGINAL DEPOSIT with no bad-faith requirement);
//! **7 BUSINESS DAY NONPAYMENT NOTICE** under § 35-9A-421
//! (UNUSUAL — most URLTA states use calendar days; AL
//! uses business days adding 2 weekend days effectively
//! lengthening to ~9 calendar days minimum); **2-DAY
//! (48-HOUR) ENTRY NOTICE** under § 35-9A-303 (matches
//! Kentucky URLTA 2-day; most states use 24 hours);
//! **14-DAY MATERIAL NONCOMPLIANCE NOTICE** under
//! § 35-9A-401; **HABITABILITY** under § 35-9A-204.
//!
//! Web research (verified 2026-06-04):
//! - **Alabama Uniform Residential Landlord and Tenant Act (AURLTA)** — adopted 2006; codified at Ala. Code §§ 35-9A-101 through 35-9A-603 ([FindLaw — Alabama Code Title 35 § 35-9A-201](https://codes.findlaw.com/al/title-35-property/al-code-sect-35-9a-201/); [LeaseLenses — Alabama Landlord-Tenant Law Guide 2026](https://www.leaselenses.com/blog/alabama-landlord-tenant-law-guide/); [iPropertyManagement — Alabama Security Deposit Law](https://ipropertymanagement.com/laws/alabama-security-deposit-returns); [FlagMyLease — Alabama Tenant Rights: The Bare Minimum](https://www.flagmylease.com/tenant-rights/alabama); [DoorLoop — Alabama Security Deposit Laws](https://www.doorloop.com/laws/alabama-security-deposit-laws); [LegalClarity — Alabama URLTA Explained](https://legalclarity.org/alabama-uniform-residential-landlord-and-tenant-act-explained/); [Nolo — Alabama Landlord-Tenant Laws: Complete Guide](https://www.nolo.com/legal-encyclopedia/overview-landlord-tenant-laws-alabama.html); [Tentunit — Alabama Landlord Responsibility Statement](https://help.tentunit.com/2025/11/04/alabama-landlord-responsibility-statement/); [DocDraft — Renting Out Your Property in Alabama](https://www.docdraft.ai/legal-guides/renting-out-my-property/alabama); [HomeRiver — Alabama Security Deposit Law](https://www.homeriver.com/blog/alabama-security-deposit-law)).
//! - **Ala. Code § 35-9A-201 Security Deposit Return — 60 Days**: landlords must return the security deposit within **60 DAYS** after the tenant vacates and provides a forwarding address; the landlord must include an **ITEMIZED STATEMENT** if any deductions are made.
//! - **Ala. Code § 35-9A-201 Double Deposit Penalty**: if the landlord fails to mail a timely refund or accounting within the 60-day period, the landlord shall pay the tenant **DOUBLE THE AMOUNT OF THE TENANT'S ORIGINAL DEPOSIT** — strict-liability automatic doubling without bad-faith requirement.
//! - **Ala. Code § 35-9A-421 Nonpayment Notice — 7 Business Days**: when an Alabama tenant fails to pay rent on time, the landlord must give the tenant a **7-BUSINESS-DAY NOTICE** to pay rent or quit (move out) before the landlord can file an eviction suit — UNUSUAL business-day count rather than calendar-day count.
//! - **Ala. Code § 35-9A-204 Habitability**: landlord shall (1) comply with **BUILDING AND HOUSING CODES** affecting health and safety; (2) maintain the premises in a **FIT AND HABITABLE CONDITION**; (3) keep **COMMON AREAS SAFE**.
//! - **Ala. Code § 35-9A-401 Material Noncompliance Notice — 14 Days**: landlord generally has **14 DAYS** to remedy a non-emergency condition after receiving written notice; symmetric tenant material noncompliance notice with 14-day cure right.
//! - **Ala. Code § 35-9A-303 Entry Notice — 2 Days (48 Hours)**: under the AURLTA, landlords must provide **TWO DAYS' (48 HOURS') NOTICE** before entering a rental unit, except in emergencies — UNUSUAL among US states (most use 24 hours; AL and KY use 48 hours).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const AL_CHAPTER_NUMBER: u32 = 9;
pub const AL_TITLE_NUMBER: u32 = 35;
pub const AL_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 60;
pub const AL_DOUBLE_DEPOSIT_PENALTY_MULTIPLIER: u32 = 2;
pub const AL_NONPAYMENT_NOTICE_BUSINESS_DAYS: u32 = 7;
pub const AL_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS: u32 = 14;
pub const AL_MATERIAL_NONCOMPLIANCE_CURE_DAYS: u32 = 14;
pub const AL_ENTRY_NOTICE_DAYS: u32 = 2;
pub const AL_ENTRY_NOTICE_HOURS: u32 = 48;
pub const AL_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromChapter35_9A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositReturnAndItemizedStatementStatus {
    BothDepositReturnedAndItemizedStatementProvidedWithin60Days,
    EitherNotReturnedOrItemizedStatementNotProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialNoncomplianceStatus {
    TenantCorrectedNoncomplianceWithin14Days,
    TenantDidNotCorrectNoncomplianceWithin14Days,
    NoMaterialNoncompliance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositReturnUnderSection359A201,
    DoubleDepositPenaltyUnderSection359A201,
    LandlordHabitabilityObligationUnderSection359A204,
    NonpaymentNoticeUnderSection359A421,
    MaterialNoncomplianceNoticeUnderSection359A401,
    LandlordEntryNoticeUnderSection359A303,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AlLandlordTenantMode {
    NotApplicableTenancyExemptFromChapter359A,
    CompliantBothDepositReturnedAndItemizedStatementProvidedWithin60Days,
    CompliantNoDoubleDepositPenaltyDepositTimelyReturned,
    CompliantLandlordMaintainsHabitabilityUnderSection359A204,
    CompliantSevenBusinessDayNonpaymentNoticeProperlyServed,
    CompliantMaterialNoncompliance14DayCureWithTenantCorrection,
    CompliantMaterialNoncompliance14DayNoticeWithoutCure,
    CompliantNoMaterialNoncompliance,
    CompliantTwoDayEntryNoticeProperlyServed,
    CompliantEmergencyEntryWithoutNotice,
    ViolationDepositReturnedPast60DayDeadlineDoubleDepositPenaltyTriggered,
    ViolationLandlordFailedHabitabilityObligation,
    ViolationNonpaymentNoticeShorterThan7BusinessDays,
    ViolationMaterialNoncomplianceNoticeShorterThan14Days,
    ViolationEntryNoticeShorterThan2Days,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub deposit_return_and_itemized_statement_status: DepositReturnAndItemizedStatementStatus,
    pub material_noncompliance_status: MaterialNoncomplianceStatus,
    pub compliance_aspect: ComplianceAspect,
    pub days_to_return_deposit_and_itemized_statement: u32,
    pub landlord_maintains_habitability: bool,
    pub nonpayment_notice_business_days_given: u32,
    pub material_noncompliance_notice_days_given: u32,
    pub entry_notice_days_given: u32,
    pub entry_was_emergency: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: AlLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type AlLandlordTenantInput = Input;
pub type AlLandlordTenantOutput = Output;
pub type AlLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Alabama Uniform Residential Landlord and Tenant Act (AURLTA) — adopted 2006; codified at Ala. Code §§ 35-9A-101 through 35-9A-603; classic URLTA adoption".to_string(),
        "Ala. Code § 35-9A-201 Security Deposit Return — 60 Days — landlords must return the security deposit within 60 DAYS after the tenant vacates and provides a forwarding address; the landlord must include an ITEMIZED STATEMENT if any deductions are made — LONGEST URLTA WINDOW AMONG THE UNITED STATES".to_string(),
        "Ala. Code § 35-9A-201 Double Deposit Penalty — if the landlord fails to mail a timely refund or accounting within the 60-day period, the landlord shall pay the tenant DOUBLE THE AMOUNT OF THE TENANT'S ORIGINAL DEPOSIT — strict-liability automatic doubling without bad-faith requirement".to_string(),
        "Ala. Code § 35-9A-421 Nonpayment Notice — 7 Business Days — when an Alabama tenant fails to pay rent on time, the landlord must give the tenant a 7-BUSINESS-DAY NOTICE to pay rent or quit (move out) before the landlord can file an eviction suit — UNUSUAL business-day count rather than calendar-day count".to_string(),
        "Ala. Code § 35-9A-204 Habitability — landlord shall (1) comply with BUILDING AND HOUSING CODES affecting health and safety; (2) maintain the premises in a FIT AND HABITABLE CONDITION; (3) keep COMMON AREAS SAFE".to_string(),
        "Ala. Code § 35-9A-401 Material Noncompliance Notice — 14 Days — landlord generally has 14 DAYS to remedy a non-emergency condition after receiving written notice; symmetric tenant material noncompliance notice with 14-day cure right".to_string(),
        "Ala. Code § 35-9A-303 Entry Notice — 2 Days (48 Hours) — under the AURLTA, landlords must provide TWO DAYS' (48 HOURS') NOTICE before entering a rental unit, except in emergencies — UNUSUAL among US states (most use 24 hours; AL and KY use 48 hours)".to_string(),
        "FindLaw + LeaseLenses + iPropertyManagement + FlagMyLease + DoorLoop + LegalClarity + Nolo + Tentunit + DocDraft + HomeRiver — practitioner overviews of Alabama URLTA".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromChapter35_9A {
        return Output {
            mode: AlLandlordTenantMode::NotApplicableTenancyExemptFromChapter359A,
            statutory_basis: "Ala. Code Chapter 35-9A jurisdiction — tenancy exempt from AURLTA coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from Alabama URLTA; statutory landlord-tenant obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositReturnUnderSection359A201 => {
            if input.days_to_return_deposit_and_itemized_statement
                <= AL_DEPOSIT_RETURN_DEADLINE_DAYS
                && input.deposit_return_and_itemized_statement_status
                    == DepositReturnAndItemizedStatementStatus::BothDepositReturnedAndItemizedStatementProvidedWithin60Days
            {
                Output {
                    mode: AlLandlordTenantMode::CompliantBothDepositReturnedAndItemizedStatementProvidedWithin60Days,
                    statutory_basis: "Ala. Code § 35-9A-201 — deposit returned with itemized statement within 60 days".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit returned with itemized statement at day {d} (within 60-day statutory window) under Ala. Code § 35-9A-201.",
                        d = input.days_to_return_deposit_and_itemized_statement,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: AlLandlordTenantMode::ViolationDepositReturnedPast60DayDeadlineDoubleDepositPenaltyTriggered,
                    statutory_basis: "Ala. Code § 35-9A-201 — deposit return exceeded 60-day window; double deposit penalty triggered".to_string(),
                    notes: format!(
                        "VIOLATION: deposit returned at day {d} past 60-day statutory window under Ala. Code § 35-9A-201; landlord shall pay tenant DOUBLE THE AMOUNT OF ORIGINAL DEPOSIT — strict-liability automatic doubling.",
                        d = input.days_to_return_deposit_and_itemized_statement,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::DoubleDepositPenaltyUnderSection359A201 => {
            if input.days_to_return_deposit_and_itemized_statement
                <= AL_DEPOSIT_RETURN_DEADLINE_DAYS
                && input.deposit_return_and_itemized_statement_status
                    == DepositReturnAndItemizedStatementStatus::BothDepositReturnedAndItemizedStatementProvidedWithin60Days
            {
                Output {
                    mode: AlLandlordTenantMode::CompliantNoDoubleDepositPenaltyDepositTimelyReturned,
                    statutory_basis: "Ala. Code § 35-9A-201 — no double deposit penalty; deposit timely returned".to_string(),
                    notes: "COMPLIANT: deposit timely returned with itemized statement under Ala. Code § 35-9A-201; double deposit penalty exposure not triggered.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: AlLandlordTenantMode::ViolationDepositReturnedPast60DayDeadlineDoubleDepositPenaltyTriggered,
                    statutory_basis: "Ala. Code § 35-9A-201 — double deposit penalty triggered".to_string(),
                    notes: "VIOLATION: double deposit penalty triggered under Ala. Code § 35-9A-201; landlord shall pay tenant DOUBLE THE AMOUNT OF ORIGINAL DEPOSIT.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordHabitabilityObligationUnderSection359A204 => {
            if input.landlord_maintains_habitability {
                Output {
                    mode: AlLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderSection359A204,
                    statutory_basis: "Ala. Code § 35-9A-204 — landlord maintains habitability".to_string(),
                    notes: "COMPLIANT: landlord maintains all three habitability obligations under Ala. Code § 35-9A-204 (building/housing codes + fit and habitable condition + common areas safe).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: AlLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation,
                    statutory_basis: "Ala. Code § 35-9A-204 — landlord failed habitability obligation".to_string(),
                    notes: "VIOLATION: landlord failed one or more habitability obligations under Ala. Code § 35-9A-204.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::NonpaymentNoticeUnderSection359A421 => {
            if input.nonpayment_notice_business_days_given >= AL_NONPAYMENT_NOTICE_BUSINESS_DAYS {
                Output {
                    mode: AlLandlordTenantMode::CompliantSevenBusinessDayNonpaymentNoticeProperlyServed,
                    statutory_basis: "Ala. Code § 35-9A-421 — 7-business-day nonpayment notice properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-business-day nonpayment notice satisfies 7-business-day statutory minimum under Ala. Code § 35-9A-421.",
                        d = input.nonpayment_notice_business_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: AlLandlordTenantMode::ViolationNonpaymentNoticeShorterThan7BusinessDays,
                    statutory_basis: "Ala. Code § 35-9A-421 — nonpayment notice shorter than 7-business-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-business-day nonpayment notice shorter than 7-business-day statutory minimum under Ala. Code § 35-9A-421.",
                        d = input.nonpayment_notice_business_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::MaterialNoncomplianceNoticeUnderSection359A401 => {
            if input.material_noncompliance_notice_days_given < AL_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS
            {
                return Output {
                    mode: AlLandlordTenantMode::ViolationMaterialNoncomplianceNoticeShorterThan14Days,
                    statutory_basis: "Ala. Code § 35-9A-401 — material noncompliance notice shorter than 14-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day material noncompliance notice shorter than 14-day statutory minimum under Ala. Code § 35-9A-401.",
                        d = input.material_noncompliance_notice_days_given,
                    ),
                    citations,
                };
            }
            match input.material_noncompliance_status {
                MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin14Days => Output {
                    mode: AlLandlordTenantMode::CompliantMaterialNoncompliance14DayCureWithTenantCorrection,
                    statutory_basis: "Ala. Code § 35-9A-401 — tenant cured material noncompliance within 14 days".to_string(),
                    notes: "COMPLIANT: tenant corrected material noncompliance within 14-day cure window under Ala. Code § 35-9A-401; tenancy continues.".to_string(),
                    citations,
                },
                MaterialNoncomplianceStatus::TenantDidNotCorrectNoncomplianceWithin14Days => Output {
                    mode: AlLandlordTenantMode::CompliantMaterialNoncompliance14DayNoticeWithoutCure,
                    statutory_basis: "Ala. Code § 35-9A-401 — 14-day notice served + tenant did not cure".to_string(),
                    notes: "COMPLIANT: 14-day material noncompliance notice properly served under Ala. Code § 35-9A-401; tenant did not correct within cure window; landlord may proceed with eviction.".to_string(),
                    citations,
                },
                MaterialNoncomplianceStatus::NoMaterialNoncompliance => Output {
                    mode: AlLandlordTenantMode::CompliantNoMaterialNoncompliance,
                    statutory_basis: "Ala. Code § 35-9A-401 — no material noncompliance".to_string(),
                    notes: "COMPLIANT: no material noncompliance condition present under Ala. Code § 35-9A-401; eviction notice not triggered.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::LandlordEntryNoticeUnderSection359A303 => {
            if input.entry_was_emergency {
                return Output {
                    mode: AlLandlordTenantMode::CompliantEmergencyEntryWithoutNotice,
                    statutory_basis: "Ala. Code § 35-9A-303 — emergency entry without 2-day notice".to_string(),
                    notes: "COMPLIANT: emergency entry under Ala. Code § 35-9A-303; 2-day notice requirement excused.".to_string(),
                    citations,
                };
            }
            if input.entry_notice_days_given >= AL_ENTRY_NOTICE_DAYS {
                Output {
                    mode: AlLandlordTenantMode::CompliantTwoDayEntryNoticeProperlyServed,
                    statutory_basis: "Ala. Code § 35-9A-303 — 2-day entry notice properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day entry notice satisfies 2-day (48-hour) statutory minimum under Ala. Code § 35-9A-303.",
                        d = input.entry_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: AlLandlordTenantMode::ViolationEntryNoticeShorterThan2Days,
                    statutory_basis: "Ala. Code § 35-9A-303 — entry notice shorter than 2-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day entry notice shorter than 2-day (48-hour) statutory minimum under Ala. Code § 35-9A-303.",
                        d = input.entry_notice_days_given,
                    ),
                    citations,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            tenancy_coverage: TenancyCoverage::CoveredResidentialTenancy,
            deposit_return_and_itemized_statement_status:
                DepositReturnAndItemizedStatementStatus::BothDepositReturnedAndItemizedStatementProvidedWithin60Days,
            material_noncompliance_status:
                MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin14Days,
            compliance_aspect: ComplianceAspect::SecurityDepositReturnUnderSection359A201,
            days_to_return_deposit_and_itemized_statement: 50,
            landlord_maintains_habitability: true,
            nonpayment_notice_business_days_given: 7,
            material_noncompliance_notice_days_given: 14,
            entry_notice_days_given: 2,
            entry_was_emergency: false,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromChapter35_9A;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::NotApplicableTenancyExemptFromChapter359A
        );
    }

    #[test]
    fn deposit_returned_at_60_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection359A201;
        input.days_to_return_deposit_and_itemized_statement = 60;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::CompliantBothDepositReturnedAndItemizedStatementProvidedWithin60Days
        );
    }

    #[test]
    fn deposit_returned_at_61_days_double_penalty_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection359A201;
        input.days_to_return_deposit_and_itemized_statement = 61;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::ViolationDepositReturnedPast60DayDeadlineDoubleDepositPenaltyTriggered
        );
    }

    #[test]
    fn deposit_without_itemized_statement_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection359A201;
        input.deposit_return_and_itemized_statement_status =
            DepositReturnAndItemizedStatementStatus::EitherNotReturnedOrItemizedStatementNotProvided;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::ViolationDepositReturnedPast60DayDeadlineDoubleDepositPenaltyTriggered
        );
    }

    #[test]
    fn no_double_deposit_penalty_timely_return_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DoubleDepositPenaltyUnderSection359A201;
        input.days_to_return_deposit_and_itemized_statement = 50;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::CompliantNoDoubleDepositPenaltyDepositTimelyReturned
        );
    }

    #[test]
    fn double_deposit_penalty_triggered_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DoubleDepositPenaltyUnderSection359A201;
        input.days_to_return_deposit_and_itemized_statement = 75;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::ViolationDepositReturnedPast60DayDeadlineDoubleDepositPenaltyTriggered
        );
    }

    #[test]
    fn landlord_maintains_habitability_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordHabitabilityObligationUnderSection359A204;
        input.landlord_maintains_habitability = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderSection359A204
        );
    }

    #[test]
    fn landlord_failed_habitability_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordHabitabilityObligationUnderSection359A204;
        input.landlord_maintains_habitability = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation
        );
    }

    #[test]
    fn seven_business_day_nonpayment_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderSection359A421;
        input.nonpayment_notice_business_days_given = 7;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::CompliantSevenBusinessDayNonpaymentNoticeProperlyServed
        );
    }

    #[test]
    fn six_business_day_nonpayment_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderSection359A421;
        input.nonpayment_notice_business_days_given = 6;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::ViolationNonpaymentNoticeShorterThan7BusinessDays
        );
    }

    #[test]
    fn material_noncompliance_14_day_cure_with_correction_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaterialNoncomplianceNoticeUnderSection359A401;
        input.material_noncompliance_notice_days_given = 14;
        input.material_noncompliance_status =
            MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin14Days;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::CompliantMaterialNoncompliance14DayCureWithTenantCorrection
        );
    }

    #[test]
    fn material_noncompliance_14_day_notice_without_cure_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaterialNoncomplianceNoticeUnderSection359A401;
        input.material_noncompliance_notice_days_given = 14;
        input.material_noncompliance_status =
            MaterialNoncomplianceStatus::TenantDidNotCorrectNoncomplianceWithin14Days;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::CompliantMaterialNoncompliance14DayNoticeWithoutCure
        );
    }

    #[test]
    fn material_noncompliance_13_day_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaterialNoncomplianceNoticeUnderSection359A401;
        input.material_noncompliance_notice_days_given = 13;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::ViolationMaterialNoncomplianceNoticeShorterThan14Days
        );
    }

    #[test]
    fn two_day_entry_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection359A303;
        input.entry_notice_days_given = 2;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::CompliantTwoDayEntryNoticeProperlyServed
        );
    }

    #[test]
    fn one_day_entry_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection359A303;
        input.entry_notice_days_given = 1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::ViolationEntryNoticeShorterThan2Days
        );
    }

    #[test]
    fn emergency_entry_without_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection359A303;
        input.entry_was_emergency = true;
        input.entry_notice_days_given = 0;
        let out = check(&input);
        assert_eq!(
            out.mode,
            AlLandlordTenantMode::CompliantEmergencyEntryWithoutNotice
        );
    }

    #[test]
    fn constants_pin_alabama_landlord_tenant_statutory_thresholds() {
        assert_eq!(AL_CHAPTER_NUMBER, 9);
        assert_eq!(AL_TITLE_NUMBER, 35);
        assert_eq!(AL_DEPOSIT_RETURN_DEADLINE_DAYS, 60);
        assert_eq!(AL_DOUBLE_DEPOSIT_PENALTY_MULTIPLIER, 2);
        assert_eq!(AL_NONPAYMENT_NOTICE_BUSINESS_DAYS, 7);
        assert_eq!(AL_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS, 14);
        assert_eq!(AL_MATERIAL_NONCOMPLIANCE_CURE_DAYS, 14);
        assert_eq!(AL_ENTRY_NOTICE_DAYS, 2);
        assert_eq!(AL_ENTRY_NOTICE_HOURS, 48);
        assert_eq!(AL_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_alabama_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Alabama Uniform Residential Landlord and Tenant Act"));
        assert!(joined.contains("AURLTA"));
        assert!(joined.contains("Ala. Code §§ 35-9A-101 through 35-9A-603"));
        assert!(joined.contains("Ala. Code § 35-9A-201"));
        assert!(joined.contains("60 DAYS"));
        assert!(joined.contains("ITEMIZED STATEMENT"));
        assert!(joined.contains("LONGEST URLTA WINDOW"));
        assert!(joined.contains("DOUBLE THE AMOUNT OF THE TENANT'S ORIGINAL DEPOSIT"));
        assert!(joined.contains("Ala. Code § 35-9A-421"));
        assert!(joined.contains("7-BUSINESS-DAY NOTICE"));
        assert!(joined.contains("Ala. Code § 35-9A-204"));
        assert!(joined.contains("BUILDING AND HOUSING CODES"));
        assert!(joined.contains("FIT AND HABITABLE CONDITION"));
        assert!(joined.contains("COMMON AREAS SAFE"));
        assert!(joined.contains("Ala. Code § 35-9A-401"));
        assert!(joined.contains("14 DAYS"));
        assert!(joined.contains("Ala. Code § 35-9A-303"));
        assert!(joined.contains("TWO DAYS' (48 HOURS') NOTICE"));
        assert!(joined.contains("UNUSUAL among US states"));
    }
}
