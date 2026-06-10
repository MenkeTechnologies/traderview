//! Electric Vehicle vs Internal Combustion Engine total cost
//! comparison over an N-year ownership horizon.
//!
//! For each path computes purchase + financing + fuel (electricity vs
//! gas) + maintenance + insurance + registration − residual, applies
//! federal/state tax credits (EV only), and reports the winner.
//!
//! Real-world EV cost-per-mile (electricity at home charging) is
//! typically ~25-50% of ICE cost-per-mile. EV maintenance is 30-50%
//! lower (no oil changes, no transmission service, regenerative
//! braking extends brake life). Battery replacement is the wild
//! card; we approximate by including a one-time battery cost at
//! year `battery_replacement_year` (set to 0 to disable).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct VehicleConfig {
    pub purchase_price_usd: f64,
    pub federal_credit_usd: f64,
    pub state_credit_usd: f64,
    /// EV: kWh per 100 miles (efficiency). ICE: ignored.
    pub kwh_per_100mi: f64,
    /// ICE: MPG. EV: ignored.
    pub mpg: f64,
    pub maintenance_annual_usd: f64,
    pub insurance_annual_usd: f64,
    pub registration_annual_usd: f64,
    pub residual_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvVsIceInput {
    pub annual_miles: u32,
    pub hold_years: u32,
    pub apr_pct: f64,
    pub loan_term_months: u32,
    pub sales_tax_pct: f64,
    pub electricity_price_per_kwh_usd: f64,
    pub gasoline_price_per_gallon_usd: f64,

    pub ev: VehicleConfig,
    pub ice: VehicleConfig,

    /// Year (1-indexed) at which the EV battery is replaced.
    /// 0 = no replacement during hold.
    #[serde(default)]
    pub battery_replacement_year: u32,
    #[serde(default)]
    pub battery_replacement_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PathReport {
    pub label: &'static str,
    pub principal_financed_usd: f64,
    pub financing_interest_usd: f64,
    pub fuel_total_usd: f64,
    pub maintenance_total_usd: f64,
    pub insurance_total_usd: f64,
    pub registration_total_usd: f64,
    pub residual_value_usd: f64,
    pub depreciation_usd: f64,
    pub credits_applied_usd: f64,
    pub battery_replacement_usd: f64,
    pub total_cost_usd: f64,
    pub cost_per_mile_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvVsIceReport {
    pub total_miles: u32,
    pub ev: PathReport,
    pub ice: PathReport,
    pub savings_ev_minus_ice_usd: f64,
    pub net_winner: &'static str,
    pub years_to_breakeven: Option<f64>,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn monthly_payment(principal: f64, apr_pct: f64, term_months: u32) -> f64 {
    if term_months == 0 || principal <= 0.0 { return 0.0; }
    let r = apr_pct / 100.0 / 12.0;
    if r.abs() < 1e-12 { return principal / term_months as f64; }
    let n = term_months as f64;
    principal * r / (1.0 - (1.0 + r).powf(-n))
}

pub fn buy_interest_in_horizon(
    principal: f64,
    apr_pct: f64,
    term_months: u32,
    horizon_months: u32,
) -> f64 {
    if principal <= 0.0 || term_months == 0 { return 0.0; }
    let r = apr_pct / 100.0 / 12.0;
    let pi = monthly_payment(principal, apr_pct, term_months);
    let mut bal = principal;
    let mut interest = 0.0;
    let cap = horizon_months.min(term_months);
    for _ in 0..cap {
        let i = bal * r;
        let p = (pi - i).max(0.0).min(bal);
        bal -= p;
        interest += i;
        if bal <= 0.005 { break; }
    }
    interest
}

fn build_path(
    label: &'static str,
    v: &VehicleConfig,
    input: &EvVsIceInput,
    fuel_total: f64,
    credits_applied: f64,
    battery_repl: f64,
) -> PathReport {
    let tax = v.purchase_price_usd * input.sales_tax_pct / 100.0;
    let principal = (v.purchase_price_usd + tax - credits_applied).max(0.0);
    let monthly = monthly_payment(principal, input.apr_pct, input.loan_term_months);
    let _ = monthly;
    let interest = buy_interest_in_horizon(
        principal, input.apr_pct, input.loan_term_months, input.hold_years * 12
    );
    let hold = input.hold_years as f64;
    let maintenance = v.maintenance_annual_usd * hold;
    let insurance = v.insurance_annual_usd * hold;
    let registration = v.registration_annual_usd * hold;
    let residual = v.purchase_price_usd * v.residual_pct / 100.0;
    let depreciation = (v.purchase_price_usd + tax - residual).max(0.0);
    let total = depreciation + interest + fuel_total + maintenance + insurance + registration
        + battery_repl;
    let total_miles = (input.annual_miles as u64 * input.hold_years as u64).max(1);
    let cpm = total / total_miles as f64;
    PathReport {
        label,
        principal_financed_usd: principal,
        financing_interest_usd: interest,
        fuel_total_usd: fuel_total,
        maintenance_total_usd: maintenance,
        insurance_total_usd: insurance,
        registration_total_usd: registration,
        residual_value_usd: residual,
        depreciation_usd: depreciation,
        credits_applied_usd: credits_applied,
        battery_replacement_usd: battery_repl,
        total_cost_usd: total,
        cost_per_mile_usd: cpm,
    }
}

pub fn compute(input: &EvVsIceInput) -> EvVsIceReport {
    let hold = input.hold_years.max(1);
    let total_miles = input.annual_miles.saturating_mul(hold);

    // EV fuel cost = kWh/100mi × miles/100 × $/kWh
    let ev_fuel = if input.ev.kwh_per_100mi > 0.0 {
        input.ev.kwh_per_100mi * total_miles as f64 / 100.0
            * input.electricity_price_per_kwh_usd
    } else { 0.0 };
    // ICE fuel cost = miles / mpg × $/gallon
    let ice_fuel = if input.ice.mpg > 0.0 {
        total_miles as f64 / input.ice.mpg * input.gasoline_price_per_gallon_usd
    } else { 0.0 };

    let ev_credits = input.ev.federal_credit_usd + input.ev.state_credit_usd;
    let ice_credits = input.ice.federal_credit_usd + input.ice.state_credit_usd;
    let battery = if input.battery_replacement_year > 0
        && input.battery_replacement_year <= hold
    {
        input.battery_replacement_cost_usd
    } else { 0.0 };

    let ev = build_path("ev", &input.ev, input, ev_fuel, ev_credits, battery);
    let ice = build_path("ice", &input.ice, input, ice_fuel, ice_credits, 0.0);

    let savings = ice.total_cost_usd - ev.total_cost_usd;
    let winner: &'static str = if ev.total_cost_usd <= ice.total_cost_usd { "ev" } else { "ice" };

    // Years to breakeven: if EV has higher upfront but lower per-year operating,
    // when does cumulative EV cost cross cumulative ICE cost?
    let ev_upfront = ev.depreciation_usd + ev.financing_interest_usd + ev.battery_replacement_usd;
    let ice_upfront = ice.depreciation_usd + ice.financing_interest_usd;
    let ev_annual = (ev.fuel_total_usd + ev.maintenance_total_usd + ev.insurance_total_usd
        + ev.registration_total_usd) / hold as f64;
    let ice_annual = (ice.fuel_total_usd + ice.maintenance_total_usd + ice.insurance_total_usd
        + ice.registration_total_usd) / hold as f64;
    let annual_delta = ice_annual - ev_annual;
    let upfront_delta = ev_upfront - ice_upfront;
    let breakeven = if annual_delta > 1e-6 && upfront_delta > 0.0 {
        Some(upfront_delta / annual_delta)
    } else if upfront_delta <= 0.0 && annual_delta >= 0.0 {
        Some(0.0)  // EV cheaper from day one
    } else {
        None
    };

    EvVsIceReport {
        total_miles,
        ev,
        ice,
        savings_ev_minus_ice_usd: savings,
        net_winner: winner,
        years_to_breakeven: breakeven,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> EvVsIceInput {
        EvVsIceInput {
            annual_miles: 12_000,
            hold_years: 10,
            apr_pct: 6.5,
            loan_term_months: 60,
            sales_tax_pct: 8.0,
            electricity_price_per_kwh_usd: 0.15,
            gasoline_price_per_gallon_usd: 3.50,
            ev: VehicleConfig {
                purchase_price_usd: 45_000.0,
                federal_credit_usd: 7_500.0,
                state_credit_usd: 2_500.0,
                kwh_per_100mi: 28.0,
                mpg: 0.0,
                maintenance_annual_usd: 400.0,
                insurance_annual_usd: 1_400.0,
                registration_annual_usd: 200.0,
                residual_pct: 35.0,
            },
            ice: VehicleConfig {
                purchase_price_usd: 35_000.0,
                federal_credit_usd: 0.0,
                state_credit_usd: 0.0,
                kwh_per_100mi: 0.0,
                mpg: 28.0,
                maintenance_annual_usd: 900.0,
                insurance_annual_usd: 1_500.0,
                registration_annual_usd: 200.0,
                residual_pct: 30.0,
            },
            battery_replacement_year: 0,
            battery_replacement_cost_usd: 0.0,
        }
    }

    #[test]
    fn monthly_payment_known() {
        let p = monthly_payment(30_000.0, 6.5, 60);
        assert!((p - 586.78).abs() < 5.0);
    }

    #[test]
    fn buy_interest_through_full_term() {
        let i = buy_interest_in_horizon(30_000.0, 6.5, 60, 60);
        assert!(i > 4_500.0 && i < 6_000.0);
    }

    #[test]
    fn compute_winner_is_ev_or_ice() {
        let r = compute(&input());
        assert!(r.net_winner == "ev" || r.net_winner == "ice");
    }

    #[test]
    fn compute_ev_credits_reduce_principal() {
        let r = compute(&input());
        // EV: 45k + 8% tax = 48.6k, minus 10k credits = 38.6k principal.
        assert!((r.ev.principal_financed_usd - 38_600.0).abs() < 0.01);
    }

    #[test]
    fn compute_ev_fuel_basic() {
        let r = compute(&input());
        // 28 kWh/100mi × 120k mi / 100 × $0.15 = $5040
        let expected = 28.0 * 120_000.0 / 100.0 * 0.15;
        assert!((r.ev.fuel_total_usd - expected).abs() < 1.0);
    }

    #[test]
    fn compute_ice_fuel_basic() {
        let r = compute(&input());
        // 120k mi / 28 mpg × $3.50 = $15,000
        let expected = 120_000.0 / 28.0 * 3.50;
        assert!((r.ice.fuel_total_usd - expected).abs() < 1.0);
    }

    #[test]
    fn compute_battery_replacement_applied_when_year_in_hold() {
        let mut i = input();
        i.battery_replacement_year = 8;
        i.battery_replacement_cost_usd = 12_000.0;
        let r = compute(&i);
        assert_eq!(r.ev.battery_replacement_usd, 12_000.0);
    }

    #[test]
    fn compute_battery_replacement_zero_when_after_hold() {
        let mut i = input();
        i.battery_replacement_year = 15;
        i.battery_replacement_cost_usd = 12_000.0;
        let r = compute(&i);
        assert_eq!(r.ev.battery_replacement_usd, 0.0);
    }

    #[test]
    fn compute_high_gas_price_ev_wins() {
        let mut i = input();
        i.gasoline_price_per_gallon_usd = 7.00;
        let r = compute(&i);
        assert_eq!(r.net_winner, "ev");
    }

    #[test]
    fn compute_ice_cost_per_mile_higher_than_ev_at_default() {
        let r = compute(&input());
        // EV: 0.15 kWh × 0.28 = $0.042/mi fuel vs ICE: 3.5/28 = $0.125/mi.
        // EV fuel CPM < ICE fuel CPM is the core EV advantage.
        let ev_fuel_cpm = r.ev.fuel_total_usd / (r.total_miles as f64);
        let ice_fuel_cpm = r.ice.fuel_total_usd / (r.total_miles as f64);
        assert!(ev_fuel_cpm < ice_fuel_cpm);
    }

    #[test]
    fn compute_breakeven_zero_when_ev_cheaper_upfront() {
        let mut i = input();
        i.ev.purchase_price_usd = 25_000.0;  // EV cheaper before credits
        let r = compute(&i);
        assert_eq!(r.years_to_breakeven, Some(0.0));
    }

    #[test]
    fn compute_total_cost_includes_all_components() {
        let r = compute(&input());
        let ev_sum = r.ev.depreciation_usd + r.ev.financing_interest_usd + r.ev.fuel_total_usd
            + r.ev.maintenance_total_usd + r.ev.insurance_total_usd + r.ev.registration_total_usd
            + r.ev.battery_replacement_usd;
        assert!((r.ev.total_cost_usd - ev_sum).abs() < 0.01);
    }
}
