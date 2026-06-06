//! Ehlers SuperSmoother — John Ehlers (Cybernetic Analysis, 2004).
//!
//! Second-order Butterworth filter applied to price. Removes
//! high-frequency noise without the lag of a moving-average chain,
//! producing the smoothest "leading-edge" line in the indicator world.
//!
//!   a1 = exp(−√2 · π / period)
//!   b1 = 2 · a1 · cos(√2 · π / period)
//!   c2 = b1
//!   c3 = −a1²
//!   c1 = 1 − c2 − c3
//!   SS_t = c1 · (close_t + close_{t−1}) / 2 + c2 · SS_{t−1} + c3 · SS_{t−2}
//!
//! Pure compute.

use std::f64::consts::PI;

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || n < 3 {
        return out;
    }
    let p = period as f64;
    let a1 = (-(2.0_f64).sqrt() * PI / p).exp();
    let b1 = 2.0 * a1 * ((2.0_f64).sqrt() * PI / p).cos();
    let c2 = b1;
    let c3 = -a1 * a1;
    let c1 = 1.0 - c2 - c3;
    // Seed first two outputs at the close (Ehlers's convention).
    out[0] = Some(closes[0]);
    out[1] = Some(closes[1]);
    for i in 2..n {
        let c = closes[i];
        let prev_c = closes[i - 1];
        if !c.is_finite() || !prev_c.is_finite() {
            out[i] = out[i - 1];
            continue;
        }
        let s_1 = out[i - 1].unwrap_or(c);
        let s_2 = out[i - 2].unwrap_or(c);
        let ss = c1 * (c + prev_c) / 2.0 + c2 * s_1 + c3 * s_2;
        if ss.is_finite() {
            out[i] = Some(ss);
        } else {
            out[i] = out[i - 1];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_too_short_returns_empty() {
        assert!(compute(&[], 10).is_empty());
        let r = compute(&[100.0, 101.0], 10);
        assert!(r.iter().all(|x| x.is_none()));
    }

    #[test]
    fn invalid_period_returns_all_none() {
        let v = vec![100.0; 30];
        assert!(compute(&v, 0).iter().all(|x| x.is_none()));
        assert!(compute(&v, 1).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_super_smoother_equals_constant() {
        let v = vec![100.0; 50];
        let out = compute(&v, 10);
        let last = out[49].expect("populated");
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn rising_series_super_smoother_tracks_with_low_lag() {
        let v: Vec<f64> = (1..=100).map(|i| 100.0 + i as f64).collect();
        let out = compute(&v, 10);
        let last = out[99].expect("populated");
        // Less lag than a 10-period EMA, but still some lag.
        assert!(last > 180.0 && last < v[99]);
    }

    #[test]
    fn noisy_series_smoother_than_raw() {
        // Construct a noisy series — verify the output has lower variance
        // than the input over the same window.
        let v: Vec<f64> = (0..200)
            .map(|i| 100.0 + (i as f64 * 0.5).sin() * 5.0 + ((i as f64 * 17.3).sin() * 2.0))
            .collect();
        let out = compute(&v, 10);
        let smoothed: Vec<f64> = out.iter().filter_map(|x| *x).collect();
        let raw_var = variance(&v);
        let smoothed_var = variance(&smoothed);
        assert!(
            smoothed_var < raw_var,
            "smoothed var {smoothed_var} should be < raw var {raw_var}"
        );
    }

    fn variance(v: &[f64]) -> f64 {
        let n = v.len() as f64;
        let mean: f64 = v.iter().sum::<f64>() / n;
        v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![100.0; 5];
        // Period 5 → e^(- sqrt2 * pi / 5) and friends are finite. Just verify no panic.
        let _ = compute(&v, 1_000_000);
    }

    #[test]
    fn nan_recovers_from_prior_value() {
        let mut v = vec![100.0; 10];
        v.push(f64::NAN);
        v.extend(vec![100.0; 5]);
        let out = compute(&v, 10);
        let at_nan = out[10];
        assert!(at_nan.unwrap_or(0.0).is_finite());
    }
}
