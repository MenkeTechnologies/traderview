//! Cost-to-borrow (CTB) spike detector for short-squeeze warning.
//!
//! Sudden jumps in the borrow rate signal scarcity — prime brokers
//! raise the fee when they can't locate shares to lend, which is
//! exactly the condition that traps shorts. A jump from 5% APR to 50%
//! APR overnight has historically preceded major squeeze events.
//!
//! ### Inputs
//!
//! Time-ordered series of `(date, ctb_apr)` pairs per symbol. CTB is
//! the annualized rate as a decimal (e.g. `0.45` for 45%).
//!
//! ### Detection rules
//!
//! Two configurable triggers:
//!
//! 1. **Absolute level**: CTB ≥ `level_threshold` (default 0.50 = 50%
//!    APR). Anything above 25-30% is historically rare and means the
//!    stock is hard to borrow.
//! 2. **Rate-of-change**: CTB increased by at least `roc_multiplier`
//!    over the lookback window (default: 3× over 5 days). A jump from
//!    5% to 50% APR easily crosses this.
//!
//! Both rules fire independently — a single observation can trigger
//! one, the other, or both.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CtbPoint {
    /// Days since epoch (or any monotonic integer). The series is
    /// indexed by relative ordering, not calendar arithmetic.
    pub day: i64,
    /// Annualized borrow rate as decimal — 0.05 = 5% APR.
    pub apr: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AlertConfig {
    pub level_threshold: f64,
    pub roc_multiplier: f64,
    pub lookback_days: i64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            level_threshold: 0.50,
            roc_multiplier: 3.0,
            lookback_days: 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertReason {
    /// Absolute level crossed `level_threshold`.
    LevelCrossed,
    /// CTB rose by ≥ `roc_multiplier` over the lookback window.
    RateOfChange,
    /// Both rules fired on this observation.
    Both,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CtbAlert {
    pub day: i64,
    /// Current CTB on the alert day.
    pub current_apr: f64,
    /// CTB at start of lookback window, if available.
    pub baseline_apr: Option<f64>,
    /// Multiplier vs baseline (current / baseline). `None` when baseline
    /// is missing or zero.
    pub multiple_vs_baseline: Option<f64>,
    pub reason: AlertReason,
}

pub fn detect(points: &[CtbPoint], cfg: &AlertConfig) -> Vec<CtbAlert> {
    let mut alerts = Vec::new();
    if cfg.lookback_days < 1
        || !cfg.level_threshold.is_finite()
        || !cfg.roc_multiplier.is_finite()
        || cfg.roc_multiplier <= 1.0
        || cfg.level_threshold <= 0.0
    {
        return alerts;
    }

    for (i, p) in points.iter().enumerate() {
        if !p.apr.is_finite() || p.apr < 0.0 {
            continue;
        }
        let current = p.apr;

        // Look backwards for the most recent point at-or-before
        // (day - lookback). We walk i-1..0 because points may have
        // gaps in their day axis.
        let cutoff = p.day - cfg.lookback_days;
        let baseline = points[..i]
            .iter()
            .rev()
            .find(|q| q.day <= cutoff && q.apr.is_finite() && q.apr > 0.0)
            .map(|q| q.apr);

        let multiple = baseline.map(|b| current / b);

        let level_fired = current >= cfg.level_threshold;
        let roc_fired = multiple.map(|m| m >= cfg.roc_multiplier).unwrap_or(false);

        let reason = match (level_fired, roc_fired) {
            (true, true) => Some(AlertReason::Both),
            (true, false) => Some(AlertReason::LevelCrossed),
            (false, true) => Some(AlertReason::RateOfChange),
            (false, false) => None,
        };
        if let Some(reason) = reason {
            alerts.push(CtbAlert {
                day: p.day,
                current_apr: current,
                baseline_apr: baseline,
                multiple_vs_baseline: multiple,
                reason,
            });
        }
    }
    alerts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(day: i64, apr: f64) -> CtbPoint {
        CtbPoint { day, apr }
    }

    #[test]
    fn empty_series_no_alerts() {
        assert!(detect(&[], &AlertConfig::default()).is_empty());
    }

    #[test]
    fn level_alert_fires_above_50_pct() {
        // Constant 60% APR. Each point is above threshold; first is
        // alone (no lookback), subsequent ones still fire on level.
        let pts = vec![p(0, 0.60), p(1, 0.60), p(2, 0.60)];
        let alerts = detect(&pts, &AlertConfig::default());
        assert!(!alerts.is_empty());
        assert!(alerts
            .iter()
            .all(|a| matches!(a.reason, AlertReason::LevelCrossed | AlertReason::Both)));
    }

    #[test]
    fn no_alerts_below_thresholds() {
        let pts = vec![p(0, 0.05), p(1, 0.06), p(2, 0.07)];
        let alerts = detect(&pts, &AlertConfig::default());
        assert!(alerts.is_empty());
    }

    #[test]
    fn rate_of_change_alert_fires_on_3x_jump() {
        // Day 0: 5% APR. Day 5: 20% APR. 20/5 = 4× over 5-day lookback.
        let pts = vec![p(0, 0.05), p(5, 0.20)];
        let alerts = detect(&pts, &AlertConfig::default());
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].day, 5);
        assert_eq!(alerts[0].reason, AlertReason::RateOfChange);
        assert!((alerts[0].multiple_vs_baseline.unwrap() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn both_reason_fires_when_level_and_roc_both_hit() {
        // 5% → 60%. Crosses level AND 12× change.
        let pts = vec![p(0, 0.05), p(5, 0.60)];
        let alerts = detect(&pts, &AlertConfig::default());
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].reason, AlertReason::Both);
    }

    #[test]
    fn roc_window_respects_lookback_days() {
        // Default lookback = 5 days. Jump happens after 10 days.
        // Within the window (5 days back from day 10 = day 5), CTB is
        // also low → roc still fires.
        let pts = vec![p(0, 0.05), p(5, 0.06), p(10, 0.30)];
        let alerts = detect(&pts, &AlertConfig::default());
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].day, 10);
        // Multiple is 30/6 = 5×, not 30/5 = 6× — uses the latest point
        // within the lookback.
        assert!((alerts[0].multiple_vs_baseline.unwrap() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn very_first_point_no_baseline_no_roc_alert() {
        // Single high point alone fires level but cannot fire ROC.
        let pts = vec![p(0, 0.60)];
        let alerts = detect(&pts, &AlertConfig::default());
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].reason, AlertReason::LevelCrossed);
        assert!(alerts[0].baseline_apr.is_none());
    }

    #[test]
    fn nan_apr_skipped() {
        let pts = vec![p(0, 0.05), p(5, f64::NAN), p(10, 0.60)];
        let alerts = detect(&pts, &AlertConfig::default());
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].day, 10);
    }

    #[test]
    fn invalid_config_no_alerts() {
        let pts = vec![p(0, 0.05), p(5, 0.60)];
        for cfg in [
            AlertConfig {
                lookback_days: 0,
                ..Default::default()
            },
            AlertConfig {
                level_threshold: -1.0,
                ..Default::default()
            },
            AlertConfig {
                roc_multiplier: 1.0,
                ..Default::default()
            },
        ] {
            assert!(detect(&pts, &cfg).is_empty());
        }
    }

    #[test]
    fn realistic_gme_jan_2021_borrow_rate_spike() {
        // Approximate weekly snapshots leading into the squeeze.
        let pts = vec![
            p(0, 0.05),
            p(7, 0.08),
            p(14, 0.20),
            p(21, 0.45),
            p(28, 1.50),
        ];
        let alerts = detect(&pts, &AlertConfig::default());
        // Latest two should fire Both; earlier ones fire RateOfChange
        // first, then LevelCrossed at 0.45+ and Both at 1.50.
        let last = alerts.last().unwrap();
        assert_eq!(last.day, 28);
        assert_eq!(last.reason, AlertReason::Both);
        assert!(last.current_apr >= 1.0);
    }
}
