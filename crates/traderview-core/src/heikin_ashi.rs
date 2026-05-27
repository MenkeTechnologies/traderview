//! Heikin-Ashi candle smoother.
//!
//! Each Heikin-Ashi candle is computed from the actual bar PLUS the
//! previous HA candle:
//!
//!   HA_close = (open + high + low + close) / 4
//!   HA_open  = (prev_HA_open + prev_HA_close) / 2   (first bar = own open)
//!   HA_high  = max(high, HA_open, HA_close)
//!   HA_low   = min(low, HA_open, HA_close)
//!
//! Smoothed visualization that makes trends clearer (long runs of same-
//! color candles) and pullbacks easier to spot. NOT a substitute for
//! real prices — HA candles can't be traded against.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct HaBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl HaBar {
    pub fn is_bull(&self) -> bool { self.close > self.open }
    pub fn is_bear(&self) -> bool { self.close < self.open }
}

pub fn compute(bars: &[Bar]) -> Vec<HaBar> {
    let mut out = Vec::with_capacity(bars.len());
    let mut prev: Option<HaBar> = None;
    for b in bars {
        let ha_close = (b.open + b.high + b.low + b.close) / 4.0;
        let ha_open = match prev {
            Some(p) => (p.open + p.close) / 2.0,
            None    => b.open,
        };
        let ha_high = b.high.max(ha_open).max(ha_close);
        let ha_low = b.low.min(ha_open).min(ha_close);
        let ha = HaBar { open: ha_open, high: ha_high, low: ha_low, close: ha_close };
        out.push(ha);
        prev = Some(ha);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar { open: o, high: h, low: l, close: c }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[]).is_empty());
    }

    #[test]
    fn first_bar_open_equals_actual_open() {
        let bars = vec![b(100.0, 105.0, 99.0, 103.0)];
        let out = compute(&bars);
        assert_eq!(out[0].open, 100.0);
    }

    #[test]
    fn ha_close_is_ohlc_average() {
        // (100 + 105 + 99 + 103) / 4 = 101.75.
        let bars = vec![b(100.0, 105.0, 99.0, 103.0)];
        let out = compute(&bars);
        assert_eq!(out[0].close, 101.75);
    }

    #[test]
    fn ha_high_at_least_as_high_as_bar_high() {
        let bars = vec![b(100.0, 105.0, 99.0, 103.0)];
        let out = compute(&bars);
        assert!(out[0].high >= 105.0);
    }

    #[test]
    fn ha_low_at_most_as_low_as_bar_low() {
        let bars = vec![b(100.0, 105.0, 99.0, 103.0)];
        let out = compute(&bars);
        assert!(out[0].low <= 99.0);
    }

    #[test]
    fn second_bar_open_is_prior_ha_midpoint() {
        // bar 1: HA close=101.75, HA open=100. Midpoint = 100.875.
        let bars = vec![
            b(100.0, 105.0, 99.0, 103.0),
            b(103.0, 108.0, 102.0, 106.0),
        ];
        let out = compute(&bars);
        assert_eq!(out[1].open, 100.875);
    }

    #[test]
    fn bull_helper_true_when_close_above_open() {
        let h = HaBar { open: 100.0, close: 105.0, high: 105.0, low: 100.0 };
        assert!(h.is_bull());
        assert!(!h.is_bear());
    }

    #[test]
    fn bear_helper_true_when_close_below_open() {
        let h = HaBar { open: 105.0, close: 100.0, high: 105.0, low: 100.0 };
        assert!(h.is_bear());
        assert!(!h.is_bull());
    }

    #[test]
    fn trending_market_produces_runs_of_same_color() {
        // 10 bars of clean uptrend → ALL HA bars should be bullish.
        let bars: Vec<Bar> = (1..=10).map(|i| {
            let o = 100.0 + i as f64;
            b(o, o + 1.0, o - 0.2, o + 0.8)
        }).collect();
        let out = compute(&bars);
        // First bar might be neutral; later bars all bullish.
        for ha in &out[1..] {
            assert!(ha.is_bull(), "trending up → all subsequent HA bars bullish");
        }
    }

    #[test]
    fn series_length_matches_input() {
        let bars: Vec<Bar> = (0..50).map(|_| b(100.0, 101.0, 99.0, 100.5)).collect();
        let out = compute(&bars);
        assert_eq!(out.len(), 50);
    }
}
