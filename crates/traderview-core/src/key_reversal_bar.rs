//! Key Reversal Bar Detector — classic single-bar trend-reversal signal.
//!
//! **Bullish key reversal (after downtrend)**:
//!   - Bar opens below prior close (gap down or lower open)
//!   - Bar trades below the prior low at some point
//!   - Closes above the prior high (full daily reversal)
//!
//! **Bearish key reversal (after uptrend)**:
//!   - Bar opens above prior close
//!   - Bar trades above the prior high
//!   - Closes below the prior low
//!
//! Heuristic context filter: requires `prior_trend_lookback` bars of
//! a clear preceding trend (consecutive higher highs or lower lows).
//!
//! Pure compute. Companion to `candle_patterns`, `island_reversal`,
//! `three_bar_reversal`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReversalKind { BullishKeyReversal, BearishKeyReversal }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyReversalEvent {
    pub kind: ReversalKind,
    pub bar_index: usize,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub prior_close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub prior_trend_lookback: usize,
}

impl Default for Config {
    fn default() -> Self { Self { prior_trend_lookback: 3 } }
}

pub fn detect(bars: &[Bar], cfg: &Config) -> Vec<KeyReversalEvent> {
    let n = bars.len();
    let mut out = Vec::new();
    if n < cfg.prior_trend_lookback + 2 || cfg.prior_trend_lookback < 1 { return out; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) { return out; }
    for i in (cfg.prior_trend_lookback + 1)..n {
        let prev = bars[i - 1];
        let cur = bars[i];
        // Check for prior downtrend (lower lows for cfg.prior_trend_lookback bars).
        let downtrend = (1..=cfg.prior_trend_lookback).all(|k| {
            bars[i - k].low < bars[i - k - 1].low
        });
        let uptrend = (1..=cfg.prior_trend_lookback).all(|k| {
            bars[i - k].high > bars[i - k - 1].high
        });
        if downtrend
            && cur.open < prev.close
            && cur.low < prev.low
            && cur.close > prev.high {
            out.push(KeyReversalEvent {
                kind: ReversalKind::BullishKeyReversal,
                bar_index: i,
                open: cur.open, high: cur.high, low: cur.low, close: cur.close,
                prior_close: prev.close,
            });
        } else if uptrend
            && cur.open > prev.close
            && cur.high > prev.high
            && cur.close < prev.low {
            out.push(KeyReversalEvent {
                kind: ReversalKind::BearishKeyReversal,
                bar_index: i,
                open: cur.open, high: cur.high, low: cur.low, close: cur.close,
                prior_close: prev.close,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> Bar { Bar { open: o, high: h, low: l, close: c } }

    #[test]
    fn empty_returns_empty() {
        assert!(detect(&[], &Config::default()).is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.0); 10];
        let cfg = Config { prior_trend_lookback: 0 };
        assert!(detect(&bars, &cfg).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(100.0, 101.0, 99.0, 100.0); 10];
        bars[3] = b(f64::NAN, 101.0, 99.0, 100.0);
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn flat_market_no_reversal() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.0); 20];
        assert!(detect(&bars, &Config::default()).is_empty());
    }

    #[test]
    fn bullish_key_reversal_detected() {
        // Three bars of lower lows: closes 100, 98, 96.
        // Bar 3: open 94 (below prev close 96), low 90 (below prev low 95),
        // close 99 (above prev high 97). Bullish key reversal.
        let bars = vec![
            b(100.0, 101.0, 99.0, 100.0),
            b(99.0, 100.0, 97.0, 98.0),
            b(97.0, 98.0, 95.0, 96.0),
            b(94.0, 99.5, 90.0, 99.0),
        ];
        let cfg = Config { prior_trend_lookback: 2 };
        let events = detect(&bars, &cfg);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, ReversalKind::BullishKeyReversal);
        assert_eq!(events[0].bar_index, 3);
    }

    #[test]
    fn bearish_key_reversal_detected() {
        // Three bars of higher highs: 100, 102, 104.
        // Bar 3: open 106, high 110 (> prev high 105), close 99 (< prev low 101).
        let bars = vec![
            b(98.0, 100.0, 97.0, 99.0),
            b(99.0, 102.0, 99.0, 101.0),
            b(101.0, 105.0, 101.0, 103.0),
            b(106.0, 110.0, 99.0, 100.0),
        ];
        let cfg = Config { prior_trend_lookback: 2 };
        let events = detect(&bars, &cfg);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, ReversalKind::BearishKeyReversal);
    }

    #[test]
    fn no_prior_trend_no_event() {
        // Mixed bars with no clear trend → no reversal even with extreme bar.
        let bars = vec![
            b(100.0, 101.0, 99.0, 100.0),
            b(99.0, 102.0, 98.0, 101.0),
            b(101.0, 100.0, 97.0, 98.0),    // mixed
            b(94.0, 99.0, 90.0, 99.0),
        ];
        let cfg = Config { prior_trend_lookback: 2 };
        assert!(detect(&bars, &cfg).is_empty());
    }
}
