//! Economic order quantity (EOQ) — the Wilson inventory model.
//!
//! How much to order at a time, and when to reorder, to minimize the total
//! of ordering cost and holding cost. Ordering in large batches cuts the
//! number of orders (less ordering cost) but raises average inventory (more
//! holding cost); EOQ is the batch size where the two are balanced:
//!
//!   * **EOQ = √(2 · D · S / H)** — D annual demand (units), S cost per
//!     order, H holding cost per unit per year.
//!   * At EOQ the annual ordering cost (D/EOQ · S) and annual holding cost
//!     (EOQ/2 · H) are equal — that's the minimum total.
//!   * **Reorder point = daily demand × lead time + safety stock** — the
//!     on-hand level that triggers the next order so it arrives before you
//!     run out.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EoqInput {
    /// Annual demand in units (D).
    pub annual_demand_units: f64,
    /// Fixed cost to place one order (S).
    pub ordering_cost_per_order_usd: f64,
    /// Cost to hold one unit for a year (H).
    pub holding_cost_per_unit_year_usd: f64,
    /// Supplier lead time in days.
    pub lead_time_days: f64,
    /// Buffer stock held against demand/lead-time variability.
    #[serde(default)]
    pub safety_stock_units: f64,
    /// Days in the demand period (365 for annual demand).
    pub period_days: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EoqResult {
    /// Optimal order quantity (units).
    pub eoq_units: f64,
    pub orders_per_year: f64,
    pub days_between_orders: f64,
    pub annual_ordering_cost_usd: f64,
    pub annual_holding_cost_usd: f64,
    /// Ordering + holding at the EOQ — the minimized total.
    pub total_annual_cost_usd: f64,
    pub daily_demand_units: f64,
    /// On-hand level that should trigger the next order.
    pub reorder_point_units: f64,
    /// False when EOQ is undefined (holding cost or demand is zero).
    pub feasible: bool,
}

pub fn analyze(i: &EoqInput) -> EoqResult {
    let period = if i.period_days > 0.0 { i.period_days } else { 365.0 };
    let d = i.annual_demand_units.max(0.0);
    let s = i.ordering_cost_per_order_usd.max(0.0);
    let h = i.holding_cost_per_unit_year_usd;

    let daily_demand = d / period;
    let safety = i.safety_stock_units.max(0.0);

    // EOQ needs a positive holding cost and demand.
    if h <= 0.0 || d <= 0.0 {
        let rop = daily_demand * i.lead_time_days + safety;
        return EoqResult {
            eoq_units: 0.0,
            orders_per_year: 0.0,
            days_between_orders: 0.0,
            annual_ordering_cost_usd: 0.0,
            annual_holding_cost_usd: 0.0,
            total_annual_cost_usd: 0.0,
            daily_demand_units: daily_demand,
            reorder_point_units: rop,
            feasible: false,
        };
    }

    let eoq = (2.0 * d * s / h).sqrt();
    let orders = d / eoq;
    let days_between = period / orders;
    let ordering_cost = orders * s;
    let holding_cost = eoq / 2.0 * h;
    let reorder_point = daily_demand * i.lead_time_days + safety;

    EoqResult {
        eoq_units: eoq,
        orders_per_year: orders,
        days_between_orders: days_between,
        annual_ordering_cost_usd: ordering_cost,
        annual_holding_cost_usd: holding_cost,
        total_annual_cost_usd: ordering_cost + holding_cost,
        daily_demand_units: daily_demand,
        reorder_point_units: reorder_point,
        feasible: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> EoqInput {
        EoqInput {
            annual_demand_units: 1_000.0,
            ordering_cost_per_order_usd: 10.0,
            holding_cost_per_unit_year_usd: 2.0,
            lead_time_days: 7.0,
            safety_stock_units: 0.0,
            period_days: 365.0,
        }
    }

    #[test]
    fn eoq_wilson_formula() {
        // √(2·1000·10 / 2) = √10000 = 100.
        let r = analyze(&base());
        assert!((r.eoq_units - 100.0).abs() < 1e-9);
        assert!(r.feasible);
    }

    #[test]
    fn orders_per_year_and_spacing() {
        let r = analyze(&base());
        assert!((r.orders_per_year - 10.0).abs() < 1e-9); // 1000/100
        assert!((r.days_between_orders - 36.5).abs() < 1e-9); // 365/10
    }

    #[test]
    fn ordering_and_holding_equal_at_eoq() {
        // Ordering = 10·10 = 100; holding = (100/2)·2 = 100; equal → total 200.
        let r = analyze(&base());
        assert!((r.annual_ordering_cost_usd - 100.0).abs() < 1e-9);
        assert!((r.annual_holding_cost_usd - 100.0).abs() < 1e-9);
        assert!((r.total_annual_cost_usd - 200.0).abs() < 1e-9);
        assert!((r.annual_ordering_cost_usd - r.annual_holding_cost_usd).abs() < 1e-9);
    }

    #[test]
    fn reorder_point_is_daily_demand_times_lead() {
        // daily = 1000/365 ≈ 2.7397; × 7 ≈ 19.178.
        let r = analyze(&base());
        let expected = 1_000.0 / 365.0 * 7.0;
        assert!((r.reorder_point_units - expected).abs() < 1e-9);
    }

    #[test]
    fn safety_stock_adds_to_reorder_point() {
        let r = analyze(&EoqInput { safety_stock_units: 50.0, ..base() });
        let expected = 1_000.0 / 365.0 * 7.0 + 50.0;
        assert!((r.reorder_point_units - expected).abs() < 1e-9);
    }

    #[test]
    fn zero_holding_cost_is_infeasible_but_keeps_reorder_point() {
        let r = analyze(&EoqInput { holding_cost_per_unit_year_usd: 0.0, ..base() });
        assert!(!r.feasible);
        assert!(r.eoq_units.abs() < 1e-9);
        // Reorder point still computable from demand + lead time.
        assert!(r.reorder_point_units > 0.0);
    }

    #[test]
    fn higher_holding_cost_shrinks_eoq() {
        let cheap = analyze(&base());
        let pricey = analyze(&EoqInput { holding_cost_per_unit_year_usd: 8.0, ..base() });
        assert!(pricey.eoq_units < cheap.eoq_units);
        // H×4 → EOQ ÷2: √(2·1000·10/8)=√2500=50.
        assert!((pricey.eoq_units - 50.0).abs() < 1e-9);
    }

    #[test]
    fn custom_period_scales_daily_demand() {
        // 90-day demand of 900 → daily 10; ROP = 10×7 = 70.
        let r = analyze(&EoqInput {
            annual_demand_units: 900.0,
            period_days: 90.0,
            ..base()
        });
        assert!((r.daily_demand_units - 10.0).abs() < 1e-9);
        assert!((r.reorder_point_units - 70.0).abs() < 1e-9);
    }
}
