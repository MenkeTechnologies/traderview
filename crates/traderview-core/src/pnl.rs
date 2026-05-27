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
/// `mfe_price`. Used for the Exit Efficiency report.
pub fn best_exit_pnl(
    asset_class: AssetClass,
    side: TradeSide,
    qty: Decimal,
    entry: Decimal,
    favorable_extreme: Decimal,
    multiplier: Decimal,
    tick_size: Option<Decimal>,
    tick_value: Option<Decimal>,
) -> Decimal {
    gross_pnl(
        asset_class,
        side,
        qty,
        PricePoint {
            entry,
            exit: favorable_extreme,
            multiplier,
            tick_size,
            tick_value,
        },
    )
}
