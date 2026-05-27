//! RSI divergence detector.
//!
//! Detects bullish and bearish divergences between price and an RSI
//! series. Classic reversal signal:
//!
//!   - **Bullish divergence**: price makes a LOWER low; RSI makes a
//!     HIGHER low → exhaustion of selling pressure.
//!   - **Bearish divergence**: price makes a HIGHER high; RSI makes a
//!     LOWER high → exhaustion of buying pressure.
//!
//! Operates on confirmed swing points (caller supplies) to avoid false
//! signals from in-progress moves.
//!
//! Pure compute. Reuses crate::swing_points for input candidates.

use crate::swing_points::SwingPoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRsiPoint {
    pub bar_index: usize,
    pub price: f64,
    pub rsi: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DivergenceKind { Bullish, Bearish }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergence {
    pub kind: DivergenceKind,
    pub from_bar: usize,
    pub to_bar: usize,
    pub from_price: f64,
    pub to_price: f64,
    pub from_rsi: f64,
    pub to_rsi: f64,
}

/// Detects divergences between consecutive swing-low pairs (bullish)
/// and consecutive swing-high pairs (bearish). `swings` must be in
/// chronological order; `series` lookup is by bar_index.
pub fn detect(swings: &[SwingPoint], series: &[PriceRsiPoint]) -> Vec<Divergence> {
    let mut out = Vec::new();
    if swings.len() < 2 { return out; }
    // Lookup by bar_index.
    let lookup: std::collections::HashMap<usize, &PriceRsiPoint> =
        series.iter().map(|p| (p.bar_index, p)).collect();

    // Walk swing lows for bullish divergence.
    let lows: Vec<&SwingPoint> = swings.iter()
        .filter(|s| matches!(s.kind, crate::swing_points::SwingKind::Low))
        .collect();
    for w in lows.windows(2) {
        let (from, to) = (w[0], w[1]);
        if let (Some(p_from), Some(p_to)) = (lookup.get(&from.index), lookup.get(&to.index)) {
            // Lower price low, higher RSI low → bullish.
            if p_to.price < p_from.price && p_to.rsi > p_from.rsi {
                out.push(Divergence {
                    kind: DivergenceKind::Bullish,
                    from_bar: from.index,
                    to_bar: to.index,
                    from_price: p_from.price,
                    to_price: p_to.price,
                    from_rsi: p_from.rsi,
                    to_rsi: p_to.rsi,
                });
            }
        }
    }

    let highs: Vec<&SwingPoint> = swings.iter()
        .filter(|s| matches!(s.kind, crate::swing_points::SwingKind::High))
        .collect();
    for w in highs.windows(2) {
        let (from, to) = (w[0], w[1]);
        if let (Some(p_from), Some(p_to)) = (lookup.get(&from.index), lookup.get(&to.index)) {
            // Higher price high, lower RSI high → bearish.
            if p_to.price > p_from.price && p_to.rsi < p_from.rsi {
                out.push(Divergence {
                    kind: DivergenceKind::Bearish,
                    from_bar: from.index,
                    to_bar: to.index,
                    from_price: p_from.price,
                    to_price: p_to.price,
                    from_rsi: p_from.rsi,
                    to_rsi: p_to.rsi,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swing_points::{SwingKind, SwingPoint};

    fn sl(index: usize) -> SwingPoint {
        SwingPoint { index, price: 0.0, kind: SwingKind::Low }
    }
    fn sh(index: usize) -> SwingPoint {
        SwingPoint { index, price: 0.0, kind: SwingKind::High }
    }
    fn p(bar_index: usize, price: f64, rsi: f64) -> PriceRsiPoint {
        PriceRsiPoint { bar_index, price, rsi }
    }

    #[test]
    fn empty_swings_returns_empty() {
        assert!(detect(&[], &[]).is_empty());
    }

    #[test]
    fn single_swing_returns_empty() {
        assert!(detect(&[sl(5)], &[p(5, 100.0, 30.0)]).is_empty());
    }

    #[test]
    fn classic_bullish_divergence_detected() {
        // Two swing lows. First @ idx 10 (price 100, rsi 30). Second @ idx 20
        // (price 95 lower, rsi 35 higher) → bullish.
        let swings = vec![sl(10), sl(20)];
        let series = vec![
            p(10, 100.0, 30.0),
            p(20,  95.0, 35.0),
        ];
        let out = detect(&swings, &series);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, DivergenceKind::Bullish);
    }

    #[test]
    fn classic_bearish_divergence_detected() {
        let swings = vec![sh(10), sh(20)];
        let series = vec![
            p(10, 100.0, 70.0),
            p(20, 105.0, 65.0),    // higher high, lower RSI
        ];
        let out = detect(&swings, &series);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, DivergenceKind::Bearish);
    }

    #[test]
    fn no_divergence_when_both_lower() {
        // Price lower, RSI ALSO lower → trend continuation, no divergence.
        let swings = vec![sl(10), sl(20)];
        let series = vec![
            p(10, 100.0, 30.0),
            p(20,  95.0, 25.0),
        ];
        assert!(detect(&swings, &series).is_empty());
    }

    #[test]
    fn no_divergence_when_both_higher() {
        // Price higher, RSI higher → trend continuation, no divergence.
        let swings = vec![sh(10), sh(20)];
        let series = vec![
            p(10, 100.0, 70.0),
            p(20, 105.0, 75.0),
        ];
        assert!(detect(&swings, &series).is_empty());
    }

    #[test]
    fn missing_lookup_point_skipped_silently() {
        // Swing references bar 30 but series doesn't contain it.
        let swings = vec![sl(10), sl(30)];
        let series = vec![p(10, 100.0, 30.0)];
        assert!(detect(&swings, &series).is_empty());
    }

    #[test]
    fn multiple_swing_lows_emit_per_pair_divergence() {
        // Three lows, each forming a bullish divergence with the prior.
        let swings = vec![sl(10), sl(20), sl(30)];
        let series = vec![
            p(10, 100.0, 30.0),
            p(20,  95.0, 35.0),    // bull div with 10
            p(30,  90.0, 40.0),    // bull div with 20
        ];
        let out = detect(&swings, &series);
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|d| d.kind == DivergenceKind::Bullish));
    }

    #[test]
    fn swing_highs_and_lows_independently_processed() {
        let swings = vec![sl(10), sh(15), sl(20), sh(25)];
        let series = vec![
            p(10, 100.0, 30.0),
            p(15, 110.0, 70.0),
            p(20,  95.0, 35.0),    // bullish vs sl(10)
            p(25, 115.0, 65.0),    // bearish vs sh(15)
        ];
        let out = detect(&swings, &series);
        assert_eq!(out.len(), 2);
        assert!(out.iter().any(|d| d.kind == DivergenceKind::Bullish));
        assert!(out.iter().any(|d| d.kind == DivergenceKind::Bearish));
    }
}
