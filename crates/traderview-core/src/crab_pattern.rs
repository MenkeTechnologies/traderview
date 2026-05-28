//! Crab Harmonic Pattern — Scott Carney (2000).
//!
//! Deepest harmonic extension. XABCD constraints:
//!   AB = 0.382 .. 0.618 of XA
//!   BC = 0.382 .. 0.886 of AB
//!   CD = 2.618 .. 3.618 of BC   (largest CD leg of any harmonic)
//!   AD = 1.618 of XA            (key Crab constraint)
//!
//! Bullish Crab: D far below X. Bearish Crab: D far above X. Carney
//! considered the Crab the highest-reward harmonic when valid because
//! D often coincides with multi-month extreme exhaustion.
//!
//! Pure compute. Companion to `gartley_pattern`, `bat_pattern`,
//! `butterfly_pattern`.

use serde::{Deserialize, Serialize};

pub use crate::gartley_pattern::Pivot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CrabDirection { #[default] Bullish, Bearish }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CrabMatch {
    pub direction: CrabDirection,
    pub x: Pivot, pub a: Pivot, pub b: Pivot, pub c: Pivot, pub d: Pivot,
    pub ab_ratio: f64, pub bc_ratio: f64, pub cd_ratio: f64, pub ad_ratio: f64,
}

pub fn detect(pivots: &[Pivot], tolerance: f64) -> Vec<CrabMatch> {
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
        if !(0.382 - tolerance..=0.618 + tolerance).contains(&ab_ratio) { continue; }
        if !(0.382 - tolerance..=0.886 + tolerance).contains(&bc_ratio) { continue; }
        if !(2.618 - tolerance..=3.618 + tolerance).contains(&cd_ratio) { continue; }
        if (ad_ratio - 1.618).abs() > tolerance { continue; }
        let direction = if x.is_high { CrabDirection::Bearish }
            else { CrabDirection::Bullish };
        out.push(CrabMatch {
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
    fn bullish_crab_detected() {
        // X=100 (low), A=140 (high). XA=40.
        // AB = 0.5·40 = 20 → B = 120.
        // BC = 0.6·20 = 12 → C = 132.
        // AD = 1.618·40 = 64.72 → D = 140 - 64.72 = 75.28 (well below X).
        // CD = |75.28 - 132| = 56.72; CD/BC = 56.72/12 = 4.727 — too high!
        // Need CD/BC in [2.618, 3.618]. With AD fixed, choose BC so CD/BC ≈ 3.
        // CD = AD + AB_dir effect — but actually CD = |D - C|.
        // D = 75.28, C must be such that CD/BC is right. Let BC = 18 (close to 0.886·20).
        // C = 120 + 18 = 138. CD = |75.28 - 138| = 62.72. CD/BC = 3.484 ✓.
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 120.0, false),
            p(30, 138.0, true),
            p(40, 75.28, false),
        ];
        let matches = detect(&pivots, 0.05);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].direction, CrabDirection::Bullish);
    }

    #[test]
    fn shallow_ad_rejected() {
        // AD = 0.786·XA → wrong (need 1.618).
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 120.0, false),
            p(30, 138.0, true),
            p(40, 108.56, false),    // AD = 31.44 → ratio 0.786
        ];
        assert!(detect(&pivots, 0.05).is_empty());
    }
}
