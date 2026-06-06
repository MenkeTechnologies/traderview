//! Three Outside Up / Down — 3-bar confirmation of an engulfing pattern.
//!
//! Bullish Three Outside Up:
//!   Bar 1: bearish bar
//!   Bar 2: bullish bar that ENGULFS bar 1 (bullish engulfing)
//!   Bar 3: bullish bar that closes above bar 2's close
//!
//! Bearish Three Outside Down: mirrored.
//!
//! The third bar's continuation in the engulfing direction confirms
//! the reversal had follow-through.
//!
//! Pure compute. Companion to `engulfing_pattern_scanner`,
//! `three_inside_up_down`, `three_white_soldiers_crows`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreeOutsideReport {
    pub three_outside_up: Vec<bool>,
    pub three_outside_down: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> ThreeOutsideReport {
    let n = bars.len();
    let mut report = ThreeOutsideReport {
        three_outside_up: vec![false; n],
        three_outside_down: vec![false; n],
    };
    if n < 3 {
        return report;
    }
    if bars.iter().any(|b| {
        !b.open.is_finite() || !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()
    }) {
        return report;
    }
    for i in 2..n {
        let (b1, b2, b3) = (bars[i - 2], bars[i - 1], bars[i]);
        if is_three_outside_up(b1, b2, b3) {
            report.three_outside_up[i] = true;
        }
        if is_three_outside_down(b1, b2, b3) {
            report.three_outside_down[i] = true;
        }
    }
    report
}

fn is_three_outside_up(b1: Bar, b2: Bar, b3: Bar) -> bool {
    // Bar 1 bearish.
    if b1.close >= b1.open {
        return false;
    }
    // Bar 2 bullish engulfs bar 1's body.
    if b2.close <= b2.open {
        return false;
    }
    if b2.open > b1.close || b2.close < b1.open {
        return false;
    }
    // Bar 3 bullish, closes above bar 2's close.
    if b3.close <= b3.open {
        return false;
    }
    b3.close > b2.close
}

fn is_three_outside_down(b1: Bar, b2: Bar, b3: Bar) -> bool {
    if b1.close <= b1.open {
        return false;
    }
    if b2.close >= b2.open {
        return false;
    }
    if b2.open < b1.close || b2.close > b1.open {
        return false;
    }
    if b3.close >= b3.open {
        return false;
    }
    b3.close < b2.close
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
        assert!(r.three_outside_up.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.0),
            bar(f64::NAN, 101.0, 99.0, 100.0),
            bar(100.0, 101.0, 99.0, 100.0),
        ];
        let r = compute(&bars);
        assert!(!r.three_outside_up.iter().any(|x| *x));
    }

    #[test]
    fn bullish_three_outside_up_detected() {
        // Bar 1: small bearish.
        // Bar 2: bullish engulfing (open ≤ b1.close, close ≥ b1.open).
        // Bar 3: bullish, closes above bar 2 close.
        let bars = vec![
            bar(105.0, 106.0, 102.0, 103.0),
            bar(102.0, 108.0, 101.0, 107.0),
            bar(107.0, 110.0, 106.0, 109.0),
        ];
        let r = compute(&bars);
        assert!(r.three_outside_up[2]);
    }

    #[test]
    fn bearish_three_outside_down_detected() {
        let bars = vec![
            bar(103.0, 106.0, 102.0, 105.0),
            bar(106.0, 107.0, 100.0, 101.0),
            bar(101.0, 102.0, 96.0, 97.0),
        ];
        let r = compute(&bars);
        assert!(r.three_outside_down[2]);
    }

    #[test]
    fn no_engulfing_rejects_pattern() {
        let bars = vec![
            bar(105.0, 106.0, 102.0, 103.0),
            bar(104.0, 108.0, 103.5, 107.0), // open 104 > b1.close 103
            bar(107.0, 110.0, 106.0, 109.0),
        ];
        let r = compute(&bars);
        assert!(!r.three_outside_up[2]);
    }

    #[test]
    fn third_bar_no_continuation_rejected() {
        let bars = vec![
            bar(105.0, 106.0, 102.0, 103.0),
            bar(102.0, 108.0, 101.0, 107.0),
            bar(107.0, 107.5, 105.0, 106.0), // bullish but closes below b2.close 107
        ];
        let r = compute(&bars);
        assert!(!r.three_outside_up[2]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.three_outside_up.len(), 10);
        assert_eq!(r.three_outside_down.len(), 10);
    }
}
