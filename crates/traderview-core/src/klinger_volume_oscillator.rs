//! Klinger Volume Oscillator (KVO) — Stephen Klinger (1997).
//!
//! Combines price action and volume to flag both short- and long-term
//! money flow. Two-EMA difference on Klinger's "volume force":
//!
//!   trend_t = +1 if (H+L+C)_t > (H+L+C)_{t−1} else −1
//!   dm_t    = H_t − L_t                                          (daily range)
//!   cm_t    = if trend_t == trend_{t−1}: cm_{t−1} + dm_t
//!             else: dm_{t−1} + dm_t                              (cumulative range)
//!   VF_t    = volume_t · trend_t · |2·dm_t/cm_t − 1| · 100       (volume force)
//!   KVO_t   = EMA(VF, fast) − EMA(VF, slow)
//!
//! Default: fast = 34, slow = 55. Signal line is 13-period EMA of KVO.
//!
//! Interpretation: KVO crossing zero or crossing its signal indicates
//! a change in money-flow regime. Strongest signals come from
//! divergence between KVO and price extremes.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KvoReport {
    pub volume_force: Vec<Option<f64>>,
    pub kvo: Vec<Option<f64>>,
    pub signal: Vec<Option<f64>>,
}

pub fn compute(
    bars: &[Bar],
    fast: usize,
    slow: usize,
    signal_period: usize,
) -> KvoReport {
    let n = bars.len();
    let mut vf = vec![None; n];
    let mut kvo = vec![None; n];
    let mut signal = vec![None; n];
    if n < 2 || fast == 0 || slow == 0 || signal_period == 0 || fast >= slow {
        return KvoReport { volume_force: vf, kvo, signal };
    }
    let typical: Vec<f64> = bars.iter().map(|b| (b.high + b.low + b.close) / 3.0).collect();
    let dm: Vec<f64> = bars.iter().map(|b| b.high - b.low).collect();
    // Build trend, cm, vf series in one pass.
    let mut trend = vec![0_i32; n];
    let mut cm = vec![0.0_f64; n];
    for i in 1..n {
        let t = if typical[i] > typical[i - 1] { 1 } else { -1 };
        trend[i] = t;
        cm[i] = if trend[i] == trend[i - 1] { cm[i - 1] + dm[i] } else { dm[i - 1] + dm[i] };
        let denom = cm[i];
        if denom > 0.0 && dm[i].is_finite() && bars[i].volume.is_finite() {
            let inner = (2.0 * dm[i] / denom - 1.0).abs();
            vf[i] = Some(bars[i].volume * t as f64 * inner * 100.0);
        }
    }
    let fast_ema = ema(&vf, fast);
    let slow_ema = ema(&vf, slow);
    for i in 0..n {
        if let (Some(f), Some(s)) = (fast_ema[i], slow_ema[i]) {
            kvo[i] = Some(f - s);
        }
    }
    signal[..n].copy_from_slice(&ema(&kvo, signal_period)[..n]);
    KvoReport { volume_force: vf, kvo, signal }
}

fn ema(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n == 0 { return out; }
    // Find first contiguous window of `period` Some values for seed.
    let mut seed_end = None;
    let mut seed_sum = 0.0;
    let mut count = 0_usize;
    for (i, v) in series.iter().enumerate() {
        match v {
            Some(x) => { seed_sum += x; count += 1; }
            None => { seed_sum = 0.0; count = 0; }
        }
        if count == period { seed_end = Some(i); break; }
    }
    let Some(end) = seed_end else { return out; };
    let k = 2.0 / (period as f64 + 1.0);
    let mut cur = seed_sum / period as f64;
    out[end] = Some(cur);
    for i in (end + 1)..n {
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
    fn empty_returns_empty_outputs() {
        let r = compute(&[], 34, 55, 13);
        assert!(r.volume_force.is_empty());
    }

    #[test]
    fn invalid_params_return_all_none() {
        let bars: Vec<_> = (0..100).map(|i| b(101.0 + i as f64 * 0.1,
            99.0 + i as f64 * 0.1, 100.0 + i as f64 * 0.1, 1000.0)).collect();
        let r = compute(&bars, 0, 55, 13);
        assert!(r.kvo.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 55, 34, 13);    // fast >= slow
        assert!(r2.kvo.iter().all(|x| x.is_none()));
    }

    #[test]
    fn shorter_than_slow_returns_all_none_kvo() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        let r = compute(&bars, 34, 55, 13);
        assert!(r.kvo.iter().all(|x| x.is_none()));
    }

    #[test]
    fn sustained_uptrend_yields_positive_volume_force() {
        let bars: Vec<_> = (0..100).map(|i| b(101.0 + i as f64,
            99.0 + i as f64, 100.0 + i as f64, 1000.0)).collect();
        let r = compute(&bars, 34, 55, 13);
        let vf_last = r.volume_force[99].unwrap();
        assert!(vf_last > 0.0, "uptrend should yield positive VF, got {vf_last}");
    }

    #[test]
    fn sustained_downtrend_yields_negative_volume_force() {
        let bars: Vec<_> = (0..100).map(|i| b(101.0 - i as f64,
            99.0 - i as f64, 100.0 - i as f64, 1000.0)).collect();
        let r = compute(&bars, 34, 55, 13);
        let vf_last = r.volume_force[99].unwrap();
        assert!(vf_last < 0.0, "downtrend should yield negative VF, got {vf_last}");
    }

    #[test]
    fn signal_line_lags_kvo() {
        let bars: Vec<_> = (0..150).map(|i| {
            let mid = 100.0 + (i as f64 * 0.1).sin() * 5.0;
            b(mid + 1.0, mid - 1.0, mid, 1000.0)
        }).collect();
        let r = compute(&bars, 34, 55, 13);
        // Signal line should be defined where KVO is and has had time to seed.
        let kvo_defined: Vec<usize> = (0..150).filter(|i| r.kvo[*i].is_some()).collect();
        let signal_defined: Vec<usize> = (0..150).filter(|i| r.signal[*i].is_some()).collect();
        assert!(!kvo_defined.is_empty());
        assert!(!signal_defined.is_empty());
        assert!(signal_defined[0] >= kvo_defined[0]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 100];
        let r = compute(&bars, 34, 55, 13);
        assert_eq!(r.volume_force.len(), 100);
        assert_eq!(r.kvo.len(), 100);
        assert_eq!(r.signal.len(), 100);
    }
}
