//! Fibonacci retracement + extension calculator.
//!
//! Given a swing high and swing low, emit the canonical retracement
//! levels (23.6%, 38.2%, 50%, 61.8%, 78.6%) traders watch for reversal
//! entries, and extension levels (127.2%, 161.8%, 261.8%) for
//! continuation targets.
//!
//! Direction matters — for an uptrend swing, retracements pull DOWN
//! from the swing high; for a downtrend swing, retracements pull UP
//! from the swing low. Extension targets project BEYOND the prior swing.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwingDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FibInput {
    pub swing_high: f64,
    pub swing_low: f64,
    pub direction: SwingDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FibLevel {
    pub ratio: f64,
    pub label: String,
    pub price: f64,
    pub kind: FibKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FibKind {
    Retracement,
    Extension,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FibReport {
    pub retracements: Vec<FibLevel>,
    pub extensions: Vec<FibLevel>,
    /// Swing range = high - low (always positive).
    pub range: f64,
}

pub fn compute(input: &FibInput) -> FibReport {
    let range = (input.swing_high - input.swing_low).abs();
    let retracement_ratios = [0.236, 0.382, 0.500, 0.618, 0.786];
    let extension_ratios = [1.272, 1.618, 2.0, 2.618];
    let mut retracements = Vec::new();
    let mut extensions = Vec::new();
    for r in retracement_ratios {
        let price = match input.direction {
            // Up swing: retracements pull down from the high.
            SwingDirection::Up => input.swing_high - range * r,
            // Down swing: retracements pull up from the low.
            SwingDirection::Down => input.swing_low + range * r,
        };
        retracements.push(FibLevel {
            ratio: r,
            label: format!("{:.1}%", r * 100.0),
            price,
            kind: FibKind::Retracement,
        });
    }
    for r in extension_ratios {
        let price = match input.direction {
            // Up swing: extensions project ABOVE the swing high.
            SwingDirection::Up => input.swing_high + range * (r - 1.0),
            // Down swing: extensions project BELOW the swing low.
            SwingDirection::Down => input.swing_low - range * (r - 1.0),
        };
        extensions.push(FibLevel {
            ratio: r,
            label: format!("{:.1}%", r * 100.0),
            price,
            kind: FibKind::Extension,
        });
    }
    FibReport {
        retracements,
        extensions,
        range,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up_swing_50pct_retracement_at_midpoint() {
        // High 200, low 100, range 100. 50% pulls down to 200-50 = 150.
        let r = compute(&FibInput {
            swing_high: 200.0,
            swing_low: 100.0,
            direction: SwingDirection::Up,
        });
        let fifty = r.retracements.iter().find(|l| l.ratio == 0.5).unwrap();
        assert_eq!(fifty.price, 150.0);
    }

    #[test]
    fn down_swing_50pct_retracement_at_midpoint() {
        // High 200, low 100. Down swing 50% pulls up from low to 150.
        let r = compute(&FibInput {
            swing_high: 200.0,
            swing_low: 100.0,
            direction: SwingDirection::Down,
        });
        let fifty = r.retracements.iter().find(|l| l.ratio == 0.5).unwrap();
        assert_eq!(fifty.price, 150.0);
    }

    #[test]
    fn up_swing_786_retracement_near_low() {
        // 200 - 100 × 0.786 = 121.4.
        let r = compute(&FibInput {
            swing_high: 200.0,
            swing_low: 100.0,
            direction: SwingDirection::Up,
        });
        let level = r
            .retracements
            .iter()
            .find(|l| (l.ratio - 0.786).abs() < 1e-6)
            .unwrap();
        assert!((level.price - 121.4).abs() < 1e-9);
    }

    #[test]
    fn up_swing_extension_161_above_high() {
        // High 200 + range × (1.618 - 1) = 200 + 61.8 = 261.8.
        let r = compute(&FibInput {
            swing_high: 200.0,
            swing_low: 100.0,
            direction: SwingDirection::Up,
        });
        let ext = r
            .extensions
            .iter()
            .find(|l| (l.ratio - 1.618).abs() < 1e-6)
            .unwrap();
        assert!((ext.price - 261.8).abs() < 1e-9);
    }

    #[test]
    fn down_swing_extension_below_low() {
        // Low 100 - range × (1.618 - 1) = 100 - 61.8 = 38.2.
        let r = compute(&FibInput {
            swing_high: 200.0,
            swing_low: 100.0,
            direction: SwingDirection::Down,
        });
        let ext = r
            .extensions
            .iter()
            .find(|l| (l.ratio - 1.618).abs() < 1e-6)
            .unwrap();
        assert!((ext.price - 38.2).abs() < 1e-9);
    }

    #[test]
    fn retracements_emit_in_increasing_ratio_order() {
        let r = compute(&FibInput {
            swing_high: 200.0,
            swing_low: 100.0,
            direction: SwingDirection::Up,
        });
        for i in 1..r.retracements.len() {
            assert!(r.retracements[i].ratio > r.retracements[i - 1].ratio);
        }
    }

    #[test]
    fn extensions_at_127_161_200_2618_all_emitted() {
        let r = compute(&FibInput {
            swing_high: 200.0,
            swing_low: 100.0,
            direction: SwingDirection::Up,
        });
        assert_eq!(r.extensions.len(), 4);
        let ratios: Vec<f64> = r.extensions.iter().map(|l| l.ratio).collect();
        assert!(ratios.contains(&1.272));
        assert!(ratios.contains(&1.618));
        assert!(ratios.contains(&2.0));
        assert!(ratios.contains(&2.618));
    }

    #[test]
    fn zero_range_swing_collapses_all_levels_to_same_price() {
        // Degenerate: high == low.
        let r = compute(&FibInput {
            swing_high: 100.0,
            swing_low: 100.0,
            direction: SwingDirection::Up,
        });
        for level in &r.retracements {
            assert_eq!(level.price, 100.0);
        }
        for level in &r.extensions {
            assert_eq!(level.price, 100.0);
        }
    }

    #[test]
    fn swing_high_lower_than_low_uses_abs_range() {
        // Defensive: if caller swaps the args, range still positive.
        let r = compute(&FibInput {
            swing_high: 100.0,
            swing_low: 200.0,
            direction: SwingDirection::Up,
        });
        assert_eq!(r.range, 100.0);
    }
}
