//! Equal highs / equal lows detector — liquidity-pool finder.
//!
//! "Equal highs" (EQH) and "equal lows" (EQL) are clusters of swing pivots
//! at near-identical prices. SMC theory treats these as liquidity pools:
//! every retail trader sees the same resistance/support line, so stop
//! orders accumulate just beyond. Algos sweep these levels to trigger
//! the stops, then reverse.
//!
//! Caller supplies swing points (from `crate::swing_points`); we group
//! pivots that fall within a configurable `cluster_tolerance` window.
//! Output: clusters of ≥ `min_cluster_size` matching pivots, with the
//! reference price and the bar indices of each member.
//!
//! Pure compute.

use crate::swing_points::{SwingKind, SwingPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualLevelsConfig {
    /// Maximum price-units a pivot can deviate from a cluster's anchor
    /// price and still belong to that cluster. Caller scales by tick
    /// size or ATR.
    pub cluster_tolerance: f64,
    /// Minimum number of pivots required to form a cluster.
    pub min_cluster_size: usize,
}

impl Default for EqualLevelsConfig {
    fn default() -> Self {
        Self {
            cluster_tolerance: 0.10,
            min_cluster_size: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LevelKind {
    EqualHighs,
    EqualLows,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelCluster {
    pub kind: LevelKind,
    /// The average price of all member pivots.
    pub reference_price: f64,
    /// Bar indices where each member pivot appeared.
    pub member_indices: Vec<usize>,
    /// Min / max price across cluster members.
    pub min_price: f64,
    pub max_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LevelsReport {
    pub clusters: Vec<LevelCluster>,
    pub n_equal_highs: usize,
    pub n_equal_lows: usize,
}

pub fn detect(swings: &[SwingPoint], cfg: &EqualLevelsConfig) -> LevelsReport {
    if swings.is_empty() || cfg.cluster_tolerance < 0.0 || cfg.min_cluster_size < 2 {
        return LevelsReport::default();
    }
    let highs: Vec<&SwingPoint> = swings
        .iter()
        .filter(|s| matches!(s.kind, SwingKind::High))
        .collect();
    let lows: Vec<&SwingPoint> = swings
        .iter()
        .filter(|s| matches!(s.kind, SwingKind::Low))
        .collect();

    let high_clusters = cluster_pivots(&highs, cfg, LevelKind::EqualHighs);
    let low_clusters = cluster_pivots(&lows, cfg, LevelKind::EqualLows);

    let n_high = high_clusters.len();
    let n_low = low_clusters.len();

    let mut clusters = high_clusters;
    clusters.extend(low_clusters);
    LevelsReport {
        clusters,
        n_equal_highs: n_high,
        n_equal_lows: n_low,
    }
}

fn cluster_pivots(
    pivots: &[&SwingPoint],
    cfg: &EqualLevelsConfig,
    kind: LevelKind,
) -> Vec<LevelCluster> {
    // Sort by price so neighboring pivots are adjacent.
    let mut sorted: Vec<&SwingPoint> = pivots.to_vec();
    sorted.sort_by(|a, b| {
        a.price
            .partial_cmp(&b.price)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut clusters = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let anchor = sorted[i].price;
        let mut members = vec![sorted[i]];
        let mut j = i + 1;
        while j < sorted.len() && (sorted[j].price - anchor).abs() <= cfg.cluster_tolerance {
            members.push(sorted[j]);
            j += 1;
        }
        if members.len() >= cfg.min_cluster_size {
            let mut indices: Vec<usize> = members.iter().map(|s| s.index).collect();
            indices.sort_unstable();
            let prices: Vec<f64> = members.iter().map(|s| s.price).collect();
            let avg = prices.iter().sum::<f64>() / prices.len() as f64;
            let min_p = prices.iter().copied().fold(f64::INFINITY, f64::min);
            let max_p = prices.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            clusters.push(LevelCluster {
                kind,
                reference_price: avg,
                member_indices: indices,
                min_price: min_p,
                max_price: max_p,
            });
        }
        i = j;
    }
    clusters
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sp(idx: usize, price: f64, kind: SwingKind) -> SwingPoint {
        SwingPoint {
            index: idx,
            price,
            kind,
        }
    }

    #[test]
    fn empty_swings_returns_empty_report() {
        let r = detect(&[], &EqualLevelsConfig::default());
        assert!(r.clusters.is_empty());
        assert_eq!(r.n_equal_highs, 0);
    }

    #[test]
    fn three_swing_highs_at_same_price_form_one_cluster() {
        let swings = vec![
            sp(5, 100.05, SwingKind::High),
            sp(20, 100.00, SwingKind::High),
            sp(40, 100.02, SwingKind::High),
            sp(60, 95.0, SwingKind::Low), // noise — low, not relevant
        ];
        let r = detect(&swings, &EqualLevelsConfig::default());
        assert_eq!(r.n_equal_highs, 1);
        let h = &r.clusters[0];
        assert_eq!(h.member_indices.len(), 3);
        assert!(matches!(h.kind, LevelKind::EqualHighs));
    }

    #[test]
    fn pivots_outside_tolerance_dont_cluster() {
        // 100.0 and 100.50 are 50¢ apart — outside the default 0.10 tolerance.
        let swings = vec![
            sp(5, 100.0, SwingKind::High),
            sp(20, 100.50, SwingKind::High),
        ];
        let r = detect(&swings, &EqualLevelsConfig::default());
        assert!(r.clusters.is_empty());
    }

    #[test]
    fn min_cluster_size_three_filters_out_pairs() {
        let cfg = EqualLevelsConfig {
            cluster_tolerance: 0.10,
            min_cluster_size: 3,
        };
        let swings = vec![
            sp(5, 100.00, SwingKind::High),
            sp(20, 100.05, SwingKind::High), // only 2 — below threshold
            sp(30, 95.0, SwingKind::Low),
            sp(35, 95.02, SwingKind::Low),
            sp(40, 94.98, SwingKind::Low), // 3 lows — qualifies
        ];
        let r = detect(&swings, &cfg);
        assert_eq!(r.n_equal_highs, 0);
        assert_eq!(r.n_equal_lows, 1);
    }

    #[test]
    fn highs_and_lows_clustered_independently() {
        // 3 highs near 105 and 3 lows near 95 — both should fire.
        let swings = vec![
            sp(5, 105.0, SwingKind::High),
            sp(15, 105.05, SwingKind::High),
            sp(25, 104.98, SwingKind::High),
            sp(35, 95.0, SwingKind::Low),
            sp(45, 95.03, SwingKind::Low),
            sp(55, 94.97, SwingKind::Low),
        ];
        let r = detect(&swings, &EqualLevelsConfig::default());
        assert_eq!(r.n_equal_highs, 1);
        assert_eq!(r.n_equal_lows, 1);
    }

    #[test]
    fn two_separate_clusters_at_different_prices() {
        // Two pairs of highs at 100 and at 110 — two separate clusters.
        let swings = vec![
            sp(5, 100.0, SwingKind::High),
            sp(15, 100.05, SwingKind::High),
            sp(25, 110.0, SwingKind::High),
            sp(35, 110.02, SwingKind::High),
        ];
        let r = detect(&swings, &EqualLevelsConfig::default());
        assert_eq!(r.n_equal_highs, 2);
    }

    #[test]
    fn invalid_config_returns_empty() {
        // Negative tolerance is nonsense — return empty.
        let cfg = EqualLevelsConfig {
            cluster_tolerance: -1.0,
            min_cluster_size: 2,
        };
        let swings = vec![
            sp(5, 100.0, SwingKind::High),
            sp(20, 100.0, SwingKind::High),
        ];
        let r = detect(&swings, &cfg);
        assert!(r.clusters.is_empty());
        // min_cluster_size < 2 also returns empty.
        let cfg2 = EqualLevelsConfig {
            cluster_tolerance: 0.10,
            min_cluster_size: 1,
        };
        let r2 = detect(&swings, &cfg2);
        assert!(r2.clusters.is_empty());
    }
}
