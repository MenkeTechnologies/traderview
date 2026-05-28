//! Tasuki Gap — 3-bar continuation pattern through a gap window.
//!
//! Bullish Upside Tasuki Gap:
//!   Bar 1: bullish body
//!   Bar 2: bullish body that GAPS UP (bar 2.low > bar 1.high)
//!   Bar 3: bearish body that opens inside bar 2's body and closes
//!     INSIDE the gap window (between bar 1.high and bar 2.low) WITHOUT
//!     fully closing the gap
//!
//! Bearish Downside Tasuki Gap: mirrored.
//!
//! The unfilled gap signals trend continuation — bears tried to close
//! the gap but failed.
//!
//! Pure compute. Companion to `gap_classifier`, `fair_value_gap`,
//! `morning_evening_star`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TasukiGapReport {
    pub upside: Vec<bool>,
    pub downside: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> TasukiGapReport {
    let n = bars.len();
    let mut report = TasukiGapReport {
        upside: vec![false; n],
        downside: vec![false; n],
    };
    if n < 3 { return report; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    for i in 2..n {
        let (b1, b2, b3) = (bars[i - 2], bars[i - 1], bars[i]);
        if is_upside_tasuki(b1, b2, b3) { report.upside[i] = true; }
        if is_downside_tasuki(b1, b2, b3) { report.downside[i] = true; }
    }
    report
}

fn is_upside_tasuki(b1: Bar, b2: Bar, b3: Bar) -> bool {
    // Bars 1 and 2 bullish.
    if b1.close <= b1.open || b2.close <= b2.open { return false; }
    // Gap up between bar 1 and bar 2.
    if b2.low <= b1.high { return false; }
    // Bar 3 bearish, opens inside bar 2's body.
    if b3.close >= b3.open { return false; }
    let b2_body_high = b2.close;
    let b2_body_low = b2.open;
    if b3.open <= b2_body_low || b3.open >= b2_body_high { return false; }
    // Bar 3 closes inside the gap window (between b1.high and b2.low)
    // but does NOT close fully below b1.high (gap remains partially open).
    b3.close < b2.low && b3.close > b1.high
}

fn is_downside_tasuki(b1: Bar, b2: Bar, b3: Bar) -> bool {
    if b1.close >= b1.open || b2.close >= b2.open { return false; }
    if b2.high >= b1.low { return false; }
    if b3.close <= b3.open { return false; }
    let b2_body_high = b2.open;
    let b2_body_low = b2.close;
    if b3.open <= b2_body_low || b3.open >= b2_body_high { return false; }
    b3.close > b2.high && b3.close < b1.low
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
        assert!(r.upside.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0),
                        bar(f64::NAN, 101.0, 99.0, 100.0),
                        bar(100.0, 101.0, 99.0, 100.0)];
        let r = compute(&bars);
        assert!(!r.upside.iter().any(|x| *x));
    }

    #[test]
    fn upside_tasuki_gap_detected() {
        // Bar 1: bullish 100→105, high=105.
        // Bar 2: bullish 108→112, low=107 (gap up: 107 > 105).
        // Bar 3: bearish opens 110 (inside bar 2 body 108..112), closes
        //   106 (between b1.high=105 and b2.low=107? Need 106 > 105 AND 106 < 107).
        let bars = vec![
            bar(100.0, 105.0, 99.5, 105.0),
            bar(108.0, 112.5, 107.0, 112.0),
            bar(110.0, 110.5, 105.5, 106.0),
        ];
        let r = compute(&bars);
        assert!(r.upside[2]);
    }

    #[test]
    fn downside_tasuki_gap_detected() {
        let bars = vec![
            bar(110.0, 110.5, 105.0, 105.0),
            bar(102.0, 103.0, 98.0, 98.0),
            bar(100.0, 104.5, 99.5, 104.0),
        ];
        let r = compute(&bars);
        assert!(r.downside[2]);
    }

    #[test]
    fn no_gap_rejects() {
        let bars = vec![
            bar(100.0, 105.0, 99.5, 105.0),
            bar(104.0, 110.0, 103.0, 109.0),    // no gap (low 103 < bar1.high 105)
            bar(108.0, 109.0, 105.0, 105.5),
        ];
        let r = compute(&bars);
        assert!(!r.upside[2]);
    }

    #[test]
    fn gap_fully_closed_rejects() {
        // Bar 3 closes BELOW bar 1.high → gap fully closed → no continuation.
        let bars = vec![
            bar(100.0, 105.0, 99.5, 105.0),
            bar(108.0, 112.5, 107.0, 112.0),
            bar(110.0, 110.5, 100.0, 101.0),    // close 101 < bar 1.high 105
        ];
        let r = compute(&bars);
        assert!(!r.upside[2]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.upside.len(), 10);
        assert_eq!(r.downside.len(), 10);
    }
}
