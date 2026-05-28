//! Q-Stick — Tushar Chande (1995).
//!
//! Simple moving average of (close - open) over `period` bars:
//!
//!   QStick_t = SMA(close_t - open_t, period)
//!
//! Interpretation:
//!   - QStick > 0 → bulls dominate (closes consistently above opens)
//!   - QStick < 0 → bears dominate
//!   - Crossings of zero used as entry/exit signals
//!   - Zero-line slope = momentum
//!
//! Default period 8. Pure compute, no allocation hot path.
//!
//! Companion to `balance_of_power`, `ease_of_movement`, `force_index`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub close: f64 }

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || n < period { return out; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.close.is_finite()) { return out; }
    let diffs: Vec<f64> = bars.iter().map(|b| b.close - b.open).collect();
    let p_f = period as f64;
    let mut sum: f64 = diffs[..period].iter().sum();
    out[period - 1] = Some(sum / p_f);
    for i in period..n {
        sum += diffs[i] - diffs[i - period];
        out[i] = Some(sum / p_f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, c: f64) -> Bar { Bar { open: o, close: c } }

    #[test]
    fn empty_returns_empty() { assert!(compute(&[], 8).is_empty()); }

    #[test]
    fn invalid_params_return_all_none() {
        let bars = vec![b(100.0, 101.0); 20];
        assert!(compute(&bars, 1).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 8).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut bars = vec![b(100.0, 101.0); 20];
        bars[3] = b(f64::NAN, 101.0);
        assert!(compute(&bars, 8).iter().all(|x| x.is_none()));
    }

    #[test]
    fn all_bullish_bars_yield_positive_qstick() {
        let bars = vec![b(100.0, 102.0); 20];
        let r = compute(&bars, 8);
        for v in r.iter().flatten() { assert!((v - 2.0).abs() < 1e-9); }
    }

    #[test]
    fn all_bearish_bars_yield_negative_qstick() {
        let bars = vec![b(100.0, 98.0); 20];
        let r = compute(&bars, 8);
        for v in r.iter().flatten() { assert!((v + 2.0).abs() < 1e-9); }
    }

    #[test]
    fn balanced_bars_yield_zero_qstick() {
        let bars: Vec<_> = (0_usize..20).map(|i| {
            if i.is_multiple_of(2) { b(100.0, 101.0) } else { b(101.0, 100.0) }
        }).collect();
        let r = compute(&bars, 8);
        // 8-bar window of alternating ±1 sums to 0.
        for v in r.iter().skip(7).flatten() { assert!(v.abs() < 1e-9); }
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(100.0, 101.0); 30];
        assert_eq!(compute(&bars, 8).len(), 30);
    }
}
