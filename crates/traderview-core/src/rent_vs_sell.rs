//! Rent vs sell — keep a property as a rental or sell and invest the proceeds.
//!
//! Compares total wealth at the end of a horizon under each choice, assuming
//! you liquidate at the end so the two are apples-to-apples:
//!
//!   * **Sell now**: net proceeds (value − selling costs − mortgage − capital
//!     gains tax) invested at an alternative return for the horizon.
//!   * **Keep**: the property appreciates and throws off rental cash flow
//!     (rent, growing, minus operating expenses and the mortgage payment);
//!     each year's cash flow is reinvested at the alternative return, and at
//!     the horizon the property is sold (appreciated value − selling costs −
//!     mortgage − capital gains on the larger gain).
//!
//! The mortgage balance is held constant — principal paydown is ignored, so
//! the comparison is slightly conservative for the keep side. Capital gains
//! use a single blended rate on the gain over the adjusted basis. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RentVsSellInput {
    pub current_value_usd: f64,
    /// Adjusted cost basis for capital-gains tax.
    pub cost_basis_usd: f64,
    pub mortgage_balance_usd: f64,
    /// Selling costs (agent + closing) as a percent of sale price.
    pub selling_cost_pct: f64,
    /// Blended capital-gains rate on the gain over basis (fed + state + recapture).
    pub capital_gains_tax_pct: f64,
    pub annual_rent_usd: f64,
    pub annual_operating_expenses_usd: f64,
    /// Annual mortgage payment (P&I); 0 if the property is paid off.
    pub annual_mortgage_payment_usd: f64,
    pub annual_appreciation_pct: f64,
    pub annual_rent_growth_pct: f64,
    /// Return earned on invested proceeds / reinvested cash flows.
    pub alternative_return_pct: f64,
    pub years: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RentVsSellResult {
    /// Net proceeds if sold today.
    pub sell_now_proceeds_usd: f64,
    /// Those proceeds compounded at the alternative return to the horizon.
    pub sell_wealth_usd: f64,
    /// Property value at the horizon after appreciation.
    pub future_value_usd: f64,
    /// Reinvested rental cash flows accumulated to the horizon.
    pub accumulated_cash_flow_usd: f64,
    /// Net proceeds from selling at the horizon.
    pub keep_sale_proceeds_usd: f64,
    /// Keep-path total wealth at the horizon (sale proceeds + cash flows).
    pub keep_wealth_usd: f64,
    /// Keep wealth − sell wealth (positive ⇒ keep wins).
    pub keep_advantage_usd: f64,
    pub keep_wins: bool,
}

fn net_proceeds(value: f64, basis: f64, mortgage: f64, sell_cost_pct: f64, cgt_pct: f64) -> f64 {
    let selling_costs = value * sell_cost_pct / 100.0;
    let gain = (value - basis).max(0.0);
    let cap_gains_tax = gain * cgt_pct / 100.0;
    value - selling_costs - mortgage - cap_gains_tax
}

