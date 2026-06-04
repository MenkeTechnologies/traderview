//! IRC § 48E Clean Electricity Investment Credit
//! Compliance Module — pure-compute check for the
//! technology-neutral investment tax credit (ITC) for
//! qualified investments in zero-emission electricity
//! generating facilities AND qualified energy storage
//! technology (batteries, pumped hydro, thermal storage)
//! placed in service after December 31, 2024. Investment
//! tax credit counterpart to the § 45Y production tax
//! credit; taxpayers may elect ITC OR PTC for the same
//! qualified facility but NOT BOTH.
//!
//! Originally enacted by **Section 13702 of the Inflation
//! Reduction Act of 2022 (Public Law 117-169)**, signed by
//! President Joe Biden on **August 16, 2022**, effective
//! for facilities **PLACED IN SERVICE AFTER DECEMBER 31,
//! 2024**. **MODIFIED by Section 70513 of the One Big
//! Beautiful Bill Act of 2025 (Public Law 119-21)**, signed
//! by President Donald Trump on **July 4, 2025**; new
//! § 48E(e)(4) added by OBBBA TERMINATES § 48E credit for
//! **APPLICABLE WIND FACILITIES** and **APPLICABLE SOLAR
//! FACILITIES** where (i) **BEGINNING OF CONSTRUCTION (BOC)**
//! occurs after **JULY 4, 2026** (12-month anniversary of
//! OBBBA enactment) OR (ii) facility is **PLACED IN SERVICE
//! AFTER DECEMBER 31, 2027**.
//!
//! Web research (verified 2026-06-03):
//! - **IRA 2022 Enactment**: IRC § 48E added by **Section 13702 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818)**; signed by President Joe Biden on **August 16, 2022**; effective for QUALIFIED FACILITIES and QUALIFIED ENERGY STORAGE TECHNOLOGY PLACED IN SERVICE AFTER **DECEMBER 31, 2024**; the technology-neutral successor to § 48 (energy investment credit) for facilities placed in service after 2024 ([Federal Register — Section 45Y Clean Electricity Production Credit and Section 48E Clean Electricity Investment Credit Final Regulations (January 15, 2025)](https://www.federalregister.gov/documents/2025/01/15/2025-00196/section-45y-clean-electricity-production-credit-and-section-48e-clean-electricity-investment-credit); [IRA Tracker — IRA Section 13702 Clean Electricity Investment Credit](https://iratracker.org/programs/ira-section-13702-clean-electricity-investment-credit/); [IRS — Clean Electricity Investment Credit](https://www.irs.gov/credits-deductions/clean-electricity-investment-credit); [Cornell LII — 26 U.S. Code § 48E](https://www.law.cornell.edu/uscode/text/26/48E); [Bloomberg Tax — Sec. 48E Clean Electricity Investment Credit](https://irc.bloombergtax.com/public/uscode/doc/irc/section_48e); [IRS — Sections 45Y and 48E Beginning of Construction Notice 2025-42](https://www.irs.gov/pub/irs-drop/n-25-42.pdf); [IRS — Clean Electricity Low-Income Communities Bonus Credit Amount Program](https://www.irs.gov/credits-deductions/clean-electricity-low-income-communities-bonus-credit-amount-program); [Build With Basis — 48E Tax Credit: Claiming the Clean Electricity ITC](https://www.buildwithbasis.com/insights/48e-itc-final-rules-what-the-clean-electricity-investment-tax-credit-means-for-project-developers); [Ryan — IRS Finalizes New Rules for ITC (IRC 48 and 48E) and PTC (IRC 45 and 45Y)](https://www.ryan.com/about-ryan/news-and-insights/2025/irs-finalizes-new-rules-for-investment-tax-credit/); [The Tax Adviser — Navigating Safe-Harbor Rules for Solar and Wind Sec. 48E Facilities (February 2026)](https://www.thetaxadviser.com/issues/2026/feb/navigating-safe-harbor-rules-for-solar-and-wind-sec-48e-facilities/); [Withum — Navigating Safe-Harbor Rules for Solar and Wind Sec. 48E Facilities](https://www.withum.com/resources/navigating-safe-harbor-rules-for-solar-and-wind-sec-48e-facilities/); [Mayer Brown — IRS Releases Updated OBBBA-Related Energy Credit Guidance](https://www.mayerbrown.com/en/insights/publications/2025/08/irs-releases-updated-obbba-related-energy-credit-guidance); [DLA Piper — IRS Guidance on Wind and Solar Facility Tax Credits](https://www.dlapiper.com/en-us/insights/publications/2025/08/latest-irs-guidance-on-wind-and-solar-facility-tax-credits); [Sidley Austin — The One Big Beautiful Bill Act Navigating the New Energy Landscape](https://www.sidley.com/en/insights/newsupdates/2025/07/the-one-big-beautiful-bill-act-navigating-the-new-energy-landscape); [RSM US — Tax Bill Significantly Changes Clean Energy Credits](https://rsmus.com/insights/services/business-tax/obbba-tax-clean-energy.html); [CESA — OBBBA Tax Credits Summary Diagram](https://www.cesa.org/resource-library/resource/obbba-summary-diagram/); [IRS — Instructions for Form 3468 (2025)](https://www.irs.gov/instructions/i3468)).
//! - **§ 48E(a) Base Credit Amount**: **6 PERCENT of qualified investment** in the qualified facility OR qualified energy storage technology placed in service during the taxable year.
//! - **§ 48E(a)(3)(A) Bonus Credit (PWA)**: base credit MULTIPLIED BY **5** (yielding **30 PERCENT** of qualified investment) for projects meeting the **PREVAILING WAGE AND APPRENTICESHIP (PWA)** requirements during construction and for the 5-year recapture period.
//! - **§ 48E(b)(2) Qualified Facility Definition**: facility used for the **GENERATION OF ELECTRICITY** that is placed in service after December 31, 2024 and has a **GREENHOUSE GAS EMISSIONS RATE of NOT GREATER THAN ZERO**.
//! - **§ 48E(c) Qualified Energy Storage Technology**: § 48E uniquely includes **ENERGY STORAGE TECHNOLOGY** (batteries, pumped hydro, thermal storage) as separately eligible for the credit — § 45Y PTC excludes storage technology because storage does not "produce" electricity for purposes of the production credit.
//! - **§ 48E(a)(3)(B) Domestic Content Bonus**: **+10 PERCENTAGE POINTS** ADDITIONAL credit for facilities meeting **DOMESTIC CONTENT** requirements (specified percentages of US-sourced steel, iron, and manufactured products); ADDITIVE adder (NOT multiplicative — different from how § 25D / § 25C adders work).
//! - **§ 48E(a)(3)(C) Energy Community Bonus**: **+10 PERCENTAGE POINTS** ADDITIONAL credit for facilities located in an **ENERGY COMMUNITY** (brownfield site, coal community, or area with high fossil fuel employment historically); ADDITIVE adder.
//! - **§ 48E Maximum Stacked Credit**: with all bonuses (PWA + domestic content + energy community) stacked, the credit may reach **50 PERCENT of qualified investment** (30 % PWA + 10 % domestic content + 10 % energy community).
//! - **§ 48E(g) Low-Income Communities Bonus Credit Amount Program**: additional bonus allocation (capped national pool) for facilities located in LOW-INCOME COMMUNITIES; administered by IRS Notice 2024-x annual allocation procedures.
//! - **§ 48E(d) Recapture**: 5-year recapture period for any disposition or cessation of qualified use during the recapture period.
//! - **OBBBA 2025 § 70513 Wind/Solar Termination (NEW § 48E(e)(4))**: § 48E credit ELIMINATED for **APPLICABLE WIND FACILITIES** and **APPLICABLE SOLAR FACILITIES** where (i) **BEGINNING OF CONSTRUCTION (BOC)** occurs after **JULY 4, 2026** (12-month anniversary of OBBBA enactment) OR (ii) facility is **PLACED IN SERVICE AFTER DECEMBER 31, 2027**.
//! - **OBBBA 2025 § 70513 Other Technologies Preserved**: § 48E credit REMAINS AVAILABLE for non-wind/non-solar qualified facilities (geothermal, hydropower, marine and hydrokinetic, nuclear, fuel cell) AND for energy storage technology under the original IRA 2022 phase-out timeline (later of 2032 or year US power-sector emissions ≤ 25 % of 2022 baseline).
//! - **IRS Notice 2025-42 Beginning of Construction Guidance**: IRS issued **Notice 2025-42** establishing **PHYSICAL WORK TEST** as the EXCLUSIVE method for establishing BOC before July 5, 2026 for wind/solar; the previously-available **5 PERCENT SAFE HARBOR** is NOT available for the OBBBA wind/solar BOC anniversary cutoff (except for certain low-output solar facilities).
//! - **Final Regulations T.D. 10024 (January 15, 2025)**: Treasury and IRS issued final regulations under § 45Y and § 48E published in the Federal Register; provide rules for greenhouse gas emissions rate determination, qualified facility components, recapture procedures, direct pay election, and transferability election.
//! - **Form 3468 (Investment Credit)**: required to claim the § 48E credit; current instructions IRS Form 3468 (2025).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_48E_IRA_ENACTMENT_DATE_YEAR: u32 = 2022;
pub const IRC_48E_IRA_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_48E_IRA_ENACTMENT_DATE_DAY: u32 = 16;
pub const IRC_48E_IRA_PUBLIC_LAW_CONGRESS: u32 = 117;
pub const IRC_48E_IRA_PUBLIC_LAW_ENACTMENT: u32 = 169;
pub const IRC_48E_IRA_ENABLING_SECTION: u32 = 13702;
pub const IRC_48E_EFFECTIVE_DATE_YEAR: u32 = 2025;
pub const IRC_48E_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const IRC_48E_EFFECTIVE_DATE_DAY: u32 = 1;
pub const IRC_48E_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_48E_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_48E_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_48E_OBBBA_PUBLIC_LAW_CONGRESS: u32 = 119;
pub const IRC_48E_OBBBA_PUBLIC_LAW_ENACTMENT: u32 = 21;
pub const IRC_48E_OBBBA_ENABLING_SECTION: u32 = 70513;
pub const IRC_48E_OBBBA_WIND_SOLAR_BOC_CUTOFF_YEAR: u32 = 2026;
pub const IRC_48E_OBBBA_WIND_SOLAR_BOC_CUTOFF_MONTH: u32 = 7;
pub const IRC_48E_OBBBA_WIND_SOLAR_BOC_CUTOFF_DAY: u32 = 4;
pub const IRC_48E_OBBBA_WIND_SOLAR_PIS_CUTOFF_YEAR: u32 = 2027;
pub const IRC_48E_OBBBA_WIND_SOLAR_PIS_CUTOFF_MONTH: u32 = 12;
pub const IRC_48E_OBBBA_WIND_SOLAR_PIS_CUTOFF_DAY: u32 = 31;
pub const IRC_48E_ORIGINAL_PHASE_OUT_YEAR: u32 = 2032;
pub const IRC_48E_BASE_RATE_BPS: u64 = 600;
pub const IRC_48E_BONUS_RATE_BPS: u64 = 3_000;
pub const IRC_48E_BONUS_MULTIPLIER: u64 = 5;
pub const IRC_48E_DOMESTIC_CONTENT_ADDER_BPS: u64 = 1_000;
pub const IRC_48E_ENERGY_COMMUNITY_ADDER_BPS: u64 = 1_000;
pub const IRC_48E_MAX_STACKED_RATE_BPS: u64 = 5_000;
pub const IRC_48E_RECAPTURE_PERIOD_YEARS: u32 = 5;
pub const IRC_48E_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_48E_GHG_EMISSIONS_RATE_MAX_GRAMS_CO2E_PER_KWH: u32 = 0;
pub const IRC_48E_FINAL_REGS_PUBLICATION_DATE_YEAR: u32 = 2025;
pub const IRC_48E_FINAL_REGS_PUBLICATION_DATE_MONTH: u32 = 1;
pub const IRC_48E_FINAL_REGS_PUBLICATION_DATE_DAY: u32 = 15;
pub const IRC_48E_FORM_NUMBER: u32 = 3468;
pub const IRC_48E_BOC_GUIDANCE_NOTICE_YEAR: u32 = 2025;
pub const IRC_48E_BOC_GUIDANCE_NOTICE_NUMBER: u32 = 42;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacedInServiceDateStatus {
    PlacedInServiceOnOrBeforeDecember31_2024PreEffective,
    PlacedInServiceAfterDecember31_2024PostEffectiveEligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FacilityTechnologyType {
    AppliableWindFacility,
    AppliableSolarFacility,
    NonWindNonSolarZeroEmissionFacility,
    QualifiedEnergyStorageTechnology,
    NonQualifyingGreenhouseGasEmittingFacility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BeginningOfConstructionStatus {
    BocOnOrBeforeJuly4_2026PreObbbaAnniversary,
    BocAfterJuly4_2026PostObbbaAnniversary,
    NotApplicableNonWindNonSolarOrStorageFacility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindSolarPlacedInServiceStatus {
    PlacedInServiceOnOrBeforeDecember31_2027PreObbbaWindSolarCutoff,
    PlacedInServiceAfterDecember31_2027PostObbbaWindSolarCutoff,
    NotApplicableNonWindNonSolarOrStorageFacility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrevailingWageApprenticeshipStatus {
    PwaRequirementsMetEligibleForBonusRate,
    PwaRequirementsNotMetBaseRateOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DomesticContentAdderStatus {
    DomesticContentRequirementMet,
    DomesticContentRequirementNotMet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnergyCommunityAdderStatus {
    LocatedInEnergyCommunity,
    NotLocatedInEnergyCommunity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    BaseCreditAmountUnderSection48EA,
    BonusCreditAmountForPwaUnderSection48EA3A,
    QualifiedFacilityDefinitionUnderSection48EB2,
    QualifiedEnergyStorageTechnologyUnderSection48EC,
    DomesticContentAdderUnderSection48EA3B,
    EnergyCommunityAdderUnderSection48EA3C,
    StackedMaximumCreditWithAllBonusAdders,
    RecaptureFiveYearPeriodUnderSection48ED,
    ObbbaWindSolarTerminationUnderSection70513,
    FormFilingUnderForm3468,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section48EMode {
    NotApplicablePlacedInServiceOnOrBeforeDecember31_2024PreEffective,
    NotApplicableNonQualifyingGreenhouseGasEmittingFacility,
    NotApplicableWindOrSolarBocAfterJuly4_2026PostObbbaAnniversary,
    NotApplicableWindOrSolarPlacedInServiceAfterDecember31_2027PostObbbaCutoff,
    CompliantBaseCreditAtSixPercentOfQualifiedInvestment,
    CompliantBonusCreditAtThirtyPercentPwaSatisfied,
    CompliantWithEnergyCommunityAdder,
    CompliantWithDomesticContentAdder,
    CompliantWithBothBonusAddersStacked,
    CompliantMaximumStackedCreditAtFiftyPercent,
    CompliantQualifiedFacilityPlacedInServiceAfterDecember31_2024,
    CompliantQualifiedEnergyStorageTechnology,
    CompliantRecapturePeriodSatisfied,
    CompliantNonWindNonSolarFacilityUnaffectedByObbbaWindSolarTermination,
    CompliantForm3468FiledCorrectly,
    ViolationPwaBonusRateClaimedWithoutMeetingPwaRequirements,
    ViolationEnergyCommunityAdderClaimedWithoutQualifyingLocation,
    ViolationDomesticContentAdderClaimedWithoutMeetingRequirement,
    ViolationRecaptureTriggeredByDisposition,
    ViolationForm3468NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub placed_in_service_date_status: PlacedInServiceDateStatus,
    pub facility_technology_type: FacilityTechnologyType,
    pub beginning_of_construction_status: BeginningOfConstructionStatus,
    pub wind_solar_placed_in_service_status: WindSolarPlacedInServiceStatus,
    pub pwa_status: PrevailingWageApprenticeshipStatus,
    pub domestic_content_status: DomesticContentAdderStatus,
    pub energy_community_status: EnergyCommunityAdderStatus,
    pub compliance_aspect: ComplianceAspect,
    pub qualified_investment_dollars: u64,
    pub years_since_placed_in_service: u32,
    pub disposition_during_recapture_period: bool,
    pub form_3468_filed_correctly: bool,
    pub claimed_pwa_bonus_rate: bool,
    pub claimed_energy_community_adder: bool,
    pub claimed_domestic_content_adder: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section48EMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section48EInput = Input;
pub type Section48EOutput = Output;
pub type Section48EResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 48E Clean Electricity Investment Credit added by Section 13702 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818); signed by President Joe Biden on August 16, 2022; effective for qualified facilities and qualified energy storage technology placed in service after December 31, 2024".to_string(),
        "IRC § 48E(a) Base Credit Amount — 6 PERCENT of qualified investment in the qualified facility or qualified energy storage technology placed in service during the taxable year".to_string(),
        "IRC § 48E(a)(3)(A) Bonus Credit Amount (PWA) — base credit MULTIPLIED BY 5 (yielding 30 PERCENT of qualified investment) for projects meeting the prevailing wage and apprenticeship (PWA) requirements during construction and for the 5-year recapture period".to_string(),
        "IRC § 48E(b)(2) Qualified Facility Definition — facility used for the GENERATION OF ELECTRICITY that is placed in service after December 31, 2024 and has a GREENHOUSE GAS EMISSIONS RATE of NOT GREATER THAN ZERO".to_string(),
        "IRC § 48E(c) Qualified Energy Storage Technology — § 48E uniquely includes ENERGY STORAGE TECHNOLOGY (batteries, pumped hydro, thermal storage) as separately eligible for the credit; § 45Y PTC excludes storage technology because storage does not produce electricity for purposes of the production credit".to_string(),
        "IRC § 48E(a)(3)(B) Domestic Content Adder — +10 PERCENTAGE POINTS additional credit for facilities meeting DOMESTIC CONTENT requirements (specified percentages of US-sourced steel, iron, and manufactured products); ADDITIVE adder (NOT multiplicative)".to_string(),
        "IRC § 48E(a)(3)(C) Energy Community Adder — +10 PERCENTAGE POINTS additional credit for facilities located in an ENERGY COMMUNITY (brownfield site, coal community, or area with high fossil fuel employment historically); ADDITIVE adder".to_string(),
        "IRC § 48E Maximum Stacked Credit — with all bonuses (PWA + domestic content + energy community) stacked, the credit may reach 50 PERCENT of qualified investment (30 % PWA + 10 % domestic content + 10 % energy community)".to_string(),
        "IRC § 48E(g) Low-Income Communities Bonus Credit Amount Program — additional bonus allocation (capped national pool) for facilities located in LOW-INCOME COMMUNITIES; administered by IRS annual allocation procedures".to_string(),
        "IRC § 48E(d) Recapture — 5-year recapture period for any disposition or cessation of qualified use during the recapture period".to_string(),
        "OBBBA 2025 § 70513 Wind/Solar Termination (NEW § 48E(e)(4)) — § 48E credit ELIMINATED for APPLICABLE WIND FACILITIES and APPLICABLE SOLAR FACILITIES where (i) BEGINNING OF CONSTRUCTION (BOC) occurs AFTER JULY 4, 2026 (12-month anniversary of OBBBA enactment) OR (ii) facility placed in service after DECEMBER 31, 2027".to_string(),
        "OBBBA 2025 § 70513 Other Technologies Preserved — § 48E credit REMAINS AVAILABLE for non-wind/non-solar qualified facilities (geothermal, hydropower, marine and hydrokinetic, nuclear, fuel cell) AND for energy storage technology under the original IRA 2022 phase-out timeline (later of 2032 or year US power-sector emissions ≤ 25 % of 2022 baseline)".to_string(),
        "IRS Notice 2025-42 Beginning of Construction Guidance — IRS established PHYSICAL WORK TEST as the EXCLUSIVE method for establishing BOC before July 5, 2026 for wind/solar; previously-available 5 PERCENT SAFE HARBOR is NOT available for OBBBA wind/solar BOC anniversary cutoff (except for certain low-output solar facilities)".to_string(),
        "Final Regulations T.D. 10024 (January 15, 2025) — Treasury and IRS issued final regulations under § 45Y and § 48E published in the Federal Register; provide rules for greenhouse gas emissions rate determination, qualified facility components, recapture procedures, direct pay election, and transferability election".to_string(),
        "Form 3468 (Investment Credit) — required to claim the § 48E credit beginning with tax year 2025".to_string(),
        "IRA Tracker + IRS Federal Register + Cornell LII + Bloomberg Tax + IRS Notice 2025-42 + Mayer Brown + DLA Piper + Sidley Austin + Withum + The Tax Adviser + Ryan + RSM US + CESA + Build With Basis — practitioner overviews of § 48E".to_string(),
    ];

    if input.placed_in_service_date_status
        == PlacedInServiceDateStatus::PlacedInServiceOnOrBeforeDecember31_2024PreEffective
    {
        return Output {
            mode: Section48EMode::NotApplicablePlacedInServiceOnOrBeforeDecember31_2024PreEffective,
            statutory_basis: "IRA 2022 § 13702 effective date — § 48E applies only to qualified facilities and qualified energy storage technology placed in service after December 31, 2024".to_string(),
            notes: "NOT APPLICABLE: facility placed in service on or before December 31, 2024 (pre-effective date); § 48E credit unavailable; § 48 energy investment credit may apply instead for facilities placed in service before 2025.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.facility_technology_type
        == FacilityTechnologyType::NonQualifyingGreenhouseGasEmittingFacility
    {
        return Output {
            mode: Section48EMode::NotApplicableNonQualifyingGreenhouseGasEmittingFacility,
            statutory_basis: "IRC § 48E(b)(2) — qualified facility must have greenhouse gas emissions rate of NOT GREATER THAN ZERO".to_string(),
            notes: "NOT APPLICABLE: facility has positive greenhouse gas emissions rate; does not meet § 48E(b)(2) qualified facility definition; credit unavailable.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    let is_wind_or_solar = matches!(
        input.facility_technology_type,
        FacilityTechnologyType::AppliableWindFacility | FacilityTechnologyType::AppliableSolarFacility
    );

    if is_wind_or_solar
        && input.beginning_of_construction_status
            == BeginningOfConstructionStatus::BocAfterJuly4_2026PostObbbaAnniversary
    {
        return Output {
            mode: Section48EMode::NotApplicableWindOrSolarBocAfterJuly4_2026PostObbbaAnniversary,
            statutory_basis: "OBBBA 2025 § 70513 + new § 48E(e)(4) — wind/solar BOC anniversary cutoff; facility with BOC after July 4, 2026 ineligible".to_string(),
            notes: "NOT APPLICABLE: applicable wind or solar facility with beginning of construction AFTER July 4, 2026 (12-month anniversary of OBBBA enactment); § 48E credit TERMINATED by Section 70513 of One Big Beautiful Bill Act of 2025 (Public Law 119-21).".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if is_wind_or_solar
        && input.wind_solar_placed_in_service_status
            == WindSolarPlacedInServiceStatus::PlacedInServiceAfterDecember31_2027PostObbbaWindSolarCutoff
    {
        return Output {
            mode: Section48EMode::NotApplicableWindOrSolarPlacedInServiceAfterDecember31_2027PostObbbaCutoff,
            statutory_basis: "OBBBA 2025 § 70513 + new § 48E(e)(4) — wind/solar placed-in-service cutoff; facility placed in service after December 31, 2027 ineligible".to_string(),
            notes: "NOT APPLICABLE: applicable wind or solar facility placed in service after December 31, 2027; § 48E credit TERMINATED by Section 70513 of One Big Beautiful Bill Act of 2025 (Public Law 119-21).".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::BaseCreditAmountUnderSection48EA => {
            let computed = apply_rate(input.qualified_investment_dollars, IRC_48E_BASE_RATE_BPS);
            Output {
                mode: Section48EMode::CompliantBaseCreditAtSixPercentOfQualifiedInvestment,
                statutory_basis: "IRC § 48E(a) — base credit at 6 % of qualified investment".to_string(),
                notes: format!(
                    "COMPLIANT: base credit at 6 % × ${inv} qualified investment = ${computed}.",
                    inv = input.qualified_investment_dollars,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::BonusCreditAmountForPwaUnderSection48EA3A => {
            if input.claimed_pwa_bonus_rate
                && input.pwa_status
                    == PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly
            {
                return Output {
                    mode: Section48EMode::ViolationPwaBonusRateClaimedWithoutMeetingPwaRequirements,
                    statutory_basis: "IRC § 48E(a)(3)(A) — bonus 30 % rate requires prevailing wage and apprenticeship compliance".to_string(),
                    notes: "VIOLATION: bonus rate (30 %) claimed but prevailing wage and apprenticeship requirements not met; only 6 % base rate available.".to_string(),
                    citations,
                    computed_credit_dollars: apply_rate(
                        input.qualified_investment_dollars,
                        IRC_48E_BASE_RATE_BPS,
                    ),
                };
            }
            let computed = apply_rate(input.qualified_investment_dollars, IRC_48E_BONUS_RATE_BPS);
            Output {
                mode: Section48EMode::CompliantBonusCreditAtThirtyPercentPwaSatisfied,
                statutory_basis: "IRC § 48E(a)(3)(A) — bonus credit at 30 % (5x base rate) with prevailing wage and apprenticeship compliance".to_string(),
                notes: format!(
                    "COMPLIANT: bonus credit at 30 % × ${inv} qualified investment = ${computed}; PWA requirements satisfied through construction and 5-year recapture period.",
                    inv = input.qualified_investment_dollars,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::QualifiedFacilityDefinitionUnderSection48EB2 => Output {
            mode: Section48EMode::CompliantQualifiedFacilityPlacedInServiceAfterDecember31_2024,
            statutory_basis: "IRC § 48E(b)(2) — qualified facility placed in service after December 31, 2024 with zero or negative greenhouse gas emissions rate".to_string(),
            notes: "COMPLIANT: facility used for electricity generation, placed in service after December 31, 2024, and has greenhouse gas emissions rate of not greater than zero; qualified facility under § 48E(b)(2).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::QualifiedEnergyStorageTechnologyUnderSection48EC => {
            if input.facility_technology_type
                == FacilityTechnologyType::QualifiedEnergyStorageTechnology
            {
                Output {
                    mode: Section48EMode::CompliantQualifiedEnergyStorageTechnology,
                    statutory_basis: "IRC § 48E(c) — qualified energy storage technology (batteries / pumped hydro / thermal storage) separately eligible for § 48E investment credit".to_string(),
                    notes: "COMPLIANT: qualified energy storage technology under § 48E(c); credit available even though storage does not 'produce' electricity (cf. § 45Y PTC which excludes storage).".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section48EMode::CompliantQualifiedFacilityPlacedInServiceAfterDecember31_2024,
                    statutory_basis: "IRC § 48E(b)(2) — qualified electricity generation facility (not storage)".to_string(),
                    notes: "COMPLIANT: facility is qualified electricity generation facility under § 48E(b)(2); separate from § 48E(c) qualified energy storage technology category.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::EnergyCommunityAdderUnderSection48EA3C => {
            if input.claimed_energy_community_adder
                && input.energy_community_status
                    == EnergyCommunityAdderStatus::NotLocatedInEnergyCommunity
            {
                return Output {
                    mode: Section48EMode::ViolationEnergyCommunityAdderClaimedWithoutQualifyingLocation,
                    statutory_basis: "IRC § 48E(a)(3)(C) — energy community +10 PP adder requires facility located in qualifying energy community".to_string(),
                    notes: "VIOLATION: energy community +10 PP adder claimed but facility not located in qualifying energy community (brownfield / coal community / fossil fuel employment area); adder disallowed.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                };
            }
            let base_rate = match input.pwa_status {
                PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate => {
                    IRC_48E_BONUS_RATE_BPS
                }
                PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly => {
                    IRC_48E_BASE_RATE_BPS
                }
            };
            let total_rate = base_rate + IRC_48E_ENERGY_COMMUNITY_ADDER_BPS;
            let computed = apply_rate(input.qualified_investment_dollars, total_rate);
            Output {
                mode: Section48EMode::CompliantWithEnergyCommunityAdder,
                statutory_basis: "IRC § 48E(a)(3)(C) — +10 PP energy community adder applied to base/bonus rate".to_string(),
                notes: format!(
                    "COMPLIANT: energy community +10 PP adder applied; total rate {tr_pct} % × ${inv} = ${computed}.",
                    tr_pct = total_rate / 100,
                    inv = input.qualified_investment_dollars,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::DomesticContentAdderUnderSection48EA3B => {
            if input.claimed_domestic_content_adder
                && input.domestic_content_status
                    == DomesticContentAdderStatus::DomesticContentRequirementNotMet
            {
                return Output {
                    mode: Section48EMode::ViolationDomesticContentAdderClaimedWithoutMeetingRequirement,
                    statutory_basis: "IRC § 48E(a)(3)(B) — domestic content +10 PP adder requires meeting US-sourced steel, iron, and manufactured products thresholds".to_string(),
                    notes: "VIOLATION: domestic content +10 PP adder claimed but US-sourced steel/iron/manufactured-products threshold not met; adder disallowed.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                };
            }
            let base_rate = match input.pwa_status {
                PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate => {
                    IRC_48E_BONUS_RATE_BPS
                }
                PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly => {
                    IRC_48E_BASE_RATE_BPS
                }
            };
            let total_rate = base_rate + IRC_48E_DOMESTIC_CONTENT_ADDER_BPS;
            let computed = apply_rate(input.qualified_investment_dollars, total_rate);
            Output {
                mode: if input.claimed_energy_community_adder
                    && input.energy_community_status
                        == EnergyCommunityAdderStatus::LocatedInEnergyCommunity
                {
                    Section48EMode::CompliantWithBothBonusAddersStacked
                } else {
                    Section48EMode::CompliantWithDomesticContentAdder
                },
                statutory_basis: "IRC § 48E(a)(3)(B) — +10 PP domestic content adder applied to base/bonus rate".to_string(),
                notes: format!(
                    "COMPLIANT: domestic content +10 PP adder applied; total rate {tr_pct} % × ${inv} = ${computed}.",
                    tr_pct = total_rate / 100,
                    inv = input.qualified_investment_dollars,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::StackedMaximumCreditWithAllBonusAdders => {
            let computed = apply_rate(
                input.qualified_investment_dollars,
                IRC_48E_MAX_STACKED_RATE_BPS,
            );
            Output {
                mode: Section48EMode::CompliantMaximumStackedCreditAtFiftyPercent,
                statutory_basis: "IRC § 48E(a) + (a)(3)(A) + (a)(3)(B) + (a)(3)(C) — maximum stacked credit at 50 % (30 % PWA + 10 % domestic content + 10 % energy community)".to_string(),
                notes: format!(
                    "COMPLIANT: maximum stacked credit at 50 % × ${inv} qualified investment = ${computed}; PWA + domestic content + energy community all satisfied.",
                    inv = input.qualified_investment_dollars,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::RecaptureFiveYearPeriodUnderSection48ED => {
            if input.disposition_during_recapture_period
                && input.years_since_placed_in_service < IRC_48E_RECAPTURE_PERIOD_YEARS
            {
                Output {
                    mode: Section48EMode::ViolationRecaptureTriggeredByDisposition,
                    statutory_basis: "IRC § 48E(d) — disposition during 5-year recapture period triggers recapture".to_string(),
                    notes: format!(
                        "VIOLATION: disposition or cessation of qualified use at year {y} during 5-year recapture period triggers § 48E(d) recapture.",
                        y = input.years_since_placed_in_service,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section48EMode::CompliantRecapturePeriodSatisfied,
                    statutory_basis: "IRC § 48E(d) — recapture period satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT: recapture period at year {y} satisfied under § 48E(d) (no disposition or cessation during 5-year recapture period).",
                        y = input.years_since_placed_in_service,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::ObbbaWindSolarTerminationUnderSection70513 => {
            if is_wind_or_solar {
                Output {
                    mode: Section48EMode::CompliantQualifiedFacilityPlacedInServiceAfterDecember31_2024,
                    statutory_basis: "OBBBA 2025 § 70513 — wind/solar facility within pre-cutoff window remains eligible".to_string(),
                    notes: "COMPLIANT: applicable wind or solar facility with BOC on or before July 4, 2026 AND placed in service on or before December 31, 2027; § 48E credit available under OBBBA 2025 wind/solar transition window.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section48EMode::CompliantNonWindNonSolarFacilityUnaffectedByObbbaWindSolarTermination,
                    statutory_basis: "OBBBA 2025 § 70513 — non-wind/non-solar facilities AND energy storage preserved under original IRA 2022 phase-out timeline".to_string(),
                    notes: "COMPLIANT: non-wind/non-solar qualified facility (geothermal / hydropower / marine and hydrokinetic / nuclear / fuel cell) OR qualified energy storage technology UNAFFECTED by OBBBA 2025 wind/solar termination; remains eligible under original IRA 2022 phase-out (later of 2032 or year US power-sector emissions ≤ 25 % of 2022 baseline).".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::FormFilingUnderForm3468 => {
            if input.form_3468_filed_correctly {
                Output {
                    mode: Section48EMode::CompliantForm3468FiledCorrectly,
                    statutory_basis: "Form 3468 — Investment Credit form required to claim § 48E credit".to_string(),
                    notes: "COMPLIANT: Form 3468 filed correctly to claim § 48E credit for tax year.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section48EMode::ViolationForm3468NotFiledOrIncorrect,
                    statutory_basis: "Form 3468 filing required to claim § 48E credit".to_string(),
                    notes: "VIOLATION: Form 3468 not filed or incorrectly filed; § 48E credit may be disallowed.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
    }
}

fn apply_rate(qualified_investment_dollars: u64, rate_bps: u64) -> u64 {
    (u128::from(qualified_investment_dollars) * u128::from(rate_bps)
        / u128::from(IRC_48E_BASIS_POINT_DENOMINATOR)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            placed_in_service_date_status:
                PlacedInServiceDateStatus::PlacedInServiceAfterDecember31_2024PostEffectiveEligible,
            facility_technology_type: FacilityTechnologyType::NonWindNonSolarZeroEmissionFacility,
            beginning_of_construction_status:
                BeginningOfConstructionStatus::NotApplicableNonWindNonSolarOrStorageFacility,
            wind_solar_placed_in_service_status:
                WindSolarPlacedInServiceStatus::NotApplicableNonWindNonSolarOrStorageFacility,
            pwa_status:
                PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate,
            domestic_content_status: DomesticContentAdderStatus::DomesticContentRequirementMet,
            energy_community_status: EnergyCommunityAdderStatus::LocatedInEnergyCommunity,
            compliance_aspect: ComplianceAspect::BaseCreditAmountUnderSection48EA,
            qualified_investment_dollars: 1_000_000,
            years_since_placed_in_service: 6,
            disposition_during_recapture_period: false,
            form_3468_filed_correctly: true,
            claimed_pwa_bonus_rate: false,
            claimed_energy_community_adder: false,
            claimed_domestic_content_adder: false,
        }
    }

    #[test]
    fn pre_effective_placed_in_service_not_applicable() {
        let mut input = baseline_input();
        input.placed_in_service_date_status =
            PlacedInServiceDateStatus::PlacedInServiceOnOrBeforeDecember31_2024PreEffective;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::NotApplicablePlacedInServiceOnOrBeforeDecember31_2024PreEffective
        );
        assert_eq!(out.computed_credit_dollars, 0);
    }

    #[test]
    fn non_qualifying_ghg_emitting_facility_not_applicable() {
        let mut input = baseline_input();
        input.facility_technology_type =
            FacilityTechnologyType::NonQualifyingGreenhouseGasEmittingFacility;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::NotApplicableNonQualifyingGreenhouseGasEmittingFacility
        );
    }

    #[test]
    fn wind_facility_boc_after_july4_2026_not_applicable() {
        let mut input = baseline_input();
        input.facility_technology_type = FacilityTechnologyType::AppliableWindFacility;
        input.beginning_of_construction_status =
            BeginningOfConstructionStatus::BocAfterJuly4_2026PostObbbaAnniversary;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::NotApplicableWindOrSolarBocAfterJuly4_2026PostObbbaAnniversary
        );
    }

    #[test]
    fn solar_facility_boc_on_or_before_july4_2026_eligible() {
        let mut input = baseline_input();
        input.facility_technology_type = FacilityTechnologyType::AppliableSolarFacility;
        input.beginning_of_construction_status =
            BeginningOfConstructionStatus::BocOnOrBeforeJuly4_2026PreObbbaAnniversary;
        input.wind_solar_placed_in_service_status =
            WindSolarPlacedInServiceStatus::PlacedInServiceOnOrBeforeDecember31_2027PreObbbaWindSolarCutoff;
        let out = check(&input);
        assert_ne!(
            out.mode,
            Section48EMode::NotApplicableWindOrSolarBocAfterJuly4_2026PostObbbaAnniversary
        );
    }

    #[test]
    fn solar_facility_pis_after_december31_2027_not_applicable() {
        let mut input = baseline_input();
        input.facility_technology_type = FacilityTechnologyType::AppliableSolarFacility;
        input.beginning_of_construction_status =
            BeginningOfConstructionStatus::BocOnOrBeforeJuly4_2026PreObbbaAnniversary;
        input.wind_solar_placed_in_service_status =
            WindSolarPlacedInServiceStatus::PlacedInServiceAfterDecember31_2027PostObbbaWindSolarCutoff;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::NotApplicableWindOrSolarPlacedInServiceAfterDecember31_2027PostObbbaCutoff
        );
    }

    #[test]
    fn base_credit_at_six_percent_of_qualified_investment_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountUnderSection48EA;
        input.qualified_investment_dollars = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::CompliantBaseCreditAtSixPercentOfQualifiedInvestment
        );
        assert_eq!(out.computed_credit_dollars, 60_000);
    }

    #[test]
    fn bonus_credit_at_thirty_percent_pwa_satisfied() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection48EA3A;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate;
        input.claimed_pwa_bonus_rate = true;
        input.qualified_investment_dollars = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::CompliantBonusCreditAtThirtyPercentPwaSatisfied
        );
        assert_eq!(out.computed_credit_dollars, 300_000);
    }

    #[test]
    fn bonus_rate_is_five_times_base_rate() {
        let inv = 1_000_000;
        let base = apply_rate(inv, IRC_48E_BASE_RATE_BPS);
        let bonus = apply_rate(inv, IRC_48E_BONUS_RATE_BPS);
        assert_eq!(bonus, base * IRC_48E_BONUS_MULTIPLIER);
    }

    #[test]
    fn pwa_bonus_claimed_without_meeting_requirements_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection48EA3A;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly;
        input.claimed_pwa_bonus_rate = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::ViolationPwaBonusRateClaimedWithoutMeetingPwaRequirements
        );
    }

    #[test]
    fn qualified_facility_definition_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::QualifiedFacilityDefinitionUnderSection48EB2;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::CompliantQualifiedFacilityPlacedInServiceAfterDecember31_2024
        );
    }

    #[test]
    fn qualified_energy_storage_technology_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::QualifiedEnergyStorageTechnologyUnderSection48EC;
        input.facility_technology_type = FacilityTechnologyType::QualifiedEnergyStorageTechnology;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::CompliantQualifiedEnergyStorageTechnology
        );
    }

    #[test]
    fn energy_community_adder_applied_compliant_with_pwa() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EnergyCommunityAdderUnderSection48EA3C;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate;
        input.energy_community_status = EnergyCommunityAdderStatus::LocatedInEnergyCommunity;
        input.claimed_energy_community_adder = true;
        input.qualified_investment_dollars = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::CompliantWithEnergyCommunityAdder
        );
        assert_eq!(out.computed_credit_dollars, 400_000);
    }

    #[test]
    fn energy_community_adder_applied_compliant_without_pwa() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EnergyCommunityAdderUnderSection48EA3C;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly;
        input.energy_community_status = EnergyCommunityAdderStatus::LocatedInEnergyCommunity;
        input.claimed_energy_community_adder = true;
        input.qualified_investment_dollars = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::CompliantWithEnergyCommunityAdder
        );
        assert_eq!(out.computed_credit_dollars, 160_000);
    }

    #[test]
    fn energy_community_adder_claimed_without_qualifying_location_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EnergyCommunityAdderUnderSection48EA3C;
        input.energy_community_status = EnergyCommunityAdderStatus::NotLocatedInEnergyCommunity;
        input.claimed_energy_community_adder = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::ViolationEnergyCommunityAdderClaimedWithoutQualifyingLocation
        );
    }

    #[test]
    fn domestic_content_adder_applied_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DomesticContentAdderUnderSection48EA3B;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate;
        input.domestic_content_status = DomesticContentAdderStatus::DomesticContentRequirementMet;
        input.claimed_domestic_content_adder = true;
        input.qualified_investment_dollars = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::CompliantWithDomesticContentAdder
        );
        assert_eq!(out.computed_credit_dollars, 400_000);
    }

    #[test]
    fn domestic_content_adder_claimed_without_meeting_requirement_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DomesticContentAdderUnderSection48EA3B;
        input.domestic_content_status =
            DomesticContentAdderStatus::DomesticContentRequirementNotMet;
        input.claimed_domestic_content_adder = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::ViolationDomesticContentAdderClaimedWithoutMeetingRequirement
        );
    }

    #[test]
    fn both_adders_stacked_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DomesticContentAdderUnderSection48EA3B;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate;
        input.domestic_content_status = DomesticContentAdderStatus::DomesticContentRequirementMet;
        input.energy_community_status = EnergyCommunityAdderStatus::LocatedInEnergyCommunity;
        input.claimed_domestic_content_adder = true;
        input.claimed_energy_community_adder = true;
        let out = check(&input);
        assert_eq!(out.mode, Section48EMode::CompliantWithBothBonusAddersStacked);
    }

    #[test]
    fn maximum_stacked_credit_at_fifty_percent_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::StackedMaximumCreditWithAllBonusAdders;
        input.qualified_investment_dollars = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::CompliantMaximumStackedCreditAtFiftyPercent
        );
        assert_eq!(out.computed_credit_dollars, 500_000);
    }

    #[test]
    fn recapture_period_satisfied_after_five_years_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RecaptureFiveYearPeriodUnderSection48ED;
        input.years_since_placed_in_service = 5;
        input.disposition_during_recapture_period = false;
        let out = check(&input);
        assert_eq!(out.mode, Section48EMode::CompliantRecapturePeriodSatisfied);
    }

    #[test]
    fn recapture_triggered_by_disposition_during_recapture_period_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RecaptureFiveYearPeriodUnderSection48ED;
        input.years_since_placed_in_service = 3;
        input.disposition_during_recapture_period = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::ViolationRecaptureTriggeredByDisposition
        );
    }

    #[test]
    fn obbba_wind_solar_termination_non_wind_non_solar_facility_unaffected() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ObbbaWindSolarTerminationUnderSection70513;
        input.facility_technology_type = FacilityTechnologyType::NonWindNonSolarZeroEmissionFacility;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::CompliantNonWindNonSolarFacilityUnaffectedByObbbaWindSolarTermination
        );
    }

    #[test]
    fn obbba_wind_solar_termination_energy_storage_unaffected() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ObbbaWindSolarTerminationUnderSection70513;
        input.facility_technology_type = FacilityTechnologyType::QualifiedEnergyStorageTechnology;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section48EMode::CompliantNonWindNonSolarFacilityUnaffectedByObbbaWindSolarTermination
        );
    }

    #[test]
    fn form_3468_filed_correctly_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm3468;
        input.form_3468_filed_correctly = true;
        let out = check(&input);
        assert_eq!(out.mode, Section48EMode::CompliantForm3468FiledCorrectly);
    }

    #[test]
    fn form_3468_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm3468;
        input.form_3468_filed_correctly = false;
        let out = check(&input);
        assert_eq!(out.mode, Section48EMode::ViolationForm3468NotFiledOrIncorrect);
    }

    #[test]
    fn constants_pin_section_48e_legislative_phases_and_credit_structure() {
        assert_eq!(IRC_48E_IRA_ENACTMENT_DATE_YEAR, 2022);
        assert_eq!(IRC_48E_IRA_ENACTMENT_DATE_MONTH, 8);
        assert_eq!(IRC_48E_IRA_ENACTMENT_DATE_DAY, 16);
        assert_eq!(IRC_48E_IRA_PUBLIC_LAW_CONGRESS, 117);
        assert_eq!(IRC_48E_IRA_PUBLIC_LAW_ENACTMENT, 169);
        assert_eq!(IRC_48E_IRA_ENABLING_SECTION, 13702);
        assert_eq!(IRC_48E_EFFECTIVE_DATE_YEAR, 2025);
        assert_eq!(IRC_48E_EFFECTIVE_DATE_MONTH, 1);
        assert_eq!(IRC_48E_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(IRC_48E_OBBBA_ENACTMENT_DATE_YEAR, 2025);
        assert_eq!(IRC_48E_OBBBA_ENACTMENT_DATE_MONTH, 7);
        assert_eq!(IRC_48E_OBBBA_ENACTMENT_DATE_DAY, 4);
        assert_eq!(IRC_48E_OBBBA_PUBLIC_LAW_CONGRESS, 119);
        assert_eq!(IRC_48E_OBBBA_PUBLIC_LAW_ENACTMENT, 21);
        assert_eq!(IRC_48E_OBBBA_ENABLING_SECTION, 70513);
        assert_eq!(IRC_48E_OBBBA_WIND_SOLAR_BOC_CUTOFF_YEAR, 2026);
        assert_eq!(IRC_48E_OBBBA_WIND_SOLAR_BOC_CUTOFF_MONTH, 7);
        assert_eq!(IRC_48E_OBBBA_WIND_SOLAR_BOC_CUTOFF_DAY, 4);
        assert_eq!(IRC_48E_OBBBA_WIND_SOLAR_PIS_CUTOFF_YEAR, 2027);
        assert_eq!(IRC_48E_OBBBA_WIND_SOLAR_PIS_CUTOFF_MONTH, 12);
        assert_eq!(IRC_48E_OBBBA_WIND_SOLAR_PIS_CUTOFF_DAY, 31);
        assert_eq!(IRC_48E_ORIGINAL_PHASE_OUT_YEAR, 2032);
        assert_eq!(IRC_48E_BASE_RATE_BPS, 600);
        assert_eq!(IRC_48E_BONUS_RATE_BPS, 3_000);
        assert_eq!(IRC_48E_BONUS_MULTIPLIER, 5);
        assert_eq!(IRC_48E_DOMESTIC_CONTENT_ADDER_BPS, 1_000);
        assert_eq!(IRC_48E_ENERGY_COMMUNITY_ADDER_BPS, 1_000);
        assert_eq!(IRC_48E_MAX_STACKED_RATE_BPS, 5_000);
        assert_eq!(IRC_48E_RECAPTURE_PERIOD_YEARS, 5);
        assert_eq!(IRC_48E_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_48E_FORM_NUMBER, 3468);
        assert_eq!(IRC_48E_BOC_GUIDANCE_NOTICE_YEAR, 2025);
        assert_eq!(IRC_48E_BOC_GUIDANCE_NOTICE_NUMBER, 42);
    }

    #[test]
    fn citations_pin_legislative_phases_and_obbba_termination_facts() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("IRC § 48E Clean Electricity Investment Credit"));
        assert!(joined.contains("Section 13702 of the Inflation Reduction Act of 2022"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("August 16, 2022"));
        assert!(joined.contains("December 31, 2024"));
        assert!(joined.contains("6 PERCENT"));
        assert!(joined.contains("30 PERCENT"));
        assert!(joined.contains("MULTIPLIED BY 5"));
        assert!(joined.contains("GREENHOUSE GAS EMISSIONS RATE of NOT GREATER THAN ZERO"));
        assert!(joined.contains("ENERGY STORAGE TECHNOLOGY"));
        assert!(joined.contains("+10 PERCENTAGE POINTS"));
        assert!(joined.contains("ENERGY COMMUNITY"));
        assert!(joined.contains("DOMESTIC CONTENT"));
        assert!(joined.contains("50 PERCENT"));
        assert!(joined.contains("LOW-INCOME COMMUNITIES"));
        assert!(joined.contains("5-year recapture period"));
        assert!(joined.contains("OBBBA 2025 § 70513"));
        assert!(joined.contains("§ 48E(e)(4)"));
        assert!(joined.contains("JULY 4, 2026"));
        assert!(joined.contains("DECEMBER 31, 2027"));
        assert!(joined.contains("APPLICABLE WIND FACILITIES"));
        assert!(joined.contains("APPLICABLE SOLAR FACILITIES"));
        assert!(joined.contains("IRS Notice 2025-42"));
        assert!(joined.contains("PHYSICAL WORK TEST"));
        assert!(joined.contains("5 PERCENT SAFE HARBOR"));
        assert!(joined.contains("Final Regulations T.D. 10024"));
        assert!(joined.contains("January 15, 2025"));
        assert!(joined.contains("Form 3468"));
    }

    #[test]
    fn saturating_overflow_defense_at_u64_max_investment() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountUnderSection48EA;
        input.qualified_investment_dollars = u64::MAX;
        let out = check(&input);
        let _ = out.computed_credit_dollars;
    }
}
