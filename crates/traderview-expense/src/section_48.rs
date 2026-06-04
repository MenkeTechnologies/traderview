//! IRC § 48 — Energy Credit (Energy Investment
//! Tax Credit / ITC) Compliance Module.
//!
//! Pure-compute check for the foundational ITC predecessor
//! to § 48E (technology-neutral clean electricity ITC).
//! § 48 applies to facilities with BOC before January 1,
//! 2025 (January 1, 2035 for geothermal); § 48E applies to
//! facilities placed in service after December 31, 2024.
//!
//! **Inflation Reduction Act of 2022 § 13102 enactment**:
//! § 48 was substantially expanded through **§ 13102 of
//! Public Law 117-169**, commonly known as the Inflation
//! Reduction Act of 2022 (IRA), **enacted August 16,
//! 2022**; pre-IRA § 48 dated back to the Energy Tax Act
//! of 1978 (Public Law 95-618, enacted November 9, 1978).
//!
//! **Distinctive § 48 features**: **11 ELIGIBLE PROPERTY
//! CATEGORIES** under § 48(a)(3): solar, geothermal, fuel
//! cells, microturbines, combined heat and power (CHP),
//! qualified small wind, ground/groundwater thermal,
//! waste energy recovery (WERP ≤ 50 MW), energy storage
//! technology, qualified biogas (≥ 52% methane),
//! microgrid controllers (4 kW to 20 MW); **DYNAMIC
//! GLASS** (electrochromic) and **LINEAR GENERATORS**
//! added by IRA 2022; **6% BASE rate** without PWA;
//! **30% PWA-bumped rate** (5x bump-up multiplier
//! identical to other IRA 2022 clean-energy credits);
//! **+10 PERCENTAGE POINTS** energy community adder;
//! **+10 PERCENTAGE POINTS** domestic content adder
//! (stackable for **MAXIMUM 50% credit**); **LOW-INCOME
//! COMMUNITIES BONUS** under § 48(e) (ITC-only, capped
//! allocation pool); **5-YEAR RECAPTURE PERIOD** (365-day
//! year increments); **BOC before January 1, 2025**
//! (January 1, 2035 for geothermal) cutoff transitioning
//! to § 48E technology-neutral ITC; **ELIGIBLE for § 6417
//! direct pay** AND **ELIGIBLE for § 6418 transferability**
//! (symmetric monetization); **Form 3468** required.
//!
//! Web research (verified 2026-06-04):
//! - **Inflation Reduction Act of 2022 § 13102 enactment**: § 13102 of Public Law 117-169 substantially expanded IRC § 48; pre-IRA § 48 dated back to Energy Tax Act of 1978 (Public Law 95-618, enacted November 9, 1978) ([Cornell LII — 26 U.S. Code § 48](https://www.law.cornell.edu/uscode/text/26/48); [Bloomberg Tax — Sec. 48 Energy Credit](https://irc.bloombergtax.com/public/uscode/doc/irc/section_48); [House.gov — 26 USC 48 Energy Credit](https://uscode.house.gov/view.xhtml?req=%28title%3A26+section%3A48+edition%3Aprelim%29); [Tax Notes — IRC Code Section 48 (Energy Tax Credit)](https://www.taxnotes.com/research/federal/usc26/48); [FindLaw — 26 U.S.C. § 48 Internal Revenue Code](https://codes.findlaw.com/us/title-26-internal-revenue-code/26-usc-sect-48.html); [IRA Tracker — IRA Section 13102 Renewable Energy Investment Tax Credit](https://iratracker.org/programs/ira-section-13102-renewable-energy-investment-tax-credit/); [Federal Register — Definition of Energy Property and Rules Applicable to the Energy Credit Final Rule December 12, 2024](https://www.federalregister.gov/documents/2024/12/12/2024-28190/definition-of-energy-property-and-rules-applicable-to-the-energy-credit); [Federal Register — Definition of Energy Property Proposed Regulations November 22, 2023](https://www.federalregister.gov/documents/2023/11/22/2023-25539/definition-of-energy-property-and-rules-applicable-to-the-energy-credit); [Mayer Brown — IRS Releases Final Energy Property Regulations Under Section 48 ITC](https://www.mayerbrown.com/en/insights/publications/2025/01/irs-releases-final-energy-property-regulations-under-section-48-investment-tax-credit); [Holland & Knight — Breaking Down the Section 48 Investment Tax Credit Proposed Regulations](https://www.hklaw.com/en/insights/publications/2023/11/breaking-down-the-section-48-investment-tax-credit); [Holland & Knight — Key Highlights of the Section 48 ITC Final Regulations](https://www.hklaw.com/en/insights/publications/2025/01/key-highlights-of-the-section-48-itc-final-regulations); [Novogradac — Section 48 ITC Proposed Regulations Detail Eligibility of Various Energy Properties](https://www.novoco.com/notes-from-novogradac/section-48-itc-proposed-regulations-detail-eligibility-of-various-energy-properties-to-reflect-new-changes-in-technology-inflation-reduction-act-changes); [PwC — Final Regulations Clarify Rules for Section 48 Tax Credit](https://www.pwc.com/us/en/services/tax/library/pwc-final-regulations-clarify-rules-for-section-48-tax-credit.html); [Reunion Infrastructure — Section 48 ITC Updates and Due Diligence Impact](https://www.reunioninfra.com/insights/section-48-itc-due-diligence-guide); [Westchester County Association — Investment Tax Credit ITC IRC Section 48](https://www.westchester.org/clean-energy/investment-tax-credit-itc-irc-section-48/); [Specialty Tax Group — Navigating IRC 48 ITC Real Estate Investment Strategy](https://www.specialtytaxgroup.com/navigating-irc-48-investment-tax-credit-real-estate-investment-strategy); [Build With Basis — 48E ITC Final Rules](https://www.buildwithbasis.com/insights/48e-itc-final-rules-what-the-clean-electricity-investment-tax-credit-means-for-project-developers); [IRS — Instructions for Form 3468 (2025)](https://www.irs.gov/instructions/i3468); [IRS — About Form 3468 Investment Credit](https://www.irs.gov/forms-pubs/about-form-3468); [IRS — 2025 Instructions for Form 3468 Investment Credit PDF](https://www.irs.gov/pub/irs-pdf/i3468.pdf)).
//! - **§ 48(a) Credit Rate Structure**: **6% BASE RATE** + **24% PWA BONUS** for projects meeting **PREVAILING WAGE AND APPRENTICESHIP REQUIREMENTS** = **30% TOTAL** (5x multiplier from base); functionally equivalent to the 5x bump-up structure used in § 45Y, § 45U, § 45V, § 45X, § 45Z, § 48E.
//! - **§ 48(a)(3) Eligible Energy Property — 11 Categories**: (1) **SOLAR ENERGY EQUIPMENT** (solar electric generation + solar process heat + heat/cool/hot water for structures); (2) **GEOTHERMAL ENERGY EQUIPMENT**; (3) **QUALIFIED FUEL CELLS**; (4) **MICROTURBINE PROPERTY**; (5) **COMBINED HEAT AND POWER (CHP) SYSTEM PROPERTY**; (6) **QUALIFIED SMALL WIND ENERGY PROPERTY**; (7) **EQUIPMENT USING GROUND OR GROUNDWATER AS THERMAL ENERGY SOURCE**; (8) **WASTE ENERGY RECOVERY PROPERTY (WERP)** (capacity ≤ 50 MW); (9) **ENERGY STORAGE TECHNOLOGY**; (10) **QUALIFIED BIOGAS PROPERTY** (≥ 52% methane by volume); (11) **MICROGRID CONTROLLERS** (4 kW to 20 MW electricity capacity).
//! - **§ 48 IRA 2022 Property Additions**: **DYNAMIC GLASS (ELECTROCHROMIC)** — property that uses electricity to change light transmittance properties to heat or cool a structure; **LINEAR GENERATORS** added to eligible property list.
//! - **§ 48(a)(14)(C) Energy Community Adder**: **+10 PERCENTAGE POINTS** ADDITIONAL credit for facilities located in an **ENERGY COMMUNITY** (brownfield site, coal community, or area with high fossil fuel employment historically); ADDITIVE adder.
//! - **§ 48(a)(12) Domestic Content Adder**: **+10 PERCENTAGE POINTS** ADDITIONAL credit for facilities meeting **DOMESTIC CONTENT** requirements (specified percentages of US-sourced steel, iron, and manufactured products); ADDITIVE adder.
//! - **§ 48 Maximum Stacked Credit**: with all bonuses (PWA + domestic content + energy community) stacked, the credit may reach **50 PERCENT of qualified investment** (30% PWA + 10% domestic content + 10% energy community).
//! - **§ 48(e) Low-Income Communities Bonus Credit Amount Program**: additional bonus allocation (capped national pool) for facilities located in **LOW-INCOME COMMUNITIES**; **ITC-ONLY** (not available for § 45 PTC); administered by IRS annual allocation procedures.
//! - **§ 48 5-Year Recapture Period**: failure to meet PWA requirements may subject taxpayers to **RECAPTURE OF THE INCREASED CREDIT AMOUNT**, with an annual recapture ramp-down of **20 PERCENT** of the recapture amount; the 5 years begin when the energy property is **PLACED IN SERVICE** and measured in **365-DAY YEAR INCREMENTS** (366 days in leap year).
//! - **§ 48 Construction and Placed-in-Service Deadlines**: projects that begin construction PRIOR TO **JANUARY 1, 2025** (or **JANUARY 1, 2035 for GEOTHERMAL PROJECTS**) and are placed in service AFTER 2021 are eligible for the full investment tax credit of 30%.
//! - **§ 48 Eligible for § 6417 Direct Pay**: one of the 12 APPLICABLE CREDIT CATEGORIES under § 6417(b) for direct pay election.
//! - **§ 48 Eligible for § 6418 Transferability**: one of the 11 ELIGIBLE CREDIT CATEGORIES under § 6418(f)(1) for transferability monetization; SYMMETRIC MONETIZATION FRAMEWORK (eligible for both § 6417 direct pay AND § 6418 transferability).
//! - **Form 3468 (Investment Credit)**: required to claim § 48 credit; separate Form 3468 needed for each facility or property; redesigned by IRS for IRA 2022 provisions.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_48_NUMBER: u32 = 48;
pub const SECTION_48_BASE_RATE_PERCENT: u32 = 6;
pub const SECTION_48_PWA_RATE_PERCENT: u32 = 30;
pub const SECTION_48_PWA_MULTIPLIER: u32 = 5;
pub const SECTION_48_ENERGY_COMMUNITY_ADDER_PERCENT: u32 = 10;
pub const SECTION_48_DOMESTIC_CONTENT_ADDER_PERCENT: u32 = 10;
pub const SECTION_48_MAXIMUM_STACKED_RATE_PERCENT: u32 = 50;
pub const SECTION_48_RECAPTURE_PERIOD_YEARS: u32 = 5;
pub const SECTION_48_RECAPTURE_RAMP_DOWN_PERCENT_PER_YEAR: u32 = 20;
pub const SECTION_48_BOC_CUTOFF_YEAR_GENERAL: u32 = 2025;
pub const SECTION_48_BOC_CUTOFF_YEAR_GEOTHERMAL: u32 = 2035;
pub const SECTION_48_PIS_START_YEAR: u32 = 2022;
pub const SECTION_48_WERP_CAPACITY_LIMIT_MW: u32 = 50;
pub const SECTION_48_BIOGAS_MIN_METHANE_PERCENT: u32 = 52;
pub const SECTION_48_MICROGRID_MIN_KW: u32 = 4;
pub const SECTION_48_MICROGRID_MAX_MW: u32 = 20;
pub const SECTION_48_ENERGY_PROPERTY_CATEGORIES_COUNT: u32 = 11;
pub const SECTION_48_IRA_2022_ENACTMENT_YEAR: u32 = 2022;
pub const SECTION_48_IRA_2022_PUBLIC_LAW_NUMBER: u32 = 117169;
pub const SECTION_48_ENERGY_TAX_ACT_1978_PUBLIC_LAW_NUMBER: u32 = 95618;
pub const SECTION_48_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EligibleEnergyPropertyCategory {
    SolarEnergyEquipment,
    GeothermalEnergyEquipment,
    QualifiedFuelCells,
    MicroturbineProperty,
    CombinedHeatAndPowerSystemProperty,
    QualifiedSmallWindEnergyProperty,
    GroundOrGroundwaterThermalEnergySourceEquipment,
    WasteEnergyRecoveryProperty,
    EnergyStorageTechnology,
    QualifiedBiogasProperty,
    MicrogridControllers,
    DynamicGlassElectrochromicAddedByIra2022,
    LinearGeneratorsAddedByIra2022,
    NotEligibleEnergyProperty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PwaStatus {
    SatisfiesPwaRequirements,
    DoesNotSatisfyPwaRequirements,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BocAndPisStatus {
    BocBeforeJanuary1_2025AndPisAfter2021,
    GeothermalBocBeforeJanuary1_2035AndPisAfter2021,
    BocOnOrAfterJanuary1_2025NonGeothermal,
    PisBeforeOrIn2021,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnergyCommunityStatus {
    LocatedInEnergyCommunity,
    NotLocatedInEnergyCommunity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DomesticContentStatus {
    SatisfiesDomesticContentRequirements,
    DoesNotSatisfyDomesticContentRequirements,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LowIncomeCommunityStatus {
    AllocatedLowIncomeCommunitiesBonusUnderSection48E,
    NotAllocatedLowIncomeCommunitiesBonus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecaptureStatus {
    No5YearRecaptureTriggered,
    RecaptureTriggeredByDispositionOrPwaFailureDuring5YearPeriod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    EligibleEnergyPropertyCategoryUnderSection48A3,
    BaseAndPwaCreditRateUnderSection48A,
    EnergyCommunity10PercentAdderUnderSection48A14C,
    DomesticContent10PercentAdderUnderSection48A12,
    MaximumStackedCredit50Percent,
    LowIncomeCommunitiesBonusUnderSection48E,
    BocAndPisDeadlinesGeneralAndGeothermal,
    FiveYearRecapturePeriod,
    EligibilityForSection6417DirectPay,
    EligibilityForSection6418Transferability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section48Mode {
    NotApplicableNotEligibleEnergyProperty,
    NotApplicableBocOnOrAfterJanuary1_2025NonGeothermal,
    NotApplicablePisBeforeOrIn2021,
    CompliantEligibleEnergyPropertyCategory,
    CompliantBaseCreditAtSixPercent,
    CompliantPwaBumpedCreditAt30Percent,
    CompliantWithEnergyCommunityAdder10Percent,
    CompliantWithDomesticContentAdder10Percent,
    CompliantMaximumStackedCreditAt50Percent,
    CompliantLowIncomeCommunitiesBonusAllocated,
    CompliantBocBeforeJanuary1_2025AndPisAfter2021,
    CompliantGeothermalBocBeforeJanuary1_2035AndPisAfter2021,
    CompliantFiveYearRecapturePeriodSatisfied,
    CompliantEligibleForSection6417DirectPay,
    CompliantEligibleForSection6418Transferability,
    ViolationPwaBumpClaimedWithoutMeetingRequirements,
    ViolationEnergyCommunityAdderClaimedWithoutQualifyingLocation,
    ViolationDomesticContentAdderClaimedWithoutMeetingRequirement,
    ViolationLowIncomeCommunitiesBonusNotAllocated,
    ViolationRecaptureTriggeredDuring5YearPeriod,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub eligible_energy_property_category: EligibleEnergyPropertyCategory,
    pub pwa_status: PwaStatus,
    pub boc_and_pis_status: BocAndPisStatus,
    pub energy_community_status: EnergyCommunityStatus,
    pub domestic_content_status: DomesticContentStatus,
    pub low_income_community_status: LowIncomeCommunityStatus,
    pub recapture_status: RecaptureStatus,
    pub compliance_aspect: ComplianceAspect,
    pub qualified_investment_dollars: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section48Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub credit_amount_dollars: u64,
    pub citations: Vec<String>,
}

pub type Section48Input = Input;
pub type Section48Output = Output;
pub type Section48Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Inflation Reduction Act of 2022 § 13102 enactment — § 13102 of Public Law 117-169 substantially expanded IRC § 48; pre-IRA § 48 dated back to Energy Tax Act of 1978 (Public Law 95-618, enacted November 9, 1978)".to_string(),
        "IRC § 48(a) Credit Rate Structure — 6% BASE RATE + 24% PWA BONUS for projects meeting PREVAILING WAGE AND APPRENTICESHIP REQUIREMENTS = 30% TOTAL (5x multiplier from base)".to_string(),
        "IRC § 48(a)(3) Eligible Energy Property — 11 Categories — (1) SOLAR ENERGY EQUIPMENT; (2) GEOTHERMAL ENERGY EQUIPMENT; (3) QUALIFIED FUEL CELLS; (4) MICROTURBINE PROPERTY; (5) COMBINED HEAT AND POWER (CHP) SYSTEM PROPERTY; (6) QUALIFIED SMALL WIND ENERGY PROPERTY; (7) EQUIPMENT USING GROUND OR GROUNDWATER AS THERMAL ENERGY SOURCE; (8) WASTE ENERGY RECOVERY PROPERTY (WERP) (capacity ≤ 50 MW); (9) ENERGY STORAGE TECHNOLOGY; (10) QUALIFIED BIOGAS PROPERTY (≥ 52% methane by volume); (11) MICROGRID CONTROLLERS (4 kW to 20 MW electricity capacity)".to_string(),
        "IRC § 48 IRA 2022 Property Additions — DYNAMIC GLASS (ELECTROCHROMIC) — property that uses electricity to change light transmittance properties to heat or cool a structure; LINEAR GENERATORS added to eligible property list".to_string(),
        "IRC § 48(a)(14)(C) Energy Community Adder — +10 PERCENTAGE POINTS ADDITIONAL credit for facilities located in an ENERGY COMMUNITY (brownfield site, coal community, or area with high fossil fuel employment historically); ADDITIVE adder".to_string(),
        "IRC § 48(a)(12) Domestic Content Adder — +10 PERCENTAGE POINTS ADDITIONAL credit for facilities meeting DOMESTIC CONTENT requirements (specified percentages of US-sourced steel, iron, and manufactured products); ADDITIVE adder".to_string(),
        "IRC § 48 Maximum Stacked Credit — with all bonuses (PWA + domestic content + energy community) stacked, the credit may reach 50 PERCENT of qualified investment (30% PWA + 10% domestic content + 10% energy community)".to_string(),
        "IRC § 48(e) Low-Income Communities Bonus Credit Amount Program — additional bonus allocation (capped national pool) for facilities located in LOW-INCOME COMMUNITIES; ITC-ONLY (not available for § 45 PTC); administered by IRS annual allocation procedures".to_string(),
        "IRC § 48 5-Year Recapture Period — failure to meet PWA requirements may subject taxpayers to RECAPTURE OF THE INCREASED CREDIT AMOUNT, with an annual recapture ramp-down of 20 PERCENT of the recapture amount; the 5 years begin when the energy property is PLACED IN SERVICE and measured in 365-DAY YEAR INCREMENTS (366 days in leap year)".to_string(),
        "IRC § 48 Construction and Placed-in-Service Deadlines — projects that begin construction PRIOR TO JANUARY 1, 2025 (or JANUARY 1, 2035 for GEOTHERMAL PROJECTS) and are placed in service AFTER 2021 are eligible for the full investment tax credit of 30%".to_string(),
        "§ 48 Eligible for § 6417 Direct Pay — one of the 12 APPLICABLE CREDIT CATEGORIES under § 6417(b)".to_string(),
        "§ 48 Eligible for § 6418 Transferability — one of the 11 ELIGIBLE CREDIT CATEGORIES under § 6418(f)(1); SYMMETRIC MONETIZATION FRAMEWORK (eligible for both § 6417 direct pay AND § 6418 transferability)".to_string(),
        "Form 3468 (Investment Credit) — required to claim § 48 credit; separate Form 3468 needed for each facility or property; redesigned by IRS for IRA 2022 provisions".to_string(),
        "Cornell LII + Bloomberg Tax + House.gov + Tax Notes + FindLaw + IRA Tracker + Federal Register + Mayer Brown + Holland & Knight + Novogradac + PwC + Reunion Infrastructure + Westchester County Association + Specialty Tax Group + Build With Basis + IRS — practitioner overviews of IRC § 48 Energy Credit".to_string(),
    ];

    if input.eligible_energy_property_category
        == EligibleEnergyPropertyCategory::NotEligibleEnergyProperty
    {
        return Output {
            mode: Section48Mode::NotApplicableNotEligibleEnergyProperty,
            statutory_basis: "IRC § 48(a)(3) — property not within 11 eligible energy categories".to_string(),
            notes: "NOT APPLICABLE: property not within the 11 eligible energy property categories under § 48(a)(3); § 48 credit unavailable.".to_string(),
            credit_amount_dollars: 0,
            citations,
        };
    }

    if input.boc_and_pis_status == BocAndPisStatus::BocOnOrAfterJanuary1_2025NonGeothermal {
        return Output {
            mode: Section48Mode::NotApplicableBocOnOrAfterJanuary1_2025NonGeothermal,
            statutory_basis: "IRC § 48 — BOC on or after January 1, 2025 (non-geothermal); transition to § 48E".to_string(),
            notes: "NOT APPLICABLE: non-geothermal facility began construction on or after January 1, 2025; § 48 credit unavailable; facility should claim § 48E technology-neutral ITC instead.".to_string(),
            credit_amount_dollars: 0,
            citations,
        };
    }

    if input.boc_and_pis_status == BocAndPisStatus::PisBeforeOrIn2021 {
        return Output {
            mode: Section48Mode::NotApplicablePisBeforeOrIn2021,
            statutory_basis: "IRC § 48 — PIS before or in 2021 (pre-IRA 2022 expansion)".to_string(),
            notes: "NOT APPLICABLE: facility placed in service before or in 2021; pre-IRA 2022 § 48 rates apply (legacy 30% step-down schedule), not the modernized IRA 2022 6%+24% PWA framework.".to_string(),
            credit_amount_dollars: 0,
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::EligibleEnergyPropertyCategoryUnderSection48A3 => Output {
            mode: Section48Mode::CompliantEligibleEnergyPropertyCategory,
            statutory_basis: format!(
                "IRC § 48(a)(3) — {c:?} within eligible energy property categories",
                c = input.eligible_energy_property_category,
            ),
            notes: format!(
                "COMPLIANT: {c:?} qualifies as eligible energy property under § 48(a)(3).",
                c = input.eligible_energy_property_category,
            ),
            credit_amount_dollars: 0,
            citations,
        },
        ComplianceAspect::BaseAndPwaCreditRateUnderSection48A => {
            let rate_percent = match input.pwa_status {
                PwaStatus::SatisfiesPwaRequirements => SECTION_48_PWA_RATE_PERCENT,
                PwaStatus::DoesNotSatisfyPwaRequirements => SECTION_48_BASE_RATE_PERCENT,
            };
            let credit_amount_dollars =
                (u128::from(input.qualified_investment_dollars) * u128::from(rate_percent) / 100)
                    .min(u128::from(u64::MAX)) as u64;
            match input.pwa_status {
                PwaStatus::SatisfiesPwaRequirements => Output {
                    mode: Section48Mode::CompliantPwaBumpedCreditAt30Percent,
                    statutory_basis: "IRC § 48(a) — 30% PWA-bumped rate".to_string(),
                    notes: format!(
                        "COMPLIANT: 30% PWA-bumped rate under § 48(a); ${qi} × 30% = ${ca} credit (5x multiplier from base 6%).",
                        qi = input.qualified_investment_dollars,
                        ca = credit_amount_dollars,
                    ),
                    credit_amount_dollars,
                    citations,
                },
                PwaStatus::DoesNotSatisfyPwaRequirements => Output {
                    mode: Section48Mode::CompliantBaseCreditAtSixPercent,
                    statutory_basis: "IRC § 48(a) — 6% base rate without PWA".to_string(),
                    notes: format!(
                        "COMPLIANT: 6% base rate without PWA under § 48(a); ${qi} × 6% = ${ca} credit.",
                        qi = input.qualified_investment_dollars,
                        ca = credit_amount_dollars,
                    ),
                    credit_amount_dollars,
                    citations,
                },
            }
        }
        ComplianceAspect::EnergyCommunity10PercentAdderUnderSection48A14C => {
            match input.energy_community_status {
                EnergyCommunityStatus::LocatedInEnergyCommunity => {
                    let base_rate_percent = match input.pwa_status {
                        PwaStatus::SatisfiesPwaRequirements => SECTION_48_PWA_RATE_PERCENT,
                        PwaStatus::DoesNotSatisfyPwaRequirements => SECTION_48_BASE_RATE_PERCENT,
                    };
                    let total_rate_percent =
                        base_rate_percent + SECTION_48_ENERGY_COMMUNITY_ADDER_PERCENT;
                    let credit_amount_dollars = (u128::from(input.qualified_investment_dollars)
                        * u128::from(total_rate_percent)
                        / 100)
                        .min(u128::from(u64::MAX)) as u64;
                    Output {
                        mode: Section48Mode::CompliantWithEnergyCommunityAdder10Percent,
                        statutory_basis: "IRC § 48(a)(14)(C) — +10 percentage points energy community adder".to_string(),
                        notes: format!(
                            "COMPLIANT: energy community +10pp adder under § 48(a)(14)(C); ${qi} × {tr}% = ${ca} credit ({br}% base + 10pp).",
                            qi = input.qualified_investment_dollars,
                            tr = total_rate_percent,
                            ca = credit_amount_dollars,
                            br = base_rate_percent,
                        ),
                        credit_amount_dollars,
                        citations,
                    }
                }
                EnergyCommunityStatus::NotLocatedInEnergyCommunity => Output {
                    mode: Section48Mode::ViolationEnergyCommunityAdderClaimedWithoutQualifyingLocation,
                    statutory_basis: "IRC § 48(a)(14)(C) — energy community adder claimed without qualifying location".to_string(),
                    notes: "VIOLATION: energy community +10pp adder claimed but facility NOT LOCATED IN energy community under § 48(a)(14)(C); brownfield site / coal community / fossil fuel employment area required.".to_string(),
                    credit_amount_dollars: 0,
                    citations,
                },
            }
        }
        ComplianceAspect::DomesticContent10PercentAdderUnderSection48A12 => {
            match input.domestic_content_status {
                DomesticContentStatus::SatisfiesDomesticContentRequirements => {
                    let base_rate_percent = match input.pwa_status {
                        PwaStatus::SatisfiesPwaRequirements => SECTION_48_PWA_RATE_PERCENT,
                        PwaStatus::DoesNotSatisfyPwaRequirements => SECTION_48_BASE_RATE_PERCENT,
                    };
                    let total_rate_percent =
                        base_rate_percent + SECTION_48_DOMESTIC_CONTENT_ADDER_PERCENT;
                    let credit_amount_dollars = (u128::from(input.qualified_investment_dollars)
                        * u128::from(total_rate_percent)
                        / 100)
                        .min(u128::from(u64::MAX)) as u64;
                    Output {
                        mode: Section48Mode::CompliantWithDomesticContentAdder10Percent,
                        statutory_basis: "IRC § 48(a)(12) — +10 percentage points domestic content adder".to_string(),
                        notes: format!(
                            "COMPLIANT: domestic content +10pp adder under § 48(a)(12); ${qi} × {tr}% = ${ca} credit ({br}% base + 10pp).",
                            qi = input.qualified_investment_dollars,
                            tr = total_rate_percent,
                            ca = credit_amount_dollars,
                            br = base_rate_percent,
                        ),
                        credit_amount_dollars,
                        citations,
                    }
                }
                DomesticContentStatus::DoesNotSatisfyDomesticContentRequirements => Output {
                    mode: Section48Mode::ViolationDomesticContentAdderClaimedWithoutMeetingRequirement,
                    statutory_basis: "IRC § 48(a)(12) — domestic content adder claimed without meeting requirement".to_string(),
                    notes: "VIOLATION: domestic content +10pp adder claimed but facility does NOT meet domestic content requirements under § 48(a)(12); specified percentages of US-sourced steel, iron, and manufactured products required.".to_string(),
                    credit_amount_dollars: 0,
                    citations,
                },
            }
        }
        ComplianceAspect::MaximumStackedCredit50Percent => {
            if input.pwa_status == PwaStatus::SatisfiesPwaRequirements
                && input.energy_community_status
                    == EnergyCommunityStatus::LocatedInEnergyCommunity
                && input.domestic_content_status
                    == DomesticContentStatus::SatisfiesDomesticContentRequirements
            {
                let credit_amount_dollars = (u128::from(input.qualified_investment_dollars)
                    * u128::from(SECTION_48_MAXIMUM_STACKED_RATE_PERCENT)
                    / 100)
                    .min(u128::from(u64::MAX)) as u64;
                Output {
                    mode: Section48Mode::CompliantMaximumStackedCreditAt50Percent,
                    statutory_basis: "IRC § 48 — maximum stacked credit 50% (PWA 30% + energy community 10pp + domestic content 10pp)".to_string(),
                    notes: format!(
                        "COMPLIANT: maximum stacked credit 50% under § 48 (PWA 30% + energy community +10pp + domestic content +10pp); ${qi} × 50% = ${ca} maximum credit.",
                        qi = input.qualified_investment_dollars,
                        ca = credit_amount_dollars,
                    ),
                    credit_amount_dollars,
                    citations,
                }
            } else {
                Output {
                    mode: Section48Mode::CompliantBaseCreditAtSixPercent,
                    statutory_basis: "IRC § 48 — maximum stacked credit not achieved; some adder requirements not met".to_string(),
                    notes: "NOTE: maximum stacked credit (50%) requires all three: PWA + energy community + domestic content; one or more requirements not met.".to_string(),
                    credit_amount_dollars: 0,
                    citations,
                }
            }
        }
        ComplianceAspect::LowIncomeCommunitiesBonusUnderSection48E => {
            match input.low_income_community_status {
                LowIncomeCommunityStatus::AllocatedLowIncomeCommunitiesBonusUnderSection48E => Output {
                    mode: Section48Mode::CompliantLowIncomeCommunitiesBonusAllocated,
                    statutory_basis: "IRC § 48(e) — low-income communities bonus allocated".to_string(),
                    notes: "COMPLIANT: facility allocated low-income communities bonus under § 48(e) capped national pool; ITC-only bonus (not available for § 45 PTC).".to_string(),
                    credit_amount_dollars: 0,
                    citations,
                },
                LowIncomeCommunityStatus::NotAllocatedLowIncomeCommunitiesBonus => Output {
                    mode: Section48Mode::ViolationLowIncomeCommunitiesBonusNotAllocated,
                    statutory_basis: "IRC § 48(e) — low-income communities bonus not allocated".to_string(),
                    notes: "VIOLATION: facility NOT allocated low-income communities bonus under § 48(e); allocation from capped national pool required.".to_string(),
                    credit_amount_dollars: 0,
                    citations,
                },
            }
        }
        ComplianceAspect::BocAndPisDeadlinesGeneralAndGeothermal => {
            match input.boc_and_pis_status {
                BocAndPisStatus::BocBeforeJanuary1_2025AndPisAfter2021 => Output {
                    mode: Section48Mode::CompliantBocBeforeJanuary1_2025AndPisAfter2021,
                    statutory_basis: "IRC § 48 — BOC before January 1, 2025 and PIS after 2021".to_string(),
                    notes: "COMPLIANT: facility began construction before January 1, 2025 and placed in service after 2021; eligible for full IRA 2022 § 48 ITC framework.".to_string(),
                    credit_amount_dollars: 0,
                    citations,
                },
                BocAndPisStatus::GeothermalBocBeforeJanuary1_2035AndPisAfter2021 => Output {
                    mode: Section48Mode::CompliantGeothermalBocBeforeJanuary1_2035AndPisAfter2021,
                    statutory_basis: "IRC § 48 — geothermal BOC before January 1, 2035 (extended cutoff) and PIS after 2021".to_string(),
                    notes: "COMPLIANT: GEOTHERMAL facility began construction before January 1, 2035 (extended 10-year cutoff compared to general § 48 cutoff of January 1, 2025) and placed in service after 2021; eligible for full IRA 2022 § 48 ITC framework.".to_string(),
                    credit_amount_dollars: 0,
                    citations,
                },
                BocAndPisStatus::BocOnOrAfterJanuary1_2025NonGeothermal => Output {
                    mode: Section48Mode::NotApplicableBocOnOrAfterJanuary1_2025NonGeothermal,
                    statutory_basis: "IRC § 48 — BOC on or after January 1, 2025 (non-geothermal); transition to § 48E".to_string(),
                    notes: "NOT APPLICABLE: non-geothermal facility began construction on or after January 1, 2025; facility should claim § 48E technology-neutral ITC instead.".to_string(),
                    credit_amount_dollars: 0,
                    citations,
                },
                BocAndPisStatus::PisBeforeOrIn2021 => Output {
                    mode: Section48Mode::NotApplicablePisBeforeOrIn2021,
                    statutory_basis: "IRC § 48 — PIS before or in 2021 (pre-IRA 2022 expansion)".to_string(),
                    notes: "NOT APPLICABLE: facility placed in service before or in 2021; pre-IRA 2022 § 48 rates apply.".to_string(),
                    credit_amount_dollars: 0,
                    citations,
                },
            }
        }
        ComplianceAspect::FiveYearRecapturePeriod => match input.recapture_status {
            RecaptureStatus::No5YearRecaptureTriggered => Output {
                mode: Section48Mode::CompliantFiveYearRecapturePeriodSatisfied,
                statutory_basis: "IRC § 48 — 5-year recapture period satisfied".to_string(),
                notes: "COMPLIANT: no disposition or PWA failure during 5-year recapture period (365-day year increments from PIS); credit fully vested.".to_string(),
                credit_amount_dollars: 0,
                citations,
            },
            RecaptureStatus::RecaptureTriggeredByDispositionOrPwaFailureDuring5YearPeriod => Output {
                mode: Section48Mode::ViolationRecaptureTriggeredDuring5YearPeriod,
                statutory_basis: "IRC § 48 — recapture triggered during 5-year period".to_string(),
                notes: "VIOLATION: disposition or PWA failure during 5-year recapture period; subject to recapture of increased credit amount with 20% annual ramp-down.".to_string(),
                credit_amount_dollars: 0,
                citations,
            },
        },
        ComplianceAspect::EligibilityForSection6417DirectPay => Output {
            mode: Section48Mode::CompliantEligibleForSection6417DirectPay,
            statutory_basis: "IRC § 6417(b) — § 48 eligible for direct pay election".to_string(),
            notes: "COMPLIANT: § 48 is one of the 12 APPLICABLE CREDIT CATEGORIES under § 6417(b) for direct pay election by applicable entities.".to_string(),
            credit_amount_dollars: 0,
            citations,
        },
        ComplianceAspect::EligibilityForSection6418Transferability => Output {
            mode: Section48Mode::CompliantEligibleForSection6418Transferability,
            statutory_basis: "IRC § 6418(f)(1) — § 48 eligible for transferability + symmetric with § 6417".to_string(),
            notes: "COMPLIANT: § 48 is one of the 11 ELIGIBLE CREDIT CATEGORIES under § 6418(f)(1) for transferability monetization; SYMMETRIC MONETIZATION FRAMEWORK (eligible for both § 6417 direct pay AND § 6418 transferability).".to_string(),
            credit_amount_dollars: 0,
            citations,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            eligible_energy_property_category: EligibleEnergyPropertyCategory::SolarEnergyEquipment,
            pwa_status: PwaStatus::SatisfiesPwaRequirements,
            boc_and_pis_status: BocAndPisStatus::BocBeforeJanuary1_2025AndPisAfter2021,
            energy_community_status: EnergyCommunityStatus::NotLocatedInEnergyCommunity,
            domestic_content_status:
                DomesticContentStatus::DoesNotSatisfyDomesticContentRequirements,
            low_income_community_status:
                LowIncomeCommunityStatus::NotAllocatedLowIncomeCommunitiesBonus,
            recapture_status: RecaptureStatus::No5YearRecaptureTriggered,
            compliance_aspect: ComplianceAspect::EligibleEnergyPropertyCategoryUnderSection48A3,
            qualified_investment_dollars: 100_000_000,
        }
    }

    #[test]
    fn not_eligible_energy_property_not_applicable() {
        let mut input = baseline_input();
        input.eligible_energy_property_category =
            EligibleEnergyPropertyCategory::NotEligibleEnergyProperty;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::NotApplicableNotEligibleEnergyProperty);
    }

    #[test]
    fn boc_after_january_2025_non_geothermal_not_applicable() {
        let mut input = baseline_input();
        input.boc_and_pis_status = BocAndPisStatus::BocOnOrAfterJanuary1_2025NonGeothermal;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::NotApplicableBocOnOrAfterJanuary1_2025NonGeothermal);
    }

    #[test]
    fn pis_before_or_in_2021_not_applicable() {
        let mut input = baseline_input();
        input.boc_and_pis_status = BocAndPisStatus::PisBeforeOrIn2021;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::NotApplicablePisBeforeOrIn2021);
    }

    #[test]
    fn solar_energy_equipment_eligible_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleEnergyPropertyCategoryUnderSection48A3;
        input.eligible_energy_property_category = EligibleEnergyPropertyCategory::SolarEnergyEquipment;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantEligibleEnergyPropertyCategory);
    }

    #[test]
    fn geothermal_eligible_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleEnergyPropertyCategoryUnderSection48A3;
        input.eligible_energy_property_category =
            EligibleEnergyPropertyCategory::GeothermalEnergyEquipment;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantEligibleEnergyPropertyCategory);
    }

    #[test]
    fn energy_storage_eligible_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleEnergyPropertyCategoryUnderSection48A3;
        input.eligible_energy_property_category = EligibleEnergyPropertyCategory::EnergyStorageTechnology;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantEligibleEnergyPropertyCategory);
    }

    #[test]
    fn qualified_biogas_eligible_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleEnergyPropertyCategoryUnderSection48A3;
        input.eligible_energy_property_category = EligibleEnergyPropertyCategory::QualifiedBiogasProperty;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantEligibleEnergyPropertyCategory);
    }

    #[test]
    fn microgrid_controllers_eligible_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleEnergyPropertyCategoryUnderSection48A3;
        input.eligible_energy_property_category = EligibleEnergyPropertyCategory::MicrogridControllers;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantEligibleEnergyPropertyCategory);
    }

    #[test]
    fn dynamic_glass_ira_addition_eligible_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleEnergyPropertyCategoryUnderSection48A3;
        input.eligible_energy_property_category =
            EligibleEnergyPropertyCategory::DynamicGlassElectrochromicAddedByIra2022;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantEligibleEnergyPropertyCategory);
    }

    #[test]
    fn linear_generators_ira_addition_eligible_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleEnergyPropertyCategoryUnderSection48A3;
        input.eligible_energy_property_category =
            EligibleEnergyPropertyCategory::LinearGeneratorsAddedByIra2022;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantEligibleEnergyPropertyCategory);
    }

    #[test]
    fn six_percent_base_rate_without_pwa_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseAndPwaCreditRateUnderSection48A;
        input.pwa_status = PwaStatus::DoesNotSatisfyPwaRequirements;
        input.qualified_investment_dollars = 100_000_000;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantBaseCreditAtSixPercent);
        assert_eq!(out.credit_amount_dollars, 6_000_000);
    }

    #[test]
    fn thirty_percent_pwa_bumped_rate_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseAndPwaCreditRateUnderSection48A;
        input.pwa_status = PwaStatus::SatisfiesPwaRequirements;
        input.qualified_investment_dollars = 100_000_000;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantPwaBumpedCreditAt30Percent);
        assert_eq!(out.credit_amount_dollars, 30_000_000);
    }

    #[test]
    fn pwa_5x_multiplier_invariant_verified() {
        let mut input_base = baseline_input();
        input_base.compliance_aspect = ComplianceAspect::BaseAndPwaCreditRateUnderSection48A;
        input_base.pwa_status = PwaStatus::DoesNotSatisfyPwaRequirements;
        input_base.qualified_investment_dollars = 1_000_000_000;

        let mut input_pwa = baseline_input();
        input_pwa.compliance_aspect = ComplianceAspect::BaseAndPwaCreditRateUnderSection48A;
        input_pwa.pwa_status = PwaStatus::SatisfiesPwaRequirements;
        input_pwa.qualified_investment_dollars = 1_000_000_000;

        let out_base = check(&input_base);
        let out_pwa = check(&input_pwa);

        assert_eq!(out_pwa.credit_amount_dollars / out_base.credit_amount_dollars, 5);
    }

    #[test]
    fn energy_community_adder_with_pwa_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EnergyCommunity10PercentAdderUnderSection48A14C;
        input.energy_community_status = EnergyCommunityStatus::LocatedInEnergyCommunity;
        input.pwa_status = PwaStatus::SatisfiesPwaRequirements;
        input.qualified_investment_dollars = 100_000_000;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantWithEnergyCommunityAdder10Percent);
        assert_eq!(out.credit_amount_dollars, 40_000_000);
    }

    #[test]
    fn energy_community_adder_without_qualifying_location_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EnergyCommunity10PercentAdderUnderSection48A14C;
        input.energy_community_status = EnergyCommunityStatus::NotLocatedInEnergyCommunity;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48Mode::ViolationEnergyCommunityAdderClaimedWithoutQualifyingLocation
        );
    }

    #[test]
    fn domestic_content_adder_with_pwa_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DomesticContent10PercentAdderUnderSection48A12;
        input.domestic_content_status =
            DomesticContentStatus::SatisfiesDomesticContentRequirements;
        input.pwa_status = PwaStatus::SatisfiesPwaRequirements;
        input.qualified_investment_dollars = 100_000_000;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantWithDomesticContentAdder10Percent);
        assert_eq!(out.credit_amount_dollars, 40_000_000);
    }

    #[test]
    fn maximum_stacked_credit_at_50_percent_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::MaximumStackedCredit50Percent;
        input.pwa_status = PwaStatus::SatisfiesPwaRequirements;
        input.energy_community_status = EnergyCommunityStatus::LocatedInEnergyCommunity;
        input.domestic_content_status =
            DomesticContentStatus::SatisfiesDomesticContentRequirements;
        input.qualified_investment_dollars = 100_000_000;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantMaximumStackedCreditAt50Percent);
        assert_eq!(out.credit_amount_dollars, 50_000_000);
    }

    #[test]
    fn low_income_communities_bonus_allocated_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LowIncomeCommunitiesBonusUnderSection48E;
        input.low_income_community_status =
            LowIncomeCommunityStatus::AllocatedLowIncomeCommunitiesBonusUnderSection48E;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantLowIncomeCommunitiesBonusAllocated);
    }

    #[test]
    fn boc_before_2025_and_pis_after_2021_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BocAndPisDeadlinesGeneralAndGeothermal;
        input.boc_and_pis_status = BocAndPisStatus::BocBeforeJanuary1_2025AndPisAfter2021;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantBocBeforeJanuary1_2025AndPisAfter2021);
    }

    #[test]
    fn geothermal_boc_before_2035_extended_cutoff_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BocAndPisDeadlinesGeneralAndGeothermal;
        input.boc_and_pis_status = BocAndPisStatus::GeothermalBocBeforeJanuary1_2035AndPisAfter2021;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48Mode::CompliantGeothermalBocBeforeJanuary1_2035AndPisAfter2021
        );
    }

    #[test]
    fn five_year_recapture_period_satisfied_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveYearRecapturePeriod;
        input.recapture_status = RecaptureStatus::No5YearRecaptureTriggered;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantFiveYearRecapturePeriodSatisfied);
    }

    #[test]
    fn recapture_triggered_during_5_year_period_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FiveYearRecapturePeriod;
        input.recapture_status =
            RecaptureStatus::RecaptureTriggeredByDispositionOrPwaFailureDuring5YearPeriod;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::ViolationRecaptureTriggeredDuring5YearPeriod);
    }

    #[test]
    fn eligible_for_section_6417_direct_pay_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibilityForSection6417DirectPay;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantEligibleForSection6417DirectPay);
    }

    #[test]
    fn eligible_for_section_6418_transferability_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibilityForSection6418Transferability;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantEligibleForSection6418Transferability);
    }

    #[test]
    fn pwa_overflow_defense_at_u64_max() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseAndPwaCreditRateUnderSection48A;
        input.pwa_status = PwaStatus::SatisfiesPwaRequirements;
        input.qualified_investment_dollars = u64::MAX;
        let out = check(&input);
        assert_eq!(out.mode, Section48Mode::CompliantPwaBumpedCreditAt30Percent);
        assert!(out.credit_amount_dollars > 0);
    }

    #[test]
    fn constants_pin_section_48_statutory_thresholds() {
        assert_eq!(SECTION_48_NUMBER, 48);
        assert_eq!(SECTION_48_BASE_RATE_PERCENT, 6);
        assert_eq!(SECTION_48_PWA_RATE_PERCENT, 30);
        assert_eq!(SECTION_48_PWA_MULTIPLIER, 5);
        assert_eq!(SECTION_48_ENERGY_COMMUNITY_ADDER_PERCENT, 10);
        assert_eq!(SECTION_48_DOMESTIC_CONTENT_ADDER_PERCENT, 10);
        assert_eq!(SECTION_48_MAXIMUM_STACKED_RATE_PERCENT, 50);
        assert_eq!(SECTION_48_RECAPTURE_PERIOD_YEARS, 5);
        assert_eq!(SECTION_48_RECAPTURE_RAMP_DOWN_PERCENT_PER_YEAR, 20);
        assert_eq!(SECTION_48_BOC_CUTOFF_YEAR_GENERAL, 2025);
        assert_eq!(SECTION_48_BOC_CUTOFF_YEAR_GEOTHERMAL, 2035);
        assert_eq!(SECTION_48_PIS_START_YEAR, 2022);
        assert_eq!(SECTION_48_WERP_CAPACITY_LIMIT_MW, 50);
        assert_eq!(SECTION_48_BIOGAS_MIN_METHANE_PERCENT, 52);
        assert_eq!(SECTION_48_MICROGRID_MIN_KW, 4);
        assert_eq!(SECTION_48_MICROGRID_MAX_MW, 20);
        assert_eq!(SECTION_48_ENERGY_PROPERTY_CATEGORIES_COUNT, 11);
        assert_eq!(SECTION_48_IRA_2022_ENACTMENT_YEAR, 2022);
        assert_eq!(SECTION_48_IRA_2022_PUBLIC_LAW_NUMBER, 117169);
        assert_eq!(SECTION_48_ENERGY_TAX_ACT_1978_PUBLIC_LAW_NUMBER, 95618);
        assert_eq!(SECTION_48_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_section_48_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Inflation Reduction Act of 2022 § 13102"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("Energy Tax Act of 1978"));
        assert!(joined.contains("Public Law 95-618"));
        assert!(joined.contains("November 9, 1978"));
        assert!(joined.contains("IRC § 48(a)"));
        assert!(joined.contains("6% BASE RATE"));
        assert!(joined.contains("24% PWA BONUS"));
        assert!(joined.contains("30% TOTAL"));
        assert!(joined.contains("PREVAILING WAGE AND APPRENTICESHIP REQUIREMENTS"));
        assert!(joined.contains("IRC § 48(a)(3)"));
        assert!(joined.contains("SOLAR ENERGY EQUIPMENT"));
        assert!(joined.contains("GEOTHERMAL ENERGY EQUIPMENT"));
        assert!(joined.contains("QUALIFIED FUEL CELLS"));
        assert!(joined.contains("MICROTURBINE PROPERTY"));
        assert!(joined.contains("COMBINED HEAT AND POWER (CHP) SYSTEM PROPERTY"));
        assert!(joined.contains("QUALIFIED SMALL WIND ENERGY PROPERTY"));
        assert!(joined.contains("GROUND OR GROUNDWATER AS THERMAL ENERGY SOURCE"));
        assert!(joined.contains("WASTE ENERGY RECOVERY PROPERTY (WERP)"));
        assert!(joined.contains("ENERGY STORAGE TECHNOLOGY"));
        assert!(joined.contains("QUALIFIED BIOGAS PROPERTY"));
        assert!(joined.contains("52% methane"));
        assert!(joined.contains("MICROGRID CONTROLLERS"));
        assert!(joined.contains("4 kW to 20 MW"));
        assert!(joined.contains("DYNAMIC GLASS (ELECTROCHROMIC)"));
        assert!(joined.contains("LINEAR GENERATORS"));
        assert!(joined.contains("IRC § 48(a)(14)(C)"));
        assert!(joined.contains("ENERGY COMMUNITY"));
        assert!(joined.contains("+10 PERCENTAGE POINTS"));
        assert!(joined.contains("IRC § 48(a)(12)"));
        assert!(joined.contains("DOMESTIC CONTENT"));
        assert!(joined.contains("50 PERCENT of qualified investment"));
        assert!(joined.contains("IRC § 48(e)"));
        assert!(joined.contains("LOW-INCOME COMMUNITIES"));
        assert!(joined.contains("5-Year Recapture Period"));
        assert!(joined.contains("365-DAY YEAR INCREMENTS"));
        assert!(joined.contains("20 PERCENT"));
        assert!(joined.contains("JANUARY 1, 2025"));
        assert!(joined.contains("JANUARY 1, 2035 for GEOTHERMAL PROJECTS"));
        assert!(joined.contains("placed in service AFTER 2021"));
        assert!(joined.contains("§ 6417(b)"));
        assert!(joined.contains("§ 6418(f)(1)"));
        assert!(joined.contains("SYMMETRIC MONETIZATION FRAMEWORK"));
        assert!(joined.contains("Form 3468"));
    }
}
