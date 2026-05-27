//! Candlestick reversal/continuation pattern detector.
//!
//! Implements the canonical 8 patterns most traders actually use:
//!   - Bullish/Bearish engulfing
//!   - Hammer / Hanging Man
//!   - Shooting Star / Inverted Hammer
//!   - Doji (indecision)
//!   - Morning Star / Evening Star (3-bar)
//!
//! Per-bar pattern detection. Caller decides what to do with the
//! signals. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub open: f64, pub high: f64, pub low: f64, pub close: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pattern {
    BullishEngulfing,
    BearishEngulfing,
    Hammer,
    HangingMan,
    ShootingStar,
    InvertedHammer,
    Doji,
    MorningStar,
    EveningStar,
}

impl Bar {
    pub fn body(&self) -> f64 { (self.close - self.open).abs() }
    pub fn upper_shadow(&self) -> f64 { self.high - self.open.max(self.close) }
    pub fn lower_shadow(&self) -> f64 { self.open.min(self.close) - self.low }
    pub fn range(&self) -> f64 { self.high - self.low }
    pub fn is_bull(&self) -> bool { self.close > self.open }
    pub fn is_bear(&self) -> bool { self.close < self.open }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternHit {
    pub bar_index: usize,
    pub pattern: Pattern,
}

/// Detect patterns across a bar series. Returns one PatternHit per
/// (bar, pattern) match. A bar may register multiple patterns.
pub fn detect(bars: &[Bar]) -> Vec<PatternHit> {
    let mut out = Vec::new();
    if bars.is_empty() { return out; }
    // Doji: tiny body, normal range.
    for (i, b) in bars.iter().enumerate() {
        if b.range() > 0.0 && b.body() / b.range() < 0.1 {
            out.push(PatternHit { bar_index: i, pattern: Pattern::Doji });
        }
    }
    // Hammer/HangingMan/ShootingStar/InvertedHammer (single-bar shapes).
    for (i, b) in bars.iter().enumerate() {
        if b.range() <= 0.0 { continue; }
        let body = b.body();
        let upper = b.upper_shadow();
        let lower = b.lower_shadow();
        // Hammer: small body at top, long lower shadow, minimal upper.
        if lower > 2.0 * body && upper < lower / 4.0 {
            // Direction: Hammer in downtrend = bullish reversal.
            // We don't know the prior trend — caller filters. Mark as Hammer.
            out.push(PatternHit { bar_index: i, pattern: Pattern::Hammer });
        }
        // Shooting Star: small body at bottom, long upper shadow, minimal lower.
        if upper > 2.0 * body && lower < upper / 4.0 {
            out.push(PatternHit { bar_index: i, pattern: Pattern::ShootingStar });
        }
    }
    // Bullish/Bearish engulfing (2-bar).
    for i in 1..bars.len() {
        let prev = bars[i - 1];
        let cur = bars[i];
        // Bullish engulfing: prior bear, current bull body engulfs prior body.
        if prev.is_bear() && cur.is_bull()
            && cur.open <= prev.close && cur.close >= prev.open
        {
            out.push(PatternHit { bar_index: i, pattern: Pattern::BullishEngulfing });
        }
        // Bearish engulfing.
        if prev.is_bull() && cur.is_bear()
            && cur.open >= prev.close && cur.close <= prev.open
        {
            out.push(PatternHit { bar_index: i, pattern: Pattern::BearishEngulfing });
        }
    }
    // Morning Star / Evening Star (3-bar).
    for i in 2..bars.len() {
        let b1 = bars[i - 2];
        let b2 = bars[i - 1];
        let b3 = bars[i];
        // Morning Star: bear, small body, bull closing into b1's body.
        if b1.is_bear() && b2.body() < b1.body() * 0.3 && b3.is_bull()
            && b3.close > (b1.open + b1.close) / 2.0
        {
            out.push(PatternHit { bar_index: i, pattern: Pattern::MorningStar });
        }
        // Evening Star.
        if b1.is_bull() && b2.body() < b1.body() * 0.3 && b3.is_bear()
            && b3.close < (b1.open + b1.close) / 2.0
        {
            out.push(PatternHit { bar_index: i, pattern: Pattern::EveningStar });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar { open: o, high: h, low: l, close: c }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(detect(&[]).is_empty());
    }

    #[test]
    fn doji_detected_for_tiny_body() {
        // Open 100, close 100.05, range 100 → body/range 0.0005 < 0.1.
        let bars = vec![b(100.0, 100.5, 99.5, 100.05)];
        let hits = detect(&bars);
        assert!(hits.iter().any(|h| h.pattern == Pattern::Doji));
    }

    #[test]
    fn no_doji_when_body_is_large() {
        // Tall body.
        let bars = vec![b(100.0, 100.5, 99.5, 100.4)];
        let hits = detect(&bars);
        assert!(!hits.iter().any(|h| h.pattern == Pattern::Doji));
    }

    #[test]
    fn hammer_detected_long_lower_shadow_small_body() {
        // O=99, H=100, L=92, C=100. Body=1, lower_shadow=99-92=7, upper=0.
        let bars = vec![b(99.0, 100.0, 92.0, 100.0)];
        let hits = detect(&bars);
        assert!(hits.iter().any(|h| h.pattern == Pattern::Hammer));
    }

    #[test]
    fn shooting_star_detected_long_upper_shadow_small_body() {
        // O=100, H=108, L=99.5, C=100.5. Body=0.5, upper=7.5, lower=0.5.
        let bars = vec![b(100.0, 108.0, 99.5, 100.5)];
        let hits = detect(&bars);
        assert!(hits.iter().any(|h| h.pattern == Pattern::ShootingStar));
    }

    #[test]
    fn bullish_engulfing_detected() {
        // Prev bear (100 → 95). Current bull (94 → 101) engulfs.
        let bars = vec![
            b(100.0, 100.5, 94.5, 95.0),    // bear
            b(94.0, 101.5, 93.5, 101.0),    // engulfs
        ];
        let hits = detect(&bars);
        assert!(hits.iter().any(|h| h.pattern == Pattern::BullishEngulfing));
    }

    #[test]
    fn bearish_engulfing_detected() {
        let bars = vec![
            b(95.0, 100.0, 94.0, 100.0),    // bull
            b(101.0, 101.5, 93.0, 94.0),    // bear engulfs
        ];
        let hits = detect(&bars);
        assert!(hits.iter().any(|h| h.pattern == Pattern::BearishEngulfing));
    }

    #[test]
    fn no_engulfing_when_second_bar_smaller() {
        let bars = vec![
            b(100.0, 100.5, 90.0, 91.0),    // bear (10-pt body)
            b(91.5, 95.0, 90.5, 94.0),       // small bull (~2.5-pt) — doesn't engulf
        ];
        let hits = detect(&bars);
        assert!(!hits.iter().any(|h| h.pattern == Pattern::BullishEngulfing));
    }

    #[test]
    fn morning_star_detected() {
        let bars = vec![
            b(100.0, 100.0, 90.0, 92.0),    // big bear (-8)
            b(91.0, 92.0, 89.0, 91.5),       // small body (0.5)
            b(91.0, 100.0, 90.5, 99.0),      // bull closing into b1's body midpoint (96)
        ];
        let hits = detect(&bars);
        assert!(hits.iter().any(|h| h.pattern == Pattern::MorningStar));
    }

    #[test]
    fn evening_star_detected() {
        let bars = vec![
            b(92.0, 100.0, 91.0, 100.0),    // big bull (+8)
            b(101.0, 102.0, 100.5, 101.5),  // small bull (0.5)
            b(101.0, 102.0, 92.0, 93.0),     // bear closing into b1 midpoint (96)
        ];
        let hits = detect(&bars);
        assert!(hits.iter().any(|h| h.pattern == Pattern::EveningStar));
    }

    #[test]
    fn zero_range_bar_skipped() {
        // H == L → no hammer/shooting-star possible.
        let bars = vec![b(100.0, 100.0, 100.0, 100.0)];
        let hits = detect(&bars);
        assert!(!hits.iter().any(|h| matches!(h.pattern, Pattern::Hammer | Pattern::ShootingStar)));
    }
}
