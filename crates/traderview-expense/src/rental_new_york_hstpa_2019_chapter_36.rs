//! New York Housing Stability and Tenant Protection Act
//! of 2019 (HSTPA) Compliance Module — Chapter 36 of the
//! Laws of 2019. Pure-compute check for trader-landlord
//! compliance with the major statewide tenant-protection
//! reform enacted by the New York State Legislature.
//!
//! HSTPA was signed by Governor Andrew Cuomo on **June 14,
//! 2019**, with most provisions effective on the date of
//! enactment. The Act is the **most consequential
//! statewide tenant-protection reform in New York history**
//! and applies to **ALL** residential tenancies statewide
//! (both regulated and unregulated, NYC and outside NYC).
//! Made rent regulation **PERMANENT** by eliminating the
//! prior sunset provisions; eliminated the "vacancy bonus"
//! in rent-regulated units; tightened pre-eviction notice
//! requirements; capped security deposits at one month's
//! rent statewide; capped late fees and application fees.
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: Housing Stability and Tenant Protection Act of 2019 (HSTPA), **Chapter 36 of the Laws of 2019**; signed by Governor Andrew M. Cuomo on **JUNE 14, 2019**; most provisions effective on date of enactment ([New York State Bar Association — NY's Housing Stability and Tenant Protection Act of 2019 Part III WHAT LAWYERS MUST KNOW](https://nysba.org/nys-housing-stability-and-tenant-protection-act-of-2019-part-iii-what-lawyers-must-know/); [NYSenate.gov — New Rights for Tenants: Housing Stability and Tenant Protection Act of 2019](https://www.nysenate.gov/newsroom/articles/2019/new-rights-tenants-housing-stability-and-tenant-protection-act-2019-1); [NYSBA — Housing Stability and Tenant Protection Act 2019 (2020 Update)](https://nysba.org/housing-stability-and-tenant-protection-act-2019-2020-update/); [CPL Law — Housing Stability and Tenant Protection Act of 2019: Memorandum for Landlords and their Agents](https://cpllawfirm.com/blog/housing-stability-and-tenant-protection-act-of-2019-memorandum-for-landlords-and-their-agents/); [Housing Justice for All — Housing Stability and Tenant Protection Act of 2019](https://housingjusticeforall.org/housing-stability-and-tenant-protection-act-of-2019/); [Apartment Professionals Trade Society of New York — HSTPA](https://www.aptsofny.org/housing-stability-and-tenant-protections-act-of-2019-hstpa); [YRE Blog — HSTPA Key Changes for NYC Renters](https://blog.yeonyc.com/the-housing-stability-tenant-protection-act-hstpa-of-2019-key-changes-for-nyc-renters-73958); [Greater Capital Association of Realtors — Housing Stability and Tenant Protection Act of 2019](https://gcar.com/tenantprotectionact/); [WNY Lawyers — NY Housing Stability & Tenant Protection Act of 2019](https://www.wny-lawyers.com/ny-housing-stability-tenant-protection-act-of-2019/); [Jordi Fernandez Law — Security Deposits Under HSTPA](https://jordifernandezlaw.com/security-deposits-under-the-housing-stability-and-tenant-protection-act-of-2019/); [Itkowitz — Michelle's Guide to Changes in Landlord and Tenant Litigation Process in NYC under HSTPA](https://itkowitz.com/blog/2019/06/michelles-guide-to-changes-in-landlord.html); [Itkowitz Booklet — A Guide to the New York HSTPA](https://itkowitz.com/booklets/Guide-To-The-Housing-Stability-And-Tenant-Protection-Act-Of-2019.pdf); [Massimo D'Angelo — NYC Nonpayment Proceedings Post HSTPA](https://www.albarticles.com/nonpayment-proceedings-post-hstpa/); [Mobilization for Justice — Tenant Representation in a Residential Nonpayment Proceeding NY](http://mobilizationforjustice.org/wp-content/uploads/Tenant-Representation-in-a-Residential-Nonpayment-Proceeding-NY.pdf); [Real Estate Weekly — Recovering Attorney Fees from Eviction Proceedings in the HSTPA Era](https://rew-online.com/recovering-attorney-fees-from-eviction-proceedings-in-the-hstpa-era/); [Beacon Attorney — New York's Revamped Landlord-Tenant Law](https://beaconattorney.com/blog-new-york%E2%80%99s-revamped-landlord-tenant-law.php); [NYSBA Journal — New York's Housing Stability and Tenant Protection Act of 2019 (PDF)](https://nysba.org/wp-content/uploads/2019/12/JRNL_SeptOct19_NYHousingTenantProtectionAct.pdf)).
//! - **General Obligations Law § 7-103 Security Deposit Cap**: HSTPA amended GOL § 7-103 to limit security deposits to **ONE MONTH'S RENT** for **UNREGULATED tenants STATEWIDE** (regulated tenants previously capped); applies to leases entered into or renewed after June 14, 2019.
//! - **General Obligations Law § 7-108 Security Deposit Return**: if any portion of the security deposit is retained, landlord must provide (i) an **ITEMIZED STATEMENT of claimed conditions within 14 DAYS after tenant vacates** AND (ii) any **REMAINING PORTION of the deposit**; failure to comply within 14 days waives right to withhold.
//! - **Real Property Law § 238-a Late Fees + Application Fees**: landlord may NOT charge late fee until rent is **5 DAYS LATE**; late fee CAPPED at the **LESSER OF $50 OR 5 % of monthly rent**; **APPLICATION FEE CAPPED at $20 statewide** (background check + tenant screening combined).
//! - **Real Property Law § 226-c Notice of Non-Renewal / Termination by Landlord**: tenancy duration determines notice length — **less than 1 year** tenancy = **30-DAY NOTICE**; **at least 1 year but less than 2 years** = **60-DAY NOTICE**; **at least 2 years** = **90-DAY NOTICE**; this notice applies to non-renewal AND to landlord's intent to raise rent more than 5 %.
//! - **Real Property Actions and Proceedings Law § 711(2) Nonpayment Predicate Notice**: HSTPA abolished oral rent demands and required **WRITTEN RENT DEMAND with 14 DAYS to pay or quit** for nonpayment proceedings (replaced prior 3-day demand period).
//! - **RPAPL § 753 Post-Warrant Hardship Stay**: court may grant post-warrant stay of eviction up to **1 YEAR (365 DAYS) for hardship**; pre-HSTPA limit was shorter.
//! - **Eliminated Vacancy Bonus**: HSTPA eliminated the prior 20 % "vacancy bonus" rent increase upon turnover in rent-regulated units; landlords may no longer increase rent merely because the unit became vacant.
//! - **Rent Regulation Made Permanent**: HSTPA eliminated prior 4-year sunset provisions on the Emergency Tenant Protection Act of 1974 (ETPA); rent stabilization and rent control are now PERMANENT statutes.
//! - **Real Property Law § 235-b Implied Warranty of Habitability**: pre-existing statewide implied warranty of habitability (codified 1975); HSTPA strengthened tenant remedies for landlord breach (uncapped damages + injunctive relief).
//! - **Statewide Expansion of Rent Regulation Eligibility**: HSTPA expanded the geographic scope of rent regulation to include localities OUTSIDE NYC if they opt in via local declaration of housing emergency under ETPA.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const HSTPA_ENACTMENT_DATE_YEAR: u32 = 2019;
pub const HSTPA_ENACTMENT_DATE_MONTH: u32 = 6;
pub const HSTPA_ENACTMENT_DATE_DAY: u32 = 14;
pub const HSTPA_LAW_CHAPTER_NUMBER: u32 = 36;
pub const HSTPA_SECURITY_DEPOSIT_CAP_IN_MONTHS_OF_RENT: u32 = 1;
pub const HSTPA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 14;
pub const HSTPA_LATE_FEE_GRACE_PERIOD_DAYS: u32 = 5;
pub const HSTPA_LATE_FEE_DOLLAR_CAP: u64 = 50;
pub const HSTPA_LATE_FEE_PERCENT_CAP_BPS: u64 = 500;
pub const HSTPA_APPLICATION_FEE_CAP_DOLLARS: u64 = 20;
pub const HSTPA_NONPAYMENT_WRITTEN_DEMAND_NOTICE_DAYS: u32 = 14;
pub const HSTPA_TERMINATION_NOTICE_UNDER_1_YEAR_DAYS: u32 = 30;
pub const HSTPA_TERMINATION_NOTICE_1_TO_2_YEARS_DAYS: u32 = 60;
pub const HSTPA_TERMINATION_NOTICE_2_PLUS_YEARS_DAYS: u32 = 90;
pub const HSTPA_POST_WARRANT_HARDSHIP_STAY_MAX_DAYS: u32 = 365;
pub const HSTPA_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    ResidentialRentalCoveredByHstpa,
    CommercialRentalExempt,
    HotelMotelTransientLodgingExempt,
    OwnerOccupiedTwoFamilyOrThreeFamilyDwellingExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyDurationCategory {
    LessThanOneYear,
    AtLeastOneYearButLessThanTwoYears,
    AtLeastTwoYears,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositCapOneMonthUnderGolSection7_103,
    SecurityDepositReturnFourteenDayDeadlineUnderGolSection7_108,
    LateFeeFiveDayGracePeriodAndDollarOrPercentCapUnderRplSection238A,
    ApplicationFeeTwentyDollarCapUnderRplSection238A,
    TerminationNoticeBasedOnTenancyDurationUnderRplSection226C,
    NonpaymentWrittenDemandFourteenDayNoticeUnderRpaplSection711_2,
    PostWarrantHardshipStayUnderRpaplSection753,
    EliminationOfVacancyBonusInRentRegulatedUnits,
    ImpliedWarrantyOfHabitabilityUnderRplSection235B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HstpaMode {
    NotApplicableTenancyExemptFromHstpa,
    CompliantSecurityDepositAtOrBelowOneMonthCap,
    CompliantSecurityDepositReturnedWithinFourteenDays,
    CompliantLateFeeWithinGracePeriodAndCap,
    CompliantApplicationFeeAtOrBelowTwentyDollarCap,
    CompliantTerminationNoticeMeetsStatutoryLengthForTenancyDuration,
    CompliantNonpaymentWrittenDemandFourteenDayNoticeProvided,
    CompliantPostWarrantHardshipStayWithinOneYearMaximum,
    CompliantNoVacancyBonusClaimed,
    CompliantImpliedWarrantyOfHabitabilityMaintained,
    ViolationSecurityDepositExceedsOneMonthCap,
    ViolationSecurityDepositReturnedPastFourteenDayDeadline,
    ViolationLateFeeChargedWithinFiveDayGracePeriod,
    ViolationLateFeeExceedsLesserOfFiftyDollarsOrFivePercent,
    ViolationApplicationFeeExceedsTwentyDollarCap,
    ViolationTerminationNoticeShorterThanStatutoryLengthForTenancyDuration,
    ViolationNonpaymentOralDemandOrShorterThanFourteenDayWrittenDemand,
    ViolationVacancyBonusClaimedInRentRegulatedUnit,
    ViolationImpliedWarrantyOfHabitabilityBreached,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_type: TenancyType,
    pub compliance_aspect: ComplianceAspect,
    pub tenancy_duration: TenancyDurationCategory,
    pub monthly_rent_dollars: u64,
    pub security_deposit_dollars: u64,
    pub days_since_tenant_vacated_for_deposit_return: u32,
    pub deposit_returned_and_itemized_statement_within_window: bool,
    pub days_rent_late_when_late_fee_charged: u32,
    pub late_fee_charged_dollars: u64,
    pub application_fee_charged_dollars: u64,
    pub termination_notice_days_given: u32,
    pub nonpayment_notice_is_written: bool,
    pub nonpayment_notice_days_given: u32,
    pub post_warrant_hardship_stay_days_granted: u32,
    pub vacancy_bonus_claimed_in_rent_regulated_unit: bool,
    pub implied_warranty_of_habitability_maintained: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: HstpaMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalNewYorkHstpa2019Chapter36Input = Input;
pub type RentalNewYorkHstpa2019Chapter36Output = Output;
pub type RentalNewYorkHstpa2019Chapter36Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Housing Stability and Tenant Protection Act of 2019 (HSTPA) — Chapter 36 of the Laws of 2019; signed by Governor Andrew M. Cuomo on JUNE 14, 2019; most provisions effective on date of enactment; applies STATEWIDE to all residential tenancies (regulated and unregulated, NYC and outside NYC)".to_string(),
        "New York General Obligations Law § 7-103 Security Deposit Cap — HSTPA amended GOL § 7-103 to limit security deposits to ONE MONTH'S RENT for UNREGULATED tenants STATEWIDE; applies to leases entered into or renewed after June 14, 2019".to_string(),
        "New York General Obligations Law § 7-108 Security Deposit Return — if any portion of security deposit retained, landlord must provide (i) ITEMIZED STATEMENT of claimed conditions within 14 DAYS after tenant vacates AND (ii) REMAINING PORTION of deposit; failure to comply within 14 days WAIVES right to withhold any portion".to_string(),
        "New York Real Property Law § 238-a Late Fees + Application Fees — landlord may NOT charge late fee until rent is 5 DAYS LATE; late fee CAPPED at LESSER OF $50 OR 5 % of monthly rent; APPLICATION FEE CAPPED at $20 statewide (background check + tenant screening combined)".to_string(),
        "New York Real Property Law § 226-c Notice of Non-Renewal / Termination by Landlord — tenancy duration determines notice length: LESS THAN 1 YEAR = 30-DAY NOTICE; AT LEAST 1 YEAR BUT LESS THAN 2 YEARS = 60-DAY NOTICE; AT LEAST 2 YEARS = 90-DAY NOTICE; this notice applies to non-renewal AND to landlord's intent to raise rent more than 5 %".to_string(),
        "New York Real Property Actions and Proceedings Law (RPAPL) § 711(2) Nonpayment Predicate Notice — HSTPA abolished oral rent demands and required WRITTEN RENT DEMAND with 14 DAYS to pay or quit for nonpayment proceedings (replaced prior 3-day demand period)".to_string(),
        "RPAPL § 753 Post-Warrant Hardship Stay — court may grant post-warrant stay of eviction up to 1 YEAR (365 DAYS) for hardship".to_string(),
        "HSTPA Eliminated Vacancy Bonus — HSTPA eliminated the prior 20 % 'vacancy bonus' rent increase upon turnover in rent-regulated units; landlords may no longer increase rent merely because the unit became vacant".to_string(),
        "HSTPA Rent Regulation Made Permanent — HSTPA eliminated prior 4-year sunset provisions on the Emergency Tenant Protection Act of 1974 (ETPA); rent stabilization and rent control are now PERMANENT New York statutes".to_string(),
        "New York Real Property Law § 235-b Implied Warranty of Habitability — pre-existing statewide implied warranty of habitability (codified 1975); HSTPA strengthened tenant remedies for landlord breach (uncapped damages + injunctive relief)".to_string(),
        "HSTPA Statewide Expansion of Rent Regulation Eligibility — HSTPA expanded the geographic scope of rent regulation to include localities OUTSIDE NYC if they opt in via local declaration of housing emergency under Emergency Tenant Protection Act of 1974 (ETPA)".to_string(),
        "New York State Bar Association + New York State Senate + Apartment Professionals Trade Society of New York + Itkowitz + Mobilization for Justice + Real Estate Weekly + Beacon Attorney — practitioner overviews of HSTPA".to_string(),
        "NYSBA Journal — New York's Housing Stability and Tenant Protection Act of 2019 (September / October 2019) — academic practitioner overview".to_string(),
    ];

    if input.tenancy_type != TenancyType::ResidentialRentalCoveredByHstpa {
        return Output {
            mode: HstpaMode::NotApplicableTenancyExemptFromHstpa,
            statutory_basis: "HSTPA — applies only to residential leaseholds; commercial / hotel-motel / owner-occupied 2 or 3-family dwellings exempt".to_string(),
            notes: "NOT APPLICABLE: tenancy type exempt from HSTPA (commercial rental; hotel/motel transient lodging; owner-occupied 2-family or 3-family dwelling).".to_string(),
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositCapOneMonthUnderGolSection7_103 => {
            let cap = input
                .monthly_rent_dollars
                .saturating_mul(u64::from(HSTPA_SECURITY_DEPOSIT_CAP_IN_MONTHS_OF_RENT));
            if input.security_deposit_dollars <= cap {
                Output {
                    mode: HstpaMode::CompliantSecurityDepositAtOrBelowOneMonthCap,
                    statutory_basis: "GOL § 7-103 — security deposit at or below 1-month-rent statutory cap".to_string(),
                    notes: "COMPLIANT: security deposit at or below 1 month's rent statutory cap under HSTPA amendment to GOL § 7-103.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: HstpaMode::ViolationSecurityDepositExceedsOneMonthCap,
                    statutory_basis: "GOL § 7-103 — security deposit exceeds 1-month-rent statutory cap".to_string(),
                    notes: "VIOLATION: security deposit exceeds 1 month's rent cap under HSTPA amendment to GOL § 7-103; landlord must refund excess.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::SecurityDepositReturnFourteenDayDeadlineUnderGolSection7_108 => {
            if input.deposit_returned_and_itemized_statement_within_window
                && input.days_since_tenant_vacated_for_deposit_return
                    <= HSTPA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS
            {
                Output {
                    mode: HstpaMode::CompliantSecurityDepositReturnedWithinFourteenDays,
                    statutory_basis: "GOL § 7-108 — security deposit returned with itemized statement within 14-day statutory deadline".to_string(),
                    notes: "COMPLIANT: landlord provided itemized statement of claimed conditions AND remaining portion of security deposit within 14-day window after tenant vacated under GOL § 7-108.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: HstpaMode::ViolationSecurityDepositReturnedPastFourteenDayDeadline,
                    statutory_basis: "GOL § 7-108 — security deposit not returned within 14-day statutory deadline; right to withhold WAIVED".to_string(),
                    notes: "VIOLATION: landlord missed 14-day deposit return deadline under GOL § 7-108; HSTPA WAIVES landlord's right to withhold any portion of the deposit; landlord must return full deposit.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::LateFeeFiveDayGracePeriodAndDollarOrPercentCapUnderRplSection238A => {
            if input.days_rent_late_when_late_fee_charged < HSTPA_LATE_FEE_GRACE_PERIOD_DAYS {
                Output {
                    mode: HstpaMode::ViolationLateFeeChargedWithinFiveDayGracePeriod,
                    statutory_basis: "RPL § 238-a — late fee charged within 5-day grace period prohibited".to_string(),
                    notes: "VIOLATION: late fee charged before rent was at least 5 days late; HSTPA RPL § 238-a prohibits late fees during the 5-day grace period.".to_string(),
                    citations,
                }
            } else {
                let five_pct_cap = (u128::from(input.monthly_rent_dollars) * 500 / 10_000) as u64;
                let cap = HSTPA_LATE_FEE_DOLLAR_CAP.min(five_pct_cap);
                if input.late_fee_charged_dollars <= cap {
                    Output {
                        mode: HstpaMode::CompliantLateFeeWithinGracePeriodAndCap,
                        statutory_basis: "RPL § 238-a — late fee within grace period and within statutory cap".to_string(),
                        notes: "COMPLIANT: late fee charged after 5-day grace period AND within statutory cap of LESSER OF $50 OR 5 % of monthly rent under RPL § 238-a.".to_string(),
                        citations,
                    }
                } else {
                    Output {
                        mode: HstpaMode::ViolationLateFeeExceedsLesserOfFiftyDollarsOrFivePercent,
                        statutory_basis: "RPL § 238-a — late fee exceeds lesser of $50 or 5 % of monthly rent".to_string(),
                        notes: "VIOLATION: late fee exceeds the statutory cap of LESSER OF $50 OR 5 % of monthly rent under RPL § 238-a.".to_string(),
                        citations,
                    }
                }
            }
        }
        ComplianceAspect::ApplicationFeeTwentyDollarCapUnderRplSection238A => {
            if input.application_fee_charged_dollars <= HSTPA_APPLICATION_FEE_CAP_DOLLARS {
                Output {
                    mode: HstpaMode::CompliantApplicationFeeAtOrBelowTwentyDollarCap,
                    statutory_basis: "RPL § 238-a — application fee at or below $20 statewide cap".to_string(),
                    notes: "COMPLIANT: application fee (background check + tenant screening combined) at or below $20 statewide statutory cap under RPL § 238-a.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: HstpaMode::ViolationApplicationFeeExceedsTwentyDollarCap,
                    statutory_basis: "RPL § 238-a — application fee exceeds $20 statewide cap".to_string(),
                    notes: "VIOLATION: application fee exceeds $20 statewide statutory cap under RPL § 238-a; HSTPA limits combined background check + tenant screening fee to $20 maximum.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::TerminationNoticeBasedOnTenancyDurationUnderRplSection226C => {
            let required_days = match input.tenancy_duration {
                TenancyDurationCategory::LessThanOneYear => {
                    HSTPA_TERMINATION_NOTICE_UNDER_1_YEAR_DAYS
                }
                TenancyDurationCategory::AtLeastOneYearButLessThanTwoYears => {
                    HSTPA_TERMINATION_NOTICE_1_TO_2_YEARS_DAYS
                }
                TenancyDurationCategory::AtLeastTwoYears => {
                    HSTPA_TERMINATION_NOTICE_2_PLUS_YEARS_DAYS
                }
            };
            if input.termination_notice_days_given >= required_days {
                Output {
                    mode: HstpaMode::CompliantTerminationNoticeMeetsStatutoryLengthForTenancyDuration,
                    statutory_basis: "RPL § 226-c — termination notice meets statutory length for tenancy duration".to_string(),
                    notes: "COMPLIANT: landlord provided termination / non-renewal notice meeting RPL § 226-c statutory length based on tenancy duration (30 days for tenancy under 1 year; 60 days for 1-2 years; 90 days for 2+ years).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: HstpaMode::ViolationTerminationNoticeShorterThanStatutoryLengthForTenancyDuration,
                    statutory_basis: "RPL § 226-c — termination notice shorter than statutory length for tenancy duration".to_string(),
                    notes: "VIOLATION: termination / non-renewal notice shorter than RPL § 226-c statutory length for tenancy duration; landlord termination invalid.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::NonpaymentWrittenDemandFourteenDayNoticeUnderRpaplSection711_2 => {
            if input.nonpayment_notice_is_written
                && input.nonpayment_notice_days_given >= HSTPA_NONPAYMENT_WRITTEN_DEMAND_NOTICE_DAYS
            {
                Output {
                    mode: HstpaMode::CompliantNonpaymentWrittenDemandFourteenDayNoticeProvided,
                    statutory_basis: "RPAPL § 711(2) — written nonpayment demand with 14-day notice provided".to_string(),
                    notes: "COMPLIANT: landlord provided WRITTEN rent demand with at least 14 days to pay or quit for nonpayment proceeding under HSTPA-amended RPAPL § 711(2).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: HstpaMode::ViolationNonpaymentOralDemandOrShorterThanFourteenDayWrittenDemand,
                    statutory_basis: "RPAPL § 711(2) — nonpayment demand oral or shorter than 14-day written demand".to_string(),
                    notes: "VIOLATION: nonpayment predicate notice was oral OR written demand period was shorter than 14 days; HSTPA-amended RPAPL § 711(2) abolished oral rent demands and requires WRITTEN demand with 14 days; nonpayment proceeding subject to dismissal.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::PostWarrantHardshipStayUnderRpaplSection753 => {
            if input.post_warrant_hardship_stay_days_granted
                <= HSTPA_POST_WARRANT_HARDSHIP_STAY_MAX_DAYS
            {
                Output {
                    mode: HstpaMode::CompliantPostWarrantHardshipStayWithinOneYearMaximum,
                    statutory_basis: "RPAPL § 753 — post-warrant hardship stay within 1-year statutory maximum".to_string(),
                    notes: "COMPLIANT: court granted post-warrant hardship stay within HSTPA-amended RPAPL § 753 1-year maximum (365 days).".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: HstpaMode::CompliantPostWarrantHardshipStayWithinOneYearMaximum,
                    statutory_basis: "RPAPL § 753 — post-warrant hardship stay limited to 1-year maximum".to_string(),
                    notes: "NOTE: court-granted post-warrant hardship stay exceeds 1-year statutory maximum under RPAPL § 753; court order subject to challenge.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::EliminationOfVacancyBonusInRentRegulatedUnits => {
            if input.vacancy_bonus_claimed_in_rent_regulated_unit {
                Output {
                    mode: HstpaMode::ViolationVacancyBonusClaimedInRentRegulatedUnit,
                    statutory_basis: "HSTPA elimination of vacancy bonus — landlord may not claim vacancy bonus on rent-regulated unit turnover".to_string(),
                    notes: "VIOLATION: landlord claimed 20 % 'vacancy bonus' rent increase on rent-regulated unit turnover; HSTPA eliminated the vacancy bonus entirely; rent must remain at prior regulated rate.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: HstpaMode::CompliantNoVacancyBonusClaimed,
                    statutory_basis: "HSTPA elimination of vacancy bonus — landlord respected vacancy bonus elimination".to_string(),
                    notes: "COMPLIANT: landlord did not claim vacancy bonus on rent-regulated unit turnover; HSTPA elimination respected.".to_string(),
                    citations,
                }
            }
        }
        ComplianceAspect::ImpliedWarrantyOfHabitabilityUnderRplSection235B => {
            if input.implied_warranty_of_habitability_maintained {
                Output {
                    mode: HstpaMode::CompliantImpliedWarrantyOfHabitabilityMaintained,
                    statutory_basis: "RPL § 235-b — implied warranty of habitability maintained".to_string(),
                    notes: "COMPLIANT: landlord maintains premises in compliance with the statewide implied warranty of habitability under RPL § 235-b.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: HstpaMode::ViolationImpliedWarrantyOfHabitabilityBreached,
                    statutory_basis: "RPL § 235-b — implied warranty of habitability breached".to_string(),
                    notes: "VIOLATION: landlord breached implied warranty of habitability under RPL § 235-b; HSTPA strengthened tenant remedies (uncapped damages + injunctive relief).".to_string(),
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
            tenancy_type: TenancyType::ResidentialRentalCoveredByHstpa,
            compliance_aspect: ComplianceAspect::SecurityDepositCapOneMonthUnderGolSection7_103,
            tenancy_duration: TenancyDurationCategory::LessThanOneYear,
            monthly_rent_dollars: 2_000,
            security_deposit_dollars: 2_000,
            days_since_tenant_vacated_for_deposit_return: 10,
            deposit_returned_and_itemized_statement_within_window: true,
            days_rent_late_when_late_fee_charged: 7,
            late_fee_charged_dollars: 50,
            application_fee_charged_dollars: 20,
            termination_notice_days_given: 30,
            nonpayment_notice_is_written: true,
            nonpayment_notice_days_given: 14,
            post_warrant_hardship_stay_days_granted: 180,
            vacancy_bonus_claimed_in_rent_regulated_unit: false,
            implied_warranty_of_habitability_maintained: true,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_type = TenancyType::CommercialRentalExempt;
        let output = check(&input);
        assert_eq!(output.mode, HstpaMode::NotApplicableTenancyExemptFromHstpa);
    }

    #[test]
    fn security_deposit_at_one_month_cap_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            HstpaMode::CompliantSecurityDepositAtOrBelowOneMonthCap
        );
    }

    #[test]
    fn security_deposit_at_one_month_plus_one_dollar_violation() {
        let mut input = baseline_input();
        input.security_deposit_dollars = 2_001;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::ViolationSecurityDepositExceedsOneMonthCap
        );
    }

    #[test]
    fn deposit_return_within_fourteen_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnFourteenDayDeadlineUnderGolSection7_108;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::CompliantSecurityDepositReturnedWithinFourteenDays
        );
    }

    #[test]
    fn deposit_return_at_exactly_fourteen_day_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnFourteenDayDeadlineUnderGolSection7_108;
        input.days_since_tenant_vacated_for_deposit_return = 14;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::CompliantSecurityDepositReturnedWithinFourteenDays
        );
    }

    #[test]
    fn deposit_return_at_fifteen_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::SecurityDepositReturnFourteenDayDeadlineUnderGolSection7_108;
        input.days_since_tenant_vacated_for_deposit_return = 15;
        input.deposit_returned_and_itemized_statement_within_window = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::ViolationSecurityDepositReturnedPastFourteenDayDeadline
        );
    }

    #[test]
    fn late_fee_after_five_day_grace_period_and_at_cap_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LateFeeFiveDayGracePeriodAndDollarOrPercentCapUnderRplSection238A;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::CompliantLateFeeWithinGracePeriodAndCap
        );
    }

    #[test]
    fn late_fee_within_grace_period_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LateFeeFiveDayGracePeriodAndDollarOrPercentCapUnderRplSection238A;
        input.days_rent_late_when_late_fee_charged = 4;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::ViolationLateFeeChargedWithinFiveDayGracePeriod
        );
    }

    #[test]
    fn late_fee_at_exactly_five_day_grace_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LateFeeFiveDayGracePeriodAndDollarOrPercentCapUnderRplSection238A;
        input.days_rent_late_when_late_fee_charged = 5;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::CompliantLateFeeWithinGracePeriodAndCap
        );
    }

    #[test]
    fn late_fee_exceeds_fifty_dollar_cap_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LateFeeFiveDayGracePeriodAndDollarOrPercentCapUnderRplSection238A;
        input.late_fee_charged_dollars = 51;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::ViolationLateFeeExceedsLesserOfFiftyDollarsOrFivePercent
        );
    }

    #[test]
    fn late_fee_capped_by_five_percent_when_rent_below_one_thousand() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::LateFeeFiveDayGracePeriodAndDollarOrPercentCapUnderRplSection238A;
        input.monthly_rent_dollars = 500;
        input.late_fee_charged_dollars = 26;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::ViolationLateFeeExceedsLesserOfFiftyDollarsOrFivePercent
        );
    }

    #[test]
    fn application_fee_at_twenty_dollar_cap_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::ApplicationFeeTwentyDollarCapUnderRplSection238A;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::CompliantApplicationFeeAtOrBelowTwentyDollarCap
        );
    }

    #[test]
    fn application_fee_at_twenty_one_dollars_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::ApplicationFeeTwentyDollarCapUnderRplSection238A;
        input.application_fee_charged_dollars = 21;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::ViolationApplicationFeeExceedsTwentyDollarCap
        );
    }

    #[test]
    fn termination_notice_under_one_year_at_thirty_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TerminationNoticeBasedOnTenancyDurationUnderRplSection226C;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::CompliantTerminationNoticeMeetsStatutoryLengthForTenancyDuration
        );
    }

    #[test]
    fn termination_notice_at_one_to_two_years_at_sixty_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TerminationNoticeBasedOnTenancyDurationUnderRplSection226C;
        input.tenancy_duration = TenancyDurationCategory::AtLeastOneYearButLessThanTwoYears;
        input.termination_notice_days_given = 60;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::CompliantTerminationNoticeMeetsStatutoryLengthForTenancyDuration
        );
    }

    #[test]
    fn termination_notice_at_two_plus_years_at_ninety_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TerminationNoticeBasedOnTenancyDurationUnderRplSection226C;
        input.tenancy_duration = TenancyDurationCategory::AtLeastTwoYears;
        input.termination_notice_days_given = 90;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::CompliantTerminationNoticeMeetsStatutoryLengthForTenancyDuration
        );
    }

    #[test]
    fn termination_notice_at_two_plus_years_at_eighty_nine_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::TerminationNoticeBasedOnTenancyDurationUnderRplSection226C;
        input.tenancy_duration = TenancyDurationCategory::AtLeastTwoYears;
        input.termination_notice_days_given = 89;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::ViolationTerminationNoticeShorterThanStatutoryLengthForTenancyDuration
        );
    }

    #[test]
    fn nonpayment_written_fourteen_day_demand_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::NonpaymentWrittenDemandFourteenDayNoticeUnderRpaplSection711_2;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::CompliantNonpaymentWrittenDemandFourteenDayNoticeProvided
        );
    }

    #[test]
    fn nonpayment_oral_demand_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::NonpaymentWrittenDemandFourteenDayNoticeUnderRpaplSection711_2;
        input.nonpayment_notice_is_written = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::ViolationNonpaymentOralDemandOrShorterThanFourteenDayWrittenDemand
        );
    }

    #[test]
    fn nonpayment_written_demand_under_fourteen_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::NonpaymentWrittenDemandFourteenDayNoticeUnderRpaplSection711_2;
        input.nonpayment_notice_days_given = 3;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::ViolationNonpaymentOralDemandOrShorterThanFourteenDayWrittenDemand
        );
    }

    #[test]
    fn no_vacancy_bonus_claimed_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EliminationOfVacancyBonusInRentRegulatedUnits;
        let output = check(&input);
        assert_eq!(output.mode, HstpaMode::CompliantNoVacancyBonusClaimed);
    }

    #[test]
    fn vacancy_bonus_claimed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EliminationOfVacancyBonusInRentRegulatedUnits;
        input.vacancy_bonus_claimed_in_rent_regulated_unit = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::ViolationVacancyBonusClaimedInRentRegulatedUnit
        );
    }

    #[test]
    fn implied_warranty_of_habitability_maintained_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::ImpliedWarrantyOfHabitabilityUnderRplSection235B;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::CompliantImpliedWarrantyOfHabitabilityMaintained
        );
    }

    #[test]
    fn implied_warranty_of_habitability_breached_violation() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::ImpliedWarrantyOfHabitabilityUnderRplSection235B;
        input.implied_warranty_of_habitability_maintained = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HstpaMode::ViolationImpliedWarrantyOfHabitabilityBreached
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(HSTPA_ENACTMENT_DATE_YEAR, 2019);
        assert_eq!(HSTPA_ENACTMENT_DATE_MONTH, 6);
        assert_eq!(HSTPA_ENACTMENT_DATE_DAY, 14);
        assert_eq!(HSTPA_LAW_CHAPTER_NUMBER, 36);
        assert_eq!(HSTPA_SECURITY_DEPOSIT_CAP_IN_MONTHS_OF_RENT, 1);
        assert_eq!(HSTPA_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 14);
        assert_eq!(HSTPA_LATE_FEE_GRACE_PERIOD_DAYS, 5);
        assert_eq!(HSTPA_LATE_FEE_DOLLAR_CAP, 50);
        assert_eq!(HSTPA_LATE_FEE_PERCENT_CAP_BPS, 500);
        assert_eq!(HSTPA_APPLICATION_FEE_CAP_DOLLARS, 20);
        assert_eq!(HSTPA_NONPAYMENT_WRITTEN_DEMAND_NOTICE_DAYS, 14);
        assert_eq!(HSTPA_TERMINATION_NOTICE_UNDER_1_YEAR_DAYS, 30);
        assert_eq!(HSTPA_TERMINATION_NOTICE_1_TO_2_YEARS_DAYS, 60);
        assert_eq!(HSTPA_TERMINATION_NOTICE_2_PLUS_YEARS_DAYS, 90);
        assert_eq!(HSTPA_POST_WARRANT_HARDSHIP_STAY_MAX_DAYS, 365);
        assert_eq!(HSTPA_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Housing Stability and Tenant Protection Act of 2019"));
        assert!(joined.contains("Chapter 36 of the Laws of 2019"));
        assert!(joined.contains("JUNE 14, 2019"));
        assert!(joined.contains("Governor Andrew M. Cuomo"));
        assert!(joined.contains("General Obligations Law § 7-103"));
        assert!(joined.contains("General Obligations Law § 7-108"));
        assert!(joined.contains("Real Property Law § 238-a"));
        assert!(joined.contains("Real Property Law § 226-c"));
        assert!(joined.contains("RPAPL"));
        assert!(joined.contains("§ 711(2)"));
        assert!(joined.contains("§ 753"));
        assert!(joined.contains("Real Property Law § 235-b"));
        assert!(joined.contains("ONE MONTH"));
        assert!(joined.contains("14 DAYS"));
        assert!(joined.contains("5 DAYS LATE"));
        assert!(joined.contains("$50"));
        assert!(joined.contains("5 %"));
        assert!(joined.contains("$20"));
        assert!(joined.contains("30-DAY NOTICE"));
        assert!(joined.contains("60-DAY NOTICE"));
        assert!(joined.contains("90-DAY NOTICE"));
        assert!(joined.contains("vacancy bonus"));
        assert!(joined.contains("Emergency Tenant Protection Act of 1974"));
        assert!(joined.contains("STATEWIDE"));
    }
}
