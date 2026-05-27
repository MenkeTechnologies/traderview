//! Volatility-based trailing stop (Chandelier exit + extensions).
//!
//! Three flavors:
//!   - **chandelier_long**: highest_high_N - multiplier × ATR_N
//!   - **chandelier_short**: lowest_low_N + multiplier × ATR_N
//!   - **vol_stop_close**: same as Chandelier but referenced to highest
//!     CLOSE, not highest HIGH — less sensitive to single-bar spikes.
//!
//! Pure compute. Caller pre-computes ATR series via `crate::stops`.

use crate::models::TradeSide;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StopConfig {
    pub lookback: usize,
    pub atr_multiplier: f64,
}

impl Default for StopConfig {
    fn default() -> Self { Self { lookback: 22, atr_multiplier: 3.0 } }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StopPoint {
    pub stop_price: f64,
    /// True when the stop has been hit on this bar.
    pub triggered: bool,
}

pub fn chandelier(bars: &[Bar], atr: &[f64], side: TradeSide, cfg: &StopConfig)
    -> Vec<StopPoint>
{
    let n = bars.len();
    let mut out = vec![StopPoint::default(); n];
    if n < cfg.lookback || cfg.lookback == 0 { return out; }
    for i in (cfg.lookback - 1)..n {
        let window = &bars[(i + 1 - cfg.lookback)..=i];
        let stop = match side {
            TradeSide::Long  => {
                let highest = window.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
                highest - cfg.atr_multiplier * atr[i]
            }
            TradeSide::Short => {
                let lowest = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
                lowest + cfg.atr_multiplier * atr[i]
            }
        };
        let triggered = match side {
            TradeSide::Long  => bars[i].low <= stop,
            TradeSide::Short => bars[i].high >= stop,
        };
        out[i] = StopPoint { stop_price: stop, triggered };
    }
    out
}

pub fn vol_stop_close(bars: &[Bar], atr: &[f64], side: TradeSide, cfg: &StopConfig)
    -> Vec<StopPoint>
{
    let n = bars.len();
    let mut out = vec![StopPoint::default(); n];
    if n < cfg.lookback || cfg.lookback == 0 { return out; }
    for i in (cfg.lookback - 1)..n {
        let window = &bars[(i + 1 - cfg.lookback)..=i];
        let stop = match side {
            TradeSide::Long  => {
                let highest_close = window.iter().map(|b| b.close).fold(f64::NEG_INFINITY, f64::max);
                highest_close - cfg.atr_multiplier * atr[i]
            }
            TradeSide::Short => {
                let lowest_close = window.iter().map(|b| b.close).fold(f64::INFINITY, f64::min);
                lowest_close + cfg.atr_multiplier * atr[i]
            }
        };
        let triggered = match side {
            TradeSide::Long  => bars[i].close <= stop,
            TradeSide::Short => bars[i].close >= stop,
        };
        out[i] = StopPoint { stop_price: stop, triggered };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn empty_returns_empty() {
        let out = chandelier(&[], &[], TradeSide::Long, &StopConfig::default());
        assert!(out.is_empty());
    }

    #[test]
    fn series_under_lookback_emits_zero_stops() {
        let bars = vec![b(10.0, 9.0, 9.5); 5];
        let atr = vec![0.5; 5];
        let out = chandelier(&bars, &atr, TradeSide::Long, &StopConfig::default());
        for p in &out {
            assert_eq!(p.stop_price, 0.0);
        }
    }

    #[test]
    fn long_chandelier_below_highest_high_by_atr_multiple() {
        // 5 bars, lookback 5, ATR=1, multiplier=2. Highest high = 105.
        // Stop = 105 - 2×1 = 103.
        let bars = vec![
            b(101.0, 99.0, 100.0),
            b(102.0, 100.0, 101.0),
            b(105.0, 102.0, 104.0),    // highest high
            b(103.0, 100.0, 101.0),
            b(104.0, 101.0, 102.0),
        ];
        let atr = vec![1.0; 5];
        let cfg = StopConfig { lookback: 5, atr_multiplier: 2.0 };
        let out = chandelier(&bars, &atr, TradeSide::Long, &cfg);
        assert_eq!(out[4].stop_price, 103.0);
    }

    #[test]
    fn short_chandelier_above_lowest_low_by_atr_multiple() {
        let bars = vec![
            b(100.0, 99.0, 99.5),
            b(99.0,  98.0, 98.5),
            b(98.0,  95.0, 96.0),    // lowest low
            b(99.0,  96.0, 97.0),
            b(100.0, 97.0, 98.0),
        ];
        let atr = vec![1.0; 5];
        let cfg = StopConfig { lookback: 5, atr_multiplier: 2.0 };
        let out = chandelier(&bars, &atr, TradeSide::Short, &cfg);
        // Stop = 95 + 2×1 = 97.
        assert_eq!(out[4].stop_price, 97.0);
    }

    #[test]
    fn long_stop_triggered_when_low_pierces_stop() {
        let bars = vec![
            b(105.0, 104.0, 105.0),
            b(105.0, 104.0, 105.0),
            b(105.0, 104.0, 105.0),
            b(105.0, 100.0, 102.0),    // wick down through stop
            b(105.0, 100.0, 102.0),
        ];
        let atr = vec![1.0; 5];
        let cfg = StopConfig { lookback: 5, atr_multiplier: 2.0 };
        let out = chandelier(&bars, &atr, TradeSide::Long, &cfg);
        // Highest high = 105, stop = 103. Bar 4 low = 100 ≤ 103 → triggered.
        assert!(out[4].triggered);
    }

    #[test]
    fn vol_stop_close_uses_closing_price_not_high() {
        // Same bars but use vol_stop_close — sensitivity is to closes.
        let bars = vec![
            b(110.0, 99.0, 100.0),    // wick up to 110 but closed at 100
            b(105.0, 99.0, 100.0),
            b(105.0, 99.0, 100.0),
            b(105.0, 99.0, 100.0),
            b(105.0, 99.0, 100.0),
        ];
        let atr = vec![1.0; 5];
        let cfg = StopConfig { lookback: 5, atr_multiplier: 2.0 };
        let chand = chandelier(&bars, &atr, TradeSide::Long, &cfg);
        let close = vol_stop_close(&bars, &atr, TradeSide::Long, &cfg);
        // Chandelier uses high (110) → stop = 108.
        assert_eq!(chand[4].stop_price, 108.0);
        // vol_stop_close uses highest close (100) → stop = 98.
        assert_eq!(close[4].stop_price, 98.0);
    }

    #[test]
    fn default_config_is_22_period_3x() {
        let cfg = StopConfig::default();
        assert_eq!(cfg.lookback, 22);
        assert_eq!(cfg.atr_multiplier, 3.0);
    }
}
