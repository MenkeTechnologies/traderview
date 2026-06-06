//! Woodie Pivots — Ken Wood (CCI Trading Club).
//!
//! Variant pivot system that double-weights the prior session close:
//!
//!   P  = (H + L + 2C) / 4                 central pivot
//!   R1 = 2P - L                           first resistance
//!   S1 = 2P - H                           first support
//!   R2 = P + (H - L)                      second resistance
//!   S2 = P - (H - L)                      second support
//!   R3 = H + 2·(P - L)                    third resistance
//!   S3 = L - 2·(H - P)                    third support
//!   R4 = R3 + (R2 - R1)                   fourth resistance
//!   S4 = S3 - (S1 - S2)                   fourth support
//!
//! The doubled-close pivot makes Woodie's P more responsive to where
//! the market actually settled vs the simple HLC average. Companion
//! to `floor_pivots`, `camarilla_pivots`, `pivot_points`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriorSession {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct WoodiePivotLevels {
    pub r4: f64,
    pub r3: f64,
    pub r2: f64,
    pub r1: f64,
    pub pivot: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
    pub s4: f64,
}

pub fn compute(session: PriorSession) -> Option<WoodiePivotLevels> {
    if !session.high.is_finite()
        || !session.low.is_finite()
        || !session.close.is_finite()
        || session.high < session.low
    {
        return None;
    }
    let p = (session.high + session.low + 2.0 * session.close) / 4.0;
    let range = session.high - session.low;
    let r1 = 2.0 * p - session.low;
    let s1 = 2.0 * p - session.high;
    let r2 = p + range;
    let s2 = p - range;
    let r3 = session.high + 2.0 * (p - session.low);
    let s3 = session.low - 2.0 * (session.high - p);
    Some(WoodiePivotLevels {
        r4: r3 + (r2 - r1),
        r3,
        r2,
        r1,
        pivot: p,
        s1,
        s2,
        s3,
        s4: s3 - (s1 - s2),
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
    fn close_weighting_shifts_pivot() {
        // Floor pivot for H=110,L=100,C=105 = 105.
        // Woodie pivot = (110 + 100 + 210)/4 = 105.
        // For C=108: Floor = (110+100+108)/3 = 106; Woodie = (110+100+216)/4 = 106.5.
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 108.0,
        })
        .unwrap();
        assert!((r.pivot - 106.5).abs() < 1e-9);
    }

    #[test]
    fn resistance_levels_ordered() {
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 108.0,
        })
        .unwrap();
        assert!(r.pivot < r.r1 && r.r1 < r.r2 && r.r2 < r.r3 && r.r3 < r.r4);
    }

    #[test]
    fn support_levels_ordered() {
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 102.0,
        })
        .unwrap();
        assert!(r.pivot > r.s1 && r.s1 > r.s2 && r.s2 > r.s3 && r.s3 > r.s4);
    }

    #[test]
    fn r4_above_r3_by_r2_minus_r1() {
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 105.0,
        })
        .unwrap();
        assert!((r.r4 - r.r3 - (r.r2 - r.r1)).abs() < 1e-9);
    }

    #[test]
    fn zero_range_collapses_levels_to_close() {
        let r = compute(PriorSession {
            high: 100.0,
            low: 100.0,
            close: 100.0,
        })
        .unwrap();
        for lvl in [r.r4, r.r3, r.r2, r.r1, r.s1, r.s2, r.s3, r.s4] {
            assert!((lvl - 100.0).abs() < 1e-9);
        }
    }
}
