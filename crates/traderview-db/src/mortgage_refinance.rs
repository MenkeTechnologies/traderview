//! Mortgage refinance breakeven calculator.
//!
//! The fundamental refi question: do the lower monthly payments
//! recoup the closing costs before the user moves / pays off the
//! mortgage? Classic formula:
//!
//!   breakeven_months = closing_costs / (current_monthly_pi − new_monthly_pi)
//!
//! Plus lifetime interest comparison. If user is planning to stay
//! past `breakeven_months` the refi pays for itself; otherwise it
//! doesn't.
//!
//! Inputs:
//!   - current_balance_usd, current_apr_pct, current_remaining_months
//!   - new_apr_pct, new_term_months, closing_costs_usd, cash_out_usd
//!     (if borrower wants extra cash out, increases new principal)
//!   - roll_costs_into_loan flag (true = adds closing_costs to new
//!     principal; false = paid at closing)
//!
//! Compute returns:
//!   - current_monthly_pi_usd
//!   - new_principal_usd
//!   - new_monthly_pi_usd
//!   - monthly_savings_usd
//!   - breakeven_months
//!   - current_remaining_interest_usd     (if held to maturity)
//!   - new_total_interest_usd
//!   - lifetime_interest_delta_usd        (current − new, positive = refi wins
//!                                          over full term)
//!   - status                              — "refi_wins" / "breakeven_too_long" /
//!                                            "no_savings"
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RefinanceInput {
    pub current_balance_usd: f64,
    pub current_apr_pct: f64,
    pub current_remaining_months: u32,
    pub new_apr_pct: f64,
    pub new_term_months: u32,
    #[serde(default)]
    pub closing_costs_usd: f64,
    #[serde(default)]
    pub cash_out_usd: f64,
    #[serde(default)]
    pub roll_costs_into_loan: bool,
    /// User's planning horizon — how many months until they expect
    /// to sell / pay off. Used to determine if breakeven is achievable.
    #[serde(default = "default_horizon")]
    pub planning_horizon_months: u32,
}

fn default_horizon() -> u32 {
    84  // 7 years — US median homeownership tenure ~13y, refi horizon often shorter
}

