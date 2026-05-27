//! Displacement detector — strong-momentum bars relative to recent ATR.
//!
//! SMC traders call a single bar whose body is multiple-ATR-wide a
//! "displacement candle." It signals institutional intent: someone aggressive
//! enough to push price the entire daily/hourly range in a few minutes.
//! Often the displacement is the move OFF a key level (FVG, order block,
//! liquidity sweep), confirming the bias.
//!
//! Detection:
//!   - **body_ratio**: |close − open| / ATR(period)
//!   - Bar is a displacement when `body_ratio ≥ min_atrs` AND the bar
//!     closes in its directional half (so a 3-ATR up-bar that fully
//!     retraces and closes at the low isn't a displacement).
//!
//! Pure compute. Caller pre-computes ATR via `crate::indicators::atr`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplacementConfig {
    /// Minimum body size in ATRs to qualify.
    pub min_atrs: f64,
    /// Minimum close-position within the bar's range required to qualify.
    /// 0.5 = close in the directional half (top half for up-bars).
    pub min_close_position: f64,
}

impl Default for DisplacementConfig {
    fn default() -> Self { Self { min_atrs: 1.5, min_close_position: 0.6 } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplacementDirection { Up, Down }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DisplacementEvent {
    pub bar_index: usize,
    pub direction: DisplacementDirection,
    pub body_atrs: f64,
    /// Fraction of bar range where the close landed (0 = low, 1 = high).
    pub close_position: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisplacementReport {
    pub events: Vec<DisplacementEvent>,
    pub n_events: usize,
}

pub fn detect(bars: &[OhlcBar], atr: &[f64], cfg: &DisplacementConfig) -> DisplacementReport {
    let n = bars.len();
    if n == 0 || atr.len() != n || cfg.min_atrs <= 0.0 {
        return DisplacementReport::default();
    }
    let mut events = Vec::new();
    for i in 0..n {
        let a = atr[i];
        if !(a.is_finite() && a > 0.0) { continue; }
        let bar = bars[i];
        let body = (bar.close - bar.open).abs();
        let body_atrs = body / a;
        if body_atrs < cfg.min_atrs { continue; }
        let range = bar.high - bar.low;
        if range <= 0.0 { continue; }
        // Close position: 0 = low, 1 = high.
        let close_pos = (bar.close - bar.low) / range;
        let (direction, qualifying) = if bar.close > bar.open {
            (DisplacementDirection::Up, close_pos)
        } else {
            (DisplacementDirection::Down, 1.0 - close_pos)
        };
        if qualifying < cfg.min_close_position { continue; }
        events.push(DisplacementEvent {
            bar_index: i, direction, body_atrs, close_position: close_pos,
        });
    }
    let n_events = events.len();
    DisplacementReport { events, n_events }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> OhlcBar { OhlcBar { open: o, high: h, low: l, close: c } }

    #[test]
    fn empty_input_returns_no_events() {
        let r = detect(&[], &[], &DisplacementConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn mismatched_lengths_return_no_events() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.5)];
        let atr  = vec![1.0, 1.0];
        let r = detect(&bars, &atr, &DisplacementConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn detects_up_displacement_bar() {
        // Body = 3 (100 → 103), ATR = 1.0, body_atrs = 3.0 — above 1.5 threshold.
        // Close (103) sits at top of range (high 103, low 100) → close_pos = 1.0.
        let bars = vec![b(100.0, 103.0, 100.0, 103.0)];
        let atr  = vec![1.0];
        let r = detect(&bars, &atr, &DisplacementConfig::default());
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].direction, DisplacementDirection::Up));
        assert!((r.events[0].body_atrs - 3.0).abs() < 1e-9);
    }

    #[test]
    fn detects_down_displacement_bar() {
        let bars = vec![b(100.0, 100.0, 97.0, 97.0)];
        let atr  = vec![1.0];
        let r = detect(&bars, &atr, &DisplacementConfig::default());
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].direction, DisplacementDirection::Down));
    }

    #[test]
    fn small_body_skipped() {
        // Body = 0.5, ATR = 1.0 → body_atrs = 0.5 — below 1.5.
        let bars = vec![b(100.0, 100.5, 99.5, 100.5)];
        let atr  = vec![1.0];
        let r = detect(&bars, &atr, &DisplacementConfig::default());
        assert!(r.events.is_empty(), "small body shouldn't qualify");
    }

    #[test]
    fn up_bar_closing_low_doesnt_qualify() {
        // Open 100, traveled up to high 103, but CLOSED at 100.5 (near the low).
        // Body = 0.5 → too small. Adjust: open 100, close 101, body=1×ATR=1.0
        // (still below 1.5 threshold). Bump ATR down to test close-position rule
        // independently.
        // Body = 100 → 102.5 (body_atrs = 2.5), but close at 100.5 (close_pos = 0.05).
        // open 100, close 102.5 ... hmm let me redesign. Up bar means close > open.
        // For close_position to be LOW on an up bar, we'd need close just barely above
        // open with the high far above. Body=0.6 ATR=0.2 body_atrs=3 close_pos near 0.
        let bars = vec![b(100.0, 110.0, 99.9, 100.6)];
        let atr  = vec![0.2];
        let r = detect(&bars, &atr, &DisplacementConfig::default());
        // body=0.6, body_atrs=3 — passes size. But close_pos = (100.6-99.9)/(110-99.9) = 0.07.
        // Up-bar qualifying = close_pos = 0.07 < 0.6 → reject.
        assert!(r.events.is_empty(),
            "up-bar closing near low isn't a displacement");
    }

    #[test]
    fn zero_atr_bars_skipped_safely() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.5)];
        let atr  = vec![0.0];
        let r = detect(&bars, &atr, &DisplacementConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn doji_bar_with_zero_range_skipped() {
        let bars = vec![b(100.0, 100.0, 100.0, 100.0)];
        let atr  = vec![1.0];
        let r = detect(&bars, &atr, &DisplacementConfig::default());
        assert!(r.events.is_empty(), "zero-range bar can't displace");
    }
}
