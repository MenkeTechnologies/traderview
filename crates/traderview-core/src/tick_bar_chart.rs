//! Tick Bar Aggregator — emits a new bar every N prints, ignoring time.
//!
//! Tick bars normalize the chart by event-count rather than calendar
//! time, so each bar represents the same number of trades. Useful for
//! liquid markets where one-minute calendar bars contain wildly
//! different trade counts (e.g. open vs midday).
//!
//! Pure compute. Companion to `range_bar_chart`, `volume_bar_chart`,
//! `dollar_bar_chart`, `equivolume_bars`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Print {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TickBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub tick_count: u32,
}

pub fn compute(prints: &[Print], ticks_per_bar: u32) -> Vec<TickBar> {
    let mut out = Vec::new();
    if prints.is_empty() || ticks_per_bar == 0 {
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
    let mut volume = 0.0_f64;
    let mut tick_count = 0_u32;
    for p in prints {
        if tick_count == 0 {
            open = p.price;
            high = p.price;
            low = p.price;
            volume = 0.0;
        }
        if p.price > high {
            high = p.price;
        }
        if p.price < low {
            low = p.price;
        }
        volume += p.size;
        tick_count += 1;
        if tick_count >= ticks_per_bar {
            out.push(TickBar {
                open,
                high,
                low,
                close: p.price,
                volume,
                tick_count,
            });
            tick_count = 0;
        }
    }
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
        assert!(compute(&[], 10).is_empty());
        let prints = vec![p(100.0, 10.0); 50];
        assert!(compute(&prints, 0).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let prints = vec![p(f64::NAN, 10.0)];
        assert!(compute(&prints, 10).is_empty());
    }

    #[test]
    fn emits_one_bar_per_n_ticks() {
        let prints: Vec<_> = (0..30).map(|i| p(100.0 + i as f64 * 0.1, 1.0)).collect();
        let bars = compute(&prints, 10);
        assert_eq!(bars.len(), 3);
        for bar in &bars {
            assert_eq!(bar.tick_count, 10);
            assert!((bar.volume - 10.0).abs() < 1e-9);
        }
    }

    #[test]
    fn open_close_match_first_last_prints() {
        let prints = vec![p(100.0, 1.0), p(102.0, 1.0), p(101.0, 1.0)];
        let bars = compute(&prints, 3);
        assert_eq!(bars.len(), 1);
        assert!((bars[0].open - 100.0).abs() < 1e-9);
        assert!((bars[0].close - 101.0).abs() < 1e-9);
    }

    #[test]
    fn high_low_tracked() {
        let prints = vec![p(100.0, 1.0), p(110.0, 1.0), p(95.0, 1.0), p(102.0, 1.0)];
        let bars = compute(&prints, 4);
        assert!((bars[0].high - 110.0).abs() < 1e-9);
        assert!((bars[0].low - 95.0).abs() < 1e-9);
    }

    #[test]
    fn trailing_partial_bar_dropped() {
        let prints: Vec<_> = (0..23).map(|i| p(100.0 + i as f64, 1.0)).collect();
        let bars = compute(&prints, 10);
        assert_eq!(bars.len(), 2);
    }
}
