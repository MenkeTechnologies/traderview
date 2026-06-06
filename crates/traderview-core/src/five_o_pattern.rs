//! 5-0 Harmonic Pattern — Scott Carney (TASC, 2003).
//!
//! 5-point pattern looking for a deep retracement at point D after
//! an extended 3-leg move (X→A→B→C). Specific ratio gates:
//!
//!   AB / XA = 1.130 .. 1.618      (extension past X)
//!   BC / AB = 1.618 .. 2.240      (deep extension past A)
//!   CD / BC = 0.500               (50 % retracement of the BC leg —
//!                                  the "5-0" name comes from this ratio)
//!
//! The trade is entered on completion of D, expecting a snap back
//! toward C. Distinct from the Shark pattern (Shark's CD also targets
//! 0.886-1.13 of XC, not the simple 0.5 BC retrace).
//!
//! Pure compute. Detector takes alternating swing-high/low pivots.
//! Companion to `shark_pattern`, `gartley_pattern`, `bat_pattern`,
//! `butterfly_pattern`, `crab_pattern`, `cypher_pattern`,
//! `abcd_pattern`.

use serde::{Deserialize, Serialize};

pub use crate::gartley_pattern::Pivot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FiveODirection {
    #[default]
    Bullish,
    Bearish,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FiveOMatch {
    pub direction: FiveODirection,
    pub x: Pivot,
    pub a: Pivot,
    pub b: Pivot,
    pub c: Pivot,
    pub d: Pivot,
    pub ab_ratio: f64,
    pub bc_ratio: f64,
    pub cd_to_bc_ratio: f64,
}

pub fn detect(pivots: &[Pivot], tolerance: f64) -> Vec<FiveOMatch> {
    let mut out = Vec::new();
    if pivots.len() < 5 || !tolerance.is_finite() || tolerance <= 0.0 {
        return out;
    }
    for w in pivots.windows(5) {
        let alternating = (1..5).all(|i| w[i].is_high != w[i - 1].is_high);
        if !alternating {
            continue;
        }
        let (x, a, b, c, d) = (w[0], w[1], w[2], w[3], w[4]);
        let xa = (a.price - x.price).abs();
        let ab = (b.price - a.price).abs();
        let bc = (c.price - b.price).abs();
        let cd = (d.price - c.price).abs();
        if xa <= 0.0 || ab <= 0.0 || bc <= 0.0 {
            continue;
        }
        let ab_ratio = ab / xa;
        let bc_ratio = bc / ab;
        let cd_to_bc_ratio = cd / bc;
        if !(1.130 - tolerance..=1.618 + tolerance).contains(&ab_ratio) {
            continue;
        }
        if !(1.618 - tolerance..=2.240 + tolerance).contains(&bc_ratio) {
            continue;
        }
        if (cd_to_bc_ratio - 0.500).abs() > tolerance {
            continue;
        }
        let direction = if x.is_high {
            FiveODirection::Bearish
        } else {
            FiveODirection::Bullish
        };
        out.push(FiveOMatch {
            direction,
            x,
            a,
            b,
            c,
            d,
            ab_ratio,
            bc_ratio,
            cd_to_bc_ratio,
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
        assert!(detect(&[p(0, 100.0, true); 5], 0.0).is_empty());
    }

    #[test]
    fn bullish_five_o_detected() {
        // X=100 (low), A=140 (high). XA = 40.
        // AB = 1.3·40 = 52 → B = 140 - 52 = 88 (low, below X).
        // BC = 2.0·52 = 104 → C = 88 + 104 = 192 (high, well above A).
        // CD = 0.5·104 = 52 → D = 192 - 52 = 140 (high).
        // But D and C are same polarity (both high) — non-alternating!
        // Fix: pivots must alternate. After C high, D must be low.
        // So D = C - 52 = 140 (low). Geometry: D=140 below C=192.
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 88.0, false),
            p(30, 192.0, true),
            p(40, 140.0, false),
        ];
        let matches = detect(&pivots, 0.05);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].direction, FiveODirection::Bullish);
    }

    #[test]
    fn wrong_cd_ratio_rejected() {
        // CD = 0.3·BC → fails 0.5 target.
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 88.0, false),
            p(30, 192.0, true),
            p(40, 161.0, false), // CD = 31, ratio = 0.298
        ];
        assert!(detect(&pivots, 0.05).is_empty());
    }

    #[test]
    fn non_alternating_pivots_rejected() {
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 88.0, true), // not alternating
            p(30, 192.0, true),
            p(40, 140.0, false),
        ];
        assert!(detect(&pivots, 0.05).is_empty());
    }
}
