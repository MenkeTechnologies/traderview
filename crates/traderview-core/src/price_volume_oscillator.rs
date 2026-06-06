//! Price-Volume Oscillator (PVO) — Klein-style oscillator that uses
//! volume × close (notional dollar flow) as the input series instead
//! of price alone.
//!
//!   pv_t = close_t · volume_t              (notional dollar flow)
//!   fast = EMA(pv, fast_period)
//!   slow = EMA(pv, slow_period)
//!   pvo = (fast - slow) / slow · 100        (% deviation)
//!   signal = EMA(pvo, signal_period)
//!   histogram = pvo - signal
//!
//! Different from `klinger_volume_oscillator` (KVO, which uses signed
//! volume force) and `volume_oscillator` (raw volume only). PVO
//! captures big-dollar bursts (price × size both rising) that pure
//! volume can miss.
//!
//! Pure compute. Defaults: 12 / 26 / 9.
//! Companion to `klinger_volume_oscillator`, `volume_oscillator`,
//! `volume_weighted_macd`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PvoReport {
    pub pvo: Vec<Option<f64>>,
    pub signal: Vec<Option<f64>>,
    pub histogram: Vec<Option<f64>>,
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

pub fn compute(
    bars: &[Bar],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> PvoReport {
    let n = bars.len();
    let mut report = PvoReport {
        pvo: vec![None; n],
        signal: vec![None; n],
        histogram: vec![None; n],
        fast_period,
        slow_period,
        signal_period,
    };
    if fast_period < 2
        || slow_period < 2
        || signal_period < 2
        || fast_period >= slow_period
        || n < slow_period + signal_period
    {
        return report;
    }
    if bars
        .iter()
        .any(|b| !b.close.is_finite() || !b.volume.is_finite() || b.volume < 0.0)
    {
        return report;
    }
    let pv: Vec<f64> = bars.iter().map(|b| b.close * b.volume).collect();
    let fast = ema(&pv, fast_period);
    let slow = ema(&pv, slow_period);
    for i in 0..n {
        if let (Some(f), Some(s)) = (fast[i], slow[i]) {
            if s != 0.0 {
                report.pvo[i] = Some((f - s) / s * 100.0);
            }
        }
    }
    report.signal = ema_opt(&report.pvo, signal_period);
    for i in 0..n {
        if let (Some(p), Some(s)) = (report.pvo[i], report.signal[i]) {
            report.histogram[i] = Some(p - s);
        }
    }
    report
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
    for (i, &v) in series.iter().enumerate().skip(period) {
        cur = v * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

fn ema_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 {
        return out;
    }
    let mut seed_end = None;
    let mut seed_sum = 0.0_f64;
    let mut count = 0_usize;
    for (i, v) in series.iter().enumerate() {
        match v {
            Some(x) => {
                seed_sum += x;
                count += 1;
            }
            None => {
                seed_sum = 0.0;
                count = 0;
            }
        }
        if count == period {
            seed_end = Some(i);
            break;
        }
    }
    let Some(end) = seed_end else {
        return out;
    };
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let mut cur = seed_sum / p_f;
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

    fn b(c: f64, v: f64) -> Bar {
        Bar {
            close: c,
            volume: v,
        }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(100.0, 1000.0); 50];
        assert!(compute(&bars, 1, 26, 9).pvo.iter().all(|x| x.is_none()));
        assert!(compute(&bars, 26, 12, 9).pvo.iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 12, 26, 9)
            .pvo
            .iter()
            .all(|x| x.is_none()));
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut bars = vec![b(100.0, 1000.0); 50];
        bars[5] = b(f64::NAN, 1000.0);
        assert!(compute(&bars, 12, 26, 9).pvo.iter().all(|x| x.is_none()));
    }

    #[test]
    fn constant_pv_yields_zero_pvo() {
        let bars = vec![b(100.0, 1000.0); 80];
        let r = compute(&bars, 12, 26, 9);
        for v in r.pvo.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn surging_pv_yields_positive_pvo() {
        let mut bars = vec![b(100.0, 1000.0); 40];
        bars.extend(vec![b(100.0, 5000.0); 40]);
        let r = compute(&bars, 12, 26, 9);
        let last = 79;
        assert!(r.pvo[last].unwrap() > 0.0);
    }

    #[test]
    fn collapsing_pv_yields_negative_pvo() {
        let mut bars = vec![b(100.0, 5000.0); 40];
        bars.extend(vec![b(100.0, 1000.0); 40]);
        let r = compute(&bars, 12, 26, 9);
        let last = 79;
        assert!(r.pvo[last].unwrap() < 0.0);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(100.0, 1000.0); 50];
        let r = compute(&bars, 12, 26, 9);
        assert_eq!(r.pvo.len(), 50);
        assert_eq!(r.signal.len(), 50);
        assert_eq!(r.histogram.len(), 50);
    }
}
