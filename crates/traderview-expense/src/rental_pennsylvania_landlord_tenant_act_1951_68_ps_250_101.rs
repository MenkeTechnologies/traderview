//! Pennsylvania Landlord-Tenant Act of 1951 Compliance
//! Module — codified at 68 P.S. §§ 250.101 through 250.602.
//! Pure-compute check for trader-landlord compliance with
//! the foundational Pennsylvania statewide residential
//! tenancy regime.
//!
//! Enacted by the Pennsylvania General Assembly as **Act 20
//! of April 6, 1951, P.L. 69**, with subsequent amendments
//! through 2025. The Act is among the oldest landlord-tenant
//! statutes still in widespread use in the United States.
//! Pennsylvania is a **major residential rental market** —
//! covers metropolitan Philadelphia (5th-largest US city),
//! Pittsburgh, Allentown, Harrisburg, Lancaster, Reading,
//! Erie, Bethlehem.
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: Pennsylvania Landlord-Tenant Act of 1951 enacted as **Act 20 of April 6, 1951, P.L. 69** ([Pennsylvania Legislature — The Landlord and Tenant Act of 1951 PDF](https://www.legis.state.pa.us/wu01/li/li/us/pdf/1951/0/0020..pdf); [City of Reading — Landlord-Tenant Act PDF (as amended through July 6, 1995)](https://www.readingpa.gov/images/pdfs/Landlord-TenantAct.pdf); [Bucks County — Pennsylvania Landlord/Tenant Act PDF](https://www.buckscounty.gov/DocumentCenter/View/1605/Landlord-Tenant-Act-PDF?bidId=); [FindLaw — Pennsylvania Statutes Title 68 P.S. Real and Personal Property § 250.501](https://codes.findlaw.com/pa/title-68-ps-real-and-personal-property/pa-st-sect-68-250-501/); [Pennsylvania Bulletin](https://www.pacodeandbulletin.gov/Display/pabull?file=/secure/pabulletin/data/vol51/51-44/1793.html); [246 Pa. Code Chapter 500 Actions for Recovery of Possession of Real Property](https://www.pacodeandbulletin.gov/Display/pacode?file=/secure/pacode/data/246/chapter500/chap500toc.html&d=reduce); [Landlord Studio — Pennsylvania Landlord Tenant Laws](https://www.landlordstudio.com/landlord-tenant-laws/pennsylvania-landlord-tenant-laws); [LandlordTenantLaw.com — Pennsylvania Landlord Tenant Law and Act in Plain English](https://www.landlord-tenant-law.com/pennsylvania-landlord-tenant-law.html); [Hemlane — Pennsylvania Landlord-Tenant Law Explained](https://www.hemlane.com/resources/pennsylvania-tenant-landlord-law/); [Nolo — Overview of Landlord-Tenant Laws in Pennsylvania](https://www.nolo.com/legal-encyclopedia/overview-landlord-tenant-laws-pennsylvania.html); [American Landlord — Pennsylvania Landlord-Tenant Laws](https://americanlandlord.com/landlord-tenant-laws-by-state/pennsylvania-landlord-tenant-laws/); [PayRent — A Guide to Pennsylvania Landlord Tenant Laws Updated 2023](https://www.payrent.com/articles/pennsylvania-landlord-tenant-laws/); [Innago — Pennsylvania Landlord Tenant Laws [2026]](https://innago.com/pennsylvania-landlord-tenant-laws/); [Prince Law — A Pennsylvania Tenant's Right to Recover a Security Deposit](https://blog.princelaw.com/2017/09/13/a-pennsylvania-tenants-right-to-recover-a-security-deposit/); [Stoner Law Offices — What Are Pennsylvania's Tenant Security Deposit Laws?](https://stoner-law.com/blog/what-are-pennsylvanias-tenant-security-deposit-laws/); [Apartments.com — Pennsylvania Rental Laws](https://www.apartments.com/rental-manager/resources/state-laws?state=Pennsylvania)).
//! - **Codification**: 68 Pennsylvania Statutes (P.S.) §§ 250.101 through 250.602 (Title 68 Real and Personal Property).
//! - **§ 250.501(b) Notice to Quit / Eviction Notice**: **10-DAY NOTICE for nonpayment of rent**; **15-DAY NOTICE for month-to-month tenancy termination OR fixed-term tenancy under 1 year**; **30-DAY NOTICE for fixed-term tenancy of 1 year or more**.
//! - **§ 250.511a Security Deposit Cap — First Year**: landlord may NOT require security deposit greater than **TWO MONTHS' RENT** during first year of lease.
//! - **§ 250.511b Security Deposit Cap — Second Year Plus**: at beginning of second year of lease, landlord may NOT retain security deposit greater than **ONE MONTH'S RENT**; must return any excess.
//! - **§ 250.512 Security Deposit Return**: landlord must provide tenant with **WRITTEN LIST OF DAMAGES + REMITTANCE OF BALANCE within 30 DAYS** of lease termination; failure to provide damages list within 30 days **WAIVES the landlord's right to withhold any portion** of the deposit; failure to remit balance within 30 days makes landlord liable for **DOUBLE THE AMOUNT** of the security deposit.
//! - **Implied Warranty of Habitability**: established by Pennsylvania Supreme Court in **Pugh v. Holmes, 486 Pa. 272, 405 A.2d 897 (1979)** and parallel Superior Court holding in **Beasley v. Freedman, 256 Pa. Super. 184, 389 A.2d 1126 (1978)**; landlord must provide premises that are **SAFE, SANITARY, AND REASONABLY COMFORTABLE FOR LIVING**; the warranty CANNOT BE WAIVED by lease terms; tenant remedies for breach include rent withholding + lease abandonment + repair-and-deduct + damages.
//! - **Unfit Premises Remedy (§ 250.205)**: if premises are unfit for human habitation, tenants may withhold rent in its entirety and have the right to abandon the rental unit.
//! - **Real Estate Licensure Requirement**: PA Real Estate Licensing and Registration Act (63 P.S. § 455.101 et seq.) requires property managers to hold a real estate broker's license; owner-managers self-managing their own properties are exempt.
//! - **246 Pa. Code Chapter 500**: Actions for Recovery of Possession of Real Property — procedural rules for eviction (magisterial district court) including filing requirements, hearings, and writs of possession.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const PA_LANDLORD_TENANT_ACT_ENACTMENT_YEAR: u32 = 1951;
pub const PA_LANDLORD_TENANT_ACT_ENACTMENT_MONTH: u32 = 4;
pub const PA_LANDLORD_TENANT_ACT_ENACTMENT_DAY: u32 = 6;
pub const PA_LANDLORD_TENANT_ACT_PUBLIC_LAW_PAGE: u32 = 69;
pub const PA_LANDLORD_TENANT_ACT_ACT_NUMBER: u32 = 20;
pub const PA_NONPAYMENT_PAY_OR_QUIT_NOTICE_DAYS: u32 = 10;
pub const PA_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS: u32 = 15;
pub const PA_FIXED_TERM_UNDER_YEAR_TERMINATION_NOTICE_DAYS: u32 = 15;
pub const PA_FIXED_TERM_ONE_YEAR_OR_MORE_TERMINATION_NOTICE_DAYS: u32 = 30;
pub const PA_SECURITY_DEPOSIT_CAP_FIRST_YEAR_MONTHS_OF_RENT: u32 = 2;
pub const PA_SECURITY_DEPOSIT_CAP_SECOND_YEAR_PLUS_MONTHS_OF_RENT: u32 = 1;
pub const PA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const PA_SECURITY_DEPOSIT_DOUBLE_DAMAGES_MULTIPLIER: u64 = 2;
pub const PA_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    ResidentialRentalCoveredByPaLandlordTenantAct,
    CommercialRentalExempt,
    HotelMotelTransientLodgingExempt,
    BoardingHouseInstitutionalExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseYearStatus {
    FirstYearOfLease,
    SecondYearOrLaterRenewal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyTermLength {
    MonthToMonth,
    FixedTermUnderOneYear,
    FixedTermOneYearOrMore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositCapFirstYearTwoMonthsRentUnderSection250_511A,
    SecurityDepositCapSecondYearPlusOneMonthRentUnderSection250_511B,
    SecurityDepositReturnAndDamagesListWithinThirtyDaysUnderSection250_512,
    NonpaymentTenDayPayOrQuitNoticeUnderSection250_501,
    TerminationNoticeBasedOnTenancyTermLengthUnderSection250_501B,
    ImpliedWarrantyOfHabitabilityUnderPughVHolmes,
    UnfitPremisesRentWithholdingRemedyUnderSection250_205,
    RealEstateLicensureRequirementForPropertyManagers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaLandlordTenantActMode {
    NotApplicableTenancyExemptFromPaLandlordTenantAct,
    CompliantSecurityDepositAtOrBelowFirstYearTwoMonthsCap,
    CompliantSecurityDepositAtOrBelowSecondYearOneMonthCap,
    CompliantSecurityDepositReturnedAndDamagesListWithinThirtyDays,
    CompliantTenDayPayOrQuitNoticeProvided,
    CompliantTerminationNoticeMeetsStatutoryLengthForTenancyType,
    CompliantImpliedWarrantyOfHabitabilityMaintained,
    CompliantNoUnfitPremisesTenantHasNoRentWithholdingGrounds,
    CompliantPropertyManagerHoldsRealEstateBrokerLicenseOrIsOwnerExempt,
    ViolationSecurityDepositExceedsFirstYearTwoMonthsCap,
    ViolationSecurityDepositExceedsSecondYearOneMonthCap,
    ViolationSecurityDepositReturnedPastThirtyDayDeadlineDoubleDamages,
    ViolationSecurityDepositNoDamagesListWithinThirtyDaysWaivesWithholding,
    ViolationPayOrQuitNoticeShorterThanTenDays,
    ViolationTerminationNoticeShorterThanStatutoryLengthForTenancyType,
    ViolationImpliedWarrantyOfHabitabilityBreached,
    ViolationUnfitPremisesNotRemediatedTenantRentWithholdingPermitted,
    ViolationPropertyManagerLacksRealEstateBrokerLicense,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_type: TenancyType,
    pub compliance_aspect: ComplianceAspect,
    pub lease_year_status: LeaseYearStatus,
    pub tenancy_term_length: TenancyTermLength,
    pub monthly_rent_dollars: u64,
    pub security_deposit_dollars: u64,
    pub deposit_returned_and_damages_list_within_window: bool,
    pub days_since_lease_termination_for_deposit_return: u32,
    pub damages_list_provided_within_window: bool,
    pub pay_or_quit_notice_days_given: u32,
    pub termination_notice_days_given: u32,
    pub implied_warranty_of_habitability_maintained: bool,
    pub premises_unfit_for_human_habitation: bool,
    pub property_manager_holds_real_estate_broker_license_or_is_owner_exempt: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: PaLandlordTenantActMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub double_damages_remedy_dollars: u64,
}

pub type RentalPennsylvaniaLandlordTenantAct1951_68Ps250_101Input = Input;
pub type RentalPennsylvaniaLandlordTenantAct1951_68Ps250_101Output = Output;
pub type RentalPennsylvaniaLandlordTenantAct1951_68Ps250_101Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Pennsylvania Landlord-Tenant Act of 1951 — Act 20 of April 6, 1951, P.L. 69; codified at 68 Pennsylvania Statutes (P.S.) §§ 250.101 through 250.602 (Title 68 Real and Personal Property)".to_string(),
        "68 P.S. § 250.501 Eviction Notice — § 250.501(b) statutory notice periods: 10 DAYS for nonpayment of rent; 15 DAYS for month-to-month tenancy termination OR fixed-term tenancy under 1 year; 30 DAYS for fixed-term tenancy of 1 year or more".to_string(),
        "68 P.S. § 250.511a Security Deposit Cap — First Year — landlord may NOT require security deposit greater than TWO MONTHS' RENT during first year of lease".to_string(),
        "68 P.S. § 250.511b Security Deposit Cap — Second Year Plus — at beginning of second year of lease, landlord may NOT retain security deposit greater than ONE MONTH'S RENT; must return excess to tenant".to_string(),
        "68 P.S. § 250.512 Security Deposit Return — landlord must provide WRITTEN LIST OF DAMAGES + REMITTANCE OF BALANCE within 30 DAYS of lease termination; failure to provide damages list within 30 days WAIVES landlord's right to withhold any portion; failure to remit balance within 30 days = DOUBLE THE AMOUNT of the security deposit damages".to_string(),
        "Pugh v. Holmes, 486 Pa. 272, 405 A.2d 897 (1979) — Pennsylvania Supreme Court established the IMPLIED WARRANTY OF HABITABILITY; landlord must provide premises that are SAFE, SANITARY, AND REASONABLY COMFORTABLE FOR LIVING; warranty CANNOT BE WAIVED by lease terms; tenant remedies include rent withholding + lease abandonment + repair-and-deduct + damages".to_string(),
        "Beasley v. Freedman, 256 Pa. Super. 184, 389 A.2d 1126 (1978) — Pennsylvania Superior Court parallel holding on implied warranty of habitability".to_string(),
        "68 P.S. § 250.205 Unfit Premises — if premises are unfit for human habitation, tenants may withhold rent in its entirety and have the right to abandon the rental unit".to_string(),
        "63 P.S. § 455.101 et seq. — Pennsylvania Real Estate Licensing and Registration Act — requires property managers to hold a real estate broker's license; owner-managers self-managing their own properties are EXEMPT".to_string(),
        "246 Pa. Code Chapter 500 — Actions for Recovery of Possession of Real Property — procedural rules for eviction (magisterial district court) including filing requirements, hearings, and writs of possession".to_string(),
        "Pennsylvania Legislature, Bucks County, City of Reading — primary statutory text + practitioner guides".to_string(),
        "Nolo + Hemlane + Landlord Studio + American Landlord + PayRent + Innago + Prince Law + Stoner Law Offices — practitioner overviews of PA Landlord-Tenant Act".to_string(),
    ];

    if input.tenancy_type != TenancyType::ResidentialRentalCoveredByPaLandlordTenantAct {
        return Output {
            mode: PaLandlordTenantActMode::NotApplicableTenancyExemptFromPaLandlordTenantAct,
            statutory_basis: "68 P.S. § 250.101 et seq. — PA Landlord-Tenant Act applies only to residential leaseholds; commercial / hotel-motel / institutional exempt".to_string(),
            notes: "NOT APPLICABLE: tenancy type exempt from PA Landlord-Tenant Act (commercial rental; hotel/motel transient lodging; boarding house institutional).".to_string(),
            citations,
            double_damages_remedy_dollars: 0,
        };
    }

    let double_damages = input
        .security_deposit_dollars
        .saturating_mul(PA_SECURITY_DEPOSIT_DOUBLE_DAMAGES_MULTIPLIER);

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositCapFirstYearTwoMonthsRentUnderSection250_511A => {
            let cap = input
                .monthly_rent_dollars
                .saturating_mul(u64::from(PA_SECURITY_DEPOSIT_CAP_FIRST_YEAR_MONTHS_OF_RENT));
            if input.lease_year_status != LeaseYearStatus::FirstYearOfLease {
                Output {
                    mode: PaLandlordTenantActMode::CompliantSecurityDepositAtOrBelowFirstYearTwoMonthsCap,
                    statutory_basis: "68 P.S. § 250.511a — first-year cap applies only to first year of lease".to_string(),
                    notes: "NOT TRIGGERED: lease is in second year or later; § 250.511b (one-month cap) applies instead of § 250.511a (two-month cap).".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else if input.security_deposit_dollars <= cap {
                Output {
                    mode: PaLandlordTenantActMode::CompliantSecurityDepositAtOrBelowFirstYearTwoMonthsCap,
                    statutory_basis: "68 P.S. § 250.511a — security deposit at or below 2-months'-rent first-year cap".to_string(),
                    notes: "COMPLIANT: security deposit at or below 2 months' rent first-year statutory cap under § 250.511a.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: PaLandlordTenantActMode::ViolationSecurityDepositExceedsFirstYearTwoMonthsCap,
                    statutory_basis: "68 P.S. § 250.511a — security deposit exceeds 2-months'-rent first-year cap".to_string(),
                    notes: "VIOLATION: security deposit exceeds 2 months' rent first-year cap under § 250.511a; landlord must refund excess.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::SecurityDepositCapSecondYearPlusOneMonthRentUnderSection250_511B => {
            let cap = input
                .monthly_rent_dollars
                .saturating_mul(u64::from(PA_SECURITY_DEPOSIT_CAP_SECOND_YEAR_PLUS_MONTHS_OF_RENT));
            if input.lease_year_status != LeaseYearStatus::SecondYearOrLaterRenewal {
                Output {
                    mode: PaLandlordTenantActMode::CompliantSecurityDepositAtOrBelowSecondYearOneMonthCap,
                    statutory_basis: "68 P.S. § 250.511b — second-year+ cap applies only at beginning of second year".to_string(),
                    notes: "NOT TRIGGERED: lease is still in first year; § 250.511a (two-month cap) applies instead of § 250.511b (one-month cap).".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else if input.security_deposit_dollars <= cap {
                Output {
                    mode: PaLandlordTenantActMode::CompliantSecurityDepositAtOrBelowSecondYearOneMonthCap,
                    statutory_basis: "68 P.S. § 250.511b — security deposit at or below 1-month-rent second-year-plus cap".to_string(),
                    notes: "COMPLIANT: security deposit at or below 1 month's rent second-year-plus statutory cap under § 250.511b; landlord returned any excess to tenant.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: PaLandlordTenantActMode::ViolationSecurityDepositExceedsSecondYearOneMonthCap,
                    statutory_basis: "68 P.S. § 250.511b — security deposit exceeds 1-month-rent second-year-plus cap".to_string(),
                    notes: "VIOLATION: security deposit exceeds 1 month's rent second-year-plus cap under § 250.511b; landlord must refund excess.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::SecurityDepositReturnAndDamagesListWithinThirtyDaysUnderSection250_512 => {
            if input.deposit_returned_and_damages_list_within_window
                && input.damages_list_provided_within_window
                && input.days_since_lease_termination_for_deposit_return
                    <= PA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS
            {
                Output {
                    mode: PaLandlordTenantActMode::CompliantSecurityDepositReturnedAndDamagesListWithinThirtyDays,
                    statutory_basis: "68 P.S. § 250.512 — deposit returned and damages list provided within 30-day window".to_string(),
                    notes: "COMPLIANT: landlord provided written list of damages AND remitted balance of security deposit within 30-day window after lease termination under § 250.512.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else if !input.damages_list_provided_within_window {
                Output {
                    mode: PaLandlordTenantActMode::ViolationSecurityDepositNoDamagesListWithinThirtyDaysWaivesWithholding,
                    statutory_basis: "68 P.S. § 250.512 — failure to provide damages list within 30 days WAIVES right to withhold any portion".to_string(),
                    notes: "VIOLATION: landlord failed to provide written list of damages within 30 days of lease termination; § 250.512 WAIVES landlord's right to withhold any portion of the security deposit; landlord must return full deposit.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: PaLandlordTenantActMode::ViolationSecurityDepositReturnedPastThirtyDayDeadlineDoubleDamages,
                    statutory_basis: "68 P.S. § 250.512 — failure to remit balance within 30 days = DOUBLE the deposit amount damages".to_string(),
                    notes: "VIOLATION: landlord failed to remit balance of security deposit within 30 days of lease termination; § 250.512 makes landlord liable for DOUBLE THE AMOUNT of the security deposit.".to_string(),
                    citations,
                    double_damages_remedy_dollars: double_damages,
                }
            }
        }
        ComplianceAspect::NonpaymentTenDayPayOrQuitNoticeUnderSection250_501 => {
            if input.pay_or_quit_notice_days_given >= PA_NONPAYMENT_PAY_OR_QUIT_NOTICE_DAYS {
                Output {
                    mode: PaLandlordTenantActMode::CompliantTenDayPayOrQuitNoticeProvided,
                    statutory_basis: "68 P.S. § 250.501 — 10-day pay or quit notice provided for nonpayment".to_string(),
                    notes: "COMPLIANT: landlord provided 10-day pay or quit written notice under § 250.501 for nonpayment of rent.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: PaLandlordTenantActMode::ViolationPayOrQuitNoticeShorterThanTenDays,
                    statutory_basis: "68 P.S. § 250.501 — pay or quit notice shorter than 10-day statutory minimum for nonpayment".to_string(),
                    notes: "VIOLATION: pay or quit notice shorter than 10-day statutory minimum under § 250.501 for nonpayment; eviction action subject to dismissal.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::TerminationNoticeBasedOnTenancyTermLengthUnderSection250_501B => {
            let required_days = match input.tenancy_term_length {
                TenancyTermLength::MonthToMonth => PA_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS,
                TenancyTermLength::FixedTermUnderOneYear => PA_FIXED_TERM_UNDER_YEAR_TERMINATION_NOTICE_DAYS,
                TenancyTermLength::FixedTermOneYearOrMore => PA_FIXED_TERM_ONE_YEAR_OR_MORE_TERMINATION_NOTICE_DAYS,
            };
            if input.termination_notice_days_given >= required_days {
                Output {
                    mode: PaLandlordTenantActMode::CompliantTerminationNoticeMeetsStatutoryLengthForTenancyType,
                    statutory_basis: "68 P.S. § 250.501(b) — termination notice meets statutory length for tenancy type".to_string(),
                    notes: "COMPLIANT: termination notice meets statutory length for tenancy term type (15 days for month-to-month or fixed term under 1 year; 30 days for fixed term of 1 year or more).".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: PaLandlordTenantActMode::ViolationTerminationNoticeShorterThanStatutoryLengthForTenancyType,
                    statutory_basis: "68 P.S. § 250.501(b) — termination notice shorter than statutory length for tenancy type".to_string(),
                    notes: "VIOLATION: termination notice shorter than statutory length for tenancy term type under § 250.501(b); eviction action subject to dismissal.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::ImpliedWarrantyOfHabitabilityUnderPughVHolmes => {
            if input.implied_warranty_of_habitability_maintained {
                Output {
                    mode: PaLandlordTenantActMode::CompliantImpliedWarrantyOfHabitabilityMaintained,
                    statutory_basis: "Pugh v. Holmes, 486 Pa. 272 (1979) + Beasley v. Freedman, 256 Pa. Super. 184 (1978) — implied warranty of habitability maintained".to_string(),
                    notes: "COMPLIANT: landlord maintains premises in a state that is SAFE, SANITARY, AND REASONABLY COMFORTABLE FOR LIVING under Pugh v. Holmes (1979) implied warranty of habitability.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: PaLandlordTenantActMode::ViolationImpliedWarrantyOfHabitabilityBreached,
                    statutory_basis: "Pugh v. Holmes, 486 Pa. 272 (1979) — implied warranty of habitability breached".to_string(),
                    notes: "VIOLATION: landlord breached implied warranty of habitability under Pugh v. Holmes (1979); tenant remedies include rent withholding + lease abandonment + repair-and-deduct + damages; warranty CANNOT BE WAIVED by lease terms.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::UnfitPremisesRentWithholdingRemedyUnderSection250_205 => {
            if input.premises_unfit_for_human_habitation {
                Output {
                    mode: PaLandlordTenantActMode::ViolationUnfitPremisesNotRemediatedTenantRentWithholdingPermitted,
                    statutory_basis: "68 P.S. § 250.205 — premises unfit for human habitation".to_string(),
                    notes: "VIOLATION: premises unfit for human habitation under § 250.205; tenant may withhold rent in its entirety and has the right to abandon the rental unit.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: PaLandlordTenantActMode::CompliantNoUnfitPremisesTenantHasNoRentWithholdingGrounds,
                    statutory_basis: "68 P.S. § 250.205 — premises fit for human habitation".to_string(),
                    notes: "COMPLIANT: premises fit for human habitation; tenant has no § 250.205 rent withholding or abandonment grounds.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::RealEstateLicensureRequirementForPropertyManagers => {
            if input.property_manager_holds_real_estate_broker_license_or_is_owner_exempt {
                Output {
                    mode: PaLandlordTenantActMode::CompliantPropertyManagerHoldsRealEstateBrokerLicenseOrIsOwnerExempt,
                    statutory_basis: "63 P.S. § 455.101 et seq. — property manager holds real estate broker license or is owner-exempt".to_string(),
                    notes: "COMPLIANT: property manager holds Pennsylvania real estate broker license OR is owner self-managing own property (statutory exemption from licensure under 63 P.S. § 455.101 et seq.).".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: PaLandlordTenantActMode::ViolationPropertyManagerLacksRealEstateBrokerLicense,
                    statutory_basis: "63 P.S. § 455.101 et seq. — property manager lacks required real estate broker license".to_string(),
                    notes: "VIOLATION: property manager does not hold required Pennsylvania real estate broker license under 63 P.S. § 455.101 et seq.; unlicensed property management exposes manager to civil and criminal penalties.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
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
            tenancy_type: TenancyType::ResidentialRentalCoveredByPaLandlordTenantAct,
            compliance_aspect: ComplianceAspect::SecurityDepositCapFirstYearTwoMonthsRentUnderSection250_511A,
            lease_year_status: LeaseYearStatus::FirstYearOfLease,
            tenancy_term_length: TenancyTermLength::MonthToMonth,
            monthly_rent_dollars: 1_500,
            security_deposit_dollars: 3_000,
            deposit_returned_and_damages_list_within_window: true,
            days_since_lease_termination_for_deposit_return: 28,
            damages_list_provided_within_window: true,
            pay_or_quit_notice_days_given: 10,
            termination_notice_days_given: 15,
            implied_warranty_of_habitability_maintained: true,
            premises_unfit_for_human_habitation: false,
            property_manager_holds_real_estate_broker_license_or_is_owner_exempt: true,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_type = TenancyType::CommercialRentalExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::NotApplicableTenancyExemptFromPaLandlordTenantAct
        );
    }

    #[test]
    fn first_year_security_deposit_at_two_months_cap_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::CompliantSecurityDepositAtOrBelowFirstYearTwoMonthsCap
        );
    }

    #[test]
    fn first_year_security_deposit_at_two_months_plus_one_dollar_violation() {
        let mut input = baseline_input();
        input.security_deposit_dollars = 3_001;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::ViolationSecurityDepositExceedsFirstYearTwoMonthsCap
        );
    }

    #[test]
    fn second_year_security_deposit_at_one_month_cap_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositCapSecondYearPlusOneMonthRentUnderSection250_511B;
        input.lease_year_status = LeaseYearStatus::SecondYearOrLaterRenewal;
        input.security_deposit_dollars = 1_500;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::CompliantSecurityDepositAtOrBelowSecondYearOneMonthCap
        );
    }

    #[test]
    fn second_year_security_deposit_at_one_month_plus_one_dollar_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositCapSecondYearPlusOneMonthRentUnderSection250_511B;
        input.lease_year_status = LeaseYearStatus::SecondYearOrLaterRenewal;
        input.security_deposit_dollars = 1_501;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::ViolationSecurityDepositExceedsSecondYearOneMonthCap
        );
    }

    #[test]
    fn deposit_return_and_damages_list_within_thirty_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnAndDamagesListWithinThirtyDaysUnderSection250_512;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::CompliantSecurityDepositReturnedAndDamagesListWithinThirtyDays
        );
    }

    #[test]
    fn deposit_return_at_exactly_thirty_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnAndDamagesListWithinThirtyDaysUnderSection250_512;
        input.days_since_lease_termination_for_deposit_return = 30;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::CompliantSecurityDepositReturnedAndDamagesListWithinThirtyDays
        );
    }

    #[test]
    fn no_damages_list_within_thirty_days_waives_withholding_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnAndDamagesListWithinThirtyDaysUnderSection250_512;
        input.damages_list_provided_within_window = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::ViolationSecurityDepositNoDamagesListWithinThirtyDaysWaivesWithholding
        );
    }

    #[test]
    fn deposit_returned_past_thirty_days_double_damages_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnAndDamagesListWithinThirtyDaysUnderSection250_512;
        input.deposit_returned_and_damages_list_within_window = false;
        input.days_since_lease_termination_for_deposit_return = 35;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::ViolationSecurityDepositReturnedPastThirtyDayDeadlineDoubleDamages
        );
        assert_eq!(output.double_damages_remedy_dollars, 3_000 * 2);
    }

    #[test]
    fn ten_day_pay_or_quit_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentTenDayPayOrQuitNoticeUnderSection250_501;
        let output = check(&input);
        assert_eq!(output.mode, PaLandlordTenantActMode::CompliantTenDayPayOrQuitNoticeProvided);
    }

    #[test]
    fn pay_or_quit_notice_under_ten_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::NonpaymentTenDayPayOrQuitNoticeUnderSection250_501;
        input.pay_or_quit_notice_days_given = 7;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::ViolationPayOrQuitNoticeShorterThanTenDays
        );
    }

    #[test]
    fn termination_notice_month_to_month_fifteen_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TerminationNoticeBasedOnTenancyTermLengthUnderSection250_501B;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::CompliantTerminationNoticeMeetsStatutoryLengthForTenancyType
        );
    }

    #[test]
    fn termination_notice_fixed_term_one_year_or_more_at_thirty_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TerminationNoticeBasedOnTenancyTermLengthUnderSection250_501B;
        input.tenancy_term_length = TenancyTermLength::FixedTermOneYearOrMore;
        input.termination_notice_days_given = 30;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::CompliantTerminationNoticeMeetsStatutoryLengthForTenancyType
        );
    }

    #[test]
    fn termination_notice_fixed_term_one_year_or_more_at_twenty_nine_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TerminationNoticeBasedOnTenancyTermLengthUnderSection250_501B;
        input.tenancy_term_length = TenancyTermLength::FixedTermOneYearOrMore;
        input.termination_notice_days_given = 29;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::ViolationTerminationNoticeShorterThanStatutoryLengthForTenancyType
        );
    }

    #[test]
    fn implied_warranty_of_habitability_maintained_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ImpliedWarrantyOfHabitabilityUnderPughVHolmes;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::CompliantImpliedWarrantyOfHabitabilityMaintained
        );
    }

    #[test]
    fn implied_warranty_of_habitability_breached_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ImpliedWarrantyOfHabitabilityUnderPughVHolmes;
        input.implied_warranty_of_habitability_maintained = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::ViolationImpliedWarrantyOfHabitabilityBreached
        );
    }

    #[test]
    fn premises_fit_for_habitation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::UnfitPremisesRentWithholdingRemedyUnderSection250_205;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::CompliantNoUnfitPremisesTenantHasNoRentWithholdingGrounds
        );
    }

    #[test]
    fn premises_unfit_for_habitation_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::UnfitPremisesRentWithholdingRemedyUnderSection250_205;
        input.premises_unfit_for_human_habitation = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::ViolationUnfitPremisesNotRemediatedTenantRentWithholdingPermitted
        );
    }

    #[test]
    fn property_manager_holds_broker_license_or_owner_exempt_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RealEstateLicensureRequirementForPropertyManagers;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::CompliantPropertyManagerHoldsRealEstateBrokerLicenseOrIsOwnerExempt
        );
    }

    #[test]
    fn property_manager_lacks_broker_license_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RealEstateLicensureRequirementForPropertyManagers;
        input.property_manager_holds_real_estate_broker_license_or_is_owner_exempt = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::ViolationPropertyManagerLacksRealEstateBrokerLicense
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(PA_LANDLORD_TENANT_ACT_ENACTMENT_YEAR, 1951);
        assert_eq!(PA_LANDLORD_TENANT_ACT_ENACTMENT_MONTH, 4);
        assert_eq!(PA_LANDLORD_TENANT_ACT_ENACTMENT_DAY, 6);
        assert_eq!(PA_LANDLORD_TENANT_ACT_PUBLIC_LAW_PAGE, 69);
        assert_eq!(PA_LANDLORD_TENANT_ACT_ACT_NUMBER, 20);
        assert_eq!(PA_NONPAYMENT_PAY_OR_QUIT_NOTICE_DAYS, 10);
        assert_eq!(PA_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS, 15);
        assert_eq!(PA_FIXED_TERM_UNDER_YEAR_TERMINATION_NOTICE_DAYS, 15);
        assert_eq!(PA_FIXED_TERM_ONE_YEAR_OR_MORE_TERMINATION_NOTICE_DAYS, 30);
        assert_eq!(PA_SECURITY_DEPOSIT_CAP_FIRST_YEAR_MONTHS_OF_RENT, 2);
        assert_eq!(PA_SECURITY_DEPOSIT_CAP_SECOND_YEAR_PLUS_MONTHS_OF_RENT, 1);
        assert_eq!(PA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(PA_SECURITY_DEPOSIT_DOUBLE_DAMAGES_MULTIPLIER, 2);
        assert_eq!(PA_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Pennsylvania Landlord-Tenant Act of 1951"));
        assert!(joined.contains("Act 20 of April 6, 1951"));
        assert!(joined.contains("P.L. 69"));
        assert!(joined.contains("68 Pennsylvania Statutes (P.S.) §§ 250.101 through 250.602"));
        assert!(joined.contains("§ 250.501"));
        assert!(joined.contains("§ 250.511a"));
        assert!(joined.contains("§ 250.511b"));
        assert!(joined.contains("§ 250.512"));
        assert!(joined.contains("§ 250.205"));
        assert!(joined.contains("Pugh v. Holmes"));
        assert!(joined.contains("486 Pa. 272"));
        assert!(joined.contains("Beasley v. Freedman"));
        assert!(joined.contains("256 Pa. Super. 184"));
        assert!(joined.contains("10 DAYS"));
        assert!(joined.contains("15 DAYS"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("TWO MONTHS"));
        assert!(joined.contains("ONE MONTH"));
        assert!(joined.contains("DOUBLE THE AMOUNT"));
        assert!(joined.contains("63 P.S. § 455.101"));
        assert!(joined.contains("246 Pa. Code Chapter 500"));
    }

    #[test]
    fn double_damages_saturating_overflow_defense() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnAndDamagesListWithinThirtyDaysUnderSection250_512;
        input.deposit_returned_and_damages_list_within_window = false;
        input.security_deposit_dollars = u64::MAX;
        let output = check(&input);
        assert_eq!(
            output.mode,
            PaLandlordTenantActMode::ViolationSecurityDepositReturnedPastThirtyDayDeadlineDoubleDamages
        );
        assert_eq!(output.double_damages_remedy_dollars, u64::MAX);
    }
}
