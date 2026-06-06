//! Change of Character (CHoCH) detector — SMC reversal indicator.
//!
//! While `break_of_structure` confirms trend CONTINUATION, CHoCH signals
//! REVERSAL: in an uptrend (sequence of HH/HL), a close breaking BELOW
//! the most recent higher-LOW reveals a structural shift to potential
//! downtrend. In a downtrend (LH/LL), close breaking ABOVE the most
//! recent lower-HIGH signals potential uptrend.
//!
//! Caller supplies:
//!   1. Close series.
//!   2. Swing points (high+low pivots, chronological).
//!   3. Pre-determined trend direction at session start (Up or Down).
//!
//! When a CHoCH fires, the trend direction effectively flips — but this
//! module is stateless and only emits events; the caller maintains
//! trend state across calls.
//!
//! Pure compute.

use crate::swing_points::{SwingKind, SwingPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChochKind {
    /// Uptrend reversed: close broke below a higher-low.
    Bearish,
    /// Downtrend reversed: close broke above a lower-high.
    Bullish,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChochEvent {
    pub bar_index: usize,
    pub kind: ChochKind,
    /// The swing pivot price whose break triggered the CHoCH.
    pub broken_level: f64,
    pub close_at_break: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChochReport {
    pub events: Vec<ChochEvent>,
    pub last_event: Option<ChochEvent>,
}

pub fn detect(closes: &[f64], swings: &[SwingPoint], initial_trend: TrendDirection) -> ChochReport {
    if closes.is_empty() || swings.is_empty() {
        return ChochReport::default();
    }
    let mut events = Vec::new();
    let mut trend = initial_trend;
    for (i, &close) in closes.iter().enumerate() {
        // Find the most recent swing of the relevant kind for the current trend.
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
        match trend {
            TrendDirection::Up => {
                if let Some(sl) = latest_low {
                    if close < sl.price {
                        events.push(ChochEvent {
                            bar_index: i,
                            kind: ChochKind::Bearish,
                            broken_level: sl.price,
                            close_at_break: close,
                        });
                        trend = TrendDirection::Down;
                    }
                }
            }
            TrendDirection::Down => {
                if let Some(sh) = latest_high {
                    if close > sh.price {
                        events.push(ChochEvent {
                            bar_index: i,
                            kind: ChochKind::Bullish,
                            broken_level: sh.price,
                            close_at_break: close,
                        });
                        trend = TrendDirection::Up;
                    }
                }
            }
        }
    }
    let last_event = events.last().copied();
    ChochReport { events, last_event }
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
    fn empty_inputs_return_empty() {
        assert!(detect(&[], &[], TrendDirection::Up).events.is_empty());
        assert!(detect(&[100.0], &[], TrendDirection::Up).events.is_empty());
    }

    #[test]
    fn uptrend_bearish_choch_on_break_of_higher_low() {
        // Uptrend with higher-low at idx 2 = 95. Close at idx 5 = 93 → Bearish CHoCH.
        let closes = vec![100.0, 102.0, 95.0, 105.0, 100.0, 93.0];
        let swings = vec![sp(2, 95.0, SwingKind::Low)];
        let r = detect(&closes, &swings, TrendDirection::Up);
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].kind, ChochKind::Bearish));
        assert_eq!(r.events[0].bar_index, 5);
    }

    #[test]
    fn downtrend_bullish_choch_on_break_of_lower_high() {
        let closes = vec![100.0, 98.0, 102.0, 95.0, 100.0, 105.0];
        let swings = vec![sp(2, 102.0, SwingKind::High)];
        let r = detect(&closes, &swings, TrendDirection::Down);
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].kind, ChochKind::Bullish));
        assert_eq!(r.events[0].bar_index, 5);
    }

    #[test]
    fn no_choch_when_close_stays_within_structure() {
        // Uptrend with higher-low at 95. Close never goes below 95.
        let closes = vec![100.0, 102.0, 95.0, 96.0, 97.0, 96.0];
        let swings = vec![sp(2, 95.0, SwingKind::Low)];
        let r = detect(&closes, &swings, TrendDirection::Up);
        assert!(r.events.is_empty());
    }

    #[test]
    fn trend_flips_after_choch_so_subsequent_breaks_use_other_side() {
        // Start uptrend. CHoCH bearish at idx 5 (close 93 < 95 low).
        // Now downtrend — a later close above a lower-high (110) should
        // fire bullish CHoCH at idx 9.
        let closes = vec![
            100.0, 102.0, 95.0, 105.0, 100.0, 93.0, 108.0, 100.0, 105.0, 115.0,
        ];
        let swings = vec![sp(2, 95.0, SwingKind::Low), sp(6, 110.0, SwingKind::High)];
        let r = detect(&closes, &swings, TrendDirection::Up);
        assert_eq!(r.events.len(), 2);
        assert!(matches!(r.events[0].kind, ChochKind::Bearish));
        assert!(matches!(r.events[1].kind, ChochKind::Bullish));
        assert_eq!(r.events[1].bar_index, 9);
    }

    #[test]
    fn each_choch_only_fires_once_per_swing() {
        // Bearish CHoCH at idx 5 (close 93 below 95-low). Trend flips
        // to Down. Subsequent closes 93, 92 are still below 95 but the
        // trend is now Down and we look at HIGHS, not LOWS → no spurious
        // re-fire of the same event.
        let closes = vec![100.0, 102.0, 95.0, 105.0, 100.0, 93.0, 92.0, 91.0];
        let swings = vec![sp(2, 95.0, SwingKind::Low)];
        let r = detect(&closes, &swings, TrendDirection::Up);
        assert_eq!(r.events.len(), 1, "single CHoCH per swing");
    }
}
