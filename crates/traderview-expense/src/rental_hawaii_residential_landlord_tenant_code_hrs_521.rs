//! Hawaii Residential Landlord-Tenant Code (HRS Chapter
//! 521) Compliance Module — comprehensive Hawaii state-level
//! landlord-tenant statute covering all residential rental
//! properties in the State of Hawaii.
//!
//! Pure-compute check for landlord compliance with Hawaii
//! Revised Statutes Chapter 521 (Residential Landlord-Tenant
//! Code), administered by the Hawaii Department of Commerce
//! and Consumer Affairs (DCCA) Regulated Industries
//! Complaints Office (RICO). HRS 521 establishes the
//! statewide framework for residential tenancies in Hawaii,
//! covering security deposits, rent increases, termination
//! notices, repair obligations, landlord entry, and
//! retaliatory eviction prohibitions. Distinct from the
//! transient-accommodations statute at HRS Chapter 521-8
//! exclusion (covering hotels, motels, vacation rentals
//! under 90 days). Hawaii does NOT have statewide rent
//! control; HRS 521 imposes only procedural notice
//! requirements for rent increases.
//!
//! Web research (verified 2026-06-03):
//! - **Statutory Basis**: HRS Chapter 521 — Residential Landlord-Tenant Code; administered by Hawaii DCCA Regulated Industries Complaints Office (RICO) ([Hawaii Capitol HRS Chapter 521](https://www.capitol.hawaii.gov/hrscurrent/vol12_ch0501-0588/HRS0521/HRS_0521-.htm); [Hawaii Capitol HRS § 521-44 Security Deposit](https://www.capitol.hawaii.gov/hrscurrent/Vol12_Ch0501-0588/HRS0521/HRS_0521-0044.htm); [Hawaii Residential Landlord-Tenant Code Handbook](https://www.hawaiis.com/tenant-landlord-code/); [Hawaii County of Kauai Section 8 Landlord-Tenant Handbook (January 2022)](https://www.kauai.gov/files/assets/public/v/2/housing-agency/documents/section-8/01-2022-landlord-tenant-handbook-003.pdf); [Hawaii DCCA REB Real Education Landlord-Tenant Code Deadlines](https://files.hawaii.gov/dcca/reb/real_ed/re_ed/ce_prelic/landlord_tenant_code_deadlines.pdf); [American Apartment Owners Association — Hawaii Landlord Tenant Law](https://american-apartment-owners-association.org/landlord-tenant-laws/hawaii/); [Landlord-Tenant Law — Hawaii Landlord Tenant Law and Code in Plain English](https://www.landlord-tenant-law.com/hawaii-landlord-tenant-law.html); [Maui Mapp — Hawaii Landlord-Tenant Code Chapter 521](http://www.mauimapp.com/government/hrs521-51.htm)).
//! - **Security Deposit Cap (HRS § 521-44(b))**: total amount of all deposits may NOT exceed **ONE MONTH'S RENT**. Landlords may not collect, hold, or demand a security deposit exceeding one month of contract rent.
//! - **Security Deposit Return (HRS § 521-44(c))**: landlord must return security deposit or remaining portion (after lawful deductions for unpaid rent, damages beyond normal wear and tear, and breach-of-lease costs) to tenant within **14 DAYS** after termination of the rental agreement. Itemized list of deductions required for any retained amounts.
//! - **Willful Retention Treble Damages (HRS § 521-44(f))**: if the court finds that the landlord WILLFULLY retained any portion of the security deposit without justification, the tenant is entitled to **TREBLE (3X) damages** plus reasonable attorney's fees and costs. Small claims court has concurrent jurisdiction.
//! - **Rent Increase Notice (HRS § 521-21(d))**: month-to-month rent increases require **45 CONSECUTIVE DAYS** written notice; week-to-week rent increases require **15 DAYS** written notice. Hawaii does NOT have statewide rent control; HRS 521 imposes only procedural notice requirements.
//! - **Landlord Termination Notice — Month-to-Month (HRS § 521-71(a))**: landlord must provide **45 DAYS** written notice to terminate a month-to-month tenancy.
//! - **Tenant Termination Notice — Month-to-Month (HRS § 521-71(b))**: tenant must provide **28 DAYS** written notice to terminate a month-to-month tenancy; tenant is responsible for full 28 days' rent regardless of when they move out.
//! - **Demolition / Conversion Notice (HRS § 521-71(d))**: special **120-DAY** notice required for termination of tenancy for the purpose of demolishing the dwelling unit or converting to non-residential use.
//! - **Non-Payment of Rent Notice (HRS § 521-68)**: landlord may demand rent and provide **5 BUSINESS DAYS** notice before initiating eviction action for non-payment of rent.
//! - **Rule Violation Cure Notice (HRS § 521-72)**: tenant has **10 DAYS** to remedy a rule violation after written notice from the landlord; failure to cure permits landlord to initiate termination.
//! - **Landlord Entry Notice (HRS § 521-53)**: landlord must provide **2 DAYS** advance notice for entry to make repairs, show the unit, or perform inspections; emergency entries (fire, flood, immediate health hazard) are exempt from the notice requirement.
//! - **Repair Response Times (HRS § 521-64)**: landlord must respond to repair requests within **3 BUSINESS DAYS** for emergency repairs (e.g., loss of essential services such as water, electricity, sewer) and **12 BUSINESS DAYS** for general repairs (non-emergency habitability concerns).
//! - **Repair-and-Deduct Remedy (HRS § 521-64(b))**: if landlord fails to make required repairs within statutory timeframes, tenant may make the repairs and deduct the cost from rent, subject to statutory limits and procedural requirements.
//! - **Retaliatory Eviction Prohibition (HRS § 521-74)**: prohibits landlord from increasing rent, decreasing services, or terminating tenancy in retaliation for tenant's good-faith complaint to a health agency, code enforcement agency, or other governmental body, or for tenant's exercise of HRS 521 rights. Rebuttable presumption of retaliation if landlord action occurs within statutory lookback (typically 6 months) of tenant's protected activity.
//! - **Transient Accommodations Exclusion (HRS § 521-8)**: HRS 521 does NOT apply to transient lodging (hotels, motels, vacation rentals under 90 days); these are governed by HRS Chapter 467D (Travel Agents and Tour Companies) and the Transient Accommodations Tax framework.
//! - **Civil Enforcement**: HRS 521 violations may be enforced through private right of action in Hawaii District Court or Small Claims Division; willful violations may trigger treble damages plus reasonable attorney's fees.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const HRS_521_SECURITY_DEPOSIT_CAP_MONTHS_RENT: u32 = 1;
pub const HRS_521_SECURITY_DEPOSIT_RETURN_DAYS: u32 = 14;
pub const HRS_521_WILLFUL_RETENTION_DAMAGES_MULTIPLIER: u32 = 3;
pub const HRS_521_RENT_INCREASE_NOTICE_DAYS_MONTH_TO_MONTH: u32 = 45;
pub const HRS_521_RENT_INCREASE_NOTICE_DAYS_WEEK_TO_WEEK: u32 = 15;
pub const HRS_521_LANDLORD_TERMINATION_NOTICE_DAYS_MONTH_TO_MONTH: u32 = 45;
pub const HRS_521_TENANT_TERMINATION_NOTICE_DAYS_MONTH_TO_MONTH: u32 = 28;
pub const HRS_521_DEMOLITION_CONVERSION_NOTICE_DAYS: u32 = 120;
pub const HRS_521_NON_PAYMENT_NOTICE_BUSINESS_DAYS: u32 = 5;
pub const HRS_521_RULE_VIOLATION_CURE_NOTICE_DAYS: u32 = 10;
pub const HRS_521_LANDLORD_ENTRY_NOTICE_DAYS: u32 = 2;
pub const HRS_521_EMERGENCY_REPAIRS_BUSINESS_DAYS: u32 = 3;
pub const HRS_521_GENERAL_REPAIRS_BUSINESS_DAYS: u32 = 12;
pub const HRS_521_TRANSIENT_LODGING_THRESHOLD_DAYS: u32 = 90;
pub const HRS_521_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    WithinStateOfHawaii,
    OutsideStateOfHawaii,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitType {
    StandardResidentialRentalCoveredByHrs521,
    TransientLodgingUnder90DaysExemptUnderHrs521_8,
    NonResidentialUnitExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositCapOneMonth,
    SecurityDepositReturnWithin14Days,
    RentIncreaseNotice45DaysMonthToMonth,
    LandlordTerminationNotice45DaysMonthToMonth,
    TenantTerminationNotice28DaysMonthToMonth,
    DemolitionConversionNotice120Days,
    NonPaymentNotice5BusinessDays,
    RuleViolationCureNotice10Days,
    LandlordEntryNotice2Days,
    EmergencyRepairsResponse3BusinessDays,
    GeneralRepairsResponse12BusinessDays,
    RetaliatoryEvictionProhibition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetaliationStatus {
    NoRetaliatoryActionTaken,
    RetaliatoryActionTakenAfterTenantHealthAgencyComplaint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HawaiiHrs521Mode {
    NotApplicablePropertyOutsideHawaii,
    NotApplicableTransientLodgingExempt,
    NotApplicableNonResidentialUnit,
    CompliantSecurityDepositAtOrBelowOneMonthRent,
    CompliantSecurityDepositReturnedWithin14Days,
    CompliantRentIncreaseNoticeAtOrAbove45DaysMonthToMonth,
    CompliantLandlordTerminationNoticeAtOrAbove45Days,
    CompliantTenantTerminationNoticeAtOrAbove28Days,
    CompliantDemolitionConversionNoticeAtOrAbove120Days,
    CompliantNonPaymentNoticeAtOrAbove5BusinessDays,
    CompliantRuleViolationCureNoticeAtOrAbove10Days,
    CompliantLandlordEntryNoticeAtOrAbove2Days,
    CompliantEmergencyRepairsWithin3BusinessDays,
    CompliantGeneralRepairsWithin12BusinessDays,
    CompliantNoRetaliatoryActionTaken,
    ViolationSecurityDepositExceedsOneMonthRent,
    ViolationSecurityDepositNotReturnedWithin14DaysTrebleDamagesExposure,
    ViolationRentIncreaseNoticeBelow45DaysMonthToMonth,
    ViolationLandlordTerminationNoticeBelow45Days,
    ViolationTenantTerminationNoticeBelow28Days,
    ViolationDemolitionConversionNoticeBelow120Days,
    ViolationNonPaymentNoticeBelow5BusinessDays,
    ViolationRuleViolationCureNoticeBelow10Days,
    ViolationLandlordEntryNoticeBelow2Days,
    ViolationEmergencyRepairsBeyond3BusinessDays,
    ViolationGeneralRepairsBeyond12BusinessDays,
    ViolationRetaliatoryEvictionAfterTenantHealthAgencyComplaint,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub unit_type: UnitType,
    pub compliance_aspect: ComplianceAspect,
    pub monthly_rent_cents: u64,
    pub security_deposit_held_cents: u64,
    pub security_deposit_return_days_after_termination: u32,
    pub rent_increase_notice_days_provided: u32,
    pub landlord_termination_notice_days_provided: u32,
    pub tenant_termination_notice_days_provided: u32,
    pub demolition_conversion_notice_days_provided: u32,
    pub non_payment_notice_business_days_provided: u32,
    pub rule_violation_cure_notice_days_provided: u32,
    pub landlord_entry_notice_days_provided: u32,
    pub emergency_repair_response_business_days: u32,
    pub general_repair_response_business_days: u32,
    pub retaliation_status: RetaliationStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: HawaiiHrs521Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub statutory_security_deposit_cap_cents: u64,
    pub statutory_treble_damages_exposure_cents: u64,
}

pub type RentalHawaiiResidentialLandlordTenantCodeHrs521Input = Input;
pub type RentalHawaiiResidentialLandlordTenantCodeHrs521Output = Output;
pub type RentalHawaiiResidentialLandlordTenantCodeHrs521Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Hawaii Revised Statutes Chapter 521 — Residential Landlord-Tenant Code; administered by Hawaii Department of Commerce and Consumer Affairs (DCCA) Regulated Industries Complaints Office (RICO); covers all residential rental properties in State of Hawaii".to_string(),
        "HRS § 521-44(b) Security Deposit Cap — total amount of all deposits may NOT exceed ONE MONTH'S RENT".to_string(),
        "HRS § 521-44(c) Security Deposit Return — landlord must return security deposit (or remaining portion after lawful deductions) within 14 DAYS after termination of rental agreement; itemized list of deductions required".to_string(),
        "HRS § 521-44(f) Willful Retention Treble Damages — if court finds landlord WILLFULLY retained any portion of security deposit without justification, tenant entitled to TREBLE (3X) damages plus reasonable attorney's fees and costs; small claims court has concurrent jurisdiction".to_string(),
        "HRS § 521-21(d) Rent Increase Notice — month-to-month rent increases require 45 CONSECUTIVE DAYS written notice; week-to-week increases require 15 DAYS notice; Hawaii does NOT have statewide rent control".to_string(),
        "HRS § 521-71(a) Landlord Termination Notice (Month-to-Month) — landlord must provide 45 DAYS written notice to terminate".to_string(),
        "HRS § 521-71(b) Tenant Termination Notice (Month-to-Month) — tenant must provide 28 DAYS written notice; tenant responsible for full 28 days' rent regardless of when they move out".to_string(),
        "HRS § 521-71(d) Demolition / Conversion Notice — special 120-DAY notice required for termination for purpose of demolishing dwelling unit or converting to non-residential use".to_string(),
        "HRS § 521-68 Non-Payment Notice — landlord may demand rent and provide 5 BUSINESS DAYS notice before initiating eviction for non-payment".to_string(),
        "HRS § 521-72 Rule Violation Cure Notice — tenant has 10 DAYS to remedy rule violation after written notice; failure to cure permits termination".to_string(),
        "HRS § 521-53 Landlord Entry Notice — landlord must provide 2 DAYS advance notice for entry to make repairs, show unit, or perform inspections; emergency entries exempt".to_string(),
        "HRS § 521-64 Repair Response Times — landlord must respond to repair requests within 3 BUSINESS DAYS for emergency repairs (loss of essential services) and 12 BUSINESS DAYS for general repairs".to_string(),
        "HRS § 521-64(b) Repair-and-Deduct Remedy — if landlord fails to make required repairs within statutory timeframes, tenant may make repairs and deduct cost from rent subject to statutory limits".to_string(),
        "HRS § 521-74 Retaliatory Eviction Prohibition — prohibits landlord from increasing rent, decreasing services, or terminating tenancy in retaliation for tenant's good-faith complaint to health agency / code enforcement agency / governmental body, or for tenant's exercise of HRS 521 rights; rebuttable presumption of retaliation within statutory lookback".to_string(),
        "HRS § 521-8 Transient Accommodations Exclusion — HRS 521 does NOT apply to transient lodging (hotels, motels, vacation rentals under 90 days); governed by HRS Chapter 467D and Transient Accommodations Tax framework".to_string(),
        "Civil Enforcement — HRS 521 violations enforced through private right of action in Hawaii District Court or Small Claims Division; willful violations may trigger treble damages plus reasonable attorney's fees".to_string(),
        "Hawaii Capitol HRS Chapter 521 — primary statutory text".to_string(),
        "Hawaii Residential Landlord-Tenant Code Handbook — official compliance guide".to_string(),
        "Hawaii DCCA REB Real Education Landlord-Tenant Code Deadlines — practitioner deadline reference".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::OutsideStateOfHawaii {
        return Output {
            mode: HawaiiHrs521Mode::NotApplicablePropertyOutsideHawaii,
            statutory_basis: "Property outside State of Hawaii; HRS Chapter 521 inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside State of Hawaii; HRS Chapter 521 Residential Landlord-Tenant Code inapplicable.".to_string(),
            citations,
            statutory_security_deposit_cap_cents: 0,
            statutory_treble_damages_exposure_cents: 0,
        };
    }

    if input.unit_type == UnitType::TransientLodgingUnder90DaysExemptUnderHrs521_8 {
        return Output {
            mode: HawaiiHrs521Mode::NotApplicableTransientLodgingExempt,
            statutory_basis: "HRS § 521-8 — transient lodging (hotels, motels, vacation rentals under 90 days) exempt".to_string(),
            notes: "NOT APPLICABLE: transient lodging under 90 days; HRS § 521-8 exemption applies; HRS Chapter 467D and Transient Accommodations Tax framework govern instead.".to_string(),
            citations,
            statutory_security_deposit_cap_cents: 0,
            statutory_treble_damages_exposure_cents: 0,
        };
    }

    if input.unit_type == UnitType::NonResidentialUnitExempt {
        return Output {
            mode: HawaiiHrs521Mode::NotApplicableNonResidentialUnit,
            statutory_basis: "HRS Chapter 521 applies only to residential units; non-residential exempt".to_string(),
            notes: "NOT APPLICABLE: unit is non-residential; HRS Chapter 521 Residential Landlord-Tenant Code applies only to residential rental units.".to_string(),
            citations,
            statutory_security_deposit_cap_cents: 0,
            statutory_treble_damages_exposure_cents: 0,
        };
    }

    let security_deposit_cap_cents = input
        .monthly_rent_cents
        .saturating_mul(u64::from(HRS_521_SECURITY_DEPOSIT_CAP_MONTHS_RENT));

    let treble_damages_exposure = input
        .security_deposit_held_cents
        .saturating_mul(u64::from(HRS_521_WILLFUL_RETENTION_DAMAGES_MULTIPLIER));

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositCapOneMonth => {
            if input.security_deposit_held_cents > security_deposit_cap_cents {
                Output {
                    mode: HawaiiHrs521Mode::ViolationSecurityDepositExceedsOneMonthRent,
                    statutory_basis: "HRS § 521-44(b) — security deposit capped at one month's rent".to_string(),
                    notes: format!(
                        "VIOLATION: security deposit ${} cents exceeds statutory cap of one month's rent (${} cents).",
                        input.security_deposit_held_cents, security_deposit_cap_cents
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            } else {
                Output {
                    mode: HawaiiHrs521Mode::CompliantSecurityDepositAtOrBelowOneMonthRent,
                    statutory_basis: "HRS § 521-44(b) — security deposit at or below one month's rent".to_string(),
                    notes: format!(
                        "COMPLIANT: security deposit ${} cents is at or below the one-month-rent cap (${} cents).",
                        input.security_deposit_held_cents, security_deposit_cap_cents
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            }
        }
        ComplianceAspect::SecurityDepositReturnWithin14Days => {
            if input.security_deposit_return_days_after_termination > HRS_521_SECURITY_DEPOSIT_RETURN_DAYS
            {
                Output {
                    mode: HawaiiHrs521Mode::ViolationSecurityDepositNotReturnedWithin14DaysTrebleDamagesExposure,
                    statutory_basis: "HRS § 521-44(c) + § 521-44(f) — 14-day return; willful retention triggers treble damages".to_string(),
                    notes: format!(
                        "VIOLATION: security deposit returned {} days after termination (> 14-day statutory deadline); willful retention without justification triggers treble (3X) damages exposure of ${} cents under § 521-44(f); tenant may file in Hawaii District Court or Small Claims Division.",
                        input.security_deposit_return_days_after_termination, treble_damages_exposure
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: treble_damages_exposure,
                }
            } else {
                Output {
                    mode: HawaiiHrs521Mode::CompliantSecurityDepositReturnedWithin14Days,
                    statutory_basis: "HRS § 521-44(c) — security deposit returned within 14 days".to_string(),
                    notes: format!(
                        "COMPLIANT: security deposit returned {} days after termination (≤ 14-day statutory window); itemized list of any deductions provided as required.",
                        input.security_deposit_return_days_after_termination
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            }
        }
        ComplianceAspect::RentIncreaseNotice45DaysMonthToMonth => {
            if input.rent_increase_notice_days_provided < HRS_521_RENT_INCREASE_NOTICE_DAYS_MONTH_TO_MONTH
            {
                Output {
                    mode: HawaiiHrs521Mode::ViolationRentIncreaseNoticeBelow45DaysMonthToMonth,
                    statutory_basis: "HRS § 521-21(d) — 45 consecutive days written notice required for month-to-month rent increase".to_string(),
                    notes: format!(
                        "VIOLATION: rent increase notice of {} days is below the 45-day statutory minimum; tenant may continue paying prior rent until proper notice given.",
                        input.rent_increase_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            } else {
                Output {
                    mode: HawaiiHrs521Mode::CompliantRentIncreaseNoticeAtOrAbove45DaysMonthToMonth,
                    statutory_basis: "HRS § 521-21(d) — 45-day rent increase notice satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT: rent increase notice of {} days satisfies the 45-day statutory requirement for month-to-month tenancy.",
                        input.rent_increase_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            }
        }
        ComplianceAspect::LandlordTerminationNotice45DaysMonthToMonth => {
            if input.landlord_termination_notice_days_provided
                < HRS_521_LANDLORD_TERMINATION_NOTICE_DAYS_MONTH_TO_MONTH
            {
                Output {
                    mode: HawaiiHrs521Mode::ViolationLandlordTerminationNoticeBelow45Days,
                    statutory_basis: "HRS § 521-71(a) — landlord must provide 45 days written notice for month-to-month termination".to_string(),
                    notes: format!(
                        "VIOLATION: landlord termination notice of {} days is below the 45-day statutory minimum for month-to-month tenancy.",
                        input.landlord_termination_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            } else {
                Output {
                    mode: HawaiiHrs521Mode::CompliantLandlordTerminationNoticeAtOrAbove45Days,
                    statutory_basis: "HRS § 521-71(a) — 45-day landlord termination notice satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT: landlord termination notice of {} days satisfies the 45-day statutory requirement.",
                        input.landlord_termination_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            }
        }
        ComplianceAspect::TenantTerminationNotice28DaysMonthToMonth => {
            if input.tenant_termination_notice_days_provided
                < HRS_521_TENANT_TERMINATION_NOTICE_DAYS_MONTH_TO_MONTH
            {
                Output {
                    mode: HawaiiHrs521Mode::ViolationTenantTerminationNoticeBelow28Days,
                    statutory_basis: "HRS § 521-71(b) — tenant must provide 28 days written notice for month-to-month termination".to_string(),
                    notes: format!(
                        "VIOLATION: tenant termination notice of {} days is below the 28-day statutory minimum; tenant remains responsible for full 28 days' rent regardless of move-out date.",
                        input.tenant_termination_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            } else {
                Output {
                    mode: HawaiiHrs521Mode::CompliantTenantTerminationNoticeAtOrAbove28Days,
                    statutory_basis: "HRS § 521-71(b) — 28-day tenant termination notice satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT: tenant termination notice of {} days satisfies the 28-day statutory requirement.",
                        input.tenant_termination_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            }
        }
        ComplianceAspect::DemolitionConversionNotice120Days => {
            if input.demolition_conversion_notice_days_provided
                < HRS_521_DEMOLITION_CONVERSION_NOTICE_DAYS
            {
                Output {
                    mode: HawaiiHrs521Mode::ViolationDemolitionConversionNoticeBelow120Days,
                    statutory_basis: "HRS § 521-71(d) — 120-day notice required for demolition or conversion".to_string(),
                    notes: format!(
                        "VIOLATION: demolition / conversion notice of {} days is below the 120-day statutory minimum.",
                        input.demolition_conversion_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            } else {
                Output {
                    mode: HawaiiHrs521Mode::CompliantDemolitionConversionNoticeAtOrAbove120Days,
                    statutory_basis: "HRS § 521-71(d) — 120-day demolition / conversion notice satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT: demolition / conversion notice of {} days satisfies the 120-day statutory requirement.",
                        input.demolition_conversion_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            }
        }
        ComplianceAspect::NonPaymentNotice5BusinessDays => {
            if input.non_payment_notice_business_days_provided < HRS_521_NON_PAYMENT_NOTICE_BUSINESS_DAYS
            {
                Output {
                    mode: HawaiiHrs521Mode::ViolationNonPaymentNoticeBelow5BusinessDays,
                    statutory_basis: "HRS § 521-68 — 5 business days notice required for non-payment of rent before eviction".to_string(),
                    notes: format!(
                        "VIOLATION: non-payment notice of {} business days is below the 5-business-day statutory minimum.",
                        input.non_payment_notice_business_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            } else {
                Output {
                    mode: HawaiiHrs521Mode::CompliantNonPaymentNoticeAtOrAbove5BusinessDays,
                    statutory_basis: "HRS § 521-68 — 5-business-day non-payment notice satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT: non-payment notice of {} business days satisfies the 5-business-day statutory requirement.",
                        input.non_payment_notice_business_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            }
        }
        ComplianceAspect::RuleViolationCureNotice10Days => {
            if input.rule_violation_cure_notice_days_provided < HRS_521_RULE_VIOLATION_CURE_NOTICE_DAYS
            {
                Output {
                    mode: HawaiiHrs521Mode::ViolationRuleViolationCureNoticeBelow10Days,
                    statutory_basis: "HRS § 521-72 — tenant has 10 days to cure rule violation after written notice".to_string(),
                    notes: format!(
                        "VIOLATION: rule violation cure notice of {} days is below the 10-day statutory minimum.",
                        input.rule_violation_cure_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            } else {
                Output {
                    mode: HawaiiHrs521Mode::CompliantRuleViolationCureNoticeAtOrAbove10Days,
                    statutory_basis: "HRS § 521-72 — 10-day rule violation cure notice satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT: rule violation cure notice of {} days satisfies the 10-day statutory requirement.",
                        input.rule_violation_cure_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            }
        }
        ComplianceAspect::LandlordEntryNotice2Days => {
            if input.landlord_entry_notice_days_provided < HRS_521_LANDLORD_ENTRY_NOTICE_DAYS {
                Output {
                    mode: HawaiiHrs521Mode::ViolationLandlordEntryNoticeBelow2Days,
                    statutory_basis: "HRS § 521-53 — 2 days advance notice required for landlord entry (except emergencies)".to_string(),
                    notes: format!(
                        "VIOLATION: landlord entry notice of {} days is below the 2-day statutory minimum (except emergency entries which are exempt).",
                        input.landlord_entry_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            } else {
                Output {
                    mode: HawaiiHrs521Mode::CompliantLandlordEntryNoticeAtOrAbove2Days,
                    statutory_basis: "HRS § 521-53 — 2-day landlord entry notice satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT: landlord entry notice of {} days satisfies the 2-day statutory requirement.",
                        input.landlord_entry_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            }
        }
        ComplianceAspect::EmergencyRepairsResponse3BusinessDays => {
            if input.emergency_repair_response_business_days > HRS_521_EMERGENCY_REPAIRS_BUSINESS_DAYS
            {
                Output {
                    mode: HawaiiHrs521Mode::ViolationEmergencyRepairsBeyond3BusinessDays,
                    statutory_basis: "HRS § 521-64 — emergency repairs must be made within 3 business days".to_string(),
                    notes: format!(
                        "VIOLATION: emergency repair response of {} business days exceeds the 3-business-day statutory deadline; tenant may invoke repair-and-deduct remedy under § 521-64(b).",
                        input.emergency_repair_response_business_days
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            } else {
                Output {
                    mode: HawaiiHrs521Mode::CompliantEmergencyRepairsWithin3BusinessDays,
                    statutory_basis: "HRS § 521-64 — emergency repairs completed within 3 business days".to_string(),
                    notes: format!(
                        "COMPLIANT: emergency repair response of {} business days satisfies the 3-business-day statutory requirement.",
                        input.emergency_repair_response_business_days
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            }
        }
        ComplianceAspect::GeneralRepairsResponse12BusinessDays => {
            if input.general_repair_response_business_days > HRS_521_GENERAL_REPAIRS_BUSINESS_DAYS {
                Output {
                    mode: HawaiiHrs521Mode::ViolationGeneralRepairsBeyond12BusinessDays,
                    statutory_basis: "HRS § 521-64 — general repairs must be made within 12 business days".to_string(),
                    notes: format!(
                        "VIOLATION: general repair response of {} business days exceeds the 12-business-day statutory deadline.",
                        input.general_repair_response_business_days
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            } else {
                Output {
                    mode: HawaiiHrs521Mode::CompliantGeneralRepairsWithin12BusinessDays,
                    statutory_basis: "HRS § 521-64 — general repairs completed within 12 business days".to_string(),
                    notes: format!(
                        "COMPLIANT: general repair response of {} business days satisfies the 12-business-day statutory requirement.",
                        input.general_repair_response_business_days
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_treble_damages_exposure_cents: 0,
                }
            }
        }
        ComplianceAspect::RetaliatoryEvictionProhibition => match input.retaliation_status {
            RetaliationStatus::NoRetaliatoryActionTaken => Output {
                mode: HawaiiHrs521Mode::CompliantNoRetaliatoryActionTaken,
                statutory_basis: "HRS § 521-74 — no retaliatory action taken".to_string(),
                notes: "COMPLIANT: no retaliatory action taken against tenant.".to_string(),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                statutory_treble_damages_exposure_cents: 0,
            },
            RetaliationStatus::RetaliatoryActionTakenAfterTenantHealthAgencyComplaint => Output {
                mode: HawaiiHrs521Mode::ViolationRetaliatoryEvictionAfterTenantHealthAgencyComplaint,
                statutory_basis: "HRS § 521-74 — retaliatory eviction / rent increase / service reduction prohibited".to_string(),
                notes: "VIOLATION: landlord took retaliatory action against tenant (rent increase / service reduction / termination) after tenant's good-faith complaint to health agency / code enforcement agency / governmental body; rebuttable presumption of retaliation within statutory lookback; tenant entitled to actual damages + statutory damages + reasonable attorney's fees + injunctive relief.".to_string(),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                statutory_treble_damages_exposure_cents: 0,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::WithinStateOfHawaii,
            unit_type: UnitType::StandardResidentialRentalCoveredByHrs521,
            compliance_aspect: ComplianceAspect::SecurityDepositCapOneMonth,
            monthly_rent_cents: 250_000,
            security_deposit_held_cents: 250_000,
            security_deposit_return_days_after_termination: 10,
            rent_increase_notice_days_provided: 45,
            landlord_termination_notice_days_provided: 45,
            tenant_termination_notice_days_provided: 28,
            demolition_conversion_notice_days_provided: 120,
            non_payment_notice_business_days_provided: 5,
            rule_violation_cure_notice_days_provided: 10,
            landlord_entry_notice_days_provided: 2,
            emergency_repair_response_business_days: 3,
            general_repair_response_business_days: 12,
            retaliation_status: RetaliationStatus::NoRetaliatoryActionTaken,
        }
    }

    #[test]
    fn property_outside_hawaii_not_applicable() {
        let mut input = baseline_input();
        input.property_jurisdiction = PropertyJurisdiction::OutsideStateOfHawaii;
        let output = check(&input);
        assert_eq!(output.mode, HawaiiHrs521Mode::NotApplicablePropertyOutsideHawaii);
    }

    #[test]
    fn transient_lodging_under_90_days_exempt() {
        let mut input = baseline_input();
        input.unit_type = UnitType::TransientLodgingUnder90DaysExemptUnderHrs521_8;
        let output = check(&input);
        assert_eq!(output.mode, HawaiiHrs521Mode::NotApplicableTransientLodgingExempt);
    }

    #[test]
    fn non_residential_unit_not_applicable() {
        let mut input = baseline_input();
        input.unit_type = UnitType::NonResidentialUnitExempt;
        let output = check(&input);
        assert_eq!(output.mode, HawaiiHrs521Mode::NotApplicableNonResidentialUnit);
    }

    #[test]
    fn security_deposit_at_one_month_cap_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantSecurityDepositAtOrBelowOneMonthRent
        );
        assert_eq!(output.statutory_security_deposit_cap_cents, 250_000);
    }

    #[test]
    fn security_deposit_exceeds_one_month_cap_violation() {
        let mut input = baseline_input();
        input.security_deposit_held_cents = 500_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::ViolationSecurityDepositExceedsOneMonthRent
        );
    }

    #[test]
    fn security_deposit_returned_within_14_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnWithin14Days;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantSecurityDepositReturnedWithin14Days
        );
    }

    #[test]
    fn security_deposit_at_exactly_14_days_compliant_boundary() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnWithin14Days;
        input.security_deposit_return_days_after_termination = 14;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantSecurityDepositReturnedWithin14Days
        );
    }

    #[test]
    fn security_deposit_at_15_days_treble_damages_violation_boundary() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnWithin14Days;
        input.security_deposit_return_days_after_termination = 15;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::ViolationSecurityDepositNotReturnedWithin14DaysTrebleDamagesExposure
        );
        // 3x $250,000 = $750,000 treble exposure
        assert_eq!(output.statutory_treble_damages_exposure_cents, 750_000);
    }

    #[test]
    fn rent_increase_notice_at_45_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentIncreaseNotice45DaysMonthToMonth;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantRentIncreaseNoticeAtOrAbove45DaysMonthToMonth
        );
    }

    #[test]
    fn rent_increase_notice_below_45_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentIncreaseNotice45DaysMonthToMonth;
        input.rent_increase_notice_days_provided = 30;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::ViolationRentIncreaseNoticeBelow45DaysMonthToMonth
        );
    }

    #[test]
    fn landlord_termination_notice_at_45_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordTerminationNotice45DaysMonthToMonth;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantLandlordTerminationNoticeAtOrAbove45Days
        );
    }

    #[test]
    fn tenant_termination_notice_at_28_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenantTerminationNotice28DaysMonthToMonth;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantTenantTerminationNoticeAtOrAbove28Days
        );
    }

    #[test]
    fn demolition_conversion_notice_at_120_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DemolitionConversionNotice120Days;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantDemolitionConversionNoticeAtOrAbove120Days
        );
    }

    #[test]
    fn demolition_conversion_notice_below_120_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DemolitionConversionNotice120Days;
        input.demolition_conversion_notice_days_provided = 90;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::ViolationDemolitionConversionNoticeBelow120Days
        );
    }

    #[test]
    fn non_payment_notice_at_5_business_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonPaymentNotice5BusinessDays;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantNonPaymentNoticeAtOrAbove5BusinessDays
        );
    }

    #[test]
    fn rule_violation_cure_notice_at_10_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RuleViolationCureNotice10Days;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantRuleViolationCureNoticeAtOrAbove10Days
        );
    }

    #[test]
    fn landlord_entry_notice_at_2_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNotice2Days;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantLandlordEntryNoticeAtOrAbove2Days
        );
    }

    #[test]
    fn landlord_entry_notice_below_2_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryNotice2Days;
        input.landlord_entry_notice_days_provided = 1;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::ViolationLandlordEntryNoticeBelow2Days
        );
    }

    #[test]
    fn emergency_repairs_within_3_business_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EmergencyRepairsResponse3BusinessDays;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantEmergencyRepairsWithin3BusinessDays
        );
    }

    #[test]
    fn emergency_repairs_beyond_3_business_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EmergencyRepairsResponse3BusinessDays;
        input.emergency_repair_response_business_days = 7;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::ViolationEmergencyRepairsBeyond3BusinessDays
        );
    }

    #[test]
    fn general_repairs_within_12_business_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::GeneralRepairsResponse12BusinessDays;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::CompliantGeneralRepairsWithin12BusinessDays
        );
    }

    #[test]
    fn general_repairs_beyond_12_business_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::GeneralRepairsResponse12BusinessDays;
        input.general_repair_response_business_days = 20;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::ViolationGeneralRepairsBeyond12BusinessDays
        );
    }

    #[test]
    fn retaliatory_eviction_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RetaliatoryEvictionProhibition;
        input.retaliation_status =
            RetaliationStatus::RetaliatoryActionTakenAfterTenantHealthAgencyComplaint;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HawaiiHrs521Mode::ViolationRetaliatoryEvictionAfterTenantHealthAgencyComplaint
        );
    }

    #[test]
    fn no_retaliation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RetaliatoryEvictionProhibition;
        let output = check(&input);
        assert_eq!(output.mode, HawaiiHrs521Mode::CompliantNoRetaliatoryActionTaken);
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(HRS_521_SECURITY_DEPOSIT_CAP_MONTHS_RENT, 1);
        assert_eq!(HRS_521_SECURITY_DEPOSIT_RETURN_DAYS, 14);
        assert_eq!(HRS_521_WILLFUL_RETENTION_DAMAGES_MULTIPLIER, 3);
        assert_eq!(HRS_521_RENT_INCREASE_NOTICE_DAYS_MONTH_TO_MONTH, 45);
        assert_eq!(HRS_521_RENT_INCREASE_NOTICE_DAYS_WEEK_TO_WEEK, 15);
        assert_eq!(HRS_521_LANDLORD_TERMINATION_NOTICE_DAYS_MONTH_TO_MONTH, 45);
        assert_eq!(HRS_521_TENANT_TERMINATION_NOTICE_DAYS_MONTH_TO_MONTH, 28);
        assert_eq!(HRS_521_DEMOLITION_CONVERSION_NOTICE_DAYS, 120);
        assert_eq!(HRS_521_NON_PAYMENT_NOTICE_BUSINESS_DAYS, 5);
        assert_eq!(HRS_521_RULE_VIOLATION_CURE_NOTICE_DAYS, 10);
        assert_eq!(HRS_521_LANDLORD_ENTRY_NOTICE_DAYS, 2);
        assert_eq!(HRS_521_EMERGENCY_REPAIRS_BUSINESS_DAYS, 3);
        assert_eq!(HRS_521_GENERAL_REPAIRS_BUSINESS_DAYS, 12);
        assert_eq!(HRS_521_TRANSIENT_LODGING_THRESHOLD_DAYS, 90);
        assert_eq!(HRS_521_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("HRS Chapter 521"));
        assert!(joined.contains("§ 521-44"));
        assert!(joined.contains("§ 521-21"));
        assert!(joined.contains("§ 521-71"));
        assert!(joined.contains("§ 521-68"));
        assert!(joined.contains("§ 521-72"));
        assert!(joined.contains("§ 521-53"));
        assert!(joined.contains("§ 521-64"));
        assert!(joined.contains("§ 521-74"));
        assert!(joined.contains("§ 521-8"));
        assert!(joined.contains("ONE MONTH'S RENT"));
        assert!(joined.contains("14 DAYS"));
        assert!(joined.contains("45 CONSECUTIVE DAYS"));
        assert!(joined.contains("45 DAYS"));
        assert!(joined.contains("28 DAYS"));
        assert!(joined.contains("120-DAY"));
        assert!(joined.contains("5 BUSINESS DAYS"));
        assert!(joined.contains("10 DAYS"));
        assert!(joined.contains("2 DAYS"));
        assert!(joined.contains("TREBLE"));
    }
}
