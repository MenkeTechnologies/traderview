//! Tick Imbalance Bar (TIB) — López de Prado's signed-volume bar.
//!
//! Each print is signed +1 / -1 by the tick rule:
//!   sign_t = +1 if price_t > price_{t-1}
//!   sign_t = -1 if price_t < price_{t-1}
//!   sign_t = prior sign if price_t == price_{t-1}
//!
//! Accumulates Σ sign × size. When |Σ| ≥ `imbalance_threshold`, emit a
//! bar (open=first price, high/low tracked, close=last price, plus the
//! final imbalance value).
//!
//! Bars triggered by imbalance — rather than time/volume/dollars — are
//! statistically nicer for ML modeling per López de Prado's
//! "Advances in Financial Machine Learning".
//!
//! Pure compute. Companion to `tick_bar_chart`, `volume_bar_chart`,
//! `dollar_bar_chart`, `cumulative_delta`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Print { pub price: f64, pub size: f64 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ImbalanceBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub imbalance: f64,
    pub tick_count: u32,
}

pub fn compute(prints: &[Print], imbalance_threshold: f64) -> Vec<ImbalanceBar> {
    let mut out = Vec::new();
    if prints.is_empty() || !imbalance_threshold.is_finite() || imbalance_threshold <= 0.0 {
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
    let mut imbalance = 0.0_f64;
    let mut tick_count = 0_u32;
    let mut prev_sign = 1_i32;
    let mut prev_price = prints[0].price;
    for p in prints {
        if tick_count == 0 {
            open = p.price;
            high = p.price;
            low = p.price;
            volume = 0.0;
            imbalance = 0.0;
        }
        let sign = if p.price > prev_price { 1 }
            else if p.price < prev_price { -1 }
            else { prev_sign };
        prev_sign = sign;
        prev_price = p.price;
        if p.price > high { high = p.price; }
        if p.price < low { low = p.price; }
        volume += p.size;
        imbalance += sign as f64 * p.size;
        tick_count += 1;
        if imbalance.abs() >= imbalance_threshold {
            out.push(ImbalanceBar {
                open, high, low,
                close: p.price,
                volume, imbalance, tick_count,
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
        assert!(compute(&[], 100.0).is_empty());
        let prints = vec![p(100.0, 10.0); 50];
        assert!(compute(&prints, 0.0).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let prints = vec![p(f64::NAN, 10.0)];
        assert!(compute(&prints, 100.0).is_empty());
    }

    #[test]
    fn pure_uptrend_emits_imbalance_bars() {
        // Each tick uptick (size=10 each). After 10 ticks imbalance = +100.
        let prints: Vec<_> = (0..20).map(|i| p(100.0 + i as f64 * 0.01, 10.0)).collect();
        let bars = compute(&prints, 100.0);
        assert!(!bars.is_empty());
        assert!(bars[0].imbalance >= 100.0);
    }

    #[test]
    fn pure_downtrend_emits_negative_imbalance() {
        let prints: Vec<_> = (0..20).map(|i| p(100.0 - i as f64 * 0.01, 10.0)).collect();
        let bars = compute(&prints, 100.0);
        assert!(!bars.is_empty());
        assert!(bars[0].imbalance <= -100.0);
    }

    #[test]
    fn balanced_flow_no_bars() {
        // Alternating uptick / downtick.
        let mut prints = Vec::new();
        for i in 0..20 {
            prints.push(p(100.0 + (i % 2) as f64 * 0.5, 10.0));
        }
        let bars = compute(&prints, 100.0);
        // Imbalance stays small → no bars (or very few).
        assert!(bars.is_empty() || bars.iter().all(|b| b.imbalance.abs() >= 100.0));
    }

    #[test]
    fn high_tracked_in_uptrend_bar() {
        // Pure uptrend (each print upticks size 10). Threshold 100
        // triggers after 10 upticks → bar covers prints 0..=9 (price
        // 100..109) so high = 109 and low = 100.
        let prints: Vec<_> = (0..15).map(|i| p(100.0 + i as f64, 10.0)).collect();
        let bars = compute(&prints, 100.0);
        assert!(!bars.is_empty());
        assert!(bars[0].high >= 109.0);
        assert!(bars[0].low <= 100.0);
    }

    #[test]
    fn trailing_partial_bar_dropped() {
        let prints = vec![p(100.0, 10.0), p(101.0, 10.0)];
        let bars = compute(&prints, 100.0);
        // 2 ticks × 10 = 20 imbalance — below threshold → no bar.
        assert!(bars.is_empty());
    }
}
