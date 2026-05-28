//! Balance of Power (BOP) — Igor Livshin.
//!
//! Per-bar oscillator that gauges who controlled the bar's price
//! action — buyers (positive) or sellers (negative):
//!
//!   BOP_t = (close_t − open_t) / (high_t − low_t)
//!
//! Bars where high == low contribute 0 (no range, no information).
//! Optionally smoothed by an SMA of `smoothing_period` bars; the
//! smoothed series is what most charting packages display.
//!
//! Range [−1, +1]:
//!   +1   = full bullish (close at high, open at low)
//!   −1   = full bearish (close at low, open at high)
//!    0   = balanced
//!
//! Pure compute. Companion to `chaikin_money_flow`, `force_index`,
//! `accumulation_distribution_line`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BalanceOfPowerReport {
    pub raw_bop: Vec<Option<f64>>,
    pub smoothed_bop: Vec<Option<f64>>,
    pub smoothing_period: usize,
}

pub fn compute(bars: &[Bar], smoothing_period: usize) -> BalanceOfPowerReport {
    let n = bars.len();
    let mut raw = vec![None; n];
    let mut smoothed = vec![None; n];
    if n == 0 || smoothing_period == 0 {
        return BalanceOfPowerReport { raw_bop: raw, smoothed_bop: smoothed, smoothing_period };
    }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return BalanceOfPowerReport { raw_bop: raw, smoothed_bop: smoothed, smoothing_period };
    }
    for (i, bar) in bars.iter().enumerate() {
        let range = bar.high - bar.low;
        raw[i] = if range > 0.0 {
            Some(((bar.close - bar.open) / range).clamp(-1.0, 1.0))
        } else {
            Some(0.0)
        };
    }
    if smoothing_period > 1 && n >= smoothing_period {
        for i in (smoothing_period - 1)..n {
            let win = &raw[i + 1 - smoothing_period..=i];
            let sum: f64 = win.iter().filter_map(|x| *x).sum();
            smoothed[i] = Some(sum / smoothing_period as f64);
        }
    } else {
        smoothed = raw.clone();
    }
    BalanceOfPowerReport { raw_bop: raw, smoothed_bop: smoothed, smoothing_period }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> Bar { Bar { open: o, high: h, low: l, close: c } }

    #[test]
    fn empty_returns_empty_outputs() {
        let r = compute(&[], 14);
        assert!(r.raw_bop.is_empty());
    }

    #[test]
    fn zero_smoothing_period_returns_all_none() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.5)];
        let r = compute(&bars, 0);
        assert!(r.raw_bop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let bars = vec![b(f64::NAN, 101.0, 99.0, 100.5)];
        let r = compute(&bars, 14);
        assert!(r.raw_bop.iter().all(|x| x.is_none()));
    }

    #[test]
    fn full_bullish_bar_yields_plus_one() {
        // open at low, close at high.
        let bars = vec![b(99.0, 101.0, 99.0, 101.0)];
        let r = compute(&bars, 1);
        assert_eq!(r.raw_bop[0].unwrap(), 1.0);
    }

    #[test]
    fn full_bearish_bar_yields_minus_one() {
        let bars = vec![b(101.0, 101.0, 99.0, 99.0)];
        let r = compute(&bars, 1);
        assert_eq!(r.raw_bop[0].unwrap(), -1.0);
    }

    #[test]
    fn balanced_bar_yields_zero() {
        // open == close.
        let bars = vec![b(100.0, 101.0, 99.0, 100.0)];
        let r = compute(&bars, 1);
        assert_eq!(r.raw_bop[0].unwrap(), 0.0);
    }

    #[test]
    fn zero_range_bar_yields_zero() {
        let bars = vec![b(100.0, 100.0, 100.0, 100.0)];
        let r = compute(&bars, 1);
        assert_eq!(r.raw_bop[0].unwrap(), 0.0);
    }

    #[test]
    fn smoothed_bop_is_sma_of_raw() {
        let bars = vec![
            b(99.0, 101.0, 99.0, 101.0),     // +1
            b(101.0, 101.0, 99.0, 99.0),     // -1
            b(100.0, 101.0, 99.0, 100.0),    // 0
        ];
        let r = compute(&bars, 3);
        // Average of +1, -1, 0 = 0.
        assert!((r.smoothed_bop[2].unwrap()).abs() < 1e-12);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.5); 30];
        let r = compute(&bars, 5);
        assert_eq!(r.raw_bop.len(), 30);
        assert_eq!(r.smoothed_bop.len(), 30);
    }
}
