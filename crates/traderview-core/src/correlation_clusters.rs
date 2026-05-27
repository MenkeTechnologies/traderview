//! Position correlation clustering.
//!
//! When the trader holds AAPL + MSFT + GOOGL + META, that looks like
//! "4 positions" but it's really 1 mega-cap-tech bet. This module takes
//! a pairwise correlation matrix and groups positions whose pairwise
//! correlation exceeds a threshold, exposing disguised concentration.
//!
//! Algorithm: single-link agglomerative clustering. Two positions go in
//! the same cluster if there's a chain of pairs each above the threshold.
//! Fast for the small N (typically < 50 positions) the user holds.
//!
//! Pure compute. Caller supplies the correlation matrix (computed
//! externally from returns history via crate::correlation).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    /// Notional exposure in dollars, signed (long > 0, short < 0).
    pub notional: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub members: Vec<String>,
    /// Sum of |notional| across cluster — total $ exposure to the
    /// shared underlying factor.
    pub gross_exposure: f64,
    /// Signed sum: positive if cluster is net-long the factor, negative
    /// if net-short.
    pub net_exposure: f64,
}

/// Build clusters where every member shares a correlation chain ≥ `threshold`
/// with at least one other member.
///
/// `corr` is the (i, j) -> rho lookup. Symmetric — caller fills both
/// orderings or this honors whichever appears first.
pub fn cluster(positions: &[Position], corr: &BTreeMap<(String, String), f64>, threshold: f64)
    -> Vec<Cluster>
{
    let n = positions.len();
    if n == 0 { return vec![]; }
    let mut parent: Vec<usize> = (0..n).collect();
    let idx_of: BTreeMap<&str, usize> = positions.iter().enumerate()
        .map(|(i, p)| (p.symbol.as_str(), i))
        .collect();
    for i in 0..n {
        for j in (i+1)..n {
            let a = &positions[i].symbol;
            let b = &positions[j].symbol;
            let rho = corr.get(&(a.clone(), b.clone()))
                .or_else(|| corr.get(&(b.clone(), a.clone())))
                .copied()
                .unwrap_or(0.0);
            if rho.abs() >= threshold {
                union(&mut parent, *idx_of.get(a.as_str()).unwrap(), *idx_of.get(b.as_str()).unwrap());
            }
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }
    let mut out: Vec<Cluster> = groups.into_values().map(|idxs| {
        let members: Vec<String> = idxs.iter().map(|&i| positions[i].symbol.clone()).collect();
        let net: f64 = idxs.iter().map(|&i| positions[i].notional).sum();
        let gross: f64 = idxs.iter().map(|&i| positions[i].notional.abs()).sum();
        Cluster { members, gross_exposure: gross, net_exposure: net }
    }).collect();
    // Sort: largest gross-exposure cluster first.
    out.sort_by(|a, b| b.gross_exposure.partial_cmp(&a.gross_exposure)
        .unwrap_or(std::cmp::Ordering::Equal));
    out
}

fn find(p: &mut [usize], i: usize) -> usize {
    let mut r = i;
    while p[r] != r { r = p[r]; }
    // Path compression.
    let mut cur = i;
    while p[cur] != r {
        let next = p[cur];
        p[cur] = r;
        cur = next;
    }
    r
}

fn union(p: &mut [usize], a: usize, b: usize) {
    let ra = find(p, a);
    let rb = find(p, b);
    if ra != rb { p[ra] = rb; }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(sym: &str, notional: f64) -> Position {
        Position { symbol: sym.into(), notional }
    }
    fn pair(a: &str, b: &str, rho: f64) -> ((String, String), f64) {
        ((a.into(), b.into()), rho)
    }

    #[test]
    fn empty_input_returns_empty() {
        let out = cluster(&[], &BTreeMap::new(), 0.7);
        assert!(out.is_empty());
    }

    #[test]
    fn isolated_positions_become_singletons() {
        let positions = vec![pos("AAPL", 10_000.0), pos("XOM", 5_000.0)];
        let corr: BTreeMap<_, _> = [pair("AAPL", "XOM", 0.10)].into_iter().collect();
        let out = cluster(&positions, &corr, 0.7);
        assert_eq!(out.len(), 2, "uncorrelated positions stay separate");
    }

    #[test]
    fn high_correlation_pair_gets_grouped() {
        let positions = vec![pos("AAPL", 10_000.0), pos("MSFT", 8_000.0)];
        let corr: BTreeMap<_, _> = [pair("AAPL", "MSFT", 0.85)].into_iter().collect();
        let out = cluster(&positions, &corr, 0.7);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].members.len(), 2);
        assert_eq!(out[0].gross_exposure, 18_000.0);
        assert_eq!(out[0].net_exposure, 18_000.0);
    }

    #[test]
    fn chain_correlation_via_transitive_link() {
        // A-B = 0.8 (linked), B-C = 0.8 (linked), A-C = 0.0 (unlinked direct).
        // Single-link clustering says A, B, C are ALL in one cluster.
        let positions = vec![pos("A", 1_000.0), pos("B", 1_000.0), pos("C", 1_000.0)];
        let corr: BTreeMap<_, _> = [
            pair("A", "B", 0.8),
            pair("B", "C", 0.8),
            pair("A", "C", 0.0),
        ].into_iter().collect();
        let out = cluster(&positions, &corr, 0.7);
        assert_eq!(out.len(), 1, "transitive chain links A-B-C into one cluster");
        assert_eq!(out[0].members.len(), 3);
    }

    #[test]
    fn negative_correlation_above_threshold_also_clusters() {
        // SQQQ inverse-3x to QQQ. rho = -0.95 → |rho| = 0.95 → cluster
        // (same underlying factor, just opposite direction).
        let positions = vec![pos("QQQ", 10_000.0), pos("SQQQ", -5_000.0)];
        let corr: BTreeMap<_, _> = [pair("QQQ", "SQQQ", -0.95)].into_iter().collect();
        let out = cluster(&positions, &corr, 0.7);
        assert_eq!(out.len(), 1);
        // Net exposure: 10000 + (-5000) = 5000 (net long the QQQ factor).
        assert_eq!(out[0].net_exposure, 5_000.0);
        assert_eq!(out[0].gross_exposure, 15_000.0);
    }

    #[test]
    fn symbol_pair_lookup_is_order_independent() {
        let positions = vec![pos("X", 1.0), pos("Y", 1.0)];
        // Only (Y, X) provided — not (X, Y).
        let corr: BTreeMap<_, _> = [pair("Y", "X", 0.9)].into_iter().collect();
        let out = cluster(&positions, &corr, 0.7);
        assert_eq!(out.len(), 1, "pair lookup must try both orderings");
    }

    #[test]
    fn missing_pair_defaults_to_zero_correlation() {
        let positions = vec![pos("X", 1.0), pos("Y", 1.0)];
        let corr: BTreeMap<_, _> = BTreeMap::new();   // none provided
        let out = cluster(&positions, &corr, 0.7);
        assert_eq!(out.len(), 2, "absent pair → rho=0 → no cluster");
    }

    #[test]
    fn clusters_sorted_by_gross_exposure_descending() {
        // Cluster 1: AAPL+MSFT (18k gross). Cluster 2: solo XOM (5k).
        let positions = vec![
            pos("XOM", 5_000.0),
            pos("AAPL", 10_000.0),
            pos("MSFT", 8_000.0),
        ];
        let corr: BTreeMap<_, _> = [pair("AAPL", "MSFT", 0.9)].into_iter().collect();
        let out = cluster(&positions, &corr, 0.7);
        assert_eq!(out[0].gross_exposure, 18_000.0);
        assert_eq!(out[1].gross_exposure, 5_000.0);
    }

    #[test]
    fn threshold_exactly_at_correlation_value_clusters() {
        // rho == threshold → must cluster (>=, not >).
        let positions = vec![pos("A", 1.0), pos("B", 1.0)];
        let corr: BTreeMap<_, _> = [pair("A", "B", 0.7)].into_iter().collect();
        let out = cluster(&positions, &corr, 0.7);
        assert_eq!(out.len(), 1, "threshold is inclusive");
    }
}
