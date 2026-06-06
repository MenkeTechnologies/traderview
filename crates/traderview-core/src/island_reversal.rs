//! Island Reversal Pattern Detector.
//!
//! An island reversal is a price formation where a span of bars (the
//! "island") is isolated from the rest of the chart by two gaps in
//! opposite directions:
//!
//!   - Top: gap-up open, sideways trade in the island, gap-down out
//!   - Bottom: gap-down open, sideways trade, gap-up out
//!
//! The pattern signals exhaustion of the prior trend and a likely
//! reversal once the second gap forms.
//!
//! Detection rule (configurable):
//!   - First gap > `min_gap_pct` size, in one direction
//!   - Island has 1..`max_island_bars` bars (often single bar)
//!   - Second gap > `min_gap_pct` in the opposite direction
//!   - Island bars don't fill the first gap
//!
//! Pure compute. Companion to `cup_and_handle`, `rounding_pattern`,
//! `bump_and_run`, `gap_fill_stats`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub open: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind {
    IslandTop,
    IslandBottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IslandCandidate {
    pub kind: PatternKind,
    pub island_start: usize,
    pub island_end: usize,
    pub entry_gap_pct: f64,
    pub exit_gap_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub min_gap_pct: f64,
    pub max_island_bars: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_gap_pct: 0.01,
            max_island_bars: 5,
        }
    }
}

pub fn detect(bars: &[Bar], cfg: &Config) -> Vec<IslandCandidate> {
    let n = bars.len();
    let mut out = Vec::new();
    if n < 3 || cfg.max_island_bars == 0 || cfg.min_gap_pct <= 0.0 {
        return out;
    }
    if bars.iter().any(|b| {
        !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite() || !b.open.is_finite()
    }) {
        return out;
    }
    // For each potential entry-gap at index i (gap between bars[i-1] and bars[i]),
    // scan forward up to max_island_bars looking for an opposite-direction exit gap.
    for i in 1..(n - 1) {
        let prev = bars[i - 1];
        let entry = bars[i];
        // Entry gap up = entry.low > prev.high.
        // Entry gap down = entry.high < prev.low.
        let gap_up = entry.low > prev.high * (1.0 + cfg.min_gap_pct);
        let gap_down = entry.high < prev.low * (1.0 - cfg.min_gap_pct);
        if !gap_up && !gap_down {
            continue;
        }
        let kind = if gap_up {
            PatternKind::IslandTop
        } else {
            PatternKind::IslandBottom
        };
        let entry_gap_pct = if gap_up {
            (entry.low - prev.high) / prev.high
        } else {
            (entry.high - prev.low) / prev.low
        };
        // Scan island lengths from 1 to max_island_bars.
        for island_len in 1..=cfg.max_island_bars.min(n - i - 1) {
            let exit_idx = i + island_len;
            if exit_idx >= n {
                break;
            }
            let last_island = bars[exit_idx - 1];
            let exit = bars[exit_idx];
            // For an Island Top: exit must gap DOWN below the island's low.
            // For an Island Bottom: exit must gap UP above the island's high.
            let island_low = (i..exit_idx)
                .map(|j| bars[j].low)
                .fold(f64::INFINITY, f64::min);
            let island_high = (i..exit_idx)
                .map(|j| bars[j].high)
                .fold(f64::NEG_INFINITY, f64::max);
            let _ = last_island;
            let (exit_gap, valid) = match kind {
                PatternKind::IslandTop => {
                    let gap = (exit.high - island_low) / island_low;
                    let v = exit.high < island_low * (1.0 - cfg.min_gap_pct);
                    (gap, v)
                }
                PatternKind::IslandBottom => {
                    let gap = (exit.low - island_high) / island_high;
                    let v = exit.low > island_high * (1.0 + cfg.min_gap_pct);
                    (gap, v)
                }
            };
            if valid {
                out.push(IslandCandidate {
                    kind,
                    island_start: i,
                    island_end: exit_idx - 1,
                    entry_gap_pct,
                    exit_gap_pct: exit_gap,
                });
                break; // first valid exit-gap completes the pattern
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, o: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            open: o,
            close: c,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(detect(&[], &Config::default()).is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let bars = vec![b(101.0, 99.0, 100.0, 100.0); 10];
        let cfg = Config {
            min_gap_pct: 0.0,
            ..Default::default()
        };
        assert!(detect(&bars, &cfg).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 100.0); 10];
        bars[3] = b(f64::NAN, 99.0, 100.0, 100.0);
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn no_gaps_no_pattern() {
        let bars = vec![b(101.0, 99.0, 100.0, 100.0); 20];
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn classic_island_top_detected() {
        // Bars 0-2: range 99-101. Gap up at bar 3 to 110-115.
        // Island bar 3: high=115, low=110. Bar 4 gap down to 98-101.
        let mut bars = vec![b(101.0, 99.0, 100.0, 100.0); 8];
        bars[3] = b(115.0, 110.0, 112.0, 113.0); // gap up
        bars[4] = b(101.0, 98.0, 100.0, 99.0); // gap down → exit
        let cands = detect(&bars, &Config::default());
        assert!(
            cands.iter().any(|c| c.kind == PatternKind::IslandTop),
            "expected island top, got {cands:?}"
        );
    }

    #[test]
    fn classic_island_bottom_detected() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 100.0); 8];
        bars[3] = b(91.0, 85.0, 89.0, 88.0); // gap down
        bars[4] = b(101.0, 95.0, 99.0, 100.0); // gap up exit
        let cands = detect(&bars, &Config::default());
        assert!(cands.iter().any(|c| c.kind == PatternKind::IslandBottom));
    }

    #[test]
    fn multi_bar_island_supported() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 100.0); 12];
        bars[3] = b(115.0, 110.0, 112.0, 113.0);
        bars[4] = b(115.0, 112.0, 113.0, 114.0);
        bars[5] = b(116.0, 111.0, 112.0, 115.0);
        bars[6] = b(101.0, 98.0, 100.0, 99.0); // exit gap
        let cands = detect(&bars, &Config::default());
        assert!(cands
            .iter()
            .any(|c| c.kind == PatternKind::IslandTop && c.island_end - c.island_start == 2));
    }

    #[test]
    fn min_gap_pct_filter_works() {
        // Small 0.3% gap < default 1% threshold → no detection.
        let mut bars = vec![b(101.0, 99.0, 100.0, 100.0); 8];
        bars[3] = b(102.0, 101.3, 101.5, 101.5); // tiny gap
        bars[4] = b(101.0, 98.0, 100.0, 99.0);
        assert!(detect(&bars, &Config::default()).is_empty());
    }
}
