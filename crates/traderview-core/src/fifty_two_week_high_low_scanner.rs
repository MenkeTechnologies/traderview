//! 52-Week High/Low Scanner.
//!
//! Per-bar proximity scan against a configurable lookback window
//! (252 trading days = 1 year by default). For each bar, reports:
//!   - distance_from_high_pct: (high - bar.close) / high · 100
//!   - distance_from_low_pct:  (bar.close - low) / low · 100
//!   - at_new_high:  bar.high  > previous max(high, lookback bars)
//!   - at_new_low:   bar.low   < previous min(low,  lookback bars)
//!
//! Wall-Street convention: a stock at or near its 52-week high is
//! a momentum-leader candidate; near 52-week low signals weakness or
//! deep-value setup depending on the broader context.
//!
//! Pure compute. Default lookback = 252.
//! Companion to `breakout_52w_scanner`, `momentum_12_1`,
//! `relative_strength_vs_market`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FiftyTwoWeekReport {
    pub distance_from_high_pct: Vec<Option<f64>>,
    pub distance_from_low_pct: Vec<Option<f64>>,
    pub at_new_high: Vec<Option<bool>>,
    pub at_new_low: Vec<Option<bool>>,
    pub lookback: usize,
}

pub fn compute(bars: &[Bar], lookback: usize) -> FiftyTwoWeekReport {
    let n = bars.len();
    let mut report = FiftyTwoWeekReport {
        distance_from_high_pct: vec![None; n],
        distance_from_low_pct: vec![None; n],
        at_new_high: vec![None; n],
        at_new_low: vec![None; n],
        lookback,
    };
    if lookback < 2 || n < lookback + 1 {
        return report;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return report;
    }
    for i in lookback..n {
        let win = &bars[i - lookback..i];
        let win_high = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let win_low = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        let cur = bars[i];
        if win_high > 0.0 {
            report.distance_from_high_pct[i] = Some((win_high - cur.close) / win_high * 100.0);
        }
        if win_low > 0.0 {
            report.distance_from_low_pct[i] = Some((cur.close - win_low) / win_low * 100.0);
        }
        report.at_new_high[i] = Some(cur.high > win_high);
        report.at_new_low[i] = Some(cur.low < win_low);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 1);
        assert!(r.at_new_high.iter().all(|x| x.is_none()));
        let r2 = compute(&bars[..10], 252);
        assert!(r2.at_new_high.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 300];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 252);
        assert!(r.at_new_high.iter().all(|x| x.is_none()));
    }

    #[test]
    fn new_high_detected() {
        // 252 bars at 100/101, then bar 252 with high 110.
        let mut bars = vec![b(101.0, 99.0, 100.0); 252];
        bars.push(b(110.0, 99.0, 109.0));
        let r = compute(&bars, 252);
        assert!(r.at_new_high[252].unwrap());
        // Distance from previous high (101) is negative (current close 109 above 101).
        // distance_from_high_pct uses prior-window high = 101, so
        // (101 - 109)/101 = -7.92%.
        assert!(r.distance_from_high_pct[252].unwrap() < 0.0);
    }

    #[test]
    fn new_low_detected() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 252];
        bars.push(b(101.0, 90.0, 91.0));
        let r = compute(&bars, 252);
        assert!(r.at_new_low[252].unwrap());
    }

    #[test]
    fn no_new_extreme_when_inside_range() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 252];
        bars.push(b(100.5, 99.5, 100.0));
        let r = compute(&bars, 252);
        assert!(!r.at_new_high[252].unwrap());
        assert!(!r.at_new_low[252].unwrap());
    }

    #[test]
    fn distance_from_high_zero_when_close_at_high() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 252];
        bars.push(b(101.0, 99.0, 101.0));
        let r = compute(&bars, 252);
        assert!(r.distance_from_high_pct[252].unwrap().abs() < 1e-9);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 300];
        let r = compute(&bars, 252);
        assert_eq!(r.at_new_high.len(), 300);
        assert_eq!(r.at_new_low.len(), 300);
        assert_eq!(r.distance_from_high_pct.len(), 300);
        assert_eq!(r.distance_from_low_pct.len(), 300);
    }
}
