//! Three Inside Up / Down — 3-bar confirmation of a harami pattern.
//!
//! Bullish Three Inside Up:
//!   Bar 1: tall bearish (body ≥ 60% of range)
//!   Bar 2: bullish bar inside bar 1's body (bullish harami)
//!   Bar 3: bullish bar that CLOSES above bar 1's high
//!
//! Bearish Three Inside Down: mirrored.
//!
//! The third bar's close ABOVE bar 1's high (bullish) or BELOW bar 1's
//! low (bearish) confirms the harami reversal had follow-through.
//!
//! Pure compute. Companion to `harami_pattern`, `three_white_soldiers_crows`,
//! `morning_evening_star`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreeInsideReport {
    pub three_inside_up: Vec<bool>,
    pub three_inside_down: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> ThreeInsideReport {
    let n = bars.len();
    let mut report = ThreeInsideReport {
        three_inside_up: vec![false; n],
        three_inside_down: vec![false; n],
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
        if is_three_inside_up(b1, b2, b3) {
            report.three_inside_up[i] = true;
        }
        if is_three_inside_down(b1, b2, b3) {
            report.three_inside_down[i] = true;
        }
    }
    report
}

fn is_three_inside_up(b1: Bar, b2: Bar, b3: Bar) -> bool {
    let body1 = b1.open - b1.close;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 {
        return false;
    }
    // Bar 2 bullish, inside bar 1's body.
    if b2.close <= b2.open {
        return false;
    }
    let b1_body_high = b1.open;
    let b1_body_low = b1.close;
    let b2_body_high = b2.close;
    let b2_body_low = b2.open;
    if b2_body_high > b1_body_high || b2_body_low < b1_body_low {
        return false;
    }
    // Bar 3 bullish, closes above b1.high.
    if b3.close <= b3.open {
        return false;
    }
    b3.close > b1.high
}

fn is_three_inside_down(b1: Bar, b2: Bar, b3: Bar) -> bool {
    let body1 = b1.close - b1.open;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 {
        return false;
    }
    if b2.close >= b2.open {
        return false;
    }
    let b1_body_high = b1.close;
    let b1_body_low = b1.open;
    let b2_body_high = b2.open;
    let b2_body_low = b2.close;
    if b2_body_high > b1_body_high || b2_body_low < b1_body_low {
        return false;
    }
    if b3.close >= b3.open {
        return false;
    }
    b3.close < b1.low
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
        assert!(r.three_inside_up.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.0),
            bar(f64::NAN, 101.0, 99.0, 100.0),
            bar(100.0, 101.0, 99.0, 100.0),
        ];
        let r = compute(&bars);
        assert!(!r.three_inside_up.iter().any(|x| *x));
    }

    #[test]
    fn bullish_three_inside_up_detected() {
        // Bar 1: tall bearish 110 → 100.
        // Bar 2: bullish harami body (101..105) inside (100..110).
        // Bar 3: bullish, closes above b1.high (110.5).
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(101.0, 105.5, 100.5, 105.0),
            bar(106.0, 112.0, 105.5, 111.0),
        ];
        let r = compute(&bars);
        assert!(r.three_inside_up[2]);
    }

    #[test]
    fn bearish_three_inside_down_detected() {
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(108.0, 108.5, 105.0, 106.0),
            bar(105.0, 105.5, 98.0, 99.0),
        ];
        let r = compute(&bars);
        assert!(r.three_inside_down[2]);
    }

    #[test]
    fn third_bar_no_breakout_rejected() {
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(101.0, 105.5, 100.5, 105.0),
            bar(106.0, 108.0, 105.5, 107.0), // doesn't close > b1.high (110.5)
        ];
        let r = compute(&bars);
        assert!(!r.three_inside_up[2]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.three_inside_up.len(), 10);
        assert_eq!(r.three_inside_down.len(), 10);
    }
}
