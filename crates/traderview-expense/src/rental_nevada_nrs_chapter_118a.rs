//! Nevada Landlord and Tenant: Dwellings —
//! NRS Chapter 118A + Chapter 40 Compliance Module.
//!
//! Pure-compute check for landlord statutory compliance
//! with Nevada's two-chapter URLTA-based regime: NRS
//! Chapter 118A governs the substantive landlord-tenant
//! relationship while NRS Chapter 40 governs eviction
//! procedure (notice periods, summary eviction
//! mechanism, unlawful detainer).
//!
//! **Distinctive Nevada features**: **3-MONTHS RENT
//! DEPOSIT CAP** under § 118A.242 — among the **HIGHEST
//! URLTA STATE CAPS** in the United States (cf. IA 2
//! months, MI 1.5 months, most URLTA states 1-2 months);
//! **SURETY BOND IN LIEU OF DEPOSIT** explicitly permitted
//! under § 118A.242; **7-DAY PAY-OR-QUIT** nonpayment
//! notice under NRS § 40.2512 (matching KY 7-day);
//! **5-DAY CURE OR QUIT** lease violation notice under
//! § 40.2516 with **3 JUDICIAL DAYS** to cure (UNUSUAL —
//! most states use 7-14 day cure windows; NV uses 3
//! business days only); **SERVICE BY CONSTABLE, SHERIFF,
//! OR LICENSED PROCESS SERVER REQUIRED** for eviction
//! notices (formal-service requirement); **24-HOUR ENTRY
//! NOTICE** during **NORMAL BUSINESS HOURS** under
//! § 118A.330 (URLTA-standard); **HABITABILITY** under
//! § 118A.290 with enumerated effective waterproofing /
//! weather protection / plumbing requirements.
//!
//! Web research (verified 2026-06-04):
//! - **Nevada Landlord and Tenant: Dwellings — NRS Chapter 118A** + **NRS Chapter 40** ([Nevada Legislature — NRS Chapter 118A Landlord and Tenant: Dwellings](https://www.leg.state.nv.us/nrs/nrs-118a.html); [Nevada Legislature — NRS Chapter 40 Actions and Proceedings in Particular Cases Concerning Property](https://www.leg.state.nv.us/nrs/nrs-040.html); [Justia — 2025 NRS § 118A.242 Security Deposit](https://law.justia.com/codes/nevada/chapter-118a/statute-118a-242/); [Justia — 2024 NRS § 40.253 Unlawful Detainer Summary Eviction Default in Payment of Rent](https://law.justia.com/codes/nevada/chapter-40/statute-40-253/); [Nevada Public Law — NRS 40.253](https://nevada.public.law/statutes/nrs_40.253); [Nevada Public Law — NRS 40.2516 Failure to Perform Conditions of Lease](https://nevada.public.law/statutes/nrs_40.2516); [Justia — 2024 NRS § 40.2516 Unlawful Detainer Possession After Failure to Perform Conditions of Lease](https://law.justia.com/codes/nevada/chapter-40/statute-40-2516/); [Washoe County RJC — Five-Day Notice to Perform Lease Condition or Quit (NRS 40.2516) PDF](https://www.washoecounty.gov/rjc/files/forms/form_civil_eviction_landlord_notice-five_day_to_perform_lease_or_quit.pdf); [Civil Law Self-Help Center — Rent Notices](https://www.civillawselfhelpcenter.org/m/self-help/evictions-housing/evictions/types-of-eviction-notices/80-rent-notices); [Civil Law Self-Help Center — Lease Violation Notices](https://www.civillawselfhelpcenter.org/self-help/evictions-housing/evictions/types-of-eviction-notices/82-lease-violation-notices); [Civil Law Self-Help Center — Security Deposits](https://www.civillawselfhelpcenter.org/self-help/evictions-housing/security-deposits); [Civil Law Self-Help Center — No-Cause Notices](https://www.civillawselfhelpcenter.org/evictionsevictions/eviction-notices/84-); [Nevada Evictions](https://www.nevadaevictions.com/e); [Tenant Rights — Nevada Security Deposit Return Law (NRS 118A.242)](https://tenant-rights.com/nevada/nevada-security-deposit-return-law); [Reno's Property Management — What Nevada Law Requires of Landlords: NRS 118A](https://www.renospropertymanagement.com/blog/what-nevada-law-requires-of-landlords-a-breakdown-of-nrs-118a); [Hemlane — Nevada Tenant-Landlord Rental Laws 2026](https://www.hemlane.com/resources/nevada-tenant-landlord-law/); [Hemlane — Nevada Security Deposit Laws 2026](https://www.hemlane.com/resources/nevada-security-deposit-laws/); [Karsaz Law — Understanding Residential Tenancy Laws in Nevada](https://karsaz-law.com/understanding-residential-tenancy-laws-in-nevada/); [Karsaz Law — Security Deposits Best Practices](https://karsaz-law.com/security-deposit-blog/); [Cirac Law — Security Deposits in Residential Tenancies](http://ciraclaw.com/security-deposits-residential-tenancies/); [iPropertyManagement — Nevada 7 Day Notice to Pay or Quit](https://ipropertymanagement.com/templates/nevada-7-day-notice-to-quit); [LPSNV — Flow Chart Summary Eviction Process Other than Non-Payment of Rent](https://www.lpsnv.com/flow_chart_summary_eviction.asp); [LawServer — Nevada Revised Statutes 40.253](https://www.lawserver.com/law/state/nevada/nrs/nevada_revised_statutes_40-253); [Clark County NV — Eviction Process](https://www.clarkcountynv.gov/government/departments/constable/constable_las_vegas_township/services/eviction-process); [Landlord-Tenant-Law — Nevada Landlord Tenant Law in Plain English](https://www.landlord-tenant-law.com/nevada-landlord-tenant-law.html); [PayRent — Nevada Landlord Tenant Laws](https://www.payrent.com/articles/nevada-landlord-tenant-laws/); [Creech AFB — Legal Brief Landlord-Tenant Law March 2014 PDF](https://www.creech.af.mil/Portals/111/Docs/AFD-140423-061.pdf)).
//! - **NRS § 118A.242 Security Deposit Cap — 3 Months' Rent**: the landlord may **NOT DEMAND OR RECEIVE A SECURITY DEPOSIT OR A SURETY BOND, OR A COMBINATION THEREOF, INCLUDING THE LAST MONTH'S RENT, WHOSE TOTAL AMOUNT OR VALUE EXCEEDS 3 MONTHS' PERIODIC RENT** — among the highest URLTA-state deposit caps.
//! - **NRS § 118A.242 Security Deposit Return — 30 Days**: the landlord shall provide the tenant with an **ITEMIZED, WRITTEN ACCOUNTING** of the disposition of the security deposit or surety bond, and **RETURN ANY REMAINING PORTION** of the security deposit to the tenant **NO LATER THAN 30 DAYS AFTER THE TERMINATION OF THE TENANCY**.
//! - **NRS § 40.2512 Nonpayment Notice — 7 Days**: the landlord must issue a **7-DAY NOTICE TO PAY OR QUIT** if a tenant fails to pay rent; this notice gives the tenant **SEVEN DAYS** to pay the overdue rent or vacate the property; this period is set at **SEVEN JUDICIAL DAYS** for real property.
//! - **NRS § 40.2516 Lease Violation Notice — 5 Days with 3-Judicial-Day Cure**: a tenant is guilty of **UNLAWFUL DETAINER** when the tenant continues in possession after a **NEGLECT OR FAILURE TO PERFORM ANY CONDITION OR COVENANT OF THE LEASE** and after notice in writing remains uncomplied with for **5 DAYS** after the service thereof; from the date the notice is served, the tenant has only **3 JUDICIAL (BUSINESS) DAYS to "CURE"** (correct) the lease violation — UNUSUAL short cure window.
//! - **NRS § 40.2516 Two-Notice Eviction Procedure**: to evict for lease violations, a landlord must serve a **FIVE-DAY NOTICE TO PERFORM LEASE CONDITION OR QUIT (NRS 40.2516)** and, if the tenant does not leave within the five-day notice period or cure the lease violation within five days, followed by a **FIVE-DAY NOTICE TO QUIT FOR UNLAWFUL DETAINER (NRS 40.254)**; both notices must be **SERVED BY A CONSTABLE, SHERIFF, LICENSED PROCESS SERVER, OR AN AGENT OF AN ATTORNEY LICENSED IN NEVADA**.
//! - **NRS § 118A.330 Entry Notice — 24 Hours**: landlords must provide **24 HOURS' NOTICE** before entering, and entry must be during **NORMAL BUSINESS HOURS** unless the tenant agrees otherwise.
//! - **NRS § 118A.290 Habitability**: the landlord shall **AT ALL TIMES DURING THE TENANCY MAINTAIN THE DWELLING UNIT IN A HABITABLE CONDITION**; a dwelling unit is not habitable if it violates provisions of housing or health codes concerning the **HEALTH, SAFETY, SANITATION OR FITNESS FOR HABITATION** of the dwelling unit or if it substantially lacks: (a) **EFFECTIVE WATERPROOFING AND WEATHER PROTECTION OF THE ROOF AND EXTERIOR WALLS, INCLUDING WINDOWS AND DOORS**; (b) **PLUMBING FACILITIES** which conformed to applicable law when installed and which are maintained in good working order.
//! - **NRS § 118A.250 Security Deposit Receipt**: the landlord must give the tenant a **SIGNED WRITTEN RECEIPT** for the security deposit or surety bond if the tenant requests it; if the tenant requests a receipt and the landlord refuses to give it, the **TENANT CAN STOP PAYING RENT** until the landlord provides the requested receipt.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const NV_CHAPTER_118A_NUMBER: u32 = 118;
pub const NV_CHAPTER_40_NUMBER: u32 = 40;
pub const NV_DEPOSIT_CAP_MONTHS_RENT: u32 = 3;
pub const NV_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const NV_NONPAYMENT_NOTICE_JUDICIAL_DAYS: u32 = 7;
pub const NV_LEASE_VIOLATION_NOTICE_DAYS: u32 = 5;
pub const NV_LEASE_VIOLATION_CURE_JUDICIAL_DAYS: u32 = 3;
pub const NV_UNLAWFUL_DETAINER_NOTICE_DAYS: u32 = 5;
pub const NV_ENTRY_NOTICE_HOURS: u32 = 24;
pub const NV_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromChapter118A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositReturnAndItemizedAccountingStatus {
    BothDepositReturnedAndItemizedAccountingProvidedWithin30Days,
    EitherNotReturnedOrItemizedAccountingNotProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseViolationCureStatus {
    TenantCuredLeaseViolationWithin3JudicialDays,
    TenantDidNotCureLeaseViolationWithin3JudicialDays,
    NoLeaseViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceMethodStatus {
    ServedByConstableSheriffLicensedProcessServerOrAttorneyAgent,
    NotServedByAuthorizedServiceMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    DepositCapThreeMonthsRentUnderSection118A242,
    SecurityDepositReturnUnderSection118A242,
    NonpaymentNoticeUnderSection402512,
    LeaseViolationNoticeUnderSection402516,
    EvictionNoticeServiceMethodUnderSection402516,
    LandlordEntryNoticeUnderSection118A330,
    LandlordHabitabilityObligationUnderSection118A290,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NvLandlordTenantMode {
    NotApplicableTenancyExemptFromChapter118A,
    CompliantDepositCapWithinThreeMonthsRent,
    CompliantBothDepositReturnedAndItemizedAccountingProvidedWithin30Days,
    CompliantSevenJudicialDayNonpaymentNoticeProperlyServed,
    CompliantFiveDayLeaseViolationNoticeWithTenantCure,
    CompliantFiveDayLeaseViolationNoticeWithoutTenantCure,
    CompliantNoLeaseViolation,
    CompliantServedByAuthorizedMethod,
    CompliantTwentyFourHourEntryNoticeDuringNormalBusinessHoursProperlyServed,
    CompliantLandlordMaintainsHabitabilityUnderSection118A290,
    ViolationDepositExceedsThreeMonthsRent,
    ViolationDepositReturnedPast30DayDeadlineOrItemizedAccountingNotProvided,
    ViolationNonpaymentNoticeShorterThan7JudicialDays,
    ViolationLeaseViolationNoticeShorterThan5Days,
    ViolationNotServedByAuthorizedMethod,
    ViolationEntryNoticeShorterThan24Hours,
    ViolationLandlordFailedHabitabilityObligation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub deposit_return_and_itemized_accounting_status: DepositReturnAndItemizedAccountingStatus,
    pub lease_violation_cure_status: LeaseViolationCureStatus,
    pub service_method_status: ServiceMethodStatus,
    pub compliance_aspect: ComplianceAspect,
    pub deposit_amount_in_months_rent: u32,
    pub days_to_return_deposit_and_itemized_accounting: u32,
    pub nonpayment_notice_judicial_days_given: u32,
    pub lease_violation_notice_days_given: u32,
    pub entry_notice_hours_given: u32,
    pub landlord_maintains_habitability: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: NvLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type NvLandlordTenantInput = Input;
pub type NvLandlordTenantOutput = Output;
pub type NvLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Nevada Landlord and Tenant: Dwellings — NRS Chapter 118A (substantive landlord-tenant relationship) + NRS Chapter 40 (eviction procedure); URLTA-based regime".to_string(),
        "NRS § 118A.242 Security Deposit Cap — 3 Months' Rent — the landlord may NOT DEMAND OR RECEIVE A SECURITY DEPOSIT OR A SURETY BOND, OR A COMBINATION THEREOF, INCLUDING THE LAST MONTH'S RENT, WHOSE TOTAL AMOUNT OR VALUE EXCEEDS 3 MONTHS' PERIODIC RENT — among the highest URLTA-state deposit caps".to_string(),
        "NRS § 118A.242 Security Deposit Return — 30 Days — landlord shall provide the tenant with an ITEMIZED, WRITTEN ACCOUNTING of the disposition of the security deposit or surety bond, and RETURN ANY REMAINING PORTION of the security deposit to the tenant NO LATER THAN 30 DAYS AFTER THE TERMINATION OF THE TENANCY".to_string(),
        "NRS § 40.2512 Nonpayment Notice — 7 Days — the landlord must issue a 7-DAY NOTICE TO PAY OR QUIT if a tenant fails to pay rent; this period is set at SEVEN JUDICIAL DAYS for real property".to_string(),
        "NRS § 40.2516 Lease Violation Notice — 5 Days with 3-Judicial-Day Cure — a tenant is guilty of UNLAWFUL DETAINER when the tenant continues in possession after a NEGLECT OR FAILURE TO PERFORM ANY CONDITION OR COVENANT OF THE LEASE and after notice in writing remains uncomplied with for 5 DAYS after the service thereof; from the date the notice is served, the tenant has only 3 JUDICIAL (BUSINESS) DAYS to CURE the lease violation".to_string(),
        "NRS § 40.2516 Two-Notice Eviction Procedure — to evict for lease violations, a landlord must serve a FIVE-DAY NOTICE TO PERFORM LEASE CONDITION OR QUIT (NRS 40.2516) and, if the tenant does not leave or cure within five days, followed by a FIVE-DAY NOTICE TO QUIT FOR UNLAWFUL DETAINER (NRS 40.254); both notices must be SERVED BY A CONSTABLE, SHERIFF, LICENSED PROCESS SERVER, OR AN AGENT OF AN ATTORNEY LICENSED IN NEVADA".to_string(),
        "NRS § 118A.330 Entry Notice — 24 Hours — landlords must provide 24 HOURS' NOTICE before entering, and entry must be during NORMAL BUSINESS HOURS unless the tenant agrees otherwise".to_string(),
        "NRS § 118A.290 Habitability — landlord shall AT ALL TIMES DURING THE TENANCY MAINTAIN THE DWELLING UNIT IN A HABITABLE CONDITION; a dwelling unit is not habitable if it violates provisions of housing or health codes concerning the HEALTH, SAFETY, SANITATION OR FITNESS FOR HABITATION or if it substantially lacks: (a) EFFECTIVE WATERPROOFING AND WEATHER PROTECTION OF THE ROOF AND EXTERIOR WALLS, INCLUDING WINDOWS AND DOORS; (b) PLUMBING FACILITIES which conformed to applicable law when installed and which are maintained in good working order".to_string(),
        "NRS § 118A.250 Security Deposit Receipt — landlord must give the tenant a SIGNED WRITTEN RECEIPT for the security deposit or surety bond if the tenant requests it; if the tenant requests a receipt and the landlord refuses to give it, the TENANT CAN STOP PAYING RENT until the landlord provides the requested receipt".to_string(),
        "Nevada Legislature + Justia + Nevada Public Law + Washoe County RJC + Civil Law Self-Help Center + Nevada Evictions + Tenant Rights + Reno's Property Management + Hemlane + Karsaz Law + Cirac Law + iPropertyManagement + LPSNV + LawServer + Clark County NV + Landlord-Tenant-Law + PayRent + Creech AFB — practitioner overviews of Nevada NRS Chapter 118A + Chapter 40 landlord-tenant regime".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromChapter118A {
        return Output {
            mode: NvLandlordTenantMode::NotApplicableTenancyExemptFromChapter118A,
            statutory_basis: "NRS Chapter 118A jurisdiction — tenancy exempt from landlord-tenant coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from Nevada NRS Chapter 118A; statutory landlord-tenant obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::DepositCapThreeMonthsRentUnderSection118A242 => {
            if input.deposit_amount_in_months_rent <= NV_DEPOSIT_CAP_MONTHS_RENT {
                Output {
                    mode: NvLandlordTenantMode::CompliantDepositCapWithinThreeMonthsRent,
                    statutory_basis: "NRS § 118A.242 — deposit within 3-month rent cap".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit equal to {m} month(s) of rent satisfies 3-month statutory cap under NRS § 118A.242 (deposit + surety bond + last month's rent combined cannot exceed 3 months' periodic rent).",
                        m = input.deposit_amount_in_months_rent,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: NvLandlordTenantMode::ViolationDepositExceedsThreeMonthsRent,
                    statutory_basis: "NRS § 118A.242 — deposit exceeds 3-month rent cap".to_string(),
                    notes: format!(
                        "VIOLATION: deposit equal to {m} month(s) of rent exceeds 3-month statutory cap under NRS § 118A.242.",
                        m = input.deposit_amount_in_months_rent,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::SecurityDepositReturnUnderSection118A242 => {
            if input.days_to_return_deposit_and_itemized_accounting
                <= NV_DEPOSIT_RETURN_DEADLINE_DAYS
                && input.deposit_return_and_itemized_accounting_status
                    == DepositReturnAndItemizedAccountingStatus::BothDepositReturnedAndItemizedAccountingProvidedWithin30Days
            {
                Output {
                    mode: NvLandlordTenantMode::CompliantBothDepositReturnedAndItemizedAccountingProvidedWithin30Days,
                    statutory_basis: "NRS § 118A.242 — deposit returned with itemized accounting within 30 days".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit returned with itemized written accounting at day {d} (within 30-day statutory window) under NRS § 118A.242.",
                        d = input.days_to_return_deposit_and_itemized_accounting,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: NvLandlordTenantMode::ViolationDepositReturnedPast30DayDeadlineOrItemizedAccountingNotProvided,
                    statutory_basis: "NRS § 118A.242 — deposit return exceeded 30-day window or itemized accounting not provided".to_string(),
                    notes: format!(
                        "VIOLATION: deposit returned at day {d} past 30-day statutory window OR itemized written accounting not provided under NRS § 118A.242.",
                        d = input.days_to_return_deposit_and_itemized_accounting,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::NonpaymentNoticeUnderSection402512 => {
            if input.nonpayment_notice_judicial_days_given >= NV_NONPAYMENT_NOTICE_JUDICIAL_DAYS {
                Output {
                    mode: NvLandlordTenantMode::CompliantSevenJudicialDayNonpaymentNoticeProperlyServed,
                    statutory_basis: "NRS § 40.2512 — 7-judicial-day nonpayment notice properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-judicial-day nonpayment notice satisfies 7-judicial-day statutory minimum under NRS § 40.2512.",
                        d = input.nonpayment_notice_judicial_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: NvLandlordTenantMode::ViolationNonpaymentNoticeShorterThan7JudicialDays,
                    statutory_basis: "NRS § 40.2512 — nonpayment notice shorter than 7-judicial-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-judicial-day nonpayment notice shorter than 7-judicial-day statutory minimum under NRS § 40.2512.",
                        d = input.nonpayment_notice_judicial_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::LeaseViolationNoticeUnderSection402516 => {
            if input.lease_violation_notice_days_given < NV_LEASE_VIOLATION_NOTICE_DAYS {
                return Output {
                    mode: NvLandlordTenantMode::ViolationLeaseViolationNoticeShorterThan5Days,
                    statutory_basis: "NRS § 40.2516 — lease violation notice shorter than 5-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day lease violation notice shorter than 5-day statutory minimum under NRS § 40.2516.",
                        d = input.lease_violation_notice_days_given,
                    ),
                    citations,
                };
            }
            match input.lease_violation_cure_status {
                LeaseViolationCureStatus::TenantCuredLeaseViolationWithin3JudicialDays => Output {
                    mode: NvLandlordTenantMode::CompliantFiveDayLeaseViolationNoticeWithTenantCure,
                    statutory_basis: "NRS § 40.2516 — tenant cured lease violation within 3 judicial days".to_string(),
                    notes: "COMPLIANT: tenant cured lease violation within 3 JUDICIAL (BUSINESS) DAYS cure window under NRS § 40.2516; tenancy continues; landlord need not proceed with unlawful detainer.".to_string(),
                    citations,
                },
                LeaseViolationCureStatus::TenantDidNotCureLeaseViolationWithin3JudicialDays => Output {
                    mode: NvLandlordTenantMode::CompliantFiveDayLeaseViolationNoticeWithoutTenantCure,
                    statutory_basis: "NRS § 40.2516 — 5-day notice served + tenant did not cure within 3 judicial days".to_string(),
                    notes: "COMPLIANT: 5-day lease violation notice properly served under NRS § 40.2516; tenant did not cure within 3-judicial-day cure window; landlord may proceed with five-day notice to quit for unlawful detainer under NRS § 40.254.".to_string(),
                    citations,
                },
                LeaseViolationCureStatus::NoLeaseViolation => Output {
                    mode: NvLandlordTenantMode::CompliantNoLeaseViolation,
                    statutory_basis: "NRS § 40.2516 — no lease violation".to_string(),
                    notes: "COMPLIANT: no lease violation condition present under NRS § 40.2516; eviction notice not triggered.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::EvictionNoticeServiceMethodUnderSection402516 => {
            match input.service_method_status {
                ServiceMethodStatus::ServedByConstableSheriffLicensedProcessServerOrAttorneyAgent => Output {
                    mode: NvLandlordTenantMode::CompliantServedByAuthorizedMethod,
                    statutory_basis: "NRS § 40.2516 — eviction notice served by authorized method".to_string(),
                    notes: "COMPLIANT: eviction notice served by CONSTABLE, SHERIFF, LICENSED PROCESS SERVER, OR AN AGENT OF AN ATTORNEY LICENSED IN NEVADA under NRS § 40.2516.".to_string(),
                    citations,
                },
                ServiceMethodStatus::NotServedByAuthorizedServiceMethod => Output {
                    mode: NvLandlordTenantMode::ViolationNotServedByAuthorizedMethod,
                    statutory_basis: "NRS § 40.2516 — eviction notice not served by authorized method".to_string(),
                    notes: "VIOLATION: eviction notice NOT served by CONSTABLE, SHERIFF, LICENSED PROCESS SERVER, OR AGENT OF NEVADA-LICENSED ATTORNEY under NRS § 40.2516; service is invalid; landlord cannot proceed with summary eviction until proper service obtained.".to_string(),
                    citations,
                },
            }
        }
        ComplianceAspect::LandlordEntryNoticeUnderSection118A330 => {
            if input.entry_notice_hours_given >= NV_ENTRY_NOTICE_HOURS {
                Output {
                    mode: NvLandlordTenantMode::CompliantTwentyFourHourEntryNoticeDuringNormalBusinessHoursProperlyServed,
                    statutory_basis: "NRS § 118A.330 — 24-hour entry notice during normal business hours properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {h}-hour entry notice satisfies 24-hour statutory minimum under NRS § 118A.330; entry must be during normal business hours unless tenant agrees otherwise.",
                        h = input.entry_notice_hours_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: NvLandlordTenantMode::ViolationEntryNoticeShorterThan24Hours,
                    statutory_basis: "NRS § 118A.330 — entry notice shorter than 24-hour statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {h}-hour entry notice shorter than 24-hour statutory minimum under NRS § 118A.330.",
                        h = input.entry_notice_hours_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordHabitabilityObligationUnderSection118A290 => {
            if input.landlord_maintains_habitability {
                Output {
                    mode: NvLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderSection118A290,
                    statutory_basis: "NRS § 118A.290 — landlord maintains habitability".to_string(),
                    notes: "COMPLIANT: landlord maintains habitability under NRS § 118A.290 (housing/health codes + effective waterproofing/weather protection of roof and exterior walls including windows and doors + plumbing facilities in good working order).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: NvLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation,
                    statutory_basis: "NRS § 118A.290 — landlord failed habitability obligation".to_string(),
                    notes: "VIOLATION: landlord failed one or more habitability obligations under NRS § 118A.290.".to_string(),
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
            deposit_return_and_itemized_accounting_status:
                DepositReturnAndItemizedAccountingStatus::BothDepositReturnedAndItemizedAccountingProvidedWithin30Days,
            lease_violation_cure_status:
                LeaseViolationCureStatus::TenantCuredLeaseViolationWithin3JudicialDays,
            service_method_status:
                ServiceMethodStatus::ServedByConstableSheriffLicensedProcessServerOrAttorneyAgent,
            compliance_aspect: ComplianceAspect::DepositCapThreeMonthsRentUnderSection118A242,
            deposit_amount_in_months_rent: 3,
            days_to_return_deposit_and_itemized_accounting: 25,
            nonpayment_notice_judicial_days_given: 7,
            lease_violation_notice_days_given: 5,
            entry_notice_hours_given: 24,
            landlord_maintains_habitability: true,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromChapter118A;
        let out = check(&input);
        assert_eq!(out.mode, NvLandlordTenantMode::NotApplicableTenancyExemptFromChapter118A);
    }

    #[test]
    fn deposit_at_3_month_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DepositCapThreeMonthsRentUnderSection118A242;
        input.deposit_amount_in_months_rent = 3;
        let out = check(&input);
        assert_eq!(out.mode, NvLandlordTenantMode::CompliantDepositCapWithinThreeMonthsRent);
    }

    #[test]
    fn deposit_exceeds_3_month_cap_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DepositCapThreeMonthsRentUnderSection118A242;
        input.deposit_amount_in_months_rent = 4;
        let out = check(&input);
        assert_eq!(out.mode, NvLandlordTenantMode::ViolationDepositExceedsThreeMonthsRent);
    }

    #[test]
    fn deposit_returned_at_30_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection118A242;
        input.days_to_return_deposit_and_itemized_accounting = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::CompliantBothDepositReturnedAndItemizedAccountingProvidedWithin30Days
        );
    }

    #[test]
    fn deposit_returned_at_31_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection118A242;
        input.days_to_return_deposit_and_itemized_accounting = 31;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::ViolationDepositReturnedPast30DayDeadlineOrItemizedAccountingNotProvided
        );
    }

    #[test]
    fn deposit_without_itemized_accounting_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderSection118A242;
        input.deposit_return_and_itemized_accounting_status =
            DepositReturnAndItemizedAccountingStatus::EitherNotReturnedOrItemizedAccountingNotProvided;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::ViolationDepositReturnedPast30DayDeadlineOrItemizedAccountingNotProvided
        );
    }

    #[test]
    fn seven_judicial_day_nonpayment_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderSection402512;
        input.nonpayment_notice_judicial_days_given = 7;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::CompliantSevenJudicialDayNonpaymentNoticeProperlyServed
        );
    }

    #[test]
    fn six_judicial_day_nonpayment_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderSection402512;
        input.nonpayment_notice_judicial_days_given = 6;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::ViolationNonpaymentNoticeShorterThan7JudicialDays
        );
    }

    #[test]
    fn lease_violation_5_day_notice_with_tenant_cure_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LeaseViolationNoticeUnderSection402516;
        input.lease_violation_notice_days_given = 5;
        input.lease_violation_cure_status =
            LeaseViolationCureStatus::TenantCuredLeaseViolationWithin3JudicialDays;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::CompliantFiveDayLeaseViolationNoticeWithTenantCure
        );
    }

    #[test]
    fn lease_violation_5_day_notice_without_tenant_cure_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LeaseViolationNoticeUnderSection402516;
        input.lease_violation_notice_days_given = 5;
        input.lease_violation_cure_status =
            LeaseViolationCureStatus::TenantDidNotCureLeaseViolationWithin3JudicialDays;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::CompliantFiveDayLeaseViolationNoticeWithoutTenantCure
        );
    }

    #[test]
    fn lease_violation_4_day_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LeaseViolationNoticeUnderSection402516;
        input.lease_violation_notice_days_given = 4;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::ViolationLeaseViolationNoticeShorterThan5Days
        );
    }

    #[test]
    fn served_by_authorized_method_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionNoticeServiceMethodUnderSection402516;
        input.service_method_status =
            ServiceMethodStatus::ServedByConstableSheriffLicensedProcessServerOrAttorneyAgent;
        let out = check(&input);
        assert_eq!(out.mode, NvLandlordTenantMode::CompliantServedByAuthorizedMethod);
    }

    #[test]
    fn not_served_by_authorized_method_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionNoticeServiceMethodUnderSection402516;
        input.service_method_status = ServiceMethodStatus::NotServedByAuthorizedServiceMethod;
        let out = check(&input);
        assert_eq!(out.mode, NvLandlordTenantMode::ViolationNotServedByAuthorizedMethod);
    }

    #[test]
    fn twenty_four_hour_entry_notice_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection118A330;
        input.entry_notice_hours_given = 24;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::CompliantTwentyFourHourEntryNoticeDuringNormalBusinessHoursProperlyServed
        );
    }

    #[test]
    fn twenty_three_hour_entry_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderSection118A330;
        input.entry_notice_hours_given = 23;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::ViolationEntryNoticeShorterThan24Hours
        );
    }

    #[test]
    fn landlord_maintains_habitability_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordHabitabilityObligationUnderSection118A290;
        input.landlord_maintains_habitability = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderSection118A290
        );
    }

    #[test]
    fn landlord_failed_habitability_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordHabitabilityObligationUnderSection118A290;
        input.landlord_maintains_habitability = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            NvLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation
        );
    }

    #[test]
    fn constants_pin_nevada_landlord_tenant_statutory_thresholds() {
        assert_eq!(NV_CHAPTER_118A_NUMBER, 118);
        assert_eq!(NV_CHAPTER_40_NUMBER, 40);
        assert_eq!(NV_DEPOSIT_CAP_MONTHS_RENT, 3);
        assert_eq!(NV_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(NV_NONPAYMENT_NOTICE_JUDICIAL_DAYS, 7);
        assert_eq!(NV_LEASE_VIOLATION_NOTICE_DAYS, 5);
        assert_eq!(NV_LEASE_VIOLATION_CURE_JUDICIAL_DAYS, 3);
        assert_eq!(NV_UNLAWFUL_DETAINER_NOTICE_DAYS, 5);
        assert_eq!(NV_ENTRY_NOTICE_HOURS, 24);
        assert_eq!(NV_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_nevada_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Nevada Landlord and Tenant"));
        assert!(joined.contains("NRS Chapter 118A"));
        assert!(joined.contains("NRS Chapter 40"));
        assert!(joined.contains("NRS § 118A.242"));
        assert!(joined.contains("EXCEEDS 3 MONTHS' PERIODIC RENT"));
        assert!(joined.contains("ITEMIZED, WRITTEN ACCOUNTING"));
        assert!(joined.contains("30 DAYS AFTER THE TERMINATION OF THE TENANCY"));
        assert!(joined.contains("NRS § 40.2512"));
        assert!(joined.contains("7-DAY NOTICE TO PAY OR QUIT"));
        assert!(joined.contains("SEVEN JUDICIAL DAYS"));
        assert!(joined.contains("NRS § 40.2516"));
        assert!(joined.contains("UNLAWFUL DETAINER"));
        assert!(joined.contains("NEGLECT OR FAILURE TO PERFORM ANY CONDITION OR COVENANT OF THE LEASE"));
        assert!(joined.contains("5 DAYS"));
        assert!(joined.contains("3 JUDICIAL (BUSINESS) DAYS"));
        assert!(joined.contains("FIVE-DAY NOTICE TO PERFORM LEASE CONDITION OR QUIT"));
        assert!(joined.contains("FIVE-DAY NOTICE TO QUIT FOR UNLAWFUL DETAINER"));
        assert!(joined.contains("CONSTABLE, SHERIFF, LICENSED PROCESS SERVER"));
        assert!(joined.contains("ATTORNEY LICENSED IN NEVADA"));
        assert!(joined.contains("NRS § 118A.330"));
        assert!(joined.contains("24 HOURS' NOTICE"));
        assert!(joined.contains("NORMAL BUSINESS HOURS"));
        assert!(joined.contains("NRS § 118A.290"));
        assert!(joined.contains("HABITABLE CONDITION"));
        assert!(joined.contains("HEALTH, SAFETY, SANITATION OR FITNESS FOR HABITATION"));
        assert!(joined.contains("EFFECTIVE WATERPROOFING AND WEATHER PROTECTION OF THE ROOF AND EXTERIOR WALLS"));
        assert!(joined.contains("PLUMBING FACILITIES"));
        assert!(joined.contains("NRS § 118A.250"));
        assert!(joined.contains("SIGNED WRITTEN RECEIPT"));
        assert!(joined.contains("TENANT CAN STOP PAYING RENT"));
    }
}
