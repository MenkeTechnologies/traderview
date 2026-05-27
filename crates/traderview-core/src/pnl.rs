//! Per-asset P&L computation.
//!
//! Stocks   : `(exit_price - entry_price) * qty * sign(side)`
//! Options  : same as stocks but multiplied by `multiplier` (100 for US equity options)
//! Futures  : `(exit_price - entry_price) / tick_size * tick_value * qty * sign(side)`
//!            (collapses to `(exit - entry) * tick_value / tick_size * qty`)
//!            multiplier may double as point value when tick_size/value are unset.
//! Forex    : `(exit_price - entry_price) * qty * sign(side)` — qty is units of base.
//!            The reporting-currency conversion lives in `crate::fx`.

use crate::models::{AssetClass, TradeSide};
use rust_decimal::Decimal;

/// Per-unit price-difference value, given an entry and exit and the asset's
/// multiplier / tick spec. `qty` is *not* applied here — callers multiply.
#[derive(Debug, Clone, Copy)]
pub struct PricePoint {
    pub entry: Decimal,
    pub exit: Decimal,
    pub multiplier: Decimal,
    pub tick_size: Option<Decimal>,
    pub tick_value: Option<Decimal>,
}

/// Gross P&L for `qty` units traded `side` from `entry` to `exit`, accounting
/// for the asset class. Does **not** subtract fees — that happens at the trade
/// roll-up layer (net_pnl = gross_pnl - fees).
pub fn gross_pnl(
    asset_class: AssetClass,
    side: TradeSide,
    qty: Decimal,
    pp: PricePoint,
) -> Decimal {
    let sign = side.sign();
    let dprice = pp.exit - pp.entry;

    match asset_class {
        AssetClass::Stock | AssetClass::Forex => dprice * qty * sign,
        AssetClass::Option => dprice * qty * pp.multiplier * sign,
        AssetClass::Future => {
            // Tick-based future: convert price delta into ticks, then dollars.
            match (pp.tick_size, pp.tick_value) {
                (Some(ts), Some(tv)) if !ts.is_zero() => (dprice / ts) * tv * qty * sign,
                // Fall back to multiplier-as-point-value if tick spec is missing.
                _ => dprice * pp.multiplier * qty * sign,
            }
        }
    }
}

/// "Best exit" P&L assuming the trade had exited at the favorable extreme of
/// `mfe_price`. Used for the Exit Efficiency report. Build the `PricePoint`
/// with `exit = favorable_extreme`.
pub fn best_exit_pnl(
    asset_class: AssetClass,
    side: TradeSide,
    qty: Decimal,
    favorable: PricePoint,
) -> Decimal {
    gross_pnl(asset_class, side, qty, favorable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn pp(entry: &str, exit: &str) -> PricePoint {
        PricePoint {
            entry: d(entry), exit: d(exit),
            multiplier: Decimal::ONE,
            tick_size: None, tick_value: None,
        }
    }

    #[test]
    fn stock_long_gain() {
        // 100 sh × ($110 - $100) × +1 = $1000.
        let p = gross_pnl(AssetClass::Stock, TradeSide::Long, d("100"), pp("100", "110"));
        assert_eq!(p, d("1000"));
    }

    #[test]
    fn stock_long_loss() {
        // 100 sh × ($95 - $100) × +1 = -$500.
        let p = gross_pnl(AssetClass::Stock, TradeSide::Long, d("100"), pp("100", "95"));
        assert_eq!(p, d("-500"));
    }

    #[test]
    fn stock_short_inverts_sign() {
        // Short 100 sh from $100 down to $90 → +$1000 (profit on short).
        let p = gross_pnl(AssetClass::Stock, TradeSide::Short, d("100"), pp("100", "90"));
        assert_eq!(p, d("1000"));
        // Short 100 sh from $100 up to $110 → -$1000 (loss on short).
        let p2 = gross_pnl(AssetClass::Stock, TradeSide::Short, d("100"), pp("100", "110"));
        assert_eq!(p2, d("-1000"));
    }

    #[test]
    fn option_uses_multiplier() {
        // 1 contract × ($7 - $5) × 100 mult × +1 = $200.
        let mut p = pp("5", "7"); p.multiplier = d("100");
        let v = gross_pnl(AssetClass::Option, TradeSide::Long, d("1"), p);
        assert_eq!(v, d("200"));
    }

    #[test]
    fn future_with_tick_spec_uses_ticks_to_dollars() {
        // ES: tick_size 0.25, tick_value $12.50. Long 1 contract from 4000 to 4002.
        // ticks = (4002 - 4000) / 0.25 = 8, dollars = 8 × $12.50 × 1 = $100.
        let mut p = pp("4000", "4002");
        p.tick_size = Some(d("0.25"));
        p.tick_value = Some(d("12.50"));
        let v = gross_pnl(AssetClass::Future, TradeSide::Long, d("1"), p);
        assert_eq!(v, d("100.00"));
    }

    #[test]
    fn future_without_tick_spec_falls_back_to_multiplier() {
        // 1 ES at $50/pt × +2 pts = $100. No tick spec → use multiplier as $/pt.
        let mut p = pp("4000", "4002"); p.multiplier = d("50");
        let v = gross_pnl(AssetClass::Future, TradeSide::Long, d("1"), p);
        assert_eq!(v, d("100"));
    }

    #[test]
    fn future_zero_tick_size_falls_back_to_multiplier() {
        // Bad input: tick_size=0 must not divide-by-zero.
        let mut p = pp("4000", "4002");
        p.tick_size = Some(Decimal::ZERO);
        p.tick_value = Some(d("12.50"));
        p.multiplier = d("50");
        let v = gross_pnl(AssetClass::Future, TradeSide::Long, d("1"), p);
        // Fell back to multiplier × delta = 50 × 2 = 100.
        assert_eq!(v, d("100"));
    }

    #[test]
    fn forex_uses_per_unit_delta_times_qty() {
        // 10,000 units EUR/USD from 1.0800 to 1.0850 = 10000 × 0.005 = $50.
        let p = gross_pnl(AssetClass::Forex, TradeSide::Long, d("10000"), pp("1.0800", "1.0850"));
        assert_eq!(p, d("50.0000"));
    }

    #[test]
    fn best_exit_pnl_delegates_to_gross_pnl() {
        // Should produce the same value as gross_pnl with exit = favorable_extreme.
        let v1 = best_exit_pnl(AssetClass::Stock, TradeSide::Long, d("100"), pp("50", "60"));
        let v2 = gross_pnl(AssetClass::Stock, TradeSide::Long, d("100"), pp("50", "60"));
        assert_eq!(v1, v2);
    }
}
