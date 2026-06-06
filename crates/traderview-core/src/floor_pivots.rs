//! Floor Trader's Pivots — classic 7-level intraday pivot system.
//!
//! Based on prior-session high/low/close. Forms one central pivot
//! plus three resistances and three supports:
//!
//!   P  = (H + L + C) / 3                  central pivot
//!   R1 = 2·P - L                          first resistance
//!   S1 = 2·P - H                          first support
//!   R2 = P + (H - L)                      second resistance
//!   S2 = P - (H - L)                      second support
//!   R3 = H + 2·(P - L)                    third resistance
//!   S3 = L - 2·(H - P)                    third support
//!
//! Note: `pivot_points` module (if shipped) may use Woodie or
//! DeMark variants — this is the classic "floor" formulation used
//! by S&P/futures pit traders.
//!
//! Pure compute. Companion to `camarilla_pivots`, `pivot_points`,
//! `murrey_math`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriorSession {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct FloorPivotLevels {
    pub r3: f64,
    pub r2: f64,
    pub r1: f64,
    pub pivot: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
}

pub fn compute(session: PriorSession) -> Option<FloorPivotLevels> {
    if !session.high.is_finite()
        || !session.low.is_finite()
        || !session.close.is_finite()
        || session.high < session.low
    {
        return None;
    }
    let p = (session.high + session.low + session.close) / 3.0;
    let range = session.high - session.low;
    Some(FloorPivotLevels {
        r3: session.high + 2.0 * (p - session.low),
        r2: p + range,
        r1: 2.0 * p - session.low,
        pivot: p,
        s1: 2.0 * p - session.high,
        s2: p - range,
        s3: session.low - 2.0 * (session.high - p),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_session_returns_none() {
        assert!(compute(PriorSession {
            high: f64::NAN,
            low: 99.0,
            close: 100.0
        })
        .is_none());
        assert!(compute(PriorSession {
            high: 99.0,
            low: 101.0,
            close: 100.0
        })
        .is_none());
    }

    #[test]
    fn exact_formula_values() {
        // H=110, L=100, C=105 → P = 105, range = 10
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 105.0,
        })
        .unwrap();
        assert!((r.pivot - 105.0).abs() < 1e-9);
        assert!((r.r1 - (2.0 * 105.0 - 100.0)).abs() < 1e-9); // 110
        assert!((r.s1 - (2.0 * 105.0 - 110.0)).abs() < 1e-9); // 100
        assert!((r.r2 - 115.0).abs() < 1e-9); // P + range
        assert!((r.s2 - 95.0).abs() < 1e-9); // P - range
        assert!((r.r3 - (110.0 + 2.0 * (105.0 - 100.0))).abs() < 1e-9); // 120
        assert!((r.s3 - (100.0 - 2.0 * (110.0 - 105.0))).abs() < 1e-9); // 90
    }

    #[test]
    fn resistance_levels_ordered() {
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 108.0,
        })
        .unwrap();
        assert!(r.pivot < r.r1 && r.r1 < r.r2 && r.r2 < r.r3);
    }

    #[test]
    fn support_levels_ordered() {
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 102.0,
        })
        .unwrap();
        assert!(r.pivot > r.s1 && r.s1 > r.s2 && r.s2 > r.s3);
    }

    #[test]
    fn zero_range_collapses_levels_to_close() {
        let r = compute(PriorSession {
            high: 100.0,
            low: 100.0,
            close: 100.0,
        })
        .unwrap();
        for lvl in [r.r3, r.r2, r.r1, r.s1, r.s2, r.s3] {
            assert!((lvl - 100.0).abs() < 1e-9);
        }
    }
}
