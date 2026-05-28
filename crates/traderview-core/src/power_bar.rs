//! Power Bar — single-bar strength classifier requiring a large body
//! AND a close near the bar's extreme.
//!
//!   body_pct = |close - open| / range
//!   close_pct = (close - low) / range   (0 = at low, 1 = at high)
//!
//! Bullish Power Bar:
//!   - bullish (close > open)
//!   - body_pct ≥ body_threshold (default 0.7)
//!   - close_pct ≥ close_at_extreme_threshold (default 0.85)
//!
//! Bearish Power Bar: mirrored — close < open, body ≥ threshold, close
//! in lower 15% of range.
//!
//! Pure compute. Companion to `spinning_top_marubozu`, `belt_hold_pattern`,
//! `engulfing_pattern_scanner`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PowerBarReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
    pub body_threshold: f64,
    pub close_at_extreme_threshold: f64,
}

pub fn compute(
    bars: &[Bar],
    body_threshold: f64,
    close_at_extreme_threshold: f64,
) -> PowerBarReport {
    let n = bars.len();
    let mut report = PowerBarReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
        body_threshold,
        close_at_extreme_threshold,
    };
    if !body_threshold.is_finite() || !(0.0..=1.0).contains(&body_threshold)
        || !close_at_extreme_threshold.is_finite()
        || !(0.0..=1.0).contains(&close_at_extreme_threshold) {
        return report;
    }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    for (i, bar) in bars.iter().enumerate() {
        let range = bar.high - bar.low;
        if range <= 0.0 { continue; }
        let body = (bar.close - bar.open).abs();
        let body_pct = body / range;
        let close_pct = (bar.close - bar.low) / range;
        if body_pct < body_threshold { continue; }
        if bar.close > bar.open && close_pct >= close_at_extreme_threshold {
            report.bullish[i] = true;
        }
        if bar.close < bar.open && close_pct <= 1.0 - close_at_extreme_threshold {
            report.bearish[i] = true;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar { open: o, high: h, low: l, close: c }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, -1.0, 0.85);
        assert!(!r.bullish.iter().any(|x| *x));
        let r2 = compute(&bars, 0.7, 1.5);
        assert!(!r2.bullish.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![bar(f64::NAN, 101.0, 99.0, 100.5); 5];
        let r = compute(&bars, 0.7, 0.85);
        assert!(!r.bullish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_power_bar_detected() {
        // Range 10, body 9 (open=100, close=109), close at 109 → close_pct=0.9.
        let bars = vec![bar(100.0, 110.0, 100.0, 109.0)];
        let r = compute(&bars, 0.7, 0.85);
        assert!(r.bullish[0]);
        assert!(!r.bearish[0]);
    }

    #[test]
    fn bearish_power_bar_detected() {
        let bars = vec![bar(110.0, 110.0, 100.0, 101.0)];
        // body=9, range=10, close=101 → close_pct=0.1 ≤ 0.15.
        let r = compute(&bars, 0.7, 0.85);
        assert!(r.bearish[0]);
    }

    #[test]
    fn small_body_rejected() {
        let bars = vec![bar(100.0, 110.0, 100.0, 102.0)];
        // body=2, range=10, ratio=0.2 < 0.7.
        let r = compute(&bars, 0.7, 0.85);
        assert!(!r.bullish[0]);
    }

    #[test]
    fn close_mid_range_rejected() {
        // Large body but close in middle, not extreme.
        let bars = vec![bar(100.0, 110.0, 100.0, 105.0)];
        let r = compute(&bars, 0.4, 0.85);
        // body=5, range=10, ratio=0.5 → still meets 0.4 threshold.
        // close_pct = 0.5 — fails 0.85.
        assert!(!r.bullish[0]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 0.7, 0.85);
        assert_eq!(r.bullish.len(), 10);
        assert_eq!(r.bearish.len(), 10);
    }
}
