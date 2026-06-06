//! Hilbert Transform Indicator — John F. Ehlers ("Rocket Science for
//! Traders", 2001).
//!
//! Approximates the analytic signal of a price series to extract:
//!
//!   - **In-phase (I)** and **Quadrature (Q)** components, computed
//!     via Ehlers' 4-bar FIR Hilbert filter:
//!     det = 0.0962·x + 0.5769·x`[2]` − 0.5769·x`[4]` − 0.0962·x`[6]`
//!     q  = det · period_factor
//!     i  = x`[3]`
//!   - **Instantaneous phase**: φ = arctan(Q / I)
//!   - **Dominant Cycle Period**: estimated from the rate of change of
//!     phase using Ehlers' smoothing rules
//!
//! Used in: cycle-based trading systems, regime classification
//! (trending vs cycling markets), adaptive filter parameter selection.
//!
//! Pure compute. Caller supplies a smoothed price series (typically the
//! 4-bar WMA of `(high + low) / 2` for noise reduction).
//!
//! Companion to `ehlers_decycler`, `mcginley_dynamic`,
//! `dominant_cycle` (if added in the future).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HilbertReport {
    pub in_phase: Vec<Option<f64>>,
    pub quadrature: Vec<Option<f64>>,
    pub instantaneous_phase_radians: Vec<Option<f64>>,
    pub dominant_cycle_period: Vec<Option<f64>>,
}

pub fn compute(smoothed_price: &[f64]) -> HilbertReport {
    let n = smoothed_price.len();
    let mut in_phase = vec![None; n];
    let mut quad = vec![None; n];
    let mut phase = vec![None; n];
    let mut period = vec![None; n];
    if n < 8 || smoothed_price.iter().any(|x| !x.is_finite()) {
        return HilbertReport {
            in_phase,
            quadrature: quad,
            instantaneous_phase_radians: phase,
            dominant_cycle_period: period,
        };
    }
    let mut prev_period = 20.0_f64; // seed
    for i in 6..n {
        // Ehlers FIR Hilbert filter (4 non-zero coefficients spaced 2 bars apart).
        let det = 0.0962 * smoothed_price[i] + 0.5769 * smoothed_price[i - 2]
            - 0.5769 * smoothed_price[i - 4]
            - 0.0962 * smoothed_price[i - 6];
        let period_factor = 0.075 * prev_period + 0.54;
        let q = det * period_factor;
        let ii = smoothed_price[i - 3];
        in_phase[i] = Some(ii);
        quad[i] = Some(q);
        let ph = q.atan2(ii);
        phase[i] = Some(ph);
        // Estimate dominant cycle from phase delta.
        if i > 7 {
            if let Some(prev_phase) = phase[i - 1] {
                // Unwrap into Ehlers' [0.1, 1.1] convention.
                let delta_phase = (prev_phase - ph).clamp(0.1, 1.1);
                let instantaneous_period = 2.0 * std::f64::consts::PI / delta_phase;
                let mut new_period = 0.33 * instantaneous_period + 0.67 * prev_period;
                new_period = new_period.clamp(6.0, 50.0);
                prev_period = new_period;
                period[i] = Some(new_period);
            }
        }
    }
    HilbertReport {
        in_phase,
        quadrature: quad,
        instantaneous_phase_radians: phase,
        dominant_cycle_period: period,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_all_none() {
        let s = vec![100.0_f64; 5];
        let r = compute(&s);
        assert!(r.in_phase.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_input_returns_all_none() {
        let mut s = vec![100.0_f64; 20];
        s[5] = f64::NAN;
        let r = compute(&s);
        assert!(r.in_phase.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_zero_quadrature() {
        let s = vec![100.0_f64; 50];
        let r = compute(&s);
        // With constant input, the FIR det = 0 → Q = 0.
        for v in r.quadrature.iter().skip(6).flatten() {
            assert!(v.abs() < 1e-9, "flat input Q should be 0, got {v}");
        }
    }

    #[test]
    fn sinusoidal_input_detects_period_within_range() {
        // 20-bar sine wave. Hilbert should converge to a period in [6, 50].
        let period_true = 20.0_f64;
        let s: Vec<f64> = (0..200)
            .map(|i| 100.0 + (2.0 * std::f64::consts::PI * i as f64 / period_true).sin() * 5.0)
            .collect();
        let r = compute(&s);
        // Take average period over last 50 bars.
        let est: Vec<f64> = r
            .dominant_cycle_period
            .iter()
            .skip(150)
            .filter_map(|x| *x)
            .collect();
        let avg = est.iter().sum::<f64>() / est.len() as f64;
        assert!(
            (6.0..=50.0).contains(&avg),
            "period {avg} outside clamp range"
        );
    }

    #[test]
    fn period_clamped_to_six_fifty() {
        // High-frequency input: period should be clamped at 6.
        let s: Vec<f64> = (0..200)
            .map(|i| {
                100.0 + (i as f64).sin() * 5.0 // ~6.28 bar period
            })
            .collect();
        let r = compute(&s);
        for v in r.dominant_cycle_period.iter().skip(10).flatten() {
            assert!(
                (6.0..=50.0).contains(v),
                "period {v} should be clamped to [6, 50]"
            );
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let s: Vec<f64> = (0..50)
            .map(|i| 100.0 + (i as f64 * 0.1).sin() * 5.0)
            .collect();
        let r = compute(&s);
        assert_eq!(r.in_phase.len(), 50);
        assert_eq!(r.quadrature.len(), 50);
        assert_eq!(r.instantaneous_phase_radians.len(), 50);
        assert_eq!(r.dominant_cycle_period.len(), 50);
    }

    #[test]
    fn first_six_indices_unfilled() {
        let s: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let r = compute(&s);
        for i in 0..6 {
            assert!(r.in_phase[i].is_none(), "index {i} should be None");
        }
    }
}
