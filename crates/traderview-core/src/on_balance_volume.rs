//! On-Balance Volume (OBV) — Joseph Granville (1963).
//!
//! Cumulative running tally that adds volume on up-close bars and
//! subtracts volume on down-close bars:
//!
//!   OBV_t = OBV_{t−1} + sign(Close_t − Close_{t−1}) · Volume_t
//!
//! where sign(0) = 0 (unchanged closes contribute zero).
//!
//! Interpretation:
//!   - Rising OBV confirms uptrend (volume favors buyers)
//!   - Falling OBV confirms downtrend (volume favors sellers)
//!   - OBV/price divergence = potential reversal
//!
//! Distinct from ADL: OBV uses CLOSE-vs-prior-close sign; ADL uses
//! INTRABAR close-position within the H-L range.
//!
//! Pure compute. Companion to `accumulation_distribution_line`,
//! `volume_burst`, `chaikin_money_flow`.

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
    let mut obv = 0.0_f64;
    out[0] = Some(0.0);
    for i in 1..n {
        let cur = bars[i];
        let prev = bars[i - 1];
        if !cur.close.is_finite() || !prev.close.is_finite() || !cur.volume.is_finite() {
            out[i] = Some(obv);
            continue;
        }
        if cur.close > prev.close {
            obv += cur.volume;
        } else if cur.close < prev.close {
            obv -= cur.volume;
        }
        out[i] = Some(obv);
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
    fn single_bar_yields_zero() {
        let bars = vec![b(100.0, 1000.0)];
        let out = compute(&bars);
        assert_eq!(out[0].unwrap(), 0.0);
    }

    #[test]
    fn rising_closes_accumulate_positive_obv() {
        let bars: Vec<_> = (0..10).map(|i| b(100.0 + i as f64, 1000.0)).collect();
        let out = compute(&bars);
        let last = out[9].unwrap();
        // 9 up-bars × 1000 = 9000.
        assert_eq!(last, 9_000.0);
    }

    #[test]
    fn falling_closes_accumulate_negative_obv() {
        let bars: Vec<_> = (0..10).map(|i| b(100.0 - i as f64, 1000.0)).collect();
        let out = compute(&bars);
        let last = out[9].unwrap();
        assert_eq!(last, -9_000.0);
    }

    #[test]
    fn unchanged_close_contributes_zero() {
        let bars = vec![b(100.0, 1000.0), b(100.0, 5000.0), b(100.0, 2000.0)];
        let out = compute(&bars);
        assert_eq!(out[0].unwrap(), 0.0);
        assert_eq!(out[1].unwrap(), 0.0);
        assert_eq!(out[2].unwrap(), 0.0);
    }

    #[test]
    fn alternating_closes_cancel_with_equal_volume() {
        let bars = vec![
            b(100.0, 1000.0),
            b(101.0, 1000.0), // +1000
            b(100.0, 1000.0), // -1000
            b(101.0, 1000.0), // +1000
            b(100.0, 1000.0), // -1000
        ];
        let out = compute(&bars);
        assert_eq!(out[4].unwrap(), 0.0);
    }

    #[test]
    fn nan_close_carries_forward_obv() {
        let bars = vec![b(100.0, 1000.0), b(101.0, 1000.0), b(f64::NAN, 1000.0)];
        let out = compute(&bars);
        assert_eq!(out[1].unwrap(), 1000.0);
        assert_eq!(out[2].unwrap(), 1000.0); // NaN bar leaves OBV unchanged
    }

    #[test]
    fn output_length_matches_input() {
        let bars: Vec<_> = (0..50).map(|i| b(100.0 + i as f64 * 0.1, 1000.0)).collect();
        let out = compute(&bars);
        assert_eq!(out.len(), 50);
    }
}