pub fn analyze(i: &RentVsSellInput) -> RentVsSellResult {
    let alt = i.alternative_return_pct / 100.0;
    let appr = i.annual_appreciation_pct / 100.0;
    let g = i.annual_rent_growth_pct / 100.0;
    let n = i.years;

    // Sell now → invest proceeds.
    let sell_now = net_proceeds(
        i.current_value_usd,
        i.cost_basis_usd,
        i.mortgage_balance_usd,
        i.selling_cost_pct,
        i.capital_gains_tax_pct,
    );
    let sell_wealth = sell_now * (1.0 + alt).powi(n as i32);

    // Keep → appreciate + reinvest cash flows, sell at horizon.
    let future_value = i.current_value_usd * (1.0 + appr).powi(n as i32);
    let mut accumulated_cf = 0.0;
    for year in 1..=n {
        let rent = i.annual_rent_usd * (1.0 + g).powi((year - 1) as i32);
        let cf = rent - i.annual_operating_expenses_usd - i.annual_mortgage_payment_usd;
        // Reinvest each year's cash flow to the horizon.
        accumulated_cf += cf * (1.0 + alt).powi((n - year) as i32);
    }
    let keep_sale = net_proceeds(
        future_value,
        i.cost_basis_usd,
        i.mortgage_balance_usd,
        i.selling_cost_pct,
        i.capital_gains_tax_pct,
    );
    let keep_wealth = keep_sale + accumulated_cf;

    let advantage = keep_wealth - sell_wealth;
    RentVsSellResult {
        sell_now_proceeds_usd: sell_now,
        sell_wealth_usd: sell_wealth,
        future_value_usd: future_value,
        accumulated_cash_flow_usd: accumulated_cf,
        keep_sale_proceeds_usd: keep_sale,
        keep_wealth_usd: keep_wealth,
        keep_advantage_usd: advantage,
        keep_wins: advantage > 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> RentVsSellInput {
        RentVsSellInput {
            current_value_usd: 300_000.0,
            cost_basis_usd: 300_000.0, // no gain
            mortgage_balance_usd: 0.0,
            selling_cost_pct: 6.0,
            capital_gains_tax_pct: 20.0,
            annual_rent_usd: 0.0,
            annual_operating_expenses_usd: 0.0,
            annual_mortgage_payment_usd: 0.0,
            annual_appreciation_pct: 0.0,
            annual_rent_growth_pct: 0.0,
            alternative_return_pct: 0.0,
            years: 5,
        }
    }

    #[test]
    fn flat_no_rent_is_a_wash() {
        // Both paths: 300k − 18k selling = 282k; no growth, no rent → tie.
        let r = analyze(&base());
        assert!((r.sell_wealth_usd - 282_000.0).abs() < 1e-6);
        assert!((r.keep_wealth_usd - 282_000.0).abs() < 1e-6);
        assert!((r.keep_advantage_usd).abs() < 1e-6);
    }

    #[test]
    fn rental_cash_flow_accumulates_and_favors_keep() {
        // Rent 20k − opex 5k = 15k/yr × 5 (alt 0%) = 75k extra for keep.
        let r = analyze(&RentVsSellInput {
            annual_rent_usd: 20_000.0,
            annual_operating_expenses_usd: 5_000.0,
            ..base()
        });
        assert!((r.accumulated_cash_flow_usd - 75_000.0).abs() < 1e-6);
        assert!((r.keep_advantage_usd - 75_000.0).abs() < 1e-6);
        assert!(r.keep_wins);
    }

    #[test]
    fn appreciation_compounds_future_value() {
        // 300k × 1.05^5 = 382,884.46…
        let r = analyze(&RentVsSellInput { annual_appreciation_pct: 5.0, ..base() });
        assert!((r.future_value_usd - 300_000.0 * 1.05_f64.powi(5)).abs() < 1e-6);
    }

    #[test]
    fn capital_gains_reduce_sell_now_proceeds() {
        // basis 200k, value 300k → gain 100k × 20% = 20k tax; 300k − 18k − 20k = 262k.
        let r = analyze(&RentVsSellInput { cost_basis_usd: 200_000.0, ..base() });
        assert!((r.sell_now_proceeds_usd - 262_000.0).abs() < 1e-6);
    }

    #[test]
    fn alternative_return_compounds_sell_proceeds() {
        // 282k × 1.07^10.
        let r = analyze(&RentVsSellInput {
            alternative_return_pct: 7.0,
            years: 10,
            ..base()
        });
        assert!((r.sell_wealth_usd - 282_000.0 * 1.07_f64.powi(10)).abs() < 1e-3);
    }

    #[test]
    fn mortgage_reduces_both_proceeds() {
        let r = analyze(&RentVsSellInput { mortgage_balance_usd: 100_000.0, ..base() });
        // Sell now: 282k − 100k = 182k.
        assert!((r.sell_now_proceeds_usd - 182_000.0).abs() < 1e-6);
        assert!((r.keep_sale_proceeds_usd - 182_000.0).abs() < 1e-6);
    }

    #[test]
    fn rent_growth_compounds_cash_flow() {
        // Rent 10k growing 10%/yr, no opex, alt 0%, 3 yrs: 10k + 11k + 12.1k = 33.1k.
        let r = analyze(&RentVsSellInput {
            annual_rent_usd: 10_000.0,
            annual_rent_growth_pct: 10.0,
            years: 3,
            ..base()
        });
        assert!((r.accumulated_cash_flow_usd - 33_100.0).abs() < 1e-6);
    }

    #[test]
    fn strong_appreciation_plus_rent_keeps_property() {
        let r = analyze(&RentVsSellInput {
            annual_appreciation_pct: 6.0,
            annual_rent_usd: 24_000.0,
            annual_operating_expenses_usd: 8_000.0,
            alternative_return_pct: 5.0,
            years: 10,
            ..base()
        });
        assert!(r.keep_wins);
        assert!(r.keep_advantage_usd > 0.0);
    }
}
