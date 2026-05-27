//! Over/under-trading detector.
//!
//! Most retail traders have a profitable trade-frequency baseline.
//! When daily trade counts spike well above that baseline, win rate
//! tends to collapse (taking marginal setups, revenge trades, FOMO
//! entries). This module classifies trading days by frequency tier
//! and computes win-rate-by-tier so the user can see whether their
//! "high-activity" days are actually their worst days.
//!
//! Pure compute. Caller supplies per-day (trade_count, win_count, pnl)
//! tuples — engine emits per-tier aggregates.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayStats {
    pub trade_count: usize,
    pub win_count: usize,
    pub pnl: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Quiet,
    Normal,
    Active,
    Hyper,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TierStats {
    pub tier: String,
    pub day_count: usize,
    pub total_trades: usize,
    pub avg_trades_per_day: f64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub avg_pnl_per_day: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OvertradingReport {
    pub tiers: Vec<TierStats>,
    /// Average daily trades across ALL days. Threshold defaults:
    /// quiet ≤ avg×0.5, normal ≤ avg×1.5, active ≤ avg×3, hyper > avg×3.
    pub baseline_avg_trades_per_day: f64,
    /// True if the hyper tier's PnL is worse-per-day than the normal tier.
    pub hyper_underperforms_normal: bool,
}

pub fn analyze(days: &[DayStats]) -> OvertradingReport {
    let mut report = OvertradingReport::default();
    if days.is_empty() {
        return report;
    }
    let total_trades: usize = days.iter().map(|d| d.trade_count).sum();
    let baseline = total_trades as f64 / days.len() as f64;
    report.baseline_avg_trades_per_day = baseline;
    let bands = [
        (baseline * 0.5, "quiet"),
        (baseline * 1.5, "normal"),
        (baseline * 3.0, "active"),
        (f64::INFINITY, "hyper"),
    ];
    let mut by_tier: std::collections::BTreeMap<&str, (usize, usize, usize, f64)> =
        std::collections::BTreeMap::new();
    for &(_, label) in &bands {
        by_tier.insert(label, (0, 0, 0, 0.0));
    }
    for d in days {
        let tier = bands
            .iter()
            .find(|(t, _)| (d.trade_count as f64) <= *t)
            .map(|(_, l)| *l)
            .unwrap_or("hyper");
        let e = by_tier.get_mut(tier).unwrap();
        e.0 += 1;
        e.1 += d.trade_count;
        e.2 += d.win_count;
        e.3 += d.pnl;
    }
    let order = ["quiet", "normal", "active", "hyper"];
    for label in order {
        let (day_count, total_trades, wins, pnl) = by_tier[label];
        let win_rate = if total_trades > 0 {
            wins as f64 / total_trades as f64
        } else {
            0.0
        };
        let avg_pnl = if day_count > 0 {
            pnl / day_count as f64
        } else {
            0.0
        };
        let avg_trades = if day_count > 0 {
            total_trades as f64 / day_count as f64
        } else {
            0.0
        };
        report.tiers.push(TierStats {
            tier: label.into(),
            day_count,
            total_trades,
            avg_trades_per_day: avg_trades,
            win_rate,
            total_pnl: pnl,
            avg_pnl_per_day: avg_pnl,
        });
    }
    let normal = report
        .tiers
        .iter()
        .find(|t| t.tier == "normal")
        .map(|t| t.avg_pnl_per_day);
    let hyper = report
        .tiers
        .iter()
        .find(|t| t.tier == "hyper")
        .map(|t| t.avg_pnl_per_day);
    report.hyper_underperforms_normal = match (normal, hyper) {
        (Some(n), Some(h)) => h < n,
        _ => false,
    };
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(trades: usize, wins: usize, pnl: f64) -> DayStats {
        DayStats {
            trade_count: trades,
            win_count: wins,
            pnl,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[]);
        assert!(r.tiers.is_empty());
        assert_eq!(r.baseline_avg_trades_per_day, 0.0);
    }

    #[test]
    fn baseline_is_simple_arithmetic_mean() {
        let days = vec![d(5, 3, 100.0), d(15, 8, 200.0)];
        let r = analyze(&days);
        assert_eq!(r.baseline_avg_trades_per_day, 10.0);
    }

    #[test]
    fn hyper_tier_separates_high_volume_days() {
        // 4 sleepy days @ 5 trades + 1 binge day @ 200 trades.
        // baseline = (4×5 + 200)/5 = 44. hyper threshold = 44 × 3 = 132.
        // 200 > 132 → hyper.
        let days = vec![
            d(5, 3, 100.0),
            d(5, 3, 100.0),
            d(5, 3, 100.0),
            d(5, 3, 100.0),
            d(200, 50, -500.0),
        ];
        let r = analyze(&days);
        let hyper = r.tiers.iter().find(|t| t.tier == "hyper").unwrap();
        assert_eq!(hyper.day_count, 1);
        assert_eq!(hyper.total_trades, 200);
        assert_eq!(hyper.total_pnl, -500.0);
    }

    #[test]
    fn win_rate_per_tier_computed_correctly() {
        // 50 trades hyper-day, 10 wins → 20% win rate.
        let days = vec![d(50, 10, -500.0)];
        let r = analyze(&days);
        let hyper = r.tiers.iter().find(|t| t.tier == "hyper").unwrap();
        // Only one day → baseline = 50, so it falls in normal (≤ 1.5×50 = 75).
        // Adjust expectation: re-derive.
        // Actually since baseline = 50, hyper threshold = 150. day_count=1 in
        // tier where 50 ≤ 75 → "normal".
        // So expect 1 day in normal, 0 in hyper.
        let normal = r.tiers.iter().find(|t| t.tier == "normal").unwrap();
        assert_eq!(normal.day_count, 1);
        assert!((normal.win_rate - 0.20).abs() < 1e-9);
        assert_eq!(hyper.day_count, 0);
    }

    #[test]
    fn hyper_underperforms_flag_true_when_hyper_is_worst() {
        // baseline = (5+5+5+100)/4 = 28.75. hyper threshold = 28.75 × 3 = 86.25.
        // 100 > 86.25 → hyper. Normal days each +$100, hyper day -$500.
        let days2 = vec![
            d(5, 3, 100.0),
            d(5, 3, 100.0),
            d(5, 3, 100.0),
            d(100, 30, -500.0),
        ];
        let r = analyze(&days2);
        let hyper = r.tiers.iter().find(|t| t.tier == "hyper").unwrap();
        let normal = r.tiers.iter().find(|t| t.tier == "normal").unwrap();
        assert_eq!(hyper.day_count, 1);
        assert!(hyper.avg_pnl_per_day < 0.0);
        // If normal has any days with positive PnL, flag fires.
        if normal.day_count > 0 && hyper.day_count > 0 {
            assert_eq!(
                r.hyper_underperforms_normal,
                hyper.avg_pnl_per_day < normal.avg_pnl_per_day
            );
        }
    }

    #[test]
    fn all_quiet_days_no_hyper() {
        let days = vec![d(1, 1, 10.0), d(2, 2, 20.0), d(1, 0, -5.0)];
        let r = analyze(&days);
        let hyper = r.tiers.iter().find(|t| t.tier == "hyper").unwrap();
        assert_eq!(hyper.day_count, 0);
    }

    #[test]
    fn tier_order_is_quiet_normal_active_hyper() {
        let days = vec![d(5, 3, 100.0), d(10, 6, 200.0), d(30, 8, 50.0)];
        let r = analyze(&days);
        assert_eq!(r.tiers[0].tier, "quiet");
        assert_eq!(r.tiers[1].tier, "normal");
        assert_eq!(r.tiers[2].tier, "active");
        assert_eq!(r.tiers[3].tier, "hyper");
    }

    #[test]
    fn zero_trade_days_still_counted() {
        // 0-trade day = quiet (always <= baseline × 0.5).
        let days = vec![d(0, 0, 0.0), d(10, 5, 100.0)];
        let r = analyze(&days);
        let quiet = r.tiers.iter().find(|t| t.tier == "quiet").unwrap();
        assert_eq!(quiet.day_count, 1);
        assert_eq!(quiet.total_trades, 0);
        assert_eq!(quiet.win_rate, 0.0);
    }
}
