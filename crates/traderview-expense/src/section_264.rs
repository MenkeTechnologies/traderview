//! IRC § 264 disallowance of insurance-related interest and premium deductions.
//!
//! § 264 disallows four categories of insurance-related deductions: (1) premiums on
//! life insurance covering officers/employees where the taxpayer is the beneficiary
//! (§ 264(a)(1)), (2) interest on debt to purchase or carry single-premium life,
//! endowment, or annuity contracts (§ 264(a)(2)), (3) interest on debt under plans of
//! systematic borrowing against cash-value buildup (§ 264(a)(3) — added by Tax Reform
//! Act of 1986), (4) pro-rata interest disallowance for businesses owning life insurance
//! on owners/employees (§ 264(f) — "inside-buildup" regime added by Taxpayer Relief
//! Act of 1997). The regime targets the historical tax-arbitrage of deducting interest
//! on debt-financed insurance while the insurance contract's inside buildup grows tax-
//! deferred.
//!
//! § 264(a)(1) PREMIUM DEDUCTION DISALLOWED: premiums on any life insurance / endowment
//! contract covering officer/employee/financially-interested person where the taxpayer
//! is DIRECTLY OR INDIRECTLY the beneficiary.
//!
//! § 264(a)(2) SINGLE-PREMIUM INTEREST DISALLOWED: interest on debt incurred to
//! purchase or carry single-premium life, endowment, or annuity contracts is per se
//! non-deductible. A contract is "single-premium" if substantially all of the premiums
//! are paid within 4 years of purchase OR an amount is deposited with the insurer for
//! payment of a substantial number of future premiums.
//!
//! § 264(a)(3) SYSTEMATIC BORROWING DISALLOWED: interest on debt under plans
//! contemplating systematic direct or indirect borrowing against cash value of life
//! insurance, endowment, or annuity contracts.
//!
//! § 264(c) "4-of-7" EXCEPTION: § 264(a)(3) disallowance does NOT apply if no part of
//! 4 of the first 7 annual premiums are paid through indebtedness. Plus unforeseen-loss
//! exception, trade-or-business exception, and de-minimis-interest exception.
//!
//! § 264(f) PRO-RATA INTEREST DISALLOWANCE: for businesses, pro-rata portion of total
//! interest expense allocable to "unborrowed policy cash value" disallowed. Mechanic:
//! disallowance = total interest × (avg unborrowed policy cash value / avg total assets).
//! § 264(f)(4) EXCEPTIONS: owner-employee policies (20%+ owner), key-person policies for
//! up to 20 individuals, $50K aggregate threshold de minimis.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/264
//! - law.cornell.edu/cfr/text/26/1.264-4
//! - irs.gov/pub/irs-drop/rr-11-09.pdf (Rev. Rul. 2011-9)
//! - taxnotes.com/research/federal/usc26/264

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractType {
    SinglePremiumLifeEndowmentOrAnnuity,
    NonSinglePremiumLifeWithSystematicBorrowingPlan,
    NonSinglePremiumLifeNoSystematicBorrowing,
    BusinessOwnedLifeInsuranceWithUnborrowedCashValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExceptionStatus {
    NoExceptionInvoked,
    /// § 264(c)(1) 4-of-7 exception — no part of 4 of first 7 annual premiums was paid
    /// through debt.
    FourOfSevenExceptionSatisfied,
    /// § 264(c)(2) trade-or-business exception — debt incurred in connection with
    /// trade or business unrelated to the insurance policy.
    TradeOrBusinessExceptionSatisfied,
    /// § 264(c)(3) unforeseen-loss exception — substantial unforeseen loss of income or
    /// increase in financial obligation.
    UnforeseenLossExceptionSatisfied,
    /// § 264(c)(4) de-minimis-interest exception — interest paid does not exceed $100
    /// for the taxable year.
    DeMinimisInterestUnderHundredDollarsExceptionSatisfied,
    /// § 264(f)(4)(A) owner-employee exception — 20%+ owner-employee policy.
    OwnerEmployeeTwentyPctPolicySection264f4Exception,
    /// § 264(f)(4)(E) key-person exception — limited to 20 individuals max.
    KeyPersonPolicyUpToTwentyIndividualsSection264f4Exception,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section264A2SinglePremiumInterestDisallowedPerSe,
    Section264A3SystematicBorrowingInterestDisallowed,
    Section264cFourOfSevenExceptionPreservesDeduction,
    Section264cTradeOrBusinessExceptionPreservesDeduction,
    Section264cUnforeseenLossExceptionPreservesDeduction,
    Section264cDeMinimisInterestExceptionPreservesDeduction,
    Section264fProRataInterestDisallowanceApplied,
    Section264f4ExceptionAppliesNoProRataDisallowance,
    NonSinglePremiumNoSystematicBorrowingNoDisallowance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub contract_type: ContractType,
    pub exception_status: ExceptionStatus,
    pub interest_expense_cents: u64,
    pub avg_unborrowed_policy_cash_value_cents: u64,
    pub avg_total_assets_cents: u64,
}

pub type Section264InsuranceInterestDisallowanceInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub disallowed_interest_cents: u64,
    pub allowed_interest_cents: u64,
    pub effective_disallowance_basis_points: u32,
    pub note: String,
}

