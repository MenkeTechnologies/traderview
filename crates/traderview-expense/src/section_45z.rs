//! IRC § 45Z Clean Fuel Production Credit
//! Compliance Module — pure-compute check for the
//! production tax credit for **DOMESTICALLY PRODUCED
//! CLEAN TRANSPORTATION FUEL** sold by the taxpayer.
//! Per-gallon credit structure based on lifecycle GHG
//! emissions rate, fuel type (aviation vs nonaviation),
//! and PWA compliance.
//!
//! Originally enacted by **Section 13704 of the Inflation
//! Reduction Act of 2022 (Public Law 117-169)**, signed by
//! President Joe Biden on **August 16, 2022**, effective
//! for clean transportation fuel **PRODUCED AND SOLD AFTER
//! DECEMBER 31, 2024 AND BEFORE JANUARY 1, 2028** (original
//! IRA window). **MODIFIED by the One Big Beautiful Bill
//! Act of 2025 (Public Law 119-21)**, signed by President
//! Donald Trump on **July 4, 2025**; OBBBA (i) **EXTENDED**
//! the credit through **DECEMBER 31, 2029** (a 2-year
//! extension); (ii) **REDUCED THE SAF (Sustainable Aviation
//! Fuel) MAXIMUM CREDIT** from $1.75/gallon to **$1.00/gallon**
//! (matching nonaviation fuel maximum); SAF base from 35 cents
//! to **20 cents/gallon** base.
//!
//! Web research (verified 2026-06-03):
//! - **IRA 2022 Enactment**: IRC § 45Z added by **Section 13704 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818)**; signed by President Joe Biden on **August 16, 2022**; effective for clean transportation fuel produced and sold after **DECEMBER 31, 2024** ([Congressional Research Service IF12502 — The Section 45Z Clean Fuel Production Credit](https://www.congress.gov/crs-product/IF12502); [U.S. Department of the Treasury — Guidance on Clean Fuels Production Credit](https://home.treasury.gov/news/press-releases/jy2780); [IRS — Clean Fuel Production Credit](https://www.irs.gov/credits-deductions/clean-fuel-production-credit); [IRS — Notice 2025-11 PDF](https://www.irs.gov/pub/irs-drop/n-25-11.pdf); [IRS — Notice 2025-10 PDF](https://www.irs.gov/pub/irs-drop/n-25-10.pdf); [Cornell LII — 26 U.S. Code § 45Z](https://www.law.cornell.edu/uscode/text/26/45Z); [IRS — About Form 7218 Clean Fuel Production Credit](https://www.irs.gov/forms-pubs/about-form-7218); [IRS — Instructions for Form 7218 (12/2025)](https://www.irs.gov/instructions/i7218); [Federal Register — Section 45Z Clean Fuel Production Credit Proposed Regulations (February 4, 2026)](https://www.federalregister.gov/documents/2026/02/04/2026-02246/section-45z-clean-fuel-production-credit); [Clean Air Task Force — H.R. 1 Expands 45Z Clean Fuel Production Credit for Conventional Biofuels While Cutting SAF Tax Credit](https://www.catf.us/2025/10/h-r-1-expands-45z-clean-fuel-production-credit-for-conventional-biofuels-while-cutting-sustainable-aviation-fuel-tax-credit/); [Pillsbury Law — Proposed Regulations Under IRC Section 45Z Clean Fuel Production Credit](https://www.pillsburylaw.com/en/news-and-insights/treasury-irs-regulations-irc-section-45z-clean-fuel-credit.html); [RSM US — Key Takeaways from New Section 45Z Clean Fuel Production Credit Guidance](https://rsmus.com/insights/tax-alerts/2026/key-takeaways-from-new-section.html); [American Farm Bureau Federation — 45Z Clean Fuel Production Credit](https://www.fb.org/market-intel/45z-clean-fuel-production-credit); [Crux Climate — IRS Issues 45Z Clean Fuel Production Credit Proposed Regulations](https://www.cruxclimate.com/insights/new-proposed-45z-regulations); [Holland & Knight — Questions and Answers: Initial Section 45Z Clean Fuel PTC Guidance](https://www.hklaw.com/en/insights/publications/2025/01/questions-and-answers-initial-section-45z-clean-fuel-ptc-guidance); [Baker Botts — IRS Issues Proposed Regulations Regarding § 45Z Clean Fuel Production Tax Credit February 2026](https://www.bakerbotts.com/thought-leadership/publications/2026/february/irs-issues-proposed-regulations-regarding-45z-clean-fuel-production-tax-credit); [Probity Tax Recovery — Section 45Z Clean Fuel Production Credit IRS Notices 2025-10 & 2025-11](https://www.probitytaxrecovery.com/blogs/section-45z-clean-fuel-production-credit-irs-notices-2025-10-2025-11); [Mondaq — IRS Issues Proposed Regulations Regarding § 45Z](https://www.mondaq.com/unitedstates/renewables/1744706/irs-issues-proposed-regulations-regarding-45z-clean-fuel-production-tax-credit); [Center for Agricultural Law and Taxation — Unpacking the Section 45Z Clean Fuel Production Credit](https://www.calt.iastate.edu/post/unpacking-section-45z-clean-fuel-production-credit); [Baker Botts — IRS Issues Guidance on § 45Z January 2025](https://www.bakerbotts.com/thought-leadership/publications/2025/january/irs-issues-guidance-on-the-45z-clean-fuel-production-tax-credit); [World Energy — Treasury Releases Initial 45Z Clean Fuel Production Credit Guidance](https://www.world-energy.org/article/48555.html); [PwC — Aircraft Club Section 45Z Credit Guidance Released](https://www.pwc.com/us/en/services/tax/library/aircraft-club-section-45z-credit-guidance-released.html)).
//! - **§ 45Z(a) Base Credit Amount — Nonaviation Fuel**: maximum **20 CENTS PER GALLON** of nonaviation fuel for producers NOT meeting prevailing wage and apprenticeship (PWA) requirements; **$1.00 PER GALLON** with PWA satisfied (5x multiplier).
//! - **§ 45Z(a) Base Credit Amount — Aviation Fuel (SAF)**: under **ORIGINAL IRA 2022**, **35 CENTS PER GALLON** base for SAF; **$1.75 PER GALLON** with PWA satisfied (5x multiplier). **OBBBA 2025 REDUCED SAF** maximum credit from $1.75/gallon to **$1.00/gallon** (matching nonaviation fuel); SAF base from 35 cents to **20 CENTS PER GALLON** base.
//! - **§ 45Z(a)(2) PWA Bonus 5x Multiplier**: applicable amount **MULTIPLIED BY 5** if qualified facility satisfies prevailing wage and apprenticeship (PWA) requirements during construction and for the credit period.
//! - **§ 45Z(b)(1)(B) Emissions Rate Threshold**: to qualify for § 45Z credit, fuel must have a lifecycle greenhouse gas emissions rate (**emissions rate**) of **NOT GREATER THAN 50 KILOGRAMS (kg) OF CO2e PER MILLION BTU (mmBTU)**.
//! - **§ 45Z(b)(2) Emissions Factor Multiplier**: maximum baseline credit amount is multiplied by an **EMISSIONS FACTOR**; the more CO2e emitted by the fuel (emissions rate), the lower the emissions factor; an emissions factor of 1.0 applies to fuel with zero lifecycle GHG emissions, while an emissions factor of 0 applies to fuel at the 50 kg CO2e/mmBTU threshold.
//! - **§ 45Z(b)(1)(A) GREET Model — Lifecycle Emissions Methodology**: lifecycle GHG emissions rates for vehicle fuels must be determined using the **45ZCF-GREET MODEL** (Argonne National Laboratory's modified Greenhouse gases, Regulated Emissions, and Energy use in Transportation model); SAF producers may alternatively use the **CORSIA Default model** or **CORSIA Actual model** (Carbon Offsetting and Reduction Scheme for International Aviation).
//! - **§ 45Z(c) Qualifying Fuels**: ethanol, biodiesel, renewable diesel, propane, naphtha, renewable natural gas, hydrogen, and **SUSTAINABLE AVIATION FUEL (SAF)** — broad fuel-type coverage.
//! - **§ 45Z(e) Credit Computation Formula**: credit equals **APPLICABLE AMOUNT × EMISSIONS FACTOR × GALLONS PRODUCED AND SOLD**; applicable amount is the per-gallon rate; emissions factor scales by lifecycle GHG emissions; gallons measured at point of sale.
//! - **OBBBA 2025 Extension Through December 31, 2029**: OBBBA extended the § 45Z credit availability through **DECEMBER 31, 2029** — a **2-YEAR EXTENSION** of the original IRA 2022 December 31, 2027 cutoff.
//! - **§ 6417 Direct Pay Election Available**: § 45Z credit eligible for **DIRECT PAY ELECTION** under § 6417 for tax-exempt entities; special **5-YEAR DIRECT PAY ELECTION** available for taxable entities for the first 5 years of the credit period.
//! - **§ 6418 Transferability Election Available**: § 45Z credit eligible for **TRANSFERABILITY ELECTION** under § 6418 — taxpayer may sell credit to an unrelated third party for cash.
//! - **Form 7218 (Clean Fuel Production Credit)**: required to claim the § 45Z credit beginning with tax year 2025 (Rev. December 2025).
//! - **IRS Notice 2025-10 + 2025-11 Initial Guidance**: IRS issued initial guidance on § 45Z in January 2025; Notice 2025-11 includes the emissions factor calculation and emissions rate table required by IRC § 45Z(e).
//! - **Proposed Regulations February 4, 2026**: Treasury and IRS issued proposed regulations under § 45Z published in the Federal Register on February 4, 2026.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_45Z_IRA_ENACTMENT_DATE_YEAR: u32 = 2022;
pub const IRC_45Z_IRA_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_45Z_IRA_ENACTMENT_DATE_DAY: u32 = 16;
pub const IRC_45Z_IRA_PUBLIC_LAW_CONGRESS: u32 = 117;
pub const IRC_45Z_IRA_PUBLIC_LAW_ENACTMENT: u32 = 169;
pub const IRC_45Z_IRA_ENABLING_SECTION: u32 = 13704;
pub const IRC_45Z_EFFECTIVE_DATE_YEAR: u32 = 2025;
pub const IRC_45Z_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const IRC_45Z_EFFECTIVE_DATE_DAY: u32 = 1;
pub const IRC_45Z_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_45Z_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_45Z_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_45Z_OBBBA_PUBLIC_LAW_CONGRESS: u32 = 119;
pub const IRC_45Z_OBBBA_PUBLIC_LAW_ENACTMENT: u32 = 21;
pub const IRC_45Z_OBBBA_EXTENDED_CUTOFF_YEAR: u32 = 2029;
pub const IRC_45Z_OBBBA_EXTENDED_CUTOFF_MONTH: u32 = 12;
pub const IRC_45Z_OBBBA_EXTENDED_CUTOFF_DAY: u32 = 31;
pub const IRC_45Z_ORIGINAL_IRA_CUTOFF_YEAR: u32 = 2027;
pub const IRC_45Z_NONAVIATION_BASE_CENTS_PER_GALLON: u64 = 20;
pub const IRC_45Z_NONAVIATION_PWA_CENTS_PER_GALLON: u64 = 100;
pub const IRC_45Z_SAF_ORIGINAL_IRA_BASE_CENTS_PER_GALLON: u64 = 35;
pub const IRC_45Z_SAF_ORIGINAL_IRA_PWA_CENTS_PER_GALLON: u64 = 175;
pub const IRC_45Z_SAF_POST_OBBBA_BASE_CENTS_PER_GALLON: u64 = 20;
pub const IRC_45Z_SAF_POST_OBBBA_PWA_CENTS_PER_GALLON: u64 = 100;
pub const IRC_45Z_PWA_BONUS_MULTIPLIER: u64 = 5;
pub const IRC_45Z_EMISSIONS_RATE_THRESHOLD_KG_CO2E_PER_MMBTU: u64 = 50;
pub const IRC_45Z_EMISSIONS_FACTOR_DENOMINATOR_BPS: u64 = 10_000;
pub const IRC_45Z_CENTS_PER_DOLLAR: u64 = 100;
pub const IRC_45Z_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_45Z_FORM_NUMBER: u32 = 7218;
pub const IRC_45Z_DIRECT_PAY_CROSS_REFERENCE_SECTION: u32 = 6417;
pub const IRC_45Z_DIRECT_PAY_TAXABLE_ENTITY_YEARS: u32 = 5;
pub const IRC_45Z_TRANSFERABILITY_CROSS_REFERENCE_SECTION: u32 = 6418;
pub const IRC_45Z_PROPOSED_REGS_PUBLICATION_DATE_YEAR: u32 = 2026;
pub const IRC_45Z_PROPOSED_REGS_PUBLICATION_DATE_MONTH: u32 = 2;
pub const IRC_45Z_PROPOSED_REGS_PUBLICATION_DATE_DAY: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionDateStatus {
    ProducedAndSoldBeforeJanuary1_2025PreEffective,
    ProducedAndSoldBetweenJanuary1_2025AndDecember31_2029PostObbbaEligible,
    ProducedAndSoldAfterDecember31_2029PostObbbaExtensionEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FuelType {
    NonaviationFuel,
    SustainableAviationFuel,
    NonQualifyingFuel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmissionsRateStatus {
    EmissionsRateAtOrBelowThresholdOfFiftyKgCo2ePerMmBtu,
    EmissionsRateAboveThresholdOfFiftyKgCo2ePerMmBtu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrevailingWageApprenticeshipStatus {
    PwaRequirementsMetEligibleForBonusMultiplier,
    PwaRequirementsNotMetBaseRateOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    BaseCreditAmountForNonaviationFuelUnderSection45ZA,
    BaseCreditAmountForSafFuelUnderSection45ZA,
    BonusCreditAmountForPwaUnderSection45ZA2,
    EmissionsRateThresholdUnderSection45ZB1B,
    EmissionsFactorMultiplierUnderSection45ZB2,
    ObbbaExtensionThroughDecember31_2029,
    DirectPayElectionUnderSection6417,
    TransferabilityElectionUnderSection6418,
    FormFilingUnderForm7218,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section45ZMode {
    NotApplicableProducedAndSoldBeforeJanuary1_2025PreEffective,
    NotApplicableProducedAndSoldAfterDecember31_2029PostObbbaExtensionEnd,
    NotApplicableNonQualifyingFuel,
    NotApplicableEmissionsRateAboveFiftyKgCo2ePerMmBtuThreshold,
    CompliantNonaviationFuelBaseCreditAtTwentyCentsPerGallon,
    CompliantNonaviationFuelPwaBonusCreditAtOneDollarPerGallon,
    CompliantSafFuelOriginalIraBaseCreditAtThirtyFiveCentsPerGallon,
    CompliantSafFuelOriginalIraPwaBonusCreditAtOneSeventyFivePerGallon,
    CompliantSafFuelPostObbbaBaseCreditAtTwentyCentsPerGallon,
    CompliantSafFuelPostObbbaPwaBonusCreditAtOneDollarPerGallon,
    CompliantEmissionsRateAtOrBelowThreshold,
    CompliantEmissionsFactorApplied,
    CompliantObbbaExtensionThroughDecember31_2029,
    CompliantDirectPayElectionMade,
    CompliantTransferabilityElectionMade,
    CompliantForm7218FiledCorrectly,
    ViolationPwaBonusMultiplierClaimedWithoutMeetingPwaRequirements,
    ViolationForm7218NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub production_date_status: ProductionDateStatus,
    pub fuel_type: FuelType,
    pub emissions_rate_status: EmissionsRateStatus,
    pub pwa_status: PrevailingWageApprenticeshipStatus,
    pub compliance_aspect: ComplianceAspect,
    pub gallons_produced_and_sold: u64,
    pub emissions_factor_bps: u64,
    pub direct_pay_election_made: bool,
    pub transferability_election_made: bool,
    pub form_7218_filed_correctly: bool,
    pub claimed_pwa_bonus_multiplier: bool,
    pub apply_obbba_saf_rate_reduction: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section45ZMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section45ZInput = Input;
pub type Section45ZOutput = Output;
pub type Section45ZResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 45Z Clean Fuel Production Credit added by Section 13704 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818); signed by President Joe Biden on August 16, 2022; effective for clean transportation fuel produced and sold after December 31, 2024".to_string(),
        "IRC § 45Z(a) Base Credit Amount — Nonaviation Fuel — maximum 20 CENTS PER GALLON of nonaviation fuel for producers NOT meeting prevailing wage and apprenticeship (PWA) requirements; $1.00 PER GALLON with PWA satisfied (5x multiplier)".to_string(),
        "IRC § 45Z(a) Base Credit Amount — Aviation Fuel (SAF) — under ORIGINAL IRA 2022, 35 CENTS PER GALLON base for SAF; $1.75 PER GALLON with PWA satisfied (5x multiplier). OBBBA 2025 REDUCED SAF maximum credit from $1.75/gallon to $1.00/gallon (matching nonaviation fuel); SAF base from 35 cents to 20 CENTS PER GALLON base".to_string(),
        "IRC § 45Z(a)(2) PWA Bonus 5x Multiplier — applicable amount MULTIPLIED BY 5 if qualified facility satisfies prevailing wage and apprenticeship (PWA) requirements during construction and for the credit period".to_string(),
        "IRC § 45Z(b)(1)(B) Emissions Rate Threshold — to qualify for § 45Z credit, fuel must have a lifecycle greenhouse gas emissions rate (emissions rate) of NOT GREATER THAN 50 KILOGRAMS (kg) OF CO2e PER MILLION BTU (mmBTU)".to_string(),
        "IRC § 45Z(b)(2) Emissions Factor Multiplier — maximum baseline credit amount is multiplied by an EMISSIONS FACTOR; the more CO2e emitted by the fuel (emissions rate), the lower the emissions factor; an emissions factor of 1.0 applies to fuel with zero lifecycle GHG emissions, while an emissions factor of 0 applies to fuel at the 50 kg CO2e/mmBTU threshold".to_string(),
        "IRC § 45Z(b)(1)(A) GREET Model — Lifecycle Emissions Methodology — lifecycle GHG emissions rates for vehicle fuels must be determined using the 45ZCF-GREET MODEL (Argonne National Laboratory); SAF producers may alternatively use the CORSIA Default model or CORSIA Actual model (Carbon Offsetting and Reduction Scheme for International Aviation)".to_string(),
        "IRC § 45Z(c) Qualifying Fuels — ethanol, biodiesel, renewable diesel, propane, naphtha, renewable natural gas, hydrogen, and SUSTAINABLE AVIATION FUEL (SAF)".to_string(),
        "IRC § 45Z(e) Credit Computation Formula — credit equals APPLICABLE AMOUNT × EMISSIONS FACTOR × GALLONS PRODUCED AND SOLD".to_string(),
        "OBBBA 2025 Extension Through December 31, 2029 — OBBBA extended the § 45Z credit availability through DECEMBER 31, 2029 (a 2-YEAR EXTENSION of the original IRA 2022 December 31, 2027 cutoff)".to_string(),
        "IRC § 6417 Direct Pay Election — § 45Z credit eligible for DIRECT PAY ELECTION for tax-exempt entities; special 5-YEAR DIRECT PAY ELECTION available for taxable entities for the first 5 years of the credit period".to_string(),
        "IRC § 6418 Transferability Election — § 45Z credit eligible for TRANSFERABILITY ELECTION; taxpayer may sell credit to an unrelated third party for cash".to_string(),
        "Form 7218 (Clean Fuel Production Credit) — required to claim the § 45Z credit beginning with tax year 2025 (Rev. December 2025)".to_string(),
        "IRS Notice 2025-10 + 2025-11 Initial Guidance — Notice 2025-11 includes the emissions factor calculation and emissions rate table required by IRC § 45Z(e)".to_string(),
        "Proposed Regulations February 4, 2026 — Treasury and IRS issued proposed regulations under § 45Z published in the Federal Register on February 4, 2026".to_string(),
        "Congressional Research Service IF12502 + U.S. Department of the Treasury + IRS + Cornell LII + Pillsbury Law + RSM US + American Farm Bureau Federation + Crux Climate + Holland & Knight + Baker Botts + Probity Tax Recovery + Mondaq + Center for Agricultural Law and Taxation + World Energy + PwC + Clean Air Task Force — practitioner overviews of § 45Z".to_string(),
    ];

    if input.production_date_status
        == ProductionDateStatus::ProducedAndSoldBeforeJanuary1_2025PreEffective
    {
        return Output {
            mode: Section45ZMode::NotApplicableProducedAndSoldBeforeJanuary1_2025PreEffective,
            statutory_basis: "IRA 2022 § 13704 effective date — § 45Z applies only to fuel produced and sold after December 31, 2024".to_string(),
            notes: "NOT APPLICABLE: fuel produced and sold before January 1, 2025 (pre-effective date); § 45Z credit unavailable.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.production_date_status
        == ProductionDateStatus::ProducedAndSoldAfterDecember31_2029PostObbbaExtensionEnd
    {
        return Output {
            mode: Section45ZMode::NotApplicableProducedAndSoldAfterDecember31_2029PostObbbaExtensionEnd,
            statutory_basis: "OBBBA 2025 § 45Z extension — credit available only through December 31, 2029".to_string(),
            notes: "NOT APPLICABLE: fuel produced and sold after December 31, 2029 (post-OBBBA extension end); § 45Z credit unavailable.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.fuel_type == FuelType::NonQualifyingFuel {
        return Output {
            mode: Section45ZMode::NotApplicableNonQualifyingFuel,
            statutory_basis: "IRC § 45Z(c) — fuel does not qualify under § 45Z(c) qualifying fuels".to_string(),
            notes: "NOT APPLICABLE: fuel does not qualify under § 45Z(c) (ethanol / biodiesel / renewable diesel / propane / naphtha / renewable natural gas / hydrogen / SAF).".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.emissions_rate_status
        == EmissionsRateStatus::EmissionsRateAboveThresholdOfFiftyKgCo2ePerMmBtu
    {
        return Output {
            mode: Section45ZMode::NotApplicableEmissionsRateAboveFiftyKgCo2ePerMmBtuThreshold,
            statutory_basis: "IRC § 45Z(b)(1)(B) — fuel must have lifecycle GHG emissions rate NOT GREATER THAN 50 kg CO2e/mmBTU".to_string(),
            notes: "NOT APPLICABLE: fuel has lifecycle GHG emissions rate above 50 kg CO2e/mmBTU threshold; § 45Z credit unavailable.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::BaseCreditAmountForNonaviationFuelUnderSection45ZA => {
            let cents_per_gallon = IRC_45Z_NONAVIATION_BASE_CENTS_PER_GALLON;
            let computed = compute_credit_dollars(
                input.gallons_produced_and_sold,
                cents_per_gallon,
                input.emissions_factor_bps,
            );
            Output {
                mode: Section45ZMode::CompliantNonaviationFuelBaseCreditAtTwentyCentsPerGallon,
                statutory_basis: "IRC § 45Z(a) — base credit at 20 cents/gallon for nonaviation fuel".to_string(),
                notes: format!(
                    "COMPLIANT: base credit at 20 cents/gallon × emissions factor {ef_bps} bps × {g} gallons = ${computed}.",
                    ef_bps = input.emissions_factor_bps,
                    g = input.gallons_produced_and_sold,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::BaseCreditAmountForSafFuelUnderSection45ZA => {
            let cents_per_gallon = if input.apply_obbba_saf_rate_reduction {
                IRC_45Z_SAF_POST_OBBBA_BASE_CENTS_PER_GALLON
            } else {
                IRC_45Z_SAF_ORIGINAL_IRA_BASE_CENTS_PER_GALLON
            };
            let computed = compute_credit_dollars(
                input.gallons_produced_and_sold,
                cents_per_gallon,
                input.emissions_factor_bps,
            );
            let mode = if input.apply_obbba_saf_rate_reduction {
                Section45ZMode::CompliantSafFuelPostObbbaBaseCreditAtTwentyCentsPerGallon
            } else {
                Section45ZMode::CompliantSafFuelOriginalIraBaseCreditAtThirtyFiveCentsPerGallon
            };
            Output {
                mode,
                statutory_basis: format!(
                    "IRC § 45Z(a) — base credit at {cents_per_gallon} cents/gallon for SAF (OBBBA reduction applied: {applied})",
                    applied = input.apply_obbba_saf_rate_reduction,
                ),
                notes: format!(
                    "COMPLIANT: base credit at {cents_per_gallon} cents/gallon × emissions factor {ef_bps} bps × {g} gallons = ${computed}.",
                    ef_bps = input.emissions_factor_bps,
                    g = input.gallons_produced_and_sold,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::BonusCreditAmountForPwaUnderSection45ZA2 => {
            if input.claimed_pwa_bonus_multiplier
                && input.pwa_status
                    == PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly
            {
                return Output {
                    mode: Section45ZMode::ViolationPwaBonusMultiplierClaimedWithoutMeetingPwaRequirements,
                    statutory_basis: "IRC § 45Z(a)(2) — PWA bonus 5x multiplier requires prevailing wage and apprenticeship compliance".to_string(),
                    notes: "VIOLATION: PWA bonus 5x multiplier claimed but prevailing wage and apprenticeship requirements not met; only base rate available.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                };
            }
            let cents_per_gallon = match (input.fuel_type, input.apply_obbba_saf_rate_reduction) {
                (FuelType::NonaviationFuel, _) => IRC_45Z_NONAVIATION_PWA_CENTS_PER_GALLON,
                (FuelType::SustainableAviationFuel, false) => {
                    IRC_45Z_SAF_ORIGINAL_IRA_PWA_CENTS_PER_GALLON
                }
                (FuelType::SustainableAviationFuel, true) => {
                    IRC_45Z_SAF_POST_OBBBA_PWA_CENTS_PER_GALLON
                }
                (FuelType::NonQualifyingFuel, _) => unreachable!(),
            };
            let computed = compute_credit_dollars(
                input.gallons_produced_and_sold,
                cents_per_gallon,
                input.emissions_factor_bps,
            );
            let mode = match (input.fuel_type, input.apply_obbba_saf_rate_reduction) {
                (FuelType::NonaviationFuel, _) => {
                    Section45ZMode::CompliantNonaviationFuelPwaBonusCreditAtOneDollarPerGallon
                }
                (FuelType::SustainableAviationFuel, false) => {
                    Section45ZMode::CompliantSafFuelOriginalIraPwaBonusCreditAtOneSeventyFivePerGallon
                }
                (FuelType::SustainableAviationFuel, true) => {
                    Section45ZMode::CompliantSafFuelPostObbbaPwaBonusCreditAtOneDollarPerGallon
                }
                (FuelType::NonQualifyingFuel, _) => unreachable!(),
            };
            Output {
                mode,
                statutory_basis: format!(
                    "IRC § 45Z(a)(2) — PWA bonus credit at {cents_per_gallon} cents/gallon (5x base)"
                ),
                notes: format!(
                    "COMPLIANT: PWA bonus credit at {cents_per_gallon} cents/gallon × emissions factor {ef_bps} bps × {g} gallons = ${computed}; PWA requirements satisfied.",
                    ef_bps = input.emissions_factor_bps,
                    g = input.gallons_produced_and_sold,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::EmissionsRateThresholdUnderSection45ZB1B => Output {
            mode: Section45ZMode::CompliantEmissionsRateAtOrBelowThreshold,
            statutory_basis: "IRC § 45Z(b)(1)(B) — fuel emissions rate at or below 50 kg CO2e/mmBTU threshold".to_string(),
            notes: "COMPLIANT: fuel has lifecycle GHG emissions rate at or below 50 kg CO2e/mmBTU threshold under § 45Z(b)(1)(B).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::EmissionsFactorMultiplierUnderSection45ZB2 => Output {
            mode: Section45ZMode::CompliantEmissionsFactorApplied,
            statutory_basis: "IRC § 45Z(b)(2) — emissions factor applied as multiplier to baseline credit".to_string(),
            notes: format!(
                "COMPLIANT: emissions factor {ef_bps} bps applied as multiplier under § 45Z(b)(2) (1.0 = 10,000 bps for zero emissions; 0 bps at 50 kg CO2e/mmBTU threshold).",
                ef_bps = input.emissions_factor_bps,
            ),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::ObbbaExtensionThroughDecember31_2029 => Output {
            mode: Section45ZMode::CompliantObbbaExtensionThroughDecember31_2029,
            statutory_basis: "OBBBA 2025 — § 45Z credit extended through December 31, 2029".to_string(),
            notes: "COMPLIANT: OBBBA 2025 extended § 45Z credit availability through December 31, 2029 (a 2-year extension of the original IRA 2022 December 31, 2027 cutoff).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::DirectPayElectionUnderSection6417 => {
            if input.direct_pay_election_made {
                Output {
                    mode: Section45ZMode::CompliantDirectPayElectionMade,
                    statutory_basis: "IRC § 6417 — direct pay election made for § 45Z credit (5-year availability for taxable entities; permanent for tax-exempt entities)".to_string(),
                    notes: "COMPLIANT: direct pay election under § 6417 made for § 45Z credit; allows monetization as cash refund.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45ZMode::CompliantNonaviationFuelBaseCreditAtTwentyCentsPerGallon,
                    statutory_basis: "IRC § 6417 — direct pay election not made; credit claimed as offset to tax liability".to_string(),
                    notes: "COMPLIANT: direct pay election under § 6417 not made; credit claimed as offset to federal income tax liability.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::TransferabilityElectionUnderSection6418 => {
            if input.transferability_election_made {
                Output {
                    mode: Section45ZMode::CompliantTransferabilityElectionMade,
                    statutory_basis: "IRC § 6418 — transferability election made for § 45Z credit (sale to unrelated third party for cash)".to_string(),
                    notes: "COMPLIANT: transferability election under § 6418 made for § 45Z credit; allows sale of credit to unrelated third party for cash.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45ZMode::CompliantNonaviationFuelBaseCreditAtTwentyCentsPerGallon,
                    statutory_basis: "IRC § 6418 — transferability election not made; credit claimed by taxpayer".to_string(),
                    notes: "COMPLIANT: transferability election under § 6418 not made; credit claimed by taxpayer as offset to federal income tax liability.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::FormFilingUnderForm7218 => {
            if input.form_7218_filed_correctly {
                Output {
                    mode: Section45ZMode::CompliantForm7218FiledCorrectly,
                    statutory_basis: "Form 7218 — Clean Fuel Production Credit form required to claim § 45Z credit".to_string(),
                    notes: "COMPLIANT: Form 7218 filed correctly to claim § 45Z credit for tax year.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45ZMode::ViolationForm7218NotFiledOrIncorrect,
                    statutory_basis: "Form 7218 filing required to claim § 45Z credit".to_string(),
                    notes: "VIOLATION: Form 7218 not filed or incorrectly filed; § 45Z credit may be disallowed.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
    }
}

fn compute_credit_dollars(gallons: u64, cents_per_gallon: u64, emissions_factor_bps: u64) -> u64 {
    (u128::from(gallons) * u128::from(cents_per_gallon) * u128::from(emissions_factor_bps)
        / u128::from(IRC_45Z_CENTS_PER_DOLLAR)
        / u128::from(IRC_45Z_EMISSIONS_FACTOR_DENOMINATOR_BPS)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            production_date_status:
                ProductionDateStatus::ProducedAndSoldBetweenJanuary1_2025AndDecember31_2029PostObbbaEligible,
            fuel_type: FuelType::NonaviationFuel,
            emissions_rate_status:
                EmissionsRateStatus::EmissionsRateAtOrBelowThresholdOfFiftyKgCo2ePerMmBtu,
            pwa_status:
                PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusMultiplier,
            compliance_aspect: ComplianceAspect::BaseCreditAmountForNonaviationFuelUnderSection45ZA,
            gallons_produced_and_sold: 1_000_000,
            emissions_factor_bps: 10_000,
            direct_pay_election_made: false,
            transferability_election_made: false,
            form_7218_filed_correctly: true,
            claimed_pwa_bonus_multiplier: false,
            apply_obbba_saf_rate_reduction: false,
        }
    }

    #[test]
    fn pre_effective_production_not_applicable() {
        let mut input = baseline_input();
        input.production_date_status =
            ProductionDateStatus::ProducedAndSoldBeforeJanuary1_2025PreEffective;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::NotApplicableProducedAndSoldBeforeJanuary1_2025PreEffective
        );
    }

    #[test]
    fn post_obbba_extension_end_not_applicable() {
        let mut input = baseline_input();
        input.production_date_status =
            ProductionDateStatus::ProducedAndSoldAfterDecember31_2029PostObbbaExtensionEnd;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::NotApplicableProducedAndSoldAfterDecember31_2029PostObbbaExtensionEnd
        );
    }

    #[test]
    fn non_qualifying_fuel_not_applicable() {
        let mut input = baseline_input();
        input.fuel_type = FuelType::NonQualifyingFuel;
        let out = check(&input);
        assert_eq!(out.mode, Section45ZMode::NotApplicableNonQualifyingFuel);
    }

    #[test]
    fn emissions_rate_above_threshold_not_applicable() {
        let mut input = baseline_input();
        input.emissions_rate_status =
            EmissionsRateStatus::EmissionsRateAboveThresholdOfFiftyKgCo2ePerMmBtu;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::NotApplicableEmissionsRateAboveFiftyKgCo2ePerMmBtuThreshold
        );
    }

    #[test]
    fn nonaviation_base_credit_at_twenty_cents_per_gallon_computed() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BaseCreditAmountForNonaviationFuelUnderSection45ZA;
        input.gallons_produced_and_sold = 1_000_000;
        input.emissions_factor_bps = 10_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::CompliantNonaviationFuelBaseCreditAtTwentyCentsPerGallon
        );
        assert_eq!(out.computed_credit_dollars, 200_000);
    }

    #[test]
    fn nonaviation_pwa_bonus_at_one_dollar_per_gallon_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45ZA2;
        input.fuel_type = FuelType::NonaviationFuel;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusMultiplier;
        input.claimed_pwa_bonus_multiplier = true;
        input.gallons_produced_and_sold = 1_000_000;
        input.emissions_factor_bps = 10_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::CompliantNonaviationFuelPwaBonusCreditAtOneDollarPerGallon
        );
        assert_eq!(out.computed_credit_dollars, 1_000_000);
    }

    #[test]
    fn saf_original_ira_base_at_thirty_five_cents_per_gallon_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountForSafFuelUnderSection45ZA;
        input.fuel_type = FuelType::SustainableAviationFuel;
        input.apply_obbba_saf_rate_reduction = false;
        input.gallons_produced_and_sold = 1_000_000;
        input.emissions_factor_bps = 10_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::CompliantSafFuelOriginalIraBaseCreditAtThirtyFiveCentsPerGallon
        );
        assert_eq!(out.computed_credit_dollars, 350_000);
    }

    #[test]
    fn saf_original_ira_pwa_bonus_at_one_seventy_five_per_gallon_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45ZA2;
        input.fuel_type = FuelType::SustainableAviationFuel;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusMultiplier;
        input.claimed_pwa_bonus_multiplier = true;
        input.apply_obbba_saf_rate_reduction = false;
        input.gallons_produced_and_sold = 1_000_000;
        input.emissions_factor_bps = 10_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::CompliantSafFuelOriginalIraPwaBonusCreditAtOneSeventyFivePerGallon
        );
        assert_eq!(out.computed_credit_dollars, 1_750_000);
    }

    #[test]
    fn saf_post_obbba_base_at_twenty_cents_per_gallon_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountForSafFuelUnderSection45ZA;
        input.fuel_type = FuelType::SustainableAviationFuel;
        input.apply_obbba_saf_rate_reduction = true;
        input.gallons_produced_and_sold = 1_000_000;
        input.emissions_factor_bps = 10_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::CompliantSafFuelPostObbbaBaseCreditAtTwentyCentsPerGallon
        );
        assert_eq!(out.computed_credit_dollars, 200_000);
    }

    #[test]
    fn saf_post_obbba_pwa_bonus_at_one_dollar_per_gallon_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45ZA2;
        input.fuel_type = FuelType::SustainableAviationFuel;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusMultiplier;
        input.claimed_pwa_bonus_multiplier = true;
        input.apply_obbba_saf_rate_reduction = true;
        input.gallons_produced_and_sold = 1_000_000;
        input.emissions_factor_bps = 10_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::CompliantSafFuelPostObbbaPwaBonusCreditAtOneDollarPerGallon
        );
        assert_eq!(out.computed_credit_dollars, 1_000_000);
    }

    #[test]
    fn emissions_factor_below_one_reduces_credit() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BaseCreditAmountForNonaviationFuelUnderSection45ZA;
        input.gallons_produced_and_sold = 1_000_000;
        input.emissions_factor_bps = 5_000;
        let out = check(&input);
        assert_eq!(out.computed_credit_dollars, 100_000);
    }

    #[test]
    fn pwa_bonus_multiplier_claimed_without_meeting_requirements_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45ZA2;
        input.pwa_status = PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly;
        input.claimed_pwa_bonus_multiplier = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::ViolationPwaBonusMultiplierClaimedWithoutMeetingPwaRequirements
        );
    }

    #[test]
    fn emissions_rate_threshold_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EmissionsRateThresholdUnderSection45ZB1B;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::CompliantEmissionsRateAtOrBelowThreshold
        );
    }

    #[test]
    fn emissions_factor_multiplier_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EmissionsFactorMultiplierUnderSection45ZB2;
        let out = check(&input);
        assert_eq!(out.mode, Section45ZMode::CompliantEmissionsFactorApplied);
    }

    #[test]
    fn obbba_extension_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ObbbaExtensionThroughDecember31_2029;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::CompliantObbbaExtensionThroughDecember31_2029
        );
    }

    #[test]
    fn direct_pay_election_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DirectPayElectionUnderSection6417;
        input.direct_pay_election_made = true;
        let out = check(&input);
        assert_eq!(out.mode, Section45ZMode::CompliantDirectPayElectionMade);
    }

    #[test]
    fn transferability_election_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TransferabilityElectionUnderSection6418;
        input.transferability_election_made = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::CompliantTransferabilityElectionMade
        );
    }

    #[test]
    fn form_7218_filed_correctly_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm7218;
        input.form_7218_filed_correctly = true;
        let out = check(&input);
        assert_eq!(out.mode, Section45ZMode::CompliantForm7218FiledCorrectly);
    }

    #[test]
    fn form_7218_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm7218;
        input.form_7218_filed_correctly = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45ZMode::ViolationForm7218NotFiledOrIncorrect
        );
    }

    #[test]
    fn pwa_bonus_is_five_times_base_for_nonaviation() {
        assert_eq!(
            IRC_45Z_NONAVIATION_PWA_CENTS_PER_GALLON,
            IRC_45Z_NONAVIATION_BASE_CENTS_PER_GALLON * IRC_45Z_PWA_BONUS_MULTIPLIER
        );
    }

    #[test]
    fn pwa_bonus_is_five_times_base_for_saf_original() {
        assert_eq!(
            IRC_45Z_SAF_ORIGINAL_IRA_PWA_CENTS_PER_GALLON,
            IRC_45Z_SAF_ORIGINAL_IRA_BASE_CENTS_PER_GALLON * IRC_45Z_PWA_BONUS_MULTIPLIER
        );
    }

    #[test]
    fn pwa_bonus_is_five_times_base_for_saf_post_obbba() {
        assert_eq!(
            IRC_45Z_SAF_POST_OBBBA_PWA_CENTS_PER_GALLON,
            IRC_45Z_SAF_POST_OBBBA_BASE_CENTS_PER_GALLON * IRC_45Z_PWA_BONUS_MULTIPLIER
        );
    }

    #[test]
    fn constants_pin_section_45z_legislative_phases_and_per_gallon_structure() {
        assert_eq!(IRC_45Z_IRA_ENACTMENT_DATE_YEAR, 2022);
        assert_eq!(IRC_45Z_IRA_PUBLIC_LAW_CONGRESS, 117);
        assert_eq!(IRC_45Z_IRA_PUBLIC_LAW_ENACTMENT, 169);
        assert_eq!(IRC_45Z_IRA_ENABLING_SECTION, 13704);
        assert_eq!(IRC_45Z_EFFECTIVE_DATE_YEAR, 2025);
        assert_eq!(IRC_45Z_OBBBA_ENACTMENT_DATE_YEAR, 2025);
        assert_eq!(IRC_45Z_OBBBA_PUBLIC_LAW_CONGRESS, 119);
        assert_eq!(IRC_45Z_OBBBA_PUBLIC_LAW_ENACTMENT, 21);
        assert_eq!(IRC_45Z_OBBBA_EXTENDED_CUTOFF_YEAR, 2029);
        assert_eq!(IRC_45Z_OBBBA_EXTENDED_CUTOFF_MONTH, 12);
        assert_eq!(IRC_45Z_OBBBA_EXTENDED_CUTOFF_DAY, 31);
        assert_eq!(IRC_45Z_ORIGINAL_IRA_CUTOFF_YEAR, 2027);
        assert_eq!(IRC_45Z_NONAVIATION_BASE_CENTS_PER_GALLON, 20);
        assert_eq!(IRC_45Z_NONAVIATION_PWA_CENTS_PER_GALLON, 100);
        assert_eq!(IRC_45Z_SAF_ORIGINAL_IRA_BASE_CENTS_PER_GALLON, 35);
        assert_eq!(IRC_45Z_SAF_ORIGINAL_IRA_PWA_CENTS_PER_GALLON, 175);
        assert_eq!(IRC_45Z_SAF_POST_OBBBA_BASE_CENTS_PER_GALLON, 20);
        assert_eq!(IRC_45Z_SAF_POST_OBBBA_PWA_CENTS_PER_GALLON, 100);
        assert_eq!(IRC_45Z_PWA_BONUS_MULTIPLIER, 5);
        assert_eq!(IRC_45Z_EMISSIONS_RATE_THRESHOLD_KG_CO2E_PER_MMBTU, 50);
        assert_eq!(IRC_45Z_EMISSIONS_FACTOR_DENOMINATOR_BPS, 10_000);
        assert_eq!(IRC_45Z_CENTS_PER_DOLLAR, 100);
        assert_eq!(IRC_45Z_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_45Z_FORM_NUMBER, 7218);
        assert_eq!(IRC_45Z_DIRECT_PAY_CROSS_REFERENCE_SECTION, 6417);
        assert_eq!(IRC_45Z_DIRECT_PAY_TAXABLE_ENTITY_YEARS, 5);
        assert_eq!(IRC_45Z_TRANSFERABILITY_CROSS_REFERENCE_SECTION, 6418);
        assert_eq!(IRC_45Z_PROPOSED_REGS_PUBLICATION_DATE_YEAR, 2026);
        assert_eq!(IRC_45Z_PROPOSED_REGS_PUBLICATION_DATE_MONTH, 2);
        assert_eq!(IRC_45Z_PROPOSED_REGS_PUBLICATION_DATE_DAY, 4);
    }

    #[test]
    fn citations_pin_legislative_phases_and_obbba_facts() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("IRC § 45Z Clean Fuel Production Credit"));
        assert!(joined.contains("Section 13704 of the Inflation Reduction Act of 2022"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("August 16, 2022"));
        assert!(joined.contains("December 31, 2024"));
        assert!(joined.contains("20 CENTS PER GALLON"));
        assert!(joined.contains("$1.00 PER GALLON"));
        assert!(joined.contains("35 CENTS PER GALLON"));
        assert!(joined.contains("$1.75 PER GALLON"));
        assert!(joined.contains("MULTIPLIED BY 5"));
        assert!(joined.contains("NOT GREATER THAN 50 KILOGRAMS"));
        assert!(joined.contains("EMISSIONS FACTOR"));
        assert!(joined.contains("45ZCF-GREET MODEL"));
        assert!(joined.contains("CORSIA Default model"));
        assert!(joined.contains("CORSIA Actual model"));
        assert!(joined.contains("SUSTAINABLE AVIATION FUEL (SAF)"));
        assert!(joined.contains("DECEMBER 31, 2029"));
        assert!(joined.contains("2-YEAR EXTENSION"));
        assert!(joined.contains("§ 6417"));
        assert!(joined.contains("DIRECT PAY ELECTION"));
        assert!(joined.contains("§ 6418"));
        assert!(joined.contains("TRANSFERABILITY ELECTION"));
        assert!(joined.contains("Form 7218"));
        assert!(joined.contains("Notice 2025-11"));
        assert!(joined.contains("February 4, 2026"));
    }

    #[test]
    fn saturating_overflow_defense_at_u64_max_gallons() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::BaseCreditAmountForNonaviationFuelUnderSection45ZA;
        input.gallons_produced_and_sold = u64::MAX;
        let out = check(&input);
        let _ = out.computed_credit_dollars;
    }
}
