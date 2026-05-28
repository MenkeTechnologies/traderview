//! Bat Harmonic Pattern — Scott M. Carney ("Harmonic Trading").
//!
//! 5-point XABCD pattern with these Fibonacci constraints:
//!   AB = 0.382 .. 0.500 of XA
//!   BC = 0.382 .. 0.886 of AB
//!   CD = 1.618 .. 2.618 of BC
//!   AD = 0.886 of XA       (key Bat constraint, tighter than Gartley)
//!
//! Pure compute. Detector takes alternating swing-high/low pivots.
//! Companion to `gartley_pattern`, `harmonic_patterns`.

use serde::{Deserialize, Serialize};

pub use crate::gartley_pattern::Pivot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BatDirection { #[default] Bullish, Bearish }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BatMatch {
    pub direction: BatDirection,
    pub x: Pivot,
    pub a: Pivot,
    pub b: Pivot,
    pub c: Pivot,
    pub d: Pivot,
    pub ab_ratio: f64,
    pub bc_ratio: f64,
    pub cd_ratio: f64,
    pub ad_ratio: f64,
}

pub fn detect(pivots: &[Pivot], tolerance: f64) -> Vec<BatMatch> {
    let mut out = Vec::new();
    if pivots.len() < 5 || !tolerance.is_finite() || tolerance <= 0.0 {
        return out;
    }
    for w in pivots.windows(5) {
        let alternating = (1..5).all(|i| w[i].is_high != w[i - 1].is_high);
        if !alternating { continue; }
        let (x, a, b, c, d) = (w[0], w[1], w[2], w[3], w[4]);
        let xa = (a.price - x.price).abs();
        let ab = (b.price - a.price).abs();
        let bc = (c.price - b.price).abs();
        let cd = (d.price - c.price).abs();
        let ad = (d.price - a.price).abs();
        if xa <= 0.0 || ab <= 0.0 || bc <= 0.0 { continue; }
        let ab_ratio = ab / xa;
        let bc_ratio = bc / ab;
        let cd_ratio = cd / bc;
        let ad_ratio = ad / xa;
        if !(0.382 - tolerance..=0.500 + tolerance).contains(&ab_ratio) { continue; }
        if !(0.382 - tolerance..=0.886 + tolerance).contains(&bc_ratio) { continue; }
        if !(1.618 - tolerance..=2.618 + tolerance).contains(&cd_ratio) { continue; }
        if (ad_ratio - 0.886).abs() > tolerance { continue; }
        let direction = if x.is_high { BatDirection::Bearish }
            else { BatDirection::Bullish };
        out.push(BatMatch {
            direction, x, a, b, c, d, ab_ratio, bc_ratio, cd_ratio, ad_ratio,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(idx: usize, price: f64, is_high: bool) -> Pivot {
        Pivot { index: idx, price, is_high }
    }

    #[test]
    fn empty_or_invalid_returns_empty() {
        assert!(detect(&[], 0.05).is_empty());
        assert!(detect(&[p(0, 100.0, true); 5], 0.0).is_empty());
    }

    #[test]
    fn bullish_bat_detected() {
        // X=100 (low), A=140 (high). XA=40.
        // AB = 0.45·40 = 18 → B = 140 - 18 = 122 (low).
        // BC = 0.6·18 = 10.8 → C = 122 + 10.8 = 132.8 (high).
        // CD = 2.0·10.8 = 21.6 → D = 132.8 - 21.6 = 111.2 (low).
        // AD = |D - A| = 28.8; AD/XA = 0.72 — too low.
        // Need AD/XA = 0.886 → AD = 35.44 → D = A - 35.44 = 104.56.
        // CD then = |104.56 - 132.8| = 28.24; CD/BC = 28.24/10.8 = 2.615 ✓.
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 122.0, false),
            p(30, 132.8, true),
            p(40, 104.56, false),
        ];
        let matches = detect(&pivots, 0.05);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].direction, BatDirection::Bullish);
    }

    #[test]
    fn wrong_ad_ratio_rejected() {
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 122.0, false),
            p(30, 132.8, true),
            p(40, 115.0, false),    // AD/XA = 25/40 = 0.625 (not 0.886)
        ];
        assert!(detect(&pivots, 0.02).is_empty());
    }

    #[test]
    fn non_alternating_pivots_rejected() {
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 122.0, false),
            p(30, 132.8, false),    // not alternating
            p(40, 104.56, false),
        ];
        assert!(detect(&pivots, 0.05).is_empty());
    }
}
