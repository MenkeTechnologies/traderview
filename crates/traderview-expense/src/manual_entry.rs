//! Manual expense entry (cash, ad-hoc, no CSV).
//!
//! The 12-broker CSV pipeline covers ~95% of business expenses, but
//! cash, Venmo, Zelle, and split-bill-with-friends transactions need a
//! hand-entered path. This module just validates + normalizes the
//! payload into the same `ParsedTransaction` shape every other importer
//! emits, so the downstream rules engine + dedup + Schedule C apply
//! verbatim.
//!
//! Pure compute. The DB INSERT happens in the route handler.

use crate::{normalize, ParsedTransaction};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualEntry {
    /// Date the cash left your hand. Free-text not allowed — caller
    /// parses ISO before constructing this struct.
    pub posted_at: DateTime<Utc>,
    /// Negative for expense, positive for refund.
    pub amount: Decimal,
    pub currency: String,
    /// Raw merchant — gets normalized below.
    pub merchant_raw: String,
    pub description: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ManualEntryError {
    #[error("amount is zero — not an expense")]
    ZeroAmount,
    #[error("currency must be exactly 3 letters (ISO 4217); got `{0}`")]
    BadCurrency(String),
    #[error("merchant is empty")]
    EmptyMerchant,
    #[error("amount {amount} exceeds the {cap} sanity cap — split the entry")]
    AmountExceedsSanityCap { amount: Decimal, cap: Decimal },
}

/// Sanity cap so a fat-finger entry of `$100000000` doesn't silently
/// post and tank the year-end report. Caller can split larger entries
/// across multiple ManualEntry rows.
const SANITY_CAP_DOLLARS: i64 = 1_000_000;

pub fn validate_and_normalize(e: ManualEntry) -> Result<ParsedTransaction, ManualEntryError> {
    if e.amount.is_zero() {
        return Err(ManualEntryError::ZeroAmount);
    }
    if e.amount.abs() > Decimal::from(SANITY_CAP_DOLLARS) {
        return Err(ManualEntryError::AmountExceedsSanityCap {
            amount: e.amount,
            cap: Decimal::from(SANITY_CAP_DOLLARS),
        });
    }
    let ccy = e.currency.trim().to_ascii_uppercase();
    if ccy.len() != 3 || !ccy.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(ManualEntryError::BadCurrency(e.currency.clone()));
    }
    let merchant_raw = e.merchant_raw.trim().to_string();
    if merchant_raw.is_empty() {
        return Err(ManualEntryError::EmptyMerchant);
    }
    let merchant_normalized = normalize::normalize(&merchant_raw);
    Ok(ParsedTransaction {
        posted_at: e.posted_at,
        amount: e.amount,
        currency: ccy,
        merchant_raw,
        merchant_normalized,
        description: e.description,
        raw: serde_json::json!({ "source": "manual_entry" }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn entry() -> ManualEntry {
        ManualEntry {
            posted_at: Utc::now(),
            amount: Decimal::from(-25),  // $25 cash expense
            currency: "USD".into(),
            merchant_raw: "Whole Foods #312".into(),
            description: "team lunch".into(),
        }
    }

    #[test]
    fn validates_and_normalizes_a_basic_cash_expense() {
        let p = validate_and_normalize(entry()).unwrap();
        assert_eq!(p.amount, Decimal::from(-25));
        assert_eq!(p.currency, "USD");
        assert!(!p.merchant_normalized.is_empty());
        assert_eq!(p.raw["source"], "manual_entry");
    }

    #[test]
    fn rejects_zero_amount() {
        let mut e = entry(); e.amount = Decimal::ZERO;
        assert!(matches!(validate_and_normalize(e), Err(ManualEntryError::ZeroAmount)));
    }

    #[test]
    fn rejects_sanity_cap_overage() {
        let mut e = entry();
        e.amount = Decimal::from_str("-1000001").unwrap();
        assert!(matches!(validate_and_normalize(e),
            Err(ManualEntryError::AmountExceedsSanityCap { .. })));
    }

    #[test]
    fn sanity_cap_inclusive_at_one_million() {
        let mut e = entry();
        e.amount = Decimal::from(-1_000_000);
        assert!(validate_and_normalize(e).is_ok(),
            "exactly the cap must be accepted; only OVER the cap rejects");
    }

    #[test]
    fn rejects_non_iso_currency() {
        let mut e = entry(); e.currency = "Dollars".into();
        assert!(matches!(validate_and_normalize(e), Err(ManualEntryError::BadCurrency(_))));
    }

    #[test]
    fn rejects_empty_merchant() {
        let mut e = entry(); e.merchant_raw = "   ".into();
        assert!(matches!(validate_and_normalize(e), Err(ManualEntryError::EmptyMerchant)));
    }

    #[test]
    fn uppercases_currency_and_trims_merchant() {
        let mut e = entry();
        e.currency = "  eur ".into();
        e.merchant_raw = "  Café Latté  ".into();
        let p = validate_and_normalize(e).unwrap();
        assert_eq!(p.currency, "EUR");
        assert_eq!(p.merchant_raw, "Café Latté");
    }

    #[test]
    fn refunds_are_positive_and_pass_validation() {
        let mut e = entry();
        e.amount = Decimal::from(15);   // refund
        let p = validate_and_normalize(e).unwrap();
        assert!(p.amount.is_sign_positive());
    }
}
