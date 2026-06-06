//! Gartley 222 Pattern — H.M. Gartley (1935, "Profits in the Stock Market").
//!
//! 5-point harmonic pattern XABCD where the Fibonacci retracement
//! relationships are:
//!   - AB = 0.618 of XA
//!   - BC = 0.382 .. 0.886 of AB
//!   - CD = 1.272 .. 1.618 of BC
//!   - AD = 0.786 of XA   (key Gartley constraint)
//!
//! Detector takes a series of pivot points (alternating swing
//! high/low) and identifies the most recent 5-pivot sequence matching
//! these ratios within `tolerance` (default 5%).
//!
//! Bullish Gartley: X high, A low, B high, C low, D low (D > A, completes
//! at potential reversal zone).
//! Bearish Gartley: X low, A high, B low, C high, D high (D < A).
//!
//! Pure compute. Use `swing_points` to construct pivot input.
//! Companion to `harmonic_patterns` (other Fibonacci patterns),
//! `three_drive_pattern`, `abc_pattern`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pivot {
    pub index: usize,
    pub price: f64,
    /// `true` for swing high, `false` for swing low.
    pub is_high: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GartleyDirection {
    #[default]
    Bullish,
    Bearish,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GartleyMatch {
    pub direction: GartleyDirection,
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

pub fn detect(pivots: &[Pivot], tolerance: f64) -> Vec<GartleyMatch> {
    let mut out = Vec::new();
    if pivots.len() < 5 || !tolerance.is_finite() || tolerance <= 0.0 {
        return out;
    }
    // Pivots must alternate high/low for harmonic interpretation. We
    // walk a sliding window of 5 consecutive pivots.
    for w in pivots.windows(5) {
        let alternating = (1..5).all(|i| w[i].is_high != w[i - 1].is_high);
        if !alternating {
            continue;
        }
        let (x, a, b, c, d) = (w[0], w[1], w[2], w[3], w[4]);
        if let Some(m) = check(x, a, b, c, d, tolerance) {
            out.push(m);
        }
    }
    out
}

fn check(x: Pivot, a: Pivot, b: Pivot, c: Pivot, d: Pivot, tol: f64) -> Option<GartleyMatch> {
    let xa = (a.price - x.price).abs();
    let ab = (b.price - a.price).abs();
    let bc = (c.price - b.price).abs();
    let cd = (d.price - c.price).abs();
    let ad = (d.price - a.price).abs();
    if xa <= 0.0 || ab <= 0.0 || bc <= 0.0 {
        return None;
    }
    let ab_ratio = ab / xa;
    let bc_ratio = bc / ab;
    let cd_ratio = cd / bc;
    let ad_ratio = ad / xa;
    let target_ab = 0.618;
    let target_ad = 0.786;
    if (ab_ratio - target_ab).abs() > tol {
        return None;
    }
    if !(0.382 - tol..=0.886 + tol).contains(&bc_ratio) {
        return None;
    }
    if !(1.272 - tol..=1.618 + tol).contains(&cd_ratio) {
        return None;
    }
    if (ad_ratio - target_ad).abs() > tol {
        return None;
    }
    // Direction by initial X.
    let direction = if x.is_high {
        GartleyDirection::Bearish
    } else {
        GartleyDirection::Bullish
    };
    Some(GartleyMatch {
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
    })
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
        assert!(detect(&[p(0, 100.0, true)], 0.05).is_empty());
        assert!(detect(&[p(0, 100.0, true); 5], 0.0).is_empty());
    }

    #[test]
    fn bullish_gartley_detected() {
        // X(low=100) → A(high=140) → B(low=140 - 0.618·40 = 115.28)
        // → C(high=115.28 + 0.5·24.72 = 127.64) → D(low=A - 0.786·XA
        //   = 140 - 0.786·40 = 108.56).
        // CD = |D - C| = |108.56 - 127.64| = 19.08; BC = 12.36;
        // CD/BC = 1.544 (within 1.272..1.618).
        let pivots = vec![
            p(0, 100.0, false),   // X (low)
            p(10, 140.0, true),   // A (high)
            p(20, 115.28, false), // B
            p(30, 127.64, true),  // C
            p(40, 108.56, false), // D
        ];
        let matches = detect(&pivots, 0.05);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].direction, GartleyDirection::Bullish);
    }

    #[test]
    fn bearish_gartley_detected() {
        let pivots = vec![
            p(0, 200.0, true),
            p(10, 160.0, false),
            p(20, 184.72, true),
            p(30, 172.36, false),
            p(40, 191.44, true),
        ];
        let matches = detect(&pivots, 0.05);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].direction, GartleyDirection::Bearish);
    }

    #[test]
    fn non_alternating_pivots_rejected() {
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 110.0, false), // not alternating
            p(20, 115.28, false),
            p(30, 127.64, true),
            p(40, 108.56, false),
        ];
        let matches = detect(&pivots, 0.05);
        assert!(matches.is_empty());
    }

    #[test]
    fn wrong_ratio_rejected() {
        // AB ratio = 0.3 (way off 0.618) → rejected.
        let pivots = vec![
            p(0, 100.0, false),
            p(10, 140.0, true),
            p(20, 128.0, false), // AB = 12, AB/XA = 0.3
            p(30, 134.0, true),
            p(40, 108.56, false),
        ];
        assert!(detect(&pivots, 0.05).is_empty());
    }
}
