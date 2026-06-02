//! IRC § 901 — Taxes of Foreign Countries and of Possessions of United
//! States.
//!
//! § 901 is the operative foreign tax credit (FTC) provision — it allows
//! taxpayers to claim a CREDIT (dollar-for-dollar reduction of US tax)
//! rather than a deduction for foreign income, war profits, and excess
//! profits taxes paid or accrued to foreign governments or US possessions.
//! § 904 (see [[section_904]] iter 516) caps the credit; § 901 establishes
//! the basic creditability.
//!
//! § 901(a) general rule: domestic corp + US citizen / resident alien
//! individuals may credit foreign income taxes against US tax. § 901(b)(1)
//! creditable for domestic corp shareholder of 10%-or-more foreign corp via
//! § 960 deemed-paid mechanism.
//!
//! § 901(j) SANCTIONED-COUNTRY DISALLOWANCE: foreign taxes imposed by
//! sanctioned countries are NOT creditable. Sanctioned-country list per
//! § 901(j)(2)(A) (Secretary of State designations) includes Iran + North
//! Korea + Syria + Cuba + Sudan (partial) + other state sponsors of
//! terrorism. § 901(j) income must be computed in SEPARATE FTC basket per
//! § 904(d). § 901(j)(5) treaty resourcing carve-out may permit partial
//! creditability.
//!
//! § 901(k) DIVIDEND HOLDING-PERIOD REQUIREMENT: foreign taxes withheld on
//! dividends are NOT creditable unless taxpayer holds stock for at least 16
//! days within the 31-day period beginning 15 days before the ex-dividend
//! date (§ 901(k)(1)(A)). For preferred-stock dividends attributable to
//! periods exceeding 366 days, the holding-period requirement is 46 days
//! within the 91-day period beginning 45 days before the ex-dividend date
//! per § 901(k)(3). This rule prevents FTC arbitrage via short-dated stock
//! purchases around ex-dividend dates.
//!
//! § 901(l) HOLDING-PERIOD REQUIREMENT FOR OTHER INCOME: foreign taxes on
//! non-dividend income subject to general holding-period requirement.
//!
//! § 901(m) COVERED ASSET ACQUISITION (CAA) FTC DISALLOWANCE: enacted by
//! Education Jobs and Medicaid Assistance Act of 2010 (Pub. L. 111-226);
//! disqualifies foreign taxes attributable to the "disqualified portion" of
//! a CAA — qualified stock purchase + § 338 election + § 754 partnership
//! basis adjustment + certain other acquisitions resulting in stepped-up
//! basis for US tax purposes but NOT for foreign tax purposes. Disqualified
//! portion = (US basis step-up amount / total tax basis) × foreign tax
//! paid. Notice 2014-44 + Notice 2014-45 + Treas. Reg. § 1.901(m)-1 through
//! § 1.901(m)-8 (proposed) implement.
//!
//! § 901(b)(5) US-possessions analogous credit (Puerto Rico, US Virgin
//! Islands, Guam, American Samoa, Northern Mariana Islands).
//!
//! Coordination: § 901 FTC limited by § 904 (separate baskets + 1-year
//! carryback + 10-year carryforward except for GILTI / NCTI basket which
//! has no carryover); § 901 / § 960 / § 902 (repealed by TCJA for taxable
//! years beginning after 2017) framework.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditabilityClassification {
    /// Foreign income/war profits/excess profits tax — creditable.
    CreditableForeignIncomeTax,
    /// Foreign tax in lieu of income tax — creditable under § 903.
    CreditableInLieuOfTax,
    /// Tax imposed by sanctioned country — non-creditable per § 901(j).
    NonCreditableSanctionedCountry,
    /// Foreign tax on dividend without holding-period satisfaction — non-
    /// creditable per § 901(k).
    NonCreditableDividendHoldingPeriodNotMet,
    /// Foreign tax attributable to disqualified portion of covered asset
    /// acquisition — non-creditable per § 901(m).
    NonCreditableCoveredAssetAcquisition,
    /// Penalty / interest / value-added tax — not creditable.
    NotCreditableOther,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DividendStockType {
    /// Common stock — 16-day holding-period rule.
    CommonStock,
    /// Preferred stock with dividends for ≤ 366-day period — 16-day rule.
    PreferredStockShortPeriod,
    /// Preferred stock with dividends for > 366-day period — 46-day rule.
    PreferredStockLongPeriod,
    /// Not a dividend (general income).
    NotDividend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    FullyCreditable,
    PartiallyCreditableCaaDisqualifiedPortion,
    NonCreditableSanctionedCountryFull,
    NonCreditableHoldingPeriodFailed,
    NonCreditableNonIncomeTax,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section901Input {
    pub classification: CreditabilityClassification,
    pub foreign_tax_paid_cents: u64,
    pub dividend_stock_type: DividendStockType,
    pub days_held_in_holding_period_window: u32,
    pub covered_asset_acquisition_us_basis_step_up_cents: u64,
    pub covered_asset_acquisition_total_basis_cents: u64,
    pub taxable_year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section901Result {
    pub severity: Severity,
    pub creditable_foreign_tax_cents: u64,
    pub disqualified_caa_portion_cents: u64,
    pub holding_period_required_days: u32,
    pub holding_period_window_days: u32,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const SECTION_901K_COMMON_STOCK_HOLDING_DAYS: u32 = 16;
pub const SECTION_901K_COMMON_STOCK_WINDOW_DAYS: u32 = 31;
pub const SECTION_901K_PREFERRED_LONG_HOLDING_DAYS: u32 = 46;
pub const SECTION_901K_PREFERRED_LONG_WINDOW_DAYS: u32 = 91;
pub const SECTION_901K_PREFERRED_PERIOD_THRESHOLD_DAYS: u32 = 366;
pub const SECTION_901M_ENACTED_PUB_L: &str = "Pub. L. 111-226";
pub const SECTION_901M_ENACTED_YEAR: i32 = 2010;

pub fn check(input: &Section901Input) -> Section901Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(input.classification, CreditabilityClassification::NotCreditableOther) {
        notes.push(
            "Foreign tax is NOT an income / war profits / excess profits tax under § 901(b) \
             or an in-lieu-of tax under § 903 — non-creditable. Penalty payments, interest, \
             value-added taxes (VAT), customs duties, and gross-receipts taxes generally \
             non-creditable. Tax may still be DEDUCTIBLE as ordinary and necessary business \
             expense under § 164(a)(3) in lieu of credit."
                .to_string(),
        );
        return empty_result(
            Severity::NonCreditableNonIncomeTax,
            input,
            actions,
            notes,
            "26 U.S.C. § 901(b); § 903; § 164(a)(3)",
        );
    }

    if matches!(
        input.classification,
        CreditabilityClassification::NonCreditableSanctionedCountry
    ) {
        notes.push(
            "Foreign tax imposed by sanctioned country (Iran + North Korea + Syria + Cuba + \
             Sudan partial + other state sponsors of terrorism per Secretary of State \
             designation under § 901(j)(2)(A)) is non-creditable. § 901(j) income computed \
             in SEPARATE § 904(d) basket. § 901(j)(5) treaty-resourcing carve-out may \
             permit partial creditability if income re-sourced under qualifying treaty."
                .to_string(),
        );
        return empty_result(
            Severity::NonCreditableSanctionedCountryFull,
            input,
            actions,
            notes,
            "26 U.S.C. § 901(j); § 904(d)",
        );
    }

    if matches!(
        input.classification,
        CreditabilityClassification::NonCreditableDividendHoldingPeriodNotMet
    ) {
        let (required_days, window_days) = match input.dividend_stock_type {
            DividendStockType::CommonStock | DividendStockType::PreferredStockShortPeriod => (
                SECTION_901K_COMMON_STOCK_HOLDING_DAYS,
                SECTION_901K_COMMON_STOCK_WINDOW_DAYS,
            ),
            DividendStockType::PreferredStockLongPeriod => (
                SECTION_901K_PREFERRED_LONG_HOLDING_DAYS,
                SECTION_901K_PREFERRED_LONG_WINDOW_DAYS,
            ),
            DividendStockType::NotDividend => (0, 0),
        };

        if input.days_held_in_holding_period_window < required_days && required_days > 0 {
            actions.push(format!(
                "§ 901(k) holding-period requirement NOT satisfied — stock held only {} \
                 days in {}-day window; required {} days within window beginning {} days \
                 before ex-dividend date. Foreign withholding tax on dividend NON-\
                 CREDITABLE. Consider claim as deduction under § 164(a)(3) in lieu of \
                 credit; deduction yields lower benefit (top marginal rate × tax) than \
                 credit (dollar-for-dollar offset).",
                input.days_held_in_holding_period_window,
                window_days,
                required_days,
                if required_days == SECTION_901K_PREFERRED_LONG_HOLDING_DAYS {
                    SECTION_901K_PREFERRED_LONG_WINDOW_DAYS / 2 + 1
                } else {
                    SECTION_901K_COMMON_STOCK_WINDOW_DAYS / 2 + 1
                }
            ));
            return Section901Result {
                severity: Severity::NonCreditableHoldingPeriodFailed,
                creditable_foreign_tax_cents: 0,
                disqualified_caa_portion_cents: 0,
                holding_period_required_days: required_days,
                holding_period_window_days: window_days,
                recommended_actions: actions,
                citation: "26 U.S.C. § 901(k)(1)-(3); § 164(a)(3)",
                notes: build_coordination_notes(notes),
            };
        }
    }

    if matches!(
        input.classification,
        CreditabilityClassification::NonCreditableCoveredAssetAcquisition
    ) {
        let total = input.covered_asset_acquisition_total_basis_cents.max(1);
        let disqualified_portion: u64 = (u128::from(input.foreign_tax_paid_cents)
            * u128::from(input.covered_asset_acquisition_us_basis_step_up_cents)
            / u128::from(total)) as u64;
        let creditable = input.foreign_tax_paid_cents.saturating_sub(disqualified_portion);
        actions.push(format!(
            "§ 901(m) Covered Asset Acquisition (CAA) FTC disallowance: foreign taxes \
             attributable to the disqualified portion are NON-CREDITABLE. Disqualified \
             portion = (US basis step-up of {} cents / total basis of {} cents) × foreign \
             tax of {} cents = {} cents non-creditable; remaining {} cents creditable. \
             Enacted by {} ({}); Notice 2014-44 + Notice 2014-45 + Treas. Reg. § 1.901(m)-1 \
             through § 1.901(m)-8 (proposed) implement.",
            input.covered_asset_acquisition_us_basis_step_up_cents,
            input.covered_asset_acquisition_total_basis_cents,
            input.foreign_tax_paid_cents,
            disqualified_portion,
            creditable,
            SECTION_901M_ENACTED_PUB_L,
            SECTION_901M_ENACTED_YEAR
        ));
        return Section901Result {
            severity: Severity::PartiallyCreditableCaaDisqualifiedPortion,
            creditable_foreign_tax_cents: creditable,
            disqualified_caa_portion_cents: disqualified_portion,
            holding_period_required_days: 0,
            holding_period_window_days: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 901(m); Pub. L. 111-226; Notice 2014-44; Notice 2014-45",
            notes: build_coordination_notes(notes),
        };
    }

    actions.push(format!(
        "Foreign tax of {} cents fully creditable under § 901(a)-(b). Apply § 904 basket-\
         specific limitation (see [[section_904]] iter 516); report on Form 1116 (individual) \
         or Form 1118 (corporate). Foreign tax credit superior to deduction — dollar-for-\
         dollar US tax offset vs. deduction yielding only marginal-rate × foreign tax \
         benefit.",
        input.foreign_tax_paid_cents
    ));

    Section901Result {
        severity: Severity::FullyCreditable,
        creditable_foreign_tax_cents: input.foreign_tax_paid_cents,
        disqualified_caa_portion_cents: 0,
        holding_period_required_days: 0,
        holding_period_window_days: 0,
        recommended_actions: actions,
        citation: "26 U.S.C. § 901(a)-(b); § 903; § 904",
        notes: build_coordination_notes(notes),
    }
}

fn build_coordination_notes(mut notes: Vec<String>) -> Vec<String> {
    notes.push(
        "Coordination with [[section_904]] (FTC limitation — iter 516 — separate baskets + \
         1-year carryback + 10-year carryforward except GILTI/NCTI), [[section_960]] \
         (deemed-paid FTC for CFC inclusions — pre-OBBBA 80% / post-OBBBA 90%), \
         [[section_903]] (in-lieu-of tax — extends § 901 creditability), [[section_245a]] \
         (foreign-source DRD — iter 502 — § 245A(d) DISALLOWS FTC for foreign tax on DRD-\
         eligible dividend), [[section_951a]] (GILTI / NCTI — iter 500), [[section_956]] \
         (CFC US property — iter 504), [[section_959]] (PTEP — iter 512), [[section_962]] \
         (individual election — iter 510 — unlocks § 960 FTC for individuals), \
         [[section_965]] (transition tax — iter 514 — § 965(g) FTC denial percentage), \
         [[section_164]] (foreign-tax deduction alternative to credit under § 164(a)(3))."
            .to_string(),
    );
    notes
}

fn empty_result(
    severity: Severity,
    input: &Section901Input,
    recommended_actions: Vec<String>,
    notes: Vec<String>,
    citation: &'static str,
) -> Section901Result {
    let _ = input;
    Section901Result {
        severity,
        creditable_foreign_tax_cents: 0,
        disqualified_caa_portion_cents: 0,
        holding_period_required_days: 0,
        holding_period_window_days: 0,
        recommended_actions,
        citation,
        notes: build_coordination_notes(notes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section901Input {
        Section901Input {
            classification: CreditabilityClassification::CreditableForeignIncomeTax,
            foreign_tax_paid_cents: 1_000_000_00,
            dividend_stock_type: DividendStockType::NotDividend,
            days_held_in_holding_period_window: 0,
            covered_asset_acquisition_us_basis_step_up_cents: 0,
            covered_asset_acquisition_total_basis_cents: 0,
            taxable_year: 2024,
        }
    }

    #[test]
    fn fully_creditable_foreign_income_tax() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullyCreditable));
        assert_eq!(r.creditable_foreign_tax_cents, i.foreign_tax_paid_cents);
    }

    #[test]
    fn in_lieu_of_tax_also_creditable() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::CreditableInLieuOfTax;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullyCreditable));
    }

    #[test]
    fn sanctioned_country_non_creditable() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableSanctionedCountry;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NonCreditableSanctionedCountryFull));
        assert_eq!(r.creditable_foreign_tax_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("Iran")));
        assert!(r.notes.iter().any(|n| n.contains("North Korea")));
        assert!(r.notes.iter().any(|n| n.contains("§ 901(j)(2)(A)")));
    }

    #[test]
    fn non_income_tax_not_creditable_but_deductible() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NotCreditableOther;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NonCreditableNonIncomeTax));
        assert!(r.notes.iter().any(|n| n.contains("§ 164(a)(3)")));
        assert!(r.notes.iter().any(|n| n.contains("VAT")));
    }

    #[test]
    fn common_stock_holding_period_failed_non_creditable() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableDividendHoldingPeriodNotMet;
        i.dividend_stock_type = DividendStockType::CommonStock;
        i.days_held_in_holding_period_window = 10;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NonCreditableHoldingPeriodFailed));
        assert_eq!(r.holding_period_required_days, 16);
        assert_eq!(r.holding_period_window_days, 31);
    }

    #[test]
    fn common_stock_holding_period_satisfied_at_16_days_creditable() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableDividendHoldingPeriodNotMet;
        i.dividend_stock_type = DividendStockType::CommonStock;
        i.days_held_in_holding_period_window = 16;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullyCreditable));
    }

    #[test]
    fn preferred_long_period_holding_period_required_46_days() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableDividendHoldingPeriodNotMet;
        i.dividend_stock_type = DividendStockType::PreferredStockLongPeriod;
        i.days_held_in_holding_period_window = 30;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NonCreditableHoldingPeriodFailed));
        assert_eq!(r.holding_period_required_days, 46);
        assert_eq!(r.holding_period_window_days, 91);
    }

    #[test]
    fn preferred_short_period_uses_16_day_rule() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableDividendHoldingPeriodNotMet;
        i.dividend_stock_type = DividendStockType::PreferredStockShortPeriod;
        i.days_held_in_holding_period_window = 10;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NonCreditableHoldingPeriodFailed));
        assert_eq!(r.holding_period_required_days, 16);
    }

    #[test]
    fn covered_asset_acquisition_partial_creditability() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableCoveredAssetAcquisition;
        i.foreign_tax_paid_cents = 1_000_000_00;
        i.covered_asset_acquisition_us_basis_step_up_cents = 300_000_00;
        i.covered_asset_acquisition_total_basis_cents = 1_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::PartiallyCreditableCaaDisqualifiedPortion
        ));
        let expected_disqualified = 1_000_000_00u64 * 300_000_00 / 1_000_000_00;
        assert_eq!(r.disqualified_caa_portion_cents, expected_disqualified);
        assert_eq!(
            r.creditable_foreign_tax_cents,
            1_000_000_00 - expected_disqualified
        );
        assert!(r.recommended_actions.iter().any(|a| a.contains("Notice 2014-44")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Pub. L. 111-226")));
    }

    #[test]
    fn caa_with_full_step_up_fully_disqualifies() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableCoveredAssetAcquisition;
        i.foreign_tax_paid_cents = 1_000_000_00;
        i.covered_asset_acquisition_us_basis_step_up_cents = 1_000_000_00;
        i.covered_asset_acquisition_total_basis_cents = 1_000_000_00;
        let r = check(&i);
        assert_eq!(r.disqualified_caa_portion_cents, 1_000_000_00);
        assert_eq!(r.creditable_foreign_tax_cents, 0);
    }

    #[test]
    fn caa_with_zero_step_up_fully_creditable() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableCoveredAssetAcquisition;
        i.foreign_tax_paid_cents = 1_000_000_00;
        i.covered_asset_acquisition_us_basis_step_up_cents = 0;
        i.covered_asset_acquisition_total_basis_cents = 1_000_000_00;
        let r = check(&i);
        assert_eq!(r.disqualified_caa_portion_cents, 0);
        assert_eq!(r.creditable_foreign_tax_cents, 1_000_000_00);
    }

    #[test]
    fn caa_zero_total_basis_does_not_panic() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableCoveredAssetAcquisition;
        i.covered_asset_acquisition_total_basis_cents = 0;
        let r = check(&i);
        let _ = r.creditable_foreign_tax_cents;
    }

    #[test]
    fn section_901k_common_holding_pins_16_days() {
        assert_eq!(SECTION_901K_COMMON_STOCK_HOLDING_DAYS, 16);
    }

    #[test]
    fn section_901k_common_window_pins_31_days() {
        assert_eq!(SECTION_901K_COMMON_STOCK_WINDOW_DAYS, 31);
    }

    #[test]
    fn section_901k_preferred_long_holding_pins_46_days() {
        assert_eq!(SECTION_901K_PREFERRED_LONG_HOLDING_DAYS, 46);
    }

    #[test]
    fn section_901k_preferred_long_window_pins_91_days() {
        assert_eq!(SECTION_901K_PREFERRED_LONG_WINDOW_DAYS, 91);
    }

    #[test]
    fn section_901k_preferred_threshold_pins_366_days() {
        assert_eq!(SECTION_901K_PREFERRED_PERIOD_THRESHOLD_DAYS, 366);
    }

    #[test]
    fn section_901m_enacted_year_pins_2010() {
        assert_eq!(SECTION_901M_ENACTED_YEAR, 2010);
    }

    #[test]
    fn section_901m_enacted_pub_l_pins_111_226() {
        assert_eq!(SECTION_901M_ENACTED_PUB_L, "Pub. L. 111-226");
    }

    #[test]
    fn action_references_form_1116_and_1118() {
        let i = baseline();
        let r = check(&i);
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 1116")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 1118")));
    }

    #[test]
    fn coordination_note_references_904_960_245a_951a_956_959_962_965() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_904")));
        assert!(r.notes.iter().any(|n| n.contains("section_960")));
        assert!(r.notes.iter().any(|n| n.contains("section_245a")));
        assert!(r.notes.iter().any(|n| n.contains("section_951a")));
        assert!(r.notes.iter().any(|n| n.contains("section_956")));
        assert!(r.notes.iter().any(|n| n.contains("section_959")));
        assert!(r.notes.iter().any(|n| n.contains("section_962")));
        assert!(r.notes.iter().any(|n| n.contains("section_965")));
    }

    #[test]
    fn citation_pins_901_904_for_creditable() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 901(a)-(b)"));
        assert!(r.citation.contains("§ 904"));
    }

    #[test]
    fn citation_901j_for_sanctioned() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableSanctionedCountry;
        let r = check(&i);
        assert!(r.citation.contains("§ 901(j)"));
    }

    #[test]
    fn citation_901k_for_holding_period_failed() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableDividendHoldingPeriodNotMet;
        i.dividend_stock_type = DividendStockType::CommonStock;
        i.days_held_in_holding_period_window = 5;
        let r = check(&i);
        assert!(r.citation.contains("§ 901(k)"));
    }

    #[test]
    fn citation_901m_for_caa() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableCoveredAssetAcquisition;
        i.covered_asset_acquisition_total_basis_cents = 100;
        let r = check(&i);
        assert!(r.citation.contains("§ 901(m)"));
    }

    #[test]
    fn realistic_individual_foreign_dividend_holding_failed() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableDividendHoldingPeriodNotMet;
        i.dividend_stock_type = DividendStockType::CommonStock;
        i.days_held_in_holding_period_window = 12;
        i.foreign_tax_paid_cents = 50_000_00;
        let r = check(&i);
        assert_eq!(r.creditable_foreign_tax_cents, 0);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("deduction under § 164(a)(3)")));
    }

    #[test]
    fn extreme_value_does_not_overflow() {
        let mut i = baseline();
        i.classification = CreditabilityClassification::NonCreditableCoveredAssetAcquisition;
        i.foreign_tax_paid_cents = u64::MAX / 100;
        i.covered_asset_acquisition_us_basis_step_up_cents = u64::MAX / 200;
        i.covered_asset_acquisition_total_basis_cents = u64::MAX / 100;
        let r = check(&i);
        let _ = r.creditable_foreign_tax_cents;
    }

    #[test]
    fn zero_foreign_tax_zero_credit() {
        let mut i = baseline();
        i.foreign_tax_paid_cents = 0;
        let r = check(&i);
        assert_eq!(r.creditable_foreign_tax_cents, 0);
    }
}
