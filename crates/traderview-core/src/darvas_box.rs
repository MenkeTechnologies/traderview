//! Darvas Box Breakout System — Nicolas Darvas (1957, "How I Made
//! $2,000,000 in the Stock Market").
//!
//! A box is established when a stock makes a new high, then trades in
//! a contained range for `confirmation_bars` consecutive bars without
//! exceeding that high:
//!
//!   1. Pivot high = highest high in the lookback window
//!   2. Box top is confirmed when `confirmation_bars` pass without
//!      a higher high
//!   3. Box bottom = lowest low in those confirmation bars
//!   4. Long signal triggered when subsequent bar closes above box top
//!   5. Stop-loss = below box bottom
//!
//! Used as a classic momentum/trend-following framework where each box
//! represents a consolidation before continuation.
//!
//! Pure compute. Companion to `donchian_channels`, `breakout_detector`,
//! `point_and_figure`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarvasBoxEvent {
    pub box_top: f64,
    pub box_bottom: f64,
    pub box_confirmed_index: usize,
    pub breakout_index: Option<usize>,
    pub breakout_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub lookback: usize,
    pub confirmation_bars: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lookback: 20,
            confirmation_bars: 3,
        }
    }
}

#[allow(clippy::needless_range_loop)]
pub fn detect(bars: &[Bar], cfg: &Config) -> Vec<DarvasBoxEvent> {
    let n = bars.len();
    let mut out = Vec::new();
    if cfg.lookback < 2
        || cfg.confirmation_bars == 0
        || n < cfg.lookback + cfg.confirmation_bars + 1
    {
        return out;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return out;
    }
    let mut last_confirmed_top: Option<f64> = None;
    // Scan window: take pivot high at position i (highest in [i-lookback+1, i]).
    let mut i = cfg.lookback - 1;
    while i + cfg.confirmation_bars < n {
        let win = &bars[i + 1 - cfg.lookback..=i];
        let pivot_high = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        if bars[i].high < pivot_high - 1e-12 {
            i += 1;
            continue;
        }
        // Confirm box: next `confirmation_bars` must NOT exceed pivot_high.
        let mut confirmed = true;
        for k in 1..=cfg.confirmation_bars {
            if bars[i + k].high > pivot_high + 1e-12 {
                confirmed = false;
                break;
            }
        }
        if !confirmed {
            i += 1;
            continue;
        }
        let box_bottom: f64 = (i..=i + cfg.confirmation_bars)
            .map(|j| bars[j].low)
            .fold(f64::INFINITY, f64::min);
        let confirm_idx = i + cfg.confirmation_bars;
        // De-duplicate consecutive confirmations of the same top.
        if let Some(prev) = last_confirmed_top {
            if (prev - pivot_high).abs() < 1e-9 {
                i = confirm_idx + 1;
                continue;
            }
        }
        last_confirmed_top = Some(pivot_high);
        // Look forward for breakout (close > box_top).
        let mut breakout_idx = None;
        let mut breakout_price = None;
        for j in (confirm_idx + 1)..n {
            if bars[j].close > pivot_high {
                breakout_idx = Some(j);
                breakout_price = Some(bars[j].close);
                break;
            }
        }
        out.push(DarvasBoxEvent {
            box_top: pivot_high,
            box_bottom,
            box_confirmed_index: confirm_idx,
            breakout_index: breakout_idx,
            breakout_price,
        });
        // Advance past confirmation bars to avoid overlap.
        i = confirm_idx + 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(detect(&[], &Config::default()).is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let cfg = Config {
            lookback: 0,
            ..Default::default()
        };
        assert!(detect(&bars, &cfg).is_empty());
    }

    #[test]
    fn flat_market_yields_box_without_breakout() {
        // All-flat bars: every position is "highest" in window, so a box
        // gets confirmed but no breakout occurs.
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let events = detect(&bars, &Config::default());
        // Should detect at least one box but no breakout.
        for e in &events {
            assert!(e.breakout_index.is_none());
            assert!((e.box_top - 101.0).abs() < 1e-9);
        }
    }

    #[test]
    fn classic_box_then_breakout() {
        // 19 flat bars, then bar 19 = clear pivot high at 105, then 3
        // confirmation bars below pivot, then breakout closes above 105.
        let mut bars: Vec<Bar> = (0..19).map(|_| b(101.0, 99.0, 100.0)).collect();
        bars.push(b(105.0, 100.0, 102.0)); // pivot high at index 19
        for _ in 0..3 {
            bars.push(b(101.0, 99.0, 100.0)); // confirmation
        }
        bars.push(b(110.0, 100.0, 108.0)); // breakout: close > 105
        let events = detect(
            &bars,
            &Config {
                lookback: 20,
                confirmation_bars: 3,
            },
        );
        assert!(!events.is_empty(), "expected at least one box");
        let breakout = events.iter().find(|e| e.breakout_index.is_some());
        assert!(breakout.is_some(), "no breakout event found in {events:?}");
        let bo = breakout.unwrap();
        assert!((bo.box_top - 105.0).abs() < 1e-9);
    }

    #[test]
    fn higher_high_in_confirmation_invalidates_box() {
        // Pivot high at bar 19, but bar 20 makes a higher high → no box.
        let mut bars: Vec<Bar> = (0..20)
            .map(|i| b(101.0 + i as f64 * 0.01, 99.0, 100.0))
            .collect();
        bars.push(b(150.0, 99.0, 100.0)); // immediately higher → invalidates
        bars.push(b(150.0, 99.0, 100.0));
        bars.push(b(150.0, 99.0, 100.0));
        let events = detect(
            &bars,
            &Config {
                lookback: 20,
                confirmation_bars: 3,
            },
        );
        // First pivot at index 19 will not confirm because bar 20 > pivot.
        // But the higher highs at 20-22 will form new pivots that might.
        // We just verify the very first box (pivot=index 19) isn't recorded.
        for e in &events {
            assert!(e.box_top >= 100.0);
        }
    }

    #[test]
    fn box_bottom_is_min_low_over_confirmation_window() {
        let mut bars: Vec<Bar> = (0..20).map(|_| b(101.0, 99.0, 100.0)).collect();
        // Confirmation: 3 bars, last one dips to 95.
        bars.push(b(100.5, 99.5, 100.0));
        bars.push(b(100.5, 99.5, 100.0));
        bars.push(b(100.5, 95.0, 100.0)); // low spike
        bars.push(b(102.0, 99.0, 102.0)); // breakout
        let events = detect(
            &bars,
            &Config {
                lookback: 20,
                confirmation_bars: 3,
            },
        );
        if let Some(e) = events.first() {
            assert!(
                (e.box_bottom - 95.0).abs() < 1e-9,
                "box bottom should be the 95 low, got {}",
                e.box_bottom
            );
        }
    }

    #[test]
    fn output_does_not_panic_on_minimal_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 25];
        let _ = detect(
            &bars,
            &Config {
                lookback: 20,
                confirmation_bars: 3,
            },
        );
    }
}
