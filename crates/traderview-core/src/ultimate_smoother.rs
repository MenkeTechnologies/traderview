//! Ultimate Smoother — John Ehlers (TASC, 2024).
//!
//! Improvement over the SuperSmoother filter: zero lag at the
//! tunable cutoff period while still suppressing higher-frequency
//! noise. The IIR coefficients come from a 2-pole low-pass design
//! tuned to be flat in the passband:
//!
//!   a1 = exp(-1.414·π / period)
//!   b1 = 2·a1·cos(1.414·180°/period)
//!   c2 = b1
//!   c3 = -a1·a1
//!   c1 = (1 + c2 - c3) / 4
//!   us_t = (1 - c1)·x_t + (2·c1 - c2)·x_{t-1} - (c1 + c3)·x_{t-2}
//!          + c2·us_{t-1} + c3·us_{t-2}
//!
//! Pure compute. Companion to `ehlers_super_smoother`, `ehlers_decycler`,
//! `roofing_filter`, `ehlers_instant_trendline`.

pub fn compute(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if n < 3 || period < 4 {
        return out;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let pi = std::f64::consts::PI;
    let a1 = (-1.414 * pi / period as f64).exp();
    let b1 = 2.0 * a1 * (1.414 * pi / period as f64).cos();
    let c2 = b1;
    let c3 = -a1 * a1;
    let c1 = (1.0 + c2 - c3) / 4.0;
    let mut us = vec![0.0_f64; n];
    // Seed first two bars at input value.
    us[0] = series[0];
    us[1] = series[1];
    out[0] = Some(us[0]);
    out[1] = Some(us[1]);
    for i in 2..n {
        us[i] = (1.0 - c1) * series[i] + (2.0 * c1 - c2) * series[i - 1]
            - (c1 + c3) * series[i - 2]
            + c2 * us[i - 1]
            + c3 * us[i - 2];
        out[i] = Some(us[i]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let s = vec![100.0_f64; 50];
        assert!(compute(&s, 3).iter().all(|x| x.is_none()));
        assert!(compute(&s[..2], 10).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut s = vec![100.0_f64; 50];
        s[5] = f64::NAN;
        assert!(compute(&s, 10).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_settles_to_constant() {
        let s = vec![100.0_f64; 200];
        let r = compute(&s, 10);
        for v in r.iter().skip(20).flatten() {
            assert!((v - 100.0).abs() < 1e-6);
        }
    }

    #[test]
    fn linear_trend_tracks_input() {
        let s: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let r = compute(&s, 10);
        let last = r[199].unwrap();
        // Ultimate Smoother claims zero lag at cutoff — should match input closely.
        assert!(
            (last - 199.0).abs() < 1.0,
            "Ultimate Smoother {last} should be near input 199"
        );
    }

    #[test]
    fn noise_amplitude_reduced() {
        let mut state: u64 = 42;
        let s: Vec<f64> = (0..400)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
                100.0 + (r - 0.5) * 10.0
            })
            .collect();
        let r = compute(&s, 10);
        let vals: Vec<f64> = r.iter().skip(50).flatten().copied().collect();
        let mean_in: f64 = s.iter().sum::<f64>() / s.len() as f64;
        let var_in: f64 = s.iter().map(|x| (x - mean_in).powi(2)).sum::<f64>() / s.len() as f64;
        let mean_us: f64 = vals.iter().sum::<f64>() / vals.len() as f64;
        let var_us: f64 =
            vals.iter().map(|x| (x - mean_us).powi(2)).sum::<f64>() / vals.len() as f64;
        assert!(var_us < var_in);
    }

    #[test]
    fn output_length_matches_input() {
        let s = vec![100.0_f64; 50];
        assert_eq!(compute(&s, 10).len(), 50);
    }
}
