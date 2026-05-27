//! Trade quality score — measures how well the trader entered and
//! exited *relative to the bar's available range*. Two components:
//!
//!   entry_quality  = 1 - (entry - bar_low ) / (bar_high - bar_low)  for LONG
//!   exit_quality   = (exit  - bar_low ) / (bar_high - bar_low)      for LONG
//!
//! For SHORT both are inverted (favorable = sell near high, cover near low).
//! Each is a 0..=1 score. The composite `quality` averages the two.
//!
//! Pure compute. The intuition: if you entered at the worst price of
//! the entry bar (top of range on a long) you scored 0; perfect bottom
//! scored 1. Same for exit. Pinned to bar ranges so the metric is
//! self-calibrating and doesn't need a reference benchmark.

use crate::models::TradeSide;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BarRange {
    pub low: Decimal,
    pub high: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityScore {
    pub entry_quality: f64,
    pub exit_quality: f64,
    /// Average of entry + exit. None when either component couldn't be
    /// computed (zero range, missing data).
    pub composite: Option<f64>,
}

pub fn score(
    side: TradeSide,
    entry_price: Decimal,
    entry_bar: BarRange,
    exit_price: Decimal,
    exit_bar: BarRange,
) -> QualityScore {
    let entry_q = position_in_range(entry_price, entry_bar);
    let exit_q  = position_in_range(exit_price,  exit_bar);
    // Invert orientation per side.
    let (eq, xq) = match (entry_q, exit_q) {
        (Some(e), Some(x)) => match side {
            // Long: low-in-range entry is good (e=0 → great), high-in-range
            // exit is good (x=1 → great).
            TradeSide::Long  => (1.0 - e, x),
            // Short: high-in-range entry is good (e=1 → great becomes 1),
            // low-in-range exit is good (x=0 → great becomes 1).
            TradeSide::Short => (e, 1.0 - x),
        },
        _ => (f64::NAN, f64::NAN),
    };
    let composite = if eq.is_nan() || xq.is_nan() {
        None
    } else {
        Some((eq + xq) / 2.0)
    };
    QualityScore {
        entry_quality: if eq.is_nan() { 0.0 } else { eq.clamp(0.0, 1.0) },
        exit_quality:  if xq.is_nan() { 0.0 } else { xq.clamp(0.0, 1.0) },
        composite,
    }
}

/// Position-in-range (0..=1) for a price within a bar. None if the bar
/// has zero range (high == low, e.g. limit-up day) — avoids divide by zero.
fn position_in_range(price: Decimal, bar: BarRange) -> Option<f64> {
    let range = bar.high - bar.low;
    if range.is_zero() { return None; }
    let pos = (price - bar.low) / range;
    Some(to_f64(pos).clamp(0.0, 1.0))
}

fn to_f64(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }
    fn bar(low: &str, high: &str) -> BarRange {
        BarRange { low: d(low), high: d(high) }
    }

    // ─── long ─────────────────────────────────────────────────────────

    #[test]
    fn long_perfect_entry_and_exit_scores_one() {
        // Entered at the low of the entry bar, exited at the high of
        // the exit bar.
        let s = score(
            TradeSide::Long,
            d("100"), bar("100", "110"),    // entry at low → quality 1
            d("120"), bar("110", "120"),    // exit at high → quality 1
        );
        assert_eq!(s.entry_quality, 1.0);
        assert_eq!(s.exit_quality, 1.0);
        assert_eq!(s.composite, Some(1.0));
    }

    #[test]
    fn long_worst_entry_and_exit_scores_zero() {
        let s = score(
            TradeSide::Long,
            d("110"), bar("100", "110"),    // entry at high → quality 0
            d("110"), bar("110", "120"),    // exit at low → quality 0
        );
        assert_eq!(s.entry_quality, 0.0);
        assert_eq!(s.exit_quality, 0.0);
        assert_eq!(s.composite, Some(0.0));
    }

    #[test]
    fn long_middle_of_range_scores_half() {
        let s = score(
            TradeSide::Long,
            d("105"), bar("100", "110"),    // entry mid → quality 0.5
            d("115"), bar("110", "120"),    // exit mid → quality 0.5
        );
        assert!((s.entry_quality - 0.5).abs() < 1e-9);
        assert!((s.exit_quality - 0.5).abs() < 1e-9);
        assert_eq!(s.composite, Some(0.5));
    }

    // ─── short ────────────────────────────────────────────────────────

    #[test]
    fn short_perfect_entry_and_exit_scores_one() {
        // Short the high, cover the low.
        let s = score(
            TradeSide::Short,
            d("110"), bar("100", "110"),    // entry at high → great for short
            d("100"), bar("100", "110"),    // exit  at low  → great for short
        );
        assert_eq!(s.entry_quality, 1.0);
        assert_eq!(s.exit_quality, 1.0);
    }

    #[test]
    fn short_worst_entry_and_exit_scores_zero() {
        // Short at low, cover at high.
        let s = score(
            TradeSide::Short,
            d("100"), bar("100", "110"),
            d("110"), bar("100", "110"),
        );
        assert_eq!(s.entry_quality, 0.0);
        assert_eq!(s.exit_quality, 0.0);
    }

    // ─── degenerate bars ──────────────────────────────────────────────

    #[test]
    fn zero_range_entry_bar_returns_none_composite() {
        // Limit-up / limit-down day, no range to score against.
        let s = score(
            TradeSide::Long,
            d("100"), bar("100", "100"),    // zero range
            d("110"), bar("105", "115"),
        );
        assert!(s.composite.is_none(),
            "any side with zero-range bar must yield None composite");
    }

    #[test]
    fn zero_range_exit_bar_returns_none_composite() {
        let s = score(
            TradeSide::Long,
            d("100"), bar("95",  "105"),
            d("110"), bar("110", "110"),
        );
        assert!(s.composite.is_none());
    }

    #[test]
    fn price_outside_bar_clamps_into_range() {
        // Entry below the bar low — pathological data, clamp to 0.
        let s = score(
            TradeSide::Long,
            d("95"), bar("100", "110"),     // entry below low — clamp
            d("115"), bar("110", "120"),
        );
        // Clamped position would be 0 → quality (1-0) = 1.
        assert_eq!(s.entry_quality, 1.0);
    }
}
