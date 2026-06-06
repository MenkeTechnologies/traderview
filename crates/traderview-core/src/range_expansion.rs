//! Range expansion detector — wide-range bars following consolidation.
//!
//! The mirror of `range_contraction`: when a series of NR4/NR7 bars
//! resolves, the resolution bar's range is typically much wider than
//! the prior bars'. That wide bar is the directional signal — it
//! confirms which way the spring uncoiled.
//!
//! Detection criteria:
//!   - Current bar's true range is at least `min_expansion_atrs` × ATR.
//!   - At least one of the previous `lookback` bars had a true range
//!     below `prior_atr_max` × ATR (i.e., we ARE coming out of
//!     compression — not just a wild bar in chop).
//!
//! Pure compute. Distinct from `displacement` which scores BODY size
//! in ATRs; this module scores the WHOLE BAR range and requires prior
//! compression.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpansionConfig {
    /// Lookback bars to scan for prior compression.
    pub lookback: usize,
    /// Current bar must exceed this multiple of ATR.
    pub min_expansion_atrs: f64,
    /// At least one of the prior `lookback` bars must have range
    /// BELOW this multiple of ATR (the compression precondition).
    pub prior_atr_max: f64,
}

impl Default for ExpansionConfig {
    fn default() -> Self {
        Self {
            lookback: 5,
            min_expansion_atrs: 1.5,
            prior_atr_max: 0.7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpansionDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExpansionEvent {
    pub bar_index: usize,
    pub direction: ExpansionDirection,
    pub range_atrs: f64,
    pub compressed_bars_in_lookback: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExpansionReport {
    pub events: Vec<ExpansionEvent>,
    pub n_events: usize,
}

pub fn detect(bars: &[OhlcBar], atr: &[f64], cfg: &ExpansionConfig) -> ExpansionReport {
    let n = bars.len();
    if n == 0 || atr.len() != n || cfg.lookback == 0 || n <= cfg.lookback {
        return ExpansionReport::default();
    }
    let mut events = Vec::new();
    let prior_close = |i: usize| -> f64 { bars[i - 1].close };
    let true_range = |i: usize| -> f64 {
        if i == 0 {
            bars[0].high - bars[0].low
        } else {
            let pc = prior_close(i);
            let a = bars[i].high - bars[i].low;
            let b = (bars[i].high - pc).abs();
            let c = (bars[i].low - pc).abs();
            a.max(b).max(c)
        }
    };
    for i in cfg.lookback..n {
        let a = atr[i];
        if !(a.is_finite() && a > 0.0) {
            continue;
        }
        let cur_range = true_range(i);
        if cur_range / a < cfg.min_expansion_atrs {
            continue;
        }
        // Look back for compression.
        let mut compressed = 0usize;
        for (j, &a_j) in atr.iter().enumerate().take(i).skip(i - cfg.lookback) {
            if a_j > 0.0 {
                let tr_j = true_range(j);
                if tr_j / a_j < cfg.prior_atr_max {
                    compressed += 1;
                }
            }
        }
        if compressed == 0 {
            continue;
        }
        let direction = if bars[i].close >= bars[i - 1].close {
            ExpansionDirection::Up
        } else {
            ExpansionDirection::Down
        };
        events.push(ExpansionEvent {
            bar_index: i,
            direction,
            range_atrs: cur_range / a,
            compressed_bars_in_lookback: compressed,
        });
    }
    let n_events = events.len();
    ExpansionReport { events, n_events }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> OhlcBar {
        OhlcBar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_input_returns_no_events() {
        let r = detect(&[], &[], &ExpansionConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn mismatched_atr_returns_no_events() {
        let bars = vec![b(100.0, 99.0, 99.5); 10];
        let atr = vec![1.0; 5];
        let r = detect(&bars, &atr, &ExpansionConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn wide_bar_after_compression_fires() {
        // 5 narrow bars (range 0.5, ATR 1.0 → ratio 0.5 ≤ 0.7 = compressed),
        // then a wide bar (range 2.0, ratio 2.0 ≥ 1.5).
        let mut bars: Vec<OhlcBar> = (0..5).map(|_| b(100.5, 100.0, 100.3)).collect();
        bars.push(b(102.0, 100.0, 101.8));
        let atr: Vec<f64> = bars.iter().map(|_| 1.0).collect();
        let r = detect(&bars, &atr, &ExpansionConfig::default());
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].bar_index, 5);
        assert!(matches!(r.events[0].direction, ExpansionDirection::Up));
        assert_eq!(r.events[0].compressed_bars_in_lookback, 5);
    }

    #[test]
    fn wide_bar_without_compression_doesnt_fire() {
        // 5 NORMAL-range bars (range 1.0, ATR 1.0 → ratio 1.0 > 0.7) → no compression.
        let mut bars: Vec<OhlcBar> = (0..5).map(|_| b(101.0, 100.0, 100.5)).collect();
        bars.push(b(103.0, 100.0, 102.5));
        let atr: Vec<f64> = bars.iter().map(|_| 1.0).collect();
        let r = detect(&bars, &atr, &ExpansionConfig::default());
        assert!(r.events.is_empty(), "no prior compression → no signal");
    }

    #[test]
    fn small_range_doesnt_fire_even_with_compression() {
        // Compressed bars + a SLIGHTLY larger bar (range 1.0, ratio 1.0 < 1.5).
        let mut bars: Vec<OhlcBar> = (0..5).map(|_| b(100.5, 100.0, 100.3)).collect();
        bars.push(b(101.0, 100.0, 100.8));
        let atr: Vec<f64> = bars.iter().map(|_| 1.0).collect();
        let r = detect(&bars, &atr, &ExpansionConfig::default());
        assert!(r.events.is_empty(), "1.0 ATR < min_expansion 1.5");
    }

    #[test]
    fn down_direction_when_close_falls() {
        let mut bars: Vec<OhlcBar> = (0..5).map(|_| b(100.5, 100.0, 100.3)).collect();
        bars.push(b(100.5, 98.0, 98.5)); // wide range, close DOWN
        let atr: Vec<f64> = bars.iter().map(|_| 1.0).collect();
        let r = detect(&bars, &atr, &ExpansionConfig::default());
        assert!(matches!(r.events[0].direction, ExpansionDirection::Down));
    }

    #[test]
    fn zero_atr_skipped_safely() {
        let bars = vec![b(100.0, 99.0, 99.5); 10];
        let atr = vec![0.0; 10];
        let r = detect(&bars, &atr, &ExpansionConfig::default());
        assert!(r.events.is_empty());
    }
}
