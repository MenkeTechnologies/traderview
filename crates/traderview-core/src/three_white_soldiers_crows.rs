//! Three White Soldiers / Three Black Crows — 3-bar continuation candle
//! patterns confirming a strong directional move.
//!
//! Three White Soldiers (bullish continuation):
//!   - 3 consecutive bullish bars (close > open)
//!   - Each close > prior close
//!   - Each open within the prior body (open > prior open AND open <
//!     prior close)
//!   - Bodies relatively large (body ≥ 50% of range each bar)
//!   - Short upper wicks (≤ 25% of body each bar)
//!
//! Three Black Crows (bearish continuation): mirrored conditions.
//!
//! Pure compute. Companion to `candle_patterns`,
//! `engulfing_pattern_scanner`, `morning_evening_star`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreeWhiteSoldiersReport {
    pub white_soldiers: Vec<bool>,
    pub black_crows: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> ThreeWhiteSoldiersReport {
    let n = bars.len();
    let mut report = ThreeWhiteSoldiersReport {
        white_soldiers: vec![false; n],
        black_crows: vec![false; n],
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
        if is_three_white_soldiers(b1, b2, b3) {
            report.white_soldiers[i] = true;
        }
        if is_three_black_crows(b1, b2, b3) {
            report.black_crows[i] = true;
        }
    }
    report
}

fn is_three_white_soldiers(b1: Bar, b2: Bar, b3: Bar) -> bool {
    let bullish = |b: Bar| b.close > b.open;
    if !(bullish(b1) && bullish(b2) && bullish(b3)) {
        return false;
    }
    if !(b2.close > b1.close && b3.close > b2.close) {
        return false;
    }
    // Each open inside prior body (not deep gap up).
    if !(b2.open > b1.open && b2.open < b1.close) {
        return false;
    }
    if !(b3.open > b2.open && b3.open < b2.close) {
        return false;
    }
    body_test(b1)
        && body_test(b2)
        && body_test(b3)
        && upper_wick_test_bull(b1)
        && upper_wick_test_bull(b2)
        && upper_wick_test_bull(b3)
}

fn is_three_black_crows(b1: Bar, b2: Bar, b3: Bar) -> bool {
    let bearish = |b: Bar| b.close < b.open;
    if !(bearish(b1) && bearish(b2) && bearish(b3)) {
        return false;
    }
    if !(b2.close < b1.close && b3.close < b2.close) {
        return false;
    }
    if !(b2.open < b1.open && b2.open > b1.close) {
        return false;
    }
    if !(b3.open < b2.open && b3.open > b2.close) {
        return false;
    }
    body_test(b1)
        && body_test(b2)
        && body_test(b3)
        && lower_wick_test_bear(b1)
        && lower_wick_test_bear(b2)
        && lower_wick_test_bear(b3)
}

fn body_test(b: Bar) -> bool {
    let body = (b.close - b.open).abs();
    let range = b.high - b.low;
    range > 0.0 && body >= 0.5 * range
}

fn upper_wick_test_bull(b: Bar) -> bool {
    let body = (b.close - b.open).abs();
    let upper = b.high - b.close.max(b.open);
    body > 0.0 && upper <= 0.25 * body
}

fn lower_wick_test_bear(b: Bar) -> bool {
    let body = (b.close - b.open).abs();
    let lower = b.close.min(b.open) - b.low;
    body > 0.0 && lower <= 0.25 * body
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
        assert!(r.white_soldiers.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.5),
            bar(f64::NAN, 101.0, 99.0, 100.5),
            bar(100.0, 101.0, 99.0, 100.5),
        ];
        let r = compute(&bars);
        assert!(!r.white_soldiers.iter().any(|x| *x));
        assert!(!r.black_crows.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_no_signal() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0); 5];
        let r = compute(&bars);
        assert!(!r.white_soldiers.iter().any(|x| *x));
        assert!(!r.black_crows.iter().any(|x| *x));
    }

    #[test]
    fn classic_three_white_soldiers_detected() {
        let bars = vec![
            bar(100.0, 105.0, 100.0, 104.5),
            bar(103.0, 110.0, 102.5, 109.5),
            bar(108.0, 115.0, 107.5, 114.5),
        ];
        let r = compute(&bars);
        assert!(r.white_soldiers[2]);
    }

    #[test]
    fn classic_three_black_crows_detected() {
        let bars = vec![
            bar(115.0, 115.5, 110.0, 110.5),
            bar(112.0, 112.5, 105.0, 105.5),
            bar(107.0, 107.5, 100.0, 100.5),
        ];
        let r = compute(&bars);
        assert!(r.black_crows[2]);
    }

    #[test]
    fn long_upper_wicks_reject_soldiers() {
        // Long upper wicks → reject (uncertain bullish commitment).
        let bars = vec![
            bar(100.0, 110.0, 100.0, 104.5),
            bar(103.0, 115.0, 102.5, 109.5),
            bar(108.0, 120.0, 107.5, 114.5),
        ];
        let r = compute(&bars);
        assert!(!r.white_soldiers[2]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.white_soldiers.len(), 10);
        assert_eq!(r.black_crows.len(), 10);
    }
}