pub type Section264InsuranceInterestDisallowanceOutput = Output;
pub type Section264InsuranceInterestDisallowanceResult = Output;

const SECTION_264C_DE_MINIMIS_THRESHOLD_CENTS: u64 = 10_000;
#[allow(dead_code)]
const SECTION_264F4_KEY_PERSON_LIMIT: u32 = 20;
#[allow(dead_code)]
const SECTION_264F4_OWNER_EMPLOYEE_THRESHOLD_PERCENT: u32 = 20;
const SECTION_264C_FOUR_OF_SEVEN_NUMERATOR: u32 = 4;
const SECTION_264C_FOUR_OF_SEVEN_DENOMINATOR: u32 = 7;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.contract_type,
        ContractType::NonSinglePremiumLifeNoSystematicBorrowing
    ) {
        return Output {
            severity: Severity::NonSinglePremiumNoSystematicBorrowingNoDisallowance,
            disallowed_interest_cents: 0,
            allowed_interest_cents: input.interest_expense_cents,
            effective_disallowance_basis_points: 0,
            note: format!(
                "§ 264 inapplicable: non-single-premium life/endowment/annuity contract with \
                 no plan of systematic borrowing against cash value. § 264(a)(2) reaches only \
                 single-premium contracts; § 264(a)(3) reaches only systematic-borrowing \
                 plans; § 264(f) reaches only business-owned-life-insurance pro-rata. Interest \
                 expense ${} deductible (subject to § 163(j) business-interest limit and \
                 § 265 if any tax-exempt income generated).",
                input.interest_expense_cents / 100
            ),
        };
    }

    if matches!(
        input.contract_type,
        ContractType::SinglePremiumLifeEndowmentOrAnnuity
    ) {
        return Output {
            severity: Severity::Section264A2SinglePremiumInterestDisallowedPerSe,
            disallowed_interest_cents: input.interest_expense_cents,
            allowed_interest_cents: 0,
            effective_disallowance_basis_points: 10_000,
            note: format!(
                "§ 264(a)(2) per se disallowance applies. Interest on indebtedness incurred \
                 or continued to purchase or carry a SINGLE-PREMIUM life, endowment, or \
                 annuity contract is non-deductible REGARDLESS of business purpose. Single- \
                 premium definition: substantially all premiums paid within 4 years OR amount \
                 deposited with insurer for substantial future premium payments. Interest \
                 expense ${} disallowed in full. No carry-forward — disallowance is permanent. \
                 The § 264(c) exceptions DO NOT apply to single-premium contracts (only to \
                 § 264(a)(3) systematic-borrowing regime).",
                input.interest_expense_cents / 100
            ),
        };
    }

    if matches!(
        input.contract_type,
        ContractType::NonSinglePremiumLifeWithSystematicBorrowingPlan
    ) {
        match input.exception_status {
            ExceptionStatus::FourOfSevenExceptionSatisfied => {
                return Output {
                    severity: Severity::Section264cFourOfSevenExceptionPreservesDeduction,
                    disallowed_interest_cents: 0,
                    allowed_interest_cents: input.interest_expense_cents,
                    effective_disallowance_basis_points: 0,
                    note: format!(
                        "§ 264(c)(1) 4-of-7 exception SATISFIED. No part of {SECTION_264C_FOUR_OF_SEVEN_NUMERATOR} of the first \
                         {SECTION_264C_FOUR_OF_SEVEN_DENOMINATOR} annual premiums was paid through indebtedness; § 264(a)(3) \
                         systematic-borrowing disallowance does not apply. Interest expense \
                         ${} deductible. Document the premium-funding source for each of the \
                         first 7 policy years; the 4-of-7 exception requires PROOF of 4 years' \
                         worth of non-debt premium funding.",
                        input.interest_expense_cents / 100
                    ),
                };
            }
            ExceptionStatus::TradeOrBusinessExceptionSatisfied => {
                return Output {
                    severity: Severity::Section264cTradeOrBusinessExceptionPreservesDeduction,
                    disallowed_interest_cents: 0,
                    allowed_interest_cents: input.interest_expense_cents,
                    effective_disallowance_basis_points: 0,
                    note: format!(
                        "§ 264(c)(2) trade-or-business exception SATISFIED. Debt incurred in \
                         connection with the taxpayer's trade or business unrelated to the \
                         insurance policy. Interest expense ${} deductible. Document the \
                         business-purpose connection with contemporaneous evidence — IRS \
                         contests this exception aggressively where the debt has any nexus \
                         with the policy financing.",
                        input.interest_expense_cents / 100
                    ),
                };
            }
            ExceptionStatus::UnforeseenLossExceptionSatisfied => {
                return Output {
                    severity: Severity::Section264cUnforeseenLossExceptionPreservesDeduction,
                    disallowed_interest_cents: 0,
                    allowed_interest_cents: input.interest_expense_cents,
                    effective_disallowance_basis_points: 0,
                    note: format!(
                        "§ 264(c)(3) unforeseen-loss exception SATISFIED. Substantial loss of \
                         income or increase in financial obligation not foreseeable at policy \
                         purchase. Interest expense ${} deductible. Document the unforeseen \
                         event with contemporaneous records (medical diagnosis, casualty \
                         report, divorce decree, bankruptcy filing).",
                        input.interest_expense_cents / 100
                    ),
                };
            }
            ExceptionStatus::DeMinimisInterestUnderHundredDollarsExceptionSatisfied
                if input.interest_expense_cents <= SECTION_264C_DE_MINIMIS_THRESHOLD_CENTS =>
            {
                return Output {
                    severity: Severity::Section264cDeMinimisInterestExceptionPreservesDeduction,
                    disallowed_interest_cents: 0,
                    allowed_interest_cents: input.interest_expense_cents,
                    effective_disallowance_basis_points: 0,
                    note: format!(
                        "§ 264(c)(4) de minimis interest exception SATISFIED. Interest paid \
                         (${}) does not exceed the $100 annual threshold. Interest expense \
                         fully deductible.",
                        input.interest_expense_cents / 100
                    ),
                };
            }
            _ => {}
        }
        return Output {
            severity: Severity::Section264A3SystematicBorrowingInterestDisallowed,
            disallowed_interest_cents: input.interest_expense_cents,
            allowed_interest_cents: 0,
            effective_disallowance_basis_points: 10_000,
            note: format!(
                "§ 264(a)(3) systematic-borrowing disallowance applies. Interest expense ${} \
                 disallowed. Plan of systematic direct or indirect borrowing against cash \
                 value of life, endowment, or annuity contract. Disallowance is permanent — \
                 no carry-forward. Consider whether any § 264(c) exception applies: 4-of-7 \
                 (§ 264(c)(1)), trade-or-business (§ 264(c)(2)), unforeseen-loss (§ 264(c)(3)), \
                 de-minimis-interest under $100 (§ 264(c)(4)).",
                input.interest_expense_cents / 100
            ),
        };
    }

    apply_section_264f_business_owned_life_insurance(input)
}

