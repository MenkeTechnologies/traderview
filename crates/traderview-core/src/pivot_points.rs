//! Pivot Points — intra-day support/resistance levels derived from the
//! prior session's OHLC.
//!
//! Three variants ship together (universal floor-trader formulas):
//!
//! Classic ("Floor Trader"):
//!   P  = (H + L + C) / 3
//!   R1 = 2P − L,   S1 = 2P − H
//!   R2 = P + (H − L),   S2 = P − (H − L)
//!   R3 = H + 2·(P − L),   S3 = L − 2·(H − P)
//!
//! Fibonacci:
//!   R1 = P + 0.382·(H − L),   S1 = P − 0.382·(H − L)
//!   R2 = P + 0.618·(H − L),   S2 = P − 0.618·(H − L)
//!   R3 = P + 1.000·(H − L),   S3 = P − 1.000·(H − L)
//!
//! Camarilla (Nick Stott):
//!   R1 = C + (H − L)·1.1/12,   S1 = C − (H − L)·1.1/12
//!   R2 = C + (H − L)·1.1/6,    S2 = C − (H − L)·1.1/6
//!   R3 = C + (H − L)·1.1/4,    S3 = C − (H − L)·1.1/4
//!   R4 = C + (H − L)·1.1/2,    S4 = C − (H − L)·1.1/2
//!
//! Pure compute. Caller supplies prior-session OHLC; all three variants
//! returned in a single report.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SessionOhlc {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClassicPivots {
    pub pivot: f64,
    pub r1: f64, pub s1: f64,
    pub r2: f64, pub s2: f64,
    pub r3: f64, pub s3: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FibonacciPivots {
    pub pivot: f64,
    pub r1: f64, pub s1: f64,
    pub r2: f64, pub s2: f64,
    pub r3: f64, pub s3: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CamarillaPivots {
    pub r1: f64, pub s1: f64,
    pub r2: f64, pub s2: f64,
    pub r3: f64, pub s3: f64,
    pub r4: f64, pub s4: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PivotReport {
    pub classic: ClassicPivots,
    pub fibonacci: FibonacciPivots,
    pub camarilla: CamarillaPivots,
}

pub fn compute(prev: SessionOhlc) -> Option<PivotReport> {
    if !prev.high.is_finite() || !prev.low.is_finite() || !prev.close.is_finite() {
        return None;
    }
    if prev.high < prev.low { return None; }
    let h = prev.high;
    let l = prev.low;
    let c = prev.close;
    let range = h - l;
    let p = (h + l + c) / 3.0;
    let classic = ClassicPivots {
        pivot: p,
        r1: 2.0 * p - l,
        s1: 2.0 * p - h,
        r2: p + range,
        s2: p - range,
        r3: h + 2.0 * (p - l),
        s3: l - 2.0 * (h - p),
    };
    let fibonacci = FibonacciPivots {
        pivot: p,
        r1: p + 0.382 * range,
        s1: p - 0.382 * range,
        r2: p + 0.618 * range,
        s2: p - 0.618 * range,
        r3: p + range,
        s3: p - range,
    };
    let camarilla = CamarillaPivots {
        r1: c + range * 1.1 / 12.0,
        s1: c - range * 1.1 / 12.0,
        r2: c + range * 1.1 / 6.0,
        s2: c - range * 1.1 / 6.0,
        r3: c + range * 1.1 / 4.0,
        s3: c - range * 1.1 / 4.0,
        r4: c + range * 1.1 / 2.0,
        s4: c - range * 1.1 / 2.0,
    };
    Some(PivotReport { classic, fibonacci, camarilla })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nan_or_invalid_returns_none() {
        assert!(compute(SessionOhlc { high: f64::NAN, low: 99.0, close: 100.0 }).is_none());
        assert!(compute(SessionOhlc { high: 95.0, low: 99.0, close: 100.0 }).is_none());
    }

    #[test]
    fn classic_pivot_is_typical_price() {
        let r = compute(SessionOhlc { high: 110.0, low: 100.0, close: 105.0 }).unwrap();
        assert!((r.classic.pivot - 105.0).abs() < 1e-9);
    }

    #[test]
    fn classic_resistance_above_support() {
        let r = compute(SessionOhlc { high: 110.0, low: 100.0, close: 105.0 }).unwrap();
        assert!(r.classic.r1 > r.classic.pivot);
        assert!(r.classic.s1 < r.classic.pivot);
        assert!(r.classic.r2 > r.classic.r1);
        assert!(r.classic.s2 < r.classic.s1);
        assert!(r.classic.r3 > r.classic.r2);
        assert!(r.classic.s3 < r.classic.s2);
    }

    #[test]
    fn fibonacci_levels_use_correct_ratios() {
        let r = compute(SessionOhlc { high: 110.0, low: 100.0, close: 105.0 }).unwrap();
        let range = 10.0;
        let p = 105.0;
        assert!((r.fibonacci.r1 - (p + 0.382 * range)).abs() < 1e-9);
        assert!((r.fibonacci.r2 - (p + 0.618 * range)).abs() < 1e-9);
        assert!((r.fibonacci.r3 - (p + range)).abs() < 1e-9);
    }

    #[test]
    fn camarilla_has_four_pairs_increasing_distance() {
        let r = compute(SessionOhlc { high: 110.0, low: 100.0, close: 105.0 }).unwrap();
        assert!(r.camarilla.r4 > r.camarilla.r3);
        assert!(r.camarilla.r3 > r.camarilla.r2);
        assert!(r.camarilla.r2 > r.camarilla.r1);
        assert!(r.camarilla.s1 > r.camarilla.s2);
        assert!(r.camarilla.s2 > r.camarilla.s3);
        assert!(r.camarilla.s3 > r.camarilla.s4);
        // Centered around close.
        assert!((r.camarilla.r1 + r.camarilla.s1 - 2.0 * 105.0).abs() < 1e-9);
    }

    #[test]
    fn zero_range_collapses_all_levels() {
        let r = compute(SessionOhlc { high: 100.0, low: 100.0, close: 100.0 }).unwrap();
        // All levels collapse to the close.
        assert!((r.classic.pivot - 100.0).abs() < 1e-12);
        assert!((r.classic.r1 - 100.0).abs() < 1e-12);
        assert!((r.classic.s1 - 100.0).abs() < 1e-12);
        assert!((r.fibonacci.r3 - 100.0).abs() < 1e-12);
        assert!((r.camarilla.r4 - 100.0).abs() < 1e-12);
    }
}
