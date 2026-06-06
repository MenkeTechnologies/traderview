//! Nadaraya-Watson Kernel Regression (1964).
//!
//! Non-parametric estimator that smooths a series by weighting each
//! observation with a kernel function centered at the evaluation
//! point:
//!
//!   ŷ(t) = Σ_i K_h(t − i) · y_i  /  Σ_i K_h(t − i)
//!
//! Using a Gaussian kernel:
//!
//!   K_h(d) = exp(−d² / (2·h²))
//!
//! Bandwidth `h` controls smoothness:
//!   small h → close-to-original (high variance, low bias)
//!   large h → very smooth (low variance, high bias)
//!
//! Variants:
//!   - `evaluate_at_indices`: smoothed series aligned to input index
//!   - `evaluate_at_grid`: arbitrary evaluation grid (useful for chart
//!     overlays at higher resolution than the bar series)
//!
//! Pure compute. Useful as a chart overlay or to feed into peak/trough
//! detection on a denoised series. Companion to `holt_winters`,
//! `hodrick_prescott`.

pub fn evaluate_at_indices(y: &[f64], bandwidth: f64) -> Vec<Option<f64>> {
    let n = y.len();
    let mut out = vec![None; n];
    if n == 0 || !bandwidth.is_finite() || bandwidth <= 0.0 {
        return out;
    }
    if y.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let two_h2 = 2.0 * bandwidth * bandwidth;
    for (t, slot) in out.iter_mut().enumerate() {
        let mut num = 0.0_f64;
        let mut den = 0.0_f64;
        for (i, yi) in y.iter().enumerate() {
            let d = t as f64 - i as f64;
            let w = (-d * d / two_h2).exp();
            num += w * yi;
            den += w;
        }
        if den > 0.0 {
            *slot = Some(num / den);
        }
    }
    out
}

pub fn evaluate_at_grid(y: &[f64], grid: &[f64], bandwidth: f64) -> Vec<Option<f64>> {
    let n = y.len();
    let mut out = vec![None; grid.len()];
    if n == 0 || grid.is_empty() || !bandwidth.is_finite() || bandwidth <= 0.0 {
        return out;
    }
    if y.iter().any(|x| !x.is_finite()) {
        return out;
    }
    if grid.iter().any(|t| !t.is_finite()) {
        return out;
    }
    let two_h2 = 2.0 * bandwidth * bandwidth;
    for (g_idx, t) in grid.iter().enumerate() {
        let mut num = 0.0_f64;
        let mut den = 0.0_f64;
        for (i, yi) in y.iter().enumerate() {
            let d = t - i as f64;
            let w = (-d * d / two_h2).exp();
            num += w * yi;
            den += w;
        }
        if den > 0.0 {
            out[g_idx] = Some(num / den);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(evaluate_at_indices(&[], 5.0).is_empty());
    }

    #[test]
    fn invalid_bandwidth_returns_all_none() {
        let y = vec![100.0_f64; 10];
        assert!(evaluate_at_indices(&y, 0.0).iter().all(|x| x.is_none()));
        assert!(evaluate_at_indices(&y, -1.0).iter().all(|x| x.is_none()));
        assert!(evaluate_at_indices(&y, f64::NAN)
            .iter()
            .all(|x| x.is_none()));
    }

    #[test]
    fn nan_input_returns_all_none() {
        let y = vec![100.0, f64::NAN, 102.0];
        assert!(evaluate_at_indices(&y, 1.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_recovers_flat_output() {
        let y = vec![100.0_f64; 20];
        let out = evaluate_at_indices(&y, 5.0);
        for v in out.iter().flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn very_small_bandwidth_recovers_input() {
        let y: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let out = evaluate_at_indices(&y, 0.01);
        for (i, v) in out.iter().enumerate() {
            assert!(
                (v.unwrap() - y[i]).abs() < 1e-9,
                "tiny bandwidth at idx {i}: got {} vs input {}",
                v.unwrap(),
                y[i]
            );
        }
    }

    #[test]
    fn large_bandwidth_recovers_mean() {
        let y: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let mean: f64 = y.iter().sum::<f64>() / y.len() as f64;
        let out = evaluate_at_indices(&y, 1_000.0);
        for v in out.iter().flatten() {
            assert!(
                (v - mean).abs() < 0.5,
                "huge bandwidth should approach mean {mean}, got {v}"
            );
        }
    }

    #[test]
    fn output_length_matches_input() {
        let y: Vec<f64> = (0..50).map(|i| (i as f64 * 0.3).sin() * 5.0).collect();
        let out = evaluate_at_indices(&y, 3.0);
        assert_eq!(out.len(), 50);
    }

    #[test]
    fn grid_evaluation_off_input_points() {
        let y = vec![0.0, 10.0, 20.0, 30.0, 40.0];
        // Evaluate at non-integer points.
        let grid = vec![0.5, 1.5, 2.5];
        let out = evaluate_at_grid(&y, &grid, 1.0);
        // At t=0.5: roughly between y[0]=0 and y[1]=10 with kernel weights.
        let v0 = out[0].unwrap();
        let v1 = out[1].unwrap();
        let v2 = out[2].unwrap();
        assert!(
            v0 < v1 && v1 < v2,
            "expected monotonic, got {v0}, {v1}, {v2}"
        );
        assert!((0.0..=10.0).contains(&v0));
    }

    #[test]
    fn noisy_input_smoothed_within_signal_range() {
        let mut state: u64 = 7;
        let y: Vec<f64> = (0..100)
            .map(|i| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let noise = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0;
                (i as f64 * 0.1).sin() * 5.0 + noise
            })
            .collect();
        let out = evaluate_at_indices(&y, 4.0);
        // Smoothed values should lie within input min/max range.
        let in_min = y.iter().cloned().fold(f64::INFINITY, f64::min);
        let in_max = y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        for v in out.iter().flatten() {
            assert!(*v >= in_min - 1e-9 && *v <= in_max + 1e-9);
        }
    }
}
