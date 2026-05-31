//! IRC §871(m) — Dividend-equivalent withholding for non-US persons.
//!
//! Pre-§871(m), non-US persons used total-return swaps and other
//! equity derivatives to receive dividend-equivalent payments on US
//! stocks WITHOUT triggering the 30% FDAP withholding under §871(a)
//! or §881 that an actual dividend would trigger. Congress and
//! Treasury closed the loophole through §871(m) (enacted 2010,
//! effective 2014 for swaps, 2017 for listed options).
//!
//! Companion to iter 32's `section_864b2`: that module handles the
//! non-US trader's own-account safe harbor (avoiding ECI
//! classification); this module handles the dividend-equivalent
//! withholding the broker / counterparty imposes on derivative
//! payments regardless of safe-harbor status. The two analyses are
//! independent.
//!
//! **Specified Equity-Linked Instrument** (SELI) per
//! Reg. §1.871-15(e): a contract that references one or more
//! dividend-paying US equities. Examples: equity options, single-
//! stock futures, total return swaps, notional principal contracts,
//! equity-linked notes.
//!
//! **Delta test** per §871(m)(3) + Reg. §1.871-15(g):
//!
//!   * **Short-term contracts** (original term ≤ 1 year), effective
//!     2017+: subject when **delta ≥ 0.80** at issuance. Classic
//!     near-the-money equity options pass; deep OTM doesn't.
//!
//!   * **Long-term contracts** (original term > 1 year): subject
//!     only when **delta = 1.0** at issuance. Deep-ITM LEAPS that
//!     functionally hold the stock are caught; standard delta-0.6
//!     LEAPS are not.
//!
//! **Statutory rate** is **30%** under §871(a)(1)(A) / §881(a)(1).
//! Tax treaties typically REDUCE the rate to 15% (US-Canada,
//! US-UK, US-Germany, US-Japan, US-Switzerland, US-Netherlands)
//! when the recipient files Form W-8BEN with the broker.
//!
//! Withholding agent per §1441 + §1.871-15(p): the broker /
//! counterparty paying the dividend-equivalent must withhold and
//! remit. The non-US person doesn't compute their own withholding;
//! they just see the net amount in the account. This module helps
//! the recipient verify the withheld amount matches statute.
//!
//! Pure compute. Caller asserts the contract facts; we classify
//! SELI status, apply the delta + term tests, and apply the
//! treaty-rate or statutory rate.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentType {
    /// Listed equity option (single-stock or index). Subject to
    /// §871(m) starting 2017 if delta + term tests pass.
    ListedEquityOption,
    /// Single-stock or narrow-based index future.
    SingleStockFuture,
    /// Equity-linked total return swap or notional principal contract.
    /// Subject from 2014.
    TotalReturnSwap,
    /// Equity-linked structured note.
    StructuredNote,
    /// Other equity-linked instrument.
    OtherEquityLinked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section871MInput {
    /// Recipient is a non-US person (foreign individual, foreign
    /// corp). §871(m) doesn't apply to US persons.
    pub non_us_person: bool,
    pub instrument_type: InstrumentType,
    /// Delta at the contract's issuance per Reg. §1.871-15(g)(2).
    /// 0.0 ≤ delta ≤ 1.0.
    pub delta_at_pricing: Decimal,
    /// Original term of the contract in days from issuance to
    /// expiration. Determines short-term (≤ 365) vs long-term
    /// (> 365) treatment.
    pub original_term_days: u32,
    /// Underlying US equity pays dividends. Non-dividend-paying
    /// stock obviates §871(m) entirely — there's no dividend-
    /// equivalent payment to withhold on.
    pub underlying_pays_dividends: bool,
    /// Dollar amount of the dividend-equivalent payment referenced
    /// by the contract this period (typically computed by broker
    /// based on delta × notional × per-share dividend).
    pub dividend_equivalent_amount: Decimal,
    /// Treaty-reduced withholding rate (0..1 scale). When None, the
    /// statutory 30% rate applies. Set to 0.15 for most major
    /// US treaties; 0.0 if the treaty fully exempts dividend
    /// equivalents (rare).
    pub treaty_rate_override: Option<Decimal>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section871MResult {
    pub subject_to_871m: bool,
    pub delta_threshold_applied: Decimal,
    pub applicable_rate: Decimal,
    pub withholding_amount: Decimal,
    pub net_dividend_equivalent_to_recipient: Decimal,
    pub reasons: Vec<String>,
    pub note: String,
}

fn statutory_rate() -> Decimal {
    Decimal::from_str("0.30").unwrap()
}

pub fn compute(input: &Section871MInput) -> Section871MResult {
    let mut r = Section871MResult::default();

    if !input.non_us_person {
        r.reasons.push("recipient is a US person — §871(m) does not apply".into());
        r.net_dividend_equivalent_to_recipient = input.dividend_equivalent_amount;
        r.note = "§871(m) inapplicable: US recipient".into();
        return r;
    }

    if !input.underlying_pays_dividends {
        r.reasons.push(
            "underlying does not pay dividends — no dividend equivalent to withhold on"
                .into(),
        );
        r.net_dividend_equivalent_to_recipient = input.dividend_equivalent_amount;
        r.note = "§871(m) inapplicable: non-dividend-paying underlying".into();
        return r;
    }

    if input.dividend_equivalent_amount <= Decimal::ZERO {
        r.reasons.push("no dividend equivalent paid this period".into());
        r.net_dividend_equivalent_to_recipient = input.dividend_equivalent_amount;
        return r;
    }

    // Delta threshold depends on original term.
    let short_term = input.original_term_days <= 365;
    let threshold = if short_term {
        Decimal::from_str("0.80").unwrap()
    } else {
        Decimal::ONE
    };
    r.delta_threshold_applied = threshold;

    let delta = input.delta_at_pricing.clamp(Decimal::ZERO, Decimal::ONE);

    if delta < threshold {
        r.subject_to_871m = false;
        r.reasons.push(format!(
            "{} contract delta {} < {} threshold — not a SELI under §871(m)",
            if short_term { "short-term" } else { "long-term" },
            delta,
            threshold
        ));
        r.net_dividend_equivalent_to_recipient = input.dividend_equivalent_amount;
        r.note = format!(
            "§871(m) inapplicable: delta {} below {} threshold for {}-term contract",
            delta,
            threshold,
            if short_term { "short" } else { "long" }
        );
        return r;
    }

    // SELI status established. Apply withholding rate.
    r.subject_to_871m = true;
    r.applicable_rate = input.treaty_rate_override
        .map(|t| t.clamp(Decimal::ZERO, Decimal::ONE))
        .unwrap_or_else(statutory_rate);
    r.withholding_amount =
        (input.dividend_equivalent_amount * r.applicable_rate).round_dp(2);
    r.net_dividend_equivalent_to_recipient =
        input.dividend_equivalent_amount - r.withholding_amount;

    let rate_pct = (r.applicable_rate * Decimal::from(100)).round_dp(0);
    r.note = format!(
        "§871(m) SELI: delta {} ≥ {} threshold; ${} dividend equivalent × {}% = ${} withheld; ${} net to recipient",
        delta,
        threshold,
        input.dividend_equivalent_amount,
        rate_pct,
        r.withholding_amount,
        r.net_dividend_equivalent_to_recipient
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section871MInput {
        // Non-US person holding a near-the-money short-term call on
        // AAPL with a $200 dividend equivalent.
        Section871MInput {
            non_us_person: true,
            instrument_type: InstrumentType::ListedEquityOption,
            delta_at_pricing: dec!(0.85),
            original_term_days: 90,
            underlying_pays_dividends: true,
            dividend_equivalent_amount: dec!(200),
            treaty_rate_override: None,
        }
    }

    #[test]
    fn short_term_delta_0_85_subject_to_871m_at_30_pct() {
        let r = compute(&base());
        assert!(r.subject_to_871m);
        assert_eq!(r.applicable_rate, dec!(0.30));
        assert_eq!(r.withholding_amount, dec!(60));
        assert_eq!(r.net_dividend_equivalent_to_recipient, dec!(140));
    }

    #[test]
    fn short_term_delta_exactly_0_80_subject() {
        let mut i = base();
        i.delta_at_pricing = dec!(0.80);
        let r = compute(&i);
        assert!(r.subject_to_871m);
    }

    #[test]
    fn short_term_delta_0_79_not_subject() {
        let mut i = base();
        i.delta_at_pricing = dec!(0.79);
        let r = compute(&i);
        assert!(!r.subject_to_871m);
        assert_eq!(r.withholding_amount, Decimal::ZERO);
    }

    #[test]
    fn long_term_delta_below_1_not_subject() {
        // 2-year LEAPS at delta 0.90 — not subject (long-term needs 1.0).
        let mut i = base();
        i.original_term_days = 730;
        i.delta_at_pricing = dec!(0.90);
        let r = compute(&i);
        assert!(!r.subject_to_871m);
        assert_eq!(r.delta_threshold_applied, Decimal::ONE);
    }

    #[test]
    fn long_term_delta_exactly_1_0_subject() {
        let mut i = base();
        i.original_term_days = 730;
        i.delta_at_pricing = Decimal::ONE;
        let r = compute(&i);
        assert!(r.subject_to_871m);
    }

    #[test]
    fn us_person_recipient_not_subject() {
        let mut i = base();
        i.non_us_person = false;
        let r = compute(&i);
        assert!(!r.subject_to_871m);
        assert_eq!(r.net_dividend_equivalent_to_recipient, dec!(200));
        assert!(r.reasons[0].contains("US person"));
    }

    #[test]
    fn non_dividend_paying_underlying_skips_871m() {
        let mut i = base();
        i.underlying_pays_dividends = false;
        let r = compute(&i);
        assert!(!r.subject_to_871m);
        assert!(r.reasons[0].contains("does not pay dividends"));
    }

    #[test]
    fn treaty_rate_15_pct_overrides_statutory_30() {
        // US-Canada / US-UK / US-Germany typical treaty rate for dividends.
        let mut i = base();
        i.treaty_rate_override = Some(dec!(0.15));
        let r = compute(&i);
        assert!(r.subject_to_871m);
        assert_eq!(r.applicable_rate, dec!(0.15));
        assert_eq!(r.withholding_amount, dec!(30));
        assert_eq!(r.net_dividend_equivalent_to_recipient, dec!(170));
    }

    #[test]
    fn treaty_rate_zero_full_exemption_zero_withholding() {
        let mut i = base();
        i.treaty_rate_override = Some(Decimal::ZERO);
        let r = compute(&i);
        assert!(r.subject_to_871m);
        assert_eq!(r.withholding_amount, Decimal::ZERO);
        assert_eq!(r.net_dividend_equivalent_to_recipient, dec!(200));
    }

    #[test]
    fn treaty_rate_above_1_clamped_to_1() {
        // Pathological — but make sure we don't compute > 100% withholding.
        let mut i = base();
        i.treaty_rate_override = Some(dec!(1.5));
        let r = compute(&i);
        assert_eq!(r.applicable_rate, Decimal::ONE);
        assert_eq!(r.withholding_amount, dec!(200));
    }

    #[test]
    fn delta_above_1_clamped_to_1() {
        let mut i = base();
        i.delta_at_pricing = dec!(1.5);
        let r = compute(&i);
        assert!(r.subject_to_871m);
        // Still uses 1.0 (clamped) which is >= 0.80 threshold.
    }

    #[test]
    fn delta_negative_clamped_to_zero_not_subject() {
        let mut i = base();
        i.delta_at_pricing = dec!(-0.5);
        let r = compute(&i);
        assert!(!r.subject_to_871m);
    }

    #[test]
    fn zero_dividend_equivalent_no_withholding_path() {
        let mut i = base();
        i.dividend_equivalent_amount = Decimal::ZERO;
        let r = compute(&i);
        // Even if SELI status would apply, no payment to withhold on.
        assert_eq!(r.withholding_amount, Decimal::ZERO);
    }

    #[test]
    fn short_term_boundary_365_days_uses_short_threshold() {
        let mut i = base();
        i.original_term_days = 365;
        i.delta_at_pricing = dec!(0.80);
        let r = compute(&i);
        assert!(r.subject_to_871m);
        assert_eq!(r.delta_threshold_applied, dec!(0.80));
    }

    #[test]
    fn long_term_boundary_366_days_uses_long_threshold() {
        let mut i = base();
        i.original_term_days = 366;
        i.delta_at_pricing = dec!(0.80); // would qualify short-term, not long-term
        let r = compute(&i);
        assert!(!r.subject_to_871m);
        assert_eq!(r.delta_threshold_applied, Decimal::ONE);
    }

    #[test]
    fn note_distinguishes_subject_vs_inapplicable_paths() {
        let subject = compute(&base());
        assert!(subject.note.contains("SELI"));

        let mut not_subject = base();
        not_subject.delta_at_pricing = dec!(0.5);
        let r = compute(&not_subject);
        assert!(r.note.contains("inapplicable"));
    }

    #[test]
    fn us_person_short_circuit_runs_first() {
        // US person + all-other-bad-facts should still short-circuit on
        // US-person check.
        let mut i = base();
        i.non_us_person = false;
        i.underlying_pays_dividends = false; // also bad
        let r = compute(&i);
        assert!(!r.subject_to_871m);
        assert!(r.reasons[0].contains("US person"));
    }
}
