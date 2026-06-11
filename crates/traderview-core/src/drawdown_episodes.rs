//! Drawdown episodes — the top-N worst peak-to-trough events with
//! their durations and recovery times, not just the single max-DD
//! number.
//!
//! An episode opens when the series leaves a running high and closes
//! when a new high prints. Depth = peak→trough %, decline = bars
//! peak→trough, recovery = bars trough→new high (None while still
//! underwater). The table answers the question max-DD hides: how
//! OFTEN it hurts and how long the hole lasts.
//!
//! Pure compute over closes/equity. Companion to `profit_factor`
//! (recovery factor), `drawdown_throttle`.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DrawdownEpisode {
    /// Indices into the input series.
    pub peak_index: usize,
    pub trough_index: usize,
    pub depth_pct: f64,
    /// Bars from peak to trough.
    pub decline_bars: usize,
    /// Bars from trough back to a new high; None = still underwater.
    pub recovery_bars: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EpisodesReport {
    /// Worst-first, capped at the requested N.
    pub episodes: Vec<DrawdownEpisode>,
    pub currently_underwater: bool,
    pub current_drawdown_pct: f64,
}

pub fn compute(series: &[f64], top_n: usize) -> Option<EpisodesReport> {
    if series.len() < 2
        || top_n == 0
        || series.iter().any(|v| !v.is_finite() || *v <= 0.0)
    {
        return None;
    }
    let mut episodes: Vec<DrawdownEpisode> = Vec::new();
    let mut peak_idx = 0usize;
    let mut trough_idx = 0usize;
    let mut in_drawdown = false;
    for (i, &v) in series.iter().enumerate() {
        if v >= series[peak_idx] {
            if in_drawdown {
                // New high closes the open episode.
                episodes.push(DrawdownEpisode {
                    peak_index: peak_idx,
                    trough_index: trough_idx,
                    depth_pct: (series[trough_idx] / series[peak_idx] - 1.0) * 100.0,
                    decline_bars: trough_idx - peak_idx,
                    recovery_bars: Some(i - trough_idx),
                });
                in_drawdown = false;
            }
            peak_idx = i;
        } else {
            if !in_drawdown {
                in_drawdown = true;
                trough_idx = i;
            } else if v < series[trough_idx] {
                trough_idx = i;
            }
        }
    }
    let last = series.len() - 1;
    let current_dd = (series[last] / series[peak_idx] - 1.0) * 100.0;
    if in_drawdown {
        episodes.push(DrawdownEpisode {
            peak_index: peak_idx,
            trough_index: trough_idx,
            depth_pct: (series[trough_idx] / series[peak_idx] - 1.0) * 100.0,
            decline_bars: trough_idx - peak_idx,
            recovery_bars: None,
        });
    }
    episodes.sort_by(|a, b| {
        a.depth_pct
            .partial_cmp(&b.depth_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    episodes.truncate(top_n);
    Some(EpisodesReport {
        episodes,
        currently_underwater: in_drawdown,
        current_drawdown_pct: current_dd.min(0.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_dips_ranked_by_depth() {
        // 100 → 90 (−10%, recovered in 2) → 110 → 88 (−20%, recovered
        // in 1) → 115.
        let series = vec![100.0, 90.0, 95.0, 102.0, 110.0, 88.0, 115.0];
        let r = compute(&series, 5).unwrap();
        assert_eq!(r.episodes.len(), 2);
        let worst = &r.episodes[0];
        assert!((worst.depth_pct + 20.0).abs() < 1e-9);
        assert_eq!(worst.peak_index, 4);
        assert_eq!(worst.trough_index, 5);
        assert_eq!(worst.decline_bars, 1);
        assert_eq!(worst.recovery_bars, Some(1));
        let second = &r.episodes[1];
        assert!((second.depth_pct + 10.0).abs() < 1e-9);
        assert_eq!(second.recovery_bars, Some(2));
        assert!(!r.currently_underwater);
        assert_eq!(r.current_drawdown_pct, 0.0);
    }

    #[test]
    fn open_drawdown_has_no_recovery() {
        let series = vec![100.0, 120.0, 100.0, 96.0];
        let r = compute(&series, 5).unwrap();
        assert_eq!(r.episodes.len(), 1);
        assert_eq!(r.episodes[0].recovery_bars, None);
        assert!(r.currently_underwater);
        assert!((r.current_drawdown_pct + 20.0).abs() < 1e-9);
    }

    #[test]
    fn top_n_caps_the_table() {
        // Three dips; ask for two.
        let series = vec![100.0, 95.0, 101.0, 91.0, 102.0, 82.0, 103.0];
        let r = compute(&series, 2).unwrap();
        assert_eq!(r.episodes.len(), 2);
        // Deepest first.
        assert!(r.episodes[0].depth_pct <= r.episodes[1].depth_pct);
    }

    #[test]
    fn monotonic_series_has_no_episodes() {
        let r = compute(&[100.0, 101.0, 102.0, 103.0], 5).unwrap();
        assert!(r.episodes.is_empty());
        assert!(!r.currently_underwater);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&[100.0], 5).is_none());
        assert!(compute(&[100.0, 0.0], 5).is_none());
        assert!(compute(&[100.0, 99.0], 0).is_none());
    }
}
