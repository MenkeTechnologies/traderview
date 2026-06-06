//! Virginia Residential Landlord and Tenant Act (VRLTA)
//! Compliance Module — Va. Code §§ 55.1-1200 through
//! 55.1-1262. Pure-compute check for trader-landlord
//! compliance with the comprehensive Virginia statewide
//! residential tenancy regime, originally enacted by the
//! Virginia General Assembly in 1974 and recodified by the
//! 2019 amendments into Title 55.1 effective **OCTOBER 1,
//! 2019**.
//!
//! VRLTA governs essentially ALL residential rental
//! relationships in Virginia after the 2019 amendments
//! eliminated the pre-2019 small-landlord (≤ 4 units)
//! exemption. Critical for trader-landlord operators of any
//! Virginia residential rental property — single-family,
//! multifamily, condo, manufactured home — and pre-empts
//! locality-level rent control under Va. Code § 36-93.
//!
//! Web research (verified 2026-06-03):
//! - **Original Enactment**: Virginia Residential Landlord and Tenant Act enacted by the Virginia General Assembly in **1974**; consolidated into Title 55.1 Chapter 12 by 2019 General Assembly with effective date **OCTOBER 1, 2019**; further amendments through 2025 General Assembly sessions ([Virginia Law Library — Code of Virginia Popular Names: Virginia Residential Landlord and Tenant Act](https://law.lis.virginia.gov/vacodepopularnames/virginia-residential-landlord-and-tenant-act/); [Virginia Law Library — § 55.1-1200 Definitions (Effective October 1, 2019)](https://law.lis.virginia.gov/vacode/title55.1/chapter12/section55.1-1200/); [Virginia Law Library — Code of Virginia Title 55.1 Chapter 12](https://law.lis.virginia.gov/vacode/title55.1/chapter12/); [Mason Veterans and Servicemembers Legal Clinic — 2019 Amendments to the Virginia Residential Landlord Tenant Act Benefit Virginia Renters](https://mvets.law.gmu.edu/2020/05/19/2019-amendments-to-the-virginia-residential-landlord-tenant-act-benefit-virginia-renters/); [Arlington County — VRLTA Presentation April 14, 2021](https://www.arlingtonva.us/files/sharedassets/public/v/1/commissions/documents/tlc-vrlta-presentation-2021-04-14.pdf); [Virginia Department of Housing and Community Development — VRLTA Landlord-Tenant Handbook Effective July 1, 2025](https://www.dhcd.virginia.gov/sites/default/files/Docx/landlord-tenant/landlord-tenant-handbook-final.pdf); [Justia — 2025 Code of Virginia § 55.1-1226 Security Deposits](https://law.justia.com/codes/virginia/title-55-1/chapter-12/section-55-1-1226/); [Richmond Redevelopment and Housing Authority — Code of Virginia VRLTA PDF](https://www.rrha.com/wp-content/uploads/2020/09/Virginia-Residential-Landlord-and-Tenant-Act.pdf); [Virginia Law Library — § 55.1-1245 (Effective the later of July 1, 2028, or seven years after the COVID-19 pandemic state of emergency expires) Noncompliance with rental agreement](https://law.lis.virginia.gov/vacode/title55.1/chapter12/section55.1-1245/); [Virginia Law Library — § 55.1-1204 Terms and conditions of rental agreement; payment of rent; copy of rental agreement for tenant](https://law.lis.virginia.gov/vacode/title55.1/chapter12/section55.1-1204/); [Virginia REALTORS — Am I allowed to add additional tenant late fees? November 21, 2024](https://virginiarealtors.org/2024/11/21/am-i-allowed-to-add-additional-tenant-late-fees/)).
//! - **Codification**: Va. Code §§ **55.1-1200 through 55.1-1262** (Title 55.1 Property and Conveyances, Chapter 12 Virginia Residential Landlord and Tenant Act).
//! - **§ 55.1-1226 Security Deposit**: cap = **2 MONTHS' RENT** (security deposit + damage insurance premiums combined cannot exceed 2 months' periodic rent); return required within **45 DAYS** after tenant vacates and surrenders possession or lease ends (whichever occurs LATER); landlord who misses 45-day deadline LOSES the right to withhold any funds; willful failure exposes landlord to **ACTUAL DAMAGES + REASONABLE ATTORNEY FEES + COURT COSTS**.
//! - **§ 55.1-1204 Late Fee Cap**: no late charge shall exceed the **LESSER OF (a) 10 PERCENT of the periodic rent OR (b) 10 PERCENT of the remaining balance due and owed by the tenant**; daily late fees PROHIBITED; once assessed, late fee stays static until rent paid.
//! - **§ 55.1-1245 Five-Day Pay or Quit Notice**: if rent unpaid when due and tenant fails to pay within **5 DAYS** after written notice is served, landlord may terminate rental agreement and proceed with unlawful detainer; notice must inform tenant of nonpayment AND of landlord's intention to terminate if rent not paid within 5-day window.
//! - **§ 55.1-1245(B) Material Noncompliance Cure or Quit Notice**: for material noncompliance (other than nonpayment) — 30-day written notice with right to cure within 21 days; if breach not curable, landlord may terminate with 30-day notice.
//! - **§ 55.1-1229 Landlord Entry**: landlord must give tenant **24 HOURS' notice** of intent to enter except in emergencies; entry must be at reasonable times.
//! - **§ 55.1-1244 Retaliatory Conduct Prohibited**: landlord may NOT retaliate by raising rent, decreasing services, or commencing eviction after tenant complaints to housing authority, requests for repairs, organization of tenants, or assertion of VRLTA rights; retaliation presumption arises if landlord adverse action within **6 MONTHS** of protected tenant activity.
//! - **§ 55.1-1248 Tenant Remedies for Landlord Noncompliance**: tenant may terminate after 30 days written notice if landlord materially breaches and fails to cure within 21 days; tenant may recover damages and obtain injunctive relief.
//! - **§ 55.1-1250 Fire / Casualty Damage**: if dwelling unit is damaged or destroyed by fire or casualty without fault of tenant such that enjoyment is substantially impaired, tenant may immediately vacate and notify landlord in writing of intention to terminate within **14 DAYS** of vacating.
//! - **§ 55.1-1227 Essential Services**: landlord must maintain in good and safe working order all electrical, plumbing, sanitary, heating, ventilating, air-conditioning, and other facilities and appliances supplied or required to be supplied by the landlord.
//! - **§ 55.1-1253 Eviction Record Sealing**: court may seal records of unlawful detainer proceedings under specified conditions.
//! - **2019 Amendment Major Reforms** (effective October 1, 2019): eliminated pre-2019 small-landlord (≤ 4 units) exemption — VRLTA now applies to ALL residential rentals statewide; required written rental agreements with mandatory disclosures; standardized statewide rent late fee cap at 10 %.
//! - **Va. Code § 36-93 Locality Preemption**: localities preempted from enacting rent control or rent stabilization more stringent than VRLTA.
//! - **§ 55.1-1245 COVID Sunset Provision**: certain § 55.1-1245 provisions effective the LATER of **July 1, 2028** OR seven years after the COVID-19 pandemic state of emergency expires.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const VRLTA_ORIGINAL_ENACTMENT_YEAR: u32 = 1974;
pub const VRLTA_RECODIFICATION_EFFECTIVE_DATE_YEAR: u32 = 2019;
pub const VRLTA_RECODIFICATION_EFFECTIVE_DATE_MONTH: u32 = 10;
pub const VRLTA_RECODIFICATION_EFFECTIVE_DATE_DAY: u32 = 1;
pub const VRLTA_SECURITY_DEPOSIT_CAP_IN_MONTHS_OF_RENT: u32 = 2;
pub const VRLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 45;
pub const VRLTA_LATE_FEE_CAP_PERCENT_OF_PERIODIC_RENT_BPS: u64 = 1_000;
pub const VRLTA_FIVE_DAY_PAY_OR_QUIT_NOTICE_DAYS: u32 = 5;
pub const VRLTA_MATERIAL_NONCOMPLIANCE_CURE_PERIOD_DAYS: u32 = 21;
pub const VRLTA_MATERIAL_NONCOMPLIANCE_TERMINATION_NOTICE_DAYS: u32 = 30;
pub const VRLTA_LANDLORD_ENTRY_NOTICE_HOURS: u32 = 24;
pub const VRLTA_RETALIATION_PRESUMPTION_WINDOW_MONTHS: u32 = 6;
pub const VRLTA_CASUALTY_VACATE_NOTICE_DEADLINE_DAYS: u32 = 14;
pub const VRLTA_COVID_SUNSET_PROVISION_TARGET_DATE_YEAR: u32 = 2028;
pub const VRLTA_COVID_SUNSET_PROVISION_TARGET_DATE_MONTH: u32 = 7;
pub const VRLTA_COVID_SUNSET_PROVISION_TARGET_DATE_DAY: u32 = 1;
pub const VRLTA_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    ResidentialRentalDwellingCoveredByVrlta,
    HotelOrMotelTransientLodgingExempt,
    CommercialRentalExempt,
    OccupancyByOwnerSpouseChildOrParentExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositCapTwoMonthsRentUnderSection55_1_1226,
    SecurityDepositReturnFortyFiveDayDeadlineUnderSection55_1_1226,
    LateFeeCapTenPercentLesserOfPeriodicRentOrRemainingBalanceUnderSection55_1_1204,
    FiveDayPayOrQuitNoticeUnderSection55_1_1245A,
    MaterialNoncomplianceCureOrQuitTwentyOneDayCureUnderSection55_1_1245B,
    LandlordEntryTwentyFourHourNoticeUnderSection55_1_1229,
    RetaliatoryConductProhibitedUnderSection55_1_1244,
    EssentialServicesMaintenanceUnderSection55_1_1227,
    CasualtyDamageTenantTerminationFourteenDayNoticeUnderSection55_1_1250,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_type: TenancyType,
    pub compliance_aspect: ComplianceAspect,
    pub monthly_rent_dollars: u64,
    pub security_deposit_dollars: u64,
    pub days_since_tenant_vacated_for_deposit_return: u32,
    pub deposit_returned_or_itemized_within_window: bool,
    pub charged_late_fee_dollars: u64,
    pub remaining_balance_due_dollars: u64,
    pub pay_or_quit_notice_period_days_given: u32,
    pub material_noncompliance_cure_period_days_given: u32,
    pub material_noncompliance_termination_notice_days_given: u32,
    pub landlord_entry_notice_hours_given: u32,
    pub entry_was_emergency: bool,
    pub adverse_action_within_six_months_of_protected_activity: bool,
    pub protected_activity_occurred: bool,
    pub essential_services_maintained: bool,
    pub casualty_vacate_notice_days_given: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: VrltaMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VrltaMode {
    NotApplicableTenancyExemptFromVrlta,
    CompliantSecurityDepositAtOrBelowTwoMonthsRentCap,
    CompliantSecurityDepositReturnedWithinFortyFiveDays,
    CompliantLateFeeAtOrBelowLesserOfTenPctPeriodicOrRemainingBalance,
    CompliantFiveDayPayOrQuitNoticeProvided,
    CompliantMaterialNoncomplianceTwentyOneDayCureAndThirtyDayTerminationProvided,
    CompliantLandlordEntryTwentyFourHourNoticeOrEmergency,
    CompliantNoRetaliatoryConductWithinSixMonthsOfProtectedActivity,
    CompliantEssentialServicesMaintained,
    CompliantCasualtyTenantTerminationFourteenDayNoticeProvided,
    ViolationSecurityDepositExceedsTwoMonthsRentCap,
    ViolationSecurityDepositReturnedPastFortyFiveDayDeadline,
    ViolationLateFeeExceedsLesserOfTenPctPeriodicOrRemainingBalance,
    ViolationPayOrQuitNoticePeriodShorterThanFiveDays,
    ViolationMaterialNoncomplianceCurePeriodShorterThanTwentyOneDays,
    ViolationLandlordEntryNoticeShorterThanTwentyFourHoursAndNotEmergency,
    ViolationRetaliatoryConductWithinSixMonthsOfProtectedActivity,
    ViolationEssentialServicesNotMaintained,
    ViolationCasualtyTenantTerminationNoticeExceededFourteenDayDeadline,
}

