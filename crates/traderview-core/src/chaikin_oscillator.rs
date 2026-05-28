//! Chaikin Oscillator — Marc Chaikin.
//!
//! MACD-style oscillator applied to the Accumulation/Distribution Line:
//!
//!   ADL_t  = running cumulative MFV (see `accumulation_distribution_line`)
//!   CO_t   = EMA(ADL, fast) − EMA(ADL, slow)
//!
//! Default: fast = 3, slow = 10.
//!
//! Interpretation:
//!   - CO > 0 = short-term ADL momentum above long-term → buying pressure
//!   - CO < 0 = selling pressure
//!   - Zero-line crossovers + divergences with price are the primary
//!     signals.
//!
//! Distinct from CMF: CMF is a bounded oscillator [−1,+1] on rolling MFV;
//! Chaikin Oscillator is an unbounded MACD on cumulative ADL.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

use crate::accumulation_distribution_line;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub fn compute(bars: &[Bar], fast: usize, slow: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if n == 0 || fast == 0 || slow == 0 || fast >= slow { return out; }
    // Compute ADL series first.
    let adl_input: Vec<accumulation_distribution_line::Bar> = bars.iter().map(|b| {
        accumulation_distribution_line::Bar {
            high: b.high, low: b.low, close: b.close, volume: b.volume,
        }
    }).collect();
    let adl = accumulation_distribution_line::compute(&adl_input);
    let fast_ema = ema(&adl, fast);
    let slow_ema = ema(&adl, slow);
    for i in 0..n {
        if let (Some(f), Some(s)) = (fast_ema[i], slow_ema[i]) {
            out[i] = Some(f - s);
        }
    }
    out
}

fn ema(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    // Seed with SMA of first `period` values.
    let mut have_seed = true;
    let mut seed_sum = 0.0;
    for v in series.iter().take(period) {
        match v {
            Some(x) => seed_sum += x,
            None => { have_seed = false; break; }
        }
    }
    if !have_seed { return out; }
    let k = 2.0 / (period as f64 + 1.0);
    let mut cur = seed_sum / period as f64;
    out[period - 1] = Some(cur);
    for i in period..n {
        if let Some(v) = series[i] {
            cur = v * k + cur * (1.0 - k);
            out[i] = Some(cur);
        } else {
            out[i] = Some(cur);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar { high: h, low: l, close: c, volume: v }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 3, 10).is_empty());
    }

    #[test]
    fn invalid_periods_return_all_none() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 20];
        assert!(compute(&bars, 0, 10).iter().all(|x| x.is_none()));
        assert!(compute(&bars, 10, 3).iter().all(|x| x.is_none()));    // fast >= slow
        assert!(compute(&bars, 5, 5).iter().all(|x| x.is_none()));
    }

    #[test]
    fn shorter_than_slow_returns_all_none() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 5];
        let out = compute(&bars, 3, 10);
        assert!(out.iter().all(|x| x.is_none()));
    }

    #[test]
    fn sustained_accumulation_yields_positive_oscillator() {
        // Closes pinned to highs every bar with steady volume.
        let bars = vec![b(101.0, 99.0, 101.0, 1000.0); 30];
        let out = compute(&bars, 3, 10);
        let last = out[29].unwrap();
        // ADL is strictly linearly increasing; the fast EMA tracks it more
        // closely than the slow EMA, so fast > slow → positive oscillator.
        assert!(last > 0.0, "accumulation should yield positive CO, got {last}");
    }

    #[test]
    fn sustained_distribution_yields_negative_oscillator() {
        let bars = vec![b(101.0, 99.0, 99.0, 1000.0); 30];
        let out = compute(&bars, 3, 10);
        let last = out[29].unwrap();
        assert!(last < 0.0, "distribution should yield negative CO, got {last}");
    }

    #[test]
    fn flat_midpoint_oscillator_near_zero() {
        // All bars MFM = 0 → ADL flat at 0 → both EMAs = 0 → oscillator = 0.
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        let out = compute(&bars, 3, 10);
        let last = out[29].unwrap();
        assert!(last.abs() < 1e-9);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 50];
        let out = compute(&bars, 3, 10);
        assert_eq!(out.len(), 50);
        // First slow-1 slots = None (no slow EMA yet).
        assert!(out[8].is_none());
        assert!(out[9].is_some());
    }
}
