//! Drawdown-conditional position-size throttle.
//!
//! When the account is in drawdown, reduce position size so a recovery
//! doesn't get derailed by a bigger DD. Implements a stepped scale:
//!
//!   |DD %|   →   size multiplier
//!   < 5%      1.00x  (no throttle)
//!   5..10%    0.75x  (small bite)
//!   10..15%   0.50x  (half-size)
//!   15..20%   0.25x  (quarter-size)
//!   ≥ 20%     0.10x  (capital-preservation mode)
//!
//! Configurable via `ThrottleConfig`. Default tiers match common
//! risk-of-ruin guidance — fixed-fractional bet sizing degrades
//! sharply at deep DDs, so cutting bet sizes >50% by the time the
//! account is at 20% DD halves the time-to-recovery curve.
//!
//! Pure compute. Caller passes equity history; engine computes current
//! DD and returns the size multiplier.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottleTier {
    /// Minimum |DD|% (e.g. 0.05 for 5%) that activates this tier.
    pub min_dd: f64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottleConfig {
    /// Ordered tiers — `min_dd` ASCENDING. The active tier is the one
    /// with the highest min_dd that current_dd >= min_dd.
    pub tiers: Vec<ThrottleTier>,
}

impl Default for ThrottleConfig {
    fn default() -> Self {
        Self {
            tiers: vec![
                ThrottleTier {
                    min_dd: 0.0,
                    multiplier: 1.00,
                },
                ThrottleTier {
                    min_dd: 0.05,
                    multiplier: 0.75,
                },
                ThrottleTier {
                    min_dd: 0.10,
                    multiplier: 0.50,
                },
                ThrottleTier {
                    min_dd: 0.15,
                    multiplier: 0.25,
                },
                ThrottleTier {
                    min_dd: 0.20,
                    multiplier: 0.10,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThrottleReport {
    pub current_equity: f64,
    pub peak_equity: f64,
    pub drawdown_pct: f64,
    pub active_multiplier: f64,
    pub tier_min_dd: f64,
    pub note: String,
}

pub fn evaluate(equity_history: &[f64], cfg: &ThrottleConfig) -> ThrottleReport {
    if equity_history.is_empty() || cfg.tiers.is_empty() {
        return ThrottleReport {
            active_multiplier: 1.0,
            note: "no history or no tiers configured — defaulting to full size".into(),
            ..Default::default()
        };
    }
    let current = *equity_history.last().unwrap();
    let peak = equity_history
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let dd = if peak <= 0.0 {
        0.0
    } else {
        ((peak - current) / peak).max(0.0)
    };
    // Pick the highest-min_dd tier that current_dd qualifies for.
    let mut chosen = &cfg.tiers[0];
    for tier in &cfg.tiers {
        if dd >= tier.min_dd {
            chosen = tier;
        }
    }
    ThrottleReport {
        current_equity: current,
        peak_equity: peak,
        drawdown_pct: dd,
        active_multiplier: chosen.multiplier,
        tier_min_dd: chosen.min_dd,
        note: format!(
            "DD {:.2}% — using {:.2}x sizing (tier ≥ {:.0}%)",
            dd * 100.0,
            chosen.multiplier,
            chosen.min_dd * 100.0,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_history_defaults_to_full_size() {
        let r = evaluate(&[], &ThrottleConfig::default());
        assert_eq!(r.active_multiplier, 1.0);
    }

    #[test]
    fn flat_equity_no_throttle() {
        let r = evaluate(&[10_000.0, 10_000.0, 10_000.0], &ThrottleConfig::default());
        assert_eq!(r.drawdown_pct, 0.0);
        assert_eq!(r.active_multiplier, 1.0);
    }

    #[test]
    fn equity_at_peak_no_throttle() {
        let r = evaluate(&[10_000.0, 11_000.0, 12_000.0], &ThrottleConfig::default());
        assert_eq!(r.drawdown_pct, 0.0);
        assert_eq!(r.active_multiplier, 1.0);
    }

    #[test]
    fn small_dd_under_5pct_no_throttle() {
        // Peak 10k, current 9700 → 3% DD.
        let r = evaluate(&[10_000.0, 9_700.0], &ThrottleConfig::default());
        assert!((r.drawdown_pct - 0.03).abs() < 1e-9);
        assert_eq!(r.active_multiplier, 1.0);
    }

    #[test]
    fn dd_5pct_to_10pct_throttles_to_three_quarter() {
        // Peak 10k, current 9300 → 7% DD.
        let r = evaluate(&[10_000.0, 9_300.0], &ThrottleConfig::default());
        assert_eq!(r.active_multiplier, 0.75);
        assert!((r.tier_min_dd - 0.05).abs() < 1e-9);
    }

    #[test]
    fn dd_10pct_to_15pct_throttles_to_half() {
        let r = evaluate(&[10_000.0, 8_800.0], &ThrottleConfig::default());
        assert_eq!(r.active_multiplier, 0.50);
    }

    #[test]
    fn dd_15pct_to_20pct_throttles_to_quarter() {
        let r = evaluate(&[10_000.0, 8_300.0], &ThrottleConfig::default());
        assert_eq!(r.active_multiplier, 0.25);
    }

    #[test]
    fn dd_over_20pct_throttles_to_ten_percent() {
        let r = evaluate(&[10_000.0, 7_500.0], &ThrottleConfig::default());
        assert!(r.drawdown_pct >= 0.20);
        assert_eq!(r.active_multiplier, 0.10);
    }

    #[test]
    fn boundary_at_exactly_5pct_uses_throttled_tier() {
        // 5% exact → should hit the 5% tier (inclusive lower bound).
        let r = evaluate(&[10_000.0, 9_500.0], &ThrottleConfig::default());
        assert_eq!(r.active_multiplier, 0.75);
        assert!((r.tier_min_dd - 0.05).abs() < 1e-9);
    }

    #[test]
    fn custom_tiers_override_default() {
        let cfg = ThrottleConfig {
            tiers: vec![
                ThrottleTier {
                    min_dd: 0.0,
                    multiplier: 1.0,
                },
                ThrottleTier {
                    min_dd: 0.02,
                    multiplier: 0.5,
                }, // aggressive
            ],
        };
        // 3% DD → tier ≥ 2% → 0.5x.
        let r = evaluate(&[10_000.0, 9_700.0], &cfg);
        assert_eq!(r.active_multiplier, 0.5);
    }

    #[test]
    fn recovery_above_old_peak_resets_dd_to_zero() {
        // Peak 10k → 9k → recover to 11k. Current is new peak → DD=0.
        let r = evaluate(&[10_000.0, 9_000.0, 11_000.0], &ThrottleConfig::default());
        assert_eq!(r.drawdown_pct, 0.0);
        assert_eq!(r.active_multiplier, 1.0);
    }
}
