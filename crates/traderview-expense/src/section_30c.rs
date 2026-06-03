//! IRC § 30C Alternative Fuel Vehicle Refueling Property
//! Credit Compliance Module — pure-compute check for
//! taxpayer eligibility for the alternative fuel vehicle
//! refueling property credit (EV chargers + hydrogen
//! refueling + natural gas + propane refueling), enacted
//! by the Energy Policy Act of 2005, substantially
//! expanded by the Inflation Reduction Act of 2022, and
//! TERMINATED by the One Big Beautiful Bill Act of 2025
//! for property placed in service after June 30, 2026.
//!
//! Originally enacted by **Section 1342 of the Energy
//! Policy Act of 2005 (Public Law 109-58)**, signed by
//! President George W. Bush on **August 8, 2005**.
//! Substantially expanded by **Section 13404 of the
//! Inflation Reduction Act of 2022 (Public Law 117-169)**,
//! signed by President Joe Biden on August 16, 2022,
//! effective for property placed in service after **December
//! 31, 2022**. IRA 2022 introduced bifurcated rate structure
//! (6 % base / 30 % with prevailing wage + apprenticeship
//! via 5× multiplier), eligible-census-tract requirement
//! (low-income community per New Markets Tax Credit OR
//! non-urban per Treasury guidance), and $100,000 per-item
//! cap for depreciable business property.
//!
//! **TERMINATED by the One Big Beautiful Bill Act of 2025
//! (Public Law 119-21)**, signed by President Donald Trump
//! on **July 4, 2025**. § 30C credit available ONLY for
//! property placed in service on or before **June 30,
//! 2026**; original IRA 2022 sunset of December 31, 2032
//! accelerated by more than 6 years.
//!
//! Web research (verified 2026-06-03):
//! - **EPAct 2005 Enactment**: IRC § 30C added by **Section 1342 of the Energy Policy Act of 2005 (Public Law 109-58, 119 Stat. 594)**, signed by President George W. Bush on **August 8, 2005** ([Federal Register — Section 30C Alternative Fuel Vehicle Refueling Property Credit Final Regulations (September 19, 2024)](https://www.federalregister.gov/documents/2024/09/19/2024-20748/section-30c-alternative-fuel-vehicle-refueling-property-credit); [IRS — Alternative Fuel Vehicle Refueling Property Credit](https://www.irs.gov/credits-deductions/alternative-fuel-vehicle-refueling-property-credit); [IRS — Alternative Fuel Vehicle Refueling Property Credit for Tax-exempt Entities](https://www.irs.gov/credits-deductions/alternative-fuel-vehicle-refueling-property-credit-for-tax-exempt-entities); [IRS — Alternative Fuel Vehicle Refueling Property Credit for Individuals](https://www.irs.gov/credits-deductions/alternative-fuel-vehicle-refueling-property-credit-for-individuals); [IRS — FAQs Regarding Eligible Census Tracts for Section 30C](https://www.irs.gov/credits-deductions/frequently-asked-questions-regarding-eligible-census-tracts-for-purposes-of-the-alternative-fuel-vehicle-refueling-property-credit-under-section-30c); [IRS — Notice 2024-20 (PDF)](https://www.irs.gov/pub/irs-drop/n-24-20.pdf); [Tax Notes — Code Sec. 30C](https://www.taxnotes.com/research/federal/usc26/30C); [Bloomberg Tax — Sec. 30C](https://irc.bloombergtax.com/public/uscode/doc/irc/section_30c); [Cornell LII — 26 U.S. Code § 30C](https://www.law.cornell.edu/uscode/text/26/30C); [ICS Tax — § 30C Alternative Fuel Vehicle Refueling Property Credit](https://ics-tax.com/30c-alternative-fuel-vehicle-refueling-property-credit/)).
//! - **IRA 2022 Expansion**: substantially expanded by **Section 13404 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818)**, signed by President Joe Biden on **August 16, 2022**, effective for property placed in service after **December 31, 2022**.
//! - **§ 30C(a) Credit Amount**: credit equal to **30 PERCENT of the cost** of qualified alternative fuel vehicle refueling property placed in service during the taxable year (post-IRA effective rate with prevailing wage + apprenticeship; base rate without PWA is 6 %).
//! - **§ 30C(e)(6) Per-Item Cap**: credit is **LIMITED TO $100,000** in the case of any such item of property of a character subject to an allowance for depreciation (business property); **$1,000** for non-depreciable property (residential).
//! - **§ 30C(g)(2)+(3) Prevailing Wage and Apprenticeship (PWA) Bonus**: if taxpayer satisfies PWA requirements OR meets the **BOC (Beginning of Construction) Exception**, the credit determined under § 30C(a) for qualified property that is depreciable is **MULTIPLIED BY 5** (base rate of 6 % becomes 30 %).
//! - **§ 30C(c) Eligible Census Tract Requirement**: qualified alternative fuel vehicle refueling property must be placed in service in one of two types of population census tracts — **(1) LOW-INCOME COMMUNITY CENSUS TRACTS** (as defined in the New Markets Tax Credit "low-income community" definition under § 45D(e)) OR **(2) NON-URBAN CENSUS TRACTS** (as defined by Treasury/IRS guidance under Notice 2024-20).
//! - **Qualifying Alternative Fuels**: electricity (EV charging stations), hydrogen (hydrogen refueling stations), natural gas (CNG/LNG), propane (LPG), and other Treasury-designated alternative fuels.
//! - **Treas. Reg. § 1.30C-1 Final Regulations**: published **September 19, 2024** (89 FR final action); provides operational rules for PWA + eligible census tracts + BOC exception.
//! - **OBBBA 2025 Termination**: § 30C ELIMINATED for property placed in service after **JUNE 30, 2026** by **Public Law 119-21 (One, Big, Beautiful Bill Act of 2025, 139 Stat. 72)**, signed by President Donald Trump on **JULY 4, 2025**; original IRA 2022 sunset of December 31, 2032 accelerated by more than **6 YEARS** ([IRS — FAQs for Modification of Sections 25C, 25D, 25E, 30C, 30D, 45L, 45W, AND 179D under Public Law 119-21](https://www.irs.gov/newsroom/faqs-for-modification-of-sections-25c-25d-25e-30c-30d-45l-45w-and-179d-under-public-law-119-21-139-stat-72-july-4-2025-commonly-known-as-the-one-big-beautiful-bill-obbb); [Arnold & Porter — From IRA to OBBBA](https://www.arnoldporter.com/en/perspectives/advisories/2025/07/from-ira-to-obbba-a-new-era-for-clean-energy-tax-credits); [Grant Thornton — Energy Incentives Under OBBBA](https://www.grantthornton.com/insights/alerts/tax/2025/insights/energy-incentives-under-obbba-what-you-need-to-know); [Novogradac — OBBBA and the Clean Energy Race Against the Clock](https://www.novoco.com/periodicals/articles/obbba-and-the-clean-energy-race-against-the-clock); [Current Federal Tax Developments — Key Modifications to Energy Credits and Deductions under OBBBA](https://www.currentfederaltaxdevelopments.com/blog/2025/8/21/key-modifications-to-energy-credits-and-deductions-under-the-one-big-beautiful-bill-act); [Wikipedia — One Big Beautiful Bill Act](https://en.wikipedia.org/wiki/One_Big_Beautiful_Bill_Act); [Buchanan Ingersoll & Rooney — High-Level Summary of Renewable Energy Tax Provisions](https://www.bipc.com/the-%E2%80%9Cone,-big,-beautiful-bill%E2%80%9D-becomes-law-high-level-summary-of-renewable-energy-tax-provisions); [RSM US — Tax Bill Significantly Changes Clean Energy Credits](https://rsmus.com/insights/services/business-tax/obbba-tax-clean-energy.html); [K&L Gates — Navigating the One Big Beautiful Bill Act: Critical Updates to Clean Energy Credits](https://www.klgates.com/Navigating-the-One-Big-Beautiful-Bill-Act-Critical-Updates-to-Clean-Energy-Credits-7-23-2025); [ICS Tax — OBBBA 2025](https://ics-tax.com/obbba-2025/)).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_30C_EPACT_2005_ENACTMENT_DATE_YEAR: u32 = 2005;
pub const IRC_30C_EPACT_2005_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_30C_EPACT_2005_ENACTMENT_DATE_DAY: u32 = 8;
pub const IRC_30C_EPACT_2005_PUBLIC_LAW_CONGRESS: u32 = 109;
pub const IRC_30C_EPACT_2005_PUBLIC_LAW_ENACTMENT: u32 = 58;
pub const IRC_30C_EPACT_2005_ENABLING_SECTION: u32 = 1342;
pub const IRC_30C_IRA_2022_ENACTMENT_DATE_YEAR: u32 = 2022;
pub const IRC_30C_IRA_2022_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_30C_IRA_2022_ENACTMENT_DATE_DAY: u32 = 16;
pub const IRC_30C_IRA_2022_PUBLIC_LAW_CONGRESS: u32 = 117;
pub const IRC_30C_IRA_2022_PUBLIC_LAW_ENACTMENT: u32 = 169;
pub const IRC_30C_IRA_2022_ENABLING_SECTION: u32 = 13404;
pub const IRC_30C_IRA_2022_EFFECTIVE_DATE_YEAR: u32 = 2022;
pub const IRC_30C_IRA_2022_EFFECTIVE_DATE_MONTH: u32 = 12;
pub const IRC_30C_IRA_2022_EFFECTIVE_DATE_DAY: u32 = 31;
pub const IRC_30C_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_30C_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_30C_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_30C_OBBBA_PUBLIC_LAW_CONGRESS: u32 = 119;
pub const IRC_30C_OBBBA_PUBLIC_LAW_ENACTMENT: u32 = 21;
pub const IRC_30C_OBBBA_TERMINATION_DATE_YEAR: u32 = 2026;
pub const IRC_30C_OBBBA_TERMINATION_DATE_MONTH: u32 = 6;
pub const IRC_30C_OBBBA_TERMINATION_DATE_DAY: u32 = 30;
pub const IRC_30C_FINAL_REGS_PUBLICATION_DATE_YEAR: u32 = 2024;
pub const IRC_30C_FINAL_REGS_PUBLICATION_DATE_MONTH: u32 = 9;
pub const IRC_30C_FINAL_REGS_PUBLICATION_DATE_DAY: u32 = 19;
pub const IRC_30C_BASE_RATE_BPS: u64 = 600;
pub const IRC_30C_PWA_BONUS_RATE_BPS: u64 = 3_000;
pub const IRC_30C_PWA_MULTIPLIER: u64 = 5;
pub const IRC_30C_BUSINESS_PROPERTY_CAP_DOLLARS: u64 = 100_000;
pub const IRC_30C_RESIDENTIAL_PROPERTY_CAP_DOLLARS: u64 = 1_000;
pub const IRC_30C_ORIGINAL_IRA_SUNSET_DATE_YEAR: u32 = 2032;
pub const IRC_30C_ORIGINAL_IRA_SUNSET_DATE_MONTH: u32 = 12;
pub const IRC_30C_ORIGINAL_IRA_SUNSET_DATE_DAY: u32 = 31;
pub const IRC_30C_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacedInServiceDateStatus {
    PlacedInServiceAfterJanuary1_2023AndOnOrBeforeJune30_2026PostIraPreObbbaTerminationEligible,
    PlacedInServiceBeforeJanuary1_2023PreIraExpansion,
    PlacedInServiceAfterJune30_2026PostObbbaTermination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    DepreciableBusinessProperty,
    NonDepreciableResidentialProperty,
    NotQualifiedAlternativeFuelVehicleRefuelingProperty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CensusTractEligibility {
    LowIncomeCommunityCensusTractUnderSection45DE,
    NonUrbanCensusTractUnderTreasuryGuidance,
    NotEligibleCensusTract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FuelType {
    Electricity,
    Hydrogen,
    NaturalGas,
    Propane,
    OtherTreasuryDesignatedAlternativeFuel,
    NotQualifyingFuel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    CreditAmountUnderSection30CA,
    PerItemCapUnderSection30CE6,
    PrevailingWageAndApprenticeshipBonusUnderSection30CG,
    EligibleCensusTractRequirementUnderSection30CC,
    QualifyingFuelTypeUnderSection30CD,
    BocExceptionUnderSection30CG3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section30CMode {
    NotApplicablePlacedInServiceBeforeJanuary1_2023PreIraExpansion,
    NotApplicablePlacedInServiceAfterJune30_2026PostObbbaTermination,
    NotApplicableNotQualifiedAlternativeFuelVehicleRefuelingProperty,
    NotApplicableNotQualifyingFuel,
    NotApplicableNotInEligibleCensusTract,
    CompliantThirtyPercentCreditWithPwaForBusinessProperty,
    CompliantSixPercentBaseCreditWithoutPwaForBusinessProperty,
    CompliantThirtyPercentCreditForResidentialProperty,
    CompliantBusinessPropertyCapAtOneHundredThousand,
    CompliantResidentialPropertyCapAtOneThousand,
    CompliantLowIncomeCommunityCensusTract,
    CompliantNonUrbanCensusTract,
    CompliantPwaRequirementsMet,
    CompliantBocExceptionMet,
    CompliantQualifyingFuel,
    ViolationBusinessPropertyClaimExceedsOneHundredThousandCap,
    ViolationResidentialPropertyClaimExceedsOneThousandCap,
    ViolationPwaRequirementsNotMet,
    ViolationFuelTypeNotQualifying,
    ViolationPropertyNotInEligibleCensusTract,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub placed_in_service_date_status: PlacedInServiceDateStatus,
    pub property_type: PropertyType,
    pub census_tract_eligibility: CensusTractEligibility,
    pub fuel_type: FuelType,
    pub compliance_aspect: ComplianceAspect,
    pub property_cost_dollars: u64,
    pub credit_claimed_dollars: u64,
    pub prevailing_wage_and_apprenticeship_requirements_met: bool,
    pub boc_exception_met: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section30CMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section30CInput = Input;
pub type Section30COutput = Output;
pub type Section30CResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 30C Alternative Fuel Vehicle Refueling Property Credit — added by Section 1342 of the Energy Policy Act of 2005 (Public Law 109-58, 119 Stat. 594); signed by President George W. Bush on August 8, 2005".to_string(),
        "Inflation Reduction Act of 2022 § 13404 (Public Law 117-169, 136 Stat. 1818); signed by President Joe Biden on August 16, 2022; substantially expanded § 30C effective for property placed in service after December 31, 2022".to_string(),
        "IRC § 30C(a) Credit Amount — credit equal to 30 PERCENT of the cost of qualified alternative fuel vehicle refueling property placed in service during the taxable year (post-IRA effective rate with prevailing wage + apprenticeship; base rate without PWA is 6 %)".to_string(),
        "IRC § 30C(e)(6) Per-Item Cap — credit is LIMITED TO $100,000 in the case of any such item of property of a character subject to an allowance for depreciation (BUSINESS PROPERTY); $1,000 for non-depreciable property (RESIDENTIAL)".to_string(),
        "IRC § 30C(g)(2) Prevailing Wage Requirement — taxpayer must satisfy prevailing wage requirements during construction and for any alteration or repair of qualified property".to_string(),
        "IRC § 30C(g)(3) Apprenticeship Requirement — taxpayer must satisfy apprenticeship requirements during construction of qualified property".to_string(),
        "IRC § 30C(g)(1) PWA Bonus 5× Multiplier — if taxpayer satisfies PWA requirements (§ 30C(g)(2)+(3)) OR meets the BOC (Beginning of Construction) Exception, credit determined under § 30C(a) for depreciable property is MULTIPLIED BY 5 (base rate of 6 % becomes 30 %)".to_string(),
        "IRC § 30C(c) Eligible Census Tract Requirement — qualified alternative fuel vehicle refueling property must be placed in service in one of two types of population census tracts: (1) LOW-INCOME COMMUNITY CENSUS TRACTS (as defined in New Markets Tax Credit 'low-income community' definition under § 45D(e)) OR (2) NON-URBAN CENSUS TRACTS (as defined by Treasury/IRS guidance under Notice 2024-20)".to_string(),
        "IRC § 30C(d) Qualifying Fuels — electricity (EV charging stations), hydrogen (hydrogen refueling stations), natural gas (CNG/LNG), propane (LPG), and other Treasury-designated alternative fuels".to_string(),
        "Treas. Reg. § 1.30C-1 Final Regulations — published September 19, 2024 (89 FR final action); provides operational rules for PWA + eligible census tracts + BOC exception".to_string(),
        "IRS Notice 2024-20 — guidance on eligible census tracts for purposes of the alternative fuel vehicle refueling property credit under § 30C".to_string(),
        "Original IRA 2022 Sunset Date — § 30C credit was originally scheduled to expire December 31, 2032 under IRA 2022".to_string(),
        "OBBBA 2025 Termination — § 30C ELIMINATED for property placed in service after JUNE 30, 2026 by Public Law 119-21 (One, Big, Beautiful Bill Act of 2025, 139 Stat. 72), signed by President Donald Trump on JULY 4, 2025; original IRA 2022 sunset of December 31, 2032 accelerated by more than 6 YEARS".to_string(),
        "IRS — FAQs for Modification of Sections 25C, 25D, 25E, 30C, 30D, 45L, 45W, AND 179D under Public Law 119-21 — official IRS termination guidance".to_string(),
        "Form 8911 (Alternative Fuel Vehicle Refueling Property Credit) — required to claim credit; current instructions IRS Form 8911 (2025)".to_string(),
        "Federal Register — Section 30C Alternative Fuel Vehicle Refueling Property Credit Final Regulations (September 19, 2024)".to_string(),
        "Arnold & Porter + Grant Thornton + Novogradac + Current Federal Tax Developments + K&L Gates + RSM US + Buchanan Ingersoll & Rooney + ICS Tax — practitioner overviews of § 30C OBBBA termination".to_string(),
    ];

    match input.placed_in_service_date_status {
        PlacedInServiceDateStatus::PlacedInServiceBeforeJanuary1_2023PreIraExpansion => {
            return Output {
                mode: Section30CMode::NotApplicablePlacedInServiceBeforeJanuary1_2023PreIraExpansion,
                statutory_basis: "IRA 2022 § 13404 effective date — substantially expanded § 30C applies only to property placed in service after December 31, 2022".to_string(),
                notes: "NOT APPLICABLE: property placed in service before January 1, 2023 (pre-IRA 2022 expansion); pre-IRA § 30C framework applies (30 % of cost up to $30,000 per location business / $1,000 residential without PWA / census tract requirements).".to_string(),
                citations,
                computed_credit_dollars: 0,
            };
        }
        PlacedInServiceDateStatus::PlacedInServiceAfterJune30_2026PostObbbaTermination => {
            return Output {
                mode: Section30CMode::NotApplicablePlacedInServiceAfterJune30_2026PostObbbaTermination,
                statutory_basis: "OBBBA 2025 § 30C termination — property placed in service after June 30, 2026 ineligible".to_string(),
                notes: "NOT APPLICABLE: property placed in service after June 30, 2026; § 30C credit TERMINATED by One Big Beautiful Bill Act of 2025 (Public Law 119-21, signed July 4, 2025); original IRA 2022 sunset of December 31, 2032 accelerated by more than 6 years.".to_string(),
                citations,
                computed_credit_dollars: 0,
            };
        }
        PlacedInServiceDateStatus::PlacedInServiceAfterJanuary1_2023AndOnOrBeforeJune30_2026PostIraPreObbbaTerminationEligible => {}
    }

    if input.property_type == PropertyType::NotQualifiedAlternativeFuelVehicleRefuelingProperty {
        return Output {
            mode: Section30CMode::NotApplicableNotQualifiedAlternativeFuelVehicleRefuelingProperty,
            statutory_basis: "IRC § 30C(d) — qualified alternative fuel vehicle refueling property eligibility definition".to_string(),
            notes: "NOT APPLICABLE: property does not qualify as alternative fuel vehicle refueling property under § 30C(d); credit unavailable.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.fuel_type == FuelType::NotQualifyingFuel {
        return Output {
            mode: Section30CMode::NotApplicableNotQualifyingFuel,
            statutory_basis: "IRC § 30C(d) — fuel type must be electricity / hydrogen / natural gas / propane / Treasury-designated alternative fuel".to_string(),
            notes: "NOT APPLICABLE: fuel type does not qualify under § 30C(d) (must be electricity / hydrogen / natural gas / propane / Treasury-designated alternative fuel).".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.census_tract_eligibility == CensusTractEligibility::NotEligibleCensusTract {
        return Output {
            mode: Section30CMode::NotApplicableNotInEligibleCensusTract,
            statutory_basis: "IRC § 30C(c) — eligible census tract requirement (low-income community OR non-urban)".to_string(),
            notes: "NOT APPLICABLE: property not placed in service in eligible census tract under § 30C(c) (must be low-income community census tract per § 45D(e) NMTC definition OR non-urban census tract per Treasury/IRS guidance).".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::CreditAmountUnderSection30CA => {
            let rate = if input.property_type == PropertyType::DepreciableBusinessProperty {
                if input.prevailing_wage_and_apprenticeship_requirements_met || input.boc_exception_met {
                    IRC_30C_BASE_RATE_BPS * IRC_30C_PWA_MULTIPLIER
                } else {
                    IRC_30C_BASE_RATE_BPS
                }
            } else {
                IRC_30C_PWA_BONUS_RATE_BPS
            };
            let pre_cap_credit = (u128::from(input.property_cost_dollars) * u128::from(rate) / 10_000) as u64;
            let cap = if input.property_type == PropertyType::DepreciableBusinessProperty {
                IRC_30C_BUSINESS_PROPERTY_CAP_DOLLARS
            } else {
                IRC_30C_RESIDENTIAL_PROPERTY_CAP_DOLLARS
            };
            let computed = pre_cap_credit.min(cap);
            let mode = match input.property_type {
                PropertyType::DepreciableBusinessProperty => {
                    if input.prevailing_wage_and_apprenticeship_requirements_met || input.boc_exception_met {
                        Section30CMode::CompliantThirtyPercentCreditWithPwaForBusinessProperty
                    } else {
                        Section30CMode::CompliantSixPercentBaseCreditWithoutPwaForBusinessProperty
                    }
                }
                PropertyType::NonDepreciableResidentialProperty => {
                    Section30CMode::CompliantThirtyPercentCreditForResidentialProperty
                }
                PropertyType::NotQualifiedAlternativeFuelVehicleRefuelingProperty => unreachable!(),
            };
            Output {
                mode,
                statutory_basis: "IRC § 30C(a) — credit computed at applicable rate (6 % base / 30 % with PWA or BOC exception for business; 30 % for residential)".to_string(),
                notes: format!(
                    "COMPLIANT: § 30C credit computed = ${computed} (capped at ${cap} per item)."
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::PerItemCapUnderSection30CE6 => {
            let cap = if input.property_type == PropertyType::DepreciableBusinessProperty {
                IRC_30C_BUSINESS_PROPERTY_CAP_DOLLARS
            } else {
                IRC_30C_RESIDENTIAL_PROPERTY_CAP_DOLLARS
            };
            if input.credit_claimed_dollars <= cap {
                Output {
                    mode: if input.property_type == PropertyType::DepreciableBusinessProperty {
                        Section30CMode::CompliantBusinessPropertyCapAtOneHundredThousand
                    } else {
                        Section30CMode::CompliantResidentialPropertyCapAtOneThousand
                    },
                    statutory_basis: "IRC § 30C(e)(6) — credit within applicable per-item cap".to_string(),
                    notes: format!(
                        "COMPLIANT: credit claimed at or below ${cap} per-item cap under § 30C(e)(6)."
                    ),
                    citations,
                    computed_credit_dollars: input.credit_claimed_dollars,
                }
            } else {
                Output {
                    mode: if input.property_type == PropertyType::DepreciableBusinessProperty {
                        Section30CMode::ViolationBusinessPropertyClaimExceedsOneHundredThousandCap
                    } else {
                        Section30CMode::ViolationResidentialPropertyClaimExceedsOneThousandCap
                    },
                    statutory_basis: "IRC § 30C(e)(6) — credit claimed exceeds applicable per-item cap".to_string(),
                    notes: format!(
                        "VIOLATION: credit claimed exceeds ${cap} per-item cap under § 30C(e)(6); claim must be reduced to cap amount."
                    ),
                    citations,
                    computed_credit_dollars: cap,
                }
            }
        }
        ComplianceAspect::PrevailingWageAndApprenticeshipBonusUnderSection30CG => {
            if input.prevailing_wage_and_apprenticeship_requirements_met {
                Output {
                    mode: Section30CMode::CompliantPwaRequirementsMet,
                    statutory_basis: "IRC § 30C(g)(2)+(3) — prevailing wage and apprenticeship requirements met".to_string(),
                    notes: "COMPLIANT: prevailing wage AND apprenticeship requirements met under § 30C(g)(2)+(3); 5× multiplier applies (base 6 % → bonus 30 %).".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section30CMode::ViolationPwaRequirementsNotMet,
                    statutory_basis: "IRC § 30C(g)(2)+(3) — prevailing wage and apprenticeship requirements not met".to_string(),
                    notes: "VIOLATION: prevailing wage and apprenticeship requirements not met under § 30C(g)(2)+(3); 5× multiplier unavailable; only 6 % base rate applies for business property.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::EligibleCensusTractRequirementUnderSection30CC => match input.census_tract_eligibility {
            CensusTractEligibility::LowIncomeCommunityCensusTractUnderSection45DE => Output {
                mode: Section30CMode::CompliantLowIncomeCommunityCensusTract,
                statutory_basis: "IRC § 30C(c) — low-income community census tract under § 45D(e)".to_string(),
                notes: "COMPLIANT: property placed in service in low-income community census tract as defined under New Markets Tax Credit § 45D(e).".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
            CensusTractEligibility::NonUrbanCensusTractUnderTreasuryGuidance => Output {
                mode: Section30CMode::CompliantNonUrbanCensusTract,
                statutory_basis: "IRC § 30C(c) — non-urban census tract under Treasury/IRS guidance Notice 2024-20".to_string(),
                notes: "COMPLIANT: property placed in service in non-urban census tract per Treasury/IRS guidance under Notice 2024-20.".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
            CensusTractEligibility::NotEligibleCensusTract => Output {
                mode: Section30CMode::ViolationPropertyNotInEligibleCensusTract,
                statutory_basis: "IRC § 30C(c) — property not in eligible census tract".to_string(),
                notes: "VIOLATION: property not placed in service in eligible census tract under § 30C(c); credit unavailable.".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
        },
        ComplianceAspect::QualifyingFuelTypeUnderSection30CD => Output {
            mode: Section30CMode::CompliantQualifyingFuel,
            statutory_basis: "IRC § 30C(d) — qualifying fuel type".to_string(),
            notes: "COMPLIANT: fuel type qualifies under § 30C(d) (electricity / hydrogen / natural gas / propane / Treasury-designated alternative fuel).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::BocExceptionUnderSection30CG3 => {
            if input.boc_exception_met {
                Output {
                    mode: Section30CMode::CompliantBocExceptionMet,
                    statutory_basis: "IRC § 30C(g)(1) — BOC (Beginning of Construction) exception met".to_string(),
                    notes: "COMPLIANT: BOC (Beginning of Construction) exception met under § 30C(g)(1); 5× multiplier applies even if PWA requirements not satisfied.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section30CMode::ViolationPwaRequirementsNotMet,
                    statutory_basis: "IRC § 30C(g)(1) — BOC exception not met".to_string(),
                    notes: "VIOLATION: BOC (Beginning of Construction) exception not met under § 30C(g)(1); 5× multiplier requires PWA requirements satisfaction.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
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
            placed_in_service_date_status: PlacedInServiceDateStatus::PlacedInServiceAfterJanuary1_2023AndOnOrBeforeJune30_2026PostIraPreObbbaTerminationEligible,
            property_type: PropertyType::DepreciableBusinessProperty,
            census_tract_eligibility: CensusTractEligibility::LowIncomeCommunityCensusTractUnderSection45DE,
            fuel_type: FuelType::Electricity,
            compliance_aspect: ComplianceAspect::CreditAmountUnderSection30CA,
            property_cost_dollars: 100_000,
            credit_claimed_dollars: 30_000,
            prevailing_wage_and_apprenticeship_requirements_met: true,
            boc_exception_met: false,
        }
    }

    #[test]
    fn pre_ira_placed_in_service_not_applicable() {
        let mut input = baseline_input();
        input.placed_in_service_date_status =
            PlacedInServiceDateStatus::PlacedInServiceBeforeJanuary1_2023PreIraExpansion;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section30CMode::NotApplicablePlacedInServiceBeforeJanuary1_2023PreIraExpansion
        );
    }

    #[test]
    fn post_obbba_termination_not_applicable() {
        let mut input = baseline_input();
        input.placed_in_service_date_status =
            PlacedInServiceDateStatus::PlacedInServiceAfterJune30_2026PostObbbaTermination;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section30CMode::NotApplicablePlacedInServiceAfterJune30_2026PostObbbaTermination
        );
    }

    #[test]
    fn not_qualified_property_not_applicable() {
        let mut input = baseline_input();
        input.property_type = PropertyType::NotQualifiedAlternativeFuelVehicleRefuelingProperty;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section30CMode::NotApplicableNotQualifiedAlternativeFuelVehicleRefuelingProperty
        );
    }

    #[test]
    fn not_qualifying_fuel_not_applicable() {
        let mut input = baseline_input();
        input.fuel_type = FuelType::NotQualifyingFuel;
        let output = check(&input);
        assert_eq!(output.mode, Section30CMode::NotApplicableNotQualifyingFuel);
    }

    #[test]
    fn not_eligible_census_tract_not_applicable() {
        let mut input = baseline_input();
        input.census_tract_eligibility = CensusTractEligibility::NotEligibleCensusTract;
        let output = check(&input);
        assert_eq!(output.mode, Section30CMode::NotApplicableNotInEligibleCensusTract);
    }

    #[test]
    fn thirty_percent_credit_with_pwa_for_business_property_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section30CMode::CompliantThirtyPercentCreditWithPwaForBusinessProperty
        );
        assert_eq!(output.computed_credit_dollars, 30_000);
    }

    #[test]
    fn six_percent_base_credit_without_pwa_for_business_property_compliant() {
        let mut input = baseline_input();
        input.prevailing_wage_and_apprenticeship_requirements_met = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section30CMode::CompliantSixPercentBaseCreditWithoutPwaForBusinessProperty
        );
        assert_eq!(output.computed_credit_dollars, 6_000);
    }

    #[test]
    fn thirty_percent_credit_for_residential_property_compliant() {
        let mut input = baseline_input();
        input.property_type = PropertyType::NonDepreciableResidentialProperty;
        input.property_cost_dollars = 3_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section30CMode::CompliantThirtyPercentCreditForResidentialProperty
        );
        assert_eq!(output.computed_credit_dollars, 900);
    }

    #[test]
    fn business_property_capped_at_one_hundred_thousand() {
        let mut input = baseline_input();
        input.property_cost_dollars = 1_000_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section30CMode::CompliantThirtyPercentCreditWithPwaForBusinessProperty
        );
        assert_eq!(output.computed_credit_dollars, 100_000);
    }

    #[test]
    fn residential_property_capped_at_one_thousand() {
        let mut input = baseline_input();
        input.property_type = PropertyType::NonDepreciableResidentialProperty;
        input.property_cost_dollars = 10_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section30CMode::CompliantThirtyPercentCreditForResidentialProperty
        );
        assert_eq!(output.computed_credit_dollars, 1_000);
    }

    #[test]
    fn business_property_cap_compliance_at_100000_boundary() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PerItemCapUnderSection30CE6;
        input.credit_claimed_dollars = 100_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section30CMode::CompliantBusinessPropertyCapAtOneHundredThousand
        );
    }

    #[test]
    fn business_property_cap_violation_at_100001() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PerItemCapUnderSection30CE6;
        input.credit_claimed_dollars = 100_001;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section30CMode::ViolationBusinessPropertyClaimExceedsOneHundredThousandCap
        );
    }

    #[test]
    fn residential_property_cap_compliance_at_1000_boundary() {
        let mut input = baseline_input();
        input.property_type = PropertyType::NonDepreciableResidentialProperty;
        input.compliance_aspect = ComplianceAspect::PerItemCapUnderSection30CE6;
        input.credit_claimed_dollars = 1_000;
        let output = check(&input);
        assert_eq!(output.mode, Section30CMode::CompliantResidentialPropertyCapAtOneThousand);
    }

    #[test]
    fn residential_property_cap_violation_at_1001() {
        let mut input = baseline_input();
        input.property_type = PropertyType::NonDepreciableResidentialProperty;
        input.compliance_aspect = ComplianceAspect::PerItemCapUnderSection30CE6;
        input.credit_claimed_dollars = 1_001;
        let output = check(&input);
        assert_eq!(output.mode, Section30CMode::ViolationResidentialPropertyClaimExceedsOneThousandCap);
    }

    #[test]
    fn pwa_requirements_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PrevailingWageAndApprenticeshipBonusUnderSection30CG;
        let output = check(&input);
        assert_eq!(output.mode, Section30CMode::CompliantPwaRequirementsMet);
    }

    #[test]
    fn pwa_requirements_not_met_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PrevailingWageAndApprenticeshipBonusUnderSection30CG;
        input.prevailing_wage_and_apprenticeship_requirements_met = false;
        let output = check(&input);
        assert_eq!(output.mode, Section30CMode::ViolationPwaRequirementsNotMet);
    }

    #[test]
    fn low_income_community_census_tract_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleCensusTractRequirementUnderSection30CC;
        let output = check(&input);
        assert_eq!(output.mode, Section30CMode::CompliantLowIncomeCommunityCensusTract);
    }

    #[test]
    fn non_urban_census_tract_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleCensusTractRequirementUnderSection30CC;
        input.census_tract_eligibility = CensusTractEligibility::NonUrbanCensusTractUnderTreasuryGuidance;
        let output = check(&input);
        assert_eq!(output.mode, Section30CMode::CompliantNonUrbanCensusTract);
    }

    #[test]
    fn qualifying_fuel_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::QualifyingFuelTypeUnderSection30CD;
        let output = check(&input);
        assert_eq!(output.mode, Section30CMode::CompliantQualifyingFuel);
    }

    #[test]
    fn boc_exception_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BocExceptionUnderSection30CG3;
        input.boc_exception_met = true;
        let output = check(&input);
        assert_eq!(output.mode, Section30CMode::CompliantBocExceptionMet);
    }

    #[test]
    fn boc_exception_with_pwa_bonus_for_business_property() {
        let mut input = baseline_input();
        input.prevailing_wage_and_apprenticeship_requirements_met = false;
        input.boc_exception_met = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section30CMode::CompliantThirtyPercentCreditWithPwaForBusinessProperty
        );
        assert_eq!(output.computed_credit_dollars, 30_000);
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_30C_EPACT_2005_ENACTMENT_DATE_YEAR, 2005);
        assert_eq!(IRC_30C_EPACT_2005_PUBLIC_LAW_CONGRESS, 109);
        assert_eq!(IRC_30C_EPACT_2005_PUBLIC_LAW_ENACTMENT, 58);
        assert_eq!(IRC_30C_EPACT_2005_ENABLING_SECTION, 1342);
        assert_eq!(IRC_30C_IRA_2022_ENACTMENT_DATE_YEAR, 2022);
        assert_eq!(IRC_30C_IRA_2022_PUBLIC_LAW_CONGRESS, 117);
        assert_eq!(IRC_30C_IRA_2022_PUBLIC_LAW_ENACTMENT, 169);
        assert_eq!(IRC_30C_IRA_2022_ENABLING_SECTION, 13404);
        assert_eq!(IRC_30C_OBBBA_ENACTMENT_DATE_YEAR, 2025);
        assert_eq!(IRC_30C_OBBBA_ENACTMENT_DATE_MONTH, 7);
        assert_eq!(IRC_30C_OBBBA_ENACTMENT_DATE_DAY, 4);
        assert_eq!(IRC_30C_OBBBA_PUBLIC_LAW_CONGRESS, 119);
        assert_eq!(IRC_30C_OBBBA_PUBLIC_LAW_ENACTMENT, 21);
        assert_eq!(IRC_30C_OBBBA_TERMINATION_DATE_YEAR, 2026);
        assert_eq!(IRC_30C_OBBBA_TERMINATION_DATE_MONTH, 6);
        assert_eq!(IRC_30C_OBBBA_TERMINATION_DATE_DAY, 30);
        assert_eq!(IRC_30C_BASE_RATE_BPS, 600);
        assert_eq!(IRC_30C_PWA_BONUS_RATE_BPS, 3_000);
        assert_eq!(IRC_30C_PWA_MULTIPLIER, 5);
        assert_eq!(IRC_30C_BUSINESS_PROPERTY_CAP_DOLLARS, 100_000);
        assert_eq!(IRC_30C_RESIDENTIAL_PROPERTY_CAP_DOLLARS, 1_000);
        assert_eq!(IRC_30C_ORIGINAL_IRA_SUNSET_DATE_YEAR, 2032);
        assert_eq!(IRC_30C_FINAL_REGS_PUBLICATION_DATE_YEAR, 2024);
        assert_eq!(IRC_30C_FINAL_REGS_PUBLICATION_DATE_MONTH, 9);
        assert_eq!(IRC_30C_FINAL_REGS_PUBLICATION_DATE_DAY, 19);
        assert_eq!(IRC_30C_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("IRC § 30C"));
        assert!(joined.contains("Section 1342 of the Energy Policy Act of 2005"));
        assert!(joined.contains("Public Law 109-58"));
        assert!(joined.contains("August 8, 2005"));
        assert!(joined.contains("Inflation Reduction Act of 2022 § 13404"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("§ 30C(a)"));
        assert!(joined.contains("§ 30C(c)"));
        assert!(joined.contains("§ 30C(d)"));
        assert!(joined.contains("§ 30C(e)(6)"));
        assert!(joined.contains("§ 30C(g)"));
        assert!(joined.contains("30 PERCENT"));
        assert!(joined.contains("$100,000"));
        assert!(joined.contains("$1,000"));
        assert!(joined.contains("LOW-INCOME COMMUNITY"));
        assert!(joined.contains("NON-URBAN"));
        assert!(joined.contains("§ 45D(e)"));
        assert!(joined.contains("MULTIPLIED BY 5"));
        assert!(joined.contains("Form 8911"));
        assert!(joined.contains("Notice 2024-20"));
        assert!(joined.contains("Treas. Reg. § 1.30C-1"));
        assert!(joined.contains("Public Law 119-21"));
        assert!(joined.contains("JUNE 30, 2026"));
        assert!(joined.contains("JULY 4, 2025"));
    }
}
