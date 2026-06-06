//! Williams Accumulation/Distribution — Larry Williams.
//!
//! Cumulative tally that adds a per-bar "true range high" component
//! on up-close bars and subtracts a "true range low" component on
//! down-close bars:
//!
//!   trh_t = max(high_t, close_{t-1})
//!   trl_t = min(low_t, close_{t-1})
//!   if close_t > close_{t-1}: add (close_t − trl_t)
//!   if close_t < close_{t-1}: subtract (trh_t − close_t)
//!   if close_t == close_{t-1}: contribute 0
//!
//! Cumulative across bars → Williams A/D line.
//!
//! Distinct from `accumulation_distribution_line` (Chaikin's MFV-based
//! cumulative) and `on_balance_volume` (sign-of-close volume).
//!
//! Pure compute. Companion to `accumulation_distribution_line`,
//! `on_balance_volume`, `chaikin_oscillator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn compute(bars: &[Bar]) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if n == 0 {
        return out;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return out;
    }
    let mut wad = 0.0_f64;
    out[0] = Some(0.0);
    for i in 1..n {
        let trh = bars[i].high.max(bars[i - 1].close);
        let trl = bars[i].low.min(bars[i - 1].close);
        let delta = if bars[i].close > bars[i - 1].close {
            bars[i].close - trl
        } else if bars[i].close < bars[i - 1].close {
            -(trh - bars[i].close)
        } else {
            0.0
        };
        wad += delta;
        out[i] = Some(wad);
    }
    out
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
        assert!(compute(&[]).is_empty());
    }

    #[test]
    fn nan_returns_all_none() {
        let bars = vec![b(101.0, 99.0, 100.0), b(f64::NAN, 99.0, 100.0)];
        assert!(compute(&bars).iter().all(|x| x.is_none()));
    }

    #[test]
    fn first_bar_yields_zero() {
        let bars = vec![b(101.0, 99.0, 100.0)];
        let r = compute(&bars);
        assert_eq!(r[0].unwrap(), 0.0);
    }

    #[test]
    fn unchanged_close_no_change() {
        let bars = vec![b(101.0, 99.0, 100.0), b(102.0, 98.0, 100.0)];
        let r = compute(&bars);
        assert_eq!(r[1].unwrap(), 0.0);
    }

    #[test]
    fn up_close_adds_positive_increment() {
        // prev close = 100; up to 102. low = 99, prev close = 100; trl = 99.
        // delta = 102 - 99 = 3 → +3.
        let bars = vec![b(101.0, 99.0, 100.0), b(103.0, 99.0, 102.0)];
        let r = compute(&bars);
        assert_eq!(r[1].unwrap(), 3.0);
    }

    #[test]
    fn down_close_subtracts() {
        let bars = vec![b(101.0, 99.0, 100.0), b(101.0, 95.0, 96.0)];
        // trh = max(101, 100) = 101. delta = -(101 - 96) = -5.
        let r = compute(&bars);
        assert_eq!(r[1].unwrap(), -5.0);
    }

    #[test]
    fn cumulative_sum_correct_over_multiple_bars() {
        let bars = vec![
            b(101.0, 99.0, 100.0),
            b(103.0, 99.0, 102.0), // +3
            b(101.0, 95.0, 96.0),  // -(102 - 96) = -6 → cum -3
            b(99.0, 95.0, 98.0),   // up close: trl = min(95, 96) = 95; +(98-95)=3 → cum 0
        ];
        let r = compute(&bars);
        assert_eq!(r[1].unwrap(), 3.0);
        assert_eq!(r[2].unwrap(), -3.0);
        assert_eq!(r[3].unwrap(), 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let bars: Vec<_> = (0..30)
            .map(|i| {
                let m = 100.0 + (i as f64).sin();
                b(m + 1.0, m - 1.0, m)
            })
            .collect();
        assert_eq!(compute(&bars).len(), 30);
    }
}
