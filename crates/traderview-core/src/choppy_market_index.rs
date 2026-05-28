//! Choppy Market Index — Tushar Chande (companion to Chande Momentum
//! Oscillator).
//!
//! Distinct from `choppiness` (Bill Dreiss). Chande's CMI quantifies
//! how much of total intrabar range was used in directional moves:
//!
//!   sum_change_t = Σ (close_k - close_{k-1}) for k in last period bars
//!   sum_range_t  = Σ (high_k - low_k)         for k in last period bars
//!   CMI_t        = | sum_change_t / sum_range_t | · 100
//!
//! Range [0, 100]:
//!   CMI > 60 → strong trending move (directional change ≈ range)
//!   CMI < 30 → choppy / mean-reverting (lots of range with little net change)
//!
//! Pure compute. Default period = 14. Companion to `choppiness`,
//! `efficiency_ratio`, `ehlers_correlation_trend_indicator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || n < period + 1 { return out; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return out;
    }
    let mut deltas = vec![0.0_f64; n];
    let mut ranges = vec![0.0_f64; n];
    for i in 1..n {
        deltas[i] = bars[i].close - bars[i - 1].close;
        ranges[i] = bars[i].high - bars[i].low;
    }
    for i in period..n {
        let win_d: f64 = deltas[i + 1 - period..=i].iter().sum();
        let win_r: f64 = ranges[i + 1 - period..=i].iter().sum();
        if win_r > 0.0 {
            out[i] = Some((win_d / win_r).abs() * 100.0);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert!(compute(&bars, 1).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        assert!(compute(&bars, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_cmi() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 14);
        for v in r.iter().flatten() { assert!(v.abs() < 1e-9); }
    }

    #[test]
    fn pure_uptrend_yields_high_cmi() {
        let bars: Vec<_> = (0..30).map(|i| {
            let m = 100.0 + i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 14);
        let last = r[29].unwrap();
        // Sum-change = 14 (one per bar), sum-range = 14 (1 per bar) → CMI = 100.
        assert!(last > 95.0,
            "pure uptrend should yield CMI near 100, got {last}");
    }

    #[test]
    fn pure_downtrend_yields_high_cmi() {
        let bars: Vec<_> = (0..30).map(|i| {
            let m = 200.0 - i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 14);
        let last = r[29].unwrap();
        assert!(last > 95.0);
    }

    #[test]
    fn output_in_zero_hundred_range() {
        let mut state: u64 = 42;
        let bars: Vec<_> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            let m = 100.0 + (r - 0.5) * 4.0;
            b(m + 1.0, m - 1.0, m)
        }).collect();
        let r = compute(&bars, 14);
        for v in r.iter().flatten() {
            assert!((0.0..=100.0).contains(v));
        }
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert_eq!(compute(&bars, 14).len(), 30);
    }
}
