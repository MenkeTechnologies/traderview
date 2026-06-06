//! Detrended Fluctuation Analysis (DFA) — Peng et al. (1994).
//!
//! Long-range correlation measure that's more robust than the Hurst
//! exponent on non-stationary inputs (returns are sometimes weakly
//! integrated). Procedure:
//!
//!   1. Integrate the series: y_i = Σ_{j≤i} (r_j − mean(r))
//!   2. For each scale n: split y into ⌊N/n⌋ non-overlapping windows;
//!      fit a least-squares linear trend in each window; compute the
//!      RMS of detrended residuals → F(n).
//!   3. Power-law fit: F(n) = c·n^α. Estimate α via log-log OLS.
//!
//! α interpretation:
//!   - α < 0.5: anti-persistent (mean-reverting)
//!   - α ≈ 0.5: random walk
//!   - α > 0.5: persistent (trending)
//!   - α ≈ 1.0: 1/f noise (long memory)
//!   - α > 1.5: non-stationary, non-fractional
//!
//! Pure compute. Companion to `hurst_exponent`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DfaReport {
    pub alpha: f64,
    pub log_n: Vec<f64>,
    pub log_f: Vec<f64>,
    pub r_squared: f64,
}

pub fn compute(returns: &[f64], scales: &[usize]) -> Option<DfaReport> {
    let n_orig = returns.len();
    if n_orig < 20 || scales.is_empty() {
        return None;
    }
    let r: Vec<f64> = returns.iter().copied().filter(|x| x.is_finite()).collect();
    let n = r.len();
    if n < 20 {
        return None;
    }
    let mean: f64 = r.iter().sum::<f64>() / n as f64;
    // Integrated series.
    let mut y = Vec::with_capacity(n);
    let mut acc = 0.0_f64;
    for v in &r {
        acc += v - mean;
        y.push(acc);
    }
    let mut points: Vec<(f64, f64)> = Vec::new();
    for &scale in scales {
        if scale < 4 || scale > n / 2 {
            continue;
        }
        let n_windows = n / scale;
        if n_windows == 0 {
            continue;
        }
        let mut sum_sq_residuals = 0.0_f64;
        let mut window_count = 0_usize;
        for w in 0..n_windows {
            let start = w * scale;
            let end = start + scale;
            if end > n {
                continue;
            }
            let win = &y[start..end];
            // OLS linear fit y_t = a + b·t over the window.
            let s_len = scale as f64;
            let sum_t: f64 = (0..scale).map(|i| i as f64).sum();
            let sum_y: f64 = win.iter().sum();
            let sum_tt: f64 = (0..scale).map(|i| (i as f64).powi(2)).sum();
            let sum_ty: f64 = win.iter().enumerate().map(|(i, v)| i as f64 * v).sum();
            let denom = s_len * sum_tt - sum_t * sum_t;
            if denom.abs() < 1e-18 {
                continue;
            }
            let b = (s_len * sum_ty - sum_t * sum_y) / denom;
            let a = (sum_y - b * sum_t) / s_len;
            for (i, v) in win.iter().enumerate() {
                let resid = v - (a + b * i as f64);
                sum_sq_residuals += resid * resid;
            }
            window_count += scale;
        }
        if window_count == 0 {
            continue;
        }
        let f_n = (sum_sq_residuals / window_count as f64).sqrt();
        if f_n > 0.0 && f_n.is_finite() {
            points.push((scale as f64, f_n));
        }
    }
    if points.len() < 2 {
        return None;
    }
    // OLS log-log fit.
    let log_n: Vec<f64> = points.iter().map(|p| p.0.ln()).collect();
    let log_f: Vec<f64> = points.iter().map(|p| p.1.ln()).collect();
    let m = log_n.len() as f64;
    let mean_x = log_n.iter().sum::<f64>() / m;
    let mean_y = log_f.iter().sum::<f64>() / m;
    let mut num = 0.0_f64;
    let mut den = 0.0_f64;
    let mut ss_tot = 0.0_f64;
    for (x, y) in log_n.iter().zip(log_f.iter()) {
        let dx = x - mean_x;
        let dy = y - mean_y;
        num += dx * dy;
        den += dx * dx;
        ss_tot += dy * dy;
    }
    if den <= 0.0 {
        return None;
    }
    let alpha = num / den;
    let intercept = mean_y - alpha * mean_x;
    let ss_res: f64 = log_n
        .iter()
        .zip(log_f.iter())
        .map(|(x, y)| (y - (alpha * x + intercept)).powi(2))
        .sum();
    let r_squared = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };
    if !alpha.is_finite() {
        return None;
    }
    Some(DfaReport {
        alpha,
        log_n,
        log_f,
        r_squared,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lcg_seed(seed: u64) -> impl FnMut() -> f64 {
        let mut state = seed;
        move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 32) as f64 / u32::MAX as f64) - 0.5
        }
    }

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01; 10], &[5, 10]).is_none());
    }

    #[test]
    fn no_scales_returns_none() {
        assert!(compute(&[0.01; 100], &[]).is_none());
    }

    #[test]
    fn iid_random_walk_yields_alpha_near_half() {
        let mut rng = lcg_seed(42);
        let r: Vec<f64> = (0..5_000).map(|_| rng()).collect();
        let scales: Vec<usize> = vec![10, 20, 50, 100, 250, 500, 1000];
        let report = compute(&r, &scales).unwrap();
        assert!(
            (report.alpha - 0.5).abs() < 0.15,
            "iid noise should give DFA α ≈ 0.5, got {}",
            report.alpha
        );
    }

    #[test]
    fn trending_series_yields_alpha_above_half() {
        let r: Vec<f64> = (0..2_000).map(|i| (i as f64) * 0.001).collect();
        let scales: Vec<usize> = vec![10, 20, 50, 100, 250];
        let report = compute(&r, &scales).unwrap();
        assert!(
            report.alpha > 0.5,
            "trending series should give α > 0.5, got {}",
            report.alpha
        );
    }

    #[test]
    fn r_squared_in_unit_range() {
        let mut rng = lcg_seed(7);
        let r: Vec<f64> = (0..1_000).map(|_| rng()).collect();
        let scales: Vec<usize> = vec![10, 20, 50, 100, 250];
        let report = compute(&r, &scales).unwrap();
        assert!((-1.0..=1.0).contains(&report.r_squared));
    }

    #[test]
    fn nan_inputs_filtered() {
        let mut r: Vec<f64> = (0..200).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        r[100] = f64::NAN;
        let scales: Vec<usize> = vec![10, 20, 50];
        let report = compute(&r, &scales).unwrap();
        assert!(report.alpha.is_finite());
    }

    #[test]
    fn invalid_scales_ignored() {
        let mut rng = lcg_seed(13);
        let r: Vec<f64> = (0..500).map(|_| rng()).collect();
        // scale=3 (too small) and scale=10000 (too big) should be ignored.
        let scales: Vec<usize> = vec![3, 10, 20, 50, 10_000];
        let report = compute(&r, &scales).unwrap();
        assert!(report.log_n.len() >= 2);
    }
}
