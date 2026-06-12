//! Wash-sale detection over the shared fill reconstruction.
//!
//! IRS §1091: a loss is disallowed when substantially identical
//! shares are bought within 30 days before or after the loss sale.
//! This is a FLAGGING tool, not a filing engine — each loss sale is
//! judged against its own ±30-day window independently, so a single
//! repurchase can flag multiple losses (the real rule matches
//! share-for-share once). "Substantially identical" is modeled as
//! exact-symbol match. Long side only — sells with no held lots
//! (shorts) are skipped, not mis-scored.

use crate::live_vs_backtest::Fill;

pub const WASH_WINDOW_SECS: i64 = 30 * 86_400;

#[derive(Debug, Clone, serde::Serialize)]
pub struct WashSale {
    /// Epoch seconds of the loss-realizing sell.
    pub sale_ts: i64,
    /// Shares matched against held lots on that sell.
    pub qty_sold: f64,
    /// Realized loss magnitude (commission-inclusive, positive).
    pub loss: f64,
    /// Replacement shares found in the ±30-day window — buys still
    /// held after this sale's FIFO matching (before-leg) plus all
    /// later buys inside the window (after-leg).
    pub replacement_qty: f64,
    /// loss × min(replacement, sold)/sold — the prorated portion
    /// §1091 disallows; the rest remains deductible.
    pub disallowed: f64,
}

struct Lot {
    remaining: f64,
    orig_qty: f64,
    price: f64,
    commission: f64,
    ts: i64,
}

/// Detect wash sales in one symbol's chronological fills.
/// Only loss sales with at least one replacement share are returned —
/// a clean loss (no repurchase in the window) is not a wash.
pub fn wash_sales(fills: &[Fill]) -> Vec<WashSale> {
    let mut lots: Vec<Lot> = Vec::new();
    let mut out = Vec::new();
    for (i, f) in fills.iter().enumerate() {
        if f.buy {
            lots.push(Lot {
                remaining: f.qty,
                orig_qty: f.qty,
                price: f.price,
                commission: f.commission,
                ts: f.ts,
            });
            continue;
        }
        let mut to_match = f.qty;
        let (mut matched, mut pnl) = (0.0, 0.0);
        for lot in lots.iter_mut().filter(|l| l.remaining > 0.0) {
            if to_match <= 0.0 {
                break;
            }
            let q = to_match.min(lot.remaining);
            // Basis includes the buy commission prorated to this slice.
            pnl += (f.price - lot.price) * q - lot.commission * (q / lot.orig_qty);
            lot.remaining -= q;
            to_match -= q;
            matched += q;
        }
        if matched <= 0.0 {
            continue; // short-side sell — out of scope
        }
        pnl -= f.commission * (matched / f.qty);
        if pnl >= 0.0 {
            continue;
        }
        // Before-leg: window buys with shares STILL held after this
        // sale's matching (the sold lots themselves can't self-flag).
        let before: f64 = lots
            .iter()
            .filter(|l| l.ts >= f.ts - WASH_WINDOW_SECS && l.ts <= f.ts)
            .map(|l| l.remaining)
            .sum();
        // After-leg: any later buy inside the window.
        let after: f64 = fills[i + 1..]
            .iter()
            .filter(|g| g.buy && g.ts <= f.ts + WASH_WINDOW_SECS)
            .map(|g| g.qty)
            .sum();
        let replacement_qty = before + after;
        if replacement_qty <= 0.0 {
            continue;
        }
        let loss = -pnl;
        out.push(WashSale {
            sale_ts: f.ts,
            qty_sold: matched,
            loss,
            replacement_qty,
            disallowed: loss * replacement_qty.min(matched) / matched,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const D: i64 = 86_400;

    fn fill(buy: bool, qty: f64, price: f64, commission: f64, ts: i64) -> Fill {
        Fill { buy, qty, price, commission, ts, flag: false }
    }

    #[test]
    fn rebuy_inside_window_flags_full_loss() {
        // Loss = (40-50)*100 - 1 - 1 = -1002; full-size rebuy at +10d.
        let f = [
            fill(true, 100.0, 50.0, 1.0, 0),
            fill(false, 100.0, 40.0, 1.0, 5 * D),
            fill(true, 100.0, 42.0, 0.0, 15 * D),
        ];
        let w = wash_sales(&f);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].sale_ts, 5 * D);
        assert!((w[0].loss - 1002.0).abs() < 1e-9);
        assert!((w[0].replacement_qty - 100.0).abs() < 1e-9);
        assert!((w[0].disallowed - 1002.0).abs() < 1e-9);
    }

    #[test]
    fn rebuy_outside_window_is_clean() {
        let f = [
            fill(true, 100.0, 50.0, 0.0, 0),
            fill(false, 100.0, 40.0, 0.0, 5 * D),
            fill(true, 100.0, 42.0, 0.0, 5 * D + WASH_WINDOW_SECS + 1),
        ];
        assert!(wash_sales(&f).is_empty());
    }

    #[test]
    fn gain_sale_never_flags() {
        let f = [
            fill(true, 100.0, 50.0, 0.0, 0),
            fill(false, 100.0, 60.0, 0.0, 5 * D),
            fill(true, 100.0, 58.0, 0.0, 6 * D),
        ];
        assert!(wash_sales(&f).is_empty());
    }

    #[test]
    fn partial_replacement_prorates_disallowed() {
        // Loss 1000 on 100 shares, only 50 repurchased → half disallowed.
        let f = [
            fill(true, 100.0, 50.0, 0.0, 0),
            fill(false, 100.0, 40.0, 0.0, 5 * D),
            fill(true, 50.0, 42.0, 0.0, 10 * D),
        ];
        let w = wash_sales(&f);
        assert_eq!(w.len(), 1);
        assert!((w[0].disallowed - 500.0).abs() < 1e-9);
        assert!((w[0].loss - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn before_leg_buy_still_held_flags() {
        // FIFO sells the old lot at a loss; the lot bought 3d earlier
        // remains held — that's the 30-days-BEFORE leg of §1091.
        let f = [
            fill(true, 100.0, 50.0, 0.0, 0),
            fill(true, 100.0, 48.0, 0.0, 25 * D),
            fill(false, 100.0, 40.0, 0.0, 28 * D),
        ];
        let w = wash_sales(&f);
        assert_eq!(w.len(), 1);
        assert!((w[0].loss - 1000.0).abs() < 1e-9);
        assert!((w[0].replacement_qty - 100.0).abs() < 1e-9);
    }

    #[test]
    fn sold_lot_does_not_self_flag() {
        // One buy, one full loss sale, nothing else: the matched lot's
        // remaining is zero, so there is no replacement.
        let f = [
            fill(true, 100.0, 50.0, 0.0, 0),
            fill(false, 100.0, 40.0, 0.0, 5 * D),
        ];
        assert!(wash_sales(&f).is_empty());
    }

    #[test]
    fn short_side_sell_is_skipped() {
        // Sell with no lots held — short open, out of scope, no panic.
        let f = [fill(false, 100.0, 40.0, 0.0, 0), fill(true, 100.0, 38.0, 0.0, D)];
        assert!(wash_sales(&f).is_empty());
    }
}
