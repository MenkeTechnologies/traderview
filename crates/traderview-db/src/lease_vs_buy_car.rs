//! Lease vs Buy car NPV comparison.
//!
//! Compares the financial cost-of-use of leasing a vehicle vs buying
//! it. We strip out the components that are identical in both paths
//! (fuel / insurance / registration are equal for the same car) and
//! focus on the contractual differences:
//!
//!   LEASE path total cost over `analysis_years` =
//!     monthly_lease_payment × min(analysis_months, lease_term)
//!     + drive_off_cost
//!     + disposition_fee
//!     − opportunity_return_on_no_downpayment    (lessee keeps cash)
//!     (after lease ends, lessee re-leases at same payment, simplified)
//!
//!   BUY path total cost over `analysis_years` =
//!     depreciation (price + tax − residual_at_horizon)
//!     + financing_interest paid through horizon
//!     + opportunity_cost_on_down_payment       (down × invest_return × years)
//!
//! `monthly_break_even_lease_payment` = solve for the lease payment that
//! makes lease cost = buy cost.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LeaseVsBuyInput {
    pub vehicle_price_usd: f64,
    pub sales_tax_pct: f64,

    // Lease parameters
    pub monthly_lease_payment_usd: f64,
    pub lease_term_months: u32,
    #[serde(default)]
    pub drive_off_cost_usd: f64,
    #[serde(default)]
    pub disposition_fee_usd: f64,

    // Buy parameters
    #[serde(default)]
    pub down_payment_usd: f64,
    pub apr_pct: f64,
    pub loan_term_months: u32,
    /// Sale value as % of MSRP after `analysis_years`.
    pub residual_at_horizon_pct: f64,

    // Shared
    pub analysis_years: u32,
    /// Opportunity cost rate on cash kept (e.g. invested in S&P).
    pub investment_return_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeaseVsBuyReport {
    pub analysis_months: u32,
    pub lease_total_payments_usd: f64,
    pub lease_drive_off_plus_disposition_usd: f64,
    pub lease_opportunity_credit_usd: f64,
    pub lease_total_cost_usd: f64,

    pub buy_principal_usd: f64,
    pub buy_monthly_pi_usd: f64,
    pub buy_total_interest_paid_in_horizon_usd: f64,
    pub buy_residual_value_usd: f64,
    pub buy_depreciation_usd: f64,
    pub buy_down_opportunity_cost_usd: f64,
    pub buy_total_cost_usd: f64,

    pub net_winner: &'static str,
    pub savings_winner_minus_loser_usd: f64,
    pub breakeven_monthly_lease_usd: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn monthly_payment(principal: f64, apr_pct: f64, term_months: u32) -> f64 {
    if term_months == 0 || principal <= 0.0 { return 0.0; }
    let r = apr_pct / 100.0 / 12.0;
    if r.abs() < 1e-12 { return principal / term_months as f64; }
    let n = term_months as f64;
    principal * r / (1.0 - (1.0 + r).powf(-n))
}

pub fn buy_total_interest_in_horizon(
    principal: f64,
    apr_pct: f64,
    term_months: u32,
    horizon_months: u32,
) -> f64 {
    if principal <= 0.0 || term_months == 0 { return 0.0; }
    let r = apr_pct / 100.0 / 12.0;
    let pi = monthly_payment(principal, apr_pct, term_months);
    let mut bal = principal;
    let mut interest_paid = 0.0;
    let cap = horizon_months.min(term_months);
    for _ in 0..cap {
        let interest = bal * r;
        let principal_portion = (pi - interest).max(0.0).min(bal);
        bal -= principal_portion;
        interest_paid += interest;
        if bal <= 0.005 { break; }
    }
    interest_paid
}

pub fn compute(input: &LeaseVsBuyInput) -> LeaseVsBuyReport {
    let analysis_months = (input.analysis_years * 12).max(1);
    let inv_r = input.investment_return_pct / 100.0;
    let analysis_years_f = input.analysis_years.max(1) as f64;

    // — Lease side —
    // Lease over analysis horizon: pay monthly until lease ends, then
    // assume re-leasing at same payment (simplified). So total payments =
    // monthly × analysis_months. Drive-off + disposition apply once per
    // lease term. # of leases = ceil(analysis_months / lease_term).
    let n_leases = if input.lease_term_months == 0 {
        1.0
    } else {
        (analysis_months as f64 / input.lease_term_months as f64).ceil()
    };
    let lease_total_payments = input.monthly_lease_payment_usd * analysis_months as f64;
    let lease_extra_fees = (input.drive_off_cost_usd + input.disposition_fee_usd) * n_leases;
    // Lessee keeps their potential down-payment cash. Simple-interest
    // approximation for the analysis horizon (good enough for a rough
    // opportunity-cost figure).
    let lease_opp_credit = input.down_payment_usd * inv_r * analysis_years_f;
    let lease_total_cost = lease_total_payments + lease_extra_fees - lease_opp_credit;

    // — Buy side —
    let sales_tax = input.vehicle_price_usd * input.sales_tax_pct / 100.0;
    let principal = (input.vehicle_price_usd + sales_tax - input.down_payment_usd).max(0.0);
    let monthly_pi = monthly_payment(principal, input.apr_pct, input.loan_term_months);
    let total_interest_in_horizon = buy_total_interest_in_horizon(
        principal, input.apr_pct, input.loan_term_months, analysis_months
    );
    let residual = input.vehicle_price_usd * input.residual_at_horizon_pct / 100.0;
    let depreciation = (input.vehicle_price_usd + sales_tax - residual).max(0.0);
    let down_opp_cost = input.down_payment_usd * inv_r * analysis_years_f;
    let buy_total_cost = depreciation + total_interest_in_horizon + down_opp_cost;

    let (winner, savings): (&'static str, f64) = if lease_total_cost < buy_total_cost {
        ("lease", buy_total_cost - lease_total_cost)
    } else {
        ("buy", lease_total_cost - buy_total_cost)
    };

    // Breakeven lease payment = (buy_total_cost − lease_extra_fees +
    // lease_opp_credit) / analysis_months
    let breakeven_monthly = (buy_total_cost - lease_extra_fees + lease_opp_credit)
        / analysis_months as f64;

    LeaseVsBuyReport {
        analysis_months,
        lease_total_payments_usd: lease_total_payments,
        lease_drive_off_plus_disposition_usd: lease_extra_fees,
        lease_opportunity_credit_usd: lease_opp_credit,
        lease_total_cost_usd: lease_total_cost,
        buy_principal_usd: principal,
        buy_monthly_pi_usd: monthly_pi,
        buy_total_interest_paid_in_horizon_usd: total_interest_in_horizon,
        buy_residual_value_usd: residual,
        buy_depreciation_usd: depreciation,
        buy_down_opportunity_cost_usd: down_opp_cost,
        buy_total_cost_usd: buy_total_cost,
        net_winner: winner,
        savings_winner_minus_loser_usd: savings,
        breakeven_monthly_lease_usd: breakeven_monthly,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> LeaseVsBuyInput {
        LeaseVsBuyInput {
            vehicle_price_usd: 35_000.0,
            sales_tax_pct: 8.0,
            monthly_lease_payment_usd: 400.0,
            lease_term_months: 36,
            drive_off_cost_usd: 2_500.0,
            disposition_fee_usd: 400.0,
            down_payment_usd: 5_000.0,
            apr_pct: 6.5,
            loan_term_months: 60,
            residual_at_horizon_pct: 40.0,
            analysis_years: 6,
            investment_return_pct: 7.0,
        }
    }

    #[test]
    fn monthly_payment_known() {
        let p = monthly_payment(30_000.0, 6.5, 60);
        assert!((p - 586.78).abs() < 5.0);
    }

    #[test]
    fn buy_interest_zero_when_principal_zero() {
        assert_eq!(buy_total_interest_in_horizon(0.0, 6.5, 60, 60), 0.0);
    }

    #[test]
    fn buy_interest_through_full_term_matches_total() {
        let principal = 30_000.0;
        let interest = buy_total_interest_in_horizon(principal, 6.5, 60, 60);
        // ~$5200 total interest over 5-year auto loan at 6.5%.
        assert!(interest > 4_500.0 && interest < 6_000.0, "got {interest}");
    }

    #[test]
    fn buy_interest_partial_horizon_less_than_full() {
        let full = buy_total_interest_in_horizon(30_000.0, 6.5, 60, 60);
        let partial = buy_total_interest_in_horizon(30_000.0, 6.5, 60, 24);
        assert!(partial < full);
    }

    #[test]
    fn compute_winner_one_of_lease_or_buy() {
        let r = compute(&input());
        assert!(r.net_winner == "lease" || r.net_winner == "buy");
    }

    #[test]
    fn compute_lease_total_includes_repeated_drive_offs() {
        let r = compute(&input());
        // 6 years / 3-year lease = 2 lease cycles → drive_off × 2 + disposition × 2.
        let expected_fees = (2_500.0 + 400.0) * 2.0;
        assert!((r.lease_drive_off_plus_disposition_usd - expected_fees).abs() < 0.01);
    }

    #[test]
    fn compute_high_lease_payment_buy_wins() {
        let mut i = input();
        i.monthly_lease_payment_usd = 800.0;
        let r = compute(&i);
        assert_eq!(r.net_winner, "buy");
    }

    #[test]
    fn compute_low_lease_payment_lease_wins() {
        let mut i = input();
        i.monthly_lease_payment_usd = 150.0;
        let r = compute(&i);
        assert_eq!(r.net_winner, "lease");
    }

    #[test]
    fn compute_breakeven_makes_costs_equal() {
        let r = compute(&input());
        let mut i = input();
        i.monthly_lease_payment_usd = r.breakeven_monthly_lease_usd;
        let r2 = compute(&i);
        assert!((r2.lease_total_cost_usd - r2.buy_total_cost_usd).abs() < 1.0);
    }

    #[test]
    fn compute_buy_principal_includes_tax_minus_down() {
        let r = compute(&input());
        // (35k + 2.8k tax) − 5k down = 32.8k
        assert_eq!(r.buy_principal_usd, 32_800.0);
    }

    #[test]
    fn compute_buy_depreciation_includes_tax() {
        let r = compute(&input());
        // (35k + 2.8k) − (35k × 0.4 = 14k) = 23.8k
        assert!((r.buy_depreciation_usd - 23_800.0).abs() < 0.01);
    }

    #[test]
    fn compute_buy_residual_basic() {
        let r = compute(&input());
        assert_eq!(r.buy_residual_value_usd, 14_000.0);
    }

    #[test]
    fn compute_savings_non_negative() {
        let r = compute(&input());
        assert!(r.savings_winner_minus_loser_usd >= 0.0);
    }
}
