//! Gann Fan — W.D. Gann angles.
//!
//! Plots nine geometric trend lines fanning out from a chosen pivot
//! (high or low) at the canonical Gann angles. Each angle is a
//! price-units-per-time-unit ratio anchored at the pivot:
//!
//!   1x8 →  1 / 8 unit per bar
//!   1x4 →  1 / 4
//!   1x3 →  1 / 3
//!   1x2 →  1 / 2
//!   1x1 →  1     (Gann's "main" 45° line)
//!   2x1 →  2
//!   3x1 →  3
//!   4x1 →  4
//!   8x1 →  8
//!
//! Each fan line is `anchor_price + sign · slope · bars_since_anchor`.
//! Sign = +1 if `up = true` (fan rising from a low), -1 if `up = false`
//! (fan falling from a high).
//!
//! `unit_per_bar` parameter scales the canonical ratios to the
//! instrument's actual price scale (a 45° line on a stock priced
//! at $100 is meaningless unless 1 price unit ≈ 1 bar visually).
//! Default 1.0; callers should set this from the chart's recent
//! median bar range / median price change for visual accuracy.
//!
//! Pure compute. Companion to `andrews_pitchfork`, `pivot_points`,
//! `murrey_math`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct GannFanLevels {
    pub line_1x8: f64,
    pub line_1x4: f64,
    pub line_1x3: f64,
    pub line_1x2: f64,
    pub line_1x1: f64,
    pub line_2x1: f64,
    pub line_3x1: f64,
    pub line_4x1: f64,
    pub line_8x1: f64,
}

/// Evaluate the 9 Gann fan lines at `bars_from_anchor` ahead of the
/// anchor. `up = true` projects upward fan from a swing low; `false`
/// projects downward fan from a swing high.
pub fn compute(
    anchor_price: f64,
    bars_from_anchor: usize,
    unit_per_bar: f64,
    up: bool,
) -> Option<GannFanLevels> {
    if !anchor_price.is_finite() || !unit_per_bar.is_finite() || unit_per_bar <= 0.0 {
        return None;
    }
    let t = bars_from_anchor as f64;
    let sign = if up { 1.0 } else { -1.0 };
    let one_unit = unit_per_bar * t;
    Some(GannFanLevels {
        line_1x8: anchor_price + sign * one_unit / 8.0,
        line_1x4: anchor_price + sign * one_unit / 4.0,
        line_1x3: anchor_price + sign * one_unit / 3.0,
        line_1x2: anchor_price + sign * one_unit / 2.0,
        line_1x1: anchor_price + sign * one_unit,
        line_2x1: anchor_price + sign * 2.0 * one_unit,
        line_3x1: anchor_price + sign * 3.0 * one_unit,
        line_4x1: anchor_price + sign * 4.0 * one_unit,
        line_8x1: anchor_price + sign * 8.0 * one_unit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(f64::NAN, 10, 1.0, true).is_none());
        assert!(compute(100.0, 10, 0.0, true).is_none());
        assert!(compute(100.0, 10, f64::INFINITY, true).is_none());
    }

    #[test]
    fn at_anchor_all_lines_equal_anchor() {
        let r = compute(100.0, 0, 1.0, true).unwrap();
        for v in [
            r.line_1x8, r.line_1x4, r.line_1x3, r.line_1x2, r.line_1x1, r.line_2x1, r.line_3x1,
            r.line_4x1, r.line_8x1,
        ] {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn upward_fan_lines_strictly_ordered() {
        let r = compute(100.0, 10, 1.0, true).unwrap();
        assert!(r.line_1x8 < r.line_1x4);
        assert!(r.line_1x4 < r.line_1x3);
        assert!(r.line_1x3 < r.line_1x2);
        assert!(r.line_1x2 < r.line_1x1);
        assert!(r.line_1x1 < r.line_2x1);
        assert!(r.line_2x1 < r.line_3x1);
        assert!(r.line_3x1 < r.line_4x1);
        assert!(r.line_4x1 < r.line_8x1);
    }

    #[test]
    fn downward_fan_lines_strictly_ordered() {
        let r = compute(100.0, 10, 1.0, false).unwrap();
        assert!(r.line_1x8 > r.line_1x4);
        assert!(r.line_8x1 < r.line_4x1);
    }

    #[test]
    fn one_by_one_line_matches_unit_per_bar() {
        // 10 bars × 1.0 unit/bar → 1x1 line = anchor + 10.
        let r = compute(100.0, 10, 1.0, true).unwrap();
        assert!((r.line_1x1 - 110.0).abs() < 1e-9);
    }

    #[test]
    fn eight_x_one_line_is_eight_times_unit() {
        let r = compute(100.0, 10, 1.0, true).unwrap();
        assert!((r.line_8x1 - (100.0 + 80.0)).abs() < 1e-9);
    }

    #[test]
    fn one_x_eight_line_is_eighth_of_unit() {
        let r = compute(100.0, 8, 1.0, true).unwrap();
        // 8 bars × 1.0 unit / 8 = 1.0.
        assert!((r.line_1x8 - 101.0).abs() < 1e-9);
    }
}
