//! Multi-product break-even — cost-volume-profit analysis when a business sells
//! several products in a fixed sales mix.
//!
//! Each product contributes (price − variable cost) per unit; weighting those by
//! the unit sales mix gives a weighted-average contribution margin per composite
//! unit. Fixed costs over that WACM is the total break-even unit volume, which
//! splits back across the products by their mix proportions.
//!
//! ```text
//! WACM            = Σ (price − variable cost)ᵢ × mix proportionᵢ
//! break-even units = fixed costs / WACM
//! unitsᵢ          = break-even units × mix proportionᵢ
//! ```
//!
//! Distinct from `break-even` (a single product) — the mix is what makes the
//! blended margin, so the answer shifts as the mix shifts.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Product {
    #[serde(default)]
    pub name: String,
    pub price_usd: f64,
    pub variable_cost_usd: f64,
    /// Relative units of this product in the sales mix.
    pub mix_units: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MultiProductInput {
    pub fixed_costs_usd: f64,
    pub products: Vec<Product>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ProductResult {
    pub name: String,
    pub contribution_margin_usd: f64,
    pub mix_proportion_pct: f64,
    pub breakeven_units: Option<f64>,
    pub breakeven_revenue_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MultiProductResult {
    /// Weighted-average contribution margin per composite unit.
    pub weighted_avg_cm_usd: f64,
    pub weighted_avg_price_usd: f64,
    /// WACM / weighted-average price.
    pub weighted_cm_ratio_pct: Option<f64>,
    pub breakeven_units_total: Option<f64>,
    pub breakeven_revenue_usd: Option<f64>,
    pub products: Vec<ProductResult>,
    /// True when the blended margin is positive (a break-even exists).
    pub is_feasible: bool,
}

pub fn analyze(input: &MultiProductInput) -> MultiProductResult {
    let total_mix: f64 = input.products.iter().map(|p| p.mix_units).sum();

    let mut wacm = 0.0;
    let mut wap = 0.0;
    if total_mix > 0.0 {
        for p in &input.products {
            let prop = p.mix_units / total_mix;
            wacm += (p.price_usd - p.variable_cost_usd) * prop;
            wap += p.price_usd * prop;
        }
    }

    let feasible = wacm > 0.0;
    let be_units_total = if feasible {
        Some(input.fixed_costs_usd / wacm)
    } else {
        None
    };

    let products = input
        .products
        .iter()
        .map(|p| {
            let prop = if total_mix > 0.0 {
                p.mix_units / total_mix
            } else {
                0.0
            };
            let units = be_units_total.map(|t| t * prop);
            ProductResult {
                name: p.name.clone(),
                contribution_margin_usd: p.price_usd - p.variable_cost_usd,
                mix_proportion_pct: prop * 100.0,
                breakeven_units: units,
                breakeven_revenue_usd: units.map(|u| u * p.price_usd),
            }
        })
        .collect::<Vec<_>>();

    let be_revenue = if be_units_total.is_some() {
        Some(products.iter().filter_map(|p| p.breakeven_revenue_usd).sum())
    } else {
        None
    };

    MultiProductResult {
        weighted_avg_cm_usd: wacm,
        weighted_avg_price_usd: wap,
        weighted_cm_ratio_pct: if wap > 0.0 {
            Some(wacm / wap * 100.0)
        } else {
            None
        },
        breakeven_units_total: be_units_total,
        breakeven_revenue_usd: be_revenue,
        products,
        is_feasible: feasible,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn two_products() -> MultiProductInput {
        MultiProductInput {
            fixed_costs_usd: 60_000.0,
            products: vec![
                Product { name: "A".into(), price_usd: 100.0, variable_cost_usd: 60.0, mix_units: 3.0 },
                Product { name: "B".into(), price_usd: 50.0, variable_cost_usd: 30.0, mix_units: 2.0 },
            ],
        }
    }

    #[test]
    fn weighted_avg_cm() {
        // 40 × 0.6 + 20 × 0.4 = 32.
        assert!(close(analyze(&two_products()).weighted_avg_cm_usd, 32.0));
    }

    #[test]
    fn weighted_avg_price_and_ratio() {
        let r = analyze(&two_products());
        // 100 × 0.6 + 50 × 0.4 = 80; ratio 32 / 80 = 40%.
        assert!(close(r.weighted_avg_price_usd, 80.0));
        assert!(close(r.weighted_cm_ratio_pct.unwrap(), 40.0));
    }

    #[test]
    fn breakeven_units_total() {
        // 60,000 / 32 = 1,875.
        assert!(close(analyze(&two_products()).breakeven_units_total.unwrap(), 1_875.0));
    }

    #[test]
    fn breakeven_revenue() {
        // 1,875 units split 60/40 → A 1,125 × 100 + B 750 × 50 = 150,000.
        // Cross-check: 60,000 / 0.40 = 150,000.
        assert!(close(analyze(&two_products()).breakeven_revenue_usd.unwrap(), 150_000.0));
    }

    #[test]
    fn per_product_units() {
        let r = analyze(&two_products());
        assert!(close(r.products[0].breakeven_units.unwrap(), 1_125.0));
        assert!(close(r.products[1].breakeven_units.unwrap(), 750.0));
    }

    #[test]
    fn per_product_proportions() {
        let r = analyze(&two_products());
        assert!(close(r.products[0].mix_proportion_pct, 60.0));
        assert!(close(r.products[1].mix_proportion_pct, 40.0));
    }

    #[test]
    fn per_product_contribution_margin() {
        let r = analyze(&two_products());
        assert!(close(r.products[0].contribution_margin_usd, 40.0));
        assert!(close(r.products[1].contribution_margin_usd, 20.0));
    }

    #[test]
    fn single_product_reduces_to_simple() {
        let r = analyze(&MultiProductInput {
            fixed_costs_usd: 40_000.0,
            products: vec![Product {
                name: "Only".into(),
                price_usd: 100.0,
                variable_cost_usd: 60.0,
                mix_units: 1.0,
            }],
        });
        // 40,000 / 40 = 1,000 units.
        assert!(close(r.breakeven_units_total.unwrap(), 1_000.0));
        assert!(close(r.weighted_cm_ratio_pct.unwrap(), 40.0));
    }

    #[test]
    fn infeasible_when_margin_nonpositive() {
        let r = analyze(&MultiProductInput {
            fixed_costs_usd: 60_000.0,
            products: vec![Product {
                name: "Loss".into(),
                price_usd: 50.0,
                variable_cost_usd: 60.0,
                mix_units: 1.0,
            }],
        });
        assert!(!r.is_feasible);
        assert!(r.breakeven_units_total.is_none());
        assert!(r.breakeven_revenue_usd.is_none());
    }

    #[test]
    fn mix_shift_changes_breakeven() {
        // Heavier weight on the high-margin product lowers break-even units.
        let mut heavy = two_products();
        heavy.products[0].mix_units = 8.0; // more of A (cm 40)
        let base_units = analyze(&two_products()).breakeven_units_total.unwrap();
        let heavy_units = analyze(&heavy).breakeven_units_total.unwrap();
        assert!(heavy_units < base_units);
    }
}
