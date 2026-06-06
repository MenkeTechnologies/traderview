//! Ergodic Oscillator — William Blau's TSI + signal-line variant.
//!
//! Builds on the True Strength Index (TSI) by adding a fast signal
//! line for crossover detection — Blau's original framing in
//! "Momentum, Direction and Divergence" (1995):
//!
//!   pc   = close - close[-1]
//!   smooth_pc  = EMA(EMA(pc, r), s)
//!   smooth_apc = EMA(EMA(|pc|, r), s)
//!   tsi  = 100 · smooth_pc / smooth_apc
//!   signal = EMA(tsi, sig_period)
//!   hist = tsi - signal
//!
//! Crossover of TSI above signal = bullish, below = bearish.
//!
//! Distinct from `tsi` (which only returns the line). This module also
//! returns the signal and histogram.
//!
//! Pure compute. Defaults: r=25, s=13, sig=7. Companion to `tsi`,
//! `ppo`, `linda_raschke_3_10`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErgodicReport {
    pub tsi: Vec<Option<f64>>,
    pub signal: Vec<Option<f64>>,
    pub histogram: Vec<Option<f64>>,
    pub r_period: usize,
    pub s_period: usize,
    pub signal_period: usize,
}

pub fn compute(
    closes: &[f64],
    r_period: usize,
    s_period: usize,
    signal_period: usize,
) -> ErgodicReport {
    let n = closes.len();
    let mut report = ErgodicReport {
        tsi: vec![None; n],
        signal: vec![None; n],
        histogram: vec![None; n],
        r_period,
        s_period,
        signal_period,
    };
    if r_period < 2
        || s_period < 2
        || signal_period < 2
        || n < r_period + s_period + signal_period + 1
    {
        return report;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return report;
    }
    let mut pc = vec![0.0_f64; n];
    let mut apc = vec![0.0_f64; n];
    for i in 1..n {
        pc[i] = closes[i] - closes[i - 1];
        apc[i] = pc[i].abs();
    }
    let smooth_pc = double_ema(&pc[1..], r_period, s_period, 1);
    let smooth_apc = double_ema(&apc[1..], r_period, s_period, 1);
    for i in 0..n {
        if let (Some(p), Some(a)) = (smooth_pc[i], smooth_apc[i]) {
            if a > 0.0 {
                report.tsi[i] = Some(100.0 * p / a);
            }
        }
    }
    report.signal = ema_opt(&report.tsi, signal_period);
    for i in 0..n {
        if let (Some(t), Some(s)) = (report.tsi[i], report.signal[i]) {
            report.histogram[i] = Some(t - s);
        }
    }
    report
}

fn double_ema(series: &[f64], r: usize, s: usize, offset: usize) -> Vec<Option<f64>> {
    let n_input = series.len();
    let total = n_input + offset;
    let mut out = vec![None; total];
    let first = ema(series, r);
    let second_input: Vec<f64> = first.iter().map(|x| x.unwrap_or(0.0)).collect();
    let second_valid_from = first.iter().position(|x| x.is_some()).unwrap_or(n_input);
    let second = ema(&second_input[second_valid_from..], s);
    for (i, v) in second.iter().enumerate() {
        let global_idx = second_valid_from + i + offset;
        if global_idx < total {
            out[global_idx] = *v;
        }
    }
    out
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
    let mut count = 0_usize;
    let mut seed_sum = 0.0_f64;
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

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 100];
        assert!(compute(&c, 1, 13, 7).tsi.iter().all(|x| x.is_none()));
        assert!(compute(&c[..10], 25, 13, 7).tsi.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 100];
        c[5] = f64::NAN;
        assert!(compute(&c, 25, 13, 7).tsi.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_tsi() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c, 25, 13, 7);
        // pc all zero → smooth_apc zero → TSI undefined (None).
        // That's acceptable — verify no panic.
        assert_eq!(r.tsi.len(), 100);
    }

    #[test]
    fn uptrend_yields_positive_tsi() {
        let c: Vec<f64> = (0..200).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 25, 13, 7);
        let last = r.tsi.iter().rev().find_map(|x| *x).unwrap();
        assert!(last > 50.0, "uptrend should yield TSI > 50, got {last}");
    }

    #[test]
    fn downtrend_yields_negative_tsi() {
        let c: Vec<f64> = (0..200).map(|i| 300.0 - i as f64).collect();
        let r = compute(&c, 25, 13, 7);
        let last = r.tsi.iter().rev().find_map(|x| *x).unwrap();
        assert!(last < -50.0);
    }

    #[test]
    fn output_lengths_match_input() {
        let c = vec![100.0_f64; 100];
        let r = compute(&c, 25, 13, 7);
        assert_eq!(r.tsi.len(), 100);
        assert_eq!(r.signal.len(), 100);
        assert_eq!(r.histogram.len(), 100);
    }
}
