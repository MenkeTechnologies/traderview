//! Volume Bar Aggregator — emits a new bar every time accumulated
//! volume reaches `volume_per_bar`, ignoring time.
//!
//! Volume bars normalize the chart by activity rather than calendar
//! time — each bar represents the same amount of traded volume.
//! Particularly useful for futures markets where intraday volume
//! varies dramatically (open, close, lunch lull).
//!
//! Pure compute. Companion to `tick_bar_chart`, `range_bar_chart`,
//! `dollar_bar_chart`, `equivolume_bars`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Print {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VolumeBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub tick_count: u32,
}

pub fn compute(prints: &[Print], volume_per_bar: f64) -> Vec<VolumeBar> {
    let mut out = Vec::new();
    if prints.is_empty() || !volume_per_bar.is_finite() || volume_per_bar <= 0.0 {
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
        if volume >= volume_per_bar {
            out.push(VolumeBar {
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
        assert!(compute(&[], 1000.0).is_empty());
        let prints = vec![p(100.0, 10.0); 50];
        assert!(compute(&prints, 0.0).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let prints = vec![p(f64::NAN, 10.0)];
        assert!(compute(&prints, 100.0).is_empty());
    }

    #[test]
    fn emits_bar_when_volume_reaches_target() {
        // 5 prints of 200 each = 1000 total, target 1000 → 1 bar.
        let prints: Vec<_> = (0..5).map(|i| p(100.0 + i as f64, 200.0)).collect();
        let bars = compute(&prints, 1000.0);
        assert_eq!(bars.len(), 1);
        assert!((bars[0].volume - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn multiple_bars_emitted() {
        let prints: Vec<_> = (0..50).map(|i| p(100.0 + i as f64 * 0.1, 100.0)).collect();
        let bars = compute(&prints, 500.0);
        assert!(bars.len() >= 5);
    }

    #[test]
    fn high_low_tracked() {
        let prints = vec![
            p(100.0, 200.0),
            p(110.0, 200.0),
            p(95.0, 200.0),
            p(102.0, 200.0),
            p(98.0, 200.0),
        ];
        let bars = compute(&prints, 1000.0);
        assert_eq!(bars.len(), 1);
        assert!((bars[0].high - 110.0).abs() < 1e-9);
        assert!((bars[0].low - 95.0).abs() < 1e-9);
    }

    #[test]
    fn trailing_partial_bar_dropped() {
        let prints: Vec<_> = (0..7).map(|i| p(100.0 + i as f64, 200.0)).collect();
        let bars = compute(&prints, 1000.0);
        assert_eq!(bars.len(), 1);
    }
}
