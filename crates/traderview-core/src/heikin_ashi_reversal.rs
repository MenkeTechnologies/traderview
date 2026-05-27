//! Heikin-Ashi color-flip reversal detector.
//!
//! Heikin-Ashi smooths candle noise; the classic signal is "HA candle
//! flips color after several same-colored bars". Variants:
//!
//!   - **Strong flip**: prior 3+ HA bars were the same color, current
//!     bar flips. Body must be at least `min_body_ratio` of the bar's
//!     full range (rejection wicks count against the signal).
//!   - **Weak flip**: prior 2 bars were the same color; one-bar flip
//!     with smaller body — earlier but less reliable.
//!
//! Caller pre-computes HA candles via `crate::heikin_ashi::compute`
//! and supplies them here.
//!
//! Pure compute.

use crate::heikin_ashi::HaBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlipDirection { BullishToBearish, BearishToBullish }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlipStrength { Strong, Weak }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FlipEvent {
    pub bar_index: usize,
    pub direction: FlipDirection,
    pub strength: FlipStrength,
    /// Prior consecutive same-color streak that just ended.
    pub prior_streak: usize,
    /// Body / range fraction on the flipping bar.
    pub body_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlipConfig {
    /// Minimum body / range fraction to count the flip as Strong.
    pub min_body_ratio: f64,
    /// Minimum prior streak length for a Strong flip.
    pub strong_streak: usize,
    /// Minimum prior streak length for a Weak flip.
    pub weak_streak: usize,
}

impl Default for FlipConfig {
    fn default() -> Self { Self { min_body_ratio: 0.6, strong_streak: 3, weak_streak: 2 } }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlipReport {
    pub events: Vec<FlipEvent>,
    pub n_events: usize,
}

fn ha_color(b: HaBar) -> i8 {
    // +1 = bullish, -1 = bearish, 0 = doji.
    if b.close > b.open      { 1 }
    else if b.close < b.open { -1 }
    else                     { 0 }
}

pub fn detect(bars: &[HaBar], cfg: &FlipConfig) -> FlipReport {
    let n = bars.len();
    if n < 2 { return FlipReport::default(); }
    let mut events = Vec::new();
    let mut streak_color = ha_color(bars[0]);
    let mut streak_len = 1usize;
    for i in 1..n {
        let color = ha_color(bars[i]);
        if color == 0 {
            // Doji — keep prior streak intact but skip the flip check.
            continue;
        }
        if color == streak_color {
            streak_len += 1;
            continue;
        }
        // Color changed — evaluate flip.
        let body = (bars[i].close - bars[i].open).abs();
        let range = bars[i].high - bars[i].low;
        let body_ratio = if range > 0.0 { body / range } else { 0.0 };
        let prior = streak_len;
        let direction = if streak_color == 1 {
            FlipDirection::BullishToBearish
        } else {
            FlipDirection::BearishToBullish
        };
        let strength = if prior >= cfg.strong_streak && body_ratio >= cfg.min_body_ratio {
            Some(FlipStrength::Strong)
        } else if prior >= cfg.weak_streak {
            Some(FlipStrength::Weak)
        } else {
            None
        };
        if let Some(s) = strength {
            events.push(FlipEvent {
                bar_index: i, direction, strength: s,
                prior_streak: prior, body_ratio,
            });
        }
        streak_color = color;
        streak_len = 1;
    }
    let n_events = events.len();
    FlipReport { events, n_events }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ha(open: f64, high: f64, low: f64, close: f64) -> HaBar {
        HaBar { open, high, low, close }
    }

    #[test]
    fn empty_or_short_returns_empty() {
        assert!(detect(&[], &FlipConfig::default()).events.is_empty());
        assert!(detect(&[ha(100.0, 101.0, 99.0, 100.5)], &FlipConfig::default()).events.is_empty());
    }

    #[test]
    fn strong_flip_after_three_bull_bars() {
        // 4 bull bars (close > open), then a strong bear bar (large body).
        let bars = vec![
            ha(100.0, 101.0, 99.5, 100.8),
            ha(100.8, 102.0, 100.5, 101.5),
            ha(101.5, 103.0, 101.0, 102.5),
            ha(102.5, 103.5, 102.0, 103.0),
            ha(103.0, 103.2, 100.0, 100.5),    // bear bar, body 2.5, range 3.2 → 0.78
        ];
        let r = detect(&bars, &FlipConfig::default());
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].direction, FlipDirection::BullishToBearish));
        assert!(matches!(r.events[0].strength, FlipStrength::Strong));
        assert_eq!(r.events[0].prior_streak, 4);
    }

    #[test]
    fn weak_flip_with_small_body() {
        // 3 bull bars, then a small-body bear bar (body < 0.6 of range).
        let bars = vec![
            ha(100.0, 101.0, 99.5, 100.8),
            ha(100.8, 102.0, 100.5, 101.5),
            ha(101.5, 103.0, 101.0, 102.5),
            ha(102.5, 103.0, 100.0, 102.0),    // body 0.5, range 3.0 → 0.17
        ];
        let r = detect(&bars, &FlipConfig::default());
        assert_eq!(r.events.len(), 1);
        // Streak met for strong (3 ≥ 3) but body 0.17 < 0.6 → Weak.
        assert!(matches!(r.events[0].strength, FlipStrength::Weak),
            "expected Weak, got {:?}", r.events[0].strength);
    }

    #[test]
    fn no_flip_after_single_bar_streak() {
        // 1 bull, 1 bear, 1 bull — streak of 1 each, doesn't meet min weak streak (2).
        let bars = vec![
            ha(100.0, 101.0, 99.5, 100.8),
            ha(100.8, 100.9, 100.0, 100.3),
            ha(100.3, 101.5, 100.1, 101.0),
        ];
        let r = detect(&bars, &FlipConfig::default());
        assert!(r.events.is_empty(), "1-bar streak < weak_streak=2 → no flip");
    }

    #[test]
    fn doji_doesnt_break_streak() {
        // 3 bull, 1 doji, then bear → flip still fires (prior streak preserved).
        let bars = vec![
            ha(100.0, 101.0, 99.5, 100.8),
            ha(100.8, 102.0, 100.5, 101.5),
            ha(101.5, 103.0, 101.0, 102.5),
            ha(102.5, 102.5, 102.5, 102.5),    // doji — equal open/close
            ha(102.5, 102.6, 100.0, 100.5),    // bear flip
        ];
        let r = detect(&bars, &FlipConfig::default());
        assert_eq!(r.events.len(), 1, "doji shouldn't reset the streak");
    }

    #[test]
    fn multiple_flips_tracked_in_order() {
        // Bull*3 → Bear*3 → Bull*3 → two flips.
        let mut bars = Vec::new();
        for _ in 0..3 { bars.push(ha(100.0, 101.0, 99.5, 100.8)); }
        for _ in 0..3 { bars.push(ha(101.0, 101.5, 99.0, 99.5)); }
        for _ in 0..3 { bars.push(ha(99.5, 101.0, 99.0, 100.5)); }
        let r = detect(&bars, &FlipConfig::default());
        assert_eq!(r.events.len(), 2);
        assert!(matches!(r.events[0].direction, FlipDirection::BullishToBearish));
        assert!(matches!(r.events[1].direction, FlipDirection::BearishToBullish));
    }
}
