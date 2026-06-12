//! Vehicle Total Cost of Ownership (TCO) calculator.
//!
//! Multi-year TCO model for a vehicle purchase: sums depreciation,
//! financing interest, fuel, insurance, maintenance, registration/fees,
//! and reduces by residual value at sale. Reports year-by-year + total +
//! cost-per-mile.
//!
//! Inputs:
//!   - purchase_price_usd, down_payment_usd, sales_tax_pct
//!   - apr_pct, loan_term_months
//!   - hold_years          — how long you keep the car
//!   - annual_miles
//!   - mpg                  — fuel economy
//!   - fuel_price_per_gallon_usd
//!   - insurance_annual_usd
//!   - maintenance_annual_usd  (starts low, grows with age — we apply
//!     a 5%/yr inflator)
//!   - registration_annual_usd
//!   - residual_pct_after_hold — assumed sale value as % of MSRP
//!     (typical 5-year used-vehicle retention ~50-60%)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

const MAINTENANCE_INFLATOR_PCT: f64 = 5.0;

#[derive(Debug, Clone, Deserialize)]
pub struct CarTcoInput {
    pub purchase_price_usd: f64,
    #[serde(default)]
    pub down_payment_usd: f64,
    #[serde(default)]
    pub sales_tax_pct: f64,
    #[serde(default)]
    pub apr_pct: f64,
    #[serde(default = "default_term")]
    pub loan_term_months: u32,
    pub hold_years: u32,
    pub annual_miles: u32,
    #[serde(default = "default_mpg")]
    pub mpg: f64,
    #[serde(default = "default_fuel")]
    pub fuel_price_per_gallon_usd: f64,
    #[serde(default = "default_insurance")]
    pub insurance_annual_usd: f64,
    #[serde(default = "default_maintenance")]
    pub maintenance_annual_usd: f64,
    #[serde(default = "default_registration")]
    pub registration_annual_usd: f64,
    #[serde(default = "default_residual")]
    pub residual_pct_after_hold: f64,
}

fn default_term() -> u32 { 60 }
fn default_mpg() -> f64 { 28.0 }
fn default_fuel() -> f64 { 3.50 }
fn default_insurance() -> f64 { 1500.0 }
fn default_maintenance() -> f64 { 800.0 }
fn default_registration() -> f64 { 200.0 }
fn default_residual() -> f64 { 50.0 }

