//! Thrusting Pattern — 2-bar bearish continuation.
//!
//!   Bar 1: tall bearish body
//!   Bar 2: bullish bar that opens BELOW bar 1 low (gap down) and
//!     closes inside bar 1's body but BELOW its midpoint
//!
//! Distinct from `dark_cloud_piercing` (which needs close ABOVE
//! midpoint) and `on_neck_in_neck` (closes at or near bar 1 low).
//! Thrusting closes between bar 1 low and bar 1 midpoint — a weaker
//! counter-rally that fails to threaten the prior bearish move.
//!
//! Pure compute. Companion to `on_neck_in_neck`, `dark_cloud_piercing`,
//! `counter_attack_lines`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThrustingReport {
    pub bearish: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> ThrustingReport {
    let n = bars.len();
    let mut report = ThrustingReport {
        bearish: vec![false; n],
    };
    if n < 2 {
        return report;
    }
    if bars.iter().any(|b| {
        !b.open.is_finite() || !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()
    }) {
        return report;
    }
    for i in 1..n {
        let prev = bars[i - 1];
        let cur = bars[i];
        let body1 = prev.open - prev.close;
        let range1 = prev.high - prev.low;
        if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 {
            continue;
        }
        // Bar 2 bullish, gap down, close between bar 1 low and midpoint.
        if cur.close <= cur.open {
            continue;
        }
        if cur.open >= prev.low {
            continue;
        }
        let mid1 = (prev.open + prev.close) / 2.0;
        if cur.close > prev.low && cur.close < mid1 {
            report.bearish[i] = true;
        }
    }
    report
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
    fn empty_or_single_bar_returns_empty() {
        let r = compute(&[]);
        assert!(r.bearish.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.0),
            bar(f64::NAN, 101.0, 99.0, 100.0),
        ];
        let r = compute(&bars);
        assert!(!r.bearish.iter().any(|x| *x));
    }

    #[test]
    fn thrusting_detected() {
        // Bar 1: bearish 110→100, low=99.5, midpoint=105.
        // Bar 2: bullish, opens 95 (< 99.5), closes 102 (between 99.5 and 105).
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(95.0, 103.0, 94.5, 102.0),
        ];
        let r = compute(&bars);
        assert!(r.bearish[1]);
    }

    #[test]
    fn close_above_midpoint_rejects() {
        // Close at 107 > midpoint 105 → piercing, not thrusting.
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(95.0, 108.0, 94.5, 107.0),
        ];
        let r = compute(&bars);
        assert!(!r.bearish[1]);
    }

    #[test]
    fn no_gap_rejects() {
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(100.5, 103.0, 100.0, 102.0), // open above bar 1 low
        ];
        let r = compute(&bars);
        assert!(!r.bearish[1]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.bearish.len(), 10);
    }
}
