//! Pinball Setup — Linda Bradford Raschke (1998 swing-trading playbook).
//!
//! Counter-trend reversal pattern at extremes. After a strong move,
//! waits for a specific bar geometry signaling exhaustion:
//!
//! Bullish pinball (long signal at extreme low):
//!   1. Down trend confirmed by close < SMA(close, sma_period)
//!   2. Bar i sets a new N-bar low (N = lookback)
//!   3. Bar i closes in the upper half of its own range
//!   4. Bar i's close > previous close (intrabar reversal)
//!
//! Bearish pinball (short signal at extreme high) is the mirror:
//!   1. Close > SMA
//!   2. New N-bar high
//!   3. Close in lower half of range
//!   4. Close < previous close
//!
//! Defaults: sma_period = 50, lookback = 20.
//! Pure compute. Companion to `holy_grail`, `key_reversal_bar`,
//! `inside_bar_breakout`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PinballReport {
    pub long_signal: Vec<bool>,
    pub short_signal: Vec<bool>,
    pub sma_period: usize,
    pub lookback: usize,
}

pub fn compute(bars: &[Bar], sma_period: usize, lookback: usize) -> PinballReport {
    let n = bars.len();
    let mut report = PinballReport {
        long_signal: vec![false; n],
        short_signal: vec![false; n],
        sma_period,
        lookback,
    };
    if sma_period < 2 || lookback < 2 || n < sma_period.max(lookback) + 1 {
        return report;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return report;
    }
    let s_f = sma_period as f64;
    let mut sum: f64 = bars[..sma_period].iter().map(|b| b.close).sum();
    let mut sma = vec![None; n];
    sma[sma_period - 1] = Some(sum / s_f);
    for i in sma_period..n {
        sum += bars[i].close - bars[i - sma_period].close;
        sma[i] = Some(sum / s_f);
    }
    for i in lookback..n {
        let Some(m) = sma[i] else { continue };
        let close = bars[i].close;
        let prev_close = bars[i - 1].close;
        let range = bars[i].high - bars[i].low;
        if range <= 0.0 {
            continue;
        }
        let mid = bars[i].low + range / 2.0;
        // Look at the prior `lookback` bars (excluding current) for new
        // extreme.
        let win = &bars[i - lookback..i];
        let prev_low = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        let prev_high = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        if close < m && bars[i].low < prev_low && close > mid && close > prev_close {
            report.long_signal[i] = true;
        }
        if close > m && bars[i].high > prev_high && close < mid && close < prev_close {
            report.short_signal[i] = true;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 80];
        let r = compute(&bars, 1, 20);
        assert!(!r.long_signal.iter().any(|x| *x));
        let r2 = compute(&bars[..10], 50, 20);
        assert!(!r2.long_signal.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_no_signals() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 80];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 50, 20);
        assert!(!r.long_signal.iter().any(|x| *x));
        assert!(!r.short_signal.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_yields_no_signals() {
        let bars = vec![b(101.0, 99.0, 100.0); 80];
        let r = compute(&bars, 50, 20);
        assert!(!r.long_signal.iter().any(|x| *x));
        assert!(!r.short_signal.iter().any(|x| *x));
    }

    #[test]
    fn classic_bullish_pinball_triggers_long() {
        // 60 bars in mild downtrend, then a new low bar that closes in
        // the upper half of its range above prior close.
        let mut bars: Vec<Bar> = (0..60)
            .map(|i| {
                let m = 100.0 - i as f64 * 0.2;
                b(m + 0.5, m - 0.5, m - 0.3)
            })
            .collect();
        // Capitulation bar: new low, but closes near high.
        bars.push(b(90.0, 80.0, 89.0));
        let r = compute(&bars, 50, 20);
        let last = bars.len() - 1;
        assert!(r.long_signal[last]);
    }

    #[test]
    fn classic_bearish_pinball_triggers_short() {
        // Mild uptrend then a blow-off bar: new high, close in lower
        // half of range, close < prior close (the intrabar reversal).
        let mut bars: Vec<Bar> = (0..60)
            .map(|i| {
                let m = 100.0 + i as f64 * 0.2;
                b(m + 0.5, m - 0.5, m + 0.3)
            })
            .collect();
        // Prior close at bar 59 = 100 + 59·0.2 + 0.3 = 112.1.
        // Blow-off: spike to 125, but close at 109 (below prior 112.1
        // and in lower half of [108..125]).
        bars.push(b(125.0, 108.0, 109.0));
        let r = compute(&bars, 50, 20);
        let last = bars.len() - 1;
        assert!(r.short_signal[last]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 80];
        let r = compute(&bars, 50, 20);
        assert_eq!(r.long_signal.len(), 80);
        assert_eq!(r.short_signal.len(), 80);
    }
}
