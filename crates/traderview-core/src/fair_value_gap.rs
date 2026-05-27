//! Fair Value Gap (FVG) detector — smart-money concepts.
//!
//! An FVG is a three-bar pattern where the gap between bar N-2's wick and
//! bar N's wick remains untouched by bar N-1. Specifically:
//!
//!   - **Bullish FVG**: `bar[i].low > bar[i-2].high`. The middle candle
//!     printed a strong-up move; the gap `[bar[i-2].high, bar[i].low]` is
//!     the unfilled inefficiency that price often returns to.
//!   - **Bearish FVG**: mirror — `bar[i].high < bar[i-2].low`.
//!
//! These gaps are tracked through subsequent bars to detect when (and
//! whether) they get "filled" (price returns into the gap zone). The
//! oldest unfilled FVG is the strongest pull on price.
//!
//! Pure compute. Caller passes OHLC bars in chronological order.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapDirection { Bullish, Bearish }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FvgEvent {
    /// Index of the bar that FORMED the gap (the right edge of the
    /// 3-bar pattern, i.e. `bar[i]`).
    pub formed_at: usize,
    pub direction: GapDirection,
    /// Low boundary of the gap (inclusive).
    pub gap_low: f64,
    /// High boundary of the gap (inclusive).
    pub gap_high: f64,
    /// Index of the bar that first re-entered the gap zone (None if still open).
    pub filled_at: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FvgReport {
    /// All FVGs found in the input, in chronological order of formation.
    pub gaps: Vec<FvgEvent>,
    /// Indexes (into `gaps`) of the still-open gaps.
    pub open_gaps: Vec<usize>,
}

pub fn detect(bars: &[OhlcBar]) -> FvgReport {
    let n = bars.len();
    if n < 3 { return FvgReport::default(); }
    let mut gaps: Vec<FvgEvent> = Vec::new();
    for i in 2..n {
        let (a, c) = (bars[i - 2], bars[i]);
        if c.low > a.high {
            gaps.push(FvgEvent {
                formed_at: i, direction: GapDirection::Bullish,
                gap_low: a.high, gap_high: c.low, filled_at: None,
            });
        } else if c.high < a.low {
            gaps.push(FvgEvent {
                formed_at: i, direction: GapDirection::Bearish,
                gap_low: c.high, gap_high: a.low, filled_at: None,
            });
        }
    }
    // For each gap, walk subsequent bars to see if/when it gets filled.
    // A bar "fills" the gap if its [low, high] range overlaps the gap zone.
    for g in &mut gaps {
        let start = g.formed_at + 1;
        for (offset, bar) in bars.iter().enumerate().skip(start).take(n.saturating_sub(start)) {
            if bar.low <= g.gap_high && bar.high >= g.gap_low {
                g.filled_at = Some(offset);
                break;
            }
        }
    }
    let open_gaps: Vec<usize> = gaps.iter().enumerate()
        .filter_map(|(i, g)| if g.filled_at.is_none() { Some(i) } else { None })
        .collect();
    FvgReport { gaps, open_gaps }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> OhlcBar { OhlcBar { high: h, low: l, close: c } }

    #[test]
    fn fewer_than_three_bars_returns_empty() {
        assert!(detect(&[]).gaps.is_empty());
        assert!(detect(&[b(100.0, 99.0, 99.5), b(101.0, 100.0, 100.5)]).gaps.is_empty());
    }

    #[test]
    fn bullish_fvg_detected_when_middle_bar_gaps_up() {
        // Bar[0]: high=100.  Bar[1]: middle. Bar[2]: low=102.
        // → gap [100, 102] uncovered by bar[1] doesn't matter for FORMATION.
        let bars = vec![
            b(100.0, 99.0, 99.5),
            b(101.5, 101.0, 101.3),    // middle bar — could be anywhere
            b(103.0, 102.0, 102.5),
        ];
        let r = detect(&bars);
        assert_eq!(r.gaps.len(), 1);
        assert!(matches!(r.gaps[0].direction, GapDirection::Bullish));
        assert!((r.gaps[0].gap_low - 100.0).abs() < 1e-9);
        assert!((r.gaps[0].gap_high - 102.0).abs() < 1e-9);
        assert!(r.gaps[0].filled_at.is_none());
        assert_eq!(r.open_gaps.len(), 1);
    }

    #[test]
    fn bearish_fvg_detected_when_middle_gaps_down() {
        // Mirror: bar[0].low=100, bar[2].high=98. Gap [98, 100].
        let bars = vec![
            b(101.0, 100.0, 100.5),
            b(99.5, 99.0, 99.3),
            b(98.0, 97.0, 97.5),
        ];
        let r = detect(&bars);
        assert_eq!(r.gaps.len(), 1);
        assert!(matches!(r.gaps[0].direction, GapDirection::Bearish));
        assert!((r.gaps[0].gap_low - 98.0).abs() < 1e-9);
        assert!((r.gaps[0].gap_high - 100.0).abs() < 1e-9);
    }

    #[test]
    fn no_fvg_when_no_gap_between_outer_bars() {
        // Bars overlap normally — no FVG.
        let bars = vec![
            b(100.0, 99.0, 99.5),
            b(100.5, 99.5, 100.0),
            b(101.0, 100.0, 100.5),
        ];
        let r = detect(&bars);
        assert!(r.gaps.is_empty());
    }

    #[test]
    fn gap_fill_marks_filled_at_index() {
        // Bullish gap [100, 102] from bars 0/1/2. Bar 3 dips into the gap zone.
        let bars = vec![
            b(100.0, 99.0, 99.5),
            b(101.5, 101.0, 101.3),
            b(103.0, 102.0, 102.5),
            b(102.5, 100.5, 101.0),    // overlaps gap [100, 102] → fills it
        ];
        let r = detect(&bars);
        assert_eq!(r.gaps[0].filled_at, Some(3));
        assert_eq!(r.open_gaps.len(), 0);
    }

    #[test]
    fn gap_stays_open_when_no_subsequent_bars_exist() {
        // The simplest "still open" scenario: the gap is the LAST event in
        // the bar stream — there are no subsequent bars to potentially fill
        // it. Trying to construct a "subsequent bars never overlap" scenario
        // in a 5-bar window doesn't work because every non-overlapping bar
        // forms a new FVG with bar[i-2]; the only clean way to leave a gap
        // open is to truncate the input right at formation.
        let bars = vec![
            b(100.0, 99.0, 99.5),
            b(101.5, 101.0, 101.3),
            b(103.0, 102.0, 102.5),
        ];
        let r = detect(&bars);
        assert_eq!(r.gaps.len(), 1);
        assert!(r.gaps[0].filled_at.is_none(), "no bars after formation → can't be filled");
        assert_eq!(r.open_gaps, vec![0]);
    }

    #[test]
    fn multiple_gaps_tracked_independently() {
        // Two bullish FVGs in sequence.
        let bars = vec![
            b(100.0, 99.0, 99.5),
            b(101.0, 100.5, 100.8),
            b(103.0, 102.0, 102.5),     // gap 1: [100, 102]
            b(103.5, 103.0, 103.2),     // makes no gap with bar 1 (high 101 vs low 103 → bullish gap [101, 103])
            b(105.0, 104.0, 104.5),     // gap 2 vs bar 2 high 103 vs low 104 → gap [103, 104]
        ];
        let r = detect(&bars);
        assert!(r.gaps.len() >= 2, "expected at least 2 gaps, got {}", r.gaps.len());
    }
}
