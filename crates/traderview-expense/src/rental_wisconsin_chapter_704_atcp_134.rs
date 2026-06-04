//! Wisconsin Landlord-Tenant Law — Wisconsin Statutes
//! Chapter 704 + ATCP 134 Compliance Module — pure-compute
//! check for landlord statutory compliance with
//! Wisconsin's two-source landlord-tenant regime spanning
//! **Wis. Stat. Chapter 704 (Landlord and Tenant;
//! §§ 704.01 through 704.95)** AND **ATCP 134 (Wisconsin
//! Administrative Code — Residential Rental Practices)**.
//!
//! Wisconsin operates a hybrid statutory + administrative
//! regime: Chapter 704 provides the broad statutory
//! framework while ATCP 134 (promulgated by the Wisconsin
//! Department of Agriculture, Trade and Consumer
//! Protection) provides detailed residential rental
//! practices regulations.
//!
//! **Distinctive Wisconsin features**: **NON-WAIVABLE
//! habitability** under § 704.07 (waiver clause VOID);
//! **TIERED EVICTION NOTICE STRUCTURE** under § 704.17
//! (5-day cure for first violation in 12 months / 14-day
//! no-cure for second or subsequent violation / 30-day for
//! leases > 1 year); **28-DAY PERIODIC TENANCY NOTICE**
//! under § 704.19; **CHECK-IN SHEET REQUIREMENT** under
//! ATCP 134.06.
//!
//! Web research (verified 2026-06-03):
//! - **Wis. Stat. Chapter 704 + ATCP 134**: Wisconsin's two-source landlord-tenant regime; Chapter 704 (§§ 704.01-704.95) provides the broad statutory framework while ATCP 134 provides detailed residential rental practices regulations ([Wisconsin Legislature — Chapter 704 PDF](https://docs.legis.wisconsin.gov/statutes/statutes/704.pdf); [Wisconsin Legislature — Chapter 704](https://docs.legis.wisconsin.gov/statutes/statutes/704); [LeaseLenses — Wisconsin Landlord-Tenant Law Guide 2026](https://www.leaselenses.com/blog/wisconsin-landlord-tenant-law-guide/); [McLario — A Guide to Tenant Rights and Responsibilities in Wisconsin](https://mclario.com/articles/a-guide-to-tenant-rights-and-responsibilities-in-wisconsin/); [iPropertyManagement — Wisconsin Landlord Tenant Laws 2026](https://ipropertymanagement.com/laws/wisconsin-landlord-tenant-rights); [Justia — 2025 Wisconsin Statutes Chapter 704 § 704.05](https://law.justia.com/codes/wisconsin/chapter-704/section-704-05/); [Justia — 2024 Wisconsin Statutes Chapter 704 § 704.28](https://law.justia.com/codes/wisconsin/chapter-704/section-704-28/); [Innago — Wisconsin Landlord Tenant Laws 2026](https://innago.com/wisconsin-landlord-tenant-laws/); [Landlord-Tenant-Law — Wisconsin Landlord Tenant Law in Plain English](https://www.landlord-tenant-law.com/wisconsin-landlord-tenant-law.html); [Wisconsin Legislature — § 704.17](https://docs.legis.wisconsin.gov/statutes/statutes/704/17); [PayRent — Wisconsin Eviction Laws and Eviction Process](https://www.payrent.com/articles/wisconsin-eviction-laws-and-eviction-process/); [Tenant Screening Background Check — Wisconsin Eviction Notice Laws](https://tenantscreeningbackgroundcheck.com/wisconsin-eviction-notice-laws/); [Justia — 2025 Wisconsin Statutes Chapter 704 § 704.17](https://law.justia.com/codes/wisconsin/chapter-704/section-704-17/); [Tenant Resource Center — Eviction](https://www.tenantresourcecenter.org/eviction); [Tenant Resource Center — Eviction Notices](https://www.tenantresourcecenter.org/eviction_notices); [Nolo — Wisconsin Eviction Laws and Tenant Defenses](https://www.nolo.com/legal-encyclopedia/tenant-defenses-evictions-wisconsin.html); [Wisconsin Legislative Council — Eviction of a Residential Tenant Information Memo 2024-12](https://docs.legis.wisconsin.gov/misc/lc/information_memos/2024/im_2024_12); [Wisconsin Legislature — ATCP 134 PDF](https://docs.legis.wisconsin.gov/code/admin_code/atcp/090/134.pdf); [Wisconsin Legislature — ATCP 134.06](https://docs.legis.wisconsin.gov/code/admin_code/atcp/090/134/06); [Justia — Wisconsin Administrative Code ATCP 134.06](https://regulations.justia.com/states/wisconsin/atcp/atcp-90-139/chapter-atcp-134/section-atcp-134-06/); [Wisconsin Legislature — Chapter ATCP 134](https://docs.legis.wisconsin.gov/code/admin_code/atcp/090/134/)).
//! - **Wis. Stat. § 704.07 Habitability — NON-WAIVABLE**: § 704.07 applies to any nonresidential tenancy if there is no contrary provision in writing signed by both parties AND **TO ALL RESIDENTIAL TENANCIES**; an agreement to **WAIVE THE REQUIREMENTS** of this section in a residential tenancy, including an agreement in a rental agreement, **IS VOID** — Wisconsin's habitability provision is one of the strongest non-waivable habitability protections in the United States.
//! - **Wis. Stat. § 704.17(1m)(a) 5-Day Notice with Cure Right (Nonpayment)**: for evictions based on non-payment of rent FIRST violation in 12 months, the landlord must provide a **5-DAY NOTICE**; the tenant has **5 DAYS TO PAY RENT OR CURE** noncompliance.
//! - **Wis. Stat. § 704.17(1m)(b) 14-Day Notice with No Cure (Second Violation in 12 Months)**: landlord must provide a **14-DAY NOTICE** to the tenant if it is the **SECOND OR SUBSEQUENT VIOLATION WITHIN 12 MONTHS** of the first violation; if the notice is the second notice, the landlord does **NOT HAVE TO GIVE THE TENANT AN OPPORTUNITY TO FIX IT** (no cure right).
//! - **Wis. Stat. § 704.17(2) 30-Day Notice (Leases > 1 Year)**: a **30-DAY EVICTION NOTICE** in Wisconsin applies to leases longer than 1 year.
//! - **Wis. Stat. § 704.19 28-Day Periodic Tenancy Termination**: month-to-month tenancies can be ended by giving at least **28 DAYS' WRITTEN NOTICE** to the other party.
//! - **Wis. Stat. § 704.28 21-Day Security Deposit Return**: a landlord shall deliver or mail to a tenant the **FULL AMOUNT OF ANY SECURITY DEPOSIT** paid by the tenant, less any amounts that may be withheld, within **21 DAYS** after the tenant vacates the premises on the termination date of the rental agreement.
//! - **Wis. Stat. § 704.28 Permissible Withholding Reasons**: landlord may withhold only amounts reasonably necessary to pay for (1) **TENANT DAMAGE, WASTE, OR NEGLECT** of the premises; AND (2) **UNPAID RENT** for which the tenant is legally responsible; the section does **NOT AUTHORIZE** a landlord to withhold any amount from a security deposit for **NORMAL WEAR AND TEAR**, or for other damages or losses for which the tenant cannot reasonably be held responsible under applicable law.
//! - **ATCP 134.06 Check-In Sheet Required**: under ATCP 134.06, landlords must provide tenants with a **CHECK-IN SHEET** at the start of the tenancy so the tenant can document the condition of the premises; before a landlord accepts a security deposit, or converts an earnest money deposit to a security deposit, the landlord shall **NOTIFY THE TENANT IN WRITING** that the tenant may complete the check-in inspection by a specified deadline date which is **NOT LESS THAN 7 DAYS** after the start of tenancy.
//! - **ATCP 134.06(2) 21-Day Return + Itemized Statement (Administrative Rule)**: ATCP 134.06(2) confirms the 21-day window applies to the return of the deposit or the delivery of an **ITEMIZED STATEMENT OF DEDUCTIONS**.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const WI_CHAPTER_NUMBER: u32 = 704;
pub const WI_ATCP_CHAPTER_NUMBER: u32 = 134;
pub const WI_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 21;
pub const WI_NONPAYMENT_5_DAY_NOTICE_DAYS: u32 = 5;
pub const WI_REPEAT_VIOLATION_14_DAY_NOTICE_DAYS: u32 = 14;
pub const WI_LONG_LEASE_30_DAY_NOTICE_DAYS: u32 = 30;
pub const WI_PERIODIC_TENANCY_TERMINATION_NOTICE_DAYS: u32 = 28;
pub const WI_REPEAT_VIOLATION_LOOKBACK_MONTHS: u32 = 12;
pub const WI_CHECK_IN_INSPECTION_NOTICE_DAYS: u32 = 7;
pub const WI_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromChapter704,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HabitabilityWaiverStatus {
    NoWaiverAttempted,
    LeaseAttemptsToWaiveHabitabilityVoid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionNoticeType {
    FirstViolationFiveDayCureNotice,
    SecondOrSubsequentViolationFourteenDayNoCureNotice,
    LeaseLongerThanOneYearThirtyDayNotice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WithholdingReasonStatus {
    WithheldForDamageWasteNeglectOrUnpaidRent,
    WithheldForNormalWearAndTearOrOtherImpermissibleReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckInSheetStatus {
    CheckInSheetProvidedAndSevenDayNoticeWritten,
    CheckInSheetNotProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    NonWaivableHabitabilityUnderSection70407,
    EvictionNoticeUnderSection70417,
    PeriodicTenancyTerminationUnderSection70419,
    SecurityDepositReturnUnderSection70428,
    PermissibleWithholdingReasonsUnderSection70428,
    CheckInSheetRequirementUnderAtcp13406,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WiLandlordTenantMode {
    NotApplicableTenancyExemptFromChapter704,
    CompliantNonWaivableHabitabilityPreserved,
    CompliantFiveDayCureNoticeProperlyServed,
    CompliantFourteenDayNoCureNoticeProperlyServedForRepeatViolation,
    CompliantThirtyDayNoticeForLeaseLongerThanOneYearProperlyServed,
    CompliantTwentyEightDayPeriodicTenancyNoticeProperlyServed,
    CompliantDepositReturnedWithItemizedStatementWithin21Days,
    CompliantWithholdingForPermissibleReasonsOnly,
    CompliantCheckInSheetProvidedWithSevenDayInspectionNotice,
    ViolationLeaseAttemptsToWaiveHabitabilityVoidUnder70407,
    ViolationFiveDayNoticeShorterThanFiveDays,
    ViolationFourteenDayNoticeShorterThanFourteenDaysForRepeatViolation,
    ViolationThirtyDayNoticeShorterThanThirtyDaysForLeaseLongerThanOneYear,
    ViolationPeriodicTenancyTerminationShorterThanTwentyEightDays,
    ViolationDepositReturnedPast21DayDeadline,
    ViolationWithheldForNormalWearAndTearOrImpermissibleReason,
    ViolationCheckInSheetNotProvidedUnderAtcp13406,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub habitability_waiver_status: HabitabilityWaiverStatus,
    pub eviction_notice_type: EvictionNoticeType,
    pub withholding_reason_status: WithholdingReasonStatus,
    pub check_in_sheet_status: CheckInSheetStatus,
    pub compliance_aspect: ComplianceAspect,
    pub eviction_notice_days_given: u32,
    pub periodic_tenancy_termination_notice_days_given: u32,
    pub days_to_return_deposit: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: WiLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type WiLandlordTenantInput = Input;
pub type WiLandlordTenantOutput = Output;
pub type WiLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Wisconsin Landlord-Tenant Law — Wisconsin Statutes Chapter 704 (Landlord and Tenant; §§ 704.01-704.95) + ATCP 134 (Wisconsin Administrative Code — Residential Rental Practices)".to_string(),
        "Wis. Stat. § 704.07 Habitability — NON-WAIVABLE — § 704.07 applies TO ALL RESIDENTIAL TENANCIES; an agreement to WAIVE THE REQUIREMENTS of this section in a residential tenancy, including an agreement in a rental agreement, IS VOID".to_string(),
        "Wis. Stat. § 704.17(1m)(a) 5-Day Notice with Cure Right — for FIRST nonpayment violation in 12 months, landlord must provide a 5-DAY NOTICE; tenant has 5 DAYS TO PAY RENT OR CURE noncompliance".to_string(),
        "Wis. Stat. § 704.17(1m)(b) 14-Day Notice with No Cure — for SECOND OR SUBSEQUENT VIOLATION WITHIN 12 MONTHS, landlord must provide 14-DAY NOTICE; landlord does NOT HAVE TO GIVE THE TENANT AN OPPORTUNITY TO FIX IT (no cure right)".to_string(),
        "Wis. Stat. § 704.17(2) 30-Day Notice for Leases > 1 Year — 30-DAY EVICTION NOTICE applies to leases longer than 1 year".to_string(),
        "Wis. Stat. § 704.19 28-Day Periodic Tenancy Termination — month-to-month tenancies can be ended by giving at least 28 DAYS' WRITTEN NOTICE to the other party".to_string(),
        "Wis. Stat. § 704.28 21-Day Security Deposit Return — landlord shall deliver or mail to tenant the FULL AMOUNT OF ANY SECURITY DEPOSIT paid by tenant, less any amounts that may be withheld, within 21 DAYS after the tenant vacates the premises on the termination date of the rental agreement".to_string(),
        "Wis. Stat. § 704.28 Permissible Withholding Reasons — landlord may withhold only amounts reasonably necessary to pay for (1) TENANT DAMAGE, WASTE, OR NEGLECT of the premises; AND (2) UNPAID RENT for which the tenant is legally responsible; the section does NOT AUTHORIZE a landlord to withhold any amount from a security deposit for NORMAL WEAR AND TEAR".to_string(),
        "ATCP 134.06 Check-In Sheet Required — landlords must provide tenants with a CHECK-IN SHEET at the start of the tenancy so the tenant can document the condition of the premises; landlord shall NOTIFY THE TENANT IN WRITING that the tenant may complete the check-in inspection by a specified deadline date which is NOT LESS THAN 7 DAYS after the start of tenancy".to_string(),
        "ATCP 134.06(2) 21-Day Return + Itemized Statement (Administrative Rule) — ATCP 134.06(2) confirms the 21-day window applies to the return of the deposit or the delivery of an ITEMIZED STATEMENT OF DEDUCTIONS".to_string(),
        "Wisconsin Legislature + LeaseLenses + McLario + iPropertyManagement + Justia + Innago + Landlord-Tenant-Law + PayRent + Tenant Screening Background Check + Tenant Resource Center + Nolo + Wisconsin Legislative Council — practitioner overviews of Wisconsin Statutes Chapter 704 + ATCP 134".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromChapter704 {
        return Output {
            mode: WiLandlordTenantMode::NotApplicableTenancyExemptFromChapter704,
            statutory_basis: "Wisconsin Statutes Chapter 704 jurisdiction — tenancy exempt from Chapter 704 coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from Wis. Stat. Chapter 704; Wisconsin landlord-tenant obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::NonWaivableHabitabilityUnderSection70407 => match input
            .habitability_waiver_status
        {
            HabitabilityWaiverStatus::NoWaiverAttempted => Output {
                mode: WiLandlordTenantMode::CompliantNonWaivableHabitabilityPreserved,
                statutory_basis: "Wis. Stat. § 704.07 — non-waivable habitability preserved".to_string(),
                notes: "COMPLIANT: non-waivable habitability under Wis. Stat. § 704.07 preserved; no waiver attempted in rental agreement.".to_string(),
                citations,
            },
            HabitabilityWaiverStatus::LeaseAttemptsToWaiveHabitabilityVoid => Output {
                mode: WiLandlordTenantMode::ViolationLeaseAttemptsToWaiveHabitabilityVoidUnder70407,
                statutory_basis: "Wis. Stat. § 704.07 — habitability waiver in rental agreement is VOID".to_string(),
                notes: "VIOLATION: lease attempts to waive habitability requirements under Wis. Stat. § 704.07; waiver clause is VOID and unenforceable.".to_string(),
                citations,
            },
        },
        ComplianceAspect::EvictionNoticeUnderSection70417 => match input.eviction_notice_type {
            EvictionNoticeType::FirstViolationFiveDayCureNotice => {
                if input.eviction_notice_days_given >= WI_NONPAYMENT_5_DAY_NOTICE_DAYS {
                    Output {
                        mode: WiLandlordTenantMode::CompliantFiveDayCureNoticeProperlyServed,
                        statutory_basis: "Wis. Stat. § 704.17(1m)(a) — 5-day cure notice properly served for first nonpayment violation in 12 months".to_string(),
                        notes: format!(
                            "COMPLIANT: {d}-day cure notice satisfies 5-day statutory minimum under Wis. Stat. § 704.17(1m)(a) for first nonpayment violation in 12 months.",
                            d = input.eviction_notice_days_given,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: WiLandlordTenantMode::ViolationFiveDayNoticeShorterThanFiveDays,
                        statutory_basis: "Wis. Stat. § 704.17(1m)(a) — 5-day cure notice shorter than 5-day statutory minimum".to_string(),
                        notes: format!(
                            "VIOLATION: {d}-day cure notice shorter than 5-day statutory minimum under Wis. Stat. § 704.17(1m)(a).",
                            d = input.eviction_notice_days_given,
                        ),
                        citations,
                    }
                }
            }
            EvictionNoticeType::SecondOrSubsequentViolationFourteenDayNoCureNotice => {
                if input.eviction_notice_days_given >= WI_REPEAT_VIOLATION_14_DAY_NOTICE_DAYS {
                    Output {
                        mode: WiLandlordTenantMode::CompliantFourteenDayNoCureNoticeProperlyServedForRepeatViolation,
                        statutory_basis: "Wis. Stat. § 704.17(1m)(b) — 14-day no-cure notice properly served for second or subsequent violation within 12 months".to_string(),
                        notes: format!(
                            "COMPLIANT: {d}-day no-cure notice satisfies 14-day statutory minimum under Wis. Stat. § 704.17(1m)(b) for second or subsequent violation within 12 months.",
                            d = input.eviction_notice_days_given,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: WiLandlordTenantMode::ViolationFourteenDayNoticeShorterThanFourteenDaysForRepeatViolation,
                        statutory_basis: "Wis. Stat. § 704.17(1m)(b) — 14-day no-cure notice shorter than 14-day statutory minimum".to_string(),
                        notes: format!(
                            "VIOLATION: {d}-day no-cure notice shorter than 14-day statutory minimum under Wis. Stat. § 704.17(1m)(b).",
                            d = input.eviction_notice_days_given,
                        ),
                        citations,
                    }
                }
            }
            EvictionNoticeType::LeaseLongerThanOneYearThirtyDayNotice => {
                if input.eviction_notice_days_given >= WI_LONG_LEASE_30_DAY_NOTICE_DAYS {
                    Output {
                        mode: WiLandlordTenantMode::CompliantThirtyDayNoticeForLeaseLongerThanOneYearProperlyServed,
                        statutory_basis: "Wis. Stat. § 704.17(2) — 30-day notice properly served for lease longer than 1 year".to_string(),
                        notes: format!(
                            "COMPLIANT: {d}-day notice satisfies 30-day statutory minimum under Wis. Stat. § 704.17(2) for lease longer than 1 year.",
                            d = input.eviction_notice_days_given,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: WiLandlordTenantMode::ViolationThirtyDayNoticeShorterThanThirtyDaysForLeaseLongerThanOneYear,
                        statutory_basis: "Wis. Stat. § 704.17(2) — 30-day notice shorter than 30-day statutory minimum for lease longer than 1 year".to_string(),
                        notes: format!(
                            "VIOLATION: {d}-day notice shorter than 30-day statutory minimum under Wis. Stat. § 704.17(2).",
                            d = input.eviction_notice_days_given,
                        ),
                        citations,
                    }
                }
            }
        },
        ComplianceAspect::PeriodicTenancyTerminationUnderSection70419 => {
            if input.periodic_tenancy_termination_notice_days_given
                >= WI_PERIODIC_TENANCY_TERMINATION_NOTICE_DAYS
            {
                Output {
                    mode: WiLandlordTenantMode::CompliantTwentyEightDayPeriodicTenancyNoticeProperlyServed,
                    statutory_basis: "Wis. Stat. § 704.19 — 28-day periodic tenancy termination notice properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day periodic tenancy termination notice satisfies 28-day statutory minimum under Wis. Stat. § 704.19.",
                        d = input.periodic_tenancy_termination_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: WiLandlordTenantMode::ViolationPeriodicTenancyTerminationShorterThanTwentyEightDays,
                    statutory_basis: "Wis. Stat. § 704.19 — periodic tenancy termination notice shorter than 28-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day periodic tenancy termination notice shorter than 28-day statutory minimum under Wis. Stat. § 704.19.",
                        d = input.periodic_tenancy_termination_notice_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::SecurityDepositReturnUnderSection70428 => {
            if input.days_to_return_deposit <= WI_DEPOSIT_RETURN_DEADLINE_DAYS {
                Output {
                    mode: WiLandlordTenantMode::CompliantDepositReturnedWithItemizedStatementWithin21Days,
                    statutory_basis: "Wis. Stat. § 704.28 + ATCP 134.06(2) — deposit returned with itemized statement within 21 days".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit returned with itemized statement at day {d} (within 21-day statutory window) under Wis. Stat. § 704.28 + ATCP 134.06(2).",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: WiLandlordTenantMode::ViolationDepositReturnedPast21DayDeadline,
                    statutory_basis: "Wis. Stat. § 704.28 — deposit return exceeded 21-day statutory window".to_string(),
                    notes: format!(
                        "VIOLATION: deposit returned at day {d} past 21-day statutory window under Wis. Stat. § 704.28.",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::PermissibleWithholdingReasonsUnderSection70428 => match input
            .withholding_reason_status
        {
            WithholdingReasonStatus::WithheldForDamageWasteNeglectOrUnpaidRent => Output {
                mode: WiLandlordTenantMode::CompliantWithholdingForPermissibleReasonsOnly,
                statutory_basis: "Wis. Stat. § 704.28 — withholding limited to damage/waste/neglect or unpaid rent only (normal wear and tear excluded)".to_string(),
                notes: "COMPLIANT: withholding limited to permissible reasons under Wis. Stat. § 704.28 (tenant damage, waste, or neglect + unpaid rent; normal wear and tear NOT authorized).".to_string(),
                citations,
            },
            WithholdingReasonStatus::WithheldForNormalWearAndTearOrOtherImpermissibleReason => {
                Output {
                    mode: WiLandlordTenantMode::ViolationWithheldForNormalWearAndTearOrImpermissibleReason,
                    statutory_basis: "Wis. Stat. § 704.28 — withholding for normal wear and tear or other impermissible reason prohibited".to_string(),
                    notes: "VIOLATION: withholding for normal wear and tear or other impermissible reason under Wis. Stat. § 704.28; only tenant damage/waste/neglect or unpaid rent permitted.".to_string(),
                    citations,
                }
            }
        },
        ComplianceAspect::CheckInSheetRequirementUnderAtcp13406 => match input.check_in_sheet_status {
            CheckInSheetStatus::CheckInSheetProvidedAndSevenDayNoticeWritten => Output {
                mode: WiLandlordTenantMode::CompliantCheckInSheetProvidedWithSevenDayInspectionNotice,
                statutory_basis: "ATCP 134.06 — check-in sheet provided with 7-day inspection notice".to_string(),
                notes: "COMPLIANT: landlord provided check-in sheet at start of tenancy AND written notice giving tenant at least 7 days to complete check-in inspection under ATCP 134.06.".to_string(),
                citations,
            },
            CheckInSheetStatus::CheckInSheetNotProvided => Output {
                mode: WiLandlordTenantMode::ViolationCheckInSheetNotProvidedUnderAtcp13406,
                statutory_basis: "ATCP 134.06 — check-in sheet not provided or 7-day inspection notice missing".to_string(),
                notes: "VIOLATION: landlord did not provide check-in sheet at start of tenancy OR did not provide written 7-day inspection notice under ATCP 134.06.".to_string(),
                citations,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            tenancy_coverage: TenancyCoverage::CoveredResidentialTenancy,
            habitability_waiver_status: HabitabilityWaiverStatus::NoWaiverAttempted,
            eviction_notice_type: EvictionNoticeType::FirstViolationFiveDayCureNotice,
            withholding_reason_status:
                WithholdingReasonStatus::WithheldForDamageWasteNeglectOrUnpaidRent,
            check_in_sheet_status:
                CheckInSheetStatus::CheckInSheetProvidedAndSevenDayNoticeWritten,
            compliance_aspect: ComplianceAspect::NonWaivableHabitabilityUnderSection70407,
            eviction_notice_days_given: 5,
            periodic_tenancy_termination_notice_days_given: 28,
            days_to_return_deposit: 14,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromChapter704;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::NotApplicableTenancyExemptFromChapter704
        );
    }

    #[test]
    fn non_waivable_habitability_preserved_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonWaivableHabitabilityUnderSection70407;
        input.habitability_waiver_status = HabitabilityWaiverStatus::NoWaiverAttempted;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::CompliantNonWaivableHabitabilityPreserved
        );
    }

    #[test]
    fn lease_attempts_to_waive_habitability_void_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonWaivableHabitabilityUnderSection70407;
        input.habitability_waiver_status =
            HabitabilityWaiverStatus::LeaseAttemptsToWaiveHabitabilityVoid;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::ViolationLeaseAttemptsToWaiveHabitabilityVoidUnder70407
        );
    }

    #[test]
    fn five_day_cure_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionNoticeUnderSection70417;
        input.eviction_notice_type = EvictionNoticeType::FirstViolationFiveDayCureNotice;
        input.eviction_notice_days_given = 5;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::CompliantFiveDayCureNoticeProperlyServed
        );
    }

    #[test]
    fn four_day_cure_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionNoticeUnderSection70417;
        input.eviction_notice_type = EvictionNoticeType::FirstViolationFiveDayCureNotice;
        input.eviction_notice_days_given = 4;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::ViolationFiveDayNoticeShorterThanFiveDays
        );
    }

    #[test]
    fn fourteen_day_no_cure_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionNoticeUnderSection70417;
        input.eviction_notice_type =
            EvictionNoticeType::SecondOrSubsequentViolationFourteenDayNoCureNotice;
        input.eviction_notice_days_given = 14;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::CompliantFourteenDayNoCureNoticeProperlyServedForRepeatViolation
        );
    }

    #[test]
    fn thirteen_day_no_cure_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionNoticeUnderSection70417;
        input.eviction_notice_type =
            EvictionNoticeType::SecondOrSubsequentViolationFourteenDayNoCureNotice;
        input.eviction_notice_days_given = 13;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::ViolationFourteenDayNoticeShorterThanFourteenDaysForRepeatViolation
        );
    }

    #[test]
    fn thirty_day_lease_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionNoticeUnderSection70417;
        input.eviction_notice_type =
            EvictionNoticeType::LeaseLongerThanOneYearThirtyDayNotice;
        input.eviction_notice_days_given = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::CompliantThirtyDayNoticeForLeaseLongerThanOneYearProperlyServed
        );
    }

    #[test]
    fn twenty_nine_day_lease_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionNoticeUnderSection70417;
        input.eviction_notice_type =
            EvictionNoticeType::LeaseLongerThanOneYearThirtyDayNotice;
        input.eviction_notice_days_given = 29;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::ViolationThirtyDayNoticeShorterThanThirtyDaysForLeaseLongerThanOneYear
        );
    }

    #[test]
    fn twenty_eight_day_periodic_tenancy_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PeriodicTenancyTerminationUnderSection70419;
        input.periodic_tenancy_termination_notice_days_given = 28;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::CompliantTwentyEightDayPeriodicTenancyNoticeProperlyServed
        );
    }

    #[test]
    fn twenty_seven_day_periodic_tenancy_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PeriodicTenancyTerminationUnderSection70419;
        input.periodic_tenancy_termination_notice_days_given = 27;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::ViolationPeriodicTenancyTerminationShorterThanTwentyEightDays
        );
    }

    #[test]
    fn deposit_returned_at_21_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection70428;
        input.days_to_return_deposit = 21;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::CompliantDepositReturnedWithItemizedStatementWithin21Days
        );
    }

    #[test]
    fn deposit_returned_at_22_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection70428;
        input.days_to_return_deposit = 22;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::ViolationDepositReturnedPast21DayDeadline
        );
    }

    #[test]
    fn permissible_withholding_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PermissibleWithholdingReasonsUnderSection70428;
        input.withholding_reason_status =
            WithholdingReasonStatus::WithheldForDamageWasteNeglectOrUnpaidRent;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::CompliantWithholdingForPermissibleReasonsOnly
        );
    }

    #[test]
    fn withholding_for_normal_wear_and_tear_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PermissibleWithholdingReasonsUnderSection70428;
        input.withholding_reason_status =
            WithholdingReasonStatus::WithheldForNormalWearAndTearOrOtherImpermissibleReason;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::ViolationWithheldForNormalWearAndTearOrImpermissibleReason
        );
    }

    #[test]
    fn check_in_sheet_provided_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CheckInSheetRequirementUnderAtcp13406;
        input.check_in_sheet_status =
            CheckInSheetStatus::CheckInSheetProvidedAndSevenDayNoticeWritten;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::CompliantCheckInSheetProvidedWithSevenDayInspectionNotice
        );
    }

    #[test]
    fn check_in_sheet_not_provided_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CheckInSheetRequirementUnderAtcp13406;
        input.check_in_sheet_status = CheckInSheetStatus::CheckInSheetNotProvided;
        let out = check(&input);
        assert_eq!(
            out.mode,
            WiLandlordTenantMode::ViolationCheckInSheetNotProvidedUnderAtcp13406
        );
    }

    #[test]
    fn constants_pin_wisconsin_landlord_tenant_statutory_thresholds() {
        assert_eq!(WI_CHAPTER_NUMBER, 704);
        assert_eq!(WI_ATCP_CHAPTER_NUMBER, 134);
        assert_eq!(WI_DEPOSIT_RETURN_DEADLINE_DAYS, 21);
        assert_eq!(WI_NONPAYMENT_5_DAY_NOTICE_DAYS, 5);
        assert_eq!(WI_REPEAT_VIOLATION_14_DAY_NOTICE_DAYS, 14);
        assert_eq!(WI_LONG_LEASE_30_DAY_NOTICE_DAYS, 30);
        assert_eq!(WI_PERIODIC_TENANCY_TERMINATION_NOTICE_DAYS, 28);
        assert_eq!(WI_REPEAT_VIOLATION_LOOKBACK_MONTHS, 12);
        assert_eq!(WI_CHECK_IN_INSPECTION_NOTICE_DAYS, 7);
        assert_eq!(WI_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_wisconsin_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Wisconsin Landlord-Tenant Law"));
        assert!(joined.contains("Wisconsin Statutes Chapter 704"));
        assert!(joined.contains("ATCP 134"));
        assert!(joined.contains("Wis. Stat. § 704.07"));
        assert!(joined.contains("NON-WAIVABLE"));
        assert!(joined.contains("TO ALL RESIDENTIAL TENANCIES"));
        assert!(joined.contains("WAIVE THE REQUIREMENTS"));
        assert!(joined.contains("IS VOID"));
        assert!(joined.contains("Wis. Stat. § 704.17(1m)(a)"));
        assert!(joined.contains("5-DAY NOTICE"));
        assert!(joined.contains("5 DAYS TO PAY RENT OR CURE"));
        assert!(joined.contains("Wis. Stat. § 704.17(1m)(b)"));
        assert!(joined.contains("14-DAY NOTICE"));
        assert!(joined.contains("SECOND OR SUBSEQUENT VIOLATION WITHIN 12 MONTHS"));
        assert!(joined.contains("NOT HAVE TO GIVE THE TENANT AN OPPORTUNITY TO FIX IT"));
        assert!(joined.contains("Wis. Stat. § 704.17(2)"));
        assert!(joined.contains("30-DAY EVICTION NOTICE"));
        assert!(joined.contains("Wis. Stat. § 704.19"));
        assert!(joined.contains("28 DAYS' WRITTEN NOTICE"));
        assert!(joined.contains("Wis. Stat. § 704.28"));
        assert!(joined.contains("21 DAYS"));
        assert!(joined.contains("TENANT DAMAGE, WASTE, OR NEGLECT"));
        assert!(joined.contains("UNPAID RENT"));
        assert!(joined.contains("NORMAL WEAR AND TEAR"));
        assert!(joined.contains("ATCP 134.06"));
        assert!(joined.contains("CHECK-IN SHEET"));
        assert!(joined.contains("NOTIFY THE TENANT IN WRITING"));
        assert!(joined.contains("NOT LESS THAN 7 DAYS"));
        assert!(joined.contains("ITEMIZED STATEMENT OF DEDUCTIONS"));
    }
}
