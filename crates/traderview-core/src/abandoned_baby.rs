//! Abandoned Baby — 3-bar reversal candle pattern.
//!
//! Bullish Abandoned Baby (reversal at lows):
//!   Bar 1: tall bearish body
//!   Bar 2: doji that gaps DOWN — bar 2.high < bar 1.low
//!     (no overlap with bar 1's range)
//!   Bar 3: tall bullish body that gaps UP — bar 3.low > bar 2.high
//!     (no overlap with bar 2's range)
//!
//! Bearish Abandoned Baby: mirrored.
//!
//! The triple-isolation (doji "abandoned" by gaps on both sides) makes
//! this pattern much rarer than morning/evening star and a stronger
//! reversal signal when it appears.
//!
//! Pure compute. Companion to `morning_evening_star`,
//! `dark_cloud_piercing`, `candle_patterns`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AbandonedBabyReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
}

pub fn compute(bars: &[Bar], doji_pct: f64) -> AbandonedBabyReport {
    let n = bars.len();
    let mut report = AbandonedBabyReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
    };
    if n < 3 || !doji_pct.is_finite() || doji_pct <= 0.0 {
        return report;
    }
    if bars.iter().any(|b| {
        !b.open.is_finite() || !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()
    }) {
        return report;
    }
    for i in 2..n {
        let (b1, b2, b3) = (bars[i - 2], bars[i - 1], bars[i]);
        if is_bullish_abandoned_baby(b1, b2, b3, doji_pct) {
            report.bullish[i] = true;
        }
        if is_bearish_abandoned_baby(b1, b2, b3, doji_pct) {
            report.bearish[i] = true;
        }
    }
    report
}

fn is_doji(b: Bar, doji_pct: f64) -> bool {
    let range = b.high - b.low;
    if range <= 0.0 {
        return false;
    }
    let body = (b.close - b.open).abs();
    body <= doji_pct * range
}

fn is_bullish_abandoned_baby(b1: Bar, b2: Bar, b3: Bar, doji_pct: f64) -> bool {
    // Bar 1: tall bearish.
    let body1 = b1.open - b1.close;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 {
        return false;
    }
    // Bar 2: doji, gap down (b2.high < b1.low).
    if !is_doji(b2, doji_pct) {
        return false;
    }
    if b2.high >= b1.low {
        return false;
    }
    // Bar 3: tall bullish, gap up (b3.low > b2.high).
    let body3 = b3.close - b3.open;
    let range3 = b3.high - b3.low;
    if range3 <= 0.0 || body3 <= 0.0 || body3 < 0.6 * range3 {
        return false;
    }
    if b3.low <= b2.high {
        return false;
    }
    true
}

fn is_bearish_abandoned_baby(b1: Bar, b2: Bar, b3: Bar, doji_pct: f64) -> bool {
    let body1 = b1.close - b1.open;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 {
        return false;
    }
    if !is_doji(b2, doji_pct) {
        return false;
    }
    if b2.low <= b1.high {
        return false;
    }
    let body3 = b3.open - b3.close;
    let range3 = b3.high - b3.low;
    if range3 <= 0.0 || body3 <= 0.0 || body3 < 0.6 * range3 {
        return false;
    }
    if b3.high >= b2.low {
        return false;
    }
    true
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
    fn invalid_inputs_return_empty() {
        let r = compute(&[], 0.1);
        assert!(r.bullish.is_empty());
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 5];
        let r2 = compute(&bars, 0.0);
        assert!(!r2.bullish.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.0),
            bar(f64::NAN, 101.0, 99.0, 100.0),
            bar(100.0, 101.0, 99.0, 100.0),
        ];
        let r = compute(&bars, 0.1);
        assert!(!r.bullish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_abandoned_baby_detected() {
        // Bar 1: tall bearish 110→100, high=110.5, low=99.5.
        // Bar 2: doji at 95-96, fully below b1 low (gap down).
        // Bar 3: tall bullish 99→105, low=98.5 > b2 high (gap up).
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(96.0, 96.5, 94.5, 95.9),
            bar(99.0, 105.5, 98.5, 105.0),
        ];
        let r = compute(&bars, 0.2);
        assert!(r.bullish[2]);
    }

    #[test]
    fn bearish_abandoned_baby_detected() {
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(114.0, 115.5, 113.5, 114.1),
            bar(111.0, 111.5, 105.0, 105.5),
        ];
        let r = compute(&bars, 0.2);
        assert!(r.bearish[2]);
    }

    #[test]
    fn overlap_with_doji_rejects_pattern() {
        // No gap between bar 1 and bar 2.
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(101.0, 101.5, 99.5, 100.9), // overlaps bar 1
            bar(102.0, 105.5, 101.5, 105.0),
        ];
        let r = compute(&bars, 0.2);
        assert!(!r.bullish[2]);
    }

    #[test]
    fn non_doji_middle_bar_rejects() {
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(96.0, 99.5, 90.0, 99.0), // big body, not doji
            bar(101.0, 105.5, 100.5, 105.0),
        ];
        let r = compute(&bars, 0.1);
        assert!(!r.bullish[2]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 0.1);
        assert_eq!(r.bullish.len(), 10);
        assert_eq!(r.bearish.len(), 10);
    }
}
