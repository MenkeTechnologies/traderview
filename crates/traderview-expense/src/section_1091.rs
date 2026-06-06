//! IRC §1091 — Loss from wash sales of stock or securities.
//!
//! Fills the long-standing `wash_sale.rs` TODO referenced by `schedule_d.rs`
//! since the module was first written. Every non-§475(f) trader needs this
//! to compute disallowed losses; §475(f) electors are exempt because they
//! mark to market and never realize a loss in the §1091 sense.
//!
//! Core rule (§1091(a)): a loss on the sale of stock or securities is
//! **disallowed** if the taxpayer (or their spouse, controlled corp, or
//! IRA) acquires substantially identical stock within a 61-day window
//! centered on the sale date — 30 days before, the sale date itself, and
//! 30 days after.
//!
//! Disallowed loss is added to the basis of the replacement shares under
//! §1091(d) — FIFO across replacement lots in purchase-date order. The
//! holding period of the replacement shares **tacks** from the original
//! sale lot under §1223(4), but this module only computes the basis side.
//!
//! **Rev. Rul. 2008-5 (IRA replacement)**: when the replacement purchase
//! is in the taxpayer's IRA or Roth IRA, §1091(d) is overridden. The loss
//! is disallowed, the IRA basis is NOT increased, and the loss is
//! permanently lost. This is the "IRA wash sale" trap — most retail
//! brokers do not warn on it.
//!
//! **§475(f) mark-to-market elector exemption**: §475(f)(1)(C) says §1091
//! does not apply to securities marked to market under §475(f)(1). The
//! `seller_is_475f_elector` flag short-circuits the compute to no wash
//! sale ever triggering. The TTS qualification is a separate predicate
//! the caller verifies.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A purchase of (potentially) substantially-identical replacement shares.
/// `account_is_ira` triggers the Rev. Rul. 2008-5 permanent-loss path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementPurchase {
    pub purchase_date: NaiveDate,
    pub shares: i64,
    pub price_per_share: Decimal,
    pub account_is_ira: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1091Input {
    pub sale_date: NaiveDate,
    pub sale_shares: i64,
    pub sale_price_per_share: Decimal,
    /// Cost basis per share of the sold lot (already-adjusted basis).
    pub basis_per_share: Decimal,
    pub replacement_purchases: Vec<ReplacementPurchase>,
    /// True if the seller has elected §475(f) trader-in-securities MTM.
    /// §1091 does not apply to §475(f) securities.
    pub seller_is_475f_elector: bool,
}

/// FIFO basis adjustment to a single replacement lot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WashSaleAdjustment {
    pub purchase_date: NaiveDate,
    pub shares_adjusted: i64,
    pub basis_added_per_share: Decimal,
    pub total_basis_added: Decimal,
    pub account_is_ira: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1091Result {
    /// Loss as realized before §1091 applies (negative number).
    /// Zero if the sale produced a gain (no wash sale possible).
    pub original_loss: Decimal,
    /// Loss disallowed under §1091(a) (positive number — represents the
    /// portion of `original_loss` that is NOT deductible).
    pub disallowed_loss: Decimal,
    /// Loss still allowed (negative number — flows to Schedule D normally).
    pub allowed_loss: Decimal,
    /// Total replacement shares within the 61-day window.
    pub replacement_shares_in_window: i64,
    /// True if at least one dollar of loss was disallowed.
    pub wash_sale_triggered: bool,
    /// disallowed / original_loss, as a ratio (0..=1).
    pub disallowance_ratio: Decimal,
    /// FIFO allocation of the disallowed loss to replacement lots.
    /// Empty when no wash sale triggered or IRA permanent-loss path taken.
    pub adjustments: Vec<WashSaleAdjustment>,
    /// True if any replacement purchase was in an IRA — Rev. Rul. 2008-5
    /// applies and the IRA-portion of the loss is permanently lost
    /// (no basis adjustment).
    pub ira_permanent_loss: bool,
    /// True if §475(f) elector exemption short-circuited the rule.
    pub mtm_election_exempts: bool,
    pub note: String,
}

