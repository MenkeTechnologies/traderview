//! Volume-Weighted MACD — MACD computed on volume-weighted price.
//!
//!   vwema_fast = volume-weighted EMA of close with fast_period
//!   vwema_slow = volume-weighted EMA of close with slow_period
//!   vw_macd    = vwema_fast - vwema_slow
//!   signal     = EMA(vw_macd, signal_period)
//!   histogram  = vw_macd - signal
//!
//! Volume weighting suppresses MACD signals on light-volume bars,
//! reducing whipsaws in low-liquidity regimes.
//!
//! Pure compute. Defaults: 12 / 26 / 9. Companion to standard MACD,
//! `klinger_volume_oscillator`, `vwema`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub close: f64, pub volume: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolumeWeightedMacdReport {
    pub vw_macd: Vec<Option<f64>>,
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
) -> VolumeWeightedMacdReport {
    let n = bars.len();
    let mut report = VolumeWeightedMacdReport {
        vw_macd: vec![None; n],
        signal: vec![None; n],
        histogram: vec![None; n],
        fast_period,
        slow_period,
        signal_period,
    };
    if fast_period < 2 || slow_period < 2 || signal_period < 2
        || fast_period >= slow_period
        || n < slow_period + signal_period { return report; }
    if bars.iter().any(|b| !b.close.is_finite() || !b.volume.is_finite() || b.volume < 0.0) {
        return report;
    }
    let fast = vw_ema(bars, fast_period);
    let slow = vw_ema(bars, slow_period);
    for i in 0..n {
        if let (Some(f), Some(s)) = (fast[i], slow[i]) {
            report.vw_macd[i] = Some(f - s);
        }
    }
    report.signal = ema_opt(&report.vw_macd, signal_period);
    for i in 0..n {
        if let (Some(m), Some(s)) = (report.vw_macd[i], report.signal[i]) {
            report.histogram[i] = Some(m - s);
        }
    }
    report
}

/// Volume-weighted EMA: weighting factor scaled by volume vs average volume
/// in the window.
fn vw_ema(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed_avg_vol: f64 = bars[..period].iter().map(|b| b.volume).sum::<f64>() / p_f;
    let seed_vw_close: f64 = if seed_avg_vol > 0.0 {
        bars[..period].iter().map(|b| b.close * b.volume).sum::<f64>()
            / (seed_avg_vol * p_f)
    } else {
        bars[..period].iter().map(|b| b.close).sum::<f64>() / p_f
    };
    out[period - 1] = Some(seed_vw_close);
    let mut cur = seed_vw_close;
    let mut avg_vol = seed_avg_vol;
    for (i, bar) in bars.iter().enumerate().skip(period) {
        // Update rolling avg volume.
        avg_vol = bar.volume * k + avg_vol * (1.0 - k);
        let weight = if avg_vol > 0.0 { (bar.volume / avg_vol).min(3.0) } else { 1.0 };
        let effective_k = (k * weight).min(1.0);
        cur = bar.close * effective_k + cur * (1.0 - effective_k);
        out[i] = Some(cur);
    }
    out
}

fn ema_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 { return out; }
    let mut seed_end = None;
    let mut seed_sum = 0.0_f64;
    let mut count = 0_usize;
    for (i, v) in series.iter().enumerate() {
        match v {
            Some(x) => { seed_sum += x; count += 1; }
            None => { seed_sum = 0.0; count = 0; }
        }
        if count == period { seed_end = Some(i); break; }
    }
    let Some(end) = seed_end else { return out; };
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

    fn b(c: f64, v: f64) -> Bar { Bar { close: c, volume: v } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(100.0, 1000.0); 50];
        assert!(compute(&bars, 1, 26, 9).vw_macd.iter().all(|x| x.is_none()));
        assert!(compute(&bars, 26, 12, 9).vw_macd.iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 12, 26, 9).vw_macd.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_or_negative_volume_returns_empty() {
        let mut bars = vec![b(100.0, 1000.0); 50];
        bars[5] = b(f64::NAN, 1000.0);
        assert!(compute(&bars, 12, 26, 9).vw_macd.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_macd() {
        let bars = vec![b(100.0, 1000.0); 80];
        let r = compute(&bars, 12, 26, 9);
        for v in r.vw_macd.iter().flatten() { assert!(v.abs() < 1e-9); }
    }

    #[test]
    fn uptrend_yields_positive_macd() {
        let bars: Vec<_> = (0..80).map(|i| b(100.0 + i as f64, 1000.0)).collect();
        let r = compute(&bars, 12, 26, 9);
        let last = 79;
        assert!(r.vw_macd[last].unwrap() > 0.0);
    }

    #[test]
    fn downtrend_yields_negative_macd() {
        let bars: Vec<_> = (0..80).map(|i| b(200.0 - i as f64, 1000.0)).collect();
        let r = compute(&bars, 12, 26, 9);
        let last = 79;
        assert!(r.vw_macd[last].unwrap() < 0.0);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(100.0, 1000.0); 50];
        let r = compute(&bars, 12, 26, 9);
        assert_eq!(r.vw_macd.len(), 50);
        assert_eq!(r.signal.len(), 50);
        assert_eq!(r.histogram.len(), 50);
    }
}
