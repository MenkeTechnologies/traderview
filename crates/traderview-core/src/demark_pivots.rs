//! DeMark Pivots — Tom DeMark.
//!
//! Pivot system in which the central base depends on the relationship
//! between the prior session's close and open:
//!
//!   if close < open:   X = high + 2·low + close
//!   if close > open:   X = 2·high + low + close
//!   if close == open:  X = high + low + 2·close
//!
//!   pivot = X / 4
//!   R1    = X / 2 - low
//!   S1    = X / 2 - high
//!
//! DeMark only defines a single resistance and a single support level
//! (no R2/S2/R3/S3), reflecting his preference for tighter, more
//! conservative bands.
//!
//! Pure compute. Companion to `floor_pivots`, `camarilla_pivots`,
//! `woodie_pivots`, `fibonacci_pivots`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriorSession {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct DemarkPivotLevels {
    pub r1: f64,
    pub pivot: f64,
    pub s1: f64,
}

pub fn compute(session: PriorSession) -> Option<DemarkPivotLevels> {
    if !session.open.is_finite() || !session.high.is_finite()
        || !session.low.is_finite() || !session.close.is_finite()
        || session.high < session.low {
        return None;
    }
    let x = if session.close < session.open {
        session.high + 2.0 * session.low + session.close
    } else if session.close > session.open {
        2.0 * session.high + session.low + session.close
    } else {
        session.high + session.low + 2.0 * session.close
    };
    Some(DemarkPivotLevels {
        r1: x / 2.0 - session.low,
        pivot: x / 4.0,
        s1: x / 2.0 - session.high,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(o: f64, h: f64, l: f64, c: f64) -> PriorSession {
        PriorSession { open: o, high: h, low: l, close: c }
    }

    #[test]
    fn invalid_session_returns_none() {
        assert!(compute(s(100.0, f64::NAN, 99.0, 100.5)).is_none());
        assert!(compute(s(100.0, 99.0, 101.0, 100.0)).is_none());
    }

    #[test]
    fn bearish_close_uses_low_heavy_x() {
        // H=110, L=100, C=102, O=108 (bearish close < open)
        // X = 110 + 200 + 102 = 412 → pivot = 103
        // R1 = 206 - 100 = 106; S1 = 206 - 110 = 96
        let r = compute(s(108.0, 110.0, 100.0, 102.0)).unwrap();
        assert!((r.pivot - 103.0).abs() < 1e-9);
        assert!((r.r1 - 106.0).abs() < 1e-9);
        assert!((r.s1 - 96.0).abs() < 1e-9);
    }

    #[test]
    fn bullish_close_uses_high_heavy_x() {
        // H=110, L=100, C=108, O=102 (bullish close > open)
        // X = 220 + 100 + 108 = 428 → pivot = 107
        let r = compute(s(102.0, 110.0, 100.0, 108.0)).unwrap();
        assert!((r.pivot - 107.0).abs() < 1e-9);
    }

    #[test]
    fn doji_close_uses_balanced_x() {
        // H=110, L=100, C=O=105
        // X = 110 + 100 + 210 = 420 → pivot = 105
        let r = compute(s(105.0, 110.0, 100.0, 105.0)).unwrap();
        assert!((r.pivot - 105.0).abs() < 1e-9);
    }

    #[test]
    fn r1_above_pivot_above_s1() {
        let r = compute(s(108.0, 110.0, 100.0, 102.0)).unwrap();
        assert!(r.r1 > r.pivot && r.pivot > r.s1);
    }

    #[test]
    fn zero_range_collapses_levels() {
        let r = compute(s(100.0, 100.0, 100.0, 100.0)).unwrap();
        assert!((r.r1 - 100.0).abs() < 1e-9);
        assert!((r.s1 - 100.0).abs() < 1e-9);
        assert!((r.pivot - 100.0).abs() < 1e-9);
    }
}
