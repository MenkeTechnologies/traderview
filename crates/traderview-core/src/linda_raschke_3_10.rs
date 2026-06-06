//! 3-10 Oscillator — Linda Bradford Raschke ("Street Smarts", 1996).
//!
//! Pre-MACD oscillator from the floor-trader era:
//!
//!   fast_t   = SMA(close, 3) - SMA(close, 10)        (3-10 line / fast)
//!   signal_t = SMA(fast, 16)                          (slow / signal)
//!   hist_t   = fast_t - signal_t                       (histogram)
//!
//! Crossings of the fast line through zero confirm momentum direction;
//! divergences between price highs/lows and oscillator highs/lows are
//! Raschke's primary signal.
//!
//! Pure compute. Defaults: 3, 10, 16. Companion to `macd`, if shipped,
//! and `elliott_wave_oscillator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreeTenReport {
    pub fast: Vec<Option<f64>>,
    pub signal: Vec<Option<f64>>,
    pub histogram: Vec<Option<f64>>,
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

pub fn compute(
    closes: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> ThreeTenReport {
    let n = closes.len();
    let mut report = ThreeTenReport {
        fast: vec![None; n],
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
    if closes.iter().any(|x| !x.is_finite()) {
        return report;
    }
    let fast_sma = sma(closes, fast_period);
    let slow_sma = sma(closes, slow_period);
    for i in 0..n {
        if let (Some(f), Some(s)) = (fast_sma[i], slow_sma[i]) {
            report.fast[i] = Some(f - s);
        }
    }
    report.signal = sma_opt(&report.fast, signal_period);
    for i in 0..n {
        if let (Some(f), Some(s)) = (report.fast[i], report.signal[i]) {
            report.histogram[i] = Some(f - s);
        }
    }
    report
}

fn sma(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let p_f = period as f64;
    let mut sum: f64 = series[..period].iter().sum();
    out[period - 1] = Some(sum / p_f);
    for i in period..n {
        sum += series[i] - series[i - period];
        out[i] = Some(sum / p_f);
    }
    out
}

fn sma_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let p_f = period as f64;
    for i in (period - 1)..n {
        let win = &series[i + 1 - period..=i];
        if win.iter().any(|x| x.is_none()) {
            continue;
        }
        let s: f64 = win.iter().filter_map(|x| *x).sum();
        out[i] = Some(s / p_f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c, 1, 10, 16);
        assert!(r.fast.iter().all(|x| x.is_none()));
        let r2 = compute(&c, 10, 3, 16); // fast > slow
        assert!(r2.fast.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 100];
        c[5] = f64::NAN;
        let r = compute(&c, 3, 10, 16);
        assert!(r.fast.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_lines() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c, 3, 10, 16);
        for v in r.fast.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
        for v in r.signal.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
        for v in r.histogram.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn uptrend_fast_above_zero() {
        let c: Vec<f64> = (0..100).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 3, 10, 16);
        let last = r.fast[99].unwrap();
        assert!(last > 0.0);
    }

    #[test]
    fn downtrend_fast_below_zero() {
        let c: Vec<f64> = (0..100).map(|i| 200.0 - i as f64).collect();
        let r = compute(&c, 3, 10, 16);
        let last = r.fast[99].unwrap();
        assert!(last < 0.0);
    }

    #[test]
    fn histogram_smaller_than_fast_in_steady_trend() {
        // In a steady trend, fast and signal converge → histogram → 0
        // relative to fast.
        let c: Vec<f64> = (0..200).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 3, 10, 16);
        let last_fast = r.fast[199].unwrap();
        let last_hist = r.histogram[199].unwrap();
        assert!(last_hist.abs() < last_fast.abs());
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c, 3, 10, 16);
        assert_eq!(r.fast.len(), 100);
        assert_eq!(r.signal.len(), 100);
        assert_eq!(r.histogram.len(), 100);
    }
}
