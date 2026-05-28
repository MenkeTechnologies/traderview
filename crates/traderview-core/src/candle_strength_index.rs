//! Candle Strength Index (CSI) — body-to-range ratio EMA.
//!
//! Per-bar signed ratio of close-to-open against the full bar range,
//! smoothed by an EMA:
//!
//!   range_t  = high - low
//!   csi_raw_t = (close - open) / range
//!   csi_t     = EMA(csi_raw, period)
//!
//! Range [-1, +1]:
//!   +1 → marubozu green (close at high after open at low)
//!   -1 → marubozu red
//!    0 → doji or balanced bars on average
//!
//! Used to gauge the average "strength" of candle bodies over the
//! recent N bars: persistent positive readings = sustained buying
//! commitment, negative = selling commitment.
//!
//! Pure compute. Default period = 14. Companion to `balance_of_power`,
//! `qstick`, `vsa`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || n < period { return out; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return out;
    }
    let raw: Vec<f64> = bars.iter().map(|b| {
        let r = b.high - b.low;
        if r > 0.0 { (b.close - b.open) / r } else { 0.0 }
    }).collect();
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = raw[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = raw[i] * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar { open: o, high: h, low: l, close: c }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        assert!(compute(&bars, 1).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        bars[5] = bar(f64::NAN, 101.0, 99.0, 100.5);
        assert!(compute(&bars, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn all_marubozu_green_yields_plus_one() {
        // Open=low, close=high → ratio = 1.0.
        let bars = vec![bar(100.0, 110.0, 100.0, 110.0); 30];
        let r = compute(&bars, 14);
        for v in r.iter().flatten() {
            assert!((v - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn all_marubozu_red_yields_minus_one() {
        let bars = vec![bar(110.0, 110.0, 100.0, 100.0); 30];
        let r = compute(&bars, 14);
        for v in r.iter().flatten() {
            assert!((v + 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn doji_bars_yield_zero() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 14);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn alternating_bars_average_to_zero() {
        let bars: Vec<_> = (0_usize..30).map(|i| {
            if i.is_multiple_of(2) { bar(100.0, 110.0, 100.0, 110.0) }
            else { bar(110.0, 110.0, 100.0, 100.0) }
        }).collect();
        let r = compute(&bars, 14);
        // After EMA settles, alternating ±1 averages near 0 (but EMA
        // weighting may bias slightly toward last value).
        let last = r[29].unwrap();
        assert!(last.abs() < 0.2);
    }

    #[test]
    fn output_in_unit_signed_range() {
        let mut state: u64 = 42;
        let bars: Vec<_> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            let m = 100.0 + (r - 0.5) * 4.0;
            bar(m, m + 1.0, m - 1.0, m + (r - 0.5) * 0.5)
        }).collect();
        let r = compute(&bars, 14);
        for v in r.iter().flatten() {
            assert!((-1.0..=1.0).contains(v),
                "CSI out of [-1, 1]: {v}");
        }
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        assert_eq!(compute(&bars, 14).len(), 30);
    }
}
