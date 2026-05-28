//! Fibonacci Extensions — target levels beyond the 100% retracement.
//!
//! Given a swing low (A) and swing high (B) of an impulse leg, projects
//! Fibonacci extension targets above (bullish trend) or below (bearish):
//!
//!   range = |B - A|
//!   ext_127 = base + range · 1.272
//!   ext_161 = base + range · 1.618
//!   ext_200 = base + range · 2.000
//!   ext_261 = base + range · 2.618
//!   ext_423 = base + range · 4.236
//!
//! `base` is the retracement low (for bullish) or retracement high (for
//! bearish). Caller picks A, B, and base from prior pivots.
//!
//! Pure compute. Companion to `fibonacci_retracements`, `gartley_pattern`,
//! `andrews_pitchfork`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct FibonacciExtensionLevels {
    pub ext_127: f64,
    pub ext_161: f64,
    pub ext_200: f64,
    pub ext_261: f64,
    pub ext_423: f64,
}

pub fn compute(a: f64, b: f64, retracement_base: f64, bullish: bool) -> Option<FibonacciExtensionLevels> {
    if !a.is_finite() || !b.is_finite() || !retracement_base.is_finite() {
        return None;
    }
    let range = (b - a).abs();
    if range <= 0.0 { return None; }
    let sign = if bullish { 1.0 } else { -1.0 };
    Some(FibonacciExtensionLevels {
        ext_127: retracement_base + sign * range * 1.272,
        ext_161: retracement_base + sign * range * 1.618,
        ext_200: retracement_base + sign * range * 2.000,
        ext_261: retracement_base + sign * range * 2.618,
        ext_423: retracement_base + sign * range * 4.236,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(f64::NAN, 100.0, 95.0, true).is_none());
        assert!(compute(100.0, 100.0, 95.0, true).is_none());    // zero range
    }

    #[test]
    fn bullish_extensions_above_base() {
        // A=100 (low), B=110 (high), range=10, base=105.
        let r = compute(100.0, 110.0, 105.0, true).unwrap();
        assert!((r.ext_127 - (105.0 + 12.72)).abs() < 1e-9);
        assert!((r.ext_161 - (105.0 + 16.18)).abs() < 1e-9);
        assert!((r.ext_200 - 125.0).abs() < 1e-9);
        assert!((r.ext_261 - (105.0 + 26.18)).abs() < 1e-9);
        assert!((r.ext_423 - (105.0 + 42.36)).abs() < 1e-9);
    }

    #[test]
    fn bearish_extensions_below_base() {
        let r = compute(110.0, 100.0, 105.0, false).unwrap();
        assert!((r.ext_127 - (105.0 - 12.72)).abs() < 1e-9);
        assert!((r.ext_423 - (105.0 - 42.36)).abs() < 1e-9);
    }

    #[test]
    fn extensions_ordered_monotone() {
        let r = compute(100.0, 110.0, 105.0, true).unwrap();
        assert!(r.ext_127 < r.ext_161);
        assert!(r.ext_161 < r.ext_200);
        assert!(r.ext_200 < r.ext_261);
        assert!(r.ext_261 < r.ext_423);
    }

    #[test]
    fn range_independent_of_direction() {
        // Same A/B in either order gives same range, but signed differently.
        let r1 = compute(100.0, 110.0, 105.0, true).unwrap();
        let r2 = compute(110.0, 100.0, 105.0, true).unwrap();
        assert!((r1.ext_161 - r2.ext_161).abs() < 1e-9);
    }
}
