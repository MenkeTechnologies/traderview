//! Separating Lines — 2-bar continuation pattern with matching opens.
//!
//! Bullish Separating Lines:
//!   Bar 1: bearish body in an uptrend (counter-trend pullback)
//!   Bar 2: bullish body that OPENS at approximately the same price
//!     as bar 1's open (within tolerance) AND closes higher, resuming
//!     the trend.
//!
//! Bearish Separating Lines: mirrored — bar 1 bullish in a downtrend,
//! bar 2 bearish opening at bar 1's open and closing lower.
//!
//! Differs from `meeting_lines` (which match CLOSES not opens).
//!
//! Pure compute. Default tolerance_pct = 0.3.
//! Companion to `counter_attack_lines`, `harami_pattern`,
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
pub struct SeparatingLinesReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
    pub tolerance_pct: f64,
    pub trend_period: usize,
}

pub fn compute(bars: &[Bar], trend_period: usize, tolerance_pct: f64) -> SeparatingLinesReport {
    let n = bars.len();
    let mut report = SeparatingLinesReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
        tolerance_pct,
        trend_period,
    };
    if trend_period < 2
        || n < trend_period + 1
        || !tolerance_pct.is_finite()
        || tolerance_pct <= 0.0
    {
        return report;
    }
    if bars.iter().any(|b| {
        !b.open.is_finite() || !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()
    }) {
        return report;
    }
    let tol_factor = tolerance_pct / 100.0;
    for i in trend_period..n {
        let prev = bars[i - 1];
        let cur = bars[i];
        let start_close = bars[i - trend_period].close;
        let end_close = bars[i - 1].close;
        let trend_up = end_close > start_close;
        let trend_down = end_close < start_close;
        let tol = prev.open.abs() * tol_factor;
        let matching_open = (cur.open - prev.open).abs() <= tol;
        // Bullish: in uptrend, b1 bearish (pullback), b2 bullish with same open.
        if trend_up
            && prev.close < prev.open
            && cur.close > cur.open
            && matching_open
            && cur.close > prev.open
        {
            report.bullish[i] = true;
        }
        // Bearish: in downtrend, b1 bullish (bounce), b2 bearish with same open.
        if trend_down
            && prev.close > prev.open
            && cur.close < cur.open
            && matching_open
            && cur.close < prev.open
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
    fn invalid_inputs_return_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 1, 0.3);
        assert!(!r.bullish.iter().any(|x| *x));
        let r2 = compute(&bars, 5, 0.0);
        assert!(!r2.bullish.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        bars[5] = bar(f64::NAN, 101.0, 99.0, 100.5);
        let r = compute(&bars, 5, 0.3);
        assert!(!r.bullish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_separating_lines_detected() {
        // 5 bars uptrend (close 100→105), bar 5 bearish pullback,
        // bar 6 bullish opening at same price.
        let mut bars: Vec<_> = (0..5)
            .map(|i| {
                let p = 100.0 + i as f64;
                bar(p, p + 0.5, p - 0.5, p + 0.4)
            })
            .collect();
        // Bar 5 bearish: open=105, close=103.
        bars.push(bar(105.0, 105.5, 102.0, 103.0));
        // Bar 6 bullish: open=105 (matches bar 5 open), close=108.
        bars.push(bar(105.0, 109.0, 104.5, 108.0));
        let r = compute(&bars, 5, 0.3);
        assert!(r.bullish[6]);
    }

    #[test]
    fn bearish_separating_lines_detected() {
        let mut bars: Vec<_> = (0..5)
            .map(|i| {
                let p = 100.0 - i as f64;
                bar(p, p + 0.5, p - 0.5, p - 0.4)
            })
            .collect();
        bars.push(bar(95.0, 97.0, 94.0, 96.5));
        bars.push(bar(95.0, 95.5, 91.0, 92.0));
        let r = compute(&bars, 5, 0.3);
        assert!(r.bearish[6]);
    }

    #[test]
    fn mismatched_opens_rejected() {
        let mut bars: Vec<_> = (0..5)
            .map(|i| {
                let p = 100.0 + i as f64;
                bar(p, p + 0.5, p - 0.5, p + 0.4)
            })
            .collect();
        bars.push(bar(105.0, 105.5, 102.0, 103.0));
        bars.push(bar(110.0, 115.0, 108.5, 113.0)); // open 110 ≠ 105
        let r = compute(&bars, 5, 0.3);
        assert!(!r.bullish[6]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 5, 0.3);
        assert_eq!(r.bullish.len(), 10);
        assert_eq!(r.bearish.len(), 10);
    }
}
