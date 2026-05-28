//! Accumulation/Distribution Oscillator — per-bar signed volume-weighted CLV.
//!
//! Distinct from `accumulation_distribution_line` (cumulative) and
//! `chaikin_oscillator` (MACD-style A/D EMA difference): this module
//! exposes the *per-bar* contribution value plus an EMA-smoothed
//! version, useful as a "current buying pressure" oscillator rather
//! than a running total.
//!
//!   clv_t = ((close - low) - (high - close)) / (high - low)
//!   ad_per_bar_t = clv_t · volume_t
//!   ad_osc_t     = EMA(ad_per_bar, period)
//!
//! Pure compute. Default period = 14.
//! Companion to `accumulation_distribution_line`, `chaikin_oscillator`,
//! `chaikin_money_flow`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64, pub volume: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdOscillatorReport {
    pub per_bar: Vec<Option<f64>>,
    pub ema: Vec<Option<f64>>,
    pub period: usize,
}

pub fn compute(bars: &[Bar], period: usize) -> AdOscillatorReport {
    let n = bars.len();
    let mut report = AdOscillatorReport {
        per_bar: vec![None; n],
        ema: vec![None; n],
        period,
    };
    if period < 2 || n < period { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()
        || !b.close.is_finite() || !b.volume.is_finite() || b.volume < 0.0) {
        return report;
    }
    let mut raw = vec![0.0_f64; n];
    for (i, bar) in bars.iter().enumerate() {
        let range = bar.high - bar.low;
        let per = if range > 0.0 {
            ((bar.close - bar.low) - (bar.high - bar.close)) / range * bar.volume
        } else {
            0.0
        };
        raw[i] = per;
        report.per_bar[i] = Some(per);
    }
    // EMA over raw.
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = raw[..period].iter().sum::<f64>() / p_f;
    report.ema[period - 1] = Some(seed);
    let mut cur = seed;
    for (i, &v) in raw.iter().enumerate().skip(period) {
        cur = v * k + cur * (1.0 - k);
        report.ema[i] = Some(cur);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar { high: h, low: l, close: c, volume: v }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        let r = compute(&bars, 1);
        assert!(r.per_bar.iter().all(|x| x.is_none()));
        let r2 = compute(&bars[..5], 14);
        assert!(r2.per_bar.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0, 1000.0);
        let r = compute(&bars, 14);
        assert!(r.per_bar.iter().all(|x| x.is_none()));
        let mut bars2 = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        bars2[5] = b(101.0, 99.0, 100.0, -100.0);
        let r2 = compute(&bars2, 14);
        assert!(r2.per_bar.iter().all(|x| x.is_none()));
    }

    #[test]
    fn close_at_high_yields_positive_per_bar() {
        let bars = vec![b(110.0, 100.0, 110.0, 1000.0); 30];
        let r = compute(&bars, 14);
        for v in r.per_bar.iter().flatten() {
            assert!((v - 1000.0).abs() < 1e-9);
        }
    }

    #[test]
    fn close_at_low_yields_negative_per_bar() {
        let bars = vec![b(110.0, 100.0, 100.0, 1000.0); 30];
        let r = compute(&bars, 14);
        for v in r.per_bar.iter().flatten() {
            assert!((v + 1000.0).abs() < 1e-9);
        }
    }

    #[test]
    fn close_at_mid_yields_zero_per_bar() {
        let bars = vec![b(110.0, 100.0, 105.0, 1000.0); 30];
        let r = compute(&bars, 14);
        for v in r.per_bar.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn zero_range_bar_yields_zero_per_bar() {
        let bars = vec![b(100.0, 100.0, 100.0, 1000.0); 30];
        let r = compute(&bars, 14);
        for v in r.per_bar.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn ema_matches_steady_state_per_bar() {
        let bars = vec![b(110.0, 100.0, 110.0, 1000.0); 30];
        let r = compute(&bars, 14);
        let last = 29;
        assert!((r.ema[last].unwrap() - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        let r = compute(&bars, 14);
        assert_eq!(r.per_bar.len(), 30);
        assert_eq!(r.ema.len(), 30);
    }
}
