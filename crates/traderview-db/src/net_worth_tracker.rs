//! Net-worth snapshot + trend tracker.
//!
//! Personal-finance fundamentals: net worth = total assets − total
//! liabilities. Given a list of assets + liabilities (each with a
//! category) plus an optional history of prior monthly snapshots,
//! computes:
//!
//!   - net_worth_usd
//!   - total_assets_usd / total_liabilities_usd
//!   - by_asset_category / by_liability_category (sorted high → low)
//!   - mom_delta_usd / mom_delta_pct  (vs last month if history ≥ 2)
//!   - yoy_delta_usd / yoy_delta_pct  (vs 12 months ago if history ≥ 13)
//!   - status = "positive" (NW > 0) | "underwater" (NW ≤ 0)
//!   - debt_to_asset_pct = liabilities / assets × 100
//!
//! Pure compute — no DB I/O, no clock reads.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
pub struct LineItem {
    pub name: String,
    pub category: String,
    pub value_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryPoint {
    pub month: String,
    pub net_worth_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetWorthInput {
    #[serde(default)]
    pub assets: Vec<LineItem>,
    #[serde(default)]
    pub liabilities: Vec<LineItem>,
    /// Oldest → newest; the last entry is treated as "this month",
    /// not the assets/liabilities sum (so the caller can choose
    /// whether to feed in history or rely on the live sum).
    #[serde(default)]
    pub history: Vec<HistoryPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryBucket {
    pub category: String,
    pub total_usd: f64,
    pub share_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetWorthReport {
    pub net_worth_usd: f64,
    pub total_assets_usd: f64,
    pub total_liabilities_usd: f64,
    pub by_asset_category: Vec<CategoryBucket>,
    pub by_liability_category: Vec<CategoryBucket>,
    pub mom_delta_usd: Option<f64>,
    pub mom_delta_pct: Option<f64>,
    pub yoy_delta_usd: Option<f64>,
    pub yoy_delta_pct: Option<f64>,
    pub status: String,
    pub debt_to_asset_pct: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn total(items: &[LineItem]) -> f64 {
    items.iter().map(|i| i.value_usd).sum()
}

pub fn by_category(items: &[LineItem]) -> Vec<CategoryBucket> {
    let total_all: f64 = total(items);
    let mut groups: BTreeMap<String, f64> = BTreeMap::new();
    for it in items {
        *groups.entry(it.category.clone()).or_default() += it.value_usd;
    }
    let mut out: Vec<CategoryBucket> = groups
        .into_iter()
        .map(|(category, total_usd)| {
            let share_pct = if total_all > 0.0 {
                total_usd / total_all * 100.0
            } else {
                0.0
            };
            CategoryBucket {
                category,
                total_usd,
                share_pct,
            }
        })
        .collect();
    out.sort_by(|a, b| b.total_usd.partial_cmp(&a.total_usd).unwrap_or(std::cmp::Ordering::Equal));
    out
}

pub fn delta(curr: f64, prev: f64) -> (f64, Option<f64>) {
    let abs_delta = curr - prev;
    let pct = if prev.abs() > 1e-9 {
        Some(abs_delta / prev.abs() * 100.0)
    } else {
        None
    };
    (abs_delta, pct)
}

pub fn debt_to_asset(total_assets: f64, total_liabilities: f64) -> f64 {
    if total_assets <= 0.0 {
        return if total_liabilities > 0.0 { 100.0 } else { 0.0 };
    }
    total_liabilities / total_assets * 100.0
}

pub fn compute(input: &NetWorthInput) -> NetWorthReport {
    let total_assets = total(&input.assets);
    let total_liabilities = total(&input.liabilities);
    let net_worth = total_assets - total_liabilities;
    let by_asset_category = by_category(&input.assets);
    let by_liability_category = by_category(&input.liabilities);
    let h = &input.history;
    let n = h.len();
    let (mom_delta_usd, mom_delta_pct) = if n >= 2 {
        let (d, p) = delta(h[n - 1].net_worth_usd, h[n - 2].net_worth_usd);
        (Some(d), p)
    } else {
        (None, None)
    };
    let (yoy_delta_usd, yoy_delta_pct) = if n >= 13 {
        let (d, p) = delta(h[n - 1].net_worth_usd, h[n - 13].net_worth_usd);
        (Some(d), p)
    } else {
        (None, None)
    };
    let status = if net_worth > 0.0 { "positive" } else { "underwater" }.to_string();
    NetWorthReport {
        net_worth_usd: net_worth,
        total_assets_usd: total_assets,
        total_liabilities_usd: total_liabilities,
        by_asset_category,
        by_liability_category,
        mom_delta_usd,
        mom_delta_pct,
        yoy_delta_usd,
        yoy_delta_pct,
        status,
        debt_to_asset_pct: debt_to_asset(total_assets, total_liabilities),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(name: &str, cat: &str, value: f64) -> LineItem {
        LineItem { name: name.into(), category: cat.into(), value_usd: value }
    }

    fn hp(month: &str, nw: f64) -> HistoryPoint {
        HistoryPoint { month: month.into(), net_worth_usd: nw }
    }

    #[test]
    fn total_empty_returns_zero() {
        assert_eq!(total(&[]), 0.0);
    }

    #[test]
    fn total_sums_values() {
        let xs = vec![mk("a", "cash", 100.0), mk("b", "cash", 250.0)];
        assert_eq!(total(&xs), 350.0);
    }

    #[test]
    fn by_category_groups_and_sums() {
        let xs = vec![
            mk("a", "cash", 100.0),
            mk("b", "cash", 200.0),
            mk("c", "stocks", 1000.0),
        ];
        let g = by_category(&xs);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].category, "stocks");
        assert_eq!(g[0].total_usd, 1000.0);
        assert!((g[0].share_pct - (1000.0 / 1300.0 * 100.0)).abs() < 1e-6);
        assert_eq!(g[1].category, "cash");
        assert_eq!(g[1].total_usd, 300.0);
    }

    #[test]
    fn by_category_zero_total_safe() {
        let g = by_category(&[]);
        assert!(g.is_empty());
    }

    #[test]
    fn delta_pct_zero_prev_is_none() {
        let (d, p) = delta(100.0, 0.0);
        assert_eq!(d, 100.0);
        assert!(p.is_none());
    }

    #[test]
    fn delta_basic_positive() {
        let (d, p) = delta(110.0, 100.0);
        assert_eq!(d, 10.0);
        assert!((p.unwrap() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn debt_to_asset_zero_assets_with_debt() {
        assert_eq!(debt_to_asset(0.0, 1000.0), 100.0);
    }

    #[test]
    fn debt_to_asset_basic() {
        assert_eq!(debt_to_asset(200_000.0, 50_000.0), 25.0);
    }

    #[test]
    fn compute_basic_positive_nw() {
        let r = compute(&NetWorthInput {
            assets: vec![mk("checking", "cash", 5_000.0), mk("brokerage", "stocks", 100_000.0)],
            liabilities: vec![mk("mortgage", "loan", 250_000.0)],
            history: vec![],
        });
        assert_eq!(r.total_assets_usd, 105_000.0);
        assert_eq!(r.total_liabilities_usd, 250_000.0);
        assert_eq!(r.net_worth_usd, -145_000.0);
        assert_eq!(r.status, "underwater");
        assert!((r.debt_to_asset_pct - 250_000.0 / 105_000.0 * 100.0).abs() < 1e-6);
        assert_eq!(r.by_asset_category.len(), 2);
        assert_eq!(r.by_liability_category.len(), 1);
    }

    #[test]
    fn compute_history_mom_delta() {
        let r = compute(&NetWorthInput {
            assets: vec![mk("brokerage", "stocks", 110_000.0)],
            liabilities: vec![],
            history: vec![hp("2026-04", 100_000.0), hp("2026-05", 110_000.0)],
        });
        assert_eq!(r.mom_delta_usd, Some(10_000.0));
        assert!((r.mom_delta_pct.unwrap() - 10.0).abs() < 1e-6);
        assert!(r.yoy_delta_usd.is_none());
    }

    #[test]
    fn compute_history_yoy_delta_needs_13_points() {
        let mut hist: Vec<HistoryPoint> = (0..13)
            .map(|i| hp(&format!("m{i}"), 100_000.0 + (i as f64) * 1_000.0))
            .collect();
        // newest = $112k, oldest = $100k
        let last = hist.last().unwrap().net_worth_usd;
        let first = hist.first().unwrap().net_worth_usd;
        let r = compute(&NetWorthInput {
            assets: vec![],
            liabilities: vec![],
            history: std::mem::take(&mut hist),
        });
        assert_eq!(r.yoy_delta_usd, Some(last - first));
        assert!((r.yoy_delta_pct.unwrap() - (last - first) / first * 100.0).abs() < 1e-6);
    }

    #[test]
    fn compute_positive_nw_status() {
        let r = compute(&NetWorthInput {
            assets: vec![mk("brokerage", "stocks", 100_000.0)],
            liabilities: vec![mk("cc", "card", 5_000.0)],
            history: vec![],
        });
        assert_eq!(r.net_worth_usd, 95_000.0);
        assert_eq!(r.status, "positive");
    }
}
