//! Cook County Residential Tenant and Landlord Ordinance
//! (RTLO) Compliance Module — Cook County, Illinois' first
//! comprehensive county-wide residential landlord-tenant
//! statute, adopted by the Cook County Board of Commissioners
//! on January 28, 2021, anti-lockout provisions effective
//! January 2021, full ordinance effective June 1, 2021.
//!
//! Pure-compute check for landlord compliance with the Cook
//! County RTLO codified at Cook County Code of Ordinances
//! Chapter 42 ("Human Relations") Article XII ("Residential
//! Tenant and Landlord Ordinance"). The RTLO extends Chicago-
//! style tenant protections to suburban Cook County (the City
//! of Chicago is EXCLUDED from RTLO scope — Chicago has its
//! own Chicago Residential Landlord and Tenant Ordinance
//! (CRLTO) under MCC Chapter 5-12 dating to 1986). The RTLO
//! covers approximately 245,000 suburban Cook County rental
//! units across 130+ municipalities.
//!
//! Web research (verified 2026-06-03):
//! - **Adoption**: Cook County Board of Commissioners adopted
//!   the RTLO on January 28, 2021. **Anti-lockout provisions**
//!   took effect immediately in January 2021. The remainder of
//!   the ordinance took effect on **June 1, 2021** ([Cook
//!   County Government — Residential Tenant Landlord
//!   Ordinance](https://www.cookcountyil.gov/rtlo); [Cook
//!   County News — Cook County's First Residential Tenant
//!   Landlord Ordinance Goes Into Effect June 1](https://www.cookcountyil.gov/news/cook-countys-first-residential-tenant-landlord-ordinance-goes-effect-june-1);
//!   [Housing Action Illinois — Cook County RTLO](https://housingactionil.org/what-we-do/advocacy/rental-affordability/rtlo/);
//!   [Progress Center for Independent Living — New Cook
//!   County Renter Rights Take Effect June 1](https://progresscil.org/2021/06/01/new-cook-county-renter-rights-and-protections-take-effect-june-1/)).
//! - **Codification**: Cook County Code of Ordinances Chapter
//!   42 Article XII (Cook County Legistar). Distinct from
//!   the **Chicago RLTO** (Chicago Municipal Code 5-12),
//!   which preempts the Cook County RTLO within Chicago
//!   city limits.
//! - **Scope**: applies to substantially all suburban Cook
//!   County rental units including mobile homes and
//!   subsidized housing. Anti-lockout provisions apply to
//!   ALL units, including otherwise-exempt categories.
//! - **Exemptions** (except anti-lockout): (1) owner-occupied
//!   buildings with **6 OR FEWER units**; (2) single-family
//!   dwelling or condominium where the owner rents only that
//!   property AND has lived in it within the past **12
//!   MONTHS**; (3) single room occupancy (SRO) housing for
//!   vulnerable residents; (4) hotel/motel units UNLESS rental
//!   period exceeds **32 DAYS** monthly; (5) school
//!   dormitories; (6) shelters; (7) employee quarters; (8)
//!   non-residential properties; (9) owner-occupied
//!   cooperatives ([Cook County Government — RTLO Coverage
//!   Guide](https://www.cookcountyil.gov/rtlo); [Rentervention
//!   — Who's Covered? A Guide to the Cook County RTLO](https://help.rentervention.com/article/80-cook-county-rtlo-general)).
//! - **Security Deposit Cap**: maximum **1.5 TIMES MONTHLY
//!   RENT**; must be held in a separate account from the
//!   landlord's personal funds; receipt required upon
//!   collection.
//! - **Security Deposit Return**: must be returned within
//!   **30 DAYS** after termination of tenancy with itemized
//!   list of deductions for damages beyond normal wear and
//!   tear.
//! - **Late Fee Cap**: **$10 PER MONTH FOR THE FIRST $1,000 OF
//!   MONTHLY RENT, PLUS 5 % OF ANY MONTHLY RENT AMOUNT IN
//!   EXCESS OF $1,000**.
//! - **Material Noncompliance Cure Notice**: tenant must
//!   receive **10 DAYS** notice to cure material
//!   noncompliance before landlord may terminate.
//! - **Lease Renewal Notice**: landlord must provide tenant
//!   **60 DAYS** advance notice of any material change to
//!   lease terms upon renewal (reduced from 90 days during
//!   ordinance drafting).
//! - **Entry Notice**: landlord must provide **2 DAYS** notice
//!   for entry; entry permitted only between **8:00 a.m. and
//!   8:00 p.m.** for repairs, prospective services, or
//!   showings within 60 days of a lease ending.
//! - **Anti-Lockout**: any self-help eviction (lockout,
//!   utility shutoff, removal of doors/windows, etc.) is
//!   prohibited; tenant entitled to actual damages, statutory
//!   damages, attorney fees, and emergency injunctive relief.
//! - **Retaliation Prohibition**: landlord may NOT retaliate
//!   against tenant for exercising rights under the
//!   ordinance, complaining to government, joining a tenant
//!   organization, or testifying. Rebuttable presumption of
//!   retaliation if landlord action occurs within statutory
//!   lookback after protected tenant activity.
//! - **Required Disclosures**: utility-payment responsibility + estimated annual utility cost; move-in fee itemization; owner/manager contact for service of process; code violations and utility shutoffs from prior 12 months (case numbers + citations); EPA bed bug pamphlet; RTLO Ordinance Summary attachment.
//! - **Enforcement Mechanisms**: Cook County Commission on
//!   Human Rights administrative complaint process; private
//!   right of action with **two times actual damages or one
//!   month's rent, whichever is greater** for security
//!   deposit violations; attorney fees for prevailing tenants.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const COOK_COUNTY_RTLO_ENACTMENT_YEAR: u32 = 2021;
pub const COOK_COUNTY_RTLO_ENACTMENT_MONTH: u32 = 1;
pub const COOK_COUNTY_RTLO_ENACTMENT_DAY: u32 = 28;
pub const COOK_COUNTY_RTLO_EFFECTIVE_DATE_YEAR: u32 = 2021;
pub const COOK_COUNTY_RTLO_EFFECTIVE_DATE_MONTH: u32 = 6;
pub const COOK_COUNTY_RTLO_EFFECTIVE_DATE_DAY: u32 = 1;
pub const COOK_COUNTY_RTLO_SECURITY_DEPOSIT_CAP_NUMERATOR: u64 = 3;
pub const COOK_COUNTY_RTLO_SECURITY_DEPOSIT_CAP_DENOMINATOR: u64 = 2;
pub const COOK_COUNTY_RTLO_SECURITY_DEPOSIT_RETURN_DAYS: u32 = 30;
pub const COOK_COUNTY_RTLO_LATE_FEE_FLAT_PORTION_DOLLARS: u64 = 10;
pub const COOK_COUNTY_RTLO_LATE_FEE_FLAT_PORTION_RENT_THRESHOLD_DOLLARS: u64 = 1_000;
pub const COOK_COUNTY_RTLO_LATE_FEE_EXCESS_BASIS_POINTS: u64 = 500;
pub const COOK_COUNTY_RTLO_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const COOK_COUNTY_RTLO_OWNER_OCCUPIED_BUILDING_EXEMPTION_MAX_UNITS: u32 = 6;
pub const COOK_COUNTY_RTLO_SINGLE_FAMILY_OWNER_LIVED_THERE_MONTHS_THRESHOLD: u32 = 12;
pub const COOK_COUNTY_RTLO_HOTEL_MOTEL_COVERAGE_DAYS_THRESHOLD: u32 = 32;
pub const COOK_COUNTY_RTLO_MATERIAL_NONCOMPLIANCE_CURE_NOTICE_DAYS: u32 = 10;
pub const COOK_COUNTY_RTLO_LEASE_RENEWAL_NOTICE_DAYS: u32 = 60;
pub const COOK_COUNTY_RTLO_ENTRY_NOTICE_DAYS: u32 = 2;
pub const COOK_COUNTY_RTLO_ENTRY_TIME_WINDOW_START_HOUR_24H: u32 = 8;
pub const COOK_COUNTY_RTLO_ENTRY_TIME_WINDOW_END_HOUR_24H: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    WithinSuburbanCookCountyCovered,
    WithinChicagoCityLimitsExcludedCoveredByChicagoCRLTO,
    OutsideCookCounty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExemptionStatus {
    NotExemptFullyCovered,
    ExemptOwnerOccupiedBuildingWith6OrFewerUnits,
    ExemptSingleFamilyOrCondoOwnerLivedThereWithin12Months,
    ExemptSroForVulnerableResidents,
    ExemptHotelOrMotelMonthlyRentalUnder32Days,
    ExemptSchoolDormitory,
    ExemptShelter,
    ExemptEmployeeQuarters,
    ExemptNonResidentialProperty,
    ExemptOwnerOccupiedCooperative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositCollectionAndReturn,
    LateFeeAssessment,
    TenDayMaterialNoncomplianceCureNotice,
    EntryNoticeAndTimeWindow,
    SixtyDayLeaseRenewalNotice,
    AntiLockoutProhibition,
    RetaliationProhibition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AntiLockoutStatus {
    NoLockoutOrSelfHelpEvictionPerformed,
    LockoutOrSelfHelpEvictionPerformedViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetaliationStatus {
    NoRetaliatoryActionTaken,
    RetaliatoryActionTakenAfterTenantProtectedActivity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CookCountyRtloMode {
    NotApplicablePropertyOutsideCookCounty,
    NotApplicablePropertyInsideChicagoCityLimitsCoveredByChicagoCRLTO,
    NotApplicableOwnerOccupiedBuildingWith6OrFewerUnitsExempt,
    NotApplicableSingleFamilyOrCondoOwnerLivedThereWithin12MonthsExempt,
    NotApplicableSroForVulnerableResidentsExempt,
    NotApplicableHotelOrMotelMonthlyRentalUnder32DaysExempt,
    NotApplicableSchoolDormitoryExempt,
    NotApplicableShelterExempt,
    NotApplicableEmployeeQuartersExempt,
    NotApplicableNonResidentialPropertyExempt,
    NotApplicableOwnerOccupiedCooperativeExempt,
    CompliantSecurityDepositAtOrBelow1_5MonthsRentReturnedWithin30DaysWithItemization,
    CompliantLateFeeAtOrBelowStatutoryCap,
    CompliantTenDayMaterialNoncomplianceCureNoticeProvided,
    CompliantTwoDayEntryNoticeWithin8AmTo8PmWindow,
    CompliantSixtyDayLeaseRenewalNoticeProvided,
    CompliantNoLockoutPerformed,
    CompliantNoRetaliatoryActionTaken,
    ViolationSecurityDepositCapExceeded1_5MonthsRent,
    ViolationSecurityDepositNotReturnedWithin30Days,
    ViolationSecurityDepositReturnedWithoutItemization,
    ViolationLateFeeExceedsStatutoryCap,
    ViolationTenDayMaterialNoncomplianceCureNoticeNotProvided,
    ViolationEntryNoticeLessThan2DaysOrEntryOutside8AmTo8PmWindow,
    ViolationSixtyDayLeaseRenewalNoticeNotProvided,
    ViolationLockoutOrSelfHelpEvictionPerformed,
    ViolationRetaliatoryActionTakenAfterTenantProtectedActivity,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub exemption_status: ExemptionStatus,
    pub compliance_aspect: ComplianceAspect,
    pub monthly_rent_cents: u64,
    pub security_deposit_held_cents: u64,
    pub security_deposit_return_days_after_termination: u32,
    pub security_deposit_itemization_provided: bool,
    pub late_fee_charged_cents: u64,
    pub ten_day_cure_notice_provided: bool,
    pub entry_notice_days_provided: u32,
    pub entry_within_8am_to_8pm_window: bool,
    pub lease_renewal_notice_days_provided: u32,
    pub anti_lockout_status: AntiLockoutStatus,
    pub retaliation_status: RetaliationStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: CookCountyRtloMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub statutory_security_deposit_cap_cents: u64,
    pub statutory_late_fee_cap_cents: u64,
    pub statutory_damages_owed_cents: u64,
}

pub type RentalCookCountyRtloInput = Input;
pub type RentalCookCountyRtloOutput = Output;
pub type RentalCookCountyRtloResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Cook County Residential Tenant and Landlord Ordinance (RTLO) — adopted by Cook County Board of Commissioners on January 28, 2021; anti-lockout provisions effective January 2021; remainder effective June 1, 2021; codified at Cook County Code of Ordinances Chapter 42 (Human Relations) Article XII (Residential Tenant and Landlord Ordinance)".to_string(),
        "Scope — substantially all suburban Cook County rental units including mobile homes and subsidized housing; anti-lockout provisions apply to ALL units (including otherwise-exempt categories)".to_string(),
        "Jurisdictional Exclusion — City of Chicago is EXCLUDED from Cook County RTLO scope; Chicago has its own Chicago Residential Landlord and Tenant Ordinance (CRLTO) codified at MCC Chapter 5-12 (effective 1986)".to_string(),
        "Exemptions (except anti-lockout) — (1) owner-occupied buildings with 6 or fewer units; (2) single-family dwelling or condominium where owner rents only that property AND has lived in it within past 12 months; (3) SRO housing for vulnerable residents; (4) hotel/motel units UNLESS monthly rental exceeds 32 days; (5) school dormitories; (6) shelters; (7) employee quarters; (8) non-residential properties; (9) owner-occupied cooperatives".to_string(),
        "Security Deposit Cap — maximum 1.5 TIMES MONTHLY RENT; must be held in separate account from landlord personal funds; receipt required upon collection".to_string(),
        "Security Deposit Return — must be returned within 30 DAYS after termination of tenancy with itemized list of deductions for damages beyond normal wear and tear".to_string(),
        "Late Fee Cap — $10 per month for the first $1,000 of monthly rent, PLUS 5 % of any monthly rent amount in EXCESS of $1,000".to_string(),
        "Material Noncompliance Cure Notice — tenant must receive 10 DAYS written notice to cure material noncompliance before landlord may terminate".to_string(),
        "Lease Renewal Notice — landlord must provide tenant 60 DAYS advance written notice of any material change to lease terms upon renewal (reduced from 90 days during ordinance drafting)".to_string(),
        "Entry Notice — landlord must provide 2 DAYS notice for entry; entry permitted only between 8:00 a.m. and 8:00 p.m. for repairs, prospective services, or showings within 60 days of a lease ending".to_string(),
        "Anti-Lockout — any self-help eviction (lockout, utility shutoff, removal of doors/windows, etc.) prohibited; tenant entitled to actual damages, statutory damages, attorney fees, and emergency injunctive relief".to_string(),
        "Retaliation Prohibition — landlord may NOT retaliate against tenant for exercising rights under ordinance, complaining to government, joining tenant organization, or testifying; rebuttable presumption of retaliation if landlord action occurs within statutory lookback after protected tenant activity".to_string(),
        "Required Disclosures — utility-payment responsibility + estimated annual utility cost; move-in fee itemization; owner/manager contact information for service of process; code violations and utility shutoffs from prior 12 months (case numbers + citations); EPA bed bug pamphlet; RTLO Ordinance Summary attachment".to_string(),
        "Remedies — Cook County Commission on Human Rights administrative complaint process; private right of action; two times actual damages or one month's rent (whichever greater) for security deposit violations; attorney fees for prevailing tenants".to_string(),
        "Cook County Government — Residential Tenant Landlord Ordinance program page".to_string(),
        "Cook County News — Cook County's First Residential Tenant Landlord Ordinance Goes Into Effect June 1 (2021)".to_string(),
        "Housing Action Illinois — Cook County RTLO advocacy resource".to_string(),
        "Chicago Association of REALTORS — Cook County RTLO Issue Summary".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::OutsideCookCounty {
        return Output {
            mode: CookCountyRtloMode::NotApplicablePropertyOutsideCookCounty,
            statutory_basis: "Property outside Cook County; Cook County RTLO inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside Cook County; Cook County Code of Ordinances Chapter 42 Article XII inapplicable.".to_string(),
            citations,
            statutory_security_deposit_cap_cents: 0,
            statutory_late_fee_cap_cents: 0,
            statutory_damages_owed_cents: 0,
        };
    }

    if input.property_jurisdiction
        == PropertyJurisdiction::WithinChicagoCityLimitsExcludedCoveredByChicagoCRLTO
    {
        return Output {
            mode: CookCountyRtloMode::NotApplicablePropertyInsideChicagoCityLimitsCoveredByChicagoCRLTO,
            statutory_basis: "Cook County RTLO scope excludes City of Chicago; Chicago RLTO (MCC Chapter 5-12) applies".to_string(),
            notes: "NOT APPLICABLE: property inside Chicago city limits; Cook County RTLO scope excludes the City of Chicago; the Chicago Residential Landlord and Tenant Ordinance (MCC Chapter 5-12) applies instead.".to_string(),
            citations,
            statutory_security_deposit_cap_cents: 0,
            statutory_late_fee_cap_cents: 0,
            statutory_damages_owed_cents: 0,
        };
    }

    // Anti-lockout provisions apply regardless of exemption status. Handle that aspect first.
    if input.compliance_aspect == ComplianceAspect::AntiLockoutProhibition {
        return match input.anti_lockout_status {
            AntiLockoutStatus::LockoutOrSelfHelpEvictionPerformedViolation => Output {
                mode: CookCountyRtloMode::ViolationLockoutOrSelfHelpEvictionPerformed,
                statutory_basis: "Cook County RTLO § 42-XII anti-lockout provision — applies to ALL units regardless of exemption".to_string(),
                notes: "VIOLATION: landlord performed lockout or self-help eviction; Cook County RTLO anti-lockout provision applies regardless of any other ordinance exemption; tenant entitled to actual damages, statutory damages, attorney fees, and emergency injunctive relief.".to_string(),
                citations,
                statutory_security_deposit_cap_cents: 0,
                statutory_late_fee_cap_cents: 0,
                statutory_damages_owed_cents: 0,
            },
            AntiLockoutStatus::NoLockoutOrSelfHelpEvictionPerformed => Output {
                mode: CookCountyRtloMode::CompliantNoLockoutPerformed,
                statutory_basis: "Cook County RTLO § 42-XII anti-lockout provision".to_string(),
                notes: "COMPLIANT: no lockout or self-help eviction performed; Cook County RTLO anti-lockout provision satisfied.".to_string(),
                citations,
                statutory_security_deposit_cap_cents: 0,
                statutory_late_fee_cap_cents: 0,
                statutory_damages_owed_cents: 0,
            },
        };
    }

    match input.exemption_status {
        ExemptionStatus::ExemptOwnerOccupiedBuildingWith6OrFewerUnits => {
            return Output {
                mode: CookCountyRtloMode::NotApplicableOwnerOccupiedBuildingWith6OrFewerUnitsExempt,
                statutory_basis: "Cook County RTLO § 42-XII — owner-occupied building with 6 or fewer units exemption".to_string(),
                notes: "NOT APPLICABLE: owner-occupied building with 6 or fewer units; RTLO substantive provisions exempt (anti-lockout still applies, addressed separately).".to_string(),
                citations,
                statutory_security_deposit_cap_cents: 0,
                statutory_late_fee_cap_cents: 0,
                statutory_damages_owed_cents: 0,
            };
        }
        ExemptionStatus::ExemptSingleFamilyOrCondoOwnerLivedThereWithin12Months => {
            return Output {
                mode: CookCountyRtloMode::NotApplicableSingleFamilyOrCondoOwnerLivedThereWithin12MonthsExempt,
                statutory_basis: "Cook County RTLO § 42-XII — single-family / condo where owner rents only that property AND lived there within past 12 months exemption".to_string(),
                notes: "NOT APPLICABLE: single-family dwelling or condominium owned by owner who rents only that property and has lived in it within past 12 months; RTLO substantive provisions exempt (anti-lockout still applies, addressed separately).".to_string(),
                citations,
                statutory_security_deposit_cap_cents: 0,
                statutory_late_fee_cap_cents: 0,
                statutory_damages_owed_cents: 0,
            };
        }
        ExemptionStatus::ExemptSroForVulnerableResidents => {
            return Output {
                mode: CookCountyRtloMode::NotApplicableSroForVulnerableResidentsExempt,
                statutory_basis: "Cook County RTLO § 42-XII — single room occupancy for vulnerable residents exemption".to_string(),
                notes: "NOT APPLICABLE: SRO housing for vulnerable residents; RTLO substantive provisions exempt (anti-lockout still applies, addressed separately).".to_string(),
                citations,
                statutory_security_deposit_cap_cents: 0,
                statutory_late_fee_cap_cents: 0,
                statutory_damages_owed_cents: 0,
            };
        }
        ExemptionStatus::ExemptHotelOrMotelMonthlyRentalUnder32Days => {
            return Output {
                mode: CookCountyRtloMode::NotApplicableHotelOrMotelMonthlyRentalUnder32DaysExempt,
                statutory_basis: "Cook County RTLO § 42-XII — hotel / motel monthly rental under 32 days exemption".to_string(),
                notes: "NOT APPLICABLE: hotel / motel unit where monthly rental period does not exceed 32 days; RTLO substantive provisions exempt (anti-lockout still applies, addressed separately).".to_string(),
                citations,
                statutory_security_deposit_cap_cents: 0,
                statutory_late_fee_cap_cents: 0,
                statutory_damages_owed_cents: 0,
            };
        }
        ExemptionStatus::ExemptSchoolDormitory => {
            return Output {
                mode: CookCountyRtloMode::NotApplicableSchoolDormitoryExempt,
                statutory_basis: "Cook County RTLO § 42-XII — school dormitory exemption".to_string(),
                notes: "NOT APPLICABLE: school dormitory occupancy; RTLO substantive provisions exempt (anti-lockout still applies, addressed separately).".to_string(),
                citations,
                statutory_security_deposit_cap_cents: 0,
                statutory_late_fee_cap_cents: 0,
                statutory_damages_owed_cents: 0,
            };
        }
        ExemptionStatus::ExemptShelter => {
            return Output {
                mode: CookCountyRtloMode::NotApplicableShelterExempt,
                statutory_basis: "Cook County RTLO § 42-XII — shelter occupancy exemption".to_string(),
                notes: "NOT APPLICABLE: shelter occupancy; RTLO substantive provisions exempt (anti-lockout still applies, addressed separately).".to_string(),
                citations,
                statutory_security_deposit_cap_cents: 0,
                statutory_late_fee_cap_cents: 0,
                statutory_damages_owed_cents: 0,
            };
        }
        ExemptionStatus::ExemptEmployeeQuarters => {
            return Output {
                mode: CookCountyRtloMode::NotApplicableEmployeeQuartersExempt,
                statutory_basis: "Cook County RTLO § 42-XII — employee quarters exemption".to_string(),
                notes: "NOT APPLICABLE: employee quarters occupancy tied to employment; RTLO substantive provisions exempt (anti-lockout still applies, addressed separately).".to_string(),
                citations,
                statutory_security_deposit_cap_cents: 0,
                statutory_late_fee_cap_cents: 0,
                statutory_damages_owed_cents: 0,
            };
        }
        ExemptionStatus::ExemptNonResidentialProperty => {
            return Output {
                mode: CookCountyRtloMode::NotApplicableNonResidentialPropertyExempt,
                statutory_basis: "Cook County RTLO § 42-XII — non-residential property exemption".to_string(),
                notes: "NOT APPLICABLE: non-residential property; RTLO substantive provisions exempt (anti-lockout still applies, addressed separately).".to_string(),
                citations,
                statutory_security_deposit_cap_cents: 0,
                statutory_late_fee_cap_cents: 0,
                statutory_damages_owed_cents: 0,
            };
        }
        ExemptionStatus::ExemptOwnerOccupiedCooperative => {
            return Output {
                mode: CookCountyRtloMode::NotApplicableOwnerOccupiedCooperativeExempt,
                statutory_basis: "Cook County RTLO § 42-XII — owner-occupied cooperative exemption".to_string(),
                notes: "NOT APPLICABLE: owner-occupied cooperative; RTLO substantive provisions exempt (anti-lockout still applies, addressed separately).".to_string(),
                citations,
                statutory_security_deposit_cap_cents: 0,
                statutory_late_fee_cap_cents: 0,
                statutory_damages_owed_cents: 0,
            };
        }
        ExemptionStatus::NotExemptFullyCovered => {}
    }

    let security_deposit_cap_cents = input
        .monthly_rent_cents
        .saturating_mul(COOK_COUNTY_RTLO_SECURITY_DEPOSIT_CAP_NUMERATOR)
        / COOK_COUNTY_RTLO_SECURITY_DEPOSIT_CAP_DENOMINATOR;

    let flat_portion_cents = COOK_COUNTY_RTLO_LATE_FEE_FLAT_PORTION_DOLLARS.saturating_mul(100);
    let monthly_rent_dollars = input.monthly_rent_cents / 100;
    let rent_excess_over_threshold_dollars = monthly_rent_dollars
        .saturating_sub(COOK_COUNTY_RTLO_LATE_FEE_FLAT_PORTION_RENT_THRESHOLD_DOLLARS);
    let excess_portion_dollars = rent_excess_over_threshold_dollars
        .saturating_mul(COOK_COUNTY_RTLO_LATE_FEE_EXCESS_BASIS_POINTS)
        / COOK_COUNTY_RTLO_BASIS_POINT_DENOMINATOR;
    let late_fee_cap_cents =
        flat_portion_cents.saturating_add(excess_portion_dollars.saturating_mul(100));

    match input.compliance_aspect {
        ComplianceAspect::AntiLockoutProhibition => {
            // Handled above unconditionally; should not reach here.
            Output {
                mode: CookCountyRtloMode::CompliantNoLockoutPerformed,
                statutory_basis: "Cook County RTLO § 42-XII anti-lockout provision".to_string(),
                notes: "COMPLIANT: anti-lockout aspect reached via fully-covered branch; no lockout performed.".to_string(),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                statutory_late_fee_cap_cents: late_fee_cap_cents,
                statutory_damages_owed_cents: 0,
            }
        }
        ComplianceAspect::SecurityDepositCollectionAndReturn => {
            if input.security_deposit_held_cents > security_deposit_cap_cents {
                let damages_cents = input
                    .security_deposit_held_cents
                    .saturating_sub(security_deposit_cap_cents)
                    .saturating_mul(2)
                    .max(input.monthly_rent_cents);
                return Output {
                    mode: CookCountyRtloMode::ViolationSecurityDepositCapExceeded1_5MonthsRent,
                    statutory_basis: "Cook County RTLO § 42-XII — security deposit cap 1.5 × monthly rent".to_string(),
                    notes: format!(
                        "VIOLATION: security deposit {} cents exceeds statutory cap of 1.5 × monthly rent ({} cents); excess {} cents; statutory damages = greater of (2 × excess) or one month's rent = {} cents.",
                        input.security_deposit_held_cents,
                        security_deposit_cap_cents,
                        input.security_deposit_held_cents.saturating_sub(security_deposit_cap_cents),
                        damages_cents
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_late_fee_cap_cents: late_fee_cap_cents,
                    statutory_damages_owed_cents: damages_cents,
                };
            }
            if input.security_deposit_return_days_after_termination
                > COOK_COUNTY_RTLO_SECURITY_DEPOSIT_RETURN_DAYS
            {
                let damages_cents = input.security_deposit_held_cents.saturating_mul(2).max(input.monthly_rent_cents);
                return Output {
                    mode: CookCountyRtloMode::ViolationSecurityDepositNotReturnedWithin30Days,
                    statutory_basis: "Cook County RTLO § 42-XII — security deposit must be returned within 30 days of termination".to_string(),
                    notes: format!(
                        "VIOLATION: security deposit returned {} days after termination (> 30-day statutory deadline); statutory damages = greater of (2 × deposit) or one month's rent = {} cents.",
                        input.security_deposit_return_days_after_termination, damages_cents
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_late_fee_cap_cents: late_fee_cap_cents,
                    statutory_damages_owed_cents: damages_cents,
                };
            }
            if !input.security_deposit_itemization_provided {
                let damages_cents = input.security_deposit_held_cents.saturating_mul(2).max(input.monthly_rent_cents);
                return Output {
                    mode: CookCountyRtloMode::ViolationSecurityDepositReturnedWithoutItemization,
                    statutory_basis: "Cook County RTLO § 42-XII — itemization of deductions required upon return".to_string(),
                    notes: format!(
                        "VIOLATION: security deposit returned without itemized list of deductions for damages beyond normal wear and tear; statutory damages = greater of (2 × deposit) or one month's rent = {} cents.",
                        damages_cents
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_late_fee_cap_cents: late_fee_cap_cents,
                    statutory_damages_owed_cents: damages_cents,
                };
            }
            Output {
                mode: CookCountyRtloMode::CompliantSecurityDepositAtOrBelow1_5MonthsRentReturnedWithin30DaysWithItemization,
                statutory_basis: "Cook County RTLO § 42-XII — security deposit cap 1.5 × monthly rent + 30-day return + itemization".to_string(),
                notes: format!(
                    "COMPLIANT: security deposit {} cents ≤ 1.5 × monthly rent cap ({} cents), returned in {} days (≤ 30), itemization provided.",
                    input.security_deposit_held_cents,
                    security_deposit_cap_cents,
                    input.security_deposit_return_days_after_termination
                ),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                statutory_late_fee_cap_cents: late_fee_cap_cents,
                statutory_damages_owed_cents: 0,
            }
        }
        ComplianceAspect::LateFeeAssessment => {
            if input.late_fee_charged_cents > late_fee_cap_cents {
                let excess_cents = input.late_fee_charged_cents.saturating_sub(late_fee_cap_cents);
                return Output {
                    mode: CookCountyRtloMode::ViolationLateFeeExceedsStatutoryCap,
                    statutory_basis: "Cook County RTLO § 42-XII — late fee cap $10 first $1000 + 5 % on excess".to_string(),
                    notes: format!(
                        "VIOLATION: late fee {} cents exceeds statutory cap {} cents (= $10 for first $1000 + 5 % of monthly rent above $1000); excess {} cents recoverable by tenant.",
                        input.late_fee_charged_cents, late_fee_cap_cents, excess_cents
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_late_fee_cap_cents: late_fee_cap_cents,
                    statutory_damages_owed_cents: excess_cents,
                };
            }
            Output {
                mode: CookCountyRtloMode::CompliantLateFeeAtOrBelowStatutoryCap,
                statutory_basis: "Cook County RTLO § 42-XII — late fee cap $10 first $1000 + 5 % on excess".to_string(),
                notes: format!(
                    "COMPLIANT: late fee {} cents ≤ statutory cap {} cents.",
                    input.late_fee_charged_cents, late_fee_cap_cents
                ),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                statutory_late_fee_cap_cents: late_fee_cap_cents,
                statutory_damages_owed_cents: 0,
            }
        }
        ComplianceAspect::TenDayMaterialNoncomplianceCureNotice => {
            if input.ten_day_cure_notice_provided {
                Output {
                    mode: CookCountyRtloMode::CompliantTenDayMaterialNoncomplianceCureNoticeProvided,
                    statutory_basis: "Cook County RTLO § 42-XII — 10-day material noncompliance cure notice".to_string(),
                    notes: "COMPLIANT: tenant received 10-day written notice to cure material noncompliance prior to termination.".to_string(),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_late_fee_cap_cents: late_fee_cap_cents,
                    statutory_damages_owed_cents: 0,
                }
            } else {
                Output {
                    mode: CookCountyRtloMode::ViolationTenDayMaterialNoncomplianceCureNoticeNotProvided,
                    statutory_basis: "Cook County RTLO § 42-XII — 10-day material noncompliance cure notice".to_string(),
                    notes: "VIOLATION: landlord attempted termination for material noncompliance without first providing 10-day written cure notice; termination invalid; tenant may assert as affirmative defense.".to_string(),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_late_fee_cap_cents: late_fee_cap_cents,
                    statutory_damages_owed_cents: 0,
                }
            }
        }
        ComplianceAspect::EntryNoticeAndTimeWindow => {
            if input.entry_notice_days_provided < COOK_COUNTY_RTLO_ENTRY_NOTICE_DAYS
                || !input.entry_within_8am_to_8pm_window
            {
                Output {
                    mode: CookCountyRtloMode::ViolationEntryNoticeLessThan2DaysOrEntryOutside8AmTo8PmWindow,
                    statutory_basis: "Cook County RTLO § 42-XII — 2-day entry notice + 8 a.m. to 8 p.m. entry window".to_string(),
                    notes: format!(
                        "VIOLATION: entry notice provided was {} days (statutory minimum 2 days) AND/OR entry outside 8 a.m. to 8 p.m. window (within_window = {}).",
                        input.entry_notice_days_provided, input.entry_within_8am_to_8pm_window
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_late_fee_cap_cents: late_fee_cap_cents,
                    statutory_damages_owed_cents: 0,
                }
            } else {
                Output {
                    mode: CookCountyRtloMode::CompliantTwoDayEntryNoticeWithin8AmTo8PmWindow,
                    statutory_basis: "Cook County RTLO § 42-XII — 2-day entry notice + 8 a.m. to 8 p.m. entry window".to_string(),
                    notes: format!(
                        "COMPLIANT: 2+ day entry notice provided ({} days) and entry within 8 a.m. to 8 p.m. window.",
                        input.entry_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_late_fee_cap_cents: late_fee_cap_cents,
                    statutory_damages_owed_cents: 0,
                }
            }
        }
        ComplianceAspect::SixtyDayLeaseRenewalNotice => {
            if input.lease_renewal_notice_days_provided < COOK_COUNTY_RTLO_LEASE_RENEWAL_NOTICE_DAYS {
                Output {
                    mode: CookCountyRtloMode::ViolationSixtyDayLeaseRenewalNoticeNotProvided,
                    statutory_basis: "Cook County RTLO § 42-XII — 60-day lease renewal notice for material lease term changes".to_string(),
                    notes: format!(
                        "VIOLATION: landlord provided only {} days lease renewal notice; statutory minimum 60 days for material lease term changes upon renewal.",
                        input.lease_renewal_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_late_fee_cap_cents: late_fee_cap_cents,
                    statutory_damages_owed_cents: 0,
                }
            } else {
                Output {
                    mode: CookCountyRtloMode::CompliantSixtyDayLeaseRenewalNoticeProvided,
                    statutory_basis: "Cook County RTLO § 42-XII — 60-day lease renewal notice for material lease term changes".to_string(),
                    notes: format!(
                        "COMPLIANT: {}-day lease renewal notice (≥ 60 days required).",
                        input.lease_renewal_notice_days_provided
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                    statutory_late_fee_cap_cents: late_fee_cap_cents,
                    statutory_damages_owed_cents: 0,
                }
            }
        }
        ComplianceAspect::RetaliationProhibition => match input.retaliation_status {
            RetaliationStatus::RetaliatoryActionTakenAfterTenantProtectedActivity => Output {
                mode: CookCountyRtloMode::ViolationRetaliatoryActionTakenAfterTenantProtectedActivity,
                statutory_basis: "Cook County RTLO § 42-XII — retaliation prohibition".to_string(),
                notes: "VIOLATION: landlord took retaliatory action against tenant after protected activity (exercising RTLO rights, complaining to government, joining tenant organization, testifying); rebuttable presumption of retaliation; tenant entitled to actual damages, attorney fees, and injunctive relief.".to_string(),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                statutory_late_fee_cap_cents: late_fee_cap_cents,
                statutory_damages_owed_cents: 0,
            },
            RetaliationStatus::NoRetaliatoryActionTaken => Output {
                mode: CookCountyRtloMode::CompliantNoRetaliatoryActionTaken,
                statutory_basis: "Cook County RTLO § 42-XII — retaliation prohibition".to_string(),
                notes: "COMPLIANT: no retaliatory action taken against tenant.".to_string(),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                statutory_late_fee_cap_cents: late_fee_cap_cents,
                statutory_damages_owed_cents: 0,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::WithinSuburbanCookCountyCovered,
            exemption_status: ExemptionStatus::NotExemptFullyCovered,
            compliance_aspect: ComplianceAspect::SecurityDepositCollectionAndReturn,
            monthly_rent_cents: 200_000,          // $2000
            security_deposit_held_cents: 300_000, // $3000 = 1.5x cap
            security_deposit_return_days_after_termination: 25,
            security_deposit_itemization_provided: true,
            late_fee_charged_cents: 1_000, // $10
            ten_day_cure_notice_provided: true,
            entry_notice_days_provided: 2,
            entry_within_8am_to_8pm_window: true,
            lease_renewal_notice_days_provided: 60,
            anti_lockout_status: AntiLockoutStatus::NoLockoutOrSelfHelpEvictionPerformed,
            retaliation_status: RetaliationStatus::NoRetaliatoryActionTaken,
        }
    }

    #[test]
    fn property_outside_cook_county_not_applicable() {
        let mut input = baseline_input();
        input.property_jurisdiction = PropertyJurisdiction::OutsideCookCounty;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::NotApplicablePropertyOutsideCookCounty
        );
    }

    #[test]
    fn property_inside_chicago_excluded_for_chicago_rlto() {
        let mut input = baseline_input();
        input.property_jurisdiction =
            PropertyJurisdiction::WithinChicagoCityLimitsExcludedCoveredByChicagoCRLTO;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::NotApplicablePropertyInsideChicagoCityLimitsCoveredByChicagoCRLTO
        );
        assert!(output.notes.contains("Chicago"));
    }

    #[test]
    fn owner_occupied_six_or_fewer_units_exempt() {
        let mut input = baseline_input();
        input.exemption_status = ExemptionStatus::ExemptOwnerOccupiedBuildingWith6OrFewerUnits;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::NotApplicableOwnerOccupiedBuildingWith6OrFewerUnitsExempt
        );
    }

    #[test]
    fn single_family_owner_lived_within_12_months_exempt() {
        let mut input = baseline_input();
        input.exemption_status =
            ExemptionStatus::ExemptSingleFamilyOrCondoOwnerLivedThereWithin12Months;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::NotApplicableSingleFamilyOrCondoOwnerLivedThereWithin12MonthsExempt
        );
    }

    #[test]
    fn hotel_under_32_days_exempt() {
        let mut input = baseline_input();
        input.exemption_status = ExemptionStatus::ExemptHotelOrMotelMonthlyRentalUnder32Days;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::NotApplicableHotelOrMotelMonthlyRentalUnder32DaysExempt
        );
    }

    #[test]
    fn anti_lockout_applies_regardless_of_exemption_violation() {
        let mut input = baseline_input();
        input.exemption_status = ExemptionStatus::ExemptOwnerOccupiedBuildingWith6OrFewerUnits;
        input.compliance_aspect = ComplianceAspect::AntiLockoutProhibition;
        input.anti_lockout_status = AntiLockoutStatus::LockoutOrSelfHelpEvictionPerformedViolation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::ViolationLockoutOrSelfHelpEvictionPerformed
        );
        assert!(output
            .notes
            .contains("regardless of any other ordinance exemption"));
    }

    #[test]
    fn anti_lockout_compliant_no_lockout_performed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AntiLockoutProhibition;
        let output = check(&input);
        assert_eq!(output.mode, CookCountyRtloMode::CompliantNoLockoutPerformed);
    }

    #[test]
    fn security_deposit_at_cap_compliant() {
        // 1.5 × $2000 = $3000 = baseline
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            CookCountyRtloMode::CompliantSecurityDepositAtOrBelow1_5MonthsRentReturnedWithin30DaysWithItemization
        );
        assert_eq!(output.statutory_security_deposit_cap_cents, 300_000);
    }

    #[test]
    fn security_deposit_exceeds_cap_violation() {
        let mut input = baseline_input();
        input.security_deposit_held_cents = 400_000; // $4000 > $3000 cap
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::ViolationSecurityDepositCapExceeded1_5MonthsRent
        );
        // Excess = $1000; 2× = $2000; or one month's rent = $2000; max = $2000
        assert_eq!(output.statutory_damages_owed_cents, 200_000);
    }

    #[test]
    fn security_deposit_not_returned_within_30_days_violation() {
        let mut input = baseline_input();
        input.security_deposit_return_days_after_termination = 45;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::ViolationSecurityDepositNotReturnedWithin30Days
        );
        // 2 × $3000 deposit = $6000
        assert_eq!(output.statutory_damages_owed_cents, 600_000);
    }

    #[test]
    fn security_deposit_no_itemization_violation() {
        let mut input = baseline_input();
        input.security_deposit_itemization_provided = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::ViolationSecurityDepositReturnedWithoutItemization
        );
        assert_eq!(output.statutory_damages_owed_cents, 600_000);
    }

    #[test]
    fn late_fee_under_cap_for_rent_under_1000_compliant() {
        let mut input = baseline_input();
        input.monthly_rent_cents = 80_000; // $800
        input.compliance_aspect = ComplianceAspect::LateFeeAssessment;
        input.late_fee_charged_cents = 1_000; // $10 — at flat-portion cap
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::CompliantLateFeeAtOrBelowStatutoryCap
        );
        // Flat-portion only: $10 = 1000 cents
        assert_eq!(output.statutory_late_fee_cap_cents, 1_000);
    }

    #[test]
    fn late_fee_cap_for_rent_above_1000_includes_5_percent_excess() {
        let mut input = baseline_input();
        input.monthly_rent_cents = 200_000; // $2000
        input.compliance_aspect = ComplianceAspect::LateFeeAssessment;
        input.late_fee_charged_cents = 6_000; // $60 — at cap
        let output = check(&input);
        // Cap = $10 + 5% × ($2000 − $1000) = $10 + $50 = $60 = 6000 cents
        assert_eq!(output.statutory_late_fee_cap_cents, 6_000);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::CompliantLateFeeAtOrBelowStatutoryCap
        );
    }

    #[test]
    fn late_fee_above_cap_violation() {
        let mut input = baseline_input();
        input.monthly_rent_cents = 200_000;
        input.compliance_aspect = ComplianceAspect::LateFeeAssessment;
        input.late_fee_charged_cents = 10_000; // $100 > $60 cap
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::ViolationLateFeeExceedsStatutoryCap
        );
        // Excess = $100 − $60 = $40 = 4000 cents
        assert_eq!(output.statutory_damages_owed_cents, 4_000);
    }

    #[test]
    fn ten_day_cure_notice_provided_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenDayMaterialNoncomplianceCureNotice;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::CompliantTenDayMaterialNoncomplianceCureNoticeProvided
        );
    }

    #[test]
    fn ten_day_cure_notice_not_provided_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenDayMaterialNoncomplianceCureNotice;
        input.ten_day_cure_notice_provided = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::ViolationTenDayMaterialNoncomplianceCureNoticeNotProvided
        );
    }

    #[test]
    fn two_day_entry_notice_within_window_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EntryNoticeAndTimeWindow;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::CompliantTwoDayEntryNoticeWithin8AmTo8PmWindow
        );
    }

    #[test]
    fn entry_notice_only_one_day_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EntryNoticeAndTimeWindow;
        input.entry_notice_days_provided = 1;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::ViolationEntryNoticeLessThan2DaysOrEntryOutside8AmTo8PmWindow
        );
    }

    #[test]
    fn entry_outside_8am_to_8pm_window_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EntryNoticeAndTimeWindow;
        input.entry_within_8am_to_8pm_window = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::ViolationEntryNoticeLessThan2DaysOrEntryOutside8AmTo8PmWindow
        );
    }

    #[test]
    fn sixty_day_lease_renewal_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SixtyDayLeaseRenewalNotice;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::CompliantSixtyDayLeaseRenewalNoticeProvided
        );
    }

    #[test]
    fn lease_renewal_notice_under_60_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SixtyDayLeaseRenewalNotice;
        input.lease_renewal_notice_days_provided = 30;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::ViolationSixtyDayLeaseRenewalNoticeNotProvided
        );
    }

    #[test]
    fn retaliation_taken_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RetaliationProhibition;
        input.retaliation_status =
            RetaliationStatus::RetaliatoryActionTakenAfterTenantProtectedActivity;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::ViolationRetaliatoryActionTakenAfterTenantProtectedActivity
        );
    }

    #[test]
    fn no_retaliation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RetaliationProhibition;
        let output = check(&input);
        assert_eq!(
            output.mode,
            CookCountyRtloMode::CompliantNoRetaliatoryActionTaken
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(COOK_COUNTY_RTLO_ENACTMENT_YEAR, 2021);
        assert_eq!(COOK_COUNTY_RTLO_ENACTMENT_MONTH, 1);
        assert_eq!(COOK_COUNTY_RTLO_ENACTMENT_DAY, 28);
        assert_eq!(COOK_COUNTY_RTLO_EFFECTIVE_DATE_YEAR, 2021);
        assert_eq!(COOK_COUNTY_RTLO_EFFECTIVE_DATE_MONTH, 6);
        assert_eq!(COOK_COUNTY_RTLO_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(COOK_COUNTY_RTLO_SECURITY_DEPOSIT_CAP_NUMERATOR, 3);
        assert_eq!(COOK_COUNTY_RTLO_SECURITY_DEPOSIT_CAP_DENOMINATOR, 2);
        assert_eq!(COOK_COUNTY_RTLO_SECURITY_DEPOSIT_RETURN_DAYS, 30);
        assert_eq!(COOK_COUNTY_RTLO_LATE_FEE_FLAT_PORTION_DOLLARS, 10);
        assert_eq!(
            COOK_COUNTY_RTLO_LATE_FEE_FLAT_PORTION_RENT_THRESHOLD_DOLLARS,
            1_000
        );
        assert_eq!(COOK_COUNTY_RTLO_LATE_FEE_EXCESS_BASIS_POINTS, 500);
        assert_eq!(COOK_COUNTY_RTLO_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(
            COOK_COUNTY_RTLO_OWNER_OCCUPIED_BUILDING_EXEMPTION_MAX_UNITS,
            6
        );
        assert_eq!(
            COOK_COUNTY_RTLO_SINGLE_FAMILY_OWNER_LIVED_THERE_MONTHS_THRESHOLD,
            12
        );
        assert_eq!(COOK_COUNTY_RTLO_HOTEL_MOTEL_COVERAGE_DAYS_THRESHOLD, 32);
        assert_eq!(COOK_COUNTY_RTLO_MATERIAL_NONCOMPLIANCE_CURE_NOTICE_DAYS, 10);
        assert_eq!(COOK_COUNTY_RTLO_LEASE_RENEWAL_NOTICE_DAYS, 60);
        assert_eq!(COOK_COUNTY_RTLO_ENTRY_NOTICE_DAYS, 2);
        assert_eq!(COOK_COUNTY_RTLO_ENTRY_TIME_WINDOW_START_HOUR_24H, 8);
        assert_eq!(COOK_COUNTY_RTLO_ENTRY_TIME_WINDOW_END_HOUR_24H, 20);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Cook County"));
        assert!(joined.contains("June 1, 2021"));
        assert!(joined.contains("January 28, 2021"));
        assert!(joined.contains("Chapter 42"));
        assert!(joined.contains("Chicago"));
    }

    #[test]
    fn security_deposit_saturating_overflow_defense() {
        let mut input = baseline_input();
        input.monthly_rent_cents = u64::MAX;
        input.security_deposit_held_cents = u64::MAX;
        let output = check(&input);
        // Saturating arithmetic prevents panic; no assertions on numeric output beyond non-panic safety
        let _ = output.mode;
    }
}
