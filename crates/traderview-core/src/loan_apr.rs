//! True loan APR — the all-in annual percentage rate once upfront fees are
//! folded in, the figure lenders must disclose (and the reason APR exceeds the
//! note rate).
//!
//! The payment is set by the note rate on the full loan, but you only receive
//! the loan less fees. The APR is the rate that equates that net amount to the
//! payment stream:
//!
//! ```text
//! net proceeds = payment · (1 − (1+r)^−n)/r
//! APR = 12 · r   (solved for the monthly r)
//! ```

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LoanAprInput {
    pub loan_amount_usd: f64,
    /// Note (stated) interest rate, percent.
    pub note_rate_pct: f64,
    pub term_months: f64,
    /// Upfront fees rolled into the APR (points, origination, etc.).
    #[serde(default)]
    pub fees_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LoanAprResult {
    pub monthly_payment_usd: f64,
    /// Loan amount − fees (what the borrower actually receives).
    pub net_proceeds_usd: f64,
    /// All-in APR, percent.
    pub true_apr_pct: f64,
    /// APR − note rate (the cost of the fees in rate terms).
    pub apr_premium_pct: f64,
    pub total_of_payments_usd: f64,
    pub total_interest_usd: f64,
}

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

fn pv(pmt: f64, r: f64, n: f64) -> f64 {
    if r.abs() < 1e-12 {
        pmt * n
    } else {
        pmt * (1.0 - (1.0 + r).powf(-n)) / r
    }
}

pub fn analyze(input: &LoanAprInput) -> LoanAprResult {
    let i = input.note_rate_pct / 100.0 / 12.0;
    let n = input.term_months;
    let pmt = payment(input.loan_amount_usd, i, n);
    let net = input.loan_amount_usd - input.fees_usd;

    // Solve for the monthly rate where the payment stream's PV equals the net
    // proceeds. PV is monotonically decreasing in r.
    let apr = if pmt > 0.0 && net > 0.0 {
        let (mut lo, mut hi) = (1e-9, 1.0);
        for _ in 0..200 {
            let mid = (lo + hi) / 2.0;
            if pv(pmt, mid, n) > net {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (lo + hi) / 2.0 * 12.0 * 100.0
    } else {
        0.0
    };

    let total = pmt * n;
    LoanAprResult {
        monthly_payment_usd: pmt,
        net_proceeds_usd: net,
        true_apr_pct: apr,
        apr_premium_pct: apr - input.note_rate_pct,
        total_of_payments_usd: total,
        total_interest_usd: total - input.loan_amount_usd,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-3
    }

    fn base() -> LoanAprInput {
        LoanAprInput {
            loan_amount_usd: 200_000.0,
            note_rate_pct: 6.0,
            term_months: 360.0,
            fees_usd: 4_000.0,
        }
    }

    #[test]
    fn payment() {
        assert!(close(analyze(&base()).monthly_payment_usd, 1199.1011));
    }

    #[test]
    fn net_proceeds() {
        assert!(close(analyze(&base()).net_proceeds_usd, 196_000.0));
    }

    #[test]
    fn true_apr_above_note() {
        let r = analyze(&base());
        assert!(close(r.true_apr_pct, 6.189476));
        assert!(r.true_apr_pct > 6.0);
    }

    #[test]
    fn apr_premium() {
        let r = analyze(&base());
        assert!(close(r.apr_premium_pct, r.true_apr_pct - 6.0));
    }

    #[test]
    fn zero_fees_apr_equals_note() {
        let r = analyze(&LoanAprInput {
            fees_usd: 0.0,
            ..base()
        });
        assert!(close(r.true_apr_pct, 6.0));
        assert!(close(r.apr_premium_pct, 0.0));
    }

    #[test]
    fn more_fees_raise_apr() {
        let low = analyze(&base());
        let high = analyze(&LoanAprInput {
            fees_usd: 10_000.0,
            ..base()
        });
        assert!(high.true_apr_pct > low.true_apr_pct);
    }

    #[test]
    fn total_of_payments() {
        let r = analyze(&base());
        assert!(close(r.total_of_payments_usd, r.monthly_payment_usd * 360.0));
    }

    #[test]
    fn total_interest() {
        let r = analyze(&base());
        assert!(close(r.total_interest_usd, r.total_of_payments_usd - 200_000.0));
    }
}
