//! LOWESS — Cleveland (1979) Locally Weighted Scatterplot Smoothing.
//!
//! For each point x_i, fit a weighted linear regression using a window
//! of `frac · n` nearest neighbors, weighted by the tri-cube kernel:
//!
//!   w_j = (1 - (|x_j - x_i| / h_i)^3)^3   for |x_j - x_i| < h_i
//!
//! where h_i is the distance to the furthest neighbor in the window.
//! Output is the predicted ŷ_i at each x_i.
//!
//! Optional `robustness` iterations down-weight outliers using
//! bisquare weights based on residuals from the prior pass — the
//! published robust LOWESS variant.
//!
//! Input must have monotonic x (sorted or already an index). For
//! evenly-spaced bar data, pass `x = (0..n)` as float indices.
//!
//! Pure compute. Companion to `savitzky_golay`, `nadaraya_watson`,
//! `kalman_filter_1d`, `wavelet_decomposition_haar`.

#![allow(clippy::needless_range_loop)]

pub fn compute(x: &[f64], y: &[f64], frac: f64, robustness_iter: u32) -> Option<Vec<f64>> {
    let n = x.len();
    if n < 5 || y.len() != n {
        return None;
    }
    if !frac.is_finite() || !(0.0..=1.0).contains(&frac) || frac == 0.0 {
        return None;
    }
    if x.iter().chain(y.iter()).any(|v| !v.is_finite()) {
        return None;
    }
    // x must be non-decreasing.
    for w in x.windows(2) {
        if w[1] < w[0] {
            return None;
        }
    }
    let r = ((frac * n as f64).round() as usize).clamp(2, n);
    let mut robust_weights = vec![1.0_f64; n];
    let mut y_hat = vec![0.0_f64; n];
    for pass in 0..=robustness_iter {
        for i in 0..n {
            let h = neighbor_radius(x, i, r);
            // Compute weights for neighborhood window.
            let mut weights = vec![0.0_f64; n];
            for j in 0..n {
                let d = (x[j] - x[i]).abs();
                if h > 0.0 && d < h {
                    let u = d / h;
                    let tri = 1.0 - u * u * u;
                    let w = tri * tri * tri;
                    weights[j] = w * robust_weights[j];
                } else if h == 0.0 && d == 0.0 {
                    weights[j] = robust_weights[j];
                }
            }
            y_hat[i] = wls_fit_predict(x, y, &weights, x[i]);
        }
        if pass == robustness_iter {
            break;
        }
        // Update robustness weights from residuals.
        let residuals: Vec<f64> = (0..n).map(|i| y[i] - y_hat[i]).collect();
        let mut abs_res: Vec<f64> = residuals.iter().map(|r| r.abs()).collect();
        abs_res.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = abs_res[n / 2];
        let s = 6.0 * median.max(1e-12);
        for i in 0..n {
            let u = (residuals[i] / s).abs().min(1.0);
            let bisq = 1.0 - u * u;
            robust_weights[i] = bisq * bisq;
        }
    }
    Some(y_hat)
}

fn neighbor_radius(x: &[f64], i: usize, r: usize) -> f64 {
    // Find the `r` nearest neighbors of x[i] (including i itself).
    let mut dists: Vec<f64> = x.iter().map(|xj| (xj - x[i]).abs()).collect();
    dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    dists[(r - 1).min(dists.len() - 1)]
}

/// Weighted least squares fit y = a + b·x, predict at `x_eval`.
fn wls_fit_predict(x: &[f64], y: &[f64], w: &[f64], x_eval: f64) -> f64 {
    let mut sw = 0.0;
    let mut swx = 0.0;
    let mut swy = 0.0;
    let mut swxx = 0.0;
    let mut swxy = 0.0;
    for i in 0..x.len() {
        let wi = w[i];
        if wi <= 0.0 {
            continue;
        }
        sw += wi;
        swx += wi * x[i];
        swy += wi * y[i];
        swxx += wi * x[i] * x[i];
        swxy += wi * x[i] * y[i];
    }
    if sw <= 0.0 {
        return 0.0;
    }
    let mean_x = swx / sw;
    let mean_y = swy / sw;
    let denom = swxx - sw * mean_x * mean_x;
    if denom.abs() < 1e-15 {
        return mean_y;
    }
    let b = (swxy - sw * mean_x * mean_y) / denom;
    let a = mean_y - b * mean_x;
    a + b * x_eval
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y = vec![1.0_f64; 10];
        assert!(compute(&x, &y, 0.0, 0).is_none());
        assert!(compute(&x, &y, 1.5, 0).is_none());
        assert!(compute(&x, &y[..5], 0.5, 0).is_none());
        assert!(compute(&x[..3], &y[..3], 0.5, 0).is_none());
        let bad_x = vec![5.0, 3.0, 4.0, 6.0, 7.0];
        let zeros = vec![0.0_f64; 5];
        assert!(compute(&bad_x, &zeros, 0.5, 0).is_none());
        let mut nan_y = y.clone();
        nan_y[0] = f64::NAN;
        assert!(compute(&x, &nan_y, 0.5, 0).is_none());
    }

    #[test]
    fn perfect_line_recovered() {
        // y = 2·x + 1 → LOWESS at every point should return ~ 2·x + 1.
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi + 1.0).collect();
        let y_hat = compute(&x, &y, 0.5, 0).unwrap();
        for i in 0..50 {
            assert!((y_hat[i] - y[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn smooths_noisy_signal_below_input_variance() {
        let x: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .enumerate()
            .map(|(i, xi)| 0.1 * xi + 5.0 * ((i as f64 * 0.7).sin()))
            .collect();
        let y_hat = compute(&x, &y, 0.3, 0).unwrap();
        let var_y = variance(&y);
        let resid: Vec<f64> = y.iter().zip(y_hat.iter()).map(|(a, b)| a - b).collect();
        let var_resid = variance(&resid);
        assert!(
            var_resid < var_y,
            "smoothed residuals should have lower variance than input"
        );
    }

    #[test]
    fn robust_iterations_resist_outliers() {
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let mut y: Vec<f64> = x.iter().map(|xi| 2.0 * xi + 1.0).collect();
        // Inject 3 wild outliers.
        y[5] = 1000.0;
        y[20] = -1000.0;
        y[40] = 2000.0;
        let plain = compute(&x, &y, 0.4, 0).unwrap();
        let robust = compute(&x, &y, 0.4, 3).unwrap();
        // Check fit at a clean (non-outlier) point.
        let expected = 2.0 * x[25] + 1.0;
        let plain_err = (plain[25] - expected).abs();
        let robust_err = (robust[25] - expected).abs();
        assert!(
            robust_err <= plain_err,
            "robust iterations must not be worse at clean points"
        );
    }

    #[test]
    fn output_length_matches_input() {
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y = vec![1.0_f64; 20];
        let r = compute(&x, &y, 0.5, 0).unwrap();
        assert_eq!(r.len(), 20);
    }

    fn variance(v: &[f64]) -> f64 {
        let m: f64 = v.iter().sum::<f64>() / v.len() as f64;
        v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64
    }
}
