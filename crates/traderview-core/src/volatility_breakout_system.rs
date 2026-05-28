//! Volatility Breakout System — Larry Williams / Marty Schwartz school.
//!
//! Classical day-trading setup: take a long when today's price exceeds
//! yesterday's close by some fraction of the recent range, take a
//! short on the symmetric breakdown:
//!
//!   range_t   = SMA(high - low, period)        N-day average true range
//!   trigger_t = range_t · multiplier
//!   long_t   = high_t > close_{t-1} + trigger_t
//!   short_t  = low_t  < close_{t-1} - trigger_t
//!
//! Pure compute. Default period = 5, multiplier = 0.5.
//! Companion to `darvas_box`, `breakout_detector`,
//! `inside_bar_breakout`, `volatility_stop`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolatilityBreakoutReport {
    pub long_signal: Vec<bool>,
    pub short_signal: Vec<bool>,
    pub trigger: Vec<Option<f64>>,
    pub period: usize,
    pub multiplier: f64,
}

pub fn compute(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
) -> VolatilityBreakoutReport {
    let n = bars.len();
    let mut report = VolatilityBreakoutReport {
        long_signal: vec![false; n],
        short_signal: vec![false; n],
        trigger: vec![None; n],
        period,
        multiplier,
    };
    if period < 2 || !multiplier.is_finite() || multiplier <= 0.0
        || n < period + 1 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    let p_f = period as f64;
    let ranges: Vec<f64> = bars.iter().map(|b| b.high - b.low).collect();
    let mut sum: f64 = ranges[..period].iter().sum();
    for i in period..n {
        if i > period {
            sum += ranges[i - 1] - ranges[i - 1 - period];
        }
        let avg = sum / p_f;
        let trig = avg * multiplier;
        report.trigger[i] = Some(trig);
        let pc = bars[i - 1].close;
        if bars[i].high > pc + trig { report.long_signal[i] = true; }
        if bars[i].low < pc - trig { report.short_signal[i] = true; }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 1, 0.5);
        assert!(!r.long_signal.iter().any(|x| *x));
        let r2 = compute(&bars, 5, 0.0);
        assert!(!r2.long_signal.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 5, 0.5);
        assert!(!r.long_signal.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_yields_no_signals() {
        // Range = 2, trigger = 1. High = 101, prior_close + trigger = 100 + 1 = 101.
        // high > 101? Strictly > → no.
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 5, 0.5);
        assert!(!r.long_signal.iter().any(|x| *x));
        assert!(!r.short_signal.iter().any(|x| *x));
    }

    #[test]
    fn upside_breakout_triggers_long() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 10];
        // Bar 10: high=110 > 100 + 0.5·2 = 101.
        bars.push(b(110.0, 99.0, 109.0));
        let r = compute(&bars, 5, 0.5);
        assert!(r.long_signal[10]);
    }

    #[test]
    fn downside_breakdown_triggers_short() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 10];
        bars.push(b(101.0, 90.0, 91.0));
        let r = compute(&bars, 5, 0.5);
        assert!(r.short_signal[10]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 5, 0.5);
        assert_eq!(r.long_signal.len(), 30);
        assert_eq!(r.short_signal.len(), 30);
        assert_eq!(r.trigger.len(), 30);
    }
}
