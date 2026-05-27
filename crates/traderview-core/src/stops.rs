//! ATR-based + trailing stop calculators.
//!
//! Two utilities the new-trade form + Risk Gate use together:
//!   * atr_stop(entry, atr, multiple, side) → suggested stop price
//!     based on N×ATR distance from entry. Standard scalping uses 1×ATR,
//!     swing 2-3×ATR.
//!   * trailing_stop(side, entry, peak, atr, multiple) → stop that
//!     tracks the favorable extreme (peak for long, trough for short)
//!     and never relaxes.
//!
//! Pure compute.

use crate::models::TradeSide;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Suggested stop loss price at `entry ± multiple × atr`.
/// For longs the stop is BELOW entry; for shorts ABOVE entry.
pub fn atr_stop(
    side: TradeSide,
    entry: Decimal,
    atr: Decimal,
    multiple: Decimal,
) -> Decimal {
    let dist = atr * multiple;
    match side {
        TradeSide::Long  => entry - dist,
        TradeSide::Short => entry + dist,
    }
}

/// Trailing stop that follows the favorable extreme.
///
/// `extreme` is the highest high reached since entry (for longs) or the
/// lowest low (for shorts). The trailing stop sits `multiple × atr`
/// behind that point and never moves against the trader — when called
/// with a less-favorable extreme than before, the caller is expected
/// to pass the running max/min, not the current price.
pub fn trailing_stop(
    side: TradeSide,
    extreme: Decimal,
    atr: Decimal,
    multiple: Decimal,
) -> Decimal {
    let dist = atr * multiple;
    match side {
        TradeSide::Long  => extreme - dist,
        TradeSide::Short => extreme + dist,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChandelierStop {
    pub stop: Decimal,
    pub distance: Decimal,
    pub distance_pct: f64,
}

/// Chandelier exit — popular trailing-stop convention:
///   long  stop = max_high_n_periods - 3 × ATR
///   short stop = min_low_n_periods  + 3 × ATR
///
/// Convenience that wraps `trailing_stop` with the conventional 3×ATR
/// multiple AND adds the % distance for UI display.
pub fn chandelier(
    side: TradeSide,
    extreme: Decimal,
    atr: Decimal,
) -> ChandelierStop {
    let multiple = Decimal::from(3);
    let stop = trailing_stop(side, extreme, atr, multiple);
    let distance = (extreme - stop).abs();
    let distance_pct = if extreme.is_zero() { 0.0 } else {
        to_f64(distance) / to_f64(extreme.abs()) * 100.0
    };
    ChandelierStop { stop, distance, distance_pct }
}

fn to_f64(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    // ─── atr_stop ──────────────────────────────────────────────────────

    #[test]
    fn long_atr_stop_is_entry_minus_distance() {
        let s = atr_stop(TradeSide::Long, d("100"), d("2"), d("1.5"));
        // 100 - 2×1.5 = 97.
        assert_eq!(s, d("97.0"));
    }

    #[test]
    fn short_atr_stop_is_entry_plus_distance() {
        let s = atr_stop(TradeSide::Short, d("100"), d("2"), d("1.5"));
        assert_eq!(s, d("103.0"));
    }

    #[test]
    fn zero_atr_stop_equals_entry() {
        let s = atr_stop(TradeSide::Long, d("100"), Decimal::ZERO, d("1"));
        assert_eq!(s, d("100"));
    }

    #[test]
    fn multiple_can_be_fractional() {
        // 0.5×ATR — half-ATR scalper stop.
        let s = atr_stop(TradeSide::Long, d("100"), d("2"), d("0.5"));
        assert_eq!(s, d("99.0"));
    }

    // ─── trailing_stop ─────────────────────────────────────────────────

    #[test]
    fn long_trailing_stop_follows_high() {
        // Entry was 100. Peak so far = 110. Stop = 110 - 2×2 = 106.
        let s = trailing_stop(TradeSide::Long, d("110"), d("2"), d("2"));
        assert_eq!(s, d("106"));
    }

    #[test]
    fn short_trailing_stop_follows_low() {
        // Entry was 100. Trough so far = 90. Stop = 90 + 2×2 = 94.
        let s = trailing_stop(TradeSide::Short, d("90"), d("2"), d("2"));
        assert_eq!(s, d("94"));
    }

    // ─── chandelier ────────────────────────────────────────────────────

    #[test]
    fn chandelier_uses_3x_atr_by_default() {
        // Long, peak 100, ATR 2 → stop 100 - 6 = 94.
        let c = chandelier(TradeSide::Long, d("100"), d("2"));
        assert_eq!(c.stop, d("94"));
        assert_eq!(c.distance, d("6"));
    }

    #[test]
    fn chandelier_distance_pct_uses_extreme_as_base() {
        let c = chandelier(TradeSide::Long, d("100"), d("2"));
        // 6 / 100 = 6%.
        assert!((c.distance_pct - 6.0).abs() < 1e-9);
    }

    #[test]
    fn chandelier_short_inverted() {
        // Short, trough 100, ATR 2 → stop 100 + 6 = 106.
        let c = chandelier(TradeSide::Short, d("100"), d("2"));
        assert_eq!(c.stop, d("106"));
    }

    #[test]
    fn chandelier_zero_extreme_no_divide_by_zero() {
        let c = chandelier(TradeSide::Long, Decimal::ZERO, d("2"));
        assert_eq!(c.distance_pct, 0.0);
    }
}
