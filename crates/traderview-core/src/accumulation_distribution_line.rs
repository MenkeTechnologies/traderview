//! Accumulation / Distribution Line — Marc Chaikin (1970s).
//!
//! Cumulative running sum of Money Flow Volume:
//!
//!   MFM_t = ((C − L) − (H − C)) / (H − L)       (Money Flow Multiplier)
//!   MFV_t = MFM_t · Volume_t
//!   ADL_t = ADL_{t−1} + MFV_t
//!
//! Interpretation:
//!   - Rising ADL = accumulation (buying pressure on closes near highs)
//!   - Falling ADL = distribution (selling pressure on closes near lows)
//!   - Divergence between ADL and price = potential reversal signal
//!
//! Distinct from CMF (which normalizes by rolling volume); ADL is
//! cumulative-unbounded and used primarily for trend confirmation.
//!
//! Bars where high == low contribute 0 MFV (avoid div-by-zero).
//!
//! Pure compute. Companion to `chaikin_money_flow`, `chaikin_oscillator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub fn compute(bars: &[Bar]) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if n == 0 {
        return out;
    }
    let mut adl = 0.0_f64;
    for i in 0..n {
        let b = bars[i];
        if !b.high.is_finite()
            || !b.low.is_finite()
            || !b.close.is_finite()
            || !b.volume.is_finite()
        {
            out[i] = Some(adl);
            continue;
        }
        let range = b.high - b.low;
        if range > 0.0 {
            let mfm = ((b.close - b.low) - (b.high - b.close)) / range;
            adl += mfm * b.volume;
        }
        out[i] = Some(adl);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
            volume: v,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[]).is_empty());
    }

    #[test]
    fn closes_at_high_accumulates_positively() {
        let bars = vec![b(101.0, 99.0, 101.0, 1000.0); 10];
        let out = compute(&bars);
        let last = out.last().unwrap().unwrap();
        // MFM = +1 each bar → ADL = 10 · 1000 = 10000.
        assert!((last - 10_000.0).abs() < 1e-9);
        // Monotonically increasing.
        for w in out.windows(2) {
            let prev = w[0].unwrap();
            let cur = w[1].unwrap();
            assert!(cur >= prev);
        }
    }

    #[test]
    fn closes_at_low_accumulates_negatively() {
        let bars = vec![b(101.0, 99.0, 99.0, 1000.0); 10];
        let out = compute(&bars);
        let last = out.last().unwrap().unwrap();
        assert!((last + 10_000.0).abs() < 1e-9);
    }

    #[test]
    fn midpoint_close_yields_zero_adl() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 10];
        let out = compute(&bars);
        let last = out.last().unwrap().unwrap();
        assert!(last.abs() < 1e-9);
    }

    #[test]
    fn zero_range_bar_contributes_nothing() {
        let mut bars = vec![b(101.0, 99.0, 101.0, 1000.0); 5];
        // ADL after bar 5 = 5*1000 = 5000.
        bars.push(b(100.0, 100.0, 100.0, 1000.0)); // doji
        let out = compute(&bars);
        assert!((out[5].unwrap() - 5_000.0).abs() < 1e-9);
    }

    #[test]
    fn nan_bar_carries_forward_previous_adl() {
        let mut bars = vec![b(101.0, 99.0, 101.0, 1000.0); 3];
        bars.push(b(f64::NAN, 99.0, 100.0, 1000.0));
        let out = compute(&bars);
        let prev = out[2].unwrap();
        let cur = out[3].unwrap();
        assert_eq!(prev, cur); // NaN bar should not change ADL.
    }

    #[test]
    fn output_length_matches_input() {
        let bars: Vec<_> = (0..50).map(|_| b(101.0, 99.0, 100.0, 1000.0)).collect();
        let out = compute(&bars);
        assert_eq!(out.len(), 50);
    }
}
