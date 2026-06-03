//! IRC § 383 special limitations on certain excess credits and capital losses.
//!
//! § 383 extends the § 382 ownership-change annual limitation regime to four categories
//! of corporate carryover attributes that fall outside § 382 itself: (1) general business
//! credits under § 39, (2) minimum tax credits under § 53, (3) net capital loss
//! carryovers under § 1212, and (4) excess foreign tax credits under § 904(c). The
//! mechanic is to compute the tax that would be attributable to taxable income equal to
//! the § 382 limitation, then cap pre-change credit/loss usage against that ceiling.
//!
//! § 383 OPERATING SEQUENCE:
//!   1. § 382(g) ownership change occurs (more-than-50-percentage-point change in stock
//!      ownership by 5-percent shareholders over the 3-year testing period).
//!   2. § 382(b) annual limitation = loss-corp FMV × federal long-term tax-exempt rate.
//!   3. Use pre-change NOLs and § 382-tier attributes first against the § 382 limit.
//!   4. § 383 LIMIT is calculated by reference to the tax that WOULD have been due on
//!      taxable income equal to the remaining (un-NOL-absorbed) § 382 limitation amount.
//!   5. Excess credits offset that "tentative" tax up to the § 383 limit; unused excess
//!      credits carry forward (with sequential annual reset of the § 383 limit).
//!
//! § 383(a) GENERAL BUSINESS CREDIT + MINIMUM TAX CREDIT regime: post-change use limited
//! to tax attributable to income within § 382 limit.
//!
//! § 383(b) NET CAPITAL LOSS CARRYOVER: § 1212(a) carryover limited under regulations
//! based on § 382 principles. Capital loss used in post-change year REDUCES the § 382
//! limitation for that year applied to pre-change NOLs (anti-stuffing rule).
//!
//! § 383(c) EXCESS FOREIGN TAX CREDIT: § 904(c) excess FTC carryover limited consistent
//! with § 382 + § 383 principles.
//!
//! § 383(d) ORDERING: terms used in § 383 have same meaning as § 382, with appropriate
//! adjustments to apply to credits and capital losses.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/383
//! - law.cornell.edu/cfr/text/26/1.383-1
//! - irc.bloombergtax.com/public/uscode/doc/irc/section_383
//! - centriconsulting.com/news/insights/key-tax-insights-for-loss-corporations-navigating-sections-382-383-and-384/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipChangeStatus {
    OwnershipChangeOccurredSection382G,
    NoOwnershipChangeNoLimitationTriggered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeType {
    /// § 39 general business credit carryforward.
    Section39GeneralBusinessCredit,
    /// § 53 minimum tax credit carryforward.
    Section53MinimumTaxCredit,
    /// § 1212(a) corporate net capital loss carryover.
    Section1212NetCapitalLossCarryover,
    /// § 904(c) excess foreign tax credit carryover.
    Section904ExcessForeignTaxCreditCarryover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoOwnershipChangeFullAttributeUsageAllowed,
    Section383CreditLimitationApplied,
    Section383BCapitalLossLimitationAppliedAntiStuffing,
    Section383CExcessFtcLimitationApplied,
    AttributeFullyUsedWithinSection383Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub ownership_change_status: OwnershipChangeStatus,
    pub attribute_type: AttributeType,
    pub section_382_annual_limitation_cents: u64,
    pub effective_corporate_tax_rate_bps: u32,
    pub pre_change_attribute_balance_cents: u64,
    pub nol_absorption_of_382_limit_cents: u64,
    pub post_change_taxable_income_cents: u64,
}

pub type Section383ExcessCreditLimitationInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub section_383_annual_credit_limit_cents: u64,
    pub allowed_attribute_usage_cents: u64,
    pub disallowed_attribute_usage_cents: u64,
    pub note: String,
}

pub type Section383ExcessCreditLimitationOutput = Output;
pub type Section383ExcessCreditLimitationResult = Output;

