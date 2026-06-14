//! Reverse mortgage (HECM) estimator. An FHA-insured Home Equity Conversion
//! Mortgage pays the borrower against home equity; the balance compounds and
//! comes due when they leave the home. The cash available keys off a Principal
//! Limit Factor (PLF) — here a linear approximation of HUD's table (older
//! borrowers and lower expected rates → higher PLF) — applied to the max claim
//! (min of home value and the FHA lending limit), net of upfront MIP, the
//! origination fee, closing costs, and any existing mortgage payoff. Projects
//! the loan balance against the appreciating home value to show when a
//! max-draw loan goes "underwater" (non-recourse: heirs never owe the excess).
//! PLF is an approximation — HUD's official table governs a binding quote.
//! Faithful port of the former client-side calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

/// FHA HECM lending limit (2025).
const FHA_MAX_2025: f64 = 1_209_750.0;

#[derive(Debug, Clone, Deserialize)]
pub struct ReverseMortgageInput {
    pub age: u32,
    pub home_value_usd: f64,
    #[serde(default)]
    pub existing_mortgage_usd: f64,
    pub expected_rate_pct: f64,
    pub appreciation_pct: f64,
    /// "lump", "tenure", or "line".
    pub payout: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ReverseRow {
    pub year: u32,
    pub loan_balance_usd: f64,
    pub home_value_usd: f64,
    pub equity_remaining_usd: f64,
    pub underwater: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct ReverseMortgageReport {
    pub max_claim_usd: f64,
    pub plf_pct: f64,
    pub principal_limit_usd: f64,
    pub upfront_mip_usd: f64,
    pub origination_fee_usd: f64,
    pub closing_usd: f64,
    pub total_costs_usd: f64,
    pub net_available_usd: f64,
    /// "lump", "tenure", or "line" (echoed for the view's description).
    pub payout: String,
    /// Tenure only: monthly payment for life, else 0.
    pub monthly_payment_usd: f64,
    /// Tenure only: assumed life expectancy in years (100 − age), else 0.
    pub years_expected: u32,
    pub balance_growth_pct: f64,
    pub rows: Vec<ReverseRow>,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

const PROJECTION_YEARS: [u32; 6] = [1, 5, 10, 15, 20, 25];

pub fn generate(i: &ReverseMortgageInput) -> ReverseMortgageReport {
    if i.home_value_usd <= 0.0 {
        return ReverseMortgageReport::default();
    }
    let age = i.age.max(62);
    let rate = i.expected_rate_pct / 100.0;
    let appr = i.appreciation_pct / 100.0;
    let payout = i.payout.trim().to_ascii_lowercase();

    let max_claim = i.home_value_usd.min(FHA_MAX_2025);
    // Linear PLF approximation: 40% at 62, +1.2pp/yr, less 4pp per point of
    // expected rate above 5%, clamped to [20%, 75%].
    let mut plf = 0.40 + (age.saturating_sub(62) as f64) * 0.012;
    plf -= (rate - 0.05).max(0.0) * 4.0;
    plf = plf.clamp(0.20, 0.75);
    let principal_limit = max_claim * plf;

    let upfront_mip = max_claim * 0.02;
    let orig_fee = 6000.0_f64.min(
        2500.0_f64.max(max_claim.min(200_000.0) * 0.02 + (max_claim - 200_000.0).max(0.0) * 0.01),
    );
    let closing = 4000.0;
    let total_costs = upfront_mip + orig_fee + closing;
    let net_available = principal_limit - i.existing_mortgage_usd - total_costs;

    let mut monthly_payment = 0.0;
    let mut years_expected = 0;
    if payout == "tenure" {
        years_expected = 100u32.saturating_sub(age);
        let r_m = (rate + 0.005) / 12.0;
        let n = (years_expected * 12) as f64;
        monthly_payment = if r_m == 0.0 {
            if n > 0.0 { net_available / n } else { 0.0 }
        } else {
            net_available * r_m / (1.0 - (1.0 + r_m).powf(-n))
        };
    }

    // Max-draw balance projection vs the appreciating home.
    let balance_growth = rate + 0.005;
    let balance0 = i.existing_mortgage_usd + total_costs + if payout == "lump" { net_available } else { 0.0 };
    let mut rows = Vec::new();
    for &yr in PROJECTION_YEARS.iter() {
        let future_balance = balance0 * (1.0 + balance_growth).powi(yr as i32);
        let future_home = i.home_value_usd * (1.0 + appr).powi(yr as i32);
        rows.push(ReverseRow {
            year: yr,
            loan_balance_usd: round2(future_balance),
            home_value_usd: round2(future_home),
            equity_remaining_usd: round2((future_home - future_balance).max(0.0)),
            underwater: future_balance > future_home,
        });
    }

    ReverseMortgageReport {
        max_claim_usd: round2(max_claim),
        plf_pct: round4(plf * 100.0),
        principal_limit_usd: round2(principal_limit),
        upfront_mip_usd: round2(upfront_mip),
        origination_fee_usd: round2(orig_fee),
        closing_usd: round2(closing),
        total_costs_usd: round2(total_costs),
        net_available_usd: round2(net_available),
        payout,
        monthly_payment_usd: round2(monthly_payment),
        years_expected,
        balance_growth_pct: round4(balance_growth * 100.0),
        rows,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> ReverseMortgageInput {
        ReverseMortgageInput {
            age: 70,
            home_value_usd: 650_000.0,
            existing_mortgage_usd: 120_000.0,
            expected_rate_pct: 7.5,
            appreciation_pct: 3.0,
            payout: "lump".into(),
        }
    }

    // Pins cross-checked against the JS compute() in Python.
    #[test]
    fn default_lump() {
        let d = generate(&base());
        assert!(d.valid);
        assert!(close(d.max_claim_usd, 650_000.0));
        assert!(close(d.plf_pct, 39.6));
        assert!(close(d.principal_limit_usd, 257_400.0));
        assert!(close(d.total_costs_usd, 23_000.0));
        assert!(close(d.net_available_usd, 114_400.0));
        assert!(close(d.balance_growth_pct, 8.0));
        assert_eq!(d.rows.len(), 6);
        assert!(close(d.rows[0].loan_balance_usd, 277_992.0));
        assert!(close(d.rows[0].home_value_usd, 669_500.0));
        assert!(!d.rows[0].underwater);
        // Year 20: balance overtakes the home → underwater, zero equity.
        assert!(d.rows[4].underwater);
        assert!(close(d.rows[4].equity_remaining_usd, 0.0));
    }

    #[test]
    fn tenure_payout_monthly() {
        let d = generate(&ReverseMortgageInput { payout: "tenure".into(), ..base() });
        assert_eq!(d.years_expected, 30); // 100 − 70
        assert!(close(d.monthly_payment_usd, 839.43));
        // Non-lump: the net draw is not added to the starting balance.
        assert!(close(d.rows[0].loan_balance_usd, 143_000.0 * 1.08));
    }

    #[test]
    fn higher_rate_lowers_plf() {
        let d = generate(&ReverseMortgageInput { expected_rate_pct: 10.0, ..base() });
        // 0.40 + 8×0.012 = 0.496, less (0.10−0.05)×4 = 0.20 → 0.296.
        assert!(close(d.plf_pct, 29.6));
    }

    #[test]
    fn invalid_when_home_zero() {
        let d = generate(&ReverseMortgageInput { home_value_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
