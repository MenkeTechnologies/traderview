//! Dark Cloud Cover / Piercing Pattern — 2-bar candle reversal patterns.
//!
//! Dark Cloud Cover (bearish reversal):
//!   Bar 1: tall bullish body
//!   Bar 2: opens ABOVE bar 1 high (gap up), closes BELOW the
//!          midpoint of bar 1 body (penetration past 50% retrace)
//!
//! Piercing Pattern (bullish reversal): mirrored —
//!   Bar 1: tall bearish body
//!   Bar 2: opens BELOW bar 1 low (gap down), closes ABOVE the
//!          midpoint of bar 1 body
//!
//! Both signal that the breakout in the gap direction failed and the
//! prior trend is exhausting.
//!
//! Pure compute. Companion to `candle_patterns`,
//! `morning_evening_star`, `three_white_soldiers_crows`,
//! `engulfing_pattern_scanner`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DarkCloudPiercingReport {
    pub dark_cloud_cover: Vec<bool>,
    pub piercing: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> DarkCloudPiercingReport {
    let n = bars.len();
    let mut report = DarkCloudPiercingReport {
        dark_cloud_cover: vec![false; n],
        piercing: vec![false; n],
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
        let (b1, b2) = (bars[i - 1], bars[i]);
        if is_dark_cloud_cover(b1, b2) {
            report.dark_cloud_cover[i] = true;
        }
        if is_piercing(b1, b2) {
            report.piercing[i] = true;
        }
    }
    report
}

fn is_dark_cloud_cover(b1: Bar, b2: Bar) -> bool {
    // Bar 1: tall bullish (close > open, body ≥ 60% of range).
    let body1 = b1.close - b1.open;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 {
        return false;
    }
    // Bar 2: opens > b1.high (gap up), bearish close.
    if b2.open <= b1.high {
        return false;
    }
    if b2.close >= b2.open {
        return false;
    }
    // Penetrates past midpoint of bar 1 body.
    let mid1 = (b1.open + b1.close) / 2.0;
    b2.close < mid1
}

fn is_piercing(b1: Bar, b2: Bar) -> bool {
    let body1 = b1.open - b1.close;
    let range1 = b1.high - b1.low;
    if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 {
        return false;
    }
    if b2.open >= b1.low {
        return false;
    }
    if b2.close <= b2.open {
        return false;
    }
    let mid1 = (b1.open + b1.close) / 2.0;
    b2.close > mid1
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
        assert!(r.dark_cloud_cover.is_empty());
        let r2 = compute(&[bar(100.0, 101.0, 99.0, 100.0)]);
        assert!(!r2.dark_cloud_cover.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.5),
            bar(f64::NAN, 101.0, 99.0, 100.5),
        ];
        let r = compute(&bars);
        assert!(!r.dark_cloud_cover.iter().any(|x| *x));
    }

    #[test]
    fn classic_dark_cloud_cover_detected() {
        // Bar 1: bullish 100 → 110. Bar 2: opens 112 (gap up),
        // closes 103 (below mid 105).
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(112.0, 113.0, 102.5, 103.0),
        ];
        let r = compute(&bars);
        assert!(r.dark_cloud_cover[1]);
    }

    #[test]
    fn classic_piercing_pattern_detected() {
        // Bar 1: bearish 110 → 100. Bar 2: opens 98 (gap down),
        // closes 107 (above mid 105).
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(98.0, 108.0, 97.5, 107.0),
        ];
        let r = compute(&bars);
        assert!(r.piercing[1]);
    }

    #[test]
    fn no_gap_rejects_dark_cloud() {
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(109.0, 113.0, 102.5, 103.0), // opens below b1 high
        ];
        let r = compute(&bars);
        assert!(!r.dark_cloud_cover[1]);
    }

    #[test]
    fn insufficient_penetration_rejected() {
        // Bar 2 closes ABOVE midpoint → engulfing-style move not deep enough.
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(112.0, 113.0, 107.5, 108.0), // close 108 > mid 105
        ];
        let r = compute(&bars);
        assert!(!r.dark_cloud_cover[1]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.dark_cloud_cover.len(), 10);
        assert_eq!(r.piercing.len(), 10);
    }
}
