//! Elliott Wave Oscillator (EWO) — Bill Williams.
//!
//! Difference of two simple moving averages of the median price:
//!
//!   median_t = (high_t + low_t) / 2
//!   EWO_t    = SMA(median, fast) - SMA(median, slow)
//!
//! Default periods: fast = 5, slow = 35. Used by Elliott Wave
//! practitioners to confirm wave 3 (largest EWO peak) vs wave 5
//! (lower EWO peak despite higher price). Zero-line crosses mark
//! impulse-vs-corrective transitions.
//!
//! Pure compute. Companion to `coppock_curve`, `awesome_oscillator`,
//! `acceleration_deceleration`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64 }

pub fn compute(bars: &[Bar], fast: usize, slow: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if fast < 2 || slow < 2 || fast >= slow || n < slow { return out; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()) { return out; }
    let median: Vec<f64> = bars.iter().map(|b| (b.high + b.low) / 2.0).collect();
    let fast_sma = sma(&median, fast);
    let slow_sma = sma(&median, slow);
    for i in 0..n {
        if let (Some(f), Some(s)) = (fast_sma[i], slow_sma[i]) {
            out[i] = Some(f - s);
        }
    }
    out
}

fn sma(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    let mut sum: f64 = series[..period].iter().sum();
    out[period - 1] = Some(sum / p_f);
    for i in period..n {
        sum += series[i] - series[i - period];
        out[i] = Some(sum / p_f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar { Bar { high: h, low: l } }

    #[test]
    fn invalid_inputs_return_all_none() {
        let bars = vec![b(101.0, 99.0); 50];
        assert!(compute(&bars, 1, 35).iter().all(|x| x.is_none()));
        assert!(compute(&bars, 35, 5).iter().all(|x| x.is_none()));    // fast > slow
        assert!(compute(&bars[..10], 5, 35).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut bars = vec![b(101.0, 99.0); 50];
        bars[5] = b(f64::NAN, 99.0);
        assert!(compute(&bars, 5, 35).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_ewo() {
        let bars = vec![b(101.0, 99.0); 50];
        let r = compute(&bars, 5, 35);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn strong_uptrend_yields_positive_ewo() {
        let bars: Vec<_> = (0..80).map(|i| {
            let m = 100.0 + i as f64;
            b(m + 0.5, m - 0.5)
        }).collect();
        let r = compute(&bars, 5, 35);
        let last = r[79].unwrap();
        // fast SMA leads the trend → above slow SMA.
        assert!(last > 0.0);
    }

    #[test]
    fn strong_downtrend_yields_negative_ewo() {
        let bars: Vec<_> = (0..80).map(|i| {
            let m = 200.0 - i as f64;
            b(m + 0.5, m - 0.5)
        }).collect();
        let r = compute(&bars, 5, 35);
        assert!(r[79].unwrap() < 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0); 50];
        assert_eq!(compute(&bars, 5, 35).len(), 50);
    }
}
