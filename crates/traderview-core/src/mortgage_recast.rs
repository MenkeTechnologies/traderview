//! Mortgage recast — re-amortize the remaining balance after a lump-sum
//! principal payment, keeping the original term. Unlike extra payments (which
//! shorten the term) or a refinance (new rate/term), a recast lowers the
//! monthly payment at the same rate over the same remaining months.
//!
//! ```text
//! new balance = balance − lump sum
//! new payment = amortize(new balance, rate, remaining term)
//! ```
//!
//! The interest saved is the drop in total remaining interest, less the
//! lender's recast fee.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RecastInput {
    pub current_balance_usd: f64,
    pub annual_rate_pct: f64,
    pub remaining_term_months: f64,
    pub lump_sum_usd: f64,
    /// Lender's recast fee (often a few hundred dollars).
    #[serde(default)]
    pub recast_fee_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RecastResult {
    pub old_payment_usd: f64,
    pub new_balance_usd: f64,
    pub new_payment_usd: f64,
    /// old payment − new payment.
    pub monthly_savings_usd: f64,
    pub old_total_interest_usd: f64,
    pub new_total_interest_usd: f64,
    /// Drop in total remaining interest.
    pub interest_saved_usd: f64,
    /// interest saved − recast fee.
    pub net_interest_saved_usd: f64,
}

/// Level payment to amortize `principal` over `n` months at monthly rate `i`.
fn payment(principal: f64, i: f64, n: f64) -> f64 {
    if principal <= 0.0 || n <= 0.0 {
        return 0.0;
    }
    if i.abs() < 1e-12 {
        principal / n
    } else {
        let f = (1.0 + i).powf(n);
        principal * i * f / (f - 1.0)
    }
}

pub fn analyze(input: &RecastInput) -> RecastResult {
    let i = input.annual_rate_pct / 100.0 / 12.0;
    let n = input.remaining_term_months;
    let new_balance = (input.current_balance_usd - input.lump_sum_usd).max(0.0);

    let old_pmt = payment(input.current_balance_usd, i, n);
    let new_pmt = payment(new_balance, i, n);

    let old_total_interest = (old_pmt * n - input.current_balance_usd).max(0.0);
    let new_total_interest = (new_pmt * n - new_balance).max(0.0);
    let interest_saved = old_total_interest - new_total_interest;

    RecastResult {
        old_payment_usd: old_pmt,
        new_balance_usd: new_balance,
        new_payment_usd: new_pmt,
        monthly_savings_usd: old_pmt - new_pmt,
        old_total_interest_usd: old_total_interest,
        new_total_interest_usd: new_total_interest,
        interest_saved_usd: interest_saved,
        net_interest_saved_usd: interest_saved - input.recast_fee_usd,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> RecastInput {
        RecastInput {
            current_balance_usd: 300_000.0,
            annual_rate_pct: 6.0,
            remaining_term_months: 360.0,
            lump_sum_usd: 50_000.0,
            recast_fee_usd: 250.0,
        }
    }

    #[test]
    fn old_payment() {
        assert!(close(analyze(&base()).old_payment_usd, 1798.651575));
    }

    #[test]
    fn new_balance_and_payment() {
        let r = analyze(&base());
        assert!(close(r.new_balance_usd, 250_000.0));
        assert!(close(r.new_payment_usd, 1498.876313));
    }

    #[test]
    fn monthly_savings() {
        assert!(close(analyze(&base()).monthly_savings_usd, 299.775263));
    }

    #[test]
    fn interest_saved() {
        assert!(close(analyze(&base()).interest_saved_usd, 57919.0945));
    }

    #[test]
    fn net_interest_saved_subtracts_fee() {
        let r = analyze(&base());
        assert!(close(r.net_interest_saved_usd, r.interest_saved_usd - 250.0));
    }

    #[test]
    fn zero_lump_no_change() {
        let mut i = base();
        i.lump_sum_usd = 0.0;
        let r = analyze(&i);
        assert!(close(r.monthly_savings_usd, 0.0));
        assert!(close(r.interest_saved_usd, 0.0));
    }

    #[test]
    fn zero_rate_amortizes_linearly() {
        let r = analyze(&RecastInput {
            current_balance_usd: 360_000.0,
            annual_rate_pct: 0.0,
            remaining_term_months: 360.0,
            lump_sum_usd: 60_000.0,
            recast_fee_usd: 0.0,
        });
        assert!(close(r.old_payment_usd, 1000.0)); // 360k / 360
        assert!(close(r.new_payment_usd, 300_000.0 / 360.0));
    }

    #[test]
    fn lump_exceeds_balance_pays_off() {
        let mut i = base();
        i.lump_sum_usd = 400_000.0;
        let r = analyze(&i);
        assert!(close(r.new_balance_usd, 0.0));
        assert!(close(r.new_payment_usd, 0.0));
    }
}
