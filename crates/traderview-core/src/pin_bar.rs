//! Pin Bar — single-bar reversal candle (Martin Pring, popularized by
//! Nial Fuller).
//!
//! Geometry requirements:
//!   - small body (≤ 1/3 of range)
//!   - long primary wick (≥ 2× body OR ≥ 2/3 of range)
//!   - tiny opposite wick (≤ 1/3 of body)
//!
//! Bullish pin (long-tail / "hammer-ish"): primary wick is the lower wick.
//! Bearish pin (long-head / "shooting star-ish"): primary wick is upper.
//!
//! Distinct from `hanging_man_shooting_star` (which adds a trend filter
//! and uses different wick thresholds) and the simpler entries in
//! `candle_patterns`. Pin Bar is a stricter geometric rule with no
//! trend context.
//!
//! Pure compute. Companion to `candle_patterns`,
//! `hanging_man_shooting_star`, `pinball_setup`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PinBarReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
}

pub fn compute(bars: &[Bar]) -> PinBarReport {
    let n = bars.len();
    let mut report = PinBarReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
    };
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    for (i, bar) in bars.iter().enumerate() {
        let range = bar.high - bar.low;
        if range <= 0.0 { continue; }
        let body = (bar.close - bar.open).abs();
        let body_high = bar.close.max(bar.open);
        let body_low = bar.close.min(bar.open);
        let upper_wick = bar.high - body_high;
        let lower_wick = body_low - bar.low;
        // Small body (≤ 1/3 of range).
        if body > range / 3.0 { continue; }
        // Body must be non-zero for primary-wick comparisons.
        if body <= 0.0 { continue; }
        // Bullish pin: long lower wick + tiny upper wick.
        if (lower_wick >= 2.0 * body || lower_wick >= 2.0 * range / 3.0)
            && upper_wick <= body / 3.0 {
            report.bullish[i] = true;
        }
        // Bearish pin: long upper wick + tiny lower wick.
        if (upper_wick >= 2.0 * body || upper_wick >= 2.0 * range / 3.0)
            && lower_wick <= body / 3.0 {
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
    fn empty_returns_empty() {
        let r = compute(&[]);
        assert!(r.bullish.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0),
                        bar(f64::NAN, 101.0, 99.0, 100.0)];
        let r = compute(&bars);
        assert!(!r.bullish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_pin_bar_detected() {
        // Range 10, body 1 (close-open), lower wick 8, upper wick 1.
        // body/range = 0.1 ≤ 1/3 ✓
        // lower wick = 8 ≥ 2×body=2 ✓
        // upper wick = 1 ≤ body/3 = 0.33? 1 > 0.33, fails!
        // Adjust: range 10, body 0.3, upper 0.1, lower 9.6.
        let bars = vec![bar(99.7, 100.0, 90.0, 100.0)];
        let r = compute(&bars);
        // body = 0.3, range = 10, lower = 99.7-90 = 9.7, upper = 100-100 = 0.
        // body/range = 0.03 ≤ 1/3 ✓; lower 9.7 ≥ 2×0.3 ✓; upper 0 ≤ 0.1 ✓.
        assert!(r.bullish[0]);
        assert!(!r.bearish[0]);
    }

    #[test]
    fn bearish_pin_bar_detected() {
        let bars = vec![bar(100.0, 110.0, 99.7, 99.7)];
        // body = 0.3, range = 10.3, upper = 10, lower = 0.
        let r = compute(&bars);
        assert!(r.bearish[0]);
    }

    #[test]
    fn no_body_no_pin_bar() {
        // Pure doji: body = 0 → reject (need non-zero body).
        let bars = vec![bar(100.0, 105.0, 95.0, 100.0)];
        let r = compute(&bars);
        assert!(!r.bullish[0] && !r.bearish[0]);
    }

    #[test]
    fn equal_wicks_no_pin() {
        // Both wicks large → not a pin (could be spinning top).
        let bars = vec![bar(99.5, 105.0, 95.0, 100.5)];
        let r = compute(&bars);
        assert!(!r.bullish[0]);
        assert!(!r.bearish[0]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars);
        assert_eq!(r.bullish.len(), 10);
        assert_eq!(r.bearish.len(), 10);
    }
}
