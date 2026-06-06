//! Order block detector — smart-money concepts (SMC) primitive.
//!
//! An order block is the last bearish candle before a sharp bullish move
//! (or the last bullish candle before a sharp bearish move). SMC theory
//! says institutional orders accumulate at those candles, so when price
//! returns to them they often act as strong support/resistance.
//!
//! Detection rules:
//!   - **Bullish order block**: a DOWN candle (close < open) immediately
//!     followed by an UP move of ≥ `expansion_atrs` × ATR within the
//!     next `expansion_window` bars. The down-candle's range becomes the
//!     order block zone.
//!   - **Bearish order block**: mirror — UP candle followed by sharp drop.
//!
//! Outputs the order-block zones in chronological order. Caller pairs
//! these with current price to find active S/R targets.
//!
//! Pure compute. Distinct from `fair_value_gap` (gap pattern) and
//! `candlestick_patterns` (single-bar shapes).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OhlcBar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBlockConfig {
    /// Number of bars after the candidate in which expansion must occur.
    pub expansion_window: usize,
    /// Required expansion size as a multiple of the candidate bar's range.
    /// The expansion move from the candidate's close must be at least
    /// `expansion_multiple × candidate_range`.
    pub expansion_multiple: f64,
}

impl Default for OrderBlockConfig {
    fn default() -> Self {
        Self {
            expansion_window: 3,
            expansion_multiple: 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    /// A down-candle before a sharp up-move. Acts as future support.
    Bullish,
    /// An up-candle before a sharp down-move. Acts as future resistance.
    Bearish,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OrderBlock {
    pub bar_index: usize,
    pub kind: BlockKind,
    /// Order-block zone boundaries (the candidate candle's high/low).
    pub zone_high: f64,
    pub zone_low: f64,
    /// Magnitude of the expansion that confirmed this order block, in
    /// price units. Bigger = stronger order block.
    pub expansion_magnitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderBlockReport {
    pub blocks: Vec<OrderBlock>,
}

pub fn detect(bars: &[OhlcBar], cfg: &OrderBlockConfig) -> OrderBlockReport {
    let n = bars.len();
    if n < 2 || cfg.expansion_window == 0 || cfg.expansion_multiple <= 0.0 {
        return OrderBlockReport::default();
    }
    let mut blocks = Vec::new();
    for i in 0..n.saturating_sub(1) {
        let cand = bars[i];
        let cand_range = cand.high - cand.low;
        if cand_range <= 0.0 {
            continue;
        }
        let need = cand_range * cfg.expansion_multiple;
        // saturating_add against a hostile JSON expansion_window of
        // usize::MAX which would otherwise wrap and produce `end < i+1`,
        // panicking the subsequent `bars[i+1..end]` slice.
        let end = i
            .saturating_add(1)
            .saturating_add(cfg.expansion_window)
            .min(n);
        // Bullish order block: candidate is a DOWN candle, expansion is UP.
        if cand.close < cand.open {
            let post_high = bars[i + 1..end]
                .iter()
                .map(|b| b.high)
                .fold(f64::NEG_INFINITY, f64::max);
            let expansion = post_high - cand.close;
            if expansion >= need {
                blocks.push(OrderBlock {
                    bar_index: i,
                    kind: BlockKind::Bullish,
                    zone_high: cand.high,
                    zone_low: cand.low,
                    expansion_magnitude: expansion,
                });
                continue;
            }
        }
        // Bearish: UP candle, expansion DOWN.
        if cand.close > cand.open {
            let post_low = bars[i + 1..end]
                .iter()
                .map(|b| b.low)
                .fold(f64::INFINITY, f64::min);
            let expansion = cand.close - post_low;
            if expansion >= need {
                blocks.push(OrderBlock {
                    bar_index: i,
                    kind: BlockKind::Bearish,
                    zone_high: cand.high,
                    zone_low: cand.low,
                    expansion_magnitude: expansion,
                });
            }
        }
    }
    OrderBlockReport { blocks }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> OhlcBar {
        OhlcBar {
            open: o,
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_or_one_bar_returns_no_blocks() {
        assert!(detect(&[], &OrderBlockConfig::default()).blocks.is_empty());
        assert!(detect(
            &[b(100.0, 101.0, 99.0, 100.5)],
            &OrderBlockConfig::default()
        )
        .blocks
        .is_empty());
    }

    #[test]
    fn bullish_order_block_detected_before_sharp_rally() {
        // Candidate: open=101 high=101 low=99 close=99 (down candle, range=2).
        // expansion_multiple = 2 → need 4 points up from close (99).
        // Next 3 bars: high goes 103, 104, 105 → post_high = 105.
        // Expansion = 105 - 99 = 6 ≥ 4 ✓.
        let bars = vec![
            b(101.0, 101.0, 99.0, 99.0),
            b(99.5, 103.0, 99.5, 102.0),
            b(102.0, 104.0, 101.0, 103.5),
            b(103.5, 105.0, 103.0, 104.5),
        ];
        let r = detect(&bars, &OrderBlockConfig::default());
        assert!(!r.blocks.is_empty());
        assert!(matches!(r.blocks[0].kind, BlockKind::Bullish));
        assert_eq!(r.blocks[0].bar_index, 0);
        assert!((r.blocks[0].zone_low - 99.0).abs() < 1e-9);
        assert!((r.blocks[0].zone_high - 101.0).abs() < 1e-9);
    }

    #[test]
    fn bearish_order_block_detected_before_sharp_drop() {
        // Candidate: up-candle close=101, range=2. Subsequent bars dump to 95.
        let bars = vec![
            b(99.0, 101.0, 99.0, 101.0),
            b(100.0, 100.5, 97.0, 98.0),
            b(98.0, 99.0, 95.0, 96.0),
        ];
        let r = detect(&bars, &OrderBlockConfig::default());
        assert!(!r.blocks.is_empty());
        assert!(matches!(r.blocks[0].kind, BlockKind::Bearish));
    }

    #[test]
    fn weak_post_move_doesnt_qualify() {
        // Candidate: down candle range=2 → need expansion ≥ 4.
        // Next bars only push price up by 2 (high 101).
        let bars = vec![
            b(101.0, 101.0, 99.0, 99.0),
            b(99.5, 101.0, 99.0, 100.5),
            b(100.0, 101.0, 100.0, 100.8),
        ];
        let r = detect(&bars, &OrderBlockConfig::default());
        assert!(
            r.blocks.is_empty(),
            "weak expansion should not qualify, got {} blocks",
            r.blocks.len()
        );
    }

    #[test]
    fn doji_candidates_skipped() {
        // Open == close → neither bullish nor bearish candle; never an order block.
        let bars = vec![
            b(100.0, 102.0, 98.0, 100.0), // doji
            b(100.0, 108.0, 100.0, 107.0),
            b(107.0, 110.0, 106.0, 109.0),
        ];
        let r = detect(&bars, &OrderBlockConfig::default());
        assert!(r.blocks.is_empty(), "doji shouldn't be an OB candidate");
    }

    #[test]
    fn zero_range_bars_skipped() {
        // Range = 0 → can't be an order block.
        let bars = vec![b(100.0, 100.0, 100.0, 100.0), b(100.0, 110.0, 100.0, 109.0)];
        let r = detect(&bars, &OrderBlockConfig::default());
        assert!(r.blocks.is_empty());
    }

    #[test]
    fn zero_window_config_returns_empty() {
        let bars = vec![b(101.0, 101.0, 99.0, 99.0); 5];
        let cfg = OrderBlockConfig {
            expansion_window: 0,
            expansion_multiple: 1.0,
        };
        let r = detect(&bars, &cfg);
        assert!(r.blocks.is_empty());
    }
}
