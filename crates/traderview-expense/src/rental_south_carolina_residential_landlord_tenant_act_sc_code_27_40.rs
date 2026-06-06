//! South Carolina Residential Landlord and Tenant Act
//! (SCRLTA) — SC Code §§ 27-40-10 through 27-40-940
//! Compliance Module — pure-compute check for landlord
//! statutory compliance with South Carolina's URLTA-based
//! landlord-tenant regime.
//!
//! **Distinctive South Carolina features**: **3x WRONGFUL
//! WITHHOLDING PENALTY** plus reasonable attorney's fees
//! under § 27-40-410 (one of the strongest deposit-return
//! tenant remedies in the United States); **5-day
//! nonpayment notice** under § 27-40-710 (shorter than
//! most URLTA states); **NO SEPARATE NOTICE REQUIRED** if
//! lease contains 5-day notice provision (saves landlord
//! procedural step).
//!
//! Web research (verified 2026-06-03):
//! - **South Carolina Residential Landlord and Tenant Act (SCRLTA)**: codified at SC Code §§ 27-40-10 through 27-40-940; classic URLTA-based regime ([South Carolina Legislature — Title 27 Chapter 40](https://www.scstatehouse.gov/code/t27c040.php); [Justia — 2025 South Carolina Code Title 27 Chapter 40 § 27-40-410](https://law.justia.com/codes/south-carolina/title-27/chapter-40/section-27-40-410/); [Justia — 2025 South Carolina Code Title 27 Chapter 40](https://law.justia.com/codes/south-carolina/title-27/chapter-40/); [Justia — 2025 § 27-40-710 Noncompliance with Rental Agreement; Failure to Pay Rent](https://law.justia.com/codes/south-carolina/title-27/chapter-40/section-27-40-710/); [LeaseLenses — South Carolina Landlord-Tenant Law Guide 2026](https://www.leaselenses.com/blog/south-carolina-landlord-tenant-law-guide/); [DoorLoop — South Carolina Security Deposit Laws](https://www.doorloop.com/laws/south-carolina-security-deposit-laws); [Hemlane — South Carolina Security Deposit Laws in 2026](https://www.hemlane.com/resources/south-carolina-security-deposit-laws/); [Hemlane — South Carolina Tenant-Landlord Rental Laws & Rights for 2026](https://www.hemlane.com/resources/south-carolina-tenant-landlord-law/); [TurboTenant — South Carolina Eviction Laws & Step-by-Step Process](https://www.turbotenant.com/rental-lease-agreement/south-carolina/laws/eviction/); [SC Justice — SC Landlord-Tenant Law 2012 PPT](https://www.scjustice.org/wp-content/uploads/2012/11/sc-landlord-tenant-law-2012.ppt); [Landlord-Tenant-Law — South Carolina Landlord Tenant Law in Plain English](https://www.landlord-tenant-law.com/south-carolina-landlord-tenant-law.html); [iPropertyManagement — South Carolina Warranty of Habitability 2026](https://ipropertymanagement.com/laws/warranty-of-habitability-south-carolina)).
//! - **SC Code § 27-40-410 Security Deposit Return — 30 Days**: any deduction from the security/rental deposit must be **ITEMIZED BY THE LANDLORD IN A WRITTEN NOTICE TO THE TENANT** together with the amount due, if any, within **30 DAYS** after termination of the tenancy and delivery of possession and demand by the tenant, whichever is later.
//! - **SC Code § 27-40-410 3x Wrongful Withholding Penalty**: if the landlord fails to return to the tenant any prepaid rent or security/rental deposit with the notice required under subsection (a), the tenant may recover **THREE TIMES THE AMOUNT WRONGFULLY WITHHELD** and **REASONABLE ATTORNEY'S FEES** — one of the strongest deposit-return tenant remedies in the United States.
//! - **SC Code § 27-40-440 Landlord Obligation to Maintain — Habitability**: a landlord shall **(1)** comply with the requirements of applicable **BUILDING AND HOUSING CODES** materially affecting health and safety; **(2)** make all repairs and do whatever is reasonably necessary to put and keep the premises in a **FIT AND HABITABLE CONDITION**; **(3)** keep all **COMMON AREAS** of the premises in a reasonably safe condition; **(4)** maintain adequate utilities including **RUNNING WATER** and reasonable amounts of **HOT WATER AT ALL TIMES** and reasonable **HEAT**.
//! - **SC Code § 27-40-710 Nonpayment Notice — 5 Days**: if a tenant does not pay rent within **5 DAYS OF THE DUE DATE**, the landlord can start eviction proceedings; if the rental agreement contains a specific notice provision regarding the 5-day payment deadline, the landlord is **NOT REQUIRED TO FURNISH ANY SEPARATE OR ADDITIONAL WRITTEN NOTICE** to the tenant in order to commence eviction proceedings for nonpayment of rent.
//! - **SC Code § 27-40-710 Material Noncompliance — 14 Days with Cure Right**: if a tenant fails to perform duties in a significant manner, the landlord can send a **14-DAY WRITTEN NOTICE** to terminate the tenancy; if the tenant **CORRECTS THE PROBLEM WITHIN 14 DAYS**, the tenancy shall continue.
//! - **SC Code § 27-40-540 Landlord Right of Access — 24-Hour Notice**: landlord shall give tenant **AT LEAST 24 HOURS NOTICE** of intent to enter (standard URLTA notice provision); exceptions for emergencies and routine maintenance with prior consent.
//! - **SC Code § 27-40-770 Periodic Tenancy Termination — 30 Days**: month-to-month tenancies may be terminated by either party with **30 DAYS WRITTEN NOTICE**; week-to-week tenancies with 7 days written notice.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SC_CHAPTER_NUMBER: u32 = 40;
pub const SC_TITLE_NUMBER: u32 = 27;
pub const SC_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const SC_WRONGFUL_WITHHOLDING_MULTIPLIER: u32 = 3;
pub const SC_NONPAYMENT_NOTICE_DAYS: u32 = 5;
pub const SC_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS: u32 = 14;
pub const SC_MATERIAL_NONCOMPLIANCE_CURE_DAYS: u32 = 14;
pub const SC_ENTRY_NOTICE_HOURS: u32 = 24;
pub const SC_PERIODIC_TENANCY_NOTICE_DAYS_MONTHLY: u32 = 30;
pub const SC_PERIODIC_TENANCY_NOTICE_DAYS_WEEKLY: u32 = 7;
pub const SC_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromChapter27_40,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseFiveDayNoticeClauseStatus {
    LeaseContainsFiveDayNoticeProvision,
    LeaseDoesNotContainFiveDayNoticeProvision,
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
pub enum PeriodicTenancyType {
    MonthToMonth,
    WeekToWeek,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositReturnUnderSection27_40_410,
    WrongfulWithholding3xPenaltyUnderSection27_40_410,
    LandlordHabitabilityObligationUnderSection27_40_440,
    NonpaymentNoticeUnderSection27_40_710,
    MaterialNoncomplianceNoticeUnderSection27_40_710,
    LandlordEntryNoticeUnderSection27_40_540,
    PeriodicTenancyTerminationUnderSection27_40_770,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScLandlordTenantMode {
    NotApplicableTenancyExemptFromChapter27_40,
    CompliantDepositReturnedWithItemizedNoticeWithin30Days,
    CompliantNoWrongfulWithholding,
    CompliantLandlordMaintainsHabitabilityUnderSection27_40_440,
    CompliantFiveDayNonpaymentNoticeProperlyServed,
    CompliantNoSeparateNoticeRequiredLeaseContainsFiveDayProvision,
    CompliantMaterialNoncompliance14DayCureWithTenantCorrection,
    CompliantMaterialNoncompliance14DayNoticeWithoutCure,
    CompliantNoMaterialNoncompliance,
    Compliant24HourEntryNoticeProperlyServed,
    CompliantThirtyDayMonthToMonthNotice,
    CompliantSevenDayWeekToWeekNotice,
    ViolationDepositReturnedPast30DayDeadline,
    ViolationWrongfulWithholding3xPenaltyTriggered,
    ViolationLandlordFailedHabitabilityObligation,
    ViolationNonpaymentNoticeShorterThan5Days,
    ViolationMaterialNoncomplianceNoticeShorterThan14Days,
    ViolationEntryNoticeShorterThan24Hours,
    ViolationPeriodicTenancyNoticeShorterThanRequired,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub lease_five_day_notice_clause_status: LeaseFiveDayNoticeClauseStatus,
    pub material_noncompliance_status: MaterialNoncomplianceStatus,
    pub periodic_tenancy_type: PeriodicTenancyType,
    pub compliance_aspect: ComplianceAspect,
    pub days_to_return_deposit: u32,
    pub deposit_wrongfully_withheld: bool,
    pub landlord_maintains_habitability: bool,
    pub nonpayment_notice_days_given: u32,
    pub material_noncompliance_notice_days_given: u32,
    pub entry_notice_hours_given: u32,
    pub periodic_tenancy_notice_days_given: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: ScLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type ScLandlordTenantInput = Input;
pub type ScLandlordTenantOutput = Output;
pub type ScLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "South Carolina Residential Landlord and Tenant Act (SCRLTA) — codified at SC Code §§ 27-40-10 through 27-40-940; classic URLTA-based regime".to_string(),
        "SC Code § 27-40-410 Security Deposit Return — 30 Days — any deduction from the security/rental deposit must be ITEMIZED BY THE LANDLORD IN A WRITTEN NOTICE TO THE TENANT together with the amount due, if any, within 30 DAYS after termination of the tenancy and delivery of possession and demand by the tenant, whichever is later".to_string(),
        "SC Code § 27-40-410 3x Wrongful Withholding Penalty — if the landlord fails to return to the tenant any prepaid rent or security/rental deposit with the notice required under subsection (a), the tenant may recover THREE TIMES THE AMOUNT WRONGFULLY WITHHELD and REASONABLE ATTORNEY'S FEES".to_string(),
        "SC Code § 27-40-440 Landlord Obligation to Maintain — Habitability — a landlord shall (1) comply with the requirements of applicable BUILDING AND HOUSING CODES materially affecting health and safety; (2) make all repairs and do whatever is reasonably necessary to put and keep the premises in a FIT AND HABITABLE CONDITION; (3) keep all COMMON AREAS of the premises in a reasonably safe condition; (4) maintain adequate utilities including RUNNING WATER and reasonable amounts of HOT WATER AT ALL TIMES and reasonable HEAT".to_string(),
        "SC Code § 27-40-710 Nonpayment Notice — 5 Days — if a tenant does not pay rent within 5 DAYS OF THE DUE DATE, the landlord can start eviction proceedings; if the rental agreement contains a specific notice provision regarding the 5-day payment deadline, the landlord is NOT REQUIRED TO FURNISH ANY SEPARATE OR ADDITIONAL WRITTEN NOTICE to the tenant in order to commence eviction proceedings for nonpayment of rent".to_string(),
        "SC Code § 27-40-710 Material Noncompliance — 14 Days with Cure Right — if a tenant fails to perform duties in a significant manner, the landlord can send a 14-DAY WRITTEN NOTICE to terminate the tenancy; if the tenant CORRECTS THE PROBLEM WITHIN 14 DAYS, the tenancy shall continue".to_string(),
        "SC Code § 27-40-540 Landlord Right of Access — 24-Hour Notice — landlord shall give tenant AT LEAST 24 HOURS NOTICE of intent to enter; exceptions for emergencies and routine maintenance with prior consent".to_string(),
        "SC Code § 27-40-770 Periodic Tenancy Termination — month-to-month tenancies may be terminated by either party with 30 DAYS WRITTEN NOTICE; week-to-week tenancies with 7 days written notice".to_string(),
        "South Carolina Legislature + Justia + LeaseLenses + DoorLoop + Hemlane + TurboTenant + SC Justice + Landlord-Tenant-Law + iPropertyManagement — practitioner overviews of South Carolina Residential Landlord and Tenant Act".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromChapter27_40 {
        return Output {
            mode: ScLandlordTenantMode::NotApplicableTenancyExemptFromChapter27_40,
            statutory_basis: "SC Code Chapter 27-40 jurisdiction — tenancy exempt from SCRLTA coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from South Carolina Residential Landlord and Tenant Act; SCRLTA landlord-tenant obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositReturnUnderSection27_40_410 => {
            if input.days_to_return_deposit <= SC_DEPOSIT_RETURN_DEADLINE_DAYS {
                Output {
                    mode: ScLandlordTenantMode::CompliantDepositReturnedWithItemizedNoticeWithin30Days,
                    statutory_basis: "SC Code § 27-40-410 — deposit returned with itemized written notice within 30 days".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit returned with itemized written notice at day {d} (within 30-day statutory window) under SC Code § 27-40-410.",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: ScLandlordTenantMode::ViolationDepositReturnedPast30DayDeadline,
                    statutory_basis: "SC Code § 27-40-410 — deposit return exceeded 30-day statutory window".to_string(),
                    notes: format!(
                        "VIOLATION: deposit returned at day {d} past 30-day statutory window under SC Code § 27-40-410.",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::WrongfulWithholding3xPenaltyUnderSection27_40_410 => {
            if input.deposit_wrongfully_withheld {
                Output {
                    mode: ScLandlordTenantMode::ViolationWrongfulWithholding3xPenaltyTriggered,
                    statutory_basis: "SC Code § 27-40-410 — wrongful withholding triggers 3x damages + reasonable attorney's fees".to_string(),
                    notes: "VIOLATION: landlord wrongfully withheld deposit; tenant may recover THREE TIMES THE AMOUNT WRONGFULLY WITHHELD plus REASONABLE ATTORNEY'S FEES under SC Code § 27-40-410.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: ScLandlordTenantMode::CompliantNoWrongfulWithholding,
                    statutory_basis: "SC Code § 27-40-410 — no wrongful withholding; 3x penalty not triggered".to_string(),
                    notes: "COMPLIANT: no wrongful withholding under SC Code § 27-40-410; 3x damages exposure not triggered.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordHabitabilityObligationUnderSection27_40_440 => {
            if input.landlord_maintains_habitability {
                Output {
                    mode: ScLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderSection27_40_440,
                    statutory_basis: "SC Code § 27-40-440 — landlord maintains habitability".to_string(),
                    notes: "COMPLIANT: landlord maintains all four habitability obligations under SC Code § 27-40-440 (building/housing codes + fit and habitable condition + common areas safe + utilities including running water, hot water at all times, and heat).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: ScLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation,
                    statutory_basis: "SC Code § 27-40-440 — landlord failed habitability obligation".to_string(),
                    notes: "VIOLATION: landlord failed one or more habitability obligations under SC Code § 27-40-440.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::NonpaymentNoticeUnderSection27_40_710 => {
            match input.lease_five_day_notice_clause_status {
                LeaseFiveDayNoticeClauseStatus::LeaseContainsFiveDayNoticeProvision => Output {
                    mode: ScLandlordTenantMode::CompliantNoSeparateNoticeRequiredLeaseContainsFiveDayProvision,
                    statutory_basis: "SC Code § 27-40-710 — lease contains 5-day notice provision; no separate notice required".to_string(),
                    notes: "COMPLIANT: lease contains specific 5-day notice provision under SC Code § 27-40-710; landlord NOT REQUIRED to furnish separate or additional written notice; may commence eviction immediately after 5-day default.".to_string(),
                    citations,
                },
                LeaseFiveDayNoticeClauseStatus::LeaseDoesNotContainFiveDayNoticeProvision => {
                    if input.nonpayment_notice_days_given >= SC_NONPAYMENT_NOTICE_DAYS {
                        Output {
                            mode: ScLandlordTenantMode::CompliantFiveDayNonpaymentNoticeProperlyServed,
                            statutory_basis: "SC Code § 27-40-710 — 5-day nonpayment notice properly served".to_string(),
                            notes: format!(
                                "COMPLIANT: {d}-day nonpayment notice satisfies 5-day statutory minimum under SC Code § 27-40-710.",
                                d = input.nonpayment_notice_days_given,
                            ),
                            citations,
                        }
                    } else {
                        Output {
                            mode: ScLandlordTenantMode::ViolationNonpaymentNoticeShorterThan5Days,
                            statutory_basis: "SC Code § 27-40-710 — nonpayment notice shorter than 5-day statutory minimum".to_string(),
                            notes: format!(
                                "VIOLATION: {d}-day nonpayment notice shorter than 5-day statutory minimum under SC Code § 27-40-710.",
                                d = input.nonpayment_notice_days_given,
                            ),
                            citations,
                        }
                    }
                }
            }
        }
        ComplianceAspect::MaterialNoncomplianceNoticeUnderSection27_40_710 => {
            if input.material_noncompliance_notice_days_given
                < SC_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS
            {
                return Output {
                    mode: ScLandlordTenantMode::ViolationMaterialNoncomplianceNoticeShorterThan14Days,
                    statutory_basis: "SC Code § 27-40-710 — material noncompliance notice shorter than 14-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day material noncompliance notice shorter than 14-day statutory minimum under SC Code § 27-40-710.",
                        d = input.material_noncompliance_notice_days_given,
                    ),
                    citations,
                };
            }
            match input.material_noncompliance_status {
                MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin14Days => Output {
                    mode: ScLandlordTenantMode::CompliantMaterialNoncompliance14DayCureWithTenantCorrection,
                    statutory_basis: "SC Code § 27-40-710 — tenant cured material noncompliance within 14 days; tenancy continues".to_string(),
                    notes: "COMPLIANT: tenant corrected material noncompliance within 14-day cure window under SC Code § 27-40-710; tenancy continues.".to_string(),
                    citations,
                },
                MaterialNoncomplianceStatus::TenantDidNotCorrectNoncomplianceWithin14Days => Output {
                    mode: ScLandlordTenantMode::CompliantMaterialNoncompliance14DayNoticeWithoutCure,
                    statutory_basis: "SC Code § 27-40-710 — 14-day notice served + tenant did not cure; tenancy may terminate".to_string(),
                    notes: "COMPLIANT: 14-day material noncompliance notice properly served under SC Code § 27-40-710; tenant did not correct within cure window; landlord may proceed with eviction.".to_string(),
                    citations,
                },
                MaterialNoncomplianceStatus::NoMaterialNoncompliance => Output {
                    mode: ScLandlordTenantMode::CompliantNoMaterialNoncompliance,
                    statutory_basis: "SC Code § 27-40-710 — no material noncompliance".to_string(),
                    notes: "COMPLIANT: no material noncompliance condition present under SC Code § 27-40-710; eviction notice not triggered.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::LandlordEntryNoticeUnderSection27_40_540 => {
            if input.entry_notice_hours_given >= SC_ENTRY_NOTICE_HOURS {
                Output {
                    mode: ScLandlordTenantMode::Compliant24HourEntryNoticeProperlyServed,
                    statutory_basis: "SC Code § 27-40-540 — 24-hour entry notice properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {h}-hour entry notice satisfies 24-hour statutory minimum under SC Code § 27-40-540.",
                        h = input.entry_notice_hours_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: ScLandlordTenantMode::ViolationEntryNoticeShorterThan24Hours,
                    statutory_basis: "SC Code § 27-40-540 — entry notice shorter than 24-hour statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {h}-hour entry notice shorter than 24-hour statutory minimum under SC Code § 27-40-540.",
                        h = input.entry_notice_hours_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::PeriodicTenancyTerminationUnderSection27_40_770 => {
            let required_days = match input.periodic_tenancy_type {
                PeriodicTenancyType::MonthToMonth => SC_PERIODIC_TENANCY_NOTICE_DAYS_MONTHLY,
                PeriodicTenancyType::WeekToWeek => SC_PERIODIC_TENANCY_NOTICE_DAYS_WEEKLY,
            };
            if input.periodic_tenancy_notice_days_given >= required_days {
                let mode = match input.periodic_tenancy_type {
                    PeriodicTenancyType::MonthToMonth => {
                        ScLandlordTenantMode::CompliantThirtyDayMonthToMonthNotice
                    }
                    PeriodicTenancyType::WeekToWeek => {
                        ScLandlordTenantMode::CompliantSevenDayWeekToWeekNotice
                    }
                };
                Output {
                    mode,
                    statutory_basis: format!(
                        "SC Code § 27-40-770 — {required_days}-day periodic tenancy notice properly served for {tt:?}",
                        tt = input.periodic_tenancy_type,
                    ),
                    notes: format!(
                        "COMPLIANT: {d}-day periodic tenancy notice satisfies {required_days}-day statutory minimum for {tt:?} under SC Code § 27-40-770.",
                        d = input.periodic_tenancy_notice_days_given,
                        tt = input.periodic_tenancy_type,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: ScLandlordTenantMode::ViolationPeriodicTenancyNoticeShorterThanRequired,
                    statutory_basis: format!(
                        "SC Code § 27-40-770 — periodic tenancy notice shorter than {required_days}-day statutory minimum",
                    ),
                    notes: format!(
                        "VIOLATION: {d}-day periodic tenancy notice shorter than {required_days}-day statutory minimum for {tt:?} under SC Code § 27-40-770.",
                        d = input.periodic_tenancy_notice_days_given,
                        tt = input.periodic_tenancy_type,
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
            lease_five_day_notice_clause_status:
                LeaseFiveDayNoticeClauseStatus::LeaseDoesNotContainFiveDayNoticeProvision,
            material_noncompliance_status:
                MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin14Days,
            periodic_tenancy_type: PeriodicTenancyType::MonthToMonth,
            compliance_aspect: ComplianceAspect::SecurityDepositReturnUnderSection27_40_410,
            days_to_return_deposit: 25,
            deposit_wrongfully_withheld: false,
            landlord_maintains_habitability: true,
            nonpayment_notice_days_given: 5,
            material_noncompliance_notice_days_given: 14,
            entry_notice_hours_given: 24,
            periodic_tenancy_notice_days_given: 30,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromChapter27_40;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::NotApplicableTenancyExemptFromChapter27_40
        );
    }

    #[test]
    fn deposit_returned_at_30_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection27_40_410;
        input.days_to_return_deposit = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::CompliantDepositReturnedWithItemizedNoticeWithin30Days
        );
    }

    #[test]
    fn deposit_returned_at_31_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection27_40_410;
        input.days_to_return_deposit = 31;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::ViolationDepositReturnedPast30DayDeadline
        );
    }

    #[test]
    fn no_wrongful_withholding_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::WrongfulWithholding3xPenaltyUnderSection27_40_410;
        input.deposit_wrongfully_withheld = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::CompliantNoWrongfulWithholding
        );
    }

    #[test]
    fn wrongful_withholding_3x_penalty_triggered_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::WrongfulWithholding3xPenaltyUnderSection27_40_410;
        input.deposit_wrongfully_withheld = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::ViolationWrongfulWithholding3xPenaltyTriggered
        );
    }

    #[test]
    fn landlord_maintains_habitability_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordHabitabilityObligationUnderSection27_40_440;
        input.landlord_maintains_habitability = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderSection27_40_440
        );
    }

    #[test]
    fn landlord_failed_habitability_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordHabitabilityObligationUnderSection27_40_440;
        input.landlord_maintains_habitability = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation
        );
    }

    #[test]
    fn lease_contains_five_day_provision_no_separate_notice_required_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderSection27_40_710;
        input.lease_five_day_notice_clause_status =
            LeaseFiveDayNoticeClauseStatus::LeaseContainsFiveDayNoticeProvision;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::CompliantNoSeparateNoticeRequiredLeaseContainsFiveDayProvision
        );
    }

    #[test]
    fn five_day_nonpayment_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderSection27_40_710;
        input.lease_five_day_notice_clause_status =
            LeaseFiveDayNoticeClauseStatus::LeaseDoesNotContainFiveDayNoticeProvision;
        input.nonpayment_notice_days_given = 5;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::CompliantFiveDayNonpaymentNoticeProperlyServed
        );
    }

    #[test]
    fn four_day_nonpayment_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderSection27_40_710;
        input.lease_five_day_notice_clause_status =
            LeaseFiveDayNoticeClauseStatus::LeaseDoesNotContainFiveDayNoticeProvision;
        input.nonpayment_notice_days_given = 4;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::ViolationNonpaymentNoticeShorterThan5Days
        );
    }

    #[test]
    fn material_noncompliance_14_day_cure_with_correction_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MaterialNoncomplianceNoticeUnderSection27_40_710;
        input.material_noncompliance_notice_days_given = 14;
        input.material_noncompliance_status =
            MaterialNoncomplianceStatus::TenantCorrectedNoncomplianceWithin14Days;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::CompliantMaterialNoncompliance14DayCureWithTenantCorrection
        );
    }

    #[test]
    fn material_noncompliance_14_day_notice_without_cure_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MaterialNoncomplianceNoticeUnderSection27_40_710;
        input.material_noncompliance_notice_days_given = 14;
        input.material_noncompliance_status =
            MaterialNoncomplianceStatus::TenantDidNotCorrectNoncomplianceWithin14Days;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::CompliantMaterialNoncompliance14DayNoticeWithoutCure
        );
    }

    #[test]
    fn material_noncompliance_13_day_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MaterialNoncomplianceNoticeUnderSection27_40_710;
        input.material_noncompliance_notice_days_given = 13;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::ViolationMaterialNoncomplianceNoticeShorterThan14Days
        );
    }

    #[test]
    fn twenty_four_hour_entry_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection27_40_540;
        input.entry_notice_hours_given = 24;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::Compliant24HourEntryNoticeProperlyServed
        );
    }

    #[test]
    fn twenty_three_hour_entry_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection27_40_540;
        input.entry_notice_hours_given = 23;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::ViolationEntryNoticeShorterThan24Hours
        );
    }

    #[test]
    fn thirty_day_month_to_month_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PeriodicTenancyTerminationUnderSection27_40_770;
        input.periodic_tenancy_type = PeriodicTenancyType::MonthToMonth;
        input.periodic_tenancy_notice_days_given = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::CompliantThirtyDayMonthToMonthNotice
        );
    }

    #[test]
    fn seven_day_week_to_week_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PeriodicTenancyTerminationUnderSection27_40_770;
        input.periodic_tenancy_type = PeriodicTenancyType::WeekToWeek;
        input.periodic_tenancy_notice_days_given = 7;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::CompliantSevenDayWeekToWeekNotice
        );
    }

    #[test]
    fn periodic_tenancy_notice_shorter_than_required_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PeriodicTenancyTerminationUnderSection27_40_770;
        input.periodic_tenancy_type = PeriodicTenancyType::MonthToMonth;
        input.periodic_tenancy_notice_days_given = 29;
        let out = check(&input);
        assert_eq!(
            out.mode,
            ScLandlordTenantMode::ViolationPeriodicTenancyNoticeShorterThanRequired
        );
    }

    #[test]
    fn constants_pin_south_carolina_landlord_tenant_statutory_thresholds() {
        assert_eq!(SC_CHAPTER_NUMBER, 40);
        assert_eq!(SC_TITLE_NUMBER, 27);
        assert_eq!(SC_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(SC_WRONGFUL_WITHHOLDING_MULTIPLIER, 3);
        assert_eq!(SC_NONPAYMENT_NOTICE_DAYS, 5);
        assert_eq!(SC_MATERIAL_NONCOMPLIANCE_NOTICE_DAYS, 14);
        assert_eq!(SC_MATERIAL_NONCOMPLIANCE_CURE_DAYS, 14);
        assert_eq!(SC_ENTRY_NOTICE_HOURS, 24);
        assert_eq!(SC_PERIODIC_TENANCY_NOTICE_DAYS_MONTHLY, 30);
        assert_eq!(SC_PERIODIC_TENANCY_NOTICE_DAYS_WEEKLY, 7);
        assert_eq!(SC_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_south_carolina_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("South Carolina Residential Landlord and Tenant Act"));
        assert!(joined.contains("SC Code §§ 27-40-10 through 27-40-940"));
        assert!(joined.contains("SC Code § 27-40-410"));
        assert!(joined.contains("ITEMIZED BY THE LANDLORD IN A WRITTEN NOTICE TO THE TENANT"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("THREE TIMES THE AMOUNT WRONGFULLY WITHHELD"));
        assert!(joined.contains("REASONABLE ATTORNEY'S FEES"));
        assert!(joined.contains("SC Code § 27-40-440"));
        assert!(joined.contains("BUILDING AND HOUSING CODES"));
        assert!(joined.contains("FIT AND HABITABLE CONDITION"));
        assert!(joined.contains("COMMON AREAS"));
        assert!(joined.contains("RUNNING WATER"));
        assert!(joined.contains("HOT WATER AT ALL TIMES"));
        assert!(joined.contains("HEAT"));
        assert!(joined.contains("SC Code § 27-40-710"));
        assert!(joined.contains("5 DAYS OF THE DUE DATE"));
        assert!(
            joined.contains("NOT REQUIRED TO FURNISH ANY SEPARATE OR ADDITIONAL WRITTEN NOTICE")
        );
        assert!(joined.contains("14-DAY WRITTEN NOTICE"));
        assert!(joined.contains("CORRECTS THE PROBLEM WITHIN 14 DAYS"));
        assert!(joined.contains("SC Code § 27-40-540"));
        assert!(joined.contains("AT LEAST 24 HOURS NOTICE"));
        assert!(joined.contains("SC Code § 27-40-770"));
        assert!(joined.contains("30 DAYS WRITTEN NOTICE"));
        assert!(joined.contains("week-to-week tenancies with 7 days written notice"));
    }
}
