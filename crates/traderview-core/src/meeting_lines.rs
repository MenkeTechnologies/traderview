//! Meeting Lines — 2-bar reversal pattern with matching CLOSES (vs
//! `separating_lines` which matches OPENS, and `counter_attack_lines`
//! which also matches closes but in a slightly different geometry).
//!
//! Bullish Meeting Lines:
//!   Bar 1: tall bearish body
//!   Bar 2: bullish body that opens LOWER than bar 1's close (continues
//!     down intrabar) but closes at approximately bar 1's close
//!     (within tolerance)
//!
//! Bearish Meeting Lines: mirrored — bar 1 bullish, bar 2 bearish
//! opening higher and closing back at bar 1's close.
//!
//! Pure compute. Default tolerance_pct = 0.3.
//! Companion to `separating_lines`, `counter_attack_lines`,
//! `harami_pattern`, `dark_cloud_piercing`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeetingLinesReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
    pub tolerance_pct: f64,
}

pub fn compute(bars: &[Bar], tolerance_pct: f64) -> MeetingLinesReport {
    let n = bars.len();
    let mut report = MeetingLinesReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
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
        let body1 = (prev.close - prev.open).abs();
        let range1 = prev.high - prev.low;
        if range1 <= 0.0 || body1 < 0.6 * range1 {
            continue;
        }
        let tol = prev.close.abs() * tol_factor;
        let matching_close = (cur.close - prev.close).abs() <= tol;
        // Bullish: bar 1 bearish, bar 2 bullish opens below bar 1 close,
        // closes back at bar 1 close.
        if prev.close < prev.open && cur.close > cur.open && cur.open < prev.close && matching_close
        {
            report.bullish[i] = true;
        }
        // Bearish: bar 1 bullish, bar 2 bearish opens above bar 1 close,
        // closes back at bar 1 close.
        if prev.close > prev.open && cur.close < cur.open && cur.open > prev.close && matching_close
        {
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
        let r = compute(&[], 0.3);
        assert!(r.bullish.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.0),
            bar(f64::NAN, 101.0, 99.0, 100.0),
        ];
        let r = compute(&bars, 0.3);
        assert!(!r.bullish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_meeting_lines_detected() {
        // Bar 1: bearish 110→100 (body 10 of range 11).
        // Bar 2: bullish opens 95 (< bar 1 close 100), closes 100.2 (within 0.3% of 100).
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(95.0, 100.5, 94.5, 100.2),
        ];
        let r = compute(&bars, 0.3);
        assert!(r.bullish[1]);
    }

    #[test]
    fn bearish_meeting_lines_detected() {
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(115.0, 115.5, 109.5, 109.9),
        ];
        let r = compute(&bars, 0.3);
        assert!(r.bearish[1]);
    }

    #[test]
    fn close_too_far_rejects() {
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(95.0, 105.0, 94.5, 104.0), // close 104, 4% off
        ];
        let r = compute(&bars, 0.3);
        assert!(!r.bullish[1]);
    }

    #[test]
    fn small_bar1_body_rejects() {
        // Bar 1 body too small (doji-ish).
        let bars = vec![bar(100.0, 101.0, 99.0, 99.8), bar(95.0, 100.0, 94.5, 99.7)];
        let r = compute(&bars, 0.3);
        assert!(!r.bullish[1]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 0.3);
        assert_eq!(r.bullish.len(), 10);
        assert_eq!(r.bearish.len(), 10);
    }
}
