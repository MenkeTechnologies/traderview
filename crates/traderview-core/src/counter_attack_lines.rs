//! Counter Attack Lines (Meeting Lines) — 2-bar reversal pattern where
//! bar 2 closes at approximately the same price as bar 1's close, but
//! in the opposite direction.
//!
//! Bullish Counter Attack:
//!   Bar 1: bearish bar
//!   Bar 2: bullish bar that opens BELOW bar 1's low (gap down) and
//!     closes near bar 1's close (within `tolerance_pct`)
//!
//! Bearish Counter Attack: mirrored — bar 2 opens above bar 1's high,
//! closes near bar 1's close.
//!
//! The matching closes show that bulls/bears "counter-attacked" the
//! gap, neutralizing it.
//!
//! Pure compute. Default tolerance_pct = 0.3 (0.3% of bar 1 close).
//! Companion to `dark_cloud_piercing`, `harami_pattern`,
//! `on_neck_in_neck`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CounterAttackReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
    pub tolerance_pct: f64,
}

pub fn compute(bars: &[Bar], tolerance_pct: f64) -> CounterAttackReport {
    let n = bars.len();
    let mut report = CounterAttackReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
        tolerance_pct,
    };
    if n < 2 || !tolerance_pct.is_finite() || tolerance_pct <= 0.0 { return report; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    let tol_factor = tolerance_pct / 100.0;
    for i in 1..n {
        let prev = bars[i - 1];
        let cur = bars[i];
        let tol = prev.close.abs() * tol_factor;
        // Bullish: bar 1 bearish, bar 2 bullish with gap down + matching close.
        if prev.close < prev.open
            && cur.close > cur.open
            && cur.open < prev.low
            && (cur.close - prev.close).abs() <= tol {
            report.bullish[i] = true;
        }
        // Bearish: bar 1 bullish, bar 2 bearish with gap up + matching close.
        if prev.close > prev.open
            && cur.close < cur.open
            && cur.open > prev.high
            && (cur.close - prev.close).abs() <= tol {
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
        let r = compute(&[], 0.3);
        assert!(r.bullish.is_empty());
        let r2 = compute(&[bar(100.0, 101.0, 99.0, 100.0)], 0.3);
        assert!(!r2.bullish.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0),
                        bar(f64::NAN, 101.0, 99.0, 100.0)];
        let r = compute(&bars, 0.3);
        assert!(!r.bullish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_counter_attack_detected() {
        // Bar 1: bearish 110→100, low=99.5.
        // Bar 2: bullish, opens 95 (< 99.5), closes 100.1 (within 0.3% of 100).
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(95.0, 101.0, 94.5, 100.1),
        ];
        let r = compute(&bars, 0.3);
        assert!(r.bullish[1]);
    }

    #[test]
    fn bearish_counter_attack_detected() {
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(115.0, 115.5, 109.5, 109.9),
        ];
        let r = compute(&bars, 0.3);
        assert!(r.bearish[1]);
    }

    #[test]
    fn close_too_far_from_match_rejects() {
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(95.0, 105.0, 94.5, 103.0),    // close 103, off by 3%
        ];
        let r = compute(&bars, 0.3);
        assert!(!r.bullish[1]);
    }

    #[test]
    fn no_gap_rejects() {
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(100.5, 101.0, 99.5, 100.1),    // opens within bar 1 range
        ];
        let r = compute(&bars, 0.3);
        assert!(!r.bullish[1]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 0.3);
        assert_eq!(r.bullish.len(), 10);
        assert_eq!(r.bearish.len(), 10);
    }
}
