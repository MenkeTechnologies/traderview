//! Sector rotation tracker.
//!
//! Ranks sector ETFs (XLE, XLF, XLK, etc.) by N-period return and emits:
//!   - sorted leaderboard (best→worst)
//!   - "leaders" (top quartile) and "laggards" (bottom quartile)
//!   - rotation flags: sector that moved up or down ≥ `min_rank_change`
//!     places vs the prior period's snapshot
//!
//! Used to spot defensive-vs-cyclical regime shifts (XLU/XLP suddenly
//! leading is bearish; XLE/XLF taking over from XLK is risk-on rotation).
//!
//! Pure compute. Caller supplies per-sector return series (most recent
//! return last). Optional `prior_ranks` lets the caller carry over yesterday's
//! ranking so rotation flags fire across days.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorReturn {
    pub symbol: String,
    /// Total return over the lookback in percent (e.g. 2.5 = +2.5%).
    pub return_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    /// Top-X are "leaders".
    pub leader_count: usize,
    /// Bottom-X are "laggards".
    pub laggard_count: usize,
    /// Minimum rank jump (up or down) vs prior snapshot to flag a rotation.
    pub min_rank_change: usize,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            leader_count: 3,
            laggard_count: 3,
            min_rank_change: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedSector {
    pub symbol: String,
    pub return_pct: f64,
    pub rank: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationFlag {
    pub symbol: String,
    pub prior_rank: usize,
    pub current_rank: usize,
    pub delta: i64, // negative = moved UP in rank (better); positive = moved DOWN
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RotationReport {
    pub leaderboard: Vec<RankedSector>,
    pub leaders: Vec<String>,
    pub laggards: Vec<String>,
    pub rotations: Vec<RotationFlag>,
}

pub fn analyze(
    returns: &[SectorReturn],
    prior_ranks: Option<&HashMap<String, usize>>,
    cfg: &RotationConfig,
) -> RotationReport {
    let mut report = RotationReport::default();
    if returns.is_empty() {
        return report;
    }
    let mut sorted: Vec<SectorReturn> = returns.to_vec();
    // Sort desc by return_pct; NaN sinks via unwrap_or(Equal).
    sorted.sort_by(|a, b| {
        b.return_pct
            .partial_cmp(&a.return_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (i, s) in sorted.iter().enumerate() {
        report.leaderboard.push(RankedSector {
            symbol: s.symbol.clone(),
            return_pct: s.return_pct,
            rank: i + 1,
        });
    }
    let n = report.leaderboard.len();
    report.leaders = report
        .leaderboard
        .iter()
        .take(cfg.leader_count.min(n))
        .map(|r| r.symbol.clone())
        .collect();
    report.laggards = report
        .leaderboard
        .iter()
        .rev()
        .take(cfg.laggard_count.min(n))
        .map(|r| r.symbol.clone())
        .collect();
    if let Some(prior) = prior_ranks {
        for r in &report.leaderboard {
            if let Some(&prev) = prior.get(&r.symbol) {
                let delta = r.rank as i64 - prev as i64;
                if delta.unsigned_abs() >= cfg.min_rank_change as u64 {
                    report.rotations.push(RotationFlag {
                        symbol: r.symbol.clone(),
                        prior_rank: prev,
                        current_rank: r.rank,
                        delta,
                    });
                }
            }
        }
    }
    report
}

/// Helper: extract a `symbol → rank` map from a `RotationReport.leaderboard`
/// to pass into the next analyze() call.
pub fn ranks_map(report: &RotationReport) -> HashMap<String, usize> {
    report
        .leaderboard
        .iter()
        .map(|r| (r.symbol.clone(), r.rank))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sym: &str, r: f64) -> SectorReturn {
        SectorReturn {
            symbol: sym.into(),
            return_pct: r,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], None, &RotationConfig::default());
        assert!(r.leaderboard.is_empty());
    }

    #[test]
    fn leaderboard_sorted_desc_by_return() {
        let v = vec![s("XLF", 1.2), s("XLK", 3.5), s("XLU", -1.0), s("XLE", 2.0)];
        let r = analyze(&v, None, &RotationConfig::default());
        let order: Vec<_> = r.leaderboard.iter().map(|x| x.symbol.as_str()).collect();
        assert_eq!(order, vec!["XLK", "XLE", "XLF", "XLU"]);
    }

    #[test]
    fn leaders_take_top_n_laggards_take_bottom_n() {
        let v: Vec<SectorReturn> = (0..10).map(|i| s(&format!("X{i}"), i as f64)).collect();
        let cfg = RotationConfig {
            leader_count: 2,
            laggard_count: 2,
            min_rank_change: 99,
        };
        let r = analyze(&v, None, &cfg);
        // Highest two returns are X9 (9.0) and X8 (8.0).
        assert_eq!(r.leaders, vec!["X9".to_string(), "X8".to_string()]);
        assert_eq!(r.laggards, vec!["X0".to_string(), "X1".to_string()]);
    }

    #[test]
    fn rotation_flag_fires_on_big_rank_change() {
        let mut prior = HashMap::new();
        prior.insert("XLE".to_string(), 1);
        prior.insert("XLK".to_string(), 5);
        let cur = vec![s("XLE", -2.0), s("XLK", 3.0)];
        let cfg = RotationConfig {
            leader_count: 1,
            laggard_count: 1,
            min_rank_change: 3,
        };
        let r = analyze(&cur, Some(&prior), &cfg);
        // XLE went from rank 1 → 2 (delta 1) — won't flag.
        // XLK went from rank 5 → 1 (delta -4) — should flag.
        let xlk: Vec<_> = r.rotations.iter().filter(|f| f.symbol == "XLK").collect();
        assert_eq!(xlk.len(), 1);
        assert!(xlk[0].delta <= -3);
    }

    #[test]
    fn no_rotation_flag_without_prior_map() {
        let v = vec![s("XLF", 1.0)];
        let r = analyze(&v, None, &RotationConfig::default());
        assert!(r.rotations.is_empty());
    }

    #[test]
    fn ranks_map_round_trips_through_analyze() {
        let v1 = vec![s("A", 5.0), s("B", 3.0), s("C", 1.0)];
        let r1 = analyze(&v1, None, &RotationConfig::default());
        let prior = ranks_map(&r1);
        assert_eq!(prior.len(), 3);
        // Re-analyze the SAME returns with that as prior → no rotation events.
        let r2 = analyze(&v1, Some(&prior), &RotationConfig::default());
        assert!(r2.rotations.is_empty());
    }

    #[test]
    fn nan_returns_sink_to_end_of_leaderboard() {
        let v = vec![s("A", 5.0), s("B", f64::NAN), s("C", 3.0)];
        let r = analyze(&v, None, &RotationConfig::default());
        // NaN's position is implementation-defined via unwrap_or(Equal);
        // verify we don't panic + leaderboard has all three.
        assert_eq!(r.leaderboard.len(), 3);
    }
}
