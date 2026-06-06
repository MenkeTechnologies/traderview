//! Volume Zone Oscillator (VZO) — Walid Khalil (Stocks & Commodities, 2009).
//!
//! Volume-direction momentum oscillator:
//!
//!   signed_vol_t = sign(close_t - close_{t-1}) · volume_t
//!   ema_vol_t    = EMA(signed_vol, period)
//!   ema_tot_t    = EMA(volume, period)
//!   VZO_t        = 100 · ema_vol_t / ema_tot_t
//!
//! Range: ±100. Khalil's zones:
//!
//!   VZO > +60  → overbought
//!   +5 .. +60 → bullish trend
//!   -5 .. +5  → neutral / consolidation
//!   -60 .. -5 → bearish trend
//!   VZO < -60 → oversold
//!
//! Pure compute. Default period = 14. Companion to `klinger_volume_oscillator`,
//! `chaikin_money_flow`, `on_balance_volume`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub close: f64,
    pub volume: f64,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || n < period + 1 {
        return out;
    }
    if bars
        .iter()
        .any(|b| !b.close.is_finite() || !b.volume.is_finite() || b.volume < 0.0)
    {
        return out;
    }
    let signed_vol: Vec<f64> = (0..n)
        .map(|i| {
            if i == 0 {
                0.0
            } else if bars[i].close > bars[i - 1].close {
                bars[i].volume
            } else if bars[i].close < bars[i - 1].close {
                -bars[i].volume
            } else {
                0.0
            }
        })
        .collect();
    let vols: Vec<f64> = bars.iter().map(|b| b.volume).collect();
    let ema_vol = ema(&signed_vol[1..], period);
    let ema_tot = ema(&vols[1..], period);
    for (k, slot) in out.iter_mut().enumerate().skip(1) {
        let idx = k - 1;
        if let (Some(v), Some(t)) = (ema_vol[idx], ema_tot[idx]) {
            if t > 0.0 {
                *slot = Some(100.0 * v / t);
            }
        }
    }
    out
}

fn ema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = series[i] * k + cur * (1.0 - k);
        out[i] = Some(cur);
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
    fn invalid_inputs_return_empty() {
        let bars = vec![b(100.0, 1000.0); 30];
        assert!(compute(&bars, 1).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut bars = vec![b(100.0, 1000.0); 30];
        bars[5] = b(f64::NAN, 1000.0);
        assert!(compute(&bars, 14).iter().all(|x| x.is_none()));
        let mut bars2 = vec![b(100.0, 1000.0); 30];
        bars2[5] = b(100.0, -1.0);
        assert!(compute(&bars2, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn pure_uptrend_yields_high_positive_vzo() {
        let bars: Vec<_> = (0..50).map(|i| b(100.0 + i as f64, 1000.0)).collect();
        let r = compute(&bars, 14);
        let last = r[49].unwrap();
        assert!(
            last > 90.0,
            "pure uptrend should yield VZO near +100, got {last}"
        );
    }

    #[test]
    fn pure_downtrend_yields_high_negative_vzo() {
        let bars: Vec<_> = (0..50).map(|i| b(200.0 - i as f64, 1000.0)).collect();
        let r = compute(&bars, 14);
        let last = r[49].unwrap();
        assert!(last < -90.0);
    }

    #[test]
    fn balanced_market_yields_near_zero_vzo() {
        // Alternating up/down with equal volume.
        let bars: Vec<_> = (0_usize..50)
            .map(|i| {
                let c = if i.is_multiple_of(2) { 100.0 } else { 101.0 };
                b(c, 1000.0)
            })
            .collect();
        let r = compute(&bars, 14);
        let last = r[49].unwrap();
        assert!(
            last.abs() < 20.0,
            "alternating closes should yield VZO near 0, got {last}"
        );
    }

    #[test]
    fn output_in_signed_hundred_range() {
        let mut state: u64 = 42;
        let bars: Vec<_> = (0..200)
            .map(|i| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
                b(100.0 + i as f64 * 0.1 + (r - 0.5) * 2.0, 1000.0)
            })
            .collect();
        let r = compute(&bars, 14);
        for v in r.iter().flatten() {
            assert!((-100.0..=100.0).contains(v));
        }
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(100.0, 1000.0); 30];
        assert_eq!(compute(&bars, 14).len(), 30);
    }
}
