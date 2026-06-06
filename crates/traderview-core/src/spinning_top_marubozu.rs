//! Spinning Top + Marubozu — opposite-extreme single-bar patterns.
//!
//! Spinning Top: small body (≤ 30% of range) with substantial wicks
//! on BOTH sides (each ≥ 0.3 of range). Signals indecision.
//!
//! Marubozu: very large body (≥ 95% of range) with minimal wicks.
//! Two flavors:
//!   bullish_marubozu: body bullish, almost no wicks (full conviction)
//!   bearish_marubozu: body bearish, almost no wicks
//!
//! Pure compute. Companion to `candle_patterns`, `doji_variants`,
//! `hanging_man_shooting_star`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpinningTopMarubozuReport {
    pub spinning_top: Vec<bool>,
    pub bullish_marubozu: Vec<bool>,
    pub bearish_marubozu: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> SpinningTopMarubozuReport {
    let n = bars.len();
    let mut report = SpinningTopMarubozuReport {
        spinning_top: vec![false; n],
        bullish_marubozu: vec![false; n],
        bearish_marubozu: vec![false; n],
    };
    if bars.iter().any(|b| {
        !b.open.is_finite() || !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()
    }) {
        return report;
    }
    for (i, bar) in bars.iter().enumerate() {
        let range = bar.high - bar.low;
        if range <= 0.0 {
            continue;
        }
        let body = (bar.close - bar.open).abs();
        let upper = bar.high - bar.close.max(bar.open);
        let lower = bar.close.min(bar.open) - bar.low;
        // Spinning top: small body, both wicks substantial.
        if body <= 0.3 * range && upper >= 0.3 * range && lower >= 0.3 * range {
            report.spinning_top[i] = true;
        }
        // Marubozu: body ≥ 95% of range.
        if body >= 0.95 * range {
            if bar.close > bar.open {
                report.bullish_marubozu[i] = true;
            } else if bar.close < bar.open {
                report.bearish_marubozu[i] = true;
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar {
            open: o,
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[]);
        assert!(r.spinning_top.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            bar(100.0, 101.0, 99.0, 100.0),
            bar(f64::NAN, 101.0, 99.0, 100.0),
        ];
        let r = compute(&bars);
        assert!(!r.spinning_top.iter().any(|x| *x));
    }

    #[test]
    fn spinning_top_detected() {
        // Range 10, body 2 (close-open), upper wick 4, lower wick 4.
        let bars = vec![bar(100.0, 105.0, 95.0, 101.0)];
        let r = compute(&bars);
        assert!(r.spinning_top[0]);
    }

    #[test]
    fn bullish_marubozu_detected() {
        // Body = full range, no wicks. open=100, close=110, high=110, low=100.
        let bars = vec![bar(100.0, 110.0, 100.0, 110.0)];
        let r = compute(&bars);
        assert!(r.bullish_marubozu[0]);
        assert!(!r.bearish_marubozu[0]);
        assert!(!r.spinning_top[0]);
    }

    #[test]
    fn bearish_marubozu_detected() {
        let bars = vec![bar(110.0, 110.0, 100.0, 100.0)];
        let r = compute(&bars);
        assert!(r.bearish_marubozu[0]);
        assert!(!r.bullish_marubozu[0]);
    }

    #[test]
    fn marubozu_with_small_wicks_rejected() {
        // open=101, high=110, low=100, close=109.
        // body = 8 of range 10 = 80% < 95% threshold → not marubozu.
        let bars = vec![bar(101.0, 110.0, 100.0, 109.0)];
        let r = compute(&bars);
        assert!(!r.bullish_marubozu[0]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.spinning_top.len(), 10);
        assert_eq!(r.bullish_marubozu.len(), 10);
        assert_eq!(r.bearish_marubozu.len(), 10);
    }
}
