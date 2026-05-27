//! Volume-Weighted MACD (VWMACD).
//!
//!   typical_v = (high + low + close) / 3 × volume
//!   sum_v = N-period sum of volume
//!   vwma = N-period sum(typical_v) / sum_v
//!
//! Then MACD-style:
//!   line   = vwma_short - vwma_long
//!   signal = EMA(line, 9)
//!   hist   = line - signal
//!
//! Captures price + volume confirmation in one signal — diverges from
//! price MACD when participation matters. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64, pub low: f64, pub close: f64, pub volume: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct VwmacdPoint {
    pub line: f64,
    pub signal: f64,
    pub histogram: f64,
}

fn vwma_series(bars: &[Bar], period: usize) -> Vec<f64> {
    let n = bars.len();
    let mut out = vec![0.0; n];
    if n < period || period == 0 { return out; }
    let typical: Vec<f64> = bars.iter().map(|b| (b.high + b.low + b.close) / 3.0).collect();
    for i in (period - 1)..n {
        let w_v: f64 = bars[(i + 1 - period)..=i].iter().map(|b| b.volume).sum();
        let w_pv: f64 = bars[(i + 1 - period)..=i].iter().enumerate()
            .map(|(j, b)| typical[i + 1 - period + j] * b.volume).sum();
        out[i] = if w_v > 0.0 { w_pv / w_v } else { 0.0 };
    }
    out
}

fn ema(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut out = vec![0.0; n];
    if n == 0 || period == 0 { return out; }
    let k = 2.0 / (period as f64 + 1.0);
    let mut prev = values[0];
    out[0] = prev;
    for i in 1..n {
        let e = k * values[i] + (1.0 - k) * prev;
        out[i] = e;
        prev = e;
    }
    out
}

pub fn compute(bars: &[Bar], short: usize, long: usize, signal_period: usize)
    -> Vec<VwmacdPoint>
{
    let n = bars.len();
    let mut out = vec![VwmacdPoint::default(); n];
    if n < long { return out; }
    let s_vwma = vwma_series(bars, short);
    let l_vwma = vwma_series(bars, long);
    let lines: Vec<f64> = s_vwma.iter().zip(&l_vwma).map(|(a, b)| a - b).collect();
    let signal = ema(&lines, signal_period);
    for i in 0..n {
        let line = lines[i];
        let sig = signal[i];
        out[i] = VwmacdPoint {
            line, signal: sig, histogram: line - sig,
        };
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
    fn empty_returns_empty() {
        assert!(compute(&[], 12, 26, 9).is_empty());
    }

    #[test]
    fn under_long_period_returns_zeros() {
        let bars = vec![b(10.0, 9.0, 9.5, 100.0); 10];
        let out = compute(&bars, 12, 26, 9);
        for p in &out { assert_eq!(p.line, 0.0); }
    }

    #[test]
    fn strong_uptrend_macd_line_positive() {
        let bars: Vec<Bar> = (1..=40).map(|i| {
            let c = 100.0 + i as f64;
            b(c + 0.5, c - 0.5, c, 1000.0)
        }).collect();
        let out = compute(&bars, 12, 26, 9);
        assert!(out[39].line > 0.0);
    }

    #[test]
    fn strong_downtrend_macd_line_negative() {
        let bars: Vec<Bar> = (1..=40).map(|i| {
            let c = 200.0 - i as f64;
            b(c + 0.5, c - 0.5, c, 1000.0)
        }).collect();
        let out = compute(&bars, 12, 26, 9);
        assert!(out[39].line < 0.0);
    }

    #[test]
    fn histogram_is_line_minus_signal() {
        let bars: Vec<Bar> = (1..=40).map(|i| {
            let c = 100.0 + i as f64;
            b(c + 0.5, c - 0.5, c, 1000.0)
        }).collect();
        let out = compute(&bars, 12, 26, 9);
        for p in &out[26..] {
            assert!((p.histogram - (p.line - p.signal)).abs() < 1e-9);
        }
    }

    #[test]
    fn zero_volume_bars_handled_gracefully() {
        let bars: Vec<Bar> = (1..=30).map(|i| b(i as f64, 0.0, i as f64, 0.0)).collect();
        let out = compute(&bars, 12, 26, 9);
        // No panic; values may be 0.
        assert_eq!(out[29].line, 0.0);
    }
}
