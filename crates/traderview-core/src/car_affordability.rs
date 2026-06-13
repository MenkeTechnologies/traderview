//! Car affordability — the 20/4/10 rule of thumb: put 20% down, finance for no
//! more than 4 years, and keep total vehicle spending under 10% of gross income.
//!
//! Working backward from the income cap to a price:
//!
//! ```text
//! monthly budget = 10% × gross monthly income − insurance/fuel
//! max loan       = present value of that payment over the term
//! max price      = max loan / (1 − down payment %)
//! ```

use serde::{Deserialize, Serialize};

fn d_down() -> f64 {
    20.0
}
fn d_term() -> f64 {
    48.0
}
fn d_pct() -> f64 {
    10.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct CarAffordInput {
    pub annual_income_usd: f64,
    #[serde(default = "d_down")]
    pub down_payment_pct: f64,
    #[serde(default = "d_term")]
    pub loan_term_months: f64,
    pub apr_pct: f64,
    /// Cap on monthly vehicle spending as a percent of gross income.
    #[serde(default = "d_pct")]
    pub max_payment_pct_of_income: f64,
    /// Monthly insurance + fuel, carved out of the budget before the payment.
    #[serde(default)]
    pub insurance_fuel_monthly_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CarAffordResult {
    /// 10% of gross monthly income.
    pub monthly_transport_budget_usd: f64,
    /// Budget left for the loan payment after insurance/fuel.
    pub monthly_payment_budget_usd: f64,
    /// Largest loan that payment supports.
    pub max_loan_usd: f64,
    /// Largest car price (loan grossed up for the down payment).
    pub max_car_price_usd: f64,
    /// Down payment required on that price.
    pub down_payment_needed_usd: f64,
}

fn present_value(pmt: f64, i: f64, n: f64) -> f64 {
    if pmt <= 0.0 || n <= 0.0 {
        return 0.0;
    }
    if i.abs() < 1e-12 {
        pmt * n
    } else {
        pmt * (1.0 - (1.0 + i).powf(-n)) / i
    }
}

pub fn analyze(input: &CarAffordInput) -> CarAffordResult {
    let monthly_gross = input.annual_income_usd / 12.0;
    let transport_budget = input.max_payment_pct_of_income / 100.0 * monthly_gross;
    let payment_budget = (transport_budget - input.insurance_fuel_monthly_usd).max(0.0);

    let i = input.apr_pct / 100.0 / 12.0;
    let max_loan = present_value(payment_budget, i, input.loan_term_months);

    let down_frac = (input.down_payment_pct / 100.0).clamp(0.0, 0.99);
    let max_price = max_loan / (1.0 - down_frac);

    CarAffordResult {
        monthly_transport_budget_usd: transport_budget,
        monthly_payment_budget_usd: payment_budget,
        max_loan_usd: max_loan,
        max_car_price_usd: max_price,
        down_payment_needed_usd: max_price * down_frac,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> CarAffordInput {
        CarAffordInput {
            annual_income_usd: 60_000.0,
            down_payment_pct: 20.0,
            loan_term_months: 48.0,
            apr_pct: 6.0,
            max_payment_pct_of_income: 10.0,
            insurance_fuel_monthly_usd: 0.0,
        }
    }

    #[test]
    fn transport_budget() {
        // 10% × 5,000 = 500.
        assert!(close(analyze(&base()).monthly_transport_budget_usd, 500.0));
    }

    #[test]
    fn max_loan() {
        assert!(close(analyze(&base()).max_loan_usd, 21290.1589));
    }

    #[test]
    fn max_price_grosses_up_for_down_payment() {
        assert!(close(analyze(&base()).max_car_price_usd, 26612.6986));
    }

    #[test]
    fn down_payment_needed() {
        let r = analyze(&base());
        assert!(close(r.down_payment_needed_usd, r.max_car_price_usd * 0.20));
    }

    #[test]
    fn insurance_fuel_reduces_budget() {
        let r = analyze(&CarAffordInput {
            insurance_fuel_monthly_usd: 200.0,
            ..base()
        });
        assert!(close(r.monthly_payment_budget_usd, 300.0));
        assert!(r.max_car_price_usd < analyze(&base()).max_car_price_usd);
    }

    #[test]
    fn higher_income_affords_more() {
        let low = analyze(&base());
        let high = analyze(&CarAffordInput {
            annual_income_usd: 120_000.0,
            ..base()
        });
        assert!(high.max_car_price_usd > low.max_car_price_usd);
    }

    #[test]
    fn zero_apr_is_payment_times_term() {
        let r = analyze(&CarAffordInput {
            apr_pct: 0.0,
            ..base()
        });
        // 500 × 48 = 24,000 loan; price = 24,000 / 0.8 = 30,000.
        assert!(close(r.max_loan_usd, 24_000.0));
        assert!(close(r.max_car_price_usd, 30_000.0));
    }

    #[test]
    fn shorter_term_lowers_loan() {
        let short = analyze(&CarAffordInput {
            loan_term_months: 36.0,
            ..base()
        });
        assert!(short.max_loan_usd < analyze(&base()).max_loan_usd);
    }
}