/// Apply IRC §1091 to a single sale + its surrounding purchase events.
pub fn compute(input: &Section1091Input) -> Section1091Result {
    // 1) Gross gain/loss.
    let sale_proceeds = Decimal::from(input.sale_shares) * input.sale_price_per_share;
    let cost_basis = Decimal::from(input.sale_shares) * input.basis_per_share;
    let gross = sale_proceeds - cost_basis;

    // Sale produced a gain — §1091 is loss-only; nothing to disallow.
    if gross >= Decimal::ZERO {
        return Section1091Result {
            original_loss: Decimal::ZERO,
            disallowed_loss: Decimal::ZERO,
            allowed_loss: Decimal::ZERO,
            replacement_shares_in_window: 0,
            wash_sale_triggered: false,
            disallowance_ratio: Decimal::ZERO,
            adjustments: vec![],
            ira_permanent_loss: false,
            mtm_election_exempts: false,
            note: "sale produced a gain — §1091 does not apply".into(),
        };
    }
    let original_loss = gross; // negative

    // 2) §475(f) elector exempt.
    if input.seller_is_475f_elector {
        return Section1091Result {
            original_loss,
            disallowed_loss: Decimal::ZERO,
            allowed_loss: original_loss,
            replacement_shares_in_window: 0,
            wash_sale_triggered: false,
            disallowance_ratio: Decimal::ZERO,
            adjustments: vec![],
            ira_permanent_loss: false,
            mtm_election_exempts: true,
            note: "§475(f) mark-to-market elector — §1091 does not apply per §475(f)(1)(C)".into(),
        };
    }

    // 3) 61-day window: -30 to +30 inclusive of the sale date.
    let window_start = input.sale_date - chrono::Duration::days(30);
    let window_end = input.sale_date + chrono::Duration::days(30);

    // 4) Filter + sort replacement purchases by date (FIFO order).
    let mut in_window: Vec<&ReplacementPurchase> = input
        .replacement_purchases
        .iter()
        .filter(|p| p.purchase_date >= window_start && p.purchase_date <= window_end)
        .collect();
    in_window.sort_by_key(|p| p.purchase_date);

    let replacement_shares_in_window: i64 = in_window.iter().map(|p| p.shares).sum();

    if replacement_shares_in_window == 0 {
        return Section1091Result {
            original_loss,
            disallowed_loss: Decimal::ZERO,
            allowed_loss: original_loss,
            replacement_shares_in_window: 0,
            wash_sale_triggered: false,
            disallowance_ratio: Decimal::ZERO,
            adjustments: vec![],
            ira_permanent_loss: false,
            mtm_election_exempts: false,
            note: "no replacement purchases in the 61-day window — full loss allowed".into(),
        };
    }

    // 5) Disallowance ratio = min(repl, sold) / sold.
    let effective_repl = replacement_shares_in_window.min(input.sale_shares);
    let ratio = if input.sale_shares == 0 {
        Decimal::ZERO
    } else {
        Decimal::from(effective_repl) / Decimal::from(input.sale_shares)
    };
    // disallowed_loss is the POSITIVE magnitude of the lost deduction.
    let disallowed_loss = (-original_loss) * ratio;
    let allowed_loss = original_loss + disallowed_loss; // closer to zero

    // 6) FIFO basis allocation across in-window replacement lots, up to
    //    `effective_repl` total shares. Per-share basis addition is the
    //    original per-share loss = (-original_loss) / sale_shares.
    let loss_per_share = if input.sale_shares == 0 {
        Decimal::ZERO
    } else {
        (-original_loss) / Decimal::from(input.sale_shares)
    };

    let mut adjustments: Vec<WashSaleAdjustment> = Vec::new();
    let mut shares_remaining = effective_repl;
    let mut ira_seen = false;
    for p in &in_window {
        if shares_remaining <= 0 {
            break;
        }
        let take = p.shares.min(shares_remaining);
        if p.account_is_ira {
            ira_seen = true;
        }
        adjustments.push(WashSaleAdjustment {
            purchase_date: p.purchase_date,
            shares_adjusted: take,
            basis_added_per_share: if p.account_is_ira {
                // Rev. Rul. 2008-5: no basis adjustment in IRA.
                Decimal::ZERO
            } else {
                loss_per_share
            },
            total_basis_added: if p.account_is_ira {
                Decimal::ZERO
            } else {
                loss_per_share * Decimal::from(take)
            },
            account_is_ira: p.account_is_ira,
        });
        shares_remaining -= take;
    }

    let note = if ira_seen {
        format!(
            "wash sale triggered — ${} disallowed across {} replacement shares; at least one IRA replacement: Rev. Rul. 2008-5 disallows basis adjustment for IRA portion (permanent loss)",
            disallowed_loss.round_dp(2),
            effective_repl
        )
    } else {
        format!(
            "wash sale triggered — ${} disallowed; basis of {} replacement shares increased FIFO under §1091(d)",
            disallowed_loss.round_dp(2),
            effective_repl
        )
    };

    Section1091Result {
        original_loss,
        disallowed_loss,
        allowed_loss,
        replacement_shares_in_window,
        wash_sale_triggered: true,
        disallowance_ratio: ratio,
        adjustments,
        ira_permanent_loss: ira_seen,
        mtm_election_exempts: false,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn repl(date: NaiveDate, shares: i64, price: Decimal) -> ReplacementPurchase {
        ReplacementPurchase {
            purchase_date: date,
            shares,
            price_per_share: price,
            account_is_ira: false,
        }
    }

    fn ira_repl(date: NaiveDate, shares: i64, price: Decimal) -> ReplacementPurchase {
        ReplacementPurchase {
            purchase_date: date,
            shares,
            price_per_share: price,
            account_is_ira: true,
        }
    }

    fn base(sale: NaiveDate) -> Section1091Input {
        Section1091Input {
            sale_date: sale,
            sale_shares: 100,
            sale_price_per_share: dec!(90),
            basis_per_share: dec!(100),
            replacement_purchases: vec![],
            seller_is_475f_elector: false,
        }
    }

    #[test]
    fn sale_at_gain_no_wash_sale() {
        // Sold 100 shares at $110 with basis $100 → $1000 gain. §1091
        // only applies to losses. Even a purchase a day later is fine.
        let mut i = base(d(2026, 3, 15));
        i.sale_price_per_share = dec!(110);
        i.replacement_purchases
            .push(repl(d(2026, 3, 16), 100, dec!(105)));
        let r = compute(&i);
        assert_eq!(r.original_loss, Decimal::ZERO);
        assert!(!r.wash_sale_triggered);
        assert!(r.note.contains("gain"));
    }

    #[test]
    fn loss_with_no_replacement_full_loss_allowed() {
        // Loss of $1000, no purchases in 61-day window → full loss flows
        // to Schedule D.
        let i = base(d(2026, 3, 15));
        let r = compute(&i);
        assert_eq!(r.original_loss, dec!(-1000));
        assert_eq!(r.allowed_loss, dec!(-1000));
        assert_eq!(r.disallowed_loss, Decimal::ZERO);
        assert!(!r.wash_sale_triggered);
    }

    #[test]
    fn loss_with_full_replacement_all_disallowed() {
        // Sold 100-share loss for $1000, repurchased 100 shares within
        // window. All $1000 disallowed; ratio 1.0; FIFO adjustment lands
        // entirely on the one replacement lot.
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases
            .push(repl(d(2026, 3, 20), 100, dec!(85)));
        let r = compute(&i);
        assert!(r.wash_sale_triggered);
        assert_eq!(r.disallowed_loss, dec!(1000));
        assert_eq!(r.allowed_loss, Decimal::ZERO);
        assert_eq!(r.disallowance_ratio, dec!(1));
        assert_eq!(r.adjustments.len(), 1);
        let adj = &r.adjustments[0];
        assert_eq!(adj.shares_adjusted, 100);
        assert_eq!(adj.basis_added_per_share, dec!(10));
        assert_eq!(adj.total_basis_added, dec!(1000));
    }

    #[test]
    fn loss_with_partial_replacement_proportional_disallowed() {
        // Sold 100, repurchased 40 in window. Disallow 40% = $400.
        // Per-share addition = $10 (original loss per share). 40 × $10 = $400.
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases
            .push(repl(d(2026, 3, 20), 40, dec!(85)));
        let r = compute(&i);
        assert!(r.wash_sale_triggered);
        assert_eq!(r.disallowed_loss, dec!(400));
        assert_eq!(r.allowed_loss, dec!(-600));
        assert_eq!(r.disallowance_ratio, dec!(0.4));
        assert_eq!(r.adjustments[0].shares_adjusted, 40);
        assert_eq!(r.adjustments[0].total_basis_added, dec!(400));
    }

    #[test]
    fn replacement_at_negative_30_day_boundary_in_window() {
        // The window is sale_date - 30 days through sale_date + 30 days,
        // BOTH inclusive. -30 days exactly should land in the window.
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases
            .push(repl(d(2026, 2, 13), 100, dec!(85))); // -30 days
        let r = compute(&i);
        assert!(r.wash_sale_triggered, "−30d boundary must be in window");
        assert_eq!(r.replacement_shares_in_window, 100);
    }

    #[test]
    fn replacement_at_negative_31_day_outside_window() {
        // One day past the −30-day boundary is outside the window.
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases
            .push(repl(d(2026, 2, 12), 100, dec!(85))); // -31
        let r = compute(&i);
        assert!(!r.wash_sale_triggered, "−31d must be outside window");
        assert_eq!(r.replacement_shares_in_window, 0);
    }

    #[test]
    fn replacement_at_positive_30_day_boundary_in_window() {
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases
            .push(repl(d(2026, 4, 14), 100, dec!(85))); // +30
        let r = compute(&i);
        assert!(r.wash_sale_triggered, "+30d boundary must be in window");
    }

    #[test]
    fn replacement_at_positive_31_day_outside_window() {
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases
            .push(repl(d(2026, 4, 15), 100, dec!(85))); // +31
        let r = compute(&i);
        assert!(!r.wash_sale_triggered);
    }

    #[test]
    fn section_475f_elector_completely_exempt() {
        // MTM trader-in-securities elector: §475(f)(1)(C) says §1091
        // does not apply. The full loss is allowed regardless of any
        // replacement purchase.
        let mut i = base(d(2026, 3, 15));
        i.seller_is_475f_elector = true;
        i.replacement_purchases
            .push(repl(d(2026, 3, 20), 100, dec!(85)));
        let r = compute(&i);
        assert!(!r.wash_sale_triggered);
        assert!(r.mtm_election_exempts);
        assert_eq!(r.allowed_loss, dec!(-1000));
        assert!(r.note.contains("§475(f)"));
    }

    #[test]
    fn ira_replacement_triggers_permanent_loss() {
        // Rev. Rul. 2008-5: replacement in IRA → loss disallowed AND
        // no basis adjustment available. The disallowed loss is gone
        // forever; only the allowed (zero in this 100% case) survives.
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases
            .push(ira_repl(d(2026, 3, 20), 100, dec!(85)));
        let r = compute(&i);
        assert!(r.wash_sale_triggered);
        assert!(r.ira_permanent_loss);
        assert_eq!(r.disallowed_loss, dec!(1000));
        assert_eq!(r.allowed_loss, Decimal::ZERO);
        // IRA adjustment has ZERO basis_added — that's the trap.
        let adj = &r.adjustments[0];
        assert_eq!(adj.basis_added_per_share, Decimal::ZERO);
        assert_eq!(adj.total_basis_added, Decimal::ZERO);
        assert!(adj.account_is_ira);
        assert!(r.note.contains("Rev. Rul. 2008-5"));
    }

    #[test]
    fn fifo_allocation_across_multiple_lots() {
        // Sale 100-share loss, three replacement purchases:
        //   −5d: 30 shares
        //   +10d: 50 shares
        //   +25d: 80 shares
        // Total = 160. effective_repl = min(160, 100) = 100.
        // FIFO chews through: 30 + 50 + 20 = 100. Last lot gets 20 only.
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases = vec![
            repl(d(2026, 3, 10), 30, dec!(85)),
            repl(d(2026, 3, 25), 50, dec!(85)),
            repl(d(2026, 4, 9), 80, dec!(85)),
        ];
        let r = compute(&i);
        assert!(r.wash_sale_triggered);
        assert_eq!(r.disallowed_loss, dec!(1000));
        assert_eq!(r.adjustments.len(), 3);
        assert_eq!(r.adjustments[0].shares_adjusted, 30);
        assert_eq!(r.adjustments[1].shares_adjusted, 50);
        assert_eq!(r.adjustments[2].shares_adjusted, 20);
        // Per-share addition is the original per-share loss = $10.
        assert_eq!(r.adjustments[0].basis_added_per_share, dec!(10));
        assert_eq!(r.adjustments[1].basis_added_per_share, dec!(10));
        assert_eq!(r.adjustments[2].basis_added_per_share, dec!(10));
        let total: Decimal = r.adjustments.iter().map(|a| a.total_basis_added).sum();
        assert_eq!(total, dec!(1000));
    }

    #[test]
    fn out_of_order_purchases_sorted_to_fifo() {
        // Purchases passed in date-reverse order should be re-sorted by
        // date so the FIFO basis allocation is correct regardless of how
        // the caller built the list.
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases = vec![
            repl(d(2026, 4, 9), 80, dec!(85)),
            repl(d(2026, 3, 10), 30, dec!(85)),
            repl(d(2026, 3, 25), 50, dec!(85)),
        ];
        let r = compute(&i);
        // After sort, first adjusted lot is the 3/10 purchase, not 4/9.
        assert_eq!(r.adjustments[0].purchase_date, d(2026, 3, 10));
        assert_eq!(r.adjustments[1].purchase_date, d(2026, 3, 25));
        assert_eq!(r.adjustments[2].purchase_date, d(2026, 4, 9));
    }

    #[test]
    fn purchases_outside_window_ignored_even_with_in_window_purchases() {
        // Two purchases, one inside, one outside. Only the inside one
        // contributes to replacement_shares_in_window.
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases = vec![
            repl(d(2026, 1, 1), 100, dec!(85)), // outside (-73d)
            repl(d(2026, 3, 20), 50, dec!(85)), // inside (+5d)
        ];
        let r = compute(&i);
        assert_eq!(r.replacement_shares_in_window, 50);
        assert_eq!(r.disallowed_loss, dec!(500));
        assert_eq!(r.adjustments.len(), 1);
    }

    #[test]
    fn replacement_exceeds_sale_capped_at_sale_shares() {
        // Sale 100 shares, replacement 250 shares. Disallowance ratio
        // caps at 1.0 (100/100); only the first 100 replacement shares
        // get a basis adjustment, the rest are unaffected.
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases
            .push(repl(d(2026, 3, 20), 250, dec!(85)));
        let r = compute(&i);
        assert_eq!(r.disallowance_ratio, dec!(1));
        assert_eq!(r.disallowed_loss, dec!(1000));
        // Only 100 of the 250 shares adjusted.
        assert_eq!(r.adjustments[0].shares_adjusted, 100);
    }

    #[test]
    fn mixed_ira_and_taxable_flags_permanent_loss() {
        // First repl in IRA (30 shares), second in taxable (70 shares).
        // FIFO walks IRA first → permanent loss flag set even though
        // the taxable lot also gets a normal adjustment.
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases = vec![
            ira_repl(d(2026, 3, 10), 30, dec!(85)),
            repl(d(2026, 3, 25), 70, dec!(85)),
        ];
        let r = compute(&i);
        assert!(r.ira_permanent_loss);
        // IRA portion: 30 × $0 = $0 basis added (Rev. Rul. 2008-5).
        assert_eq!(r.adjustments[0].total_basis_added, Decimal::ZERO);
        assert!(r.adjustments[0].account_is_ira);
        // Taxable portion: 70 × $10 = $700 basis added normally.
        assert_eq!(r.adjustments[1].total_basis_added, dec!(700));
        assert!(!r.adjustments[1].account_is_ira);
    }

    #[test]
    fn sale_on_friday_replacement_following_monday_in_window() {
        // 2026-03-13 is a Friday. Monday 2026-03-16 is +3 days, well
        // within window. Weekend doesn't carve a hole.
        let mut i = base(d(2026, 3, 13));
        i.replacement_purchases
            .push(repl(d(2026, 3, 16), 100, dec!(85)));
        let r = compute(&i);
        assert!(r.wash_sale_triggered);
    }

    #[test]
    fn zero_share_sale_is_no_op() {
        // Pathological input — 0 shares sold can't realize a loss.
        let mut i = base(d(2026, 3, 15));
        i.sale_shares = 0;
        i.replacement_purchases
            .push(repl(d(2026, 3, 20), 100, dec!(85)));
        let r = compute(&i);
        assert!(!r.wash_sale_triggered);
        assert_eq!(r.original_loss, Decimal::ZERO);
    }

    #[test]
    fn allowed_loss_plus_disallowed_equals_original_magnitude() {
        // |allowed_loss| + disallowed_loss must equal |original_loss| for
        // every partial-disallowance ratio. This is the conservation law
        // for §1091 — no loss is created or destroyed, only deferred.
        for repl_shares in [10, 25, 50, 75, 100] {
            let mut i = base(d(2026, 3, 15));
            i.replacement_purchases
                .push(repl(d(2026, 3, 20), repl_shares, dec!(85)));
            let r = compute(&i);
            let total = (-r.allowed_loss) + r.disallowed_loss;
            assert_eq!(
                total,
                dec!(1000),
                "conservation broken for {repl_shares} replacement shares: \
                 allowed={:?}, disallowed={:?}",
                r.allowed_loss,
                r.disallowed_loss
            );
        }
    }

    #[test]
    fn mtm_election_short_circuits_before_window_check() {
        // The §475(f) check must happen BEFORE the window scan — otherwise
        // a 475(f) elector with a same-day repurchase would falsely trigger.
        // Pinning the ordering.
        let mut i = base(d(2026, 3, 15));
        i.seller_is_475f_elector = true;
        i.replacement_purchases
            .push(repl(d(2026, 3, 15), 100, dec!(85)));
        let r = compute(&i);
        assert!(!r.wash_sale_triggered);
        assert!(r.mtm_election_exempts);
        // No adjustments produced even though replacement was same-day.
        assert!(r.adjustments.is_empty());
    }

    #[test]
    fn ira_partial_with_taxable_overflow_correct_split() {
        // Sale 100-share loss $1000. IRA repl 60 shares first (FIFO), then
        // taxable repl 80 shares. effective_repl = 100. FIFO order:
        //   IRA: 60 shares (no basis added — permanent loss for those)
        //   Tax: 40 shares (+$10 each, $400 basis added)
        // Total disallowed = $1000, but only $400 lands on a basis
        // adjustment; $600 is permanently lost in the IRA.
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases = vec![
            ira_repl(d(2026, 3, 10), 60, dec!(85)),
            repl(d(2026, 3, 25), 80, dec!(85)),
        ];
        let r = compute(&i);
        assert_eq!(r.disallowed_loss, dec!(1000));
        assert!(r.ira_permanent_loss);
        assert_eq!(r.adjustments[0].shares_adjusted, 60);
        assert_eq!(r.adjustments[0].total_basis_added, Decimal::ZERO);
        assert_eq!(r.adjustments[1].shares_adjusted, 40);
        assert_eq!(r.adjustments[1].total_basis_added, dec!(400));
    }

    #[test]
    fn same_day_replacement_is_in_window() {
        // Repurchase on the same calendar day as the sale is the
        // classic accidental wash sale (T+0 round trip).
        let mut i = base(d(2026, 3, 15));
        i.replacement_purchases
            .push(repl(d(2026, 3, 15), 100, dec!(85)));
        let r = compute(&i);
        assert!(r.wash_sale_triggered);
        assert_eq!(r.disallowed_loss, dec!(1000));
    }
}
