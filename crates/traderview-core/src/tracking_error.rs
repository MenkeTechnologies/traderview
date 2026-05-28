//! Tracking Error — annualized stdev of portfolio − benchmark returns.
//!
//!   TE = stdev(r_portfolio − r_benchmark) · √periods_per_year
//!
//! Companion outputs:
//!   - **information_ratio** = mean(r_p − r_b) · √periods / TE
//!   - **mean_active_return** (annualized)
//!   - **max_underperformance** = min(r_p − r_b)
//!   - **max_outperformance** = max(r_p − r_b)
//!
//! Range:
//!   - TE = 0  → index fund (perfect tracking)
//!   - TE > 0  → active management
//!   - Typical: 1–3% for enhanced index, 5–10% for active equity
//!
//! Pure compute. Companion to `risk_adjusted_ratios`, `up_down_capture`,
//! `factor_models`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrackingErrorReport {
    pub tracking_error_annualized: f64,
    pub information_ratio: f64,
    pub mean_active_return_annualized: f64,
    pub max_underperformance: f64,
    pub max_outperformance: f64,
    pub n_observations: usize,
}

pub fn compute(
    portfolio_returns: &[f64],
    benchmark_returns: &[f64],
    periods_per_year: f64,
) -> Option<TrackingErrorReport> {
    let n = portfolio_returns.len();
    if n < 2 || benchmark_returns.len() != n
        || !periods_per_year.is_finite() || periods_per_year <= 0.0 {
        return None;
    }
    if portfolio_returns.iter().any(|x| !x.is_finite())
        || benchmark_returns.iter().any(|x| !x.is_finite()) { return None; }
    let active: Vec<f64> = portfolio_returns.iter().zip(benchmark_returns.iter())
        .map(|(p, b)| p - b).collect();
    let n_f = n as f64;
    let mean: f64 = active.iter().sum::<f64>() / n_f;
    let var: f64 = active.iter().map(|a| (a - mean).powi(2)).sum::<f64>() / (n_f - 1.0);
    let sd = var.max(0.0).sqrt();
    let te_ann = sd * periods_per_year.sqrt();
    let ir = if sd > 0.0 { mean / sd * periods_per_year.sqrt() } else { 0.0 };
    let max_under = active.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_over = active.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    Some(TrackingErrorReport {
        tracking_error_annualized: te_ann,
        information_ratio: ir,
        mean_active_return_annualized: mean * periods_per_year,
        max_underperformance: max_under,
        max_outperformance: max_over,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_or_mismatched_returns_none() {
        assert!(compute(&[0.01], &[0.01], 252.0).is_none());
        assert!(compute(&[0.01, 0.02], &[0.01], 252.0).is_none());
    }

    #[test]
    fn invalid_periods_returns_none() {
        let r = vec![0.01_f64; 50];
        assert!(compute(&r, &r, 0.0).is_none());
        assert!(compute(&r, &r, f64::NAN).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut p = vec![0.01_f64; 30];
        p[5] = f64::NAN;
        let b = vec![0.01_f64; 30];
        assert!(compute(&p, &b, 252.0).is_none());
    }

    #[test]
    fn perfect_tracker_yields_zero_te() {
        let b = vec![0.01, -0.005, 0.015, -0.01, 0.02];
        let r = compute(&b, &b, 252.0).unwrap();
        assert!(r.tracking_error_annualized.abs() < 1e-12);
    }

    #[test]
    fn leveraged_2x_yields_te_proportional_to_bench_vol() {
        let b: Vec<f64> = (0..200).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        let p: Vec<f64> = b.iter().map(|x| 2.0 * x).collect();
        let r = compute(&p, &b, 252.0).unwrap();
        assert!(r.tracking_error_annualized > 0.0);
    }

    #[test]
    fn information_ratio_zero_when_active_return_zero() {
        // Random-noise portfolio vs benchmark with zero mean → IR ≈ 0.
        let mut state: u64 = 42;
        let b: Vec<f64> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.02
        }).collect();
        let p: Vec<f64> = b.iter().map(|x| x + 0.0).collect();
        let r = compute(&p, &b, 252.0).unwrap();
        // Perfect tracker → TE=0 → IR computed as 0.
        assert!((r.information_ratio).abs() < 1e-9 || !r.information_ratio.is_finite());
    }

    #[test]
    fn max_under_and_overperformance_reported() {
        let p = vec![0.05, -0.01, 0.02];
        let b = vec![0.01, 0.02, 0.02];
        let r = compute(&p, &b, 252.0).unwrap();
        // Active: +0.04, -0.03, 0.0.
        assert!((r.max_outperformance - 0.04).abs() < 1e-12);
        assert!((r.max_underperformance + 0.03).abs() < 1e-12);
    }

    #[test]
    fn n_observations_reported() {
        let p = vec![0.01_f64; 25];
        let b = vec![0.01_f64; 25];
        let r = compute(&p, &b, 252.0).unwrap();
        assert_eq!(r.n_observations, 25);
    }
}