fn apply_section_264f_business_owned_life_insurance(input: &Input) -> Output {
    if matches!(
        input.exception_status,
        ExceptionStatus::OwnerEmployeeTwentyPctPolicySection264f4Exception
            | ExceptionStatus::KeyPersonPolicyUpToTwentyIndividualsSection264f4Exception
    ) {
        let exception_label = match input.exception_status {
            ExceptionStatus::OwnerEmployeeTwentyPctPolicySection264f4Exception => {
                "§ 264(f)(4)(A) owner-employee exception — 20%+ owner-employee policy"
            }
            ExceptionStatus::KeyPersonPolicyUpToTwentyIndividualsSection264f4Exception => {
                "§ 264(f)(4)(E) key-person exception — limited to 20 individuals"
            }
            _ => unreachable!(),
        };
        return Output {
            severity: Severity::Section264f4ExceptionAppliesNoProRataDisallowance,
            disallowed_interest_cents: 0,
            allowed_interest_cents: input.interest_expense_cents,
            effective_disallowance_basis_points: 0,
            note: format!(
                "§ 264(f) pro-rata disallowance EXEMPTED by {}. Interest expense ${} fully \
                 deductible (subject to § 163(j) business-interest limit). Coordinate with \
                 § 101(j) employer-owned life insurance (EOLI) notice + consent requirements \
                 enacted by Pension Protection Act of 2006.",
                exception_label,
                input.interest_expense_cents / 100
            ),
        };
    }

    let pro_rata_ratio_bps = if input.avg_total_assets_cents == 0 {
        0u128
    } else {
        u128::from(input.avg_unborrowed_policy_cash_value_cents)
            .saturating_mul(10_000)
            .saturating_div(u128::from(input.avg_total_assets_cents))
            .min(10_000)
    };

    let disallowed = u128::from(input.interest_expense_cents)
        .saturating_mul(pro_rata_ratio_bps)
        .saturating_div(10_000);
    let disallowed_u64 = u64::try_from(disallowed).unwrap_or(u64::MAX);
    let allowed = input.interest_expense_cents.saturating_sub(disallowed_u64);
    let effective_bps = u32::try_from(pro_rata_ratio_bps).unwrap_or(0);

    Output {
        severity: Severity::Section264fProRataInterestDisallowanceApplied,
        disallowed_interest_cents: disallowed_u64,
        allowed_interest_cents: allowed,
        effective_disallowance_basis_points: effective_bps,
        note: format!(
            "§ 264(f) pro-rata interest disallowance applied. Formula: disallowance = total \
             interest × (avg unborrowed policy cash value / avg total assets). Pro-rata ratio \
             = ${} / ${} = {} bps. Interest ${}: ${} disallowed, ${} allowed. Coordinates with \
             § 163(j) business-interest limit (separate cap on remaining interest), § 101(j) \
             EOLI notice + consent requirements (Pension Protection Act of 2006), § 265 \
             tax-exempt-income interest disallowance, § 246A debt-financed-portfolio-stock \
             DRD reduction.",
            input.avg_unborrowed_policy_cash_value_cents / 100,
            input.avg_total_assets_cents / 100,
            effective_bps,
            input.interest_expense_cents / 100,
            disallowed_u64 / 100,
            allowed / 100
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            contract_type: ContractType::SinglePremiumLifeEndowmentOrAnnuity,
            exception_status: ExceptionStatus::NoExceptionInvoked,
            interest_expense_cents: 50_000_00,
            avg_unborrowed_policy_cash_value_cents: 0,
            avg_total_assets_cents: 0,
        }
    }

    #[test]
    fn single_premium_contract_interest_per_se_disallowed() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section264A2SinglePremiumInterestDisallowedPerSe
        );
        assert_eq!(output.disallowed_interest_cents, 50_000_00);
        assert_eq!(output.allowed_interest_cents, 0);
        assert!(output.note.contains("§ 264(a)(2)"));
        assert!(output.note.contains("permanent"));
    }

    #[test]
    fn non_single_premium_no_systematic_borrowing_no_disallowance() {
        let mut input = base();
        input.contract_type = ContractType::NonSinglePremiumLifeNoSystematicBorrowing;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NonSinglePremiumNoSystematicBorrowingNoDisallowance
        );
        assert_eq!(output.allowed_interest_cents, 50_000_00);
        assert!(output.note.contains("§ 163(j)"));
    }

    #[test]
    fn systematic_borrowing_no_exception_disallowance_applies() {
        let mut input = base();
        input.contract_type = ContractType::NonSinglePremiumLifeWithSystematicBorrowingPlan;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section264A3SystematicBorrowingInterestDisallowed
        );
        assert_eq!(output.disallowed_interest_cents, 50_000_00);
        assert!(output.note.contains("§ 264(a)(3)"));
        assert!(output.note.contains("§ 264(c)"));
    }

    #[test]
    fn four_of_seven_exception_preserves_deduction() {
        let mut input = base();
        input.contract_type = ContractType::NonSinglePremiumLifeWithSystematicBorrowingPlan;
        input.exception_status = ExceptionStatus::FourOfSevenExceptionSatisfied;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section264cFourOfSevenExceptionPreservesDeduction
        );
        assert_eq!(output.allowed_interest_cents, 50_000_00);
        assert!(output.note.contains("§ 264(c)(1)"));
        assert!(output.note.contains("4 of the first 7"));
    }

    #[test]
    fn trade_or_business_exception_preserves_deduction() {
        let mut input = base();
        input.contract_type = ContractType::NonSinglePremiumLifeWithSystematicBorrowingPlan;
        input.exception_status = ExceptionStatus::TradeOrBusinessExceptionSatisfied;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section264cTradeOrBusinessExceptionPreservesDeduction
        );
        assert!(output.note.contains("§ 264(c)(2)"));
    }

    #[test]
    fn unforeseen_loss_exception_preserves_deduction() {
        let mut input = base();
        input.contract_type = ContractType::NonSinglePremiumLifeWithSystematicBorrowingPlan;
        input.exception_status = ExceptionStatus::UnforeseenLossExceptionSatisfied;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section264cUnforeseenLossExceptionPreservesDeduction
        );
        assert!(output.note.contains("§ 264(c)(3)"));
    }

    #[test]
    fn de_minimis_interest_under_100_preserves_deduction() {
        let mut input = base();
        input.contract_type = ContractType::NonSinglePremiumLifeWithSystematicBorrowingPlan;
        input.exception_status =
            ExceptionStatus::DeMinimisInterestUnderHundredDollarsExceptionSatisfied;
        input.interest_expense_cents = 99_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section264cDeMinimisInterestExceptionPreservesDeduction
        );
        assert!(output.note.contains("§ 264(c)(4)"));
        assert!(output.note.contains("$100 annual threshold"));
    }

    #[test]
    fn de_minimis_exception_over_100_falls_through_to_disallowance() {
        let mut input = base();
        input.contract_type = ContractType::NonSinglePremiumLifeWithSystematicBorrowingPlan;
        input.exception_status =
            ExceptionStatus::DeMinimisInterestUnderHundredDollarsExceptionSatisfied;
        input.interest_expense_cents = 200_00;
        let output = check(&input);
        // Interest exceeds threshold → fall-through to § 264(a)(3) disallowance
        assert_eq!(
            output.severity,
            Severity::Section264A3SystematicBorrowingInterestDisallowed
        );
    }

    #[test]
    fn section_264f_pro_rata_basic_calculation() {
        let mut input = base();
        input.contract_type = ContractType::BusinessOwnedLifeInsuranceWithUnborrowedCashValue;
        input.avg_unborrowed_policy_cash_value_cents = 10_000_000_00;
        input.avg_total_assets_cents = 100_000_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section264fProRataInterestDisallowanceApplied
        );
        // $10M / $100M = 10% = 1,000 bps disallowance
        // $50K × 10% = $5K disallowed
        assert_eq!(output.effective_disallowance_basis_points, 1_000);
        assert_eq!(output.disallowed_interest_cents, 5_000_00);
        assert_eq!(output.allowed_interest_cents, 45_000_00);
        assert!(output.note.contains("§ 264(f)"));
    }

    #[test]
    fn section_264f4_owner_employee_exception_no_disallowance() {
        let mut input = base();
        input.contract_type = ContractType::BusinessOwnedLifeInsuranceWithUnborrowedCashValue;
        input.exception_status = ExceptionStatus::OwnerEmployeeTwentyPctPolicySection264f4Exception;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section264f4ExceptionAppliesNoProRataDisallowance
        );
        assert_eq!(output.allowed_interest_cents, 50_000_00);
        assert!(output.note.contains("§ 264(f)(4)(A)"));
        assert!(output.note.contains("§ 101(j)"));
    }

    #[test]
    fn section_264f4_key_person_exception_no_disallowance() {
        let mut input = base();
        input.contract_type = ContractType::BusinessOwnedLifeInsuranceWithUnborrowedCashValue;
        input.exception_status =
            ExceptionStatus::KeyPersonPolicyUpToTwentyIndividualsSection264f4Exception;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section264f4ExceptionAppliesNoProRataDisallowance
        );
        assert!(output.note.contains("§ 264(f)(4)(E)"));
        assert!(output.note.contains("20 individuals"));
    }

    #[test]
    fn section_264f_zero_total_assets_no_panic_safe_division() {
        let mut input = base();
        input.contract_type = ContractType::BusinessOwnedLifeInsuranceWithUnborrowedCashValue;
        input.avg_unborrowed_policy_cash_value_cents = 10_000_000_00;
        input.avg_total_assets_cents = 0;
        let output = check(&input);
        assert_eq!(output.disallowed_interest_cents, 0);
    }

    #[test]
    fn section_264f_100_pct_unborrowed_caps_at_total_assets() {
        let mut input = base();
        input.contract_type = ContractType::BusinessOwnedLifeInsuranceWithUnborrowedCashValue;
        input.avg_unborrowed_policy_cash_value_cents = 100_000_000_00;
        input.avg_total_assets_cents = 100_000_000_00;
        let output = check(&input);
        // 100% ratio = full disallowance
        assert_eq!(output.effective_disallowance_basis_points, 10_000);
        assert_eq!(output.disallowed_interest_cents, 50_000_00);
        assert_eq!(output.allowed_interest_cents, 0);
    }

    #[test]
    fn section_264c_de_minimis_threshold_constant_pins_100_dollars() {
        assert_eq!(SECTION_264C_DE_MINIMIS_THRESHOLD_CENTS, 10_000);
    }

    #[test]
    fn section_264f4_key_person_limit_pins_20() {
        assert_eq!(SECTION_264F4_KEY_PERSON_LIMIT, 20);
    }

    #[test]
    fn section_264f4_owner_employee_threshold_pins_20_pct() {
        assert_eq!(SECTION_264F4_OWNER_EMPLOYEE_THRESHOLD_PERCENT, 20);
    }

    #[test]
    fn section_264c_four_of_seven_constants_pin_4_and_7() {
        assert_eq!(SECTION_264C_FOUR_OF_SEVEN_NUMERATOR, 4);
        assert_eq!(SECTION_264C_FOUR_OF_SEVEN_DENOMINATOR, 7);
    }

    #[test]
    fn very_large_interest_no_overflow() {
        let mut input = base();
        input.interest_expense_cents = u64::MAX;
        let output = check(&input);
        // Single-premium → full disallowance regardless
        assert_eq!(output.disallowed_interest_cents, u64::MAX);
    }

    #[test]
    fn zero_interest_no_disallowance() {
        let mut input = base();
        input.interest_expense_cents = 0;
        let output = check(&input);
        assert_eq!(output.disallowed_interest_cents, 0);
        assert_eq!(output.allowed_interest_cents, 0);
    }

    #[test]
    fn note_pins_265_tax_exempt_income_disallowance() {
        let mut input = base();
        input.contract_type = ContractType::BusinessOwnedLifeInsuranceWithUnborrowedCashValue;
        input.avg_unborrowed_policy_cash_value_cents = 1_000_000_00;
        input.avg_total_assets_cents = 10_000_000_00;
        let output = check(&input);
        assert!(output.note.contains("§ 265"));
    }

    #[test]
    fn note_pins_101j_eoli_notice_consent_pension_protection_act_2006() {
        let mut input = base();
        input.contract_type = ContractType::BusinessOwnedLifeInsuranceWithUnborrowedCashValue;
        input.exception_status = ExceptionStatus::OwnerEmployeeTwentyPctPolicySection264f4Exception;
        let output = check(&input);
        assert!(output.note.contains("§ 101(j)"));
        assert!(output.note.contains("Pension Protection Act of 2006"));
    }

    #[test]
    fn single_premium_takes_priority_over_systematic_borrowing_exception() {
        let mut input = base();
        input.contract_type = ContractType::SinglePremiumLifeEndowmentOrAnnuity;
        input.exception_status = ExceptionStatus::FourOfSevenExceptionSatisfied;
        let output = check(&input);
        // Single-premium per se rule applies; § 264(c) exceptions don't help
        assert_eq!(
            output.severity,
            Severity::Section264A2SinglePremiumInterestDisallowedPerSe
        );
    }
}