const SECTION_382_TESTING_PERIOD_YEARS: u32 = 3;
const SECTION_382_OWNERSHIP_CHANGE_THRESHOLD_PERCENTAGE_POINTS: u32 = 50;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.ownership_change_status,
        OwnershipChangeStatus::NoOwnershipChangeNoLimitationTriggered
    ) {
        return Output {
            severity: Severity::NoOwnershipChangeFullAttributeUsageAllowed,
            section_383_annual_credit_limit_cents: 0,
            allowed_attribute_usage_cents: input.pre_change_attribute_balance_cents.min(
                input.post_change_taxable_income_cents,
            ),
            disallowed_attribute_usage_cents: 0,
            note: format!(
                "§ 383 inapplicable: no § 382(g) ownership change. § 383 limitation triggers \
                 ONLY when ownership of more than {SECTION_382_OWNERSHIP_CHANGE_THRESHOLD_PERCENTAGE_POINTS}-percentage-point \
                 change occurs over the {SECTION_382_TESTING_PERIOD_YEARS}-year testing period \
                 by 5-percent shareholders. Pre-change attributes (${}) fully usable against \
                 post-change income (${}) subject only to ordinary tax-law limits.",
                input.pre_change_attribute_balance_cents / 100,
                input.post_change_taxable_income_cents / 100
            ),
        };
    }

    let remaining_382_limit_after_nols = input
        .section_382_annual_limitation_cents
        .saturating_sub(input.nol_absorption_of_382_limit_cents);

    let section_383_credit_limit = u128::from(remaining_382_limit_after_nols)
        .saturating_mul(u128::from(input.effective_corporate_tax_rate_bps))
        .saturating_div(10_000);
    let section_383_credit_limit_u64 =
        u64::try_from(section_383_credit_limit).unwrap_or(u64::MAX);

    match input.attribute_type {
        AttributeType::Section1212NetCapitalLossCarryover => {
            let allowed = input
                .pre_change_attribute_balance_cents
                .min(remaining_382_limit_after_nols);
            let disallowed = input
                .pre_change_attribute_balance_cents
                .saturating_sub(allowed);
            let severity = if disallowed == 0 {
                Severity::AttributeFullyUsedWithinSection383Limit
            } else {
                Severity::Section383BCapitalLossLimitationAppliedAntiStuffing
            };
            Output {
                severity,
                section_383_annual_credit_limit_cents: remaining_382_limit_after_nols,
                allowed_attribute_usage_cents: allowed,
                disallowed_attribute_usage_cents: disallowed,
                note: format!(
                    "§ 383(b) net capital loss carryover limitation applied. Pre-change § 1212 \
                     net capital loss (${}) competes for the § 382 annual limitation with \
                     pre-change NOLs. § 382 limit (${}) minus NOL absorption (${}) = ${} \
                     remaining capacity. Capital loss used in this post-change year REDUCES \
                     the § 382 limit applied to pre-change NOLs in the same year \
                     (anti-stuffing rule). Allowed ${}; disallowed (carries forward) ${}.",
                    input.pre_change_attribute_balance_cents / 100,
                    input.section_382_annual_limitation_cents / 100,
                    input.nol_absorption_of_382_limit_cents / 100,
                    remaining_382_limit_after_nols / 100,
                    allowed / 100,
                    disallowed / 100
                ),
            }
        }
        AttributeType::Section904ExcessForeignTaxCreditCarryover => {
            let allowed = input
                .pre_change_attribute_balance_cents
                .min(section_383_credit_limit_u64);
            let disallowed = input
                .pre_change_attribute_balance_cents
                .saturating_sub(allowed);
            let severity = if disallowed == 0 {
                Severity::AttributeFullyUsedWithinSection383Limit
            } else {
                Severity::Section383CExcessFtcLimitationApplied
            };
            Output {
                severity,
                section_383_annual_credit_limit_cents: section_383_credit_limit_u64,
                allowed_attribute_usage_cents: allowed,
                disallowed_attribute_usage_cents: disallowed,
                note: format!(
                    "§ 383(c) excess foreign tax credit limitation applied. § 904(c) excess FTC \
                     carryover (${}) competes for the post-NOL § 382 limitation. § 383 credit \
                     limit = remaining § 382 limit (${}) × effective corporate tax rate \
                     ({} bps) = ${}. Excess FTC used in post-change year limited to that \
                     amount. Allowed ${}; disallowed (carries forward subject to § 904(c) \
                     10-year carryforward + 1-year carryback) ${}. Coordinates with § 904 \
                     basket-by-basket limitation.",
                    input.pre_change_attribute_balance_cents / 100,
                    remaining_382_limit_after_nols / 100,
                    input.effective_corporate_tax_rate_bps,
                    section_383_credit_limit_u64 / 100,
                    allowed / 100,
                    disallowed / 100
                ),
            }
        }
        AttributeType::Section39GeneralBusinessCredit
        | AttributeType::Section53MinimumTaxCredit => {
            let allowed = input
                .pre_change_attribute_balance_cents
                .min(section_383_credit_limit_u64);
            let disallowed = input
                .pre_change_attribute_balance_cents
                .saturating_sub(allowed);
            let severity = if disallowed == 0 {
                Severity::AttributeFullyUsedWithinSection383Limit
            } else {
                Severity::Section383CreditLimitationApplied
            };
            let credit_label = match input.attribute_type {
                AttributeType::Section39GeneralBusinessCredit => "§ 39 general business credit",
                AttributeType::Section53MinimumTaxCredit => "§ 53 minimum tax credit",
                _ => unreachable!(),
            };
            Output {
                severity,
                section_383_annual_credit_limit_cents: section_383_credit_limit_u64,
                allowed_attribute_usage_cents: allowed,
                disallowed_attribute_usage_cents: disallowed,
                note: format!(
                    "§ 383(a) excess credit limitation applied. {} carryforward (${}) limited \
                     to tax attributable to taxable income within the § 382 limit. § 383 \
                     credit limit = remaining § 382 limit (${}) × effective corporate tax \
                     rate ({} bps) = ${}. Allowed ${}; disallowed (carries forward subject to \
                     § 39 / § 53 carryforward rules) ${}. Coordinates with § 382 NOL annual \
                     cap, § 384 preacquisition-loss built-in-gain disallowance, § 269 \
                     discretionary disallowance.",
                    credit_label,
                    input.pre_change_attribute_balance_cents / 100,
                    remaining_382_limit_after_nols / 100,
                    input.effective_corporate_tax_rate_bps,
                    section_383_credit_limit_u64 / 100,
                    allowed / 100,
                    disallowed / 100
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            ownership_change_status:
                OwnershipChangeStatus::OwnershipChangeOccurredSection382G,
            attribute_type: AttributeType::Section39GeneralBusinessCredit,
            section_382_annual_limitation_cents: 10_000_000_00,
            effective_corporate_tax_rate_bps: 2_100,
            pre_change_attribute_balance_cents: 5_000_000_00,
            nol_absorption_of_382_limit_cents: 0,
            post_change_taxable_income_cents: 50_000_000_00,
        }
    }

    #[test]
    fn no_ownership_change_full_attribute_usage_allowed() {
        let mut input = base();
        input.ownership_change_status =
            OwnershipChangeStatus::NoOwnershipChangeNoLimitationTriggered;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoOwnershipChangeFullAttributeUsageAllowed
        );
        assert_eq!(output.disallowed_attribute_usage_cents, 0);
        assert!(output.note.contains("50"));
        assert!(output.note.contains("3-year"));
    }

    #[test]
    fn section_383_general_business_credit_limited_to_tax_on_382_limit() {
        let input = base();
        let output = check(&input);
        // § 383 limit = $10M × 21% = $2.1M
        assert_eq!(output.section_383_annual_credit_limit_cents, 2_100_000_00);
        // $5M pre-change credit, only $2.1M usable
        assert_eq!(output.allowed_attribute_usage_cents, 2_100_000_00);
        assert_eq!(output.disallowed_attribute_usage_cents, 2_900_000_00);
        assert_eq!(output.severity, Severity::Section383CreditLimitationApplied);
        assert!(output.note.contains("§ 383(a)"));
        assert!(output.note.contains("§ 39 general business credit"));
    }

    #[test]
    fn section_383_minimum_tax_credit_uses_same_formula() {
        let mut input = base();
        input.attribute_type = AttributeType::Section53MinimumTaxCredit;
        let output = check(&input);
        assert_eq!(output.section_383_annual_credit_limit_cents, 2_100_000_00);
        assert_eq!(output.allowed_attribute_usage_cents, 2_100_000_00);
        assert!(output.note.contains("§ 53 minimum tax credit"));
    }

    #[test]
    fn section_383b_capital_loss_competes_with_382_limit_anti_stuffing() {
        let mut input = base();
        input.attribute_type = AttributeType::Section1212NetCapitalLossCarryover;
        input.pre_change_attribute_balance_cents = 15_000_000_00;
        let output = check(&input);
        // Capital loss directly competes with § 382 limit ($10M)
        // $15M attempted vs $10M limit → $10M allowed, $5M disallowed
        assert_eq!(output.allowed_attribute_usage_cents, 10_000_000_00);
        assert_eq!(output.disallowed_attribute_usage_cents, 5_000_000_00);
        assert_eq!(
            output.severity,
            Severity::Section383BCapitalLossLimitationAppliedAntiStuffing
        );
        assert!(output.note.contains("§ 383(b)"));
        assert!(output.note.contains("anti-stuffing"));
    }

    #[test]
    fn section_383c_excess_ftc_uses_credit_formula() {
        let mut input = base();
        input.attribute_type = AttributeType::Section904ExcessForeignTaxCreditCarryover;
        let output = check(&input);
        assert_eq!(output.section_383_annual_credit_limit_cents, 2_100_000_00);
        assert_eq!(output.allowed_attribute_usage_cents, 2_100_000_00);
        assert_eq!(output.disallowed_attribute_usage_cents, 2_900_000_00);
        assert_eq!(
            output.severity,
            Severity::Section383CExcessFtcLimitationApplied
        );
        assert!(output.note.contains("§ 904(c)"));
        assert!(output.note.contains("10-year carryforward"));
    }

    #[test]
    fn nol_absorption_reduces_section_383_credit_limit() {
        let mut input = base();
        // $7M NOL absorbs from $10M § 382 limit; $3M remaining
        input.nol_absorption_of_382_limit_cents = 7_000_000_00;
        let output = check(&input);
        // § 383 limit = $3M × 21% = $630K
        assert_eq!(output.section_383_annual_credit_limit_cents, 630_000_00);
        assert_eq!(output.allowed_attribute_usage_cents, 630_000_00);
    }

    #[test]
    fn full_nol_absorption_zeroes_section_383_limit() {
        let mut input = base();
        input.nol_absorption_of_382_limit_cents = 10_000_000_00;
        let output = check(&input);
        assert_eq!(output.section_383_annual_credit_limit_cents, 0);
        assert_eq!(output.allowed_attribute_usage_cents, 0);
        assert_eq!(output.disallowed_attribute_usage_cents, 5_000_000_00);
    }

    #[test]
    fn small_credit_under_383_limit_fully_used() {
        let mut input = base();
        input.pre_change_attribute_balance_cents = 1_000_000_00;
        let output = check(&input);
        // $1M < $2.1M § 383 limit → fully usable
        assert_eq!(output.allowed_attribute_usage_cents, 1_000_000_00);
        assert_eq!(output.disallowed_attribute_usage_cents, 0);
        assert_eq!(
            output.severity,
            Severity::AttributeFullyUsedWithinSection383Limit
        );
    }

    #[test]
    fn capital_loss_smaller_than_382_limit_fully_used() {
        let mut input = base();
        input.attribute_type = AttributeType::Section1212NetCapitalLossCarryover;
        input.pre_change_attribute_balance_cents = 5_000_000_00;
        let output = check(&input);
        // $5M < $10M § 382 limit → fully usable
        assert_eq!(output.allowed_attribute_usage_cents, 5_000_000_00);
        assert_eq!(output.disallowed_attribute_usage_cents, 0);
        assert_eq!(
            output.severity,
            Severity::AttributeFullyUsedWithinSection383Limit
        );
    }

    #[test]
    fn higher_tax_rate_increases_section_383_credit_limit() {
        let mut input = base();
        input.effective_corporate_tax_rate_bps = 3_500; // 35% pre-TCJA rate
        let output = check(&input);
        // $10M × 35% = $3.5M § 383 limit
        assert_eq!(output.section_383_annual_credit_limit_cents, 3_500_000_00);
    }

    #[test]
    fn section_382_testing_period_constant_pins_3_years() {
        assert_eq!(SECTION_382_TESTING_PERIOD_YEARS, 3);
    }

    #[test]
    fn section_382_ownership_change_threshold_pins_50_pct_points() {
        assert_eq!(SECTION_382_OWNERSHIP_CHANGE_THRESHOLD_PERCENTAGE_POINTS, 50);
    }

    #[test]
    fn very_large_credit_no_overflow() {
        let mut input = base();
        input.section_382_annual_limitation_cents = u64::MAX / 2;
        input.pre_change_attribute_balance_cents = u64::MAX;
        let output = check(&input);
        assert!(output.section_383_annual_credit_limit_cents > 0);
    }

    #[test]
    fn zero_credit_balance_no_disallowance() {
        let mut input = base();
        input.pre_change_attribute_balance_cents = 0;
        let output = check(&input);
        assert_eq!(output.allowed_attribute_usage_cents, 0);
        assert_eq!(output.disallowed_attribute_usage_cents, 0);
    }

    #[test]
    fn note_pins_section_269_discretionary_disallowance() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 269"));
    }

    #[test]
    fn note_pins_section_384_preacquisition_loss() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 384"));
    }

    #[test]
    fn note_pins_section_382_nol_annual_cap() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 382"));
    }

    #[test]
    fn capital_loss_anti_stuffing_note_present() {
        let mut input = base();
        input.attribute_type = AttributeType::Section1212NetCapitalLossCarryover;
        input.pre_change_attribute_balance_cents = 20_000_000_00;
        let output = check(&input);
        assert!(output.note.contains("REDUCES"));
        assert!(output.note.contains("anti-stuffing rule"));
    }
}
