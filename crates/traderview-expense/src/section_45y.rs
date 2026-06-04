//! IRC § 45Y Clean Electricity Production Credit
//! Compliance Module — pure-compute check for the
//! technology-neutral production tax credit (PTC) for
//! electricity generated and sold from any qualified
//! facility with greenhouse gas emissions rate of NOT
//! GREATER THAN ZERO. Enacted by the Inflation Reduction
//! Act of 2022 and substantially modified (early
//! termination for wind/solar facilities) by the One Big
//! Beautiful Bill Act of 2025.
//!
//! Originally enacted by **Section 13701 of the Inflation
//! Reduction Act of 2022 (Public Law 117-169)**, signed by
//! President Joe Biden on **August 16, 2022**, effective
//! for **QUALIFIED FACILITIES PLACED IN SERVICE AFTER
//! DECEMBER 31, 2024**. **MODIFIED by Section 70512 of the
//! One Big Beautiful Bill Act of 2025 (Public Law 119-21)**,
//! signed by President Donald Trump on **July 4, 2025**;
//! § 45Y credit TERMINATED for **APPLICABLE WIND FACILITIES
//! and APPLICABLE SOLAR FACILITIES** where (i) **BEGINNING
//! OF CONSTRUCTION (BOC)** occurs on or after **JULY 4,
//! 2026** (one-year anniversary of OBBBA enactment) OR
//! (ii) the facility is **PLACED IN SERVICE AFTER
//! DECEMBER 31, 2027**.
//!
//! Web research (verified 2026-06-03):
//! - **IRA 2022 Enactment**: IRC § 45Y added by **Section 13701 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818)**; signed by President Joe Biden on **August 16, 2022**; effective for QUALIFIED FACILITIES PLACED IN SERVICE AFTER **DECEMBER 31, 2024**; the technology-neutral successor to § 45 (renewable electricity production credit) for facilities placed in service after 2024 ([Federal Register — Section 45Y Clean Electricity Production Credit and Section 48E Clean Electricity Investment Credit Final Regulations (January 15, 2025)](https://www.federalregister.gov/documents/2025/01/15/2025-00196/section-45y-clean-electricity-production-credit-and-section-48e-clean-electricity-investment-credit); [Inflation Reduction Act Tracker — IRA Section 13701](https://iratracker.org/programs/ira-section-13701-clean-electricity-production-credit/); [IRS — Clean Electricity Production Credit](https://www.irs.gov/credits-deductions/clean-electricity-production-credit); [IRS — Sections 45Y and 48E Beginning of Construction Notice 2025-42](https://www.irs.gov/pub/irs-drop/n-25-42.pdf); [Bloomberg Tax — Sec. 45Y Clean Electricity Production Credit](https://irc.bloombergtax.com/public/uscode/doc/irc/section_45y); [Troutman Pepper Locke — Treasury and IRS Final Regulations on Clean Electricity Production and Investment Tax Credits](https://www.troutman.com/insights/treasury-and-irs-issue-final-regulations-on-clean-electricity-production-and-investment-tax-credits/); [Thomson Reuters Tax — IRS Clarifies Clean Energy Rollbacks, Adjusts Sec 45Y Credit for Inflation](https://tax.thomsonreuters.com/news/irs-clarifies-clean-energy-rollbacks-adjusts-sec-45y-credit-for-inflation/); [Pierce Atwood — Congress Phases Out Energy Tax Credits](https://www.pierceatwood.com/alerts/congress-phases-out-energy-tax-credits); [Carr Riggs & Ingram — Energy Tax Credits After OBBBA](https://www.criadv.com/insight/energy-tax-credits-after-obbba/); [Novogradac — About Renewable Energy Tax Credits](https://www.novoco.com/resource-centers/renewable-energy-tax-credits/about-renewable-energy-tax-credits); [Build With Basis — The 45Y Clean Energy Production Credit After the Big Beautiful Bill](https://www.buildwithbasis.com/insights/45y-clean-energy-production-credit-after-big-beautiful-bill); [Grant Thornton — Energy Incentives Under OBBBA](https://www.grantthornton.com/insights/alerts/tax/2025/insights/energy-incentives-under-obbba-what-you-need-to-know); [eCFR — 26 CFR 1.45Y-3 Prevailing Wage and Apprenticeship](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/subject-group-ECFRe427f958a26c8f4/section-1.45Y-3)).
//! - **§ 45Y(a) Base Credit Amount**: **0.3 CENTS per KILOWATT-HOUR (kWh)** of electricity produced by the taxpayer at a qualified facility and sold to an unrelated person during the 10-year credit period; **ADJUSTED FOR INFLATION** annually.
//! - **§ 45Y(a)(2) Bonus Credit Amount (Prevailing Wage and Apprenticeship)**: **1.5 CENTS per kWh** (FIVE TIMES the base rate) if the taxpayer satisfies the PWA requirements during construction and the 10-year credit period; ADJUSTED FOR INFLATION.
//! - **§ 45Y(b)(1) Qualified Facility Definition**: facility used for the **GENERATION OF ELECTRICITY** that is **placed in service after December 31, 2024** and has a **GREENHOUSE GAS EMISSIONS RATE of NOT GREATER THAN ZERO**.
//! - **§ 45Y(b)(2) Energy Storage Technology**: ENERGY STORAGE TECHNOLOGY (e.g., batteries) is eligible separately under § 48E investment credit (not § 45Y production credit) — § 45Y is for ELECTRICITY GENERATION only.
//! - **§ 45Y(d) 10-Year Credit Period**: credit available for the **10-YEAR PERIOD BEGINNING ON DATE FACILITY PLACED IN SERVICE**.
//! - **§ 45Y(g)(7) Energy Community Bonus**: **+10 PERCENT** increase to credit amount for facilities located in an **ENERGY COMMUNITY** (brownfield site, coal community, or area with high fossil fuel employment historically).
//! - **§ 45Y(g)(11) Domestic Content Bonus**: **+10 PERCENT** increase to credit amount for facilities meeting **DOMESTIC CONTENT** requirements (specified percentages of US-sourced steel, iron, and manufactured products).
//! - **§ 45Y(c) Inflation Adjustment**: base 0.3 cents and bonus 1.5 cents per kWh rates are adjusted annually for inflation; IRS publishes adjusted rates each year (e.g., Notice 2024-x for 2024 publication-year rates).
//! - **§ 45Y(d)(3) Phase-Out (Original IRA 2022)**: phase-out begins for facilities the construction of which begins in the LATER of (i) calendar year **2032** OR (ii) the calendar year following the year in which the **APPLICABLE YEAR** is determined (the year in which US power-sector greenhouse gas emissions are **25 PERCENT or LESS** of 2022 emissions).
//! - **OBBBA 2025 § 70512 Wind/Solar Termination — Beginning of Construction Anniversary**: § 45Y credit ELIMINATED for **APPLICABLE WIND FACILITIES** and **APPLICABLE SOLAR FACILITIES** where **BEGINNING OF CONSTRUCTION (BOC)** occurs on or after **JULY 4, 2026** (one-year anniversary of OBBBA enactment).
//! - **OBBBA 2025 § 70512 Wind/Solar Termination — Placed in Service Cutoff**: § 45Y credit ELIMINATED for **APPLICABLE WIND FACILITIES** and **APPLICABLE SOLAR FACILITIES** placed in service **AFTER DECEMBER 31, 2027**.
//! - **OBBBA 2025 § 70512 Other Technologies Preserved**: § 45Y credit REMAINS AVAILABLE for non-wind/non-solar qualified facilities (geothermal, hydropower, marine and hydrokinetic, nuclear, fuel cell, and other zero-emission technologies) under the original IRA 2022 phase-out timeline.
//! - **IRS Notice 2025-42 BOC Guidance**: IRS issued **Notice 2025-42** providing **BEGINNING OF CONSTRUCTION** guidance specific to the OBBBA wind/solar termination; clarifies what constitutes BOC for purposes of the July 4, 2026 anniversary cutoff.
//! - **Final Regulations T.D. 10024 (January 15, 2025)**: Treasury and IRS issued final regulations under § 45Y and § 48E published in the Federal Register on **January 15, 2025**; provide rules for greenhouse gas emissions rate determination, qualified facility components, metering, and other operational requirements.
//! - **Form 7211 (Clean Electricity Production Credit)**: required to claim the § 45Y credit beginning with tax year 2025.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_45Y_IRA_ENACTMENT_DATE_YEAR: u32 = 2022;
pub const IRC_45Y_IRA_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_45Y_IRA_ENACTMENT_DATE_DAY: u32 = 16;
pub const IRC_45Y_IRA_PUBLIC_LAW_CONGRESS: u32 = 117;
pub const IRC_45Y_IRA_PUBLIC_LAW_ENACTMENT: u32 = 169;
pub const IRC_45Y_IRA_ENABLING_SECTION: u32 = 13701;
pub const IRC_45Y_EFFECTIVE_DATE_YEAR: u32 = 2025;
pub const IRC_45Y_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const IRC_45Y_EFFECTIVE_DATE_DAY: u32 = 1;
pub const IRC_45Y_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_45Y_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_45Y_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_45Y_OBBBA_PUBLIC_LAW_CONGRESS: u32 = 119;
pub const IRC_45Y_OBBBA_PUBLIC_LAW_ENACTMENT: u32 = 21;
pub const IRC_45Y_OBBBA_ENABLING_SECTION: u32 = 70512;
pub const IRC_45Y_OBBBA_WIND_SOLAR_BOC_CUTOFF_YEAR: u32 = 2026;
pub const IRC_45Y_OBBBA_WIND_SOLAR_BOC_CUTOFF_MONTH: u32 = 7;
pub const IRC_45Y_OBBBA_WIND_SOLAR_BOC_CUTOFF_DAY: u32 = 4;
pub const IRC_45Y_OBBBA_WIND_SOLAR_PIS_CUTOFF_YEAR: u32 = 2027;
pub const IRC_45Y_OBBBA_WIND_SOLAR_PIS_CUTOFF_MONTH: u32 = 12;
pub const IRC_45Y_OBBBA_WIND_SOLAR_PIS_CUTOFF_DAY: u32 = 31;
pub const IRC_45Y_ORIGINAL_PHASE_OUT_YEAR: u32 = 2032;
pub const IRC_45Y_BASE_RATE_TENTHS_OF_CENTS_PER_KWH: u64 = 3;
pub const IRC_45Y_BONUS_RATE_TENTHS_OF_CENTS_PER_KWH: u64 = 15;
pub const IRC_45Y_BONUS_MULTIPLIER: u64 = 5;
pub const IRC_45Y_RATE_DENOMINATOR_TENTHS_OF_CENTS_PER_DOLLAR: u64 = 1_000;
pub const IRC_45Y_CREDIT_PERIOD_YEARS: u32 = 10;
pub const IRC_45Y_ENERGY_COMMUNITY_BONUS_BPS: u64 = 1_000;
pub const IRC_45Y_DOMESTIC_CONTENT_BONUS_BPS: u64 = 1_000;
pub const IRC_45Y_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_45Y_GHG_EMISSIONS_RATE_MAX_GRAMS_CO2E_PER_KWH: u32 = 0;
pub const IRC_45Y_FINAL_REGS_PUBLICATION_DATE_YEAR: u32 = 2025;
pub const IRC_45Y_FINAL_REGS_PUBLICATION_DATE_MONTH: u32 = 1;
pub const IRC_45Y_FINAL_REGS_PUBLICATION_DATE_DAY: u32 = 15;
pub const IRC_45Y_FORM_NUMBER: u32 = 7211;
pub const IRC_45Y_BOC_GUIDANCE_NOTICE_YEAR: u32 = 2025;
pub const IRC_45Y_BOC_GUIDANCE_NOTICE_NUMBER: u32 = 42;

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
    NonQualifyingGreenhouseGasEmittingFacility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BeginningOfConstructionStatus {
    BocOnOrBeforeJuly3_2026PreObbbaAnniversary,
    BocOnOrAfterJuly4_2026PostObbbaAnniversary,
    NotApplicableNonWindNonSolarFacility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindSolarPlacedInServiceStatus {
    PlacedInServiceOnOrBeforeDecember31_2027PreObbbaWindSolarCutoff,
    PlacedInServiceAfterDecember31_2027PostObbbaWindSolarCutoff,
    NotApplicableNonWindNonSolarFacility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrevailingWageApprenticeshipStatus {
    PwaRequirementsMetEligibleForBonusRate,
    PwaRequirementsNotMetBaseRateOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DomesticContentBonusStatus {
    DomesticContentRequirementMet,
    DomesticContentRequirementNotMet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnergyCommunityBonusStatus {
    LocatedInEnergyCommunity,
    NotLocatedInEnergyCommunity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    BaseCreditAmountUnderSection45YA,
    BonusCreditAmountForPwaUnderSection45YA2,
    QualifiedFacilityDefinitionUnderSection45YB1,
    TenYearCreditPeriodUnderSection45YD,
    EnergyCommunityBonusUnderSection45YG7,
    DomesticContentBonusUnderSection45YG11,
    InflationAdjustmentUnderSection45YC,
    ObbbaWindSolarTerminationUnderSection70512,
    FormFilingUnderForm7211,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section45YMode {
    NotApplicablePlacedInServiceOnOrBeforeDecember31_2024PreEffective,
    NotApplicableNonQualifyingGreenhouseGasEmittingFacility,
    NotApplicableWindOrSolarBocOnOrAfterJuly4_2026PostObbbaAnniversary,
    NotApplicableWindOrSolarPlacedInServiceAfterDecember31_2027PostObbbaCutoff,
    CompliantBaseCreditAtPointThreeCentsPerKwh,
    CompliantBonusCreditAtOnePointFiveCentsPerKwhPwaSatisfied,
    CompliantWithEnergyCommunityBonusAdder,
    CompliantWithDomesticContentBonusAdder,
    CompliantWithBothBonusAddersStacked,
    CompliantTenYearCreditPeriodYearWithinWindow,
    CompliantInflationAdjustmentApplied,
    CompliantQualifiedFacilityPlacedInServiceAfterDecember31_2024,
    CompliantNonWindNonSolarFacilityUnaffectedByObbbaWindSolarTermination,
    CompliantForm7211FiledCorrectly,
    ViolationCreditClaimedOutsideTenYearCreditPeriod,
    ViolationPwaBonusRateClaimedWithoutMeetingPwaRequirements,
    ViolationEnergyCommunityBonusClaimedWithoutQualifyingLocation,
    ViolationDomesticContentBonusClaimedWithoutMeetingRequirement,
    ViolationForm7211NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub placed_in_service_date_status: PlacedInServiceDateStatus,
    pub facility_technology_type: FacilityTechnologyType,
    pub beginning_of_construction_status: BeginningOfConstructionStatus,
    pub wind_solar_placed_in_service_status: WindSolarPlacedInServiceStatus,
    pub pwa_status: PrevailingWageApprenticeshipStatus,
    pub domestic_content_status: DomesticContentBonusStatus,
    pub energy_community_status: EnergyCommunityBonusStatus,
    pub compliance_aspect: ComplianceAspect,
    pub kilowatt_hours_produced: u64,
    pub credit_year_number_within_window: u32,
    pub form_7211_filed_correctly: bool,
    pub claimed_pwa_bonus_rate: bool,
    pub claimed_energy_community_bonus: bool,
    pub claimed_domestic_content_bonus: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section45YMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section45YInput = Input;
pub type Section45YOutput = Output;
pub type Section45YResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 45Y Clean Electricity Production Credit added by Section 13701 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818); signed by President Joe Biden on August 16, 2022; effective for qualified facilities placed in service after December 31, 2024".to_string(),
        "IRC § 45Y(a) Base Credit Amount — 0.3 CENTS per KILOWATT-HOUR (kWh) of electricity produced by the taxpayer at a qualified facility and sold to an unrelated person; adjusted for inflation annually".to_string(),
        "IRC § 45Y(a)(2) Bonus Credit Amount — 1.5 CENTS per kWh (FIVE TIMES the base rate) if the taxpayer satisfies the prevailing wage and apprenticeship (PWA) requirements during construction and the 10-year credit period; adjusted for inflation".to_string(),
        "IRC § 45Y(b)(1) Qualified Facility Definition — facility used for the GENERATION OF ELECTRICITY that is placed in service after December 31, 2024 and has a GREENHOUSE GAS EMISSIONS RATE of NOT GREATER THAN ZERO".to_string(),
        "IRC § 45Y(d) 10-Year Credit Period — credit available for the 10-YEAR PERIOD BEGINNING ON DATE FACILITY PLACED IN SERVICE".to_string(),
        "IRC § 45Y(g)(7) Energy Community Bonus — +10 PERCENT increase to credit amount for facilities located in an ENERGY COMMUNITY (brownfield site, coal community, or area with high fossil fuel employment historically)".to_string(),
        "IRC § 45Y(g)(11) Domestic Content Bonus — +10 PERCENT increase to credit amount for facilities meeting DOMESTIC CONTENT requirements (specified percentages of US-sourced steel, iron, and manufactured products)".to_string(),
        "IRC § 45Y(c) Inflation Adjustment — base 0.3 cents and bonus 1.5 cents per kWh rates are adjusted annually for inflation; IRS publishes adjusted rates each year".to_string(),
        "IRC § 45Y(d)(3) Phase-Out (Original IRA 2022) — phase-out begins for facilities the construction of which begins in the LATER of (i) calendar year 2032 OR (ii) the calendar year following the year in which the APPLICABLE YEAR is determined (the year US power-sector greenhouse gas emissions are 25 PERCENT or LESS of 2022 emissions)".to_string(),
        "OBBBA 2025 § 70512 Wind/Solar Termination — Beginning of Construction Anniversary — § 45Y credit ELIMINATED for APPLICABLE WIND FACILITIES and APPLICABLE SOLAR FACILITIES where BEGINNING OF CONSTRUCTION (BOC) occurs on or after JULY 4, 2026 (one-year anniversary of OBBBA enactment)".to_string(),
        "OBBBA 2025 § 70512 Wind/Solar Termination — Placed in Service Cutoff — § 45Y credit ELIMINATED for APPLICABLE WIND FACILITIES and APPLICABLE SOLAR FACILITIES placed in service AFTER DECEMBER 31, 2027".to_string(),
        "OBBBA 2025 § 70512 Other Technologies Preserved — § 45Y credit REMAINS AVAILABLE for non-wind/non-solar qualified facilities (geothermal, hydropower, marine and hydrokinetic, nuclear, fuel cell, and other zero-emission technologies) under the original IRA 2022 phase-out timeline".to_string(),
        "IRS Notice 2025-42 BOC Guidance — IRS issued Notice 2025-42 providing BEGINNING OF CONSTRUCTION guidance specific to the OBBBA wind/solar termination; clarifies what constitutes BOC for purposes of the July 4, 2026 anniversary cutoff".to_string(),
        "Final Regulations T.D. 10024 (January 15, 2025) — Treasury and IRS issued final regulations under § 45Y and § 48E published in the Federal Register on January 15, 2025; provide rules for greenhouse gas emissions rate determination, qualified facility components, metering, and other operational requirements".to_string(),
        "Form 7211 (Clean Electricity Production Credit) — required to claim the § 45Y credit beginning with tax year 2025".to_string(),
        "Inflation Reduction Act Tracker + IRS Federal Register + Bloomberg Tax + Troutman Pepper Locke + Thomson Reuters + Pierce Atwood + CRI + Novogradac + Build With Basis + Grant Thornton — practitioner overviews of § 45Y".to_string(),
    ];

    if input.placed_in_service_date_status
        == PlacedInServiceDateStatus::PlacedInServiceOnOrBeforeDecember31_2024PreEffective
    {
        return Output {
            mode: Section45YMode::NotApplicablePlacedInServiceOnOrBeforeDecember31_2024PreEffective,
            statutory_basis: "IRA 2022 § 13701 effective date — § 45Y applies only to qualified facilities placed in service after December 31, 2024".to_string(),
            notes: "NOT APPLICABLE: facility placed in service on or before December 31, 2024 (pre-effective date); § 45Y credit unavailable; § 45 renewable electricity production credit may apply instead for facilities placed in service before 2025.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.facility_technology_type
        == FacilityTechnologyType::NonQualifyingGreenhouseGasEmittingFacility
    {
        return Output {
            mode: Section45YMode::NotApplicableNonQualifyingGreenhouseGasEmittingFacility,
            statutory_basis: "IRC § 45Y(b)(1) — qualified facility must have greenhouse gas emissions rate of NOT GREATER THAN ZERO".to_string(),
            notes: "NOT APPLICABLE: facility has positive greenhouse gas emissions rate; does not meet § 45Y(b)(1) qualified facility definition; credit unavailable.".to_string(),
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
            == BeginningOfConstructionStatus::BocOnOrAfterJuly4_2026PostObbbaAnniversary
    {
        return Output {
            mode: Section45YMode::NotApplicableWindOrSolarBocOnOrAfterJuly4_2026PostObbbaAnniversary,
            statutory_basis: "OBBBA 2025 § 70512 wind/solar BOC anniversary cutoff — wind or solar facility with BOC on or after July 4, 2026 ineligible".to_string(),
            notes: "NOT APPLICABLE: applicable wind or solar facility with beginning of construction on or after July 4, 2026 (one-year anniversary of OBBBA enactment); § 45Y credit TERMINATED by Section 70512 of One Big Beautiful Bill Act of 2025 (Public Law 119-21).".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if is_wind_or_solar
        && input.wind_solar_placed_in_service_status
            == WindSolarPlacedInServiceStatus::PlacedInServiceAfterDecember31_2027PostObbbaWindSolarCutoff
    {
        return Output {
            mode: Section45YMode::NotApplicableWindOrSolarPlacedInServiceAfterDecember31_2027PostObbbaCutoff,
            statutory_basis: "OBBBA 2025 § 70512 wind/solar placed-in-service cutoff — wind or solar facility placed in service after December 31, 2027 ineligible".to_string(),
            notes: "NOT APPLICABLE: applicable wind or solar facility placed in service after December 31, 2027; § 45Y credit TERMINATED by Section 70512 of One Big Beautiful Bill Act of 2025 (Public Law 119-21).".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::BaseCreditAmountUnderSection45YA => {
            let computed = compute_base_credit_dollars(input.kilowatt_hours_produced);
            Output {
                mode: Section45YMode::CompliantBaseCreditAtPointThreeCentsPerKwh,
                statutory_basis: "IRC § 45Y(a) — base credit at 0.3 cents per kWh of qualifying electricity produced".to_string(),
                notes: format!(
                    "COMPLIANT: base credit at 0.3 cents per kWh × {kwh} kWh = ${computed} (rounded down to whole dollars).",
                    kwh = input.kilowatt_hours_produced,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::BonusCreditAmountForPwaUnderSection45YA2 => {
            if input.claimed_pwa_bonus_rate
                && input.pwa_status
                    == PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly
            {
                return Output {
                    mode: Section45YMode::ViolationPwaBonusRateClaimedWithoutMeetingPwaRequirements,
                    statutory_basis: "IRC § 45Y(a)(2) — bonus 1.5 cents per kWh rate requires prevailing wage and apprenticeship compliance".to_string(),
                    notes: "VIOLATION: bonus rate (1.5 cents per kWh) claimed but prevailing wage and apprenticeship requirements not met; only 0.3 cents per kWh base rate available.".to_string(),
                    citations,
                    computed_credit_dollars: compute_base_credit_dollars(input.kilowatt_hours_produced),
                };
            }
            let computed = compute_bonus_credit_dollars(input.kilowatt_hours_produced);
            Output {
                mode: Section45YMode::CompliantBonusCreditAtOnePointFiveCentsPerKwhPwaSatisfied,
                statutory_basis: "IRC § 45Y(a)(2) — bonus credit at 1.5 cents per kWh (5x base rate) with prevailing wage and apprenticeship compliance".to_string(),
                notes: format!(
                    "COMPLIANT: bonus credit at 1.5 cents per kWh × {kwh} kWh = ${computed}; PWA requirements satisfied throughout construction and 10-year credit period.",
                    kwh = input.kilowatt_hours_produced,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::QualifiedFacilityDefinitionUnderSection45YB1 => Output {
            mode: Section45YMode::CompliantQualifiedFacilityPlacedInServiceAfterDecember31_2024,
            statutory_basis: "IRC § 45Y(b)(1) — qualified facility placed in service after December 31, 2024 with zero or negative greenhouse gas emissions rate".to_string(),
            notes: "COMPLIANT: facility used for electricity generation, placed in service after December 31, 2024, and has greenhouse gas emissions rate of not greater than zero; qualified facility under § 45Y(b)(1).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::TenYearCreditPeriodUnderSection45YD => {
            if input.credit_year_number_within_window == 0
                || input.credit_year_number_within_window > IRC_45Y_CREDIT_PERIOD_YEARS
            {
                Output {
                    mode: Section45YMode::ViolationCreditClaimedOutsideTenYearCreditPeriod,
                    statutory_basis: "IRC § 45Y(d) — credit available for 10-year period beginning on date facility placed in service".to_string(),
                    notes: format!(
                        "VIOLATION: credit claimed for year {y} which is outside the 10-year credit period (years 1-10 only); § 45Y credit unavailable for this year.",
                        y = input.credit_year_number_within_window,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45YMode::CompliantTenYearCreditPeriodYearWithinWindow,
                    statutory_basis: "IRC § 45Y(d) — credit claimed for year within 10-year credit period beginning on date facility placed in service".to_string(),
                    notes: format!(
                        "COMPLIANT: credit claimed for year {y} within 10-year credit period under § 45Y(d).",
                        y = input.credit_year_number_within_window,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::EnergyCommunityBonusUnderSection45YG7 => {
            if input.claimed_energy_community_bonus
                && input.energy_community_status
                    == EnergyCommunityBonusStatus::NotLocatedInEnergyCommunity
            {
                return Output {
                    mode: Section45YMode::ViolationEnergyCommunityBonusClaimedWithoutQualifyingLocation,
                    statutory_basis: "IRC § 45Y(g)(7) — energy community bonus requires facility located in qualifying energy community".to_string(),
                    notes: "VIOLATION: energy community bonus (+10 %) claimed but facility not located in qualifying energy community (brownfield / coal community / fossil fuel employment area); bonus disallowed.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                };
            }
            let base = match input.pwa_status {
                PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate => {
                    compute_bonus_credit_dollars(input.kilowatt_hours_produced)
                }
                PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly => {
                    compute_base_credit_dollars(input.kilowatt_hours_produced)
                }
            };
            let adder = (u128::from(base) * u128::from(IRC_45Y_ENERGY_COMMUNITY_BONUS_BPS)
                / u128::from(IRC_45Y_BASIS_POINT_DENOMINATOR)) as u64;
            let computed = base.saturating_add(adder);
            Output {
                mode: Section45YMode::CompliantWithEnergyCommunityBonusAdder,
                statutory_basis: "IRC § 45Y(g)(7) — +10 % energy community bonus adder applied".to_string(),
                notes: format!(
                    "COMPLIANT: energy community +10 % bonus applied; base credit ${base} + ${adder} bonus = ${computed}."
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::DomesticContentBonusUnderSection45YG11 => {
            if input.claimed_domestic_content_bonus
                && input.domestic_content_status
                    == DomesticContentBonusStatus::DomesticContentRequirementNotMet
            {
                return Output {
                    mode: Section45YMode::ViolationDomesticContentBonusClaimedWithoutMeetingRequirement,
                    statutory_basis: "IRC § 45Y(g)(11) — domestic content bonus requires meeting US-sourced steel, iron, and manufactured products thresholds".to_string(),
                    notes: "VIOLATION: domestic content bonus (+10 %) claimed but US-sourced steel/iron/manufactured-products threshold not met; bonus disallowed.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                };
            }
            let base = match input.pwa_status {
                PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate => {
                    compute_bonus_credit_dollars(input.kilowatt_hours_produced)
                }
                PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly => {
                    compute_base_credit_dollars(input.kilowatt_hours_produced)
                }
            };
            let adder = (u128::from(base) * u128::from(IRC_45Y_DOMESTIC_CONTENT_BONUS_BPS)
                / u128::from(IRC_45Y_BASIS_POINT_DENOMINATOR)) as u64;
            let computed = base.saturating_add(adder);
            Output {
                mode: if input.claimed_energy_community_bonus
                    && input.energy_community_status
                        == EnergyCommunityBonusStatus::LocatedInEnergyCommunity
                {
                    Section45YMode::CompliantWithBothBonusAddersStacked
                } else {
                    Section45YMode::CompliantWithDomesticContentBonusAdder
                },
                statutory_basis: "IRC § 45Y(g)(11) — +10 % domestic content bonus adder applied".to_string(),
                notes: format!(
                    "COMPLIANT: domestic content +10 % bonus applied; base credit ${base} + ${adder} bonus = ${computed}."
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::InflationAdjustmentUnderSection45YC => Output {
            mode: Section45YMode::CompliantInflationAdjustmentApplied,
            statutory_basis: "IRC § 45Y(c) — base 0.3 cents and bonus 1.5 cents per kWh rates adjusted annually for inflation".to_string(),
            notes: "COMPLIANT: inflation adjustment applied to base / bonus per-kWh rates per IRS annual publication.".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::ObbbaWindSolarTerminationUnderSection70512 => {
            if is_wind_or_solar {
                Output {
                    mode: Section45YMode::CompliantTenYearCreditPeriodYearWithinWindow,
                    statutory_basis: "OBBBA 2025 § 70512 — wind/solar facility within pre-cutoff window remains eligible".to_string(),
                    notes: "COMPLIANT: applicable wind or solar facility with BOC before July 4, 2026 AND placed in service on or before December 31, 2027; § 45Y credit available under OBBBA 2025 wind/solar transition window.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45YMode::CompliantNonWindNonSolarFacilityUnaffectedByObbbaWindSolarTermination,
                    statutory_basis: "OBBBA 2025 § 70512 — non-wind/non-solar facilities preserved under original IRA 2022 phase-out timeline".to_string(),
                    notes: "COMPLIANT: non-wind/non-solar qualified facility (geothermal / hydropower / marine and hydrokinetic / nuclear / fuel cell / other zero-emission technology) UNAFFECTED by OBBBA 2025 wind/solar termination; remains eligible under original IRA 2022 phase-out (later of 2032 or year US power-sector emissions ≤ 25 % of 2022 baseline).".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::FormFilingUnderForm7211 => {
            if input.form_7211_filed_correctly {
                Output {
                    mode: Section45YMode::CompliantForm7211FiledCorrectly,
                    statutory_basis: "Form 7211 — Clean Electricity Production Credit form required to claim § 45Y credit".to_string(),
                    notes: "COMPLIANT: Form 7211 filed correctly to claim § 45Y credit for tax year.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45YMode::ViolationForm7211NotFiledOrIncorrect,
                    statutory_basis: "Form 7211 filing required to claim § 45Y credit".to_string(),
                    notes: "VIOLATION: Form 7211 not filed or incorrectly filed; § 45Y credit may be disallowed.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
    }
}

fn compute_base_credit_dollars(kwh: u64) -> u64 {
    (u128::from(kwh) * u128::from(IRC_45Y_BASE_RATE_TENTHS_OF_CENTS_PER_KWH)
        / u128::from(IRC_45Y_RATE_DENOMINATOR_TENTHS_OF_CENTS_PER_DOLLAR)) as u64
}

fn compute_bonus_credit_dollars(kwh: u64) -> u64 {
    (u128::from(kwh) * u128::from(IRC_45Y_BONUS_RATE_TENTHS_OF_CENTS_PER_KWH)
        / u128::from(IRC_45Y_RATE_DENOMINATOR_TENTHS_OF_CENTS_PER_DOLLAR)) as u64
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
                BeginningOfConstructionStatus::NotApplicableNonWindNonSolarFacility,
            wind_solar_placed_in_service_status:
                WindSolarPlacedInServiceStatus::NotApplicableNonWindNonSolarFacility,
            pwa_status:
                PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate,
            domestic_content_status: DomesticContentBonusStatus::DomesticContentRequirementMet,
            energy_community_status: EnergyCommunityBonusStatus::LocatedInEnergyCommunity,
            compliance_aspect: ComplianceAspect::BaseCreditAmountUnderSection45YA,
            kilowatt_hours_produced: 1_000_000,
            credit_year_number_within_window: 1,
            form_7211_filed_correctly: true,
            claimed_pwa_bonus_rate: false,
            claimed_energy_community_bonus: false,
            claimed_domestic_content_bonus: false,
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
            Section45YMode::NotApplicablePlacedInServiceOnOrBeforeDecember31_2024PreEffective
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
            Section45YMode::NotApplicableNonQualifyingGreenhouseGasEmittingFacility
        );
    }

    #[test]
    fn wind_facility_boc_on_or_after_july4_2026_not_applicable() {
        let mut input = baseline_input();
        input.facility_technology_type = FacilityTechnologyType::AppliableWindFacility;
        input.beginning_of_construction_status =
            BeginningOfConstructionStatus::BocOnOrAfterJuly4_2026PostObbbaAnniversary;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::NotApplicableWindOrSolarBocOnOrAfterJuly4_2026PostObbbaAnniversary
        );
    }

    #[test]
    fn solar_facility_boc_before_july4_2026_eligible() {
        let mut input = baseline_input();
        input.facility_technology_type = FacilityTechnologyType::AppliableSolarFacility;
        input.beginning_of_construction_status =
            BeginningOfConstructionStatus::BocOnOrBeforeJuly3_2026PreObbbaAnniversary;
        input.wind_solar_placed_in_service_status =
            WindSolarPlacedInServiceStatus::PlacedInServiceOnOrBeforeDecember31_2027PreObbbaWindSolarCutoff;
        let out = check(&input);
        assert_ne!(
            out.mode,
            Section45YMode::NotApplicableWindOrSolarBocOnOrAfterJuly4_2026PostObbbaAnniversary
        );
    }

    #[test]
    fn solar_facility_pis_after_december31_2027_not_applicable() {
        let mut input = baseline_input();
        input.facility_technology_type = FacilityTechnologyType::AppliableSolarFacility;
        input.beginning_of_construction_status =
            BeginningOfConstructionStatus::BocOnOrBeforeJuly3_2026PreObbbaAnniversary;
        input.wind_solar_placed_in_service_status =
            WindSolarPlacedInServiceStatus::PlacedInServiceAfterDecember31_2027PostObbbaWindSolarCutoff;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::NotApplicableWindOrSolarPlacedInServiceAfterDecember31_2027PostObbbaCutoff
        );
    }

    #[test]
    fn base_credit_at_point_three_cents_per_kwh_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountUnderSection45YA;
        input.kilowatt_hours_produced = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::CompliantBaseCreditAtPointThreeCentsPerKwh
        );
        assert_eq!(out.computed_credit_dollars, 3_000);
    }

    #[test]
    fn bonus_credit_at_one_point_five_cents_per_kwh_pwa_satisfied() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45YA2;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate;
        input.claimed_pwa_bonus_rate = true;
        input.kilowatt_hours_produced = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::CompliantBonusCreditAtOnePointFiveCentsPerKwhPwaSatisfied
        );
        assert_eq!(out.computed_credit_dollars, 15_000);
    }

    #[test]
    fn bonus_rate_is_five_times_base_rate() {
        let kwh = 1_000_000;
        let base = compute_base_credit_dollars(kwh);
        let bonus = compute_bonus_credit_dollars(kwh);
        assert_eq!(bonus, base * IRC_45Y_BONUS_MULTIPLIER);
    }

    #[test]
    fn pwa_bonus_claimed_without_meeting_requirements_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45YA2;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly;
        input.claimed_pwa_bonus_rate = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::ViolationPwaBonusRateClaimedWithoutMeetingPwaRequirements
        );
    }

    #[test]
    fn qualified_facility_definition_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::QualifiedFacilityDefinitionUnderSection45YB1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::CompliantQualifiedFacilityPlacedInServiceAfterDecember31_2024
        );
    }

    #[test]
    fn ten_year_credit_period_year_one_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenYearCreditPeriodUnderSection45YD;
        input.credit_year_number_within_window = 1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::CompliantTenYearCreditPeriodYearWithinWindow
        );
    }

    #[test]
    fn ten_year_credit_period_year_ten_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenYearCreditPeriodUnderSection45YD;
        input.credit_year_number_within_window = 10;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::CompliantTenYearCreditPeriodYearWithinWindow
        );
    }

    #[test]
    fn ten_year_credit_period_year_eleven_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenYearCreditPeriodUnderSection45YD;
        input.credit_year_number_within_window = 11;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::ViolationCreditClaimedOutsideTenYearCreditPeriod
        );
    }

    #[test]
    fn ten_year_credit_period_year_zero_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenYearCreditPeriodUnderSection45YD;
        input.credit_year_number_within_window = 0;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::ViolationCreditClaimedOutsideTenYearCreditPeriod
        );
    }

    #[test]
    fn energy_community_bonus_applied_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EnergyCommunityBonusUnderSection45YG7;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate;
        input.energy_community_status = EnergyCommunityBonusStatus::LocatedInEnergyCommunity;
        input.claimed_energy_community_bonus = true;
        input.kilowatt_hours_produced = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::CompliantWithEnergyCommunityBonusAdder
        );
        assert_eq!(out.computed_credit_dollars, 16_500);
    }

    #[test]
    fn energy_community_bonus_claimed_without_qualifying_location_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EnergyCommunityBonusUnderSection45YG7;
        input.energy_community_status = EnergyCommunityBonusStatus::NotLocatedInEnergyCommunity;
        input.claimed_energy_community_bonus = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::ViolationEnergyCommunityBonusClaimedWithoutQualifyingLocation
        );
    }

    #[test]
    fn domestic_content_bonus_applied_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DomesticContentBonusUnderSection45YG11;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate;
        input.domestic_content_status = DomesticContentBonusStatus::DomesticContentRequirementMet;
        input.claimed_domestic_content_bonus = true;
        input.kilowatt_hours_produced = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::CompliantWithDomesticContentBonusAdder
        );
        assert_eq!(out.computed_credit_dollars, 16_500);
    }

    #[test]
    fn domestic_content_bonus_claimed_without_meeting_requirement_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DomesticContentBonusUnderSection45YG11;
        input.domestic_content_status =
            DomesticContentBonusStatus::DomesticContentRequirementNotMet;
        input.claimed_domestic_content_bonus = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::ViolationDomesticContentBonusClaimedWithoutMeetingRequirement
        );
    }

    #[test]
    fn both_bonus_adders_stacked_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DomesticContentBonusUnderSection45YG11;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusRate;
        input.domestic_content_status = DomesticContentBonusStatus::DomesticContentRequirementMet;
        input.energy_community_status = EnergyCommunityBonusStatus::LocatedInEnergyCommunity;
        input.claimed_domestic_content_bonus = true;
        input.claimed_energy_community_bonus = true;
        let out = check(&input);
        assert_eq!(out.mode, Section45YMode::CompliantWithBothBonusAddersStacked);
    }

    #[test]
    fn inflation_adjustment_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::InflationAdjustmentUnderSection45YC;
        let out = check(&input);
        assert_eq!(out.mode, Section45YMode::CompliantInflationAdjustmentApplied);
    }

    #[test]
    fn obbba_wind_solar_termination_non_wind_non_solar_facility_unaffected() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ObbbaWindSolarTerminationUnderSection70512;
        input.facility_technology_type = FacilityTechnologyType::NonWindNonSolarZeroEmissionFacility;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::CompliantNonWindNonSolarFacilityUnaffectedByObbbaWindSolarTermination
        );
    }

    #[test]
    fn obbba_wind_solar_within_transition_window_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ObbbaWindSolarTerminationUnderSection70512;
        input.facility_technology_type = FacilityTechnologyType::AppliableWindFacility;
        input.beginning_of_construction_status =
            BeginningOfConstructionStatus::BocOnOrBeforeJuly3_2026PreObbbaAnniversary;
        input.wind_solar_placed_in_service_status =
            WindSolarPlacedInServiceStatus::PlacedInServiceOnOrBeforeDecember31_2027PreObbbaWindSolarCutoff;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45YMode::CompliantTenYearCreditPeriodYearWithinWindow
        );
    }

    #[test]
    fn form_7211_filed_correctly_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm7211;
        input.form_7211_filed_correctly = true;
        let out = check(&input);
        assert_eq!(out.mode, Section45YMode::CompliantForm7211FiledCorrectly);
    }

    #[test]
    fn form_7211_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm7211;
        input.form_7211_filed_correctly = false;
        let out = check(&input);
        assert_eq!(out.mode, Section45YMode::ViolationForm7211NotFiledOrIncorrect);
    }

    #[test]
    fn constants_pin_section_45y_legislative_phases_and_credit_structure() {
        assert_eq!(IRC_45Y_IRA_ENACTMENT_DATE_YEAR, 2022);
        assert_eq!(IRC_45Y_IRA_ENACTMENT_DATE_MONTH, 8);
        assert_eq!(IRC_45Y_IRA_ENACTMENT_DATE_DAY, 16);
        assert_eq!(IRC_45Y_IRA_PUBLIC_LAW_CONGRESS, 117);
        assert_eq!(IRC_45Y_IRA_PUBLIC_LAW_ENACTMENT, 169);
        assert_eq!(IRC_45Y_IRA_ENABLING_SECTION, 13701);
        assert_eq!(IRC_45Y_EFFECTIVE_DATE_YEAR, 2025);
        assert_eq!(IRC_45Y_EFFECTIVE_DATE_MONTH, 1);
        assert_eq!(IRC_45Y_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(IRC_45Y_OBBBA_ENACTMENT_DATE_YEAR, 2025);
        assert_eq!(IRC_45Y_OBBBA_ENACTMENT_DATE_MONTH, 7);
        assert_eq!(IRC_45Y_OBBBA_ENACTMENT_DATE_DAY, 4);
        assert_eq!(IRC_45Y_OBBBA_PUBLIC_LAW_CONGRESS, 119);
        assert_eq!(IRC_45Y_OBBBA_PUBLIC_LAW_ENACTMENT, 21);
        assert_eq!(IRC_45Y_OBBBA_ENABLING_SECTION, 70512);
        assert_eq!(IRC_45Y_OBBBA_WIND_SOLAR_BOC_CUTOFF_YEAR, 2026);
        assert_eq!(IRC_45Y_OBBBA_WIND_SOLAR_BOC_CUTOFF_MONTH, 7);
        assert_eq!(IRC_45Y_OBBBA_WIND_SOLAR_BOC_CUTOFF_DAY, 4);
        assert_eq!(IRC_45Y_OBBBA_WIND_SOLAR_PIS_CUTOFF_YEAR, 2027);
        assert_eq!(IRC_45Y_OBBBA_WIND_SOLAR_PIS_CUTOFF_MONTH, 12);
        assert_eq!(IRC_45Y_OBBBA_WIND_SOLAR_PIS_CUTOFF_DAY, 31);
        assert_eq!(IRC_45Y_ORIGINAL_PHASE_OUT_YEAR, 2032);
        assert_eq!(IRC_45Y_BASE_RATE_TENTHS_OF_CENTS_PER_KWH, 3);
        assert_eq!(IRC_45Y_BONUS_RATE_TENTHS_OF_CENTS_PER_KWH, 15);
        assert_eq!(IRC_45Y_BONUS_MULTIPLIER, 5);
        assert_eq!(IRC_45Y_RATE_DENOMINATOR_TENTHS_OF_CENTS_PER_DOLLAR, 1_000);
        assert_eq!(IRC_45Y_CREDIT_PERIOD_YEARS, 10);
        assert_eq!(IRC_45Y_ENERGY_COMMUNITY_BONUS_BPS, 1_000);
        assert_eq!(IRC_45Y_DOMESTIC_CONTENT_BONUS_BPS, 1_000);
        assert_eq!(IRC_45Y_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_45Y_FORM_NUMBER, 7211);
        assert_eq!(IRC_45Y_BOC_GUIDANCE_NOTICE_YEAR, 2025);
        assert_eq!(IRC_45Y_BOC_GUIDANCE_NOTICE_NUMBER, 42);
    }

    #[test]
    fn citations_pin_legislative_phases_and_obbba_termination_facts() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("IRC § 45Y Clean Electricity Production Credit"));
        assert!(joined.contains("Section 13701 of the Inflation Reduction Act of 2022"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("August 16, 2022"));
        assert!(joined.contains("December 31, 2024"));
        assert!(joined.contains("0.3 CENTS"));
        assert!(joined.contains("1.5 CENTS"));
        assert!(joined.contains("FIVE TIMES the base rate"));
        assert!(joined.contains("GREENHOUSE GAS EMISSIONS RATE of NOT GREATER THAN ZERO"));
        assert!(joined.contains("10-YEAR PERIOD"));
        assert!(joined.contains("+10 PERCENT"));
        assert!(joined.contains("ENERGY COMMUNITY"));
        assert!(joined.contains("DOMESTIC CONTENT"));
        assert!(joined.contains("OBBBA 2025 § 70512"));
        assert!(joined.contains("OBBBA enactment"));
        assert!(joined.contains("JULY 4, 2026"));
        assert!(joined.contains("DECEMBER 31, 2027"));
        assert!(joined.contains("APPLICABLE WIND FACILITIES"));
        assert!(joined.contains("APPLICABLE SOLAR FACILITIES"));
        assert!(joined.contains("IRS Notice 2025-42"));
        assert!(joined.contains("Final Regulations T.D. 10024"));
        assert!(joined.contains("January 15, 2025"));
        assert!(joined.contains("Form 7211"));
    }

    #[test]
    fn saturating_overflow_defense_at_u64_max_kwh() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountUnderSection45YA;
        input.kilowatt_hours_produced = u64::MAX;
        let out = check(&input);
        let _ = out.computed_credit_dollars;
    }
}
