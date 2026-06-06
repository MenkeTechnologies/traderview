//! Kaufman Efficiency Ratio (KER) — directional movement / total movement.
//!
//! Perry Kaufman's measure used as the speed of an adaptive moving
//! average (KAMA). KER is signed (positive = up-trending) and ranges
//! `[-1, 1]`:
//!
//!   `KER = (close − close[lookback]) / sum(|close − close[i-1]| for i in lookback)`
//!
//! Magnitude near 1 → strongly trending (price is moving cleanly with
//! little wiggle); near 0 → fully chopping (lots of motion, no net
//! progress). Sign tells the direction of the net move.
//!
//! Companion to `choppiness` — these two are the most popular trend-
//! efficiency primitives in adaptive-system design.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EfficiencyReport {
    /// Per-bar KER values, None for pre-warmup positions.
    pub series: Vec<Option<f64>>,
    /// Latest value (None if input was too short).
    pub latest: Option<f64>,
}

pub fn compute(closes: &[f64], lookback: usize) -> EfficiencyReport {
    let n = closes.len();
    if n == 0 || lookback < 1 || n <= lookback {
        return EfficiencyReport::default();
    }
    let mut out: Vec<Option<f64>> = vec![None; n];
    for i in lookback..n {
        let direction = closes[i] - closes[i - lookback];
        let total: f64 = (i - lookback + 1..=i)
            .map(|j| (closes[j] - closes[j - 1]).abs())
            .sum();
        if total > 0.0 {
            out[i] = Some(direction / total);
        }
        // total == 0 means a flat segment — leave as None (undefined).
    }
    let latest = out.last().copied().flatten();
    EfficiencyReport {
        series: out,
        latest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_short_returns_default() {
        let r = compute(&[], 10);
        assert!(r.latest.is_none());
        let r = compute(&[1.0, 2.0], 10);
        assert!(r.latest.is_none());
    }

    #[test]
    fn perfect_uptrend_gives_ker_one() {
        // 10 monotonically rising closes — direction = total. KER = +1.
        let closes: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let r = compute(&closes, 9);
        assert!((r.latest.expect("populated") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn perfect_downtrend_gives_ker_minus_one() {
        let closes: Vec<f64> = (1..=10).rev().map(|i| i as f64).collect();
        let r = compute(&closes, 9);
        assert!((r.latest.expect("populated") - (-1.0)).abs() < 1e-9);
    }

    #[test]
    fn full_oscillation_returns_to_start_yields_zero() {
        // 1, 2, 1, 2, 1, 2, 1 — start and end are equal, but lots of motion.
        let closes = vec![1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0];
        let r = compute(&closes, 6);
        let v = r.latest.expect("populated");
        assert!(v.abs() < 1e-9, "round-trip should produce KER ≈ 0, got {v}");
    }

    #[test]
    fn flat_segment_returns_none() {
        // All-equal closes → no motion → KER undefined.
        let closes = vec![5.0; 20];
        let r = compute(&closes, 10);
        assert!(r.latest.is_none(), "flat series → KER undefined");
    }

    #[test]
    fn lookback_one_uses_just_immediate_bar() {
        // KER with lookback 1 = sign of the single delta.
        let closes = vec![100.0, 105.0];
        let r = compute(&closes, 1);
        assert!((r.latest.expect("populated") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn series_lower_than_lookback_returns_none() {
        // n == lookback isn't enough — the formula needs n > lookback.
        let closes = vec![1.0, 2.0, 3.0];
        let r = compute(&closes, 3);
        assert!(r.latest.is_none());
    }
}
