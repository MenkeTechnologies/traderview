//! Mat Hold Pattern — 5-bar bullish continuation (more reliable
//! cousin of Rising Three Methods, with stricter gap and direction
//! requirements).
//!
//!   Bar 1: tall bullish body
//!   Bar 2: small bullish body that GAPS UP above bar 1's close
//!     (creating a "rising window")
//!   Bars 3-4: small bodies that drift down toward but do NOT close
//!     below bar 1's close
//!   Bar 5: bullish body that CLOSES above bar 2's high
//!
//! Bearish Mat Hold: mirrored.
//!
//! Pure compute. Companion to `rising_falling_three_methods`,
//! `three_white_soldiers_crows`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatHoldReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> MatHoldReport {
    let n = bars.len();
    let mut report = MatHoldReport {
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
        if is_bullish_mat_hold(b1, b2, b3, b4, b5) {
            report.bullish[i] = true;
        }
        if is_bearish_mat_hold(b1, b2, b3, b4, b5) {
            report.bearish[i] = true;
        }
    }
    report
}

fn is_bullish_mat_hold(b1: Bar, b2: Bar, b3: Bar, b4: Bar, b5: Bar) -> bool {
    let body1 = b1.close - b1.open;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 { return false; }
    // Bar 2: small bullish, gaps up.
    let body2 = b2.close - b2.open;
    let range2 = b2.high - b2.low;
    if range2 <= 0.0 || body2 <= 0.0 || body2 > 0.5 * range2 { return false; }
    if b2.low <= b1.close { return false; }
    // Bars 3-4: small bodies, do not close below bar 1's close.
    let small_close_above = |b: Bar, threshold: f64| {
        let r = b.high - b.low;
        let bd = (b.close - b.open).abs();
        r > 0.0 && bd < 0.6 * r && b.close >= threshold
    };
    if !(small_close_above(b3, b1.close) && small_close_above(b4, b1.close)) { return false; }
    // Bar 5: bullish, closes above bar 2's high.
    let body5 = b5.close - b5.open;
    let range5 = b5.high - b5.low;
    if range5 <= 0.0 || body5 <= 0.0 || body5 < 0.6 * range5 { return false; }
    b5.close > b2.high
}

fn is_bearish_mat_hold(b1: Bar, b2: Bar, b3: Bar, b4: Bar, b5: Bar) -> bool {
    let body1 = b1.open - b1.close;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 { return false; }
    let body2 = b2.open - b2.close;
    let range2 = b2.high - b2.low;
    if range2 <= 0.0 || body2 <= 0.0 || body2 > 0.5 * range2 { return false; }
    if b2.high >= b1.close { return false; }
    let small_close_below = |b: Bar, threshold: f64| {
        let r = b.high - b.low;
        let bd = (b.close - b.open).abs();
        r > 0.0 && bd < 0.6 * r && b.close <= threshold
    };
    if !(small_close_below(b3, b1.close) && small_close_below(b4, b1.close)) { return false; }
    let body5 = b5.open - b5.close;
    let range5 = b5.high - b5.low;
    if range5 <= 0.0 || body5 <= 0.0 || body5 < 0.6 * range5 { return false; }
    b5.close < b2.low
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
    fn bullish_mat_hold_detected() {
        // Bar 1: bullish 100→110.
        // Bar 2: small bullish 112→113, low 111.5 > bar 1 close 110 (gap up).
        // Bars 3,4: small bars closing above 110.
        // Bar 5: bullish closing above bar 2 high (113).
        let bars = vec![
            bar(100.0, 111.0, 99.0, 110.0),
            bar(112.0, 113.5, 111.5, 113.0),
            bar(112.5, 113.0, 111.0, 111.5),
            bar(111.5, 113.0, 110.5, 111.0),
            bar(112.0, 116.0, 111.5, 115.0),
        ];
        let r = compute(&bars);
        assert!(r.bullish[4]);
    }

    #[test]
    fn no_gap_at_bar2_rejects() {
        let bars = vec![
            bar(100.0, 111.0, 99.0, 110.0),
            bar(110.5, 112.0, 109.5, 111.5),    // no gap up (low 109.5 < bar 1 close 110)
            bar(111.0, 112.0, 110.0, 110.5),
            bar(110.5, 112.0, 109.5, 111.0),
            bar(111.0, 116.0, 110.0, 115.0),
        ];
        let r = compute(&bars);
        assert!(!r.bullish[4]);
    }

    #[test]
    fn middle_bar_closing_below_bar1_close_rejects() {
        let bars = vec![
            bar(100.0, 111.0, 99.0, 110.0),
            bar(112.0, 113.5, 111.5, 113.0),
            bar(111.5, 112.0, 105.0, 107.0),    // close 107 < bar 1 close 110
            bar(107.0, 108.0, 106.0, 107.5),
            bar(108.0, 116.0, 107.5, 115.0),
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
