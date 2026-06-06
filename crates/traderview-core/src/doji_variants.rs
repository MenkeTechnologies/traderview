//! Doji variants — long-legged, gravestone, and dragonfly.
//!
//! All variants share the doji condition (|close - open| ≤ doji_pct of
//! range, default 10%) but differ in wick geometry:
//!
//!   long_legged: large upper AND lower wicks (each ≥ 0.4 of range)
//!   gravestone : long upper wick (≥ 0.7 of range), tiny lower wick
//!                (≤ 0.1 of range), bearish reversal at tops
//!   dragonfly  : long lower wick (≥ 0.7 of range), tiny upper wick,
//!                bullish reversal at bottoms
//!
//! Pure compute. Companion to `candle_patterns` (basic doji),
//! `morning_evening_star`, `hanging_man_shooting_star`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DojiVariantsReport {
    pub long_legged: Vec<bool>,
    pub gravestone: Vec<bool>,
    pub dragonfly: Vec<bool>,
    pub doji_pct: f64,
}

pub fn compute(bars: &[Bar], doji_pct: f64) -> DojiVariantsReport {
    let n = bars.len();
    let mut report = DojiVariantsReport {
        long_legged: vec![false; n],
        gravestone: vec![false; n],
        dragonfly: vec![false; n],
        doji_pct,
    };
    if !doji_pct.is_finite() || doji_pct <= 0.0 || doji_pct >= 1.0 {
        return report;
    }
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
        let is_doji = body <= doji_pct * range;
        if !is_doji {
            continue;
        }
        let upper = bar.high - bar.close.max(bar.open);
        let lower = bar.close.min(bar.open) - bar.low;
        let upper_pct = upper / range;
        let lower_pct = lower / range;
        if upper_pct >= 0.4 && lower_pct >= 0.4 {
            report.long_legged[i] = true;
        }
        if upper_pct >= 0.7 && lower_pct <= 0.1 {
            report.gravestone[i] = true;
        }
        if lower_pct >= 0.7 && upper_pct <= 0.1 {
            report.dragonfly[i] = true;
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
        let r = compute(&[], 0.1);
        assert!(r.long_legged.is_empty());
    }

    #[test]
    fn nan_or_invalid_returns_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0); 5];
        let r = compute(&bars, 0.0);
        assert!(!r.long_legged.iter().any(|x| *x));
        let mut bars2 = vec![bar(100.0, 101.0, 99.0, 100.0); 5];
        bars2[2] = bar(f64::NAN, 101.0, 99.0, 100.0);
        let r2 = compute(&bars2, 0.1);
        assert!(!r2.long_legged.iter().any(|x| *x));
    }

    #[test]
    fn long_legged_doji_detected() {
        // open ≈ close (body ≈ 0), high - low = 10, upper and lower
        // wicks each ≈ 5.
        let bars = vec![bar(100.0, 105.0, 95.0, 100.0)];
        let r = compute(&bars, 0.1);
        assert!(r.long_legged[0]);
        assert!(!r.gravestone[0]);
        assert!(!r.dragonfly[0]);
    }

    #[test]
    fn gravestone_doji_detected() {
        // open/close near low; long upper wick.
        let bars = vec![bar(100.0, 110.0, 99.5, 100.0)];
        let r = compute(&bars, 0.1);
        assert!(r.gravestone[0]);
        assert!(!r.dragonfly[0]);
    }

    #[test]
    fn dragonfly_doji_detected() {
        // open/close near high; long lower wick.
        let bars = vec![bar(100.0, 100.5, 90.0, 100.0)];
        let r = compute(&bars, 0.1);
        assert!(r.dragonfly[0]);
        assert!(!r.gravestone[0]);
    }

    #[test]
    fn non_doji_bar_no_signal() {
        // Large body → not doji.
        let bars = vec![bar(100.0, 105.0, 95.0, 104.0)];
        let r = compute(&bars, 0.1);
        assert!(!r.long_legged[0]);
        assert!(!r.gravestone[0]);
        assert!(!r.dragonfly[0]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0); 10];
        let r = compute(&bars, 0.1);
        assert_eq!(r.long_legged.len(), 10);
        assert_eq!(r.gravestone.len(), 10);
        assert_eq!(r.dragonfly.len(), 10);
    }
}
