//! PPO — Percentage Price Oscillator.
//!
//!   PPO = ((EMA_fast − EMA_slow) / EMA_slow) × 100
//!   Signal = EMA(PPO, signal_period)
//!   Histogram = PPO − Signal
//!
//! Same shape as MACD but expressed as a percentage of the slow EMA —
//! makes the indicator comparable across symbols with very different
//! price levels (a 1-point MACD spread on a $5 stock is huge; on a
//! $5000 stock it's nothing). Standard params: 12/26/9.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PpoReport {
    pub line: Vec<Option<f64>>,
    pub signal: Vec<Option<f64>>,
    pub histogram: Vec<Option<f64>>,
}

pub fn compute(
    closes: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> PpoReport {
    let n = closes.len();
    let mut report = PpoReport {
        line: vec![None; n],
        signal: vec![None; n],
        histogram: vec![None; n],
    };
    if fast_period == 0 || slow_period == 0 || signal_period == 0 {
        return report;
    }
    let fast = ema(closes, fast_period);
    let slow = ema(closes, slow_period);
    for i in 0..n {
        if let (Some(f), Some(s)) = (fast[i], slow[i]) {
            if s > 0.0 {
                let v = (f - s) / s * 100.0;
                if v.is_finite() {
                    report.line[i] = Some(v);
                }
            }
        }
    }
    let sig = ema_optional(&report.line, signal_period);
    report.signal = sig;
    for i in 0..n {
        if let (Some(l), Some(s)) = (report.line[i], report.signal[i]) {
            report.histogram[i] = Some(l - s);
        }
    }
    report
}

fn ema(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let seed: f64 = values[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = Some(seed);
    let mut prev = seed;
    for i in period..n {
        prev = alpha * values[i] + (1.0 - alpha) * prev;
        out[i] = Some(prev);
    }
    out
}

fn ema_optional(values: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let mut start: Option<usize> = None;
    let mut run = 0;
    for (i, v) in values.iter().enumerate() {
        if v.is_some() {
            run += 1;
            if run >= period {
                start = Some(i);
                break;
            }
        } else {
            run = 0;
        }
    }
    let _ = n;
    let Some(s) = start else { return out };
    let alpha = 2.0 / (period as f64 + 1.0);
    let seed: f64 = values[s + 1 - period..=s]
        .iter()
        .map(|x| x.unwrap())
        .sum::<f64>()
        / period as f64;
    out[s] = Some(seed);
    let mut prev = seed;
    for i in (s + 1)..n {
        if let Some(v) = values[i] {
            prev = alpha * v + (1.0 - alpha) * prev;
            out[i] = Some(prev);
        } else {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[], 12, 26, 9);
        assert!(r.line.is_empty());
    }

    #[test]
    fn zero_period_returns_all_none() {
        let v = vec![100.0; 50];
        for (f, s, sg) in [(0, 26, 9), (12, 0, 9), (12, 26, 0)] {
            let r = compute(&v, f, s, sg);
            assert!(r.line.iter().all(|x| x.is_none()), "({f},{s},{sg})");
        }
    }

    #[test]
    fn flat_series_ppo_zero() {
        let v = vec![100.0; 100];
        let r = compute(&v, 12, 26, 9);
        let last = r.line.last().copied().flatten().expect("populated");
        assert!(last.abs() < 1e-9);
    }

    #[test]
    fn rising_series_ppo_positive() {
        let v: Vec<f64> = (1..=100).map(|i| 100.0 + i as f64).collect();
        let r = compute(&v, 12, 26, 9);
        let last = r.line[99].expect("populated");
        assert!(last > 0.0, "rising → PPO > 0, got {last}");
    }

    #[test]
    fn falling_series_ppo_negative() {
        let v: Vec<f64> = (1..=100).map(|i| 200.0 - i as f64).collect();
        let r = compute(&v, 12, 26, 9);
        let last = r.line[99].expect("populated");
        assert!(last < 0.0);
    }

    #[test]
    fn histogram_equals_line_minus_signal() {
        let v: Vec<f64> = (1..=100).map(|i| 100.0 + (i as f64 * 0.5).sin()).collect();
        let r = compute(&v, 12, 26, 9);
        for i in 0..r.line.len() {
            if let (Some(l), Some(s), Some(h)) = (r.line[i], r.signal[i], r.histogram[i]) {
                assert!((h - (l - s)).abs() < 1e-12, "i={i}");
            }
        }
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![1.0; 5];
        let r = compute(&v, usize::MAX, 26, 9);
        assert!(r.line.iter().all(|x| x.is_none()));
    }
}
