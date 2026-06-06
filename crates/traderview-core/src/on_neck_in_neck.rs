//! On-Neck / In-Neck Lines — 2-bar bearish continuation patterns.
//!
//! Both patterns show a small bullish bar attempting to recover after
//! a tall bearish bar, but failing to close past key resistance levels.
//!
//! On-Neck Line (bearish continuation):
//!   Bar 1: tall bearish body
//!   Bar 2: bullish, opens BELOW bar 1's low, closes near bar 1's LOW
//!     (within `tolerance_pct`)
//!
//! In-Neck Line (bearish continuation):
//!   Bar 1: tall bearish body
//!   Bar 2: bullish, opens BELOW bar 1's low, closes slightly above
//!     bar 1's close but BELOW bar 1's midpoint
//!
//! Pure compute. Default tolerance_pct = 0.5 (0.5% of bar 1 low).
//! Companion to `dark_cloud_piercing`, `harami_pattern`,
//! `three_white_soldiers_crows`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NeckPatternReport {
    pub on_neck: Vec<bool>,
    pub in_neck: Vec<bool>,
    pub tolerance_pct: f64,
}

pub fn compute(bars: &[Bar], tolerance_pct: f64) -> NeckPatternReport {
    let n = bars.len();
    let mut report = NeckPatternReport {
        on_neck: vec![false; n],
        in_neck: vec![false; n],
        tolerance_pct,
    };
    if n < 2 || !tolerance_pct.is_finite() || tolerance_pct <= 0.0 {
        return report;
    }
    if bars.iter().any(|b| {
        !b.open.is_finite() || !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()
    }) {
        return report;
    }
    let tol_factor = tolerance_pct / 100.0;
    for i in 1..n {
        let prev = bars[i - 1];
        let cur = bars[i];
        // Bar 1 must be tall bearish.
        let body1 = prev.open - prev.close;
        let range1 = prev.high - prev.low;
        if range1 <= 0.0 || body1 <= 0.0 || body1 < 0.6 * range1 {
            continue;
        }
        // Bar 2 must be bullish opening below bar 1 low.
        if cur.close <= cur.open {
            continue;
        }
        if cur.open >= prev.low {
            continue;
        }
        let mid1 = (prev.open + prev.close) / 2.0;
        // On-neck: closes near bar 1 low.
        let tol = prev.low.abs() * tol_factor;
        if (cur.close - prev.low).abs() <= tol {
            report.on_neck[i] = true;
        }
        // In-neck: closes between bar 1 close and bar 1 midpoint.
        else if cur.close > prev.close && cur.close < mid1 {
            report.in_neck[i] = true;
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
        let r = compute(&[], 0.5);
        assert!(r.on_neck.is_empty());
        let r2 = compute(&[bar(100.0, 101.0, 99.0, 100.0)], 0.5);
        assert!(!r2.on_neck.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.0),
            bar(f64::NAN, 101.0, 99.0, 100.0),
        ];
        let r = compute(&bars, 0.5);
        assert!(!r.on_neck.iter().any(|x| *x));
    }

    #[test]
    fn on_neck_detected() {
        // Bar 1: bearish 110→100, low=99.5.
        // Bar 2: bullish, opens 95 (< 99.5), closes near low (99.5).
        let bars = vec![bar(110.0, 110.5, 99.5, 100.0), bar(95.0, 100.0, 94.5, 99.7)];
        let r = compute(&bars, 0.5);
        assert!(r.on_neck[1]);
        assert!(!r.in_neck[1]);
    }

    #[test]
    fn in_neck_detected() {
        // Bar 1: bearish 110→100.
        // Bar 2: bullish, opens 95, closes 102 (above bar 1 close 100,
        // below bar 1 midpoint 105).
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(95.0, 102.5, 94.5, 102.0),
        ];
        let r = compute(&bars, 0.5);
        assert!(r.in_neck[1]);
    }

    #[test]
    fn close_above_midpoint_no_signal() {
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(95.0, 108.0, 94.5, 107.0), // close above midpoint 105
        ];
        let r = compute(&bars, 0.5);
        assert!(!r.on_neck[1]);
        assert!(!r.in_neck[1]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 0.5);
        assert_eq!(r.on_neck.len(), 10);
        assert_eq!(r.in_neck.len(), 10);
    }
}
