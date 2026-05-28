//! Lee-Ready (1991) trade direction classifier.
//!
//! Classifies each trade as a **buy** (lift the offer) or **sell**
//! (hit the bid) using the quote-rule + tick-rule combination:
//!
//!   1. **Quote rule** — if trade > midpoint → buy; trade < midpoint → sell.
//!   2. **Tick rule** (when trade == midpoint or quotes missing) —
//!      compare with the *previous trade* price: uptick → buy,
//!      downtick → sell, zero-tick → carry prior direction.
//!
//! Returns per-trade `Direction` (Buy / Sell / Unknown when neither
//! rule can decide — e.g. very first trade with no quote and no prior).
//!
//! Pure compute. Inputs: trade prices, optional (best_bid, best_ask)
//! per trade, both must be finite to participate in the quote rule.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TradeWithQuote {
    pub price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction { Buy, Sell, Unknown }

pub fn classify(trades: &[TradeWithQuote]) -> Vec<Direction> {
    let n = trades.len();
    let mut out = vec![Direction::Unknown; n];
    let mut last_direction = Direction::Unknown;
    let mut last_price: Option<f64> = None;
    for (i, t) in trades.iter().enumerate() {
        if !t.price.is_finite() {
            // Unknown; don't update last_price/last_direction.
            continue;
        }
        let quote_decision = match (t.bid, t.ask) {
            (Some(b), Some(a)) if b.is_finite() && a.is_finite() && a >= b => {
                let mid = (a + b) / 2.0;
                if t.price > mid { Some(Direction::Buy) }
                else if t.price < mid { Some(Direction::Sell) }
                else { None }
            }
            _ => None,
        };
        let direction = if let Some(d) = quote_decision {
            d
        } else {
            // Tick rule.
            match last_price {
                None => Direction::Unknown,
                Some(prev) if prev.is_finite() => {
                    if t.price > prev { Direction::Buy }
                    else if t.price < prev { Direction::Sell }
                    else { last_direction }
                }
                Some(_) => Direction::Unknown,
            }
        };
        out[i] = direction;
        if direction != Direction::Unknown {
            last_direction = direction;
        }
        last_price = Some(t.price);
    }
    out
}

/// Convenience: summary counts.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct ClassificationSummary {
    pub buys: usize,
    pub sells: usize,
    pub unknown: usize,
    pub buy_volume: f64,
    pub sell_volume: f64,
}

pub fn summarize(
    trades: &[TradeWithQuote],
    volumes: &[f64],
    directions: &[Direction],
) -> ClassificationSummary {
    let mut s = ClassificationSummary::default();
    if trades.len() != directions.len() || volumes.len() != directions.len() {
        return s;
    }
    for (i, d) in directions.iter().enumerate() {
        let v = volumes[i];
        let v_safe = if v.is_finite() && v >= 0.0 { v } else { 0.0 };
        match d {
            Direction::Buy => { s.buys += 1; s.buy_volume += v_safe; }
            Direction::Sell => { s.sells += 1; s.sell_volume += v_safe; }
            Direction::Unknown => { s.unknown += 1; }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(p: f64, b: Option<f64>, a: Option<f64>) -> TradeWithQuote {
        TradeWithQuote { price: p, bid: b, ask: a }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(classify(&[]).is_empty());
    }

    #[test]
    fn quote_rule_buy_at_ask() {
        let trades = vec![t(100.10, Some(100.00), Some(100.10))];
        let out = classify(&trades);
        assert_eq!(out[0], Direction::Buy);
    }

    #[test]
    fn quote_rule_sell_at_bid() {
        let trades = vec![t(100.00, Some(100.00), Some(100.10))];
        let out = classify(&trades);
        assert_eq!(out[0], Direction::Sell);
    }

    #[test]
    fn tick_rule_used_when_quotes_missing() {
        let trades = vec![
            t(100.00, None, None),    // unknown — no prior
            t(100.05, None, None),    // uptick → buy
            t(100.05, None, None),    // zero-tick → carry buy
            t(100.00, None, None),    // downtick → sell
        ];
        let out = classify(&trades);
        assert_eq!(out[0], Direction::Unknown);
        assert_eq!(out[1], Direction::Buy);
        assert_eq!(out[2], Direction::Buy);
        assert_eq!(out[3], Direction::Sell);
    }

    #[test]
    fn midpoint_trade_falls_through_to_tick_rule() {
        // Trade at midpoint → quote rule None → tick rule decides.
        let trades = vec![
            t(100.00, Some(99.95), Some(100.05)),     // mid = 100.00 → tick: no prior → unknown
            t(100.05, Some(100.00), Some(100.10)),    // above mid → buy (quote rule)
            t(100.05, Some(100.00), Some(100.10)),    // mid = 100.05 → tick: uptick from 100.00 → wait, prev was 100.05 → zero-tick → carry buy
        ];
        let out = classify(&trades);
        // Trade 0: mid trade, no prior price → unknown
        assert_eq!(out[0], Direction::Unknown);
        // Trade 1: above mid → quote rule buy
        assert_eq!(out[1], Direction::Buy);
        // Trade 2: at mid → tick rule with prev=100.05 → zero-tick → carries buy
        assert_eq!(out[2], Direction::Buy);
    }

    #[test]
    fn nan_price_skipped() {
        let trades = vec![
            t(100.00, Some(99.95), Some(100.05)),
            t(f64::NAN, Some(99.95), Some(100.05)),
            t(100.05, Some(100.00), Some(100.10)),
        ];
        let out = classify(&trades);
        assert_eq!(out[1], Direction::Unknown);
    }

    #[test]
    fn crossed_quotes_skipped_for_quote_rule() {
        // bid > ask is malformed; quote rule should reject, tick rule decides.
        let trades = vec![
            t(100.00, None, None),
            t(100.05, Some(100.10), Some(100.00)),    // crossed: quote rule skips → tick: uptick → buy
        ];
        let out = classify(&trades);
        assert_eq!(out[1], Direction::Buy);
    }

    #[test]
    fn summarize_counts_correctly() {
        let trades = vec![
            t(100.10, Some(100.00), Some(100.10)),    // quote-buy at ask
            t(100.00, Some(100.00), Some(100.10)),    // quote-sell at bid (downtick from 100.10)
            t(100.05, Some(100.00), Some(100.10)),    // mid → tick: uptick from 100.00 → buy
        ];
        let dirs = classify(&trades);
        let vols = vec![100.0, 200.0, 300.0];
        let s = summarize(&trades, &vols, &dirs);
        // Two buys (trades 0 and 2), one sell (trade 1), no unknowns.
        assert_eq!(s.buys, 2);
        assert_eq!(s.sells, 1);
        assert_eq!(s.unknown, 0);
        assert_eq!(s.buy_volume, 100.0 + 300.0);
        assert_eq!(s.sell_volume, 200.0);
    }

    #[test]
    fn summarize_dimension_mismatch_returns_default() {
        let trades = vec![t(100.0, None, None)];
        let dirs = vec![Direction::Buy];
        let vols = vec![100.0, 200.0];
        let s = summarize(&trades, &vols, &dirs);
        assert_eq!(s.buys, 0);
    }
}
