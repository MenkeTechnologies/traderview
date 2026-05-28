//! Rank correlations — Spearman ρ and Kendall τ.
//!
//! Both measure monotonic (not necessarily linear) association.
//! Spearman = Pearson correlation of the rank-transformed data.
//! Kendall τ = (n_concordant − n_discordant) / (n choose 2) — robust
//! to outliers and small-sample-size; common in pairs-trading
//! association tests and copula calibration.
//!
//! Both ∈ [−1, +1]. Independence ⇒ both ≈ 0 in expectation.
//!
//! Pure compute. Caller supplies equal-length arrays of finite values.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RankCorrReport {
    pub spearman_rho: f64,
    pub kendall_tau: f64,
    pub n_observations: usize,
    pub n_concordant_pairs: usize,
    pub n_discordant_pairs: usize,
    pub n_tied_pairs: usize,
}

pub fn compute(x: &[f64], y: &[f64]) -> Option<RankCorrReport> {
    let n = x.len();
    if y.len() != n || n < 3 {
        return None;
    }
    // Filter to finite pairs.
    let clean: Vec<(f64, f64)> = x.iter().zip(y.iter())
        .filter(|(a, b)| a.is_finite() && b.is_finite())
        .map(|(a, b)| (*a, *b))
        .collect();
    let m = clean.len();
    if m < 3 { return None; }
    // Spearman ρ via Pearson on ranks (average-tied ranks).
    let xs: Vec<f64> = clean.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = clean.iter().map(|p| p.1).collect();
    let rx = average_ranks(&xs);
    let ry = average_ranks(&ys);
    let spearman = pearson(&rx, &ry)?;
    // Kendall τ — O(n²) sweep over all pairs.
    let mut conc = 0_usize;
    let mut disc = 0_usize;
    let mut tied = 0_usize;
    for i in 0..m {
        for j in (i + 1)..m {
            let dx = clean[i].0 - clean[j].0;
            let dy = clean[i].1 - clean[j].1;
            if dx == 0.0 || dy == 0.0 { tied += 1; continue; }
            if dx.signum() == dy.signum() { conc += 1; }
            else { disc += 1; }
        }
    }
    let total_pairs = m * (m - 1) / 2;
    let tau = if total_pairs > 0 {
        (conc as f64 - disc as f64) / total_pairs as f64
    } else { 0.0 };
    Some(RankCorrReport {
        spearman_rho: spearman,
        kendall_tau: tau,
        n_observations: m,
        n_concordant_pairs: conc,
        n_discordant_pairs: disc,
        n_tied_pairs: tied,
    })
}

fn average_ranks(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut indexed: Vec<(usize, f64)> = values.iter().enumerate().map(|(i, v)| (i, *v)).collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0_f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && (indexed[j].1 - indexed[i].1).abs() < f64::EPSILON {
            j += 1;
        }
        // ties span [i, j). Their average rank is ((i+1) + j) / 2.
        let avg_rank = (((i + 1) + j) as f64) / 2.0;
        for k in i..j {
            ranks[indexed[k].0] = avg_rank;
        }
        i = j;
    }
    ranks
}

fn pearson(a: &[f64], b: &[f64]) -> Option<f64> {
    let n = a.len();
    if n != b.len() || n < 2 { return None; }
    let n_f = n as f64;
    let mean_a = a.iter().sum::<f64>() / n_f;
    let mean_b = b.iter().sum::<f64>() / n_f;
    let mut sxy = 0.0_f64;
    let mut sxx = 0.0_f64;
    let mut syy = 0.0_f64;
    for (x, y) in a.iter().zip(b.iter()) {
        let dx = x - mean_a;
        let dy = y - mean_b;
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx <= 0.0 || syy <= 0.0 { return None; }
    let r = sxy / (sxx * syy).sqrt();
    if r.is_finite() { Some(r.clamp(-1.0, 1.0)) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_mismatch_returns_none() {
        assert!(compute(&[1.0, 2.0, 3.0], &[1.0, 2.0]).is_none());
    }

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[1.0, 2.0], &[1.0, 2.0]).is_none());
    }

    #[test]
    fn perfect_increasing_yields_one() {
        let x: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let y: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let r = compute(&x, &y).unwrap();
        assert!((r.spearman_rho - 1.0).abs() < 1e-12);
        assert!((r.kendall_tau - 1.0).abs() < 1e-12);
    }

    #[test]
    fn perfect_decreasing_yields_minus_one() {
        let x: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let y: Vec<f64> = (1..=10).map(|i| -(i as f64)).collect();
        let r = compute(&x, &y).unwrap();
        assert!((r.spearman_rho + 1.0).abs() < 1e-12);
        assert!((r.kendall_tau + 1.0).abs() < 1e-12);
    }

    #[test]
    fn monotone_nonlinear_yields_spearman_one() {
        // Spearman captures monotonic, not just linear. y = x³ is monotone increasing.
        let x: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|v| v.powi(3)).collect();
        let r = compute(&x, &y).unwrap();
        assert!((r.spearman_rho - 1.0).abs() < 1e-9);
        assert!((r.kendall_tau - 1.0).abs() < 1e-9);
    }

    #[test]
    fn tied_values_handled() {
        let x = vec![1.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![5.0, 6.0, 7.0, 7.0, 8.0];
        let r = compute(&x, &y).unwrap();
        assert!(r.spearman_rho.is_finite());
        assert!(r.kendall_tau.is_finite());
        assert!(r.n_tied_pairs > 0);
    }

    #[test]
    fn independent_data_yields_low_magnitude_correlation() {
        let mut state: u64 = 42;
        let mut x = Vec::with_capacity(200);
        let mut y = Vec::with_capacity(200);
        for _ in 0..200 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            x.push((state >> 32) as f64 / u32::MAX as f64);
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            y.push((state >> 32) as f64 / u32::MAX as f64);
        }
        let r = compute(&x, &y).unwrap();
        assert!(r.spearman_rho.abs() < 0.2,
            "spearman of independent series should be small, got {}", r.spearman_rho);
        assert!(r.kendall_tau.abs() < 0.2);
    }

    #[test]
    fn nan_pairs_skipped() {
        let x = vec![1.0, f64::NAN, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = compute(&x, &y).unwrap();
        assert_eq!(r.n_observations, 4);
    }
}
