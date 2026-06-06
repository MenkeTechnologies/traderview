//! Turtle Soup — Linda Bradford Raschke ("Street Smarts", 1996).
//!
//! False-breakout reversal pattern. After a new N-bar low (or high),
//! enter the OPPOSITE direction when price closes back inside the
//! prior range within `confirm_bars` (typically 1-2 bars):
//!
//! Bearish turtle soup (short):
//!   1. Bar i sets a new N-bar high (high_i > max(prior N bars highs))
//!   2. Within `confirm_bars` after, close_j < prior_high (the breakout
//!      level), confirming the trap
//!
//! Bullish turtle soup (long):
//!   1. New N-bar low
//!   2. Subsequent close > prior_low within `confirm_bars`
//!
//! Pure compute. Defaults: lookback = 20, confirm_bars = 2.
//! Companion to `pinball_setup`, `holy_grail`, `darvas_box`,
//! `key_reversal_bar`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TurtleSoupReport {
    pub long_signal: Vec<bool>,
    pub short_signal: Vec<bool>,
    pub lookback: usize,
    pub confirm_bars: usize,
}

pub fn compute(bars: &[Bar], lookback: usize, confirm_bars: usize) -> TurtleSoupReport {
    let n = bars.len();
    let mut report = TurtleSoupReport {
        long_signal: vec![false; n],
        short_signal: vec![false; n],
        lookback,
        confirm_bars,
    };
    if lookback < 2 || confirm_bars < 1 || n < lookback + confirm_bars + 1 {
        return report;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return report;
    }
    for i in lookback..(n - confirm_bars) {
        let win = &bars[i - lookback..i];
        let prior_high = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let prior_low = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        let bar = bars[i];
        // New high → check for close-back-below within confirm_bars.
        if bar.high > prior_high {
            for k in 1..=confirm_bars {
                if bars[i + k].close < prior_high {
                    report.short_signal[i + k] = true;
                    break;
                }
            }
        }
        // New low → check for close-back-above.
        if bar.low < prior_low {
            for k in 1..=confirm_bars {
                if bars[i + k].close > prior_low {
                    report.long_signal[i + k] = true;
                    break;
                }
            }
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
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 1, 2);
        assert!(!r.long_signal.iter().any(|x| *x));
        let r2 = compute(&bars[..10], 20, 2);
        assert!(!r2.long_signal.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 20, 2);
        assert!(!r.long_signal.iter().any(|x| *x));
        assert!(!r.short_signal.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_yields_no_signals() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 20, 2);
        assert!(!r.long_signal.iter().any(|x| *x));
        assert!(!r.short_signal.iter().any(|x| *x));
    }

    #[test]
    fn false_breakout_high_triggers_short() {
        // Loop covers i in lookback..(n - confirm_bars). For lookback=20
        // and confirm_bars=2, need n ≥ 23 so i can reach 20.
        let mut bars = vec![b(101.0, 99.0, 100.0); 20];
        bars.push(b(110.0, 95.0, 105.0)); // bar 20: new high
        bars.push(b(102.0, 95.0, 99.0)); // bar 21: confirms (close < 101)
        bars.push(b(99.0, 95.0, 96.0)); // bar 22: padding for loop bound
        let r = compute(&bars, 20, 2);
        assert!(
            r.short_signal[21],
            "false high breakout should trigger short signal at bar 21"
        );
    }

    #[test]
    fn false_breakout_low_triggers_long() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 20];
        bars.push(b(105.0, 90.0, 95.0));
        bars.push(b(105.0, 98.0, 101.0));
        bars.push(b(105.0, 99.0, 102.0));
        let r = compute(&bars, 20, 2);
        assert!(r.long_signal[21]);
    }

    #[test]
    fn true_breakout_does_not_signal() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 20];
        bars.push(b(110.0, 100.0, 108.0));
        bars.push(b(115.0, 105.0, 112.0));
        bars.push(b(118.0, 110.0, 115.0));
        let r = compute(&bars, 20, 2);
        // Both confirm bars close above prior_high 101 → no short signal.
        assert!(!r.short_signal.iter().any(|x| *x));
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 20, 2);
        assert_eq!(r.long_signal.len(), 30);
        assert_eq!(r.short_signal.len(), 30);
    }
}
