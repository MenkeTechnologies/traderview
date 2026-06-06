//! IRC § 45V Clean Hydrogen Production Credit
//! Compliance Module — pure-compute check for the
//! production tax credit for **QUALIFIED CLEAN HYDROGEN
//! (QCH)** produced and sold by the taxpayer. Provides
//! up to **$3.00 per kilogram** of qualified clean
//! hydrogen produced (with PWA bonus) over a 10-year
//! credit period, based on a four-tier lifecycle GHG
//! emissions rate determined via the **GREET model**
//! (Argonne National Laboratory's Greenhouse gases,
//! Regulated Emissions, and Energy use in Transportation
//! model).
//!
//! Originally enacted by **Section 13204 of the Inflation
//! Reduction Act of 2022 (Public Law 117-169)**, signed by
//! President Joe Biden on **August 16, 2022**, effective
//! for clean hydrogen produced at **QUALIFYING FACILITIES
//! PLACED IN SERVICE AFTER DECEMBER 31, 2022 AND BEFORE
//! JANUARY 1, 2033** (original IRA window). **MODIFIED by
//! Section 70511 of the One Big Beautiful Bill Act of 2025
//! (Public Law 119-21)**, signed by President Donald
//! Trump on **July 4, 2025**; § 45V credit phased out by
//! **JANUARY 1, 2028** — facilities must **BEGIN
//! CONSTRUCTION BEFORE JANUARY 1, 2028** to qualify (a
//! 5-year acceleration of the original IRA construction
//! cutoff).
//!
//! Web research (verified 2026-06-03):
//! - **IRA 2022 Enactment**: IRC § 45V added by **Section 13204 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818)**; signed by President Joe Biden on **August 16, 2022**; effective for clean hydrogen produced at qualifying facilities placed in service after **DECEMBER 31, 2022** ([U.S. Department of Energy — Clean Hydrogen Production Tax Credit (45V) Resources](https://www.energy.gov/articles/clean-hydrogen-production-tax-credit-45v-resources); [Federal Register — Credit for Production of Clean Hydrogen and Energy Credit Final Regulations (January 10, 2025)](https://www.federalregister.gov/documents/2025/01/10/2024-31513/credit-for-production-of-clean-hydrogen-and-energy-credit); [Cornell LII — 26 U.S. Code § 45V](https://www.law.cornell.edu/uscode/text/26/45V); [King & Spalding — Guidance on Section 45V Clean Hydrogen Production Tax Credit](https://www.kslaw.com/news-and-insights/guidance-on-section-45v-clean-hydrogen-production-tax-credit); [IRA Tracker — IRA Section 13204 Clean Hydrogen Tax Credit](https://iratracker.org/programs/ira-section-13204-clean-hydrogen-tax-credit/); [Novogradac — Section 45V Clean Hydrogen Production Credit Proposed Guidance](https://www.novoco.com/notes-from-novogradac/section-45v-clean-hydrogen-production-credit-proposes-guidance-on-determining-greenhouse-gas-lifecycles-certification-of-hydrogen-production-generating-electricity-from-renewable-resources-and-modifying-and-retrofitting-old-facilities); [U.S. Treasury — Final Rules for Clean Hydrogen Production Tax Credit](https://home.treasury.gov/news/press-releases/jy2768); [Holland & Knight — Treasury Department IRS Release Section 45V Clean Hydrogen PTC Final Regulations](https://www.hklaw.com/en/insights/publications/2025/01/treasury-department-irs-release-section-45v-clean-hydrogen-ptc); [House USC — 26 USC 45V Credit for Production of Clean Hydrogen](https://uscode.house.gov/view.xhtml?req=granuleid%3AUSC-prelim-title26-section45V&num=0&edition=prelim); [IRS — Clean Hydrogen Production Credit](https://www.irs.gov/credits-deductions/clean-hydrogen-production-credit); [IRS — Instructions for Form 7210 (2025)](https://www.irs.gov/instructions/i7210); [Accounting Insights — The Section 45V Clean Hydrogen Production Tax Credit](https://accountinginsights.org/the-section-45v-clean-hydrogen-production-tax-credit/); [PwC — Regulations on Wage and Apprenticeship Credit Bonus Finalized](https://www.pwc.com/us/en/services/tax/library/pwc-regulations-on-wage-and-apprenticeship-credit-bonus-finalized.html)).
//! - **§ 45V(a) Base Credit Amount**: credit equals **$0.60 PER KILOGRAM (kg)** of qualified clean hydrogen produced × **APPLICABLE PERCENTAGE** based on lifecycle GHG emissions rate; $0.60 amount **ADJUSTED FOR INFLATION** annually.
//! - **§ 45V(b)(2) Applicable Percentage — Four-Tier Lifecycle Emissions Structure**: **TIER 1** lifecycle GHG emissions rate **NOT GREATER THAN 4 kg CO2e/kg H2 AND NOT LESS THAN 2.5 kg CO2e/kg H2** = **20 PERCENT applicable percentage** ($0.12/kg base); **TIER 2** rate **LESS THAN 2.5 kg AND NOT LESS THAN 1.5 kg CO2e/kg H2** = **25 PERCENT applicable percentage** ($0.15/kg base); **TIER 3** rate **LESS THAN 1.5 kg AND NOT LESS THAN 0.45 kg CO2e/kg H2** = **33.4 PERCENT applicable percentage** ($0.20/kg base, rounded); **TIER 4** rate **LESS THAN 0.45 kg CO2e/kg H2** = **100 PERCENT applicable percentage** ($0.60/kg base — maximum tier).
//! - **§ 45V(b)(2)(B) PWA Bonus 5x Multiplier**: credit **MULTIPLIED BY 5** if taxpayer satisfies the **PREVAILING WAGE AND APPRENTICESHIP (PWA)** requirements during construction and for the 10-year credit period; with PWA bonus, maximum credit reaches **$3.00 per kg** (Tier 4 × 5 = $0.60 × 5 = $3.00/kg) ($0.60/kg base × 100 % applicable percentage × 5 multiplier).
//! - **§ 45V(b)(2)(C) Lifecycle GHG Emissions — Well-to-Gate Methodology**: lifecycle emissions include emissions through the point of production (**WELL-TO-GATE**) as determined under the **45VH2-GREET MODEL** developed by **Argonne National Laboratory** (Greenhouse gases, Regulated Emissions, and Energy use in Transportation model), or a successor model as determined by the Secretary.
//! - **§ 45V(a)(2) 10-Year Credit Period**: credit available for the **10-YEAR PERIOD BEGINNING ON DATE FACILITY PLACED IN SERVICE**.
//! - **§ 45V(c) Definitions — Qualified Clean Hydrogen (QCH)**: hydrogen which is produced (i) through a process that results in a lifecycle GHG emissions rate of **NOT GREATER THAN 4 kg CO2e PER kg H2**; (ii) **PROVIDED THROUGH VERIFICATION** as conforming to applicable certification requirements; AND (iii) produced in the United States by the taxpayer in the ordinary course of a trade or business of the taxpayer.
//! - **§ 45V(c)(2) Three Pillars Compliance — Final Regulations**: the **45VH2-GREET MODEL** applies the **THREE PILLARS** compliance framework for electrolysis hydrogen produced using grid electricity: (1) **TEMPORAL MATCHING** (hourly or annual matching of clean electricity and hydrogen production); (2) **DELIVERABILITY** (clean electricity must be deliverable to the hydrogen production facility); AND (3) **ADDITIONALITY / INCREMENTALITY** (clean electricity must be from new generation sources commissioned within 36 months of the hydrogen facility's placed-in-service date).
//! - **OBBBA 2025 § 70511 Phase-Out — Begin-of-Construction Cutoff**: § 45V credit phased out by **JANUARY 1, 2028** — facilities must **BEGIN CONSTRUCTION BEFORE JANUARY 1, 2028** to qualify (a **5-YEAR ACCELERATION** of the original IRA 2032 construction cutoff under § 45V(d)(2)).
//! - **§ 45V(d) Original IRA 2022 Phase-Out**: under original IRA 2022, credit available for clean hydrogen produced at qualified clean hydrogen production facilities the construction of which begins **BEFORE JANUARY 1, 2033** — accelerated by 5 years under OBBBA 2025 § 70511.
//! - **§ 6417 Direct Pay Election Available**: § 45V credit eligible for **DIRECT PAY ELECTION** under § 6417 for tax-exempt entities (501(c)(3), state/local/tribal governments, rural electric coops); special **5-YEAR DIRECT PAY ELECTION** available for taxable entities for the first 5 years of the 10-year credit period.
//! - **§ 6418 Transferability Election Available**: § 45V credit eligible for **TRANSFERABILITY ELECTION** under § 6418 — taxpayer may sell credit to an unrelated third party for cash.
//! - **Form 7210 (Clean Hydrogen Production Credit)**: required to claim the § 45V credit beginning with tax year 2023; 2025 instructions reflect § 6417 5-year direct pay election and § 6418 transferability.
//! - **Final Regulations T.D. 10023 (January 10, 2025)**: Treasury and IRS issued final regulations under § 45V published in the Federal Register on **January 10, 2025**; provide rules for lifecycle GHG emissions rate determination, three pillars compliance, certification, electrolysis facility eligibility, modification/retrofitting rules.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_45V_IRA_ENACTMENT_DATE_YEAR: u32 = 2022;
pub const IRC_45V_IRA_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_45V_IRA_ENACTMENT_DATE_DAY: u32 = 16;
pub const IRC_45V_IRA_PUBLIC_LAW_CONGRESS: u32 = 117;
pub const IRC_45V_IRA_PUBLIC_LAW_ENACTMENT: u32 = 169;
pub const IRC_45V_IRA_ENABLING_SECTION: u32 = 13204;
pub const IRC_45V_EFFECTIVE_DATE_YEAR: u32 = 2023;
pub const IRC_45V_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const IRC_45V_EFFECTIVE_DATE_DAY: u32 = 1;
pub const IRC_45V_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_45V_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_45V_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_45V_OBBBA_PUBLIC_LAW_CONGRESS: u32 = 119;
pub const IRC_45V_OBBBA_PUBLIC_LAW_ENACTMENT: u32 = 21;
pub const IRC_45V_OBBBA_ENABLING_SECTION: u32 = 70511;
pub const IRC_45V_OBBBA_BOC_CUTOFF_YEAR: u32 = 2028;
pub const IRC_45V_OBBBA_BOC_CUTOFF_MONTH: u32 = 1;
pub const IRC_45V_OBBBA_BOC_CUTOFF_DAY: u32 = 1;
pub const IRC_45V_ORIGINAL_IRA_BOC_CUTOFF_YEAR: u32 = 2033;
pub const IRC_45V_BASE_RATE_CENTS_PER_KG: u64 = 60;
pub const IRC_45V_CENTS_PER_DOLLAR: u64 = 100;
pub const IRC_45V_BONUS_MULTIPLIER: u64 = 5;
pub const IRC_45V_TIER_1_LOWER_GHG_TENTHS_KG_CO2E_PER_KG_H2: u64 = 25;
pub const IRC_45V_TIER_1_UPPER_GHG_TENTHS_KG_CO2E_PER_KG_H2: u64 = 40;
pub const IRC_45V_TIER_2_LOWER_GHG_TENTHS_KG_CO2E_PER_KG_H2: u64 = 15;
pub const IRC_45V_TIER_3_LOWER_GHG_HUNDREDTHS_KG_CO2E_PER_KG_H2: u64 = 45;
pub const IRC_45V_TIER_1_APPLICABLE_PERCENTAGE_BPS: u64 = 2_000;
pub const IRC_45V_TIER_2_APPLICABLE_PERCENTAGE_BPS: u64 = 2_500;
pub const IRC_45V_TIER_3_APPLICABLE_PERCENTAGE_BPS: u64 = 3_340;
pub const IRC_45V_TIER_4_APPLICABLE_PERCENTAGE_BPS: u64 = 10_000;
pub const IRC_45V_CREDIT_PERIOD_YEARS: u32 = 10;
pub const IRC_45V_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_45V_DIRECT_PAY_CROSS_REFERENCE_SECTION: u32 = 6417;
pub const IRC_45V_DIRECT_PAY_TAXABLE_ENTITY_YEARS: u32 = 5;
pub const IRC_45V_TRANSFERABILITY_CROSS_REFERENCE_SECTION: u32 = 6418;
pub const IRC_45V_FORM_NUMBER: u32 = 7210;
pub const IRC_45V_FINAL_REGS_PUBLICATION_DATE_YEAR: u32 = 2025;
pub const IRC_45V_FINAL_REGS_PUBLICATION_DATE_MONTH: u32 = 1;
pub const IRC_45V_FINAL_REGS_PUBLICATION_DATE_DAY: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacedInServiceDateStatus {
    PlacedInServiceOnOrBeforeDecember31_2022PreEffective,
    PlacedInServiceAfterDecember31_2022PostEffectiveEligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BeginningOfConstructionStatus {
    BocBeforeJanuary1_2028PreObbbaBocCutoff,
    BocOnOrAfterJanuary1_2028PostObbbaBocCutoff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleGhgEmissionsTier {
    Tier1Between2Point5And4KgCo2ePerKgH2,
    Tier2Between1Point5And2Point5KgCo2ePerKgH2,
    Tier3Between0Point45And1Point5KgCo2ePerKgH2,
    Tier4LessThan0Point45KgCo2ePerKgH2,
    NonQualifyingAboveFourKgCo2ePerKgH2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrevailingWageApprenticeshipStatus {
    PwaRequirementsMetEligibleForBonusMultiplier,
    PwaRequirementsNotMetBaseRateOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreePillarsComplianceStatus {
    ThreePillarsMet,
    TemporalMatchingNotMet,
    DeliverabilityNotMet,
    AdditionalityIncrementalityNotMet,
    NotApplicableNonElectrolysisHydrogen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    BaseCreditAmountUnderSection45VA,
    BonusCreditAmountForPwaUnderSection45VB2B,
    LifecycleGhgEmissionsTierUnderSection45VB2,
    QualifiedCleanHydrogenDefinitionUnderSection45VC,
    ThreePillarsComplianceUnderFinalRegulations,
    TenYearCreditPeriodUnderSection45VA2,
    ObbbaPhaseOutUnderSection70511,
    DirectPayElectionUnderSection6417,
    TransferabilityElectionUnderSection6418,
    FormFilingUnderForm7210,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section45VMode {
    NotApplicablePlacedInServiceOnOrBeforeDecember31_2022PreEffective,
    NotApplicableBocOnOrAfterJanuary1_2028PostObbbaCutoff,
    NotApplicableNonQualifyingHydrogenAboveFourKgCo2eEmissions,
    NotApplicableThreePillarsNotMetForElectrolysisHydrogen,
    CompliantBaseCreditAmount,
    CompliantBonusCreditAmountAtFivexMultiplierWithPwa,
    CompliantTier1ApplicablePercentage20Percent,
    CompliantTier2ApplicablePercentage25Percent,
    CompliantTier3ApplicablePercentage33Point4Percent,
    CompliantTier4ApplicablePercentage100Percent,
    CompliantQualifiedCleanHydrogen,
    CompliantThreePillarsMet,
    CompliantTenYearCreditPeriodYearWithinWindow,
    CompliantBocBeforeJanuary1_2028PreObbbaCutoff,
    CompliantDirectPayElectionMade,
    CompliantTransferabilityElectionMade,
    CompliantForm7210FiledCorrectly,
    ViolationPwaBonusMultiplierClaimedWithoutMeetingPwaRequirements,
    ViolationCreditClaimedOutsideTenYearCreditPeriod,
    ViolationForm7210NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub placed_in_service_date_status: PlacedInServiceDateStatus,
    pub beginning_of_construction_status: BeginningOfConstructionStatus,
    pub lifecycle_ghg_emissions_tier: LifecycleGhgEmissionsTier,
    pub pwa_status: PrevailingWageApprenticeshipStatus,
    pub three_pillars_compliance_status: ThreePillarsComplianceStatus,
    pub compliance_aspect: ComplianceAspect,
    pub kilograms_qch_produced: u64,
    pub credit_year_number_within_window: u32,
    pub direct_pay_election_made: bool,
    pub transferability_election_made: bool,
    pub form_7210_filed_correctly: bool,
    pub claimed_pwa_bonus_multiplier: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section45VMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section45VInput = Input;
pub type Section45VOutput = Output;
pub type Section45VResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 45V Clean Hydrogen Production Credit added by Section 13204 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818); signed by President Joe Biden on August 16, 2022; effective for clean hydrogen produced at qualifying facilities placed in service after December 31, 2022".to_string(),
        "IRC § 45V(a) Base Credit Amount — credit equals $0.60 PER KILOGRAM (kg) of qualified clean hydrogen produced × APPLICABLE PERCENTAGE based on lifecycle GHG emissions rate; $0.60 amount ADJUSTED FOR INFLATION annually".to_string(),
        "IRC § 45V(b)(2) Applicable Percentage — Four-Tier Lifecycle Emissions Structure — TIER 1 (2.5-4 kg CO2e/kg H2) = 20 PERCENT applicable percentage ($0.12/kg base); TIER 2 (1.5-2.5 kg CO2e/kg H2) = 25 PERCENT applicable percentage ($0.15/kg base); TIER 3 (0.45-1.5 kg CO2e/kg H2) = 33.4 PERCENT applicable percentage ($0.20/kg base, rounded); TIER 4 (less than 0.45 kg CO2e/kg H2) = 100 PERCENT applicable percentage ($0.60/kg base — maximum tier)".to_string(),
        "IRC § 45V(b)(2)(B) PWA Bonus 5x Multiplier — credit MULTIPLIED BY 5 if taxpayer satisfies prevailing wage and apprenticeship (PWA) requirements during construction and for the 10-year credit period; with PWA bonus, maximum credit reaches $3.00 per kg (Tier 4 × 5 = $0.60 × 5 = $3.00/kg)".to_string(),
        "IRC § 45V(b)(2)(C) Lifecycle GHG Emissions — Well-to-Gate Methodology — lifecycle emissions include emissions through the point of production (WELL-TO-GATE) as determined under the 45VH2-GREET MODEL developed by Argonne National Laboratory (Greenhouse gases, Regulated Emissions, and Energy use in Transportation model)".to_string(),
        "IRC § 45V(a)(2) 10-Year Credit Period — credit available for the 10-YEAR PERIOD BEGINNING ON DATE FACILITY PLACED IN SERVICE".to_string(),
        "IRC § 45V(c) Qualified Clean Hydrogen Definition — hydrogen which is produced (i) through a process that results in a lifecycle GHG emissions rate of NOT GREATER THAN 4 kg CO2e PER kg H2; (ii) PROVIDED THROUGH VERIFICATION as conforming to applicable certification requirements; AND (iii) produced in the United States by the taxpayer in the ordinary course of a trade or business of the taxpayer".to_string(),
        "IRC § 45V(c)(2) Three Pillars Compliance (Final Regulations) — 45VH2-GREET MODEL applies the THREE PILLARS compliance framework for electrolysis hydrogen produced using grid electricity: (1) TEMPORAL MATCHING (hourly or annual matching of clean electricity and hydrogen production); (2) DELIVERABILITY (clean electricity must be deliverable to the hydrogen production facility); AND (3) ADDITIONALITY / INCREMENTALITY (clean electricity must be from new generation sources commissioned within 36 months of the hydrogen facility's placed-in-service date)".to_string(),
        "OBBBA 2025 § 70511 Phase-Out — Begin-of-Construction Cutoff — § 45V credit phased out by JANUARY 1, 2028; facilities must BEGIN CONSTRUCTION BEFORE JANUARY 1, 2028 to qualify (a 5-YEAR ACCELERATION of the original IRA 2033 construction cutoff under § 45V(d)(2))".to_string(),
        "IRC § 45V(d) Original IRA 2022 Phase-Out — under original IRA 2022, credit available for clean hydrogen produced at qualified clean hydrogen production facilities the construction of which begins BEFORE JANUARY 1, 2033 — accelerated by 5 years under OBBBA 2025 § 70511".to_string(),
        "IRC § 6417 Direct Pay Election — § 45V credit eligible for DIRECT PAY ELECTION for tax-exempt entities (501(c)(3), state/local/tribal governments, rural electric coops); special 5-YEAR DIRECT PAY ELECTION available for taxable entities for the first 5 years of the 10-year credit period".to_string(),
        "IRC § 6418 Transferability Election — § 45V credit eligible for TRANSFERABILITY ELECTION; taxpayer may sell credit to an unrelated third party for cash".to_string(),
        "Form 7210 (Clean Hydrogen Production Credit) — required to claim the § 45V credit beginning with tax year 2023; 2025 instructions reflect § 6417 5-year direct pay election and § 6418 transferability".to_string(),
        "Final Regulations T.D. 10023 (January 10, 2025) — Treasury and IRS issued final regulations under § 45V published in the Federal Register on January 10, 2025; provide rules for lifecycle GHG emissions rate determination, three pillars compliance, certification, electrolysis facility eligibility, modification/retrofitting rules".to_string(),
        "U.S. Department of Energy + Federal Register + Cornell LII + King & Spalding + IRA Tracker + Novogradac + U.S. Treasury + Holland & Knight + IRS + Accounting Insights + PwC — practitioner overviews of § 45V".to_string(),
    ];

    if input.placed_in_service_date_status
        == PlacedInServiceDateStatus::PlacedInServiceOnOrBeforeDecember31_2022PreEffective
    {
        return Output {
            mode: Section45VMode::NotApplicablePlacedInServiceOnOrBeforeDecember31_2022PreEffective,
            statutory_basis: "IRA 2022 § 13204 effective date — § 45V applies only to qualifying facilities placed in service after December 31, 2022".to_string(),
            notes: "NOT APPLICABLE: facility placed in service on or before December 31, 2022 (pre-effective date); § 45V credit unavailable.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.beginning_of_construction_status
        == BeginningOfConstructionStatus::BocOnOrAfterJanuary1_2028PostObbbaBocCutoff
    {
        return Output {
            mode: Section45VMode::NotApplicableBocOnOrAfterJanuary1_2028PostObbbaCutoff,
            statutory_basis: "OBBBA 2025 § 70511 — § 45V phase-out; facilities must begin construction before January 1, 2028".to_string(),
            notes: "NOT APPLICABLE: facility began construction on or after January 1, 2028; § 45V credit phased out by Section 70511 of One Big Beautiful Bill Act of 2025 (Public Law 119-21); original IRA 2022 January 1, 2033 BOC cutoff accelerated by 5 years.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.lifecycle_ghg_emissions_tier
        == LifecycleGhgEmissionsTier::NonQualifyingAboveFourKgCo2ePerKgH2
    {
        return Output {
            mode: Section45VMode::NotApplicableNonQualifyingHydrogenAboveFourKgCo2eEmissions,
            statutory_basis: "IRC § 45V(c) — hydrogen with lifecycle GHG emissions above 4 kg CO2e/kg H2 does not qualify as clean hydrogen".to_string(),
            notes: "NOT APPLICABLE: hydrogen has lifecycle GHG emissions above 4 kg CO2e/kg H2 (NOT GREATER THAN 4 threshold); does not meet § 45V(c) qualified clean hydrogen definition.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    let is_electrolysis_using_grid_electricity = matches!(
        input.three_pillars_compliance_status,
        ThreePillarsComplianceStatus::TemporalMatchingNotMet
            | ThreePillarsComplianceStatus::DeliverabilityNotMet
            | ThreePillarsComplianceStatus::AdditionalityIncrementalityNotMet
    );

    if is_electrolysis_using_grid_electricity {
        return Output {
            mode: Section45VMode::NotApplicableThreePillarsNotMetForElectrolysisHydrogen,
            statutory_basis: "Final Regulations T.D. 10023 — § 45V three pillars compliance not met for electrolysis hydrogen using grid electricity".to_string(),
            notes: "NOT APPLICABLE: electrolysis hydrogen using grid electricity must satisfy three pillars (temporal matching + deliverability + additionality / incrementality); one or more pillar not met; lifecycle emissions counted at full grid mix.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::BaseCreditAmountUnderSection45VA => {
            let applicable_percentage_bps =
                applicable_percentage_bps_for_tier(input.lifecycle_ghg_emissions_tier);
            let computed = (u128::from(input.kilograms_qch_produced)
                * u128::from(IRC_45V_BASE_RATE_CENTS_PER_KG)
                * u128::from(applicable_percentage_bps)
                / u128::from(IRC_45V_CENTS_PER_DOLLAR)
                / u128::from(IRC_45V_BASIS_POINT_DENOMINATOR)) as u64;
            Output {
                mode: Section45VMode::CompliantBaseCreditAmount,
                statutory_basis: "IRC § 45V(a) + (b)(2) — base credit at $0.60/kg × tiered applicable percentage".to_string(),
                notes: format!(
                    "COMPLIANT: base credit at $0.60/kg × {pct_bps} bps tier applicable percentage × {kg} kg = ${computed}.",
                    pct_bps = applicable_percentage_bps,
                    kg = input.kilograms_qch_produced,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::BonusCreditAmountForPwaUnderSection45VB2B => {
            if input.claimed_pwa_bonus_multiplier
                && input.pwa_status
                    == PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly
            {
                return Output {
                    mode: Section45VMode::ViolationPwaBonusMultiplierClaimedWithoutMeetingPwaRequirements,
                    statutory_basis: "IRC § 45V(b)(2)(B) — PWA bonus 5x multiplier requires prevailing wage and apprenticeship compliance".to_string(),
                    notes: "VIOLATION: PWA bonus 5x multiplier claimed but prevailing wage and apprenticeship requirements not met; only base rate (without 5x multiplier) available.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                };
            }
            let applicable_percentage_bps =
                applicable_percentage_bps_for_tier(input.lifecycle_ghg_emissions_tier);
            let computed = (u128::from(input.kilograms_qch_produced)
                * u128::from(IRC_45V_BASE_RATE_CENTS_PER_KG)
                * u128::from(applicable_percentage_bps)
                * u128::from(IRC_45V_BONUS_MULTIPLIER)
                / u128::from(IRC_45V_CENTS_PER_DOLLAR)
                / u128::from(IRC_45V_BASIS_POINT_DENOMINATOR)) as u64;
            Output {
                mode: Section45VMode::CompliantBonusCreditAmountAtFivexMultiplierWithPwa,
                statutory_basis: "IRC § 45V(b)(2)(B) — bonus credit at base × 5x multiplier with PWA compliance".to_string(),
                notes: format!(
                    "COMPLIANT: bonus credit at $0.60/kg × {pct_bps} bps tier applicable percentage × 5x PWA multiplier × {kg} kg = ${computed}; PWA requirements satisfied during construction and 10-year credit period.",
                    pct_bps = applicable_percentage_bps,
                    kg = input.kilograms_qch_produced,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::LifecycleGhgEmissionsTierUnderSection45VB2 => match input
            .lifecycle_ghg_emissions_tier
        {
            LifecycleGhgEmissionsTier::Tier1Between2Point5And4KgCo2ePerKgH2 => Output {
                mode: Section45VMode::CompliantTier1ApplicablePercentage20Percent,
                statutory_basis: "IRC § 45V(b)(2) — Tier 1 lifecycle GHG emissions rate (2.5-4 kg CO2e/kg H2) = 20 % applicable percentage".to_string(),
                notes: "COMPLIANT: Tier 1 lifecycle GHG emissions rate (2.5-4 kg CO2e/kg H2); 20 % applicable percentage; $0.12/kg base credit before PWA bonus.".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
            LifecycleGhgEmissionsTier::Tier2Between1Point5And2Point5KgCo2ePerKgH2 => Output {
                mode: Section45VMode::CompliantTier2ApplicablePercentage25Percent,
                statutory_basis: "IRC § 45V(b)(2) — Tier 2 lifecycle GHG emissions rate (1.5-2.5 kg CO2e/kg H2) = 25 % applicable percentage".to_string(),
                notes: "COMPLIANT: Tier 2 lifecycle GHG emissions rate (1.5-2.5 kg CO2e/kg H2); 25 % applicable percentage; $0.15/kg base credit before PWA bonus.".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
            LifecycleGhgEmissionsTier::Tier3Between0Point45And1Point5KgCo2ePerKgH2 => Output {
                mode: Section45VMode::CompliantTier3ApplicablePercentage33Point4Percent,
                statutory_basis: "IRC § 45V(b)(2) — Tier 3 lifecycle GHG emissions rate (0.45-1.5 kg CO2e/kg H2) = 33.4 % applicable percentage".to_string(),
                notes: "COMPLIANT: Tier 3 lifecycle GHG emissions rate (0.45-1.5 kg CO2e/kg H2); 33.4 % applicable percentage; $0.20/kg base credit before PWA bonus.".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
            LifecycleGhgEmissionsTier::Tier4LessThan0Point45KgCo2ePerKgH2 => Output {
                mode: Section45VMode::CompliantTier4ApplicablePercentage100Percent,
                statutory_basis: "IRC § 45V(b)(2) — Tier 4 lifecycle GHG emissions rate (< 0.45 kg CO2e/kg H2) = 100 % applicable percentage (maximum tier)".to_string(),
                notes: "COMPLIANT: Tier 4 lifecycle GHG emissions rate (< 0.45 kg CO2e/kg H2); 100 % applicable percentage (maximum tier); $0.60/kg base credit before PWA bonus; $3.00/kg with PWA 5x multiplier.".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
            LifecycleGhgEmissionsTier::NonQualifyingAboveFourKgCo2ePerKgH2 => unreachable!(),
        },
        ComplianceAspect::QualifiedCleanHydrogenDefinitionUnderSection45VC => Output {
            mode: Section45VMode::CompliantQualifiedCleanHydrogen,
            statutory_basis: "IRC § 45V(c) — qualified clean hydrogen (QCH) definition met (lifecycle GHG emissions not greater than 4 kg CO2e/kg H2 + verification + produced in United States in trade or business)".to_string(),
            notes: "COMPLIANT: hydrogen meets § 45V(c) qualified clean hydrogen definition (lifecycle GHG emissions not greater than 4 kg CO2e/kg H2 + verification + produced in United States in ordinary course of trade or business).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::ThreePillarsComplianceUnderFinalRegulations => Output {
            mode: Section45VMode::CompliantThreePillarsMet,
            statutory_basis: "Final Regulations T.D. 10023 — three pillars compliance met (temporal matching + deliverability + additionality / incrementality)".to_string(),
            notes: "COMPLIANT: electrolysis hydrogen using grid electricity satisfies three pillars compliance framework (temporal matching + deliverability + additionality / incrementality) under § 45V final regulations.".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::TenYearCreditPeriodUnderSection45VA2 => {
            if input.credit_year_number_within_window == 0
                || input.credit_year_number_within_window > IRC_45V_CREDIT_PERIOD_YEARS
            {
                Output {
                    mode: Section45VMode::ViolationCreditClaimedOutsideTenYearCreditPeriod,
                    statutory_basis: "IRC § 45V(a)(2) — credit available for 10-year period beginning on date facility placed in service".to_string(),
                    notes: format!(
                        "VIOLATION: credit claimed for year {y} which is outside the 10-year credit period (years 1-10 only); § 45V credit unavailable for this year.",
                        y = input.credit_year_number_within_window,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45VMode::CompliantTenYearCreditPeriodYearWithinWindow,
                    statutory_basis: "IRC § 45V(a)(2) — credit claimed for year within 10-year credit period beginning on date facility placed in service".to_string(),
                    notes: format!(
                        "COMPLIANT: credit claimed for year {y} within 10-year credit period under § 45V(a)(2).",
                        y = input.credit_year_number_within_window,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::ObbbaPhaseOutUnderSection70511 => Output {
            mode: Section45VMode::CompliantBocBeforeJanuary1_2028PreObbbaCutoff,
            statutory_basis: "OBBBA 2025 § 70511 — facility began construction before January 1, 2028 (pre-OBBBA cutoff)".to_string(),
            notes: "COMPLIANT: facility began construction before January 1, 2028; § 45V credit available under OBBBA 2025 transition window; original IRA 2022 January 1, 2033 BOC cutoff accelerated by 5 years.".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::DirectPayElectionUnderSection6417 => {
            if input.direct_pay_election_made {
                Output {
                    mode: Section45VMode::CompliantDirectPayElectionMade,
                    statutory_basis: "IRC § 6417 — direct pay election made for § 45V credit (5-year availability for taxable entities; permanent for tax-exempt entities)".to_string(),
                    notes: "COMPLIANT: direct pay election under § 6417 made for § 45V credit; allows monetization as cash refund.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45VMode::CompliantBaseCreditAmount,
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
                    mode: Section45VMode::CompliantTransferabilityElectionMade,
                    statutory_basis: "IRC § 6418 — transferability election made for § 45V credit (sale to unrelated third party for cash)".to_string(),
                    notes: "COMPLIANT: transferability election under § 6418 made for § 45V credit; allows sale of credit to unrelated third party for cash.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45VMode::CompliantBaseCreditAmount,
                    statutory_basis: "IRC § 6418 — transferability election not made; credit claimed by taxpayer".to_string(),
                    notes: "COMPLIANT: transferability election under § 6418 not made; credit claimed by taxpayer as offset to federal income tax liability.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::FormFilingUnderForm7210 => {
            if input.form_7210_filed_correctly {
                Output {
                    mode: Section45VMode::CompliantForm7210FiledCorrectly,
                    statutory_basis: "Form 7210 — Clean Hydrogen Production Credit form required to claim § 45V credit".to_string(),
                    notes: "COMPLIANT: Form 7210 filed correctly to claim § 45V credit for tax year.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45VMode::ViolationForm7210NotFiledOrIncorrect,
                    statutory_basis: "Form 7210 filing required to claim § 45V credit".to_string(),
                    notes: "VIOLATION: Form 7210 not filed or incorrectly filed; § 45V credit may be disallowed.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
    }
}

fn applicable_percentage_bps_for_tier(tier: LifecycleGhgEmissionsTier) -> u64 {
    match tier {
        LifecycleGhgEmissionsTier::Tier1Between2Point5And4KgCo2ePerKgH2 => {
            IRC_45V_TIER_1_APPLICABLE_PERCENTAGE_BPS
        }
        LifecycleGhgEmissionsTier::Tier2Between1Point5And2Point5KgCo2ePerKgH2 => {
            IRC_45V_TIER_2_APPLICABLE_PERCENTAGE_BPS
        }
        LifecycleGhgEmissionsTier::Tier3Between0Point45And1Point5KgCo2ePerKgH2 => {
            IRC_45V_TIER_3_APPLICABLE_PERCENTAGE_BPS
        }
        LifecycleGhgEmissionsTier::Tier4LessThan0Point45KgCo2ePerKgH2 => {
            IRC_45V_TIER_4_APPLICABLE_PERCENTAGE_BPS
        }
        LifecycleGhgEmissionsTier::NonQualifyingAboveFourKgCo2ePerKgH2 => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            placed_in_service_date_status:
                PlacedInServiceDateStatus::PlacedInServiceAfterDecember31_2022PostEffectiveEligible,
            beginning_of_construction_status:
                BeginningOfConstructionStatus::BocBeforeJanuary1_2028PreObbbaBocCutoff,
            lifecycle_ghg_emissions_tier:
                LifecycleGhgEmissionsTier::Tier4LessThan0Point45KgCo2ePerKgH2,
            pwa_status:
                PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusMultiplier,
            three_pillars_compliance_status: ThreePillarsComplianceStatus::ThreePillarsMet,
            compliance_aspect: ComplianceAspect::BaseCreditAmountUnderSection45VA,
            kilograms_qch_produced: 1_000_000,
            credit_year_number_within_window: 1,
            direct_pay_election_made: false,
            transferability_election_made: false,
            form_7210_filed_correctly: true,
            claimed_pwa_bonus_multiplier: false,
        }
    }

    #[test]
    fn pre_effective_placed_in_service_not_applicable() {
        let mut input = baseline_input();
        input.placed_in_service_date_status =
            PlacedInServiceDateStatus::PlacedInServiceOnOrBeforeDecember31_2022PreEffective;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::NotApplicablePlacedInServiceOnOrBeforeDecember31_2022PreEffective
        );
    }

    #[test]
    fn boc_after_january1_2028_not_applicable() {
        let mut input = baseline_input();
        input.beginning_of_construction_status =
            BeginningOfConstructionStatus::BocOnOrAfterJanuary1_2028PostObbbaBocCutoff;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::NotApplicableBocOnOrAfterJanuary1_2028PostObbbaCutoff
        );
    }

    #[test]
    fn non_qualifying_hydrogen_above_4_kg_co2e_not_applicable() {
        let mut input = baseline_input();
        input.lifecycle_ghg_emissions_tier =
            LifecycleGhgEmissionsTier::NonQualifyingAboveFourKgCo2ePerKgH2;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::NotApplicableNonQualifyingHydrogenAboveFourKgCo2eEmissions
        );
    }

    #[test]
    fn three_pillars_not_met_for_electrolysis_not_applicable() {
        let mut input = baseline_input();
        input.three_pillars_compliance_status =
            ThreePillarsComplianceStatus::TemporalMatchingNotMet;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::NotApplicableThreePillarsNotMetForElectrolysisHydrogen
        );
    }

    #[test]
    fn tier_4_base_credit_at_sixty_cents_per_kg_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountUnderSection45VA;
        input.lifecycle_ghg_emissions_tier =
            LifecycleGhgEmissionsTier::Tier4LessThan0Point45KgCo2ePerKgH2;
        input.kilograms_qch_produced = 1_000_000;
        let out = check(&input);
        assert_eq!(out.mode, Section45VMode::CompliantBaseCreditAmount);
        assert_eq!(out.computed_credit_dollars, 600_000);
    }

    #[test]
    fn tier_4_bonus_credit_at_three_dollars_per_kg_with_pwa_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45VB2B;
        input.lifecycle_ghg_emissions_tier =
            LifecycleGhgEmissionsTier::Tier4LessThan0Point45KgCo2ePerKgH2;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusMultiplier;
        input.claimed_pwa_bonus_multiplier = true;
        input.kilograms_qch_produced = 1_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::CompliantBonusCreditAmountAtFivexMultiplierWithPwa
        );
        assert_eq!(out.computed_credit_dollars, 3_000_000);
    }

    #[test]
    fn tier_1_base_credit_at_twelve_cents_per_kg_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountUnderSection45VA;
        input.lifecycle_ghg_emissions_tier =
            LifecycleGhgEmissionsTier::Tier1Between2Point5And4KgCo2ePerKgH2;
        input.kilograms_qch_produced = 1_000_000;
        let out = check(&input);
        assert_eq!(out.mode, Section45VMode::CompliantBaseCreditAmount);
        assert_eq!(out.computed_credit_dollars, 120_000);
    }

    #[test]
    fn tier_2_base_credit_at_fifteen_cents_per_kg_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountUnderSection45VA;
        input.lifecycle_ghg_emissions_tier =
            LifecycleGhgEmissionsTier::Tier2Between1Point5And2Point5KgCo2ePerKgH2;
        input.kilograms_qch_produced = 1_000_000;
        let out = check(&input);
        assert_eq!(out.mode, Section45VMode::CompliantBaseCreditAmount);
        assert_eq!(out.computed_credit_dollars, 150_000);
    }

    #[test]
    fn tier_3_base_credit_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountUnderSection45VA;
        input.lifecycle_ghg_emissions_tier =
            LifecycleGhgEmissionsTier::Tier3Between0Point45And1Point5KgCo2ePerKgH2;
        input.kilograms_qch_produced = 1_000_000;
        let out = check(&input);
        assert_eq!(out.mode, Section45VMode::CompliantBaseCreditAmount);
        assert_eq!(out.computed_credit_dollars, 200_400);
    }

    #[test]
    fn pwa_bonus_multiplier_claimed_without_meeting_requirements_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45VB2B;
        input.pwa_status = PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly;
        input.claimed_pwa_bonus_multiplier = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::ViolationPwaBonusMultiplierClaimedWithoutMeetingPwaRequirements
        );
    }

    #[test]
    fn tier_1_applicable_percentage_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LifecycleGhgEmissionsTierUnderSection45VB2;
        input.lifecycle_ghg_emissions_tier =
            LifecycleGhgEmissionsTier::Tier1Between2Point5And4KgCo2ePerKgH2;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::CompliantTier1ApplicablePercentage20Percent
        );
    }

    #[test]
    fn tier_2_applicable_percentage_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LifecycleGhgEmissionsTierUnderSection45VB2;
        input.lifecycle_ghg_emissions_tier =
            LifecycleGhgEmissionsTier::Tier2Between1Point5And2Point5KgCo2ePerKgH2;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::CompliantTier2ApplicablePercentage25Percent
        );
    }

    #[test]
    fn tier_3_applicable_percentage_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LifecycleGhgEmissionsTierUnderSection45VB2;
        input.lifecycle_ghg_emissions_tier =
            LifecycleGhgEmissionsTier::Tier3Between0Point45And1Point5KgCo2ePerKgH2;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::CompliantTier3ApplicablePercentage33Point4Percent
        );
    }

    #[test]
    fn tier_4_applicable_percentage_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LifecycleGhgEmissionsTierUnderSection45VB2;
        input.lifecycle_ghg_emissions_tier =
            LifecycleGhgEmissionsTier::Tier4LessThan0Point45KgCo2ePerKgH2;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::CompliantTier4ApplicablePercentage100Percent
        );
    }

    #[test]
    fn qualified_clean_hydrogen_definition_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::QualifiedCleanHydrogenDefinitionUnderSection45VC;
        let out = check(&input);
        assert_eq!(out.mode, Section45VMode::CompliantQualifiedCleanHydrogen);
    }

    #[test]
    fn three_pillars_compliance_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThreePillarsComplianceUnderFinalRegulations;
        input.three_pillars_compliance_status = ThreePillarsComplianceStatus::ThreePillarsMet;
        let out = check(&input);
        assert_eq!(out.mode, Section45VMode::CompliantThreePillarsMet);
    }

    #[test]
    fn ten_year_credit_period_year_one_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenYearCreditPeriodUnderSection45VA2;
        input.credit_year_number_within_window = 1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::CompliantTenYearCreditPeriodYearWithinWindow
        );
    }

    #[test]
    fn ten_year_credit_period_year_ten_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenYearCreditPeriodUnderSection45VA2;
        input.credit_year_number_within_window = 10;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::CompliantTenYearCreditPeriodYearWithinWindow
        );
    }

    #[test]
    fn ten_year_credit_period_year_eleven_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenYearCreditPeriodUnderSection45VA2;
        input.credit_year_number_within_window = 11;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::ViolationCreditClaimedOutsideTenYearCreditPeriod
        );
    }

    #[test]
    fn direct_pay_election_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DirectPayElectionUnderSection6417;
        input.direct_pay_election_made = true;
        let out = check(&input);
        assert_eq!(out.mode, Section45VMode::CompliantDirectPayElectionMade);
    }

    #[test]
    fn transferability_election_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TransferabilityElectionUnderSection6418;
        input.transferability_election_made = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::CompliantTransferabilityElectionMade
        );
    }

    #[test]
    fn form_7210_filed_correctly_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm7210;
        input.form_7210_filed_correctly = true;
        let out = check(&input);
        assert_eq!(out.mode, Section45VMode::CompliantForm7210FiledCorrectly);
    }

    #[test]
    fn form_7210_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm7210;
        input.form_7210_filed_correctly = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45VMode::ViolationForm7210NotFiledOrIncorrect
        );
    }

    #[test]
    fn constants_pin_section_45v_legislative_phases_and_credit_structure() {
        assert_eq!(IRC_45V_IRA_ENACTMENT_DATE_YEAR, 2022);
        assert_eq!(IRC_45V_IRA_PUBLIC_LAW_CONGRESS, 117);
        assert_eq!(IRC_45V_IRA_PUBLIC_LAW_ENACTMENT, 169);
        assert_eq!(IRC_45V_IRA_ENABLING_SECTION, 13204);
        assert_eq!(IRC_45V_EFFECTIVE_DATE_YEAR, 2023);
        assert_eq!(IRC_45V_OBBBA_ENACTMENT_DATE_YEAR, 2025);
        assert_eq!(IRC_45V_OBBBA_PUBLIC_LAW_CONGRESS, 119);
        assert_eq!(IRC_45V_OBBBA_PUBLIC_LAW_ENACTMENT, 21);
        assert_eq!(IRC_45V_OBBBA_ENABLING_SECTION, 70511);
        assert_eq!(IRC_45V_OBBBA_BOC_CUTOFF_YEAR, 2028);
        assert_eq!(IRC_45V_ORIGINAL_IRA_BOC_CUTOFF_YEAR, 2033);
        assert_eq!(IRC_45V_BASE_RATE_CENTS_PER_KG, 60);
        assert_eq!(IRC_45V_BONUS_MULTIPLIER, 5);
        assert_eq!(IRC_45V_TIER_1_APPLICABLE_PERCENTAGE_BPS, 2_000);
        assert_eq!(IRC_45V_TIER_2_APPLICABLE_PERCENTAGE_BPS, 2_500);
        assert_eq!(IRC_45V_TIER_3_APPLICABLE_PERCENTAGE_BPS, 3_340);
        assert_eq!(IRC_45V_TIER_4_APPLICABLE_PERCENTAGE_BPS, 10_000);
        assert_eq!(IRC_45V_CREDIT_PERIOD_YEARS, 10);
        assert_eq!(IRC_45V_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_45V_DIRECT_PAY_CROSS_REFERENCE_SECTION, 6417);
        assert_eq!(IRC_45V_DIRECT_PAY_TAXABLE_ENTITY_YEARS, 5);
        assert_eq!(IRC_45V_TRANSFERABILITY_CROSS_REFERENCE_SECTION, 6418);
        assert_eq!(IRC_45V_FORM_NUMBER, 7210);
        assert_eq!(IRC_45V_FINAL_REGS_PUBLICATION_DATE_YEAR, 2025);
        assert_eq!(IRC_45V_FINAL_REGS_PUBLICATION_DATE_MONTH, 1);
        assert_eq!(IRC_45V_FINAL_REGS_PUBLICATION_DATE_DAY, 10);
    }

    #[test]
    fn citations_pin_legislative_phases_and_obbba_phase_out_facts() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("IRC § 45V Clean Hydrogen Production Credit"));
        assert!(joined.contains("Section 13204 of the Inflation Reduction Act of 2022"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("August 16, 2022"));
        assert!(joined.contains("December 31, 2022"));
        assert!(joined.contains("$0.60 PER KILOGRAM"));
        assert!(joined.contains("APPLICABLE PERCENTAGE"));
        assert!(joined.contains("TIER 1"));
        assert!(joined.contains("TIER 2"));
        assert!(joined.contains("TIER 3"));
        assert!(joined.contains("TIER 4"));
        assert!(joined.contains("20 PERCENT"));
        assert!(joined.contains("25 PERCENT"));
        assert!(joined.contains("33.4 PERCENT"));
        assert!(joined.contains("100 PERCENT"));
        assert!(joined.contains("MULTIPLIED BY 5"));
        assert!(joined.contains("$3.00 per kg"));
        assert!(joined.contains("WELL-TO-GATE"));
        assert!(joined.contains("45VH2-GREET MODEL"));
        assert!(joined.contains("Argonne National Laboratory"));
        assert!(joined.contains("10-YEAR PERIOD"));
        assert!(joined.contains("NOT GREATER THAN 4 kg CO2e PER kg H2"));
        assert!(joined.contains("THREE PILLARS"));
        assert!(joined.contains("TEMPORAL MATCHING"));
        assert!(joined.contains("DELIVERABILITY"));
        assert!(joined.contains("ADDITIONALITY / INCREMENTALITY"));
        assert!(joined.contains("OBBBA 2025 § 70511"));
        assert!(joined.contains("JANUARY 1, 2028"));
        assert!(joined.contains("BEGIN CONSTRUCTION BEFORE JANUARY 1, 2028"));
        assert!(joined.contains("5-YEAR ACCELERATION"));
        assert!(joined.contains("JANUARY 1, 2033"));
        assert!(joined.contains("§ 6417"));
        assert!(joined.contains("DIRECT PAY ELECTION"));
        assert!(joined.contains("5-YEAR DIRECT PAY ELECTION"));
        assert!(joined.contains("§ 6418"));
        assert!(joined.contains("TRANSFERABILITY ELECTION"));
        assert!(joined.contains("Form 7210"));
        assert!(joined.contains("Final Regulations T.D. 10023"));
        assert!(joined.contains("January 10, 2025"));
    }

    #[test]
    fn saturating_overflow_defense_at_u64_max_kg() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountUnderSection45VA;
        input.kilograms_qch_produced = u64::MAX;
        let out = check(&input);
        let _ = out.computed_credit_dollars;
    }
}
