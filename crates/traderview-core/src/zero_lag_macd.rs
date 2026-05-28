//! Zero-Lag MACD — John Ehlers / Tradingview community variant.
//!
//! Standard MACD lags its input by the (fast - 1)/2 + (slow - 1)/2
//! offset. Zero-Lag MACD reduces lag by EMA'ing a forward-shifted
//! input (per the de-Lag, or "ZLEMA", construction):
//!
//!   lag_n = (period - 1) / 2
//!   zlema_t = EMA(2·x_t - x_{t-lag_n}, period)
//!
//!   zl_fast = ZLEMA(close, fast_period)
//!   zl_slow = ZLEMA(close, slow_period)
//!   macd    = zl_fast - zl_slow
//!   signal  = ZLEMA(macd, signal_period)
//!   hist    = macd - signal
//!
//! Pure compute. Defaults: 12 / 26 / 9. Companion to `ppo`,
//! `tsi`, `linda_raschke_3_10`, `ehlers_decycler_oscillator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZeroLagMacdReport {
    pub macd: Vec<Option<f64>>,
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
) -> ZeroLagMacdReport {
    let n = closes.len();
    let mut report = ZeroLagMacdReport {
        macd: vec![None; n],
        signal: vec![None; n],
        histogram: vec![None; n],
        fast_period,
        slow_period,
        signal_period,
    };
    if fast_period < 2 || slow_period < 2 || signal_period < 2
        || fast_period >= slow_period
        || n < slow_period + signal_period { return report; }
    if closes.iter().any(|x| !x.is_finite()) { return report; }
    let fast = zlema(closes, fast_period);
    let slow = zlema(closes, slow_period);
    for i in 0..n {
        if let (Some(f), Some(s)) = (fast[i], slow[i]) {
            report.macd[i] = Some(f - s);
        }
    }
    report.signal = zlema_opt(&report.macd, signal_period);
    for i in 0..n {
        if let (Some(m), Some(s)) = (report.macd[i], report.signal[i]) {
            report.histogram[i] = Some(m - s);
        }
    }
    report
}

fn zlema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let lag = (period - 1) / 2;
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    // De-lag input: shifted = 2·x_t - x_{t-lag}.
    let mut shifted = vec![0.0_f64; n];
    for i in 0..n {
        let prev = if i >= lag { series[i - lag] } else { series[0] };
        shifted[i] = 2.0 * series[i] - prev;
    }
    let seed: f64 = shifted[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for (i, &v) in shifted.iter().enumerate().skip(period) {
        cur = v * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

fn zlema_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 { return out; }
    // Find first index with at least `period` consecutive Somes.
    let mut seed_end = None;
    let mut count = 0_usize;
    let mut seed_sum = 0.0_f64;
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
    let lag = (period - 1) / 2;
    let mut cur = seed_sum / p_f;
    out[end] = Some(cur);
    for i in (end + 1)..n {
        if let Some(v) = series[i] {
            let prev = if i >= lag {
                series[i - lag].unwrap_or(v)
            } else {
                v
            };
            let shifted = 2.0 * v - prev;
            cur = shifted * k + cur * (1.0 - k);
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

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 50];
        assert!(compute(&c, 1, 26, 9).macd.iter().all(|x| x.is_none()));
        assert!(compute(&c, 26, 12, 9).macd.iter().all(|x| x.is_none()));
        assert!(compute(&c[..5], 12, 26, 9).macd.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 50];
        c[5] = f64::NAN;
        assert!(compute(&c, 12, 26, 9).macd.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_macd() {
        let c = vec![100.0_f64; 80];
        let r = compute(&c, 12, 26, 9);
        for v in r.macd.iter().flatten() { assert!(v.abs() < 1e-9); }
    }

    #[test]
    fn uptrend_yields_positive_macd() {
        let c: Vec<f64> = (0..80).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 12, 26, 9);
        let last = 79;
        assert!(r.macd[last].unwrap() > 0.0);
    }

    #[test]
    fn downtrend_yields_negative_macd() {
        let c: Vec<f64> = (0..80).map(|i| 200.0 - i as f64).collect();
        let r = compute(&c, 12, 26, 9);
        let last = 79;
        assert!(r.macd[last].unwrap() < 0.0);
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 12, 26, 9);
        assert_eq!(r.macd.len(), 50);
        assert_eq!(r.signal.len(), 50);
        assert_eq!(r.histogram.len(), 50);
    }
}
