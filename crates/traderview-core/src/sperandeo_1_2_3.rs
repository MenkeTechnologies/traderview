//! Sperandeo 1-2-3 Reversal Pattern — Victor Sperandeo
//! ("Trader Vic", 1991).
//!
//! 3-point swing reversal that identifies trend changes:
//!
//! Bullish 1-2-3 (after a downtrend):
//!   point 1: swing low (anchor)
//!   point 2: swing high above 1 (counter-rally)
//!   point 3: swing low above 1 (higher low — first higher low after
//!     downtrend)
//!   Confirmation: close > point 2 high
//!
//! Bearish 1-2-3 (after an uptrend):
//!   point 1: swing high (anchor)
//!   point 2: swing low below 1 (counter-decline)
//!   point 3: swing high below 1 (lower high — first lower high after
//!     uptrend)
//!   Confirmation: close < point 2 low
//!
//! Detector takes alternating-polarity pivots (use `swing_points`) and
//! the subsequent bar series for confirmation. Returns the indices in
//! the bar series where each confirmed pattern triggers.
//!
//! Pure compute. Companion to `darvas_box`, `pinball_setup`, `holy_grail`.

use serde::{Deserialize, Serialize};

pub use crate::gartley_pattern::Pivot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SperandoeDirection { #[default] Bullish, Bearish }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SperandoMatch {
    pub direction: SperandoeDirection,
    pub p1: Pivot,
    pub p2: Pivot,
    pub p3: Pivot,
    /// Bar index where the breakout confirmation closed.
    pub confirmation_bar: usize,
}

pub fn detect(
    pivots: &[Pivot],
    closes: &[f64],
    max_confirmation_lookahead: usize,
) -> Vec<SperandoMatch> {
    let mut out = Vec::new();
    if pivots.len() < 3 || closes.is_empty()
        || max_confirmation_lookahead < 1 { return out; }
    if closes.iter().any(|x| !x.is_finite()) { return out; }
    for w in pivots.windows(3) {
        let (p1, p2, p3) = (w[0], w[1], w[2]);
        // Bullish 1-2-3: p1 low, p2 high, p3 low.
        if !p1.is_high && p2.is_high && !p3.is_high
            && p3.price > p1.price && p2.price > p1.price
        {
            let conf_end = (p3.index + max_confirmation_lookahead).min(closes.len() - 1);
            let start = p3.index + 1;
            for (k, &close) in closes.iter().enumerate().skip(start).take(conf_end + 1 - start) {
                if close > p2.price {
                    out.push(SperandoMatch {
                        direction: SperandoeDirection::Bullish,
                        p1, p2, p3,
                        confirmation_bar: k,
                    });
                    break;
                }
            }
        }
        // Bearish 1-2-3: p1 high, p2 low, p3 high.
        if p1.is_high && !p2.is_high && p3.is_high
            && p3.price < p1.price && p2.price < p1.price
        {
            let conf_end = (p3.index + max_confirmation_lookahead).min(closes.len() - 1);
            let start = p3.index + 1;
            for (k, &close) in closes.iter().enumerate().skip(start).take(conf_end + 1 - start) {
                if close < p2.price {
                    out.push(SperandoMatch {
                        direction: SperandoeDirection::Bearish,
                        p1, p2, p3,
                        confirmation_bar: k,
                    });
                    break;
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(idx: usize, price: f64, is_high: bool) -> Pivot {
        Pivot { index: idx, price, is_high }
    }

    #[test]
    fn empty_or_invalid_returns_empty() {
        assert!(detect(&[], &[100.0; 10], 5).is_empty());
        assert!(detect(&[p(0, 100.0, false)], &[100.0; 10], 5).is_empty());
        let pivots = vec![p(0, 100.0, false), p(10, 110.0, true), p(20, 105.0, false)];
        assert!(detect(&pivots, &[], 5).is_empty());
    }

    #[test]
    fn nan_closes_return_empty() {
        let pivots = vec![p(0, 100.0, false), p(10, 110.0, true), p(20, 105.0, false)];
        let closes = vec![f64::NAN; 30];
        assert!(detect(&pivots, &closes, 5).is_empty());
    }

    #[test]
    fn bullish_1_2_3_confirmed() {
        let pivots = vec![
            p(0, 100.0, false),    // low at 100
            p(10, 110.0, true),    // high at 110
            p(20, 105.0, false),   // higher low at 105
        ];
        let mut closes = vec![107.0_f64; 30];
        closes[25] = 112.0;        // close above p2 high (110) → confirm
        let matches = detect(&pivots, &closes, 10);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].direction, SperandoeDirection::Bullish);
        assert_eq!(matches[0].confirmation_bar, 25);
    }

    #[test]
    fn bearish_1_2_3_confirmed() {
        let pivots = vec![
            p(0, 110.0, true),
            p(10, 100.0, false),
            p(20, 105.0, true),
        ];
        let mut closes = vec![103.0_f64; 30];
        closes[25] = 98.0;
        let matches = detect(&pivots, &closes, 10);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].direction, SperandoeDirection::Bearish);
    }

    #[test]
    fn no_confirmation_within_window_no_match() {
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 110.0, true),
            p(20, 105.0, false),
        ];
        let closes = vec![107.0_f64; 30];
        let matches = detect(&pivots, &closes, 5);
        assert!(matches.is_empty());
    }

    #[test]
    fn lower_low_at_p3_rejects_bullish() {
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 110.0, true),
            p(20, 95.0, false),    // lower low → not bullish
        ];
        let closes = vec![112.0_f64; 30];
        let matches = detect(&pivots, &closes, 10);
        assert!(matches.is_empty());
    }
}
