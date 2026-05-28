//! Harami Pattern — 2-bar inside reversal (Japanese for "pregnant").
//!
//! Prior bar has a tall body; current bar's body is entirely INSIDE
//! the prior body and has the OPPOSITE color:
//!
//! Bullish Harami: prior bearish, current bullish inside.
//! Bearish Harami: prior bullish, current bearish inside.
//!
//! Indicates indecision after a strong directional bar — often
//! precedes a reversal when it appears at a swing extreme.
//!
//! Pure compute. Companion to `candle_patterns`,
//! `engulfing_pattern_scanner`, `morning_evening_star`,
//! `dark_cloud_piercing`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HaramiReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> HaramiReport {
    let n = bars.len();
    let mut report = HaramiReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
    };
    if n < 2 { return report; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    for i in 1..n {
        let prev = bars[i - 1];
        let cur = bars[i];
        let prev_body = (prev.close - prev.open).abs();
        let cur_body = (cur.close - cur.open).abs();
        let prev_range = prev.high - prev.low;
        if prev_body <= 0.0 || prev_range <= 0.0 { continue; }
        // Prior bar must be tall (body ≥ 60% of range) and current
        // bar's body must fit inside it.
        if prev_body < 0.6 * prev_range { continue; }
        let prev_body_high = prev.open.max(prev.close);
        let prev_body_low = prev.open.min(prev.close);
        let cur_body_high = cur.open.max(cur.close);
        let cur_body_low = cur.open.min(cur.close);
        if cur_body_high > prev_body_high || cur_body_low < prev_body_low { continue; }
        if cur_body <= 0.0 { continue; }
        // Opposite color.
        let prev_bullish = prev.close > prev.open;
        let cur_bullish = cur.close > cur.open;
        if prev_bullish == cur_bullish { continue; }
        if cur_bullish {
            report.bullish[i] = true;
        } else {
            report.bearish[i] = true;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar { open: o, high: h, low: l, close: c }
    }

    #[test]
    fn empty_or_single_bar_returns_empty() {
        let r = compute(&[]);
        assert!(r.bullish.is_empty());
        let r2 = compute(&[bar(100.0, 101.0, 99.0, 100.0)]);
        assert!(!r2.bullish.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0),
                        bar(f64::NAN, 101.0, 99.0, 100.0)];
        let r = compute(&bars);
        assert!(!r.bullish.iter().any(|x| *x));
        assert!(!r.bearish.iter().any(|x| *x));
    }

    #[test]
    fn classic_bullish_harami_detected() {
        // Prior tall bearish 110 → 100. Current bullish body (101..105)
        // INSIDE prior body (100..110).
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(101.0, 105.5, 100.5, 105.0),
        ];
        let r = compute(&bars);
        assert!(r.bullish[1]);
    }

    #[test]
    fn classic_bearish_harami_detected() {
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(108.0, 108.5, 105.0, 106.0),
        ];
        let r = compute(&bars);
        assert!(r.bearish[1]);
    }

    #[test]
    fn current_body_outside_prior_rejected() {
        // Engulfing-style: current body bigger than prior → not harami.
        let bars = vec![
            bar(108.0, 110.5, 105.0, 106.0),
            bar(105.0, 112.0, 100.0, 111.0),
        ];
        let r = compute(&bars);
        assert!(!r.bullish[1]);
    }

    #[test]
    fn same_color_bars_rejected() {
        // Both bearish — not harami.
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(108.0, 108.5, 103.0, 104.0),
        ];
        let r = compute(&bars);
        assert!(!r.bullish[1]);
        assert!(!r.bearish[1]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.bullish.len(), 10);
        assert_eq!(r.bearish.len(), 10);
    }
}
