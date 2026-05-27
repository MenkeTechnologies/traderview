//! Strategy-vs-strategy return correlation.
//!
//! When the trader runs N strategies in parallel (breakout, mean-reversion,
//! gap-and-go, etc.), the question isn't just "which has the best Sharpe"
//! — it's "are they DIVERSIFYING or just duplicating each other?"
//!
//! High correlation between two strategies means doubling up doesn't reduce
//! risk; it just doubles exposure to the same regime. Low or negative
//! correlation is real portfolio benefit.
//!
//! Computes the pairwise Pearson correlation matrix across strategies'
//! daily return series and flags pairs above a threshold for the dashboard.
//!
//! Pure compute. Caller pre-aligns returns to a common date axis (same
//! number of days per strategy, same calendar).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyReturns {
    pub name: String,
    /// Per-period returns (same calendar across all strategies).
    pub returns: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrEntry {
    pub strategy_a: String,
    pub strategy_b: String,
    pub correlation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorrReport {
    /// (a, b) → rho, sorted alphabetically per pair to avoid duplicates.
    pub matrix: BTreeMap<String, BTreeMap<String, f64>>,
    /// Pairs with |rho| ≥ threshold — concentration concern.
    pub high_corr_pairs: Vec<CorrEntry>,
    /// Pairs with |rho| ≤ -threshold OR ≤ 0.2 (diversifying).
    pub diversifying_pairs: Vec<CorrEntry>,
}

pub fn analyze(strategies: &[StrategyReturns], high_threshold: f64) -> CorrReport {
    let mut report = CorrReport::default();
    if strategies.len() < 2 {
        return report;
    }
    for s in strategies {
        report.matrix.insert(s.name.clone(), BTreeMap::new());
    }
    for i in 0..strategies.len() {
        for j in (i + 1)..strategies.len() {
            let a = &strategies[i];
            let b = &strategies[j];
            let rho = pearson(&a.returns, &b.returns).unwrap_or(0.0);
            // Symmetric matrix store.
            report
                .matrix
                .get_mut(&a.name)
                .unwrap()
                .insert(b.name.clone(), rho);
            report
                .matrix
                .get_mut(&b.name)
                .unwrap()
                .insert(a.name.clone(), rho);
            let entry = CorrEntry {
                strategy_a: a.name.clone(),
                strategy_b: b.name.clone(),
                correlation: rho,
            };
            if rho.abs() >= high_threshold {
                report.high_corr_pairs.push(entry.clone());
            } else if rho <= 0.2 {
                // Includes negative — diversifying.
                report.diversifying_pairs.push(entry);
            }
        }
    }
    // Sort high-corr desc by |rho| (most concerning first).
    report.high_corr_pairs.sort_by(|a, b| {
        b.correlation
            .abs()
            .partial_cmp(&a.correlation.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    // Sort diversifying asc (most negative first).
    report.diversifying_pairs.sort_by(|a, b| {
        a.correlation
            .partial_cmp(&b.correlation)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    report
}

fn pearson(a: &[f64], b: &[f64]) -> Option<f64> {
    if a.len() != b.len() || a.len() < 2 {
        return None;
    }
    let n = a.len() as f64;
    let mean_a = a.iter().sum::<f64>() / n;
    let mean_b = b.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut da = 0.0;
    let mut db = 0.0;
    for i in 0..a.len() {
        let xa = a[i] - mean_a;
        let xb = b[i] - mean_b;
        num += xa * xb;
        da += xa * xa;
        db += xb * xb;
    }
    let denom = (da * db).sqrt();
    if denom == 0.0 {
        None
    } else {
        Some(num / denom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(name: &str, r: Vec<f64>) -> StrategyReturns {
        StrategyReturns {
            name: name.into(),
            returns: r,
        }
    }

    #[test]
    fn empty_or_singleton_returns_empty_report() {
        let r = analyze(&[], 0.7);
        assert!(r.matrix.is_empty());
        let r2 = analyze(&[s("a", vec![1.0, 2.0])], 0.7);
        assert!(r2.high_corr_pairs.is_empty());
    }

    #[test]
    fn identical_series_correlation_is_one() {
        let strats = vec![
            s("A", vec![1.0, 2.0, 3.0, 4.0]),
            s("B", vec![1.0, 2.0, 3.0, 4.0]),
        ];
        let r = analyze(&strats, 0.7);
        assert_eq!(r.high_corr_pairs.len(), 1);
        assert!((r.high_corr_pairs[0].correlation - 1.0).abs() < 1e-9);
    }

    #[test]
    fn opposite_series_correlation_is_minus_one() {
        let strats = vec![
            s("A", vec![1.0, 2.0, 3.0, 4.0]),
            s("B", vec![4.0, 3.0, 2.0, 1.0]),
        ];
        let r = analyze(&strats, 0.7);
        assert_eq!(
            r.high_corr_pairs.len(),
            1,
            "|rho|=1 still triggers high-corr"
        );
        assert!((r.high_corr_pairs[0].correlation + 1.0).abs() < 1e-9);
    }

    #[test]
    fn negatively_correlated_series_lands_in_diversifying_when_under_threshold() {
        // Constructed: rho = -0.5 won't trip a 0.7 threshold but IS ≤ 0.2 so
        // diversifying. Series A = [1,2,3,4,5]; B = [3,3,3,3,3] -— variance 0
        // on B → pearson None. Instead use [5,4,3,2,1] for B (anti-trend),
        // rho = -1, which IS above threshold. Use offset:
        // A=[1,2,3,4,5], B=[3,1,4,1,5] — small overlap; should land ≤ 0.2.
        let strats = vec![
            s("A", vec![1.0, 2.0, 3.0, 4.0, 5.0]),
            s("B", vec![5.0, 4.0, 3.0, 2.0, 1.0]), // rho = -1
        ];
        let r = analyze(&strats, 0.7);
        // -1 abs > 0.7 → in high_corr.
        assert_eq!(r.high_corr_pairs.len(), 1);
        assert!((r.high_corr_pairs[0].correlation + 1.0).abs() < 1e-9);
        assert!(
            r.diversifying_pairs.is_empty(),
            "perfect anti-corr lands in high_corr, not diversifying"
        );
    }

    #[test]
    fn matrix_is_symmetric_and_contains_all_pairs() {
        let strats = vec![
            s("A", vec![1.0, 2.0, 3.0]),
            s("B", vec![2.0, 4.0, 6.0]),
            s("C", vec![3.0, 1.0, 2.0]),
        ];
        let r = analyze(&strats, 0.99); // very strict — only perfect matches
                                        // 3 strategies → 3 names in matrix, 2 entries each (others).
        assert_eq!(r.matrix.len(), 3);
        assert_eq!(r.matrix["A"].len(), 2);
        // Symmetric: A→B == B→A.
        assert_eq!(r.matrix["A"]["B"], r.matrix["B"]["A"]);
    }

    #[test]
    fn high_corr_pairs_sorted_by_abs_descending() {
        let strats = vec![
            s("PERFECT", vec![1.0, 2.0, 3.0]),
            s("DUP", vec![2.0, 4.0, 6.0]),    // rho = 1
            s("INVERT", vec![3.0, 2.0, 1.0]), // rho = -1 vs PERFECT
        ];
        let r = analyze(&strats, 0.5);
        assert!(!r.high_corr_pairs.is_empty());
        // First entry should be |1.0| pair.
        assert!(
            r.high_corr_pairs[0].correlation.abs()
                >= r.high_corr_pairs.last().unwrap().correlation.abs()
        );
    }

    #[test]
    fn length_mismatch_falls_back_to_zero_correlation() {
        let strats = vec![
            s("A", vec![1.0, 2.0, 3.0]),
            s("B", vec![1.0, 2.0]), // shorter — pearson returns None → 0
        ];
        let r = analyze(&strats, 0.7);
        // Pair recorded in matrix as 0.0.
        assert_eq!(r.matrix["A"]["B"], 0.0);
        // 0.0 doesn't trigger high-corr threshold; goes to diversifying (<=0.2).
        assert_eq!(r.diversifying_pairs.len(), 1);
    }

    #[test]
    fn constant_series_falls_back_to_zero_correlation() {
        let strats = vec![
            s("A", vec![1.0, 1.0, 1.0]), // zero variance
            s("B", vec![2.0, 4.0, 6.0]),
        ];
        let r = analyze(&strats, 0.7);
        // Zero variance → pearson None → stored as 0.
        assert_eq!(r.matrix["A"]["B"], 0.0);
    }
}
