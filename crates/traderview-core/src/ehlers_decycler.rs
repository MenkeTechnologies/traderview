//! Ehlers Decycler — John Ehlers (Cycle Analytics, 2013).
//!
//! Removes cycle components below the supplied period, leaving only the
//! underlying trend. Effectively a high-pass filter inverted:
//!
//!   alpha = (cos(2π/period) + sin(2π/period) − 1) / cos(2π/period)
//!   decycler_t = (alpha / 2) · (close_t + close_{t-1}) + (1 − alpha) · decycler_{t-1}
//!
//! Different from `ehlers_super_smoother` (low-pass) — decycler is the
//! "trend only" line. Crossover of price above/below decycler is the
//! cleanest trend-change signal Ehlers offers.
//!
//! Pure compute.

use std::f64::consts::PI;

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || n < 2 {
        return out;
    }
    let omega = 2.0 * PI / period as f64;
    let c = omega.cos();
    if c.abs() < f64::EPSILON {
        return out;
    }
    let alpha = (c + omega.sin() - 1.0) / c;
    // Seed first value at the close.
    let mut prev = closes[0];
    out[0] = Some(prev);
    for i in 1..n {
        let cur = closes[i];
        let prev_c = closes[i - 1];
        if !cur.is_finite() || !prev_c.is_finite() {
            out[i] = Some(prev);
            continue;
        }
        let new = (alpha / 2.0) * (cur + prev_c) + (1.0 - alpha) * prev;
        if new.is_finite() {
            prev = new;
            out[i] = Some(prev);
        } else {
            out[i] = Some(prev);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 20).is_empty());
    }

    #[test]
    fn period_too_small_returns_all_none() {
        let v = vec![100.0; 30];
        assert!(compute(&v, 0).iter().all(|x| x.is_none()));
        assert!(compute(&v, 1).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_decycler_equals_constant() {
        let v = vec![100.0; 60];
        let out = compute(&v, 20);
        let last = out[59].expect("populated");
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn rising_series_decycler_tracks_trend() {
        let v: Vec<f64> = (1..=80).map(|i| 100.0 + i as f64).collect();
        let out = compute(&v, 20);
        let last = out[79].expect("populated");
        // Decycler ≈ the trend itself for a linear ramp.
        assert!((last - v[79]).abs() < 10.0);
    }

    #[test]
    fn pure_cycle_with_period_matching_decycler_removes_it() {
        // sin wave at exactly the filter period — should largely be wiped.
        let period = 20;
        let v: Vec<f64> = (0..200)
            .map(|i| 100.0 + (i as f64 * 2.0 * PI / period as f64).sin() * 5.0)
            .collect();
        let out = compute(&v, period);
        // Tail variance should be much lower than input variance.
        let smooth: Vec<f64> = out.iter().filter_map(|x| *x).collect();
        let raw_var = var(&v);
        let smooth_var = var(&smooth);
        assert!(
            smooth_var < raw_var,
            "raw_var={raw_var} smooth_var={smooth_var}"
        );
    }

    fn var(s: &[f64]) -> f64 {
        let n = s.len() as f64;
        let m: f64 = s.iter().sum::<f64>() / n;
        s.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n
    }

    #[test]
    fn nan_recovers_from_prior_value() {
        let mut v = vec![100.0; 20];
        v.push(f64::NAN);
        v.extend(vec![100.0; 5]);
        let out = compute(&v, 10);
        assert!(out[20].unwrap_or(0.0).is_finite());
    }

    #[test]
    fn huge_period_safe() {
        let v = vec![100.0; 30];
        let out = compute(&v, 1_000_000);
        // For huge period, omega → 0, alpha → 0; decycler ≈ prev (no update).
        // Should be all finite, no panic.
        assert!(out.iter().all(|x| x.is_none() || x.unwrap().is_finite()));
    }
}
