//! Order-book depth imbalance + snapshot-to-snapshot delta tracker.
//!
//! Given a series of depth snapshots (bids + asks + sizes at each level),
//! emit:
//!   - per-snapshot imbalance metric: `(bid_total − ask_total) / (bid + ask)`
//!     across the top `levels` rungs of the book.
//!   - per-snapshot delta: which levels saw added or pulled liquidity
//!     vs the prior snapshot, flagged when the change exceeds
//!     `min_delta_pct` of total book depth.
//!
//! Useful for spotting whale-spoofing (large size posted then pulled
//! immediately) and absorption (one side keeps reposting at the same
//! level despite repeated takes).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthSnapshot {
    /// Bids: best (highest price) first.
    pub bids: Vec<DepthLevel>,
    /// Asks: best (lowest price) first.
    pub asks: Vec<DepthLevel>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DepthLevel {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthConfig {
    /// Aggregate top-N levels per side for the imbalance ratio.
    pub levels: usize,
    /// Minimum absolute delta as a fraction of total book depth to flag
    /// a level as added/pulled (0.05 = 5%).
    pub min_delta_pct: f64,
}

impl Default for DepthConfig {
    fn default() -> Self { Self { levels: 10, min_delta_pct: 0.05 } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideChange {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Pulled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthEvent {
    pub snapshot_index: usize,
    pub side: SideChange,
    pub kind: ChangeKind,
    pub price: f64,
    pub size_delta: f64,
    pub pct_of_book: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DepthReport {
    /// Imbalance ∈ [-1, +1] per snapshot.
    pub imbalances: Vec<f64>,
    /// Add/pull events between adjacent snapshots.
    pub events: Vec<DepthEvent>,
}

pub fn analyze(snapshots: &[DepthSnapshot], cfg: &DepthConfig) -> DepthReport {
    let mut report = DepthReport::default();
    if cfg.levels == 0 || !cfg.min_delta_pct.is_finite() || cfg.min_delta_pct < 0.0 {
        return report;
    }
    let levels = cfg.levels;
    // Per-snapshot imbalance.
    for snap in snapshots {
        let bid_total: f64 = snap.bids.iter().take(levels).map(|l| l.size.max(0.0)).sum();
        let ask_total: f64 = snap.asks.iter().take(levels).map(|l| l.size.max(0.0)).sum();
        let denom = bid_total + ask_total;
        let imb = if denom > 0.0 { (bid_total - ask_total) / denom } else { 0.0 };
        report.imbalances.push(if imb.is_finite() { imb } else { 0.0 });
    }
    // Cross-snapshot deltas.
    for i in 1..snapshots.len() {
        let prev = &snapshots[i - 1];
        let cur = &snapshots[i];
        let book_total_cur: f64 = cur.bids.iter().take(levels).map(|l| l.size.max(0.0)).sum::<f64>()
            + cur.asks.iter().take(levels).map(|l| l.size.max(0.0)).sum::<f64>();
        if book_total_cur <= 0.0 {
            continue;
        }
        // Walk each side; index by price (rounded to f64 hash via
        // string would be expensive — simple O(n²) match on .price ==
        // works for small `levels`).
        for side in [SideChange::Bid, SideChange::Ask] {
            let (prev_levels, cur_levels) = match side {
                SideChange::Bid => (&prev.bids, &cur.bids),
                SideChange::Ask => (&prev.asks, &cur.asks),
            };
            let p = prev_levels.iter().take(levels).collect::<Vec<_>>();
            let c = cur_levels.iter().take(levels).collect::<Vec<_>>();
            for cl in &c {
                let prior = p.iter().find(|x| (x.price - cl.price).abs() < f64::EPSILON);
                let prev_size = prior.map(|x| x.size).unwrap_or(0.0);
                let delta = cl.size - prev_size;
                if delta.abs() / book_total_cur >= cfg.min_delta_pct {
                    report.events.push(DepthEvent {
                        snapshot_index: i,
                        side,
                        kind: if delta > 0.0 { ChangeKind::Added } else { ChangeKind::Pulled },
                        price: cl.price,
                        size_delta: delta,
                        pct_of_book: delta.abs() / book_total_cur,
                    });
                }
            }
            // Detect FULL pulls (level was present prev, gone now).
            for pl in &p {
                let still_there = c.iter().any(|x| (x.price - pl.price).abs() < f64::EPSILON);
                if !still_there {
                    let delta = -pl.size;
                    if pl.size.abs() / book_total_cur >= cfg.min_delta_pct {
                        report.events.push(DepthEvent {
                            snapshot_index: i,
                            side,
                            kind: ChangeKind::Pulled,
                            price: pl.price,
                            size_delta: delta,
                            pct_of_book: pl.size.abs() / book_total_cur,
                        });
                    }
                }
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lvl(p: f64, s: f64) -> DepthLevel { DepthLevel { price: p, size: s } }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], &DepthConfig::default());
        assert!(r.imbalances.is_empty());
    }

    #[test]
    fn zero_levels_returns_default() {
        let snap = DepthSnapshot { bids: vec![lvl(100.0, 10.0)], asks: vec![lvl(101.0, 10.0)] };
        let r = analyze(&[snap], &DepthConfig { levels: 0, min_delta_pct: 0.05 });
        assert!(r.imbalances.is_empty());
    }

    #[test]
    fn balanced_book_imbalance_zero() {
        let snap = DepthSnapshot {
            bids: vec![lvl(100.0, 100.0); 5],
            asks: vec![lvl(101.0, 100.0); 5],
        };
        let r = analyze(&[snap], &DepthConfig::default());
        assert!(r.imbalances[0].abs() < 1e-9);
    }

    #[test]
    fn bid_heavy_book_positive_imbalance() {
        let snap = DepthSnapshot {
            bids: vec![lvl(100.0, 300.0); 5],
            asks: vec![lvl(101.0, 100.0); 5],
        };
        let r = analyze(&[snap], &DepthConfig::default());
        assert!(r.imbalances[0] > 0.0);
    }

    #[test]
    fn pulled_level_flagged_between_snapshots() {
        let s1 = DepthSnapshot {
            bids: vec![lvl(100.0, 500.0), lvl(99.0, 100.0)],
            asks: vec![lvl(101.0, 100.0)],
        };
        let s2 = DepthSnapshot {
            // The 500-size bid at 100 has been pulled.
            bids: vec![lvl(99.0, 100.0)],
            asks: vec![lvl(101.0, 100.0)],
        };
        let r = analyze(&[s1, s2], &DepthConfig { levels: 10, min_delta_pct: 0.10 });
        let pulled: Vec<_> = r.events.iter()
            .filter(|e| e.kind == ChangeKind::Pulled && (e.price - 100.0).abs() < 1e-9)
            .collect();
        assert!(!pulled.is_empty(), "expected pull event at price 100");
    }

    #[test]
    fn added_level_flagged() {
        let s1 = DepthSnapshot {
            bids: vec![lvl(100.0, 100.0)],
            asks: vec![lvl(101.0, 100.0)],
        };
        let s2 = DepthSnapshot {
            bids: vec![lvl(100.0, 500.0)],   // 4× added
            asks: vec![lvl(101.0, 100.0)],
        };
        let r = analyze(&[s1, s2], &DepthConfig { levels: 10, min_delta_pct: 0.10 });
        let added: Vec<_> = r.events.iter().filter(|e| e.kind == ChangeKind::Added).collect();
        assert!(!added.is_empty());
    }

    #[test]
    fn small_delta_below_threshold_not_flagged() {
        let s1 = DepthSnapshot {
            bids: vec![lvl(100.0, 1000.0); 5],
            asks: vec![lvl(101.0, 1000.0); 5],
        };
        let s2 = DepthSnapshot {
            bids: vec![lvl(100.0, 1005.0); 5],    // 0.5% change
            asks: vec![lvl(101.0, 1000.0); 5],
        };
        let r = analyze(&[s1, s2], &DepthConfig { levels: 10, min_delta_pct: 0.10 });
        assert!(r.events.is_empty());
    }
}
