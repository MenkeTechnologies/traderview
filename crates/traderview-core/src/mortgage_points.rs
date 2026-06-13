//! Mortgage discount points — break-even on buying down the rate.
//!
//! Paying discount points up front (1 point = 1% of the loan) lowers the
//! interest rate, which lowers the monthly payment. Whether it's worth it
//! comes down to how long you keep the loan: the **break-even** is the
//! months it takes for the payment savings to recoup the points cost.
//!
//!   * bought-down rate = base rate − points × reduction-per-point
//!   * payment uses the standard amortization formula
//!   * break-even months = points cost / monthly payment savings
//!
//! Keep the loan past break-even and the points pay off; sell or refinance
//! before it and you lose money. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PointsInput {
    pub loan_amount_usd: f64,
    pub term_years: f64,
    pub base_rate_pct: f64,
    /// Discount points purchased (1 = 1% of the loan).
    pub points: f64,
    /// Rate reduction per point (commonly ~0.25%).
    pub rate_reduction_per_point_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PointsResult {
    pub bought_down_rate_pct: f64,
    pub points_cost_usd: f64,
    pub base_monthly_payment_usd: f64,
    pub new_monthly_payment_usd: f64,
    pub monthly_savings_usd: f64,
    /// Months to recoup the points cost; `None` if there's no payment savings.
    pub breakeven_months: Option<f64>,
    /// Payment savings over the full term, net of the points cost.
    pub lifetime_net_savings_usd: f64,
}

/// Standard fixed-rate amortized monthly payment.
fn monthly_payment(loan: f64, annual_rate_pct: f64, months: f64) -> f64 {
    if months <= 0.0 {
        return 0.0;
    }
    let r = annual_rate_pct / 100.0 / 12.0;
    if r.abs() < 1e-12 {
        loan / months
    } else {
        let factor = (1.0 + r).powf(months);
        loan * r * factor / (factor - 1.0)
    }
}

pub fn analyze(i: &PointsInput) -> PointsResult {
    let loan = i.loan_amount_usd.max(0.0);
    let months = (i.term_years * 12.0).max(0.0);
    let bought_down = (i.base_rate_pct - i.points * i.rate_reduction_per_point_pct).max(0.0);

    let base_pmt = monthly_payment(loan, i.base_rate_pct, months);
    let new_pmt = monthly_payment(loan, bought_down, months);
    let monthly_savings = base_pmt - new_pmt;
    let points_cost = i.points / 100.0 * loan;

    let breakeven = if monthly_savings > 0.0 {
        Some(points_cost / monthly_savings)
    } else {
        None
    };
    let lifetime_net = monthly_savings * months - points_cost;

    PointsResult {
        bought_down_rate_pct: bought_down,
        points_cost_usd: points_cost,
        base_monthly_payment_usd: base_pmt,
        new_monthly_payment_usd: new_pmt,
        monthly_savings_usd: monthly_savings,
        breakeven_months: breakeven,
        lifetime_net_savings_usd: lifetime_net,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> PointsInput {
        PointsInput {
            loan_amount_usd: 400_000.0,
            term_years: 30.0,
            base_rate_pct: 7.0,
            points: 2.0,
            rate_reduction_per_point_pct: 0.25,
        }
    }

    #[test]
    fn bought_down_rate() {
        // 7.0 − 2 × 0.25 = 6.5%.
        let r = analyze(&base());
        assert!((r.bought_down_rate_pct - 6.5).abs() < 1e-9);
    }

    #[test]
    fn points_cost_is_pct_of_loan() {
        // 2% of 400k = 8,000.
        let r = analyze(&base());
        assert!((r.points_cost_usd - 8_000.0).abs() < 1e-6);
    }

    #[test]
    fn base_payment_matches_amortization() {
        // 400k @ 7% / 30y ≈ $2,661.21.
        let r = analyze(&base());
        assert!((r.base_monthly_payment_usd - 2_661.21).abs() < 0.5);
    }

    #[test]
    fn lower_rate_lowers_payment() {
        let r = analyze(&base());
        assert!(r.new_monthly_payment_usd < r.base_monthly_payment_usd);
        assert!(r.monthly_savings_usd > 0.0);
    }

    #[test]
    fn breakeven_is_cost_over_savings() {
        let r = analyze(&base());
        let expected = r.points_cost_usd / r.monthly_savings_usd;
        assert!((r.breakeven_months.unwrap() - expected).abs() < 1e-6);
    }

    #[test]
    fn zero_points_no_savings_no_breakeven() {
        let r = analyze(&PointsInput { points: 0.0, ..base() });
        assert!(r.points_cost_usd.abs() < 1e-9);
        assert!(r.monthly_savings_usd.abs() < 1e-6);
        assert!(r.breakeven_months.is_none());
    }

    #[test]
    fn lifetime_net_savings_positive_for_full_term() {
        // Held 30 years → total savings far exceeds the 8k cost.
        let r = analyze(&base());
        assert!(r.lifetime_net_savings_usd > 0.0);
        let expected = r.monthly_savings_usd * 360.0 - r.points_cost_usd;
        assert!((r.lifetime_net_savings_usd - expected).abs() < 1e-3);
    }

    #[test]
    fn zero_rate_loan_uses_straight_line_payment() {
        let r = analyze(&PointsInput { base_rate_pct: 0.0, points: 0.0, ..base() });
        // 400k / 360 ≈ 1,111.11.
        assert!((r.base_monthly_payment_usd - 400_000.0 / 360.0).abs() < 1e-6);
    }
}
