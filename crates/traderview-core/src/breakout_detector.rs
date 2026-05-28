//! N-day high/low breakout detector.
//!
//! The textbook trend-following entry: long when price breaks above the
//! highest high of the prior N bars; short when below the lowest low.
//! Variations: Donchian (mid-channel as additional reference), turtle
//! 20/55-day, ATR-buffer'd to filter false breakouts.
//!
//! Caller supplies an OHLC bar series + lookback N + an optional ATR
//! buffer (in price units) requiring price to exceed the level by at
//! least `buffer` to confirm. Output: per-bar breakout events with
//! direction + level + how-far-past flags.
//!
//! Pure compute. Distinct from `donchian.rs` which emits the channel —
//! this module emits the BREAKOUT EVENTS.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakoutConfig {
    pub lookback: usize,
    /// Optional confirmation buffer in price units; price must exceed the
    /// reference level by at least this much. 0.0 disables.
    pub buffer: f64,
    /// If true, only emit when the CLOSE breaches (not intraday high/low).
    /// Reduces false signals from wicks.
    pub close_only: bool,
}

impl Default for BreakoutConfig {
    fn default() -> Self { Self { lookback: 20, buffer: 0.0, close_only: false } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakoutKind { Up, Down }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BreakoutEvent {
    pub bar_index: usize,
    pub kind: BreakoutKind,
    /// The N-day high (Up) or N-day low (Down) that was breached.
    pub reference_level: f64,
    pub breach_price: f64,
    /// `breach_price - reference_level` (positive on Up, negative on Down).
    pub breach_distance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BreakoutReport {
    pub events: Vec<BreakoutEvent>,
    pub n_events: usize,
}

pub fn detect(bars: &[OhlcBar], cfg: &BreakoutConfig) -> BreakoutReport {
    let n = bars.len();
    if n == 0 || cfg.lookback == 0 || n <= cfg.lookback {
        return BreakoutReport::default();
    }
    let mut events = Vec::new();
    for i in cfg.lookback..n {
        let window = &bars[(i - cfg.lookback)..i];
        let prior_high = window.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let prior_low  = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        // Skip windows where every bar's high/low was non-finite — the
        // fold seeds (NEG_INFINITY / INFINITY) would otherwise treat any
        // finite probe price as a breakout, spuriously firing on every
        // following bar. Real data never trips this; defensive against
        // hostile JSON.
        if !prior_high.is_finite() || !prior_low.is_finite() {
            continue;
        }
        let cur = bars[i];
        let probe_up   = if cfg.close_only { cur.close } else { cur.high };
        let probe_down = if cfg.close_only { cur.close } else { cur.low };
        if probe_up > prior_high + cfg.buffer {
            events.push(BreakoutEvent {
                bar_index: i, kind: BreakoutKind::Up,
                reference_level: prior_high,
                breach_price: probe_up,
                breach_distance: probe_up - prior_high,
            });
        } else if probe_down < prior_low - cfg.buffer {
            events.push(BreakoutEvent {
                bar_index: i, kind: BreakoutKind::Down,
                reference_level: prior_low,
                breach_price: probe_down,
                breach_distance: probe_down - prior_low,
            });
        }
    }
    let n_events = events.len();
    BreakoutReport { events, n_events }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> OhlcBar { OhlcBar { high: h, low: l, close: c } }

    #[test]
    fn empty_or_short_input_returns_no_events() {
        assert!(detect(&[], &BreakoutConfig::default()).events.is_empty());
        let short: Vec<OhlcBar> = (0..10).map(|_| b(100.0, 100.0, 100.0)).collect();
        let r = detect(&short, &BreakoutConfig { lookback: 20, ..Default::default() });
        assert!(r.events.is_empty());
    }

    #[test]
    fn detects_upward_breakout_above_n_day_high() {
        // 20 bars at high=100, then bar 21 with high=105.
        let mut bars: Vec<OhlcBar> = (0..20).map(|_| b(100.0, 99.0, 99.5)).collect();
        bars.push(b(105.0, 100.0, 104.0));
        let r = detect(&bars, &BreakoutConfig::default());
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].kind, BreakoutKind::Up));
        assert_eq!(r.events[0].bar_index, 20);
        assert!((r.events[0].reference_level - 100.0).abs() < 1e-9);
        assert!((r.events[0].breach_distance - 5.0).abs() < 1e-9);
    }

    #[test]
    fn detects_downward_breakdown_below_n_day_low() {
        let mut bars: Vec<OhlcBar> = (0..20).map(|_| b(100.0, 99.0, 99.5)).collect();
        bars.push(b(99.5, 95.0, 96.0));
        let r = detect(&bars, &BreakoutConfig::default());
        assert!(matches!(r.events[0].kind, BreakoutKind::Down));
        assert!((r.events[0].reference_level - 99.0).abs() < 1e-9);
        assert!(r.events[0].breach_distance < 0.0);
    }

    #[test]
    fn buffer_filters_marginal_breakouts() {
        // Highest of prior 20 = 100. Bar 21 high = 100.10. With $0.50 buffer
        // we want NO event (10 cents isn't enough).
        let mut bars: Vec<OhlcBar> = (0..20).map(|_| b(100.0, 99.0, 99.5)).collect();
        bars.push(b(100.10, 99.5, 100.05));
        let cfg = BreakoutConfig { lookback: 20, buffer: 0.50, close_only: false };
        let r = detect(&bars, &cfg);
        assert!(r.events.is_empty(),
            "marginal breakout should be filtered by buffer, got {} events", r.events.len());
    }

    #[test]
    fn close_only_filters_wick_false_signals() {
        // Highest of prior 20 = 100. Bar 21 wicks to 105 but closes at 99.
        // close_only mode should NOT emit. Default mode WOULD emit.
        let mut bars: Vec<OhlcBar> = (0..20).map(|_| b(100.0, 99.0, 99.5)).collect();
        bars.push(b(105.0, 98.0, 99.0));
        let intraday = detect(&bars, &BreakoutConfig { lookback: 20, buffer: 0.0, close_only: false });
        assert_eq!(intraday.events.len(), 1, "wick should trigger intraday-mode breakout");
        let closes = detect(&bars, &BreakoutConfig { lookback: 20, buffer: 0.0, close_only: true });
        assert!(closes.events.is_empty(), "wick should NOT trigger close-only breakout");
    }

    #[test]
    fn lookback_zero_returns_empty() {
        let bars = vec![b(100.0, 99.0, 99.5); 5];
        let r = detect(&bars, &BreakoutConfig { lookback: 0, buffer: 0.0, close_only: false });
        assert!(r.events.is_empty());
    }

    #[test]
    fn multiple_breakouts_tracked_in_order() {
        let mut bars: Vec<OhlcBar> = (0..20).map(|_| b(100.0, 99.0, 99.5)).collect();
        bars.push(b(105.0, 100.0, 104.0));    // Up @ 20
        bars.push(b(106.0, 102.0, 103.0));    // Up @ 21 (now ref is 105 from prior 20)
        bars.push(b(101.0, 90.0, 95.0));      // Down @ 22 (low 90 < prior low 95)
        let r = detect(&bars, &BreakoutConfig::default());
        assert_eq!(r.events.len(), 3);
        assert!(matches!(r.events[0].kind, BreakoutKind::Up));
        assert!(matches!(r.events[1].kind, BreakoutKind::Up));
        assert!(matches!(r.events[2].kind, BreakoutKind::Down));
    }
}
