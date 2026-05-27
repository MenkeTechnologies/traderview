//! Three-bar reversal pattern detector.
//!
//! A classic short-term reversal: a "key reversal" forms when three
//! consecutive bars satisfy:
//!
//!   - **Bullish 3-bar reversal**: bar i-2 is a down-bar making a new
//!     low; bar i-1 is small/inside/doji; bar i is an up-bar that closes
//!     above bar i-2's high. The pattern marks the end of selling.
//!   - **Bearish 3-bar reversal**: mirror — up-bar new high, small bar,
//!     down-bar closing below bar i-2's low.
//!
//! Detection uses simple OHLC arithmetic — no indicator dependencies.
//! Caller can wire this through chart annotations or strategy alerts.
//!
//! Pure compute.

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
pub enum ReversalKind { Bullish, Bearish }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReversalEvent {
    /// Index of the THIRD bar (the confirming bar).
    pub bar_index: usize,
    pub kind: ReversalKind,
    pub bar1_open: f64,
    pub bar3_close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReversalReport {
    pub events: Vec<ReversalEvent>,
    pub n_events: usize,
}

pub fn detect(bars: &[OhlcBar]) -> ReversalReport {
    let n = bars.len();
    if n < 3 { return ReversalReport::default(); }
    let mut events = Vec::new();
    for i in 2..n {
        let b1 = bars[i - 2];
        let b2 = bars[i - 1];
        let b3 = bars[i];
        let b1_down = b1.close < b1.open;
        let b1_up   = b1.close > b1.open;
        let b3_down = b3.close < b3.open;
        let b3_up   = b3.close > b3.open;
        // Middle bar must be "small" — body ≤ 50% of bar-1's body.
        let b1_body = (b1.close - b1.open).abs();
        let b2_body = (b2.close - b2.open).abs();
        let middle_small = b2_body <= 0.5 * b1_body;

        if b1_down && b3_up && middle_small && b3.close > b1.high {
            events.push(ReversalEvent {
                bar_index: i, kind: ReversalKind::Bullish,
                bar1_open: b1.open, bar3_close: b3.close,
            });
        } else if b1_up && b3_down && middle_small && b3.close < b1.low {
            events.push(ReversalEvent {
                bar_index: i, kind: ReversalKind::Bearish,
                bar1_open: b1.open, bar3_close: b3.close,
            });
        }
    }
    let n_events = events.len();
    ReversalReport { events, n_events }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> OhlcBar { OhlcBar { open: o, high: h, low: l, close: c } }

    #[test]
    fn empty_or_short_returns_no_events() {
        assert!(detect(&[]).events.is_empty());
        assert!(detect(&[b(100.0, 101.0, 99.0, 100.5)]).events.is_empty());
    }

    #[test]
    fn classic_bullish_three_bar_reversal() {
        // Bar 1: big down (open 105, close 100). Bar 2: small doji at 99.
        // Bar 3: big up closing at 106 — above bar 1's high of 105.5.
        let bars = vec![
            b(105.0, 105.5, 99.8, 100.0),    // big down body 5
            b(99.8, 100.2, 99.0, 99.2),      // small body 0.6
            b(99.2, 106.5, 99.0, 106.0),     // big up, closes above 105.5
        ];
        let r = detect(&bars);
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].kind, ReversalKind::Bullish));
        assert_eq!(r.events[0].bar_index, 2);
    }

    #[test]
    fn classic_bearish_three_bar_reversal() {
        // Bar 1: big up. Bar 2: small. Bar 3: big down below bar 1's low.
        let bars = vec![
            b(100.0, 105.5, 99.8, 105.0),
            b(105.0, 105.5, 104.5, 104.8),    // body 0.2
            b(104.8, 105.0, 99.0, 99.5),       // closes below bar1.low 99.8
        ];
        let r = detect(&bars);
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].kind, ReversalKind::Bearish));
    }

    #[test]
    fn middle_bar_too_large_disqualifies() {
        // Middle body 3.0 vs bar-1 body 5.0 → 0.6 > 0.5 threshold → no pattern.
        let bars = vec![
            b(105.0, 105.5, 99.8, 100.0),    // body 5
            b(100.0, 104.0, 99.0, 103.0),    // body 3 — too big
            b(103.0, 107.0, 102.0, 106.0),
        ];
        let r = detect(&bars);
        assert!(r.events.is_empty());
    }

    #[test]
    fn bar3_close_below_bar1_high_disqualifies_bullish() {
        // Bar 3 closes at 105 — equal to bar1.high, not ABOVE.
        let bars = vec![
            b(105.0, 105.5, 99.8, 100.0),
            b(99.8, 100.2, 99.0, 99.2),
            b(99.2, 105.0, 99.0, 105.0),
        ];
        let r = detect(&bars);
        assert!(r.events.is_empty(), "close must be strictly above bar1's high");
    }

    #[test]
    fn doji_middle_bar_counts() {
        // Doji = body 0 ≤ 50% of bar1 body. Should still qualify.
        let bars = vec![
            b(105.0, 105.5, 99.8, 100.0),
            b(99.5, 100.0, 99.0, 99.5),    // doji (open == close)
            b(99.5, 107.0, 99.0, 106.5),
        ];
        let r = detect(&bars);
        assert_eq!(r.events.len(), 1);
    }

    #[test]
    fn first_bar_doji_disqualifies() {
        // Bar 1 is doji (body=0) → not bearish nor bullish.
        let bars = vec![
            b(100.0, 101.0, 99.0, 100.0),
            b(100.0, 100.5, 99.5, 100.2),
            b(100.2, 105.0, 100.0, 104.0),
        ];
        let r = detect(&bars);
        assert!(r.events.is_empty(), "doji bar 1 isn't a setup bar");
    }
}
