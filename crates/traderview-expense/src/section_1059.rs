//! IRC § 1059 corporate shareholder's basis in stock reduced by
//! nontaxed portion of extraordinary dividends.
//!
//! Trader-corporation anti-abuse provision: when a corporate
//! shareholder claims the dividends-received deduction (DRD) under
//! § 243, § 245, or § 245A and the dividend exceeds an
//! "extraordinary" threshold, § 1059 requires the corporate
//! shareholder to REDUCE basis in the stock by the nontaxed portion
//! of the dividend (the DRD-excluded amount). If the nontaxed
//! portion exceeds basis, the excess is recognized as gain from
//! sale or exchange of the stock. Designed to prevent the
//! "dividend stripping" pattern where corporations would buy
//! dividend-paying stock just before a large dividend, claim the
//! DRD, then sell the stock at a loss equal to the dividend amount
//! — generating an artificial loss while excluding most of the
//! dividend from income.
//!
//! **§ 1059(a) general rule**: basis reduced (but not below zero)
//! by nontaxed portion. Excess of nontaxed portion over basis is
//! recognized as gain from sale or exchange of stock for the
//! taxable year in which the extraordinary dividend is received.
//!
//! **§ 1059(b) nontaxed portion**: amount excluded from gross
//! income by virtue of § 243 (general corporate DRD), § 245
//! (foreign-source DRD), or § 245A (TCJA 2017 100% DRD for foreign-
//! source income from specified 10%-owned foreign corporations).
//!
//! **§ 1059(c) extraordinary dividend definition**: dividend equals
//! or exceeds **10%** of corporate shareholder's adjusted basis in
//! common stock; **5%** threshold for preferred stock. All
//! dividends within an 85-day aggregation window (or 365-day window
//! at taxpayer election) may be aggregated.
//!
//! **§ 1059(d)(6) exception**: stock held during the entire
//! existence of the distributing corporation is exempt.
//!
//! **§ 1059(e)(1) per se extraordinary dividends** — override both
//! the threshold and 2-year holding requirements: (1) any redemption
//! treated as dividend under § 301 that is non-pro-rata; (2)
//! redemption that would not have been treated as a dividend but
//! for § 318(a)(4) options attribution; (3) any distribution in
//! partial liquidation treated as a dividend.
//!
//! **§ 1059(a)(2) 2-year holding-period exception**: if corporate
//! shareholder held stock for more than 2 years before the dividend
//! announcement date, basis reduction does not apply (except for
//! § 1059(e)(1) per se cases). TCJA 2017 did not modify this
//! exception but added § 245A as a basis for the nontaxed portion,
//! significantly expanding the scope of § 1059 in cross-border
//! contexts.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const COMMON_STOCK_EXTRAORDINARY_THRESHOLD_PERCENT: u32 = 10;
#[allow(dead_code)]
pub const PREFERRED_STOCK_EXTRAORDINARY_THRESHOLD_PERCENT: u32 = 5;
#[allow(dead_code)]
pub const HOLDING_PERIOD_THRESHOLD_YEARS: u32 = 2;
#[allow(dead_code)]
pub const HOLDING_PERIOD_THRESHOLD_DAYS: u32 = 730;
#[allow(dead_code)]
pub const AGGREGATION_WINDOW_DAYS_SHORT: u32 = 85;
#[allow(dead_code)]
pub const AGGREGATION_WINDOW_DAYS_LONG: u32 = 365;
#[allow(dead_code)]
pub const TCJA_2017_SECTION_245A_EFFECTIVE_YEAR: u32 = 2018;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerSeExtraordinaryEvent {
    None,
    NonProRataRedemption,
    PartialLiquidation,
    SecondaryOptionAttributionRedemption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotCorporateShareholderNoBasisReduction,
    NoExtraordinaryDividendBelowThreshold,
    HoldingPeriodOver2YearsNoBasisReduction,
    ExtraordinaryDividendBasisReductionApplied,
    ExtraordinaryDividendExcessOverBasisRecognizedAsGain,
    PerSeExtraordinaryNonProRataRedemptionUnder1059e,
    PerSeExtraordinaryPartialLiquidationUnder1059e,
    PerSeExtraordinarySecondaryOptionAttributionUnder1059e,
    ViolationFailedToReduceBasisForExtraordinaryDividend,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub corporate_shareholder: bool,
    pub stock_is_preferred: bool,
    pub dividend_amount_cents: u64,
    pub aggregated_dividends_85_day_window_cents: u64,
    pub corporate_shareholder_basis_in_stock_cents: u64,
    pub days_held_before_dividend_announcement: u32,
    pub nontaxed_portion_cents: u64,
    pub per_se_event: PerSeExtraordinaryEvent,
    pub taxpayer_reduced_basis: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub is_extraordinary_dividend: bool,
    pub basis_reduction_cents: u64,
    pub gain_recognized_cents: u64,
    pub adjusted_basis_after_cents: u64,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section1059Input = Input;
pub type Section1059Output = Output;
pub type Section1059Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 1059(a) (basis reduction by nontaxed portion of extraordinary dividend)".to_string(),
        "IRC § 1059(a)(2) (gain recognition for excess over basis)".to_string(),
        "IRC § 1059(b) (nontaxed portion definition — DRD amounts)".to_string(),
        "IRC § 1059(c) (extraordinary dividend 10%/5% threshold)".to_string(),
        "IRC § 1059(c)(3) (aggregation window — 85 days short, 365 days long)".to_string(),
        "IRC § 1059(d)(6) (entire-existence-of-corporation exception)".to_string(),
        "IRC § 1059(e)(1) (per se extraordinary dividends — non-pro-rata redemptions, partial liquidations)".to_string(),
        "IRC § 243 (general dividends received deduction)".to_string(),
        "IRC § 245 (foreign-source dividends received deduction)".to_string(),
        "IRC § 245A (TCJA 2017 — 100% DRD for foreign-source income from specified 10%-owned foreign corporations)".to_string(),
        "IRC § 301 (dividend distribution rules)".to_string(),
        "IRC § 318(a)(4) (options attribution — cross-reference for § 1059(e)(1))".to_string(),
        "Treas. Reg. § 1.1059(e)-1 (non-pro-rata redemptions implementation)".to_string(),
        "TCJA 2017 § 14101 (added § 245A — significantly expanded § 1059 cross-border scope)".to_string(),
    ];

    if !input.corporate_shareholder {
        notes.push("Non-corporate shareholder — § 1059 applies only to corporate shareholders.".to_string());
        return Output {
            severity: Severity::NotCorporateShareholderNoBasisReduction,
            is_extraordinary_dividend: false,
            basis_reduction_cents: 0,
            gain_recognized_cents: 0,
            adjusted_basis_after_cents: input.corporate_shareholder_basis_in_stock_cents,
            notes,
            citations,
        };
    }

    let per_se = !matches!(input.per_se_event, PerSeExtraordinaryEvent::None);
    if per_se {
        let basis = input.corporate_shareholder_basis_in_stock_cents;
        let nontaxed = input.nontaxed_portion_cents;
        let (reduction, gain, new_basis) = if nontaxed <= basis {
            (nontaxed, 0u64, basis - nontaxed)
        } else {
            let excess = nontaxed - basis;
            (basis, excess, 0u64)
        };
        let severity = match input.per_se_event {
            PerSeExtraordinaryEvent::NonProRataRedemption => {
                notes.push(format!(
                    "§ 1059(e)(1) per se extraordinary: non-pro-rata redemption under § 301 — basis reduction ${}, gain ${} regardless of 10% threshold or 2-year holding period.",
                    reduction / 100,
                    gain / 100
                ));
                Severity::PerSeExtraordinaryNonProRataRedemptionUnder1059e
            }
            PerSeExtraordinaryEvent::PartialLiquidation => {
                notes.push(format!(
                    "§ 1059(e)(1) per se extraordinary: partial liquidation treated as dividend — basis reduction ${}, gain ${}.",
                    reduction / 100,
                    gain / 100
                ));
                Severity::PerSeExtraordinaryPartialLiquidationUnder1059e
            }
            PerSeExtraordinaryEvent::SecondaryOptionAttributionRedemption => {
                notes.push(format!(
                    "§ 1059(e)(1)(A)(ii) per se extraordinary: redemption would not have been dividend but for § 318(a)(4) options attribution — basis reduction ${}, gain ${}.",
                    reduction / 100,
                    gain / 100
                ));
                Severity::PerSeExtraordinarySecondaryOptionAttributionUnder1059e
            }
            PerSeExtraordinaryEvent::None => unreachable!(),
        };
        return Output {
            severity,
            is_extraordinary_dividend: true,
            basis_reduction_cents: reduction,
            gain_recognized_cents: gain,
            adjusted_basis_after_cents: new_basis,
            notes,
            citations,
        };
    }

    if input.days_held_before_dividend_announcement > HOLDING_PERIOD_THRESHOLD_DAYS {
        notes.push(format!(
            "Stock held {} days > {} (2 years) before dividend announcement — § 1059(a)(2) exception applies; no basis reduction.",
            input.days_held_before_dividend_announcement,
            HOLDING_PERIOD_THRESHOLD_DAYS
        ));
        return Output {
            severity: Severity::HoldingPeriodOver2YearsNoBasisReduction,
            is_extraordinary_dividend: false,
            basis_reduction_cents: 0,
            gain_recognized_cents: 0,
            adjusted_basis_after_cents: input.corporate_shareholder_basis_in_stock_cents,
            notes,
            citations,
        };
    }

    let threshold_pct = if input.stock_is_preferred {
        PREFERRED_STOCK_EXTRAORDINARY_THRESHOLD_PERCENT
    } else {
        COMMON_STOCK_EXTRAORDINARY_THRESHOLD_PERCENT
    };
    let threshold_amount = input
        .corporate_shareholder_basis_in_stock_cents
        .saturating_mul(threshold_pct as u64)
        / 100;
    let aggregate_dividend = input
        .dividend_amount_cents
        .max(input.aggregated_dividends_85_day_window_cents);
    let is_extraordinary = aggregate_dividend >= threshold_amount;

    if !is_extraordinary {
        notes.push(format!(
            "Dividend ${} below extraordinary threshold ${} ({}% × ${} basis); not § 1059 extraordinary dividend.",
            aggregate_dividend / 100,
            threshold_amount / 100,
            threshold_pct,
            input.corporate_shareholder_basis_in_stock_cents / 100
        ));
        return Output {
            severity: Severity::NoExtraordinaryDividendBelowThreshold,
            is_extraordinary_dividend: false,
            basis_reduction_cents: 0,
            gain_recognized_cents: 0,
            adjusted_basis_after_cents: input.corporate_shareholder_basis_in_stock_cents,
            notes,
            citations,
        };
    }

    let basis = input.corporate_shareholder_basis_in_stock_cents;
    let nontaxed = input.nontaxed_portion_cents;
    let (reduction, gain, new_basis) = if nontaxed <= basis {
        (nontaxed, 0u64, basis - nontaxed)
    } else {
        let excess = nontaxed - basis;
        (basis, excess, 0u64)
    };

    if !input.taxpayer_reduced_basis {
        notes.push(format!(
            "Extraordinary dividend ${} requires § 1059(a) basis reduction of ${}; taxpayer failed to reduce basis.",
            aggregate_dividend / 100,
            reduction / 100
        ));
        return Output {
            severity: Severity::ViolationFailedToReduceBasisForExtraordinaryDividend,
            is_extraordinary_dividend: true,
            basis_reduction_cents: reduction,
            gain_recognized_cents: gain,
            adjusted_basis_after_cents: new_basis,
            notes,
            citations,
        };
    }

    if gain > 0 {
        notes.push(format!(
            "§ 1059(a)(2): nontaxed portion ${} exceeds basis ${} by ${} — gain recognized as sale/exchange.",
            nontaxed / 100,
            basis / 100,
            gain / 100
        ));
        return Output {
            severity: Severity::ExtraordinaryDividendExcessOverBasisRecognizedAsGain,
            is_extraordinary_dividend: true,
            basis_reduction_cents: reduction,
            gain_recognized_cents: gain,
            adjusted_basis_after_cents: new_basis,
            notes,
            citations,
        };
    }

    notes.push(format!(
        "§ 1059(a) basis reduction applied: ${} reduction; adjusted basis ${} (no gain — nontaxed portion ≤ basis).",
        reduction / 100,
        new_basis / 100
    ));
    Output {
        severity: Severity::ExtraordinaryDividendBasisReductionApplied,
        is_extraordinary_dividend: true,
        basis_reduction_cents: reduction,
        gain_recognized_cents: 0,
        adjusted_basis_after_cents: new_basis,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_corporate_extraordinary() -> Input {
        Input {
            corporate_shareholder: true,
            stock_is_preferred: false,
            dividend_amount_cents: 2_000_000,
            aggregated_dividends_85_day_window_cents: 0,
            corporate_shareholder_basis_in_stock_cents: 10_000_000,
            days_held_before_dividend_announcement: 365,
            nontaxed_portion_cents: 1_500_000,
            per_se_event: PerSeExtraordinaryEvent::None,
            taxpayer_reduced_basis: true,
        }
    }

    #[test]
    fn corporate_extraordinary_dividend_basis_reduction_applied() {
        let out = check(&base_corporate_extraordinary());
        assert_eq!(
            out.severity,
            Severity::ExtraordinaryDividendBasisReductionApplied
        );
        assert!(out.is_extraordinary_dividend);
        assert_eq!(out.basis_reduction_cents, 1_500_000);
        assert_eq!(out.adjusted_basis_after_cents, 8_500_000);
    }

    #[test]
    fn non_corporate_shareholder_not_applicable() {
        let mut i = base_corporate_extraordinary();
        i.corporate_shareholder = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NotCorporateShareholderNoBasisReduction
        );
    }

    #[test]
    fn dividend_below_10_pct_threshold_not_extraordinary() {
        let mut i = base_corporate_extraordinary();
        i.dividend_amount_cents = 500_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NoExtraordinaryDividendBelowThreshold
        );
        assert!(!out.is_extraordinary_dividend);
    }

    #[test]
    fn dividend_exactly_at_10_pct_is_extraordinary() {
        let mut i = base_corporate_extraordinary();
        i.dividend_amount_cents = 1_000_000;
        let out = check(&i);
        assert!(out.is_extraordinary_dividend);
    }

    #[test]
    fn preferred_stock_uses_5_pct_threshold() {
        let mut i = base_corporate_extraordinary();
        i.stock_is_preferred = true;
        i.dividend_amount_cents = 600_000;
        let out = check(&i);
        assert!(out.is_extraordinary_dividend);
    }

    #[test]
    fn preferred_stock_below_5_pct_threshold_not_extraordinary() {
        let mut i = base_corporate_extraordinary();
        i.stock_is_preferred = true;
        i.dividend_amount_cents = 400_000;
        let out = check(&i);
        assert!(!out.is_extraordinary_dividend);
    }

    #[test]
    fn holding_period_over_2_years_no_reduction() {
        let mut i = base_corporate_extraordinary();
        i.days_held_before_dividend_announcement = 731;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::HoldingPeriodOver2YearsNoBasisReduction
        );
    }

    #[test]
    fn holding_period_at_exactly_730_days_still_triggers() {
        let mut i = base_corporate_extraordinary();
        i.days_held_before_dividend_announcement = 730;
        let out = check(&i);
        assert!(out.is_extraordinary_dividend);
    }

    #[test]
    fn aggregation_85_day_window_combines_dividends() {
        let mut i = base_corporate_extraordinary();
        i.dividend_amount_cents = 600_000;
        i.aggregated_dividends_85_day_window_cents = 1_200_000;
        let out = check(&i);
        assert!(out.is_extraordinary_dividend);
    }

    #[test]
    fn nontaxed_portion_exceeds_basis_recognized_as_gain() {
        let mut i = base_corporate_extraordinary();
        i.corporate_shareholder_basis_in_stock_cents = 1_000_000;
        i.dividend_amount_cents = 200_000;
        i.nontaxed_portion_cents = 1_500_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ExtraordinaryDividendExcessOverBasisRecognizedAsGain
        );
        assert_eq!(out.basis_reduction_cents, 1_000_000);
        assert_eq!(out.gain_recognized_cents, 500_000);
        assert_eq!(out.adjusted_basis_after_cents, 0);
    }

    #[test]
    fn taxpayer_failed_to_reduce_basis_violation() {
        let mut i = base_corporate_extraordinary();
        i.taxpayer_reduced_basis = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFailedToReduceBasisForExtraordinaryDividend
        );
    }

    #[test]
    fn per_se_non_pro_rata_redemption_overrides_threshold_and_holding() {
        let mut i = base_corporate_extraordinary();
        i.per_se_event = PerSeExtraordinaryEvent::NonProRataRedemption;
        i.dividend_amount_cents = 100_000;
        i.days_held_before_dividend_announcement = 3650;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::PerSeExtraordinaryNonProRataRedemptionUnder1059e
        );
        assert!(out.is_extraordinary_dividend);
    }

    #[test]
    fn per_se_partial_liquidation_overrides_threshold_and_holding() {
        let mut i = base_corporate_extraordinary();
        i.per_se_event = PerSeExtraordinaryEvent::PartialLiquidation;
        i.dividend_amount_cents = 100_000;
        i.days_held_before_dividend_announcement = 3650;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::PerSeExtraordinaryPartialLiquidationUnder1059e
        );
    }

    #[test]
    fn per_se_318_options_attribution_overrides_threshold_and_holding() {
        let mut i = base_corporate_extraordinary();
        i.per_se_event = PerSeExtraordinaryEvent::SecondaryOptionAttributionRedemption;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::PerSeExtraordinarySecondaryOptionAttributionUnder1059e
        );
    }

    #[test]
    fn citations_pin_1059_subsections() {
        let out = check(&base_corporate_extraordinary());
        assert!(out.citations.iter().any(|c| c.contains("§ 1059(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1059(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1059(c)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1059(d)(6)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1059(e)(1)")));
    }

    #[test]
    fn citations_pin_drd_243_245_245a_cross_refs() {
        let out = check(&base_corporate_extraordinary());
        assert!(out.citations.iter().any(|c| c.contains("§ 243")));
        assert!(out.citations.iter().any(|c| c.contains("§ 245")));
        assert!(out.citations.iter().any(|c| c.contains("§ 245A")));
    }

    #[test]
    fn citations_pin_301_318a4_treas_reg_1059e1() {
        let out = check(&base_corporate_extraordinary());
        assert!(out.citations.iter().any(|c| c.contains("§ 301")));
        assert!(out.citations.iter().any(|c| c.contains("§ 318(a)(4)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.1059(e)-1")));
    }

    #[test]
    fn citations_pin_tcja_2017_245a_addition() {
        let out = check(&base_corporate_extraordinary());
        assert!(out.citations.iter().any(|c| c.contains("TCJA 2017")));
        assert!(out.citations.iter().any(|c| c.contains("§ 14101")));
    }

    #[test]
    fn constant_pin_10_pct_common_threshold() {
        assert_eq!(COMMON_STOCK_EXTRAORDINARY_THRESHOLD_PERCENT, 10);
    }

    #[test]
    fn constant_pin_5_pct_preferred_threshold() {
        assert_eq!(PREFERRED_STOCK_EXTRAORDINARY_THRESHOLD_PERCENT, 5);
    }

    #[test]
    fn constant_pin_2_year_holding_period() {
        assert_eq!(HOLDING_PERIOD_THRESHOLD_YEARS, 2);
        assert_eq!(HOLDING_PERIOD_THRESHOLD_DAYS, 730);
    }

    #[test]
    fn constant_pin_85_and_365_day_aggregation_windows() {
        assert_eq!(AGGREGATION_WINDOW_DAYS_SHORT, 85);
        assert_eq!(AGGREGATION_WINDOW_DAYS_LONG, 365);
    }

    #[test]
    fn very_large_nontaxed_portion_saturating_no_overflow() {
        let mut i = base_corporate_extraordinary();
        i.nontaxed_portion_cents = u64::MAX;
        let out = check(&i);
        assert_eq!(out.adjusted_basis_after_cents, 0);
        assert!(out.gain_recognized_cents > 0);
    }
}
