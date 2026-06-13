//! Equivalent annual cost (EAC) — the level yearly cost that has the same present
//! value as owning and running an asset over its life. It is the right way to
//! compare assets with different lifespans (a cheap machine replaced every 3
//! years vs a durable one replaced every 7): annualize each and pick the lower.
//!
//! ```text
//! annuity factor    = (1 − (1+r)^−n) / r
//! PV of costs        = initial + annual operating × AF − salvage·(1+r)^−n
//! EAC                = PV of costs / AF
//!                    = annual operating + (initial − salvage PV) / AF
//! ```
//!
//! The second term is the capital recovery cost — the annualized net cost of
//! buying the asset and recovering its salvage. Distinct from `npv-irr` (project
//! NPV/IRR) and `mirr` (a return measure).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EacInput {
    /// Up-front purchase cost (a positive number).
    pub initial_cost_usd: f64,
    /// Recurring operating cost each year.
    #[serde(default)]
    pub annual_operating_cost_usd: f64,
    /// Salvage value received at the end of the life.
    #[serde(default)]
    pub salvage_value_usd: f64,
    pub discount_rate_pct: f64,
    pub years: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EacResult {
    /// (1 − (1+r)^−n) / r.
    pub annuity_factor: f64,
    /// Present value of the salvage received at the end.
    pub salvage_pv_usd: f64,
    /// Net present value of every cost over the life.
    pub pv_of_costs_usd: f64,
    /// Annualized net capital cost: (initial − salvage PV) / AF.
    pub capital_recovery_usd: f64,
    /// The level equivalent annual cost.
    pub equivalent_annual_cost_usd: Option<f64>,
}

pub fn analyze(input: &EacInput) -> EacResult {
    let r = input.discount_rate_pct / 100.0;
    let n = input.years;

    let af = if n <= 0.0 {
        0.0
    } else if r == 0.0 {
        n
    } else {
        (1.0 - (1.0 + r).powf(-n)) / r
    };

    let discount = if n <= 0.0 {
        1.0
    } else {
        (1.0 + r).powf(-n)
    };
    let salvage_pv = input.salvage_value_usd * discount;

    let pv_costs = input.initial_cost_usd + input.annual_operating_cost_usd * af - salvage_pv;
    let capital_recovery = if af > 0.0 {
        (input.initial_cost_usd - salvage_pv) / af
    } else {
        0.0
    };
    let eac = if af > 0.0 {
        Some(pv_costs / af)
    } else {
        None
    };

    EacResult {
        annuity_factor: af,
        salvage_pv_usd: salvage_pv,
        pv_of_costs_usd: pv_costs,
        capital_recovery_usd: capital_recovery,
        equivalent_annual_cost_usd: eac,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.5
    }

    fn base() -> EacInput {
        EacInput {
            initial_cost_usd: 100_000.0,
            annual_operating_cost_usd: 10_000.0,
            salvage_value_usd: 20_000.0,
            discount_rate_pct: 8.0,
            years: 5.0,
        }
    }

    #[test]
    fn annuity_factor() {
        // (1 − 1.08^−5) / 0.08 = 3.992710.
        assert!((analyze(&base()).annuity_factor - 3.992710).abs() < 1e-4);
    }

    #[test]
    fn salvage_present_value() {
        // 20,000 × 1.08^−5 = 13,611.66.
        assert!(close(analyze(&base()).salvage_pv_usd, 13_611.66));
    }

    #[test]
    fn pv_of_costs() {
        // 100,000 + 10,000 × 3.992710 − 13,611.66 = 126,315.44.
        assert!(close(analyze(&base()).pv_of_costs_usd, 126_315.44));
    }

    #[test]
    fn capital_recovery() {
        // (100,000 − 13,611.66) / 3.992710 = 21,636.52.
        assert!(close(analyze(&base()).capital_recovery_usd, 21_636.52));
    }

    #[test]
    fn equivalent_annual_cost() {
        // 126,315.44 / 3.992710 = 31,636.52.
        assert!(close(analyze(&base()).equivalent_annual_cost_usd.unwrap(), 31_636.52));
    }

    #[test]
    fn eac_identity() {
        // EAC = annual operating + capital recovery.
        let r = analyze(&base());
        assert!(close(
            r.equivalent_annual_cost_usd.unwrap(),
            10_000.0 + r.capital_recovery_usd
        ));
    }

    #[test]
    fn zero_rate_simple_average() {
        let r = analyze(&EacInput {
            discount_rate_pct: 0.0,
            ..base()
        });
        // AF = 5; PV costs = 100,000 + 50,000 − 20,000 = 130,000; EAC = 26,000.
        assert!(close(r.annuity_factor, 5.0));
        assert!(close(r.equivalent_annual_cost_usd.unwrap(), 26_000.0));
    }

    #[test]
    fn no_salvage() {
        let r = analyze(&EacInput {
            salvage_value_usd: 0.0,
            ..base()
        });
        assert!(close(r.salvage_pv_usd, 0.0));
        // EAC = 10,000 + 100,000 / 3.992710 = 35,045.65.
        assert!(close(r.equivalent_annual_cost_usd.unwrap(), 35_045.65));
    }

    #[test]
    fn longer_life_lowers_capital_recovery() {
        let short = analyze(&base());
        let long = analyze(&EacInput {
            years: 10.0,
            ..base()
        });
        // Spreading the purchase over more years lowers the annual capital cost.
        assert!(long.capital_recovery_usd < short.capital_recovery_usd);
    }

    #[test]
    fn zero_years_no_eac() {
        let r = analyze(&EacInput {
            years: 0.0,
            ..base()
        });
        assert!(r.equivalent_annual_cost_usd.is_none());
    }
}
