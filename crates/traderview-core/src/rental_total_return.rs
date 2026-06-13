//! Rental total return — the four ways a leveraged rental builds wealth in a
//! year, decomposed and expressed as a return on cash invested:
//!
//!   1. Cash flow      — NOI less the mortgage debt service.
//!   2. Appreciation   — the property's market-value gain.
//!   3. Loan paydown   — the first-year principal the tenant's rent retires.
//!   4. Tax shield     — depreciation (27.5-yr straight line on the building)
//!                       times the marginal rate, the tax the paper loss saves.
//!
//! Their sum over cash invested (down payment + closing costs) is the total
//! return, far above the cap rate because leverage, appreciation, amortization,
//! and the depreciation deduction all compound on a small cash outlay. Distinct
//! from `real-estate-cap-rate` (unlevered NOI yield) and the cash-on-cash inside
//! `fix-and-flip`.

use serde::{Deserialize, Serialize};

fn d_term() -> f64 {
    30.0
}
fn d_appreciation() -> f64 {
    3.0
}
fn d_tax_rate() -> f64 {
    24.0
}
fn d_land_pct() -> f64 {
    20.0
}
fn d_dep_years() -> f64 {
    27.5
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalReturnInput {
    pub purchase_price_usd: f64,
    pub down_payment_usd: f64,
    #[serde(default)]
    pub closing_costs_usd: f64,
    pub loan_rate_pct: f64,
    #[serde(default = "d_term")]
    pub loan_term_years: f64,
    /// Net operating income before debt service.
    pub annual_noi_usd: f64,
    #[serde(default = "d_appreciation")]
    pub appreciation_rate_pct: f64,
    #[serde(default = "d_tax_rate")]
    pub marginal_tax_rate_pct: f64,
    /// Non-depreciable land share of the price.
    #[serde(default = "d_land_pct")]
    pub land_value_pct: f64,
    #[serde(default = "d_dep_years")]
    pub depreciation_years: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RentalReturnResult {
    pub loan_amount_usd: f64,
    pub annual_debt_service_usd: f64,
    pub cash_invested_usd: f64,
    // The four components, in dollars.
    pub cash_flow_usd: f64,
    pub appreciation_gain_usd: f64,
    pub principal_paydown_usd: f64,
    pub annual_depreciation_usd: f64,
    pub tax_shield_usd: f64,
    pub total_return_usd: f64,
    // Each component as a percent of cash invested. None if no cash invested.
    pub cash_on_cash_pct: Option<f64>,
    pub appreciation_return_pct: Option<f64>,
    pub paydown_return_pct: Option<f64>,
    pub tax_shield_return_pct: Option<f64>,
    pub total_return_pct: Option<f64>,
}

fn monthly_payment(principal: f64, monthly_rate: f64, n_months: f64) -> f64 {
    if principal <= 0.0 {
        return 0.0;
    }
    if monthly_rate == 0.0 {
        return principal / n_months;
    }
    principal * monthly_rate / (1.0 - (1.0 + monthly_rate).powf(-n_months))
}

/// Principal retired over the first 12 months.
fn first_year_principal(principal: f64, monthly_rate: f64, payment: f64) -> f64 {
    let mut balance = principal;
    let mut paid = 0.0;
    for _ in 0..12 {
        let interest = balance * monthly_rate;
        let p = payment - interest;
        paid += p;
        balance -= p;
    }
    paid
}

pub fn analyze(input: &RentalReturnInput) -> RentalReturnResult {
    let loan = (input.purchase_price_usd - input.down_payment_usd).max(0.0);
    let i = input.loan_rate_pct / 100.0 / 12.0;
    let n = input.loan_term_years * 12.0;
    let pmt = monthly_payment(loan, i, n);
    let debt_service = pmt * 12.0;
    let paydown = if loan > 0.0 {
        first_year_principal(loan, i, pmt)
    } else {
        0.0
    };

    let cash_flow = input.annual_noi_usd - debt_service;
    let appreciation = input.purchase_price_usd * input.appreciation_rate_pct / 100.0;
    let depreciable_basis = input.purchase_price_usd * (1.0 - input.land_value_pct / 100.0);
    let depreciation = if input.depreciation_years > 0.0 {
        depreciable_basis / input.depreciation_years
    } else {
        0.0
    };
    let tax_shield = depreciation * input.marginal_tax_rate_pct / 100.0;

    let cash_invested = input.down_payment_usd + input.closing_costs_usd;
    let total = cash_flow + appreciation + paydown + tax_shield;

    let pct = |x: f64| {
        if cash_invested > 0.0 {
            Some(x / cash_invested * 100.0)
        } else {
            None
        }
    };

    RentalReturnResult {
        loan_amount_usd: loan,
        annual_debt_service_usd: debt_service,
        cash_invested_usd: cash_invested,
        cash_flow_usd: cash_flow,
        appreciation_gain_usd: appreciation,
        principal_paydown_usd: paydown,
        annual_depreciation_usd: depreciation,
        tax_shield_usd: tax_shield,
        total_return_usd: total,
        cash_on_cash_pct: pct(cash_flow),
        appreciation_return_pct: pct(appreciation),
        paydown_return_pct: pct(paydown),
        tax_shield_return_pct: pct(tax_shield),
        total_return_pct: pct(total),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.5
    }

    fn base() -> RentalReturnInput {
        RentalReturnInput {
            purchase_price_usd: 300_000.0,
            down_payment_usd: 60_000.0,
            closing_costs_usd: 6_000.0,
            loan_rate_pct: 7.0,
            loan_term_years: 30.0,
            annual_noi_usd: 24_000.0,
            appreciation_rate_pct: 3.0,
            marginal_tax_rate_pct: 24.0,
            land_value_pct: 20.0,
            depreciation_years: 27.5,
        }
    }

    // Expected values independently computed in Python (amortization formula).
    #[test]
    fn loan_and_debt_service() {
        let r = analyze(&base());
        assert!(close(r.loan_amount_usd, 240_000.0));
        // monthly payment 1596.73 → annual 19160.76.
        assert!(close(r.annual_debt_service_usd, 19_160.76));
    }

    #[test]
    fn cash_flow_component() {
        let r = analyze(&base());
        // 24,000 − 19,160.71 = 4,839.29.
        assert!(close(r.cash_flow_usd, 4_839.29));
    }

    #[test]
    fn appreciation_component() {
        let r = analyze(&base());
        // 300,000 × 3% = 9,000.
        assert!(close(r.appreciation_gain_usd, 9_000.0));
    }

    #[test]
    fn principal_paydown_component() {
        let r = analyze(&base());
        // First-year principal on a 240k/7%/30yr loan ≈ 2,437.94.
        assert!(close(r.principal_paydown_usd, 2_437.94));
    }

    #[test]
    fn depreciation_and_tax_shield() {
        let r = analyze(&base());
        // basis 240,000 / 27.5 = 8,727.27; × 24% = 2,094.55.
        assert!(close(r.annual_depreciation_usd, 8_727.27));
        assert!(close(r.tax_shield_usd, 2_094.55));
    }

    #[test]
    fn total_return_sums_components() {
        let r = analyze(&base());
        let sum = r.cash_flow_usd + r.appreciation_gain_usd + r.principal_paydown_usd + r.tax_shield_usd;
        assert!(close(r.total_return_usd, sum));
        // 4,839.29 + 9,000 + 2,437.94 + 2,094.55 ≈ 18,371.78.
        assert!(close(r.total_return_usd, 18_371.78));
    }

    #[test]
    fn total_return_pct_on_cash() {
        let r = analyze(&base());
        // 18,372.74 / 66,000 cash invested ≈ 27.84%.
        assert!((r.total_return_pct.unwrap() - 27.837).abs() < 0.05);
    }

    #[test]
    fn zero_interest_amortizes_evenly() {
        let r = analyze(&RentalReturnInput {
            loan_rate_pct: 0.0,
            ..base()
        });
        // 240,000 / 30 yr = 8,000/yr principal; debt service equals it.
        assert!(close(r.principal_paydown_usd, 8_000.0));
        assert!(close(r.annual_debt_service_usd, 8_000.0));
    }

    #[test]
    fn all_cash_no_loan() {
        let r = analyze(&RentalReturnInput {
            down_payment_usd: 300_000.0,
            ..base()
        });
        assert!(close(r.loan_amount_usd, 0.0));
        assert!(close(r.annual_debt_service_usd, 0.0));
        assert!(close(r.principal_paydown_usd, 0.0));
        // Cash flow is the full NOI with no debt service.
        assert!(close(r.cash_flow_usd, 24_000.0));
    }

    #[test]
    fn no_cash_invested_yields_none() {
        let r = analyze(&RentalReturnInput {
            down_payment_usd: 0.0,
            closing_costs_usd: 0.0,
            ..base()
        });
        assert!(r.total_return_pct.is_none());
    }
}
