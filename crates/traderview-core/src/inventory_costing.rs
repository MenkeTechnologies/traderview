//! Inventory costing (FIFO / LIFO / weighted-average) — given chronological
//! purchase layers and the units sold in the period, it computes cost of goods
//! sold and ending inventory under each of the three cost-flow assumptions. FIFO
//! consumes the oldest layers first; LIFO the newest; weighted-average uses the
//! blended unit cost across all layers. Distinct from the EOQ and GMROI modules,
//! which size and measure inventory rather than cost it. Pure compute, not tax advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Layer {
    /// Units purchased in this layer (chronological order, oldest first).
    pub quantity: f64,
    pub unit_cost_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InventoryInput {
    pub layers: Vec<Layer>,
    pub units_sold: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct MethodResult {
    pub cogs_usd: f64,
    pub ending_inventory_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct InventoryReport {
    pub total_units: f64,
    pub total_cost_usd: f64,
    pub units_sold: f64,
    pub ending_units: f64,
    /// Blended unit cost (total cost ÷ total units).
    pub weighted_avg_unit_cost_usd: f64,
    pub fifo: MethodResult,
    pub lifo: MethodResult,
    pub weighted_average: MethodResult,
    pub valid: bool,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Consume `sold` units from layers (already in the desired consumption order),
/// returning the cost of goods sold.
fn consume(layers: &[(f64, f64)], sold: f64) -> f64 {
    let mut remaining = sold;
    let mut cogs = 0.0;
    for &(q, c) in layers {
        if remaining <= 0.0 {
            break;
        }
        let take = remaining.min(q);
        cogs += take * c;
        remaining -= take;
    }
    cogs
}

pub fn generate(i: &InventoryInput) -> InventoryReport {
    if i.layers.is_empty() {
        return InventoryReport::default();
    }
    let total_units: f64 = i.layers.iter().map(|l| l.quantity).sum();
    let total_cost: f64 = i.layers.iter().map(|l| l.quantity * l.unit_cost_usd).sum();
    if total_units <= 0.0 || i.units_sold < 0.0 || i.units_sold > total_units {
        return InventoryReport { total_units, total_cost_usd: cents(total_cost), valid: false, ..Default::default() };
    }
    let sold = i.units_sold;
    let ending_units = total_units - sold;

    let fwd: Vec<(f64, f64)> = i.layers.iter().map(|l| (l.quantity, l.unit_cost_usd)).collect();
    let mut rev = fwd.clone();
    rev.reverse();

    let fifo_cogs = consume(&fwd, sold);
    let lifo_cogs = consume(&rev, sold);
    let avg = total_cost / total_units;
    let wac_cogs = sold * avg;

    InventoryReport {
        total_units,
        total_cost_usd: cents(total_cost),
        units_sold: sold,
        ending_units,
        weighted_avg_unit_cost_usd: (avg * 10_000.0).round() / 10_000.0,
        fifo: MethodResult { cogs_usd: cents(fifo_cogs), ending_inventory_usd: cents(total_cost - fifo_cogs) },
        lifo: MethodResult { cogs_usd: cents(lifo_cogs), ending_inventory_usd: cents(total_cost - lifo_cogs) },
        weighted_average: MethodResult { cogs_usd: cents(wac_cogs), ending_inventory_usd: cents(ending_units * avg) },
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> InventoryInput {
        InventoryInput {
            layers: vec![
                Layer { quantity: 100.0, unit_cost_usd: 10.0 },
                Layer { quantity: 100.0, unit_cost_usd: 12.0 },
                Layer { quantity: 100.0, unit_cost_usd: 15.0 },
            ],
            units_sold: 150.0,
        }
    }

    #[test]
    fn three_methods() {
        let d = generate(&base());
        assert!(d.valid);
        assert!(close(d.total_cost_usd, 3_700.0));
        assert!(close(d.fifo.cogs_usd, 1_600.0));
        assert!(close(d.fifo.ending_inventory_usd, 2_100.0));
        assert!(close(d.lifo.cogs_usd, 2_100.0));
        assert!(close(d.lifo.ending_inventory_usd, 1_600.0));
        assert!(close(d.weighted_average.cogs_usd, 1_850.0));
        assert!(close(d.weighted_average.ending_inventory_usd, 1_850.0));
        assert!(close(d.weighted_avg_unit_cost_usd, 12.3333));
    }

    #[test]
    fn cogs_plus_ending_equals_total_each_method() {
        let d = generate(&base());
        assert!(close(d.fifo.cogs_usd + d.fifo.ending_inventory_usd, d.total_cost_usd));
        assert!(close(d.lifo.cogs_usd + d.lifo.ending_inventory_usd, d.total_cost_usd));
        assert!(close(d.weighted_average.cogs_usd + d.weighted_average.ending_inventory_usd, d.total_cost_usd));
    }

    #[test]
    fn rising_prices_lifo_higher_cogs() {
        // With rising costs LIFO expenses the dearer units → higher COGS than FIFO.
        let d = generate(&base());
        assert!(d.lifo.cogs_usd > d.fifo.cogs_usd);
    }

    #[test]
    fn sell_everything_zero_ending() {
        let d = generate(&InventoryInput { units_sold: 300.0, ..base() });
        assert!(close(d.fifo.ending_inventory_usd, 0.0));
        assert!(close(d.fifo.cogs_usd, 3_700.0));
    }

    #[test]
    fn oversold_invalid() {
        let d = generate(&InventoryInput { units_sold: 400.0, ..base() });
        assert!(!d.valid);
    }
}
