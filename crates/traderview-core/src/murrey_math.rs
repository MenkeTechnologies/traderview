//! Murrey Math Levels — T. Henning Murrey (1995).
//!
//! Divides a price range into eighths (the "octave"), with each level
//! having traditional significance:
//!   0/8 = "ultimate support" (lowest)
//!   1/8 = "weak, stall, reverse" zone
//!   2/8 = "pivot, reverse" zone
//!   3/8 = "bottom of trading range" (lower 25%)
//!   4/8 = "major support/resistance" (midpoint)
//!   5/8 = "top of trading range" (upper 25%)
//!   6/8 = "pivot, reverse" zone
//!   7/8 = "weak, stall, reverse" zone
//!   8/8 = "ultimate resistance" (highest)
//!
//! Plus 4 extended levels at -2/8, -1/8, 9/8, 10/8 for breakout targets.
//!
//! Auto-detects the octave from the most recent `lookback_bars` price
//! action: rounds the high/low to a "Murrey base" (power-of-2 fraction
//! that contains the range with a small margin) — then divides into 8.
//!
//! Pure compute. Returns the 13 levels (0/8..8/8 + 4 extended) and the
//! current price's nearest level for display.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MurreyLevels {
    pub levels: Vec<(String, f64)>,
    pub current_price: f64,
    pub nearest_level: Option<(String, f64)>,
    pub distance_to_nearest_pct: Option<f64>,
}

pub fn compute(bars: &[Bar], lookback_bars: usize) -> Option<MurreyLevels> {
    if lookback_bars == 0 || bars.is_empty() {
        return None;
    }
    let n = bars.len();
    let window = lookback_bars.min(n);
    let mut hi = f64::NEG_INFINITY;
    let mut lo = f64::INFINITY;
    for b in bars.iter().rev().take(window) {
        if !b.high.is_finite() || !b.low.is_finite() { continue; }
        if b.high > hi { hi = b.high; }
        if b.low < lo { lo = b.low; }
    }
    if !hi.is_finite() || !lo.is_finite() || hi <= lo {
        return None;
    }
    let current_price = bars[n - 1].close;
    if !current_price.is_finite() { return None; }
    // Choose a Murrey "base" — the power-of-2 fraction whose octave
    // comfortably contains [lo, hi]. We pick base such that
    //   (hi − lo) ≤ base · 8/8.
    //
    // Equivalently, base = pow2_ceil(range). Then bottom of octave is
    // floor(lo / base) · base.
    let range = hi - lo;
    if range <= 0.0 { return None; }
    let base = next_power_of_two(range);
    if base <= 0.0 || !base.is_finite() { return None; }
    let octave_bottom = (lo / base).floor() * base;
    let mut levels = Vec::with_capacity(13);
    for k in -2..=10 {
        let label = format!("{k}/8");
        let value = octave_bottom + (k as f64) * (base / 8.0);
        levels.push((label, value));
    }
    // Find nearest level to current price.
    let mut nearest: Option<(String, f64, f64)> = None;
    for (lbl, v) in &levels {
        let d = (v - current_price).abs();
        if nearest.as_ref().is_none_or(|(_, _, best_d)| d < *best_d) {
            nearest = Some((lbl.clone(), *v, d));
        }
    }
    let (nearest_label, nearest_price, _) = nearest?;
    let distance_pct = (nearest_price - current_price).abs() / current_price * 100.0;
    Some(MurreyLevels {
        levels,
        current_price,
        nearest_level: Some((nearest_label, nearest_price)),
        distance_to_nearest_pct: Some(distance_pct),
    })
}

fn next_power_of_two(x: f64) -> f64 {
    if !x.is_finite() || x <= 0.0 { return f64::NAN; }
    // 2^ceil(log2(x))
    let p = x.log2().ceil();
    2.0_f64.powf(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[], 20).is_none());
    }

    #[test]
    fn zero_lookback_returns_none() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert!(compute(&bars, 0).is_none());
    }

    #[test]
    fn flat_bars_return_none() {
        let bars = vec![b(100.0, 100.0, 100.0); 30];
        assert!(compute(&bars, 20).is_none());
    }

    #[test]
    fn nan_bars_skipped() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[15] = b(f64::NAN, f64::NAN, f64::NAN);
        let r = compute(&bars, 20).expect("populated");
        assert_eq!(r.levels.len(), 13);
    }

    #[test]
    fn levels_are_ordered() {
        let bars = vec![b(105.0, 95.0, 100.0); 30];
        let r = compute(&bars, 20).expect("populated");
        for w in r.levels.windows(2) {
            assert!(w[1].1 > w[0].1, "levels must be ascending");
        }
    }

    #[test]
    fn level_spacing_equals_one_eighth_of_base() {
        let bars = vec![b(105.0, 95.0, 100.0); 30];
        let r = compute(&bars, 20).expect("populated");
        let spacing = r.levels[1].1 - r.levels[0].1;
        for w in r.levels.windows(2) {
            assert!(((w[1].1 - w[0].1) - spacing).abs() < 1e-9);
        }
    }

    #[test]
    fn nearest_level_within_one_eighth_of_current() {
        let bars = vec![b(105.0, 95.0, 100.0); 30];
        let r = compute(&bars, 20).expect("populated");
        let (_lbl, near_price) = r.nearest_level.unwrap();
        let spacing = r.levels[1].1 - r.levels[0].1;
        // Distance must be ≤ half the spacing.
        assert!((near_price - r.current_price).abs() <= spacing / 2.0 + 1e-9);
    }

    #[test]
    fn distance_pct_finite_and_nonnegative() {
        let bars = vec![b(105.0, 95.0, 100.0); 30];
        let r = compute(&bars, 20).expect("populated");
        let d = r.distance_to_nearest_pct.unwrap();
        assert!(d.is_finite() && d >= 0.0);
    }

    #[test]
    fn lookback_larger_than_input_uses_all_bars() {
        let bars = vec![b(101.0, 99.0, 100.0); 5];
        let r = compute(&bars, 1000).expect("populated");
        assert_eq!(r.levels.len(), 13);
    }
}
