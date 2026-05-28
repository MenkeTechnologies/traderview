//! Kalman Dynamic Beta — time-varying β of an asset vs a benchmark
//! estimated by a 1-D Kalman filter (random-walk on β).
//!
//! Model (per bar t):
//!   state:        β_t = β_{t-1} + w_t,    w_t ~ N(0, q)
//!   observation:  r_t = β_t · r_bench_t + v_t, v_t ~ N(0, r)
//!
//! Filter recursion:
//!   predict:  β̂_t|t-1 = β̂_{t-1}; P_t|t-1 = P_{t-1} + q
//!   gain:     K_t = P_t|t-1 · r_b_t / (r_b_t² · P_t|t-1 + r)
//!   update:   β̂_t = β̂_t|t-1 + K_t (r_t - β̂_t|t-1 · r_b_t)
//!             P_t = (1 - K_t · r_b_t) · P_t|t-1
//!
//! Choosing q small / r large → slow-moving β (smooth). Choosing q
//! large / r small → fast-moving β (responsive).
//!
//! Pure compute. Companion to `rolling_beta`, `kalman_filter_pair`,
//! `kalman_alpha_beta`.

pub fn compute(
    asset: &[f64], bench: &[f64],
    process_noise_q: f64, obs_noise_r: f64,
    beta0: f64, p0: f64,
) -> Vec<Option<f64>> {
    let n = asset.len();
    let mut out = vec![None; n];
    if n == 0 || bench.len() != n { return out; }
    if !process_noise_q.is_finite() || !obs_noise_r.is_finite() { return out; }
    if !beta0.is_finite() || !p0.is_finite() { return out; }
    if process_noise_q < 0.0 || obs_noise_r <= 0.0 || p0 < 0.0 { return out; }
    if asset.iter().chain(bench.iter()).any(|x| !x.is_finite()) { return out; }
    let mut beta = beta0;
    let mut p = p0;
    for (i, slot) in out.iter_mut().enumerate() {
        // Predict.
        p += process_noise_q;
        let rb = bench[i];
        let s = rb * rb * p + obs_noise_r;
        let k = if s > 0.0 { p * rb / s } else { 0.0 };
        let resid = asset[i] - beta * rb;
        beta += k * resid;
        p *= 1.0 - k * rb;
        if p < 0.0 { p = 0.0; }
        *slot = Some(beta);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let a = vec![0.01_f64; 50];
        let b = vec![0.01_f64; 50];
        let bad = vec![0.01_f64; 40];
        assert!(compute(&a, &bad, 1e-6, 1e-4, 1.0, 1.0).iter().all(|x| x.is_none()));
        assert!(compute(&a, &b, -1.0, 1e-4, 1.0, 1.0).iter().all(|x| x.is_none()));
        assert!(compute(&a, &b, 1e-6, 0.0, 1.0, 1.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut a = vec![0.01_f64; 50];
        let b = vec![0.01_f64; 50];
        a[5] = f64::NAN;
        assert!(compute(&a, &b, 1e-6, 1e-4, 1.0, 1.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn perfectly_correlated_unit_beta_converges_to_one() {
        // asset_t = bench_t exactly → β → 1.
        let b: Vec<f64> = (0..200).map(|i| ((i as f64 * 0.3).sin()) * 0.02).collect();
        let a = b.clone();
        let r = compute(&a, &b, 1e-6, 1e-4, 0.5, 1.0);
        let final_beta = r.last().unwrap().unwrap();
        assert!((final_beta - 1.0).abs() < 0.1);
    }

    #[test]
    fn scaled_correlated_converges_to_scale() {
        // asset_t = 2 · bench_t → β → 2.
        let b: Vec<f64> = (0..300).map(|i| ((i as f64 * 0.3).sin()) * 0.02).collect();
        let a: Vec<f64> = b.iter().map(|x| 2.0 * x).collect();
        let r = compute(&a, &b, 1e-6, 1e-4, 0.0, 1.0);
        let final_beta = r.last().unwrap().unwrap();
        assert!((final_beta - 2.0).abs() < 0.2);
    }

    #[test]
    fn beta_responds_to_regime_change() {
        // First 100 bars β=1, next 100 bars β=3.
        let mut a = Vec::new();
        let mut b = Vec::new();
        for i in 0..100 {
            let bench_r = ((i as f64 * 0.3).sin()) * 0.02;
            b.push(bench_r);
            a.push(bench_r);
        }
        for i in 0..100 {
            let bench_r = ((i as f64 * 0.3).sin()) * 0.02;
            b.push(bench_r);
            a.push(3.0 * bench_r);
        }
        // Higher q for responsiveness.
        let r = compute(&a, &b, 1e-3, 1e-5, 1.0, 1.0);
        let mid = r[100].unwrap();
        let final_beta = r.last().unwrap().unwrap();
        // β should drift from 1 toward 3 by end.
        assert!(final_beta > mid);
    }

    #[test]
    fn output_length_matches_input() {
        let a = vec![0.01_f64; 50];
        let b = vec![0.01_f64; 50];
        assert_eq!(compute(&a, &b, 1e-6, 1e-4, 1.0, 1.0).len(), 50);
    }
}
