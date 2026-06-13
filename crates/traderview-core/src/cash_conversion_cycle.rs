//! Cash conversion cycle — how long a dollar is tied up in operations.
//!
//! The CCC measures the days between paying for inventory and collecting the
//! cash from selling it. Three components:
//!
//!   * **DSO** (Days Sales Outstanding) — how long customers take to pay:
//!     accounts receivable / revenue × period.
//!   * **DIO** (Days Inventory Outstanding) — how long inventory sits before
//!     it sells: inventory / COGS × period.
//!   * **DPO** (Days Payable Outstanding) — how long you take to pay
//!     suppliers: accounts payable / COGS × period.
//!
//! **CCC = DSO + DIO − DPO.** The operating cycle (DSO + DIO) is the time
//! from buying inventory to collecting; subtracting DPO credits the float
//! your suppliers finance. A **negative** CCC is excellent — you collect
//! from customers before you pay suppliers, so growth funds itself (the
//! Amazon/Dell model). Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CccInput {
    pub accounts_receivable_usd: f64,
    pub annual_revenue_usd: f64,
    pub inventory_usd: f64,
    pub annual_cogs_usd: f64,
    pub accounts_payable_usd: f64,
    /// Days in the period the revenue/COGS figures cover (365 for a year).
    pub period_days: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CccResult {
    /// Days Sales Outstanding — average days to collect receivables.
    pub dso_days: f64,
    /// Days Inventory Outstanding — average days inventory is held.
    pub dio_days: f64,
    /// Days Payable Outstanding — average days to pay suppliers.
    pub dpo_days: f64,
    /// DSO + DIO: buy-inventory-to-collect-cash span.
    pub operating_cycle_days: f64,
    /// DSO + DIO − DPO: net days cash is tied up.
    pub cash_conversion_cycle_days: f64,
    /// True when the CCC is negative (suppliers finance the cycle).
    pub self_financing: bool,
}

pub fn analyze(i: &CccInput) -> CccResult {
    let period = if i.period_days > 0.0 { i.period_days } else { 365.0 };

    // Guard zero denominators: an undefined ratio contributes 0 days.
    let dso = if i.annual_revenue_usd > 0.0 {
        i.accounts_receivable_usd / i.annual_revenue_usd * period
    } else {
        0.0
    };
    let dio = if i.annual_cogs_usd > 0.0 {
        i.inventory_usd / i.annual_cogs_usd * period
    } else {
        0.0
    };
    let dpo = if i.annual_cogs_usd > 0.0 {
        i.accounts_payable_usd / i.annual_cogs_usd * period
    } else {
        0.0
    };

    let operating_cycle = dso + dio;
    let ccc = operating_cycle - dpo;

    CccResult {
        dso_days: dso,
        dio_days: dio,
        dpo_days: dpo,
        operating_cycle_days: operating_cycle,
        cash_conversion_cycle_days: ccc,
        self_financing: ccc < 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> CccInput {
        CccInput {
            accounts_receivable_usd: 50_000.0,
            annual_revenue_usd: 365_000.0,
            inventory_usd: 30_000.0,
            annual_cogs_usd: 219_000.0,
            accounts_payable_usd: 24_000.0,
            period_days: 365.0,
        }
    }

    #[test]
    fn dso_dio_dpo_basic() {
        // DSO: 50k/365k×365 = 50; DIO: 30k/219k×365 = 50; DPO: 24k/219k×365 = 40.
        let r = analyze(&base());
        assert!((r.dso_days - 50.0).abs() < 1e-9);
        assert!((r.dio_days - 50.0).abs() < 1e-9);
        assert!((r.dpo_days - 40.0).abs() < 1e-9);
    }

    #[test]
    fn ccc_is_dso_plus_dio_minus_dpo() {
        // 50 + 50 − 40 = 60.
        let r = analyze(&base());
        assert!((r.cash_conversion_cycle_days - 60.0).abs() < 1e-9);
        assert!(!r.self_financing);
    }

    #[test]
    fn operating_cycle_is_dso_plus_dio() {
        let r = analyze(&base());
        assert!((r.operating_cycle_days - 100.0).abs() < 1e-9);
    }

    #[test]
    fn negative_ccc_is_self_financing() {
        // Huge payables (pay suppliers slowly) → DPO swamps the cycle.
        let r = analyze(&CccInput { accounts_payable_usd: 120_000.0, ..base() });
        // DPO = 120k/219k×365 = 200; CCC = 100 − 200 = −100.
        assert!((r.cash_conversion_cycle_days - (-100.0)).abs() < 1e-9);
        assert!(r.self_financing);
    }

    #[test]
    fn custom_period_scales_days() {
        // A 90-day quarter with quarterly revenue/COGS figures.
        let r = analyze(&CccInput {
            annual_revenue_usd: 90_000.0,
            annual_cogs_usd: 54_000.0,
            accounts_receivable_usd: 10_000.0,
            inventory_usd: 6_000.0,
            accounts_payable_usd: 6_000.0,
            period_days: 90.0,
        });
        // DSO 10k/90k×90 = 10; DIO 6k/54k×90 = 10; DPO 6k/54k×90 = 10; CCC = 10.
        assert!((r.dso_days - 10.0).abs() < 1e-9);
        assert!((r.cash_conversion_cycle_days - 10.0).abs() < 1e-9);
    }

    #[test]
    fn zero_revenue_guards_dso() {
        let r = analyze(&CccInput { annual_revenue_usd: 0.0, ..base() });
        assert!(r.dso_days.abs() < 1e-9);
    }

    #[test]
    fn zero_cogs_guards_dio_and_dpo() {
        let r = analyze(&CccInput { annual_cogs_usd: 0.0, ..base() });
        assert!(r.dio_days.abs() < 1e-9);
        assert!(r.dpo_days.abs() < 1e-9);
    }
}
