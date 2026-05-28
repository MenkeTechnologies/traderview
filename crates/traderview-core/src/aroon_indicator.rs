//! Aroon Indicator — Tushar Chande (1995).
//!
//! Measures the time since the highest high / lowest low within a
//! lookback window:
//!
//!   AroonUp_t   = 100 · (period − bars_since_period_high) / period
//!   AroonDown_t = 100 · (period − bars_since_period_low)  / period
//!   AroonOsc_t  = AroonUp_t − AroonDown_t
//!
//! Range [0, 100] for each line; Oscillator ∈ [−100, +100].
//!
//! Interpretation:
//!   AroonUp ≈ 100   = price made new high recently → strong uptrend
//!   AroonDown ≈ 100 = price made new low recently → strong downtrend
//!   Both < 50        = consolidation
//!   Crossovers       = trend reversals
//!
//! Default period = 25 (Chande's original recommendation).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AroonReport {
    pub aroon_up: Vec<Option<f64>>,
    pub aroon_down: Vec<Option<f64>>,
    pub aroon_oscillator: Vec<Option<f64>>,
}

pub fn compute(bars: &[Bar], period: usize) -> AroonReport {
    let n = bars.len();
    let mut up = vec![None; n];
    let mut dn = vec![None; n];
    let mut osc = vec![None; n];
    if period < 2 || n < period + 1 {
        return AroonReport { aroon_up: up, aroon_down: dn, aroon_oscillator: osc };
    }
    for i in period..n {
        let win = &bars[i - period..=i];     // (period + 1) bars: 0..period
        if win.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()) { continue; }
        let mut high_idx = 0;
        let mut low_idx = 0;
        for (k, b) in win.iter().enumerate() {
            if b.high > win[high_idx].high { high_idx = k; }
            if b.low < win[low_idx].low { low_idx = k; }
        }
        // bars_since_high = period − high_idx; the most recent bar at idx
        // `period` has bars_since = 0 → Aroon = 100.
        let bars_since_high = (period - high_idx) as f64;
        let bars_since_low = (period - low_idx) as f64;
        let pf = period as f64;
        let u = 100.0 * (pf - bars_since_high) / pf;
        let d = 100.0 * (pf - bars_since_low) / pf;
        up[i] = Some(u);
        dn[i] = Some(d);
        osc[i] = Some(u - d);
    }
    AroonReport { aroon_up: up, aroon_down: dn, aroon_oscillator: osc }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar { Bar { high: h, low: l } }

    #[test]
    fn empty_returns_empty_outputs() {
        let r = compute(&[], 25);
        assert!(r.aroon_up.is_empty());
    }

    #[test]
    fn period_too_small_returns_all_none() {
        let bars: Vec<_> = (0..30).map(|_| b(101.0, 99.0)).collect();
        let r = compute(&bars, 1);
        assert!(r.aroon_up.iter().all(|x| x.is_none()));
    }

    #[test]
    fn strict_uptrend_yields_aroonup_100() {
        let bars: Vec<_> = (0..30).map(|i| b(100.0 + i as f64, 99.0 + i as f64)).collect();
        let r = compute(&bars, 25);
        let last_up = r.aroon_up[29].unwrap();
        let last_down = r.aroon_down[29].unwrap();
        assert!((last_up - 100.0).abs() < 1e-9, "uptrend: AroonUp should be 100, got {last_up}");
        // Lowest low is at the OLDEST bar (index 0 in window) → AroonDown = 0.
        assert!(last_down.abs() < 1e-9, "uptrend: AroonDown should be 0, got {last_down}");
    }

    #[test]
    fn strict_downtrend_yields_aroondown_100() {
        let bars: Vec<_> = (0..30).map(|i| b(100.0 - i as f64, 99.0 - i as f64)).collect();
        let r = compute(&bars, 25);
        let last_up = r.aroon_up[29].unwrap();
        let last_down = r.aroon_down[29].unwrap();
        assert!((last_down - 100.0).abs() < 1e-9, "downtrend: AroonDown should be 100, got {last_down}");
        assert!(last_up.abs() < 1e-9, "downtrend: AroonUp should be 0, got {last_up}");
    }

    #[test]
    fn oscillator_equals_up_minus_down() {
        let bars: Vec<_> = (0..30).map(|i| b(100.0 + (i % 5) as f64,
            99.0 + (i % 5) as f64)).collect();
        let r = compute(&bars, 25);
        for i in 25..30 {
            let u = r.aroon_up[i].unwrap();
            let d = r.aroon_down[i].unwrap();
            let o = r.aroon_oscillator[i].unwrap();
            assert!((u - d - o).abs() < 1e-9);
        }
    }

    #[test]
    fn flat_market_both_indicators_one_hundred_at_first_bar() {
        // All bars identical → the first bar in the window is both highest
        // high and lowest low; bars_since_extreme = period → Aroon = 0.
        // Actually: ties resolve to oldest, so high_idx = low_idx = 0,
        // bars_since = period, Aroon = 100·0/period = 0.
        let bars: Vec<_> = (0..30).map(|_| b(101.0, 99.0)).collect();
        let r = compute(&bars, 25);
        let u = r.aroon_up[29].unwrap();
        let d = r.aroon_down[29].unwrap();
        assert_eq!(u, 0.0);
        assert_eq!(d, 0.0);
    }

    #[test]
    fn outputs_aligned_to_input_length() {
        let bars: Vec<_> = (0..50).map(|i| b(101.0 + i as f64 * 0.1,
            99.0 + i as f64 * 0.1)).collect();
        let r = compute(&bars, 25);
        assert_eq!(r.aroon_up.len(), 50);
        assert!(r.aroon_up[24].is_none());
        assert!(r.aroon_up[25].is_some());
    }
}
