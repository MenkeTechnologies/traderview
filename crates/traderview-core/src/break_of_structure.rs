//! Break of Structure (BOS) detector — smart-money concepts trend confirmation.
//!
//! Walking a series of swing highs and lows in chronological order, a Bullish
//! BOS fires when the close BREAKS ABOVE the most recent confirmed swing
//! HIGH (the prior trend-high is taken out). A Bearish BOS fires when close
//! breaks BELOW the most recent confirmed swing LOW. BOS confirms the
//! trend that was in place, distinct from CHoCH which signals a reversal.
//!
//! Caller pre-computes swing points via `crate::swing_points` and supplies
//! them alongside the close series. Each closing bar is checked against
//! the most-recent appropriate swing.
//!
//! Pure compute. Distinct from `breakout_detector` (rolling N-bar high/low):
//! BOS uses pivots, not lookback windows.

use crate::swing_points::{SwingKind, SwingPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BosKind {
    /// Close > prior swing high — trend continuation up.
    Bullish,
    /// Close < prior swing low — trend continuation down.
    Bearish,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BosEvent {
    pub bar_index: usize,
    pub kind: BosKind,
    /// The swing pivot price that got broken.
    pub broken_level: f64,
    pub close_at_break: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BosReport {
    pub events: Vec<BosEvent>,
    pub last_event: Option<BosEvent>,
}

pub fn detect(closes: &[f64], swings: &[SwingPoint]) -> BosReport {
    if closes.is_empty() || swings.is_empty() {
        return BosReport::default();
    }
    let mut events = Vec::new();
    // For each bar, find the most recent swing (high or low) confirmed
    // BEFORE this bar. Check whether the close breaks the prior swing.
    // Track which swings have already been "broken" so we don't re-fire.
    let mut last_broken_high_idx: Option<usize> = None;
    let mut last_broken_low_idx: Option<usize> = None;
    for (i, &close) in closes.iter().enumerate() {
        // Find the most recent swing HIGH and LOW with index < i.
        let mut latest_high: Option<&SwingPoint> = None;
        let mut latest_low: Option<&SwingPoint> = None;
        for s in swings {
            if s.index >= i {
                break;
            }
            match s.kind {
                SwingKind::High => latest_high = Some(s),
                SwingKind::Low => latest_low = Some(s),
            }
        }
        if let Some(sh) = latest_high {
            if Some(sh.index) != last_broken_high_idx && close > sh.price {
                events.push(BosEvent {
                    bar_index: i,
                    kind: BosKind::Bullish,
                    broken_level: sh.price,
                    close_at_break: close,
                });
                last_broken_high_idx = Some(sh.index);
            }
        }
        if let Some(sl) = latest_low {
            if Some(sl.index) != last_broken_low_idx && close < sl.price {
                events.push(BosEvent {
                    bar_index: i,
                    kind: BosKind::Bearish,
                    broken_level: sl.price,
                    close_at_break: close,
                });
                last_broken_low_idx = Some(sl.index);
            }
        }
    }
    let last_event = events.last().copied();
    BosReport { events, last_event }
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
    fn empty_input_returns_empty() {
        assert!(detect(&[], &[]).events.is_empty());
        assert!(detect(&[100.0, 101.0], &[]).events.is_empty());
    }

    #[test]
    fn bullish_bos_when_close_breaks_prior_swing_high() {
        // Swing high at index 2 = 105. Bar 5 closes at 106 → Bullish BOS.
        let closes = vec![100.0, 102.0, 105.0, 103.0, 104.0, 106.0];
        let swings = vec![sp(2, 105.0, SwingKind::High)];
        let r = detect(&closes, &swings);
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].kind, BosKind::Bullish));
        assert_eq!(r.events[0].bar_index, 5);
        assert!((r.events[0].broken_level - 105.0).abs() < 1e-9);
    }

    #[test]
    fn bearish_bos_when_close_breaks_prior_swing_low() {
        let closes = vec![100.0, 98.0, 95.0, 97.0, 96.0, 94.0];
        let swings = vec![sp(2, 95.0, SwingKind::Low)];
        let r = detect(&closes, &swings);
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].kind, BosKind::Bearish));
        assert_eq!(r.events[0].bar_index, 5);
    }

    #[test]
    fn each_swing_fires_at_most_once() {
        // Same swing high (105) gets broken at bar 5 (106) and would
        // technically be "above" again at bar 6 (107) — but only the
        // FIRST break fires; subsequent bars don't re-emit the same event.
        let closes = vec![100.0, 102.0, 105.0, 103.0, 104.0, 106.0, 107.0];
        let swings = vec![sp(2, 105.0, SwingKind::High)];
        let r = detect(&closes, &swings);
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].bar_index, 5);
    }

    #[test]
    fn newer_swing_supersedes_older_one() {
        // Swing high at idx 2 = 105 (broken at idx 5 = 106).
        // Then a new swing high at idx 6 = 108. First subsequent close that
        // breaks 108 is at idx 8 (109 > 108). 110 at idx 9 would re-fire if
        // the engine didn't dedupe — but it shouldn't.
        let closes = vec![
            100.0, 102.0, 105.0, 103.0, 104.0, 106.0, 108.0, 107.0, 109.0, 110.0,
        ];
        let swings = vec![sp(2, 105.0, SwingKind::High), sp(6, 108.0, SwingKind::High)];
        let r = detect(&closes, &swings);
        assert_eq!(r.events.len(), 2);
        assert_eq!(r.events[0].bar_index, 5);
        assert_eq!(
            r.events[1].bar_index, 8,
            "first close > 108 lives at idx 8 (close 109), not idx 9"
        );
        assert!((r.events[1].broken_level - 108.0).abs() < 1e-9);
    }

    #[test]
    fn swing_at_or_after_current_bar_is_excluded() {
        // Swing high at index 5. Bar 5 close = 110 — but the swing isn't
        // "in the past" at i=5, so it shouldn't fire BOS at i=5.
        let closes = vec![100.0, 100.0, 100.0, 100.0, 100.0, 110.0, 100.0];
        let swings = vec![sp(5, 105.0, SwingKind::High)];
        let r = detect(&closes, &swings);
        // Bar 5 → no swing strictly before it → no event.
        // Bar 6 close=100 → swing high 105 in the past → 100 < 105 so no break.
        assert!(r.events.is_empty());
    }
}
