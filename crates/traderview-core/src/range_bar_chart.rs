//! Range Bar Aggregator — alternative-bar chart where each emitted bar
//! has exactly `range` price units (high - low) and time is ignored.
//!
//! Bars accumulate prints until the range from `bar_low` to `bar_high`
//! reaches `target_range`, then close and start a new bar.
//!
//! Range bars filter out flat low-volatility periods (no bars emitted)
//! and emit more bars during volatile periods — providing a more
//! activity-uniform view of the tape.
//!
//! Pure compute. Companion to `renko`, `point_and_figure`,
//! `kagi_chart`, `three_line_break`, `volume_bar_chart`,
//! `tick_bar_chart`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Print {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RangeBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub tick_count: u32,
}

pub fn compute(prints: &[Print], target_range: f64) -> Vec<RangeBar> {
    let mut out = Vec::new();
    if prints.is_empty() || !target_range.is_finite() || target_range <= 0.0 {
        return out;
    }
    if prints
        .iter()
        .any(|p| !p.price.is_finite() || !p.size.is_finite() || p.price <= 0.0 || p.size < 0.0)
    {
        return out;
    }
    let mut open = prints[0].price;
    let mut high = prints[0].price;
    let mut low = prints[0].price;
    let mut volume = prints[0].size;
    let mut tick_count = 1_u32;
    for p in prints.iter().skip(1) {
        if p.price > high {
            high = p.price;
        }
        if p.price < low {
            low = p.price;
        }
        volume += p.size;
        tick_count += 1;
        if high - low >= target_range {
            out.push(RangeBar {
                open,
                high,
                low,
                close: p.price,
                volume,
                tick_count,
            });
            open = p.price;
            high = p.price;
            low = p.price;
            volume = 0.0;
            tick_count = 0;
        }
    }
    // Don't emit the trailing partial bar — only full-range bars are
    // valid in range-bar charts.
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(price: f64, size: f64) -> Print {
        Print { price, size }
    }

    #[test]
    fn empty_or_invalid_returns_empty() {
        assert!(compute(&[], 1.0).is_empty());
        let prints = vec![p(100.0, 10.0); 5];
        assert!(compute(&prints, 0.0).is_empty());
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let prints = vec![p(f64::NAN, 10.0)];
        assert!(compute(&prints, 1.0).is_empty());
        let prints2 = vec![p(100.0, -1.0)];
        assert!(compute(&prints2, 1.0).is_empty());
    }

    #[test]
    fn flat_market_no_bars() {
        let prints = vec![p(100.0, 10.0); 50];
        assert!(compute(&prints, 1.0).is_empty());
    }

    #[test]
    fn pure_uptrend_emits_range_bars() {
        // Prints 100, 101, 102, ..., 110. Target range 5.0.
        // Bar 1 closes at 105 (high=105, low=100, range=5). Then opens
        // at 105, accumulates until 110. Bar 2 closes at 110.
        let prints: Vec<_> = (0..11).map(|i| p(100.0 + i as f64, 10.0)).collect();
        let bars = compute(&prints, 5.0);
        assert_eq!(bars.len(), 2);
        assert!((bars[0].open - 100.0).abs() < 1e-9);
        assert!((bars[0].close - 105.0).abs() < 1e-9);
        assert!((bars[1].close - 110.0).abs() < 1e-9);
    }

    #[test]
    fn small_range_no_bar() {
        let prints = vec![p(100.0, 10.0), p(100.5, 10.0), p(100.3, 10.0)];
        let bars = compute(&prints, 1.0);
        assert!(bars.is_empty());
    }

    #[test]
    fn volume_aggregates_per_bar() {
        let prints = vec![p(100.0, 10.0), p(103.0, 20.0), p(105.0, 30.0)];
        let bars = compute(&prints, 5.0);
        assert_eq!(bars.len(), 1);
        assert!((bars[0].volume - 60.0).abs() < 1e-9);
        assert_eq!(bars[0].tick_count, 3);
    }
}
