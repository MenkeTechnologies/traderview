//! Minneapolis Renter Protections Ordinance Compliance
//! Module — one of the strongest US municipal tenant
//! protection regimes for criminal-screening look-back
//! limits, eviction-record look-back limits, security
//! deposit cap, and source-of-income (public assistance)
//! anti-discrimination protections.
//!
//! Pure-compute check for landlord compliance with the
//! Minneapolis Renter Protections Ordinance, unanimously
//! passed by the Minneapolis City Council on **September
//! 13, 2019** and made effective in two phases: **June 1,
//! 2020** for large landlords (more than 15 rental homes)
//! and **December 1, 2020** for small landlords (15 or
//! fewer rental homes). The ordinance is administered by
//! the Minneapolis Department of Regulatory Services
//! Inspections Division and was passed in response to
//! local advocacy concerning criminal-background-screening
//! barriers and eviction-record screening that
//! disproportionately excluded BIPOC and low-income
//! applicants from the Minneapolis rental market.
//!
//! Web research (verified 2026-06-03):
//! - **Enactment and Effective Dates**: Minneapolis City Council unanimously passed the Renter Protections Ordinance on **September 13, 2019**; effective **JUNE 1, 2020** for large landlords (more than 15 rental homes) and **DECEMBER 1, 2020** for small landlords (15 or fewer rental homes) ([City of Minneapolis Renter Protections program page](https://www2.minneapolismn.gov/business-services/licenses-permits-inspections/rental-licenses/renter-protections/); [City of Minneapolis Renting Resource Center](https://www.minneapolismn.gov/resident-services/property-housing/renting/renters/rights/); [Minneapolis LIMS — Renter Protections Ordinance text](https://lims.minneapolismn.gov/Download/FileV2/20980/Renter-Protections-Ordinance.pdf); [Minneapolis LIMS — Phase II Renter Protections Report](https://lims.minneapolismn.gov/Download/RCAV2/34747/Phase-II-Renter-Protections-Report.pdf); [Minnesota Law Review — Housing Is Justice: The Minneapolis Renters Protection Ordinance Is a Step in the Right Direction for Criminal Justice Reform](https://minnesotalawreview.org/2020/01/23/housing-is-justice-the-minneapolis-renters-protection-ordinance-is-a-step-in-the-right-direction-for-criminal-justice-reform/); [NLIHC — From the Field: Minneapolis City Council Passes Historic Tenant Protections](https://nlihc.org/resource/field-minneapolis-city-council-passes-historic-tenant-protections); [HOME Line Minnesota Tenant Bill of Rights v. 8 November 2024](https://homelinemn.org/wp-content/uploads/2024/11/Minnesota-Tenant-Bill-of-Rights_2024_11-08.pdf)).
//! - **Security Deposit Cap**: security deposits capped at **ONE MONTH'S RENT**; landlords may not collect, hold, or demand a security deposit exceeding one month of contract rent.
//! - **Renter Screening — Criminal Background Look-Back Limits**: landlords may NOT reject applicants based on (1) **MISDEMEANORS older than 3 YEARS** from the date of conviction; (2) **FELONIES older than 7 YEARS** from the date of conviction; (3) certain **SERIOUS OFFENSES (FIRST-DEGREE ARSON, ASSAULT, MANSLAUGHTER, KIDNAPPING, CRIMINAL SEXUAL CONDUCT, MURDER, AGGRAVATED ROBBERY) older than 10 YEARS** from the date of sentencing. Convictions WITHIN these look-back windows may still be considered through individualized assessment.
//! - **Renter Screening — Eviction Record Look-Back Limits**: landlords may NOT reject applicants based on (1) **EVICTION JUDGMENTS older than 3 YEARS** from the date of judgment; (2) **EVICTION SETTLEMENTS older than 1 YEAR** from the date of settlement; (3) **DISMISSED EVICTIONS at any time** (dismissed evictions can NEVER be considered as a basis for rejection).
//! - **Two Screening Options Under § 244.2025**: the ordinance offers landlords two screening options: (1) **STANDARD CRITERIA + INDIVIDUALIZED ASSESSMENT**: the landlord may apply objective screening criteria but must also conduct an individualized assessment of any rejection based on criminal history or eviction record; OR (2) **INCLUSIONARY SCREENING**: the landlord adopts the City's pre-approved "inclusionary" screening criteria that automatically comply with the ordinance.
//! - **Source-of-Income (Public Assistance) Protections**: rental property owners must **ACCEPT renters who use public assistance to pay for housing**, including Section 8 Housing Choice Voucher, MNsure, RAP, HOPWA, Section 202 Supportive Housing for the Elderly, and other federal, state, and local rental assistance programs.
//! - **Energy Cost Disclosure Requirement**: rental property owners must **DISCLOSE PAST AVERAGE ENERGY COSTS** for the rental unit when applicants apply for housing; the disclosure must cover the prior 12 months of utility costs.
//! - **Tenant Notification Requirements**: landlords must inform renters about specific policies and procedures related to their tenancy, including renter rights, complaint procedures, and contact information for the Minneapolis Department of Regulatory Services Inspections Division.
//! - **Civil Enforcement**: violations of the Renter Protections Ordinance may result in administrative citations + civil penalties + denial / suspension / revocation of the landlord's rental license under Minneapolis Code Chapter 244 + private right of action with statutory damages + attorney fees under § 244.2055.
//! - **Federal Preemption**: certain HUD-funded properties subject to federal screening regulations (PHA-administered public housing, project-based Section 8) may have federal preemption analyses that override or modify the Minneapolis ordinance's screening look-back limits; landlords operating under federal preemption must document the basis for the preemption claim.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const MINNEAPOLIS_ORDINANCE_ENACTMENT_YEAR: u32 = 2019;
pub const MINNEAPOLIS_ORDINANCE_ENACTMENT_MONTH: u32 = 9;
pub const MINNEAPOLIS_ORDINANCE_ENACTMENT_DAY: u32 = 13;
pub const MINNEAPOLIS_LARGE_LANDLORD_EFFECTIVE_DATE_YEAR: u32 = 2020;
pub const MINNEAPOLIS_LARGE_LANDLORD_EFFECTIVE_DATE_MONTH: u32 = 6;
pub const MINNEAPOLIS_LARGE_LANDLORD_EFFECTIVE_DATE_DAY: u32 = 1;
pub const MINNEAPOLIS_SMALL_LANDLORD_EFFECTIVE_DATE_YEAR: u32 = 2020;
pub const MINNEAPOLIS_SMALL_LANDLORD_EFFECTIVE_DATE_MONTH: u32 = 12;
pub const MINNEAPOLIS_SMALL_LANDLORD_EFFECTIVE_DATE_DAY: u32 = 1;
pub const MINNEAPOLIS_SMALL_LANDLORD_THRESHOLD_UNITS: u32 = 15;
pub const MINNEAPOLIS_SECURITY_DEPOSIT_CAP_MONTHS_RENT: u32 = 1;
pub const MINNEAPOLIS_MISDEMEANOR_LOOK_BACK_YEARS: u32 = 3;
pub const MINNEAPOLIS_FELONY_LOOK_BACK_YEARS: u32 = 7;
pub const MINNEAPOLIS_SERIOUS_OFFENSE_LOOK_BACK_YEARS: u32 = 10;
pub const MINNEAPOLIS_EVICTION_JUDGMENT_LOOK_BACK_YEARS: u32 = 3;
pub const MINNEAPOLIS_EVICTION_SETTLEMENT_LOOK_BACK_YEARS: u32 = 1;
pub const MINNEAPOLIS_ENERGY_COST_DISCLOSURE_LOOKBACK_MONTHS: u32 = 12;
pub const MINNEAPOLIS_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    WithinMinneapolisCityLimits,
    OutsideMinneapolisCityLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordSize {
    LargeLandlordOverFifteenRentalHomesEffectiveJune1_2020,
    SmallLandlordAtOrUnderFifteenRentalHomesEffectiveDecember1_2020,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SecurityDepositOneMonthCap,
    CriminalScreeningLookBackLimits,
    EvictionRecordScreeningLookBackLimits,
    SourceOfIncomePublicAssistanceProtection,
    EnergyCostDisclosure,
    DismissedEvictionConsideration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CriminalOffenseClassification {
    MisdemeanorConsideredWithin3YearsCompliant,
    MisdemeanorConsideredBeyond3YearsViolation,
    FelonyConsideredWithin7YearsCompliant,
    FelonyConsideredBeyond7YearsViolation,
    SeriousOffenseConsideredWithin10YearsCompliant,
    SeriousOffenseConsideredBeyond10YearsViolation,
    NoCriminalRecordConsidered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionRecordClassification {
    JudgmentConsideredWithin3YearsCompliant,
    JudgmentConsideredBeyond3YearsViolation,
    SettlementConsideredWithin1YearCompliant,
    SettlementConsideredBeyond1YearViolation,
    DismissedEvictionConsideredViolation,
    NoEvictionRecordConsidered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicAssistanceStatus {
    PublicAssistanceAcceptedCompliant,
    PublicAssistanceRejectedViolation,
    ApplicantNotUsingPublicAssistance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MinneapolisRenterProtectionsMode {
    NotApplicablePropertyOutsideMinneapolis,
    NotApplicableLargeLandlordPreJune1_2020EffectiveDate,
    NotApplicableSmallLandlordPreDecember1_2020EffectiveDate,
    NotApplicableSubjectToFederalPreemption,
    CompliantSecurityDepositAtOrBelowOneMonthRent,
    CompliantCriminalScreeningWithinLookBackLimits,
    CompliantEvictionScreeningWithinLookBackLimits,
    CompliantPublicAssistanceAccepted,
    CompliantEnergyCostDisclosureProvided,
    CompliantDismissedEvictionNotConsidered,
    ViolationSecurityDepositExceedsOneMonthRent,
    ViolationCriminalScreeningBeyondLookBackPeriod,
    ViolationEvictionScreeningBeyondLookBackPeriod,
    ViolationPublicAssistanceRejected,
    ViolationDismissedEvictionConsidered,
    ViolationEnergyCostDisclosureNotProvided,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub landlord_size: LandlordSize,
    pub effective_date_satisfied: bool,
    pub subject_to_federal_preemption: bool,
    pub compliance_aspect: ComplianceAspect,
    pub monthly_rent_cents: u64,
    pub security_deposit_held_cents: u64,
    pub criminal_offense_classification: CriminalOffenseClassification,
    pub eviction_record_classification: EvictionRecordClassification,
    pub public_assistance_status: PublicAssistanceStatus,
    pub energy_cost_disclosure_provided: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: MinneapolisRenterProtectionsMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub statutory_security_deposit_cap_cents: u64,
}

pub type RentalMinneapolisRenterProtectionsOrdinance2020Input = Input;
pub type RentalMinneapolisRenterProtectionsOrdinance2020Output = Output;
pub type RentalMinneapolisRenterProtectionsOrdinance2020Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Minneapolis Renter Protections Ordinance — passed unanimously by Minneapolis City Council on September 13, 2019; effective June 1, 2020 for large landlords (more than 15 rental homes) and December 1, 2020 for small landlords (15 or fewer rental homes); administered by Minneapolis Department of Regulatory Services Inspections Division".to_string(),
        "Security Deposit Cap — capped at ONE MONTH'S RENT; landlords may not collect, hold, or demand security deposit exceeding one month of contract rent".to_string(),
        "Criminal Background Look-Back Limits — landlords may NOT reject applicants based on (1) MISDEMEANORS older than 3 YEARS from date of conviction; (2) FELONIES older than 7 YEARS from date of conviction; (3) certain SERIOUS OFFENSES (first-degree arson, assault, manslaughter, kidnapping, criminal sexual conduct, murder, aggravated robbery) older than 10 YEARS from date of sentencing; convictions WITHIN look-back windows may still be considered through individualized assessment".to_string(),
        "Eviction Record Look-Back Limits — landlords may NOT reject applicants based on (1) EVICTION JUDGMENTS older than 3 YEARS from date of judgment; (2) EVICTION SETTLEMENTS older than 1 YEAR from date of settlement; (3) DISMISSED EVICTIONS at any time (dismissed evictions can NEVER be considered as basis for rejection)".to_string(),
        "Two Screening Options under § 244.2025 — (1) STANDARD CRITERIA + INDIVIDUALIZED ASSESSMENT: landlord may apply objective screening criteria but must also conduct individualized assessment of any rejection based on criminal history or eviction record; (2) INCLUSIONARY SCREENING: landlord adopts City's pre-approved inclusionary screening criteria that automatically comply".to_string(),
        "Source-of-Income (Public Assistance) Protections — rental property owners must ACCEPT renters who use public assistance to pay for housing, including Section 8 Housing Choice Voucher, MNsure, RAP, HOPWA, Section 202 Supportive Housing for the Elderly, and other federal/state/local rental assistance programs".to_string(),
        "Energy Cost Disclosure Requirement — rental property owners must DISCLOSE PAST AVERAGE ENERGY COSTS for rental unit when applicants apply for housing; disclosure must cover prior 12 months of utility costs".to_string(),
        "Tenant Notification Requirements — landlords must inform renters about specific policies and procedures related to tenancy including renter rights, complaint procedures, and contact information for Minneapolis Department of Regulatory Services Inspections Division".to_string(),
        "Civil Enforcement — violations may result in administrative citations + civil penalties + denial / suspension / revocation of rental license under Minneapolis Code Chapter 244 + private right of action with statutory damages + attorney fees under § 244.2055".to_string(),
        "Federal Preemption — certain HUD-funded properties subject to federal screening regulations (PHA-administered public housing, project-based Section 8) may have federal preemption analyses that override or modify Minneapolis ordinance's screening look-back limits".to_string(),
        "City of Minneapolis Renter Protections program page — primary HUD-funded landlord guidance".to_string(),
        "Minneapolis LIMS — Renter Protections Ordinance text".to_string(),
        "Minneapolis LIMS — Phase II Renter Protections Report".to_string(),
        "Minnesota Law Review — Housing Is Justice: The Minneapolis Renters Protection Ordinance Is a Step in the Right Direction for Criminal Justice Reform".to_string(),
        "NLIHC — From the Field: Minneapolis City Council Passes Historic Tenant Protections".to_string(),
        "HOME Line — Minnesota Tenant Bill of Rights v. 8 November 2024".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::OutsideMinneapolisCityLimits {
        return Output {
            mode: MinneapolisRenterProtectionsMode::NotApplicablePropertyOutsideMinneapolis,
            statutory_basis: "Property outside Minneapolis city limits; Minneapolis Renter Protections Ordinance inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside Minneapolis city limits; Minneapolis Renter Protections Ordinance inapplicable.".to_string(),
            citations,
            statutory_security_deposit_cap_cents: 0,
        };
    }

    if !input.effective_date_satisfied {
        let mode = match input.landlord_size {
            LandlordSize::LargeLandlordOverFifteenRentalHomesEffectiveJune1_2020 => {
                MinneapolisRenterProtectionsMode::NotApplicableLargeLandlordPreJune1_2020EffectiveDate
            }
            LandlordSize::SmallLandlordAtOrUnderFifteenRentalHomesEffectiveDecember1_2020 => {
                MinneapolisRenterProtectionsMode::NotApplicableSmallLandlordPreDecember1_2020EffectiveDate
            }
        };
        return Output {
            mode,
            statutory_basis: "Minneapolis Renter Protections Ordinance phased effective date — June 1, 2020 (large landlords > 15 units) and December 1, 2020 (small landlords ≤ 15 units)".to_string(),
            notes: "NOT APPLICABLE: pre-effective-date; ordinance does not apply to the asserted action.".to_string(),
            citations,
            statutory_security_deposit_cap_cents: 0,
        };
    }

    if input.subject_to_federal_preemption {
        return Output {
            mode: MinneapolisRenterProtectionsMode::NotApplicableSubjectToFederalPreemption,
            statutory_basis: "Federal preemption — HUD-funded property subject to federal screening regulations may have federal preemption over Minneapolis ordinance".to_string(),
            notes: "NOT APPLICABLE: HUD-funded property subject to federal screening regulations under Section 8 HCV, Section 8 PBRA, public housing, or other HUD programs; federal preemption analyzed under HUD guidance; Minneapolis screening look-back limits may be overridden or modified by federal regulations.".to_string(),
            citations,
            statutory_security_deposit_cap_cents: 0,
        };
    }

    let security_deposit_cap_cents = input
        .monthly_rent_cents
        .saturating_mul(u64::from(MINNEAPOLIS_SECURITY_DEPOSIT_CAP_MONTHS_RENT));

    match input.compliance_aspect {
        ComplianceAspect::SecurityDepositOneMonthCap => {
            if input.security_deposit_held_cents > security_deposit_cap_cents {
                Output {
                    mode: MinneapolisRenterProtectionsMode::ViolationSecurityDepositExceedsOneMonthRent,
                    statutory_basis: "Minneapolis Renter Protections Ordinance — security deposit capped at one month's rent".to_string(),
                    notes: format!(
                        "VIOLATION: security deposit ${} cents exceeds statutory cap of one month's rent (${} cents); tenant may petition for return of excess + statutory damages + attorney fees.",
                        input.security_deposit_held_cents, security_deposit_cap_cents
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                }
            } else {
                Output {
                    mode: MinneapolisRenterProtectionsMode::CompliantSecurityDepositAtOrBelowOneMonthRent,
                    statutory_basis: "Minneapolis Renter Protections Ordinance — security deposit at or below one month's rent".to_string(),
                    notes: format!(
                        "COMPLIANT: security deposit ${} cents is at or below the one-month-rent cap (${} cents).",
                        input.security_deposit_held_cents, security_deposit_cap_cents
                    ),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                }
            }
        }
        ComplianceAspect::CriminalScreeningLookBackLimits => match input.criminal_offense_classification {
            CriminalOffenseClassification::MisdemeanorConsideredWithin3YearsCompliant
            | CriminalOffenseClassification::FelonyConsideredWithin7YearsCompliant
            | CriminalOffenseClassification::SeriousOffenseConsideredWithin10YearsCompliant
            | CriminalOffenseClassification::NoCriminalRecordConsidered => Output {
                mode: MinneapolisRenterProtectionsMode::CompliantCriminalScreeningWithinLookBackLimits,
                statutory_basis: "Minneapolis Renter Protections Ordinance — criminal screening within look-back limits (3-year misdemeanor / 7-year felony / 10-year serious offense)".to_string(),
                notes: format!(
                    "COMPLIANT: criminal-record consideration within statutory look-back windows ({:?}); individualized assessment may still be required for rejection based on within-window conviction.",
                    input.criminal_offense_classification
                ),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
            },
            CriminalOffenseClassification::MisdemeanorConsideredBeyond3YearsViolation
            | CriminalOffenseClassification::FelonyConsideredBeyond7YearsViolation
            | CriminalOffenseClassification::SeriousOffenseConsideredBeyond10YearsViolation => Output {
                mode: MinneapolisRenterProtectionsMode::ViolationCriminalScreeningBeyondLookBackPeriod,
                statutory_basis: "Minneapolis Renter Protections Ordinance — criminal screening look-back limits exceeded (3-year misdemeanor / 7-year felony / 10-year serious offense)".to_string(),
                notes: format!(
                    "VIOLATION: criminal-record consideration extended beyond statutory look-back window ({:?}); ordinance prohibits rejection based on misdemeanors > 3 years, felonies > 7 years, or specified serious offenses > 10 years.",
                    input.criminal_offense_classification
                ),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
            },
        },
        ComplianceAspect::EvictionRecordScreeningLookBackLimits => match input.eviction_record_classification {
            EvictionRecordClassification::JudgmentConsideredWithin3YearsCompliant
            | EvictionRecordClassification::SettlementConsideredWithin1YearCompliant
            | EvictionRecordClassification::NoEvictionRecordConsidered => Output {
                mode: MinneapolisRenterProtectionsMode::CompliantEvictionScreeningWithinLookBackLimits,
                statutory_basis: "Minneapolis Renter Protections Ordinance — eviction screening within look-back limits (3-year judgment / 1-year settlement)".to_string(),
                notes: format!(
                    "COMPLIANT: eviction-record consideration within statutory look-back windows ({:?}); individualized assessment may still be required.",
                    input.eviction_record_classification
                ),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
            },
            EvictionRecordClassification::JudgmentConsideredBeyond3YearsViolation
            | EvictionRecordClassification::SettlementConsideredBeyond1YearViolation => Output {
                mode: MinneapolisRenterProtectionsMode::ViolationEvictionScreeningBeyondLookBackPeriod,
                statutory_basis: "Minneapolis Renter Protections Ordinance — eviction screening look-back limits exceeded".to_string(),
                notes: format!(
                    "VIOLATION: eviction-record consideration extended beyond statutory look-back window ({:?}); ordinance prohibits rejection based on judgments > 3 years or settlements > 1 year.",
                    input.eviction_record_classification
                ),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
            },
            EvictionRecordClassification::DismissedEvictionConsideredViolation => Output {
                mode: MinneapolisRenterProtectionsMode::ViolationDismissedEvictionConsidered,
                statutory_basis: "Minneapolis Renter Protections Ordinance — dismissed evictions cannot be considered at any time".to_string(),
                notes: "VIOLATION: landlord considered a dismissed eviction in screening; ordinance prohibits use of dismissed evictions as a basis for rejection at ANY time (no look-back limit — the absolute prohibition applies forever).".to_string(),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
            },
        },
        ComplianceAspect::SourceOfIncomePublicAssistanceProtection => match input.public_assistance_status {
            PublicAssistanceStatus::PublicAssistanceAcceptedCompliant
            | PublicAssistanceStatus::ApplicantNotUsingPublicAssistance => Output {
                mode: MinneapolisRenterProtectionsMode::CompliantPublicAssistanceAccepted,
                statutory_basis: "Minneapolis Renter Protections Ordinance — rental property owners must accept renters using public assistance".to_string(),
                notes: "COMPLIANT: landlord accepted applicant using public assistance (Section 8 HCV, MNsure, RAP, HOPWA, etc.) OR applicant did not use public assistance.".to_string(),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
            },
            PublicAssistanceStatus::PublicAssistanceRejectedViolation => Output {
                mode: MinneapolisRenterProtectionsMode::ViolationPublicAssistanceRejected,
                statutory_basis: "Minneapolis Renter Protections Ordinance — source-of-income discrimination prohibited".to_string(),
                notes: "VIOLATION: landlord rejected applicant based on use of public assistance; ordinance requires acceptance of Section 8 HCV, MNsure, RAP, HOPWA, Section 202, and other federal/state/local rental assistance programs.".to_string(),
                citations,
                statutory_security_deposit_cap_cents: security_deposit_cap_cents,
            },
        },
        ComplianceAspect::EnergyCostDisclosure => {
            if input.energy_cost_disclosure_provided {
                Output {
                    mode: MinneapolisRenterProtectionsMode::CompliantEnergyCostDisclosureProvided,
                    statutory_basis: "Minneapolis Renter Protections Ordinance — energy cost disclosure required at application".to_string(),
                    notes: "COMPLIANT: landlord disclosed past average energy costs (prior 12 months) for the rental unit at time of application.".to_string(),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                }
            } else {
                Output {
                    mode: MinneapolisRenterProtectionsMode::ViolationEnergyCostDisclosureNotProvided,
                    statutory_basis: "Minneapolis Renter Protections Ordinance — energy cost disclosure required at application".to_string(),
                    notes: "VIOLATION: landlord failed to disclose past average energy costs (prior 12 months) for the rental unit at time of application.".to_string(),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                }
            }
        }
        ComplianceAspect::DismissedEvictionConsideration => {
            if input.eviction_record_classification
                == EvictionRecordClassification::DismissedEvictionConsideredViolation
            {
                Output {
                    mode: MinneapolisRenterProtectionsMode::ViolationDismissedEvictionConsidered,
                    statutory_basis: "Minneapolis Renter Protections Ordinance — dismissed evictions cannot be considered at any time".to_string(),
                    notes: "VIOLATION: landlord considered dismissed eviction in screening; ordinance prohibits at any time.".to_string(),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
                }
            } else {
                Output {
                    mode: MinneapolisRenterProtectionsMode::CompliantDismissedEvictionNotConsidered,
                    statutory_basis: "Minneapolis Renter Protections Ordinance — dismissed evictions properly excluded from screening".to_string(),
                    notes: "COMPLIANT: dismissed evictions properly excluded from screening; ordinance's absolute prohibition on consideration of dismissed evictions satisfied.".to_string(),
                    citations,
                    statutory_security_deposit_cap_cents: security_deposit_cap_cents,
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
            property_jurisdiction: PropertyJurisdiction::WithinMinneapolisCityLimits,
            landlord_size: LandlordSize::LargeLandlordOverFifteenRentalHomesEffectiveJune1_2020,
            effective_date_satisfied: true,
            subject_to_federal_preemption: false,
            compliance_aspect: ComplianceAspect::SecurityDepositOneMonthCap,
            monthly_rent_cents: 150_000, // $1500
            security_deposit_held_cents: 150_000,
            criminal_offense_classification:
                CriminalOffenseClassification::NoCriminalRecordConsidered,
            eviction_record_classification:
                EvictionRecordClassification::NoEvictionRecordConsidered,
            public_assistance_status: PublicAssistanceStatus::PublicAssistanceAcceptedCompliant,
            energy_cost_disclosure_provided: true,
        }
    }

    #[test]
    fn property_outside_minneapolis_not_applicable() {
        let mut input = baseline_input();
        input.property_jurisdiction = PropertyJurisdiction::OutsideMinneapolisCityLimits;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::NotApplicablePropertyOutsideMinneapolis
        );
    }

    #[test]
    fn large_landlord_pre_june_1_2020_not_applicable() {
        let mut input = baseline_input();
        input.effective_date_satisfied = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::NotApplicableLargeLandlordPreJune1_2020EffectiveDate
        );
    }

    #[test]
    fn small_landlord_pre_december_1_2020_not_applicable() {
        let mut input = baseline_input();
        input.landlord_size =
            LandlordSize::SmallLandlordAtOrUnderFifteenRentalHomesEffectiveDecember1_2020;
        input.effective_date_satisfied = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::NotApplicableSmallLandlordPreDecember1_2020EffectiveDate
        );
    }

    #[test]
    fn federal_preemption_not_applicable() {
        let mut input = baseline_input();
        input.subject_to_federal_preemption = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::NotApplicableSubjectToFederalPreemption
        );
    }

    #[test]
    fn security_deposit_at_one_month_cap_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::CompliantSecurityDepositAtOrBelowOneMonthRent
        );
        assert_eq!(output.statutory_security_deposit_cap_cents, 150_000);
    }

    #[test]
    fn security_deposit_exceeds_one_month_cap_violation() {
        let mut input = baseline_input();
        input.security_deposit_held_cents = 300_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::ViolationSecurityDepositExceedsOneMonthRent
        );
    }

    #[test]
    fn criminal_misdemeanor_within_3_years_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CriminalScreeningLookBackLimits;
        input.criminal_offense_classification =
            CriminalOffenseClassification::MisdemeanorConsideredWithin3YearsCompliant;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::CompliantCriminalScreeningWithinLookBackLimits
        );
    }

    #[test]
    fn criminal_misdemeanor_beyond_3_years_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CriminalScreeningLookBackLimits;
        input.criminal_offense_classification =
            CriminalOffenseClassification::MisdemeanorConsideredBeyond3YearsViolation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::ViolationCriminalScreeningBeyondLookBackPeriod
        );
    }

    #[test]
    fn criminal_felony_within_7_years_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CriminalScreeningLookBackLimits;
        input.criminal_offense_classification =
            CriminalOffenseClassification::FelonyConsideredWithin7YearsCompliant;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::CompliantCriminalScreeningWithinLookBackLimits
        );
    }

    #[test]
    fn criminal_felony_beyond_7_years_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CriminalScreeningLookBackLimits;
        input.criminal_offense_classification =
            CriminalOffenseClassification::FelonyConsideredBeyond7YearsViolation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::ViolationCriminalScreeningBeyondLookBackPeriod
        );
    }

    #[test]
    fn criminal_serious_offense_within_10_years_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CriminalScreeningLookBackLimits;
        input.criminal_offense_classification =
            CriminalOffenseClassification::SeriousOffenseConsideredWithin10YearsCompliant;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::CompliantCriminalScreeningWithinLookBackLimits
        );
    }

    #[test]
    fn criminal_serious_offense_beyond_10_years_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CriminalScreeningLookBackLimits;
        input.criminal_offense_classification =
            CriminalOffenseClassification::SeriousOffenseConsideredBeyond10YearsViolation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::ViolationCriminalScreeningBeyondLookBackPeriod
        );
    }

    #[test]
    fn eviction_judgment_within_3_years_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionRecordScreeningLookBackLimits;
        input.eviction_record_classification =
            EvictionRecordClassification::JudgmentConsideredWithin3YearsCompliant;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::CompliantEvictionScreeningWithinLookBackLimits
        );
    }

    #[test]
    fn eviction_judgment_beyond_3_years_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionRecordScreeningLookBackLimits;
        input.eviction_record_classification =
            EvictionRecordClassification::JudgmentConsideredBeyond3YearsViolation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::ViolationEvictionScreeningBeyondLookBackPeriod
        );
    }

    #[test]
    fn eviction_settlement_within_1_year_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionRecordScreeningLookBackLimits;
        input.eviction_record_classification =
            EvictionRecordClassification::SettlementConsideredWithin1YearCompliant;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::CompliantEvictionScreeningWithinLookBackLimits
        );
    }

    #[test]
    fn eviction_settlement_beyond_1_year_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionRecordScreeningLookBackLimits;
        input.eviction_record_classification =
            EvictionRecordClassification::SettlementConsideredBeyond1YearViolation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::ViolationEvictionScreeningBeyondLookBackPeriod
        );
    }

    #[test]
    fn dismissed_eviction_considered_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EvictionRecordScreeningLookBackLimits;
        input.eviction_record_classification =
            EvictionRecordClassification::DismissedEvictionConsideredViolation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::ViolationDismissedEvictionConsidered
        );
    }

    #[test]
    fn public_assistance_accepted_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SourceOfIncomePublicAssistanceProtection;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::CompliantPublicAssistanceAccepted
        );
    }

    #[test]
    fn public_assistance_rejected_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SourceOfIncomePublicAssistanceProtection;
        input.public_assistance_status = PublicAssistanceStatus::PublicAssistanceRejectedViolation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::ViolationPublicAssistanceRejected
        );
    }

    #[test]
    fn energy_cost_disclosure_provided_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EnergyCostDisclosure;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::CompliantEnergyCostDisclosureProvided
        );
    }

    #[test]
    fn energy_cost_disclosure_not_provided_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EnergyCostDisclosure;
        input.energy_cost_disclosure_provided = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            MinneapolisRenterProtectionsMode::ViolationEnergyCostDisclosureNotProvided
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(MINNEAPOLIS_ORDINANCE_ENACTMENT_YEAR, 2019);
        assert_eq!(MINNEAPOLIS_ORDINANCE_ENACTMENT_MONTH, 9);
        assert_eq!(MINNEAPOLIS_ORDINANCE_ENACTMENT_DAY, 13);
        assert_eq!(MINNEAPOLIS_LARGE_LANDLORD_EFFECTIVE_DATE_YEAR, 2020);
        assert_eq!(MINNEAPOLIS_LARGE_LANDLORD_EFFECTIVE_DATE_MONTH, 6);
        assert_eq!(MINNEAPOLIS_LARGE_LANDLORD_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(MINNEAPOLIS_SMALL_LANDLORD_EFFECTIVE_DATE_YEAR, 2020);
        assert_eq!(MINNEAPOLIS_SMALL_LANDLORD_EFFECTIVE_DATE_MONTH, 12);
        assert_eq!(MINNEAPOLIS_SMALL_LANDLORD_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(MINNEAPOLIS_SMALL_LANDLORD_THRESHOLD_UNITS, 15);
        assert_eq!(MINNEAPOLIS_SECURITY_DEPOSIT_CAP_MONTHS_RENT, 1);
        assert_eq!(MINNEAPOLIS_MISDEMEANOR_LOOK_BACK_YEARS, 3);
        assert_eq!(MINNEAPOLIS_FELONY_LOOK_BACK_YEARS, 7);
        assert_eq!(MINNEAPOLIS_SERIOUS_OFFENSE_LOOK_BACK_YEARS, 10);
        assert_eq!(MINNEAPOLIS_EVICTION_JUDGMENT_LOOK_BACK_YEARS, 3);
        assert_eq!(MINNEAPOLIS_EVICTION_SETTLEMENT_LOOK_BACK_YEARS, 1);
        assert_eq!(MINNEAPOLIS_ENERGY_COST_DISCLOSURE_LOOKBACK_MONTHS, 12);
        assert_eq!(MINNEAPOLIS_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Minneapolis Renter Protections Ordinance"));
        assert!(joined.contains("September 13, 2019"));
        assert!(joined.contains("June 1, 2020"));
        assert!(joined.contains("December 1, 2020"));
        assert!(joined.contains("15 rental homes"));
        assert!(joined.contains("ONE MONTH'S RENT"));
        assert!(joined.contains("3 YEARS"));
        assert!(joined.contains("7 YEARS"));
        assert!(joined.contains("10 YEARS"));
        assert!(joined.contains("1 YEAR"));
        assert!(joined.contains("DISMISSED EVICTIONS"));
        assert!(joined.contains("Section 8"));
        assert!(joined.contains("12 months"));
    }
}
