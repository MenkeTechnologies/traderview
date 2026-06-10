//! Public Service Loan Forgiveness (PSLF) tracker.
//!
//! PSLF rules (Department of Education, established by the College Cost
//! Reduction and Access Act of 2007):
//!
//!   - 120 qualifying monthly payments under an income-driven repayment
//!     plan (IBR/PAYE/SAVE) — NOT consecutive — while employed by an
//!     eligible public-service / 501(c)(3) employer.
//!   - After the 120th qualifying payment is made AND verified, the
//!     remaining federal loan balance is forgiven, tax-free.
//!
//! Inputs:
//!   - qualifying_payments_made — number of already-confirmed PSLF
//!     payments
//!   - current_balance_usd
//!   - apr_pct
//!   - monthly_payment_usd     — current IDR payment
//!   - currently_eligible_employer — boolean flag
//!
//! Compute returns:
//!   - payments_remaining = max(0, 120 − payments_made)
//!   - months_to_forgiveness = payments_remaining (assuming uninterrupted
//!     qualifying employment)
//!   - years_to_forgiveness = months / 12
//!   - projected_balance_at_forgiveness — month-by-month sim until 120
//!     payments OR balance hits zero (whichever first)
//!   - total_paid_until_forgiveness
//!   - projected_forgiven_balance
//!   - status — "complete" (already 120) / "on_track" (eligible + making
//!     payments) / "ineligible_employer" / "paused" (no payment)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PslfInput {
    pub qualifying_payments_made: u32,
    pub current_balance_usd: f64,
    pub apr_pct: f64,
    pub monthly_payment_usd: f64,
    #[serde(default)]
    pub currently_eligible_employer: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PslfReport {
    pub qualifying_payments_made: u32,
    pub payments_remaining: u32,
    pub months_to_forgiveness: u32,
    pub years_to_forgiveness: f64,
    pub projected_balance_at_forgiveness_usd: f64,
    pub total_paid_until_forgiveness_usd: f64,
    pub projected_forgiven_balance_usd: f64,
    pub status: &'static str,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn project(
    balance: f64,
    apr_pct: f64,
    monthly_payment: f64,
    months: u32,
) -> (f64, f64) {
    if balance <= 0.0 || months == 0 {
        return (balance.max(0.0), 0.0);
    }
    let r = apr_pct / 100.0 / 12.0;
    let mut bal = balance;
    let mut paid = 0.0_f64;
    for _ in 0..months {
        if bal <= 0.005 { bal = 0.0; break; }
        let interest = bal * r;
        if monthly_payment >= interest + bal {
            paid += interest + bal;
            bal = 0.0;
            break;
        } else {
            // Could be positive or negative amortization.
            let net_principal = monthly_payment - interest;
            bal -= net_principal;
            paid += monthly_payment;
        }
    }
    (bal.max(0.0), paid)
}

