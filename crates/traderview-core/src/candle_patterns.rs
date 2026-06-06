//! Classic candlestick pattern scanner — 6 high-value patterns:
//!   - **Doji**: |close − open| ≤ 10% of range
//:   - **Hammer**: small body near the top, lower wick ≥ 2× body
//!   - **ShootingStar**: small body near the bottom, upper wick ≥ 2× body
//!   - **BullishEngulfing**: prior red body fully engulfed by green body
//!   - **BearishEngulfing**: mirror — prior green body engulfed by red
//!   - **InsideBar**: current bar's range fully inside prior bar's range
//!
//! Distinct from the existing `candlestick_patterns` (which has its
//! own pattern set). This module focuses on the six most-used patterns
//! and emits a per-bar bitmask + cumulative bull/bear scores.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct PatternFlags {
    pub doji: bool,
    pub hammer: bool,
    pub shooting_star: bool,
    pub bullish_engulfing: bool,
    pub bearish_engulfing: bool,
    pub inside_bar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternReport {
    pub per_bar: Vec<PatternFlags>,
    pub bullish_indices: Vec<usize>,
    pub bearish_indices: Vec<usize>,
}

pub fn scan(bars: &[Bar]) -> PatternReport {
    let n = bars.len();
    let mut report = PatternReport {
        per_bar: vec![PatternFlags::default(); n],
        bullish_indices: Vec::new(),
        bearish_indices: Vec::new(),
    };
    for (i, b) in bars.iter().enumerate() {
        if !b.open.is_finite()
            || !b.high.is_finite()
            || !b.low.is_finite()
            || !b.close.is_finite()
            || b.high < b.low
        {
            continue;
        }
        let range = b.high - b.low;
        if range <= 0.0 {
            continue;
        }
        let body = (b.close - b.open).abs();
        let upper_wick = b.high - b.close.max(b.open);
        let lower_wick = b.close.min(b.open) - b.low;
        let mut flags = PatternFlags::default();
        // Doji: tiny body relative to range.
        if body <= 0.10 * range {
            flags.doji = true;
        }
        // Hammer: small body near top, long lower wick (≥ 2× body).
        if body > 0.0 && lower_wick >= 2.0 * body && upper_wick <= 0.5 * body {
            flags.hammer = true;
        }
        // Shooting star: small body near bottom, long upper wick.
        if body > 0.0 && upper_wick >= 2.0 * body && lower_wick <= 0.5 * body {
            flags.shooting_star = true;
        }
        // Engulfing + inside-bar both need a prior bar.
        if i > 0 {
            let prev = bars[i - 1];
            if prev.open.is_finite()
                && prev.high.is_finite()
                && prev.low.is_finite()
                && prev.close.is_finite()
                && prev.high >= prev.low
            {
                let prev_body_lo = prev.open.min(prev.close);
                let prev_body_hi = prev.open.max(prev.close);
                let curr_body_lo = b.open.min(b.close);
                let curr_body_hi = b.open.max(b.close);
                let prev_red = prev.close < prev.open;
                let prev_green = prev.close > prev.open;
                let curr_green = b.close > b.open;
                let curr_red = b.close < b.open;
                // Bullish engulfing: prev red, current green, current body
                // fully covers prev body.
                if prev_red
                    && curr_green
                    && curr_body_lo <= prev_body_lo
                    && curr_body_hi >= prev_body_hi
                {
                    flags.bullish_engulfing = true;
                }
                if prev_green
                    && curr_red
                    && curr_body_lo <= prev_body_lo
                    && curr_body_hi >= prev_body_hi
                {
                    flags.bearish_engulfing = true;
                }
                // Inside bar: current high < prev high AND current low > prev low.
                if b.high < prev.high && b.low > prev.low {
                    flags.inside_bar = true;
                }
            }
        }
        if flags.hammer || flags.bullish_engulfing {
            report.bullish_indices.push(i);
        }
        if flags.shooting_star || flags.bearish_engulfing {
            report.bearish_indices.push(i);
        }
        report.per_bar[i] = flags;
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar {
            open: o,
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_input_yields_empty_report() {
        let r = scan(&[]);
        assert!(r.per_bar.is_empty());
        assert!(r.bullish_indices.is_empty());
    }

    #[test]
    fn doji_detected_when_body_tiny_vs_range() {
        // O=100, C=100.05 (body=0.05), H=110, L=90 (range=20) → body/range=0.0025.
        let r = scan(&[b(100.0, 110.0, 90.0, 100.05)]);
        assert!(r.per_bar[0].doji);
    }

    #[test]
    fn hammer_detected_long_lower_wick_small_body_at_top() {
        // O=100, C=101 (body=1, green near top), H=101.5, L=95 (lower wick = 5).
        let r = scan(&[b(100.0, 101.5, 95.0, 101.0)]);
        assert!(r.per_bar[0].hammer);
    }

    #[test]
    fn shooting_star_detected_long_upper_wick_small_body_at_bottom() {
        let r = scan(&[b(100.0, 110.0, 99.5, 101.0)]);
        assert!(r.per_bar[0].shooting_star);
    }

    #[test]
    fn bullish_engulfing_detected() {
        // Prev red: O=110, C=100. Current green: O=99, C=112. Engulfs.
        let r = scan(&[b(110.0, 112.0, 99.0, 100.0), b(99.0, 115.0, 98.0, 112.0)]);
        assert!(r.per_bar[1].bullish_engulfing);
        assert!(r.bullish_indices.contains(&1));
    }

    #[test]
    fn bearish_engulfing_detected() {
        let r = scan(&[
            b(100.0, 112.0, 99.0, 110.0), // prev green
            b(112.0, 115.0, 98.0, 99.0),  // current red engulfs body 100..110
        ]);
        assert!(r.per_bar[1].bearish_engulfing);
        assert!(r.bearish_indices.contains(&1));
    }

    #[test]
    fn inside_bar_detected_when_range_inside_prior() {
        let r = scan(&[b(100.0, 110.0, 90.0, 105.0), b(102.0, 108.0, 95.0, 104.0)]);
        assert!(r.per_bar[1].inside_bar);
    }

    #[test]
    fn nan_bars_skipped_safely() {
        let mut bars = vec![b(100.0, 110.0, 90.0, 100.05); 5];
        bars[2] = b(f64::NAN, f64::NAN, f64::NAN, f64::NAN);
        let r = scan(&bars);
        // No panic, output length matches.
        assert_eq!(r.per_bar.len(), 5);
        assert!(!r.per_bar[2].doji); // NaN bar has no flags
    }

    #[test]
    fn engulfing_requires_a_prior_bar() {
        // First bar can't be engulfing.
        let r = scan(&[b(100.0, 105.0, 99.0, 104.0)]);
        assert!(!r.per_bar[0].bullish_engulfing);
        assert!(!r.per_bar[0].bearish_engulfing);
    }

    #[test]
    fn zero_range_bar_skipped() {
        let r = scan(&[b(100.0, 100.0, 100.0, 100.0)]);
        assert!(!r.per_bar[0].doji);
    }
}
