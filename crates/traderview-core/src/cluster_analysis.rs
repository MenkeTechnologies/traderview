//! Trade cluster analysis — k-means over (entry_time, hold_duration, R-multiple).
//!
//! Identifies natural clusters in the trader's behavior — e.g. "you have
//! a profitable cluster of short-hold morning trades AND a money-losing
//! cluster of long-hold afternoon trades". Pure compute Lloyd's k-means
//! with deterministic seeding (no RNG dependency).
//!
//! Output: per-trade cluster assignment + per-cluster centroid + stats.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TradeFeature {
    pub entry_minute_of_day: f64,    // 0..1440
    pub hold_duration_minutes: f64,
    pub r_multiple: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Centroid {
    pub entry_minute: f64,
    pub hold_minutes: f64,
    pub r_multiple: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStat {
    pub cluster_id: usize,
    pub size: usize,
    pub centroid: Centroid,
    pub mean_r: f64,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClusterReport {
    pub assignments: Vec<usize>,
    pub clusters: Vec<ClusterStat>,
}

pub fn analyze(features: &[TradeFeature], k: usize, max_iters: usize) -> ClusterReport {
    let mut report = ClusterReport::default();
    if features.is_empty() || k == 0 { return report; }
    let k = k.min(features.len());
    // Deterministic seeding: pick first k points (or stride if k < n).
    let stride = features.len() / k;
    let mut centroids: Vec<Centroid> = (0..k).map(|i| {
        let f = features[i * stride];
        Centroid {
            entry_minute: f.entry_minute_of_day,
            hold_minutes: f.hold_duration_minutes,
            r_multiple: f.r_multiple,
        }
    }).collect();

    let mut assignments = vec![0; features.len()];
    for _ in 0..max_iters {
        // Assign each point to nearest centroid.
        let mut changed = false;
        for (i, f) in features.iter().enumerate() {
            let mut best = 0;
            let mut best_dist = f64::INFINITY;
            for (j, c) in centroids.iter().enumerate() {
                let d = sq_dist(f, c);
                if d < best_dist { best_dist = d; best = j; }
            }
            if assignments[i] != best { changed = true; assignments[i] = best; }
        }
        if !changed { break; }
        // Update centroids.
        for (j, centroid) in centroids.iter_mut().enumerate().take(k) {
            let members: Vec<&TradeFeature> = features.iter().enumerate()
                .filter(|(i, _)| assignments[*i] == j)
                .map(|(_, f)| f)
                .collect();
            if members.is_empty() { continue; }
            let n = members.len() as f64;
            *centroid = Centroid {
                entry_minute: members.iter().map(|f| f.entry_minute_of_day).sum::<f64>() / n,
                hold_minutes: members.iter().map(|f| f.hold_duration_minutes).sum::<f64>() / n,
                r_multiple: members.iter().map(|f| f.r_multiple).sum::<f64>() / n,
            };
        }
    }
    // Build per-cluster stats.
    for (j, centroid) in centroids.iter().enumerate().take(k) {
        let idxs: Vec<usize> = (0..features.len()).filter(|i| assignments[*i] == j).collect();
        let size = idxs.len();
        let mean_r = if size > 0 {
            idxs.iter().map(|i| features[*i].r_multiple).sum::<f64>() / size as f64
        } else { 0.0 };
        let wins = idxs.iter().filter(|i| features[**i].r_multiple > 0.0).count();
        let win_rate = if size > 0 { wins as f64 / size as f64 } else { 0.0 };
        report.clusters.push(ClusterStat {
            cluster_id: j,
            size,
            centroid: centroid.clone(),
            mean_r,
            win_rate,
        });
    }
    report.assignments = assignments;
    report
}

fn sq_dist(f: &TradeFeature, c: &Centroid) -> f64 {
    // Standard k-means uses Euclidean; normalize so all 3 dims contribute
    // comparably (entry_minute on 1440 scale dominates otherwise).
    let de = (f.entry_minute_of_day - c.entry_minute) / 1440.0;
    let dh = (f.hold_duration_minutes - c.hold_minutes) / 1440.0;
    let dr = (f.r_multiple - c.r_multiple) / 5.0;    // typical |R| range ±5.
    de * de + dh * dh + dr * dr
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(em: f64, hd: f64, r: f64) -> TradeFeature {
        TradeFeature { entry_minute_of_day: em, hold_duration_minutes: hd, r_multiple: r }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], 3, 10);
        assert!(r.assignments.is_empty());
    }

    #[test]
    fn k_zero_returns_default() {
        let r = analyze(&[f(10.0, 30.0, 1.0)], 0, 10);
        assert!(r.assignments.is_empty());
    }

    #[test]
    fn single_cluster_assigns_all_to_one() {
        let trades = vec![f(570.0, 30.0, 1.0), f(575.0, 35.0, 1.5), f(580.0, 28.0, 0.9)];
        let r = analyze(&trades, 1, 10);
        assert!(r.assignments.iter().all(|a| *a == 0));
        assert_eq!(r.clusters[0].size, 3);
    }

    #[test]
    fn two_distinct_clusters_separated() {
        // Morning short-hold winning cluster + afternoon long-hold losing cluster.
        let trades = vec![
            f(540.0, 30.0, 1.5),     // morning, short, win
            f(545.0, 25.0, 2.0),
            f(550.0, 35.0, 1.0),
            f(840.0, 240.0, -0.8),    // afternoon, long, loss
            f(850.0, 250.0, -1.0),
            f(860.0, 220.0, -0.5),
        ];
        let r = analyze(&trades, 2, 20);
        // First three should be in one cluster, last three in another.
        let first_assignment = r.assignments[0];
        let other = if first_assignment == 0 { 1 } else { 0 };
        for i in 0..3 { assert_eq!(r.assignments[i], first_assignment); }
        for i in 3..6 { assert_eq!(r.assignments[i], other); }
    }

    #[test]
    fn cluster_stats_reflect_membership() {
        let trades = vec![
            f(540.0, 30.0, 1.5),
            f(545.0, 25.0, 2.0),
            f(550.0, 35.0, -0.5),
            f(900.0, 200.0, -1.0),
        ];
        let r = analyze(&trades, 2, 20);
        // Sum of sizes = total trades.
        let total: usize = r.clusters.iter().map(|c| c.size).sum();
        assert_eq!(total, 4);
    }

    #[test]
    fn k_exceeds_n_capped_to_n() {
        let trades = vec![f(100.0, 30.0, 1.0), f(200.0, 60.0, 2.0)];
        let r = analyze(&trades, 10, 5);    // k=10 but only 2 trades
        // Should be capped to 2 (or fewer if collapsed) clusters.
        assert!(r.clusters.len() <= 2);
    }

    #[test]
    fn deterministic_seeding_produces_repeatable_results() {
        let trades = vec![
            f(540.0, 30.0, 1.5), f(545.0, 25.0, 2.0), f(550.0, 35.0, 1.0),
            f(840.0, 240.0, -0.8), f(850.0, 250.0, -1.0),
        ];
        let r1 = analyze(&trades, 2, 20);
        let r2 = analyze(&trades, 2, 20);
        assert_eq!(r1.assignments, r2.assignments);
    }
}
