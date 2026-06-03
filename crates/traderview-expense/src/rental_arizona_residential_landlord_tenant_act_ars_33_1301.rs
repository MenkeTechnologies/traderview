//! Arizona Residential Landlord and Tenant Act (ARLTA)
//! Compliance Module — codified at A.R.S. §§ 33-1301
//! through 33-1381 (Title 33 Property, Chapter 10
//! Residential Landlord and Tenant Act). Pure-compute
//! check for trader-landlord compliance with Arizona's
//! foundational statewide residential tenancy regime.
//!
//! Adopted by the Arizona Legislature in **1973** based on
//! the **Uniform Residential Landlord and Tenant Act
//! (URLTA)**. Arizona is a major Sun Belt rental market
//! covering metropolitan Phoenix (5th-largest US city),
//! Tucson, Mesa, Chandler, Scottsdale, Glendale, Gilbert,
//! and Tempe. The Act is supplemented by the **Arizona
//! Forcible Entry and Detainer statute (A.R.S. § 12-1171
//! et seq.)** for eviction procedure.
//!
//! Web research (verified 2026-06-03):
//! - **Adoption**: Arizona Residential Landlord and Tenant Act adopted by Arizona Legislature in **1973** based on Uniform Residential Landlord and Tenant Act (URLTA); codified at **A.R.S. §§ 33-1301 through 33-1381** (Title 33 Property, Chapter 10 Residential Landlord and Tenant Act) ([Arizona Legislature — A.R.S. § 33-1321 Security Deposits](https://www.azleg.gov/ars/33/01321.htm); [Arizona Legislature — A.R.S. § 33-1324 Landlord to Maintain Fit Premises](https://www.azleg.gov/ars/33/01324.htm); [Arizona Legislature — Title 33 Chapter 16](https://www.azleg.gov/arsDetail/?title=33); [Arizona Department of Housing — Arizona Residential Landlord and Tenant Act Updated May 2023 PDF](https://housing.az.gov/sites/default/files/2024-07/Landlord_Tenant_Act_May-2023_1.pdf); [University of Arizona Law Library — Arizona Residential Landlord-Tenant Law LibGuide](https://law-arizona.libguides.com/c.php?g=1270587&p=9319718); [Community Legal Services AZ — Arizona Tenants' Rights & Responsibilities Handbook](https://clsaz.org/wp-content/uploads/2019/10/az-tenants-english-191005.pdf); [Justia — 2025 Arizona Revised Statutes Title 33 Property](https://law.justia.com/codes/arizona/title-33/); [Justia — 2025 Arizona Revised Statutes § 33-1431 Security Deposits](https://law.justia.com/codes/arizona/title-33/section-33-1431/); [Morris Institute for Justice — ARLTA Training Presentation PDF](https://morrisinstituteforjustice.org/helpful-information/landlord-and-tenant/3-landlord-and-tenant-law-training-presentation/file); [Tenant.net — Arizona Residential Landlord-Tenant Act](http://tenant.net/Other_Areas/Arizona/AZresidential.html); [FreeNetLaw — Tenant Rights in Arizona](https://freenetlaw.com/tenant-rights/arizona/); [Innago — Arizona Landlord Tenant Rental Laws 2025](https://innago.com/arizona-landlord-tenant-laws/); [St. Clair Law Tucson — Landlord and Tenant Disputes Rights Remedies and Eviction in Arizona](https://www.stclairlawtucson.com/real-estate/landlord-and-tenant-disputes-rights-remedies-and-eviction-in-arizona/); [Arizona Legal Center — Notice Requirements Security Deposit Refunds Repairs and Other Tenant Rights](https://arizonalegalcenter.org/notice-requirements-security-deposit-refunds-repairs-and-other-tenant-rights-in-arizona/); [iPropertyManagement — Arizona Security Deposit Law Returns Interest Deductions](https://ipropertymanagement.com/laws/arizona-security-deposit-returns); [Nolo — Arizona Landlord-Tenant Laws Complete Guide](https://www.nolo.com/legal-encyclopedia/overview-landlord-tenant-laws-arizona.html); [Rentable — Arizona Security Deposit Laws Complete Guide](https://www.rentable.com/blog/arizona-security-deposit-laws-a-complete-guide-for-landlords-tenants/); [Landlord Studio — Arizona Landlord Tenant Laws](https://www.landlordstudio.com/landlord-tenant-laws/arizona-landlord-tenant-laws/)).
//! - **A.R.S. § 33-1321 Security Deposits**: landlord shall NOT demand or receive security (including prepaid rent) in an amount or value of more than **ONE AND ONE-HALF MONTH'S RENT (1.5 MONTHS)**; within **14 DAYS (excluding Saturdays, Sundays, or other legal holidays — i.e., 14 BUSINESS DAYS)** after termination of tenancy and delivery of possession and demand by tenant, landlord shall provide itemized list of deductions together with amount due and payable to tenant.
//! - **A.R.S. § 33-1324 Landlord Obligation to Maintain Premises**: landlord must comply with applicable building codes materially affecting health and safety; make all repairs to keep premises in fit and habitable condition; keep common areas clean and safe; maintain in good and safe working order all electrical, plumbing, sanitary, heating, ventilating, air-conditioning, and other facilities and appliances supplied by landlord.
//! - **A.R.S. § 33-1343 Landlord Access**: landlord must give tenant **TWO DAYS' (48 HOURS) NOTICE** before entering dwelling unit for inspection, repairs, decorations, alterations, improvements, supplying agreed services, or exhibiting to prospective purchasers / mortgagees / tenants / workers / contractors; landlord may enter immediately only in **EMERGENCY** or by court order.
//! - **A.R.S. § 33-1368(B) Five-Day Pay or Quit Notice (Nonpayment)**: for nonpayment of rent, landlord must give tenant **5-DAY WRITTEN NOTICE** specifying acts and omissions constituting the breach AND that the rental agreement will terminate upon a date not less than 5 days after receipt of notice.
//! - **A.R.S. § 33-1368(A) Ten-Day Cure Notice (Material Noncompliance)**: for material noncompliance with rental agreement or A.R.S. § 33-1341 obligations, landlord must give tenant **10-DAY WRITTEN NOTICE** specifying acts and omissions constituting the breach AND that the rental agreement will terminate upon a date not less than 10 days after receipt of notice unless breach is remedied; **5-DAY CURE PERIOD** applies for material noncompliance affecting **HEALTH AND SAFETY**.
//! - **A.R.S. § 33-1376 Retaliation Prohibited**: landlord may NOT retaliate by increasing rent, decreasing services, or bringing or threatening to bring an action for possession after tenant has (a) complained to a governmental agency about violation of building or housing code materially affecting health and safety; (b) complained to landlord of violation under § 33-1324; (c) organized or become member of a tenants' union or similar organization.
//! - **A.R.S. § 33-1377 Retaliation Presumption**: presumption of retaliation arises if landlord adverse action occurs within **6 MONTHS** of tenant's protected activity.
//! - **A.R.S. § 33-1370 Tenant Remedies for Unlawful Ouster**: tenant whom landlord unlawfully removes from premises, excludes from premises, willfully diminishes services to (such as electricity, gas, water, etc.) may recover possession or terminate the rental agreement, and recover an amount equal to actual damages or two months' periodic rent, whichever is greater.
//! - **A.R.S. § 33-1361 Tenant Termination Remedies**: tenant may terminate rental agreement after giving landlord 14-day notice if landlord materially fails to comply with rental agreement or § 33-1324; tenant may recover damages, obtain injunctive relief, deduct from rent the actual diminution in fair rental value, or recover reasonable attorney's fees.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const AZ_RLTA_TITLE_NUMBER: u32 = 33;
pub const AZ_RLTA_CHAPTER_NUMBER: u32 = 10;
pub const AZ_RLTA_ENACTMENT_YEAR: u32 = 1973;
pub const AZ_RLTA_SECURITY_DEPOSIT_CAP_NUMERATOR_TENTHS_OF_MONTHS_RENT: u64 = 15;
pub const AZ_RLTA_SECURITY_DEPOSIT_CAP_DENOMINATOR_TENTHS: u64 = 10;
pub const AZ_RLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_BUSINESS_DAYS: u32 = 14;
pub const AZ_RLTA_LANDLORD_ENTRY_NOTICE_HOURS: u32 = 48;
pub const AZ_RLTA_PAY_OR_QUIT_NOTICE_DAYS: u32 = 5;
pub const AZ_RLTA_MATERIAL_NONCOMPLIANCE_CURE_PERIOD_DAYS: u32 = 10;
pub const AZ_RLTA_HEALTH_SAFETY_CURE_PERIOD_DAYS: u32 = 5;
pub const AZ_RLTA_RETALIATION_PRESUMPTION_WINDOW_MONTHS: u32 = 6;
pub const AZ_RLTA_TENANT_TERMINATION_NOTICE_DAYS: u32 = 14;
pub const AZ_RLTA_UNLAWFUL_OUSTER_TWO_MONTHS_DAMAGES_MULTIPLIER: u64 = 2;
pub const AZ_RLTA_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    ResidentialRentalCoveredByArlta,
    CommercialRentalExempt,
    HotelMotelTransientLodgingExempt,
    InstitutionalCareFacilityExempt,
    PublicHousingExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialBreachCategory {
    NonpaymentOfRent,
    MaterialNoncomplianceGeneralBreach,
    MaterialNoncomplianceHealthAndSafetyBreach,
    NoBreachAlleged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositCapOneAndOneHalfMonthsRentUnderArs33_1321,
    SecurityDepositReturnFourteenBusinessDayDeadlineUnderArs33_1321,
    LandlordObligationToMaintainPremisesUnderArs33_1324,
    LandlordEntryFortyEightHourNoticeUnderArs33_1343,
    FiveDayPayOrQuitNoticeUnderArs33_1368B,
    TenDayCureOrFiveDayHealthSafetyCureNoticeUnderArs33_1368A,
    RetaliationProhibitedSixMonthPresumptionUnderArs33_1376_1377,
    UnlawfulOusterTwoMonthsDamagesRemedyUnderArs33_1370,
    TenantTerminationFourteenDayNoticeUnderArs33_1361,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArltaMode {
    NotApplicableTenancyExemptFromArlta,
    CompliantSecurityDepositAtOrBelowOneAndOneHalfMonthsRentCap,
    CompliantSecurityDepositReturnedWithinFourteenBusinessDays,
    CompliantLandlordObligationsMet,
    CompliantLandlordEntryFortyEightHourNoticeOrEmergency,
    CompliantFiveDayPayOrQuitNoticeProvided,
    CompliantTenDayCureOrFiveDayHealthSafetyCureNoticeProvided,
    CompliantNoRetaliationWithinSixMonthsOfProtectedActivity,
    CompliantNoUnlawfulOusterTenantInPossession,
    CompliantTenantTerminationFourteenDayNoticeProvided,
    ViolationSecurityDepositExceedsOneAndOneHalfMonthsRentCap,
    ViolationSecurityDepositReturnedPastFourteenBusinessDayDeadline,
    ViolationLandlordObligationsBreached,
    ViolationLandlordEntryWithoutFortyEightHourNoticeAndNotEmergency,
    ViolationPayOrQuitNoticePeriodShorterThanFiveDays,
    ViolationCureNoticePeriodShorterThanStatutoryMinimum,
    ViolationRetaliatoryConductWithinSixMonthsOfProtectedActivity,
    ViolationUnlawfulOusterTriggersTwoMonthsRentOrActualDamages,
    ViolationTenantTerminationNoticeShorterThanFourteenDays,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_type: TenancyType,
    pub compliance_aspect: ComplianceAspect,
    pub material_breach_category: MaterialBreachCategory,
    pub monthly_rent_dollars: u64,
    pub security_deposit_plus_prepaid_rent_dollars: u64,
    pub business_days_since_tenant_demanded_deposit_return: u32,
    pub deposit_returned_with_itemized_statement_within_window: bool,
    pub landlord_obligations_under_section_33_1324_met: bool,
    pub landlord_entry_notice_hours_given: u32,
    pub entry_was_emergency_or_court_order: bool,
    pub pay_or_quit_notice_days_given: u32,
    pub cure_notice_days_given: u32,
    pub protected_activity_within_six_months: bool,
    pub adverse_action_taken: bool,
    pub unlawful_ouster_occurred: bool,
    pub tenant_termination_notice_days_given: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: ArltaMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub two_months_rent_unlawful_ouster_damages_dollars: u64,
}

pub type RentalArizonaResidentialLandlordTenantActArs33_1301Input = Input;
pub type RentalArizonaResidentialLandlordTenantActArs33_1301Output = Output;
pub type RentalArizonaResidentialLandlordTenantActArs33_1301Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Arizona Residential Landlord and Tenant Act — adopted by Arizona Legislature in 1973 based on Uniform Residential Landlord and Tenant Act (URLTA); codified at A.R.S. §§ 33-1301 through 33-1381 (Title 33 Property, Chapter 10 Residential Landlord and Tenant Act)".to_string(),
        "A.R.S. § 33-1321 Security Deposits — landlord shall NOT demand or receive security (including prepaid rent) in an amount or value of more than ONE AND ONE-HALF MONTH'S RENT (1.5 MONTHS); within 14 DAYS (excluding Saturdays, Sundays, or other legal holidays — i.e., 14 BUSINESS DAYS) after termination of tenancy and delivery of possession and demand by tenant, landlord shall provide itemized list of deductions together with amount due and payable to tenant".to_string(),
        "A.R.S. § 33-1324 Landlord Obligation to Maintain Premises — landlord must comply with applicable building codes materially affecting health and safety; make all repairs to keep premises in fit and habitable condition; keep common areas clean and safe; maintain in good and safe working order all electrical, plumbing, sanitary, heating, ventilating, air-conditioning, and other facilities and appliances".to_string(),
        "A.R.S. § 33-1341 Tenant Obligations — tenant must comply with building and housing codes materially affecting health and safety; keep premises clean and safe; dispose of rubbish; keep plumbing fixtures clean; use facilities and appliances reasonably; not damage premises; conduct themselves and require guests to conduct themselves in a manner not disturbing neighbors".to_string(),
        "A.R.S. § 33-1343 Landlord Access — landlord must give tenant TWO DAYS' (48 HOURS) NOTICE before entering dwelling unit for inspection, repairs, decorations, alterations, improvements, supplying agreed services, or exhibiting to prospective purchasers / mortgagees / tenants / workers / contractors; landlord may enter immediately only in EMERGENCY or by court order".to_string(),
        "A.R.S. § 33-1361 Tenant Termination Remedies — tenant may terminate rental agreement after giving landlord 14-DAY NOTICE if landlord materially fails to comply with rental agreement or § 33-1324; tenant may recover damages, obtain injunctive relief, deduct from rent the actual diminution in fair rental value, or recover reasonable attorney's fees".to_string(),
        "A.R.S. § 33-1368(A) Ten-Day Cure Notice (Material Noncompliance) — for material noncompliance with rental agreement or § 33-1341 obligations, landlord must give tenant 10-DAY WRITTEN NOTICE specifying acts and omissions constituting the breach AND that the rental agreement will terminate upon a date not less than 10 days after receipt of notice unless breach is remedied; 5-DAY CURE PERIOD applies for material noncompliance affecting HEALTH AND SAFETY".to_string(),
        "A.R.S. § 33-1368(B) Five-Day Pay or Quit Notice (Nonpayment) — for nonpayment of rent, landlord must give tenant 5-DAY WRITTEN NOTICE specifying acts and omissions constituting the breach AND that the rental agreement will terminate upon a date not less than 5 days after receipt of notice".to_string(),
        "A.R.S. § 33-1370 Tenant Remedies for Unlawful Ouster — tenant whom landlord unlawfully removes from premises, excludes from premises, or willfully diminishes services to (such as electricity, gas, water, etc.) may recover possession or terminate the rental agreement, and recover an amount EQUAL TO ACTUAL DAMAGES OR TWO MONTHS' PERIODIC RENT, WHICHEVER IS GREATER".to_string(),
        "A.R.S. § 33-1376 Retaliation Prohibited — landlord may NOT retaliate by increasing rent, decreasing services, or bringing or threatening to bring an action for possession after tenant has (a) complained to a governmental agency about violation of building or housing code materially affecting health and safety; (b) complained to landlord of violation under § 33-1324; (c) organized or become member of a tenants' union or similar organization".to_string(),
        "A.R.S. § 33-1377 Retaliation Presumption — presumption of retaliation arises if landlord adverse action occurs within 6 MONTHS of tenant's protected activity".to_string(),
        "Arizona Forcible Entry and Detainer (A.R.S. § 12-1171 et seq.) — eviction procedure following service of § 33-1368 notice; jurisdiction in justice court (under $10,000) or superior court (over $10,000)".to_string(),
        "Arizona Department of Housing — Arizona Residential Landlord and Tenant Act Updated May 2023 (official text)".to_string(),
        "University of Arizona Law Library + Arizona Legal Center + Morris Institute for Justice + Community Legal Services AZ + Nolo + Innago + iPropertyManagement + Rentable + Landlord Studio + Tenant.net + FreeNetLaw + St. Clair Law Tucson — practitioner overviews of ARLTA".to_string(),
    ];

    if input.tenancy_type != TenancyType::ResidentialRentalCoveredByArlta {
        return Output {
            mode: ArltaMode::NotApplicableTenancyExemptFromArlta,
            statutory_basis: "A.R.S. § 33-1308 — Arizona Residential Landlord and Tenant Act applies only to residential leaseholds; commercial / hotel-motel / institutional / public housing exempt".to_string(),
            notes: "NOT APPLICABLE: tenancy type exempt from ARLTA (commercial rental; hotel/motel transient lodging; institutional care facility; public housing).".to_string(),
            citations,
            two_months_rent_unlawful_ouster_damages_dollars: 0,
        };
    }

    let unlawful_ouster_damages = input
        .monthly_rent_dollars
        .saturating_mul(AZ_RLTA_UNLAWFUL_OUSTER_TWO_MONTHS_DAMAGES_MULTIPLIER);

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositCapOneAndOneHalfMonthsRentUnderArs33_1321 => {
            let cap = (u128::from(input.monthly_rent_dollars)
                * u128::from(AZ_RLTA_SECURITY_DEPOSIT_CAP_NUMERATOR_TENTHS_OF_MONTHS_RENT)
                / u128::from(AZ_RLTA_SECURITY_DEPOSIT_CAP_DENOMINATOR_TENTHS)) as u64;
            if input.security_deposit_plus_prepaid_rent_dollars <= cap {
                Output {
                    mode: ArltaMode::CompliantSecurityDepositAtOrBelowOneAndOneHalfMonthsRentCap,
                    statutory_basis: "A.R.S. § 33-1321 — security deposit (including prepaid rent) at or below 1.5-months'-rent statutory cap".to_string(),
                    notes: "COMPLIANT: security deposit (including prepaid rent) at or below 1.5 months' rent statutory cap under A.R.S. § 33-1321.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            } else {
                Output {
                    mode: ArltaMode::ViolationSecurityDepositExceedsOneAndOneHalfMonthsRentCap,
                    statutory_basis: "A.R.S. § 33-1321 — security deposit (including prepaid rent) exceeds 1.5-months'-rent statutory cap".to_string(),
                    notes: "VIOLATION: security deposit (including prepaid rent) exceeds 1.5 months' rent cap under A.R.S. § 33-1321; landlord must refund excess.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            }
        }
        ComplianceAspect::SecurityDepositReturnFourteenBusinessDayDeadlineUnderArs33_1321 => {
            if input.deposit_returned_with_itemized_statement_within_window
                && input.business_days_since_tenant_demanded_deposit_return
                    <= AZ_RLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_BUSINESS_DAYS
            {
                Output {
                    mode: ArltaMode::CompliantSecurityDepositReturnedWithinFourteenBusinessDays,
                    statutory_basis: "A.R.S. § 33-1321 — security deposit returned with itemized statement within 14-business-day statutory deadline".to_string(),
                    notes: "COMPLIANT: landlord returned security deposit with itemized statement of deductions within 14 BUSINESS DAYS (excluding Saturdays, Sundays, and legal holidays) after tenant's termination, delivery of possession, and demand under A.R.S. § 33-1321.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            } else {
                Output {
                    mode: ArltaMode::ViolationSecurityDepositReturnedPastFourteenBusinessDayDeadline,
                    statutory_basis: "A.R.S. § 33-1321 — security deposit not returned within 14-business-day statutory deadline".to_string(),
                    notes: "VIOLATION: landlord missed 14-business-day security deposit return deadline under A.R.S. § 33-1321; tenant may seek damages including reasonable attorney's fees.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            }
        }
        ComplianceAspect::LandlordObligationToMaintainPremisesUnderArs33_1324 => {
            if input.landlord_obligations_under_section_33_1324_met {
                Output {
                    mode: ArltaMode::CompliantLandlordObligationsMet,
                    statutory_basis: "A.R.S. § 33-1324 — landlord obligations met".to_string(),
                    notes: "COMPLIANT: landlord complies with applicable building codes affecting health and safety; maintains premises in fit and habitable condition; keeps common areas clean and safe; maintains facilities and appliances in good working order.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            } else {
                Output {
                    mode: ArltaMode::ViolationLandlordObligationsBreached,
                    statutory_basis: "A.R.S. § 33-1324 — landlord obligations breached".to_string(),
                    notes: "VIOLATION: landlord breached § 33-1324 maintenance obligations; tenant remedies under § 33-1361 (14-day termination notice + damages + injunctive relief + rent reduction + attorney's fees).".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            }
        }
        ComplianceAspect::LandlordEntryFortyEightHourNoticeUnderArs33_1343 => {
            if input.entry_was_emergency_or_court_order
                || input.landlord_entry_notice_hours_given >= AZ_RLTA_LANDLORD_ENTRY_NOTICE_HOURS
            {
                Output {
                    mode: ArltaMode::CompliantLandlordEntryFortyEightHourNoticeOrEmergency,
                    statutory_basis: "A.R.S. § 33-1343 — 48-hour entry notice OR emergency / court order exception".to_string(),
                    notes: "COMPLIANT: landlord provided at least 48 hours' (2 days') notice prior to entering dwelling unit, OR entry was during emergency or under court order under A.R.S. § 33-1343.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            } else {
                Output {
                    mode: ArltaMode::ViolationLandlordEntryWithoutFortyEightHourNoticeAndNotEmergency,
                    statutory_basis: "A.R.S. § 33-1343 — landlord entry without 48-hour notice and not emergency".to_string(),
                    notes: "VIOLATION: landlord entered without 48 hours' notice and not under emergency or court order under A.R.S. § 33-1343; tenant may obtain injunctive relief.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            }
        }
        ComplianceAspect::FiveDayPayOrQuitNoticeUnderArs33_1368B => {
            if input.pay_or_quit_notice_days_given >= AZ_RLTA_PAY_OR_QUIT_NOTICE_DAYS {
                Output {
                    mode: ArltaMode::CompliantFiveDayPayOrQuitNoticeProvided,
                    statutory_basis: "A.R.S. § 33-1368(B) — 5-day pay or quit notice provided for nonpayment".to_string(),
                    notes: "COMPLIANT: landlord provided 5-day pay or quit written notice under A.R.S. § 33-1368(B) for nonpayment of rent.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            } else {
                Output {
                    mode: ArltaMode::ViolationPayOrQuitNoticePeriodShorterThanFiveDays,
                    statutory_basis: "A.R.S. § 33-1368(B) — pay or quit notice period shorter than 5-day statutory minimum".to_string(),
                    notes: "VIOLATION: pay or quit notice period shorter than 5-day statutory minimum under A.R.S. § 33-1368(B); special detainer action subject to dismissal.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            }
        }
        ComplianceAspect::TenDayCureOrFiveDayHealthSafetyCureNoticeUnderArs33_1368A => {
            let required_days = match input.material_breach_category {
                MaterialBreachCategory::MaterialNoncomplianceHealthAndSafetyBreach => {
                    AZ_RLTA_HEALTH_SAFETY_CURE_PERIOD_DAYS
                }
                _ => AZ_RLTA_MATERIAL_NONCOMPLIANCE_CURE_PERIOD_DAYS,
            };
            if input.cure_notice_days_given >= required_days {
                Output {
                    mode: ArltaMode::CompliantTenDayCureOrFiveDayHealthSafetyCureNoticeProvided,
                    statutory_basis: "A.R.S. § 33-1368(A) — cure notice meets statutory minimum for breach category".to_string(),
                    notes: "COMPLIANT: cure notice meets statutory minimum under A.R.S. § 33-1368(A) (10 days for general material noncompliance; 5 days for health and safety breach).".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            } else {
                Output {
                    mode: ArltaMode::ViolationCureNoticePeriodShorterThanStatutoryMinimum,
                    statutory_basis: "A.R.S. § 33-1368(A) — cure notice shorter than statutory minimum".to_string(),
                    notes: "VIOLATION: cure notice shorter than statutory minimum under A.R.S. § 33-1368(A) (10 days for general material noncompliance OR 5 days for health and safety breach); special detainer action subject to dismissal.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            }
        }
        ComplianceAspect::RetaliationProhibitedSixMonthPresumptionUnderArs33_1376_1377 => {
            if input.protected_activity_within_six_months && input.adverse_action_taken {
                Output {
                    mode: ArltaMode::ViolationRetaliatoryConductWithinSixMonthsOfProtectedActivity,
                    statutory_basis: "A.R.S. § 33-1376 + § 33-1377 — retaliatory conduct within 6-month presumption window".to_string(),
                    notes: "VIOLATION: landlord engaged in adverse action (rent raise / service reduction / possession action / threat thereof) within 6-month retaliation presumption window after tenant's protected activity (governmental agency complaint / § 33-1324 complaint to landlord / tenants' union organization).".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            } else {
                Output {
                    mode: ArltaMode::CompliantNoRetaliationWithinSixMonthsOfProtectedActivity,
                    statutory_basis: "A.R.S. § 33-1377 — no retaliatory conduct presumption arises".to_string(),
                    notes: "COMPLIANT: no adverse action within 6-month retaliation window OR no protected tenant activity to trigger A.R.S. § 33-1377 presumption.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            }
        }
        ComplianceAspect::UnlawfulOusterTwoMonthsDamagesRemedyUnderArs33_1370 => {
            if input.unlawful_ouster_occurred {
                Output {
                    mode: ArltaMode::ViolationUnlawfulOusterTriggersTwoMonthsRentOrActualDamages,
                    statutory_basis: "A.R.S. § 33-1370 — unlawful ouster triggers two months' rent OR actual damages remedy".to_string(),
                    notes: "VIOLATION: landlord unlawfully removed tenant, excluded tenant from premises, or willfully diminished services; A.R.S. § 33-1370 entitles tenant to recover possession or terminate AND recover ACTUAL DAMAGES OR TWO MONTHS' PERIODIC RENT WHICHEVER IS GREATER plus reasonable attorney's fees.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: unlawful_ouster_damages,
                }
            } else {
                Output {
                    mode: ArltaMode::CompliantNoUnlawfulOusterTenantInPossession,
                    statutory_basis: "A.R.S. § 33-1370 — no unlawful ouster".to_string(),
                    notes: "COMPLIANT: no unlawful ouster; tenant remains in lawful possession; § 33-1370 damages remedy does not attach.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            }
        }
        ComplianceAspect::TenantTerminationFourteenDayNoticeUnderArs33_1361 => {
            if input.tenant_termination_notice_days_given >= AZ_RLTA_TENANT_TERMINATION_NOTICE_DAYS {
                Output {
                    mode: ArltaMode::CompliantTenantTerminationFourteenDayNoticeProvided,
                    statutory_basis: "A.R.S. § 33-1361 — tenant termination 14-day notice provided".to_string(),
                    notes: "COMPLIANT: tenant provided 14-day written notice to terminate rental agreement under A.R.S. § 33-1361 for landlord's material noncompliance with rental agreement or § 33-1324 obligations.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
                }
            } else {
                Output {
                    mode: ArltaMode::ViolationTenantTerminationNoticeShorterThanFourteenDays,
                    statutory_basis: "A.R.S. § 33-1361 — tenant termination notice shorter than 14-day minimum".to_string(),
                    notes: "VIOLATION: tenant termination notice shorter than 14-day statutory minimum under A.R.S. § 33-1361; termination invalid; tenant cannot recover damages or attorney's fees.".to_string(),
                    citations,
                    two_months_rent_unlawful_ouster_damages_dollars: 0,
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
            tenancy_type: TenancyType::ResidentialRentalCoveredByArlta,
            compliance_aspect: ComplianceAspect::SecurityDepositCapOneAndOneHalfMonthsRentUnderArs33_1321,
            material_breach_category: MaterialBreachCategory::MaterialNoncomplianceGeneralBreach,
            monthly_rent_dollars: 2_000,
            security_deposit_plus_prepaid_rent_dollars: 3_000,
            business_days_since_tenant_demanded_deposit_return: 10,
            deposit_returned_with_itemized_statement_within_window: true,
            landlord_obligations_under_section_33_1324_met: true,
            landlord_entry_notice_hours_given: 48,
            entry_was_emergency_or_court_order: false,
            pay_or_quit_notice_days_given: 5,
            cure_notice_days_given: 10,
            protected_activity_within_six_months: false,
            adverse_action_taken: false,
            unlawful_ouster_occurred: false,
            tenant_termination_notice_days_given: 14,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_type = TenancyType::CommercialRentalExempt;
        let output = check(&input);
        assert_eq!(output.mode, ArltaMode::NotApplicableTenancyExemptFromArlta);
    }

    #[test]
    fn security_deposit_at_one_and_one_half_months_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            ArltaMode::CompliantSecurityDepositAtOrBelowOneAndOneHalfMonthsRentCap
        );
    }

    #[test]
    fn security_deposit_at_one_and_one_half_months_plus_one_dollar_violation() {
        let mut input = baseline_input();
        input.security_deposit_plus_prepaid_rent_dollars = 3_001;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::ViolationSecurityDepositExceedsOneAndOneHalfMonthsRentCap
        );
    }

    #[test]
    fn deposit_return_within_fourteen_business_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnFourteenBusinessDayDeadlineUnderArs33_1321;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::CompliantSecurityDepositReturnedWithinFourteenBusinessDays
        );
    }

    #[test]
    fn deposit_return_at_exactly_fourteen_business_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnFourteenBusinessDayDeadlineUnderArs33_1321;
        input.business_days_since_tenant_demanded_deposit_return = 14;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::CompliantSecurityDepositReturnedWithinFourteenBusinessDays
        );
    }

    #[test]
    fn deposit_return_at_fifteen_business_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositReturnFourteenBusinessDayDeadlineUnderArs33_1321;
        input.business_days_since_tenant_demanded_deposit_return = 15;
        input.deposit_returned_with_itemized_statement_within_window = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::ViolationSecurityDepositReturnedPastFourteenBusinessDayDeadline
        );
    }

    #[test]
    fn landlord_obligations_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordObligationToMaintainPremisesUnderArs33_1324;
        let output = check(&input);
        assert_eq!(output.mode, ArltaMode::CompliantLandlordObligationsMet);
    }

    #[test]
    fn landlord_obligations_breached_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordObligationToMaintainPremisesUnderArs33_1324;
        input.landlord_obligations_under_section_33_1324_met = false;
        let output = check(&input);
        assert_eq!(output.mode, ArltaMode::ViolationLandlordObligationsBreached);
    }

    #[test]
    fn landlord_entry_at_forty_eight_hours_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryFortyEightHourNoticeUnderArs33_1343;
        let output = check(&input);
        assert_eq!(output.mode, ArltaMode::CompliantLandlordEntryFortyEightHourNoticeOrEmergency);
    }

    #[test]
    fn landlord_entry_emergency_exception_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryFortyEightHourNoticeUnderArs33_1343;
        input.landlord_entry_notice_hours_given = 0;
        input.entry_was_emergency_or_court_order = true;
        let output = check(&input);
        assert_eq!(output.mode, ArltaMode::CompliantLandlordEntryFortyEightHourNoticeOrEmergency);
    }

    #[test]
    fn landlord_entry_under_forty_eight_hours_and_not_emergency_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryFortyEightHourNoticeUnderArs33_1343;
        input.landlord_entry_notice_hours_given = 24;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::ViolationLandlordEntryWithoutFortyEightHourNoticeAndNotEmergency
        );
    }

    #[test]
    fn five_day_pay_or_quit_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveDayPayOrQuitNoticeUnderArs33_1368B;
        let output = check(&input);
        assert_eq!(output.mode, ArltaMode::CompliantFiveDayPayOrQuitNoticeProvided);
    }

    #[test]
    fn pay_or_quit_under_five_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveDayPayOrQuitNoticeUnderArs33_1368B;
        input.pay_or_quit_notice_days_given = 4;
        let output = check(&input);
        assert_eq!(output.mode, ArltaMode::ViolationPayOrQuitNoticePeriodShorterThanFiveDays);
    }

    #[test]
    fn ten_day_cure_notice_for_general_material_noncompliance_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenDayCureOrFiveDayHealthSafetyCureNoticeUnderArs33_1368A;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::CompliantTenDayCureOrFiveDayHealthSafetyCureNoticeProvided
        );
    }

    #[test]
    fn five_day_cure_notice_for_health_safety_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenDayCureOrFiveDayHealthSafetyCureNoticeUnderArs33_1368A;
        input.material_breach_category = MaterialBreachCategory::MaterialNoncomplianceHealthAndSafetyBreach;
        input.cure_notice_days_given = 5;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::CompliantTenDayCureOrFiveDayHealthSafetyCureNoticeProvided
        );
    }

    #[test]
    fn cure_notice_under_ten_days_for_general_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenDayCureOrFiveDayHealthSafetyCureNoticeUnderArs33_1368A;
        input.cure_notice_days_given = 7;
        let output = check(&input);
        assert_eq!(output.mode, ArltaMode::ViolationCureNoticePeriodShorterThanStatutoryMinimum);
    }

    #[test]
    fn cure_notice_under_five_days_for_health_safety_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenDayCureOrFiveDayHealthSafetyCureNoticeUnderArs33_1368A;
        input.material_breach_category = MaterialBreachCategory::MaterialNoncomplianceHealthAndSafetyBreach;
        input.cure_notice_days_given = 3;
        let output = check(&input);
        assert_eq!(output.mode, ArltaMode::ViolationCureNoticePeriodShorterThanStatutoryMinimum);
    }

    #[test]
    fn retaliation_within_six_months_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RetaliationProhibitedSixMonthPresumptionUnderArs33_1376_1377;
        input.protected_activity_within_six_months = true;
        input.adverse_action_taken = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::ViolationRetaliatoryConductWithinSixMonthsOfProtectedActivity
        );
    }

    #[test]
    fn no_retaliation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RetaliationProhibitedSixMonthPresumptionUnderArs33_1376_1377;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::CompliantNoRetaliationWithinSixMonthsOfProtectedActivity
        );
    }

    #[test]
    fn unlawful_ouster_triggers_two_months_rent_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::UnlawfulOusterTwoMonthsDamagesRemedyUnderArs33_1370;
        input.unlawful_ouster_occurred = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::ViolationUnlawfulOusterTriggersTwoMonthsRentOrActualDamages
        );
        assert_eq!(output.two_months_rent_unlawful_ouster_damages_dollars, 4_000);
    }

    #[test]
    fn no_unlawful_ouster_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::UnlawfulOusterTwoMonthsDamagesRemedyUnderArs33_1370;
        let output = check(&input);
        assert_eq!(output.mode, ArltaMode::CompliantNoUnlawfulOusterTenantInPossession);
    }

    #[test]
    fn tenant_termination_fourteen_day_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenantTerminationFourteenDayNoticeUnderArs33_1361;
        let output = check(&input);
        assert_eq!(output.mode, ArltaMode::CompliantTenantTerminationFourteenDayNoticeProvided);
    }

    #[test]
    fn tenant_termination_under_fourteen_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenantTerminationFourteenDayNoticeUnderArs33_1361;
        input.tenant_termination_notice_days_given = 13;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::ViolationTenantTerminationNoticeShorterThanFourteenDays
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(AZ_RLTA_TITLE_NUMBER, 33);
        assert_eq!(AZ_RLTA_CHAPTER_NUMBER, 10);
        assert_eq!(AZ_RLTA_ENACTMENT_YEAR, 1973);
        assert_eq!(AZ_RLTA_SECURITY_DEPOSIT_CAP_NUMERATOR_TENTHS_OF_MONTHS_RENT, 15);
        assert_eq!(AZ_RLTA_SECURITY_DEPOSIT_CAP_DENOMINATOR_TENTHS, 10);
        assert_eq!(AZ_RLTA_SECURITY_DEPOSIT_RETURN_DEADLINE_BUSINESS_DAYS, 14);
        assert_eq!(AZ_RLTA_LANDLORD_ENTRY_NOTICE_HOURS, 48);
        assert_eq!(AZ_RLTA_PAY_OR_QUIT_NOTICE_DAYS, 5);
        assert_eq!(AZ_RLTA_MATERIAL_NONCOMPLIANCE_CURE_PERIOD_DAYS, 10);
        assert_eq!(AZ_RLTA_HEALTH_SAFETY_CURE_PERIOD_DAYS, 5);
        assert_eq!(AZ_RLTA_RETALIATION_PRESUMPTION_WINDOW_MONTHS, 6);
        assert_eq!(AZ_RLTA_TENANT_TERMINATION_NOTICE_DAYS, 14);
        assert_eq!(AZ_RLTA_UNLAWFUL_OUSTER_TWO_MONTHS_DAMAGES_MULTIPLIER, 2);
        assert_eq!(AZ_RLTA_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Arizona Residential Landlord and Tenant Act"));
        assert!(joined.contains("1973"));
        assert!(joined.contains("A.R.S. §§ 33-1301 through 33-1381"));
        assert!(joined.contains("§ 33-1321"));
        assert!(joined.contains("§ 33-1324"));
        assert!(joined.contains("§ 33-1341"));
        assert!(joined.contains("§ 33-1343"));
        assert!(joined.contains("§ 33-1361"));
        assert!(joined.contains("§ 33-1368(A)"));
        assert!(joined.contains("§ 33-1368(B)"));
        assert!(joined.contains("§ 33-1370"));
        assert!(joined.contains("§ 33-1376"));
        assert!(joined.contains("§ 33-1377"));
        assert!(joined.contains("ONE AND ONE-HALF MONTH"));
        assert!(joined.contains("14 BUSINESS DAYS"));
        assert!(joined.contains("48 HOURS"));
        assert!(joined.contains("5-DAY WRITTEN NOTICE"));
        assert!(joined.contains("10-DAY WRITTEN NOTICE"));
        assert!(joined.contains("5-DAY CURE PERIOD"));
        assert!(joined.contains("6 MONTHS"));
        assert!(joined.contains("TWO MONTHS' PERIODIC RENT"));
        assert!(joined.contains("Uniform Residential Landlord and Tenant Act"));
        assert!(joined.contains("URLTA"));
    }

    #[test]
    fn unlawful_ouster_damages_saturating_overflow_defense() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::UnlawfulOusterTwoMonthsDamagesRemedyUnderArs33_1370;
        input.unlawful_ouster_occurred = true;
        input.monthly_rent_dollars = u64::MAX;
        let output = check(&input);
        assert_eq!(
            output.mode,
            ArltaMode::ViolationUnlawfulOusterTriggersTwoMonthsRentOrActualDamages
        );
        assert_eq!(output.two_months_rent_unlawful_ouster_damages_dollars, u64::MAX);
    }
}
