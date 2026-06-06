//! ABCD Harmonic Pattern — Larry Pesavento.
//!
//! 4-pivot ABCD pattern (simpler than 5-pivot Gartley/Bat). Equal-leg
//! variant: BC retraces AB by 0.618 to 0.786, then CD = AB in length
//! and direction (mirror leg):
//!
//!   AB direction defines pattern direction
//!   BC = 0.618 .. 0.786 of AB
//!   CD = 1.000 .. 1.272 of AB           (equal-leg with mild extension)
//!   CD also = 1.272 .. 1.618 of BC      (Fibonacci extension check)
//!
//! Bullish ABCD: A high, B low, C high, D low (D < B).
//! Bearish ABCD: A low, B high, C low, D high (D > B).
//!
//! Pure compute. Detector takes alternating swing-high/low pivots.
//! Companion to other harmonic patterns.

use serde::{Deserialize, Serialize};

pub use crate::gartley_pattern::Pivot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AbcdDirection {
    #[default]
    Bullish,
    Bearish,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AbcdMatch {
    pub direction: AbcdDirection,
    pub a: Pivot,
    pub b: Pivot,
    pub c: Pivot,
    pub d: Pivot,
    pub bc_to_ab: f64,
    pub cd_to_ab: f64,
    pub cd_to_bc: f64,
}

pub fn detect(pivots: &[Pivot], tolerance: f64) -> Vec<AbcdMatch> {
    let mut out = Vec::new();
    if pivots.len() < 4 || !tolerance.is_finite() || tolerance <= 0.0 {
        return out;
    }
    for w in pivots.windows(4) {
        let alternating = (1..4).all(|i| w[i].is_high != w[i - 1].is_high);
        if !alternating {
            continue;
        }
        let (a, b, c, d) = (w[0], w[1], w[2], w[3]);
        let ab = (b.price - a.price).abs();
        let bc = (c.price - b.price).abs();
        let cd = (d.price - c.price).abs();
        if ab <= 0.0 || bc <= 0.0 {
            continue;
        }
        let bc_to_ab = bc / ab;
        let cd_to_ab = cd / ab;
        let cd_to_bc = cd / bc;
        if !(0.618 - tolerance..=0.786 + tolerance).contains(&bc_to_ab) {
            continue;
        }
        if !(1.000 - tolerance..=1.272 + tolerance).contains(&cd_to_ab) {
            continue;
        }
        if !(1.272 - tolerance..=1.618 + tolerance).contains(&cd_to_bc) {
            continue;
        }
        let direction = if a.is_high {
            AbcdDirection::Bullish
        } else {
            AbcdDirection::Bearish
        };
        out.push(AbcdMatch {
            direction,
            a,
            b,
            c,
            d,
            bc_to_ab,
            cd_to_ab,
            cd_to_bc,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(idx: usize, price: f64, is_high: bool) -> Pivot {
        Pivot {
            index: idx,
            price,
            is_high,
        }
    }

    #[test]
    fn empty_or_invalid_returns_empty() {
        assert!(detect(&[], 0.05).is_empty());
        assert!(detect(&[p(0, 100.0, true); 4], 0.0).is_empty());
    }

    #[test]
    fn bullish_abcd_detected() {
        // A=140 high, B=100 low. AB = 40.
        // BC = 0.7·40 = 28 → C = 128 high.
        // CD = 1.0·40 = 40 → D = 88 low. CD/BC = 1.43 ✓.
        let pivots = vec![
            p(0, 140.0, true),
            p(10, 100.0, false),
            p(20, 128.0, true),
            p(30, 88.0, false),
        ];
        let matches = detect(&pivots, 0.05);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].direction, AbcdDirection::Bullish);
    }

    #[test]
    fn bearish_abcd_detected() {
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 112.0, false),
            p(30, 152.0, true),
        ];
        let matches = detect(&pivots, 0.05);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].direction, AbcdDirection::Bearish);
    }

    #[test]
    fn wrong_bc_ratio_rejected() {
        let pivots = vec![
            p(0, 140.0, true),
            p(10, 100.0, false),
            p(20, 120.0, true), // BC = 20, ratio = 0.5 (not 0.618+)
            p(30, 88.0, false),
        ];
        assert!(detect(&pivots, 0.02).is_empty());
    }

    #[test]
    fn non_alternating_pivots_rejected() {
        let pivots = vec![
            p(0, 140.0, true),
            p(10, 100.0, false),
            p(20, 80.0, false), // not alternating
            p(30, 88.0, false),
        ];
        assert!(detect(&pivots, 0.05).is_empty());
    }
}
