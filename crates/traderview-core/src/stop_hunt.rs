//! Stop-hunt / liquidity-sweep detector.
//!
//! A common HFT/algo behavior: price pierces below a recent swing low (or
//! above a swing high), triggers retail stop orders, then immediately
//! reverses back into the prior range. Smart-money traders look for these
//! "fakeouts" because they often mark the actual low of a move.
//!
//! Pattern criteria (configurable):
//!   1. Today's LOW is below the lowest low of the prior N bars (or HIGH
//!      above the highest high) by at least `min_pierce` price units.
//!   2. Today's CLOSE finishes BACK INSIDE the prior range — i.e. above
//!      the prior low (for downward sweeps) or below the prior high (for
//!      upward sweeps).
//!   3. Reversal magnitude (`close - low` / `bar_range`) ≥ `min_reversal_pct`
//!      so we don't flag bars that merely closed at the low.
//!
//! Pure compute. Distinct from `breakout_detector` which signals confirmed
//! continuation — this module is the OPPOSITE: rejected breakouts.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopHuntConfig {
    pub lookback: usize,
    /// Minimum amount (price units) by which the wick must exceed the prior
    /// extreme to count as a sweep.
    pub min_pierce: f64,
    /// Minimum reversal magnitude as a fraction of the bar's range
    /// (0.5 = the close came back at least 50% from the wick extreme).
    pub min_reversal_pct: f64,
}

impl Default for StopHuntConfig {
    fn default() -> Self { Self { lookback: 10, min_pierce: 0.0, min_reversal_pct: 0.5 } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SweepDirection {
    /// Wicked below the recent low and closed back up. Bullish reversal.
    DownSweep,
    /// Wicked above the recent high and closed back down. Bearish reversal.
    UpSweep,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SweepEvent {
    pub bar_index: usize,
    pub direction: SweepDirection,
    pub swept_level: f64,
    pub bar_extreme: f64,
    pub bar_close: f64,
    pub reversal_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SweepReport {
    pub events: Vec<SweepEvent>,
    pub n_events: usize,
}

pub fn detect(bars: &[OhlcBar], cfg: &StopHuntConfig) -> SweepReport {
    let n = bars.len();
    if n == 0 || cfg.lookback == 0 || n <= cfg.lookback {
        return SweepReport::default();
    }
    let mut events = Vec::new();
    for i in cfg.lookback..n {
        let window = &bars[(i - cfg.lookback)..i];
        let prior_high = window.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let prior_low  = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let cur = bars[i];
        let bar_range = cur.high - cur.low;
        if bar_range <= 0.0 { continue; }

        // Down sweep: low below prior_low by min_pierce AND close back above prior_low.
        if cur.low < prior_low - cfg.min_pierce && cur.close > prior_low {
            let reversal = (cur.close - cur.low) / bar_range;
            if reversal >= cfg.min_reversal_pct {
                events.push(SweepEvent {
                    bar_index: i, direction: SweepDirection::DownSweep,
                    swept_level: prior_low,
                    bar_extreme: cur.low, bar_close: cur.close,
                    reversal_pct: reversal,
                });
                continue;
            }
        }
        // Up sweep: mirror.
        if cur.high > prior_high + cfg.min_pierce && cur.close < prior_high {
            let reversal = (cur.high - cur.close) / bar_range;
            if reversal >= cfg.min_reversal_pct {
                events.push(SweepEvent {
                    bar_index: i, direction: SweepDirection::UpSweep,
                    swept_level: prior_high,
                    bar_extreme: cur.high, bar_close: cur.close,
                    reversal_pct: reversal,
                });
            }
        }
    }
    let n_events = events.len();
    SweepReport { events, n_events }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> OhlcBar { OhlcBar { high: h, low: l, close: c } }

    #[test]
    fn empty_input_returns_no_events() {
        assert!(detect(&[], &StopHuntConfig::default()).events.is_empty());
    }

    #[test]
    fn detects_classic_down_sweep() {
        // 10 bars with low=99. Bar 11 wicks to 98, closes at 99.8 (back inside).
        // Reversal: (99.8 - 98) / (100 - 98) = 1.8 / 2.0 = 0.90 → passes.
        let mut bars: Vec<OhlcBar> = (0..10).map(|_| b(100.0, 99.0, 99.5)).collect();
        bars.push(b(100.0, 98.0, 99.8));
        let r = detect(&bars, &StopHuntConfig::default());
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].direction, SweepDirection::DownSweep));
        assert!((r.events[0].swept_level - 99.0).abs() < 1e-9);
    }

    #[test]
    fn detects_up_sweep_with_close_back_inside() {
        let mut bars: Vec<OhlcBar> = (0..10).map(|_| b(101.0, 100.0, 100.5)).collect();
        bars.push(b(103.0, 100.5, 100.7));    // pierces 101, closes back below
        let r = detect(&bars, &StopHuntConfig::default());
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0].direction, SweepDirection::UpSweep));
    }

    #[test]
    fn weak_reversal_doesnt_qualify() {
        // Low pierces but close finishes near the low — not a stop hunt,
        // just a continuation. Reversal pct < 0.5.
        let mut bars: Vec<OhlcBar> = (0..10).map(|_| b(100.0, 99.0, 99.5)).collect();
        bars.push(b(100.0, 98.0, 98.5));    // close near low — only 0.25 reversal
        let r = detect(&bars, &StopHuntConfig::default());
        assert!(r.events.is_empty(),
            "close near low is continuation, not sweep — got {} events", r.events.len());
    }

    #[test]
    fn close_back_above_required() {
        // Wicked below prior low (98.5 < 99.0) BUT closed at 99.0 exactly
        // — that's at the prior_low boundary, not strictly above.
        let mut bars: Vec<OhlcBar> = (0..10).map(|_| b(100.0, 99.0, 99.5)).collect();
        bars.push(b(100.0, 98.5, 99.0));
        let r = detect(&bars, &StopHuntConfig::default());
        assert!(r.events.is_empty(), "close at prior_low boundary is not a sweep");
    }

    #[test]
    fn min_pierce_filters_marginal_wicks() {
        // Pierce of just 0.05 below prior low — config requires 0.10 minimum.
        let mut bars: Vec<OhlcBar> = (0..10).map(|_| b(100.0, 99.0, 99.5)).collect();
        bars.push(b(100.0, 98.95, 99.8));
        let cfg = StopHuntConfig { lookback: 10, min_pierce: 0.10, min_reversal_pct: 0.5 };
        let r = detect(&bars, &cfg);
        assert!(r.events.is_empty(), "0.05 wick below pivot doesn't qualify with 0.10 buffer");
    }

    #[test]
    fn zero_range_bars_dont_panic() {
        // High == low → bar_range = 0 → skip without crash.
        let mut bars: Vec<OhlcBar> = (0..10).map(|_| b(100.0, 99.0, 99.5)).collect();
        bars.push(b(99.5, 99.5, 99.5));
        let r = detect(&bars, &StopHuntConfig::default());
        // Zero range → skipped silently.
        assert!(r.events.is_empty());
    }
}
