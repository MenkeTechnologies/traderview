//! Stochastic RSI — Tushar Chande & Stanley Kroll (1994).
//!
//! Apply the stochastic oscillator FORMULA to the RSI series instead of
//! to price directly. The result is a faster, more sensitive oscillator
//! than plain RSI (the canonical "is RSI itself in its own
//! overbought/oversold range" question).
//!
//!   RSI_t  = standard RSI(close, rsi_period)
//!   K_t    = (RSI_t − min(RSI, stoch_period)) / (max − min) × 100
//!   %K     = `smooth_k`-period SMA of K_t
//!   %D     = `smooth_d`-period SMA of %K
//!
//! Convention: >80 overbought, <20 oversold. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StochRsiReport {
    pub raw: Vec<Option<f64>>,
    pub k: Vec<Option<f64>>,
    pub d: Vec<Option<f64>>,
}

pub fn compute(
    closes: &[f64],
    rsi_period: usize,
    stoch_period: usize,
    smooth_k: usize,
    smooth_d: usize,
) -> StochRsiReport {
    let n = closes.len();
    let mut report = StochRsiReport {
        raw: vec![None; n],
        k: vec![None; n],
        d: vec![None; n],
    };
    if rsi_period == 0 || stoch_period == 0 || smooth_k == 0 || smooth_d == 0 || n <= rsi_period {
        return report;
    }
    let rsi_series = rsi(closes, rsi_period);
    // raw StochRSI: needs `stoch_period` consecutive Some RSI values.
    for i in 0..n {
        if i + 1 < stoch_period {
            continue;
        }
        let window = &rsi_series[i + 1 - stoch_period..=i];
        if !window.iter().all(|x| x.is_some()) {
            continue;
        }
        let mut mn = f64::INFINITY;
        let mut mx = f64::NEG_INFINITY;
        for v in window.iter().filter_map(|x| *x) {
            mn = mn.min(v);
            mx = mx.max(v);
        }
        let range = mx - mn;
        let cur = rsi_series[i].unwrap();
        report.raw[i] = Some(if range > 0.0 {
            (cur - mn) / range * 100.0
        } else {
            50.0
        });
    }
    // %K = SMA(raw, smooth_k).
    sma_optional_into(&report.raw, smooth_k, &mut report.k);
    // %D = SMA(%K, smooth_d).
    sma_optional_into(&report.k, smooth_d, &mut report.d);
    report
}

fn rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n <= period {
        return out;
    }
    let mut gain = 0.0;
    let mut loss = 0.0;
    for i in 1..=period {
        let d = closes[i] - closes[i - 1];
        if d >= 0.0 {
            gain += d;
        } else {
            loss -= d;
        }
    }
    gain /= period as f64;
    loss /= period as f64;
    out[period] = Some(rsi_from(gain, loss));
    for i in (period + 1)..n {
        let d = closes[i] - closes[i - 1];
        let (g, l) = if d >= 0.0 { (d, 0.0) } else { (0.0, -d) };
        gain = (gain * (period as f64 - 1.0) + g) / period as f64;
        loss = (loss * (period as f64 - 1.0) + l) / period as f64;
        out[i] = Some(rsi_from(gain, loss));
    }
    out
}

fn rsi_from(gain: f64, loss: f64) -> f64 {
    if loss == 0.0 {
        return 100.0;
    }
    let rs = gain / loss;
    100.0 - 100.0 / (1.0 + rs)
}

fn sma_optional_into(src: &[Option<f64>], period: usize, dst: &mut [Option<f64>]) {
    let n = src.len();
    if period == 0 || n < period {
        return;
    }
    for i in (period - 1)..n {
        let window = &src[i + 1 - period..=i];
        if let Some(sum) = window.iter().try_fold(0.0_f64, |s, x| x.map(|v| s + v)) {
            dst[i] = Some(sum / period as f64);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_default() {
        let r = compute(&[], 14, 14, 3, 3);
        assert!(r.raw.is_empty());
    }

    #[test]
    fn zero_period_returns_all_none() {
        let v = vec![1.0; 50];
        let r = compute(&v, 0, 14, 3, 3);
        assert!(r.raw.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_raw_falls_back_to_50() {
        // Flat closes → RSI is undefined / constant → range=0 → fallback 50.
        let v = vec![100.0; 60];
        let r = compute(&v, 14, 14, 3, 3);
        let last = r.raw.last().copied().flatten().expect("populated");
        // RSI of flat is 100 (loss==0). Then range across StochRSI window is 0
        // → fallback 50.
        assert!((last - 50.0).abs() < 1e-9);
    }

    #[test]
    fn rising_series_after_chop_yields_high_stoch_rsi() {
        // A pure monotonic rise pins RSI at 100 for the whole window →
        // StochRSI window range = 0 → fallback 50. To exercise the
        // "overbought" path the RSI itself needs to RISE within the
        // StochRSI window, so start with chop then ramp up.
        let mut v: Vec<f64> = (0..30)
            .map(|i| if i % 2 == 0 { 100.0 } else { 101.0 })
            .collect();
        v.extend((1..=30).map(|i| 100.5 + i as f64));
        let r = compute(&v, 14, 14, 3, 3);
        let last = r.k.last().copied().flatten().expect("populated");
        assert!(
            last > 70.0,
            "post-chop rising series should be overbought, got {last}"
        );
    }

    #[test]
    fn monotonic_rise_pins_at_50_fallback() {
        // Document the corner: RSI saturated at 100 → StochRSI window has
        // zero range → fallback to 50.
        let v: Vec<f64> = (1..=60).map(|i| 100.0 + i as f64).collect();
        let r = compute(&v, 14, 14, 3, 3);
        let last = r.k.last().copied().flatten().expect("populated");
        assert!(
            (last - 50.0).abs() < 1.0,
            "monotonic rise → RSI saturates at 100 → StochRSI fallback ≈ 50, got {last}"
        );
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![1.0; 5];
        let r = compute(&v, usize::MAX, 14, 3, 3);
        assert!(r.raw.iter().all(|x| x.is_none()));
    }
}
