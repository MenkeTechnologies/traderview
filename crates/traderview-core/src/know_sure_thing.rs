//! Know Sure Thing (KST) — Martin Pring.
//!
//! Smoothed sum of four rate-of-change series:
//!
//!   ROCMA1 = SMA(ROC(close, 10), 10)
//!   ROCMA2 = SMA(ROC(close, 15), 10)
//!   ROCMA3 = SMA(ROC(close, 20), 10)
//!   ROCMA4 = SMA(ROC(close, 30), 15)
//!   KST = ROCMA1 + 2·ROCMA2 + 3·ROCMA3 + 4·ROCMA4
//!   Signal = SMA(KST, 9)
//!
//! Range is unbounded; centered at zero. Bullish above zero, bearish below.
//! Signal-line crossovers are the primary trade signal.
//!
//! Pure compute. Companion to `coppock_curve`, `trix`, `tsi`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KstReport {
    pub kst: Vec<Option<f64>>,
    pub signal: Vec<Option<f64>>,
}

/// Standard Pring KST with default (10,15,20,30) ROC periods, (10,10,10,15)
/// SMA periods, and a 9-bar signal SMA.
pub fn compute(closes: &[f64]) -> KstReport {
    compute_with(closes, (10, 15, 20, 30), (10, 10, 10, 15), 9)
}

pub fn compute_with(
    closes: &[f64],
    roc_periods: (usize, usize, usize, usize),
    sma_periods: (usize, usize, usize, usize),
    signal_period: usize,
) -> KstReport {
    let n = closes.len();
    let mut report = KstReport { kst: vec![None; n], signal: vec![None; n] };
    if signal_period < 2 { return report; }
    if [roc_periods.0, roc_periods.1, roc_periods.2, roc_periods.3,
        sma_periods.0, sma_periods.1, sma_periods.2, sma_periods.3]
        .iter().any(|p| *p < 2) { return report; }
    if closes.iter().any(|x| !x.is_finite()) { return report; }
    let roc1 = roc(closes, roc_periods.0);
    let roc2 = roc(closes, roc_periods.1);
    let roc3 = roc(closes, roc_periods.2);
    let roc4 = roc(closes, roc_periods.3);
    let rocma1 = sma_opt(&roc1, sma_periods.0);
    let rocma2 = sma_opt(&roc2, sma_periods.1);
    let rocma3 = sma_opt(&roc3, sma_periods.2);
    let rocma4 = sma_opt(&roc4, sma_periods.3);
    for i in 0..n {
        if let (Some(a), Some(b), Some(c), Some(d))
            = (rocma1[i], rocma2[i], rocma3[i], rocma4[i]) {
            report.kst[i] = Some(a + 2.0 * b + 3.0 * c + 4.0 * d);
        }
    }
    report.signal = sma_opt(&report.kst, signal_period);
    report
}

fn roc(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    for i in period..n {
        let prev = series[i - period];
        if prev != 0.0 {
            out[i] = Some(((series[i] - prev) / prev) * 100.0);
        }
    }
    out
}

fn sma_opt(series: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    for i in (period - 1)..n {
        let win = &series[i + 1 - period..=i];
        if win.iter().any(|x| x.is_none()) { continue; }
        let s: f64 = win.iter().filter_map(|x| *x).sum();
        out[i] = Some(s / p_f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty_kst() {
        let r = compute(&[]);
        assert!(r.kst.is_empty());
        assert!(r.signal.is_empty());
    }

    #[test]
    fn nan_returns_all_none() {
        let mut closes = vec![100.0_f64; 80];
        closes[5] = f64::NAN;
        let r = compute(&closes);
        assert!(r.kst.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_kst() {
        let closes = vec![100.0_f64; 100];
        let r = compute(&closes);
        // All ROCs are zero → KST zero, signal zero.
        for v in r.kst.iter().flatten() { assert!(v.abs() < 1e-9); }
        for v in r.signal.iter().flatten() { assert!(v.abs() < 1e-9); }
    }

    #[test]
    fn strong_uptrend_yields_positive_kst() {
        let closes: Vec<f64> = (0..100).map(|i| 100.0 * 1.01_f64.powi(i)).collect();
        let r = compute(&closes);
        let last = r.kst.iter().rev().find_map(|x| *x).unwrap();
        assert!(last > 0.0,
            "uptrend should yield positive KST, got {last}");
    }

    #[test]
    fn strong_downtrend_yields_negative_kst() {
        let closes: Vec<f64> = (0..100).map(|i| 200.0 * 0.99_f64.powi(i)).collect();
        let r = compute(&closes);
        let last = r.kst.iter().rev().find_map(|x| *x).unwrap();
        assert!(last < 0.0);
    }

    #[test]
    fn signal_smoother_than_kst() {
        // Sinusoidal closes → KST oscillates → 9-bar SMA signal has
        // smaller magnitude swings than the raw KST line.
        let closes: Vec<f64> = (0..400).map(|i| {
            100.0 + (i as f64 * 0.05).sin() * 20.0
        }).collect();
        let r = compute(&closes);
        let kst_vals: Vec<f64> = r.kst.iter().flatten().copied().collect();
        let sig_vals: Vec<f64> = r.signal.iter().flatten().copied().collect();
        let kst_amp = kst_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            - kst_vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let sig_amp = sig_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            - sig_vals.iter().cloned().fold(f64::INFINITY, f64::min);
        assert!(sig_amp < kst_amp,
            "signal range {sig_amp} should be smaller than KST range {kst_amp}");
    }

    #[test]
    fn output_length_matches_input() {
        let closes = vec![100.0_f64; 100];
        let r = compute(&closes);
        assert_eq!(r.kst.len(), 100);
        assert_eq!(r.signal.len(), 100);
    }
}
