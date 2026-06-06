//! IRC § 45U — Zero-Emission Nuclear Power
//! Production Credit Compliance Module.
//!
//! Pure-compute check for IRA 2022 § 13105 existing-
//! nuclear-facility production tax credit framework.
//!
//! **Inflation Reduction Act of 2022 enactment**: Section
//! 45U was implemented through **§ 13105 of Public Law
//! 117-169** (136 Stat. 1818), commonly known as the
//! Inflation Reduction Act of 2022 (IRA), **enacted
//! August 16, 2022**; credit available for electricity
//! produced and sold in tax years beginning after
//! **December 31, 2023** and before **January 1, 2033**.
//!
//! **Distinctive § 45U features**: **EXISTING-FACILITY
//! ONLY** — qualifying facilities must have **BEGUN
//! SUPPLYING ELECTRICITY TO CUSTOMERS BEFORE AUGUST 16,
//! 2022** (IRA enactment date); cannot have previously
//! received **§ 45J** credit (anti-double-dip); **0.3
//! CENTS PER KWH** base rate (inflation-adjusted after
//! 2024); **1.5 CENTS PER KWH** PWA-bumped rate (5x bump-
//! up multiplier identical to other IRA 2022 clean-energy
//! credits); **GROSS RECEIPTS REDUCTION FORMULA** under
//! § 45U(b) — credit = LESSER OF (a) 0.3¢ × kWh OR
//! (b) 16% × (gross receipts − 2.5¢ × kWh) when gross
//! receipts exceed 2.5¢ × kWh threshold (phase-out
//! mechanism for nuclear electricity sold above
//! economic-floor price); **ELIGIBLE for § 6417 direct
//! pay** (one of 12 § 6417(b) categories) AND **ELIGIBLE
//! for § 6418 transferability** (one of 11 § 6418(f)(1)
//! categories) — symmetric monetization framework;
//! **FORM 7213** required for § 45U claims.
//!
//! Web research (verified 2026-06-04):
//! - **Inflation Reduction Act of 2022 § 13105 enactment**: § 13105 of Public Law 117-169, commonly known as the Inflation Reduction Act of 2022 (IRA), enacted August 16, 2022 ([Cornell LII — 26 U.S. Code § 45U](https://www.law.cornell.edu/uscode/text/26/45U); [Bloomberg Tax — Sec. 45U Zero-Emission Nuclear Power Production Credit](https://irc.bloombergtax.com/public/uscode/doc/irc/section_45u); [House.gov — 26 USC 45U Zero-Emission Nuclear Power Production Credit](https://uscode.house.gov/view.xhtml?req=(title:26+section:45U+edition:prelim)); [IRS — Zero-Emission Nuclear Power Production Credit](https://www.irs.gov/credits-deductions/zero-emission-nuclear-power-production-credit); [IRS — Instructions for Form 7213 (12/2025)](https://www.irs.gov/instructions/i7213); [IRA Tracker — IRA Section 13105 Zero-Emission Nuclear Power Production Tax Credit](https://iratracker.org/programs/ira-section-13105-zero-emission-nuclear-power-production-tax-credit/); [Foss & Co — 45J and 45U Tax Credits](https://fossandco.com/tax-credits/45j-and-45u-tax-credits/); [Congress.gov CRS — Nuclear Power Tax Credits IN12557](https://www.congress.gov/crs-product/IN12557); [Morgan Lewis — Inflation Reduction Act of 2022 Boosts Nuclear Power with Tax Credits and Funding](https://www.morganlewis.com/pubs/2022/08/inflation-reduction-act-of-2022-boosts-nuclear-power-with-tax-credits-and-funding); [Taxpayers for Common Sense — Section 45U Zero-Emission Nuclear Power Production Credit](https://www.taxpayer.net/energy-natural-resources/section-45u-zero-emission-nuclear-power-production-credit/); [Crux Climate — Understanding the § 45U Tax Credit for Existing Nuclear Power Plants](https://www.cruxclimate.com/insights/understanding-the-45u-tax-credit-for-existing-nuclear-power-plants)).
//! - **§ 45U(a) Base Credit Amount — 0.3 cents/kWh**: the base amount of the Zero-Emission Nuclear Power Production Credit is **0.3 CENTS/KWH**, inflation adjusted after 2024.
//! - **§ 45U(a) PWA Bumped Rate — 1.5 cents/kWh (5x Bump)**: the credit amount may be **INCREASED BY A MULTIPLE OF 5** (to **1.5 CENTS PER KILOWATT HOUR** of electricity produced) if certain **WAGE REQUIREMENTS** are met; identical 5x bump-up multiplier structure used in § 45Y, § 45V, § 45Z, and other IRA 2022 clean-energy production credits.
//! - **§ 45U(b) Gross Receipts Reduction Formula**: the credit is the **LESSER OF** (a) the product of **0.3 CENTS MULTIPLIED BY THE KILOWATT HOURS** of electricity produced by the taxpayer at the qualified nuclear power facility and sold by the taxpayer to an unrelated person, OR (b) the amount equal to **16% OF THE EXCESS OF GROSS RECEIPTS** over the product of **2.5 CENTS** (adjusted for inflation) multiplied by the kilowatt hours of electricity produced by the taxpayer at a qualified nuclear power facility and sold to an unrelated person — phase-out mechanism that reduces or eliminates the credit when nuclear electricity sells above an economic-floor price.
//! - **§ 45U(c) Qualified Nuclear Power Facility — Existing Facility Requirement**: facilities qualifying for the 45U credit must have **BEGUN SUPPLYING ELECTRICITY TO CUSTOMERS BEFORE AUGUST 16, 2022** (IRA enactment date) and **CANNOT HAVE PREVIOUSLY RECEIVED A SECTION 45J TAX CREDIT** (anti-double-dip; § 45J is the predecessor advanced nuclear power facility production tax credit for newer reactors).
//! - **§ 45U(d) Effective Period — January 1, 2024 to January 1, 2033**: the credit is available for **ELECTRICITY GENERATED AND SOLD AFTER DECEMBER 31, 2023**; **EXPIRES ON DECEMBER 31, 2032** — same 10-year credit window as other IRA 2022 clean-energy PTC framework.
//! - **§ 45U Eligible for § 6417 Direct Pay**: § 45U is one of the **12 APPLICABLE CREDIT CATEGORIES** under § 6417(b) for direct pay election by applicable entities (tax-exempt organizations + states + TVA + Indian tribal governments + Alaska Native Corporations + rural electric cooperatives).
//! - **§ 45U Eligible for § 6418 Transferability**: § 45U is one of the **11 ELIGIBLE CREDIT CATEGORIES** under § 6418(f)(1) for transferability monetization by taxable taxpayers — § 45U has **SYMMETRIC MONETIZATION FRAMEWORK** (eligible for both § 6417 direct pay AND § 6418 transferability — unlike § 48C which is asymmetric).
//! - **Form 7213**: required for § 45U claims; published by IRS for the 2025 tax year.
//! - **Sold-to-Unrelated-Person Requirement**: § 45U credit only applies to electricity **SOLD BY THE TAXPAYER TO AN UNRELATED PERSON**; self-consumed electricity does not qualify; intra-affiliate sales do not qualify.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_45U_NUMBER: u32 = 45;
pub const SECTION_45U_BASE_RATE_CENTS_PER_KWH_X100: u32 = 30;
pub const SECTION_45U_PWA_RATE_CENTS_PER_KWH_X100: u32 = 150;
pub const SECTION_45U_PWA_MULTIPLIER: u32 = 5;
pub const SECTION_45U_GROSS_RECEIPTS_PHASEOUT_PERCENT: u32 = 16;
pub const SECTION_45U_GROSS_RECEIPTS_PHASEOUT_THRESHOLD_CENTS_PER_KWH_X100: u32 = 250;
pub const SECTION_45U_EFFECTIVE_START_YEAR: u32 = 2024;
pub const SECTION_45U_EFFECTIVE_END_YEAR: u32 = 2032;
pub const SECTION_45U_EXISTING_FACILITY_CUTOFF_YEAR: u32 = 2022;
pub const SECTION_45U_EXISTING_FACILITY_CUTOFF_MONTH: u32 = 8;
pub const SECTION_45U_EXISTING_FACILITY_CUTOFF_DAY: u32 = 16;
pub const SECTION_45U_IRA_2022_ENACTMENT_YEAR: u32 = 2022;
pub const SECTION_45U_IRA_2022_PUBLIC_LAW_NUMBER: u32 = 117169;
pub const SECTION_45U_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FacilityEligibilityStatus {
    QualifiedNuclearPowerFacilityBeganSupplyingElectricityBeforeAugust16_2022,
    NewerFacilityBeganSupplyingElectricityOnOrAfterAugust16_2022,
    FacilityPreviouslyReceivedSection45JCredit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PwaStatus {
    SatisfiesPwaRequirements,
    DoesNotSatisfyPwaRequirements,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxYearStatus {
    TaxYearBetween2024And2032Inclusive,
    TaxYearBefore2024,
    TaxYearOnOrAfterJanuary1_2033,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SaleToUnrelatedPersonStatus {
    SoldToUnrelatedPerson,
    SelfConsumedOrSoldToRelatedParty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GrossReceiptsPhaseoutStatus {
    GrossReceiptsBelowOrAtPhaseoutThreshold,
    GrossReceiptsExceedPhaseoutThreshold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    QualifiedNuclearPowerFacilityEligibilityUnderSection45UC,
    BaseCreditRateUnderSection45UA,
    PwaBumpedCreditRateUnderSection45UA,
    GrossReceiptsPhaseoutFormulaUnderSection45UB,
    SoldToUnrelatedPersonRequirement,
    EffectivePeriod2024To2032UnderSection45UD,
    EligibilityForSection6417DirectPay,
    EligibilityForSection6418Transferability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section45uMode {
    NotApplicableNewerFacilityBeganSupplyingElectricityOnOrAfterAugust16_2022,
    NotApplicableFacilityPreviouslyReceivedSection45JCredit,
    NotApplicableSelfConsumedOrSoldToRelatedParty,
    NotApplicableTaxYearBefore2024,
    NotApplicableTaxYearOnOrAfterJanuary1_2033,
    CompliantQualifiedNuclearPowerFacility,
    CompliantBaseCreditRateAt03CentsPerKwh,
    CompliantPwaBumpedCreditRateAt15CentsPerKwh,
    CompliantGrossReceiptsBelowPhaseoutThresholdFullCredit,
    CompliantGrossReceiptsExceedPhaseoutThresholdReducedCredit,
    CompliantSoldToUnrelatedPerson,
    CompliantEffectivePeriodBetween2024And2032,
    CompliantEligibleForSection6417DirectPay,
    CompliantEligibleForSection6418TransferabilityWithSection6417,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub facility_eligibility_status: FacilityEligibilityStatus,
    pub pwa_status: PwaStatus,
    pub tax_year_status: TaxYearStatus,
    pub sale_to_unrelated_person_status: SaleToUnrelatedPersonStatus,
    pub gross_receipts_phaseout_status: GrossReceiptsPhaseoutStatus,
    pub compliance_aspect: ComplianceAspect,
    pub kwh_produced_and_sold: u64,
    pub gross_receipts_dollars_x100: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section45uMode,
    pub statutory_basis: String,
    pub notes: String,
    pub credit_amount_dollars_x100: u64,
    pub citations: Vec<String>,
}

pub type Section45uInput = Input;
pub type Section45uOutput = Output;
pub type Section45uResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Inflation Reduction Act of 2022 § 13105 enactment — § 13105 of Public Law 117-169 (136 Stat. 1818), commonly known as the Inflation Reduction Act of 2022 (IRA), enacted August 16, 2022".to_string(),
        "IRC § 45U(a) Base Credit Amount — 0.3 cents/kWh — the base amount of the Zero-Emission Nuclear Power Production Credit is 0.3 CENTS/KWH, inflation adjusted after 2024".to_string(),
        "IRC § 45U(a) PWA Bumped Rate — 1.5 cents/kWh (5x Bump) — the credit amount may be INCREASED BY A MULTIPLE OF 5 (to 1.5 CENTS PER KILOWATT HOUR) if WAGE REQUIREMENTS are met".to_string(),
        "IRC § 45U(b) Gross Receipts Reduction Formula — the credit is the LESSER OF (a) 0.3 CENTS MULTIPLIED BY THE KILOWATT HOURS of electricity produced and sold to an unrelated person, OR (b) the amount equal to 16% OF THE EXCESS OF GROSS RECEIPTS over the product of 2.5 CENTS multiplied by the kilowatt hours produced and sold — phase-out mechanism".to_string(),
        "IRC § 45U(c) Qualified Nuclear Power Facility — facilities must have BEGUN SUPPLYING ELECTRICITY TO CUSTOMERS BEFORE AUGUST 16, 2022 (IRA enactment date) and CANNOT HAVE PREVIOUSLY RECEIVED A SECTION 45J TAX CREDIT (anti-double-dip; § 45J is the predecessor advanced nuclear power facility production tax credit for newer reactors)".to_string(),
        "IRC § 45U(d) Effective Period — January 1, 2024 to January 1, 2033 — the credit is available for ELECTRICITY GENERATED AND SOLD AFTER DECEMBER 31, 2023; EXPIRES ON DECEMBER 31, 2032".to_string(),
        "§ 45U Eligible for § 6417 Direct Pay — § 45U is one of the 12 APPLICABLE CREDIT CATEGORIES under § 6417(b) for direct pay election by applicable entities".to_string(),
        "§ 45U Eligible for § 6418 Transferability — § 45U is one of the 11 ELIGIBLE CREDIT CATEGORIES under § 6418(f)(1) for transferability monetization by taxable taxpayers; SYMMETRIC MONETIZATION FRAMEWORK (eligible for both § 6417 direct pay AND § 6418 transferability — unlike § 48C which is asymmetric)".to_string(),
        "Form 7213 — required for § 45U claims; published by IRS for the 2025 tax year".to_string(),
        "Sold-to-Unrelated-Person Requirement — § 45U credit only applies to electricity SOLD BY THE TAXPAYER TO AN UNRELATED PERSON; self-consumed electricity does not qualify; intra-affiliate sales do not qualify".to_string(),
        "Cornell LII + Bloomberg Tax + House.gov + IRS + IRA Tracker + Foss & Co + Congress.gov CRS + Morgan Lewis + Taxpayers for Common Sense + Crux Climate — practitioner overviews of IRC § 45U Zero-Emission Nuclear Power Production Credit".to_string(),
    ];

    if input.facility_eligibility_status
        == FacilityEligibilityStatus::NewerFacilityBeganSupplyingElectricityOnOrAfterAugust16_2022
    {
        return Output {
            mode: Section45uMode::NotApplicableNewerFacilityBeganSupplyingElectricityOnOrAfterAugust16_2022,
            statutory_basis: "IRC § 45U(c) — newer facility not eligible".to_string(),
            notes: "NOT APPLICABLE: facility began supplying electricity on or after August 16, 2022 (IRA enactment date); not a qualified nuclear power facility under § 45U(c); newer reactors may be eligible under § 45J or § 45Y instead.".to_string(),
            credit_amount_dollars_x100: 0,
            citations,
        };
    }

    if input.facility_eligibility_status
        == FacilityEligibilityStatus::FacilityPreviouslyReceivedSection45JCredit
    {
        return Output {
            mode: Section45uMode::NotApplicableFacilityPreviouslyReceivedSection45JCredit,
            statutory_basis: "IRC § 45U(c) — facility previously received § 45J credit (anti-double-dip)".to_string(),
            notes: "NOT APPLICABLE: facility previously received a § 45J tax credit; cannot also claim § 45U under anti-double-dip rule.".to_string(),
            credit_amount_dollars_x100: 0,
            citations,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::QualifiedNuclearPowerFacilityEligibilityUnderSection45UC => Output {
            mode: Section45uMode::CompliantQualifiedNuclearPowerFacility,
            statutory_basis: "IRC § 45U(c) — qualified nuclear power facility".to_string(),
            notes: "COMPLIANT: facility began supplying electricity to customers BEFORE AUGUST 16, 2022 and has NOT previously received § 45J credit; qualifies as QUALIFIED NUCLEAR POWER FACILITY under § 45U(c).".to_string(),
            credit_amount_dollars_x100: 0,
            citations,
        },
        ComplianceAspect::BaseCreditRateUnderSection45UA => {
            let credit_amount_dollars_x100 = (u128::from(input.kwh_produced_and_sold)
                * u128::from(SECTION_45U_BASE_RATE_CENTS_PER_KWH_X100)
                / 100)
                .min(u128::from(u64::MAX)) as u64;
            Output {
                mode: Section45uMode::CompliantBaseCreditRateAt03CentsPerKwh,
                statutory_basis: "IRC § 45U(a) — 0.3 cents/kWh base rate".to_string(),
                notes: format!(
                    "COMPLIANT: 0.3 cents/kWh base rate under § 45U(a); {kwh} kWh × 0.3¢/kWh = ${ca}.{cb:02} credit (before phase-out and PWA bump).",
                    kwh = input.kwh_produced_and_sold,
                    ca = credit_amount_dollars_x100 / 100,
                    cb = credit_amount_dollars_x100 % 100,
                ),
                credit_amount_dollars_x100,
                citations,
            }
        }
        ComplianceAspect::PwaBumpedCreditRateUnderSection45UA => {
            let rate_x100 = match input.pwa_status {
                PwaStatus::SatisfiesPwaRequirements => SECTION_45U_PWA_RATE_CENTS_PER_KWH_X100,
                PwaStatus::DoesNotSatisfyPwaRequirements => {
                    SECTION_45U_BASE_RATE_CENTS_PER_KWH_X100
                }
            };
            let credit_amount_dollars_x100 = (u128::from(input.kwh_produced_and_sold)
                * u128::from(rate_x100)
                / 100)
                .min(u128::from(u64::MAX)) as u64;
            match input.pwa_status {
                PwaStatus::SatisfiesPwaRequirements => Output {
                    mode: Section45uMode::CompliantPwaBumpedCreditRateAt15CentsPerKwh,
                    statutory_basis: "IRC § 45U(a) — 1.5 cents/kWh PWA-bumped rate (5x bump)".to_string(),
                    notes: format!(
                        "COMPLIANT: 1.5 cents/kWh PWA-bumped rate under § 45U(a); {kwh} kWh × 1.5¢/kWh = ${ca}.{cb:02} credit (5x base 0.3¢/kWh).",
                        kwh = input.kwh_produced_and_sold,
                        ca = credit_amount_dollars_x100 / 100,
                        cb = credit_amount_dollars_x100 % 100,
                    ),
                    credit_amount_dollars_x100,
                    citations,
                },
                PwaStatus::DoesNotSatisfyPwaRequirements => Output {
                    mode: Section45uMode::CompliantBaseCreditRateAt03CentsPerKwh,
                    statutory_basis: "IRC § 45U(a) — 0.3 cents/kWh base rate (PWA not satisfied)".to_string(),
                    notes: format!(
                        "COMPLIANT: 0.3 cents/kWh base rate under § 45U(a); {kwh} kWh × 0.3¢/kWh = ${ca}.{cb:02} credit; PWA bump-up unavailable.",
                        kwh = input.kwh_produced_and_sold,
                        ca = credit_amount_dollars_x100 / 100,
                        cb = credit_amount_dollars_x100 % 100,
                    ),
                    credit_amount_dollars_x100,
                    citations,
                },
            }
        }
        ComplianceAspect::GrossReceiptsPhaseoutFormulaUnderSection45UB => {
            let base_credit_dollars_x100 = (u128::from(input.kwh_produced_and_sold)
                * u128::from(SECTION_45U_BASE_RATE_CENTS_PER_KWH_X100)
                / 100)
                .min(u128::from(u64::MAX));
            let threshold_dollars_x100 = (u128::from(input.kwh_produced_and_sold)
                * u128::from(SECTION_45U_GROSS_RECEIPTS_PHASEOUT_THRESHOLD_CENTS_PER_KWH_X100)
                / 100)
                .min(u128::from(u64::MAX));
            let gross_receipts_x100 = u128::from(input.gross_receipts_dollars_x100);

            if gross_receipts_x100 <= threshold_dollars_x100 {
                let credit_amount_dollars_x100 = base_credit_dollars_x100 as u64;
                return Output {
                    mode: Section45uMode::CompliantGrossReceiptsBelowPhaseoutThresholdFullCredit,
                    statutory_basis: "IRC § 45U(b) — gross receipts at or below 2.5¢ × kWh phase-out threshold; full credit".to_string(),
                    notes: format!(
                        "COMPLIANT: gross receipts ${gr_a}.{gr_b:02} at or below 2.5¢ × {kwh} kWh phase-out threshold (${th_a}.{th_b:02}); full credit = ${ca}.{cb:02} under § 45U(b).",
                        gr_a = input.gross_receipts_dollars_x100 / 100,
                        gr_b = input.gross_receipts_dollars_x100 % 100,
                        kwh = input.kwh_produced_and_sold,
                        th_a = threshold_dollars_x100 / 100,
                        th_b = threshold_dollars_x100 % 100,
                        ca = credit_amount_dollars_x100 / 100,
                        cb = credit_amount_dollars_x100 % 100,
                    ),
                    credit_amount_dollars_x100,
                    citations,
                };
            }
            let excess_x100 = gross_receipts_x100 - threshold_dollars_x100;
            let reduction_cap_dollars_x100 = (excess_x100
                * u128::from(SECTION_45U_GROSS_RECEIPTS_PHASEOUT_PERCENT)
                / 100)
                .min(u128::from(u64::MAX));
            let credit_amount_dollars_x100 =
                base_credit_dollars_x100.min(reduction_cap_dollars_x100) as u64;
            Output {
                mode: Section45uMode::CompliantGrossReceiptsExceedPhaseoutThresholdReducedCredit,
                statutory_basis: "IRC § 45U(b) — gross receipts exceed 2.5¢ × kWh phase-out threshold; reduced credit = lesser of base or 16% of excess".to_string(),
                notes: format!(
                    "COMPLIANT: gross receipts ${gr_a}.{gr_b:02} exceed 2.5¢ × {kwh} kWh phase-out threshold (${th_a}.{th_b:02}); excess = ${ex_a}.{ex_b:02}; reduction cap (16% × excess) = ${rc_a}.{rc_b:02}; reduced credit = lesser of base (${bc_a}.{bc_b:02}) or reduction cap = ${ca}.{cb:02} under § 45U(b).",
                    gr_a = input.gross_receipts_dollars_x100 / 100,
                    gr_b = input.gross_receipts_dollars_x100 % 100,
                    kwh = input.kwh_produced_and_sold,
                    th_a = threshold_dollars_x100 / 100,
                    th_b = threshold_dollars_x100 % 100,
                    ex_a = excess_x100 / 100,
                    ex_b = excess_x100 % 100,
                    rc_a = reduction_cap_dollars_x100 / 100,
                    rc_b = reduction_cap_dollars_x100 % 100,
                    bc_a = base_credit_dollars_x100 / 100,
                    bc_b = base_credit_dollars_x100 % 100,
                    ca = credit_amount_dollars_x100 / 100,
                    cb = credit_amount_dollars_x100 % 100,
                ),
                credit_amount_dollars_x100,
                citations,
            }
        }
        ComplianceAspect::SoldToUnrelatedPersonRequirement => match input.sale_to_unrelated_person_status {
            SaleToUnrelatedPersonStatus::SoldToUnrelatedPerson => Output {
                mode: Section45uMode::CompliantSoldToUnrelatedPerson,
                statutory_basis: "IRC § 45U — electricity sold to unrelated person".to_string(),
                notes: "COMPLIANT: electricity sold by taxpayer to unrelated person under § 45U; self-consumption and intra-affiliate sales do not qualify.".to_string(),
                credit_amount_dollars_x100: 0,
                citations,
            },
            SaleToUnrelatedPersonStatus::SelfConsumedOrSoldToRelatedParty => Output {
                mode: Section45uMode::NotApplicableSelfConsumedOrSoldToRelatedParty,
                statutory_basis: "IRC § 45U — electricity self-consumed or sold to related party (ineligible)".to_string(),
                notes: "NOT APPLICABLE: electricity self-consumed or sold to related party under § 45U; credit only applies to electricity SOLD BY TAXPAYER TO UNRELATED PERSON.".to_string(),
                credit_amount_dollars_x100: 0,
                citations,
            },
        },
        ComplianceAspect::EffectivePeriod2024To2032UnderSection45UD => {
            match input.tax_year_status {
                TaxYearStatus::TaxYearBetween2024And2032Inclusive => Output {
                    mode: Section45uMode::CompliantEffectivePeriodBetween2024And2032,
                    statutory_basis: "IRC § 45U(d) — tax year between 2024 and 2032 inclusive".to_string(),
                    notes: "COMPLIANT: tax year within effective period (after December 31, 2023 and before January 1, 2033) under § 45U(d).".to_string(),
                    credit_amount_dollars_x100: 0,
                    citations,
                },
                TaxYearStatus::TaxYearBefore2024 => Output {
                    mode: Section45uMode::NotApplicableTaxYearBefore2024,
                    statutory_basis: "IRC § 45U(d) — tax year before 2024 (pre-effective)".to_string(),
                    notes: "NOT APPLICABLE: tax year before January 1, 2024 (pre-effective) under § 45U(d); credit not yet available.".to_string(),
                    credit_amount_dollars_x100: 0,
                    citations,
                },
                TaxYearStatus::TaxYearOnOrAfterJanuary1_2033 => Output {
                    mode: Section45uMode::NotApplicableTaxYearOnOrAfterJanuary1_2033,
                    statutory_basis: "IRC § 45U(d) — tax year on or after January 1, 2033 (post-sunset)".to_string(),
                    notes: "NOT APPLICABLE: tax year on or after January 1, 2033 (post-sunset) under § 45U(d); credit expired.".to_string(),
                    credit_amount_dollars_x100: 0,
                    citations,
                },
            }
        }
        ComplianceAspect::EligibilityForSection6417DirectPay => Output {
            mode: Section45uMode::CompliantEligibleForSection6417DirectPay,
            statutory_basis: "IRC § 6417(b) — § 45U eligible for direct pay election".to_string(),
            notes: "COMPLIANT: § 45U is one of the 12 APPLICABLE CREDIT CATEGORIES under § 6417(b) for direct pay election by applicable entities.".to_string(),
            credit_amount_dollars_x100: 0,
            citations,
        },
        ComplianceAspect::EligibilityForSection6418Transferability => Output {
            mode: Section45uMode::CompliantEligibleForSection6418TransferabilityWithSection6417,
            statutory_basis: "IRC § 6418(f)(1) — § 45U eligible for transferability + § 6417 direct pay (symmetric monetization)".to_string(),
            notes: "COMPLIANT: § 45U is one of the 11 ELIGIBLE CREDIT CATEGORIES under § 6418(f)(1) for transferability monetization; SYMMETRIC MONETIZATION FRAMEWORK (eligible for both § 6417 direct pay AND § 6418 transferability — unlike § 48C which is asymmetric).".to_string(),
            credit_amount_dollars_x100: 0,
            citations,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            facility_eligibility_status:
                FacilityEligibilityStatus::QualifiedNuclearPowerFacilityBeganSupplyingElectricityBeforeAugust16_2022,
            pwa_status: PwaStatus::SatisfiesPwaRequirements,
            tax_year_status: TaxYearStatus::TaxYearBetween2024And2032Inclusive,
            sale_to_unrelated_person_status: SaleToUnrelatedPersonStatus::SoldToUnrelatedPerson,
            gross_receipts_phaseout_status: GrossReceiptsPhaseoutStatus::GrossReceiptsBelowOrAtPhaseoutThreshold,
            compliance_aspect: ComplianceAspect::QualifiedNuclearPowerFacilityEligibilityUnderSection45UC,
            kwh_produced_and_sold: 1_000_000_000,
            gross_receipts_dollars_x100: 2_000_000_000,
        }
    }

    #[test]
    fn newer_facility_not_applicable() {
        let mut input = baseline_input();
        input.facility_eligibility_status =
            FacilityEligibilityStatus::NewerFacilityBeganSupplyingElectricityOnOrAfterAugust16_2022;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::NotApplicableNewerFacilityBeganSupplyingElectricityOnOrAfterAugust16_2022
        );
    }

    #[test]
    fn previously_received_45j_credit_not_applicable() {
        let mut input = baseline_input();
        input.facility_eligibility_status =
            FacilityEligibilityStatus::FacilityPreviouslyReceivedSection45JCredit;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::NotApplicableFacilityPreviouslyReceivedSection45JCredit
        );
    }

    #[test]
    fn qualified_nuclear_power_facility_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::QualifiedNuclearPowerFacilityEligibilityUnderSection45UC;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::CompliantQualifiedNuclearPowerFacility
        );
    }

    #[test]
    fn base_credit_rate_arithmetic() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditRateUnderSection45UA;
        input.kwh_produced_and_sold = 1_000_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::CompliantBaseCreditRateAt03CentsPerKwh
        );
        assert_eq!(out.credit_amount_dollars_x100, 300_000_000);
    }

    #[test]
    fn pwa_bumped_rate_arithmetic_5x_multiplier() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::PwaBumpedCreditRateUnderSection45UA;
        input.pwa_status = PwaStatus::SatisfiesPwaRequirements;
        input.kwh_produced_and_sold = 1_000_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::CompliantPwaBumpedCreditRateAt15CentsPerKwh
        );
        assert_eq!(out.credit_amount_dollars_x100, 1_500_000_000);
    }

    #[test]
    fn pwa_5x_multiplier_invariant_verified() {
        let mut input_base = baseline_input();
        input_base.compliance_aspect = ComplianceAspect::PwaBumpedCreditRateUnderSection45UA;
        input_base.pwa_status = PwaStatus::DoesNotSatisfyPwaRequirements;
        input_base.kwh_produced_and_sold = 1_000_000_000;
        let out_base = check(&input_base);

        let mut input_pwa = baseline_input();
        input_pwa.compliance_aspect = ComplianceAspect::PwaBumpedCreditRateUnderSection45UA;
        input_pwa.pwa_status = PwaStatus::SatisfiesPwaRequirements;
        input_pwa.kwh_produced_and_sold = 1_000_000_000;
        let out_pwa = check(&input_pwa);

        assert_eq!(
            out_pwa.credit_amount_dollars_x100 / out_base.credit_amount_dollars_x100,
            5
        );
    }

    #[test]
    fn gross_receipts_below_threshold_full_credit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::GrossReceiptsPhaseoutFormulaUnderSection45UB;
        input.kwh_produced_and_sold = 1_000_000_000;
        input.gross_receipts_dollars_x100 = 2_000_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::CompliantGrossReceiptsBelowPhaseoutThresholdFullCredit
        );
        assert_eq!(out.credit_amount_dollars_x100, 300_000_000);
    }

    #[test]
    fn gross_receipts_at_threshold_full_credit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::GrossReceiptsPhaseoutFormulaUnderSection45UB;
        input.kwh_produced_and_sold = 1_000_000_000;
        input.gross_receipts_dollars_x100 = 2_500_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::CompliantGrossReceiptsBelowPhaseoutThresholdFullCredit
        );
        assert_eq!(out.credit_amount_dollars_x100, 300_000_000);
    }

    #[test]
    fn gross_receipts_exceed_threshold_phase_out_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::GrossReceiptsPhaseoutFormulaUnderSection45UB;
        input.kwh_produced_and_sold = 1_000_000_000;
        input.gross_receipts_dollars_x100 = 5_000_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::CompliantGrossReceiptsExceedPhaseoutThresholdReducedCredit
        );
        assert_eq!(out.credit_amount_dollars_x100, 300_000_000);
    }

    #[test]
    fn gross_receipts_severely_exceed_threshold_reduced_credit_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::GrossReceiptsPhaseoutFormulaUnderSection45UB;
        input.kwh_produced_and_sold = 1_000_000_000;
        input.gross_receipts_dollars_x100 = 3_000_000_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::CompliantGrossReceiptsExceedPhaseoutThresholdReducedCredit
        );
        assert_eq!(out.credit_amount_dollars_x100, 80_000_000);
    }

    #[test]
    fn sold_to_unrelated_person_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SoldToUnrelatedPersonRequirement;
        input.sale_to_unrelated_person_status = SaleToUnrelatedPersonStatus::SoldToUnrelatedPerson;
        let out = check(&input);
        assert_eq!(out.mode, Section45uMode::CompliantSoldToUnrelatedPerson);
    }

    #[test]
    fn self_consumed_not_applicable() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SoldToUnrelatedPersonRequirement;
        input.sale_to_unrelated_person_status =
            SaleToUnrelatedPersonStatus::SelfConsumedOrSoldToRelatedParty;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::NotApplicableSelfConsumedOrSoldToRelatedParty
        );
    }

    #[test]
    fn tax_year_2024_to_2032_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EffectivePeriod2024To2032UnderSection45UD;
        input.tax_year_status = TaxYearStatus::TaxYearBetween2024And2032Inclusive;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::CompliantEffectivePeriodBetween2024And2032
        );
    }

    #[test]
    fn tax_year_before_2024_not_applicable() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EffectivePeriod2024To2032UnderSection45UD;
        input.tax_year_status = TaxYearStatus::TaxYearBefore2024;
        let out = check(&input);
        assert_eq!(out.mode, Section45uMode::NotApplicableTaxYearBefore2024);
    }

    #[test]
    fn tax_year_on_or_after_2033_not_applicable() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EffectivePeriod2024To2032UnderSection45UD;
        input.tax_year_status = TaxYearStatus::TaxYearOnOrAfterJanuary1_2033;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::NotApplicableTaxYearOnOrAfterJanuary1_2033
        );
    }

    #[test]
    fn eligible_for_section_6417_direct_pay_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibilityForSection6417DirectPay;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::CompliantEligibleForSection6417DirectPay
        );
    }

    #[test]
    fn eligible_for_section_6418_transferability_symmetric_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibilityForSection6418Transferability;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::CompliantEligibleForSection6418TransferabilityWithSection6417
        );
    }

    #[test]
    fn overflow_defense_at_u64_max_kwh() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::BaseCreditRateUnderSection45UA;
        input.kwh_produced_and_sold = u64::MAX;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section45uMode::CompliantBaseCreditRateAt03CentsPerKwh
        );
        assert!(out.credit_amount_dollars_x100 > 0);
    }

    #[test]
    fn constants_pin_section_45u_statutory_thresholds() {
        assert_eq!(SECTION_45U_BASE_RATE_CENTS_PER_KWH_X100, 30);
        assert_eq!(SECTION_45U_PWA_RATE_CENTS_PER_KWH_X100, 150);
        assert_eq!(SECTION_45U_PWA_MULTIPLIER, 5);
        assert_eq!(SECTION_45U_GROSS_RECEIPTS_PHASEOUT_PERCENT, 16);
        assert_eq!(
            SECTION_45U_GROSS_RECEIPTS_PHASEOUT_THRESHOLD_CENTS_PER_KWH_X100,
            250
        );
        assert_eq!(SECTION_45U_EFFECTIVE_START_YEAR, 2024);
        assert_eq!(SECTION_45U_EFFECTIVE_END_YEAR, 2032);
        assert_eq!(SECTION_45U_EXISTING_FACILITY_CUTOFF_YEAR, 2022);
        assert_eq!(SECTION_45U_EXISTING_FACILITY_CUTOFF_MONTH, 8);
        assert_eq!(SECTION_45U_EXISTING_FACILITY_CUTOFF_DAY, 16);
        assert_eq!(SECTION_45U_IRA_2022_ENACTMENT_YEAR, 2022);
        assert_eq!(SECTION_45U_IRA_2022_PUBLIC_LAW_NUMBER, 117169);
        assert_eq!(SECTION_45U_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_section_45u_statutory_provisions() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("Inflation Reduction Act of 2022 § 13105"));
        assert!(joined.contains("Public Law 117-169"));
        assert!(joined.contains("August 16, 2022"));
        assert!(joined.contains("IRC § 45U(a)"));
        assert!(joined.contains("0.3 CENTS/KWH"));
        assert!(joined.contains("INCREASED BY A MULTIPLE OF 5"));
        assert!(joined.contains("1.5 CENTS PER KILOWATT HOUR"));
        assert!(joined.contains("IRC § 45U(b)"));
        assert!(joined.contains("LESSER OF"));
        assert!(joined.contains("0.3 CENTS MULTIPLIED BY THE KILOWATT HOURS"));
        assert!(joined.contains("16% OF THE EXCESS OF GROSS RECEIPTS"));
        assert!(joined.contains("2.5 CENTS"));
        assert!(joined.contains("IRC § 45U(c)"));
        assert!(joined.contains("BEGUN SUPPLYING ELECTRICITY TO CUSTOMERS BEFORE AUGUST 16, 2022"));
        assert!(joined.contains("CANNOT HAVE PREVIOUSLY RECEIVED A SECTION 45J TAX CREDIT"));
        assert!(joined.contains("IRC § 45U(d)"));
        assert!(joined.contains("ELECTRICITY GENERATED AND SOLD AFTER DECEMBER 31, 2023"));
        assert!(joined.contains("EXPIRES ON DECEMBER 31, 2032"));
        assert!(joined.contains("§ 6417(b)"));
        assert!(joined.contains("§ 6418(f)(1)"));
        assert!(joined.contains("SYMMETRIC MONETIZATION FRAMEWORK"));
        assert!(joined.contains("Form 7213"));
        assert!(joined.contains("SOLD BY THE TAXPAYER TO AN UNRELATED PERSON"));
    }
}
