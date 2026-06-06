//! Hodrick-Prescott filter — trend extraction from time series.
//!
//! Decomposes a series y_t into a smooth trend τ_t and a cycle c_t
//! by minimizing:
//!
//!   Σ (y_t − τ_t)² + λ · Σ [(τ_{t+1} − τ_t) − (τ_t − τ_{t−1})]²
//!
//! The first term penalizes deviation from the data; the second
//! penalizes "kinks" in the trend. Larger λ → smoother trend. Standard
//! choices:
//!   - λ = 100 for annual data
//!   - λ = 1600 for quarterly (Ravn & Uhlig 2002)
//!   - λ = 14400 for monthly
//!   - λ = 129600 for daily
//!
//! Closed-form solution: τ = (I + λ·F'F)⁻¹ · y, where F is the second-
//! difference matrix. We solve via direct Gauss-Jordan inversion of
//! the small pentadiagonal system (~O(n²) memory acceptable for n ≤ 1k).
//!
//! Pure compute. Caller picks λ; typical default = 1600.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HpReport {
    pub trend: Vec<f64>,
    pub cycle: Vec<f64>,
}

pub fn compute(series: &[f64], lambda: f64) -> Option<HpReport> {
    let n = series.len();
    if n < 4 || !lambda.is_finite() || lambda <= 0.0 || series.iter().any(|x| !x.is_finite()) {
        return None;
    }
    if n > 1_000 {
        // O(n²) inversion gets expensive; reject pathological sizes for
        // now. Caller should downsample or chunk for longer series.
        return None;
    }
    // Build (I + λ·F'·F) where F (n−2 × n) is the 2nd-difference matrix:
    //   F[i, i] = 1, F[i, i+1] = -2, F[i, i+2] = 1
    let mut m = vec![vec![0.0_f64; n]; n];
    for (i, row) in m.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    // λ·F'·F is the pentadiagonal "smoothness penalty" matrix.
    // For each row i in F, contributes λ to:
    //   m[i][i] += λ;   m[i][i+1] -= 2λ;  m[i][i+2] += λ
    //   m[i+1][i] -= 2λ; m[i+1][i+1] += 4λ; m[i+1][i+2] -= 2λ
    //   m[i+2][i] += λ;   m[i+2][i+1] -= 2λ; m[i+2][i+2] += λ
    for i in 0..(n - 2) {
        m[i][i] += lambda;
        m[i][i + 1] -= 2.0 * lambda;
        m[i][i + 2] += lambda;
        m[i + 1][i] -= 2.0 * lambda;
        m[i + 1][i + 1] += 4.0 * lambda;
        m[i + 1][i + 2] -= 2.0 * lambda;
        m[i + 2][i] += lambda;
        m[i + 2][i + 1] -= 2.0 * lambda;
        m[i + 2][i + 2] += lambda;
    }
    let trend = solve_linear(&m, series)?;
    let cycle: Vec<f64> = series
        .iter()
        .zip(trend.iter())
        .map(|(y, t)| y - t)
        .collect();
    Some(HpReport { trend, cycle })
}

fn solve_linear(m: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let n = m.len();
    if n == 0 || m.iter().any(|r| r.len() != n) || y.len() != n {
        return None;
    }
    let mut aug = vec![vec![0.0_f64; n + 1]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = m[i][j];
        }
        aug[i][n] = y[i];
    }
    for i in 0..n {
        let mut pivot = i;
        for r in (i + 1)..n {
            if aug[r][i].abs() > aug[pivot][i].abs() {
                pivot = r;
            }
        }
        if aug[pivot][i].abs() < 1e-18 {
            return None;
        }
        aug.swap(i, pivot);
        let div = aug[i][i];
        for v in aug[i].iter_mut() {
            *v /= div;
        }
        for r in 0..n {
            if r == i {
                continue;
            }
            let f = aug[r][i];
            if f == 0.0 {
                continue;
            }
            let pivot_row = aug[i].clone();
            for (j, v) in aug[r].iter_mut().enumerate() {
                *v -= f * pivot_row[j];
            }
        }
    }
    Some((0..n).map(|i| aug[i][n]).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[1.0; 3], 1600.0).is_none());
    }

    #[test]
    fn invalid_lambda_returns_none() {
        assert!(compute(&[1.0; 10], 0.0).is_none());
        assert!(compute(&[1.0; 10], -100.0).is_none());
        assert!(compute(&[1.0; 10], f64::NAN).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let mut s = vec![1.0; 10];
        s[5] = f64::NAN;
        assert!(compute(&s, 1600.0).is_none());
    }

    #[test]
    fn flat_series_yields_flat_trend_and_zero_cycle() {
        let s = vec![100.0; 50];
        let r = compute(&s, 1600.0).unwrap();
        for t in &r.trend {
            assert!((t - 100.0).abs() < 1e-9);
        }
        for c in &r.cycle {
            assert!(c.abs() < 1e-9);
        }
    }

    #[test]
    fn linear_trend_recovered_exactly() {
        // HP filter recovers a linear trend exactly (zero curvature → no penalty).
        let s: Vec<f64> = (0..50).map(|i| 100.0 + 2.0 * i as f64).collect();
        let r = compute(&s, 1600.0).unwrap();
        for (y, t) in s.iter().zip(r.trend.iter()) {
            assert!((y - t).abs() < 1e-6, "trend should equal data: y={y} t={t}");
        }
    }

    #[test]
    fn higher_lambda_yields_smoother_trend() {
        let mut state: u64 = 42;
        let s: Vec<f64> = (0..50)
            .map(|i| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let noise = ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 5.0;
                100.0 + i as f64 * 0.5 + noise
            })
            .collect();
        let r_low = compute(&s, 100.0).unwrap();
        let r_high = compute(&s, 1_000_000.0).unwrap();
        // Sum of squared 2nd-differences of the trend should be smaller
        // for the larger-λ run.
        let smoothness = |trend: &[f64]| -> f64 {
            let mut sum = 0.0;
            for i in 1..trend.len() - 1 {
                let dd = trend[i + 1] - 2.0 * trend[i] + trend[i - 1];
                sum += dd * dd;
            }
            sum
        };
        assert!(
            smoothness(&r_high.trend) < smoothness(&r_low.trend),
            "high-λ trend should be smoother"
        );
    }

    #[test]
    fn cycle_plus_trend_equals_series() {
        let s: Vec<f64> = (0..20)
            .map(|i| (i as f64 * 0.1).sin() * 5.0 + 100.0)
            .collect();
        let r = compute(&s, 1600.0).unwrap();
        for (i, y) in s.iter().enumerate() {
            assert!((y - r.trend[i] - r.cycle[i]).abs() < 1e-9);
        }
    }

    #[test]
    fn very_long_series_rejected() {
        let s = vec![100.0; 1_500];
        assert!(compute(&s, 1600.0).is_none());
    }
}
