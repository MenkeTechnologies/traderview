//! Price Volume Trend (PVT) — Joe Granville (variant of OBV).
//!
//! Cumulative volume-weighted % price change:
//!
//!   PVT_t = PVT_{t-1} + volume_t · (close_t - close_{t-1}) / close_{t-1}
//!   PVT_0 = 0
//!
//! Unlike On-Balance Volume (which adds raw volume on up-days and
//! subtracts it on down-days), PVT scales each contribution by the
//! magnitude of the percent move. Lighter contribution from small
//! moves, larger contribution from big moves.
//!
//! Interpretation:
//!   - Rising PVT confirms uptrend; falling confirms downtrend
//!   - Divergence between price highs/lows and PVT highs/lows is the
//!     primary signal
//!
//! Pure compute. Companion to `on_balance_volume`, `accumulation_distribution_line`,
//! `chaikin_oscillator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub close: f64,
    pub volume: f64,
}

pub fn compute(bars: &[Bar]) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if n == 0 {
        return out;
    }
    if bars
        .iter()
        .any(|b| !b.close.is_finite() || !b.volume.is_finite())
    {
        return out;
    }
    let mut pvt = 0.0_f64;
    out[0] = Some(pvt);
    for i in 1..n {
        let prev = bars[i - 1].close;
        if prev != 0.0 {
            pvt += bars[i].volume * (bars[i].close - prev) / prev;
        }
        out[i] = Some(pvt);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(c: f64, v: f64) -> Bar {
        Bar {
            close: c,
            volume: v,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[]).is_empty());
    }

    #[test]
    fn nan_returns_all_none() {
        let bars = vec![b(100.0, 1000.0), b(f64::NAN, 1000.0)];
        assert!(compute(&bars).iter().all(|x| x.is_none()));
    }

    #[test]
    fn first_bar_is_zero() {
        let bars = vec![b(100.0, 1000.0), b(101.0, 1000.0)];
        let r = compute(&bars);
        assert!((r[0].unwrap() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn up_move_adds_positive_pvt() {
        // close 100 → 110, vol 1000. PVT step = 1000 · 0.10 = 100.
        let bars = vec![b(100.0, 1000.0), b(110.0, 1000.0)];
        let r = compute(&bars);
        assert!((r[1].unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn down_move_subtracts() {
        let bars = vec![b(100.0, 1000.0), b(90.0, 1000.0)];
        let r = compute(&bars);
        assert!((r[1].unwrap() + 100.0).abs() < 1e-9);
    }

    #[test]
    fn flat_market_yields_constant_pvt() {
        let bars = vec![b(100.0, 1000.0); 10];
        let r = compute(&bars);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(100.0, 1000.0); 30];
        assert_eq!(compute(&bars).len(), 30);
    }

    #[test]
    fn cumulative_correct_three_bar() {
        let bars = vec![
            b(100.0, 1000.0),
            b(110.0, 2000.0), // +100
            b(99.0, 1000.0),  // 1000·(-11/110) = -100
        ];
        let r = compute(&bars);
        assert!((r[1].unwrap() - 200.0).abs() < 1e-9); // 0 + 2000·0.1
        assert!((r[2].unwrap() - 100.0).abs() < 1e-9); // 200 + 1000·(-0.1)
    }
}
