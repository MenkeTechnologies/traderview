//! Ladder Bottom / Ladder Top — 5-bar reversal candle pattern.
//!
//! Ladder Bottom (bullish reversal):
//!   Bars 1-3: three consecutive bearish bars, each closing lower
//!     than the prior (step-down "ladder")
//!   Bar 4: bearish bar with a long UPPER WICK (rejection of lower
//!     prices — close ≤ open but high > prior bar's high)
//!   Bar 5: bullish bar that opens above bar 4's close and closes
//!     ABOVE bar 4's high (breakout from the ladder)
//!
//! Ladder Top (bearish reversal): mirrored.
//!
//! Pure compute. Companion to `morning_evening_star`, `breakaway_pattern`,
//! `abandoned_baby`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LadderReport {
    pub bottom: Vec<bool>,
    pub top: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> LadderReport {
    let n = bars.len();
    let mut report = LadderReport {
        bottom: vec![false; n],
        top: vec![false; n],
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
        if is_ladder_bottom(b1, b2, b3, b4, b5) {
            report.bottom[i] = true;
        }
        if is_ladder_top(b1, b2, b3, b4, b5) {
            report.top[i] = true;
        }
    }
    report
}

fn is_ladder_bottom(b1: Bar, b2: Bar, b3: Bar, b4: Bar, b5: Bar) -> bool {
    // Bars 1-3: bearish step-down.
    if b1.close >= b1.open || b2.close >= b2.open || b3.close >= b3.open {
        return false;
    }
    if !(b2.close < b1.close && b3.close < b2.close) {
        return false;
    }
    // Bar 4: bearish with long upper wick.
    if b4.close >= b4.open {
        return false;
    }
    let body4 = b4.open - b4.close;
    let upper4 = b4.high - b4.open;
    if !(upper4 >= body4 && b4.high > b3.high) {
        return false;
    }
    // Bar 5: bullish, opens above bar 4 close, closes above bar 4 high.
    if b5.close <= b5.open {
        return false;
    }
    b5.open > b4.close && b5.close > b4.high
}

fn is_ladder_top(b1: Bar, b2: Bar, b3: Bar, b4: Bar, b5: Bar) -> bool {
    if b1.close <= b1.open || b2.close <= b2.open || b3.close <= b3.open {
        return false;
    }
    if !(b2.close > b1.close && b3.close > b2.close) {
        return false;
    }
    if b4.close <= b4.open {
        return false;
    }
    let body4 = b4.close - b4.open;
    let lower4 = b4.open - b4.low;
    if !(lower4 >= body4 && b4.low < b3.low) {
        return false;
    }
    if b5.close >= b5.open {
        return false;
    }
    b5.open < b4.close && b5.close < b4.low
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
        assert!(r.bottom.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.0); 5];
        bars[2] = bar(f64::NAN, 101.0, 99.0, 100.0);
        let r = compute(&bars);
        assert!(!r.bottom.iter().any(|x| *x));
    }

    #[test]
    fn ladder_bottom_detected() {
        // 3 bearish step-down bars, 4th has long upper wick + new high,
        // 5th opens above bar 4 close and closes above bar 4 high.
        let bars = vec![
            bar(110.0, 110.5, 99.0, 100.0),
            bar(99.0, 99.5, 90.0, 91.0),
            bar(91.0, 91.5, 84.0, 85.0),
            bar(85.0, 95.0, 80.0, 81.0),  // long upper wick, new high
            bar(82.0, 100.0, 81.5, 99.0), // closes above bar 4 high (95)
        ];
        let r = compute(&bars);
        assert!(r.bottom[4]);
    }

    #[test]
    fn no_step_down_rejects() {
        let bars = vec![
            bar(110.0, 110.5, 99.0, 100.0),
            bar(99.0, 99.5, 92.0, 105.0), // bullish, not bearish step-down
            bar(105.0, 106.0, 99.0, 100.0),
            bar(100.0, 110.0, 95.0, 96.0),
            bar(97.0, 115.0, 96.5, 113.0),
        ];
        let r = compute(&bars);
        assert!(!r.bottom[4]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.bottom.len(), 10);
        assert_eq!(r.top.len(), 10);
    }
}
