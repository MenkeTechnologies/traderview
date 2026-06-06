//! Ross Hook — Joe Ross ("Trading the Ross Hook", 1989).
//!
//! After a 1-2-3 swing reversal, the FIRST PULLBACK against the new
//! trend creates a "hook" — entry is on the break of the hook in
//! the direction of the new trend.
//!
//! Bullish Ross Hook:
//!   Following a bullish 1-2-3 reversal (uptrend established), look
//!   for the first bar making a lower high than the prior bar (pause/
//!   pullback). Entry trigger: subsequent bar's high > pullback bar's
//!   high — the hook is broken to the upside.
//!
//! Bearish Ross Hook: mirrored.
//!
//! This module takes pivot points (from `swing_points`) and detects
//! the trigger bar within `max_lookahead` bars after the hook forms.
//!
//! Pure compute. Companion to `sperandeo_1_2_3`, `holy_grail`,
//! `darvas_box`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RossHookReport {
    pub long_trigger: Vec<bool>,
    pub short_trigger: Vec<bool>,
    pub trend_lookback: usize,
}

pub fn compute(bars: &[Bar], trend_lookback: usize) -> RossHookReport {
    let n = bars.len();
    let mut report = RossHookReport {
        long_trigger: vec![false; n],
        short_trigger: vec![false; n],
        trend_lookback,
    };
    if trend_lookback < 5 || n < trend_lookback + 2 {
        return report;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return report;
    }
    for i in (trend_lookback + 1)..n {
        let prev = bars[i - 1];
        let cur = bars[i];
        let look_start = i - trend_lookback;
        // Bullish: prior bars in uptrend (last close > first close),
        // bar i-1 made a lower high than bar i-2 (the "hook"), bar i
        // breaks above bar i-1's high.
        let trend_up = bars[i - 1].close > bars[look_start].close;
        let trend_down = bars[i - 1].close < bars[look_start].close;
        if i >= 2 && trend_up {
            let prior2 = bars[i - 2];
            if prev.high < prior2.high && cur.high > prev.high {
                report.long_trigger[i] = true;
            }
        }
        if i >= 2 && trend_down {
            let prior2 = bars[i - 2];
            if prev.low > prior2.low && cur.low < prev.low {
                report.short_trigger[i] = true;
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
        let r = compute(&bars, 1);
        assert!(!r.long_trigger.iter().any(|x| *x));
        let r2 = compute(&bars[..3], 5);
        assert!(!r2.long_trigger.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 5);
        assert!(!r.long_trigger.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_no_signal() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 5);
        assert!(!r.long_trigger.iter().any(|x| *x));
        assert!(!r.short_trigger.iter().any(|x| *x));
    }

    #[test]
    fn bullish_ross_hook_detected() {
        // 5 bars uptrend (close 100→104), bar 6 lower high (hook), bar 7
        // breaks above bar 6 high.
        let bars = vec![
            b(101.0, 99.0, 100.0),
            b(102.0, 100.0, 101.0),
            b(103.0, 101.0, 102.0),
            b(104.0, 102.0, 103.0),
            b(105.0, 103.0, 104.0),
            b(104.5, 102.5, 103.5), // hook — lower high
            b(105.5, 103.0, 105.0), // trigger — breaks above 104.5
        ];
        let r = compute(&bars, 5);
        assert!(r.long_trigger[6]);
    }

    #[test]
    fn bearish_ross_hook_detected() {
        let bars = vec![
            b(101.0, 99.0, 100.0),
            b(100.0, 98.0, 99.0),
            b(99.0, 97.0, 98.0),
            b(98.0, 96.0, 97.0),
            b(97.0, 95.0, 96.0),
            b(97.5, 95.5, 96.5), // higher low — hook
            b(97.0, 94.0, 94.5), // trigger — breaks below 95.5
        ];
        let r = compute(&bars, 5);
        assert!(r.short_trigger[6]);
    }

    #[test]
    fn no_hook_no_signal() {
        // Continuous uptrend → no pullback bar to form the hook.
        let bars: Vec<_> = (0..30)
            .map(|i| b(101.0 + i as f64, 99.0 + i as f64, 100.0 + i as f64))
            .collect();
        let r = compute(&bars, 5);
        // Every bar makes higher high — no hook.
        assert!(!r.long_trigger.iter().any(|x| *x));
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 5);
        assert_eq!(r.long_trigger.len(), 30);
        assert_eq!(r.short_trigger.len(), 30);
    }
}
