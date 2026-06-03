//! IRC § 246 rules applying to deductions for dividends received.
//!
//! § 246 sets out the general limitations on the dividends-received deduction (DRD)
//! claimable under § 243 (domestic-corp DRD), § 245(a) (US-source-portion DRD on
//! foreign-corp dividends), § 245A (100% participation DRD on foreign-source dividends),
//! and (indirectly) § 246A (debt-financed-portfolio-stock DRD reduction). § 246 imposes:
//! (1) the holding-period requirement under § 246(c) — the corporate shareholder must
//! own the stock for a sufficient number of days during a window spanning the ex-
//! dividend date; (2) the taxable-income cap under § 246(b) — DRD limited to 70% / 65% /
//! 100% of taxable income depending on ownership tier; (3) the exclusions under
//! § 246(a) for dividends from certain corporations (sanctioned foreign corps,
//! mutual savings banks, RIC/REIT distributions other than as RIC/REIT dividends).
//!
//! § 246(c) HOLDING PERIOD: stock must be held for MORE THAN 45 days during the 91-day
//! period beginning 45 days before the ex-dividend date (for common stock). For
//! PREFERRED stock with dividends for periods exceeding 366 days: stock must be held
//! for MORE THAN 90 days during the 181-day period beginning 90 days before the
//! ex-dividend date.
//!
//! § 246(c)(4) holding-period TOLLING: any day on which the taxpayer has substantially
//! diminished risk of loss (short sale, equity put, contractual hedge) does NOT count
//! toward the holding period.
//!
//! § 246(b) TAXABLE-INCOME CAP: DRD limited to 50% of taxable income for less-than-20%
//! ownership tier (50% DRD), 65% of taxable income for 20%-50%-ownership tier (65% DRD),
//! and 100% (no cap) for 80%+ affiliated-group tier. § 246(b)(2) NOL EXCEPTION:
//! taxable-income limit does NOT apply for the year if it would create or increase a
//! net operating loss.
//!
//! § 246(a) DRD EXCLUSIONS: no DRD for: (1) dividends from corporations exempt from
//! tax under § 501 or § 521; (2) dividends from RICs that are not "qualified
//! dividends" under § 854; (3) dividends from REITs that are not RIC/REIT capital-
//! gain distributions; (4) dividends from § 901(j) sanctioned-country foreign
//! corporations; (5) certain dividends on debt-financed portfolio stock under § 246A.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/246
//! - law.cornell.edu/cfr/text/26/1.246-3
//! - irc.bloombergtax.com/public/uscode/doc/irc/section_246
//! - en.wikipedia.org/wiki/Dividends_received_deduction

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StockType {
    /// Common stock — 46-day-of-91-day-window holding-period rule.
    CommonStockSection246cOne,
    /// Preferred stock with dividends for periods exceeding 366 days — 91-day-of-
    /// 181-day-window holding-period rule.
    PreferredStockExtendedDividendPeriodSection246cTwo,
    /// Preferred stock with dividends for periods 366 days or less — falls under
    /// common-stock 46-day rule.
    PreferredStockShortDividendPeriodTreatedAsCommon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipTier {
    /// Less than 20% ownership — 50% DRD baseline + 50%-of-TI cap.
    LessThanTwentyPercent50PctDrd,
    /// 20% to less than 80% — 65% DRD baseline + 65%-of-TI cap.
    TwentyToSeventyNinePercent65PctDrd,
    /// 80% or more affiliated group — 100% DRD + no taxable-income cap.
    EightyPercentPlusAffiliated100PctDrdNoCap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DividendIssuerType {
    /// Standard domestic C corporation.
    DomesticCCorporation,
    /// Foreign corporation in § 901(j) sanctioned country — DRD denied.
    SanctionedCountryForeignCorporation,
    /// Tax-exempt corporation under § 501 or § 521 — DRD denied.
    Section501Or521TaxExemptCorporation,
    /// RIC distribution (not a § 854 qualified dividend pass-through).
    NonQualifiedRicDistribution,
    /// REIT distribution (not a RIC/REIT capital-gain pass-through).
    NonCapitalGainReitDistribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NolStatus {
    NoNolYear,
    NolYearTaxableIncomeCapInapplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section246AHoldingPeriodFailedDrdFullyDisallowed,
    Section246AExcludedIssuerDrdDenied,
    Section246BTaxableIncomeCapAppliedReducedDrd,
    Section246BTwoNolExceptionFullDrdPreserved,
    FullDrdAllowedHoldingPeriodAndCapsMet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub stock_type: StockType,
    pub ownership_tier: OwnershipTier,
    pub dividend_issuer_type: DividendIssuerType,
    pub nol_status: NolStatus,
    pub days_held_during_required_window: u32,
    pub dividend_received_cents: u64,
    pub taxable_income_before_drd_cents: u64,
}

pub type Section246DividendsReceivedDeductionLimitsInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub required_holding_period_days: u32,
    pub testing_window_days: u32,
    pub drd_percentage_basis_points: u32,
    pub taxable_income_cap_cents: u64,
    pub allowed_drd_cents: u64,
    pub note: String,
}

pub type Section246DividendsReceivedDeductionLimitsOutput = Output;
pub type Section246DividendsReceivedDeductionLimitsResult = Output;

const COMMON_HOLDING_PERIOD_DAYS: u32 = 46;
const COMMON_TESTING_WINDOW_DAYS: u32 = 91;
const PREFERRED_EXTENDED_HOLDING_PERIOD_DAYS: u32 = 91;
const PREFERRED_EXTENDED_TESTING_WINDOW_DAYS: u32 = 181;
const LESS_THAN_TWENTY_DRD_BPS: u32 = 5_000;
const TWENTY_TO_EIGHTY_DRD_BPS: u32 = 6_500;
const EIGHTY_PLUS_DRD_BPS: u32 = 10_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    let (required_holding, testing_window) = match input.stock_type {
        StockType::CommonStockSection246cOne
        | StockType::PreferredStockShortDividendPeriodTreatedAsCommon => {
            (COMMON_HOLDING_PERIOD_DAYS, COMMON_TESTING_WINDOW_DAYS)
        }
        StockType::PreferredStockExtendedDividendPeriodSection246cTwo => (
            PREFERRED_EXTENDED_HOLDING_PERIOD_DAYS,
            PREFERRED_EXTENDED_TESTING_WINDOW_DAYS,
        ),
    };

    if !matches!(
        input.dividend_issuer_type,
        DividendIssuerType::DomesticCCorporation
    ) {
        return Output {
            severity: Severity::Section246AExcludedIssuerDrdDenied,
            required_holding_period_days: required_holding,
            testing_window_days: testing_window,
            drd_percentage_basis_points: 0,
            taxable_income_cap_cents: 0,
            allowed_drd_cents: 0,
            note: format!(
                "§ 246(a) DRD DENIAL: dividend from {} is categorically excluded from DRD. \
                 § 246(a) denies DRD for dividends from (1) § 501 / § 521 tax-exempt \
                 corporations, (2) non-qualified RIC distributions, (3) non-capital-gain \
                 REIT distributions, (4) § 901(j) sanctioned-country foreign corporations, \
                 (5) debt-financed portfolio stock under § 246A. Dividend ${} fully taxable; \
                 no DRD available.",
                issuer_label(input.dividend_issuer_type),
                input.dividend_received_cents / 100
            ),
        };
    }

    if input.days_held_during_required_window <= required_holding {
        return Output {
            severity: Severity::Section246AHoldingPeriodFailedDrdFullyDisallowed,
            required_holding_period_days: required_holding,
            testing_window_days: testing_window,
            drd_percentage_basis_points: 0,
            taxable_income_cap_cents: 0,
            allowed_drd_cents: 0,
            note: format!(
                "§ 246(c) holding-period requirement FAILED. Stock held {} days during the \
                 {}-day testing window; § 246(c) requires MORE THAN {} days. Note: § 246(c)(4) \
                 holding-period TOLLING — any day on which the corporate shareholder had \
                 substantially diminished risk of loss (short sale, equity put, contractual \
                 hedge) does NOT count toward the holding period. Dividend ${} fully taxable; \
                 no DRD available.",
                input.days_held_during_required_window,
                testing_window,
                required_holding,
                input.dividend_received_cents / 100
            ),
        };
    }

    let drd_percentage = match input.ownership_tier {
        OwnershipTier::LessThanTwentyPercent50PctDrd => LESS_THAN_TWENTY_DRD_BPS,
        OwnershipTier::TwentyToSeventyNinePercent65PctDrd => TWENTY_TO_EIGHTY_DRD_BPS,
        OwnershipTier::EightyPercentPlusAffiliated100PctDrdNoCap => EIGHTY_PLUS_DRD_BPS,
    };

    let raw_drd = u64::try_from(
        u128::from(input.dividend_received_cents)
            .saturating_mul(u128::from(drd_percentage))
            .saturating_div(10_000),
    )
    .unwrap_or(u64::MAX);

    if matches!(
        input.ownership_tier,
        OwnershipTier::EightyPercentPlusAffiliated100PctDrdNoCap
    ) {
        return Output {
            severity: Severity::FullDrdAllowedHoldingPeriodAndCapsMet,
            required_holding_period_days: required_holding,
            testing_window_days: testing_window,
            drd_percentage_basis_points: drd_percentage,
            taxable_income_cap_cents: 0,
            allowed_drd_cents: raw_drd,
            note: format!(
                "§ 246(b) taxable-income cap does NOT apply to 100% DRD tier (80%+ \
                 affiliated group per § 243(b) / § 1504). Full DRD ${} allowed.",
                raw_drd / 100
            ),
        };
    }

    if matches!(
        input.nol_status,
        NolStatus::NolYearTaxableIncomeCapInapplicable
    ) {
        return Output {
            severity: Severity::Section246BTwoNolExceptionFullDrdPreserved,
            required_holding_period_days: required_holding,
            testing_window_days: testing_window,
            drd_percentage_basis_points: drd_percentage,
            taxable_income_cap_cents: 0,
            allowed_drd_cents: raw_drd,
            note: format!(
                "§ 246(b)(2) NOL exception: taxable-income cap does NOT apply for the year if \
                 it would create or increase a net operating loss. Full DRD ${} allowed. The \
                 resulting NOL becomes subject to § 172 NOL carryback / carryforward rules \
                 (post-TCJA: no carryback, indefinite carryforward at 80% of taxable income).",
                raw_drd / 100
            ),
        };
    }

    let taxable_income_cap = u64::try_from(
        u128::from(input.taxable_income_before_drd_cents)
            .saturating_mul(u128::from(drd_percentage))
            .saturating_div(10_000),
    )
    .unwrap_or(u64::MAX);

    if raw_drd <= taxable_income_cap {
        return Output {
            severity: Severity::FullDrdAllowedHoldingPeriodAndCapsMet,
            required_holding_period_days: required_holding,
            testing_window_days: testing_window,
            drd_percentage_basis_points: drd_percentage,
            taxable_income_cap_cents: taxable_income_cap,
            allowed_drd_cents: raw_drd,
            note: format!(
                "Full DRD allowed: dividend ${} × {}% = ${} fits within § 246(b) taxable- \
                 income cap of ${} ({}% of taxable income before DRD ${}).",
                input.dividend_received_cents / 100,
                drd_percentage / 100,
                raw_drd / 100,
                taxable_income_cap / 100,
                drd_percentage / 100,
                input.taxable_income_before_drd_cents / 100
            ),
        };
    }

    let allowed = taxable_income_cap;
    Output {
        severity: Severity::Section246BTaxableIncomeCapAppliedReducedDrd,
        required_holding_period_days: required_holding,
        testing_window_days: testing_window,
        drd_percentage_basis_points: drd_percentage,
        taxable_income_cap_cents: taxable_income_cap,
        allowed_drd_cents: allowed,
        note: format!(
            "§ 246(b) taxable-income cap REDUCES DRD. Raw DRD ${} × ({}%) exceeds the \
             § 246(b) cap of {}% × taxable income before DRD (${}) = ${}. DRD limited to \
             ${}. § 246(b)(2) NOL exception not invoked. Coordinates with § 246A debt- \
             financed portfolio stock DRD reduction, § 243(a)/(b)/(c) ownership-tier DRD \
             percentages, § 246(c) holding-period rule, § 172 NOL carryover.",
            raw_drd / 100,
            drd_percentage / 100,
            drd_percentage / 100,
            input.taxable_income_before_drd_cents / 100,
            taxable_income_cap / 100,
            allowed / 100
        ),
    }
}

fn issuer_label(issuer: DividendIssuerType) -> &'static str {
    match issuer {
        DividendIssuerType::DomesticCCorporation => "domestic C corporation",
        DividendIssuerType::SanctionedCountryForeignCorporation => {
            "§ 901(j) sanctioned-country foreign corporation"
        }
        DividendIssuerType::Section501Or521TaxExemptCorporation => {
            "§ 501 or § 521 tax-exempt corporation"
        }
        DividendIssuerType::NonQualifiedRicDistribution => {
            "non-qualified RIC distribution (not a § 854 pass-through)"
        }
        DividendIssuerType::NonCapitalGainReitDistribution => {
            "non-capital-gain REIT distribution"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            stock_type: StockType::CommonStockSection246cOne,
            ownership_tier: OwnershipTier::LessThanTwentyPercent50PctDrd,
            dividend_issuer_type: DividendIssuerType::DomesticCCorporation,
            nol_status: NolStatus::NoNolYear,
            days_held_during_required_window: 50,
            dividend_received_cents: 100_000_00,
            taxable_income_before_drd_cents: 1_000_000_00,
        }
    }

    #[test]
    fn full_drd_allowed_common_stock_50_pct_tier() {
        let input = base();
        let output = check(&input);
        assert_eq!(output.severity, Severity::FullDrdAllowedHoldingPeriodAndCapsMet);
        // $100K × 50% = $50K; cap = $1M × 50% = $500K; $50K ≤ $500K → full DRD
        assert_eq!(output.allowed_drd_cents, 50_000_00);
        assert_eq!(output.drd_percentage_basis_points, 5_000);
        assert_eq!(output.required_holding_period_days, 46);
        assert_eq!(output.testing_window_days, 91);
    }

    #[test]
    fn holding_period_46_days_exactly_fails_must_be_more_than() {
        let mut input = base();
        input.days_held_during_required_window = 46;
        let output = check(&input);
        // > 46 required; exactly 46 fails
        assert_eq!(
            output.severity,
            Severity::Section246AHoldingPeriodFailedDrdFullyDisallowed
        );
    }

    #[test]
    fn holding_period_47_days_satisfies_more_than_46() {
        let mut input = base();
        input.days_held_during_required_window = 47;
        let output = check(&input);
        assert_eq!(output.severity, Severity::FullDrdAllowedHoldingPeriodAndCapsMet);
    }

    #[test]
    fn preferred_stock_extended_period_91_day_rule() {
        let mut input = base();
        input.stock_type = StockType::PreferredStockExtendedDividendPeriodSection246cTwo;
        input.days_held_during_required_window = 95;
        let output = check(&input);
        assert_eq!(output.required_holding_period_days, 91);
        assert_eq!(output.testing_window_days, 181);
        assert_eq!(output.severity, Severity::FullDrdAllowedHoldingPeriodAndCapsMet);
    }

    #[test]
    fn preferred_stock_extended_91_days_exactly_fails() {
        let mut input = base();
        input.stock_type = StockType::PreferredStockExtendedDividendPeriodSection246cTwo;
        input.days_held_during_required_window = 91;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section246AHoldingPeriodFailedDrdFullyDisallowed
        );
    }

    #[test]
    fn preferred_stock_short_period_uses_common_46_day_rule() {
        let mut input = base();
        input.stock_type = StockType::PreferredStockShortDividendPeriodTreatedAsCommon;
        input.days_held_during_required_window = 50;
        let output = check(&input);
        assert_eq!(output.required_holding_period_days, 46);
        assert_eq!(output.severity, Severity::FullDrdAllowedHoldingPeriodAndCapsMet);
    }

    #[test]
    fn taxable_income_cap_reduces_drd_when_low_ti() {
        let mut input = base();
        input.taxable_income_before_drd_cents = 50_000_00;
        let output = check(&input);
        // Raw DRD = $100K × 50% = $50K; cap = $50K × 50% = $25K
        // $50K > $25K → cap applies
        assert_eq!(
            output.severity,
            Severity::Section246BTaxableIncomeCapAppliedReducedDrd
        );
        assert_eq!(output.allowed_drd_cents, 25_000_00);
    }

    #[test]
    fn nol_year_taxable_income_cap_inapplicable() {
        let mut input = base();
        input.nol_status = NolStatus::NolYearTaxableIncomeCapInapplicable;
        input.taxable_income_before_drd_cents = 10_000_00;
        let output = check(&input);
        // Even with low TI, NOL exception allows full DRD
        assert_eq!(
            output.severity,
            Severity::Section246BTwoNolExceptionFullDrdPreserved
        );
        assert_eq!(output.allowed_drd_cents, 50_000_00);
        assert!(output.note.contains("§ 246(b)(2)"));
        assert!(output.note.contains("§ 172"));
    }

    #[test]
    fn twenty_to_eighty_pct_tier_65_pct_drd() {
        let mut input = base();
        input.ownership_tier = OwnershipTier::TwentyToSeventyNinePercent65PctDrd;
        let output = check(&input);
        assert_eq!(output.drd_percentage_basis_points, 6_500);
        // $100K × 65% = $65K; cap = $1M × 65% = $650K → full
        assert_eq!(output.allowed_drd_cents, 65_000_00);
    }

    #[test]
    fn eighty_pct_plus_affiliated_100_pct_drd_no_cap() {
        let mut input = base();
        input.ownership_tier = OwnershipTier::EightyPercentPlusAffiliated100PctDrdNoCap;
        input.taxable_income_before_drd_cents = 10_000_00; // Even with low TI
        let output = check(&input);
        // 80%+ tier has no TI cap; full 100% DRD
        assert_eq!(output.drd_percentage_basis_points, 10_000);
        assert_eq!(output.allowed_drd_cents, 100_000_00);
        assert!(output.note.contains("§ 243(b)"));
        assert!(output.note.contains("§ 1504"));
    }

    #[test]
    fn sanctioned_country_drd_denied() {
        let mut input = base();
        input.dividend_issuer_type =
            DividendIssuerType::SanctionedCountryForeignCorporation;
        let output = check(&input);
        assert_eq!(output.severity, Severity::Section246AExcludedIssuerDrdDenied);
        assert_eq!(output.allowed_drd_cents, 0);
        assert!(output.note.contains("§ 901(j)"));
    }

    #[test]
    fn section_501_tax_exempt_drd_denied() {
        let mut input = base();
        input.dividend_issuer_type =
            DividendIssuerType::Section501Or521TaxExemptCorporation;
        let output = check(&input);
        assert_eq!(output.severity, Severity::Section246AExcludedIssuerDrdDenied);
        assert!(output.note.contains("§ 501"));
        assert!(output.note.contains("§ 521"));
    }

    #[test]
    fn non_qualified_ric_distribution_drd_denied() {
        let mut input = base();
        input.dividend_issuer_type = DividendIssuerType::NonQualifiedRicDistribution;
        let output = check(&input);
        assert_eq!(output.severity, Severity::Section246AExcludedIssuerDrdDenied);
        assert!(output.note.contains("§ 854"));
    }

    #[test]
    fn non_capital_gain_reit_distribution_drd_denied() {
        let mut input = base();
        input.dividend_issuer_type = DividendIssuerType::NonCapitalGainReitDistribution;
        let output = check(&input);
        assert_eq!(output.severity, Severity::Section246AExcludedIssuerDrdDenied);
        assert!(output.note.contains("REIT"));
    }

    #[test]
    fn common_holding_period_constant_pins_46_days() {
        assert_eq!(COMMON_HOLDING_PERIOD_DAYS, 46);
    }

    #[test]
    fn common_testing_window_constant_pins_91_days() {
        assert_eq!(COMMON_TESTING_WINDOW_DAYS, 91);
    }

    #[test]
    fn preferred_extended_holding_period_constant_pins_91_days() {
        assert_eq!(PREFERRED_EXTENDED_HOLDING_PERIOD_DAYS, 91);
    }

    #[test]
    fn preferred_extended_testing_window_constant_pins_181_days() {
        assert_eq!(PREFERRED_EXTENDED_TESTING_WINDOW_DAYS, 181);
    }

    #[test]
    fn less_than_20_drd_constant_pins_50_pct() {
        assert_eq!(LESS_THAN_TWENTY_DRD_BPS, 5_000);
    }

    #[test]
    fn twenty_to_eighty_drd_constant_pins_65_pct() {
        assert_eq!(TWENTY_TO_EIGHTY_DRD_BPS, 6_500);
    }

    #[test]
    fn eighty_plus_drd_constant_pins_100_pct() {
        assert_eq!(EIGHTY_PLUS_DRD_BPS, 10_000);
    }

    #[test]
    fn very_large_dividend_no_overflow() {
        let mut input = base();
        input.dividend_received_cents = u64::MAX;
        input.taxable_income_before_drd_cents = u64::MAX;
        let output = check(&input);
        // u128 intermediate prevents overflow
        assert!(output.allowed_drd_cents > 0);
    }

    #[test]
    fn zero_dividend_zero_drd() {
        let mut input = base();
        input.dividend_received_cents = 0;
        let output = check(&input);
        assert_eq!(output.allowed_drd_cents, 0);
    }

    #[test]
    fn note_pins_section_246c4_tolling_substantially_diminished_risk() {
        let mut input = base();
        input.days_held_during_required_window = 30;
        let output = check(&input);
        assert!(output.note.contains("§ 246(c)(4)"));
        assert!(output.note.contains("substantially diminished risk"));
    }

    #[test]
    fn note_pins_section_172_nol_carryover() {
        let mut input = base();
        input.nol_status = NolStatus::NolYearTaxableIncomeCapInapplicable;
        let output = check(&input);
        assert!(output.note.contains("§ 172"));
    }
}
