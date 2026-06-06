//! TTM Trend Bars — John Carter ("Trade the Markets").
//!
//! Classifies each bar as "trend up" / "trend down" / "neutral" based
//! on whether its close exceeds the body-midpoint of every bar in a
//! lookback window. Used as a bar-color overlay to visually separate
//! impulse legs from chop on any timeframe.
//!
//! For each bar i with lookback N:
//!   refs = midpoints (open + close) / 2 for bars i-N..i-1
//!   if close_i > max(refs)        → TrendUp
//!   elif close_i < min(refs)      → TrendDown
//!   else                          → Neutral
//!
//! Default N = 5 (one trading week of daily bars). Pure compute.
//!
//! Companion to `heikin_ashi_reversal`, `td_sequential`, `vsa`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TtmTrendState {
    #[default]
    Neutral,
    TrendUp,
    TrendDown,
}

pub fn compute(bars: &[Bar], lookback: usize) -> Vec<Option<TtmTrendState>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if lookback < 1 || n < lookback + 1 {
        return out;
    }
    if bars
        .iter()
        .any(|b| !b.open.is_finite() || !b.close.is_finite())
    {
        return out;
    }
    for i in lookback..n {
        let win = &bars[i - lookback..i];
        let mid_max = win
            .iter()
            .map(|b| (b.open + b.close) / 2.0)
            .fold(f64::NEG_INFINITY, f64::max);
        let mid_min = win
            .iter()
            .map(|b| (b.open + b.close) / 2.0)
            .fold(f64::INFINITY, f64::min);
        let close = bars[i].close;
        out[i] = Some(if close > mid_max {
            TtmTrendState::TrendUp
        } else if close < mid_min {
            TtmTrendState::TrendDown
        } else {
            TtmTrendState::Neutral
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, c: f64) -> Bar {
        Bar { open: o, close: c }
    }

    #[test]
    fn invalid_inputs_return_all_none() {
        let bars = vec![b(100.0, 101.0); 10];
        assert!(compute(&bars, 0).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..3], 5).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut bars = vec![b(100.0, 101.0); 10];
        bars[5] = b(f64::NAN, 101.0);
        assert!(compute(&bars, 5).iter().all(|x| x.is_none()));
    }

    #[test]
    fn close_above_all_prior_midpoints_marks_trend_up() {
        // 5 prior bars with midpoint 100, then one bar closing at 110.
        let mut bars = vec![b(100.0, 100.0); 5];
        bars.push(b(105.0, 110.0));
        let r = compute(&bars, 5);
        assert_eq!(r[5].unwrap(), TtmTrendState::TrendUp);
    }

    #[test]
    fn close_below_all_prior_midpoints_marks_trend_down() {
        let mut bars = vec![b(100.0, 100.0); 5];
        bars.push(b(95.0, 90.0));
        let r = compute(&bars, 5);
        assert_eq!(r[5].unwrap(), TtmTrendState::TrendDown);
    }

    #[test]
    fn close_inside_window_marks_neutral() {
        // Midpoints spread 99..=103 → window has min=99, max=103.
        let bars = vec![
            b(98.0, 100.0),  // mid = 99
            b(99.0, 101.0),  // mid = 100
            b(100.0, 102.0), // mid = 101
            b(101.0, 103.0), // mid = 102
            b(102.0, 104.0), // mid = 103
            b(101.0, 101.0), // close 101 strictly between 99 and 103
        ];
        let r = compute(&bars, 5);
        assert_eq!(r[5].unwrap(), TtmTrendState::Neutral);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(100.0, 101.0); 30];
        assert_eq!(compute(&bars, 5).len(), 30);
    }
}
