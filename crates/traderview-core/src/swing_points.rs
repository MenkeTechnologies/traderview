//! Swing high / swing low detector.
//!
//! Identifies pivots in a price series using the standard N-bar
//! lookback/lookforward rule: a bar is a swing high if its high
//! exceeds the highs of the N bars before AND after; a swing low
//! is symmetric on lows. N is typically 3, 5, or 10.
//!
//! Used as input to Fibonacci, pivot lines, support/resistance.
//!
//! Pure compute. Returns indices into the input series so the caller
//! can correlate with timestamps.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwingKind { High, Low }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwingPoint {
    pub index: usize,
    pub price: f64,
    pub kind: SwingKind,
}

pub fn detect(bars: &[Bar], lookback: usize) -> Vec<SwingPoint> {
    let mut out = Vec::new();
    if bars.len() < 2 * lookback + 1 || lookback == 0 { return out; }
    for i in lookback..(bars.len() - lookback) {
        let center = bars[i];
        let mut is_high = true;
        let mut is_low = true;
        for bar in &bars[(i - lookback)..i] {
            if bar.high >= center.high { is_high = false; }
            if bar.low <= center.low   { is_low = false; }
        }
        for bar in &bars[(i + 1)..=(i + lookback)] {
            if bar.high >= center.high { is_high = false; }
            if bar.low <= center.low   { is_low = false; }
        }
        if is_high {
            out.push(SwingPoint { index: i, price: center.high, kind: SwingKind::High });
        }
        if is_low {
            out.push(SwingPoint { index: i, price: center.low, kind: SwingKind::Low });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar { Bar { high: h, low: l } }

    #[test]
    fn empty_series_returns_empty() {
        assert!(detect(&[], 3).is_empty());
    }

    #[test]
    fn series_shorter_than_window_returns_empty() {
        // Need at least 2N+1 bars; with N=3 that's 7. Provide 5.
        let bars = vec![b(10.0, 8.0); 5];
        assert!(detect(&bars, 3).is_empty());
    }

    #[test]
    fn zero_lookback_returns_empty() {
        let bars = vec![b(10.0, 8.0); 10];
        assert!(detect(&bars, 0).is_empty());
    }

    #[test]
    fn classic_swing_high_in_middle() {
        // ^ at index 3. 7 bars, lookback=3.
        let bars = vec![
            b(1.0, 0.5),
            b(2.0, 1.5),
            b(3.0, 2.5),
            b(5.0, 4.0),   // swing high
            b(3.0, 2.5),
            b(2.0, 1.5),
            b(1.0, 0.5),
        ];
        let swings = detect(&bars, 3);
        assert_eq!(swings.len(), 1);
        assert_eq!(swings[0].index, 3);
        assert_eq!(swings[0].kind, SwingKind::High);
        assert_eq!(swings[0].price, 5.0);
    }

    #[test]
    fn classic_swing_low_in_middle() {
        let bars = vec![
            b(5.0, 4.0),
            b(4.0, 3.0),
            b(3.0, 2.0),
            b(2.0, 1.0),   // swing low
            b(3.0, 2.0),
            b(4.0, 3.0),
            b(5.0, 4.0),
        ];
        let swings = detect(&bars, 3);
        assert_eq!(swings.len(), 1);
        assert_eq!(swings[0].kind, SwingKind::Low);
        assert_eq!(swings[0].price, 1.0);
    }

    #[test]
    fn equal_neighbor_disqualifies_swing() {
        // Center high tied with neighbor → NOT a swing high (strict > on both sides).
        let bars = vec![
            b(1.0, 0.5),
            b(2.0, 1.5),
            b(3.0, 2.5),
            b(5.0, 4.0),   // candidate
            b(5.0, 4.0),   // tied — disqualifies
            b(2.0, 1.5),
            b(1.0, 0.5),
        ];
        let swings = detect(&bars, 3);
        assert!(swings.is_empty(), "ties disqualify (strict >)");
    }

    #[test]
    fn multiple_swings_in_long_series() {
        // Two swing highs separated by valley.
        // Lookback = 2 → need at least 5 bars.
        let bars = vec![
            b(1.0, 0.5),
            b(2.0, 1.5),
            b(5.0, 4.0),   // swing high (idx 2)
            b(2.0, 1.5),
            b(1.0, 0.5),
            b(2.0, 1.5),
            b(6.0, 5.0),   // swing high (idx 6)
            b(3.0, 2.5),
            b(2.0, 1.5),
        ];
        let swings = detect(&bars, 2);
        let highs: Vec<_> = swings.iter().filter(|s| s.kind == SwingKind::High).collect();
        assert_eq!(highs.len(), 2);
        assert_eq!(highs[0].index, 2);
        assert_eq!(highs[1].index, 6);
    }

    #[test]
    fn flat_series_yields_no_swings() {
        let bars = vec![b(10.0, 8.0); 10];
        assert!(detect(&bars, 3).is_empty());
    }

    #[test]
    fn lookback_window_pinned_at_boundaries() {
        // First and last `lookback` bars never qualify (insufficient window).
        let bars = vec![
            b(100.0, 99.0),   // can't be swing — at boundary
            b(99.0, 98.0),
            b(98.0, 97.0),
            b(97.0, 96.0),
            b(98.0, 97.0),
            b(99.0, 98.0),
            b(100.0, 99.0),   // can't be swing — at boundary
        ];
        let swings = detect(&bars, 3);
        // idx 3 is candidate swing low (96).
        assert_eq!(swings.len(), 1);
        assert_eq!(swings[0].index, 3);
    }

    #[test]
    fn one_bar_can_be_both_swing_high_and_low() {
        // Theoretically possible only if the bar's range is wider than
        // neighbors on BOTH sides. Construct deliberately:
        let bars = vec![
            b(5.0, 4.0),
            b(5.5, 4.5),
            b(6.0, 3.0),    // wide bar — high above, low below all neighbors
            b(5.5, 4.5),
            b(5.0, 4.0),
        ];
        let swings = detect(&bars, 2);
        assert_eq!(swings.len(), 2, "wide bar registers as both swing high AND swing low");
    }
}
