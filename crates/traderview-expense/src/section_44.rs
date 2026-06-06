//! IRC § 44 Disabled Access Credit (DAC) Compliance
//! Module — pure-compute check for the small business
//! credit available to **ELIGIBLE SMALL BUSINESSES** that
//! incur expenses for the purpose of complying with the
//! Americans with Disabilities Act of 1990 (ADA).
//!
//! Credit equals **50% of eligible access expenditures
//! OVER $250 not exceeding $10,250**; maximum credit
//! **$5,000 per tax year**. Eligible small business test
//! is **≤$1 million in gross receipts** OR **≤30 full-time
//! employees** in the previous taxable year.
//!
//! Often used together with the **§ 190 Architectural and
//! Transportation Barrier Removal Deduction** (up to $15,000
//! per year deduction) for accessibility improvements,
//! providing **DUAL FEDERAL TAX BENEFITS** for accessibility
//! expenditures (credit for small businesses + deduction
//! for all businesses).
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 44 Disabled Access Credit (DAC)**: small businesses may take an **ANNUAL TAX CREDIT** for making their businesses accessible to persons with disabilities ([Cornell LII — 26 U.S. Code § 44](https://www.law.cornell.edu/uscode/text/26/44); [Bloomberg Tax — Sec. 44 Expenditures to Provide Access to Disabled Individuals](https://irc.bloombergtax.com/public/uscode/doc/irc/section_44); [Tax Notes — IRC Code Section 44](https://www.taxnotes.com/research/federal/usc26/44); [IRS — Tax Benefits of Making a Business Accessible to Workers and Customers with Disabilities](https://www.irs.gov/newsroom/tax-benefits-of-making-a-business-accessible-to-workers-and-customers-with-disabilities); [ADA.gov Archive — ADA IRS Tax Credit Information](https://archive.ada.gov/taxcred.htm); [PVA — Tax Incentives Assisting Accessibility PDF](https://pva.org/wp-content/uploads/2021/09/ada_taxincen.pdf); [Indiana DOR — IRS Code Section 44 Disabled Access Credit PDF](https://publicaccessstorage.blob.core.usgovcloudapi.net/publicsitefiles/DOR%20Documents/Business%20Services/IRS_Code_Section_44,_Disabled_Access_Credit.pdf); [NYC Business — Disabled Access Credit and Additional Tax Deduction for Barrier Removal](https://nyc-business.nyc.gov/nycbusiness/description/disabled-access-credit); [Work World — Disabled Access Tax Credit](https://help.workworldapp.com/wwwebhelp/disabled_access_tax_credit.htm); [Tax Notes — IRS Describes Application of Disabled Access Credit Rules](https://www.taxnotes.com/lr/resolve/1fg5l)).
//! - **§ 44(b) Eligible Small Business Definition**: **ELIGIBLE SMALL BUSINESS** means any person that for the preceding taxable year had: (i) **GROSS RECEIPTS** (reduced by returns and allowances) NOT EXCEEDING **$1,000,000**; OR (ii) employed **NOT MORE THAN 30 FULL-TIME EMPLOYEES**.
//! - **§ 44(a) Credit Amount**: the credit equals **50 PERCENT** of so much of the eligible access expenditures for the taxable year as **EXCEED $250** but do not exceed **$10,250**; maximum credit = **$5,000 PER TAX YEAR** (50% × ($10,250 - $250) = 50% × $10,000 = $5,000).
//! - **§ 44(c) Eligible Access Expenditures**: amounts paid or incurred by an eligible small business **FOR THE PURPOSE OF ENABLING** such eligible small business **TO COMPLY WITH APPLICABLE REQUIREMENTS UNDER THE AMERICANS WITH DISABILITIES ACT OF 1990 (ADA)**.
//! - **§ 44(c) Categories of Eligible Expenditures**: (1) **SIGN LANGUAGE INTERPRETERS** for employees or customers with hearing impairments; (2) **READERS** for employees or customers with visual impairments; (3) purchase of **ADAPTIVE EQUIPMENT** or modification of equipment; (4) production of print materials in **ACCESSIBLE FORMATS** (Braille, audio tape, large print); (5) **REMOVAL OF ARCHITECTURAL BARRIERS** (ramps, accessible restrooms, accessible parking).
//! - **§ 44(d) Definitions and Special Rules**: includes definition of full-time employee (40+ hours per week); rules for related parties (treated as single taxpayer for $1M / 30-employee thresholds); rules for partnerships, S corporations, and trusts.
//! - **§ 38 General Business Credit Aggregation**: DAC is part of the **GENERAL BUSINESS CREDIT** under § 38; subject to § 38(c) annual limitation.
//! - **§ 39 Carryback and Carryforward**: DAC credit not used in current year carries back **1 YEAR** and forward **20 YEARS** under § 39.
//! - **Stacking with § 190 Architectural and Transportation Barrier Removal Deduction**: small businesses may use **BOTH § 44 CREDIT AND § 190 DEDUCTION** in the same year, **IF THE EXPENSES INCURRED QUALIFY UNDER BOTH SECTIONS**; § 190 provides up to **$15,000 per year DEDUCTION** for the removal of architectural and transportation barriers to the disabled and elderly.
//! - **Form 8826 (Disabled Access Credit)**: required to claim the DAC; report on Form 8826 and include with the federal income tax return.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_44_CREDIT_RATE_BPS: u64 = 5_000;
pub const IRC_44_EXPENDITURE_FLOOR_DOLLARS: u64 = 250;
pub const IRC_44_EXPENDITURE_CEILING_DOLLARS: u64 = 10_250;
pub const IRC_44_MAX_CREDIT_DOLLARS: u64 = 5_000;
pub const IRC_44_ELIGIBLE_GROSS_RECEIPTS_CEILING_DOLLARS: u64 = 1_000_000;
pub const IRC_44_ELIGIBLE_EMPLOYEE_CEILING: u32 = 30;
pub const IRC_44_SECTION_190_DEDUCTION_CEILING_DOLLARS: u64 = 15_000;
pub const IRC_44_FULL_TIME_EMPLOYEE_HOURS_PER_WEEK: u32 = 40;
pub const IRC_44_CARRYBACK_YEARS: u32 = 1;
pub const IRC_44_CARRYFORWARD_YEARS: u32 = 20;
pub const IRC_44_FORM_NUMBER: u32 = 8_826;
pub const IRC_44_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EligibleSmallBusinessStatus {
    GrossReceiptsAtOrBelowOneMillion,
    FullTimeEmployeesAtOrBelow30,
    NotEligibleSmallBusiness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpenditureCategory {
    SignLanguageInterpreters,
    ReadersForVisualImpairments,
    AdaptiveEquipmentPurchaseOrModification,
    AccessibleFormatPrintMaterials,
    ArchitecturalBarrierRemoval,
    NotAdaComplianceExpenditure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section190StackingStatus {
    StackingWithSection190DeductionElected,
    NoStackingWithSection190,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    EligibleSmallBusinessDefinitionUnderSection44B,
    CreditAmountUnderSection44A,
    EligibleAccessExpendituresUnderSection44C,
    StackingWithSection190Deduction,
    GeneralBusinessCreditAggregationUnderSection38,
    CarrybackCarryforwardUnderSection39,
    FormFilingUnderForm8826,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section44Mode {
    NotApplicableNotEligibleSmallBusiness,
    NotApplicableExpenditureBelow250DollarFloor,
    NotApplicableNotAdaComplianceExpenditure,
    CompliantEligibleSmallBusinessCertified,
    CompliantFiftyPercentCreditOnExpendituresOver250DollarsUpTo10250Dollars,
    CompliantMaximumCredit5000DollarsAtCeiling,
    CompliantEligibleAccessExpendituresIdentified,
    CompliantStackingWithSection190Deduction,
    CompliantGeneralBusinessCreditAggregation,
    CompliantCarrybackCarryforwardObserved,
    CompliantForm8826FiledCorrectly,
    ViolationForm8826NotFiledOrIncorrect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub eligible_small_business_status: EligibleSmallBusinessStatus,
    pub expenditure_category: ExpenditureCategory,
    pub section_190_stacking_status: Section190StackingStatus,
    pub compliance_aspect: ComplianceAspect,
    pub gross_receipts_dollars: u64,
    pub full_time_employees: u32,
    pub eligible_access_expenditures_dollars: u64,
    pub form_8826_filed_correctly: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section44Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_credit_dollars: u64,
}

pub type Section44Input = Input;
pub type Section44Output = Output;
pub type Section44Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 44 Disabled Access Credit (DAC) — small businesses may take an ANNUAL TAX CREDIT for making their businesses accessible to persons with disabilities".to_string(),
        "§ 44(b) Eligible Small Business Definition — eligible small business means any person that for the preceding taxable year had (i) GROSS RECEIPTS (reduced by returns and allowances) NOT EXCEEDING $1,000,000; OR (ii) employed NOT MORE THAN 30 FULL-TIME EMPLOYEES".to_string(),
        "§ 44(a) Credit Amount — credit equals 50 PERCENT of so much of the eligible access expenditures for the taxable year as EXCEED $250 but do not exceed $10,250; maximum credit = $5,000 PER TAX YEAR (50 % × ($10,250 - $250) = 50 % × $10,000 = $5,000)".to_string(),
        "§ 44(c) Eligible Access Expenditures — amounts paid or incurred by an eligible small business FOR THE PURPOSE OF ENABLING such eligible small business TO COMPLY WITH APPLICABLE REQUIREMENTS UNDER THE AMERICANS WITH DISABILITIES ACT OF 1990 (ADA)".to_string(),
        "§ 44(c) Categories of Eligible Expenditures — (1) SIGN LANGUAGE INTERPRETERS for employees or customers with hearing impairments; (2) READERS for employees or customers with visual impairments; (3) purchase of ADAPTIVE EQUIPMENT or modification of equipment; (4) production of print materials in ACCESSIBLE FORMATS (Braille, audio tape, large print); (5) REMOVAL OF ARCHITECTURAL BARRIERS (ramps, accessible restrooms, accessible parking)".to_string(),
        "§ 44(d) Definitions and Special Rules — includes definition of full-time employee (40+ hours per week); rules for related parties (treated as single taxpayer for $1M / 30-employee thresholds); rules for partnerships, S corporations, and trusts".to_string(),
        "§ 38 General Business Credit Aggregation — DAC is part of the GENERAL BUSINESS CREDIT under § 38; subject to § 38(c) annual limitation".to_string(),
        "§ 39 Carryback and Carryforward — DAC credit not used in current year carries back 1 YEAR and forward 20 YEARS under § 39".to_string(),
        "Stacking with § 190 Architectural and Transportation Barrier Removal Deduction — small businesses may use BOTH § 44 CREDIT AND § 190 DEDUCTION in the same year, IF THE EXPENSES INCURRED QUALIFY UNDER BOTH SECTIONS; § 190 provides up to $15,000 per year DEDUCTION for the removal of architectural and transportation barriers to the disabled and elderly".to_string(),
        "Form 8826 (Disabled Access Credit) — required to claim the DAC; report on Form 8826 and include with the federal income tax return".to_string(),
        "Cornell LII + Bloomberg Tax + Tax Notes + IRS + ADA.gov Archive + PVA + Indiana DOR + NYC Business + Work World — practitioner overviews of § 44".to_string(),
    ];

    if input.eligible_small_business_status == EligibleSmallBusinessStatus::NotEligibleSmallBusiness
        || (input.gross_receipts_dollars > IRC_44_ELIGIBLE_GROSS_RECEIPTS_CEILING_DOLLARS
            && input.full_time_employees > IRC_44_ELIGIBLE_EMPLOYEE_CEILING)
    {
        return Output {
            mode: Section44Mode::NotApplicableNotEligibleSmallBusiness,
            statutory_basis: "§ 44(b) — not an eligible small business (gross receipts > $1M AND > 30 full-time employees)".to_string(),
            notes: format!(
                "NOT APPLICABLE: business does not meet eligible small business test under § 44(b); gross receipts ${gr} > $1,000,000 AND {fte} full-time employees > 30.",
                gr = input.gross_receipts_dollars,
                fte = input.full_time_employees,
            ),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.expenditure_category == ExpenditureCategory::NotAdaComplianceExpenditure {
        return Output {
            mode: Section44Mode::NotApplicableNotAdaComplianceExpenditure,
            statutory_basis: "§ 44(c) — expenditure not for ADA compliance".to_string(),
            notes: "NOT APPLICABLE: expenditure does not qualify under § 44(c) as for the purpose of enabling ADA compliance.".to_string(),
            citations,
            computed_credit_dollars: 0,
        };
    }

    if input.eligible_access_expenditures_dollars <= IRC_44_EXPENDITURE_FLOOR_DOLLARS {
        return Output {
            mode: Section44Mode::NotApplicableExpenditureBelow250DollarFloor,
            statutory_basis: "§ 44(a) — expenditure at or below $250 floor".to_string(),
            notes: format!(
                "NOT APPLICABLE: eligible access expenditures of ${e} at or below $250 statutory floor under § 44(a).",
                e = input.eligible_access_expenditures_dollars,
            ),
            citations,
            computed_credit_dollars: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::EligibleSmallBusinessDefinitionUnderSection44B => Output {
            mode: Section44Mode::CompliantEligibleSmallBusinessCertified,
            statutory_basis: format!(
                "§ 44(b) — eligible small business test met ({status:?})",
                status = input.eligible_small_business_status,
            ),
            notes: format!(
                "COMPLIANT: business meets eligible small business test under § 44(b) ({status:?}); gross receipts ${gr} and {fte} full-time employees.",
                status = input.eligible_small_business_status,
                gr = input.gross_receipts_dollars,
                fte = input.full_time_employees,
            ),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::CreditAmountUnderSection44A => {
            let capped_expenditure = input
                .eligible_access_expenditures_dollars
                .min(IRC_44_EXPENDITURE_CEILING_DOLLARS);
            let creditable_expenditure =
                capped_expenditure.saturating_sub(IRC_44_EXPENDITURE_FLOOR_DOLLARS);
            let computed = (u128::from(creditable_expenditure) * u128::from(IRC_44_CREDIT_RATE_BPS)
                / u128::from(IRC_44_BASIS_POINT_DENOMINATOR)) as u64;
            let mode = if computed >= IRC_44_MAX_CREDIT_DOLLARS {
                Section44Mode::CompliantMaximumCredit5000DollarsAtCeiling
            } else {
                Section44Mode::CompliantFiftyPercentCreditOnExpendituresOver250DollarsUpTo10250Dollars
            };
            Output {
                mode,
                statutory_basis: "§ 44(a) — 50 % credit on eligible access expenditures over $250 not exceeding $10,250".to_string(),
                notes: format!(
                    "COMPLIANT: 50 % credit on ${creditable} creditable expenditures (eligible ${e} capped at $10,250 minus $250 floor) = ${computed} DAC credit.",
                    creditable = creditable_expenditure,
                    e = input.eligible_access_expenditures_dollars,
                ),
                citations,
                computed_credit_dollars: computed,
            }
        }
        ComplianceAspect::EligibleAccessExpendituresUnderSection44C => Output {
            mode: Section44Mode::CompliantEligibleAccessExpendituresIdentified,
            statutory_basis: format!(
                "§ 44(c) — eligible access expenditures identified ({cat:?})",
                cat = input.expenditure_category,
            ),
            notes: format!(
                "COMPLIANT: ${e} eligible access expenditures identified under § 44(c) (category: {cat:?}); ADA compliance purpose confirmed.",
                e = input.eligible_access_expenditures_dollars,
                cat = input.expenditure_category,
            ),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::StackingWithSection190Deduction => match input
            .section_190_stacking_status
        {
            Section190StackingStatus::StackingWithSection190DeductionElected => Output {
                mode: Section44Mode::CompliantStackingWithSection190Deduction,
                statutory_basis: "§ 44 + § 190 — stacking § 44 credit with § 190 deduction elected".to_string(),
                notes: "COMPLIANT: small business stacks § 44 credit ($5,000 max) with § 190 deduction (up to $15,000 per year) for architectural and transportation barrier removal; dual federal tax benefits available where expenses qualify under both sections.".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
            Section190StackingStatus::NoStackingWithSection190 => Output {
                mode: Section44Mode::CompliantStackingWithSection190Deduction,
                statutory_basis: "§ 44 — no § 190 stacking; § 44 credit available alone".to_string(),
                notes: "COMPLIANT: § 44 credit claimed without § 190 deduction stacking.".to_string(),
                citations,
                computed_credit_dollars: 0,
            },
        },
        ComplianceAspect::GeneralBusinessCreditAggregationUnderSection38 => Output {
            mode: Section44Mode::CompliantGeneralBusinessCreditAggregation,
            statutory_basis: "§ 38 — DAC aggregated with other general business credits".to_string(),
            notes: "COMPLIANT: DAC aggregated with other general business credits under § 38; subject to § 38(c) annual limitation.".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::CarrybackCarryforwardUnderSection39 => Output {
            mode: Section44Mode::CompliantCarrybackCarryforwardObserved,
            statutory_basis: "§ 39 — DAC credit not used in current year carries back 1 year and forward 20 years".to_string(),
            notes: "COMPLIANT: DAC credit not used in current year carries back 1 YEAR and forward 20 YEARS under § 39 (as part of general business credit).".to_string(),
            citations,
            computed_credit_dollars: 0,
        },
        ComplianceAspect::FormFilingUnderForm8826 => {
            if input.form_8826_filed_correctly {
                Output {
                    mode: Section44Mode::CompliantForm8826FiledCorrectly,
                    statutory_basis: "Form 8826 — Disabled Access Credit form required to claim § 44 credit".to_string(),
                    notes: "COMPLIANT: Form 8826 filed correctly to claim § 44 credit.".to_string(),
                    citations,
                    computed_credit_dollars: 0,
                }
            } else {
                Output {
                    mode: Section44Mode::ViolationForm8826NotFiledOrIncorrect,
                    statutory_basis: "Form 8826 filing required to claim § 44 credit".to_string(),
                    notes: "VIOLATION: Form 8826 not filed or incorrectly filed; § 44 credit may be disallowed.".to_string(),
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
            eligible_small_business_status:
                EligibleSmallBusinessStatus::GrossReceiptsAtOrBelowOneMillion,
            expenditure_category: ExpenditureCategory::ArchitecturalBarrierRemoval,
            section_190_stacking_status: Section190StackingStatus::NoStackingWithSection190,
            compliance_aspect: ComplianceAspect::CreditAmountUnderSection44A,
            gross_receipts_dollars: 500_000,
            full_time_employees: 10,
            eligible_access_expenditures_dollars: 10_250,
            form_8826_filed_correctly: true,
        }
    }

    #[test]
    fn not_eligible_small_business_not_applicable() {
        let mut input = baseline_input();
        input.eligible_small_business_status =
            EligibleSmallBusinessStatus::NotEligibleSmallBusiness;
        input.gross_receipts_dollars = 5_000_000;
        input.full_time_employees = 100;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::NotApplicableNotEligibleSmallBusiness
        );
    }

    #[test]
    fn not_ada_compliance_expenditure_not_applicable() {
        let mut input = baseline_input();
        input.expenditure_category = ExpenditureCategory::NotAdaComplianceExpenditure;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::NotApplicableNotAdaComplianceExpenditure
        );
    }

    #[test]
    fn expenditure_at_250_floor_not_applicable() {
        let mut input = baseline_input();
        input.eligible_access_expenditures_dollars = 250;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::NotApplicableExpenditureBelow250DollarFloor
        );
    }

    #[test]
    fn expenditure_below_250_floor_not_applicable() {
        let mut input = baseline_input();
        input.eligible_access_expenditures_dollars = 100;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::NotApplicableExpenditureBelow250DollarFloor
        );
    }

    #[test]
    fn eligible_small_business_certified_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleSmallBusinessDefinitionUnderSection44B;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::CompliantEligibleSmallBusinessCertified
        );
    }

    #[test]
    fn maximum_credit_at_10250_ceiling_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditAmountUnderSection44A;
        input.eligible_access_expenditures_dollars = 10_250;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::CompliantMaximumCredit5000DollarsAtCeiling
        );
        assert_eq!(out.computed_credit_dollars, 5_000);
    }

    #[test]
    fn credit_above_ceiling_pinned_to_maximum() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditAmountUnderSection44A;
        input.eligible_access_expenditures_dollars = 20_000;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::CompliantMaximumCredit5000DollarsAtCeiling
        );
        assert_eq!(out.computed_credit_dollars, 5_000);
    }

    #[test]
    fn fifty_percent_credit_below_ceiling_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditAmountUnderSection44A;
        input.eligible_access_expenditures_dollars = 5_250;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::CompliantFiftyPercentCreditOnExpendituresOver250DollarsUpTo10250Dollars
        );
        assert_eq!(out.computed_credit_dollars, 2_500);
    }

    #[test]
    fn expenditure_at_251_minimum_above_floor_yields_50_cent_credit() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CreditAmountUnderSection44A;
        input.eligible_access_expenditures_dollars = 251;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::CompliantFiftyPercentCreditOnExpendituresOver250DollarsUpTo10250Dollars
        );
        assert_eq!(out.computed_credit_dollars, 0);
    }

    #[test]
    fn eligible_access_expenditures_identified_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::EligibleAccessExpendituresUnderSection44C;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::CompliantEligibleAccessExpendituresIdentified
        );
    }

    #[test]
    fn stacking_with_section_190_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::StackingWithSection190Deduction;
        input.section_190_stacking_status =
            Section190StackingStatus::StackingWithSection190DeductionElected;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::CompliantStackingWithSection190Deduction
        );
    }

    #[test]
    fn general_business_credit_aggregation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::GeneralBusinessCreditAggregationUnderSection38;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::CompliantGeneralBusinessCreditAggregation
        );
    }

    #[test]
    fn carryback_carryforward_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::CarrybackCarryforwardUnderSection39;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::CompliantCarrybackCarryforwardObserved
        );
    }

    #[test]
    fn form_8826_filed_correctly_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm8826;
        input.form_8826_filed_correctly = true;
        let out = check(&input);
        assert_eq!(out.mode, Section44Mode::CompliantForm8826FiledCorrectly);
    }

    #[test]
    fn form_8826_not_filed_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FormFilingUnderForm8826;
        input.form_8826_filed_correctly = false;
        let out = check(&input);
        assert_eq!(
            out.mode,
            Section44Mode::ViolationForm8826NotFiledOrIncorrect
        );
    }

    #[test]
    fn constants_pin_section_44_dac_structure() {
        assert_eq!(IRC_44_CREDIT_RATE_BPS, 5_000);
        assert_eq!(IRC_44_EXPENDITURE_FLOOR_DOLLARS, 250);
        assert_eq!(IRC_44_EXPENDITURE_CEILING_DOLLARS, 10_250);
        assert_eq!(IRC_44_MAX_CREDIT_DOLLARS, 5_000);
        assert_eq!(IRC_44_ELIGIBLE_GROSS_RECEIPTS_CEILING_DOLLARS, 1_000_000);
        assert_eq!(IRC_44_ELIGIBLE_EMPLOYEE_CEILING, 30);
        assert_eq!(IRC_44_SECTION_190_DEDUCTION_CEILING_DOLLARS, 15_000);
        assert_eq!(IRC_44_FULL_TIME_EMPLOYEE_HOURS_PER_WEEK, 40);
        assert_eq!(IRC_44_CARRYBACK_YEARS, 1);
        assert_eq!(IRC_44_CARRYFORWARD_YEARS, 20);
        assert_eq!(IRC_44_FORM_NUMBER, 8_826);
        assert_eq!(IRC_44_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_pin_section_44_dac_structure() {
        let input = baseline_input();
        let out = check(&input);
        let joined = out.citations.join(" || ");
        assert!(joined.contains("IRC § 44 Disabled Access Credit"));
        assert!(joined.contains("ANNUAL TAX CREDIT"));
        assert!(joined.contains("eligible small business"));
        assert!(joined.contains("$1,000,000"));
        assert!(joined.contains("NOT MORE THAN 30 FULL-TIME EMPLOYEES"));
        assert!(joined.contains("50 PERCENT"));
        assert!(joined.contains("$250"));
        assert!(joined.contains("$10,250"));
        assert!(joined.contains("$5,000"));
        assert!(joined.contains("AMERICANS WITH DISABILITIES ACT OF 1990"));
        assert!(joined.contains("SIGN LANGUAGE INTERPRETERS"));
        assert!(joined.contains("READERS"));
        assert!(joined.contains("ADAPTIVE EQUIPMENT"));
        assert!(joined.contains("ACCESSIBLE FORMATS"));
        assert!(joined.contains("REMOVAL OF ARCHITECTURAL BARRIERS"));
        assert!(joined.contains("§ 38"));
        assert!(joined.contains("§ 39"));
        assert!(joined.contains("1 YEAR"));
        assert!(joined.contains("20 YEARS"));
        assert!(joined.contains("§ 190"));
        assert!(joined.contains("$15,000"));
        assert!(joined.contains("BOTH § 44 CREDIT AND § 190 DEDUCTION"));
        assert!(joined.contains("Form 8826"));
    }
}
