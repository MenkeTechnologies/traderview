//! Butterfly Harmonic Pattern — Bryce Gilmore / Scott Carney.
//!
//! Extension pattern with D beyond X. XABCD constraints:
//!   AB = 0.786 of XA
//!   BC = 0.382 .. 0.886 of AB
//!   CD = 1.618 .. 2.618 of BC
//!   AD = 1.272 .. 1.618 of XA   (key Butterfly: D extends past X)
//!
//! For a bullish butterfly: D is below X (deeper low). For a bearish:
//! D is above X (higher high). The deep AD extension is the trade
//! setup's distinguishing feature versus Gartley or Bat.
//!
//! Pure compute. Companion to `gartley_pattern`, `bat_pattern`.

use serde::{Deserialize, Serialize};

pub use crate::gartley_pattern::Pivot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ButterflyDirection {
    #[default]
    Bullish,
    Bearish,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ButterflyMatch {
    pub direction: ButterflyDirection,
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

pub fn detect(pivots: &[Pivot], tolerance: f64) -> Vec<ButterflyMatch> {
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
        let ad = (d.price - a.price).abs();
        if xa <= 0.0 || ab <= 0.0 || bc <= 0.0 {
            continue;
        }
        let ab_ratio = ab / xa;
        let bc_ratio = bc / ab;
        let cd_ratio = cd / bc;
        let ad_ratio = ad / xa;
        if (ab_ratio - 0.786).abs() > tolerance {
            continue;
        }
        if !(0.382 - tolerance..=0.886 + tolerance).contains(&bc_ratio) {
            continue;
        }
        if !(1.618 - tolerance..=2.618 + tolerance).contains(&cd_ratio) {
            continue;
        }
        if !(1.272 - tolerance..=1.618 + tolerance).contains(&ad_ratio) {
            continue;
        }
        let direction = if x.is_high {
            ButterflyDirection::Bearish
        } else {
            ButterflyDirection::Bullish
        };
        out.push(ButterflyMatch {
            direction,
            x,
            a,
            b,
            c,
            d,
            ab_ratio,
            bc_ratio,
            cd_ratio,
            ad_ratio,
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
    fn bullish_butterfly_detected() {
        // X=100 (low), A=140 (high). XA=40.
        // AB = 0.786·40 = 31.44 → B = 140 - 31.44 = 108.56.
        // BC = 0.5·31.44 = 15.72 → C = 108.56 + 15.72 = 124.28.
        // AD = 1.272·40 = 50.88 → D = A - 50.88 = 89.12 (below X — extension).
        // CD = |89.12 - 124.28| = 35.16; CD/BC = 35.16/15.72 = 2.237 ✓.
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 108.56, false),
            p(30, 124.28, true),
            p(40, 89.12, false),
        ];
        let matches = detect(&pivots, 0.05);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].direction, ButterflyDirection::Bullish);
    }

    #[test]
    fn wrong_ab_ratio_rejected() {
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 120.0, false), // AB ratio = 0.5 (not 0.786)
            p(30, 124.28, true),
            p(40, 89.12, false),
        ];
        assert!(detect(&pivots, 0.02).is_empty());
    }

    #[test]
    fn d_inside_x_rejected_when_ad_short() {
        // AD = 0.786·XA → ad_ratio = 0.786 → outside [1.272, 1.618].
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 108.56, false),
            p(30, 124.28, true),
            p(40, 108.56, false), // AD = 31.44, ratio 0.786 → wrong
        ];
        assert!(detect(&pivots, 0.05).is_empty());
    }
}
