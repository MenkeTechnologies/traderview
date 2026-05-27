//! Williams %R — Larry Williams's momentum oscillator.
//!
//!   %R = (highest_high_N - close) / (highest_high_N - lowest_low_N) × -100
//!
//! Range [-100, 0]. Inverse of stochastic %K — values near 0 indicate
//! close at top of N-period range (overbought); near -100 indicates
//! close at bottom (oversold).
//!
//! Convention:
//!   - %R > -20 → overbought
//!   - %R < -80 → oversold
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<f64> {
    let n = bars.len();
    let mut out = vec![0.0; n];
    if n < period || period == 0 { return out; }
    for i in (period - 1)..n {
        let window = &bars[(i + 1 - period)..=i];
        let highest = window.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let lowest  = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let range = highest - lowest;
        out[i] = if range > 0.0 {
            (highest - bars[i].close) / range * -100.0
        } else { -50.0 };    // flat range convention
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WilliamsRZone { Oversold, Neutral, Overbought }

pub fn classify(wr: f64) -> WilliamsRZone {
    if wr > -20.0 { WilliamsRZone::Overbought }
    else if wr < -80.0 { WilliamsRZone::Oversold }
    else { WilliamsRZone::Neutral }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 14).is_empty());
    }

    #[test]
    fn close_at_window_high_yields_zero() {
        // Close at exact high → numerator 0 → %R = 0 (most overbought).
        let bars: Vec<Bar> = vec![
            b(105.0, 100.0, 102.0),
            b(106.0, 101.0, 103.0),
            b(110.0, 100.0, 110.0),    // close at high
        ];
        let out = compute(&bars, 3);
        assert_eq!(out[2], 0.0);
    }

    #[test]
    fn close_at_window_low_yields_minus_100() {
        let bars: Vec<Bar> = vec![
            b(105.0, 100.0, 102.0),
            b(106.0, 101.0, 103.0),
            b(110.0, 100.0, 100.0),
        ];
        let out = compute(&bars, 3);
        assert_eq!(out[2], -100.0);
    }

    #[test]
    fn close_at_midpoint_yields_minus_50() {
        let bars: Vec<Bar> = vec![
            b(110.0, 100.0, 102.0),
            b(110.0, 100.0, 103.0),
            b(110.0, 100.0, 105.0),    // exact midpoint of 100-110
        ];
        let out = compute(&bars, 3);
        assert_eq!(out[2], -50.0);
    }

    #[test]
    fn flat_range_returns_neutral_50() {
        let bars = vec![b(100.0, 100.0, 100.0); 5];
        let out = compute(&bars, 5);
        assert_eq!(out[4], -50.0);
    }

    #[test]
    fn warmup_bars_zero_values() {
        let bars = vec![b(100.0, 99.0, 99.5); 5];
        let out = compute(&bars, 14);
        for v in &out {
            assert_eq!(*v, 0.0);
        }
    }

    // ─── classify ────────────────────────────────────────────────────

    #[test]
    fn classify_above_minus_20_overbought() {
        assert_eq!(classify(-10.0), WilliamsRZone::Overbought);
        assert_eq!(classify(0.0), WilliamsRZone::Overbought);
    }

    #[test]
    fn classify_below_minus_80_oversold() {
        assert_eq!(classify(-90.0), WilliamsRZone::Oversold);
        assert_eq!(classify(-100.0), WilliamsRZone::Oversold);
    }

    #[test]
    fn classify_middle_neutral() {
        assert_eq!(classify(-50.0), WilliamsRZone::Neutral);
        assert_eq!(classify(-30.0), WilliamsRZone::Neutral);
    }

    #[test]
    fn classify_boundary_at_minus_20_or_minus_80_neutral() {
        // Strict > -20 and < -80 — boundary itself is neutral.
        assert_eq!(classify(-20.0), WilliamsRZone::Neutral);
        assert_eq!(classify(-80.0), WilliamsRZone::Neutral);
    }
}
