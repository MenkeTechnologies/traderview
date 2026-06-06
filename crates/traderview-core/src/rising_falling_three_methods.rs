//! Rising Three Methods / Falling Three Methods — 5-bar continuation
//! candle patterns.
//!
//! Rising Three Methods (bullish continuation):
//!   Bar 1: tall bullish body
//!   Bars 2-4: small bodies CONTAINED within bar 1's range, drifting
//!     LOWER but not breaking bar 1's low
//!   Bar 5: tall bullish body that closes ABOVE bar 1's close
//!
//! Falling Three Methods (bearish continuation): mirrored.
//!
//! The middle 3 bars act as a pause/consolidation within the trend.
//! Bar 5's continuation in the trend direction confirms the
//! consolidation resolved with the trend, not against it.
//!
//! Pure compute. Companion to `mat_hold_pattern`,
//! `three_white_soldiers_crows`, `engulfing_pattern_scanner`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreeMethodsReport {
    pub rising: Vec<bool>,
    pub falling: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> ThreeMethodsReport {
    let n = bars.len();
    let mut report = ThreeMethodsReport {
        rising: vec![false; n],
        falling: vec![false; n],
    };
    if n < 5 {
        return report;
    }
    if bars.iter().any(|b| {
        !b.open.is_finite() || !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()
    }) {
        return report;
    }
    for i in 4..n {
        let (b1, b2, b3, b4, b5) = (bars[i - 4], bars[i - 3], bars[i - 2], bars[i - 1], bars[i]);
        if is_rising_three_methods(b1, b2, b3, b4, b5) {
            report.rising[i] = true;
        }
        if is_falling_three_methods(b1, b2, b3, b4, b5) {
            report.falling[i] = true;
        }
    }
    report
}

fn is_rising_three_methods(b1: Bar, b2: Bar, b3: Bar, b4: Bar, b5: Bar) -> bool {
    let body1 = b1.close - b1.open;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 {
        return false;
    }
    // Bars 2-4: small bodies, contained within bar 1's range.
    let small = |b: Bar| {
        let r = b.high - b.low;
        let bd = (b.close - b.open).abs();
        r > 0.0 && bd < 0.5 * r
    };
    if !(small(b2) && small(b3) && small(b4)) {
        return false;
    }
    let middle_low = b2.low.min(b3.low).min(b4.low);
    let middle_high = b2.high.max(b3.high).max(b4.high);
    if middle_low < b1.low {
        return false;
    }
    if middle_high > b1.high {
        return false;
    }
    // Bar 5: bullish, closes above bar 1's close.
    let body5 = b5.close - b5.open;
    let range5 = b5.high - b5.low;
    if range5 <= 0.0 || body5 <= 0.0 || body5 < 0.6 * range5 {
        return false;
    }
    b5.close > b1.close
}

fn is_falling_three_methods(b1: Bar, b2: Bar, b3: Bar, b4: Bar, b5: Bar) -> bool {
    let body1 = b1.open - b1.close;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 {
        return false;
    }
    let small = |b: Bar| {
        let r = b.high - b.low;
        let bd = (b.close - b.open).abs();
        r > 0.0 && bd < 0.5 * r
    };
    if !(small(b2) && small(b3) && small(b4)) {
        return false;
    }
    let middle_low = b2.low.min(b3.low).min(b4.low);
    let middle_high = b2.high.max(b3.high).max(b4.high);
    if middle_low < b1.low {
        return false;
    }
    if middle_high > b1.high {
        return false;
    }
    let body5 = b5.open - b5.close;
    let range5 = b5.high - b5.low;
    if range5 <= 0.0 || body5 <= 0.0 || body5 < 0.6 * range5 {
        return false;
    }
    b5.close < b1.close
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar {
            open: o,
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[]);
        assert!(r.rising.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0); 5];
        let mut b = bars.clone();
        b[2] = bar(f64::NAN, 101.0, 99.0, 100.0);
        let r = compute(&b);
        assert!(!r.rising.iter().any(|x| *x));
    }

    #[test]
    fn rising_three_methods_detected() {
        // Bar 1: bullish 100→110 (range 99..111).
        // Bars 2-4: small bodies (body/range < 0.5) inside bar 1 range.
        // Bar 5: bullish, closes above bar 1 close (110).
        let bars = vec![
            bar(100.0, 111.0, 99.0, 110.0),
            bar(109.0, 110.0, 105.0, 107.0),
            bar(107.0, 108.0, 104.0, 105.5),
            bar(106.0, 109.0, 103.0, 104.0),
            bar(105.0, 116.0, 104.0, 115.0),
        ];
        let r = compute(&bars);
        assert!(r.rising[4]);
    }

    #[test]
    fn falling_three_methods_detected() {
        let bars = vec![
            bar(110.0, 111.0, 99.0, 100.0),
            bar(101.0, 105.0, 100.5, 103.0),
            bar(104.0, 106.0, 102.0, 105.0),
            bar(105.0, 107.0, 101.0, 106.0),
            bar(105.0, 106.0, 94.0, 95.0),
        ];
        let r = compute(&bars);
        assert!(r.falling[4]);
    }

    #[test]
    fn middle_bars_breaking_range_rejects() {
        // Bar 3 breaks bar 1's low → not valid.
        let bars = vec![
            bar(100.0, 111.0, 99.0, 110.0),
            bar(109.0, 110.0, 105.0, 106.0),
            bar(106.0, 108.0, 95.0, 97.0), // breaks bar 1 low (99)
            bar(107.0, 109.0, 103.0, 105.0),
            bar(105.0, 116.0, 104.0, 115.0),
        ];
        let r = compute(&bars);
        assert!(!r.rising[4]);
    }

    #[test]
    fn fifth_bar_no_breakout_rejects() {
        let bars = vec![
            bar(100.0, 111.0, 99.0, 110.0),
            bar(109.0, 110.0, 105.0, 106.0),
            bar(106.0, 108.0, 104.0, 107.0),
            bar(107.0, 109.0, 103.0, 105.0),
            bar(105.0, 109.0, 104.0, 108.0), // doesn't close above bar 1.close
        ];
        let r = compute(&bars);
        assert!(!r.rising[4]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.rising.len(), 10);
        assert_eq!(r.falling.len(), 10);
    }
}
