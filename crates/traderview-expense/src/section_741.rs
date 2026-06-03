//! IRC § 741 recognition and character of gain or loss on sale or
//! exchange of partnership interest.
//!
//! Foundational Subchapter K character-classification provision that
//! pairs with `section_751` (iter 572 hot assets), `section_752`
//! (partnership liabilities — liability relief treated as deemed
//! distribution under § 752(d) which becomes part of amount
//! realized), `section_706` (iter 584 — partnership year closes
//! under § 706(c)(2) on entire-interest transfer triggering § 741),
//! `section_736` (iter 582 — distinguishes § 741 sale from § 736
//! redemption), `section_743` (iter 576 — transferee basis adjustment
//! downstream of § 741 sale).
//!
//! **Operative rule** (§ 741): "In the case of a sale or exchange
//! of an interest in a partnership, gain or loss shall be recognized
//! to the transferor partner. Such gain or loss shall be considered
//! as gain or loss from the sale or exchange of a capital asset,
//! except as otherwise provided in section 751 (relating to
//! unrealized receivables and inventory items)."
//!
//! **Three-step computation**:
//!
//! Amount Realized equals cash received plus FMV of property received
//! plus liability relief (§ 752 treats decrease in partner's share of
//! partnership liabilities as deemed cash distribution that becomes
//! part of amount realized on sale). Adjusted Basis equals transferor's
//! outside basis under § 705 (initial basis plus distributive shares
//! plus contributions minus money plus property distributions minus
//! liabilities). Gain or Loss equals Amount Realized minus Adjusted
//! Basis.
//!
//! **Character**:
//!
//! - Default: capital character under § 741.
//! - Holding period > 1 year (§ 1223 + § 1(h)): long-term capital
//!   gain (0%/15%/20% preferred federal rates).
//! - Holding period ≤ 1 year: short-term capital gain (ordinary
//!   rates up to 37%).
//! - § 751(a) override: portion of amount realized attributable to
//!   unrealized receivables (§ 751(c)) + inventory items (§ 751(d))
//!   is treated as ORDINARY income; remainder is § 741 capital.
//!
//! **Common compliance failures**:
//!
//! 1. **Failure to include liability relief in amount realized**:
//!    § 752(d) deems liability relief a cash distribution that adds
//!    to amount realized. Taxpayers often forget to add the share-
//!    of-partnership-liabilities relieved on sale.
//! 2. **Failure to bifurcate § 751 ordinary portion**: § 741
//!    character analysis is DEFAULT; § 751(a) override is
//!    AUTOMATIC; taxpayer who reports entire gain as capital when
//!    partnership holds substantial § 751 hot assets has
//!    misreported character.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const LONG_TERM_HOLDING_PERIOD_DAYS: u32 = 366;
#[allow(dead_code)]
pub const LONG_TERM_HOLDING_PERIOD_MONTHS: u32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    LongTermCapitalGainPureNoHotAssets,
    ShortTermCapitalGainPureNoHotAssets,
    LongTermCapitalLossPureNoHotAssets,
    ShortTermCapitalLossPureNoHotAssets,
    BifurcatedSection751OrdinaryPlusSection741Capital,
    ViolationTaxpayerReportedAllCapitalIgnoringSection751,
    ViolationFailedToIncludeLiabilityReliefInAmountRealized,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub transferor_outside_basis_cents: u64,
    pub cash_received_cents: u64,
    pub fmv_of_property_received_cents: u64,
    pub liability_relief_cents: u64,
    pub holding_period_days: u32,
    pub section_751_amount_realized_portion_cents: u64,
    pub section_751_basis_portion_cents: u64,
    pub taxpayer_included_liability_relief: bool,
    pub taxpayer_reported_all_capital_no_751_recognition: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub amount_realized_cents: u64,
    pub gain_or_loss_cents: i128,
    pub section_751_ordinary_portion_cents: i128,
    pub section_741_capital_portion_cents: i128,
    pub character_long_term: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section741Input = Input;
