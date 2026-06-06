//! Kicker Pattern — strong 2-bar reversal with opposite gap and zero
//! overlap. Considered one of the most reliable single-pattern reversal
//! signals because both bars are full-body marubozu-like and the gap
//! cuts across the prior body entirely.
//!
//! Bullish Kicker:
//!   Bar 1: bearish bar (close < open), body ≥ 70% of range
//!   Bar 2: opens ABOVE bar 1.open (full gap up past prior body),
//!     bullish (close > open), body ≥ 70% of range
//!   No overlap: bar 2.low ≥ bar 1.open
//!
//! Bearish Kicker: mirrored — bar 1 bullish, bar 2 opens below bar 1.open
//! with no overlap and bearish body.
//!
//! Pure compute. Companion to `engulfing_pattern_scanner`,
//! `morning_evening_star`, `abandoned_baby`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KickerReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> KickerReport {
    let n = bars.len();
    let mut report = KickerReport {
        bullish: vec![false; n],
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
        if is_bullish_kicker(prev, cur) {
            report.bullish[i] = true;
        }
        if is_bearish_kicker(prev, cur) {
            report.bearish[i] = true;
        }
    }
    report
}

fn is_full_body(b: Bar) -> bool {
    let range = b.high - b.low;
    let body = (b.close - b.open).abs();
    range > 0.0 && body >= 0.7 * range
}

fn is_bullish_kicker(prev: Bar, cur: Bar) -> bool {
    // Bar 1 bearish.
    if prev.close >= prev.open {
        return false;
    }
    if !is_full_body(prev) {
        return false;
    }
    // Bar 2 bullish.
    if cur.close <= cur.open {
        return false;
    }
    if !is_full_body(cur) {
        return false;
    }
    // Gap above prior body (open > prev.open, full skip over body).
    cur.open >= prev.open && cur.low >= prev.open
}

fn is_bearish_kicker(prev: Bar, cur: Bar) -> bool {
    if prev.close <= prev.open {
        return false;
    }
    if !is_full_body(prev) {
        return false;
    }
    if cur.close >= cur.open {
        return false;
    }
    if !is_full_body(cur) {
        return false;
    }
    cur.open <= prev.open && cur.high <= prev.open
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
        assert!(r.bullish.is_empty());
        let r2 = compute(&[bar(100.0, 101.0, 99.0, 100.5)]);
        assert!(!r2.bullish.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.0),
            bar(f64::NAN, 101.0, 99.0, 100.5),
        ];
        let r = compute(&bars);
        assert!(!r.bullish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_kicker_detected() {
        // Bar 1: bearish 110 → 100 (full body).
        // Bar 2: opens 115 (above bar 1.open=110), closes 125, low 115.
        let bars = vec![
            bar(110.0, 110.0, 100.0, 100.0),
            bar(115.0, 125.0, 115.0, 125.0),
        ];
        let r = compute(&bars);
        assert!(r.bullish[1]);
    }

    #[test]
    fn bearish_kicker_detected() {
        let bars = vec![bar(100.0, 110.0, 100.0, 110.0), bar(95.0, 95.0, 85.0, 85.0)];
        let r = compute(&bars);
        assert!(r.bearish[1]);
    }

    #[test]
    fn overlap_rejects_kicker() {
        // Bar 2 opens above bar 1.open but bar 2.low < bar 1.open → overlap.
        let bars = vec![
            bar(110.0, 110.0, 100.0, 100.0),
            bar(115.0, 125.0, 108.0, 125.0), // low 108 < 110
        ];
        let r = compute(&bars);
        assert!(!r.bullish[1]);
    }

    #[test]
    fn small_body_rejects_kicker() {
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(115.0, 125.0, 115.0, 117.0), // body 2 of range 10 = 20%
        ];
        let r = compute(&bars);
        assert!(!r.bullish[1]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.bullish.len(), 10);
        assert_eq!(r.bearish.len(), 10);
    }
}
