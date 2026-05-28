//! Tweezer Top / Bottom — 2-bar reversal pattern at matching extremes.
//!
//! Two consecutive bars share approximately the same HIGH (tweezer
//! top) or LOW (tweezer bottom), within `tolerance_pct` of each other,
//! and have OPPOSITE body directions. The matched extreme acts as a
//! short-term resistance or support that the second bar rejected.
//!
//! Bullish tweezer bottom: matching lows; bar 1 bearish, bar 2 bullish.
//! Bearish tweezer top: matching highs; bar 1 bullish, bar 2 bearish.
//!
//! Pure compute. Default tolerance_pct = 0.05 (5 bps of the extreme).
//! Companion to `candle_patterns`, `harami_pattern`,
//! `engulfing_pattern_scanner`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TweezerReport {
    pub top: Vec<bool>,
    pub bottom: Vec<bool>,
    pub tolerance_pct: f64,
}

pub fn compute(bars: &[Bar], tolerance_pct: f64) -> TweezerReport {
    let n = bars.len();
    let mut report = TweezerReport {
        top: vec![false; n],
        bottom: vec![false; n],
        tolerance_pct,
    };
    if n < 2 || !tolerance_pct.is_finite() || tolerance_pct <= 0.0 { return report; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    let tol_factor = tolerance_pct / 100.0;
    for i in 1..n {
        let prev = bars[i - 1];
        let cur = bars[i];
        let prev_bullish = prev.close > prev.open;
        let cur_bullish = cur.close > cur.open;
        // Tweezer top: matching highs, prev bullish, cur bearish.
        let high_tol = prev.high * tol_factor;
        if (cur.high - prev.high).abs() <= high_tol
            && prev_bullish && !cur_bullish {
            report.top[i] = true;
        }
        let low_tol = prev.low * tol_factor;
        if (cur.low - prev.low).abs() <= low_tol
            && !prev_bullish && cur_bullish {
            report.bottom[i] = true;
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
    fn empty_or_invalid_returns_empty() {
        let r = compute(&[], 0.05);
        assert!(r.top.is_empty());
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0); 5];
        let r2 = compute(&bars, 0.0);
        assert!(!r2.top.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0),
                        bar(f64::NAN, 101.0, 99.0, 100.0)];
        let r = compute(&bars, 0.05);
        assert!(!r.top.iter().any(|x| *x));
    }

    #[test]
    fn tweezer_top_detected() {
        let bars = vec![
            bar(100.0, 110.0, 99.5, 109.0),    // bullish, high=110
            bar(108.0, 110.05, 100.0, 101.0),  // bearish, high=110.05 (≈ 110)
        ];
        let r = compute(&bars, 0.05);
        assert!(r.top[1]);
        assert!(!r.bottom[1]);
    }

    #[test]
    fn tweezer_bottom_detected() {
        let bars = vec![
            bar(110.0, 110.5, 100.0, 101.0),
            bar(101.0, 109.0, 99.95, 108.0),
        ];
        let r = compute(&bars, 0.05);
        assert!(r.bottom[1]);
        assert!(!r.top[1]);
    }

    #[test]
    fn mismatched_extremes_no_signal() {
        let bars = vec![
            bar(100.0, 110.0, 99.5, 109.0),
            bar(108.0, 115.0, 100.0, 101.0),    // high 115 ≠ 110
        ];
        let r = compute(&bars, 0.05);
        assert!(!r.top[1]);
    }

    #[test]
    fn same_color_bars_no_signal() {
        let bars = vec![
            bar(100.0, 110.0, 99.5, 109.0),
            bar(101.0, 110.05, 100.0, 109.5),    // both bullish
        ];
        let r = compute(&bars, 0.05);
        assert!(!r.top[1]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 0.05);
        assert_eq!(r.top.len(), 10);
        assert_eq!(r.bottom.len(), 10);
    }
}
