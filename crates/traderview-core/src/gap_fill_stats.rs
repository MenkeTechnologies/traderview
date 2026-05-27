//! Overnight gap detection + historical fill-probability tracker.
//!
//! For each bar, a "gap" is defined when `today.open` is outside
//! `[yesterday.low, yesterday.high]`. The gap "fills" the same day if
//! `today.low <= yesterday.high` (for up gaps) or
//! `today.high >= yesterday.low` (for down gaps).
//!
//! Reports: event list per gap + summary fill rates by direction +
//! average gap size in ATRs.
//!
//! Pure compute. Standard equity-research primitive; used to gauge
//! "gap-fade" strategy edge.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapDirection { Up, Down }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GapEvent {
    pub bar_index: usize,
    pub direction: GapDirection,
    pub gap_size: f64,
    pub gap_size_atrs: f64,
    pub filled_same_day: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GapStatsReport {
    pub events: Vec<GapEvent>,
    pub n_gaps: usize,
    pub n_up_gaps: usize,
    pub n_down_gaps: usize,
    pub up_fill_rate: f64,
    pub down_fill_rate: f64,
    pub mean_gap_atrs: f64,
}

pub fn analyze(bars: &[OhlcBar], atr: &[f64]) -> GapStatsReport {
    let n = bars.len();
    if n < 2 || atr.len() != n {
        return GapStatsReport::default();
    }
    let mut events = Vec::new();
    for i in 1..n {
        let prev = bars[i - 1];
        let cur = bars[i];
        let a = atr[i];
        let direction = if cur.open > prev.high {
            Some(GapDirection::Up)
        } else if cur.open < prev.low {
            Some(GapDirection::Down)
        } else {
            None
        };
        if let Some(dir) = direction {
            let gap_size = match dir {
                GapDirection::Up   => cur.open - prev.high,
                GapDirection::Down => prev.low - cur.open,
            };
            let gap_size_atrs = if a > 0.0 { gap_size / a } else { 0.0 };
            let filled = match dir {
                GapDirection::Up   => cur.low <= prev.high,
                GapDirection::Down => cur.high >= prev.low,
            };
            events.push(GapEvent {
                bar_index: i, direction: dir, gap_size, gap_size_atrs,
                filled_same_day: filled,
            });
        }
    }
    let n_gaps = events.len();
    let n_up_gaps = events.iter().filter(|e| matches!(e.direction, GapDirection::Up)).count();
    let n_down_gaps = n_gaps - n_up_gaps;
    let up_filled = events.iter()
        .filter(|e| matches!(e.direction, GapDirection::Up) && e.filled_same_day)
        .count();
    let down_filled = events.iter()
        .filter(|e| matches!(e.direction, GapDirection::Down) && e.filled_same_day)
        .count();
    let up_fill_rate = if n_up_gaps > 0 { up_filled as f64 / n_up_gaps as f64 } else { 0.0 };
    let down_fill_rate = if n_down_gaps > 0 { down_filled as f64 / n_down_gaps as f64 } else { 0.0 };
    let mean_gap_atrs = if n_gaps > 0 {
        events.iter().map(|e| e.gap_size_atrs).sum::<f64>() / n_gaps as f64
    } else { 0.0 };
    GapStatsReport {
        events, n_gaps, n_up_gaps, n_down_gaps,
        up_fill_rate, down_fill_rate, mean_gap_atrs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> OhlcBar { OhlcBar { open: o, high: h, low: l, close: c } }

    #[test]
    fn empty_or_short_returns_default() {
        assert_eq!(analyze(&[], &[]).n_gaps, 0);
        let bars = vec![b(100.0, 101.0, 99.0, 100.5)];
        let atr = vec![1.0];
        assert_eq!(analyze(&bars, &atr).n_gaps, 0);
    }

    #[test]
    fn no_gap_when_open_inside_prior_range() {
        let bars = vec![
            b(100.0, 102.0, 98.0, 101.0),
            b(101.0, 103.0, 100.0, 102.0),   // open 101 ∈ [98, 102]
        ];
        let atr = vec![1.0; 2];
        assert_eq!(analyze(&bars, &atr).n_gaps, 0);
    }

    #[test]
    fn up_gap_filled_same_day() {
        // Prior bar 98..102. Today opens at 103 (gap up), low 101 (fills).
        let bars = vec![
            b(100.0, 102.0, 98.0, 101.0),
            b(103.0, 105.0, 101.0, 104.5),
        ];
        let atr = vec![1.0; 2];
        let r = analyze(&bars, &atr);
        assert_eq!(r.n_up_gaps, 1);
        assert!(r.events[0].filled_same_day);
        assert!((r.events[0].gap_size - 1.0).abs() < 1e-9);
        assert!((r.up_fill_rate - 1.0).abs() < 1e-9);
    }

    #[test]
    fn up_gap_unfilled() {
        // Today gaps up to 103, low stays at 102.5 (above 102 prior high).
        let bars = vec![
            b(100.0, 102.0, 98.0, 101.0),
            b(103.0, 105.0, 102.5, 104.5),
        ];
        let atr = vec![1.0; 2];
        let r = analyze(&bars, &atr);
        assert_eq!(r.n_up_gaps, 1);
        assert!(!r.events[0].filled_same_day);
        assert!((r.up_fill_rate - 0.0).abs() < 1e-9);
    }

    #[test]
    fn down_gap_filled_same_day() {
        let bars = vec![
            b(100.0, 102.0, 98.0, 101.0),
            b(96.0, 99.0, 95.0, 98.5),     // gap down (open 96 < prior low 98), high 99 fills back to 98
        ];
        let atr = vec![1.0; 2];
        let r = analyze(&bars, &atr);
        assert_eq!(r.n_down_gaps, 1);
        assert!(r.events[0].filled_same_day);
    }

    #[test]
    fn down_gap_unfilled() {
        let bars = vec![
            b(100.0, 102.0, 98.0, 101.0),
            b(96.0, 97.5, 95.0, 96.5),
        ];
        let atr = vec![1.0; 2];
        let r = analyze(&bars, &atr);
        assert_eq!(r.n_down_gaps, 1);
        assert!(!r.events[0].filled_same_day);
    }

    #[test]
    fn mixed_series_aggregates_correctly() {
        // 4 bars: 1 up filled, 1 up unfilled, 1 down filled.
        let bars = vec![
            b(100.0, 102.0, 98.0, 101.0),    // 0
            b(103.0, 105.0, 101.0, 104.5),   // 1 up gap, filled (low 101 ≤ 102)
            b(108.0, 110.0, 107.0, 109.0),   // 2 up gap (105 → 108), unfilled (low 107 > 105)
            b(104.0, 106.0, 102.0, 103.0),   // 3 down gap (107 → 104), filled (high 106 ≥ 107? no — 106 < 107, NOT filled)
        ];
        let atr = vec![1.0; 4];
        let r = analyze(&bars, &atr);
        assert_eq!(r.n_gaps, 3);
        assert_eq!(r.n_up_gaps, 2);
        assert_eq!(r.n_down_gaps, 1);
        assert!((r.up_fill_rate - 0.5).abs() < 1e-9, "1/2 up gaps filled, got {}", r.up_fill_rate);
        assert!((r.down_fill_rate - 0.0).abs() < 1e-9, "0/1 down gaps filled, got {}", r.down_fill_rate);
    }

    #[test]
    fn mismatched_atr_returns_default() {
        let bars = vec![b(100.0, 102.0, 98.0, 101.0); 5];
        let atr = vec![1.0; 3];
        assert_eq!(analyze(&bars, &atr).n_gaps, 0);
    }
}
