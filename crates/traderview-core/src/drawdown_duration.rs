//! Drawdown duration analysis.
//!
//! For each historical drawdown period (peak → trough → new peak),
//! compute:
//!   - Depth (peak-to-trough %)
//!   - Time to trough (peak → trough days)
//!   - Recovery time (trough → new peak days)
//!   - Total underwater duration (peak → new peak)
//!
//! Distribution stats: longest DD, longest unrecovered DD, average
//! recovery time.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownEpisode {
    pub peak_index: usize,
    pub trough_index: usize,
    /// None if still under water at series end.
    pub recovery_index: Option<usize>,
    pub depth_pct: f64,
    pub time_to_trough: usize,
    pub recovery_time: Option<usize>,
    pub underwater_duration: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DrawdownReport {
    pub episodes: Vec<DrawdownEpisode>,
    pub max_depth_pct: f64,
    pub longest_underwater: Option<usize>,
    pub avg_recovery_time: Option<f64>,
    /// Indices where the series is still under water relative to the
    /// last peak.
    pub currently_under_water: bool,
}

pub fn analyze(equity: &[f64]) -> DrawdownReport {
    let mut report = DrawdownReport::default();
    if equity.is_empty() {
        return report;
    }
    let mut peak = equity[0];
    let mut peak_idx = 0;
    let mut in_dd = false;
    let mut trough = peak;
    let mut trough_idx = 0;
    for (i, &v) in equity.iter().enumerate() {
        if v > peak {
            if in_dd {
                let depth = (peak - trough) / peak;
                let underwater = i - peak_idx;
                report.episodes.push(DrawdownEpisode {
                    peak_index: peak_idx,
                    trough_index: trough_idx,
                    recovery_index: Some(i),
                    depth_pct: depth,
                    time_to_trough: trough_idx - peak_idx,
                    recovery_time: Some(i - trough_idx),
                    underwater_duration: Some(underwater),
                });
                if depth > report.max_depth_pct {
                    report.max_depth_pct = depth;
                }
                in_dd = false;
            }
            peak = v;
            peak_idx = i;
            trough = v;
            trough_idx = i;
        } else if v < peak {
            if !in_dd {
                in_dd = true;
                trough = v;
                trough_idx = i;
            } else if v < trough {
                trough = v;
                trough_idx = i;
            }
        }
    }
    // Unfinished DD at series end.
    if in_dd {
        let depth = (peak - trough) / peak;
        report.episodes.push(DrawdownEpisode {
            peak_index: peak_idx,
            trough_index: trough_idx,
            recovery_index: None,
            depth_pct: depth,
            time_to_trough: trough_idx - peak_idx,
            recovery_time: None,
            underwater_duration: None,
        });
        if depth > report.max_depth_pct {
            report.max_depth_pct = depth;
        }
        report.currently_under_water = true;
    }
    let recovered: Vec<_> = report
        .episodes
        .iter()
        .filter_map(|e| e.recovery_time)
        .collect();
    if !recovered.is_empty() {
        report.avg_recovery_time =
            Some(recovered.iter().sum::<usize>() as f64 / recovered.len() as f64);
    }
    report.longest_underwater = report
        .episodes
        .iter()
        .filter_map(|e| e.underwater_duration)
        .max();
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[]);
        assert!(r.episodes.is_empty());
    }

    #[test]
    fn monotonic_uptrend_no_dd() {
        let r = analyze(&[100.0, 110.0, 120.0, 130.0]);
        assert!(r.episodes.is_empty());
        assert!(!r.currently_under_water);
    }

    #[test]
    fn single_dd_recovered_emits_one_episode() {
        let r = analyze(&[100.0, 110.0, 90.0, 105.0, 115.0]);
        assert_eq!(r.episodes.len(), 1);
        let e = &r.episodes[0];
        assert_eq!(e.peak_index, 1);
        assert_eq!(e.trough_index, 2);
        assert_eq!(e.recovery_index, Some(4));
        // Depth: (110-90)/110 = 0.1818.
        assert!((e.depth_pct - 0.1818).abs() < 0.001);
    }

    #[test]
    fn ongoing_dd_at_end_records_no_recovery() {
        let r = analyze(&[100.0, 110.0, 90.0, 85.0]);
        assert_eq!(r.episodes.len(), 1);
        assert!(r.episodes[0].recovery_index.is_none());
        assert!(r.currently_under_water);
    }

    #[test]
    fn max_depth_extracted_across_episodes() {
        // Two dds: 10% then 30%.
        let r = analyze(&[
            100.0, 110.0, 99.0, 115.0, // first DD ~10% deep
            120.0, 84.0, 130.0, // second DD 30% deep
        ]);
        assert_eq!(r.episodes.len(), 2);
        assert!((r.max_depth_pct - 0.30).abs() < 0.001);
    }

    #[test]
    fn longest_underwater_picks_max() {
        let r = analyze(&[
            100.0, 95.0, 105.0, // 2-period DD
            110.0, 100.0, 90.0, 95.0, 115.0, // 4-period DD
        ]);
        assert_eq!(r.longest_underwater, Some(4));
    }

    #[test]
    fn avg_recovery_time_from_completed_episodes_only() {
        // First DD: peak idx 0 (100), trough idx 1 (90), recovery idx 2 (110).
        // recovery_time = 2 - 1 = 1.
        // Then peak resets to 110. Next equity 100 < 110 → new DD.
        // Last bar 95 < trough → ongoing.
        // Recovered episodes: [1]. Mean = 1.0.
        let r = analyze(&[100.0, 90.0, 110.0, 100.0, 95.0]);
        assert_eq!(r.avg_recovery_time, Some(1.0));
    }

    #[test]
    fn flat_series_no_dd() {
        let r = analyze(&[100.0; 10]);
        assert!(r.episodes.is_empty());
    }

    #[test]
    fn depth_capped_at_dollar_loss_relative_to_peak() {
        // 100 → 50 = 50% DD.
        let r = analyze(&[100.0, 50.0, 110.0]);
        assert!((r.episodes[0].depth_pct - 0.5).abs() < 1e-9);
    }
}
