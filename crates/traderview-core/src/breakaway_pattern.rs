//! Breakaway Pattern — 5-bar reversal candle pattern.
//!
//! Bullish Breakaway (reversal at lows):
//!   Bar 1: tall bearish body
//!   Bar 2: bearish body that GAPS DOWN from bar 1 (creates "window")
//!   Bars 3-4: bearish bars continuing lower, each making a lower low
//!     than the prior bar
//!   Bar 5: tall bullish body that CLOSES inside the window between
//!     bar 1's close and bar 2's open (closes back through the gap)
//!
//! Bearish Breakaway: mirrored.
//!
//! Pure compute. Companion to `morning_evening_star`, `abandoned_baby`,
//! `mat_hold_pattern`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BreakawayReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> BreakawayReport {
    let n = bars.len();
    let mut report = BreakawayReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
    };
    if n < 5 { return report; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    for i in 4..n {
        let (b1, b2, b3, b4, b5) = (bars[i - 4], bars[i - 3], bars[i - 2],
                                     bars[i - 1], bars[i]);
        if is_bullish_breakaway(b1, b2, b3, b4, b5) {
            report.bullish[i] = true;
        }
        if is_bearish_breakaway(b1, b2, b3, b4, b5) {
            report.bearish[i] = true;
        }
    }
    report
}

fn is_bullish_breakaway(b1: Bar, b2: Bar, b3: Bar, b4: Bar, b5: Bar) -> bool {
    // Bar 1: tall bearish.
    let body1 = b1.open - b1.close;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 { return false; }
    // Bar 2: bearish, gaps down from bar 1.
    if b2.close >= b2.open { return false; }
    if b2.high >= b1.low { return false; }
    // Bars 3, 4: bearish, each making lower lows.
    if b3.close >= b3.open || b4.close >= b4.open { return false; }
    if b3.low >= b2.low { return false; }
    if b4.low >= b3.low { return false; }
    // Bar 5: tall bullish.
    let body5 = b5.close - b5.open;
    let range5 = b5.high - b5.low;
    if range5 <= 0.0 || body5 <= 0.0 || body5 < 0.6 * range5 { return false; }
    // Closes back through the gap (above b2.open).
    b5.close > b2.open && b5.close < b1.close
}

fn is_bearish_breakaway(b1: Bar, b2: Bar, b3: Bar, b4: Bar, b5: Bar) -> bool {
    let body1 = b1.close - b1.open;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 { return false; }
    if b2.close <= b2.open { return false; }
    if b2.low <= b1.high { return false; }
    if b3.close <= b3.open || b4.close <= b4.open { return false; }
    if b3.high <= b2.high { return false; }
    if b4.high <= b3.high { return false; }
    let body5 = b5.open - b5.close;
    let range5 = b5.high - b5.low;
    if range5 <= 0.0 || body5 <= 0.0 || body5 < 0.6 * range5 { return false; }
    b5.close < b2.open && b5.close > b1.close
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar { open: o, high: h, low: l, close: c }
    }

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[]);
        assert!(r.bullish.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.0); 5];
        bars[2] = bar(f64::NAN, 101.0, 99.0, 100.0);
        let r = compute(&bars);
        assert!(!r.bullish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_breakaway_detected() {
        // Bar 1: bearish 110→100, low=99.
        // Bar 2: bearish gap down 96→92, high=96 < bar 1 low=99.
        // Bar 3: bearish, low=89 < bar 2 low=92.
        // Bar 4: bearish, low=86 < bar 3 low=89.
        // Bar 5: bullish closes 95 (between bar 1 close 100 and bar 2 open 96).
        let bars = vec![
            bar(110.0, 110.5, 99.0, 100.0),
            bar(96.0, 96.5, 92.0, 92.0),
            bar(92.0, 92.5, 89.0, 89.0),
            bar(89.0, 89.5, 86.0, 86.5),
            // Bar 5 close 97 must be > bar 2.open (96) AND < bar 1.close (100).
            bar(85.0, 98.0, 84.5, 97.0),
        ];
        let r = compute(&bars);
        assert!(r.bullish[4]);
    }

    #[test]
    fn no_gap_at_bar2_rejects() {
        let bars = vec![
            bar(110.0, 110.5, 99.0, 100.0),
            bar(100.0, 101.0, 95.0, 96.0),    // no gap (high 101 > bar 1 low 99)
            bar(96.0, 96.5, 92.0, 93.0),
            bar(93.0, 93.5, 89.0, 90.0),
            bar(89.0, 96.0, 88.5, 95.0),
        ];
        let r = compute(&bars);
        assert!(!r.bullish[4]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.bullish.len(), 10);
        assert_eq!(r.bearish.len(), 10);
    }
}
