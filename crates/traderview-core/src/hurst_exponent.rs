//! Hurst Exponent (R/S analysis) — long-memory persistence estimator.
//!
//! H < 0.5 = mean-reverting series (anti-persistent)
//! H ≈ 0.5 = random walk (no memory)
//! H > 0.5 = trending series (persistent)
//!
//! Procedure (rescaled-range / Mandelbrot R/S): for each chunk size n
//! in `chunk_sizes`, split returns into ⌊len/n⌋ non-overlapping chunks,
//! compute per-chunk mean-deviated cumulative series Y with range
//! R = max(Y) − min(Y) and stdev S, then average R/S across all chunks
//! of that size to get (R/S)_n. Finally, linear-regress log(R/S)_n vs
//! log(n); the slope is H.
//!
//! Pure compute. Returns the Hurst estimate and the regression points
//! so the caller can sanity-check the fit quality.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HurstReport {
    pub hurst: f64,
    pub log_n: Vec<f64>,
    pub log_rs: Vec<f64>,
    pub r_squared: f64,
}

pub fn compute(returns: &[f64], chunk_sizes: &[usize]) -> Option<HurstReport> {
    let n = returns.len();
    if n < 10 || chunk_sizes.is_empty() {
        return None;
    }
    // Filter input to finite returns.
    let r: Vec<f64> = returns.iter().copied().filter(|x| x.is_finite()).collect();
    if r.len() < 10 {
        return None;
    }
    let mut points: Vec<(f64, f64)> = Vec::new();
    for &chunk in chunk_sizes {
        if chunk < 4 || chunk > r.len() {
            continue;
        }
        let n_chunks = r.len() / chunk;
        if n_chunks == 0 {
            continue;
        }
        let mut rs_sum = 0.0_f64;
        let mut rs_count = 0usize;
        for c in 0..n_chunks {
            let lo = c * chunk;
            let chunk_slice = &r[lo..lo + chunk];
            let mean = chunk_slice.iter().sum::<f64>() / chunk as f64;
            let dev: Vec<f64> = chunk_slice.iter().map(|x| x - mean).collect();
            // Cumulative sum.
            let mut cum = 0.0;
            let mut max_y = f64::NEG_INFINITY;
            let mut min_y = f64::INFINITY;
            for d in &dev {
                cum += d;
                if cum > max_y {
                    max_y = cum;
                }
                if cum < min_y {
                    min_y = cum;
                }
            }
            let r_range = max_y - min_y;
            // Sample stdev (n-1 denom) for robustness — match canonical Mandelbrot.
            let var = dev.iter().map(|x| x * x).sum::<f64>() / (chunk as f64 - 1.0);
            let s = var.sqrt();
            if s > 0.0 && r_range.is_finite() {
                rs_sum += r_range / s;
                rs_count += 1;
            }
        }
        if rs_count > 0 {
            let avg_rs = rs_sum / rs_count as f64;
            if avg_rs > 0.0 && avg_rs.is_finite() {
                points.push((chunk as f64, avg_rs));
            }
        }
    }
    if points.len() < 2 {
        return None;
    }
    let log_n: Vec<f64> = points.iter().map(|p| p.0.ln()).collect();
    let log_rs: Vec<f64> = points.iter().map(|p| p.1.ln()).collect();
    // OLS: slope = Σ(x − x̄)(y − ȳ) / Σ(x − x̄)²
    let m = log_n.len() as f64;
    let x_mean = log_n.iter().sum::<f64>() / m;
    let y_mean = log_rs.iter().sum::<f64>() / m;
    let mut num = 0.0_f64;
    let mut den = 0.0_f64;
    let mut ss_tot = 0.0_f64;
    for (x, y) in log_n.iter().zip(log_rs.iter()) {
        let dx = x - x_mean;
        let dy = y - y_mean;
        num += dx * dy;
        den += dx * dx;
        ss_tot += dy * dy;
    }
    if den <= 0.0 {
        return None;
    }
    let slope = num / den;
    let intercept = y_mean - slope * x_mean;
    let mut ss_res = 0.0_f64;
    for (x, y) in log_n.iter().zip(log_rs.iter()) {
        let predicted = slope * x + intercept;
        ss_res += (y - predicted).powi(2);
    }
    let r_squared = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };
    if !slope.is_finite() {
        return None;
    }
    Some(HurstReport {
        hurst: slope,
        log_n,
        log_rs,
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
    fn empty_returns_none() {
        assert!(compute(&[], &[10, 20]).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01; 5], &[10, 20]).is_none());
    }

    #[test]
    fn no_chunk_sizes_returns_none() {
        assert!(compute(&[0.01; 100], &[]).is_none());
    }

    #[test]
    fn random_walk_increments_near_half() {
        // Pseudo-random uniform mean-zero series should yield H ≈ 0.5.
        let mut rng = lcg_seed(42);
        let r: Vec<f64> = (0..5_000).map(|_| rng()).collect();
        let chunks: Vec<usize> = vec![10, 20, 50, 100, 250, 500];
        let report = compute(&r, &chunks).expect("populated");
        // R/S is biased on finite samples — allow generous band.
        assert!(
            (report.hurst - 0.5).abs() < 0.15,
            "random walk should yield H ≈ 0.5, got {}",
            report.hurst
        );
    }

    #[test]
    fn strong_trend_yields_h_above_half() {
        // Strictly increasing series — every increment positive → persistent.
        let r: Vec<f64> = (0..2_000).map(|i| 0.001 + (i as f64 * 0.0001)).collect();
        let chunks: Vec<usize> = vec![10, 20, 50, 100, 250];
        let report = compute(&r, &chunks).expect("populated");
        assert!(
            report.hurst > 0.5,
            "trending series should yield H > 0.5, got {}",
            report.hurst
        );
    }

    #[test]
    fn nan_inputs_filtered() {
        let mut r: Vec<f64> = (0..500).map(|i| (i as f64 * 0.01).sin()).collect();
        r[100] = f64::NAN;
        let report = compute(&r, &[10, 20, 50]).expect("populated");
        assert!(report.hurst.is_finite());
    }

    #[test]
    fn r_squared_in_unit_range() {
        let mut rng = lcg_seed(123);
        let r: Vec<f64> = (0..1_000).map(|_| rng()).collect();
        let report = compute(&r, &[10, 20, 50, 100]).expect("populated");
        assert!((-1.0..=1.0).contains(&report.r_squared));
    }
}
