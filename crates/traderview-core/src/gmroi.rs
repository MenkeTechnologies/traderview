//! GMROI — Gross Margin Return On Inventory Investment.
//!
//! The core retail-profitability metric: how many gross-margin dollars each
//! dollar tied up in inventory returns over a period.
//!
//! ```text
//! gross margin   = revenue − COGS
//! GMROI          = gross margin / average inventory (at cost)
//! inventory turns = COGS / average inventory
//! GMROI          = (gross margin / COGS) × inventory turns
//! ```
//!
//! A GMROI above 1.0 means each inventory dollar earns more than a dollar of
//! margin; general-merchandise retail often targets ~3.2.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GmroiInput {
    /// Net sales over the period.
    pub revenue_usd: f64,
    /// Cost of goods sold over the period.
    pub cogs_usd: f64,
    /// Average inventory held, valued at cost.
    pub average_inventory_usd: f64,
    /// Days in the period, for days-of-inventory. Defaults to 365.
    #[serde(default)]
    pub period_days: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GmroiResult {
    /// revenue − COGS.
    pub gross_margin_usd: f64,
    /// Gross margin as a percent of revenue; `None` if revenue ≤ 0.
    pub gross_margin_pct: Option<f64>,
    /// COGS / average inventory; `None` if inventory ≤ 0.
    pub inventory_turnover: Option<f64>,
    /// Days to sell through average inventory; `None` if turnover undefined.
    pub days_inventory: Option<f64>,
    /// Gross margin / average inventory; `None` if inventory ≤ 0.
    pub gmroi: Option<f64>,
}

pub fn analyze(input: &GmroiInput) -> GmroiResult {
    let days = if input.period_days > 0.0 {
        input.period_days
    } else {
        365.0
    };
    let gross_margin = input.revenue_usd - input.cogs_usd;

    let gross_margin_pct = if input.revenue_usd > 0.0 {
        Some(gross_margin / input.revenue_usd * 100.0)
    } else {
        None
    };

    let (inventory_turnover, gmroi, days_inventory) = if input.average_inventory_usd > 0.0 {
        let turns = input.cogs_usd / input.average_inventory_usd;
        let doi = if turns > 0.0 { Some(days / turns) } else { None };
        (
            Some(turns),
            Some(gross_margin / input.average_inventory_usd),
            doi,
        )
    } else {
        (None, None, None)
    };

    GmroiResult {
        gross_margin_usd: gross_margin,
        gross_margin_pct,
        inventory_turnover,
        days_inventory,
        gmroi,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(rev: f64, cogs: f64, inv: f64) -> GmroiResult {
        analyze(&GmroiInput {
            revenue_usd: rev,
            cogs_usd: cogs,
            average_inventory_usd: inv,
            period_days: 365.0,
        })
    }

    #[test]
    fn gross_margin_and_pct() {
        let r = run(1000.0, 600.0, 200.0);
        assert!(close(r.gross_margin_usd, 400.0));
        assert!(close(r.gross_margin_pct.unwrap(), 40.0));
    }

    #[test]
    fn inventory_turnover() {
        let r = run(1000.0, 600.0, 200.0);
        assert!(close(r.inventory_turnover.unwrap(), 3.0));
    }

    #[test]
    fn gmroi_value() {
        // 400 margin / 200 inventory = 2.0.
        let r = run(1000.0, 600.0, 200.0);
        assert!(close(r.gmroi.unwrap(), 2.0));
    }

    #[test]
    fn days_inventory() {
        // 365 / 3 turns = 121.6667 days.
        let r = run(1000.0, 600.0, 200.0);
        assert!(close(r.days_inventory.unwrap(), 365.0 / 3.0));
    }

    #[test]
    fn gmroi_equals_margin_on_cost_times_turns() {
        let r = run(1000.0, 600.0, 200.0);
        let identity = (r.gross_margin_usd / 600.0) * r.inventory_turnover.unwrap();
        assert!(close(r.gmroi.unwrap(), identity));
    }

    #[test]
    fn more_inventory_lowers_gmroi() {
        let lean = run(1000.0, 600.0, 100.0);
        let bloated = run(1000.0, 600.0, 400.0);
        assert!(lean.gmroi.unwrap() > bloated.gmroi.unwrap());
    }

    #[test]
    fn zero_inventory_guards() {
        let r = run(1000.0, 600.0, 0.0);
        assert!(r.gmroi.is_none());
        assert!(r.inventory_turnover.is_none());
        assert!(r.days_inventory.is_none());
        assert!(close(r.gross_margin_usd, 400.0));
    }

    #[test]
    fn zero_revenue_guards_margin_pct() {
        let r = run(0.0, 600.0, 200.0);
        assert!(r.gross_margin_pct.is_none());
        // Negative margin (selling below cost) still flows through GMROI.
        assert!(r.gmroi.unwrap() < 0.0);
    }
}
