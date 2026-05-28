//! Dollar Bar Aggregator — emits a new bar every time accumulated
//! notional (price × size) reaches `dollars_per_bar`.
//!
//! Marcos López de Prado ("Advances in Financial Machine Learning")
//! showed dollar bars are statistically superior to time/tick/volume
//! bars for ML modeling — they better approximate i.i.d. returns by
//! normalizing for both activity AND price level changes.
//!
//! Pure compute. Companion to `tick_bar_chart`, `volume_bar_chart`,
//! `range_bar_chart`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Print { pub price: f64, pub size: f64 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DollarBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub notional: f64,
    pub tick_count: u32,
}

pub fn compute(prints: &[Print], dollars_per_bar: f64) -> Vec<DollarBar> {
    let mut out = Vec::new();
    if prints.is_empty() || !dollars_per_bar.is_finite() || dollars_per_bar <= 0.0 {
        return out;
    }
    if prints.iter().any(|p| !p.price.is_finite() || !p.size.is_finite()
        || p.price <= 0.0 || p.size < 0.0) {
        return out;
    }
    let mut open = prints[0].price;
    let mut high = prints[0].price;
    let mut low = prints[0].price;
    let mut volume = 0.0_f64;
    let mut notional = 0.0_f64;
    let mut tick_count = 0_u32;
    for p in prints {
        if tick_count == 0 {
            open = p.price;
            high = p.price;
            low = p.price;
            volume = 0.0;
            notional = 0.0;
        }
        if p.price > high { high = p.price; }
        if p.price < low { low = p.price; }
        volume += p.size;
        notional += p.price * p.size;
        tick_count += 1;
        if notional >= dollars_per_bar {
            out.push(DollarBar {
                open, high, low,
                close: p.price,
                volume, notional, tick_count,
            });
            tick_count = 0;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(price: f64, size: f64) -> Print { Print { price, size } }

    #[test]
    fn empty_or_invalid_returns_empty() {
        assert!(compute(&[], 100000.0).is_empty());
        let prints = vec![p(100.0, 10.0); 50];
        assert!(compute(&prints, 0.0).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let prints = vec![p(f64::NAN, 10.0)];
        assert!(compute(&prints, 10000.0).is_empty());
    }

    #[test]
    fn emits_bar_when_notional_reaches_target() {
        // 10 prints, price ≈100, size 100 each → notional = 100×100 = 10000 each.
        // Target 50000 → one bar after ~5 prints.
        let prints: Vec<_> = (0..10).map(|i| p(100.0 + i as f64 * 0.1, 100.0)).collect();
        let bars = compute(&prints, 50000.0);
        assert!(!bars.is_empty());
        assert!(bars[0].notional >= 50000.0);
    }

    #[test]
    fn high_low_tracked() {
        let prints = vec![p(100.0, 100.0), p(110.0, 100.0), p(95.0, 100.0),
                          p(102.0, 100.0), p(98.0, 100.0), p(101.0, 100.0)];
        let bars = compute(&prints, 60000.0);
        assert_eq!(bars.len(), 1);
        assert!((bars[0].high - 110.0).abs() < 1e-9);
        assert!((bars[0].low - 95.0).abs() < 1e-9);
    }

    #[test]
    fn trailing_partial_bar_dropped() {
        let prints: Vec<_> = (0..3).map(|_| p(100.0, 100.0)).collect();
        let bars = compute(&prints, 50000.0);
        assert!(bars.is_empty());
    }

    #[test]
    fn notional_matches_sum_price_times_size() {
        let prints = vec![p(100.0, 200.0), p(110.0, 300.0)];
        let bars = compute(&prints, 50000.0);
        assert!(!bars.is_empty());
        // 100·200 + 110·300 = 20000 + 33000 = 53000.
        assert!((bars[0].notional - 53000.0).abs() < 1e-9);
    }
}
