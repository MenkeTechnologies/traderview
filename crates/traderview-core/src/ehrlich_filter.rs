//! Ehrlich Filter — Stan Ehrlich's range-based cycle filter (TASC, 1997).
//!
//! Smooths the median price using an adaptive weighting scheme based on
//! the relative size of true range vs the filter period's value. Used
//! to remove noise in choppy markets while preserving trend turns:
//!
//!   median_t = (high + low) / 2
//!   tr_t = max(high - low, |high - close_prev|, |low - close_prev|)
//!   coef_t = period · tr_t / SUM(tr over period bars)
//!     (proportional weight: bigger TR → larger smoothing coefficient,
//!     clamped to [0.5, 2.0])
//!   filter_t = (coef · median_t + (period - coef) · filter_{t-1}) / period
//!
//! Pure compute. Default period = 14. Companion to `kalman_filter_1d`,
//! `ehlers_super_smoother`, `vidya`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || n < period + 1 { return out; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return out;
    }
    let median: Vec<f64> = bars.iter().map(|b| (b.high + b.low) / 2.0).collect();
    let mut tr = vec![0.0_f64; n];
    tr[0] = bars[0].high - bars[0].low;
    for i in 1..n {
        let pc = bars[i - 1].close;
        tr[i] = (bars[i].high - bars[i].low)
            .max((bars[i].high - pc).abs())
            .max((bars[i].low - pc).abs());
    }
    let p_f = period as f64;
    // Seed filter with median at end of warmup window.
    let mut filter = median[period - 1];
    out[period - 1] = Some(filter);
    for i in period..n {
        let sum_tr: f64 = tr[i + 1 - period..=i].iter().sum();
        if sum_tr > 0.0 {
            let coef = (p_f * tr[i] / sum_tr).clamp(0.5, 2.0);
            filter = (coef * median[i] + (p_f - coef) * filter) / p_f;
            out[i] = Some(filter);
        } else {
            out[i] = Some(filter);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert!(compute(&bars, 1).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        assert!(compute(&bars, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_constant_filter() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 14);
        for v in r.iter().flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn linear_trend_tracks_input() {
        let bars: Vec<_> = (0..50).map(|i| {
            let m = 100.0 + i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 14);
        let last = r[49].unwrap();
        // Filter lags input by < period.
        assert!((149.0 - last).abs() < 14.0);
    }

    #[test]
    fn filter_smoother_than_input() {
        let mut state: u64 = 42;
        let bars: Vec<_> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            let m = 100.0 + (r - 0.5) * 10.0;
            b(m + 1.0, m - 1.0, m)
        }).collect();
        let r = compute(&bars, 14);
        let medians: Vec<f64> = bars.iter().map(|b| (b.high + b.low) / 2.0).collect();
        let vals: Vec<f64> = r.iter().flatten().copied().collect();
        let mean_m: f64 = medians.iter().sum::<f64>() / medians.len() as f64;
        let var_m: f64 = medians.iter().map(|x| (x - mean_m).powi(2)).sum::<f64>() / medians.len() as f64;
        let mean_f: f64 = vals.iter().sum::<f64>() / vals.len() as f64;
        let var_f: f64 = vals.iter().map(|x| (x - mean_f).powi(2)).sum::<f64>() / vals.len() as f64;
        assert!(var_f < var_m);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert_eq!(compute(&bars, 14).len(), 30);
    }
}
