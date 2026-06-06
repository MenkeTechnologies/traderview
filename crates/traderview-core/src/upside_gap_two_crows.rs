//! Upside Gap Two Crows — 3-bar bearish reversal at uptrend highs.
//!
//!   Bar 1: tall bullish body
//!   Bar 2: bearish body that GAPS UP from bar 1 (open > bar 1.close)
//!     — first "crow"
//!   Bar 3: bearish body that opens ABOVE bar 2's open and closes
//!     BELOW bar 2's close (engulfing the first crow) but still
//!     CLOSING ABOVE bar 1's close (gap not yet fully closed) —
//!     second "crow"
//!
//! Distinct from bearish abandoned baby (middle bar is doji not body)
//! and bearish engulfing (which only needs 2 bars).
//!
//! Pure compute. Companion to `morning_evening_star`, `abandoned_baby`,
//! `three_white_soldiers_crows`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn compute(bars: &[Bar]) -> Vec<bool> {
    let n = bars.len();
    let mut out = vec![false; n];
    if n < 3 {
        return out;
    }
    if bars.iter().any(|b| {
        !b.open.is_finite() || !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()
    }) {
        return out;
    }
    for i in 2..n {
        let (b1, b2, b3) = (bars[i - 2], bars[i - 1], bars[i]);
        // Bar 1: tall bullish (body ≥ 60% of range).
        let body1 = b1.close - b1.open;
        let range1 = b1.high - b1.low;
        if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 {
            continue;
        }
        // Bar 2: bearish, opens above bar 1's close (gap up).
        if b2.close >= b2.open {
            continue;
        }
        if b2.open <= b1.close {
            continue;
        }
        // Bar 3: bearish, opens above bar 2's open, closes below bar 2's
        // close, but still above bar 1's close (gap not yet filled).
        if b3.close >= b3.open {
            continue;
        }
        if b3.open <= b2.open {
            continue;
        }
        if b3.close >= b2.close {
            continue;
        }
        if b3.close <= b1.close {
            continue;
        }
        out[i] = true;
    }
    out
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
        assert!(r.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.0),
            bar(f64::NAN, 101.0, 99.0, 100.0),
            bar(100.0, 101.0, 99.0, 100.0),
        ];
        let r = compute(&bars);
        assert!(!r.iter().any(|x| *x));
    }

    #[test]
    fn upside_gap_two_crows_detected() {
        // Bar 1: bullish 100→110.
        // Bar 2: bearish opens 113 (> bar 1 close 110), closes 112.
        // Bar 3: bearish opens 114 (> bar 2 open 113), closes 111
        //   (< bar 2 close 112 AND > bar 1 close 110).
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(113.0, 113.5, 111.5, 112.0),
            bar(114.0, 114.5, 110.5, 111.0),
        ];
        let r = compute(&bars);
        assert!(r[2]);
    }

    #[test]
    fn no_gap_at_bar2_rejects() {
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(109.0, 110.0, 107.0, 108.0), // opens below bar 1 close
            bar(108.0, 109.0, 105.0, 106.0),
        ];
        let r = compute(&bars);
        assert!(!r[2]);
    }

    #[test]
    fn gap_fully_closed_at_bar3_rejects() {
        // b3 closes ≤ b1.close → gap fully closed; not the pattern.
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(113.0, 113.5, 111.5, 112.0),
            bar(114.0, 114.5, 108.0, 109.0), // close 109 ≤ b1.close 110
        ];
        let r = compute(&bars);
        assert!(!r[2]);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        assert_eq!(compute(&bars).len(), 10);
    }
}
