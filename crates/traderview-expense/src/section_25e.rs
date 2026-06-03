//! IRC § 25E Previously-Owned Clean Vehicle Credit
//! Compliance Module — pure-compute check for taxpayer
//! eligibility for the $4,000 used clean vehicle credit
//! created by the Inflation Reduction Act of 2022 and
//! TERMINATED by the One Big Beautiful Bill Act of 2025
//! for vehicles acquired on or after October 1, 2025.
//!
//! Originally enacted by **Section 13402 of the Inflation
//! Reduction Act of 2022 (Public Law 117-169)**, signed by
//! President Joe Biden on **August 16, 2022**, with
//! effective date for vehicles acquired after December 31,
//! 2022. Provides a credit equal to the **LESSER of $4,000
//! OR 30 % of the sale price** for previously-owned clean
//! vehicles meeting statutory eligibility requirements
//! (sale price ≤ $25,000; model year ≥ 2 years earlier
//! than purchase year; battery capacity ≥ 7 kWh; GVWR <
//! 14,000 lbs; purchased from licensed dealer; first
//! transfer to non-original-owner since August 16, 2022;
//! taxpayer MAGI below filing-status threshold; not
//! claimed within prior 3 years).
//!
//! **TERMINATED by the One Big Beautiful Bill Act of 2025
//! (Public Law 119-21)**, signed by President Donald Trump
//! on **July 4, 2025**. § 25E credit available **ONLY for
//! vehicles acquired BEFORE OCTOBER 1, 2025** (September
//! 30, 2025 last eligible acquisition date).
//!
//! Web research (verified 2026-06-03):
//! - **IRA 2022 Enactment**: IRC § 25E added by **Section 13402 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818)**; signed by President Joe Biden on **August 16, 2022**; effective for vehicles acquired after **December 31, 2022** ([IRS — Used Clean Vehicle Credit](https://www.irs.gov/credits-deductions/used-clean-vehicle-credit); [House Office of Law Revision Counsel — 26 USC § 25E](https://uscode.house.gov/view.xhtml?req=(title:26+section:25E+edition:prelim)); [Cornell LII — 26 CFR § 1.25E-1](https://www.law.cornell.edu/cfr/text/26/1.25E-1); [IRS — Topic B FAQs Income and Price Limitations for New Clean Vehicle Credit](https://www.irs.gov/newsroom/topic-b-frequently-asked-questions-about-income-and-price-limitations-for-the-new-clean-vehicle-credit); [IRS — Frequently Asked Questions about the New, Previously Owned and Qualified Commercial Clean Vehicles Credit](https://www.irs.gov/newsroom/frequently-asked-questions-about-the-new-previously-owned-and-qualified-commercial-clean-vehicles-credit); [Congress.gov CRS — Clean Vehicle Tax Credits IF12600](https://www.congress.gov/crs-product/IF12600); [Federal Register — Clean Vehicle Credits Under Sections 25E and 30D; Transfer of Credits; Critical Minerals and Battery Components; Foreign Entities of Concern (May 6, 2024)](https://www.federalregister.gov/documents/2024/05/06/2024-09094/clean-vehicle-credits-under-sections-25e-and-30d-transfer-of-credits-critical-minerals-and-battery); [Inflation Reduction Act Tracker — IRA Section 13402 Credit for Previously-Owned Clean Vehicles](https://iratracker.org/programs/ira-section-13402-credit-for-previously-owned-clean-vehicles/); [IRS — Clean Vehicle Credit Seller or Dealer Requirements](https://www.irs.gov/credits-deductions/clean-vehicle-credit-seller-or-dealer-requirements); [IRS — Instructions for Form 8936 (2025)](https://www.irs.gov/instructions/i8936); [IRS — Topic H FAQs Transfer of New Clean Vehicle Credit and Previously Owned Clean Vehicles Credit](https://www.irs.gov/newsroom/topic-h-frequently-asked-questions-about-transfer-of-new-clean-vehicle-credit-and-previously-owned-clean-vehicles-credit); [IRS — Topic D FAQs Eligibility Rules for Previously-Owned Clean Vehicles Credit](https://www.irs.gov/newsroom/topic-d-frequently-asked-questions-about-eligibility-rules-for-the-previously-owned-clean-vehicles-credit); [Congress.gov CRS — Clean Vehicle Tax Credit Transfers to Car Dealers IF12570](https://www.congress.gov/crs-product/IF12570); [Arnold & Porter — From IRA to OBBBA: A New Era for Clean Energy Tax Credits (July 2025)](https://www.arnoldporter.com/en/perspectives/advisories/2025/07/from-ira-to-obbba-a-new-era-for-clean-energy-tax-credits); [Plante Moran — The OBBB and the End of EV Tax Credits: Opportunities Still Exist On or Before September 30](https://www.plantemoran.com/explore-our-thinking/insight/2025/09/the-obbb-and-the-end-of-ev-tax-credits); [IRS — One, Big, Beautiful Bill Provisions](https://www.irs.gov/newsroom/one-big-beautiful-bill-provisions); [IRS — FAQs for Modification of Sections 25C, 25D, 25E, 30C, 30D, 45L, 45W, AND 179D under Public Law 119-21, 139 Stat. 72 (July 4, 2025)](https://www.irs.gov/newsroom/faqs-for-modification-of-sections-25c-25d-25e-30c-30d-45l-45w-and-179d-under-public-law-119-21-139-stat-72-july-4-2025-commonly-known-as-the-one-big-beautiful-bill-obbb)).
//! - **§ 25E(a) Credit Amount**: credit equal to **LESSER OF $4,000 OR 30 PERCENT of the sale price** with respect to a previously-owned clean vehicle.
//! - **§ 25E(c)(1) Vehicle Eligibility**: vehicle must (i) be purchased from a **LICENSED DEALER**; (ii) have a **GVWR < 14,000 LBS**; (iii) have a **BATTERY CAPACITY ≥ 7 KILOWATT-HOURS**; (iv) have a **MODEL YEAR AT LEAST 2 YEARS earlier** than the calendar year of purchase; (v) be the **FIRST TRANSFER of the vehicle since August 16, 2022** to a person other than the person with whom the original use of such vehicle commenced; (vi) have a **SALE PRICE NOT EXCEEDING $25,000**.
//! - **§ 25E(b) Income Limits**: taxpayer modified AGI must NOT EXCEED — **$150,000 for joint filers or surviving spouses; $112,500 for head of household; $75,000 for single or married filing separately** (use LESSER of current-year or prior-year MAGI).
//! - **§ 25E(c)(2)(C) Once Per 3-Year Limit**: taxpayer must NOT have claimed another previously-owned clean vehicle credit under § 25E in the **3 YEARS BEFORE** the purchase date.
//! - **Credit Transfer Election** under § 25E(f) (added by IRA 2022 § 13402 + § 30D(g) parallel for new vehicles): for vehicles acquired **AFTER DECEMBER 31, 2023**, taxpayer may transfer entire credit to eligible entity (registered dealer) in exchange for financial benefit equal to credit amount.
//! - **Form 8936 + Schedule A**: Form 8936 (Clean Vehicle Credits) and Schedule A (Specific Vehicle Information) required to claim credit; current instructions IRS Form 8936 (2025).
//! - **OBBBA 2025 Termination**: **Section 25E TERMINATED** by **Public Law 119-21 (One, Big, Beautiful Bill Act of 2025, 139 Stat. 72)**, signed by President Donald Trump on **JULY 4, 2025**. § 25E credit available **ONLY for vehicles acquired BEFORE OCTOBER 1, 2025** (September 30, 2025 last eligible acquisition date).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_25E_IRA_ENACTMENT_DATE_YEAR: u32 = 2022;
pub const IRC_25E_IRA_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_25E_IRA_ENACTMENT_DATE_DAY: u32 = 16;
pub const IRC_25E_IRA_PUBLIC_LAW_CONGRESS: u32 = 117;
pub const IRC_25E_IRA_PUBLIC_LAW_ENACTMENT: u32 = 169;
pub const IRC_25E_IRA_ENABLING_SECTION_NUMBER: u32 = 13402;
pub const IRC_25E_IRA_STAT_VOLUME: u32 = 136;
pub const IRC_25E_IRA_STAT_PAGE: u32 = 1818;
pub const IRC_25E_OBBBA_TERMINATION_DATE_YEAR: u32 = 2025;
pub const IRC_25E_OBBBA_TERMINATION_DATE_MONTH: u32 = 9;
pub const IRC_25E_OBBBA_TERMINATION_DATE_DAY: u32 = 30;
pub const IRC_25E_OBBBA_PUBLIC_LAW_CONGRESS: u32 = 119;
pub const IRC_25E_OBBBA_PUBLIC_LAW_ENACTMENT: u32 = 21;
pub const IRC_25E_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_25E_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_25E_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_25E_OBBBA_STAT_VOLUME: u32 = 139;
pub const IRC_25E_OBBBA_STAT_PAGE: u32 = 72;
pub const IRC_25E_MAXIMUM_CREDIT_DOLLARS: u64 = 4_000;
pub const IRC_25E_CREDIT_RATE_BPS: u64 = 3_000;
pub const IRC_25E_MAXIMUM_SALE_PRICE_DOLLARS: u64 = 25_000;
pub const IRC_25E_VEHICLE_MINIMUM_AGE_YEARS: u32 = 2;
pub const IRC_25E_MINIMUM_BATTERY_CAPACITY_KWH: u32 = 7;
pub const IRC_25E_MAXIMUM_GVWR_LBS: u32 = 14_000;
pub const IRC_25E_AGI_LIMIT_SINGLE_DOLLARS: u64 = 75_000;
pub const IRC_25E_AGI_LIMIT_HOH_DOLLARS: u64 = 112_500;
pub const IRC_25E_AGI_LIMIT_MFJ_DOLLARS: u64 = 150_000;
pub const IRC_25E_ONCE_PER_LOOKBACK_YEARS: u32 = 3;
pub const IRC_25E_FORM_NUMBER: u32 = 8936;
pub const IRC_25E_FIRST_TRANSFER_REFERENCE_DATE_YEAR: u32 = 2022;
pub const IRC_25E_FIRST_TRANSFER_REFERENCE_DATE_MONTH: u32 = 8;
pub const IRC_25E_FIRST_TRANSFER_REFERENCE_DATE_DAY: u32 = 16;
pub const IRC_25E_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    SingleOrMarriedFilingSeparately,
    HeadOfHousehold,
    MarriedFilingJointlyOrSurvivingSpouse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionDateStatus {
    AcquiredOnOrAfterJanuary1_2023AndBeforeOctober1_2025Eligible,
    AcquiredBeforeJanuary1_2023PreIra,
    AcquiredOnOrAfterOctober1_2025PostObbbaTermination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleEligibilityStatus {
    PreviouslyOwnedCleanVehicleMeetingAllStatutoryRequirements,
    NotPreviouslyOwnedCleanVehicleNotEligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    CreditAmountLesserOf4000Or30PctSalePriceUnderSection25EA,
    SalePriceAtOrBelow25000UnderSection25EC1,
    VehicleModelYearAtLeast2YearsOlderUnderSection25EC1,
    BatteryCapacityAtOrAbove7KwhUnderSection25EC1,
    GvwrUnder14000LbsUnderSection25EC1,
    PurchasedFromLicensedDealerUnderSection25EC1,
    FirstTransferSinceAugust16_2022UnderSection25EC1,
    AgiBelowFilingStatusThresholdUnderSection25EB,
    OncePerThreeYearsLimitUnderSection25EC2C,
    CreditTransferToDealerUnderSection25EF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section25EMode {
    NotApplicableVehicleAcquiredBeforeJanuary1_2023PreIra,
    NotApplicableVehicleAcquiredOnOrAfterOctober1_2025PostObbbaTermination,
    NotApplicableNotPreviouslyOwnedCleanVehicle,
    CompliantFullCreditAtLesserOf4000Or30PctSalePrice,
    CompliantSalePriceAtOrBelow25000,
    CompliantVehicleModelYearAtLeast2YearsOlder,
    CompliantBatteryCapacityAtOrAbove7Kwh,
    CompliantGvwrUnder14000Lbs,
    CompliantPurchasedFromLicensedDealer,
    CompliantFirstTransferSinceAugust16_2022,
    CompliantAgiBelowFilingStatusThreshold,
    CompliantOncePerThreeYearsLimitRespected,
    CompliantCreditTransferToDealerForVehicleAcquiredAfterDecember31_2023,
    ViolationSalePriceExceeds25000,
    ViolationVehicleModelYearLessThan2YearsOld,
    ViolationBatteryCapacityBelow7Kwh,
    ViolationGvwrAtOrAbove14000Lbs,
    ViolationNotPurchasedFromLicensedDealer,
    ViolationNotFirstTransferSinceAugust16_2022,
    ViolationAgiExceedsFilingStatusThreshold,
    ViolationClaimedAnotherSection25ECreditWithinThreeYears,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub acquisition_date_status: AcquisitionDateStatus,
    pub vehicle_eligibility_status: VehicleEligibilityStatus,
    pub compliance_aspect: ComplianceAspect,
    pub filing_status: FilingStatus,
    pub sale_price_dollars: u64,
    pub model_year_age_years: u32,
    pub battery_capacity_kwh: u32,
    pub gvwr_lbs: u32,
    pub purchased_from_licensed_dealer: bool,
    pub first_transfer_since_august_16_2022: bool,
    pub modified_agi_lesser_of_current_or_prior_year_dollars: u64,
    pub claimed_another_section_25e_credit_within_three_years: bool,
    pub vehicle_acquired_after_december_31_2023: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section25EMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub credit_amount_dollars: u64,
}

pub type Section25EInput = Input;
pub type Section25EOutput = Output;
pub type Section25EResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 25E added by Section 13402 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818); signed by President Joe Biden on August 16, 2022; effective for vehicles acquired after December 31, 2022".to_string(),
        "IRC § 25E(a) Credit Amount — credit equal to LESSER OF $4,000 OR 30 PERCENT of the sale price with respect to a previously-owned clean vehicle".to_string(),
        "IRC § 25E(b) Income Limits — taxpayer modified AGI must NOT EXCEED $150,000 for joint filers or surviving spouses; $112,500 for head of household; $75,000 for single or married filing separately (use LESSER of current-year or prior-year MAGI)".to_string(),
        "IRC § 25E(c)(1) Vehicle Eligibility — vehicle must (i) be purchased from a LICENSED DEALER; (ii) have a GVWR < 14,000 LBS; (iii) have a BATTERY CAPACITY ≥ 7 KILOWATT-HOURS; (iv) have a MODEL YEAR AT LEAST 2 YEARS earlier than the calendar year of purchase; (v) be the FIRST TRANSFER of the vehicle since August 16, 2022 to a person other than the person with whom the original use of such vehicle commenced; (vi) have a SALE PRICE NOT EXCEEDING $25,000".to_string(),
        "IRC § 25E(c)(2)(C) Once Per 3-Year Limit — taxpayer must NOT have claimed another previously-owned clean vehicle credit under § 25E in the 3 YEARS BEFORE the purchase date".to_string(),
        "IRC § 25E(f) Credit Transfer Election — for vehicles acquired AFTER DECEMBER 31, 2023, taxpayer may transfer entire credit to eligible entity (registered dealer) in exchange for financial benefit equal to credit amount".to_string(),
        "IRS Form 8936 (Clean Vehicle Credits) + Schedule A (Specific Vehicle Information) — required to claim credit; current instructions IRS Form 8936 (2025)".to_string(),
        "26 CFR § 1.25E-1 Implementing Regulations — final regulations for § 25E previously-owned clean vehicle credit including transfer election and reporting".to_string(),
        "Federal Register — Clean Vehicle Credits Under Sections 25E and 30D; Transfer of Credits; Critical Minerals and Battery Components; Foreign Entities of Concern (May 6, 2024)".to_string(),
        "OBBBA 2025 Termination — § 25E TERMINATED by Public Law 119-21 (One, Big, Beautiful Bill Act of 2025, 139 Stat. 72), signed by President Donald Trump on JULY 4, 2025; § 25E credit available ONLY for vehicles acquired BEFORE OCTOBER 1, 2025 (September 30, 2025 last eligible acquisition date)".to_string(),
        "IRS — FAQs for Modification of Sections 25C, 25D, 25E, 30C, 30D, 45L, 45W, AND 179D under Public Law 119-21, 139 Stat. 72 (July 4, 2025) — official IRS termination guidance".to_string(),
        "Arnold & Porter (July 2025) + Plante Moran (September 2025) + Congress.gov CRS — practitioner overviews of OBBBA termination of IRA clean energy credits".to_string(),
    ];

    match input.acquisition_date_status {
        AcquisitionDateStatus::AcquiredBeforeJanuary1_2023PreIra => {
            return Output {
                mode: Section25EMode::NotApplicableVehicleAcquiredBeforeJanuary1_2023PreIra,
                statutory_basis: "Inflation Reduction Act of 2022 § 13402(e) effective date — § 25E credit available only for vehicles acquired after December 31, 2022".to_string(),
                notes: "NOT APPLICABLE: vehicle acquired before January 1, 2023 (pre-IRA effective date); § 25E credit unavailable.".to_string(),
                citations,
                credit_amount_dollars: 0,
            };
        }
        AcquisitionDateStatus::AcquiredOnOrAfterOctober1_2025PostObbbaTermination => {
            return Output {
                mode: Section25EMode::NotApplicableVehicleAcquiredOnOrAfterOctober1_2025PostObbbaTermination,
                statutory_basis: "OBBBA 2025 § 25E termination — credit available only for vehicles acquired BEFORE October 1, 2025".to_string(),
                notes: "NOT APPLICABLE: vehicle acquired on or after October 1, 2025; § 25E credit TERMINATED by One Big Beautiful Bill Act of 2025 (Public Law 119-21, signed July 4, 2025); September 30, 2025 was last eligible acquisition date.".to_string(),
                citations,
                credit_amount_dollars: 0,
            };
        }
        AcquisitionDateStatus::AcquiredOnOrAfterJanuary1_2023AndBeforeOctober1_2025Eligible => {}
    }

    if input.vehicle_eligibility_status == VehicleEligibilityStatus::NotPreviouslyOwnedCleanVehicleNotEligible {
        return Output {
            mode: Section25EMode::NotApplicableNotPreviouslyOwnedCleanVehicle,
            statutory_basis: "IRC § 25E(c)(2) — vehicle must qualify as previously-owned clean vehicle under statutory definitions".to_string(),
            notes: "NOT APPLICABLE: vehicle does not qualify as a previously-owned clean vehicle under § 25E(c)(2) (must be BEV / FCV / PHEV with battery capacity ≥ 7 kWh and GVWR < 14,000 lbs).".to_string(),
            citations,
            credit_amount_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::CreditAmountLesserOf4000Or30PctSalePriceUnderSection25EA => {
            let thirty_pct = (u128::from(input.sale_price_dollars) * 3_000 / 10_000) as u64;
            let credit = thirty_pct.min(IRC_25E_MAXIMUM_CREDIT_DOLLARS);
            Output {
                mode: Section25EMode::CompliantFullCreditAtLesserOf4000Or30PctSalePrice,
                statutory_basis: "IRC § 25E(a) — credit equal to LESSER OF $4,000 OR 30 % of sale price".to_string(),
                notes: format!(
                    "COMPLIANT: § 25E credit computed as LESSER OF $4,000 OR 30 % of sale price = ${credit}."
                ),
                citations,
                credit_amount_dollars: credit,
            }
        }
        ComplianceAspect::SalePriceAtOrBelow25000UnderSection25EC1 => {
            if input.sale_price_dollars <= IRC_25E_MAXIMUM_SALE_PRICE_DOLLARS {
                Output {
                    mode: Section25EMode::CompliantSalePriceAtOrBelow25000,
                    statutory_basis: "IRC § 25E(c)(1) — sale price at or below $25,000 statutory cap".to_string(),
                    notes: "COMPLIANT: vehicle sale price at or below $25,000 statutory cap under § 25E(c)(1).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section25EMode::ViolationSalePriceExceeds25000,
                    statutory_basis: "IRC § 25E(c)(1) — sale price exceeds $25,000 statutory cap".to_string(),
                    notes: "VIOLATION: vehicle sale price exceeds $25,000 statutory cap under § 25E(c)(1); credit unavailable.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::VehicleModelYearAtLeast2YearsOlderUnderSection25EC1 => {
            if input.model_year_age_years >= IRC_25E_VEHICLE_MINIMUM_AGE_YEARS {
                Output {
                    mode: Section25EMode::CompliantVehicleModelYearAtLeast2YearsOlder,
                    statutory_basis: "IRC § 25E(c)(1) — vehicle model year at least 2 years older than purchase year".to_string(),
                    notes: "COMPLIANT: vehicle model year at least 2 years earlier than calendar year of purchase under § 25E(c)(1).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section25EMode::ViolationVehicleModelYearLessThan2YearsOld,
                    statutory_basis: "IRC § 25E(c)(1) — vehicle model year less than 2 years older than purchase year".to_string(),
                    notes: "VIOLATION: vehicle model year less than 2 years earlier than calendar year of purchase under § 25E(c)(1); credit unavailable.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::BatteryCapacityAtOrAbove7KwhUnderSection25EC1 => {
            if input.battery_capacity_kwh >= IRC_25E_MINIMUM_BATTERY_CAPACITY_KWH {
                Output {
                    mode: Section25EMode::CompliantBatteryCapacityAtOrAbove7Kwh,
                    statutory_basis: "IRC § 25E(c)(1) — battery capacity at or above 7 kWh statutory minimum".to_string(),
                    notes: "COMPLIANT: battery capacity at or above 7 kWh statutory minimum under § 25E(c)(1).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section25EMode::ViolationBatteryCapacityBelow7Kwh,
                    statutory_basis: "IRC § 25E(c)(1) — battery capacity below 7 kWh statutory minimum".to_string(),
                    notes: "VIOLATION: battery capacity below 7 kWh statutory minimum under § 25E(c)(1); credit unavailable.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::GvwrUnder14000LbsUnderSection25EC1 => {
            if input.gvwr_lbs < IRC_25E_MAXIMUM_GVWR_LBS {
                Output {
                    mode: Section25EMode::CompliantGvwrUnder14000Lbs,
                    statutory_basis: "IRC § 25E(c)(1) — GVWR under 14,000 lbs statutory maximum".to_string(),
                    notes: "COMPLIANT: vehicle GVWR under 14,000 lbs statutory maximum under § 25E(c)(1).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section25EMode::ViolationGvwrAtOrAbove14000Lbs,
                    statutory_basis: "IRC § 25E(c)(1) — GVWR at or above 14,000 lbs".to_string(),
                    notes: "VIOLATION: vehicle GVWR at or above 14,000 lbs under § 25E(c)(1); credit unavailable.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::PurchasedFromLicensedDealerUnderSection25EC1 => {
            if input.purchased_from_licensed_dealer {
                Output {
                    mode: Section25EMode::CompliantPurchasedFromLicensedDealer,
                    statutory_basis: "IRC § 25E(c)(1) — vehicle purchased from licensed dealer".to_string(),
                    notes: "COMPLIANT: vehicle purchased from licensed dealer under § 25E(c)(1).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section25EMode::ViolationNotPurchasedFromLicensedDealer,
                    statutory_basis: "IRC § 25E(c)(1) — vehicle not purchased from licensed dealer".to_string(),
                    notes: "VIOLATION: vehicle not purchased from licensed dealer under § 25E(c)(1); private-party sales not eligible; credit unavailable.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::FirstTransferSinceAugust16_2022UnderSection25EC1 => {
            if input.first_transfer_since_august_16_2022 {
                Output {
                    mode: Section25EMode::CompliantFirstTransferSinceAugust16_2022,
                    statutory_basis: "IRC § 25E(c)(1) — first transfer of vehicle since August 16, 2022 to non-original-owner".to_string(),
                    notes: "COMPLIANT: this is the first transfer of the vehicle since August 16, 2022 to a person other than the original owner under § 25E(c)(1).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section25EMode::ViolationNotFirstTransferSinceAugust16_2022,
                    statutory_basis: "IRC § 25E(c)(1) — not first transfer since August 16, 2022".to_string(),
                    notes: "VIOLATION: this is NOT the first transfer of the vehicle since August 16, 2022 to a non-original-owner under § 25E(c)(1); credit unavailable (anti-double-credit-claim guard).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::AgiBelowFilingStatusThresholdUnderSection25EB => {
            let threshold = match input.filing_status {
                FilingStatus::SingleOrMarriedFilingSeparately => IRC_25E_AGI_LIMIT_SINGLE_DOLLARS,
                FilingStatus::HeadOfHousehold => IRC_25E_AGI_LIMIT_HOH_DOLLARS,
                FilingStatus::MarriedFilingJointlyOrSurvivingSpouse => IRC_25E_AGI_LIMIT_MFJ_DOLLARS,
            };
            if input.modified_agi_lesser_of_current_or_prior_year_dollars <= threshold {
                Output {
                    mode: Section25EMode::CompliantAgiBelowFilingStatusThreshold,
                    statutory_basis: "IRC § 25E(b) — modified AGI at or below filing status threshold".to_string(),
                    notes: "COMPLIANT: taxpayer modified AGI (lesser of current-year or prior-year) at or below applicable filing status threshold under § 25E(b).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section25EMode::ViolationAgiExceedsFilingStatusThreshold,
                    statutory_basis: "IRC § 25E(b) — modified AGI exceeds filing status threshold".to_string(),
                    notes: "VIOLATION: taxpayer modified AGI (lesser of current-year or prior-year) exceeds applicable filing status threshold under § 25E(b); credit unavailable.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::OncePerThreeYearsLimitUnderSection25EC2C => {
            if input.claimed_another_section_25e_credit_within_three_years {
                Output {
                    mode: Section25EMode::ViolationClaimedAnotherSection25ECreditWithinThreeYears,
                    statutory_basis: "IRC § 25E(c)(2)(C) — taxpayer claimed another § 25E credit within prior 3 years".to_string(),
                    notes: "VIOLATION: taxpayer has claimed another § 25E previously-owned clean vehicle credit in the 3 years before purchase date; credit unavailable under § 25E(c)(2)(C) once-per-3-years limit.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section25EMode::CompliantOncePerThreeYearsLimitRespected,
                    statutory_basis: "IRC § 25E(c)(2)(C) — once-per-3-years limit respected".to_string(),
                    notes: "COMPLIANT: taxpayer has not claimed another § 25E credit within prior 3 years under § 25E(c)(2)(C).".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            }
        }
        ComplianceAspect::CreditTransferToDealerUnderSection25EF => {
            if input.vehicle_acquired_after_december_31_2023 {
                Output {
                    mode: Section25EMode::CompliantCreditTransferToDealerForVehicleAcquiredAfterDecember31_2023,
                    statutory_basis: "IRC § 25E(f) — credit transfer to dealer available for vehicles acquired after December 31, 2023".to_string(),
                    notes: "COMPLIANT: vehicle acquired after December 31, 2023; taxpayer may transfer entire § 25E credit to eligible entity (registered dealer) in exchange for financial benefit equal to credit amount.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
                }
            } else {
                Output {
                    mode: Section25EMode::CompliantOncePerThreeYearsLimitRespected,
                    statutory_basis: "IRC § 25E(f) — credit transfer election unavailable for vehicles acquired on or before December 31, 2023".to_string(),
                    notes: "NOT TRIGGERED: vehicle acquired on or before December 31, 2023; credit transfer election unavailable; taxpayer must claim credit on income tax return via Form 8936.".to_string(),
                    citations,
                    credit_amount_dollars: 0,
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
            acquisition_date_status:
                AcquisitionDateStatus::AcquiredOnOrAfterJanuary1_2023AndBeforeOctober1_2025Eligible,
            vehicle_eligibility_status:
                VehicleEligibilityStatus::PreviouslyOwnedCleanVehicleMeetingAllStatutoryRequirements,
            compliance_aspect: ComplianceAspect::CreditAmountLesserOf4000Or30PctSalePriceUnderSection25EA,
            filing_status: FilingStatus::SingleOrMarriedFilingSeparately,
            sale_price_dollars: 20_000,
            model_year_age_years: 3,
            battery_capacity_kwh: 10,
            gvwr_lbs: 5_000,
            purchased_from_licensed_dealer: true,
            first_transfer_since_august_16_2022: true,
            modified_agi_lesser_of_current_or_prior_year_dollars: 60_000,
            claimed_another_section_25e_credit_within_three_years: false,
            vehicle_acquired_after_december_31_2023: true,
        }
    }

    #[test]
    fn pre_ira_acquisition_not_applicable() {
        let mut input = baseline_input();
        input.acquisition_date_status = AcquisitionDateStatus::AcquiredBeforeJanuary1_2023PreIra;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section25EMode::NotApplicableVehicleAcquiredBeforeJanuary1_2023PreIra
        );
    }

    #[test]
    fn post_obbba_termination_acquisition_not_applicable() {
        let mut input = baseline_input();
        input.acquisition_date_status =
            AcquisitionDateStatus::AcquiredOnOrAfterOctober1_2025PostObbbaTermination;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section25EMode::NotApplicableVehicleAcquiredOnOrAfterOctober1_2025PostObbbaTermination
        );
    }

    #[test]
    fn not_previously_owned_clean_vehicle_not_applicable() {
        let mut input = baseline_input();
        input.vehicle_eligibility_status =
            VehicleEligibilityStatus::NotPreviouslyOwnedCleanVehicleNotEligible;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::NotApplicableNotPreviouslyOwnedCleanVehicle);
    }

    #[test]
    fn credit_at_4000_cap_when_30_pct_exceeds() {
        let mut input = baseline_input();
        input.sale_price_dollars = 20_000;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantFullCreditAtLesserOf4000Or30PctSalePrice);
        assert_eq!(output.credit_amount_dollars, 4_000);
    }

    #[test]
    fn credit_at_30_pct_when_sale_price_below_4000_div_30_pct() {
        let mut input = baseline_input();
        input.sale_price_dollars = 10_000;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantFullCreditAtLesserOf4000Or30PctSalePrice);
        assert_eq!(output.credit_amount_dollars, 3_000);
    }

    #[test]
    fn sale_price_at_25000_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SalePriceAtOrBelow25000UnderSection25EC1;
        input.sale_price_dollars = 25_000;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantSalePriceAtOrBelow25000);
    }

    #[test]
    fn sale_price_at_25001_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SalePriceAtOrBelow25000UnderSection25EC1;
        input.sale_price_dollars = 25_001;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::ViolationSalePriceExceeds25000);
    }

    #[test]
    fn model_year_at_2_years_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::VehicleModelYearAtLeast2YearsOlderUnderSection25EC1;
        input.model_year_age_years = 2;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantVehicleModelYearAtLeast2YearsOlder);
    }

    #[test]
    fn model_year_at_1_year_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::VehicleModelYearAtLeast2YearsOlderUnderSection25EC1;
        input.model_year_age_years = 1;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::ViolationVehicleModelYearLessThan2YearsOld);
    }

    #[test]
    fn battery_at_7_kwh_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BatteryCapacityAtOrAbove7KwhUnderSection25EC1;
        input.battery_capacity_kwh = 7;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantBatteryCapacityAtOrAbove7Kwh);
    }

    #[test]
    fn battery_at_6_kwh_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BatteryCapacityAtOrAbove7KwhUnderSection25EC1;
        input.battery_capacity_kwh = 6;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::ViolationBatteryCapacityBelow7Kwh);
    }

    #[test]
    fn gvwr_at_13999_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::GvwrUnder14000LbsUnderSection25EC1;
        input.gvwr_lbs = 13_999;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantGvwrUnder14000Lbs);
    }

    #[test]
    fn gvwr_at_14000_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::GvwrUnder14000LbsUnderSection25EC1;
        input.gvwr_lbs = 14_000;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::ViolationGvwrAtOrAbove14000Lbs);
    }

    #[test]
    fn purchased_from_licensed_dealer_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PurchasedFromLicensedDealerUnderSection25EC1;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantPurchasedFromLicensedDealer);
    }

    #[test]
    fn not_from_licensed_dealer_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PurchasedFromLicensedDealerUnderSection25EC1;
        input.purchased_from_licensed_dealer = false;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::ViolationNotPurchasedFromLicensedDealer);
    }

    #[test]
    fn first_transfer_since_august_16_2022_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FirstTransferSinceAugust16_2022UnderSection25EC1;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantFirstTransferSinceAugust16_2022);
    }

    #[test]
    fn not_first_transfer_since_august_16_2022_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FirstTransferSinceAugust16_2022UnderSection25EC1;
        input.first_transfer_since_august_16_2022 = false;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::ViolationNotFirstTransferSinceAugust16_2022);
    }

    #[test]
    fn agi_single_at_75000_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AgiBelowFilingStatusThresholdUnderSection25EB;
        input.modified_agi_lesser_of_current_or_prior_year_dollars = 75_000;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantAgiBelowFilingStatusThreshold);
    }

    #[test]
    fn agi_single_at_75001_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AgiBelowFilingStatusThresholdUnderSection25EB;
        input.modified_agi_lesser_of_current_or_prior_year_dollars = 75_001;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::ViolationAgiExceedsFilingStatusThreshold);
    }

    #[test]
    fn agi_mfj_at_150000_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AgiBelowFilingStatusThresholdUnderSection25EB;
        input.filing_status = FilingStatus::MarriedFilingJointlyOrSurvivingSpouse;
        input.modified_agi_lesser_of_current_or_prior_year_dollars = 150_000;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantAgiBelowFilingStatusThreshold);
    }

    #[test]
    fn agi_hoh_at_112500_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AgiBelowFilingStatusThresholdUnderSection25EB;
        input.filing_status = FilingStatus::HeadOfHousehold;
        input.modified_agi_lesser_of_current_or_prior_year_dollars = 112_500;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantAgiBelowFilingStatusThreshold);
    }

    #[test]
    fn once_per_three_years_limit_respected_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OncePerThreeYearsLimitUnderSection25EC2C;
        let output = check(&input);
        assert_eq!(output.mode, Section25EMode::CompliantOncePerThreeYearsLimitRespected);
    }

    #[test]
    fn claimed_another_section_25e_credit_within_three_years_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::OncePerThreeYearsLimitUnderSection25EC2C;
        input.claimed_another_section_25e_credit_within_three_years = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section25EMode::ViolationClaimedAnotherSection25ECreditWithinThreeYears
        );
    }

    #[test]
    fn credit_transfer_to_dealer_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditTransferToDealerUnderSection25EF;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section25EMode::CompliantCreditTransferToDealerForVehicleAcquiredAfterDecember31_2023
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_25E_IRA_ENACTMENT_DATE_YEAR, 2022);
        assert_eq!(IRC_25E_IRA_ENACTMENT_DATE_MONTH, 8);
        assert_eq!(IRC_25E_IRA_ENACTMENT_DATE_DAY, 16);
        assert_eq!(IRC_25E_IRA_PUBLIC_LAW_CONGRESS, 117);
        assert_eq!(IRC_25E_IRA_PUBLIC_LAW_ENACTMENT, 169);
        assert_eq!(IRC_25E_IRA_ENABLING_SECTION_NUMBER, 13402);
        assert_eq!(IRC_25E_IRA_STAT_VOLUME, 136);
        assert_eq!(IRC_25E_IRA_STAT_PAGE, 1818);
        assert_eq!(IRC_25E_OBBBA_TERMINATION_DATE_YEAR, 2025);
        assert_eq!(IRC_25E_OBBBA_TERMINATION_DATE_MONTH, 9);
        assert_eq!(IRC_25E_OBBBA_TERMINATION_DATE_DAY, 30);
        assert_eq!(IRC_25E_OBBBA_PUBLIC_LAW_CONGRESS, 119);
        assert_eq!(IRC_25E_OBBBA_PUBLIC_LAW_ENACTMENT, 21);
        assert_eq!(IRC_25E_OBBBA_ENACTMENT_DATE_YEAR, 2025);
        assert_eq!(IRC_25E_OBBBA_ENACTMENT_DATE_MONTH, 7);
        assert_eq!(IRC_25E_OBBBA_ENACTMENT_DATE_DAY, 4);
        assert_eq!(IRC_25E_OBBBA_STAT_VOLUME, 139);
        assert_eq!(IRC_25E_OBBBA_STAT_PAGE, 72);
        assert_eq!(IRC_25E_MAXIMUM_CREDIT_DOLLARS, 4_000);
        assert_eq!(IRC_25E_CREDIT_RATE_BPS, 3_000);
        assert_eq!(IRC_25E_MAXIMUM_SALE_PRICE_DOLLARS, 25_000);
        assert_eq!(IRC_25E_VEHICLE_MINIMUM_AGE_YEARS, 2);
        assert_eq!(IRC_25E_MINIMUM_BATTERY_CAPACITY_KWH, 7);
        assert_eq!(IRC_25E_MAXIMUM_GVWR_LBS, 14_000);
        assert_eq!(IRC_25E_AGI_LIMIT_SINGLE_DOLLARS, 75_000);
        assert_eq!(IRC_25E_AGI_LIMIT_HOH_DOLLARS, 112_500);
        assert_eq!(IRC_25E_AGI_LIMIT_MFJ_DOLLARS, 150_000);
        assert_eq!(IRC_25E_ONCE_PER_LOOKBACK_YEARS, 3);
        assert_eq!(IRC_25E_FORM_NUMBER, 8936);
        assert_eq!(IRC_25E_FIRST_TRANSFER_REFERENCE_DATE_YEAR, 2022);
        assert_eq!(IRC_25E_FIRST_TRANSFER_REFERENCE_DATE_MONTH, 8);
        assert_eq!(IRC_25E_FIRST_TRANSFER_REFERENCE_DATE_DAY, 16);
        assert_eq!(IRC_25E_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("IRC § 25E"));
        assert!(joined.contains("Section 13402 of the Inflation Reduction Act of 2022"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("136 Stat. 1818"));
        assert!(joined.contains("August 16, 2022"));
        assert!(joined.contains("§ 25E(a)"));
        assert!(joined.contains("§ 25E(b)"));
        assert!(joined.contains("§ 25E(c)(1)"));
        assert!(joined.contains("§ 25E(c)(2)(C)"));
        assert!(joined.contains("§ 25E(f)"));
        assert!(joined.contains("LESSER OF $4,000 OR 30 PERCENT"));
        assert!(joined.contains("$150,000"));
        assert!(joined.contains("$112,500"));
        assert!(joined.contains("$75,000"));
        assert!(joined.contains("$25,000"));
        assert!(joined.contains("7 KILOWATT-HOURS"));
        assert!(joined.contains("14,000 LBS"));
        assert!(joined.contains("2 YEARS"));
        assert!(joined.contains("3 YEARS BEFORE"));
        assert!(joined.contains("Form 8936"));
        assert!(joined.contains("26 CFR § 1.25E-1"));
        assert!(joined.contains("One, Big, Beautiful Bill Act of 2025"));
        assert!(joined.contains("Public Law 119-21"));
        assert!(joined.contains("139 Stat. 72"));
        assert!(joined.contains("JULY 4, 2025"));
        assert!(joined.contains("OCTOBER 1, 2025"));
        assert!(joined.contains("September 30, 2025"));
    }
}
