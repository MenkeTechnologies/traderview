//! IRC §1233 — Gains and losses from short sales.
//!
//! Anti-abuse rules that prevent a trader from using a short sale to convert
//! the character of a gain or loss on substantially identical property
//! (long-term ↔ short-term). Pairs with §1259 (constructive sales, already
//! implemented) to close the "short against the box" tax-deferral trap.
//!
//! Three pieces:
//!
//! **§1233(b)(1) — gain → short-term.**  If on the date of the short sale
//! the taxpayer holds substantially identical property for ≤ 1 year, OR
//! acquires substantially identical property between the short open and
//! the short close, then any **gain** on closing the short is short-term —
//! regardless of how long the property used to close the short was held.
//! This prevents shorting against a long-term position to lock in gains
//! while preserving capital-gain-rate optionality.
//!
//! **§1233(b)(2) — holding-period reset.**  When §1233(b)(1) applies, the
//! substantially identical property's holding period is **reset to begin
//! on the date the short is closed**. FIFO across substantially identical
//! lots, capped at `short_shares`. Treas. Reg. §1.1233-1(c)(3).
//!
//! **§1233(d) — loss → long-term.**  If on the date of the short sale the
//! taxpayer holds substantially identical property for > 1 year, any
//! **loss** on closing the short is long-term — regardless of how long
//! the property used to close was held. This prevents using a short sale
//! to convert a long-term loss into the short-term loss bucket (STCL
//! absorbs first against ordinary income under §1212(b)(2), so STCL is
//! cash-flow-preferable).
//!
//! Both rules can trigger simultaneously when the taxpayer holds BOTH
//! short-held and long-held substantially identical property at the time
//! of the short. §1233(d) governs losses, §1233(b) governs gains —
//! whichever sign the close produces is the rule that applies.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A long position in substantially identical property at the time of
/// the short sale (or acquired during the short period).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongPosition {
    pub acquisition_date: NaiveDate,
    pub shares: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1233Input {
    pub short_sale_date: NaiveDate,
    pub short_close_date: NaiveDate,
    pub short_shares: i64,
    /// Net gain/loss on closing the short. Positive = gain, negative = loss.
    pub gain_loss_amount: Decimal,
    /// Substantially identical positions held at the time of the short
    /// open. Acquisition dates must be ≤ short_sale_date.
    pub substantially_identical_held_at_open: Vec<LongPosition>,
    /// Substantially identical positions acquired AFTER the short open and
    /// on or before the short close. Triggers §1233(b)(1) regardless of
    /// holding period.
    pub substantially_identical_acquired_during_short: Vec<LongPosition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxCharacter {
    ShortTerm,
    LongTerm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1233Rule {
    /// Neither §1233(b) nor §1233(d) triggered — default short-term close.
    None,
    /// §1233(b)(1) applied — gain on close is short-term.
    SubsectionB,
    /// §1233(d) applied — loss on close is long-term.
    SubsectionD,
}

/// A holding-period reset under §1233(b)(2) for one lot of substantially
/// identical property. The lot's holding period now begins on
/// `new_holding_period_start` (= short close date).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingPeriodReset {
    pub original_acquisition_date: NaiveDate,
    pub new_holding_period_start: NaiveDate,
    pub shares_affected: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1233Result {
    pub character: TaxCharacter,
    pub rule_triggered: Section1233Rule,
    /// Lots whose holding periods reset under §1233(b)(2), FIFO across
    /// short-held + during-short lots, capped at `short_shares`.
    pub holding_period_resets: Vec<HoldingPeriodReset>,
    pub gain_loss_amount: Decimal,
    pub note: String,
}

/// Holding-period boundary under §1233: ≤ 365 days = short-term;
/// > 365 days = long-term. Matches the §1222 boundary used elsewhere.
const ONE_YEAR_DAYS: i64 = 365;

/// Apply IRC §1233 to a closed short sale.
pub fn compute(input: &Section1233Input) -> Section1233Result {
    let is_gain = input.gain_loss_amount > Decimal::ZERO;
    let is_loss = input.gain_loss_amount < Decimal::ZERO;

    // §1233(b)(1) triggers:
    //   (a) any substantially identical held ≤ 1 year on short_sale_date, OR
    //   (b) any substantially identical acquired during the short period
    let any_short_held_at_open = input
        .substantially_identical_held_at_open
        .iter()
        .any(|p| {
            let days = (input.short_sale_date - p.acquisition_date).num_days();
            days <= ONE_YEAR_DAYS
        });
    let any_acquired_during_short = !input.substantially_identical_acquired_during_short.is_empty();
    let subsection_b_triggered = any_short_held_at_open || any_acquired_during_short;

    // §1233(d) triggers if any substantially identical held > 1 year on
    // short_sale_date.
    let any_long_held_at_open = input
        .substantially_identical_held_at_open
        .iter()
        .any(|p| {
            let days = (input.short_sale_date - p.acquisition_date).num_days();
            days > ONE_YEAR_DAYS
        });
    let subsection_d_triggered = any_long_held_at_open;

    // Loss path: §1233(d) governs.
    if is_loss && subsection_d_triggered {
        return Section1233Result {
            character: TaxCharacter::LongTerm,
            rule_triggered: Section1233Rule::SubsectionD,
            holding_period_resets: vec![],
            gain_loss_amount: input.gain_loss_amount,
            note: format!(
                "§1233(d) — taxpayer held substantially identical property > 1 year on short open ({}); ${} loss recharacterized as long-term",
                input.short_sale_date,
                (-input.gain_loss_amount).round_dp(2)
            ),
        };
    }

    // Gain path: §1233(b) governs. Default is already short-term, but the
    // rule flag matters because (b)(2) also resets the long position's
    // holding period.
    if is_gain && subsection_b_triggered {
        let resets = build_resets(input);
        return Section1233Result {
            character: TaxCharacter::ShortTerm,
            rule_triggered: Section1233Rule::SubsectionB,
            holding_period_resets: resets,
            gain_loss_amount: input.gain_loss_amount,
            note: format!(
                "§1233(b)(1) — short-held or during-short substantially identical at open of {}; ${} gain is short-term and §1233(b)(2) resets holding period of substantially identical lots to {}",
                input.short_sale_date,
                input.gain_loss_amount.round_dp(2),
                input.short_close_date
            ),
        };
    }

    // No §1233 modification — default ST close, no rule, no resets. This
    // covers: zero gain/loss; gain with only long-held; loss with only
    // short-held; no substantially identical at all.
    Section1233Result {
        character: TaxCharacter::ShortTerm,
        rule_triggered: Section1233Rule::None,
        holding_period_resets: vec![],
        gain_loss_amount: input.gain_loss_amount,
        note: "no §1233 modification — short-sale close is short-term by default".into(),
    }
}

/// FIFO-allocate §1233(b)(2) holding-period resets across substantially
/// identical lots. Combines `held_at_open` (filtered to short-held only)
/// and `acquired_during_short` in acquisition-date order, capped at
/// `short_shares` total per the statute.
fn build_resets(input: &Section1233Input) -> Vec<HoldingPeriodReset> {
    let mut candidates: Vec<(NaiveDate, i64, &'static str)> = Vec::new();
    for p in &input.substantially_identical_held_at_open {
        let days = (input.short_sale_date - p.acquisition_date).num_days();
        if days <= ONE_YEAR_DAYS {
            candidates.push((p.acquisition_date, p.shares, "§1233(b)(1)(A)"));
        }
    }
    for p in &input.substantially_identical_acquired_during_short {
        candidates.push((p.acquisition_date, p.shares, "§1233(b)(1)(B)"));
    }
    candidates.sort_by_key(|c| c.0);

    let mut out: Vec<HoldingPeriodReset> = Vec::new();
    let mut remaining = input.short_shares;
    for (acq, shares, reason) in candidates {
        if remaining <= 0 {
            break;
        }
        let take = shares.min(remaining);
        out.push(HoldingPeriodReset {
            original_acquisition_date: acq,
            new_holding_period_start: input.short_close_date,
            shares_affected: take,
            reason: reason.to_string(),
        });
        remaining -= take;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn long(acq: NaiveDate, shares: i64) -> LongPosition {
        LongPosition {
            acquisition_date: acq,
            shares,
        }
    }

    fn base() -> Section1233Input {
        Section1233Input {
            short_sale_date: d(2026, 6, 1),
            short_close_date: d(2026, 12, 1),
            short_shares: 100,
            gain_loss_amount: dec!(1000),
            substantially_identical_held_at_open: vec![],
            substantially_identical_acquired_during_short: vec![],
        }
    }

    #[test]
    fn no_substantially_identical_default_short_term() {
        // Plain short sale with no offsetting long → default ST close.
        let r = compute(&base());
        assert_eq!(r.character, TaxCharacter::ShortTerm);
        assert_eq!(r.rule_triggered, Section1233Rule::None);
        assert!(r.holding_period_resets.is_empty());
    }

    #[test]
    fn short_held_long_at_open_with_gain_triggers_subsection_b() {
        // Long bought 90 days before short → short-held. Gain on short
        // close. §1233(b) applies: gain is ST (default anyway) and
        // holding period of long position resets to short close date.
        let mut i = base();
        i.substantially_identical_held_at_open.push(long(d(2026, 3, 3), 100));
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::ShortTerm);
        assert_eq!(r.rule_triggered, Section1233Rule::SubsectionB);
        assert_eq!(r.holding_period_resets.len(), 1);
        let reset = &r.holding_period_resets[0];
        assert_eq!(reset.original_acquisition_date, d(2026, 3, 3));
        assert_eq!(reset.new_holding_period_start, d(2026, 12, 1));
        assert_eq!(reset.shares_affected, 100);
        assert_eq!(reset.reason, "§1233(b)(1)(A)");
    }

    #[test]
    fn long_held_at_open_with_loss_triggers_subsection_d() {
        // Long bought 10 years before short → long-held. Loss on short
        // close. §1233(d): loss is LT regardless of how long the close
        // property was held.
        let mut i = base();
        i.gain_loss_amount = dec!(-500);
        i.substantially_identical_held_at_open.push(long(d(2016, 1, 1), 100));
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::LongTerm);
        assert_eq!(r.rule_triggered, Section1233Rule::SubsectionD);
        assert!(r.note.contains("§1233(d)"));
    }

    #[test]
    fn long_held_at_open_with_gain_no_rule() {
        // Long-held substantially identical exists, but the close is a
        // GAIN. §1233(d) only governs losses; §1233(b) requires
        // short-held. Default ST applies, no rule triggered.
        let mut i = base();
        i.substantially_identical_held_at_open.push(long(d(2016, 1, 1), 100));
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::ShortTerm);
        assert_eq!(r.rule_triggered, Section1233Rule::None);
        assert!(r.holding_period_resets.is_empty());
    }

    #[test]
    fn short_held_at_open_with_loss_no_rule_modification() {
        // Short-held substantially identical exists, but the close is a
        // LOSS. §1233(b) only governs gains; §1233(d) requires long-held.
        // Default ST loss; no rule triggered.
        let mut i = base();
        i.gain_loss_amount = dec!(-500);
        i.substantially_identical_held_at_open.push(long(d(2026, 3, 3), 100));
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::ShortTerm);
        assert_eq!(r.rule_triggered, Section1233Rule::None);
    }

    #[test]
    fn substantially_identical_acquired_during_short_triggers_subsection_b() {
        // No long position at open, but taxpayer buys 100 shares 30 days
        // into the short period. §1233(b)(1)(B) — the acquisition during
        // the short window triggers the rule on gain.
        let mut i = base();
        i.substantially_identical_acquired_during_short
            .push(long(d(2026, 7, 1), 100));
        let r = compute(&i);
        assert_eq!(r.rule_triggered, Section1233Rule::SubsectionB);
        assert_eq!(r.holding_period_resets.len(), 1);
        assert_eq!(
            r.holding_period_resets[0].reason,
            "§1233(b)(1)(B)"
        );
    }

    #[test]
    fn both_short_and_long_held_with_loss_subsection_d_wins() {
        // Both 1233(b) and 1233(d) triggers exist. Loss → §1233(d)
        // governs and recharacterizes as LT. No holding-period resets
        // (those are §1233(b)(2) which doesn't fire on the loss path).
        let mut i = base();
        i.gain_loss_amount = dec!(-2000);
        i.substantially_identical_held_at_open.push(long(d(2026, 3, 3), 50));
        i.substantially_identical_held_at_open.push(long(d(2016, 1, 1), 50));
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::LongTerm);
        assert_eq!(r.rule_triggered, Section1233Rule::SubsectionD);
        assert!(r.holding_period_resets.is_empty());
    }

    #[test]
    fn both_short_and_long_held_with_gain_subsection_b_wins() {
        // Same setup as above but gain. §1233(b) governs. Only the
        // short-held lot resets — the long-held lot stays at its
        // original acquisition date (the rule only resets property that
        // triggered (b), and only the short-held does).
        let mut i = base();
        i.gain_loss_amount = dec!(2000);
        i.substantially_identical_held_at_open.push(long(d(2026, 3, 3), 50));
        i.substantially_identical_held_at_open.push(long(d(2016, 1, 1), 50));
        let r = compute(&i);
        assert_eq!(r.character, TaxCharacter::ShortTerm);
        assert_eq!(r.rule_triggered, Section1233Rule::SubsectionB);
        assert_eq!(r.holding_period_resets.len(), 1);
        assert_eq!(
            r.holding_period_resets[0].original_acquisition_date,
            d(2026, 3, 3)
        );
        assert_eq!(r.holding_period_resets[0].shares_affected, 50);
    }

    #[test]
    fn fifo_resets_capped_at_short_shares() {
        // Three short-held lots, 50 + 50 + 50 = 150 shares; short is
        // 100 shares. FIFO: oldest two lots fully reset (100 total),
        // third lot not reset (per statute "only to so much of such
        // property as does not exceed the quantity sold short").
        let mut i = base();
        i.substantially_identical_held_at_open = vec![
            long(d(2026, 3, 1), 50),
            long(d(2026, 4, 1), 50),
            long(d(2026, 5, 1), 50),
        ];
        let r = compute(&i);
        assert_eq!(r.holding_period_resets.len(), 2);
        assert_eq!(r.holding_period_resets[0].original_acquisition_date, d(2026, 3, 1));
        assert_eq!(r.holding_period_resets[0].shares_affected, 50);
        assert_eq!(r.holding_period_resets[1].original_acquisition_date, d(2026, 4, 1));
        assert_eq!(r.holding_period_resets[1].shares_affected, 50);
    }

    #[test]
    fn one_year_boundary_held_exactly_365_days_is_short_term() {
        // Holding period of "not more than 1 year" = ≤ 365 days under
        // §1222 / §1233. Day 365 exact → short-held → §1233(b) triggers
        // on gain. Day 366 → long-held → §1233(b) does NOT trigger on
        // gain (but §1233(d) triggers on loss).
        let mut i = base();
        i.substantially_identical_held_at_open.push(long(d(2025, 6, 1), 100));
        let days = (i.short_sale_date - d(2025, 6, 1)).num_days();
        assert_eq!(days, 365, "test setup: exactly 365 days");
        let r = compute(&i);
        assert_eq!(r.rule_triggered, Section1233Rule::SubsectionB);
    }

    #[test]
    fn one_year_boundary_held_366_days_is_long_term() {
        let mut i = base();
        i.substantially_identical_held_at_open.push(long(d(2025, 5, 31), 100));
        let days = (i.short_sale_date - d(2025, 5, 31)).num_days();
        assert_eq!(days, 366, "test setup: exactly 366 days");
        // Gain path — no rule triggers since long-held.
        let r = compute(&i);
        assert_eq!(r.rule_triggered, Section1233Rule::None);

        // Loss path — §1233(d) triggers because long-held.
        let mut i2 = i.clone();
        i2.gain_loss_amount = dec!(-500);
        let r2 = compute(&i2);
        assert_eq!(r2.rule_triggered, Section1233Rule::SubsectionD);
        assert_eq!(r2.character, TaxCharacter::LongTerm);
    }

    #[test]
    fn zero_gain_loss_no_rule() {
        let mut i = base();
        i.gain_loss_amount = Decimal::ZERO;
        i.substantially_identical_held_at_open.push(long(d(2026, 3, 3), 100));
        let r = compute(&i);
        assert_eq!(r.rule_triggered, Section1233Rule::None);
    }

    #[test]
    fn holding_period_reset_date_equals_short_close_not_short_open() {
        // §1233(b)(2) resets to "date of closing of the short sale",
        // NOT the date of the short open. Easy regression target.
        let mut i = base();
        i.short_sale_date = d(2026, 6, 1);
        i.short_close_date = d(2027, 1, 15);
        i.substantially_identical_held_at_open.push(long(d(2026, 4, 1), 100));
        let r = compute(&i);
        assert_eq!(
            r.holding_period_resets[0].new_holding_period_start,
            d(2027, 1, 15)
        );
    }

    #[test]
    fn during_short_acquisition_resets_to_close_date() {
        // Acquisition during short triggers (b)(1)(B). The acquired lot's
        // holding period resets to the close date — same rule as (A).
        let mut i = base();
        i.substantially_identical_acquired_during_short
            .push(long(d(2026, 7, 15), 100));
        let r = compute(&i);
        assert_eq!(
            r.holding_period_resets[0].new_holding_period_start,
            i.short_close_date
        );
    }

    #[test]
    fn long_held_lots_never_appear_in_reset_list() {
        // Only short-held + during-short lots appear in resets. A long-
        // held lot is "older than the holding period clock anyway" and
        // §1233(b)(2) deliberately excludes it — the rule's whole point
        // is to penalize NEW positions, not existing LTCG-qualified ones.
        let mut i = base();
        i.substantially_identical_held_at_open.push(long(d(2016, 1, 1), 200));
        i.substantially_identical_held_at_open.push(long(d(2026, 5, 1), 50));
        let r = compute(&i);
        assert_eq!(r.rule_triggered, Section1233Rule::SubsectionB);
        assert_eq!(r.holding_period_resets.len(), 1);
        assert_eq!(
            r.holding_period_resets[0].original_acquisition_date,
            d(2026, 5, 1)
        );
    }

    #[test]
    fn short_held_and_during_short_combined_fifo_order() {
        // Both buckets contribute. Short-held lot acquired 2026-04-01,
        // during-short lot acquired 2026-08-01. FIFO order is by
        // acquisition date — April first.
        let mut i = base();
        i.short_shares = 200;
        i.substantially_identical_held_at_open.push(long(d(2026, 4, 1), 100));
        i.substantially_identical_acquired_during_short.push(long(d(2026, 8, 1), 100));
        let r = compute(&i);
        assert_eq!(r.holding_period_resets.len(), 2);
        assert_eq!(r.holding_period_resets[0].original_acquisition_date, d(2026, 4, 1));
        assert_eq!(r.holding_period_resets[0].reason, "§1233(b)(1)(A)");
        assert_eq!(r.holding_period_resets[1].original_acquisition_date, d(2026, 8, 1));
        assert_eq!(r.holding_period_resets[1].reason, "§1233(b)(1)(B)");
    }

    #[test]
    fn acquisition_on_short_open_date_is_short_held_at_open() {
        // Acquired on the exact short_sale_date → days_held = 0 → ≤ 1 year
        // → §1233(b)(1)(A) applies. Not (B), because acquisition wasn't
        // strictly AFTER the short open.
        let mut i = base();
        i.substantially_identical_held_at_open.push(long(d(2026, 6, 1), 100));
        let r = compute(&i);
        assert_eq!(r.holding_period_resets[0].reason, "§1233(b)(1)(A)");
    }

    #[test]
    fn empty_during_short_with_short_held_at_open_works() {
        // Only the (A) bucket populated — should still trigger §1233(b).
        let mut i = base();
        i.substantially_identical_held_at_open.push(long(d(2026, 4, 1), 100));
        let r = compute(&i);
        assert_eq!(r.rule_triggered, Section1233Rule::SubsectionB);
    }

    #[test]
    fn empty_held_at_open_with_during_short_only_works() {
        // Only the (B) bucket populated — should still trigger §1233(b).
        let mut i = base();
        i.substantially_identical_acquired_during_short.push(long(d(2026, 7, 1), 100));
        let r = compute(&i);
        assert_eq!(r.rule_triggered, Section1233Rule::SubsectionB);
    }

    #[test]
    fn note_mentions_short_close_date_when_resets_emit() {
        let mut i = base();
        i.substantially_identical_held_at_open.push(long(d(2026, 4, 1), 100));
        let r = compute(&i);
        assert!(r.note.contains("§1233(b)(1)"));
        assert!(r.note.contains("2026-12-01"));
    }

    #[test]
    fn note_mentions_loss_amount_when_subsection_d_fires() {
        let mut i = base();
        i.gain_loss_amount = dec!(-1234.56);
        i.substantially_identical_held_at_open.push(long(d(2016, 1, 1), 100));
        let r = compute(&i);
        assert!(r.note.contains("§1233(d)"));
        assert!(r.note.contains("1234.56"));
    }
}
