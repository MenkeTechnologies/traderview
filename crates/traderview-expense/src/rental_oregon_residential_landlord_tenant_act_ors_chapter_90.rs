//! Oregon Residential Landlord and Tenant Act — ORS
//! Chapter 90 Compliance Module — pure-compute check for
//! landlord statutory compliance with Oregon's statewide
//! URLTA-based landlord-tenant regime spanning **ORS
//! §§ 90.100 through 90.875**.
//!
//! Companion to the existing
//! `rental_oregon_sb_608_sb_611_rent_stabilization`
//! module (iter 647) which covers SB 608 (2019) just-cause
//! eviction and statewide rent stabilization specifically.
//! Major recent reforms covered include **HB 2001 (2023)**
//! nonpayment notice procedural updates and **SB 1069
//! (2023)** rent and habitability amendments.
//!
//! Web research (verified 2026-06-03):
//! - **ORS Chapter 90 Residential Landlord and Tenant Act**: Oregon's statewide URLTA-based landlord-tenant regime; codified at ORS §§ 90.100 through 90.875 ([Oregon Legislature — Chapter 90 of Oregon Revised Statutes](https://www.oregonlegislature.gov/bills_laws/ors/ors090.html); [Oregon Public Law — ORS Chapter 90 Residential Landlord and Tenant](https://oregon.public.law/statutes/ors_chapter_90); [Justia — 2025 Oregon Revised Statutes Chapter 90](https://law.justia.com/codes/oregon/volume-03/chapter-090/); [SETA — Chapter 90 Oregon Landlord-Tenant Law](https://www.springfieldeugenetenantassociation.com/chapter_90); [The TLC Realty Team — Oregon Rental Laws Overview Cottage Grove](https://thetlcrealtyteam.com/oregon-landlord-tenant-law); [Oregon Public Law — ORS 90.394 Termination of Tenancy for Failure to Pay Rent](https://oregon.public.law/statutes/ors_90.394); [Oregon Realtors — Evictions Guidance](https://oregonrealtors.org/protect/evictions-guidance/); [Portland.gov — HB2001 Passed by Oregon Legislature](https://www.portland.gov/phb/rental-services/news/2023/3/29/hb2001-passed-oregon-legislature); [Oregon Public Law — ORS 90.630 Termination by Landlord; Causes; Notice; Cure; Repeated Nonpayment of Rent](https://oregon.public.law/statutes/ors_90.630); [FindLaw — ORS § 90.394](https://codes.findlaw.com/or/title-10-property-rights-and-transactions/or-rev-st-sect-90-394/); [Justia — ORS § 90.394 2025](https://law.justia.com/codes/oregon/volume-03/chapter-090/section-90-394/); [RocketRent — ORS § 90.394](https://rocketrent.com/landlord-tenant-laws/oregon/statutes/ors-%C2%A7-90-394/); [Oregon Courts — FED Instructions for Landlords January 2026 PDF](https://www.courts.oregon.gov/forms/Documents/FED-Inst-LL.pdf); [Oregon Legislature 2023R1 — SB 1069 A-Engrossed PDF](https://olis.oregonlegislature.gov/liz/2023R1/Downloads/MeasureDocument/SB1069/A-Engrossed)).
//! - **ORS § 90.300 Security Deposits — 31-Day Return**: security deposits must be returned within **31 DAYS** after termination of the tenancy and delivery of possession, with **ITEMIZED DEDUCTIONS** supported by photos, receipts, and inspection records; Oregon has **NO STATUTORY CAP** on security deposit amount.
//! - **ORS § 90.394 Termination of Tenancy for Failure to Pay Rent — Tiered Notice Structure**: HB 2001 (2023) updated the nonpayment notice procedural requirements; tenants in week-to-week tenancies receive different notice than tenants in other tenancies.
//! - **ORS § 90.394(2)(a) Week-to-Week Tenancy — 72-Hour Notice**: for **WEEK-TO-WEEK TENANCIES**, the landlord may deliver at least **72 HOURS' WRITTEN NOTICE** of nonpayment and the landlord's intention to terminate the rental agreement if rent is not paid within that period; notice given **NO SOONER THAN ON THE FIFTH DAY** of the rental period (including the first day rent is due).
//! - **ORS § 90.394(2)(b)(A) Other Tenancies — 10-Day Notice After Day 8**: for **ALL TENANCIES OTHER THAN WEEK-TO-WEEK**, the landlord may deliver at least **10 DAYS' WRITTEN NOTICE** of nonpayment given **NO SOONER THAN ON THE EIGHTH DAY** of the rental period (including the first day rent is due).
//! - **ORS § 90.394(2)(b)(B) Other Tenancies — 13-Day Notice After Day 5 Alternative**: for **ALL TENANCIES OTHER THAN WEEK-TO-WEEK**, the landlord may alternatively deliver at least **13 DAYS' WRITTEN NOTICE** of nonpayment given **NO SOONER THAN ON THE FIFTH DAY** of the rental period (including the first day rent is due).
//! - **ORS § 90.394(3) Notice Content Requirements**: the notice must specify the **AMOUNT OF RENT THAT MUST BE PAID** and the **DATE AND TIME** by which the tenant must pay the rent to cure the nonpayment.
//! - **ORS § 90.392 Termination for Cause — 30-Day Notice for Material Violation**: landlord may terminate for material violation of the rental agreement with at least **30 DAYS' WRITTEN NOTICE**; tenant has **14 DAYS** to cure most material violations.
//! - **ORS § 90.320 Landlord Obligation to Maintain**: landlord must (1) effect repairs to keep premises **HABITABLE**; (2) comply with **BUILDING AND HOUSING CODES** materially affecting health and safety; (3) provide and maintain **PLUMBING SYSTEMS** for adequate **HOT AND COLD RUNNING WATER**; (4) provide and maintain electrical, heating, ventilating, air conditioning, sanitary, and other facilities and appliances; (5) provide and maintain **GARBAGE RECEPTACLES**.
//! - **ORS § 90.322 Landlord or Tenant Right of Access — 24-Hour Notice**: landlord shall give tenant **AT LEAST 24 HOURS' ACTUAL NOTICE** of intent to enter and may enter only at reasonable times; exceptions for emergencies, abandonment, court orders, or with tenant consent.
//! - **ORS § 90.220 Terms and Conditions of Rental Agreement — Disclosures**: rental agreement must include certain disclosures including **NAME AND ADDRESS** of landlord OR person authorized to manage premises OR person authorized to act for landlord; landlord must furnish a written copy of the rental agreement.
//! - **ORS § 90.427 Termination of Tenancies Without Tenant Cause — SB 608 of 2019**: covered by separate `rental_oregon_sb_608_sb_611_rent_stabilization` module (iter 647); requires just-cause for terminations after first year of occupancy plus statewide rent stabilization (7% + CPI annual cap).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const OR_CHAPTER_NUMBER: u32 = 90;
pub const OR_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 31;
pub const OR_WEEK_TO_WEEK_NOTICE_HOURS: u32 = 72;
pub const OR_WEEK_TO_WEEK_NOTICE_EARLIEST_DAY: u32 = 5;
pub const OR_OTHER_TENANCY_10_DAY_NOTICE_DAYS: u32 = 10;
pub const OR_OTHER_TENANCY_10_DAY_NOTICE_EARLIEST_DAY: u32 = 8;
pub const OR_OTHER_TENANCY_13_DAY_NOTICE_DAYS: u32 = 13;
pub const OR_OTHER_TENANCY_13_DAY_NOTICE_EARLIEST_DAY: u32 = 5;
pub const OR_MATERIAL_VIOLATION_NOTICE_DAYS: u32 = 30;
pub const OR_MATERIAL_VIOLATION_CURE_DAYS: u32 = 14;
pub const OR_ENTRY_NOTICE_HOURS: u32 = 24;
pub const OR_HB_2001_ENACTMENT_YEAR: u32 = 2023;
pub const OR_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyCoverage {
    CoveredResidentialTenancy,
    ExemptFromChapter90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    WeekToWeek,
    MonthToMonthOrFixedTerm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NonpaymentNoticeVariant {
    SeventyTwoHourForWeekToWeek,
    TenDayAfterDay8ForOtherTenancies,
    ThirteenDayAfterDay5ForOtherTenancies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryNoticeStatus {
    LandlordGave24HourActualNotice,
    LandlordEnteredForEmergencyOrAbandonment,
    LandlordEnteredWithoutNoticeAndNotEmergency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositReturnUnderOrs90300,
    NonpaymentNoticeUnderOrs90394,
    MaterialViolationNoticeUnderOrs90392,
    LandlordHabitabilityObligationUnderOrs90320,
    LandlordEntryNoticeUnderOrs90322,
    RentalAgreementDisclosuresUnderOrs90220,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrLandlordTenantMode {
    NotApplicableTenancyExemptFromChapter90,
    CompliantDepositReturnedWithItemizedDeductionsWithin31Days,
    CompliantWeekToWeek72HourNoticeProperlyServed,
    CompliantOtherTenancy10DayAfterDay8NoticeProperlyServed,
    CompliantOtherTenancy13DayAfterDay5NoticeProperlyServed,
    CompliantMaterialViolation30DayNoticeWith14DayCureProperlyServed,
    CompliantLandlordMaintainsHabitabilityUnderOrs90320,
    CompliantLandlordGave24HourEntryNotice,
    CompliantEmergencyOrAbandonmentEntryWithoutNotice,
    CompliantRentalAgreementDisclosuresProvided,
    ViolationDepositReturnedPast31DayDeadline,
    ViolationWeekToWeekNoticeShorterThan72Hours,
    ViolationOtherTenancyNoticeShorterThan10DaysOr13DaysAfterEarliestPermittedDay,
    ViolationMaterialViolationNoticeShorterThan30Days,
    ViolationLandlordFailedHabitabilityObligation,
    ViolationLandlordEnteredWithoutNoticeAndNotEmergency,
    ViolationRentalAgreementDisclosuresOmitted,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_coverage: TenancyCoverage,
    pub tenancy_type: TenancyType,
    pub nonpayment_notice_variant: NonpaymentNoticeVariant,
    pub entry_notice_status: EntryNoticeStatus,
    pub compliance_aspect: ComplianceAspect,
    pub days_to_return_deposit: u32,
    pub itemized_deductions_provided: bool,
    pub nonpayment_notice_hours_given: u32,
    pub nonpayment_notice_earliest_day_served: u32,
    pub material_violation_notice_days_given: u32,
    pub landlord_maintains_habitability: bool,
    pub entry_notice_hours_given: u32,
    pub rental_agreement_disclosures_provided: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: OrLandlordTenantMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type OrLandlordTenantInput = Input;
pub type OrLandlordTenantOutput = Output;
pub type OrLandlordTenantResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Oregon Residential Landlord and Tenant Act — ORS Chapter 90; statewide URLTA-based regime codified at ORS §§ 90.100 through 90.875".to_string(),
        "ORS § 90.300 Security Deposits — 31-Day Return — security deposits must be returned within 31 DAYS after termination of the tenancy and delivery of possession, with ITEMIZED DEDUCTIONS supported by photos, receipts, and inspection records; Oregon has NO STATUTORY CAP on security deposit amount".to_string(),
        "ORS § 90.394 Termination of Tenancy for Failure to Pay Rent — Tiered Notice Structure — HB 2001 (2023) updated the nonpayment notice procedural requirements".to_string(),
        "ORS § 90.394(2)(a) Week-to-Week Tenancy — 72-Hour Notice — for WEEK-TO-WEEK TENANCIES, the landlord may deliver at least 72 HOURS' WRITTEN NOTICE of nonpayment; notice given NO SOONER THAN ON THE FIFTH DAY of the rental period (including the first day rent is due)".to_string(),
        "ORS § 90.394(2)(b)(A) Other Tenancies — 10-Day Notice After Day 8 — for ALL TENANCIES OTHER THAN WEEK-TO-WEEK, the landlord may deliver at least 10 DAYS' WRITTEN NOTICE of nonpayment given NO SOONER THAN ON THE EIGHTH DAY of the rental period".to_string(),
        "ORS § 90.394(2)(b)(B) Other Tenancies — 13-Day Notice After Day 5 Alternative — for ALL TENANCIES OTHER THAN WEEK-TO-WEEK, the landlord may alternatively deliver at least 13 DAYS' WRITTEN NOTICE of nonpayment given NO SOONER THAN ON THE FIFTH DAY of the rental period".to_string(),
        "ORS § 90.394(3) Notice Content Requirements — the notice must specify the AMOUNT OF RENT THAT MUST BE PAID and the DATE AND TIME by which the tenant must pay the rent to cure the nonpayment".to_string(),
        "ORS § 90.392 Termination for Cause — 30-Day Notice for Material Violation — landlord may terminate for material violation of the rental agreement with at least 30 DAYS' WRITTEN NOTICE; tenant has 14 DAYS to cure most material violations".to_string(),
        "ORS § 90.320 Landlord Obligation to Maintain — landlord must (1) effect repairs to keep premises HABITABLE; (2) comply with BUILDING AND HOUSING CODES materially affecting health and safety; (3) provide and maintain PLUMBING SYSTEMS for adequate HOT AND COLD RUNNING WATER; (4) provide and maintain electrical, heating, ventilating, air conditioning, sanitary, and other facilities and appliances; (5) provide and maintain GARBAGE RECEPTACLES".to_string(),
        "ORS § 90.322 Landlord or Tenant Right of Access — 24-Hour Notice — landlord shall give tenant AT LEAST 24 HOURS' ACTUAL NOTICE of intent to enter and may enter only at reasonable times; exceptions for emergencies, abandonment, court orders, or with tenant consent".to_string(),
        "ORS § 90.220 Terms and Conditions of Rental Agreement — Disclosures — rental agreement must include certain disclosures including NAME AND ADDRESS of landlord OR person authorized to manage premises OR person authorized to act for landlord; landlord must furnish a written copy of the rental agreement".to_string(),
        "ORS § 90.427 Termination of Tenancies Without Tenant Cause — SB 608 of 2019 — covered by separate rental_oregon_sb_608_sb_611_rent_stabilization module".to_string(),
        "Oregon Legislature + Oregon Public Law + Justia + SETA + The TLC Realty Team + Oregon Realtors + Portland.gov + FindLaw + RocketRent + Oregon Courts — practitioner overviews of ORS Chapter 90".to_string(),
    ];

    if input.tenancy_coverage == TenancyCoverage::ExemptFromChapter90 {
        return Output {
            mode: OrLandlordTenantMode::NotApplicableTenancyExemptFromChapter90,
            statutory_basis: "ORS Chapter 90 jurisdiction — tenancy exempt from Chapter 90 coverage".to_string(),
            notes: "NOT APPLICABLE: tenancy exempt from ORS Chapter 90; Oregon Residential Landlord and Tenant Act obligations unavailable.".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositReturnUnderOrs90300 => {
            if input.days_to_return_deposit <= OR_DEPOSIT_RETURN_DEADLINE_DAYS
                && input.itemized_deductions_provided
            {
                Output {
                    mode: OrLandlordTenantMode::CompliantDepositReturnedWithItemizedDeductionsWithin31Days,
                    statutory_basis: "ORS § 90.300 — deposit returned with itemized deductions within 31 days".to_string(),
                    notes: format!(
                        "COMPLIANT: deposit returned with itemized deductions at day {d} (within 31-day statutory window) under ORS § 90.300.",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: OrLandlordTenantMode::ViolationDepositReturnedPast31DayDeadline,
                    statutory_basis: "ORS § 90.300 — deposit return exceeded 31-day statutory window or itemized deductions omitted".to_string(),
                    notes: format!(
                        "VIOLATION: deposit returned at day {d} past 31-day window or itemized deductions omitted under ORS § 90.300.",
                        d = input.days_to_return_deposit,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::NonpaymentNoticeUnderOrs90394 => match input.nonpayment_notice_variant {
            NonpaymentNoticeVariant::SeventyTwoHourForWeekToWeek => {
                if input.tenancy_type != TenancyType::WeekToWeek {
                    Output {
                        mode: OrLandlordTenantMode::ViolationOtherTenancyNoticeShorterThan10DaysOr13DaysAfterEarliestPermittedDay,
                        statutory_basis: "ORS § 90.394(2)(a) — 72-hour notice only available for week-to-week tenancies".to_string(),
                        notes: "VIOLATION: 72-hour notice variant used for non-week-to-week tenancy under ORS § 90.394(2)(a); must use 10-day-after-day-8 or 13-day-after-day-5 variant.".to_string(),
                        citations,
                    }
                } else if input.nonpayment_notice_hours_given >= OR_WEEK_TO_WEEK_NOTICE_HOURS
                    && input.nonpayment_notice_earliest_day_served
                        >= OR_WEEK_TO_WEEK_NOTICE_EARLIEST_DAY
                {
                    Output {
                        mode: OrLandlordTenantMode::CompliantWeekToWeek72HourNoticeProperlyServed,
                        statutory_basis: "ORS § 90.394(2)(a) — 72-hour week-to-week nonpayment notice properly served".to_string(),
                        notes: format!(
                            "COMPLIANT: {h}-hour week-to-week nonpayment notice served on day {d} (≥ 72-hour minimum + no earlier than day 5) under ORS § 90.394(2)(a).",
                            h = input.nonpayment_notice_hours_given,
                            d = input.nonpayment_notice_earliest_day_served,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: OrLandlordTenantMode::ViolationWeekToWeekNoticeShorterThan72Hours,
                        statutory_basis: "ORS § 90.394(2)(a) — week-to-week notice shorter than 72 hours or served before day 5".to_string(),
                        notes: format!(
                            "VIOLATION: {h}-hour week-to-week notice served on day {d} (shorter than 72-hour minimum or earlier than day 5) under ORS § 90.394(2)(a).",
                            h = input.nonpayment_notice_hours_given,
                            d = input.nonpayment_notice_earliest_day_served,
                        ),
                        citations,
                    }
                }
            }
            NonpaymentNoticeVariant::TenDayAfterDay8ForOtherTenancies => {
                let notice_days = input.nonpayment_notice_hours_given / 24;
                if input.tenancy_type == TenancyType::WeekToWeek {
                    Output {
                        mode: OrLandlordTenantMode::ViolationOtherTenancyNoticeShorterThan10DaysOr13DaysAfterEarliestPermittedDay,
                        statutory_basis: "ORS § 90.394(2)(b) — 10-day notice not available for week-to-week tenancies".to_string(),
                        notes: "VIOLATION: 10-day-after-day-8 notice variant used for week-to-week tenancy; must use 72-hour week-to-week variant.".to_string(),
                        citations,
                    }
                } else if notice_days >= OR_OTHER_TENANCY_10_DAY_NOTICE_DAYS
                    && input.nonpayment_notice_earliest_day_served
                        >= OR_OTHER_TENANCY_10_DAY_NOTICE_EARLIEST_DAY
                {
                    Output {
                        mode: OrLandlordTenantMode::CompliantOtherTenancy10DayAfterDay8NoticeProperlyServed,
                        statutory_basis: "ORS § 90.394(2)(b)(A) — 10-day-after-day-8 nonpayment notice properly served".to_string(),
                        notes: format!(
                            "COMPLIANT: {nd}-day notice served on day {d} (≥ 10-day minimum + no earlier than day 8) under ORS § 90.394(2)(b)(A).",
                            nd = notice_days,
                            d = input.nonpayment_notice_earliest_day_served,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: OrLandlordTenantMode::ViolationOtherTenancyNoticeShorterThan10DaysOr13DaysAfterEarliestPermittedDay,
                        statutory_basis: "ORS § 90.394(2)(b)(A) — notice shorter than 10 days or served before day 8".to_string(),
                        notes: format!(
                            "VIOLATION: {nd}-day notice served on day {d} (shorter than 10-day minimum or earlier than day 8) under ORS § 90.394(2)(b)(A).",
                            nd = notice_days,
                            d = input.nonpayment_notice_earliest_day_served,
                        ),
                        citations,
                    }
                }
            }
            NonpaymentNoticeVariant::ThirteenDayAfterDay5ForOtherTenancies => {
                let notice_days = input.nonpayment_notice_hours_given / 24;
                if input.tenancy_type == TenancyType::WeekToWeek {
                    Output {
                        mode: OrLandlordTenantMode::ViolationOtherTenancyNoticeShorterThan10DaysOr13DaysAfterEarliestPermittedDay,
                        statutory_basis: "ORS § 90.394(2)(b) — 13-day notice not available for week-to-week tenancies".to_string(),
                        notes: "VIOLATION: 13-day-after-day-5 notice variant used for week-to-week tenancy; must use 72-hour week-to-week variant.".to_string(),
                        citations,
                    }
                } else if notice_days >= OR_OTHER_TENANCY_13_DAY_NOTICE_DAYS
                    && input.nonpayment_notice_earliest_day_served
                        >= OR_OTHER_TENANCY_13_DAY_NOTICE_EARLIEST_DAY
                {
                    Output {
                        mode: OrLandlordTenantMode::CompliantOtherTenancy13DayAfterDay5NoticeProperlyServed,
                        statutory_basis: "ORS § 90.394(2)(b)(B) — 13-day-after-day-5 nonpayment notice properly served".to_string(),
                        notes: format!(
                            "COMPLIANT: {nd}-day notice served on day {d} (≥ 13-day minimum + no earlier than day 5) under ORS § 90.394(2)(b)(B).",
                            nd = notice_days,
                            d = input.nonpayment_notice_earliest_day_served,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: OrLandlordTenantMode::ViolationOtherTenancyNoticeShorterThan10DaysOr13DaysAfterEarliestPermittedDay,
                        statutory_basis: "ORS § 90.394(2)(b)(B) — notice shorter than 13 days or served before day 5".to_string(),
                        notes: format!(
                            "VIOLATION: {nd}-day notice served on day {d} (shorter than 13-day minimum or earlier than day 5) under ORS § 90.394(2)(b)(B).",
                            nd = notice_days,
                            d = input.nonpayment_notice_earliest_day_served,
                        ),
                        citations,
                    }
                }
            }
        },
        ComplianceAspect::MaterialViolationNoticeUnderOrs90392 => {
            if input.material_violation_notice_days_given >= OR_MATERIAL_VIOLATION_NOTICE_DAYS {
                Output {
                    mode: OrLandlordTenantMode::CompliantMaterialViolation30DayNoticeWith14DayCureProperlyServed,
                    statutory_basis: "ORS § 90.392 — 30-day material violation notice with 14-day cure period properly served".to_string(),
                    notes: format!(
                        "COMPLIANT: {d}-day material violation notice satisfies 30-day statutory minimum under ORS § 90.392; tenant has 14 days to cure most material violations.",
                        d = input.material_violation_notice_days_given,
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: OrLandlordTenantMode::ViolationMaterialViolationNoticeShorterThan30Days,
                    statutory_basis: "ORS § 90.392 — material violation notice shorter than 30-day statutory minimum".to_string(),
                    notes: format!(
                        "VIOLATION: {d}-day material violation notice shorter than 30-day statutory minimum under ORS § 90.392.",
                        d = input.material_violation_notice_days_given,
                    ),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordHabitabilityObligationUnderOrs90320 => {
            if input.landlord_maintains_habitability {
                Output {
                    mode: OrLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderOrs90320,
                    statutory_basis: "ORS § 90.320 — landlord maintains habitability obligations".to_string(),
                    notes: "COMPLIANT: landlord maintains habitability under ORS § 90.320 (repairs + building/housing code compliance + plumbing/hot and cold water + electrical/heating/sanitary systems + garbage receptacles).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: OrLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation,
                    statutory_basis: "ORS § 90.320 — landlord failed habitability obligation".to_string(),
                    notes: "VIOLATION: landlord failed one or more habitability obligations under ORS § 90.320.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordEntryNoticeUnderOrs90322 => match input.entry_notice_status {
            EntryNoticeStatus::LandlordGave24HourActualNotice => {
                if input.entry_notice_hours_given >= OR_ENTRY_NOTICE_HOURS {
                    Output {
                        mode: OrLandlordTenantMode::CompliantLandlordGave24HourEntryNotice,
                        statutory_basis: "ORS § 90.322 — landlord gave 24-hour actual notice of intent to enter".to_string(),
                        notes: format!(
                            "COMPLIANT: landlord gave {h}-hour actual notice (≥ 24-hour minimum) under ORS § 90.322.",
                            h = input.entry_notice_hours_given,
                        ),
                        citations,
                    }
                } else {
                    Output {
                        mode: OrLandlordTenantMode::ViolationLandlordEnteredWithoutNoticeAndNotEmergency,
                        statutory_basis: "ORS § 90.322 — landlord entry notice shorter than 24 hours".to_string(),
                        notes: format!(
                            "VIOLATION: landlord gave only {h}-hour notice (shorter than 24-hour minimum) under ORS § 90.322.",
                            h = input.entry_notice_hours_given,
                        ),
                        citations,
                    }
                }
            }
            EntryNoticeStatus::LandlordEnteredForEmergencyOrAbandonment => Output {
                mode: OrLandlordTenantMode::CompliantEmergencyOrAbandonmentEntryWithoutNotice,
                statutory_basis: "ORS § 90.322 — emergency/abandonment exception applies".to_string(),
                notes: "COMPLIANT: landlord entered for emergency, abandonment, court order, or with tenant consent under ORS § 90.322 exceptions.".to_string(),
                citations,
            },
            EntryNoticeStatus::LandlordEnteredWithoutNoticeAndNotEmergency => Output {
                mode: OrLandlordTenantMode::ViolationLandlordEnteredWithoutNoticeAndNotEmergency,
                statutory_basis: "ORS § 90.322 — landlord entered without notice and no emergency exception".to_string(),
                notes: "VIOLATION: landlord entered without 24-hour notice and no emergency/abandonment/court order/consent exception applies under ORS § 90.322.".to_string(),
                citations,
            },
        },
        ComplianceAspect::RentalAgreementDisclosuresUnderOrs90220 => {
            if input.rental_agreement_disclosures_provided {
                Output {
                    mode: OrLandlordTenantMode::CompliantRentalAgreementDisclosuresProvided,
                    statutory_basis: "ORS § 90.220 — rental agreement disclosures provided".to_string(),
                    notes: "COMPLIANT: rental agreement includes required disclosures (name and address of landlord OR person authorized to manage premises OR person authorized to act for landlord) under ORS § 90.220.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: OrLandlordTenantMode::ViolationRentalAgreementDisclosuresOmitted,
                    statutory_basis: "ORS § 90.220 — required rental agreement disclosures omitted".to_string(),
                    notes: "VIOLATION: required rental agreement disclosures omitted under ORS § 90.220.".to_string(),
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
            tenancy_type: TenancyType::MonthToMonthOrFixedTerm,
            nonpayment_notice_variant: NonpaymentNoticeVariant::TenDayAfterDay8ForOtherTenancies,
            entry_notice_status: EntryNoticeStatus::LandlordGave24HourActualNotice,
            compliance_aspect: ComplianceAspect::SecurityDepositReturnUnderOrs90300,
            days_to_return_deposit: 25,
            itemized_deductions_provided: true,
            nonpayment_notice_hours_given: 240,
            nonpayment_notice_earliest_day_served: 8,
            material_violation_notice_days_given: 30,
            landlord_maintains_habitability: true,
            entry_notice_hours_given: 24,
            rental_agreement_disclosures_provided: true,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_coverage = TenancyCoverage::ExemptFromChapter90;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::NotApplicableTenancyExemptFromChapter90
        );
    }

    #[test]
    fn deposit_returned_within_31_days_with_itemized_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderOrs90300;
        input.days_to_return_deposit = 31;
        input.itemized_deductions_provided = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::CompliantDepositReturnedWithItemizedDeductionsWithin31Days
        );
    }

    #[test]
    fn deposit_returned_at_32_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderOrs90300;
        input.days_to_return_deposit = 32;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::ViolationDepositReturnedPast31DayDeadline
        );
    }

    #[test]
    fn deposit_without_itemized_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnUnderOrs90300;
        input.days_to_return_deposit = 25;
        input.itemized_deductions_provided = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::ViolationDepositReturnedPast31DayDeadline
        );
    }

    #[test]
    fn week_to_week_72_hour_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderOrs90394;
        input.tenancy_type = TenancyType::WeekToWeek;
        input.nonpayment_notice_variant = NonpaymentNoticeVariant::SeventyTwoHourForWeekToWeek;
        input.nonpayment_notice_hours_given = 72;
        input.nonpayment_notice_earliest_day_served = 5;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::CompliantWeekToWeek72HourNoticeProperlyServed
        );
    }

    #[test]
    fn week_to_week_71_hour_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderOrs90394;
        input.tenancy_type = TenancyType::WeekToWeek;
        input.nonpayment_notice_variant = NonpaymentNoticeVariant::SeventyTwoHourForWeekToWeek;
        input.nonpayment_notice_hours_given = 71;
        input.nonpayment_notice_earliest_day_served = 5;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::ViolationWeekToWeekNoticeShorterThan72Hours
        );
    }

    #[test]
    fn week_to_week_notice_served_too_early_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderOrs90394;
        input.tenancy_type = TenancyType::WeekToWeek;
        input.nonpayment_notice_variant = NonpaymentNoticeVariant::SeventyTwoHourForWeekToWeek;
        input.nonpayment_notice_hours_given = 72;
        input.nonpayment_notice_earliest_day_served = 4;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::ViolationWeekToWeekNoticeShorterThan72Hours
        );
    }

    #[test]
    fn other_tenancy_10_day_after_day_8_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderOrs90394;
        input.tenancy_type = TenancyType::MonthToMonthOrFixedTerm;
        input.nonpayment_notice_variant = NonpaymentNoticeVariant::TenDayAfterDay8ForOtherTenancies;
        input.nonpayment_notice_hours_given = 240;
        input.nonpayment_notice_earliest_day_served = 8;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::CompliantOtherTenancy10DayAfterDay8NoticeProperlyServed
        );
    }

    #[test]
    fn other_tenancy_13_day_after_day_5_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderOrs90394;
        input.tenancy_type = TenancyType::MonthToMonthOrFixedTerm;
        input.nonpayment_notice_variant =
            NonpaymentNoticeVariant::ThirteenDayAfterDay5ForOtherTenancies;
        input.nonpayment_notice_hours_given = 312;
        input.nonpayment_notice_earliest_day_served = 5;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::CompliantOtherTenancy13DayAfterDay5NoticeProperlyServed
        );
    }

    #[test]
    fn other_tenancy_10_day_served_too_early_day_7_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderOrs90394;
        input.tenancy_type = TenancyType::MonthToMonthOrFixedTerm;
        input.nonpayment_notice_variant = NonpaymentNoticeVariant::TenDayAfterDay8ForOtherTenancies;
        input.nonpayment_notice_hours_given = 240;
        input.nonpayment_notice_earliest_day_served = 7;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::ViolationOtherTenancyNoticeShorterThan10DaysOr13DaysAfterEarliestPermittedDay
        );
    }

    #[test]
    fn week_to_week_using_10_day_variant_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderOrs90394;
        input.tenancy_type = TenancyType::WeekToWeek;
        input.nonpayment_notice_variant = NonpaymentNoticeVariant::TenDayAfterDay8ForOtherTenancies;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::ViolationOtherTenancyNoticeShorterThan10DaysOr13DaysAfterEarliestPermittedDay
        );
    }

    #[test]
    fn other_tenancy_using_72_hour_variant_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentNoticeUnderOrs90394;
        input.tenancy_type = TenancyType::MonthToMonthOrFixedTerm;
        input.nonpayment_notice_variant = NonpaymentNoticeVariant::SeventyTwoHourForWeekToWeek;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::ViolationOtherTenancyNoticeShorterThan10DaysOr13DaysAfterEarliestPermittedDay
        );
    }

    #[test]
    fn material_violation_30_day_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaterialViolationNoticeUnderOrs90392;
        input.material_violation_notice_days_given = 30;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::CompliantMaterialViolation30DayNoticeWith14DayCureProperlyServed
        );
    }

    #[test]
    fn material_violation_29_day_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaterialViolationNoticeUnderOrs90392;
        input.material_violation_notice_days_given = 29;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::ViolationMaterialViolationNoticeShorterThan30Days
        );
    }

    #[test]
    fn landlord_maintains_habitability_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordHabitabilityObligationUnderOrs90320;
        input.landlord_maintains_habitability = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::CompliantLandlordMaintainsHabitabilityUnderOrs90320
        );
    }

    #[test]
    fn landlord_failed_habitability_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordHabitabilityObligationUnderOrs90320;
        input.landlord_maintains_habitability = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::ViolationLandlordFailedHabitabilityObligation
        );
    }

    #[test]
    fn landlord_gave_24_hour_entry_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderOrs90322;
        input.entry_notice_status = EntryNoticeStatus::LandlordGave24HourActualNotice;
        input.entry_notice_hours_given = 24;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::CompliantLandlordGave24HourEntryNotice
        );
    }

    #[test]
    fn landlord_emergency_entry_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderOrs90322;
        input.entry_notice_status = EntryNoticeStatus::LandlordEnteredForEmergencyOrAbandonment;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::CompliantEmergencyOrAbandonmentEntryWithoutNotice
        );
    }

    #[test]
    fn landlord_entered_without_notice_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNoticeUnderOrs90322;
        input.entry_notice_status = EntryNoticeStatus::LandlordEnteredWithoutNoticeAndNotEmergency;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::ViolationLandlordEnteredWithoutNoticeAndNotEmergency
        );
    }

    #[test]
    fn rental_agreement_disclosures_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentalAgreementDisclosuresUnderOrs90220;
        input.rental_agreement_disclosures_provided = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::CompliantRentalAgreementDisclosuresProvided
        );
    }

    #[test]
    fn rental_agreement_disclosures_omitted_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentalAgreementDisclosuresUnderOrs90220;
        input.rental_agreement_disclosures_provided = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            OrLandlordTenantMode::ViolationRentalAgreementDisclosuresOmitted
        );
    }

    #[test]
    fn constants_pin_oregon_landlord_tenant_statutory_thresholds() {
        assert_eq!(OR_CHAPTER_NUMBER, 90);
        assert_eq!(OR_DEPOSIT_RETURN_DEADLINE_DAYS, 31);
        assert_eq!(OR_WEEK_TO_WEEK_NOTICE_HOURS, 72);
        assert_eq!(OR_WEEK_TO_WEEK_NOTICE_EARLIEST_DAY, 5);
        assert_eq!(OR_OTHER_TENANCY_10_DAY_NOTICE_DAYS, 10);
        assert_eq!(OR_OTHER_TENANCY_10_DAY_NOTICE_EARLIEST_DAY, 8);
        assert_eq!(OR_OTHER_TENANCY_13_DAY_NOTICE_DAYS, 13);
        assert_eq!(OR_OTHER_TENANCY_13_DAY_NOTICE_EARLIEST_DAY, 5);
        assert_eq!(OR_MATERIAL_VIOLATION_NOTICE_DAYS, 30);
        assert_eq!(OR_MATERIAL_VIOLATION_CURE_DAYS, 14);
        assert_eq!(OR_ENTRY_NOTICE_HOURS, 24);
        assert_eq!(OR_HB_2001_ENACTMENT_YEAR, 2023);
        assert_eq!(OR_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_oregon_landlord_tenant_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Oregon Residential Landlord and Tenant Act"));
        assert!(joined.contains("ORS Chapter 90"));
        assert!(joined.contains("ORS § 90.300"));
        assert!(joined.contains("31 DAYS"));
        assert!(joined.contains("ITEMIZED DEDUCTIONS"));
        assert!(joined.contains("NO STATUTORY CAP"));
        assert!(joined.contains("ORS § 90.394"));
        assert!(joined.contains("HB 2001"));
        assert!(joined.contains("WEEK-TO-WEEK TENANCIES"));
        assert!(joined.contains("72 HOURS' WRITTEN NOTICE"));
        assert!(joined.contains("FIFTH DAY"));
        assert!(joined.contains("10 DAYS' WRITTEN NOTICE"));
        assert!(joined.contains("EIGHTH DAY"));
        assert!(joined.contains("13 DAYS' WRITTEN NOTICE"));
        assert!(joined.contains("AMOUNT OF RENT THAT MUST BE PAID"));
        assert!(joined.contains("ORS § 90.392"));
        assert!(joined.contains("30 DAYS' WRITTEN NOTICE"));
        assert!(joined.contains("14 DAYS to cure"));
        assert!(joined.contains("ORS § 90.320"));
        assert!(joined.contains("HABITABLE"));
        assert!(joined.contains("BUILDING AND HOUSING CODES"));
        assert!(joined.contains("HOT AND COLD RUNNING WATER"));
        assert!(joined.contains("GARBAGE RECEPTACLES"));
        assert!(joined.contains("ORS § 90.322"));
        assert!(joined.contains("24 HOURS' ACTUAL NOTICE"));
        assert!(joined.contains("ORS § 90.220"));
        assert!(joined.contains("NAME AND ADDRESS"));
        assert!(joined.contains("ORS § 90.427"));
        assert!(joined.contains("rental_oregon_sb_608_sb_611_rent_stabilization"));
    }
}
