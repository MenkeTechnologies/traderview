//! Rauch-Tung-Striebel (RTS) Kalman Smoother — backward pass over a
//! 1-D Kalman filter trajectory, producing smoothed state estimates
//! that condition on the WHOLE series (past + future), not just the
//! past as the forward filter does.
//!
//! Model (matches `kalman_filter_1d` 1-D random-walk):
//!   state:        x_t = x_{t-1} + w_t,         w_t ~ N(0, q)
//!   observation:  y_t = x_t + v_t,             v_t ~ N(0, r)
//!
//! Forward pass: standard Kalman filter giving (x̂_t|t, P_t|t).
//! Backward pass (RTS recursion, t = N-1, ..., 0):
//!   x̂_t|t+1|t = x̂_t|t                              (predict next)
//!   P_t|t+1|t = P_t|t + q
//!   J_t       = P_t|t / P_t|t+1|t                  (smoother gain)
//!   x̂_t|N    = x̂_t|t + J_t · (x̂_{t+1}|N - x̂_t|t)
//!   P_t|N    = P_t|t + J_t² · (P_{t+1}|N - P_t|t+1|t)
//!
//! Smoothed estimates have strictly LOWER variance than filtered ones
//! (more data → less uncertainty), and the smoothed series is
//! end-point-consistent: the last smoothed value equals the last
//! filtered value.
//!
//! Pure compute. Companion to `kalman_filter_1d`,
//! `kalman_dynamic_beta`, `savitzky_golay`.

#[derive(Debug)]
pub struct Report {
    pub smoothed_state: Vec<f64>,
    pub smoothed_variance: Vec<f64>,
}

pub fn compute(
    observations: &[f64],
    process_noise_q: f64,
    obs_noise_r: f64,
    x0: f64,
    p0: f64,
) -> Option<Report> {
    let n = observations.len();
    if n == 0 {
        return None;
    }
    if !process_noise_q.is_finite() || !obs_noise_r.is_finite() {
        return None;
    }
    if !x0.is_finite() || !p0.is_finite() {
        return None;
    }
    if process_noise_q < 0.0 || obs_noise_r <= 0.0 || p0 < 0.0 {
        return None;
    }
    if observations.iter().any(|y| !y.is_finite()) {
        return None;
    }
    // Forward pass: standard 1-D Kalman.
    let mut x_pred = vec![0.0_f64; n];
    let mut p_pred = vec![0.0_f64; n];
    let mut x_filt = vec![0.0_f64; n];
    let mut p_filt = vec![0.0_f64; n];
    let mut x = x0;
    let mut p = p0;
    for t in 0..n {
        // Predict.
        let xp = x; // F = 1
        let pp = p + process_noise_q;
        x_pred[t] = xp;
        p_pred[t] = pp;
        // Update.
        let s = pp + obs_noise_r;
        let k = pp / s;
        x = xp + k * (observations[t] - xp);
        p = (1.0 - k) * pp;
        if p < 0.0 {
            p = 0.0;
        }
        x_filt[t] = x;
        p_filt[t] = p;
    }
    // Backward smoother.
    let mut x_smooth = x_filt.clone();
    let mut p_smooth = p_filt.clone();
    for t in (0..n - 1).rev() {
        let p_next_pred = p_pred[t + 1];
        if p_next_pred < 1e-15 {
            x_smooth[t] = x_filt[t];
            p_smooth[t] = p_filt[t];
            continue;
        }
        let j = p_filt[t] / p_next_pred;
        x_smooth[t] = x_filt[t] + j * (x_smooth[t + 1] - x_pred[t + 1]);
        p_smooth[t] = p_filt[t] + j * j * (p_smooth[t + 1] - p_next_pred);
        if p_smooth[t] < 0.0 {
            p_smooth[t] = 0.0;
        }
    }
    Some(Report {
        smoothed_state: x_smooth,
        smoothed_variance: p_smooth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let y = vec![1.0_f64; 20];
        assert!(compute(&[], 1e-3, 1e-2, 0.0, 1.0).is_none());
        assert!(compute(&y, -1.0, 1e-2, 0.0, 1.0).is_none());
        assert!(compute(&y, 1e-3, 0.0, 0.0, 1.0).is_none());
        assert!(compute(&y, 1e-3, 1e-2, f64::NAN, 1.0).is_none());
        let mut bad = y.clone();
        bad[0] = f64::NAN;
        assert!(compute(&bad, 1e-3, 1e-2, 0.0, 1.0).is_none());
    }

    #[test]
    fn endpoint_equals_filtered_endpoint() {
        // RTS smoother converges to the forward filter at the last sample.
        let y: Vec<f64> = (0..30)
            .map(|i| (i as f64 * 0.2).sin() + 0.01 * i as f64)
            .collect();
        let r = compute(&y, 1e-3, 1e-2, 0.0, 1.0).unwrap();
        // Run a manual forward pass to get the filtered endpoint.
        let mut x = 0.0;
        let mut p = 1.0;
        for &obs in &y {
            let xp = x;
            let pp = p + 1e-3;
            let s = pp + 1e-2;
            let k = pp / s;
            x = xp + k * (obs - xp);
            p = (1.0 - k) * pp;
        }
        let last = y.len() - 1;
        assert!((r.smoothed_state[last] - x).abs() < 1e-9);
    }

    #[test]
    fn smoother_reduces_endpoint_variance_at_earlier_indices() {
        // Smoothed P at index 0 should be ≤ filtered P at index 0 because
        // the smoother conditions on future data.
        let y: Vec<f64> = (0..50).map(|i| (i as f64 * 0.3).sin()).collect();
        let r = compute(&y, 1e-3, 1e-2, 0.0, 1.0).unwrap();
        // Manual forward to get P_0 filtered.
        let mut p = 1.0;
        let pp = p + 1e-3;
        let s = pp + 1e-2;
        let k = pp / s;
        p = (1.0 - k) * pp;
        assert!(r.smoothed_variance[0] <= p + 1e-12);
    }

    #[test]
    fn flat_signal_tracks_constant() {
        // Constant input → smoother converges to that constant.
        let y = vec![5.0_f64; 50];
        let r = compute(&y, 1e-6, 1e-4, 0.0, 1.0).unwrap();
        for v in &r.smoothed_state[10..] {
            assert!((v - 5.0).abs() < 0.1);
        }
    }

    #[test]
    fn smoothed_variance_non_negative() {
        let y: Vec<f64> = (0..30).map(|i| (i as f64 * 0.5).sin()).collect();
        let r = compute(&y, 1e-3, 1e-2, 0.0, 1.0).unwrap();
        for v in &r.smoothed_variance {
            assert!(*v >= 0.0);
        }
    }

    #[test]
    fn output_lengths_match_observations() {
        let y = vec![1.0_f64; 20];
        let r = compute(&y, 1e-3, 1e-2, 0.0, 1.0).unwrap();
        assert_eq!(r.smoothed_state.len(), 20);
        assert_eq!(r.smoothed_variance.len(), 20);
    }
}
