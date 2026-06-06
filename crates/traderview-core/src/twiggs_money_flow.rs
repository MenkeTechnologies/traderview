//! Twiggs Money Flow (TMF) — Colin Twiggs.
//!
//! Modification of Chaikin Money Flow that uses true-range-aware
//! denominator (so single-bar gaps don't zero-out the contribution)
//! and an exponentially-weighted moving average instead of a simple
//! one:
//!
//!   tr_t = max(high_t, close_{t-1}) - min(low_t, close_{t-1})
//!   ad_t = ((close_t - min(low_t, close_{t-1})) - (max(high_t, close_{t-1}) - close_t))
//!          / tr_t · volume_t
//!     (= 2·close - tr_low - tr_high) / tr  ·  volume
//!   TMF_t = EMA(ad, N) / EMA(volume, N)
//!
//! Range [-1, +1]. > 0.20 = strong accumulation, < -0.20 = strong
//! distribution.
//!
//! Pure compute. Companion to `accumulation_distribution_line`,
//! `chaikin_oscillator`, `on_balance_volume`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || n < period + 1 {
        return out;
    }
    if bars.iter().any(|b| {
        !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite() || !b.volume.is_finite()
    }) {
        return out;
    }
    // Per-bar volume-weighted AD using true-range high/low.
    let mut ad = vec![0.0_f64; n];
    for i in 1..n {
        let tr_high = bars[i].high.max(bars[i - 1].close);
        let tr_low = bars[i].low.min(bars[i - 1].close);
        let tr = tr_high - tr_low;
        if tr > 0.0 {
            let clv = ((bars[i].close - tr_low) - (tr_high - bars[i].close)) / tr;
            ad[i] = clv * bars[i].volume;
        }
    }
    let ema_ad = wilder_ema(&ad[1..], period);
    let vols: Vec<f64> = bars[1..].iter().map(|b| b.volume).collect();
    let ema_vol = wilder_ema(&vols, period);
    for (k, slot) in out.iter_mut().enumerate().skip(1) {
        let idx = k - 1;
        if let (Some(a), Some(v)) = (ema_ad[idx], ema_vol[idx]) {
            if v > 0.0 {
                *slot = Some(a / v);
            }
        }
    }
    out
}

fn wilder_ema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let p_f = period as f64;
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = (cur * (p_f - 1.0) + series[i]) / p_f;
        out[i] = Some(cur);
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
    fn empty_or_invalid_returns_all_none() {
        assert!(compute(&[], 14).is_empty());
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        assert!(compute(&bars, 1).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0, 1000.0);
        assert!(compute(&bars, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn close_at_high_yields_positive_tmf() {
        // Closes at the top of every bar → strong accumulation.
        let bars = vec![b(101.0, 99.0, 101.0, 1000.0); 30];
        let r = compute(&bars, 14);
        let last = r[29].unwrap();
        assert!(
            last > 0.9,
            "close-at-high should yield TMF near +1, got {last}"
        );
    }

    #[test]
    fn close_at_low_yields_negative_tmf() {
        let bars = vec![b(101.0, 99.0, 99.0, 1000.0); 30];
        let r = compute(&bars, 14);
        let last = r[29].unwrap();
        assert!(last < -0.9);
    }

    #[test]
    fn close_at_mid_yields_zero_tmf() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        let r = compute(&bars, 14);
        for v in r.iter().skip(15).flatten() {
            assert!(
                v.abs() < 0.05,
                "close-at-mid should yield TMF near zero, got {v}"
            );
        }
    }

    #[test]
    fn output_in_unit_signed_range() {
        let mut state: u64 = 42;
        let bars: Vec<_> = (0..200)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let r1 = (state >> 32) as u32 as f64 / u32::MAX as f64;
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let r2 = (state >> 32) as u32 as f64 / u32::MAX as f64;
                let mid = 100.0 + (r1 - 0.5) * 4.0;
                let close_in_range = (mid - 1.0) + r2 * 2.0; // strictly within [low, high]
                b(mid + 1.0, mid - 1.0, close_in_range, 1000.0)
            })
            .collect();
        let r = compute(&bars, 14);
        for v in r.iter().flatten() {
            assert!((-1.0..=1.0).contains(v), "TMF out of [-1, 1]: {v}");
        }
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        assert_eq!(compute(&bars, 14).len(), 30);
    }
}
