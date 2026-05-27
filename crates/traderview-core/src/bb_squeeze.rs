//! Bollinger Band Squeeze detector (TTM Squeeze, John Carter).
//!
//! Volatility-contraction → expansion regime-change signal. When
//! Bollinger Bands (2σ around 20-period MA) shrink INSIDE the Keltner
//! Channels (1.5×ATR around 20-period EMA), the market is unusually
//! quiet — historically that's followed by an outsized expansion move.
//!
//! Three states:
//!   - **InSqueeze**: BB inside KC. Wait for release.
//!   - **Released**: just exited a squeeze — entry signal.
//!   - **Normal**: BB outside KC.
//!
//! Pure compute. Operates on a pre-computed history of (close, sma20,
//! stdev20, ema20, atr20) so the engine doesn't recompute indicators
//! callers already have.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct SqueezeInput {
    pub close: f64,
    pub sma_20: f64,
    pub stdev_20: f64,
    pub ema_20: f64,
    pub atr_20: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqueezeState {
    InSqueeze,
    Released,
    Normal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqueezeBar {
    pub state: SqueezeState,
    pub bb_upper: f64,
    pub bb_lower: f64,
    pub kc_upper: f64,
    pub kc_lower: f64,
}

pub fn analyze(input: &[SqueezeInput]) -> Vec<SqueezeBar> {
    let mut out = Vec::with_capacity(input.len());
    let mut prev_in_squeeze = false;
    for bar in input {
        let bb_upper = bar.sma_20 + 2.0 * bar.stdev_20;
        let bb_lower = bar.sma_20 - 2.0 * bar.stdev_20;
        let kc_upper = bar.ema_20 + 1.5 * bar.atr_20;
        let kc_lower = bar.ema_20 - 1.5 * bar.atr_20;
        let in_squeeze = bb_upper < kc_upper && bb_lower > kc_lower;
        let state = if in_squeeze {
            SqueezeState::InSqueeze
        } else if prev_in_squeeze {
            SqueezeState::Released
        } else {
            SqueezeState::Normal
        };
        prev_in_squeeze = in_squeeze;
        out.push(SqueezeBar {
            state,
            bb_upper,
            bb_lower,
            kc_upper,
            kc_lower,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(close: f64, sma: f64, stdev: f64, ema: f64, atr: f64) -> SqueezeInput {
        SqueezeInput {
            close,
            sma_20: sma,
            stdev_20: stdev,
            ema_20: ema,
            atr_20: atr,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(analyze(&[]).is_empty());
    }

    #[test]
    fn wide_bb_outside_kc_is_normal_state() {
        // sma=100, stdev=5 → BB ±10. ema=100, atr=3 → KC ±4.5.
        // BB (90, 110) is OUTSIDE KC (95.5, 104.5) → Normal.
        let out = analyze(&[inp(100.0, 100.0, 5.0, 100.0, 3.0)]);
        assert_eq!(out[0].state, SqueezeState::Normal);
    }

    #[test]
    fn narrow_bb_inside_kc_is_in_squeeze() {
        // sma=100, stdev=1 → BB ±2. ema=100, atr=5 → KC ±7.5.
        // BB (98, 102) INSIDE KC (92.5, 107.5) → InSqueeze.
        let out = analyze(&[inp(100.0, 100.0, 1.0, 100.0, 5.0)]);
        assert_eq!(out[0].state, SqueezeState::InSqueeze);
    }

    #[test]
    fn transition_from_squeeze_to_normal_emits_released() {
        let out = analyze(&[
            inp(100.0, 100.0, 1.0, 100.0, 5.0), // InSqueeze
            inp(100.0, 100.0, 5.0, 100.0, 3.0), // Released
            inp(100.0, 100.0, 5.0, 100.0, 3.0), // Normal
        ]);
        assert_eq!(out[0].state, SqueezeState::InSqueeze);
        assert_eq!(out[1].state, SqueezeState::Released);
        assert_eq!(out[2].state, SqueezeState::Normal);
    }

    #[test]
    fn continuous_squeeze_stays_in_squeeze() {
        let out = analyze(&[
            inp(100.0, 100.0, 1.0, 100.0, 5.0),
            inp(100.0, 100.0, 1.0, 100.0, 5.0),
            inp(100.0, 100.0, 1.0, 100.0, 5.0),
        ]);
        for bar in &out {
            assert_eq!(bar.state, SqueezeState::InSqueeze);
        }
    }

    #[test]
    fn release_only_fires_once_then_returns_to_normal() {
        let out = analyze(&[
            inp(100.0, 100.0, 1.0, 100.0, 5.0), // InSqueeze
            inp(100.0, 100.0, 5.0, 100.0, 3.0), // Released
            inp(100.0, 100.0, 5.0, 100.0, 3.0), // Normal
            inp(100.0, 100.0, 5.0, 100.0, 3.0), // Normal
        ]);
        let released_count = out
            .iter()
            .filter(|b| b.state == SqueezeState::Released)
            .count();
        assert_eq!(released_count, 1);
    }

    #[test]
    fn bb_kc_bands_emitted_per_bar() {
        let out = analyze(&[inp(100.0, 100.0, 5.0, 100.0, 4.0)]);
        assert_eq!(out[0].bb_upper, 110.0);
        assert_eq!(out[0].bb_lower, 90.0);
        assert_eq!(out[0].kc_upper, 106.0);
        assert_eq!(out[0].kc_lower, 94.0);
    }

    #[test]
    fn first_bar_normal_when_not_in_squeeze() {
        // No prior state → not released on bar 0.
        let out = analyze(&[inp(100.0, 100.0, 5.0, 100.0, 3.0)]);
        assert_eq!(out[0].state, SqueezeState::Normal);
    }

    #[test]
    fn bb_equal_kc_is_not_strictly_in_squeeze() {
        // BB == KC exactly → strict < fails → Normal.
        // sma=100, stdev=2 → BB ±4. ema=100, atr=8/3 → KC ±4.
        // BB upper 104 NOT < KC upper 104 → Normal.
        let out = analyze(&[inp(100.0, 100.0, 2.0, 100.0, 8.0 / 3.0)]);
        assert_eq!(out[0].state, SqueezeState::Normal);
    }
}
