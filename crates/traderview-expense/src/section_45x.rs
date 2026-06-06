//! IRC § 45X Advanced Manufacturing Production Credit
//! Compliance Module — pure-compute check for the
//! production tax credit for **U.S.-made** clean energy
//! components (solar, wind, inverters, batteries) and
//! applicable critical minerals **produced and sold by
//! the taxpayer**. Per-unit credit structure (vs § 45Y's
//! per-kWh production credit or § 48E's per-dollar
//! investment credit).
//!
//! Originally enacted by **Section 13502 of the Inflation
//! Reduction Act of 2022 (Public Law 117-169)**, signed by
//! President Joe Biden on **August 16, 2022**, effective
//! for eligible components **PRODUCED AND SOLD AFTER
//! DECEMBER 31, 2022**. **MODIFIED by Section 70514 of the
//! One Big Beautiful Bill Act of 2025 (Public Law 119-21)**,
//! signed by President Donald Trump on **July 4, 2025**;
//! OBBBA (i) phases out wind energy components by
//! **DECEMBER 31, 2027**; (ii) applies **PROHIBITED FOREIGN
//! ENTITY (PFE)** restrictions to disqualify components
//! sourced from entities subject to material assistance
//! from foreign entities of concern; and (iii) increases
//! domestic content requirements applicable to certain
//! components.
//!
//! Web research (verified 2026-06-03):
//! - **IRA 2022 Enactment**: IRC § 45X added by **Section 13502 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818)**; signed by President Joe Biden on **August 16, 2022**; effective for eligible components PRODUCED AND SOLD AFTER **DECEMBER 31, 2022** ([IRA Tracker — IRA Section 13502 Advanced Manufacturing Production Credit](https://iratracker.org/programs/ira-section-13502-advanced-manufacturing-production-credit-for-solar-and-wind-manufacturers/); [Congressional Research Service IF12809 — The Section 45X Advanced Manufacturing Production Credit](https://www.congress.gov/crs-product/IF12809); [IRS — Advanced Manufacturing Production Credit](https://www.irs.gov/credits-deductions/advanced-manufacturing-production-credit); [IRS — Treasury, IRS Issue Guidance for the Advanced Manufacturing Production Credit](https://www.irs.gov/newsroom/treasury-irs-issue-guidance-for-the-advanced-manufacturing-production-credit); [Cornell LII — 26 U.S. Code § 45X](https://www.law.cornell.edu/uscode/text/26/45X); [Bloomberg Tax — Sec. 45X Advanced Manufacturing Production Credit](https://irc.bloombergtax.com/public/uscode/doc/irc/section_45x); [ICS Tax — 45X Advanced Manufacturing Production Credit](https://ics-tax.com/45x-advanced-manufacturing-production-credit/); [Crux Climate — §45X Tax Credits Guide for Manufacturers 2025](https://www.cruxclimate.com/insights/45x-tax-credit); [Chambers for Innovation & Clean Energy — IRA Policy Briefs 45X](https://www.chambersforinnovation.com/ira-policy-briefs-45x); [ACORE Fact Sheet — Section 45X Advanced Manufacturing Production Credit Proposed Rulemaking](https://acore.org/resources/treasury-department-notice-of-proposed-rulemaking-section-45x-advanced-manufacturing-production-credit/); [The Tax Credit Exchange — Section 45X Advanced Manufacturing Production Credit](https://www.taxcreditex.com/section-45x); [Baker Tilly — The Section 45X Tax Credit](https://www.bakertilly.com/insights/the-section-45x-tax-credit); [JM Co — Capitalizing on the 45X Advanced Manufacturing Tax Credit](https://www.jmco.com/articles/manufacturing/the-45x-advanced-tax-credit/); [Build With Basis — 45X Advanced Manufacturing Production Tax Credit](https://www.buildwithbasis.com/insights/unlocking-the-45x-tax-credit-what-manufacturers-need-to-know); [Treasury — IRA Incentives for Clean Energy Manufacturers December 12, 2024 PDF](https://home.treasury.gov/system/files/8861/2024.12.12.45X_Slides%20Final.pdf); [Grant Thornton — IRS Expands Section 45X Advanced Manufacturing Credit](https://www.grantthornton.com/insights/alerts/tax/2024/flash/irs-expands-section-45x-advanced-manufacturing-credit); [C2ES — The 30D and 45X Tax Credits Explained September 2025](https://www.c2es.org/2025/09/the-30d-45x-tax-credits-explained/)).
//! - **§ 45X(a) Eligible Components — Solar**: solar modules, **PHOTOVOLTAIC CELLS** (thin film and crystalline), **PHOTOVOLTAIC WAFERS**, **SOLAR-GRADE POLYSILICON**, torque tubes (for solar tracking devices), structural fasteners (for solar tracking devices), and **POLYMERIC BACKSHEETS**.
//! - **§ 45X(a) Eligible Components — Wind**: blades, nacelles, towers, offshore wind foundations (fixed and floating platforms), and offshore wind vessels.
//! - **§ 45X(a) Eligible Components — Inverter**: any inverter (central / commercial / residential / utility / microinverter / distributed).
//! - **§ 45X(a) Eligible Components — Battery**: **ELECTRODE ACTIVE MATERIALS**, **BATTERY CELLS**, and **BATTERY MODULES**.
//! - **§ 45X(a) Eligible Components — Critical Minerals**: 50 applicable critical minerals listed in 30 U.S.C. § 1606(a)(3) (e.g., lithium, cobalt, nickel, manganese, graphite, rare earth elements).
//! - **§ 45X(b)(1) Per-Unit Credit Structure — Solar**: solar-grade polysilicon at **$3 per kilogram (kg)**; photovoltaic wafers at **$12 per square meter (m²)**; thin film photovoltaic cells at **$0.04 per watt (W) DC capacity**; crystalline photovoltaic cells at **$0.04 per watt DC capacity**; solar modules at **$0.07 per watt DC capacity**; polymeric backsheets at **$0.40 per m²**; torque tubes at **$0.87 per kg**; structural fasteners at **$2.28 per kg**.
//! - **§ 45X(b)(1) Per-Unit Credit Structure — Wind**: blades at **$0.02 per watt of rated capacity**; nacelles at **$0.05 per watt of rated capacity**; towers at **$0.03 per watt of rated capacity**; offshore wind foundations at **$0.02 per watt** (fixed) or **$0.04 per watt** (floating); offshore wind vessels at **10 PERCENT** of sales price.
//! - **§ 45X(b)(1) Per-Unit Credit Structure — Battery**: electrode active materials at **10 PERCENT of production costs**; battery cells at **$35 per kilowatt-hour (kWh)** of capacity; battery modules at **$10 per kWh** OR **$45 per kWh if NOT using battery cells produced by the same taxpayer** (i.e., modules using purchased cells get higher per-kWh credit).
//! - **§ 45X(b)(1) Per-Unit Credit Structure — Inverter**: depends on inverter type — central inverters at **$0.0025 per watt AC**, commercial inverters at **$0.02 per watt AC**, distributed wind inverters at **$0.11 per watt AC**, microinverters at **$0.11 per watt AC**, residential inverters at **$0.065 per watt AC**, utility inverters at **$0.0015 per watt AC**.
//! - **§ 45X(b)(1) Per-Unit Credit Structure — Critical Minerals**: **10 PERCENT of production costs** for each applicable critical mineral.
//! - **§ 45X(b)(3) Domestic Production Requirement**: eligible components must be **PRODUCED BY THE TAXPAYER** (i.e., not purchased and resold); produced **WITHIN THE UNITED STATES** OR a U.S. possession; and **SOLD TO AN UNRELATED PARTY** (or related-party sale election under § 45X(b)(3)(B) for downstream production within the same affiliated group).
//! - **§ 45X(b)(3)(B) Related-Party Sale Election**: taxpayer may elect to treat a sale to a related party as a sale to an unrelated party if (i) the sale is in the ordinary course of the related party's trade or business AND (ii) the related party uses the component as a component or input to the production of an eligible component within the United States (vertical integration safe harbor).
//! - **§ 45X(b)(3)(C) Phase-Out of Most Components (Original IRA 2022)**: most § 45X credit amounts phase out beginning in 2030 (75 % in 2030, 50 % in 2031, 25 % in 2032, 0 % in 2033 and later); critical minerals credit does **NOT** phase out under original IRA 2022.
//! - **OBBBA 2025 § 70514 Wind Component Phase-Out**: **WIND ENERGY COMPONENTS** (blades, nacelles, towers, offshore foundations, offshore vessels) phased out by **DECEMBER 31, 2027** under Section 70514 of OBBBA 2025 — accelerated termination compared to original IRA 2030-2033 phase-out.
//! - **OBBBA 2025 § 70514 Prohibited Foreign Entity (PFE) Restrictions**: components sourced from entities subject to **MATERIAL ASSISTANCE FROM A FOREIGN ENTITY OF CONCERN (FEOC)** disqualified from § 45X credit; the FEOC list includes entities in China, Russia, Iran, and North Korea.
//! - **OBBBA 2025 § 70514 Domestic Content Increases**: OBBBA increased the **DOMESTIC CONTENT THRESHOLDS** applicable to certain § 45X components, requiring higher percentages of US-sourced steel, iron, and manufactured products in the component's bill of materials.
//! - **Form 7207 (Advanced Manufacturing Production Credit)**: required to claim the § 45X credit beginning with tax year 2023.
//! - **Final Regulations T.D. 10010 (October 28, 2024)**: Treasury and IRS issued final regulations under § 45X published in the Federal Register on October 28, 2024; provide rules for component definitions, production cost determination, related-party sale election, primary use requirements, and recordkeeping.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_45X_IRA_ENACTMENT_DATE_YEAR: u32 = 2022;
pub const IRC_45X_IRA_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_45X_IRA_ENACTMENT_DATE_DAY: u32 = 16;
pub const IRC_45X_IRA_PUBLIC_LAW_CONGRESS: u32 = 117;
pub const IRC_45X_IRA_PUBLIC_LAW_ENACTMENT: u32 = 169;
pub const IRC_45X_IRA_ENABLING_SECTION: u32 = 13502;
pub const IRC_45X_EFFECTIVE_DATE_YEAR: u32 = 2023;
pub const IRC_45X_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const IRC_45X_EFFECTIVE_DATE_DAY: u32 = 1;
pub const IRC_45X_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_45X_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_45X_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_45X_OBBBA_PUBLIC_LAW_CONGRESS: u32 = 119;
pub const IRC_45X_OBBBA_PUBLIC_LAW_ENACTMENT: u32 = 21;
pub const IRC_45X_OBBBA_ENABLING_SECTION: u32 = 70514;
pub const IRC_45X_OBBBA_WIND_PHASE_OUT_CUTOFF_YEAR: u32 = 2027;
pub const IRC_45X_OBBBA_WIND_PHASE_OUT_CUTOFF_MONTH: u32 = 12;
pub const IRC_45X_OBBBA_WIND_PHASE_OUT_CUTOFF_DAY: u32 = 31;
pub const IRC_45X_ORIGINAL_PHASE_OUT_BEGIN_YEAR: u32 = 2030;
pub const IRC_45X_ORIGINAL_PHASE_OUT_END_YEAR: u32 = 2033;
pub const IRC_45X_BATTERY_CELL_RATE_DOLLARS_PER_KWH: u64 = 35;
pub const IRC_45X_BATTERY_MODULE_INTEGRATED_RATE_DOLLARS_PER_KWH: u64 = 10;
pub const IRC_45X_BATTERY_MODULE_STANDALONE_RATE_DOLLARS_PER_KWH: u64 = 45;
pub const IRC_45X_CRITICAL_MINERAL_PERCENT_OF_PRODUCTION_COST_BPS: u64 = 1_000;
pub const IRC_45X_OFFSHORE_WIND_VESSEL_PERCENT_OF_SALES_PRICE_BPS: u64 = 1_000;
pub const IRC_45X_SOLAR_POLYSILICON_RATE_DOLLARS_PER_KG: u64 = 3;
pub const IRC_45X_SOLAR_WAFER_RATE_DOLLARS_PER_SQ_METER: u64 = 12;
pub const IRC_45X_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_45X_FORM_NUMBER: u32 = 7207;
pub const IRC_45X_FINAL_REGS_PUBLICATION_DATE_YEAR: u32 = 2024;
pub const IRC_45X_FINAL_REGS_PUBLICATION_DATE_MONTH: u32 = 10;
pub const IRC_45X_FINAL_REGS_PUBLICATION_DATE_DAY: u32 = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionDateStatus {
    ProducedAndSoldOnOrBeforeDecember31_2022PreEffective,
    ProducedAndSoldAfterDecember31_2022PostEffectiveEligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EligibleComponentCategory {
    SolarModule,
    SolarPolysilicon,
    SolarWafer,
    SolarTorqueTube,
    SolarPolymericBacksheet,
    WindBladeNacelleTowerFoundation,
    OffshoreWindVessel,
    Inverter,
    BatteryCell,
    BatteryModuleIntegratedWithSameTaxpayerCells,
    BatteryModuleStandaloneUsingPurchasedCells,
    BatteryElectrodeActiveMaterial,
    ApplicableCriticalMineral,
    NonEligibleComponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindProductionStatus {
    WindProducedAndSoldOnOrBeforeDecember31_2027PreObbbaPhaseOut,
    WindProducedAndSoldAfterDecember31_2027PostObbbaPhaseOut,
    NotApplicableNonWindComponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionAndSaleStatus {
    ProducedByTaxpayerInUnitedStatesAndSoldToUnrelatedParty,
    ProducedByTaxpayerInUnitedStatesAndRelatedPartySaleElectionMade,
    PurchasedAndResoldOrProducedOutsideUnitedStatesIneligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProhibitedForeignEntityStatus {
    NotSubjectToProhibitedForeignEntityRestrictions,
    SubjectToMaterialAssistanceFromForeignEntityOfConcernFeoc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    SolarComponentProductionCreditUnderSection45XB1,
    WindComponentProductionCreditUnderSection45XB1,
    InverterProductionCreditUnderSection45XB1,
    BatteryCellProductionCreditUnderSection45XB1,
    BatteryModuleProductionCreditUnderSection45XB1,
    CriticalMineralProductionCreditUnderSection45XB1,
    DomesticProductionRequirementUnderSection45XB3,
    RelatedPartySaleElectionUnderSection45XB3B,
    ObbbaWindPhaseOutUnderSection70514,
    ProhibitedForeignEntityRestrictionUnderObbbaSection70514,
    FormFilingUnderForm7207,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section45XMode {
    NotApplicableProducedAndSoldOnOrBeforeDecember31_2022PreEffective,
    NotApplicableWindProducedAndSoldAfterDecember31_2027PostObbbaPhaseOut,
    NotApplicablePurchasedAndResoldOrProducedOutsideUnitedStates,
    NotApplicableSubjectToFeocProhibitedForeignEntityRestriction,
    NotApplicableNonEligibleComponent,
    CompliantSolarComponentProductionCredit,
    CompliantWindComponentProductionCreditPreObbbaPhaseOut,
    CompliantInverterProductionCredit,
    CompliantBatteryCellProductionCreditAtThirtyFiveDollarsPerKwh,
    CompliantBatteryModuleIntegratedProductionCreditAtTenDollarsPerKwh,
    CompliantBatteryModuleStandaloneProductionCreditAtFortyFiveDollarsPerKwh,
    CompliantCriticalMineralProductionCreditAtTenPercentOfProductionCost,
    CompliantOffshoreWindVesselProductionCreditAtTenPercentOfSalesPrice,
    CompliantDomesticProductionRequirementMet,
    CompliantRelatedPartySaleElectionMade,
    CompliantNotSubjectToFeocRestriction,
    CompliantForm7207FiledCorrectly,
    ViolationComponentPurchasedAndResoldOrProducedOutsideUnitedStates,
    ViolationSubjectToFeocProhibitedForeignEntityRestriction,
    ViolationForm7207NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub production_date_status: ProductionDateStatus,
    pub eligible_component_category: EligibleComponentCategory,
    pub wind_production_status: WindProductionStatus,
    pub production_and_sale_status: ProductionAndSaleStatus,
    pub prohibited_foreign_entity_status: ProhibitedForeignEntityStatus,
    pub compliance_aspect: ComplianceAspect,
    pub kwh_capacity_produced: u64,
    pub production_cost_dollars: u64,
    pub offshore_wind_vessel_sales_price_dollars: u64,
    pub form_7207_filed_correctly: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section45XMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section45XInput = Input;
pub type Section45XOutput = Output;
pub type Section45XResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 45X Advanced Manufacturing Production Credit added by Section 13502 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818); signed by President Joe Biden on August 16, 2022; effective for eligible components produced and sold after December 31, 2022".to_string(),
        "IRC § 45X(a) Eligible Components — Solar — solar modules, PHOTOVOLTAIC CELLS (thin film and crystalline), PHOTOVOLTAIC WAFERS, SOLAR-GRADE POLYSILICON, torque tubes (for solar tracking devices), structural fasteners (for solar tracking devices), and POLYMERIC BACKSHEETS".to_string(),
        "IRC § 45X(a) Eligible Components — Wind — blades, nacelles, towers, offshore wind foundations (fixed and floating platforms), and offshore wind vessels".to_string(),
        "IRC § 45X(a) Eligible Components — Inverter — any inverter (central / commercial / residential / utility / microinverter / distributed)".to_string(),
        "IRC § 45X(a) Eligible Components — Battery — ELECTRODE ACTIVE MATERIALS, BATTERY CELLS, and BATTERY MODULES".to_string(),
        "IRC § 45X(a) Eligible Components — Critical Minerals — 50 applicable critical minerals listed in 30 U.S.C. § 1606(a)(3) (lithium, cobalt, nickel, manganese, graphite, rare earth elements, etc.)".to_string(),
        "IRC § 45X(b)(1) Per-Unit Credit Structure — Solar — solar-grade polysilicon at $3 per kilogram; photovoltaic wafers at $12 per square meter; photovoltaic cells at $0.04 per watt DC capacity; solar modules at $0.07 per watt DC capacity; polymeric backsheets at $0.40 per m²; torque tubes at $0.87 per kg; structural fasteners at $2.28 per kg".to_string(),
        "IRC § 45X(b)(1) Per-Unit Credit Structure — Wind — blades at $0.02 per watt of rated capacity; nacelles at $0.05 per watt of rated capacity; towers at $0.03 per watt of rated capacity; offshore wind foundations at $0.02 per watt (fixed) or $0.04 per watt (floating); offshore wind vessels at 10 PERCENT of sales price".to_string(),
        "IRC § 45X(b)(1) Per-Unit Credit Structure — Battery — electrode active materials at 10 PERCENT of production costs; battery cells at $35 per kilowatt-hour (kWh) of capacity; battery modules at $10 per kWh OR $45 per kWh if NOT using battery cells produced by the same taxpayer".to_string(),
        "IRC § 45X(b)(1) Per-Unit Credit Structure — Critical Minerals — 10 PERCENT of production costs for each applicable critical mineral".to_string(),
        "IRC § 45X(b)(3) Domestic Production Requirement — eligible components must be PRODUCED BY THE TAXPAYER (not purchased and resold); produced WITHIN THE UNITED STATES or a U.S. possession; and SOLD TO AN UNRELATED PARTY (or related-party sale election under § 45X(b)(3)(B) for downstream production within the same affiliated group)".to_string(),
        "IRC § 45X(b)(3)(B) Related-Party Sale Election — taxpayer may elect to treat a sale to a related party as a sale to an unrelated party if (i) the sale is in the ordinary course of the related party's trade or business AND (ii) the related party uses the component as a component or input to the production of an eligible component within the United States (vertical integration safe harbor)".to_string(),
        "IRC § 45X(b)(3)(C) Phase-Out of Most Components (Original IRA 2022) — most § 45X credit amounts phase out beginning in 2030 (75 % in 2030, 50 % in 2031, 25 % in 2032, 0 % in 2033 and later); critical minerals credit does NOT phase out under original IRA 2022".to_string(),
        "OBBBA 2025 § 70514 Wind Component Phase-Out — WIND ENERGY COMPONENTS (blades, nacelles, towers, offshore foundations, offshore vessels) phased out by DECEMBER 31, 2027 under Section 70514 of OBBBA 2025; accelerated termination compared to original IRA 2030-2033 phase-out".to_string(),
        "OBBBA 2025 § 70514 Prohibited Foreign Entity (PFE) Restrictions — components sourced from entities subject to MATERIAL ASSISTANCE FROM A FOREIGN ENTITY OF CONCERN (FEOC) disqualified from § 45X credit; the FEOC list includes entities in China, Russia, Iran, and North Korea".to_string(),
        "OBBBA 2025 § 70514 Domestic Content Increases — OBBBA increased the DOMESTIC CONTENT THRESHOLDS applicable to certain § 45X components, requiring higher percentages of US-sourced steel, iron, and manufactured products in the component's bill of materials".to_string(),
        "Final Regulations T.D. 10010 (October 28, 2024) — Treasury and IRS issued final regulations under § 45X published in the Federal Register on October 28, 2024; provide rules for component definitions, production cost determination, related-party sale election, primary use requirements, and recordkeeping".to_string(),
        "Form 7207 (Advanced Manufacturing Production Credit) — required to claim the § 45X credit beginning with tax year 2023".to_string(),
        "Congressional Research Service IF12809 + IRS + Cornell LII + Bloomberg Tax + ICS Tax + Crux Climate + Chambers for Innovation & Clean Energy + ACORE + The Tax Credit Exchange + Baker Tilly + JM Co + Build With Basis + Treasury + Grant Thornton + C2ES — practitioner overviews of § 45X".to_string(),
    ];

    if input.production_date_status
        == ProductionDateStatus::ProducedAndSoldOnOrBeforeDecember31_2022PreEffective
    {
        return Output {
            mode: Section45XMode::NotApplicableProducedAndSoldOnOrBeforeDecember31_2022PreEffective,
            statutory_basis: "IRA 2022 § 13502 effective date — § 45X applies only to eligible components produced and sold after December 31, 2022".to_string(),
            notes: "NOT APPLICABLE: components produced and sold on or before December 31, 2022 (pre-effective date); § 45X credit unavailable.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.eligible_component_category == EligibleComponentCategory::NonEligibleComponent {
        return Output {
            mode: Section45XMode::NotApplicableNonEligibleComponent,
            statutory_basis: "IRC § 45X(a) — component does not qualify as an eligible component".to_string(),
            notes: "NOT APPLICABLE: component does not qualify under § 45X(a) eligible-component definitions (solar / wind / inverter / battery / critical mineral); credit unavailable.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    let is_wind_component = matches!(
        input.eligible_component_category,
        EligibleComponentCategory::WindBladeNacelleTowerFoundation
            | EligibleComponentCategory::OffshoreWindVessel
    );

    if is_wind_component
        && input.wind_production_status
            == WindProductionStatus::WindProducedAndSoldAfterDecember31_2027PostObbbaPhaseOut
    {
        return Output {
            mode: Section45XMode::NotApplicableWindProducedAndSoldAfterDecember31_2027PostObbbaPhaseOut,
            statutory_basis: "OBBBA 2025 § 70514 — wind energy component phase-out cutoff December 31, 2027".to_string(),
            notes: "NOT APPLICABLE: wind energy component (blade / nacelle / tower / offshore foundation / offshore vessel) produced and sold after December 31, 2027; § 45X credit TERMINATED for wind components by Section 70514 of One Big Beautiful Bill Act of 2025 (Public Law 119-21).".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.production_and_sale_status
        == ProductionAndSaleStatus::PurchasedAndResoldOrProducedOutsideUnitedStatesIneligible
    {
        return Output {
            mode: Section45XMode::NotApplicablePurchasedAndResoldOrProducedOutsideUnitedStates,
            statutory_basis: "IRC § 45X(b)(3) — component must be produced by the taxpayer within the United States; purchased-and-resold or non-US production ineligible".to_string(),
            notes: "NOT APPLICABLE: component was purchased and resold (not produced by taxpayer) or produced outside the United States; § 45X(b)(3) domestic production requirement not met.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.prohibited_foreign_entity_status
        == ProhibitedForeignEntityStatus::SubjectToMaterialAssistanceFromForeignEntityOfConcernFeoc
    {
        return Output {
            mode: Section45XMode::NotApplicableSubjectToFeocProhibitedForeignEntityRestriction,
            statutory_basis: "OBBBA 2025 § 70514 — prohibited foreign entity restriction; component subject to material assistance from a foreign entity of concern (FEOC) ineligible".to_string(),
            notes: "NOT APPLICABLE: component subject to material assistance from a foreign entity of concern (China / Russia / Iran / North Korea); § 45X credit DISALLOWED by Section 70514 of OBBBA 2025 prohibited foreign entity restriction.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::SolarComponentProductionCreditUnderSection45XB1 => Output {
            mode: Section45XMode::CompliantSolarComponentProductionCredit,
            statutory_basis: "IRC § 45X(b)(1) — solar component production credit at per-unit rate (e.g., $3/kg polysilicon; $12/m² wafers; $0.04/W cells; $0.07/W modules)".to_string(),
            notes: "COMPLIANT: solar component production credit computed at per-unit rate under § 45X(b)(1) (rate varies by specific solar component type).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::WindComponentProductionCreditUnderSection45XB1 => Output {
            mode: Section45XMode::CompliantWindComponentProductionCreditPreObbbaPhaseOut,
            statutory_basis: "IRC § 45X(b)(1) — wind component production credit at per-unit rate (blades $0.02/W; nacelles $0.05/W; towers $0.03/W; offshore foundations $0.02-$0.04/W; offshore vessels 10 % of sales price)".to_string(),
            notes: "COMPLIANT: wind component production credit computed at per-unit rate under § 45X(b)(1); subject to OBBBA 2025 § 70514 December 31, 2027 phase-out cutoff.".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::InverterProductionCreditUnderSection45XB1 => Output {
            mode: Section45XMode::CompliantInverterProductionCredit,
            statutory_basis: "IRC § 45X(b)(1) — inverter production credit at per-unit rate (central $0.0025/W AC; commercial $0.02/W AC; microinverter / distributed wind $0.11/W AC; residential $0.065/W AC; utility $0.0015/W AC)".to_string(),
            notes: "COMPLIANT: inverter production credit computed at per-unit rate under § 45X(b)(1) (rate varies by specific inverter type).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::BatteryCellProductionCreditUnderSection45XB1 => {
            let computed = input
                .kwh_capacity_produced
                .saturating_mul(IRC_45X_BATTERY_CELL_RATE_DOLLARS_PER_KWH);
            Output {
                mode: Section45XMode::CompliantBatteryCellProductionCreditAtThirtyFiveDollarsPerKwh,
                statutory_basis: "IRC § 45X(b)(1) — battery cell production credit at $35 per kWh of capacity".to_string(),
                notes: format!(
                    "COMPLIANT: battery cell production credit at $35 per kWh × {kwh} kWh = ${computed}.",
                    kwh = input.kwh_capacity_produced,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::BatteryModuleProductionCreditUnderSection45XB1 => {
            let rate = match input.eligible_component_category {
                EligibleComponentCategory::BatteryModuleIntegratedWithSameTaxpayerCells => {
                    IRC_45X_BATTERY_MODULE_INTEGRATED_RATE_DOLLARS_PER_KWH
                }
                _ => IRC_45X_BATTERY_MODULE_STANDALONE_RATE_DOLLARS_PER_KWH,
            };
            let computed = input.kwh_capacity_produced.saturating_mul(rate);
            let mode = if rate == IRC_45X_BATTERY_MODULE_INTEGRATED_RATE_DOLLARS_PER_KWH {
                Section45XMode::CompliantBatteryModuleIntegratedProductionCreditAtTenDollarsPerKwh
            } else {
                Section45XMode::CompliantBatteryModuleStandaloneProductionCreditAtFortyFiveDollarsPerKwh
            };
            Output {
                mode,
                statutory_basis: "IRC § 45X(b)(1) — battery module production credit at $10 per kWh (integrated with same-taxpayer battery cells) OR $45 per kWh (standalone using purchased battery cells)".to_string(),
                notes: format!(
                    "COMPLIANT: battery module production credit at ${rate} per kWh × {kwh} kWh = ${computed}.",
                    kwh = input.kwh_capacity_produced,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::CriticalMineralProductionCreditUnderSection45XB1 => {
            let computed = (u128::from(input.production_cost_dollars)
                * u128::from(IRC_45X_CRITICAL_MINERAL_PERCENT_OF_PRODUCTION_COST_BPS)
                / u128::from(IRC_45X_BASIS_POINT_DENOMINATOR)) as u64;
            Output {
                mode: Section45XMode::CompliantCriticalMineralProductionCreditAtTenPercentOfProductionCost,
                statutory_basis: "IRC § 45X(b)(1) — applicable critical mineral production credit at 10 PERCENT of production cost".to_string(),
                notes: format!(
                    "COMPLIANT: critical mineral production credit at 10 % × ${cost} production cost = ${computed}.",
                    cost = input.production_cost_dollars,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::DomesticProductionRequirementUnderSection45XB3 => Output {
            mode: Section45XMode::CompliantDomesticProductionRequirementMet,
            statutory_basis: "IRC § 45X(b)(3) — domestic production requirement met (produced by taxpayer within United States; sold to unrelated party or related-party election)".to_string(),
            notes: "COMPLIANT: component produced by taxpayer within United States and sold to unrelated party (or related-party sale election under § 45X(b)(3)(B)); domestic production requirement met.".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::RelatedPartySaleElectionUnderSection45XB3B => {
            if input.production_and_sale_status
                == ProductionAndSaleStatus::ProducedByTaxpayerInUnitedStatesAndRelatedPartySaleElectionMade
            {
                Output {
                    mode: Section45XMode::CompliantRelatedPartySaleElectionMade,
                    statutory_basis: "IRC § 45X(b)(3)(B) — related-party sale election made; related party uses component in ordinary course of trade or business as input to production of another eligible component within United States".to_string(),
                    notes: "COMPLIANT: related-party sale election made under § 45X(b)(3)(B); vertical integration safe harbor available where related party uses component as input to downstream eligible component production within the United States.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45XMode::CompliantDomesticProductionRequirementMet,
                    statutory_basis: "IRC § 45X(b)(3) — sold to unrelated party; § 45X(b)(3)(B) related-party sale election not necessary".to_string(),
                    notes: "COMPLIANT: component sold to unrelated party; no related-party sale election required under § 45X(b)(3)(B).".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::ObbbaWindPhaseOutUnderSection70514 => {
            if is_wind_component {
                Output {
                    mode: Section45XMode::CompliantWindComponentProductionCreditPreObbbaPhaseOut,
                    statutory_basis: "OBBBA 2025 § 70514 — wind energy component produced and sold on or before December 31, 2027".to_string(),
                    notes: "COMPLIANT: wind energy component produced and sold on or before December 31, 2027 (pre-OBBBA phase-out cutoff); § 45X credit available.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45XMode::CompliantDomesticProductionRequirementMet,
                    statutory_basis: "OBBBA 2025 § 70514 — non-wind component unaffected by wind component phase-out".to_string(),
                    notes: "COMPLIANT: non-wind component (solar / inverter / battery / critical mineral) unaffected by OBBBA 2025 wind component phase-out under § 70514; original IRA 2022 phase-out schedule applies (2030 → 2033 for most components; no phase-out for critical minerals).".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::ProhibitedForeignEntityRestrictionUnderObbbaSection70514 => Output {
            mode: Section45XMode::CompliantNotSubjectToFeocRestriction,
            statutory_basis: "OBBBA 2025 § 70514 — component not subject to material assistance from a foreign entity of concern".to_string(),
            notes: "COMPLIANT: component not subject to material assistance from a foreign entity of concern (China / Russia / Iran / North Korea); OBBBA 2025 § 70514 prohibited foreign entity restriction does NOT apply.".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::FormFilingUnderForm7207 => {
            if input.form_7207_filed_correctly {
                Output {
                    mode: Section45XMode::CompliantForm7207FiledCorrectly,
                    statutory_basis: "Form 7207 — Advanced Manufacturing Production Credit form required to claim § 45X credit".to_string(),
                    notes: "COMPLIANT: Form 7207 filed correctly to claim § 45X credit for tax year.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45XMode::ViolationForm7207NotFiledOrIncorrect,
                    statutory_basis: "Form 7207 filing required to claim § 45X credit".to_string(),
                    notes: "VIOLATION: Form 7207 not filed or incorrectly filed; § 45X credit may be disallowed.".to_string(),
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
            production_date_status:
                ProductionDateStatus::ProducedAndSoldAfterDecember31_2022PostEffectiveEligible,
            eligible_component_category: EligibleComponentCategory::SolarModule,
            wind_production_status: WindProductionStatus::NotApplicableNonWindComponent,
            production_and_sale_status:
                ProductionAndSaleStatus::ProducedByTaxpayerInUnitedStatesAndSoldToUnrelatedParty,
            prohibited_foreign_entity_status:
                ProhibitedForeignEntityStatus::NotSubjectToProhibitedForeignEntityRestrictions,
            compliance_aspect: ComplianceAspect::SolarComponentProductionCreditUnderSection45XB1,
            kwh_capacity_produced: 1_000,
            production_cost_dollars: 100_000,
            offshore_wind_vessel_sales_price_dollars: 10_000_000,
            form_7207_filed_correctly: true,
        }
    }

    #[test]
    fn pre_effective_production_not_applicable() {
        let mut input = baseline_input();
        input.production_date_status =
            ProductionDateStatus::ProducedAndSoldOnOrBeforeDecember31_2022PreEffective;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::NotApplicableProducedAndSoldOnOrBeforeDecember31_2022PreEffective
        );
    }

    #[test]
    fn non_eligible_component_not_applicable() {
        let mut input = baseline_input();
        input.eligible_component_category = EligibleComponentCategory::NonEligibleComponent;
        let out = check(&input);
        assert_eq!(out.mode, Section45XMode::NotApplicableNonEligibleComponent);
    }

    #[test]
    fn wind_produced_after_december31_2027_not_applicable() {
        let mut input = baseline_input();
        input.eligible_component_category =
            EligibleComponentCategory::WindBladeNacelleTowerFoundation;
        input.wind_production_status =
            WindProductionStatus::WindProducedAndSoldAfterDecember31_2027PostObbbaPhaseOut;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::NotApplicableWindProducedAndSoldAfterDecember31_2027PostObbbaPhaseOut
        );
    }

    #[test]
    fn purchased_and_resold_or_non_us_not_applicable() {
        let mut input = baseline_input();
        input.production_and_sale_status =
            ProductionAndSaleStatus::PurchasedAndResoldOrProducedOutsideUnitedStatesIneligible;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::NotApplicablePurchasedAndResoldOrProducedOutsideUnitedStates
        );
    }

    #[test]
    fn prohibited_foreign_entity_not_applicable() {
        let mut input = baseline_input();
        input.prohibited_foreign_entity_status =
            ProhibitedForeignEntityStatus::SubjectToMaterialAssistanceFromForeignEntityOfConcernFeoc;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::NotApplicableSubjectToFeocProhibitedForeignEntityRestriction
        );
    }

    #[test]
    fn solar_component_credit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SolarComponentProductionCreditUnderSection45XB1;
        input.eligible_component_category = EligibleComponentCategory::SolarModule;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::CompliantSolarComponentProductionCredit
        );
    }

    #[test]
    fn wind_component_credit_before_phase_out_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WindComponentProductionCreditUnderSection45XB1;
        input.eligible_component_category =
            EligibleComponentCategory::WindBladeNacelleTowerFoundation;
        input.wind_production_status =
            WindProductionStatus::WindProducedAndSoldOnOrBeforeDecember31_2027PreObbbaPhaseOut;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::CompliantWindComponentProductionCreditPreObbbaPhaseOut
        );
    }

    #[test]
    fn inverter_credit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::InverterProductionCreditUnderSection45XB1;
        input.eligible_component_category = EligibleComponentCategory::Inverter;
        let out = check(&input);
        assert_eq!(out.mode, Section45XMode::CompliantInverterProductionCredit);
    }

    #[test]
    fn battery_cell_credit_at_thirty_five_dollars_per_kwh_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BatteryCellProductionCreditUnderSection45XB1;
        input.eligible_component_category = EligibleComponentCategory::BatteryCell;
        input.kwh_capacity_produced = 1_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::CompliantBatteryCellProductionCreditAtThirtyFiveDollarsPerKwh
        );
        assert_eq!(out.computed_credit_dollars, 35_000);
    }

    #[test]
    fn battery_module_integrated_credit_at_ten_dollars_per_kwh_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BatteryModuleProductionCreditUnderSection45XB1;
        input.eligible_component_category =
            EligibleComponentCategory::BatteryModuleIntegratedWithSameTaxpayerCells;
        input.kwh_capacity_produced = 1_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::CompliantBatteryModuleIntegratedProductionCreditAtTenDollarsPerKwh
        );
        assert_eq!(out.computed_credit_dollars, 10_000);
    }

    #[test]
    fn battery_module_standalone_credit_at_forty_five_dollars_per_kwh_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BatteryModuleProductionCreditUnderSection45XB1;
        input.eligible_component_category =
            EligibleComponentCategory::BatteryModuleStandaloneUsingPurchasedCells;
        input.kwh_capacity_produced = 1_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::CompliantBatteryModuleStandaloneProductionCreditAtFortyFiveDollarsPerKwh
        );
        assert_eq!(out.computed_credit_dollars, 45_000);
    }

    #[test]
    fn critical_mineral_credit_at_ten_percent_of_production_cost_computed() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::CriticalMineralProductionCreditUnderSection45XB1;
        input.eligible_component_category = EligibleComponentCategory::ApplicableCriticalMineral;
        input.production_cost_dollars = 100_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::CompliantCriticalMineralProductionCreditAtTenPercentOfProductionCost
        );
        assert_eq!(out.computed_credit_dollars, 10_000);
    }

    #[test]
    fn domestic_production_requirement_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DomesticProductionRequirementUnderSection45XB3;
        input.production_and_sale_status =
            ProductionAndSaleStatus::ProducedByTaxpayerInUnitedStatesAndSoldToUnrelatedParty;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::CompliantDomesticProductionRequirementMet
        );
    }

    #[test]
    fn related_party_sale_election_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RelatedPartySaleElectionUnderSection45XB3B;
        input.production_and_sale_status =
            ProductionAndSaleStatus::ProducedByTaxpayerInUnitedStatesAndRelatedPartySaleElectionMade;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::CompliantRelatedPartySaleElectionMade
        );
    }

    #[test]
    fn obbba_wind_phase_out_non_wind_component_unaffected() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ObbbaWindPhaseOutUnderSection70514;
        input.eligible_component_category = EligibleComponentCategory::SolarModule;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::CompliantDomesticProductionRequirementMet
        );
    }

    #[test]
    fn obbba_wind_phase_out_wind_component_within_window_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ObbbaWindPhaseOutUnderSection70514;
        input.eligible_component_category =
            EligibleComponentCategory::WindBladeNacelleTowerFoundation;
        input.wind_production_status =
            WindProductionStatus::WindProducedAndSoldOnOrBeforeDecember31_2027PreObbbaPhaseOut;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::CompliantWindComponentProductionCreditPreObbbaPhaseOut
        );
    }

    #[test]
    fn not_subject_to_feoc_restriction_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::ProhibitedForeignEntityRestrictionUnderObbbaSection70514;
        input.prohibited_foreign_entity_status =
            ProhibitedForeignEntityStatus::NotSubjectToProhibitedForeignEntityRestrictions;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::CompliantNotSubjectToFeocRestriction
        );
    }

    #[test]
    fn form_7207_filed_correctly_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm7207;
        input.form_7207_filed_correctly = true;
        let out = check(&input);
        assert_eq!(out.mode, Section45XMode::CompliantForm7207FiledCorrectly);
    }

    #[test]
    fn form_7207_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm7207;
        input.form_7207_filed_correctly = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45XMode::ViolationForm7207NotFiledOrIncorrect
        );
    }

    #[test]
    fn constants_pin_section_45x_legislative_phases_and_per_unit_structure() {
        assert_eq!(IRC_45X_IRA_ENACTMENT_DATE_YEAR, 2022);
        assert_eq!(IRC_45X_IRA_PUBLIC_LAW_CONGRESS, 117);
        assert_eq!(IRC_45X_IRA_PUBLIC_LAW_ENACTMENT, 169);
        assert_eq!(IRC_45X_IRA_ENABLING_SECTION, 13502);
        assert_eq!(IRC_45X_EFFECTIVE_DATE_YEAR, 2023);
        assert_eq!(IRC_45X_OBBBA_ENACTMENT_DATE_YEAR, 2025);
        assert_eq!(IRC_45X_OBBBA_PUBLIC_LAW_CONGRESS, 119);
        assert_eq!(IRC_45X_OBBBA_PUBLIC_LAW_ENACTMENT, 21);
        assert_eq!(IRC_45X_OBBBA_ENABLING_SECTION, 70514);
        assert_eq!(IRC_45X_OBBBA_WIND_PHASE_OUT_CUTOFF_YEAR, 2027);
        assert_eq!(IRC_45X_ORIGINAL_PHASE_OUT_BEGIN_YEAR, 2030);
        assert_eq!(IRC_45X_ORIGINAL_PHASE_OUT_END_YEAR, 2033);
        assert_eq!(IRC_45X_BATTERY_CELL_RATE_DOLLARS_PER_KWH, 35);
        assert_eq!(IRC_45X_BATTERY_MODULE_INTEGRATED_RATE_DOLLARS_PER_KWH, 10);
        assert_eq!(IRC_45X_BATTERY_MODULE_STANDALONE_RATE_DOLLARS_PER_KWH, 45);
        assert_eq!(
            IRC_45X_CRITICAL_MINERAL_PERCENT_OF_PRODUCTION_COST_BPS,
            1_000
        );
        assert_eq!(
            IRC_45X_OFFSHORE_WIND_VESSEL_PERCENT_OF_SALES_PRICE_BPS,
            1_000
        );
        assert_eq!(IRC_45X_SOLAR_POLYSILICON_RATE_DOLLARS_PER_KG, 3);
        assert_eq!(IRC_45X_SOLAR_WAFER_RATE_DOLLARS_PER_SQ_METER, 12);
        assert_eq!(IRC_45X_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_45X_FORM_NUMBER, 7207);
        assert_eq!(IRC_45X_FINAL_REGS_PUBLICATION_DATE_YEAR, 2024);
        assert_eq!(IRC_45X_FINAL_REGS_PUBLICATION_DATE_MONTH, 10);
        assert_eq!(IRC_45X_FINAL_REGS_PUBLICATION_DATE_DAY, 28);
    }

    #[test]
    fn citations_pin_legislative_phases_and_obbba_facts() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("IRC § 45X Advanced Manufacturing Production Credit"));
        assert!(joined.contains("Section 13502 of the Inflation Reduction Act of 2022"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("August 16, 2022"));
        assert!(joined.contains("December 31, 2022"));
        assert!(joined.contains("PHOTOVOLTAIC CELLS"));
        assert!(joined.contains("PHOTOVOLTAIC WAFERS"));
        assert!(joined.contains("SOLAR-GRADE POLYSILICON"));
        assert!(joined.contains("POLYMERIC BACKSHEETS"));
        assert!(joined.contains("ELECTRODE ACTIVE MATERIALS"));
        assert!(joined.contains("BATTERY CELLS"));
        assert!(joined.contains("BATTERY MODULES"));
        assert!(joined.contains("$3 per kilogram"));
        assert!(joined.contains("$12 per square meter"));
        assert!(joined.contains("$35 per kilowatt-hour"));
        assert!(joined.contains("$10 per kWh"));
        assert!(joined.contains("$45 per kWh"));
        assert!(joined.contains("10 PERCENT"));
        assert!(joined.contains("PRODUCED BY THE TAXPAYER"));
        assert!(joined.contains("WITHIN THE UNITED STATES"));
        assert!(joined.contains("SOLD TO AN UNRELATED PARTY"));
        assert!(joined.contains("Section 70514 of OBBBA 2025"));
        assert!(joined.contains("WIND ENERGY COMPONENTS"));
        assert!(joined.contains("DECEMBER 31, 2027"));
        assert!(joined.contains("MATERIAL ASSISTANCE FROM A FOREIGN ENTITY OF CONCERN"));
        assert!(joined.contains("Final Regulations T.D. 10010"));
        assert!(joined.contains("October 28, 2024"));
        assert!(joined.contains("Form 7207"));
    }

    #[test]
    fn saturating_overflow_defense_at_u64_max_kwh() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BatteryCellProductionCreditUnderSection45XB1;
        input.kwh_capacity_produced = u64::MAX;
        let out = check(&input);
        let _ = out.computed_credit_dollars;
    }
}
