//! PELT (Pruned Exact Linear Time) Changepoint Detection —
//! Killick, Fearnhead & Eckley (2012).
//!
//! Detects K* multiple changepoints in a time series that minimize:
//!
//!   Σ_segments cost(segment) + penalty · K
//!
//! Pruning rule allows expected O(n) runtime vs O(n²) for naive DP
//! while remaining exact under mild conditions.
//!
//! Default cost: per-segment sum of squared deviations from the
//! segment mean (Gaussian likelihood for known variance):
//!
//!   cost([a, b)) = Σ_{t=a..b} (x_t − mean_{a..b})²
//!
//! Penalty: BIC-style `2σ² · log(n)` if `penalty = None`. Otherwise
//! caller-supplied; lower penalty = more changepoints, higher = fewer.
//!
//! Pure compute. Companion to `cusum`, `kpss_test`, `chow_test`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeltReport {
    /// Indices where each segment starts (always includes 0).
    pub segment_starts: Vec<usize>,
    /// Per-segment mean.
    pub segment_means: Vec<f64>,
    pub n_changepoints: usize,
    pub total_cost: f64,
    pub penalty_used: f64,
    pub n_observations: usize,
}

pub fn detect(series: &[f64], penalty: Option<f64>) -> Option<PeltReport> {
    let n = series.len();
    if n < 4 { return None; }
    if series.iter().any(|x| !x.is_finite()) { return None; }
    // Default penalty: 2·σ²·ln(n).
    let mean: f64 = series.iter().sum::<f64>() / n as f64;
    let var: f64 = series.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    let lambda = penalty.unwrap_or(2.0 * var.max(1e-12) * (n as f64).ln());
    // Precompute cumulative sums for O(1) segment cost queries:
    //   cost([a, b)) = Σ x² − (Σ x)² / (b − a)
    let mut csum = vec![0.0_f64; n + 1];
    let mut csum_sq = vec![0.0_f64; n + 1];
    for i in 0..n {
        csum[i + 1] = csum[i] + series[i];
        csum_sq[i + 1] = csum_sq[i] + series[i] * series[i];
    }
    let seg_cost = |a: usize, b: usize| -> f64 {
        let s = csum[b] - csum[a];
        let ss = csum_sq[b] - csum_sq[a];
        let len = (b - a) as f64;
        ss - s * s / len
    };
    // PELT DP.
    let mut f = vec![0.0_f64; n + 1];
    f[0] = -lambda;
    let mut prev_cp = vec![0_usize; n + 1];
    let mut candidates: Vec<usize> = vec![0];
    for t in 1..=n {
        let mut best_cost = f64::INFINITY;
        let mut best_tau = 0_usize;
        for &tau in &candidates {
            let c = f[tau] + seg_cost(tau, t) + lambda;
            if c < best_cost {
                best_cost = c;
                best_tau = tau;
            }
        }
        f[t] = best_cost;
        prev_cp[t] = best_tau;
        // Prune: keep only τ where f[τ] + cost(τ, t) ≤ f[t].
        let f_t = f[t];
        candidates.retain(|&tau| f[tau] + seg_cost(tau, t) <= f_t);
        candidates.push(t);
    }
    // Backtrack.
    let mut starts: Vec<usize> = Vec::new();
    let mut t = n;
    while t > 0 {
        let tau = prev_cp[t];
        starts.push(tau);
        if tau == 0 { break; }
        t = tau;
    }
    starts.reverse();
    if starts.first() != Some(&0) { starts.insert(0, 0); }
    let mut means = Vec::with_capacity(starts.len());
    for (i, &start) in starts.iter().enumerate() {
        let end = if i + 1 < starts.len() { starts[i + 1] } else { n };
        let seg_sum: f64 = (csum[end] - csum[start]) / (end - start) as f64;
        means.push(seg_sum);
    }
    Some(PeltReport {
        n_changepoints: starts.len() - 1,
        segment_starts: starts,
        segment_means: means,
        total_cost: f[n],
        penalty_used: lambda,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(detect(&[1.0, 2.0], None).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(detect(&[1.0, f64::NAN, 3.0, 4.0], None).is_none());
    }

    #[test]
    fn flat_series_yields_single_segment() {
        let s = vec![5.0_f64; 50];
        let r = detect(&s, None).unwrap();
        assert_eq!(r.n_changepoints, 0);
        assert_eq!(r.segment_starts.len(), 1);
        assert!((r.segment_means[0] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn clear_step_change_detected() {
        // Flat 0 for 50 bars, then flat 10 for 50 bars.
        let mut s = vec![0.0_f64; 50];
        s.extend(vec![10.0_f64; 50]);
        let r = detect(&s, None).unwrap();
        assert!(r.n_changepoints >= 1,
            "expected ≥1 changepoint, got {}", r.n_changepoints);
        // Some segment start should be near index 50.
        let near_50 = r.segment_starts.iter().any(|s| s.abs_diff(50) <= 3);
        assert!(near_50, "no segment start near index 50: {:?}", r.segment_starts);
    }

    #[test]
    fn higher_penalty_yields_fewer_segments() {
        let mut s = vec![0.0_f64; 30];
        s.extend(vec![5.0_f64; 30]);
        s.extend(vec![0.0_f64; 30]);
        let low_pen = detect(&s, Some(1.0)).unwrap();
        let high_pen = detect(&s, Some(1000.0)).unwrap();
        assert!(low_pen.n_changepoints >= high_pen.n_changepoints,
            "high penalty {} should yield ≤ low-penalty CPs {}",
            high_pen.n_changepoints, low_pen.n_changepoints);
    }

    #[test]
    fn segment_means_match_data() {
        let mut s = vec![0.0_f64; 30];
        s.extend(vec![10.0_f64; 30]);
        let r = detect(&s, None).unwrap();
        if r.n_changepoints >= 1 {
            for (i, &start) in r.segment_starts.iter().enumerate() {
                let end = if i + 1 < r.segment_starts.len() {
                    r.segment_starts[i + 1]
                } else { s.len() };
                let actual_mean: f64 = s[start..end].iter().sum::<f64>()
                    / (end - start) as f64;
                assert!((r.segment_means[i] - actual_mean).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn n_observations_reported() {
        let s = vec![1.0_f64; 25];
        let r = detect(&s, None).unwrap();
        assert_eq!(r.n_observations, 25);
    }
}
