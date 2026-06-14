//! Rule of 78s (sum-of-the-digits interest rebate) — how a precomputed-interest
//! loan allocates its finance charge and what rebate a borrower gets on early
//! payoff. The total finance charge is spread over the term weighted by the
//! remaining months: month 1 carries n/SOD of the interest, the last month 1/SOD,
//! where SOD = n(n+1)/2. On payoff after m payments the unearned interest
//! (rebate) is `F × (n−m)(n−m+1) / (n(n+1))`. Because it front-loads interest, the
//! borrower earns a smaller rebate than simple pro-rata — the early-payoff
//! penalty this module quantifies. Not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RuleOf78Input {
    /// Total precomputed finance charge over the full term.
    pub total_finance_charge_usd: f64,
    /// Original loan term in months.
    pub original_term_months: u32,
    /// Payments already made at the point of payoff.
    #[serde(default)]
    pub payments_made: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct RuleOf78Report {
    /// Sum of the digits, n(n+1)/2.
    pub sum_of_digits: u32,
    /// Unearned interest refunded on early payoff.
    pub rebate_usd: f64,
    /// Finance charge the lender keeps (total − rebate).
    pub earned_interest_usd: f64,
    /// Pro-rata (straight-line) earned interest for comparison.
    pub straight_line_earned_usd: f64,
    /// Earned − straight-line: the extra cost of the Rule-of-78 front-loading.
    pub early_payoff_penalty_usd: f64,
    pub valid: bool,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &RuleOf78Input) -> RuleOf78Report {
    let n = i.original_term_months;
    if n == 0 || i.payments_made > n {
        return RuleOf78Report::default();
    }
    let m = i.payments_made;
    let sod = n * (n + 1) / 2;
    let rem = n - m;
    let f = i.total_finance_charge_usd;
    // Unearned fraction = remaining-months' digit sum ÷ total digit sum.
    let rebate = f * (rem as f64 * (rem as f64 + 1.0)) / (n as f64 * (n as f64 + 1.0));
    let earned = f - rebate;
    let straight = f * m as f64 / n as f64;
    RuleOf78Report {
        sum_of_digits: sod,
        rebate_usd: cents(rebate),
        earned_interest_usd: cents(earned),
        straight_line_earned_usd: cents(straight),
        early_payoff_penalty_usd: cents(earned - straight),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> RuleOf78Input {
        RuleOf78Input {
            total_finance_charge_usd: 1_200.0,
            original_term_months: 36,
            payments_made: 12,
        }
    }

    #[test]
    fn rebate_and_penalty() {
        let d = generate(&base());
        assert_eq!(d.sum_of_digits, 666);
        assert!(close(d.rebate_usd, 540.54));
        assert!(close(d.earned_interest_usd, 659.46));
        assert!(close(d.straight_line_earned_usd, 400.0));
        assert!(close(d.early_payoff_penalty_usd, 259.46));
    }

    #[test]
    fn full_term_no_rebate() {
        let d = generate(&RuleOf78Input { payments_made: 36, ..base() });
        assert!(close(d.rebate_usd, 0.0));
        assert!(close(d.earned_interest_usd, 1_200.0));
    }

    #[test]
    fn no_payments_full_rebate() {
        let d = generate(&RuleOf78Input { payments_made: 0, ..base() });
        assert!(close(d.rebate_usd, 1_200.0));
        assert!(close(d.earned_interest_usd, 0.0));
    }

    #[test]
    fn front_loaded_penalty_positive_early() {
        // Early payoff always earns the lender more than straight-line.
        let d = generate(&base());
        assert!(d.early_payoff_penalty_usd > 0.0);
    }

    #[test]
    fn invalid_inputs() {
        assert!(!generate(&RuleOf78Input { original_term_months: 0, ..base() }).valid);
        assert!(!generate(&RuleOf78Input { payments_made: 40, ..base() }).valid);
    }
}