pub type RentalVirginiaVrltaVaCode55_1_1200Input = Input;
pub type RentalVirginiaVrltaVaCode55_1_1200Output = Output;
pub type RentalVirginiaVrltaVaCode55_1_1200Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Virginia Residential Landlord and Tenant Act — originally enacted by Virginia General Assembly in 1974; recodified into Title 55.1 Chapter 12 (Va. Code §§ 55.1-1200 through 55.1-1262) by 2019 General Assembly with effective date October 1, 2019; further amendments through 2025 General Assembly".to_string(),
        "Va. Code § 55.1-1200 Definitions (effective October 1, 2019) — VRLTA applies to ALL residential rental relationships in Virginia after the 2019 amendments eliminated the pre-2019 small-landlord (≤ 4 units) exemption".to_string(),
        "Va. Code § 55.1-1204 Terms and conditions of rental agreement; payment of rent; copy of rental agreement for tenant — late fee cap = LESSER of (a) 10 PERCENT of periodic rent OR (b) 10 PERCENT of remaining balance due and owed by tenant; daily late fees PROHIBITED; once assessed, late fee stays static".to_string(),
        "Va. Code § 55.1-1226 Security deposits — cap = 2 MONTHS' RENT (security deposit + damage insurance premiums combined cannot exceed 2 months' periodic rent); return required within 45 DAYS after tenant vacates and surrenders possession or lease ends (whichever is LATER); landlord who misses 45-day deadline LOSES right to withhold any funds; willful failure exposes landlord to ACTUAL DAMAGES + REASONABLE ATTORNEY FEES + COURT COSTS".to_string(),
        "Va. Code § 55.1-1227 Essential services — landlord must maintain in good and safe working order all electrical, plumbing, sanitary, heating, ventilating, air-conditioning, and other facilities and appliances supplied or required to be supplied by the landlord".to_string(),
        "Va. Code § 55.1-1229 Landlord access to dwelling unit — landlord must give tenant 24 HOURS' notice of intent to enter except in emergencies; entry must be at reasonable times".to_string(),
        "Va. Code § 55.1-1244 Retaliatory conduct prohibited — landlord may NOT retaliate by raising rent, decreasing services, or commencing eviction after tenant complaints to housing authority, requests for repairs, organization of tenants, or assertion of VRLTA rights; retaliation presumption arises if landlord adverse action within 6 MONTHS of protected tenant activity".to_string(),
        "Va. Code § 55.1-1245 Noncompliance with rental agreement (effective the later of July 1, 2028 or seven years after COVID-19 pandemic state of emergency expires) — § 55.1-1245(A) 5-day pay or quit notice for nonpayment of rent; § 55.1-1245(B) 21-day cure period + 30-day termination notice for material noncompliance".to_string(),
        "Va. Code § 55.1-1248 Tenant remedies for landlord noncompliance — tenant may terminate after 30 days written notice if landlord materially breaches and fails to cure within 21 days; tenant may recover damages and obtain injunctive relief".to_string(),
        "Va. Code § 55.1-1250 Fire or casualty damage — if dwelling unit damaged or destroyed by fire or casualty without fault of tenant such that enjoyment substantially impaired, tenant may immediately vacate and notify landlord in writing of intention to terminate within 14 DAYS of vacating".to_string(),
        "Va. Code § 55.1-1253 Eviction record sealing — court may seal records of unlawful detainer proceedings under specified conditions".to_string(),
        "Va. Code § 36-93 Locality preemption — Virginia localities preempted from enacting rent control or rent stabilization more stringent than VRLTA".to_string(),
        "2019 General Assembly Amendments — Major reforms effective October 1, 2019: eliminated pre-2019 small-landlord (≤ 4 units) exemption; required written rental agreements with mandatory disclosures; standardized statewide rent late fee cap at 10 percent".to_string(),
        "Virginia Department of Housing and Community Development — VRLTA Landlord-Tenant Handbook effective July 1, 2025 — practitioner guide".to_string(),
        "Mason Veterans and Servicemembers Legal Clinic — 2019 Amendments to Virginia Residential Landlord Tenant Act Benefit Virginia Renters — academic / pro bono synthesis".to_string(),
    ];

    if input.tenancy_type != TenancyType::ResidentialRentalDwellingCoveredByVrlta {
        return Output {
            mode: VrltaMode::NotApplicableTenancyExemptFromVrlta,
            statutory_basis: "Va. Code § 55.1-1201 — exempt tenancies (hotel/motel transient lodging; commercial rentals; occupancy by owner relations)".to_string(),
            notes: "NOT APPLICABLE: tenancy type exempt from VRLTA under § 55.1-1201 (hotel/motel transient lodging; commercial rental; occupancy by owner's spouse/child/parent).".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositCapTwoMonthsRentUnderSection55_1_1226 => {
            let cap = input
                .monthly_rent_dollars
                .saturating_mul(u64::from(VRLTA_SECURITY_DEPOSIT_CAP_IN_MONTHS_OF_RENT));
            if input.security_deposit_dollars <= cap {
                Output {
                    mode: VrltaMode::CompliantSecurityDepositAtOrBelowTwoMonthsRentCap,
                    statutory_basis: "Va. Code § 55.1-1226 — security deposit at or below 2-months'-rent statutory cap".to_string(),
                    notes: "COMPLIANT: security deposit at or below 2 months' rent statutory cap; combined security deposit and damage insurance premiums must not exceed 2 months' periodic rent.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: VrltaMode::ViolationSecurityDepositExceedsTwoMonthsRentCap,
                    statutory_basis: "Va. Code § 55.1-1226 — security deposit exceeds 2-months'-rent statutory cap".to_string(),
                    notes: "VIOLATION: security deposit exceeds 2 months' rent cap under § 55.1-1226; landlord must refund excess.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::SecurityDepositReturnFortyFiveDayDeadlineUnderSection55_1_1226 => {
            if input.deposit_returned_or_itemized_within_window
                && input.days_since_tenant_vacated_for_deposit_return
                    <= VRLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS
            {
                Output {
                    mode: VrltaMode::CompliantSecurityDepositReturnedWithinFortyFiveDays,
                    statutory_basis: "Va. Code § 55.1-1226 — security deposit returned or itemized within 45-day statutory deadline".to_string(),
                    notes: "COMPLIANT: landlord returned security deposit (or provided itemized statement of deductions with balance) within 45-day window after tenant vacated.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: VrltaMode::ViolationSecurityDepositReturnedPastFortyFiveDayDeadline,
                    statutory_basis: "Va. Code § 55.1-1226 — security deposit not returned within 45-day statutory deadline".to_string(),
                    notes: "VIOLATION: landlord missed 45-day deposit return deadline under § 55.1-1226; LOSES right to withhold any funds; willful failure exposes landlord to ACTUAL DAMAGES + REASONABLE ATTORNEY FEES + COURT COSTS.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LateFeeCapTenPercentLesserOfPeriodicRentOrRemainingBalanceUnderSection55_1_1204 => {
            let ten_pct_rent = u128::from(input.monthly_rent_dollars) * 1_000 / 10_000;
            let ten_pct_balance = u128::from(input.remaining_balance_due_dollars) * 1_000 / 10_000;
            let cap = u64::try_from(ten_pct_rent.min(ten_pct_balance)).unwrap_or(u64::MAX);
            if input.charged_late_fee_dollars <= cap {
                Output {
                    mode: VrltaMode::CompliantLateFeeAtOrBelowLesserOfTenPctPeriodicOrRemainingBalance,
                    statutory_basis: "Va. Code § 55.1-1204 — late fee at or below lesser of 10 percent of periodic rent or 10 percent of remaining balance".to_string(),
                    notes: "COMPLIANT: late fee charged is at or below the statutory cap of the LESSER of 10 percent of periodic rent or 10 percent of remaining balance due.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: VrltaMode::ViolationLateFeeExceedsLesserOfTenPctPeriodicOrRemainingBalance,
                    statutory_basis: "Va. Code § 55.1-1204 — late fee exceeds statutory cap".to_string(),
                    notes: "VIOLATION: late fee exceeds the statutory cap of the LESSER of 10 percent of periodic rent or 10 percent of remaining balance due under § 55.1-1204; daily late fees prohibited.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::FiveDayPayOrQuitNoticeUnderSection55_1_1245A => {
            if input.pay_or_quit_notice_period_days_given >= VRLTA_FIVE_DAY_PAY_OR_QUIT_NOTICE_DAYS {
                Output {
                    mode: VrltaMode::CompliantFiveDayPayOrQuitNoticeProvided,
                    statutory_basis: "Va. Code § 55.1-1245(A) — 5-day pay or quit notice provided for nonpayment".to_string(),
                    notes: "COMPLIANT: landlord provided 5-day pay or quit written notice under § 55.1-1245(A); notice properly informs tenant of nonpayment and of landlord's intention to terminate if rent not paid within 5-day window.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: VrltaMode::ViolationPayOrQuitNoticePeriodShorterThanFiveDays,
                    statutory_basis: "Va. Code § 55.1-1245(A) — pay or quit notice period shorter than statutory 5-day minimum".to_string(),
                    notes: "VIOLATION: pay or quit notice period shorter than the 5-day statutory minimum under § 55.1-1245(A); unlawful detainer action subject to dismissal.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::MaterialNoncomplianceCureOrQuitTwentyOneDayCureUnderSection55_1_1245B => {
            if input.material_noncompliance_cure_period_days_given
                >= VRLTA_MATERIAL_NONCOMPLIANCE_CURE_PERIOD_DAYS
                && input.material_noncompliance_termination_notice_days_given
                    >= VRLTA_MATERIAL_NONCOMPLIANCE_TERMINATION_NOTICE_DAYS
            {
                Output {
                    mode: VrltaMode::CompliantMaterialNoncomplianceTwentyOneDayCureAndThirtyDayTerminationProvided,
                    statutory_basis: "Va. Code § 55.1-1245(B) — 21-day cure and 30-day termination notice provided".to_string(),
                    notes: "COMPLIANT: landlord provided 21-day cure period and 30-day termination notice for material noncompliance under § 55.1-1245(B).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: VrltaMode::ViolationMaterialNoncomplianceCurePeriodShorterThanTwentyOneDays,
                    statutory_basis: "Va. Code § 55.1-1245(B) — material noncompliance cure or termination notice period shorter than statutory minimums".to_string(),
                    notes: "VIOLATION: material noncompliance cure period shorter than 21 days or termination notice shorter than 30 days under § 55.1-1245(B); unlawful detainer action subject to dismissal.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LandlordEntryTwentyFourHourNoticeUnderSection55_1_1229 => {
            if input.entry_was_emergency
                || input.landlord_entry_notice_hours_given >= VRLTA_LANDLORD_ENTRY_NOTICE_HOURS
            {
                Output {
                    mode: VrltaMode::CompliantLandlordEntryTwentyFourHourNoticeOrEmergency,
                    statutory_basis: "Va. Code § 55.1-1229 — 24-hour entry notice or emergency exception".to_string(),
                    notes: "COMPLIANT: landlord provided at least 24-hour notice prior to entering dwelling unit, or entry was during emergency.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: VrltaMode::ViolationLandlordEntryNoticeShorterThanTwentyFourHoursAndNotEmergency,
                    statutory_basis: "Va. Code § 55.1-1229 — landlord entry without 24-hour notice and not emergency".to_string(),
                    notes: "VIOLATION: landlord entered without 24-hour notice and not under emergency conditions; § 55.1-1229 violation; tenant may obtain injunctive relief.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::RetaliatoryConductProhibitedUnderSection55_1_1244 => {
            if input.protected_activity_occurred
                && input.adverse_action_within_six_months_of_protected_activity
            {
                Output {
                    mode: VrltaMode::ViolationRetaliatoryConductWithinSixMonthsOfProtectedActivity,
                    statutory_basis: "Va. Code § 55.1-1244 — retaliatory conduct within 6 months of protected tenant activity".to_string(),
                    notes: "VIOLATION: landlord engaged in adverse action (rent raise / service reduction / eviction commencement) within 6 months of tenant's protected activity (housing authority complaint / repair request / tenant organization / VRLTA rights assertion); retaliation presumption arises under § 55.1-1244.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: VrltaMode::CompliantNoRetaliatoryConductWithinSixMonthsOfProtectedActivity,
                    statutory_basis: "Va. Code § 55.1-1244 — no retaliatory conduct presumption arises".to_string(),
                    notes: "COMPLIANT: no adverse action within 6-month retaliation window OR no protected tenant activity to trigger presumption.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::EssentialServicesMaintenanceUnderSection55_1_1227 => {
            if input.essential_services_maintained {
                Output {
                    mode: VrltaMode::CompliantEssentialServicesMaintained,
                    statutory_basis: "Va. Code § 55.1-1227 — essential services maintained".to_string(),
                    notes: "COMPLIANT: landlord maintains in good and safe working order all electrical, plumbing, sanitary, heating, ventilating, air-conditioning, and other facilities supplied or required.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: VrltaMode::ViolationEssentialServicesNotMaintained,
                    statutory_basis: "Va. Code § 55.1-1227 — essential services not maintained".to_string(),
                    notes: "VIOLATION: landlord failed to maintain essential services; tenant remedies under § 55.1-1248 (30-day termination after 21-day cure failure + damages + injunctive relief).".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::CasualtyDamageTenantTerminationFourteenDayNoticeUnderSection55_1_1250 => {
            if input.casualty_vacate_notice_days_given <= VRLTA_CASUALTY_VACATE_NOTICE_DEADLINE_DAYS {
                Output {
                    mode: VrltaMode::CompliantCasualtyTenantTerminationFourteenDayNoticeProvided,
                    statutory_basis: "Va. Code § 55.1-1250 — casualty tenant termination notice within 14 days of vacating".to_string(),
                    notes: "COMPLIANT: tenant provided written termination notice within 14 days of vacating after fire or casualty damage without fault of tenant.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: VrltaMode::ViolationCasualtyTenantTerminationNoticeExceededFourteenDayDeadline,
                    statutory_basis: "Va. Code § 55.1-1250 — casualty tenant termination notice exceeded 14-day deadline".to_string(),
                    notes: "VIOLATION: tenant casualty termination notice exceeded 14-day deadline under § 55.1-1250; termination right not preserved.".to_string(),
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
            tenancy_type: TenancyType::ResidentialRentalDwellingCoveredByVrlta,
            compliance_aspect:
                ComplianceAspect::SecurityDepositCapTwoMonthsRentUnderSection55_1_1226,
            monthly_rent_dollars: 2_000,
            security_deposit_dollars: 4_000,
            days_since_tenant_vacated_for_deposit_return: 30,
            deposit_returned_or_itemized_within_window: true,
            charged_late_fee_dollars: 200,
            remaining_balance_due_dollars: 2_000,
            pay_or_quit_notice_period_days_given: 5,
            material_noncompliance_cure_period_days_given: 21,
            material_noncompliance_termination_notice_days_given: 30,
            landlord_entry_notice_hours_given: 24,
            entry_was_emergency: false,
            adverse_action_within_six_months_of_protected_activity: false,
            protected_activity_occurred: false,
            essential_services_maintained: true,
            casualty_vacate_notice_days_given: 14,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_type = TenancyType::HotelOrMotelTransientLodgingExempt;
        let output = check(&input);
        assert_eq!(output.mode, VrltaMode::NotApplicableTenancyExemptFromVrlta);
    }

    #[test]
    fn security_deposit_at_two_months_cap_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            VrltaMode::CompliantSecurityDepositAtOrBelowTwoMonthsRentCap
        );
    }

    #[test]
    fn security_deposit_at_exactly_two_months_boundary_compliant() {
        let mut input = baseline_input();
        input.monthly_rent_dollars = 1_500;
        input.security_deposit_dollars = 3_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::CompliantSecurityDepositAtOrBelowTwoMonthsRentCap
        );
    }

    #[test]
    fn security_deposit_at_two_months_plus_one_dollar_violation() {
        let mut input = baseline_input();
        input.monthly_rent_dollars = 1_500;
        input.security_deposit_dollars = 3_001;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::ViolationSecurityDepositExceedsTwoMonthsRentCap
        );
    }

    #[test]
    fn security_deposit_returned_within_forty_five_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnFortyFiveDayDeadlineUnderSection55_1_1226;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::CompliantSecurityDepositReturnedWithinFortyFiveDays
        );
    }

    #[test]
    fn security_deposit_returned_at_exactly_forty_five_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnFortyFiveDayDeadlineUnderSection55_1_1226;
        input.days_since_tenant_vacated_for_deposit_return = 45;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::CompliantSecurityDepositReturnedWithinFortyFiveDays
        );
    }

    #[test]
    fn security_deposit_returned_at_forty_six_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnFortyFiveDayDeadlineUnderSection55_1_1226;
        input.days_since_tenant_vacated_for_deposit_return = 46;
        input.deposit_returned_or_itemized_within_window = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::ViolationSecurityDepositReturnedPastFortyFiveDayDeadline
        );
    }

    #[test]
    fn late_fee_at_ten_percent_of_rent_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LateFeeCapTenPercentLesserOfPeriodicRentOrRemainingBalanceUnderSection55_1_1204;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::CompliantLateFeeAtOrBelowLesserOfTenPctPeriodicOrRemainingBalance
        );
    }

    #[test]
    fn late_fee_exceeds_ten_percent_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LateFeeCapTenPercentLesserOfPeriodicRentOrRemainingBalanceUnderSection55_1_1204;
        input.charged_late_fee_dollars = 250;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::ViolationLateFeeExceedsLesserOfTenPctPeriodicOrRemainingBalance
        );
    }

    #[test]
    fn late_fee_capped_at_lesser_remaining_balance() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LateFeeCapTenPercentLesserOfPeriodicRentOrRemainingBalanceUnderSection55_1_1204;
        input.monthly_rent_dollars = 2_000;
        input.remaining_balance_due_dollars = 500;
        input.charged_late_fee_dollars = 51;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::ViolationLateFeeExceedsLesserOfTenPctPeriodicOrRemainingBalance
        );
    }

    #[test]
    fn five_day_pay_or_quit_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveDayPayOrQuitNoticeUnderSection55_1_1245A;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::CompliantFiveDayPayOrQuitNoticeProvided
        );
    }

    #[test]
    fn pay_or_quit_notice_under_five_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveDayPayOrQuitNoticeUnderSection55_1_1245A;
        input.pay_or_quit_notice_period_days_given = 3;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::ViolationPayOrQuitNoticePeriodShorterThanFiveDays
        );
    }

    #[test]
    fn twenty_one_day_cure_and_thirty_day_termination_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MaterialNoncomplianceCureOrQuitTwentyOneDayCureUnderSection55_1_1245B;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::CompliantMaterialNoncomplianceTwentyOneDayCureAndThirtyDayTerminationProvided
        );
    }

    #[test]
    fn cure_period_shorter_than_twenty_one_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MaterialNoncomplianceCureOrQuitTwentyOneDayCureUnderSection55_1_1245B;
        input.material_noncompliance_cure_period_days_given = 14;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::ViolationMaterialNoncomplianceCurePeriodShorterThanTwentyOneDays
        );
    }

    #[test]
    fn landlord_entry_twenty_four_hour_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordEntryTwentyFourHourNoticeUnderSection55_1_1229;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::CompliantLandlordEntryTwentyFourHourNoticeOrEmergency
        );
    }

    #[test]
    fn landlord_entry_emergency_exception_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordEntryTwentyFourHourNoticeUnderSection55_1_1229;
        input.landlord_entry_notice_hours_given = 0;
        input.entry_was_emergency = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::CompliantLandlordEntryTwentyFourHourNoticeOrEmergency
        );
    }

    #[test]
    fn landlord_entry_under_twenty_four_hours_and_not_emergency_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LandlordEntryTwentyFourHourNoticeUnderSection55_1_1229;
        input.landlord_entry_notice_hours_given = 12;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::ViolationLandlordEntryNoticeShorterThanTwentyFourHoursAndNotEmergency
        );
    }

    #[test]
    fn retaliation_within_six_months_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::RetaliatoryConductProhibitedUnderSection55_1_1244;
        input.protected_activity_occurred = true;
        input.adverse_action_within_six_months_of_protected_activity = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::ViolationRetaliatoryConductWithinSixMonthsOfProtectedActivity
        );
    }

    #[test]
    fn no_retaliation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::RetaliatoryConductProhibitedUnderSection55_1_1244;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::CompliantNoRetaliatoryConductWithinSixMonthsOfProtectedActivity
        );
    }

    #[test]
    fn essential_services_maintained_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::EssentialServicesMaintenanceUnderSection55_1_1227;
        let output = check(&input);
        assert_eq!(output.mode, VrltaMode::CompliantEssentialServicesMaintained);
    }

    #[test]
    fn essential_services_not_maintained_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::EssentialServicesMaintenanceUnderSection55_1_1227;
        input.essential_services_maintained = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::ViolationEssentialServicesNotMaintained
        );
    }

    #[test]
    fn casualty_termination_within_fourteen_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::CasualtyDamageTenantTerminationFourteenDayNoticeUnderSection55_1_1250;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::CompliantCasualtyTenantTerminationFourteenDayNoticeProvided
        );
    }

    #[test]
    fn casualty_termination_past_fourteen_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::CasualtyDamageTenantTerminationFourteenDayNoticeUnderSection55_1_1250;
        input.casualty_vacate_notice_days_given = 20;
        let output = check(&input);
        assert_eq!(
            output.mode,
            VrltaMode::ViolationCasualtyTenantTerminationNoticeExceededFourteenDayDeadline
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(VRLTA_ORIGINAL_ENACTMENT_YEAR, 1974);
        assert_eq!(VRLTA_RECODIFICATION_EFFECTIVE_DATE_YEAR, 2019);
        assert_eq!(VRLTA_RECODIFICATION_EFFECTIVE_DATE_MONTH, 10);
        assert_eq!(VRLTA_RECODIFICATION_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(VRLTA_SECURITY_DEPOSIT_CAP_IN_MONTHS_OF_RENT, 2);
        assert_eq!(VRLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 45);
        assert_eq!(VRLTA_LATE_FEE_CAP_PERCENT_OF_PERIODIC_RENT_BPS, 1_000);
        assert_eq!(VRLTA_FIVE_DAY_PAY_OR_QUIT_NOTICE_DAYS, 5);
        assert_eq!(VRLTA_MATERIAL_NONCOMPLIANCE_CURE_PERIOD_DAYS, 21);
        assert_eq!(VRLTA_MATERIAL_NONCOMPLIANCE_TERMINATION_NOTICE_DAYS, 30);
        assert_eq!(VRLTA_LANDLORD_ENTRY_NOTICE_HOURS, 24);
        assert_eq!(VRLTA_RETALIATION_PRESUMPTION_WINDOW_MONTHS, 6);
        assert_eq!(VRLTA_CASUALTY_VACATE_NOTICE_DEADLINE_DAYS, 14);
        assert_eq!(VRLTA_COVID_SUNSET_PROVISION_TARGET_DATE_YEAR, 2028);
        assert_eq!(VRLTA_COVID_SUNSET_PROVISION_TARGET_DATE_MONTH, 7);
        assert_eq!(VRLTA_COVID_SUNSET_PROVISION_TARGET_DATE_DAY, 1);
        assert_eq!(VRLTA_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Virginia Residential Landlord and Tenant Act"));
        assert!(joined.contains("1974"));
        assert!(joined.contains("October 1, 2019"));
        assert!(joined.contains("§§ 55.1-1200 through 55.1-1262"));
        assert!(joined.contains("§ 55.1-1204"));
        assert!(joined.contains("§ 55.1-1226"));
        assert!(joined.contains("§ 55.1-1227"));
        assert!(joined.contains("§ 55.1-1229"));
        assert!(joined.contains("§ 55.1-1244"));
        assert!(joined.contains("§ 55.1-1245"));
        assert!(joined.contains("§ 55.1-1248"));
        assert!(joined.contains("§ 55.1-1250"));
        assert!(joined.contains("2 MONTHS"));
        assert!(joined.contains("45 DAYS"));
        assert!(joined.contains("10 PERCENT"));
        assert!(joined.contains("5-day"));
        assert!(joined.contains("21-day"));
        assert!(joined.contains("30-day"));
        assert!(joined.contains("24 HOURS"));
        assert!(joined.contains("6 MONTHS"));
        assert!(joined.contains("14 DAYS"));
        assert!(joined.contains("Va. Code § 36-93"));
    }
}
