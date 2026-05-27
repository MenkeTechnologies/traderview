//! Force Index — Alexander Elder.
//!
//!   force = (close_t - close_{t-1}) × volume_t
//!
//! Raw FI; the smoothed version is the EMA over `period` bars
//! (default 13). Convention:
//!   - FI > 0 = buying force dominant.
//!   - FI < 0 = selling force dominant.
//!   - Crossover of zero is a signal.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar { pub close: f64, pub volume: f64 }

pub fn raw(bars: &[Bar]) -> Vec<f64> {
    let n = bars.len();
    let mut out = vec![0.0; n];
    for i in 1..n {
        out[i] = (bars[i].close - bars[i - 1].close) * bars[i].volume;
    }
    out
}

pub fn smoothed(bars: &[Bar], period: usize) -> Vec<f64> {
    let raw_fi = raw(bars);
    let n = raw_fi.len();
    let mut out = vec![0.0; n];
    if n == 0 || period == 0 { return out; }
    let k = 2.0 / (period as f64 + 1.0);
    let mut prev = raw_fi[0];
    out[0] = prev;
    for i in 1..n {
        let e = k * raw_fi[i] + (1.0 - k) * prev;
        out[i] = e;
        prev = e;
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForceCross { Bullish, Bearish, NoCross }

pub fn detect_zero_cross(fi: &[f64]) -> Vec<ForceCross> {
    let mut out = vec![ForceCross::NoCross; fi.len()];
    for i in 1..fi.len() {
        out[i] = if fi[i - 1] < 0.0 && fi[i] >= 0.0 { ForceCross::Bullish }
            else if fi[i - 1] >= 0.0 && fi[i] < 0.0 { ForceCross::Bearish }
            else { ForceCross::NoCross };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(c: f64, v: f64) -> Bar { Bar { close: c, volume: v } }

    #[test]
    fn empty_returns_empty() {
        assert!(raw(&[]).is_empty());
        assert!(smoothed(&[], 13).is_empty());
    }

    #[test]
    fn first_bar_force_index_zero() {
        let out = raw(&[b(100.0, 1000.0)]);
        assert_eq!(out[0], 0.0);
    }

    #[test]
    fn up_close_positive_force() {
        let out = raw(&[
            b(100.0, 1000.0),
            b(105.0, 500.0),    // up $5 × 500 vol = +2500.
        ]);
        assert_eq!(out[1], 2500.0);
    }

    #[test]
    fn down_close_negative_force() {
        let out = raw(&[
            b(100.0, 1000.0),
            b(95.0, 500.0),
        ]);
        assert_eq!(out[1], -2500.0);
    }

    #[test]
    fn flat_close_zero_force() {
        let out = raw(&[
            b(100.0, 1000.0),
            b(100.0, 500.0),
        ]);
        assert_eq!(out[1], 0.0);
    }

    #[test]
    fn smoothed_attenuates_volatile_raw() {
        let bars: Vec<Bar> = (1..=20).map(|i|
            b(100.0 + (i % 2) as f64 * 10.0, 1000.0)
        ).collect();
        let r = raw(&bars);
        let s = smoothed(&bars, 5);
        // Last smoothed should be in absolute terms less than the max raw.
        let max_raw = r.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(s[19].abs() <= max_raw);
    }

    // ─── zero-cross detection ──────────────────────────────────────────

    #[test]
    fn detect_bullish_cross_when_fi_goes_positive() {
        let fi = vec![-100.0, -50.0, 50.0];
        let crosses = detect_zero_cross(&fi);
        assert_eq!(crosses[2], ForceCross::Bullish);
    }

    #[test]
    fn detect_bearish_cross_when_fi_goes_negative() {
        let fi = vec![100.0, 50.0, -50.0];
        let crosses = detect_zero_cross(&fi);
        assert_eq!(crosses[2], ForceCross::Bearish);
    }

    #[test]
    fn no_cross_when_fi_stays_same_sign() {
        let fi = vec![100.0, 200.0, 300.0];
        let crosses = detect_zero_cross(&fi);
        for c in &crosses[1..] {
            assert_eq!(*c, ForceCross::NoCross);
        }
    }
}
