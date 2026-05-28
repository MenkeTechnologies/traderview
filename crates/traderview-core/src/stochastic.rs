//! Stochastic Oscillator (Lane).
//!
//! Two flavors:
//!   - Fast %K = (close - lowest_low_N) / (highest_high_N - lowest_low_N) × 100
//!   - Fast %D = 3-period SMA of %K
//!   - Slow %K = Fast %D (smoothed once)
//!   - Slow %D = 3-period SMA of Slow %K
//!
//! Standard (14, 3, 3) parameter set is the canonical reading.
//! Convention: >80 = overbought, <20 = oversold. Crossovers of %K and %D
//! are the entry signals.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StochPoint {
    pub fast_k: f64,
    pub fast_d: f64,
    pub slow_k: f64,
    pub slow_d: f64,
}

pub fn compute(bars: &[Bar], k_period: usize, d_period: usize) -> Vec<StochPoint> {
    let n = bars.len();
    let mut out = vec![StochPoint::default(); n];
    if n < k_period || k_period == 0 || d_period == 0 {
        return out;
    }
    let mut fast_k_series = vec![0.0; n];
    for i in (k_period - 1)..n {
        let window = &bars[(i + 1 - k_period)..=i];
        let lowest = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let highest = window
            .iter()
            .map(|b| b.high)
            .fold(f64::NEG_INFINITY, f64::max);
        let range = highest - lowest;
        fast_k_series[i] = if range > 0.0 {
            (bars[i].close - lowest) / range * 100.0
        } else {
            50.0
        };
        out[i].fast_k = fast_k_series[i];
    }
    // Fast %D = SMA(fast_k, d_period). Saturating math against hostile
    // JSON `k_period = usize::MAX` / `d_period = usize::MAX` — naive
    // `k_period + d_period - 2` would panic in debug / wrap in release
    // and corrupt the loop bounds (potentially OOB-slicing fast_k_series).
    let mut fast_d_series = vec![0.0; n];
    let fast_d_start = k_period.saturating_add(d_period).saturating_sub(2);
    for i in fast_d_start..n {
        let window = &fast_k_series[(i + 1 - d_period)..=i];
        let avg = window.iter().sum::<f64>() / d_period as f64;
        fast_d_series[i] = avg;
        out[i].fast_d = avg;
        // Slow %K = Fast %D.
        out[i].slow_k = avg;
    }
    // Slow %D = SMA(slow_k, d_period) = SMA(fast_d, d_period).
    let slow_d_start = k_period
        .saturating_add(d_period.saturating_mul(2))
        .saturating_sub(3);
    for i in slow_d_start..n {
        let window = &fast_d_series[(i + 1 - d_period)..=i];
        out[i].slow_d = window.iter().sum::<f64>() / d_period as f64;
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StochZone {
    Oversold,
    Neutral,
    Overbought,
}

pub fn classify(k: f64) -> StochZone {
    if k > 80.0 {
        StochZone::Overbought
    } else if k < 20.0 {
        StochZone::Oversold
    } else {
        StochZone::Neutral
    }
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
    fn empty_returns_empty() {
        assert!(compute(&[], 14, 3).is_empty());
    }

    #[test]
    fn series_under_k_period_emits_zero_points() {
        let bars = vec![b(10.0, 9.0, 9.5); 5];
        let out = compute(&bars, 14, 3);
        for p in &out {
            assert_eq!(p.fast_k, 0.0);
        }
    }

    #[test]
    fn close_at_window_high_yields_fast_k_100() {
        // 5-period window. Last bar closes at high of range.
        let bars: Vec<Bar> = vec![
            b(105.0, 100.0, 102.0),
            b(106.0, 101.0, 103.0),
            b(107.0, 102.0, 104.0),
            b(108.0, 103.0, 105.0),
            b(110.0, 100.0, 110.0), // close at high of window
        ];
        let out = compute(&bars, 5, 3);
        assert_eq!(out[4].fast_k, 100.0);
    }

    #[test]
    fn close_at_window_low_yields_fast_k_zero() {
        let bars: Vec<Bar> = vec![
            b(105.0, 100.0, 102.0),
            b(106.0, 101.0, 103.0),
            b(107.0, 102.0, 104.0),
            b(108.0, 103.0, 105.0),
            b(110.0, 100.0, 100.0), // close at low
        ];
        let out = compute(&bars, 5, 3);
        assert_eq!(out[4].fast_k, 0.0);
    }

    #[test]
    fn flat_range_yields_50_pct_fast_k() {
        // All bars exact same H/L/C → range = 0 → 50% convention.
        let bars = vec![b(100.0, 100.0, 100.0); 5];
        let out = compute(&bars, 5, 3);
        assert_eq!(out[4].fast_k, 50.0);
    }

    #[test]
    fn fast_d_is_three_period_sma_of_fast_k() {
        // Construct bars so fast_k values are deterministic.
        let bars: Vec<Bar> = vec![
            b(110.0, 100.0, 105.0),
            b(110.0, 100.0, 100.0), // fast_k = 0
            b(110.0, 100.0, 110.0), // fast_k = 100
            b(110.0, 100.0, 105.0), // fast_k = 50
        ];
        let out = compute(&bars, 1, 3);
        // After k_period=1 + d_period=3 - 1 = 3 bars warm-up, fast_d available.
        // fast_k at idx 1=0, idx 2=100, idx 3=50. SMA = (0+100+50)/3 = 50.
        assert_eq!(out[3].fast_d, 50.0);
    }

    #[test]
    fn slow_k_equals_fast_d() {
        let bars: Vec<Bar> = vec![
            b(110.0, 100.0, 105.0),
            b(110.0, 100.0, 100.0),
            b(110.0, 100.0, 110.0),
            b(110.0, 100.0, 105.0),
        ];
        let out = compute(&bars, 1, 3);
        assert_eq!(out[3].slow_k, out[3].fast_d);
    }

    // ─── classify ────────────────────────────────────────────────────

    #[test]
    fn classify_over_80_overbought() {
        assert_eq!(classify(85.0), StochZone::Overbought);
    }

    #[test]
    fn classify_under_20_oversold() {
        assert_eq!(classify(15.0), StochZone::Oversold);
    }

    #[test]
    fn classify_middle_neutral() {
        assert_eq!(classify(50.0), StochZone::Neutral);
    }

    #[test]
    fn classify_boundary_at_20_or_80_neutral() {
        // strict > 80 and strict < 20 — boundary itself is neutral.
        assert_eq!(classify(20.0), StochZone::Neutral);
        assert_eq!(classify(80.0), StochZone::Neutral);
    }
}
