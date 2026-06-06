//! IRC § 1258 — Recharacterization of Gain from Certain
//! Financial Transactions / Conversion Transactions Anti-Abuse
//! Module.
//!
//! Pure-compute check for IRC § 1258 conversion-transaction
//! recharacterization. § 1258 prevents traders from converting
//! what would otherwise be ordinary interest-equivalent income
//! into long-term capital gain via structured straddle, buy-
//! and-forward-sale, or similar time-value-dominant
//! transactions. Trader / hedge-fund critical because § 1258
//! converts the lesser of (a) capital gain on disposition or
//! (b) **applicable imputed income amount** computed at 120 %
//! of the applicable federal rate to ordinary income — caps
//! the preferential rate available on time-value-dominant
//! financial positions.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 1258(a) General Rule**: any gain that would
//!   otherwise be treated as gain from sale or exchange of a
//!   capital asset and is recognized on the disposition or
//!   other termination of any position held as part of a
//!   **CONVERSION TRANSACTION**, to the extent such gain does
//!   not exceed the **APPLICABLE IMPUTED INCOME AMOUNT**,
//!   shall be treated as **ORDINARY INCOME** ([Cornell LII 26
//!   USC § 1258](https://www.law.cornell.edu/uscode/text/26/1258);
//!   [Bloomberg Tax Sec. 1258](https://irc.bloombergtax.com/public/uscode/doc/irc/section_1258)).
//! - **IRC § 1258(b) Applicable Imputed Income Amount**:
//!   amount equal to interest that would have accrued on
//!   taxpayer's **NET INVESTMENT** in the conversion
//!   transaction for the period ending on disposition (or
//!   earlier disqualifying date) at a rate equal to **120
//!   PERCENT of the applicable rate**, reduced by amounts
//!   treated as ordinary income on prior dispositions, amounts
//!   capitalized under § 263(g), ordinary income previously
//!   received, or otherwise.
//! - **IRC § 1258(c) Conversion Transaction Definition** —
//!   transaction satisfying BOTH (1) substantially all of
//!   taxpayer's expected return is attributable to **TIME
//!   VALUE** of taxpayer's net investment in the transaction,
//!   AND (2) transaction falls within one of **FOUR
//!   ENUMERATED CATEGORIES** under § 1258(c)(2):
//!   - **§ 1258(c)(2)(A)**: applicable straddle within meaning
//!     of § 1092(c)
//!   - **§ 1258(c)(2)(B)**: buy and forward sale (acquisition
//!     of property + substantially contemporaneous agreement
//!     to sell same or substantially identical property at
//!     price determined under such agreement)
//!   - **§ 1258(c)(2)(C)**: marketed/sold under marketing
//!     materials promising overall after-tax economic profit
//!     attributable to capital-gain conversion
//!   - **§ 1258(c)(2)(D)**: other transactions specified by
//!     regulations under § 1258(c)(2)(D)
//! - **IRC § 1258(d) Applicable Rate**: standard case = the
//!   **Applicable Federal Rate (AFR) under § 1274(d)** for the
//!   term of the conversion transaction, **compounded semi-
//!   annually**; indefinite term = **Federal short-term rate
//!   under § 6621(b)**, **compounded daily** ([CCH AnswerConnect
//!   § 1258(c) Conversion Transaction Analysis](https://answerconnect.cch.com/document/arp1209013e2c83dc65bbSPLIT1258c/federal/irc/current/conversion-transaction);
//!   [The Retirement Group — Anti-Conversion Rule](https://www.theretirementgroup.com/blog/anti-conversion-rule)).
//! - **Treas. Reg. § 1.1258-1 NETTING RULE**: implementing
//!   regulation for netting prior ordinary income (from § 1258
//!   recharacterization) against subsequent gain on related
//!   conversion-transaction positions ([26 CFR § 1.1258-1](https://www.law.cornell.edu/cfr/text/26/1.1258-1)).
//! - **Enacted**: Omnibus Budget Reconciliation Act of 1993
//!   (Public Law 103-66), § 13206 — added § 1258 in response
//!   to growing use of time-value-based "tax-favored" capital-
//!   gain conversion structures (Federal Register 60 FR 65548;
//!   December 21, 1995 final regs).
//! - **Trader / Hedge-Fund Examples**: marketed box spreads
//!   structured to produce LTCG on residual time-value;
//!   buy-stock-plus-short-forward synthetic loans; calendar
//!   spreads with dominant time-decay leg; "deep-in-the-money"
//!   covered-call structures designed to surrender minimal
//!   directional exposure for fixed time-value pickup.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_1258_RATE_MULTIPLIER_PCT_BASIS_POINTS: u64 = 12_000;
pub const IRC_1258_RATE_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_1258_CONVERSION_CATEGORIES_COUNT: u32 = 4;
pub const IRC_1258_OBRA_1993_PL_NUMBER: u32 = 103_066;
pub const IRC_1258_OBRA_1993_SECTION: u32 = 13_206;
pub const TREAS_REG_1258_FINAL_DATE_YEAR: u32 = 1995;
pub const TREAS_REG_1258_FINAL_DATE_MONTH: u32 = 12;
pub const TREAS_REG_1258_FINAL_DATE_DAY: u32 = 21;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversionTransactionCategory {
    Section1258C2AApplicableStraddleUnderSection1092C,
    Section1258C2BBuyAndForwardSale,
    Section1258C2CMarketedAsCapitalGainConversion,
    Section1258C2DOtherTransactionsSpecifiedByRegulations,
    NotAConversionTransaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicableRateType {
    StandardCaseAfrSection1274dCompoundedSemiannually,
    IndefiniteTermFederalShortTermRateSection6621bCompoundedDaily,
    NoApplicableRateNotConversionTransaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1258Mode {
    NotApplicableNoConversionTransactionTimeValueNotDominant,
    NotApplicableNoConversionTransactionCategoryMet,
    NotApplicableNoGainOnDisposition,
    CompliantSection1258ARecharacterizesGainAsOrdinaryWithinImputedAmount,
    CompliantSection1258AGainExceedsImputedAmountResidualCapitalGain,
    CompliantNettingRuleTreasReg1258_1AppliedReducingImputedAmount,
    ViolationSection1258AImputedAmountNotRecharacterizedAsOrdinary,
    ViolationConversionTransactionCategoryC2BBuyAndForwardSaleNotReportedAsOrdinary,
    ViolationApplicableRateMisidentified,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub time_value_substantially_dominant_expected_return: bool,
    pub conversion_transaction_category: ConversionTransactionCategory,
    pub applicable_rate_type: ApplicableRateType,
    pub gain_recognized_on_disposition_cents: u64,
    pub net_investment_cents: u64,
    pub applicable_rate_basis_points: u64,
    pub holding_period_days: u32,
    pub prior_ordinary_income_under_netting_rule_cents: u64,
    pub section_263g_capitalized_amount_cents: u64,
    pub other_ordinary_income_received_cents: u64,
    pub taxpayer_treated_gain_as_ordinary: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1258Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub applicable_imputed_income_amount_cents: u64,
    pub ordinary_income_recharacterization_cents: u64,
    pub residual_capital_gain_cents: u64,
}

pub type Section1258Input = Input;
pub type Section1258Output = Output;
pub type Section1258Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 1258(a) — gain on disposition of position held as part of CONVERSION TRANSACTION treated as ORDINARY INCOME to extent of APPLICABLE IMPUTED INCOME AMOUNT".to_string(),
        "IRC § 1258(b) — applicable imputed income amount = interest accrued on net investment at 120 % of applicable rate, reduced by prior ordinary income, § 263(g) capitalized amounts, and other ordinary income received".to_string(),
        "IRC § 1258(c) — CONVERSION TRANSACTION definition: (1) substantially all expected return attributable to TIME VALUE of net investment, AND (2) transaction falls within one of four enumerated categories".to_string(),
        "IRC § 1258(c)(2)(A) — applicable straddle within meaning of § 1092(c)".to_string(),
        "IRC § 1258(c)(2)(B) — buy and forward sale (acquisition of property + substantially contemporaneous agreement to sell same or substantially identical property at price determined under such agreement)".to_string(),
        "IRC § 1258(c)(2)(C) — marketed/sold under marketing materials promising overall after-tax economic profit attributable to capital-gain conversion".to_string(),
        "IRC § 1258(c)(2)(D) — other transactions specified by Treasury regulations".to_string(),
        "IRC § 1258(d) — applicable rate: standard case = Applicable Federal Rate (AFR) under § 1274(d) for term of conversion transaction, compounded semiannually; indefinite term = Federal short-term rate under § 6621(b), compounded daily".to_string(),
        "Treas. Reg. § 1.1258-1 — NETTING RULE for certain conversion transactions; allows netting prior ordinary income against subsequent gain on related positions".to_string(),
        "Enacted by Omnibus Budget Reconciliation Act of 1993 (Public Law 103-66) § 13206 — added § 1258 in response to growing use of time-value-based capital-gain conversion structures".to_string(),
        "Federal Register 60 FR 65548 (December 21, 1995) — final Treas. Reg. § 1.1258-1 implementing regulations".to_string(),
        "IRC § 1092(c) — straddle definition cross-reference for § 1258(c)(2)(A) applicable straddle".to_string(),
        "IRC § 263(g) — capitalization of interest and carrying charges in straddles (reduces applicable imputed income amount)".to_string(),
        "IRC § 1274(d) — applicable federal rate for § 1258(d) standard case".to_string(),
        "IRC § 6621(b) — federal short-term rate for § 1258(d) indefinite-term case".to_string(),
        "Cornell LII 26 USC § 1258 — primary statutory text".to_string(),
        "CCH AnswerConnect — § 1258(c) Conversion Transaction practitioner guide".to_string(),
        "Bloomberg Tax Sec. 1258 — comprehensive code commentary".to_string(),
    ];

    if input.gain_recognized_on_disposition_cents == 0 {
        return Output {
            mode: Section1258Mode::NotApplicableNoGainOnDisposition,
            statutory_basis: "IRC § 1258 — recharacterizes only realized gain on disposition".to_string(),
            notes: "NOT APPLICABLE: no gain recognized on disposition; § 1258 recharacterizes only realized capital gain.".to_string(),
            citations,
            applicable_imputed_income_amount_cents: 0,
            ordinary_income_recharacterization_cents: 0,
            residual_capital_gain_cents: 0,
        };
    }

    if !input.time_value_substantially_dominant_expected_return {
        return Output {
            mode: Section1258Mode::NotApplicableNoConversionTransactionTimeValueNotDominant,
            statutory_basis: "IRC § 1258(c)(1) — time value must substantially dominate expected return".to_string(),
            notes: "NOT APPLICABLE: § 1258(c)(1) requires substantially all of taxpayer's expected return to be attributable to time value of net investment; this transaction does not meet that requirement.".to_string(),
            citations,
            applicable_imputed_income_amount_cents: 0,
            ordinary_income_recharacterization_cents: 0,
            residual_capital_gain_cents: input.gain_recognized_on_disposition_cents,
        };
    }

    if input.conversion_transaction_category
        == ConversionTransactionCategory::NotAConversionTransaction
    {
        return Output {
            mode: Section1258Mode::NotApplicableNoConversionTransactionCategoryMet,
            statutory_basis: "IRC § 1258(c)(2) — transaction must fall within one of four enumerated categories".to_string(),
            notes: "NOT APPLICABLE: § 1258(c)(2) requires transaction to fall within one of four enumerated categories (applicable straddle, buy-and-forward-sale, marketed capital-gain-conversion, or other regs); this transaction is not within any category.".to_string(),
            citations,
            applicable_imputed_income_amount_cents: 0,
            ordinary_income_recharacterization_cents: 0,
            residual_capital_gain_cents: input.gain_recognized_on_disposition_cents,
        };
    }

    if input.applicable_rate_type == ApplicableRateType::NoApplicableRateNotConversionTransaction {
        return Output {
            mode: Section1258Mode::ViolationApplicableRateMisidentified,
            statutory_basis: "IRC § 1258(d) — applicable rate must be § 1274(d) AFR or § 6621(b) short-term rate".to_string(),
            notes: "VIOLATION: applicable rate misidentified; § 1258(d) requires use of § 1274(d) AFR for standard-term conversion transaction or § 6621(b) federal short-term rate for indefinite-term conversion transaction.".to_string(),
            citations,
            applicable_imputed_income_amount_cents: 0,
            ordinary_income_recharacterization_cents: 0,
            residual_capital_gain_cents: 0,
        };
    }

    let imputed_interest_at_120pct = input
        .net_investment_cents
        .saturating_mul(input.applicable_rate_basis_points)
        .saturating_mul(IRC_1258_RATE_MULTIPLIER_PCT_BASIS_POINTS)
        .saturating_mul(u64::from(input.holding_period_days))
        / (IRC_1258_RATE_BASIS_POINT_DENOMINATOR
            .saturating_mul(IRC_1258_RATE_BASIS_POINT_DENOMINATOR)
            .saturating_mul(365));

    let reductions = input
        .prior_ordinary_income_under_netting_rule_cents
        .saturating_add(input.section_263g_capitalized_amount_cents)
        .saturating_add(input.other_ordinary_income_received_cents);

    let applicable_imputed_income_amount_cents =
        imputed_interest_at_120pct.saturating_sub(reductions);

    let ordinary_income_recharacterization_cents = input
        .gain_recognized_on_disposition_cents
        .min(applicable_imputed_income_amount_cents);

    let residual_capital_gain_cents = input
        .gain_recognized_on_disposition_cents
        .saturating_sub(ordinary_income_recharacterization_cents);

    if !input.taxpayer_treated_gain_as_ordinary && ordinary_income_recharacterization_cents > 0 {
        if matches!(
            input.conversion_transaction_category,
            ConversionTransactionCategory::Section1258C2BBuyAndForwardSale
        ) {
            return Output {
                mode: Section1258Mode::ViolationConversionTransactionCategoryC2BBuyAndForwardSaleNotReportedAsOrdinary,
                statutory_basis: "IRC § 1258(c)(2)(B) buy-and-forward-sale category".to_string(),
                notes: format!(
                    "VIOLATION: § 1258(c)(2)(B) buy-and-forward-sale conversion transaction generated {} cents of capital gain but {} cents of applicable imputed income amount should be recharacterized as ordinary income.",
                    input.gain_recognized_on_disposition_cents, applicable_imputed_income_amount_cents
                ),
                citations,
                applicable_imputed_income_amount_cents,
                ordinary_income_recharacterization_cents,
                residual_capital_gain_cents,
            };
        }
        return Output {
            mode: Section1258Mode::ViolationSection1258AImputedAmountNotRecharacterizedAsOrdinary,
            statutory_basis: "IRC § 1258(a) — applicable imputed income amount must be treated as ordinary income".to_string(),
            notes: format!(
                "VIOLATION: § 1258(a) requires {} cents of conversion-transaction gain recharacterized as ordinary income (applicable imputed income amount at 120 % of applicable rate); taxpayer treated entire {} cents as capital gain.",
                ordinary_income_recharacterization_cents, input.gain_recognized_on_disposition_cents
            ),
            citations,
            applicable_imputed_income_amount_cents,
            ordinary_income_recharacterization_cents,
            residual_capital_gain_cents,
        };
    }

    if input.prior_ordinary_income_under_netting_rule_cents > 0 {
        return Output {
            mode: Section1258Mode::CompliantNettingRuleTreasReg1258_1AppliedReducingImputedAmount,
            statutory_basis: "Treas. Reg. § 1.1258-1 — netting rule reduces applicable imputed income amount".to_string(),
            notes: format!(
                "COMPLIANT: Treas. Reg. § 1.1258-1 netting rule applied; {} cents of prior ordinary income under conversion-transaction netting reduces applicable imputed income amount to {} cents; {} cents recharacterized as ordinary, {} cents residual capital gain.",
                input.prior_ordinary_income_under_netting_rule_cents,
                applicable_imputed_income_amount_cents,
                ordinary_income_recharacterization_cents,
                residual_capital_gain_cents
            ),
            citations,
            applicable_imputed_income_amount_cents,
            ordinary_income_recharacterization_cents,
            residual_capital_gain_cents,
        };
    }

    if input.gain_recognized_on_disposition_cents > applicable_imputed_income_amount_cents {
        return Output {
            mode: Section1258Mode::CompliantSection1258AGainExceedsImputedAmountResidualCapitalGain,
            statutory_basis: "IRC § 1258(a) — gain exceeds applicable imputed income amount; residual capital gain".to_string(),
            notes: format!(
                "COMPLIANT: gain of {} cents exceeds applicable imputed income amount of {} cents; {} cents recharacterized as ordinary income under § 1258(a); residual {} cents retains capital gain character.",
                input.gain_recognized_on_disposition_cents,
                applicable_imputed_income_amount_cents,
                ordinary_income_recharacterization_cents,
                residual_capital_gain_cents
            ),
            citations,
            applicable_imputed_income_amount_cents,
            ordinary_income_recharacterization_cents,
            residual_capital_gain_cents,
        };
    }

    Output {
        mode: Section1258Mode::CompliantSection1258ARecharacterizesGainAsOrdinaryWithinImputedAmount,
        statutory_basis: "IRC § 1258(a) — entire gain recharacterized as ordinary income within applicable imputed income amount".to_string(),
        notes: format!(
            "COMPLIANT: entire {} cents of gain recharacterized as ordinary income under § 1258(a) because gain ≤ applicable imputed income amount of {} cents.",
            input.gain_recognized_on_disposition_cents, applicable_imputed_income_amount_cents
        ),
        citations,
        applicable_imputed_income_amount_cents,
        ordinary_income_recharacterization_cents,
        residual_capital_gain_cents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_conversion_transaction_compliant() -> Input {
        Input {
            time_value_substantially_dominant_expected_return: true,
            conversion_transaction_category:
                ConversionTransactionCategory::Section1258C2BBuyAndForwardSale,
            applicable_rate_type:
                ApplicableRateType::StandardCaseAfrSection1274dCompoundedSemiannually,
            gain_recognized_on_disposition_cents: 50_000,
            net_investment_cents: 1_000_000,
            applicable_rate_basis_points: 500,
            holding_period_days: 365,
            prior_ordinary_income_under_netting_rule_cents: 0,
            section_263g_capitalized_amount_cents: 0,
            other_ordinary_income_received_cents: 0,
            taxpayer_treated_gain_as_ordinary: true,
        }
    }

    #[test]
    fn no_gain_on_disposition_not_applicable() {
        let input = Input {
            gain_recognized_on_disposition_cents: 0,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::NotApplicableNoGainOnDisposition
        );
    }

    #[test]
    fn time_value_not_dominant_not_applicable() {
        let input = Input {
            time_value_substantially_dominant_expected_return: false,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::NotApplicableNoConversionTransactionTimeValueNotDominant
        );
        assert_eq!(result.residual_capital_gain_cents, 50_000);
    }

    #[test]
    fn no_conversion_category_not_applicable() {
        let input = Input {
            conversion_transaction_category:
                ConversionTransactionCategory::NotAConversionTransaction,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::NotApplicableNoConversionTransactionCategoryMet
        );
        assert_eq!(result.residual_capital_gain_cents, 50_000);
    }

    #[test]
    fn applicable_rate_misidentified_violation() {
        let input = Input {
            applicable_rate_type: ApplicableRateType::NoApplicableRateNotConversionTransaction,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::ViolationApplicableRateMisidentified
        );
    }

    #[test]
    fn buy_and_forward_sale_compliant_baseline() {
        let result = check(&baseline_conversion_transaction_compliant());
        assert_eq!(
            result.mode,
            Section1258Mode::CompliantSection1258ARecharacterizesGainAsOrdinaryWithinImputedAmount
        );
    }

    #[test]
    fn applicable_straddle_section_1258_c2_a_compliant() {
        let input = Input {
            conversion_transaction_category:
                ConversionTransactionCategory::Section1258C2AApplicableStraddleUnderSection1092C,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::CompliantSection1258ARecharacterizesGainAsOrdinaryWithinImputedAmount
        );
    }

    #[test]
    fn marketed_capital_gain_conversion_section_1258_c2_c_compliant() {
        let input = Input {
            conversion_transaction_category:
                ConversionTransactionCategory::Section1258C2CMarketedAsCapitalGainConversion,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::CompliantSection1258ARecharacterizesGainAsOrdinaryWithinImputedAmount
        );
    }

    #[test]
    fn other_regs_section_1258_c2_d_compliant() {
        let input = Input {
            conversion_transaction_category:
                ConversionTransactionCategory::Section1258C2DOtherTransactionsSpecifiedByRegulations,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::CompliantSection1258ARecharacterizesGainAsOrdinaryWithinImputedAmount
        );
    }

    #[test]
    fn indefinite_term_applicable_rate_compliant() {
        let input = Input {
            applicable_rate_type:
                ApplicableRateType::IndefiniteTermFederalShortTermRateSection6621bCompoundedDaily,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::CompliantSection1258ARecharacterizesGainAsOrdinaryWithinImputedAmount
        );
    }

    #[test]
    fn buy_and_forward_sale_violation_taxpayer_kept_capital_treatment() {
        let input = Input {
            taxpayer_treated_gain_as_ordinary: false,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::ViolationConversionTransactionCategoryC2BBuyAndForwardSaleNotReportedAsOrdinary
        );
    }

    #[test]
    fn applicable_straddle_violation_taxpayer_kept_capital_treatment() {
        let input = Input {
            conversion_transaction_category:
                ConversionTransactionCategory::Section1258C2AApplicableStraddleUnderSection1092C,
            taxpayer_treated_gain_as_ordinary: false,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::ViolationSection1258AImputedAmountNotRecharacterizedAsOrdinary
        );
    }

    #[test]
    fn gain_exceeds_imputed_amount_residual_capital_gain_compliant() {
        let input = Input {
            gain_recognized_on_disposition_cents: 200_000,
            net_investment_cents: 100_000,
            applicable_rate_basis_points: 500,
            holding_period_days: 365,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::CompliantSection1258AGainExceedsImputedAmountResidualCapitalGain
        );
        assert!(result.residual_capital_gain_cents > 0);
    }

    #[test]
    fn netting_rule_applied_compliant() {
        let input = Input {
            prior_ordinary_income_under_netting_rule_cents: 30_000,
            net_investment_cents: 2_000_000,
            applicable_rate_basis_points: 500,
            holding_period_days: 365,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section1258Mode::CompliantNettingRuleTreasReg1258_1AppliedReducingImputedAmount
        );
    }

    #[test]
    fn section_263g_capitalized_amount_reduces_imputed_income() {
        let input = Input {
            section_263g_capitalized_amount_cents: 20_000,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert!(result.applicable_imputed_income_amount_cents < 50_000);
    }

    #[test]
    fn imputed_income_calculation_at_120_pct_of_applicable_rate() {
        let input = Input {
            net_investment_cents: 1_000_000,
            applicable_rate_basis_points: 500,
            holding_period_days: 365,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert!(result.applicable_imputed_income_amount_cents > 50_000);
        assert!(result.applicable_imputed_income_amount_cents <= 60_001);
    }

    #[test]
    fn citations_pin_section_1258_subsections_and_obra_1993() {
        let result = check(&baseline_conversion_transaction_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 1258(a)"));
        assert!(joined.contains("IRC § 1258(b)"));
        assert!(joined.contains("IRC § 1258(c)"));
        assert!(joined.contains("IRC § 1258(c)(2)(A)"));
        assert!(joined.contains("IRC § 1258(c)(2)(B)"));
        assert!(joined.contains("IRC § 1258(c)(2)(C)"));
        assert!(joined.contains("IRC § 1258(c)(2)(D)"));
        assert!(joined.contains("IRC § 1258(d)"));
        assert!(joined.contains("CONVERSION TRANSACTION"));
        assert!(joined.contains("APPLICABLE IMPUTED INCOME AMOUNT"));
        assert!(joined.contains("ORDINARY INCOME"));
        assert!(joined.contains("TIME VALUE"));
        assert!(joined.contains("120 % of applicable rate"));
        assert!(joined.contains("§ 1092(c)"));
        assert!(joined.contains("§ 263(g)"));
        assert!(joined.contains("§ 1274(d)"));
        assert!(joined.contains("§ 6621(b)"));
        assert!(joined.contains("Treas. Reg. § 1.1258-1"));
        assert!(joined.contains("Omnibus Budget Reconciliation Act of 1993"));
        assert!(joined.contains("Public Law 103-66"));
        assert!(joined.contains("§ 13206"));
        assert!(joined.contains("60 FR 65548"));
        assert!(joined.contains("December 21, 1995"));
    }

    #[test]
    fn constant_pin_multiplier_categories_and_dates() {
        assert_eq!(IRC_1258_RATE_MULTIPLIER_PCT_BASIS_POINTS, 12_000);
        assert_eq!(IRC_1258_RATE_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_1258_CONVERSION_CATEGORIES_COUNT, 4);
        assert_eq!(IRC_1258_OBRA_1993_PL_NUMBER, 103_066);
        assert_eq!(IRC_1258_OBRA_1993_SECTION, 13_206);
        assert_eq!(TREAS_REG_1258_FINAL_DATE_YEAR, 1995);
        assert_eq!(TREAS_REG_1258_FINAL_DATE_MONTH, 12);
        assert_eq!(TREAS_REG_1258_FINAL_DATE_DAY, 21);
    }

    #[test]
    fn saturating_overflow_defense_extreme_inputs() {
        let input = Input {
            net_investment_cents: u64::MAX,
            applicable_rate_basis_points: u64::MAX,
            holding_period_days: u32::MAX,
            ..baseline_conversion_transaction_compliant()
        };
        let result = check(&input);
        assert!(matches!(
            result.mode,
            Section1258Mode::CompliantSection1258ARecharacterizesGainAsOrdinaryWithinImputedAmount
        ));
    }
}
