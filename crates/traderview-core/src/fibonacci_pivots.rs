//! Fibonacci Pivots.
//!
//! Pivot system using Fibonacci ratios as resistance/support multipliers:
//!
//!   P  = (H + L + C) / 3                      classic pivot
//!   R1 = P + 0.382·(H - L)
//!   R2 = P + 0.618·(H - L)
//!   R3 = P + 1.000·(H - L)
//!   S1 = P - 0.382·(H - L)
//!   S2 = P - 0.618·(H - L)
//!   S3 = P - 1.000·(H - L)
//!
//! Used as objective Fib-aligned intraday/swing levels off the prior
//! session's HLC. Companion to `floor_pivots`, `camarilla_pivots`,
//! `woodie_pivots`, `fibonacci_retracements`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriorSession {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct FibPivotLevels {
    pub r3: f64,
    pub r2: f64,
    pub r1: f64,
    pub pivot: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
}

pub fn compute(session: PriorSession) -> Option<FibPivotLevels> {
    if !session.high.is_finite()
        || !session.low.is_finite()
        || !session.close.is_finite()
        || session.high < session.low
    {
        return None;
    }
    let p = (session.high + session.low + session.close) / 3.0;
    let range = session.high - session.low;
    Some(FibPivotLevels {
        r3: p + range,
        r2: p + 0.618 * range,
        r1: p + 0.382 * range,
        pivot: p,
        s1: p - 0.382 * range,
        s2: p - 0.618 * range,
        s3: p - range,
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
    fn exact_fib_ratios_applied() {
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 105.0,
        })
        .unwrap();
        // P = 105, range = 10.
        assert!((r.r1 - (105.0 + 3.82)).abs() < 1e-9);
        assert!((r.r2 - (105.0 + 6.18)).abs() < 1e-9);
        assert!((r.r3 - 115.0).abs() < 1e-9);
        assert!((r.s1 - (105.0 - 3.82)).abs() < 1e-9);
        assert!((r.s2 - (105.0 - 6.18)).abs() < 1e-9);
        assert!((r.s3 - 95.0).abs() < 1e-9);
    }

    #[test]
    fn resistance_levels_ordered() {
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 105.0,
        })
        .unwrap();
        assert!(r.pivot < r.r1 && r.r1 < r.r2 && r.r2 < r.r3);
    }

    #[test]
    fn support_levels_ordered() {
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 105.0,
        })
        .unwrap();
        assert!(r.pivot > r.s1 && r.s1 > r.s2 && r.s2 > r.s3);
    }

    #[test]
    fn symmetric_around_pivot() {
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 105.0,
        })
        .unwrap();
        assert!((r.r1 - r.pivot - (r.pivot - r.s1)).abs() < 1e-9);
        assert!((r.r2 - r.pivot - (r.pivot - r.s2)).abs() < 1e-9);
        assert!((r.r3 - r.pivot - (r.pivot - r.s3)).abs() < 1e-9);
    }

    #[test]
    fn zero_range_collapses_to_close() {
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