pub fn compute(input: &PslfInput) -> PslfReport {
    let made = input.qualifying_payments_made.min(120);
    let remaining = 120u32.saturating_sub(made);
    let (proj_bal, paid) = project(
        input.current_balance_usd,
        input.apr_pct,
        input.monthly_payment_usd,
        remaining,
    );
    let forgiven = if remaining == 0 { input.current_balance_usd.max(0.0) } else { proj_bal };
    let status: &'static str = if made >= 120 {
        "complete"
    } else if !input.currently_eligible_employer {
        "ineligible_employer"
    } else if input.monthly_payment_usd <= 0.0 {
        "paused"
    } else {
        "on_track"
    };
    PslfReport {
        qualifying_payments_made: made,
        payments_remaining: remaining,
        months_to_forgiveness: remaining,
        years_to_forgiveness: remaining as f64 / 12.0,
        projected_balance_at_forgiveness_usd: proj_bal,
        total_paid_until_forgiveness_usd: paid,
        projected_forgiven_balance_usd: forgiven,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_zero_balance() {
        let (bal, paid) = project(0.0, 6.0, 200.0, 60);
        assert_eq!(bal, 0.0);
        assert_eq!(paid, 0.0);
    }

    #[test]
    fn project_zero_months() {
        let (bal, paid) = project(10_000.0, 6.0, 200.0, 0);
        assert_eq!(bal, 10_000.0);
        assert_eq!(paid, 0.0);
    }

    #[test]
    fn project_pays_off_when_payment_high() {
        let (bal, _) = project(10_000.0, 5.0, 1000.0, 24);
        assert_eq!(bal, 0.0);
    }

    #[test]
    fn project_negative_amortization_balance_grows() {
        // $50k @ 7% with $100/mo payment over 24 months → balance grows.
        let (bal, _) = project(50_000.0, 7.0, 100.0, 24);
        assert!(bal > 50_000.0);
    }

    #[test]
    fn compute_already_complete() {
        let r = compute(&PslfInput {
            qualifying_payments_made: 130,  // > 120 caps to 120
            current_balance_usd: 5_000.0,
            apr_pct: 6.0,
            monthly_payment_usd: 200.0,
            currently_eligible_employer: true,
        });
        assert_eq!(r.qualifying_payments_made, 120);
        assert_eq!(r.payments_remaining, 0);
        assert_eq!(r.status, "complete");
        // Already complete — projected forgiven balance is current.
        assert_eq!(r.projected_forgiven_balance_usd, 5_000.0);
    }

    #[test]
    fn compute_mid_progress_on_track() {
        let r = compute(&PslfInput {
            qualifying_payments_made: 60,
            current_balance_usd: 40_000.0,
            apr_pct: 6.0,
            monthly_payment_usd: 300.0,
            currently_eligible_employer: true,
        });
        assert_eq!(r.payments_remaining, 60);
        assert_eq!(r.months_to_forgiveness, 60);
        assert_eq!(r.years_to_forgiveness, 5.0);
        assert_eq!(r.status, "on_track");
    }

    #[test]
    fn compute_ineligible_employer() {
        let r = compute(&PslfInput {
            qualifying_payments_made: 10,
            current_balance_usd: 40_000.0,
            apr_pct: 6.0,
            monthly_payment_usd: 300.0,
            currently_eligible_employer: false,
        });
        assert_eq!(r.status, "ineligible_employer");
    }

    #[test]
    fn compute_paused_zero_payment() {
        let r = compute(&PslfInput {
            qualifying_payments_made: 10,
            current_balance_usd: 40_000.0,
            apr_pct: 6.0,
            monthly_payment_usd: 0.0,
            currently_eligible_employer: true,
        });
        assert_eq!(r.status, "paused");
    }

    #[test]
    fn compute_forgiven_balance_grows_under_negative_amortization() {
        let r = compute(&PslfInput {
            qualifying_payments_made: 0,
            current_balance_usd: 100_000.0,
            apr_pct: 7.0,
            monthly_payment_usd: 200.0,  // < interest
            currently_eligible_employer: true,
        });
        // 120 payments at $200 = $24k paid, balance has grown.
        assert!(r.projected_forgiven_balance_usd > 100_000.0);
    }

    #[test]
    fn compute_zero_progress_120_remaining() {
        let r = compute(&PslfInput {
            qualifying_payments_made: 0,
            current_balance_usd: 40_000.0,
            apr_pct: 6.0,
            monthly_payment_usd: 300.0,
            currently_eligible_employer: true,
        });
        assert_eq!(r.payments_remaining, 120);
    }

    #[test]
    fn compute_total_paid_basic() {
        let r = compute(&PslfInput {
            qualifying_payments_made: 60,
            current_balance_usd: 40_000.0,
            apr_pct: 6.0,
            monthly_payment_usd: 300.0,
            currently_eligible_employer: true,
        });
        // 60 more payments × $300 = $18k upper bound.
        // Could be less if balance hits zero early (here it won't).
        assert!(r.total_paid_until_forgiveness_usd > 0.0);
        assert!(r.total_paid_until_forgiveness_usd <= 60.0 * 300.0 + 1.0);
    }
}
