//! Credit-card minimum-payment trap — simulates paying only the (declining)
//! minimum versus a fixed payment.
//!
//! The minimum due is `max(floor, percent × balance)`, so it shrinks as the
//! balance falls and most of each payment goes to interest — stretching a small
//! balance into decades. A fixed payment kills it far faster.
//!
//! Month by month: `interest = balance × apr/12`, then the payment less
//! interest reduces principal. If a payment can't cover the interest the debt
//! never amortizes (flagged).

use serde::{Deserialize, Serialize};

const MAX_MONTHS: u32 = 1200; // 100-year cap

#[derive(Debug, Clone, Deserialize)]
pub struct CardInput {
    pub balance_usd: f64,
    pub apr_pct: f64,
    /// Minimum payment as a percent of balance (e.g. 2.0).
    pub min_payment_pct: f64,
    /// Minimum payment dollar floor (e.g. 25.0).
    #[serde(default)]
    pub min_payment_floor_usd: f64,
    /// A fixed monthly payment to compare against (0 to skip).
    #[serde(default)]
    pub fixed_payment_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CardResult {
    /// The first month's minimum payment (for context).
    pub first_minimum_usd: f64,
    /// Months to pay off at the minimum; `None` if it never amortizes.
    pub months_minimum: Option<u32>,
    /// Total interest paid on the minimum path.
    pub total_interest_minimum_usd: f64,
    /// Total paid (balance + interest) on the minimum path.
    pub total_paid_minimum_usd: f64,
    /// Whether the minimum payment fails to ever pay the card off.
    pub never_pays_off: bool,
    /// Months to pay off at the fixed payment; `None` if not supplied/never.
    pub months_fixed: Option<u32>,
    /// Total interest on the fixed path; `None` if not supplied.
    pub total_interest_fixed_usd: Option<f64>,
    /// Interest saved by the fixed payment vs the minimum; `None` if no fixed.
    pub interest_saved_usd: Option<f64>,
}

/// Returns (months, total_interest, paid_off).
fn simulate<F: Fn(f64) -> f64>(balance: f64, monthly_rate: f64, payment_for: F) -> (u32, f64, bool) {
    let mut bal = balance;
    let mut months = 0u32;
    let mut total_interest = 0.0;
    while bal > 0.005 && months < MAX_MONTHS {
        let interest = bal * monthly_rate;
        let pay = payment_for(bal).min(bal + interest);
        if pay <= interest {
            return (months, total_interest, false); // never amortizes
        }
        bal -= pay - interest;
        total_interest += interest;
        months += 1;
    }
    (months, total_interest, bal <= 0.005)
}

pub fn analyze(input: &CardInput) -> CardResult {
    let mr = input.apr_pct / 100.0 / 12.0;
    let pct = input.min_payment_pct / 100.0;
    let floor = input.min_payment_floor_usd;

    let first_minimum = (pct * input.balance_usd).max(floor);

    let (min_months, min_interest, paid_off) =
        simulate(input.balance_usd, mr, |bal| (pct * bal).max(floor));

    let (months_fixed, interest_fixed, interest_saved) = if input.fixed_payment_usd > 0.0 {
        let (fm, fi, fpaid) = simulate(input.balance_usd, mr, |_| input.fixed_payment_usd);
        if fpaid {
            (
                Some(fm),
                Some(fi),
                if paid_off { Some(min_interest - fi) } else { None },
            )
        } else {
            (None, Some(fi), None)
        }
    } else {
        (None, None, None)
    };

    CardResult {
        first_minimum_usd: first_minimum,
        months_minimum: if paid_off { Some(min_months) } else { None },
        total_interest_minimum_usd: min_interest,
        total_paid_minimum_usd: input.balance_usd + min_interest,
        never_pays_off: !paid_off,
        months_fixed,
        total_interest_fixed_usd: interest_fixed,
        interest_saved_usd: interest_saved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> CardInput {
        CardInput {
            balance_usd: 5000.0,
            apr_pct: 22.0,
            min_payment_pct: 2.0,
            min_payment_floor_usd: 25.0,
            fixed_payment_usd: 200.0,
        }
    }

    #[test]
    fn first_minimum() {
        // max(25, 2% × 5000) = 100.
        assert!(close(analyze(&base()).first_minimum_usd, 100.0));
    }

    #[test]
    fn minimum_trap_takes_decades() {
        let r = analyze(&base());
        assert_eq!(r.months_minimum, Some(968));
        assert!(close(r.total_interest_minimum_usd, 43419.4861));
        assert!(!r.never_pays_off);
    }

    #[test]
    fn total_paid_minimum() {
        let r = analyze(&base());
        assert!(close(r.total_paid_minimum_usd, 5000.0 + 43419.4861));
    }

    #[test]
    fn fixed_payment_far_faster() {
        let r = analyze(&base());
        assert_eq!(r.months_fixed, Some(34));
        assert!(close(r.total_interest_fixed_usd.unwrap(), 1749.8795));
    }

    #[test]
    fn interest_saved() {
        let r = analyze(&base());
        assert!(close(r.interest_saved_usd.unwrap(), 43419.4861 - 1749.8795));
    }

    #[test]
    fn no_fixed_comparison_when_zero() {
        let mut i = base();
        i.fixed_payment_usd = 0.0;
        let r = analyze(&i);
        assert!(r.months_fixed.is_none());
        assert!(r.interest_saved_usd.is_none());
    }

    #[test]
    fn floor_drives_small_balance() {
        // Small balance: floor $25 dominates 2% × 300 = $6.
        let r = analyze(&CardInput {
            balance_usd: 300.0,
            apr_pct: 22.0,
            min_payment_pct: 2.0,
            min_payment_floor_usd: 25.0,
            fixed_payment_usd: 0.0,
        });
        assert!(close(r.first_minimum_usd, 25.0));
        assert!(!r.never_pays_off);
    }

    #[test]
    fn never_pays_off_when_no_floor_and_low_percent() {
        // 1% minimum on 22% APR with no floor: minimum < interest forever.
        let r = analyze(&CardInput {
            balance_usd: 5000.0,
            apr_pct: 22.0,
            min_payment_pct: 1.0,
            min_payment_floor_usd: 0.0,
            fixed_payment_usd: 0.0,
        });
        assert!(r.never_pays_off);
        assert!(r.months_minimum.is_none());
    }
}
