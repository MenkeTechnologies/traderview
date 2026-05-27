//! Cross-account transfer detection.
//!
//! When a credit-card payment shows up in both the bank account (money out)
//! and the credit card account (money in), naively summing transactions
//! double-counts. We detect the pair and mark both `is_transfer = true` so
//! the Schedule C report excludes them.
//!
//! Heuristic, in order of weight:
//!   * Same user.
//!   * Opposite signs.
//!   * |amount_a + amount_b| < 0.01 (the credit-card payment equals the bank debit).
//!   * posted_at within ±3 days.
//!   * Pair spans accounts of different `kind` (typically bank ↔ credit_card).
//!   * Description on either side contains a payment-ish token ("payment",
//!     "pymt", "autopay", "thank you").
//!
//! Returns a list of (id_a, id_b) pairs to mark. Caller does the UPDATE.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DedupCandidate {
    pub id: Uuid,
    pub account_kind: AccountKind,
    pub posted_at: DateTime<Utc>,
    pub amount: Decimal,
    pub description_lower: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    Bank,
    CreditCard,
    Marketplace,
}

pub fn detect_pairs(rows: &[DedupCandidate]) -> Vec<(Uuid, Uuid)> {
    let window = Duration::days(3);
    let mut pairs = Vec::new();
    let mut consumed = vec![false; rows.len()];

    for i in 0..rows.len() {
        if consumed[i] {
            continue;
        }
        let a = &rows[i];
        for j in (i + 1)..rows.len() {
            if consumed[j] {
                continue;
            }
            let b = &rows[j];
            if !is_transfer_pair(a, b, window) {
                continue;
            }
            pairs.push((a.id, b.id));
            consumed[i] = true;
            consumed[j] = true;
            break;
        }
    }
    pairs
}

fn is_transfer_pair(a: &DedupCandidate, b: &DedupCandidate, window: Duration) -> bool {
    // Opposite signs (one positive, one negative). Equal-magnitude check below
    // requires both to be non-zero, so this rejects two zero-amount rows too.
    if (a.amount.is_sign_negative() == b.amount.is_sign_negative()) || a.amount.is_zero()
    {
        return false;
    }

    // Magnitude match within a penny.
    let sum = a.amount + b.amount;
    if sum.abs() > Decimal::new(1, 2) {
        return false;
    }

    // Time window.
    let delta = (a.posted_at - b.posted_at).num_seconds().abs();
    if delta > window.num_seconds() {
        return false;
    }

    // Cross-account-kind preference: bank↔credit_card is the canonical case.
    // Same-kind pairs are still allowed but need a description tell.
    let cross_kind = a.account_kind != b.account_kind;
    let payment_tell = has_payment_tell(&a.description_lower) || has_payment_tell(&b.description_lower);

    cross_kind || payment_tell
}

fn has_payment_tell(s: &str) -> bool {
    s.contains("payment")
        || s.contains("pymt")
        || s.contains("autopay")
        || s.contains("auto pay")
        || s.contains("thank you")
        || s.contains("statement balance")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn cand(kind: AccountKind, day: u32, amount: i64, desc: &str) -> DedupCandidate {
        DedupCandidate {
            id: Uuid::new_v4(),
            account_kind: kind,
            posted_at: Utc.with_ymd_and_hms(2026, 5, day, 12, 0, 0).unwrap(),
            amount: Decimal::new(amount, 2), // cents
            description_lower: desc.to_lowercase(),
        }
    }

    #[test]
    fn matches_bank_to_credit_card() {
        let rows = vec![
            cand(AccountKind::Bank, 15, -43210, "chase epay autopay"),
            cand(AccountKind::CreditCard, 15, 43210, "automatic payment - thank you"),
        ];
        let pairs = detect_pairs(&rows);
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn rejects_same_sign() {
        let rows = vec![
            cand(AccountKind::Bank, 15, -43210, "x"),
            cand(AccountKind::CreditCard, 15, -43210, "y"),
        ];
        assert!(detect_pairs(&rows).is_empty());
    }

    #[test]
    fn rejects_outside_window() {
        let rows = vec![
            cand(AccountKind::Bank, 1, -43210, "payment"),
            cand(AccountKind::CreditCard, 10, 43210, "payment"),
        ];
        assert!(detect_pairs(&rows).is_empty());
    }

    #[test]
    fn rejects_amount_mismatch() {
        let rows = vec![
            cand(AccountKind::Bank, 15, -43210, "payment"),
            cand(AccountKind::CreditCard, 15, 43200, "payment"),
        ];
        assert!(detect_pairs(&rows).is_empty());
    }

    #[test]
    fn rejects_zero_amount() {
        let rows = vec![
            cand(AccountKind::Bank, 15, 0, "payment"),
            cand(AccountKind::CreditCard, 15, 0, "payment"),
        ];
        assert!(detect_pairs(&rows).is_empty());
    }

    #[test]
    fn same_kind_needs_payment_tell() {
        // No payment-tell, same kind → not a transfer (just a coincidence).
        let rows = vec![
            cand(AccountKind::Bank, 15, -43210, "groceries"),
            cand(AccountKind::Bank, 15, 43210, "venmo cashout"),
        ];
        assert!(detect_pairs(&rows).is_empty());

        // With payment-tell, same kind → counted.
        let rows = vec![
            cand(AccountKind::Bank, 15, -43210, "chase autopay"),
            cand(AccountKind::Bank, 15, 43210, "statement balance refund"),
        ];
        assert_eq!(detect_pairs(&rows).len(), 1);
    }

    #[test]
    fn does_not_double_pair() {
        // Three rows, only one valid pair should fire.
        let rows = vec![
            cand(AccountKind::Bank, 15, -43210, "payment"),
            cand(AccountKind::CreditCard, 15, 43210, "thank you"),
            cand(AccountKind::CreditCard, 15, 43210, "thank you"),
        ];
        assert_eq!(detect_pairs(&rows).len(), 1);
    }
}
