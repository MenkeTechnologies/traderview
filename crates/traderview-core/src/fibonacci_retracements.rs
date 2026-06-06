//! Fibonacci Retracements + Extensions.
//!
//! Given a swing leg from price A (start) to price B (end), compute
//! the standard horizontal levels that traders watch:
//!
//! Retracements (counter-trend pullbacks measured back from B):
//!   23.6% : 0.236
//!   38.2% : 0.382
//!   50.0% : 0.500     (not strictly Fibonacci, kept by convention)
//!   61.8% : 0.618     (golden ratio)
//!   78.6% : 0.786
//!
//! Extensions (continuation beyond B):
//!   100.0% : 1.000    (B itself)
//!   127.2% : 1.272
//!   161.8% : 1.618
//!   200.0% : 2.000
//!   261.8% : 2.618
//!
//! Direction is inferred from A vs B:
//!   - B > A (uptrend leg)  → retracements descend from B toward A;
//!     extensions sit above B
//!   - B < A (downtrend leg) → retracements ascend from B toward A;
//!     extensions sit below B
//!
//! Pure compute. The hard part is the swing-leg detection (use
//! `swing_points` or similar); this module takes the leg as given and
//! emits the price levels.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FibonacciLevels {
    pub leg_start: f64,
    pub leg_end: f64,
    pub leg_direction: Option<LegDirection>,
    pub retracement_pct: Vec<f64>,
    pub retracement_levels: Vec<f64>,
    pub extension_pct: Vec<f64>,
    pub extension_levels: Vec<f64>,
}

const RETRACEMENT_PCTS: [f64; 5] = [0.236, 0.382, 0.500, 0.618, 0.786];
const EXTENSION_PCTS: [f64; 5] = [1.000, 1.272, 1.618, 2.000, 2.618];

pub fn compute(leg_start: f64, leg_end: f64) -> Option<FibonacciLevels> {
    if !leg_start.is_finite() || !leg_end.is_finite() {
        return None;
    }
    let dir = match leg_end.partial_cmp(&leg_start) {
        Some(std::cmp::Ordering::Greater) => Some(LegDirection::Up),
        Some(std::cmp::Ordering::Less) => Some(LegDirection::Down),
        _ => None, // zero-length leg
    };
    let range = leg_end - leg_start;
    // Retracements: pull back from leg_end toward leg_start.
    let retracement: Vec<f64> = RETRACEMENT_PCTS
        .iter()
        .map(|p| leg_end - range * p)
        .collect();
    // Extensions: project beyond leg_end in the leg direction.
    let extension: Vec<f64> = EXTENSION_PCTS
        .iter()
        .map(|p| leg_start + range * p)
        .collect();
    Some(FibonacciLevels {
        leg_start,
        leg_end,
        leg_direction: dir,
        retracement_pct: RETRACEMENT_PCTS.to_vec(),
        retracement_levels: retracement,
        extension_pct: EXTENSION_PCTS.to_vec(),
        extension_levels: extension,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nan_inputs_return_none() {
        assert!(compute(f64::NAN, 100.0).is_none());
        assert!(compute(100.0, f64::NAN).is_none());
    }

    #[test]
    fn uptrend_leg_direction_detected() {
        let r = compute(100.0, 200.0).unwrap();
        assert_eq!(r.leg_direction, Some(LegDirection::Up));
    }

    #[test]
    fn downtrend_leg_direction_detected() {
        let r = compute(200.0, 100.0).unwrap();
        assert_eq!(r.leg_direction, Some(LegDirection::Down));
    }

    #[test]
    fn zero_length_leg_yields_no_direction() {
        let r = compute(100.0, 100.0).unwrap();
        assert!(r.leg_direction.is_none());
        // All retracements and extensions degenerate to the same price.
        for v in &r.retracement_levels {
            assert_eq!(*v, 100.0);
        }
        for v in &r.extension_levels {
            assert_eq!(*v, 100.0);
        }
    }

    #[test]
    fn uptrend_retracements_below_leg_end() {
        let r = compute(100.0, 200.0).unwrap();
        // All retracements should sit between leg_start and leg_end.
        for v in &r.retracement_levels {
            assert!(*v >= 100.0 && *v <= 200.0);
        }
        // 50% retracement = midpoint.
        let idx = r
            .retracement_pct
            .iter()
            .position(|p| (p - 0.5).abs() < 1e-9)
            .unwrap();
        assert!((r.retracement_levels[idx] - 150.0).abs() < 1e-9);
    }

    #[test]
    fn downtrend_retracements_above_leg_end() {
        let r = compute(200.0, 100.0).unwrap();
        for v in &r.retracement_levels {
            assert!(*v >= 100.0 && *v <= 200.0);
        }
    }

    #[test]
    fn uptrend_extensions_above_leg_end() {
        let r = compute(100.0, 200.0).unwrap();
        for (pct, lvl) in r.extension_pct.iter().zip(r.extension_levels.iter()) {
            if *pct > 1.0 {
                assert!(
                    *lvl > 200.0,
                    "extension at {pct} = {lvl} should exceed leg_end 200"
                );
            }
        }
    }

    #[test]
    fn downtrend_extensions_below_leg_end() {
        let r = compute(200.0, 100.0).unwrap();
        for (pct, lvl) in r.extension_pct.iter().zip(r.extension_levels.iter()) {
            if *pct > 1.0 {
                assert!(
                    *lvl < 100.0,
                    "downtrend extension at {pct} = {lvl} should sit below 100"
                );
            }
        }
    }

    #[test]
    fn known_618_retracement_value() {
        // From 100 to 200, the 61.8% retracement is 200 - 100·0.618 = 138.2.
        let r = compute(100.0, 200.0).unwrap();
        let idx = r
            .retracement_pct
            .iter()
            .position(|p| (p - 0.618).abs() < 1e-9)
            .unwrap();
        assert!((r.retracement_levels[idx] - 138.2).abs() < 1e-9);
    }
}