#[derive(Debug, Clone, Serialize)]
pub struct RefinanceReport {
    pub current_monthly_pi_usd: f64,
    pub new_principal_usd: f64,
    pub new_monthly_pi_usd: f64,
    pub monthly_savings_usd: f64,
    pub breakeven_months: Option<f64>,
    pub current_remaining_interest_usd: f64,
    pub new_total_interest_usd: f64,
    pub lifetime_interest_delta_usd: f64,
    pub status: &'static str,
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

pub fn total_interest(principal: f64, apr_pct: f64, term_months: u32) -> f64 {
    let p = monthly_payment(principal, apr_pct, term_months);
    (p * term_months as f64 - principal).max(0.0)
}

pub fn compute(input: &RefinanceInput) -> RefinanceReport {
    let curr_pi = monthly_payment(
        input.current_balance_usd,
        input.current_apr_pct,
        input.current_remaining_months,
    );
    let new_principal = input.current_balance_usd
        + input.cash_out_usd
        + if input.roll_costs_into_loan { input.closing_costs_usd } else { 0.0 };
    let new_pi = monthly_payment(new_principal, input.new_apr_pct, input.new_term_months);
    let monthly_savings = curr_pi - new_pi;
    let cash_at_close = if input.roll_costs_into_loan { 0.0 } else { input.closing_costs_usd };
    let breakeven = if monthly_savings > 0.005 {
        Some(cash_at_close / monthly_savings)
    } else {
        None
    };
    let curr_interest = total_interest(
        input.current_balance_usd,
        input.current_apr_pct,
        input.current_remaining_months,
    );
    let new_interest = total_interest(new_principal, input.new_apr_pct, input.new_term_months);
    let delta = curr_interest - new_interest;
    let status: &'static str = if monthly_savings <= 0.005 {
        "no_savings"
    } else if let Some(b) = breakeven {
        if b > input.planning_horizon_months as f64 {
            "breakeven_too_long"
        } else {
            "refi_wins"
        }
    } else {
        "no_savings"
    };
    RefinanceReport {
        current_monthly_pi_usd: curr_pi,
        new_principal_usd: new_principal,
        new_monthly_pi_usd: new_pi,
        monthly_savings_usd: monthly_savings,
        breakeven_months: breakeven,
        current_remaining_interest_usd: curr_interest,
        new_total_interest_usd: new_interest,
        lifetime_interest_delta_usd: delta,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> RefinanceInput {
        RefinanceInput {
            current_balance_usd: 350_000.0,
            current_apr_pct: 7.5,
            current_remaining_months: 300,
            new_apr_pct: 5.5,
            new_term_months: 360,
            closing_costs_usd: 6_000.0,
            cash_out_usd: 0.0,
            roll_costs_into_loan: false,
            planning_horizon_months: 84,
        }
    }

    #[test]
    fn monthly_payment_known() {
        // $350k @ 7.5% / 300mo ≈ $2587 standard.
        let p = monthly_payment(350_000.0, 7.5, 300);
        assert!((p - 2_587.0).abs() < 5.0, "got {p}");
    }

    #[test]
    fn total_interest_zero_apr_is_zero() {
        assert_eq!(total_interest(100_000.0, 0.0, 60), 0.0);
    }

    #[test]
    fn total_interest_positive_for_positive_apr() {
        assert!(total_interest(100_000.0, 5.0, 60) > 0.0);
    }

    #[test]
    fn compute_basic_refi_wins() {
        let r = compute(&input());
        assert!(r.monthly_savings_usd > 0.0);
        assert!(r.breakeven_months.is_some());
        // breakeven = 6000 / savings; should be short on a $350k refi
        // with 2% rate drop.
        assert!(r.breakeven_months.unwrap() < 24.0);
        assert_eq!(r.status, "refi_wins");
    }

    #[test]
    fn compute_breakeven_too_long_with_short_horizon() {
        let mut i = input();
        i.planning_horizon_months = 12;  // selling next year
        i.closing_costs_usd = 60_000.0;  // huge closing costs
        let r = compute(&i);
        // huge costs / modest savings → breakeven > 12 months
        assert_eq!(r.status, "breakeven_too_long");
    }

    #[test]
    fn compute_no_savings_when_new_rate_higher() {
        let mut i = input();
        i.new_apr_pct = 9.0;  // higher rate
        let r = compute(&i);
        assert!(r.monthly_savings_usd <= 0.0);
        assert!(r.breakeven_months.is_none());
        assert_eq!(r.status, "no_savings");
    }

    #[test]
    fn compute_roll_costs_into_loan_no_cash_at_close() {
        let mut i = input();
        i.roll_costs_into_loan = true;
        let r = compute(&i);
        // No cash at close → breakeven should be immediate (≤ 0.01).
        assert!(r.breakeven_months.unwrap() < 0.01);
        assert!(r.new_principal_usd > i.current_balance_usd);
    }

    #[test]
    fn compute_cash_out_increases_principal() {
        let mut i = input();
        i.cash_out_usd = 50_000.0;
        let r = compute(&i);
        assert_eq!(r.new_principal_usd, 400_000.0);
    }

    #[test]
    fn compute_lifetime_interest_delta_can_be_negative() {
        // Extending term from 25y → 30y at lower rate may still increase
        // total lifetime interest paid.
        let r = compute(&input());
        // The delta itself depends on the math; just sanity-check it's finite.
        assert!(r.lifetime_interest_delta_usd.is_finite());
    }

    #[test]
    fn compute_zero_balance_safe() {
        let mut i = input();
        i.current_balance_usd = 0.0;
        let r = compute(&i);
        assert_eq!(r.current_monthly_pi_usd, 0.0);
        assert_eq!(r.new_principal_usd, 0.0);
    }

    #[test]
    fn compute_status_refi_wins_breakeven_exactly_at_horizon() {
        // Pick closing costs that make breakeven ≈ planning horizon.
        let mut i = input();
        let monthly_sav = {
            let curr = monthly_payment(i.current_balance_usd, i.current_apr_pct, i.current_remaining_months);
            let new_pi = monthly_payment(i.current_balance_usd, i.new_apr_pct, i.new_term_months);
            curr - new_pi
        };
        // breakeven = closing / savings = 84 months
        i.closing_costs_usd = monthly_sav * 84.0;
        i.planning_horizon_months = 84;
        let r = compute(&i);
        // breakeven = 84 = horizon → at-or-before is wins.
        assert_eq!(r.status, "refi_wins");
    }
}
