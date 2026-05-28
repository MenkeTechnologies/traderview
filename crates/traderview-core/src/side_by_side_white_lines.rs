//! Side-by-Side White Lines — 3-bar continuation pattern after an
//! upside gap.
//!
//!   Bar 1: bullish bar in an uptrend
//!   Bar 2: bullish bar that GAPS UP from bar 1 (bar 2.low > bar 1.high)
//!   Bar 3: another bullish bar with approximately the same OPEN AND
//!     CLOSE as bar 2 (within tolerance) — confirms the gap holds
//!     and bears can't fade it
//!
//! Bearish mirror (Side-by-Side Black Lines): bar 1 bearish, gap down,
//! bars 2 and 3 bearish with matching opens/closes.
//!
//! Pure compute. Default tolerance_pct = 0.3.
//! Companion to `tasuki_gap`, `gap_classifier`, `mat_hold_pattern`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SideBySideReport {
    pub bullish_white: Vec<bool>,
    pub bearish_black: Vec<bool>,
    pub tolerance_pct: f64,
}

pub fn compute(bars: &[Bar], tolerance_pct: f64) -> SideBySideReport {
    let n = bars.len();
    let mut report = SideBySideReport {
        bullish_white: vec![false; n],
        bearish_black: vec![false; n],
        tolerance_pct,
    };
    if n < 3 || !tolerance_pct.is_finite() || tolerance_pct <= 0.0 { return report; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    let tol_factor = tolerance_pct / 100.0;
    for i in 2..n {
        let (b1, b2, b3) = (bars[i - 2], bars[i - 1], bars[i]);
        // Bullish white lines.
        if b1.close > b1.open
            && b2.close > b2.open
            && b3.close > b3.open
            && b2.low > b1.high
        {
            let tol_open = b2.open.abs() * tol_factor;
            let tol_close = b2.close.abs() * tol_factor;
            if (b3.open - b2.open).abs() <= tol_open
                && (b3.close - b2.close).abs() <= tol_close {
                report.bullish_white[i] = true;
            }
        }
        // Bearish black lines.
        if b1.close < b1.open
            && b2.close < b2.open
            && b3.close < b3.open
            && b2.high < b1.low
        {
            let tol_open = b2.open.abs() * tol_factor;
            let tol_close = b2.close.abs() * tol_factor;
            if (b3.open - b2.open).abs() <= tol_open
                && (b3.close - b2.close).abs() <= tol_close {
                report.bearish_black[i] = true;
            }
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
    fn empty_returns_empty() {
        let r = compute(&[], 0.3);
        assert!(r.bullish_white.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0),
                        bar(f64::NAN, 101.0, 99.0, 100.0),
                        bar(100.0, 101.0, 99.0, 100.0)];
        let r = compute(&bars, 0.3);
        assert!(!r.bullish_white.iter().any(|x| *x));
    }

    #[test]
    fn bullish_side_by_side_white_detected() {
        // Bar 1: bullish 100→105, high=105.
        // Bar 2: bullish 108→113, low=107 (gap up).
        // Bar 3: bullish 108.1→112.9 (matches bar 2).
        let bars = vec![
            bar(100.0, 105.0, 99.5, 105.0),
            bar(108.0, 113.5, 107.0, 113.0),
            bar(108.1, 113.5, 107.5, 112.9),
        ];
        let r = compute(&bars, 0.3);
        assert!(r.bullish_white[2]);
    }

    #[test]
    fn bearish_side_by_side_black_detected() {
        let bars = vec![
            bar(110.0, 110.5, 105.0, 105.0),
            bar(102.0, 103.0, 97.0, 97.0),
            bar(101.9, 103.0, 97.0, 97.0),
        ];
        let r = compute(&bars, 0.3);
        assert!(r.bearish_black[2]);
    }

    #[test]
    fn no_gap_rejects() {
        let bars = vec![
            bar(100.0, 105.0, 99.5, 105.0),
            bar(104.0, 110.0, 103.0, 109.0),    // low 103 < bar 1 high 105
            bar(104.1, 110.0, 103.5, 109.0),
        ];
        let r = compute(&bars, 0.3);
        assert!(!r.bullish_white[2]);
    }

    #[test]
    fn mismatched_bar3_rejects() {
        let bars = vec![
            bar(100.0, 105.0, 99.5, 105.0),
            bar(108.0, 113.5, 107.0, 113.0),
            bar(120.0, 125.0, 119.5, 124.0),    // open 120 ≠ 108
        ];
        let r = compute(&bars, 0.3);
        assert!(!r.bullish_white[2]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 0.3);
        assert_eq!(r.bullish_white.len(), 10);
        assert_eq!(r.bearish_black.len(), 10);
    }
}
