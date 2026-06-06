//! Volume Force Index — Alexander Elder volume-force momentum
//! (signed price-change × volume) smoothed by an EMA.
//!
//! Distinct from `force_index` (which is the raw per-bar value).
//! This module reports both the raw per-bar series and an EMA-smoothed
//! version, plus the short (default 2-bar) and long (default 13-bar)
//! smoothings that Elder used:
//!
//!   raw_t   = (close_t - close_{t-1}) · volume_t
//!   short_t = EMA(raw, 2)
//!   long_t  = EMA(raw, 13)
//!
//! Short FI is for entry timing, long FI for trend confirmation.
//!
//! Pure compute. Companion to `force_index`, `elder_thermometer`,
//! `elder_safezone_stop`, `klinger_volume_oscillator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolumeForceIndexReport {
    pub raw: Vec<Option<f64>>,
    pub short_fi: Vec<Option<f64>>,
    pub long_fi: Vec<Option<f64>>,
    pub short_period: usize,
    pub long_period: usize,
}

pub fn compute(bars: &[Bar], short_period: usize, long_period: usize) -> VolumeForceIndexReport {
    let n = bars.len();
    let mut report = VolumeForceIndexReport {
        raw: vec![None; n],
        short_fi: vec![None; n],
        long_fi: vec![None; n],
        short_period,
        long_period,
    };
    if short_period < 2 || long_period < 2 || short_period >= long_period || n < long_period + 1 {
        return report;
    }
    if bars
        .iter()
        .any(|b| !b.close.is_finite() || !b.volume.is_finite() || b.volume < 0.0)
    {
        return report;
    }
    let mut raw = vec![0.0_f64; n];
    for i in 1..n {
        raw[i] = (bars[i].close - bars[i - 1].close) * bars[i].volume;
        report.raw[i] = Some(raw[i]);
    }
    report.short_fi = ema(&raw[1..], short_period, 1);
    report.long_fi = ema(&raw[1..], long_period, 1);
    report
}

fn ema(series: &[f64], period: usize, offset: usize) -> Vec<Option<f64>> {
    let n_input = series.len();
    let total = n_input + offset;
    let mut out = vec![None; total];
    if period == 0 || n_input < period {
        return out;
    }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1 + offset] = Some(seed);
    let mut cur = seed;
    for (i, &v) in series.iter().enumerate().skip(period) {
        cur = v * k + cur * (1.0 - k);
        out[i + offset] = Some(cur);
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
        let r = compute(&bars, 1, 13);
        assert!(r.raw.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 13, 2);
        assert!(r2.raw.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut bars = vec![b(100.0, 1000.0); 30];
        bars[5] = b(f64::NAN, 1000.0);
        let r = compute(&bars, 2, 13);
        assert!(r.raw.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_fi() {
        let bars = vec![b(100.0, 1000.0); 30];
        let r = compute(&bars, 2, 13);
        for v in r.long_fi.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn uptrend_yields_positive_long_fi() {
        let bars: Vec<_> = (0..30).map(|i| b(100.0 + i as f64, 1000.0)).collect();
        let r = compute(&bars, 2, 13);
        let last = 29;
        assert!(r.long_fi[last].unwrap() > 0.0);
    }

    #[test]
    fn downtrend_yields_negative_long_fi() {
        let bars: Vec<_> = (0..30).map(|i| b(200.0 - i as f64, 1000.0)).collect();
        let r = compute(&bars, 2, 13);
        let last = 29;
        assert!(r.long_fi[last].unwrap() < 0.0);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(100.0, 1000.0); 30];
        let r = compute(&bars, 2, 13);
        assert_eq!(r.raw.len(), 30);
        assert_eq!(r.short_fi.len(), 30);
        assert_eq!(r.long_fi.len(), 30);
    }
}
