//! Treasury Inflation-Protected Securities (TIPS) calculator.
//!
//! TIPS mechanics (US Treasury):
//!   - Principal adjusts with CPI-U semi-annually.
//!   - Fixed REAL coupon rate paid on adjusted principal each 6 months.
//!   - At maturity, the greater of (adjusted principal, original face)
//!     is repaid (deflation floor at par).
//!
//! Inflation supplied as annual percentage; per-period inflation
//! applied semi-annually.
//!
//! Reports per-period principal accretion + coupon, total nominal
//! return, total real return, final principal vs face.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TipsInput {
    pub face_value_usd: f64,
    /// Real coupon rate (annual %). E.g. 1.875 = 1.875%/yr REAL.
    pub real_coupon_rate_pct: f64,
    pub term_years: u32,
    /// Assumed annual CPI inflation (e.g. 2.5 = 2.5%/yr).
    pub annual_cpi_inflation_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TipsPeriodRow {
    pub period: u32,
    pub adjusted_principal_usd: f64,
    pub coupon_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TipsReport {
    pub face_value_usd: f64,
    pub maturity_periods: u32,
    pub final_adjusted_principal_usd: f64,
    pub final_principal_paid_at_maturity_usd: f64,
    pub total_coupons_usd: f64,
    pub total_nominal_return_pct: f64,
    pub total_real_return_pct: f64,
    pub deflation_floor_active: bool,
    pub schedule: Vec<TipsPeriodRow>,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn compute(input: &TipsInput) -> TipsReport {
    let periods = input.term_years * 2;  // semi-annual
    let semi_inflation = input.annual_cpi_inflation_pct / 100.0 / 2.0;
    let real_coupon_per_period = input.real_coupon_rate_pct / 100.0 / 2.0;

    let mut principal = input.face_value_usd;
    let mut total_coupons = 0.0_f64;
    let mut schedule: Vec<TipsPeriodRow> = Vec::with_capacity(periods as usize);
    for p in 1..=periods {
        // Apply inflation to principal first.
        principal *= 1.0 + semi_inflation;
        // Pay coupon on the new adjusted principal.
        let coupon = principal * real_coupon_per_period;
        total_coupons += coupon;
        schedule.push(TipsPeriodRow {
            period: p,
            adjusted_principal_usd: principal,
            coupon_usd: coupon,
        });
    }
    // At maturity: deflation floor — pay back max(adjusted, face).
    let final_paid = principal.max(input.face_value_usd);
    let deflation_floor_active = final_paid > principal + 0.005;

    let total_nominal_return = (final_paid + total_coupons - input.face_value_usd)
        / input.face_value_usd
        * 100.0;
    // Total real return: nominal return minus cumulative inflation drag.
    // Cumulative inflation = (1+annual)^years − 1.
    let cum_inflation_factor =
        (1.0 + input.annual_cpi_inflation_pct / 100.0).powi(input.term_years as i32);
    let total_real_return = (1.0 + total_nominal_return / 100.0) / cum_inflation_factor;
    let total_real_return_pct = (total_real_return - 1.0) * 100.0;

    TipsReport {
        face_value_usd: input.face_value_usd,
        maturity_periods: periods,
        final_adjusted_principal_usd: principal,
        final_principal_paid_at_maturity_usd: final_paid,
        total_coupons_usd: total_coupons,
        total_nominal_return_pct: total_nominal_return,
        total_real_return_pct,
        deflation_floor_active,
        schedule,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> TipsInput {
        TipsInput {
            face_value_usd: 1000.0,
            real_coupon_rate_pct: 1.875,
            term_years: 10,
            annual_cpi_inflation_pct: 2.5,
        }
    }

    #[test]
    fn compute_period_count() {
        let r = compute(&input());
        assert_eq!(r.schedule.len(), 20);
        assert_eq!(r.maturity_periods, 20);
    }

    #[test]
    fn compute_principal_grows_with_inflation() {
        let r = compute(&input());
        // After 10y at 2.5% annual / semi-compound: $1000 × 1.0125^20 ≈ $1282.04
        let last = r.schedule.last().unwrap().adjusted_principal_usd;
        let expected = 1000.0 * 1.0125_f64.powi(20);
        assert!((last - expected).abs() < 0.5);
        assert_eq!(r.final_adjusted_principal_usd, last);
    }

    #[test]
    fn compute_deflation_floor_inactive_when_inflation_positive() {
        let r = compute(&input());
        assert_eq!(r.final_principal_paid_at_maturity_usd, r.final_adjusted_principal_usd);
        assert!(!r.deflation_floor_active);
    }

    #[test]
    fn compute_deflation_floor_active_with_negative_inflation() {
        let mut i = input();
        i.annual_cpi_inflation_pct = -2.0;
        let r = compute(&i);
        assert_eq!(r.final_principal_paid_at_maturity_usd, r.face_value_usd);
        assert!(r.deflation_floor_active);
        assert!(r.final_adjusted_principal_usd < r.face_value_usd);
    }

    #[test]
    fn compute_coupons_grow_with_principal() {
        let r = compute(&input());
        let first = r.schedule[0].coupon_usd;
        let last = r.schedule.last().unwrap().coupon_usd;
        assert!(last > first);
    }

    #[test]
    fn compute_zero_inflation_real_equals_nominal() {
        let mut i = input();
        i.annual_cpi_inflation_pct = 0.0;
        let r = compute(&i);
        assert!((r.total_nominal_return_pct - r.total_real_return_pct).abs() < 1e-6);
    }

    #[test]
    fn compute_positive_inflation_nominal_above_real() {
        let r = compute(&input());
        assert!(r.total_nominal_return_pct > r.total_real_return_pct);
    }

    #[test]
    fn compute_real_return_approx_matches_real_coupon_over_term() {
        let mut i = input();
        i.annual_cpi_inflation_pct = 2.5;
        let r = compute(&i);
        // Real return over 10 years at 1.875% real coupon ≈ ~20% real.
        // (1.0094^20 ≈ 1.205, ~20.5% real)
        assert!(r.total_real_return_pct > 15.0 && r.total_real_return_pct < 26.0,
            "got {}", r.total_real_return_pct);
    }

    #[test]
    fn compute_zero_real_coupon_pure_inflation_protection() {
        let mut i = input();
        i.real_coupon_rate_pct = 0.0;
        let r = compute(&i);
        assert_eq!(r.total_coupons_usd, 0.0);
        // Total nominal return = principal accretion only.
        let expected_nominal = (r.final_adjusted_principal_usd - i.face_value_usd)
            / i.face_value_usd * 100.0;
        assert!((r.total_nominal_return_pct - expected_nominal).abs() < 1e-6);
    }

    #[test]
    fn compute_total_coupons_sum_matches_schedule() {
        let r = compute(&input());
        let sum: f64 = r.schedule.iter().map(|p| p.coupon_usd).sum();
        assert!((r.total_coupons_usd - sum).abs() < 0.001);
    }
}
