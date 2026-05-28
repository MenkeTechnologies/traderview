//! Morning Star / Evening Star — 3-bar reversal candle patterns.
//!
//! Morning Star (bullish reversal at lows):
//!   Bar 1: tall bearish body
//!   Bar 2: small body that gaps DOWN below bar 1 close
//!         (any color, often called a "star")
//!   Bar 3: tall bullish body closing inside (past midpoint of)
//!         bar 1 body, signaling reversal
//!
//! Evening Star (bearish reversal at highs): mirrored.
//!
//! Pure compute. Companion to `candle_patterns`,
//! `three_white_soldiers_crows`, `dark_cloud_piercing`,
//! `engulfing_pattern_scanner`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StarPatternReport {
    pub morning_star: Vec<bool>,
    pub evening_star: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> StarPatternReport {
    let n = bars.len();
    let mut report = StarPatternReport {
        morning_star: vec![false; n],
        evening_star: vec![false; n],
    };
    if n < 3 { return report; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    for i in 2..n {
        let (b1, b2, b3) = (bars[i - 2], bars[i - 1], bars[i]);
        if is_morning_star(b1, b2, b3) {
            report.morning_star[i] = true;
        }
        if is_evening_star(b1, b2, b3) {
            report.evening_star[i] = true;
        }
    }
    report
}

fn is_morning_star(b1: Bar, b2: Bar, b3: Bar) -> bool {
    let body1 = b1.open - b1.close;            // > 0 if bearish
    let range1 = b1.high - b1.low;
    let body2 = (b2.close - b2.open).abs();
    let range2 = b2.high - b2.low;
    let body3 = b3.close - b3.open;             // > 0 if bullish
    let range3 = b3.high - b3.low;
    if range1 <= 0.0 || range2 <= 0.0 || range3 <= 0.0 { return false; }
    // Bar 1: tall bearish body (≥ 60% of range, close < open).
    if body1 <= 0.0 || body1 < 0.6 * range1 { return false; }
    // Bar 2: small body, gaps down vs bar 1 close.
    if body2 >= 0.4 * range2 { return false; }
    if b2.high >= b1.close { return false; }
    // Bar 3: tall bullish body, closes above midpoint of bar 1.
    if body3 <= 0.0 || body3 < 0.6 * range3 { return false; }
    let mid1 = (b1.open + b1.close) / 2.0;
    if b3.close <= mid1 { return false; }
    true
}

fn is_evening_star(b1: Bar, b2: Bar, b3: Bar) -> bool {
    let body1 = b1.close - b1.open;            // > 0 if bullish
    let range1 = b1.high - b1.low;
    let body2 = (b2.close - b2.open).abs();
    let range2 = b2.high - b2.low;
    let body3 = b3.open - b3.close;             // > 0 if bearish
    let range3 = b3.high - b3.low;
    if range1 <= 0.0 || range2 <= 0.0 || range3 <= 0.0 { return false; }
    if body1 <= 0.0 || body1 < 0.6 * range1 { return false; }
    if body2 >= 0.4 * range2 { return false; }
    if b2.low <= b1.close { return false; }
    if body3 <= 0.0 || body3 < 0.6 * range3 { return false; }
    let mid1 = (b1.open + b1.close) / 2.0;
    if b3.close >= mid1 { return false; }
    true
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
        assert!(r.morning_star.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5),
                        bar(f64::NAN, 101.0, 99.0, 100.5),
                        bar(100.0, 101.0, 99.0, 100.5)];
        let r = compute(&bars);
        assert!(!r.morning_star.iter().any(|x| *x));
        assert!(!r.evening_star.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_no_signal() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0); 5];
        let r = compute(&bars);
        assert!(!r.morning_star.iter().any(|x| *x));
        assert!(!r.evening_star.iter().any(|x| *x));
    }

    #[test]
    fn classic_morning_star_detected() {
        // Bar 1: tall bearish (110→90, close=90).
        // Bar 2: small star, gaps down (87→89, range 87..89).
        // Bar 3: tall bullish closing above midpoint 100.
        let bars = vec![
            bar(110.0, 110.5, 89.5, 90.0),
            bar(88.0, 89.0, 86.0, 88.5),
            bar(89.0, 105.0, 88.5, 104.0),
        ];
        let r = compute(&bars);
        assert!(r.morning_star[2]);
    }

    #[test]
    fn classic_evening_star_detected() {
        let bars = vec![
            bar(90.0, 110.5, 89.5, 110.0),
            bar(112.0, 114.0, 111.0, 111.5),
            bar(111.0, 111.5, 95.0, 96.0),
        ];
        let r = compute(&bars);
        assert!(r.evening_star[2]);
    }

    #[test]
    fn no_gap_rejects_morning_star() {
        // Bar 2 overlaps bar 1 close → no gap → reject.
        let bars = vec![
            bar(110.0, 110.5, 89.5, 90.0),
            bar(92.0, 94.0, 91.0, 92.5),
            bar(93.0, 105.0, 92.5, 104.0),
        ];
        let r = compute(&bars);
        assert!(!r.morning_star[2]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.morning_star.len(), 10);
        assert_eq!(r.evening_star.len(), 10);
    }
}
