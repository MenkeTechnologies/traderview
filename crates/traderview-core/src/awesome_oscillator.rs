//! Awesome Oscillator (AO) — Bill Williams.
//!
//!   AO_t = SMA(median, 5) - SMA(median, 34)
//!
//! where median = (high + low) / 2.
//!
//! Oscillates around zero. Crossover above zero = momentum bullish;
//! below = bearish. The "saucer" + "twin peaks" patterns are classic
//! AO entry signals. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar { pub high: f64, pub low: f64 }

pub fn compute(bars: &[Bar], short: usize, long: usize) -> Vec<f64> {
    let n = bars.len();
    let mut out = vec![0.0; n];
    if n < long || short == 0 || long == 0 || long < short { return out; }
    let medians: Vec<f64> = bars.iter().map(|b| (b.high + b.low) / 2.0).collect();
    for i in (long - 1)..n {
        let s: f64 = medians[(i + 1 - short)..=i].iter().sum::<f64>() / short as f64;
        let l: f64 = medians[(i + 1 - long)..=i].iter().sum::<f64>() / long as f64;
        out[i] = s - l;
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AoCross { CrossedUp, CrossedDown, NoCross }

pub fn detect_zero_cross(ao: &[f64]) -> Vec<AoCross> {
    let mut out = vec![AoCross::NoCross; ao.len()];
    for i in 1..ao.len() {
        let cross = if ao[i - 1] < 0.0 && ao[i] >= 0.0 { AoCross::CrossedUp }
            else if ao[i - 1] >= 0.0 && ao[i] < 0.0 { AoCross::CrossedDown }
            else { AoCross::NoCross };
        out[i] = cross;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar { Bar { high: h, low: l } }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 5, 34).is_empty());
    }

    #[test]
    fn under_long_period_returns_zeros() {
        let bars = vec![b(10.0, 9.0); 10];
        let out = compute(&bars, 5, 34);
        for v in &out { assert_eq!(*v, 0.0); }
    }

    #[test]
    fn ao_positive_in_strong_uptrend() {
        let bars: Vec<Bar> = (1..=40).map(|i| {
            let p = i as f64;
            b(p + 1.0, p - 1.0)
        }).collect();
        let out = compute(&bars, 5, 34);
        // Recent SMA > long SMA in uptrend → AO > 0.
        assert!(out[39] > 0.0);
    }

    #[test]
    fn ao_negative_in_strong_downtrend() {
        let bars: Vec<Bar> = (1..=40).map(|i| {
            let p = 50.0 - i as f64;
            b(p + 1.0, p - 1.0)
        }).collect();
        let out = compute(&bars, 5, 34);
        assert!(out[39] < 0.0);
    }

    #[test]
    fn ao_near_zero_for_choppy_series() {
        let bars: Vec<Bar> = (1..=40).map(|_| b(101.0, 99.0)).collect();
        let out = compute(&bars, 5, 34);
        assert!(out[39].abs() < 1e-9);
    }

    // ─── zero-cross detection ──────────────────────────────────────────

    #[test]
    fn detect_cross_up_when_ao_turns_positive() {
        let ao = vec![-1.0, -0.5, 0.0, 0.5];
        let crosses = detect_zero_cross(&ao);
        assert_eq!(crosses[2], AoCross::CrossedUp);
    }

    #[test]
    fn detect_cross_down_when_ao_turns_negative() {
        let ao = vec![1.0, 0.5, -0.1, -0.5];
        let crosses = detect_zero_cross(&ao);
        assert_eq!(crosses[2], AoCross::CrossedDown);
    }

    #[test]
    fn no_cross_when_ao_stays_positive() {
        let ao = vec![1.0, 1.5, 2.0];
        let crosses = detect_zero_cross(&ao);
        for c in &crosses[1..] {
            assert_eq!(*c, AoCross::NoCross);
        }
    }

    #[test]
    fn long_less_than_short_returns_zeros() {
        // Defensive: invalid args.
        let bars = vec![b(10.0, 9.0); 40];
        let out = compute(&bars, 34, 5);
        for v in &out { assert_eq!(*v, 0.0); }
    }
}
