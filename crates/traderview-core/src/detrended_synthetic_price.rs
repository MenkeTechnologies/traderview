//! Detrended Synthetic Price (DSP) — John Ehlers (TASC).
//!
//! Removes the trend component from price by subtracting a smoothed
//! reference, leaving only the cyclical/oscillatory part for easier
//! cycle-period estimation downstream.
//!
//! Construction:
//!   smooth_t = (3·close_t + 2·close_{t-1} + close_{t-2}) / 6
//!   detrend_t = smooth_t - EMA(smooth, period)
//!
//! Then the "synthetic price" rescales detrend to its rolling range
//! so it can be plotted on a familiar price-like scale:
//!
//!   max_t  = max(detrend over period bars)
//!   min_t  = min(detrend over period bars)
//!   dsp_t  = (detrend_t - mid) where mid = (max + min) / 2
//!
//! Output is centered near zero with amplitude bounded by the
//! detrended swing. Pure compute. Default period = 14.
//!
//! Companion to `hodrick_prescott`, `ehlers_decycler`, `roofing_filter`,
//! `ehlers_instant_trendline`.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 3 || n < period + 2 { return out; }
    if closes.iter().any(|x| !x.is_finite()) { return out; }
    let mut smooth = vec![0.0_f64; n];
    for i in 0..n {
        smooth[i] = if i < 2 { closes[i] }
            else { (3.0 * closes[i] + 2.0 * closes[i - 1] + closes[i - 2]) / 6.0 };
    }
    let ema_smooth = ema(&smooth, period);
    let mut detrend = vec![None; n];
    for (i, slot) in detrend.iter_mut().enumerate() {
        if let Some(e) = ema_smooth[i] {
            *slot = Some(smooth[i] - e);
        }
    }
    // Center detrended series by its rolling midline over period.
    for i in (period - 1)..n {
        let win = &detrend[i + 1 - period..=i];
        if win.iter().any(|x| x.is_none()) { continue; }
        let vals: Vec<f64> = win.iter().filter_map(|x| *x).collect();
        let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let mid = (max + min) / 2.0;
        out[i] = Some(detrend[i].unwrap() - mid);
    }
    out
}

fn ema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = series[i] * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let s = vec![100.0_f64; 50];
        assert!(compute(&s, 2).iter().all(|x| x.is_none()));
        assert!(compute(&s[..5], 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut s = vec![100.0_f64; 50];
        s[5] = f64::NAN;
        assert!(compute(&s, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_zero_dsp() {
        let s = vec![100.0_f64; 100];
        let r = compute(&s, 14);
        for v in r.iter().skip(30).flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn linear_trend_removes_drift() {
        // Detrended → near zero for steady linear trend.
        let s: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let r = compute(&s, 14);
        // Expect bounded magnitude regardless of how large the trend
        // gets, since the EMA-subtraction removes the level drift.
        let max_abs = r.iter().flatten().fold(0.0_f64, |a, b| a.max(b.abs()));
        assert!(max_abs < 50.0,
            "linear trend should be detrended to bounded amplitude, got {max_abs}");
    }

    #[test]
    fn sin_wave_centered_near_zero() {
        let s: Vec<f64> = (0..400).map(|i| {
            100.0 + (i as f64 * 0.1).sin() * 5.0
        }).collect();
        let r = compute(&s, 14);
        let vals: Vec<f64> = r.iter().flatten().copied().collect();
        let mean: f64 = vals.iter().sum::<f64>() / vals.len() as f64;
        assert!(mean.abs() < 0.5,
            "sin wave detrended mean should be near 0, got {mean}");
    }

    #[test]
    fn output_length_matches_input() {
        let s = vec![100.0_f64; 50];
        assert_eq!(compute(&s, 14).len(), 50);
    }
}
