//! IRC § 42 Low-Income Housing Tax Credit (LIHTC)
//! Compliance Module — pure-compute check for the
//! federal government's **PRIMARY METHOD for encouraging
//! the development of affordable rental housing**;
//! established by the **Tax Reform Act of 1986**.
//!
//! Two-tier credit rate structure (9% for new construction
//! / substantial rehab WITHOUT federal subsidy / tax-exempt
//! bonds, allocated competitively; 4% for acquisition of
//! existing buildings OR new construction/rehab WITH
//! tax-exempt bonds, available as-of-right). 10-year
//! credit period; 15-year compliance period; additional
//! 15-year extended use period = **30-year total
//! affordability commitment**.
//!
//! Web research (verified 2026-06-03):
//! - **Original Enactment**: IRC § 42 added by the **Tax Reform Act of 1986 (Public Law 99-514)**, signed by President Ronald Reagan on **October 22, 1986**; replaced previous accelerated depreciation incentives for low-income housing ([Cornell LII — 26 U.S. Code § 42](https://www.law.cornell.edu/uscode/text/26/42); [Tax Notes — IRC Code Section 42 (Low-Income Housing Credit)](https://www.taxnotes.com/research/federal/usc26/42); [Congressional Research Service RS22389 — An Introduction to the Low-Income Housing Tax Credit](https://www.congress.gov/crs-product/RS22389); [Novogradac — Compliance Differences Between IRC 42 and 142: Part 1](https://www.novoco.com/periodicals/articles/compliance-differences-between-irc-42-and-142-part-1); [HUD223f Loans — What are 4% and 9% LIHTC Credits?](https://www.hud223f.loans/glossary/4-and-9-percent-lihtcs/); [IRS — Revenue Ruling 2004-82 Section 42 Low-Income Housing Credit](https://www.irs.gov/pub/irs-drop/rr-04-82.pdf); [IRS — Revenue Ruling 2020-04 Section 42 Low-Income Housing Credit](https://www.irs.gov/pub/irs-drop/rr-20-04.pdf); [NYC HPD — HOME and the Low-Income Housing Tax Credit Guidebook](https://www.nyc.gov/assets/hpd/downloads/pdfs/services/HOME-LowIncomeHousing-Tax-CreditGuidebook.pdf); [Accounting Insights — What is Section 42 of the Internal Revenue Code?](https://accountinginsights.org/what-is-section-42-of-the-internal-revenue-code/); [The Habitat Group — How to Comply with IRC § 42(l)(1) First-Year Certification](https://www.thehabitatgroup.com/how-to-comply-with-irc-section-42l1-first-year-certification/); [Federal Register — Section 42 Low-Income Housing Credit Average Income Test Regulations (October 12, 2022)](https://www.federalregister.gov/documents/2022/10/12/2022-22070/section-42-low-income-housing-credit-average-income-test-regulations); [Housing Finance — Confronting Minimum Set-Asides and Applicable Fractions](https://www.housingfinance.com/news/confronting-minimum-set-asides-and-applicable-fractions_o); [CohnReznick — Final Regulations Issued for LIHTC Average Income Test](https://www.cohnreznick.com/insights/final-regulations-issued-lihtc-average-income-test); [Westmont Advisors — How to Calculate the Low Income Housing Tax Credit](https://westmontadvisors.com/tax-credit-advisory/how-to-calculate-the-low-income-housing-tax-credit-lihtc/); [IRS — IRC § 42 Low-Income Housing Credit Part IV Applicable Fraction PDF](https://www.irs.gov/pub/irs-utl/irc42_low_income_housing_credit_atg_part_4.pdf)).
//! - **§ 42(b) Two-Tier Credit Rate Structure**: **9 PERCENT LIHTC** generally applies to **NEW CONSTRUCTION OR REHABILITATION COSTS WITHOUT TAX-EXEMPT HOUSING BONDS** — allocated competitively by state housing finance agencies subject to **PER-CAPITA STATE CEILINGS**. **4 PERCENT LIHTC** applies to (i) **ACQUISITION OF EXISTING BUILDINGS** OR (ii) **NEW CONSTRUCTION OR REHABILITATION COSTS WITH TAX-EXEMPT HOUSING BONDS** — available **AS-OF-RIGHT** when project is financed with at least 50% tax-exempt bonds.
//! - **§ 42(f) 10-Year Credit Period**: credit period is the **10-YEAR SPAN** when the owner can claim tax credits, **BEGINNING IN THE YEAR THE BUILDING IS PLACED IN SERVICE**.
//! - **§ 42(i)(1) 15-Year Compliance Period**: the **15-YEAR COMPLIANCE PERIOD** is when the project must adhere to **ALL LIHTC RULES**, including the chosen set-aside test and rent restrictions; failure to maintain compliance during this period leads to **CREDIT RECAPTURE**.
//! - **§ 42(h)(6) Extended Use Agreement — 30-Year Affordability**: in addition to the 15-year compliance period, project owners must enter into an **EXTENDED USE AGREEMENT** with the state housing finance agency requiring an additional **15 YEARS of affordability** (total 30 years from placed-in-service date).
//! - **§ 42(g)(1) Set-Aside Election — Three Tests**: project owners must elect ONE of three set-aside tests: **(1) 20/50 TEST** — at least **20%** of units rent-restricted to households at or below **50% of Area Median Income (AMI)**; **(2) 40/60 TEST** — at least **40%** of units rent-restricted to households at or below **60% of AMI**; **(3) AVERAGE INCOME TEST** (added by Consolidated Appropriations Act of 2018) — at least **40%** of units rent-restricted with **AVERAGE INCOME LIMITATION 60% of AMI** (individual designations may range from 20% to 80% AMI).
//! - **§ 42(c)(1) Qualified Basis = Eligible Basis × Applicable Fraction**: the **QUALIFIED BASIS** is equal to the **ELIGIBLE BASIS** (depreciable basis of the building) **MULTIPLIED BY THE APPLICABLE FRACTION**.
//! - **§ 42(c)(1)(B) Applicable Fraction**: the term "applicable fraction" means the **SMALLER OF (i) the UNIT FRACTION** (low-income units / total units) **OR (ii) the FLOOR SPACE FRACTION** (low-income floor space / total floor space).
//! - **§ 42(g)(2) Rent Restriction**: the gross rent for the unit may **NOT EXCEED 30% OF the qualifying income** for the unit (e.g., for a 60% AMI unit, maximum rent is 30% × 60% × AMI = **18% of AMI**); rent restriction includes utility allowances under § 42(g)(2)(B)(ii).
//! - **§ 42(g)(2)(A) Imputed Income Limitation**: for purposes of the rent restriction, the imputed income limitation is based on family sizes derived from **HUD published income limits** for the area.
//! - **§ 42(j) Recapture**: if the project fails to maintain compliance during the 15-year compliance period, the IRS may **RECAPTURE** all or a portion of the credits previously claimed, plus interest; recapture is **ONE-THIRD OF ACCELERATED CREDIT** in years 11-15 of compliance period.
//! - **§ 42(j)(4)(E) Decrease in Qualified Basis**: if the qualified basis decreases (e.g., a low-income unit is converted to market rate), the credit is **PROPORTIONALLY REDUCED** and prior years' credits are recaptured.
//! - **State Per-Capita 9% Ceiling — Inflation Adjusted**: 9% LIHTC is allocated by state housing finance agencies subject to a **PER-CAPITA STATE CEILING** indexed for inflation; for 2026, the per-capita allocation is approximately **$3.00 per resident** with state minimum of approximately **$3.5 million**.
//! - **Form 8609 (Low-Income Housing Credit Allocation and Certification)**: required to claim § 42 credit; Form 8609-A required annually for ongoing compliance certification.
//! - **Final Regulations Average Income Test (October 12, 2022)**: Treasury and IRS issued final regulations on the Average Income test under § 42 published in the Federal Register on October 12, 2022; provides for designation of "qualified group of units" for the average income calculation.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_42_TAX_REFORM_ACT_ENACTMENT_DATE_YEAR: u32 = 1986;
pub const IRC_42_TAX_REFORM_ACT_ENACTMENT_DATE_MONTH: u32 = 10;
pub const IRC_42_TAX_REFORM_ACT_ENACTMENT_DATE_DAY: u32 = 22;
pub const IRC_42_TAX_REFORM_ACT_PUBLIC_LAW_CONGRESS: u32 = 99;
pub const IRC_42_TAX_REFORM_ACT_PUBLIC_LAW_ENACTMENT: u32 = 514;
pub const IRC_42_AVERAGE_INCOME_TEST_FINAL_REGS_YEAR: u32 = 2022;
pub const IRC_42_AVERAGE_INCOME_TEST_FINAL_REGS_MONTH: u32 = 10;
pub const IRC_42_AVERAGE_INCOME_TEST_FINAL_REGS_DAY: u32 = 12;
pub const IRC_42_NINE_PERCENT_RATE_BPS: u64 = 900;
pub const IRC_42_FOUR_PERCENT_RATE_BPS: u64 = 400;
pub const IRC_42_CREDIT_PERIOD_YEARS: u32 = 10;
pub const IRC_42_COMPLIANCE_PERIOD_YEARS: u32 = 15;
pub const IRC_42_EXTENDED_USE_PERIOD_YEARS: u32 = 15;
pub const IRC_42_TOTAL_AFFORDABILITY_PERIOD_YEARS: u32 = 30;
pub const IRC_42_SET_ASIDE_20_50_UNIT_PCT_BPS: u64 = 2_000;
pub const IRC_42_SET_ASIDE_20_50_AMI_PCT_BPS: u64 = 5_000;
pub const IRC_42_SET_ASIDE_40_60_UNIT_PCT_BPS: u64 = 4_000;
pub const IRC_42_SET_ASIDE_40_60_AMI_PCT_BPS: u64 = 6_000;
pub const IRC_42_SET_ASIDE_AVERAGE_INCOME_UNIT_PCT_BPS: u64 = 4_000;
pub const IRC_42_SET_ASIDE_AVERAGE_INCOME_AVERAGE_AMI_PCT_BPS: u64 = 6_000;
pub const IRC_42_RENT_RESTRICTION_GROSS_RENT_OF_QUALIFYING_INCOME_BPS: u64 = 3_000;
pub const IRC_42_AVERAGE_INCOME_DESIGNATION_MIN_AMI_PCT_BPS: u64 = 2_000;
pub const IRC_42_AVERAGE_INCOME_DESIGNATION_MAX_AMI_PCT_BPS: u64 = 8_000;
pub const IRC_42_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_42_BOND_FINANCED_MIN_PCT_BPS: u64 = 5_000;
pub const IRC_42_FORM_NUMBER: u32 = 8609;
pub const IRC_42_FORM_8609A_NUMBER: u32 = 8609;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditRateTier {
    NinePercentNewConstructionOrRehabWithoutBonds,
    FourPercentAcquisitionOrBondFinanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SetAsideElection {
    Test2050,
    Test4060,
    AverageIncome,
    NoSetAsideElectionMade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompliancePeriodStatus {
    WithinFifteenYearCompliancePeriod,
    WithinExtendedUsePeriod,
    AfterTotalAffordabilityPeriodEnded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    CreditRateTwoTierStructureUnderSection42B,
    TenYearCreditPeriodUnderSection42F,
    FifteenYearCompliancePeriodUnderSection42I1,
    ThirtyYearExtendedUseAgreementUnderSection42H6,
    SetAsideElectionUnderSection42G1,
    QualifiedBasisFormulaUnderSection42C1,
    ApplicableFractionUnderSection42C1B,
    RentRestrictionUnderSection42G2,
    RecaptureUnderSection42J,
    FormFilingUnderForm8609,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section42Mode {
    CompliantNinePercentCreditForNewConstructionWithoutBonds,
    CompliantFourPercentCreditForAcquisitionOrBondFinanced,
    CompliantTenYearCreditPeriodYearWithinWindow,
    CompliantFifteenYearCompliancePeriodMet,
    CompliantExtendedUseAgreementInForce,
    CompliantSetAsideElection2050Met,
    CompliantSetAsideElection4060Met,
    CompliantSetAsideElectionAverageIncomeMet,
    CompliantQualifiedBasisComputed,
    CompliantApplicableFractionIsSmallerOfUnitOrFloorSpace,
    CompliantRentRestrictionAtOrBelow30PercentOfQualifyingIncome,
    CompliantNoRecaptureTriggered,
    CompliantForm8609FiledCorrectly,
    ViolationNoSetAsideElectionMade,
    ViolationSetAsideElectionThresholdNotMet,
    ViolationRentRestrictionExceeded30PercentOfQualifyingIncome,
    ViolationCreditClaimedOutsideTenYearCreditPeriod,
    ViolationDecreaseInQualifiedBasisTriggersRecapture,
    ViolationExtendedUseAgreementBreached,
    ViolationForm8609NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub credit_rate_tier: CreditRateTier,
    pub set_aside_election: SetAsideElection,
    pub compliance_period_status: CompliancePeriodStatus,
    pub compliance_aspect: ComplianceAspect,
    pub eligible_basis_dollars: u64,
    pub low_income_units: u64,
    pub total_units: u64,
    pub low_income_floor_space_sq_ft: u64,
    pub total_floor_space_sq_ft: u64,
    pub credit_year_number_within_window: u32,
    pub years_since_placed_in_service: u32,
    pub gross_rent_dollars_per_month: u64,
    pub qualifying_income_dollars_per_month: u64,
    pub set_aside_election_unit_pct_bps: u64,
    pub set_aside_election_ami_pct_bps: u64,
    pub decrease_in_qualified_basis: bool,
    pub extended_use_agreement_breached: bool,
    pub form_8609_filed_correctly: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section42Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section42Input = Input;
pub type Section42Output = Output;
pub type Section42Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 42 Low-Income Housing Tax Credit (LIHTC) originally added by the Tax Reform Act of 1986 (Public Law 99-514), signed by President Ronald Reagan on October 22, 1986; replaced previous accelerated depreciation incentives for low-income housing".to_string(),
        "IRC § 42(b) Two-Tier Credit Rate Structure — 9 PERCENT LIHTC generally applies to NEW CONSTRUCTION OR REHABILITATION COSTS WITHOUT TAX-EXEMPT HOUSING BONDS (allocated competitively by state housing finance agencies subject to per-capita state ceilings); 4 PERCENT LIHTC applies to (i) ACQUISITION OF EXISTING BUILDINGS OR (ii) NEW CONSTRUCTION OR REHABILITATION COSTS WITH TAX-EXEMPT HOUSING BONDS (available as-of-right when project is financed with at least 50 % tax-exempt bonds)".to_string(),
        "IRC § 42(f) 10-Year Credit Period — credit period is the 10-YEAR SPAN when the owner can claim tax credits, BEGINNING IN THE YEAR THE BUILDING IS PLACED IN SERVICE".to_string(),
        "IRC § 42(i)(1) 15-Year Compliance Period — the 15-YEAR COMPLIANCE PERIOD is when the project must adhere to ALL LIHTC RULES, including the chosen set-aside test and rent restrictions; failure to maintain compliance during this period leads to CREDIT RECAPTURE".to_string(),
        "IRC § 42(h)(6) Extended Use Agreement — 30-Year Affordability — in addition to the 15-year compliance period, project owners must enter into an EXTENDED USE AGREEMENT with the state housing finance agency requiring an additional 15 YEARS of affordability (total 30 years from placed-in-service date)".to_string(),
        "IRC § 42(g)(1) Set-Aside Election — Three Tests — (1) 20/50 TEST: at least 20 % of units rent-restricted to households at or below 50 % of Area Median Income (AMI); (2) 40/60 TEST: at least 40 % of units rent-restricted to households at or below 60 % of AMI; (3) AVERAGE INCOME TEST (added by Consolidated Appropriations Act of 2018): at least 40 % of units rent-restricted with AVERAGE INCOME LIMITATION 60 % of AMI (individual designations may range from 20 % to 80 % AMI)".to_string(),
        "IRC § 42(c)(1) Qualified Basis = Eligible Basis × Applicable Fraction — the QUALIFIED BASIS is equal to the ELIGIBLE BASIS (depreciable basis of the building) MULTIPLIED BY THE APPLICABLE FRACTION".to_string(),
        "IRC § 42(c)(1)(B) Applicable Fraction — the term applicable fraction means the SMALLER OF (i) the UNIT FRACTION (low-income units / total units) OR (ii) the FLOOR SPACE FRACTION (low-income floor space / total floor space)".to_string(),
        "IRC § 42(g)(2) Rent Restriction — the gross rent for the unit may NOT EXCEED 30 % of the qualifying income for the unit (e.g., for a 60 % AMI unit, maximum rent is 30 % × 60 % × AMI = 18 % of AMI); rent restriction includes utility allowances under § 42(g)(2)(B)(ii)".to_string(),
        "IRC § 42(g)(2)(A) Imputed Income Limitation — for purposes of the rent restriction, the imputed income limitation is based on family sizes derived from HUD published income limits for the area".to_string(),
        "IRC § 42(j) Recapture — if the project fails to maintain compliance during the 15-year compliance period, the IRS may RECAPTURE all or a portion of the credits previously claimed, plus interest".to_string(),
        "IRC § 42(j)(4)(E) Decrease in Qualified Basis — if the qualified basis decreases (e.g., a low-income unit is converted to market rate), the credit is PROPORTIONALLY REDUCED and prior years' credits are recaptured".to_string(),
        "State Per-Capita 9% Ceiling — 9% LIHTC is allocated by state housing finance agencies subject to a PER-CAPITA STATE CEILING indexed for inflation".to_string(),
        "Form 8609 (Low-Income Housing Credit Allocation and Certification) — required to claim § 42 credit; Form 8609-A required annually for ongoing compliance certification".to_string(),
        "Final Regulations Average Income Test (October 12, 2022) — Treasury and IRS issued final regulations on the Average Income test under § 42; provides for designation of qualified group of units for the average income calculation".to_string(),
        "Cornell LII + Tax Notes + Congressional Research Service RS22389 + Novogradac + HUD223f Loans + IRS + NYC HPD + Accounting Insights + The Habitat Group + Federal Register + Housing Finance + CohnReznick + Westmont Advisors — practitioner overviews of § 42".to_string(),
    ];

    match input.compliance_aspect {
        ComplianceAspect::CreditRateTwoTierStructureUnderSection42B => {
            let rate_bps = match input.credit_rate_tier {
                CreditRateTier::NinePercentNewConstructionOrRehabWithoutBonds => {
                    IRC_42_NINE_PERCENT_RATE_BPS
                }
                CreditRateTier::FourPercentAcquisitionOrBondFinanced => IRC_42_FOUR_PERCENT_RATE_BPS,
            };
            let qualified_basis = compute_qualified_basis(input);
            let computed = (u128::from(qualified_basis) * u128::from(rate_bps)
                / u128::from(IRC_42_BASIS_POINT_DENOMINATOR)) as u64;
            let mode = match input.credit_rate_tier {
                CreditRateTier::NinePercentNewConstructionOrRehabWithoutBonds => {
                    Section42Mode::CompliantNinePercentCreditForNewConstructionWithoutBonds
                }
                CreditRateTier::FourPercentAcquisitionOrBondFinanced => {
                    Section42Mode::CompliantFourPercentCreditForAcquisitionOrBondFinanced
                }
            };
            Output {
                mode,
                statutory_basis: format!(
                    "IRC § 42(b) — {rate_bps} bps credit rate × qualified basis = annual credit"
                ),
                notes: format!(
                    "COMPLIANT: {rate_bps} bps credit rate × ${qualified_basis} qualified basis = ${computed} annual credit."
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::TenYearCreditPeriodUnderSection42F => {
            if input.credit_year_number_within_window == 0
                || input.credit_year_number_within_window > IRC_42_CREDIT_PERIOD_YEARS
            {
                Output {
                    mode: Section42Mode::ViolationCreditClaimedOutsideTenYearCreditPeriod,
                    statutory_basis: "IRC § 42(f) — credit period is 10 years beginning in year building placed in service".to_string(),
                    notes: format!(
                        "VIOLATION: credit claimed for year {y} outside 10-year credit period under § 42(f).",
                        y = input.credit_year_number_within_window,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section42Mode::CompliantTenYearCreditPeriodYearWithinWindow,
                    statutory_basis: "IRC § 42(f) — credit claimed for year within 10-year credit period".to_string(),
                    notes: format!(
                        "COMPLIANT: credit claimed for year {y} within 10-year credit period under § 42(f).",
                        y = input.credit_year_number_within_window,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::FifteenYearCompliancePeriodUnderSection42I1 => {
            if input.compliance_period_status
                == CompliancePeriodStatus::WithinFifteenYearCompliancePeriod
            {
                Output {
                    mode: Section42Mode::CompliantFifteenYearCompliancePeriodMet,
                    statutory_basis: "IRC § 42(i)(1) — within 15-year compliance period; compliance with set-aside and rent restrictions required".to_string(),
                    notes: format!(
                        "COMPLIANT: within 15-year compliance period at year {y} since placed in service; all LIHTC rules apply.",
                        y = input.years_since_placed_in_service,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section42Mode::CompliantExtendedUseAgreementInForce,
                    statutory_basis: "IRC § 42(i)(1) — outside initial 15-year compliance period; extended use agreement governs".to_string(),
                    notes: "COMPLIANT: outside initial 15-year compliance period; extended use agreement remains in force for an additional 15 years (total 30-year affordability).".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::ThirtyYearExtendedUseAgreementUnderSection42H6 => {
            if input.extended_use_agreement_breached {
                Output {
                    mode: Section42Mode::ViolationExtendedUseAgreementBreached,
                    statutory_basis: "IRC § 42(h)(6) — extended use agreement breached".to_string(),
                    notes: "VIOLATION: extended use agreement breached; project lost affordability commitment before 30-year total period ended.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section42Mode::CompliantExtendedUseAgreementInForce,
                    statutory_basis: "IRC § 42(h)(6) — extended use agreement in force; 30-year total affordability commitment maintained".to_string(),
                    notes: "COMPLIANT: extended use agreement in force; project maintains 30-year total affordability commitment under § 42(h)(6).".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::SetAsideElectionUnderSection42G1 => match input.set_aside_election {
            SetAsideElection::Test2050 => {
                if input.set_aside_election_unit_pct_bps >= IRC_42_SET_ASIDE_20_50_UNIT_PCT_BPS
                    && input.set_aside_election_ami_pct_bps <= IRC_42_SET_ASIDE_20_50_AMI_PCT_BPS
                {
                    Output {
                        mode: Section42Mode::CompliantSetAsideElection2050Met,
                        statutory_basis: "IRC § 42(g)(1) — 20/50 set-aside test met (at least 20 % of units at or below 50 % AMI)".to_string(),
                        notes: "COMPLIANT: 20/50 set-aside test met under § 42(g)(1); at least 20 % of units rent-restricted to households at or below 50 % AMI.".to_string(),
                        citations,
                        computed_credit_dollars: 0,
                    }
                } else {
                    Output {
                        mode: Section42Mode::ViolationSetAsideElectionThresholdNotMet,
                        statutory_basis: "IRC § 42(g)(1) — 20/50 set-aside test thresholds not met".to_string(),
                        notes: "VIOLATION: 20/50 set-aside test thresholds not met under § 42(g)(1); requires at least 20 % of units at or below 50 % AMI.".to_string(),
                        citations,
                        computed_credit_dollars: 0,
                    }
                }
            }
            SetAsideElection::Test4060 => {
                if input.set_aside_election_unit_pct_bps >= IRC_42_SET_ASIDE_40_60_UNIT_PCT_BPS
                    && input.set_aside_election_ami_pct_bps <= IRC_42_SET_ASIDE_40_60_AMI_PCT_BPS
                {
                    Output {
                        mode: Section42Mode::CompliantSetAsideElection4060Met,
                        statutory_basis: "IRC § 42(g)(1) — 40/60 set-aside test met (at least 40 % of units at or below 60 % AMI)".to_string(),
                        notes: "COMPLIANT: 40/60 set-aside test met under § 42(g)(1); at least 40 % of units rent-restricted to households at or below 60 % AMI.".to_string(),
                        citations,
                        computed_credit_dollars: 0,
                    }
                } else {
                    Output {
                        mode: Section42Mode::ViolationSetAsideElectionThresholdNotMet,
                        statutory_basis: "IRC § 42(g)(1) — 40/60 set-aside test thresholds not met".to_string(),
                        notes: "VIOLATION: 40/60 set-aside test thresholds not met under § 42(g)(1); requires at least 40 % of units at or below 60 % AMI.".to_string(),
                        citations,
                        computed_credit_dollars: 0,
                    }
                }
            }
            SetAsideElection::AverageIncome => {
                if input.set_aside_election_unit_pct_bps
                    >= IRC_42_SET_ASIDE_AVERAGE_INCOME_UNIT_PCT_BPS
                    && input.set_aside_election_ami_pct_bps
                        <= IRC_42_SET_ASIDE_AVERAGE_INCOME_AVERAGE_AMI_PCT_BPS
                {
                    Output {
                        mode: Section42Mode::CompliantSetAsideElectionAverageIncomeMet,
                        statutory_basis: "IRC § 42(g)(1) — Average Income set-aside test met (at least 40 % of units with average income limitation 60 % AMI)".to_string(),
                        notes: "COMPLIANT: Average Income set-aside test met under § 42(g)(1) (added by Consolidated Appropriations Act of 2018); at least 40 % of units rent-restricted with average income limitation 60 % AMI; individual designations may range from 20 % to 80 % AMI.".to_string(),
                        citations,
                        computed_credit_dollars: 0,
                    }
                } else {
                    Output {
                        mode: Section42Mode::ViolationSetAsideElectionThresholdNotMet,
                        statutory_basis: "IRC § 42(g)(1) — Average Income set-aside test thresholds not met".to_string(),
                        notes: "VIOLATION: Average Income set-aside test thresholds not met under § 42(g)(1).".to_string(),
                        citations,
                        computed_credit_dollars: 0,
                    }
                }
            }
            SetAsideElection::NoSetAsideElectionMade => Output {
                mode: Section42Mode::ViolationNoSetAsideElectionMade,
                statutory_basis: "IRC § 42(g)(1) — no set-aside election made; § 42 credit unavailable".to_string(),
                notes: "VIOLATION: no set-aside election made under § 42(g)(1); project must elect 20/50 / 40/60 / Average Income test.".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
        },
        ComplianceAspect::QualifiedBasisFormulaUnderSection42C1 => {
            let qualified_basis = compute_qualified_basis(input);
            Output {
                mode: Section42Mode::CompliantQualifiedBasisComputed,
                statutory_basis: "IRC § 42(c)(1) — qualified basis = eligible basis × applicable fraction".to_string(),
                notes: format!(
                    "COMPLIANT: qualified basis = ${eb} eligible basis × applicable fraction = ${qualified_basis}.",
                    eb = input.eligible_basis_dollars,
                ),
                citations,
                computed_credit_dollars: qualified_basis,
            }
        }
        ComplianceAspect::ApplicableFractionUnderSection42C1B => {
            let unit_fraction_bps = if input.total_units == 0 {
                0
            } else {
                (u128::from(input.low_income_units) * u128::from(IRC_42_BASIS_POINT_DENOMINATOR)
                    / u128::from(input.total_units)) as u64
            };
            let floor_space_fraction_bps = if input.total_floor_space_sq_ft == 0 {
                0
            } else {
                (u128::from(input.low_income_floor_space_sq_ft)
                    * u128::from(IRC_42_BASIS_POINT_DENOMINATOR)
                    / u128::from(input.total_floor_space_sq_ft)) as u64
            };
            let applicable_fraction_bps = unit_fraction_bps.min(floor_space_fraction_bps);
            Output {
                mode: Section42Mode::CompliantApplicableFractionIsSmallerOfUnitOrFloorSpace,
                statutory_basis: "IRC § 42(c)(1)(B) — applicable fraction = smaller of unit fraction or floor space fraction".to_string(),
                notes: format!(
                    "COMPLIANT: applicable fraction = smaller of {unit_fraction_bps} bps unit fraction or {floor_space_fraction_bps} bps floor space fraction = {applicable_fraction_bps} bps."
                ),
                citations,
                computed_credit_dollars: 0,
            }
        }
        ComplianceAspect::RentRestrictionUnderSection42G2 => {
            let max_allowable_rent_dollars = (u128::from(input.qualifying_income_dollars_per_month)
                * u128::from(IRC_42_RENT_RESTRICTION_GROSS_RENT_OF_QUALIFYING_INCOME_BPS)
                / u128::from(IRC_42_BASIS_POINT_DENOMINATOR)) as u64;
            if input.gross_rent_dollars_per_month <= max_allowable_rent_dollars {
                Output {
                    mode: Section42Mode::CompliantRentRestrictionAtOrBelow30PercentOfQualifyingIncome,
                    statutory_basis: "IRC § 42(g)(2) — gross rent at or below 30 % of qualifying income".to_string(),
                    notes: format!(
                        "COMPLIANT: gross rent of ${gr}/month at or below ${max} maximum (30 % × ${qi} qualifying income) under § 42(g)(2).",
                        gr = input.gross_rent_dollars_per_month,
                        max = max_allowable_rent_dollars,
                        qi = input.qualifying_income_dollars_per_month,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section42Mode::ViolationRentRestrictionExceeded30PercentOfQualifyingIncome,
                    statutory_basis: "IRC § 42(g)(2) — gross rent exceeded 30 % of qualifying income".to_string(),
                    notes: format!(
                        "VIOLATION: gross rent of ${gr}/month exceeds ${max} maximum (30 % × ${qi} qualifying income) under § 42(g)(2).",
                        gr = input.gross_rent_dollars_per_month,
                        max = max_allowable_rent_dollars,
                        qi = input.qualifying_income_dollars_per_month,
                    ),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::RecaptureUnderSection42J => {
            if input.decrease_in_qualified_basis {
                Output {
                    mode: Section42Mode::ViolationDecreaseInQualifiedBasisTriggersRecapture,
                    statutory_basis: "IRC § 42(j)(4)(E) — decrease in qualified basis triggers recapture".to_string(),
                    notes: "VIOLATION: decrease in qualified basis triggers credit recapture under § 42(j)(4)(E); credit is proportionally reduced and prior years' credits are recaptured plus interest.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section42Mode::CompliantNoRecaptureTriggered,
                    statutory_basis: "IRC § 42(j) — no recapture triggered".to_string(),
                    notes: "COMPLIANT: no recapture triggered under § 42(j); qualified basis maintained during compliance period.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
        ComplianceAspect::FormFilingUnderForm8609 => {
            if input.form_8609_filed_correctly {
                Output {
                    mode: Section42Mode::CompliantForm8609FiledCorrectly,
                    statutory_basis: "Form 8609 — Low-Income Housing Credit Allocation and Certification form required to claim § 42 credit".to_string(),
                    notes: "COMPLIANT: Form 8609 filed correctly to claim § 42 credit; Form 8609-A required annually for ongoing compliance certification.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section42Mode::ViolationForm8609NotFiledOrIncorrect,
                    statutory_basis: "Form 8609 filing required to claim § 42 credit".to_string(),
                    notes: "VIOLATION: Form 8609 not filed or incorrectly filed; § 42 credit may be disallowed.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            }
        }
    }
}

fn compute_qualified_basis(input: &Input) -> u64 {
    let unit_fraction_bps = if input.total_units == 0 {
        0
    } else {
        (u128::from(input.low_income_units) * u128::from(IRC_42_BASIS_POINT_DENOMINATOR)
            / u128::from(input.total_units)) as u64
    };
    let floor_space_fraction_bps = if input.total_floor_space_sq_ft == 0 {
        0
    } else {
        (u128::from(input.low_income_floor_space_sq_ft)
            * u128::from(IRC_42_BASIS_POINT_DENOMINATOR)
            / u128::from(input.total_floor_space_sq_ft)) as u64
    };
    let applicable_fraction_bps = unit_fraction_bps.min(floor_space_fraction_bps);
    (u128::from(input.eligible_basis_dollars) * u128::from(applicable_fraction_bps)
        / u128::from(IRC_42_BASIS_POINT_DENOMINATOR)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            credit_rate_tier: CreditRateTier::NinePercentNewConstructionOrRehabWithoutBonds,
            set_aside_election: SetAsideElection::Test4060,
            compliance_period_status: CompliancePeriodStatus::WithinFifteenYearCompliancePeriod,
            compliance_aspect: ComplianceAspect::CreditRateTwoTierStructureUnderSection42B,
            eligible_basis_dollars: 10_000_000,
            low_income_units: 100,
            total_units: 100,
            low_income_floor_space_sq_ft: 100_000,
            total_floor_space_sq_ft: 100_000,
            credit_year_number_within_window: 1,
            years_since_placed_in_service: 1,
            gross_rent_dollars_per_month: 1_000,
            qualifying_income_dollars_per_month: 4_000,
            set_aside_election_unit_pct_bps: 4_000,
            set_aside_election_ami_pct_bps: 6_000,
            decrease_in_qualified_basis: false,
            extended_use_agreement_breached: false,
            form_8609_filed_correctly: true,
        }
    }

    #[test]
    fn nine_percent_credit_for_new_construction_without_bonds_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditRateTwoTierStructureUnderSection42B;
        input.credit_rate_tier = CreditRateTier::NinePercentNewConstructionOrRehabWithoutBonds;
        input.eligible_basis_dollars = 10_000_000;
        input.low_income_units = 100;
        input.total_units = 100;
        input.low_income_floor_space_sq_ft = 100_000;
        input.total_floor_space_sq_ft = 100_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::CompliantNinePercentCreditForNewConstructionWithoutBonds
        );
        assert_eq!(out.computed_credit_dollars, 900_000);
    }

    #[test]
    fn four_percent_credit_for_acquisition_or_bond_financed_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditRateTwoTierStructureUnderSection42B;
        input.credit_rate_tier = CreditRateTier::FourPercentAcquisitionOrBondFinanced;
        input.eligible_basis_dollars = 10_000_000;
        input.low_income_units = 100;
        input.total_units = 100;
        input.low_income_floor_space_sq_ft = 100_000;
        input.total_floor_space_sq_ft = 100_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::CompliantFourPercentCreditForAcquisitionOrBondFinanced
        );
        assert_eq!(out.computed_credit_dollars, 400_000);
    }

    #[test]
    fn ten_year_credit_period_year_one_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenYearCreditPeriodUnderSection42F;
        input.credit_year_number_within_window = 1;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::CompliantTenYearCreditPeriodYearWithinWindow
        );
    }

    #[test]
    fn ten_year_credit_period_year_ten_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenYearCreditPeriodUnderSection42F;
        input.credit_year_number_within_window = 10;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::CompliantTenYearCreditPeriodYearWithinWindow
        );
    }

    #[test]
    fn ten_year_credit_period_year_eleven_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TenYearCreditPeriodUnderSection42F;
        input.credit_year_number_within_window = 11;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::ViolationCreditClaimedOutsideTenYearCreditPeriod
        );
    }

    #[test]
    fn fifteen_year_compliance_period_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FifteenYearCompliancePeriodUnderSection42I1;
        input.compliance_period_status = CompliancePeriodStatus::WithinFifteenYearCompliancePeriod;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::CompliantFifteenYearCompliancePeriodMet
        );
    }

    #[test]
    fn extended_use_agreement_in_force_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThirtyYearExtendedUseAgreementUnderSection42H6;
        input.extended_use_agreement_breached = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::CompliantExtendedUseAgreementInForce
        );
    }

    #[test]
    fn extended_use_agreement_breached_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThirtyYearExtendedUseAgreementUnderSection42H6;
        input.extended_use_agreement_breached = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::ViolationExtendedUseAgreementBreached
        );
    }

    #[test]
    fn set_aside_election_20_50_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SetAsideElectionUnderSection42G1;
        input.set_aside_election = SetAsideElection::Test2050;
        input.set_aside_election_unit_pct_bps = 2_000;
        input.set_aside_election_ami_pct_bps = 5_000;
        let out = check(&input);
        assert_eq!(out.mode, Section42Mode::CompliantSetAsideElection2050Met);
    }

    #[test]
    fn set_aside_election_40_60_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SetAsideElectionUnderSection42G1;
        input.set_aside_election = SetAsideElection::Test4060;
        input.set_aside_election_unit_pct_bps = 4_000;
        input.set_aside_election_ami_pct_bps = 6_000;
        let out = check(&input);
        assert_eq!(out.mode, Section42Mode::CompliantSetAsideElection4060Met);
    }

    #[test]
    fn set_aside_election_average_income_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SetAsideElectionUnderSection42G1;
        input.set_aside_election = SetAsideElection::AverageIncome;
        input.set_aside_election_unit_pct_bps = 4_000;
        input.set_aside_election_ami_pct_bps = 6_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::CompliantSetAsideElectionAverageIncomeMet
        );
    }

    #[test]
    fn no_set_aside_election_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SetAsideElectionUnderSection42G1;
        input.set_aside_election = SetAsideElection::NoSetAsideElectionMade;
        let out = check(&input);
        assert_eq!(out.mode, Section42Mode::ViolationNoSetAsideElectionMade);
    }

    #[test]
    fn set_aside_election_threshold_not_met_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SetAsideElectionUnderSection42G1;
        input.set_aside_election = SetAsideElection::Test4060;
        input.set_aside_election_unit_pct_bps = 3_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::ViolationSetAsideElectionThresholdNotMet
        );
    }

    #[test]
    fn qualified_basis_computed_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::QualifiedBasisFormulaUnderSection42C1;
        input.eligible_basis_dollars = 10_000_000;
        input.low_income_units = 100;
        input.total_units = 100;
        input.low_income_floor_space_sq_ft = 100_000;
        input.total_floor_space_sq_ft = 100_000;
        let out = check(&input);
        assert_eq!(out.mode, Section42Mode::CompliantQualifiedBasisComputed);
        assert_eq!(out.computed_credit_dollars, 10_000_000);
    }

    #[test]
    fn applicable_fraction_is_smaller_of_unit_or_floor_space() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ApplicableFractionUnderSection42C1B;
        input.low_income_units = 50;
        input.total_units = 100;
        input.low_income_floor_space_sq_ft = 40_000;
        input.total_floor_space_sq_ft = 100_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::CompliantApplicableFractionIsSmallerOfUnitOrFloorSpace
        );
    }

    #[test]
    fn rent_restriction_at_or_below_30_percent_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentRestrictionUnderSection42G2;
        input.gross_rent_dollars_per_month = 1_000;
        input.qualifying_income_dollars_per_month = 4_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::CompliantRentRestrictionAtOrBelow30PercentOfQualifyingIncome
        );
    }

    #[test]
    fn rent_restriction_at_exact_30_percent_boundary_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentRestrictionUnderSection42G2;
        input.gross_rent_dollars_per_month = 1_200;
        input.qualifying_income_dollars_per_month = 4_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::CompliantRentRestrictionAtOrBelow30PercentOfQualifyingIncome
        );
    }

    #[test]
    fn rent_restriction_exceeded_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentRestrictionUnderSection42G2;
        input.gross_rent_dollars_per_month = 1_201;
        input.qualifying_income_dollars_per_month = 4_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::ViolationRentRestrictionExceeded30PercentOfQualifyingIncome
        );
    }

    #[test]
    fn no_recapture_triggered_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RecaptureUnderSection42J;
        input.decrease_in_qualified_basis = false;
        let out = check(&input);
        assert_eq!(out.mode, Section42Mode::CompliantNoRecaptureTriggered);
    }

    #[test]
    fn decrease_in_qualified_basis_triggers_recapture() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RecaptureUnderSection42J;
        input.decrease_in_qualified_basis = true;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::ViolationDecreaseInQualifiedBasisTriggersRecapture
        );
    }

    #[test]
    fn form_8609_filed_correctly_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm8609;
        input.form_8609_filed_correctly = true;
        let out = check(&input);
        assert_eq!(out.mode, Section42Mode::CompliantForm8609FiledCorrectly);
    }

    #[test]
    fn form_8609_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm8609;
        input.form_8609_filed_correctly = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section42Mode::ViolationForm8609NotFiledOrIncorrect
        );
    }

    #[test]
    fn constants_pin_section_42_lihtc_structure() {
        assert_eq!(IRC_42_TAX_REFORM_ACT_ENACTMENT_DATE_YEAR, 1986);
        assert_eq!(IRC_42_TAX_REFORM_ACT_PUBLIC_LAW_CONGRESS, 99);
        assert_eq!(IRC_42_TAX_REFORM_ACT_PUBLIC_LAW_ENACTMENT, 514);
        assert_eq!(IRC_42_AVERAGE_INCOME_TEST_FINAL_REGS_YEAR, 2022);
        assert_eq!(IRC_42_NINE_PERCENT_RATE_BPS, 900);
        assert_eq!(IRC_42_FOUR_PERCENT_RATE_BPS, 400);
        assert_eq!(IRC_42_CREDIT_PERIOD_YEARS, 10);
        assert_eq!(IRC_42_COMPLIANCE_PERIOD_YEARS, 15);
        assert_eq!(IRC_42_EXTENDED_USE_PERIOD_YEARS, 15);
        assert_eq!(IRC_42_TOTAL_AFFORDABILITY_PERIOD_YEARS, 30);
        assert_eq!(IRC_42_SET_ASIDE_20_50_UNIT_PCT_BPS, 2_000);
        assert_eq!(IRC_42_SET_ASIDE_20_50_AMI_PCT_BPS, 5_000);
        assert_eq!(IRC_42_SET_ASIDE_40_60_UNIT_PCT_BPS, 4_000);
        assert_eq!(IRC_42_SET_ASIDE_40_60_AMI_PCT_BPS, 6_000);
        assert_eq!(IRC_42_SET_ASIDE_AVERAGE_INCOME_UNIT_PCT_BPS, 4_000);
        assert_eq!(IRC_42_SET_ASIDE_AVERAGE_INCOME_AVERAGE_AMI_PCT_BPS, 6_000);
        assert_eq!(
            IRC_42_RENT_RESTRICTION_GROSS_RENT_OF_QUALIFYING_INCOME_BPS,
            3_000
        );
        assert_eq!(IRC_42_AVERAGE_INCOME_DESIGNATION_MIN_AMI_PCT_BPS, 2_000);
        assert_eq!(IRC_42_AVERAGE_INCOME_DESIGNATION_MAX_AMI_PCT_BPS, 8_000);
        assert_eq!(IRC_42_BOND_FINANCED_MIN_PCT_BPS, 5_000);
        assert_eq!(IRC_42_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_42_FORM_NUMBER, 8609);
    }

    #[test]
    fn citations_pin_lihtc_statutory_structure() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("IRC § 42 Low-Income Housing Tax Credit"));
        assert!(joined.contains("Tax Reform Act of 1986"));
        assert!(joined.contains("Public Law 99-514"));
        assert!(joined.contains("October 22, 1986"));
        assert!(joined.contains("9 PERCENT LIHTC"));
        assert!(joined.contains("4 PERCENT LIHTC"));
        assert!(joined.contains("TAX-EXEMPT HOUSING BONDS"));
        assert!(joined.contains("PER-CAPITA STATE CEILING"));
        assert!(joined.contains("10-YEAR SPAN"));
        assert!(joined.contains("PLACED IN SERVICE"));
        assert!(joined.contains("15-YEAR COMPLIANCE PERIOD"));
        assert!(joined.contains("CREDIT RECAPTURE"));
        assert!(joined.contains("EXTENDED USE AGREEMENT"));
        assert!(joined.contains("15 YEARS"));
        assert!(joined.contains("20/50 TEST"));
        assert!(joined.contains("40/60 TEST"));
        assert!(joined.contains("AVERAGE INCOME TEST"));
        assert!(joined.contains("Consolidated Appropriations Act of 2018"));
        assert!(joined.contains("Area Median Income (AMI)"));
        assert!(joined.contains("60 % of AMI"));
        assert!(joined.contains("ELIGIBLE BASIS"));
        assert!(joined.contains("MULTIPLIED BY THE APPLICABLE FRACTION"));
        assert!(joined.contains("UNIT FRACTION"));
        assert!(joined.contains("FLOOR SPACE FRACTION"));
        assert!(joined.contains("NOT EXCEED 30 %"));
        assert!(joined.contains("HUD published income limits"));
        assert!(joined.contains("RECAPTURE"));
        assert!(joined.contains("PROPORTIONALLY REDUCED"));
        assert!(joined.contains("Form 8609"));
        assert!(joined.contains("Form 8609-A"));
        assert!(joined.contains("October 12, 2022"));
    }

    #[test]
    fn saturating_overflow_defense_at_u64_max_eligible_basis() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditRateTwoTierStructureUnderSection42B;
        input.eligible_basis_dollars = u64::MAX;
        let out = check(&input);
        let _ = out.computed_credit_dollars;
    }
}