pub type Section741Output = Output;
pub type Section741Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 741 (recognition and character of gain or loss on partnership-interest sale)".to_string(),
        "IRC § 705 (outside basis determination)".to_string(),
        "IRC § 751(a) (ordinary character carve-out for unrealized receivables + inventory items)".to_string(),
        "IRC § 751(c) (unrealized receivables definition)".to_string(),
        "IRC § 751(d) (inventory items definition)".to_string(),
        "IRC § 752(d) (liability relief treated as deemed distribution on sale)".to_string(),
        "IRC § 1223 (holding period determination)".to_string(),
        "IRC § 1(h) (long-term capital gain preferred rates: 0/15/20%)".to_string(),
        "IRC § 706(c)(2) (partnership year closes for transferor on entire-interest sale)".to_string(),
        "Treas. Reg. § 1.741-1 (recognition and character)".to_string(),
        "Treas. Reg. § 1.752-1 (treatment of partnership liabilities)".to_string(),
    ];

    let amount_realized_partner_reported = input
        .cash_received_cents
        .saturating_add(input.fmv_of_property_received_cents);
    let amount_realized_full = amount_realized_partner_reported.saturating_add(input.liability_relief_cents);

    if input.liability_relief_cents > 0 && !input.taxpayer_included_liability_relief {
        notes.push(format!(
            "§ 752(d) requires liability relief ${} added to amount realized; taxpayer omitted, understating amount realized by ${}.",
            input.liability_relief_cents / 100,
            input.liability_relief_cents / 100
        ));
        return Output {
            severity: Severity::ViolationFailedToIncludeLiabilityReliefInAmountRealized,
            amount_realized_cents: amount_realized_full,
            gain_or_loss_cents: (amount_realized_full as i128)
                - (input.transferor_outside_basis_cents as i128),
            section_751_ordinary_portion_cents: 0,
            section_741_capital_portion_cents: 0,
            character_long_term: input.holding_period_days >= LONG_TERM_HOLDING_PERIOD_DAYS,
            notes,
            citations,
        };
    }

    let total_gain_loss: i128 = (amount_realized_full as i128)
        - (input.transferor_outside_basis_cents as i128);

    let hot_assets_present = input.section_751_amount_realized_portion_cents > 0
        || input.section_751_basis_portion_cents > 0;

    if hot_assets_present && input.taxpayer_reported_all_capital_no_751_recognition {
        let section_751_ordinary = (input.section_751_amount_realized_portion_cents as i128)
            - (input.section_751_basis_portion_cents as i128);
        let section_741_capital = total_gain_loss - section_751_ordinary;
        notes.push(format!(
            "§ 751(a) requires bifurcation: ${} ordinary (hot-asset portion) + ${} capital; taxpayer reported full ${} as capital — character misreported.",
            section_751_ordinary / 100,
            section_741_capital / 100,
            total_gain_loss / 100
        ));
        return Output {
            severity: Severity::ViolationTaxpayerReportedAllCapitalIgnoringSection751,
            amount_realized_cents: amount_realized_full,
            gain_or_loss_cents: total_gain_loss,
            section_751_ordinary_portion_cents: section_751_ordinary,
            section_741_capital_portion_cents: section_741_capital,
            character_long_term: input.holding_period_days >= LONG_TERM_HOLDING_PERIOD_DAYS,
            notes,
            citations,
        };
    }

    let character_long_term = input.holding_period_days >= LONG_TERM_HOLDING_PERIOD_DAYS;

    if hot_assets_present {
        let section_751_ordinary = (input.section_751_amount_realized_portion_cents as i128)
            - (input.section_751_basis_portion_cents as i128);
        let section_741_capital = total_gain_loss - section_751_ordinary;
        notes.push(format!(
            "§ 741 bifurcation: ${} ordinary (§ 751(a) hot assets) + ${} {}-term capital (§ 741).",
            section_751_ordinary / 100,
            section_741_capital / 100,
            if character_long_term { "long" } else { "short" }
        ));
        return Output {
            severity: Severity::BifurcatedSection751OrdinaryPlusSection741Capital,
            amount_realized_cents: amount_realized_full,
            gain_or_loss_cents: total_gain_loss,
            section_751_ordinary_portion_cents: section_751_ordinary,
            section_741_capital_portion_cents: section_741_capital,
            character_long_term,
            notes,
            citations,
        };
    }

    let severity = if total_gain_loss > 0 && character_long_term {
        notes.push(format!(
            "§ 741 long-term capital gain ${}; no hot assets; § 1(h) preferred 0/15/20% rates apply.",
            total_gain_loss / 100
        ));
        Severity::LongTermCapitalGainPureNoHotAssets
    } else if total_gain_loss > 0 {
        notes.push(format!(
            "§ 741 short-term capital gain ${}; no hot assets; ordinary rates up to 37%.",
            total_gain_loss / 100
        ));
        Severity::ShortTermCapitalGainPureNoHotAssets
    } else if total_gain_loss < 0 && character_long_term {
        notes.push(format!(
            "§ 741 long-term capital loss ${}; subject to § 1211(b) $3,000 cap absent § 475(f) MTM election.",
            (-total_gain_loss) / 100
        ));
        Severity::LongTermCapitalLossPureNoHotAssets
    } else if total_gain_loss < 0 {
        notes.push(format!(
            "§ 741 short-term capital loss ${}; subject to § 1211(b) $3,000 cap.",
            (-total_gain_loss) / 100
        ));
        Severity::ShortTermCapitalLossPureNoHotAssets
    } else {
        notes.push("§ 741 sale produced no gain or loss (amount realized = outside basis).".to_string());
        Severity::NotApplicable
    };

    Output {
        severity,
        amount_realized_cents: amount_realized_full,
        gain_or_loss_cents: total_gain_loss,
        section_751_ordinary_portion_cents: 0,
        section_741_capital_portion_cents: total_gain_loss,
        character_long_term,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_long_term_capital_gain() -> Input {
        Input {
            transferor_outside_basis_cents: 5_000_000,
            cash_received_cents: 10_000_000,
            fmv_of_property_received_cents: 0,
            liability_relief_cents: 0,
            holding_period_days: 1_000,
            section_751_amount_realized_portion_cents: 0,
            section_751_basis_portion_cents: 0,
            taxpayer_included_liability_relief: true,
            taxpayer_reported_all_capital_no_751_recognition: false,
        }
    }

    #[test]
    fn long_term_capital_gain_pure() {
        let out = check(&base_long_term_capital_gain());
        assert_eq!(out.severity, Severity::LongTermCapitalGainPureNoHotAssets);
        assert_eq!(out.gain_or_loss_cents, 5_000_000);
        assert!(out.character_long_term);
    }

    #[test]
    fn short_term_capital_gain_when_holding_period_under_366_days() {
        let mut i = base_long_term_capital_gain();
        i.holding_period_days = 200;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ShortTermCapitalGainPureNoHotAssets);
        assert!(!out.character_long_term);
    }

    #[test]
    fn long_term_boundary_exactly_366_days_qualifies() {
        let mut i = base_long_term_capital_gain();
        i.holding_period_days = 366;
        let out = check(&i);
        assert!(out.character_long_term);
    }

    #[test]
    fn long_term_boundary_365_days_does_not_qualify() {
        let mut i = base_long_term_capital_gain();
        i.holding_period_days = 365;
        let out = check(&i);
        assert!(!out.character_long_term);
    }

    #[test]
    fn long_term_capital_loss_when_basis_exceeds_amount_realized() {
        let mut i = base_long_term_capital_gain();
        i.cash_received_cents = 2_000_000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::LongTermCapitalLossPureNoHotAssets);
        assert_eq!(out.gain_or_loss_cents, -3_000_000);
    }

    #[test]
    fn short_term_capital_loss_under_366_days() {
        let mut i = base_long_term_capital_gain();
        i.cash_received_cents = 2_000_000;
        i.holding_period_days = 100;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ShortTermCapitalLossPureNoHotAssets);
    }

    #[test]
    fn liability_relief_added_to_amount_realized() {
        let mut i = base_long_term_capital_gain();
        i.cash_received_cents = 8_000_000;
        i.liability_relief_cents = 2_000_000;
        i.taxpayer_included_liability_relief = true;
        let out = check(&i);
        assert_eq!(out.amount_realized_cents, 10_000_000);
        assert_eq!(out.gain_or_loss_cents, 5_000_000);
    }

    #[test]
    fn failure_to_include_liability_relief_violation() {
        let mut i = base_long_term_capital_gain();
        i.liability_relief_cents = 2_000_000;
        i.taxpayer_included_liability_relief = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFailedToIncludeLiabilityReliefInAmountRealized
        );
    }

    #[test]
    fn bifurcated_751_ordinary_plus_741_capital() {
        let mut i = base_long_term_capital_gain();
        i.section_751_amount_realized_portion_cents = 3_000_000;
        i.section_751_basis_portion_cents = 1_500_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::BifurcatedSection751OrdinaryPlusSection741Capital
        );
        assert_eq!(out.section_751_ordinary_portion_cents, 1_500_000);
        assert_eq!(out.section_741_capital_portion_cents, 3_500_000);
    }

    #[test]
    fn taxpayer_ignored_751_reported_all_capital_violation() {
        let mut i = base_long_term_capital_gain();
        i.section_751_amount_realized_portion_cents = 3_000_000;
        i.section_751_basis_portion_cents = 1_500_000;
        i.taxpayer_reported_all_capital_no_751_recognition = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationTaxpayerReportedAllCapitalIgnoringSection751
        );
        assert_eq!(out.section_751_ordinary_portion_cents, 1_500_000);
    }

    #[test]
    fn zero_gain_loss_when_amount_realized_equals_basis() {
        let mut i = base_long_term_capital_gain();
        i.cash_received_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(out.gain_or_loss_cents, 0);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn citations_pin_741_751a_752d_subsections() {
        let out = check(&base_long_term_capital_gain());
        assert!(out.citations.iter().any(|c| c.contains("§ 741")));
        assert!(out.citations.iter().any(|c| c.contains("§ 751(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 752(d)")));
    }

    #[test]
    fn citations_pin_705_1223_1h_cross_refs() {
        let out = check(&base_long_term_capital_gain());
        assert!(out.citations.iter().any(|c| c.contains("§ 705")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1223")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1(h)")));
    }

    #[test]
    fn citations_pin_treas_reg_1_741_1_and_1_752_1() {
        let out = check(&base_long_term_capital_gain());
        assert!(out.citations.iter().any(|c| c.contains("§ 1.741-1")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.752-1")));
    }

    #[test]
    fn constant_pin_366_day_long_term_threshold() {
        assert_eq!(LONG_TERM_HOLDING_PERIOD_DAYS, 366);
    }

    #[test]
    fn constant_pin_12_month_long_term_threshold() {
        assert_eq!(LONG_TERM_HOLDING_PERIOD_MONTHS, 12);
    }

    #[test]
    fn very_large_amount_realized_no_overflow() {
        let mut i = base_long_term_capital_gain();
        i.cash_received_cents = u64::MAX;
        let out = check(&i);
        assert_eq!(out.amount_realized_cents, u64::MAX);
    }

    #[test]
    fn fmv_of_property_received_added_to_amount_realized() {
        let mut i = base_long_term_capital_gain();
        i.cash_received_cents = 6_000_000;
        i.fmv_of_property_received_cents = 4_000_000;
        let out = check(&i);
        assert_eq!(out.amount_realized_cents, 10_000_000);
        assert_eq!(out.gain_or_loss_cents, 5_000_000);
    }
}