#[derive(Debug, Clone, Serialize)]
pub struct YearRow {
    pub year: u32,
    pub financing_usd: f64,
    pub fuel_usd: f64,
    pub insurance_usd: f64,
    pub maintenance_usd: f64,
    pub registration_usd: f64,
    pub total_year_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CarTcoReport {
    pub principal_financed_usd: f64,
    pub monthly_payment_usd: f64,
    pub total_financing_interest_usd: f64,
    pub total_fuel_usd: f64,
    pub total_insurance_usd: f64,
    pub total_maintenance_usd: f64,
    pub total_registration_usd: f64,
    pub residual_value_usd: f64,
    pub depreciation_usd: f64,
    pub total_cost_usd: f64,
    pub total_miles: u32,
    pub cost_per_mile_usd: f64,
    pub yearly: Vec<YearRow>,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn monthly_payment(principal: f64, apr_pct: f64, term_months: u32) -> f64 {
    if term_months == 0 || principal <= 0.0 {
        return 0.0;
    }
    let r = apr_pct / 100.0 / 12.0;
    if r.abs() < 1e-12 {
        return principal / term_months as f64;
    }
    let n = term_months as f64;
    principal * r / (1.0 - (1.0 + r).powf(-n))
}

/// Sum of monthly interest charges over the first `months_to_count`
/// months of a standard amortization. Returns 0 for degenerate inputs
/// (non-positive monthly payment, zero principal). Handles rate=0 as a
/// special case (interest is 0 every month).
fn amortized_interest_for_months(
    principal: f64,
    apr_pct: f64,
    monthly: f64,
    months_to_count: u32,
) -> f64 {
    if principal <= 0.0 || monthly <= 0.0 || months_to_count == 0 {
        return 0.0;
    }
    let r = apr_pct / 100.0 / 12.0;
    if r == 0.0 {
        return 0.0;
    }
    let mut balance = principal;
    let mut interest_total = 0.0;
    for _ in 0..months_to_count {
        if balance <= 0.0 {
            break;
        }
        let interest = balance * r;
        let principal_portion = (monthly - interest).max(0.0);
        let paid_principal = principal_portion.min(balance);
        interest_total += interest;
        balance -= paid_principal;
    }
    interest_total
}

pub fn compute(input: &CarTcoInput) -> CarTcoReport {
    let tax = input.purchase_price_usd * input.sales_tax_pct / 100.0;
    let principal = (input.purchase_price_usd + tax - input.down_payment_usd).max(0.0);
    let monthly = monthly_payment(principal, input.apr_pct, input.loan_term_months);
    let hold = input.hold_years.max(1);
    let mut yearly: Vec<YearRow> = Vec::with_capacity(hold as usize);
    let mut total_fuel = 0.0;
    let mut total_insurance = 0.0;
    let mut total_maintenance = 0.0;
    let mut total_registration = 0.0;
    for y in 1..=hold {
        // Financing for the year — sum 12 monthly payments, but only
        // count payments through end of loan term.
        let months_into_year = ((y - 1) * 12 + 12).min(input.loan_term_months);
        let months_prev_year = ((y - 1) * 12).min(input.loan_term_months);
        let financing_year = (months_into_year - months_prev_year) as f64 * monthly;

        let fuel_year = if input.mpg > 0.0 {
            input.annual_miles as f64 / input.mpg * input.fuel_price_per_gallon_usd
        } else { 0.0 };
        let insurance_year = input.insurance_annual_usd;
        let maint_year = input.maintenance_annual_usd
            * (1.0_f64 + MAINTENANCE_INFLATOR_PCT / 100.0).powi((y - 1) as i32);
        let reg_year = input.registration_annual_usd;
        total_fuel += fuel_year;
        total_insurance += insurance_year;
        total_maintenance += maint_year;
        total_registration += reg_year;
        yearly.push(YearRow {
            year: y,
            financing_usd: financing_year,
            fuel_usd: fuel_year,
            insurance_usd: insurance_year,
            maintenance_usd: maint_year,
            registration_usd: reg_year,
            total_year_usd: financing_year + fuel_year + insurance_year + maint_year + reg_year,
        });
    }
    // Real interest paid during the hold = sum of each month's
    // interest portion from a proper amortization schedule, capped at
    // hold*12 months. The previous formula was
    // `(total_payments - principal).max(0)` which silently CLAMPED to
    // 0 whenever hold < loan_term — because by month `hold*12` the
    // borrower has paid only `hold*12 * monthly` < principal, so the
    // subtraction goes negative and gets zeroed. A 3-year hold of a
    // 5-year auto loan reported $0 financing interest under the old
    // math, even though ~3 years of monthly interest accrued.
    let total_financing_interest =
        amortized_interest_for_months(principal, input.apr_pct, monthly, hold * 12);
    let residual = input.purchase_price_usd * input.residual_pct_after_hold / 100.0;
    let depreciation = (input.purchase_price_usd + tax - residual).max(0.0);
    // Total cost = depreciation + interest paid + fuel + insurance + maintenance + registration
    let total_cost = depreciation + total_financing_interest + total_fuel + total_insurance
        + total_maintenance + total_registration;
    let total_miles = input.annual_miles.saturating_mul(hold);
    let cost_per_mile = if total_miles > 0 { total_cost / total_miles as f64 } else { 0.0 };
    CarTcoReport {
        principal_financed_usd: principal,
        monthly_payment_usd: monthly,
        total_financing_interest_usd: total_financing_interest,
        total_fuel_usd: total_fuel,
        total_insurance_usd: total_insurance,
        total_maintenance_usd: total_maintenance,
        total_registration_usd: total_registration,
        residual_value_usd: residual,
        depreciation_usd: depreciation,
        total_cost_usd: total_cost,
        total_miles,
        cost_per_mile_usd: cost_per_mile,
        yearly,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> CarTcoInput {
        CarTcoInput {
            purchase_price_usd: 35_000.0,
            down_payment_usd: 5_000.0,
            sales_tax_pct: 8.0,
            apr_pct: 6.5,
            loan_term_months: 60,
            hold_years: 7,
            annual_miles: 12_000,
            mpg: 28.0,
            fuel_price_per_gallon_usd: 3.50,
            insurance_annual_usd: 1_500.0,
            maintenance_annual_usd: 800.0,
            registration_annual_usd: 200.0,
            residual_pct_after_hold: 30.0,
        }
    }

    #[test]
    fn monthly_payment_known() {
        let p = monthly_payment(30_000.0, 6.5, 60);
        assert!((p - 586.78).abs() < 5.0);
    }

    #[test]
    fn monthly_payment_zero_apr() {
        assert_eq!(monthly_payment(36_000.0, 0.0, 60), 600.0);
    }

    #[test]
    fn compute_yearly_count_matches_hold() {
        let r = compute(&input());
        assert_eq!(r.yearly.len(), 7);
    }

    #[test]
    fn compute_residual_basic() {
        let r = compute(&input());
        // residual = $35k × 0.30 = $10.5k
        assert_eq!(r.residual_value_usd, 10_500.0);
    }

    #[test]
    fn compute_depreciation_includes_tax() {
        let r = compute(&input());
        // depreciation = (35k + 8% tax) − 10.5k = 37.8k − 10.5k = 27.3k
        let expected = 35_000.0 + 2_800.0 - 10_500.0;
        assert!((r.depreciation_usd - expected).abs() < 0.01);
    }

    #[test]
    fn compute_fuel_basic() {
        let r = compute(&input());
        // 12k mi / 28 mpg × $3.50 × 7 years = $10,500
        let expected = 12_000.0 / 28.0 * 3.50 * 7.0;
        assert!((r.total_fuel_usd - expected).abs() < 1.0);
    }

    #[test]
    fn compute_insurance_summed() {
        let r = compute(&input());
        assert_eq!(r.total_insurance_usd, 1_500.0 * 7.0);
    }

    #[test]
    fn compute_maintenance_inflates() {
        let r = compute(&input());
        // First year = 800, last (year 7) = 800 × 1.05^6 ≈ 1072
        assert_eq!(r.yearly[0].maintenance_usd, 800.0);
        let last = r.yearly.last().unwrap().maintenance_usd;
        assert!(last > 800.0);
        assert!((last - 800.0 * 1.05_f64.powi(6)).abs() < 0.5);
    }

    #[test]
    fn compute_financing_in_year_capped_at_term() {
        let r = compute(&input());
        // hold = 7 years (84 months), loan = 60 months. Years 6 + 7 should
        // have zero or partial financing.
        assert!(r.yearly[5].financing_usd >= 0.0);
        assert_eq!(r.yearly[6].financing_usd, 0.0);
    }

    #[test]
    fn compute_zero_mpg_zero_fuel() {
        let mut i = input();
        i.mpg = 0.0;
        let r = compute(&i);
        assert_eq!(r.total_fuel_usd, 0.0);
    }

    #[test]
    fn compute_cost_per_mile_positive() {
        let r = compute(&input());
        assert!(r.cost_per_mile_usd > 0.0);
        assert!(r.cost_per_mile_usd < 10.0); // sanity bound
    }

    #[test]
    fn compute_zero_miles_zero_cpm() {
        let mut i = input();
        i.annual_miles = 0;
        let r = compute(&i);
        assert_eq!(r.cost_per_mile_usd, 0.0);
    }

    #[test]
    fn compute_total_cost_includes_all_components() {
        let r = compute(&input());
        let sum = r.depreciation_usd + r.total_financing_interest_usd + r.total_fuel_usd
            + r.total_insurance_usd + r.total_maintenance_usd + r.total_registration_usd;
        assert!((r.total_cost_usd - sum).abs() < 0.01);
    }
}
