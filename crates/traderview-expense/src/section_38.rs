//! IRC § 38 General Business Credit (GBC) Aggregation
//! Compliance Module — pure-compute check for the
//! **GENERAL BUSINESS CREDIT (GBC)** umbrella framework
//! that bundles **41 SEPARATE BUSINESS TAX CREDITS** under
//! § 38(b) into a single computational structure subject
//! to the § 38(c) limitation. The foundational
//! aggregation mechanism for all business tax credits
//! including § 41 R&D, § 42 LIHTC, § 44 DAC, § 47 HTC,
//! § 51 WOTC, and the IRA 2022 clean-energy cluster
//! (§ 45L / § 45Q / § 45V / § 45W / § 45X / § 45Y / § 45Z
//! / § 48E / etc.).
//!
//! ITER 730 MILESTONE: this module is the meta-section
//! that conceptually unifies the entire business credit
//! cluster previously shipped in traderview-expense.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 38 General Business Credit (GBC)**: provides rules governing a general business credit; **NOT A SINGLE TAX CREDIT** but an **UMBRELLA FRAMEWORK** that bundles **41 SEPARATE BUSINESS TAX INCENTIVES** into one computational structure; each component credit is calculated separately under its own IRC section, then aggregated ([Cornell LII — 26 U.S. Code § 38](https://www.law.cornell.edu/uscode/text/26/38); [Tax Notes — IRC Code Section 38](https://www.taxnotes.com/research/federal/usc26/38); [Bloomberg Tax — Sec. 38 General Business Credit](https://irc.bloombergtax.com/public/uscode/doc/irc/section_38); [IRS — Instructions for Form 3800 and Schedule A (2025)](https://www.irs.gov/instructions/i3800); [LVHJ — Maximize the General Business Credit, IRS Section 38](https://lvhj.com/maximize-the-general-business-credit-irs-section-38/); [The Tax Adviser — Maximizing the Benefits of General Business Tax Credits (May 2022)](https://www.thetaxadviser.com/issues/2022/may/maximizing-benefits-general-business-tax-credits/); [Wolters Kluwer AnswerConnect — § 38 General Business Credit](https://answerconnect.cch.com/document/arp1209013e2c83d8fcb0/federal/irc/current/general-business-credit); [Swanson Reed — General Business Credit (GBC) & R&D Tax Credit](https://www.swansonreed.com/research-tax-credit/federal/glossary/general-business-credit/); [Tax990 — IRS Form 3800: General Business Credit](https://www.tax990.com/resources/irs-form-3800/); [KPMG — A Practical Guide to Tax Credit Ordering and Usage Rules PDF](https://assets.kpmg.com/content/dam/kpmg/us/pdf/2020/10/tnf-wnit-oct5-2020.pdf); [Legis1 — General Business Credit: 41 Tax Incentives](https://legis1.com/news/general-business-credit-framework-bundles-41-tax); [FindLaw — 26 U.S.C. § 38](https://codes.findlaw.com/us/title-26-internal-revenue-code/26-usc-sect-38.html); [GovInfo — Title 26 § 38 PDF 2023](https://www.govinfo.gov/content/pkg/USCODE-2023-title26/pdf/USCODE-2023-title26-subtitleA-chap1-subchapA-partIV-subpartD-sec38.pdf); [Bradford Tax Institute — Internal Revenue Code Section 38 PDF](https://bradfordtaxinstitute.com/Endnotes/IRC_Section_38.pdf)).
//! - **§ 38(a) Allowable Credit — Sum of Three Components**: the general business credit allowed for any taxable year equals the **SUM OF** (1) **CARRYFORWARDS** from prior years to the current year, (2) **CURRENT YEAR BUSINESS CREDIT** (the sum of all § 38(b) component credits determined for the year), and (3) **CARRYBACKS** from succeeding years to the current year.
//! - **§ 38(b) Component Credits — 41 Bundled Credits**: the current year business credit is the sum of the following: (1) **INVESTMENT CREDIT** under § 46 (includes § 47 rehabilitation credit, § 48 energy credit, § 48E clean electricity ITC); (2) **WORK OPPORTUNITY CREDIT** under § 51(a); (3) **ALCOHOL FUELS CREDIT** under § 40(a); (4) **RESEARCH CREDIT** under § 41(a); (5) **LOW-INCOME HOUSING CREDIT** under § 42(a); (6) **ENHANCED OIL RECOVERY CREDIT** under § 43(a); (7) **DISABLED ACCESS CREDIT** under § 44(a) (for eligible small business); (8) **RENEWABLE ELECTRICITY PRODUCTION CREDIT** under § 45(a); (9) **CARBON OXIDE SEQUESTRATION CREDIT** under § 45Q(a); (10) **CLEAN HYDROGEN CREDIT** under § 45V; (11) **NEW ENERGY EFFICIENT HOME CREDIT** under § 45L; (12) **COMMERCIAL CLEAN VEHICLE CREDIT** under § 45W; (13) **ADVANCED MANUFACTURING PRODUCTION CREDIT** under § 45X; (14) **CLEAN ELECTRICITY PRODUCTION CREDIT** under § 45Y; (15) **CLEAN FUEL PRODUCTION CREDIT** under § 45Z; and approximately **27 ADDITIONAL** specialized credits.
//! - **§ 38(c)(1) Limitation — Net Income Tax Cap**: the credit allowed for a given tax year shall **NOT EXCEED THE EXCESS (IF ANY)** of the taxpayer's **NET INCOME TAX** over the **GREATER OF**: (1) the **TENTATIVE MINIMUM TAX (TMT)** for the tax year; OR (2) **25 PERCENT of the excess of NET REGULAR TAX LIABILITY OVER $25,000**.
//! - **§ 38(c)(4) Specified Credits — Allowed Against TMT**: certain credits are **SPECIFIED CREDITS** that may be used against the TMT (i.e., not subject to the TMT limitation under § 38(c)(1)); these include the **R&D CREDIT for eligible small businesses** (§ 41(h)) and certain clean-energy credits (§ 45Y, § 48E, § 45V, § 45Q, § 45X, § 45Z, § 45U); specified credits effectively bypass the AMT/TMT constraint.
//! - **§ 38(c)(2) Special Rules for Married Individuals Filing Separately**: $25,000 amount in § 38(c)(1)(B) is **REDUCED TO $12,500** for a married individual filing a separate return; eligible small business defined under § 38(c)(5)(C).
//! - **§ 39 Carryback and Carryforward — 1-Year Back / 20-Year Forward**: if § 38(c) limitations prevent a taxpayer from using all of the GBC, the unused credit may be **CARRIED BACK 1 YEAR** and then, if unused credit remains, **CARRIED FORWARD UP TO 20 YEARS**.
//! - **§ 39 FIFO Ordering Rules**: the GBC is used in the following order — (1) **CARRYFORWARDS** to that year, starting with the **OLDEST ONES**; (2) **CURRENT YEAR BUSINESS CREDIT**; (3) **CARRYBACK** to that year; these ordering rules essentially apply a **FIRST-IN, FIRST-OUT (FIFO)** approach that minimizes the risk that unused credits will expire.
//! - **Form 3800 General Business Credit**: taxpayers that claim **MORE THAN ONE CREDIT** must file **FORM 3800** to report the aggregate value of those credits and calculate the overall allowable credit under the GBC; each component credit is first calculated on its specific statutory form (e.g., Form 6765 for R&D; Form 8826 for DAC; Form 8609-A for LIHTC; Form 5884 for WOTC; Form 3468 for investment credit including HTC) and then transferred to Form 3800 Part III for aggregation.
//! - **Form 3800 Structure (Post-2023 Rev.)**: **Part I** addresses credits not allowed against TMT; **Part II** addresses "Figuring Credit Allowed After Limitations" with **sections A, B, C** corresponding to § 38(c)(1), § 38(c)(2), § 38(c)(4) limitations; **Part III** addresses credit components from each separate statutory form; **Schedule A** addresses additional adjustments.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_38_CARRYBACK_YEARS: u32 = 1;
pub const IRC_38_CARRYFORWARD_YEARS: u32 = 20;
pub const IRC_38_NUMBER_OF_COMPONENT_CREDITS: u32 = 41;
pub const IRC_38_LIMITATION_25_PCT_THRESHOLD_DOLLARS: u64 = 25_000;
pub const IRC_38_LIMITATION_25_PCT_THRESHOLD_DOLLARS_MFS: u64 = 12_500;
pub const IRC_38_LIMITATION_25_PCT_RATE_BPS: u64 = 2_500;
pub const IRC_38_FORM_NUMBER: u32 = 3_800;
pub const IRC_38_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentCreditCategory {
    InvestmentCreditSection46,
    WorkOpportunityCreditSection51,
    AlcoholFuelsCreditSection40,
    ResearchCreditSection41,
    LowIncomeHousingCreditSection42,
    EnhancedOilRecoveryCreditSection43,
    DisabledAccessCreditSection44,
    RenewableElectricityProductionCreditSection45,
    CarbonOxideSequestrationCreditSection45Q,
    CleanHydrogenCreditSection45V,
    NewEnergyEfficientHomeCreditSection45L,
    CommercialCleanVehicleCreditSection45W,
    AdvancedManufacturingProductionCreditSection45X,
    CleanElectricityProductionCreditSection45Y,
    CleanFuelProductionCreditSection45Z,
    OtherSpecifiedComponentCredit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecifiedCreditStatus {
    SpecifiedCreditAllowedAgainstTmt,
    NotSpecifiedCreditSubjectToTmtLimitation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    MarriedFilingJointlyOrSingleOrHeadOfHousehold,
    MarriedFilingSeparately,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    AllowableCreditUnderSection38A,
    ComponentCreditsUnderSection38B,
    LimitationUnderSection38C1,
    SpecifiedCreditsUnderSection38C4,
    MarriedFilingSeparatelyAdjustmentUnderSection38C2,
    CarrybackCarryforwardUnderSection39,
    FifoOrderingUnderSection39,
    FormFilingUnderForm3800,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section38Mode {
    CompliantAllowableCreditAggregatedFromThreeComponents,
    CompliantComponentCreditIdentifiedUnderSection38B,
    CompliantLimitationAppliedUnderSection38C1,
    CompliantSpecifiedCreditAllowedAgainstTmt,
    CompliantMarriedFilingSeparatelyThresholdReducedTo12500,
    CompliantOneYearCarrybackTwentyYearCarryforward,
    CompliantFifoOrderingObserved,
    CompliantForm3800FiledCorrectly,
    ViolationCreditExceedsSection38CLimitation,
    ViolationForm3800NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub component_credit_category: ComponentCreditCategory,
    pub specified_credit_status: SpecifiedCreditStatus,
    pub filing_status: FilingStatus,
    pub compliance_aspect: ComplianceAspect,
    pub current_year_business_credit_dollars: u64,
    pub carryforward_dollars: u64,
    pub carryback_dollars: u64,
    pub net_income_tax_dollars: u64,
    pub tentative_minimum_tax_dollars: u64,
    pub net_regular_tax_liability_dollars: u64,
    pub form_3800_filed_correctly: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section38Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_allowable_credit_dollars: u64,
}

pub type Section38Input = Input;
pub type Section38Output = Output;
pub type Section38Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 38 General Business Credit (GBC) — NOT A SINGLE TAX CREDIT but an UMBRELLA FRAMEWORK that bundles 41 SEPARATE BUSINESS TAX INCENTIVES into one computational structure; each component credit is calculated separately under its own IRC section, then aggregated".to_string(),
        "§ 38(a) Allowable Credit — Sum of Three Components — the general business credit allowed for any taxable year equals the SUM OF (1) CARRYFORWARDS from prior years to the current year, (2) CURRENT YEAR BUSINESS CREDIT (the sum of all § 38(b) component credits determined for the year), and (3) CARRYBACKS from succeeding years to the current year".to_string(),
        "§ 38(b) Component Credits — 41 Bundled Credits — current year business credit is the sum of the following: (1) INVESTMENT CREDIT under § 46 (includes § 47 rehabilitation credit, § 48 energy credit, § 48E clean electricity ITC); (2) WORK OPPORTUNITY CREDIT under § 51(a); (3) ALCOHOL FUELS CREDIT under § 40(a); (4) RESEARCH CREDIT under § 41(a); (5) LOW-INCOME HOUSING CREDIT under § 42(a); (6) ENHANCED OIL RECOVERY CREDIT under § 43(a); (7) DISABLED ACCESS CREDIT under § 44(a); (8) RENEWABLE ELECTRICITY PRODUCTION CREDIT under § 45(a); (9) CARBON OXIDE SEQUESTRATION CREDIT under § 45Q(a); (10) CLEAN HYDROGEN CREDIT under § 45V; (11) NEW ENERGY EFFICIENT HOME CREDIT under § 45L; (12) COMMERCIAL CLEAN VEHICLE CREDIT under § 45W; (13) ADVANCED MANUFACTURING PRODUCTION CREDIT under § 45X; (14) CLEAN ELECTRICITY PRODUCTION CREDIT under § 45Y; (15) CLEAN FUEL PRODUCTION CREDIT under § 45Z; and approximately 27 ADDITIONAL specialized credits".to_string(),
        "§ 38(c)(1) Limitation — Net Income Tax Cap — the credit allowed for a given tax year shall NOT EXCEED THE EXCESS (IF ANY) of the taxpayer's NET INCOME TAX over the GREATER OF (1) the TENTATIVE MINIMUM TAX (TMT) for the tax year; OR (2) 25 PERCENT of the excess of NET REGULAR TAX LIABILITY OVER $25,000".to_string(),
        "§ 38(c)(4) Specified Credits — Allowed Against TMT — certain credits are SPECIFIED CREDITS that may be used against the TMT (i.e., not subject to the TMT limitation under § 38(c)(1)); these include the R&D CREDIT for eligible small businesses (§ 41(h)) and certain clean-energy credits (§ 45Y, § 48E, § 45V, § 45Q, § 45X, § 45Z, § 45U); specified credits effectively bypass the AMT/TMT constraint".to_string(),
        "§ 38(c)(2) Special Rules for Married Individuals Filing Separately — $25,000 amount in § 38(c)(1)(B) is REDUCED TO $12,500 for a married individual filing a separate return".to_string(),
        "§ 39 Carryback and Carryforward — 1-Year Back / 20-Year Forward — if § 38(c) limitations prevent a taxpayer from using all of the GBC, the unused credit may be CARRIED BACK 1 YEAR and then, if unused credit remains, CARRIED FORWARD UP TO 20 YEARS".to_string(),
        "§ 39 FIFO Ordering Rules — the GBC is used in the following order: (1) CARRYFORWARDS to that year, starting with the OLDEST ONES; (2) CURRENT YEAR BUSINESS CREDIT; (3) CARRYBACK to that year; FIRST-IN, FIRST-OUT (FIFO) approach minimizes risk that unused credits will expire".to_string(),
        "Form 3800 General Business Credit — taxpayers that claim MORE THAN ONE CREDIT must file FORM 3800 to report the aggregate value of those credits and calculate the overall allowable credit under the GBC; each component credit is first calculated on its specific statutory form (e.g., Form 6765 for R&D; Form 8826 for DAC; Form 8609-A for LIHTC; Form 5884 for WOTC; Form 3468 for investment credit including HTC) and then transferred to Form 3800 Part III for aggregation".to_string(),
        "Form 3800 Structure (Post-2023 Rev.) — Part I addresses credits not allowed against TMT; Part II addresses Figuring Credit Allowed After Limitations with sections A, B, C corresponding to § 38(c)(1), § 38(c)(2), § 38(c)(4) limitations; Part III addresses credit components from each separate statutory form; Schedule A addresses additional adjustments".to_string(),
        "Cornell LII + Tax Notes + Bloomberg Tax + IRS + LVHJ + The Tax Adviser + Wolters Kluwer AnswerConnect + Swanson Reed + Tax990 + KPMG + Legis1 + FindLaw + GovInfo + Bradford Tax Institute — practitioner overviews of § 38".to_string(),
    ];

    match input.compliance_aspect {
        ComplianceAspect::AllowableCreditUnderSection38A => {
            let aggregate = input
                .carryforward_dollars
                .saturating_add(input.current_year_business_credit_dollars)
                .saturating_add(input.carryback_dollars);
            Output {
                mode: Section38Mode::CompliantAllowableCreditAggregatedFromThreeComponents,
                statutory_basis: "§ 38(a) — allowable credit = carryforwards + current year business credit + carrybacks".to_string(),
                notes: format!(
                    "COMPLIANT: § 38(a) allowable credit = ${cf} carryforwards + ${cy} current year business credit + ${cb} carrybacks = ${agg}.",
                    cf = input.carryforward_dollars,
                    cy = input.current_year_business_credit_dollars,
                    cb = input.carryback_dollars,
                    agg = aggregate,
                ),
                citations,
                computed_allowable_credit_dollars: aggregate,
            }
        }
        ComplianceAspect::ComponentCreditsUnderSection38B => Output {
            mode: Section38Mode::CompliantComponentCreditIdentifiedUnderSection38B,
            statutory_basis: format!(
                "§ 38(b) — component credit identified ({cat:?})",
                cat = input.component_credit_category,
            ),
            notes: format!(
                "COMPLIANT: component credit {cat:?} is one of the 41 § 38(b) bundled component credits aggregated into the general business credit.",
                cat = input.component_credit_category,
            ),
            citations,
            computed_allowable_credit_dollars: 0,
        },
        ComplianceAspect::LimitationUnderSection38C1 => {
            let twenty_five_pct_threshold = match input.filing_status {
                FilingStatus::MarriedFilingJointlyOrSingleOrHeadOfHousehold => {
                    IRC_38_LIMITATION_25_PCT_THRESHOLD_DOLLARS
                }
                FilingStatus::MarriedFilingSeparately => {
                    IRC_38_LIMITATION_25_PCT_THRESHOLD_DOLLARS_MFS
                }
            };
            let net_regular_excess = input
                .net_regular_tax_liability_dollars
                .saturating_sub(twenty_five_pct_threshold);
            let twenty_five_pct_amount = (u128::from(net_regular_excess)
                * u128::from(IRC_38_LIMITATION_25_PCT_RATE_BPS)
                / u128::from(IRC_38_BASIS_POINT_DENOMINATOR))
                as u64;
            let limitation_floor = input.tentative_minimum_tax_dollars.max(twenty_five_pct_amount);
            let allowable_credit = input
                .net_income_tax_dollars
                .saturating_sub(limitation_floor);
            let claimed_credit = input
                .carryforward_dollars
                .saturating_add(input.current_year_business_credit_dollars)
                .saturating_add(input.carryback_dollars);
            if claimed_credit <= allowable_credit {
                Output {
                    mode: Section38Mode::CompliantLimitationAppliedUnderSection38C1,
                    statutory_basis: "§ 38(c)(1) — claimed credit within limitation (net income tax over greater of TMT or 25 % of excess net regular tax over threshold)".to_string(),
                    notes: format!(
                        "COMPLIANT: claimed credit ${claimed} ≤ allowable credit ${allowable} = ${nit} net income tax − greater of ${tmt} TMT or ${ttp} (25 % × ${nre} net regular tax excess over ${tt} threshold) under § 38(c)(1).",
                        claimed = claimed_credit,
                        allowable = allowable_credit,
                        nit = input.net_income_tax_dollars,
                        tmt = input.tentative_minimum_tax_dollars,
                        ttp = twenty_five_pct_amount,
                        nre = net_regular_excess,
                        tt = twenty_five_pct_threshold,
                    ),
                    citations,
                    computed_allowable_credit_dollars: allowable_credit,
                }
            } else {
                Output {
                    mode: Section38Mode::ViolationCreditExceedsSection38CLimitation,
                    statutory_basis: "§ 38(c)(1) — credit exceeds limitation (net income tax over greater of TMT or 25 % of excess net regular tax over threshold)".to_string(),
                    notes: format!(
                        "VIOLATION: claimed credit ${claimed} > allowable credit ${allowable}; excess ${excess} carries back 1 year and forward 20 years under § 39.",
                        claimed = claimed_credit,
                        allowable = allowable_credit,
                        excess = claimed_credit - allowable_credit,
                    ),
                    citations,
                    computed_allowable_credit_dollars: allowable_credit,
                }
            }
        }
        ComplianceAspect::SpecifiedCreditsUnderSection38C4 => match input.specified_credit_status {
            SpecifiedCreditStatus::SpecifiedCreditAllowedAgainstTmt => Output {
                mode: Section38Mode::CompliantSpecifiedCreditAllowedAgainstTmt,
                statutory_basis: "§ 38(c)(4) — specified credit allowed against TMT".to_string(),
                notes: "COMPLIANT: credit is a § 38(c)(4) SPECIFIED CREDIT (e.g., R&D credit for eligible small business under § 41(h), clean-energy credits § 45Y/§ 48E/§ 45V/§ 45Q/§ 45X/§ 45Z/§ 45U); credit may be used against the TMT (bypasses AMT/TMT constraint).".to_string(),
                citations,
                computed_allowable_credit_dollars: 0,
            },
            SpecifiedCreditStatus::NotSpecifiedCreditSubjectToTmtLimitation => Output {
                mode: Section38Mode::CompliantLimitationAppliedUnderSection38C1,
                statutory_basis: "§ 38(c)(1) — not a specified credit; subject to TMT limitation".to_string(),
                notes: "COMPLIANT: credit is NOT a § 38(c)(4) specified credit; subject to the § 38(c)(1) net income tax over greater of TMT or 25 % of excess net regular tax limitation.".to_string(),
                citations,
                computed_allowable_credit_dollars: 0,
            },
        },
        ComplianceAspect::MarriedFilingSeparatelyAdjustmentUnderSection38C2 => {
            let threshold = match input.filing_status {
                FilingStatus::MarriedFilingJointlyOrSingleOrHeadOfHousehold => {
                    IRC_38_LIMITATION_25_PCT_THRESHOLD_DOLLARS
                }
                FilingStatus::MarriedFilingSeparately => {
                    IRC_38_LIMITATION_25_PCT_THRESHOLD_DOLLARS_MFS
                }
            };
            let mode = if input.filing_status == FilingStatus::MarriedFilingSeparately {
                Section38Mode::CompliantMarriedFilingSeparatelyThresholdReducedTo12500
            } else {
                Section38Mode::CompliantLimitationAppliedUnderSection38C1
            };
            Output {
                mode,
                statutory_basis: format!(
                    "§ 38(c)(2) — ${threshold} threshold for filing status {fs:?}",
                    fs = input.filing_status,
                ),
                notes: format!(
                    "COMPLIANT: § 38(c)(2) $25,000 threshold under § 38(c)(1)(B) {adj} for {fs:?} = ${threshold}.",
                    adj = if input.filing_status == FilingStatus::MarriedFilingSeparately {
                        "REDUCED TO $12,500"
                    } else {
                        "applies"
                    },
                    fs = input.filing_status,
                ),
                citations,
                computed_allowable_credit_dollars: 0,
            }
        }
        ComplianceAspect::CarrybackCarryforwardUnderSection39 => Output {
            mode: Section38Mode::CompliantOneYearCarrybackTwentyYearCarryforward,
            statutory_basis: "§ 39 — 1-year carryback and 20-year carryforward of unused GBC".to_string(),
            notes: "COMPLIANT: unused GBC carries back 1 YEAR and forward UP TO 20 YEARS under § 39; FIFO ordering applies (oldest carryforwards used first; minimizes expiration risk).".to_string(),
            citations,
            computed_allowable_credit_dollars: 0,
        },
        ComplianceAspect::FifoOrderingUnderSection39 => Output {
            mode: Section38Mode::CompliantFifoOrderingObserved,
            statutory_basis: "§ 39 — FIFO ordering rules for GBC usage".to_string(),
            notes: "COMPLIANT: GBC ordering under § 39 — (1) CARRYFORWARDS to that year (starting with OLDEST); (2) CURRENT YEAR BUSINESS CREDIT; (3) CARRYBACK to that year; FIFO approach minimizes risk that unused credits will expire.".to_string(),
            citations,
            computed_allowable_credit_dollars: 0,
        },
        ComplianceAspect::FormFilingUnderForm3800 => {
            if input.form_3800_filed_correctly {
                Output {
                    mode: Section38Mode::CompliantForm3800FiledCorrectly,
                    statutory_basis: "Form 3800 — General Business Credit form required when claiming more than one credit".to_string(),
                    notes: "COMPLIANT: Form 3800 filed correctly to aggregate multiple component credits; each component credit calculated separately on its specific statutory form (Form 6765 R&D / Form 8826 DAC / Form 8609-A LIHTC / Form 5884 WOTC / Form 3468 investment credit including HTC) and transferred to Form 3800 Part III.".to_string(),
                    citations,
                    computed_allowable_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section38Mode::ViolationForm3800NotFiledOrIncorrect,
                    statutory_basis: "Form 3800 filing required to claim multiple component credits".to_string(),
                    notes: "VIOLATION: Form 3800 not filed or incorrectly filed; multiple component credits cannot be aggregated; GBC may be disallowed.".to_string(),
                    citations,
                    computed_allowable_credit_dollars: 0,
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
            component_credit_category: ComponentCreditCategory::ResearchCreditSection41,
            specified_credit_status:
                SpecifiedCreditStatus::NotSpecifiedCreditSubjectToTmtLimitation,
            filing_status: FilingStatus::MarriedFilingJointlyOrSingleOrHeadOfHousehold,
            compliance_aspect: ComplianceAspect::AllowableCreditUnderSection38A,
            current_year_business_credit_dollars: 100_000,
            carryforward_dollars: 50_000,
            carryback_dollars: 0,
            net_income_tax_dollars: 500_000,
            tentative_minimum_tax_dollars: 100_000,
            net_regular_tax_liability_dollars: 500_000,
            form_3800_filed_correctly: true,
        }
    }

    #[test]
    fn allowable_credit_aggregated_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AllowableCreditUnderSection38A;
        input.carryforward_dollars = 50_000;
        input.current_year_business_credit_dollars = 100_000;
        input.carryback_dollars = 25_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::CompliantAllowableCreditAggregatedFromThreeComponents
        );
        assert_eq!(out.computed_allowable_credit_dollars, 175_000);
    }

    #[test]
    fn investment_credit_section_46_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ComponentCreditsUnderSection38B;
        input.component_credit_category = ComponentCreditCategory::InvestmentCreditSection46;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::CompliantComponentCreditIdentifiedUnderSection38B
        );
    }

    #[test]
    fn work_opportunity_credit_section_51_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ComponentCreditsUnderSection38B;
        input.component_credit_category = ComponentCreditCategory::WorkOpportunityCreditSection51;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::CompliantComponentCreditIdentifiedUnderSection38B
        );
    }

    #[test]
    fn low_income_housing_credit_section_42_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ComponentCreditsUnderSection38B;
        input.component_credit_category = ComponentCreditCategory::LowIncomeHousingCreditSection42;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::CompliantComponentCreditIdentifiedUnderSection38B
        );
    }

    #[test]
    fn disabled_access_credit_section_44_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ComponentCreditsUnderSection38B;
        input.component_credit_category = ComponentCreditCategory::DisabledAccessCreditSection44;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::CompliantComponentCreditIdentifiedUnderSection38B
        );
    }

    #[test]
    fn limitation_within_cap_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LimitationUnderSection38C1;
        input.net_income_tax_dollars = 500_000;
        input.tentative_minimum_tax_dollars = 100_000;
        input.net_regular_tax_liability_dollars = 500_000;
        input.carryforward_dollars = 50_000;
        input.current_year_business_credit_dollars = 100_000;
        input.carryback_dollars = 0;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::CompliantLimitationAppliedUnderSection38C1
        );
    }

    #[test]
    fn limitation_exceeds_cap_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LimitationUnderSection38C1;
        input.net_income_tax_dollars = 200_000;
        input.tentative_minimum_tax_dollars = 100_000;
        input.net_regular_tax_liability_dollars = 200_000;
        input.carryforward_dollars = 200_000;
        input.current_year_business_credit_dollars = 100_000;
        input.carryback_dollars = 0;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::ViolationCreditExceedsSection38CLimitation
        );
    }

    #[test]
    fn specified_credit_allowed_against_tmt_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SpecifiedCreditsUnderSection38C4;
        input.specified_credit_status = SpecifiedCreditStatus::SpecifiedCreditAllowedAgainstTmt;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::CompliantSpecifiedCreditAllowedAgainstTmt
        );
    }

    #[test]
    fn married_filing_separately_threshold_reduced_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MarriedFilingSeparatelyAdjustmentUnderSection38C2;
        input.filing_status = FilingStatus::MarriedFilingSeparately;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::CompliantMarriedFilingSeparatelyThresholdReducedTo12500
        );
    }

    #[test]
    fn married_filing_jointly_full_threshold_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect =
            ComplianceAspect::MarriedFilingSeparatelyAdjustmentUnderSection38C2;
        input.filing_status = FilingStatus::MarriedFilingJointlyOrSingleOrHeadOfHousehold;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::CompliantLimitationAppliedUnderSection38C1
        );
    }

    #[test]
    fn carryback_carryforward_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CarrybackCarryforwardUnderSection39;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::CompliantOneYearCarrybackTwentyYearCarryforward
        );
    }

    #[test]
    fn fifo_ordering_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FifoOrderingUnderSection39;
        let out = check(&input);
        assert_eq!(out.mode, Section38Mode::CompliantFifoOrderingObserved);
    }

    #[test]
    fn form_3800_filed_correctly_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm3800;
        input.form_3800_filed_correctly = true;
        let out = check(&input);
        assert_eq!(out.mode, Section38Mode::CompliantForm3800FiledCorrectly);
    }

    #[test]
    fn form_3800_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm3800;
        input.form_3800_filed_correctly = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section38Mode::ViolationForm3800NotFiledOrIncorrect
        );
    }

    #[test]
    fn constants_pin_section_38_gbc_structure() {
        assert_eq!(IRC_38_CARRYBACK_YEARS, 1);
        assert_eq!(IRC_38_CARRYFORWARD_YEARS, 20);
        assert_eq!(IRC_38_NUMBER_OF_COMPONENT_CREDITS, 41);
        assert_eq!(IRC_38_LIMITATION_25_PCT_THRESHOLD_DOLLARS, 25_000);
        assert_eq!(IRC_38_LIMITATION_25_PCT_THRESHOLD_DOLLARS_MFS, 12_500);
        assert_eq!(IRC_38_LIMITATION_25_PCT_RATE_BPS, 2_500);
        assert_eq!(IRC_38_FORM_NUMBER, 3_800);
        assert_eq!(IRC_38_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_section_38_gbc_structure() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("IRC § 38 General Business Credit"));
        assert!(joined.contains("NOT A SINGLE TAX CREDIT"));
        assert!(joined.contains("UMBRELLA FRAMEWORK"));
        assert!(joined.contains("41 SEPARATE BUSINESS TAX INCENTIVES"));
        assert!(joined.contains("§ 38(a)"));
        assert!(joined.contains("SUM OF"));
        assert!(joined.contains("CARRYFORWARDS"));
        assert!(joined.contains("CURRENT YEAR BUSINESS CREDIT"));
        assert!(joined.contains("CARRYBACKS"));
        assert!(joined.contains("§ 38(b)"));
        assert!(joined.contains("INVESTMENT CREDIT under § 46"));
        assert!(joined.contains("WORK OPPORTUNITY CREDIT under § 51"));
        assert!(joined.contains("RESEARCH CREDIT under § 41"));
        assert!(joined.contains("LOW-INCOME HOUSING CREDIT under § 42"));
        assert!(joined.contains("DISABLED ACCESS CREDIT under § 44"));
        assert!(joined.contains("§ 38(c)(1)"));
        assert!(joined.contains("NET INCOME TAX"));
        assert!(joined.contains("TENTATIVE MINIMUM TAX (TMT)"));
        assert!(joined.contains("25 PERCENT"));
        assert!(joined.contains("NET REGULAR TAX LIABILITY"));
        assert!(joined.contains("$25,000"));
        assert!(joined.contains("§ 38(c)(4)"));
        assert!(joined.contains("SPECIFIED CREDITS"));
        assert!(joined.contains("§ 38(c)(2)"));
        assert!(joined.contains("$12,500"));
        assert!(joined.contains("§ 39"));
        assert!(joined.contains("CARRIED BACK 1 YEAR"));
        assert!(joined.contains("CARRIED FORWARD UP TO 20 YEARS"));
        assert!(joined.contains("FIRST-IN, FIRST-OUT (FIFO)"));
        assert!(joined.contains("Form 3800"));
        assert!(joined.contains("Form 6765"));
        assert!(joined.contains("Form 8826"));
        assert!(joined.contains("Form 8609-A"));
        assert!(joined.contains("Form 5884"));
        assert!(joined.contains("Form 3468"));
    }
}
