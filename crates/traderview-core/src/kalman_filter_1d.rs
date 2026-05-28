//! 1-D Kalman filter — scalar state-space tracking.
//!
//! State: x_t = f · x_{t−1} + w (process noise: var = Q)
//! Obs:   z_t = h · x_t + v   (measurement noise: var = R)
//!
//! Predict:
//!   x_pred = f · x_prev
//!   p_pred = f² · p_prev + Q
//!
//! Update:
//!   k     = (h · p_pred) / (h² · p_pred + R)
//!   x_new = x_pred + k · (z_t − h · x_pred)
//!   p_new = (1 − k · h) · p_pred
//!
//! Common uses:
//!   - Smoothing noisy price series
//!   - Tracking moving averages with adaptive bandwidth
//!   - Building anchored mean-reversion models with state-space drift
//!
//! Pure compute. Caller supplies (f, h, Q, R, x0, p0). Returns the
//! filtered state and gain at each step.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KalmanParams1d {
    pub transition_f: f64,
    pub observation_h: f64,
    pub process_noise_q: f64,
    pub measurement_noise_r: f64,
    pub initial_state: f64,
    pub initial_uncertainty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KalmanReport1d {
    pub filtered_state: Vec<f64>,
    pub uncertainty: Vec<f64>,
    pub kalman_gain: Vec<f64>,
}

pub fn filter(observations: &[f64], params: &KalmanParams1d) -> Option<KalmanReport1d> {
    if !params.transition_f.is_finite()
        || !params.observation_h.is_finite()
        || !params.process_noise_q.is_finite() || params.process_noise_q < 0.0
        || !params.measurement_noise_r.is_finite() || params.measurement_noise_r < 0.0
        || !params.initial_state.is_finite()
        || !params.initial_uncertainty.is_finite() || params.initial_uncertainty < 0.0
        || observations.is_empty()
    {
        return None;
    }
    let mut x = params.initial_state;
    let mut p = params.initial_uncertainty;
    let f = params.transition_f;
    let h = params.observation_h;
    let q = params.process_noise_q;
    let r_meas = params.measurement_noise_r;
    let mut filtered = Vec::with_capacity(observations.len());
    let mut uncertainty = Vec::with_capacity(observations.len());
    let mut gain = Vec::with_capacity(observations.len());
    for &z in observations {
        // Predict.
        let x_pred = f * x;
        let p_pred = f * f * p + q;
        if !z.is_finite() {
            // Skip update; carry prediction forward (handles missing data).
            x = x_pred;
            p = p_pred;
            filtered.push(x);
            uncertainty.push(p);
            gain.push(0.0);
            continue;
        }
        let denom = h * h * p_pred + r_meas;
        let k = if denom > 0.0 { h * p_pred / denom } else { 0.0 };
        x = x_pred + k * (z - h * x_pred);
        p = (1.0 - k * h) * p_pred;
        // Guard against numerical drift to non-finite.
        if !x.is_finite() { x = x_pred; }
        if !p.is_finite() || p < 0.0 { p = p_pred.max(0.0); }
        filtered.push(x);
        uncertainty.push(p);
        gain.push(k);
    }
    Some(KalmanReport1d {
        filtered_state: filtered,
        uncertainty,
        kalman_gain: gain,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(f: f64, h: f64, q: f64, r: f64, x0: f64, p0: f64) -> KalmanParams1d {
        KalmanParams1d {
            transition_f: f, observation_h: h,
            process_noise_q: q, measurement_noise_r: r,
            initial_state: x0, initial_uncertainty: p0,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(filter(&[1.0], &p(f64::NAN, 1.0, 0.1, 0.1, 0.0, 1.0)).is_none());
        assert!(filter(&[1.0], &p(1.0, 1.0, -0.1, 0.1, 0.0, 1.0)).is_none());
        assert!(filter(&[1.0], &p(1.0, 1.0, 0.1, -0.1, 0.0, 1.0)).is_none());
        assert!(filter(&[], &p(1.0, 1.0, 0.1, 0.1, 0.0, 1.0)).is_none());
    }

    #[test]
    fn flat_observations_converge_to_mean() {
        // Constant observation 100 → filtered state should converge to 100.
        let obs = vec![100.0; 50];
        let r = filter(&obs, &p(1.0, 1.0, 0.01, 1.0, 0.0, 1000.0)).unwrap();
        let last = *r.filtered_state.last().unwrap();
        assert!((last - 100.0).abs() < 0.5,
            "filtered should converge to 100, got {last}");
    }

    #[test]
    fn high_measurement_noise_dampens_updates() {
        // Large R → small Kalman gain → state moves slowly toward observations.
        let obs = vec![10.0, 100.0, 10.0, 100.0, 10.0];
        let r_lo = filter(&obs, &p(1.0, 1.0, 0.1, 0.01, 0.0, 1.0)).unwrap();
        let r_hi = filter(&obs, &p(1.0, 1.0, 0.1, 1000.0, 0.0, 1.0)).unwrap();
        // Variability of filtered state should be smaller with high R.
        let stdev = |v: &[f64]| -> f64 {
            let m: f64 = v.iter().sum::<f64>() / v.len() as f64;
            (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
        };
        assert!(stdev(&r_hi.filtered_state) < stdev(&r_lo.filtered_state));
    }

    #[test]
    fn kalman_gain_in_unit_range() {
        let obs = vec![10.0; 30];
        let r = filter(&obs, &p(1.0, 1.0, 0.1, 0.5, 0.0, 1.0)).unwrap();
        for k in &r.kalman_gain {
            assert!((0.0..=1.0).contains(k));
        }
    }

    #[test]
    fn uncertainty_decreases_with_observations() {
        // With Q small and stable measurements, uncertainty should
        // monotonically decrease (initial high uncertainty shrinks).
        let obs = vec![10.0; 50];
        let r = filter(&obs, &p(1.0, 1.0, 0.0, 1.0, 0.0, 100.0)).unwrap();
        let last_unc = *r.uncertainty.last().unwrap();
        let first_unc = r.uncertainty[0];
        assert!(last_unc < first_unc);
    }

    #[test]
    fn nan_observation_carries_prediction() {
        let obs = vec![10.0, 12.0, f64::NAN, 14.0];
        let r = filter(&obs, &p(1.0, 1.0, 0.1, 0.5, 10.0, 1.0)).unwrap();
        assert_eq!(r.filtered_state.len(), 4);
        // Skipped observation → gain at index 2 should be 0.
        assert_eq!(r.kalman_gain[2], 0.0);
    }

    #[test]
    fn negative_uncertainty_clamped_to_zero() {
        // Pathological inputs (Q=0, R=0) can drive uncertainty negative;
        // guard clamps it.
        let obs = vec![10.0; 5];
        let r = filter(&obs, &p(1.0, 1.0, 0.0, 0.0, 10.0, 1.0)).unwrap();
        for u in &r.uncertainty {
            assert!(*u >= 0.0);
        }
    }
}
