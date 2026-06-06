//! IRC § 45Q Credit for Carbon Oxide Sequestration
//! Compliance Module — pure-compute check for the
//! production tax credit for **CARBON OXIDE CAPTURE AND
//! SEQUESTRATION (CCUS)** from qualified facilities and
//! **DIRECT AIR CAPTURE (DAC)** facilities. Per-ton credit
//! structure varying by facility type, disposition method
//! (saline geological storage vs utilization/enhanced oil
//! recovery), and PWA compliance.
//!
//! Three-phase legislative history:
//! 1. **Energy Improvement and Extension Act of 2008** (Public Law 110-343, Division B), signed October 3, 2008 — Original § 45Q. Capped at 75 million metric tons cumulative.
//! 2. **Bipartisan Budget Act of 2018** (Public Law 115-123, § 41119), signed February 9, 2018 — Substantial expansion. Removed 75M ton cap; added 12-year credit period; added DAC; added BOC deadline.
//! 3. **Inflation Reduction Act of 2022 § 13104** (Public Law 117-169), signed August 16, 2022 — Major expansion. Raised credit rates (industrial saline $50→$85 PWA; utilization/EOR $35→$60 PWA; DAC saline $50→$180 PWA; DAC utilization/EOR $35→$130 PWA). Lowered tonnage thresholds (industrial 100,000→12,500; DAC 100,000→1,000). Established two-tier PWA structure (base 1/5 of headline rate; 5x bonus with PWA).
//!
//! **OBBBA 2025 modifications**: Public Law 119-21 standardized
//! certain base credit amounts at **$17 per metric ton** for
//! facilities/equipment placed in service after July 4, 2025
//! and before 2027.
//!
//! Web research (verified 2026-06-03):
//! - **Energy Improvement and Extension Act of 2008 Enactment**: IRC § 45Q originally added by **Energy Improvement and Extension Act of 2008 (Division B of Public Law 110-343)**, signed by President George W. Bush on **October 3, 2008**; original credit capped at 75 million metric tons cumulative ([Congressional Research Service IF11455 — The Section 45Q Tax Credit for Carbon Sequestration](https://www.congress.gov/crs-product/IF11455); [Elliott Davis — Tax Credit for Carbon Oxide Sequestration IRA Section 45Q](https://www.elliottdavis.com/insights/tax-credit-for-carbon-oxide-sequestration-ira-section-45q); [Cornell LII — 26 U.S. Code § 45Q](https://www.law.cornell.edu/uscode/text/26/45Q); [IRS — Request for Comments on the Credit for Carbon Oxide Sequestration Notice 22-57 PDF](https://www.irs.gov/pub/irs-drop/n-22-57.pdf); [IRS — Instructions for Form 8933 (12/2025)](https://www.irs.gov/instructions/i8933); [Gibson Dunn — The Inflation Reduction Act Includes Significant Benefits for the Carbon Capture Industry](https://www.gibsondunn.com/the-inflation-reduction-act-includes-significant-benefits-for-the-carbon-capture-industry/); [Tax Notes — IRC Section 45Q Tax Credit for Carbon Oxide Sequestration](https://www.taxnotes.com/research/federal/usc26/45Q); [House USC — 26 USC 45Q](https://uscode.house.gov/view.xhtml?req=(title:26+section:45Q+edition:prelim)); [McDermott Will & Emery — Carbon Capture, Utilization and Sequestration Tax Benefits under the IRA](https://www.mwe.com/insights/carbon-capture-utilization-and-sequestration-tax-benefits-under-the-proposed-inflation-reduction-act/); [EisnerAmper — IRC Section 45Q Carbon Tax Credit](https://www.eisneramper.com/insights/tax/45q-credit-advantage-1123/); [BrownWinick — 45Q Carbon Sequestration Tax Credit](https://www.brownwinick.com/insights/45q-carbon-sequestration-tax-credit-what-it-is-how-to-get-it); [Carbon Herald — What is The 45Q Tax Credit?](https://carbonherald.com/what-is-45q-tax-credit/); [Davis Graham — Treasury Issues Final Regulations on Section 45Q Tax Credits](https://davisgraham.com/news-events/treasury-issues-final-regulations-on-section-45q-tax-credits-for-carbon-capture-and-sequestration/); [Energy Communities — Credit for Carbon Oxide Sequestration 26 U.S. Code § 45Q](https://energycommunities.gov/funding-opportunity/credit-for-carbon-oxide-sequestration-26-u-s-code-%C2%A4-45q/); [LegalClarity — How to Qualify for the Section 45Q Tax Credit](https://legalclarity.org/how-to-qualify-for-the-section-45q-tax-credit/); [Accounting Insights — Section 45Q Tax Credit Eligibility, Rates, and Claims](https://accountinginsights.org/section-45q-tax-credit-eligibility-rates-and-claims/); [IEAGHG — New Beginnings for Carbon Capture with Section 45Q Tax Credits](https://ieaghg.org/news/new-beginnings-for-carbon-capture-with-section-45q-tax-credits-in-the-united-states/); [IEA — Section 45Q Credit for Carbon Oxide Sequestration Policies](https://www.iea.org/policies/4986-section-45q-credit-for-carbon-oxide-sequestration)).
//! - **Bipartisan Budget Act of 2018 Expansion**: IRC § 45Q substantially expanded by **Bipartisan Budget Act of 2018 (Public Law 115-123, § 41119)**, signed by President Donald Trump on **February 9, 2018**; (i) **REMOVED** the original 75 million metric ton cumulative cap; (ii) introduced **12-YEAR CREDIT PERIOD** for capture equipment; (iii) added eligibility for **DIRECT AIR CAPTURE (DAC)** and CO2 utilization; (iv) added **START-OF-CONSTRUCTION DEADLINE** (originally January 1, 2024, subsequently extended to January 1, 2026 under IRA 2022, then to January 1, 2033 under IRA 2022); (v) allowed **OWNERS OF CAPTURE EQUIPMENT** to claim credits and for that entity to transfer the credit to the entity storing the CO2.
//! - **IRA 2022 § 13104 Expansion**: IRC § 45Q substantially amended by **Section 13104 of the Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818)**, signed by President Joe Biden on **August 16, 2022**; (i) **TWO-TIER PWA STRUCTURE** established (base rates 1/5 of headline; 5x bonus with PWA); (ii) **RAISED RATES** for facilities meeting PWA: **industrial saline geological storage $85/MT**, **industrial utilization/EOR $60/MT**, **DAC saline geological storage $180/MT**, **DAC utilization/EOR $130/MT**; (iii) **LOWERED TONNAGE THRESHOLDS**: industrial CCUS lowered from 100,000 to **12,500 metric tons/year**, DAC lowered from 100,000 to **1,000 metric tons/year**; (iv) BOC deadline extended to **JANUARY 1, 2033**.
//! - **§ 45Q(a) Tiered Credit Rates by Facility Type and Disposition**: **INDUSTRIAL FACILITY SALINE GEOLOGICAL STORAGE**: $17/ton base, $85/ton with PWA (5x); **INDUSTRIAL FACILITY UTILIZATION/EOR**: $12/ton base, $60/ton with PWA (5x); **DAC FACILITY SALINE GEOLOGICAL STORAGE**: $36/ton base, $180/ton with PWA (5x); **DAC FACILITY UTILIZATION/EOR**: $26/ton base, $130/ton with PWA (5x).
//! - **§ 45Q(d)(2)(A) Industrial CCUS Tonnage Threshold**: **12,500 METRIC TONS** of carbon oxide captured per taxable year minimum for industrial facilities to qualify under IRA 2022.
//! - **§ 45Q(d)(2)(B) DAC Facility Tonnage Threshold**: **1,000 METRIC TONS** of carbon oxide captured per taxable year minimum for direct air capture facilities to qualify under IRA 2022.
//! - **§ 45Q(a) 12-Year Credit Period**: credit available for the **12-YEAR PERIOD BEGINNING ON DATE CAPTURE EQUIPMENT IS PLACED IN SERVICE**.
//! - **§ 45Q(b) Start-of-Construction Deadline**: facility must **BEGIN CONSTRUCTION BEFORE JANUARY 1, 2033** to qualify (extended under IRA 2022 from original BBA 2018 January 1, 2024 cutoff).
//! - **OBBBA 2025 Standardization at $17/Ton Base**: Public Law 119-21 standardized certain base credit amounts at **$17 PER METRIC TON** for facilities/equipment placed in service **AFTER JULY 4, 2025 AND BEFORE 2027**; this is a reduction for DAC (was $36 base before OBBBA) but no change for industrial saline storage.
//! - **§ 6417 Direct Pay Election Available**: § 45Q credit eligible for **DIRECT PAY ELECTION** under § 6417 for tax-exempt entities; special **5-YEAR DIRECT PAY ELECTION** available for taxable entities for the first 5 years of the 12-year credit period.
//! - **§ 6418 Transferability Election Available**: § 45Q credit eligible for **TRANSFERABILITY ELECTION** under § 6418 — taxpayer may sell credit to an unrelated third party for cash.
//! - **Form 8933 (Carbon Oxide Sequestration Credit)**: required to claim the § 45Q credit; current instructions IRS Form 8933 (12/2025).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_45Q_EIEA_ENACTMENT_DATE_YEAR: u32 = 2008;
pub const IRC_45Q_EIEA_ENACTMENT_DATE_MONTH: u32 = 10;
pub const IRC_45Q_EIEA_ENACTMENT_DATE_DAY: u32 = 3;
pub const IRC_45Q_EIEA_PUBLIC_LAW_CONGRESS: u32 = 110;
pub const IRC_45Q_EIEA_PUBLIC_LAW_ENACTMENT: u32 = 343;
pub const IRC_45Q_BBA_ENACTMENT_DATE_YEAR: u32 = 2018;
pub const IRC_45Q_BBA_ENACTMENT_DATE_MONTH: u32 = 2;
pub const IRC_45Q_BBA_ENACTMENT_DATE_DAY: u32 = 9;
pub const IRC_45Q_BBA_PUBLIC_LAW_CONGRESS: u32 = 115;
pub const IRC_45Q_BBA_PUBLIC_LAW_ENACTMENT: u32 = 123;
pub const IRC_45Q_BBA_ENABLING_SECTION: u32 = 41119;
pub const IRC_45Q_IRA_ENACTMENT_DATE_YEAR: u32 = 2022;
pub const IRC_45Q_IRA_ENACTMENT_DATE_MONTH: u32 = 8;
pub const IRC_45Q_IRA_ENACTMENT_DATE_DAY: u32 = 16;
pub const IRC_45Q_IRA_PUBLIC_LAW_CONGRESS: u32 = 117;
pub const IRC_45Q_IRA_PUBLIC_LAW_ENACTMENT: u32 = 169;
pub const IRC_45Q_IRA_ENABLING_SECTION: u32 = 13104;
pub const IRC_45Q_BOC_CUTOFF_YEAR: u32 = 2033;
pub const IRC_45Q_BOC_CUTOFF_MONTH: u32 = 1;
pub const IRC_45Q_BOC_CUTOFF_DAY: u32 = 1;
pub const IRC_45Q_INDUSTRIAL_SALINE_BASE_DOLLARS_PER_TON: u64 = 17;
pub const IRC_45Q_INDUSTRIAL_SALINE_PWA_DOLLARS_PER_TON: u64 = 85;
pub const IRC_45Q_INDUSTRIAL_UTILIZATION_BASE_DOLLARS_PER_TON: u64 = 12;
pub const IRC_45Q_INDUSTRIAL_UTILIZATION_PWA_DOLLARS_PER_TON: u64 = 60;
pub const IRC_45Q_DAC_SALINE_BASE_DOLLARS_PER_TON: u64 = 36;
pub const IRC_45Q_DAC_SALINE_PWA_DOLLARS_PER_TON: u64 = 180;
pub const IRC_45Q_DAC_UTILIZATION_BASE_DOLLARS_PER_TON: u64 = 26;
pub const IRC_45Q_DAC_UTILIZATION_PWA_DOLLARS_PER_TON: u64 = 130;
pub const IRC_45Q_PWA_BONUS_MULTIPLIER: u64 = 5;
pub const IRC_45Q_INDUSTRIAL_TONNAGE_THRESHOLD_TONS_PER_YEAR: u64 = 12_500;
pub const IRC_45Q_DAC_TONNAGE_THRESHOLD_TONS_PER_YEAR: u64 = 1_000;
pub const IRC_45Q_CREDIT_PERIOD_YEARS: u32 = 12;
pub const IRC_45Q_OBBBA_STANDARDIZED_BASE_DOLLARS_PER_TON: u64 = 17;
pub const IRC_45Q_OBBBA_ENACTMENT_DATE_YEAR: u32 = 2025;
pub const IRC_45Q_OBBBA_ENACTMENT_DATE_MONTH: u32 = 7;
pub const IRC_45Q_OBBBA_ENACTMENT_DATE_DAY: u32 = 4;
pub const IRC_45Q_OBBBA_PUBLIC_LAW_CONGRESS: u32 = 119;
pub const IRC_45Q_OBBBA_PUBLIC_LAW_ENACTMENT: u32 = 21;
pub const IRC_45Q_DIRECT_PAY_CROSS_REFERENCE_SECTION: u32 = 6417;
pub const IRC_45Q_DIRECT_PAY_TAXABLE_ENTITY_YEARS: u32 = 5;
pub const IRC_45Q_TRANSFERABILITY_CROSS_REFERENCE_SECTION: u32 = 6418;
pub const IRC_45Q_FORM_NUMBER: u32 = 8933;
pub const IRC_45Q_ORIGINAL_CUMULATIVE_CAP_MILLION_TONS: u32 = 75;
pub const IRC_45Q_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BeginningOfConstructionStatus {
    BocBeforeJanuary1_2033PreCutoff,
    BocOnOrAfterJanuary1_2033PostCutoff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FacilityType {
    IndustrialFacilityCcus,
    DirectAirCaptureFacility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DispositionMethod {
    SalineGeologicalStorage,
    UtilizationOrEnhancedOilRecovery,
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
    BaseCreditAmountUnderSection45QA,
    BonusCreditAmountForPwaUnderSection45QA2,
    TonnageThresholdUnderSection45QD2,
    TwelveYearCreditPeriodUnderSection45QA,
    BeginningOfConstructionDeadlineUnderSection45QB,
    DirectPayElectionUnderSection6417,
    TransferabilityElectionUnderSection6418,
    FormFilingUnderForm8933,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section45QMode {
    NotApplicableBocOnOrAfterJanuary1_2033PostCutoff,
    NotApplicableTonnageBelowThresholdForFacilityType,
    CompliantIndustrialSalineBaseCreditAtSeventeenDollarsPerTon,
    CompliantIndustrialSalinePwaBonusCreditAtEightyFiveDollarsPerTon,
    CompliantIndustrialUtilizationBaseCreditAtTwelveDollarsPerTon,
    CompliantIndustrialUtilizationPwaBonusCreditAtSixtyDollarsPerTon,
    CompliantDacSalineBaseCreditAtThirtySixDollarsPerTon,
    CompliantDacSalinePwaBonusCreditAtOneHundredEightyDollarsPerTon,
    CompliantDacUtilizationBaseCreditAtTwentySixDollarsPerTon,
    CompliantDacUtilizationPwaBonusCreditAtOneHundredThirtyDollarsPerTon,
    CompliantTonnageMeetsThresholdForFacilityType,
    CompliantTwelveYearCreditPeriodYearWithinWindow,
    CompliantBocBeforeJanuary1_2033PreCutoff,
    CompliantDirectPayElectionMade,
    CompliantTransferabilityElectionMade,
    CompliantForm8933FiledCorrectly,
    ViolationCreditClaimedOutsideTwelveYearCreditPeriod,
    ViolationPwaBonusMultiplierClaimedWithoutMeetingPwaRequirements,
    ViolationForm8933NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub beginning_of_construction_status: BeginningOfConstructionStatus,
    pub facility_type: FacilityType,
    pub disposition_method: DispositionMethod,
    pub pwa_status: PrevailingWageApprenticeshipStatus,
    pub compliance_aspect: ComplianceAspect,
    pub metric_tons_captured_per_year: u64,
    pub credit_year_number_within_window: u32,
    pub direct_pay_election_made: bool,
    pub transferability_election_made: bool,
    pub form_8933_filed_correctly: bool,
    pub claimed_pwa_bonus_multiplier: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section45QMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section45QInput = Input;
pub type Section45QOutput = Output;
pub type Section45QResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 45Q Credit for Carbon Oxide Sequestration originally added by Energy Improvement and Extension Act of 2008 (Division B of Public Law 110-343), signed by President George W. Bush on October 3, 2008; original credit capped at 75 million metric tons cumulative".to_string(),
        "Bipartisan Budget Act of 2018 (Public Law 115-123, § 41119), signed by President Donald Trump on February 9, 2018 — REMOVED 75M ton cumulative cap; introduced 12-YEAR CREDIT PERIOD; added eligibility for DIRECT AIR CAPTURE (DAC) and CO2 utilization; added START-OF-CONSTRUCTION DEADLINE; allowed OWNERS OF CAPTURE EQUIPMENT to claim credits and transfer to storage entity".to_string(),
        "IRA 2022 § 13104 (Section 13104 of the Inflation Reduction Act of 2022, Public Law 117-169, 136 Stat. 1818), signed by President Joe Biden on August 16, 2022 — TWO-TIER PWA STRUCTURE established (base rates 1/5 of headline; 5x bonus with PWA); RAISED RATES; LOWERED TONNAGE THRESHOLDS (industrial 100,000 → 12,500 metric tons/year; DAC 100,000 → 1,000 metric tons/year); BOC deadline extended to JANUARY 1, 2033".to_string(),
        "IRC § 45Q(a) Tiered Credit Rates — INDUSTRIAL FACILITY SALINE GEOLOGICAL STORAGE: $17/ton base, $85/ton with PWA (5x); INDUSTRIAL FACILITY UTILIZATION/EOR: $12/ton base, $60/ton with PWA (5x); DAC FACILITY SALINE GEOLOGICAL STORAGE: $36/ton base, $180/ton with PWA (5x); DAC FACILITY UTILIZATION/EOR: $26/ton base, $130/ton with PWA (5x)".to_string(),
        "IRC § 45Q(a)(2) PWA Bonus 5x Multiplier — credit MULTIPLIED BY 5 if taxpayer satisfies prevailing wage and apprenticeship (PWA) requirements during construction and for the 12-year credit period".to_string(),
        "IRC § 45Q(d)(2)(A) Industrial CCUS Tonnage Threshold — 12,500 METRIC TONS of carbon oxide captured per taxable year minimum for industrial facilities to qualify under IRA 2022 (lowered from previous 100,000 ton threshold)".to_string(),
        "IRC § 45Q(d)(2)(B) DAC Facility Tonnage Threshold — 1,000 METRIC TONS of carbon oxide captured per taxable year minimum for direct air capture facilities to qualify under IRA 2022 (lowered from previous 100,000 ton threshold)".to_string(),
        "IRC § 45Q(a) 12-Year Credit Period — credit available for the 12-YEAR PERIOD BEGINNING ON DATE CAPTURE EQUIPMENT IS PLACED IN SERVICE".to_string(),
        "IRC § 45Q(b) Start-of-Construction Deadline — facility must BEGIN CONSTRUCTION BEFORE JANUARY 1, 2033 to qualify (extended under IRA 2022 from original BBA 2018 January 1, 2024 cutoff)".to_string(),
        "OBBBA 2025 Standardization at $17/Ton Base — Public Law 119-21 standardized certain base credit amounts at $17 PER METRIC TON for facilities/equipment placed in service AFTER JULY 4, 2025 AND BEFORE 2027".to_string(),
        "IRC § 6417 Direct Pay Election — § 45Q credit eligible for DIRECT PAY ELECTION for tax-exempt entities; special 5-YEAR DIRECT PAY ELECTION available for taxable entities for the first 5 years of the 12-year credit period".to_string(),
        "IRC § 6418 Transferability Election — § 45Q credit eligible for TRANSFERABILITY ELECTION; taxpayer may sell credit to an unrelated third party for cash".to_string(),
        "Form 8933 (Carbon Oxide Sequestration Credit) — required to claim the § 45Q credit; current instructions IRS Form 8933 (12/2025)".to_string(),
        "Congressional Research Service IF11455 + Cornell LII + Bloomberg Tax + Elliott Davis + EisnerAmper + Tax Notes + Gibson Dunn + McDermott Will & Emery + BrownWinick + Carbon Herald + Davis Graham + Energy Communities + LegalClarity + Accounting Insights + IEAGHG + IEA — practitioner overviews of § 45Q".to_string(),
    ];

    if input.beginning_of_construction_status
        == BeginningOfConstructionStatus::BocOnOrAfterJanuary1_2033PostCutoff
    {
        return Output {
            mode: Section45QMode::NotApplicableBocOnOrAfterJanuary1_2033PostCutoff,
            statutory_basis: "IRC § 45Q(b) — facility must begin construction before January 1, 2033 to qualify".to_string(),
            notes: "NOT APPLICABLE: facility began construction on or after January 1, 2033; § 45Q credit unavailable.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    let tonnage_threshold = match input.facility_type {
        FacilityType::IndustrialFacilityCcus => IRC_45Q_INDUSTRIAL_TONNAGE_THRESHOLD_TONS_PER_YEAR,
        FacilityType::DirectAirCaptureFacility => IRC_45Q_DAC_TONNAGE_THRESHOLD_TONS_PER_YEAR,
    };

    if input.metric_tons_captured_per_year < tonnage_threshold {
        return Output {
            mode: Section45QMode::NotApplicableTonnageBelowThresholdForFacilityType,
            statutory_basis: format!(
                "IRC § 45Q(d)(2) — facility captured below {tonnage_threshold} metric tons/year threshold for facility type"
            ),
            notes: format!(
                "NOT APPLICABLE: facility captured {captured} metric tons/year, below {tonnage_threshold} threshold for {facility:?}.",
                captured = input.metric_tons_captured_per_year,
                facility = input.facility_type,
            ),
            citations,
            computed_credit_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::BaseCreditAmountUnderSection45QA => {
            let rate = base_rate_dollars_per_ton(input.facility_type, input.disposition_method);
            let computed = input.metric_tons_captured_per_year.saturating_mul(rate);
            let mode = base_mode_for_facility_and_disposition(
                input.facility_type,
                input.disposition_method,
            );
            Output {
                mode,
                statutory_basis: format!(
                    "IRC § 45Q(a) — base credit at ${rate}/ton for facility type and disposition method"
                ),
                notes: format!(
                    "COMPLIANT: base credit at ${rate}/ton × {tons} tons = ${computed}.",
                    tons = input.metric_tons_captured_per_year,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::BonusCreditAmountForPwaUnderSection45QA2 => {
            if input.claimed_pwa_bonus_multiplier
                && input.pwa_status
                    == PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly
            {
                return Output {
                    mode: Section45QMode::ViolationPwaBonusMultiplierClaimedWithoutMeetingPwaRequirements,
                    statutory_basis: "IRC § 45Q(a)(2) — PWA bonus 5x multiplier requires prevailing wage and apprenticeship compliance".to_string(),
                    notes: "VIOLATION: PWA bonus 5x multiplier claimed but prevailing wage and apprenticeship requirements not met; only base rate available.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                };
            }
            let rate = pwa_rate_dollars_per_ton(input.facility_type, input.disposition_method);
            let computed = input.metric_tons_captured_per_year.saturating_mul(rate);
            let mode = pwa_mode_for_facility_and_disposition(
                input.facility_type,
                input.disposition_method,
            );
            Output {
                mode,
                statutory_basis: format!(
                    "IRC § 45Q(a)(2) — PWA bonus credit at ${rate}/ton (5x base) for facility type and disposition method"
                ),
                notes: format!(
                    "COMPLIANT: PWA bonus credit at ${rate}/ton (5x base) × {tons} tons = ${computed}; PWA requirements satisfied during construction and 12-year credit period.",
                    tons = input.metric_tons_captured_per_year,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::TonnageThresholdUnderSection45QD2 => Output {
            mode: Section45QMode::CompliantTonnageMeetsThresholdForFacilityType,
            statutory_basis: format!(
                "IRC § 45Q(d)(2) — tonnage meets {tonnage_threshold} threshold for facility type"
            ),
            notes: format!(
                "COMPLIANT: {captured} metric tons/year meets {tonnage_threshold} threshold for {facility:?}.",
                captured = input.metric_tons_captured_per_year,
                facility = input.facility_type,
            ),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::TwelveYearCreditPeriodUnderSection45QA => {
            if input.credit_year_number_within_window == 0
                || input.credit_year_number_within_window > IRC_45Q_CREDIT_PERIOD_YEARS
            {
                Output {
                    mode: Section45QMode::ViolationCreditClaimedOutsideTwelveYearCreditPeriod,
                    statutory_basis: "IRC § 45Q(a) — credit available for 12-year period beginning on date capture equipment placed in service".to_string(),
                    notes: format!(
                        "VIOLATION: credit claimed for year {y} which is outside the 12-year credit period (years 1-12 only); § 45Q credit unavailable for this year.",
                        y = input.credit_year_number_within_window,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45QMode::CompliantTwelveYearCreditPeriodYearWithinWindow,
                    statutory_basis: "IRC § 45Q(a) — credit claimed for year within 12-year credit period beginning on date capture equipment placed in service".to_string(),
                    notes: format!(
                        "COMPLIANT: credit claimed for year {y} within 12-year credit period under § 45Q(a).",
                        y = input.credit_year_number_within_window,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::BeginningOfConstructionDeadlineUnderSection45QB => Output {
            mode: Section45QMode::CompliantBocBeforeJanuary1_2033PreCutoff,
            statutory_basis: "IRC § 45Q(b) — facility began construction before January 1, 2033 (pre-cutoff)".to_string(),
            notes: "COMPLIANT: facility began construction before January 1, 2033; § 45Q credit available under IRA 2022 extended BOC deadline.".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::DirectPayElectionUnderSection6417 => {
            if input.direct_pay_election_made {
                Output {
                    mode: Section45QMode::CompliantDirectPayElectionMade,
                    statutory_basis: "IRC § 6417 — direct pay election made for § 45Q credit (5-year availability for taxable entities; permanent for tax-exempt entities)".to_string(),
                    notes: "COMPLIANT: direct pay election under § 6417 made for § 45Q credit; allows monetization as cash refund.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45QMode::CompliantTwelveYearCreditPeriodYearWithinWindow,
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
                    mode: Section45QMode::CompliantTransferabilityElectionMade,
                    statutory_basis: "IRC § 6418 — transferability election made for § 45Q credit (sale to unrelated third party for cash)".to_string(),
                    notes: "COMPLIANT: transferability election under § 6418 made for § 45Q credit; allows sale of credit to unrelated third party for cash.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45QMode::CompliantTwelveYearCreditPeriodYearWithinWindow,
                    statutory_basis: "IRC § 6418 — transferability election not made; credit claimed by taxpayer".to_string(),
                    notes: "COMPLIANT: transferability election under § 6418 not made; credit claimed by taxpayer as offset to federal income tax liability.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::FormFilingUnderForm8933 => {
            if input.form_8933_filed_correctly {
                Output {
                    mode: Section45QMode::CompliantForm8933FiledCorrectly,
                    statutory_basis: "Form 8933 — Carbon Oxide Sequestration Credit form required to claim § 45Q credit".to_string(),
                    notes: "COMPLIANT: Form 8933 filed correctly to claim § 45Q credit for tax year.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section45QMode::ViolationForm8933NotFiledOrIncorrect,
                    statutory_basis: "Form 8933 filing required to claim § 45Q credit".to_string(),
                    notes: "VIOLATION: Form 8933 not filed or incorrectly filed; § 45Q credit may be disallowed.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
    }
}

fn base_rate_dollars_per_ton(
    facility_type: FacilityType,
    disposition_method: DispositionMethod,
) -> u64 {
    match (facility_type, disposition_method) {
        (FacilityType::IndustrialFacilityCcus, DispositionMethod::SalineGeologicalStorage) => {
            IRC_45Q_INDUSTRIAL_SALINE_BASE_DOLLARS_PER_TON
        }
        (
            FacilityType::IndustrialFacilityCcus,
            DispositionMethod::UtilizationOrEnhancedOilRecovery,
        ) => IRC_45Q_INDUSTRIAL_UTILIZATION_BASE_DOLLARS_PER_TON,
        (FacilityType::DirectAirCaptureFacility, DispositionMethod::SalineGeologicalStorage) => {
            IRC_45Q_DAC_SALINE_BASE_DOLLARS_PER_TON
        }
        (
            FacilityType::DirectAirCaptureFacility,
            DispositionMethod::UtilizationOrEnhancedOilRecovery,
        ) => IRC_45Q_DAC_UTILIZATION_BASE_DOLLARS_PER_TON,
    }
}

fn pwa_rate_dollars_per_ton(
    facility_type: FacilityType,
    disposition_method: DispositionMethod,
) -> u64 {
    match (facility_type, disposition_method) {
        (FacilityType::IndustrialFacilityCcus, DispositionMethod::SalineGeologicalStorage) => {
            IRC_45Q_INDUSTRIAL_SALINE_PWA_DOLLARS_PER_TON
        }
        (
            FacilityType::IndustrialFacilityCcus,
            DispositionMethod::UtilizationOrEnhancedOilRecovery,
        ) => IRC_45Q_INDUSTRIAL_UTILIZATION_PWA_DOLLARS_PER_TON,
        (FacilityType::DirectAirCaptureFacility, DispositionMethod::SalineGeologicalStorage) => {
            IRC_45Q_DAC_SALINE_PWA_DOLLARS_PER_TON
        }
        (
            FacilityType::DirectAirCaptureFacility,
            DispositionMethod::UtilizationOrEnhancedOilRecovery,
        ) => IRC_45Q_DAC_UTILIZATION_PWA_DOLLARS_PER_TON,
    }
}

fn base_mode_for_facility_and_disposition(
    facility_type: FacilityType,
    disposition_method: DispositionMethod,
) -> Section45QMode {
    match (facility_type, disposition_method) {
        (FacilityType::IndustrialFacilityCcus, DispositionMethod::SalineGeologicalStorage) => {
            Section45QMode::CompliantIndustrialSalineBaseCreditAtSeventeenDollarsPerTon
        }
        (
            FacilityType::IndustrialFacilityCcus,
            DispositionMethod::UtilizationOrEnhancedOilRecovery,
        ) => Section45QMode::CompliantIndustrialUtilizationBaseCreditAtTwelveDollarsPerTon,
        (FacilityType::DirectAirCaptureFacility, DispositionMethod::SalineGeologicalStorage) => {
            Section45QMode::CompliantDacSalineBaseCreditAtThirtySixDollarsPerTon
        }
        (
            FacilityType::DirectAirCaptureFacility,
            DispositionMethod::UtilizationOrEnhancedOilRecovery,
        ) => Section45QMode::CompliantDacUtilizationBaseCreditAtTwentySixDollarsPerTon,
    }
}

fn pwa_mode_for_facility_and_disposition(
    facility_type: FacilityType,
    disposition_method: DispositionMethod,
) -> Section45QMode {
    match (facility_type, disposition_method) {
        (FacilityType::IndustrialFacilityCcus, DispositionMethod::SalineGeologicalStorage) => {
            Section45QMode::CompliantIndustrialSalinePwaBonusCreditAtEightyFiveDollarsPerTon
        }
        (
            FacilityType::IndustrialFacilityCcus,
            DispositionMethod::UtilizationOrEnhancedOilRecovery,
        ) => Section45QMode::CompliantIndustrialUtilizationPwaBonusCreditAtSixtyDollarsPerTon,
        (FacilityType::DirectAirCaptureFacility, DispositionMethod::SalineGeologicalStorage) => {
            Section45QMode::CompliantDacSalinePwaBonusCreditAtOneHundredEightyDollarsPerTon
        }
        (
            FacilityType::DirectAirCaptureFacility,
            DispositionMethod::UtilizationOrEnhancedOilRecovery,
        ) => Section45QMode::CompliantDacUtilizationPwaBonusCreditAtOneHundredThirtyDollarsPerTon,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            beginning_of_construction_status:
                BeginningOfConstructionStatus::BocBeforeJanuary1_2033PreCutoff,
            facility_type: FacilityType::IndustrialFacilityCcus,
            disposition_method: DispositionMethod::SalineGeologicalStorage,
            pwa_status:
                PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusMultiplier,
            compliance_aspect: ComplianceAspect::BaseCreditAmountUnderSection45QA,
            metric_tons_captured_per_year: 100_000,
            credit_year_number_within_window: 1,
            direct_pay_election_made: false,
            transferability_election_made: false,
            form_8933_filed_correctly: true,
            claimed_pwa_bonus_multiplier: false,
        }
    }

    #[test]
    fn boc_after_january1_2033_not_applicable() {
        let mut input = baseline_input();
        input.beginning_of_construction_status =
            BeginningOfConstructionStatus::BocOnOrAfterJanuary1_2033PostCutoff;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::NotApplicableBocOnOrAfterJanuary1_2033PostCutoff
        );
    }

    #[test]
    fn industrial_tonnage_below_threshold_not_applicable() {
        let mut input = baseline_input();
        input.facility_type = FacilityType::IndustrialFacilityCcus;
        input.metric_tons_captured_per_year = 12_499;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::NotApplicableTonnageBelowThresholdForFacilityType
        );
    }

    #[test]
    fn dac_tonnage_below_threshold_not_applicable() {
        let mut input = baseline_input();
        input.facility_type = FacilityType::DirectAirCaptureFacility;
        input.metric_tons_captured_per_year = 999;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::NotApplicableTonnageBelowThresholdForFacilityType
        );
    }

    #[test]
    fn industrial_saline_base_credit_at_seventeen_dollars_per_ton_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountUnderSection45QA;
        input.facility_type = FacilityType::IndustrialFacilityCcus;
        input.disposition_method = DispositionMethod::SalineGeologicalStorage;
        input.metric_tons_captured_per_year = 100_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::CompliantIndustrialSalineBaseCreditAtSeventeenDollarsPerTon
        );
        assert_eq!(out.computed_credit_dollars, 1_700_000);
    }

    #[test]
    fn industrial_saline_pwa_bonus_at_eighty_five_dollars_per_ton_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45QA2;
        input.facility_type = FacilityType::IndustrialFacilityCcus;
        input.disposition_method = DispositionMethod::SalineGeologicalStorage;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusMultiplier;
        input.claimed_pwa_bonus_multiplier = true;
        input.metric_tons_captured_per_year = 100_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::CompliantIndustrialSalinePwaBonusCreditAtEightyFiveDollarsPerTon
        );
        assert_eq!(out.computed_credit_dollars, 8_500_000);
    }

    #[test]
    fn industrial_utilization_pwa_bonus_at_sixty_dollars_per_ton_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45QA2;
        input.facility_type = FacilityType::IndustrialFacilityCcus;
        input.disposition_method = DispositionMethod::UtilizationOrEnhancedOilRecovery;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusMultiplier;
        input.claimed_pwa_bonus_multiplier = true;
        input.metric_tons_captured_per_year = 100_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::CompliantIndustrialUtilizationPwaBonusCreditAtSixtyDollarsPerTon
        );
        assert_eq!(out.computed_credit_dollars, 6_000_000);
    }

    #[test]
    fn dac_saline_pwa_bonus_at_one_hundred_eighty_dollars_per_ton_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45QA2;
        input.facility_type = FacilityType::DirectAirCaptureFacility;
        input.disposition_method = DispositionMethod::SalineGeologicalStorage;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusMultiplier;
        input.claimed_pwa_bonus_multiplier = true;
        input.metric_tons_captured_per_year = 100_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::CompliantDacSalinePwaBonusCreditAtOneHundredEightyDollarsPerTon
        );
        assert_eq!(out.computed_credit_dollars, 18_000_000);
    }

    #[test]
    fn dac_utilization_pwa_bonus_at_one_hundred_thirty_dollars_per_ton_computed() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45QA2;
        input.facility_type = FacilityType::DirectAirCaptureFacility;
        input.disposition_method = DispositionMethod::UtilizationOrEnhancedOilRecovery;
        input.pwa_status =
            PrevailingWageApprenticeshipStatus::PwaRequirementsMetEligibleForBonusMultiplier;
        input.claimed_pwa_bonus_multiplier = true;
        input.metric_tons_captured_per_year = 100_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::CompliantDacUtilizationPwaBonusCreditAtOneHundredThirtyDollarsPerTon
        );
        assert_eq!(out.computed_credit_dollars, 13_000_000);
    }

    #[test]
    fn pwa_bonus_is_five_times_base_rate_for_industrial_saline() {
        let base = base_rate_dollars_per_ton(
            FacilityType::IndustrialFacilityCcus,
            DispositionMethod::SalineGeologicalStorage,
        );
        let pwa = pwa_rate_dollars_per_ton(
            FacilityType::IndustrialFacilityCcus,
            DispositionMethod::SalineGeologicalStorage,
        );
        assert_eq!(pwa, base * IRC_45Q_PWA_BONUS_MULTIPLIER);
    }

    #[test]
    fn pwa_bonus_is_five_times_base_rate_for_dac_saline() {
        let base = base_rate_dollars_per_ton(
            FacilityType::DirectAirCaptureFacility,
            DispositionMethod::SalineGeologicalStorage,
        );
        let pwa = pwa_rate_dollars_per_ton(
            FacilityType::DirectAirCaptureFacility,
            DispositionMethod::SalineGeologicalStorage,
        );
        assert_eq!(pwa, base * IRC_45Q_PWA_BONUS_MULTIPLIER);
    }

    #[test]
    fn pwa_bonus_claimed_without_meeting_requirements_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BonusCreditAmountForPwaUnderSection45QA2;
        input.pwa_status = PrevailingWageApprenticeshipStatus::PwaRequirementsNotMetBaseRateOnly;
        input.claimed_pwa_bonus_multiplier = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::ViolationPwaBonusMultiplierClaimedWithoutMeetingPwaRequirements
        );
    }

    #[test]
    fn industrial_tonnage_at_threshold_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TonnageThresholdUnderSection45QD2;
        input.facility_type = FacilityType::IndustrialFacilityCcus;
        input.metric_tons_captured_per_year = 12_500;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::CompliantTonnageMeetsThresholdForFacilityType
        );
    }

    #[test]
    fn dac_tonnage_at_threshold_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TonnageThresholdUnderSection45QD2;
        input.facility_type = FacilityType::DirectAirCaptureFacility;
        input.metric_tons_captured_per_year = 1_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::CompliantTonnageMeetsThresholdForFacilityType
        );
    }

    #[test]
    fn twelve_year_credit_period_year_one_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TwelveYearCreditPeriodUnderSection45QA;
        input.credit_year_number_within_window = 1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::CompliantTwelveYearCreditPeriodYearWithinWindow
        );
    }

    #[test]
    fn twelve_year_credit_period_year_twelve_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TwelveYearCreditPeriodUnderSection45QA;
        input.credit_year_number_within_window = 12;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::CompliantTwelveYearCreditPeriodYearWithinWindow
        );
    }

    #[test]
    fn twelve_year_credit_period_year_thirteen_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TwelveYearCreditPeriodUnderSection45QA;
        input.credit_year_number_within_window = 13;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::ViolationCreditClaimedOutsideTwelveYearCreditPeriod
        );
    }

    #[test]
    fn boc_before_january1_2033_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BeginningOfConstructionDeadlineUnderSection45QB;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::CompliantBocBeforeJanuary1_2033PreCutoff
        );
    }

    #[test]
    fn direct_pay_election_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DirectPayElectionUnderSection6417;
        input.direct_pay_election_made = true;
        let out = check(&input);
        assert_eq!(out.mode, Section45QMode::CompliantDirectPayElectionMade);
    }

    #[test]
    fn transferability_election_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TransferabilityElectionUnderSection6418;
        input.transferability_election_made = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::CompliantTransferabilityElectionMade
        );
    }

    #[test]
    fn form_8933_filed_correctly_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm8933;
        input.form_8933_filed_correctly = true;
        let out = check(&input);
        assert_eq!(out.mode, Section45QMode::CompliantForm8933FiledCorrectly);
    }

    #[test]
    fn form_8933_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm8933;
        input.form_8933_filed_correctly = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45QMode::ViolationForm8933NotFiledOrIncorrect
        );
    }

    #[test]
    fn constants_pin_section_45q_three_phase_legislative_history_and_rates() {
        assert_eq!(IRC_45Q_EIEA_ENACTMENT_DATE_YEAR, 2008);
        assert_eq!(IRC_45Q_EIEA_PUBLIC_LAW_CONGRESS, 110);
        assert_eq!(IRC_45Q_EIEA_PUBLIC_LAW_ENACTMENT, 343);
        assert_eq!(IRC_45Q_BBA_ENACTMENT_DATE_YEAR, 2018);
        assert_eq!(IRC_45Q_BBA_PUBLIC_LAW_CONGRESS, 115);
        assert_eq!(IRC_45Q_BBA_PUBLIC_LAW_ENACTMENT, 123);
        assert_eq!(IRC_45Q_BBA_ENABLING_SECTION, 41119);
        assert_eq!(IRC_45Q_IRA_ENACTMENT_DATE_YEAR, 2022);
        assert_eq!(IRC_45Q_IRA_PUBLIC_LAW_CONGRESS, 117);
        assert_eq!(IRC_45Q_IRA_PUBLIC_LAW_ENACTMENT, 169);
        assert_eq!(IRC_45Q_IRA_ENABLING_SECTION, 13104);
        assert_eq!(IRC_45Q_BOC_CUTOFF_YEAR, 2033);
        assert_eq!(IRC_45Q_INDUSTRIAL_SALINE_BASE_DOLLARS_PER_TON, 17);
        assert_eq!(IRC_45Q_INDUSTRIAL_SALINE_PWA_DOLLARS_PER_TON, 85);
        assert_eq!(IRC_45Q_INDUSTRIAL_UTILIZATION_BASE_DOLLARS_PER_TON, 12);
        assert_eq!(IRC_45Q_INDUSTRIAL_UTILIZATION_PWA_DOLLARS_PER_TON, 60);
        assert_eq!(IRC_45Q_DAC_SALINE_BASE_DOLLARS_PER_TON, 36);
        assert_eq!(IRC_45Q_DAC_SALINE_PWA_DOLLARS_PER_TON, 180);
        assert_eq!(IRC_45Q_DAC_UTILIZATION_BASE_DOLLARS_PER_TON, 26);
        assert_eq!(IRC_45Q_DAC_UTILIZATION_PWA_DOLLARS_PER_TON, 130);
        assert_eq!(IRC_45Q_PWA_BONUS_MULTIPLIER, 5);
        assert_eq!(IRC_45Q_INDUSTRIAL_TONNAGE_THRESHOLD_TONS_PER_YEAR, 12_500);
        assert_eq!(IRC_45Q_DAC_TONNAGE_THRESHOLD_TONS_PER_YEAR, 1_000);
        assert_eq!(IRC_45Q_CREDIT_PERIOD_YEARS, 12);
        assert_eq!(IRC_45Q_OBBBA_STANDARDIZED_BASE_DOLLARS_PER_TON, 17);
        assert_eq!(IRC_45Q_OBBBA_PUBLIC_LAW_CONGRESS, 119);
        assert_eq!(IRC_45Q_OBBBA_PUBLIC_LAW_ENACTMENT, 21);
        assert_eq!(IRC_45Q_DIRECT_PAY_CROSS_REFERENCE_SECTION, 6417);
        assert_eq!(IRC_45Q_DIRECT_PAY_TAXABLE_ENTITY_YEARS, 5);
        assert_eq!(IRC_45Q_TRANSFERABILITY_CROSS_REFERENCE_SECTION, 6418);
        assert_eq!(IRC_45Q_FORM_NUMBER, 8933);
        assert_eq!(IRC_45Q_ORIGINAL_CUMULATIVE_CAP_MILLION_TONS, 75);
    }

    #[test]
    fn citations_pin_three_phase_legislative_history_and_per_ton_rate_structure() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("IRC § 45Q Credit for Carbon Oxide Sequestration"));
        assert!(joined.contains("Energy Improvement and Extension Act of 2008"));
        assert!(joined.contains("Public Law 110-343"));
        assert!(joined.contains("October 3, 2008"));
        assert!(joined.contains("75 million metric tons cumulative"));
        assert!(joined.contains("Bipartisan Budget Act of 2018"));
        assert!(joined.contains("Public Law 115-123"));
        assert!(joined.contains("February 9, 2018"));
        assert!(joined.contains("12-YEAR CREDIT PERIOD"));
        assert!(joined.contains("DIRECT AIR CAPTURE (DAC)"));
        assert!(joined.contains("OWNERS OF CAPTURE EQUIPMENT"));
        assert!(joined.contains("Section 13104 of the Inflation Reduction Act of 2022"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("August 16, 2022"));
        assert!(joined.contains("TWO-TIER PWA STRUCTURE"));
        assert!(joined.contains("$17/ton base"));
        assert!(joined.contains("$85/ton with PWA"));
        assert!(joined.contains("$12/ton base"));
        assert!(joined.contains("$60/ton with PWA"));
        assert!(joined.contains("$36/ton base"));
        assert!(joined.contains("$180/ton with PWA"));
        assert!(joined.contains("$26/ton base"));
        assert!(joined.contains("$130/ton with PWA"));
        assert!(joined.contains("12,500 METRIC TONS"));
        assert!(joined.contains("1,000 METRIC TONS"));
        assert!(joined.contains("JANUARY 1, 2033"));
        assert!(joined.contains("OBBBA 2025"));
        assert!(joined.contains("$17 PER METRIC TON"));
        assert!(joined.contains("§ 6417"));
        assert!(joined.contains("DIRECT PAY ELECTION"));
        assert!(joined.contains("§ 6418"));
        assert!(joined.contains("TRANSFERABILITY ELECTION"));
        assert!(joined.contains("Form 8933"));
    }

    #[test]
    fn saturating_overflow_defense_at_u64_max_tons() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditAmountUnderSection45QA;
        input.metric_tons_captured_per_year = u64::MAX;
        let out = check(&input);
        let _ = out.computed_credit_dollars;
    }
}
