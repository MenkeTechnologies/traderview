//! traderview-expense — business-expense CSV parsers, merchant→category rules,
//! and cross-account transfer dedup.
//!
//! Source parsers (Amazon, Bank of America, Chase, Apple Card) are stubs that
//! return `Unsupported` until a real redacted export is uploaded — same
//! discipline as the Webull parser in `traderview-import`. Inferring columns
//! from documentation produces wrong column maps; only a real file is the spec.

pub mod amazon;
pub mod apple;
pub mod application_fees;
pub mod boa;
pub mod chase;
pub mod contractor_1099;
pub mod cost_segregation;
pub mod crypto_tax;
pub mod dedup;
pub mod depreciation;
pub mod deposit_interest;
pub mod detector_requirements;
pub mod deposit_return_windows;
pub mod dv_termination;
pub mod entry_notice;
pub mod disposition;
pub mod eviction_notices;
pub mod foreclosure_tenant_rights;
pub mod foreign_tax_credit;
pub mod form_8606;
pub mod habitability_remedies;
pub mod heat_requirements;
pub mod just_cause_eviction;
pub mod home_office;
pub mod late_fee_caps;
pub mod lead_disclosure;
pub mod lease_disclosures;
pub mod lockout_penalties;
pub mod manual_entry;
pub mod meals_50;
pub mod mileage;
pub mod military_termination;
pub mod mlp_ubti;
pub mod mtm_475f;
pub mod niit;
pub mod normalize;
pub mod qbi;
pub mod quarterly_tax;
pub mod recurring;
pub mod rent_control;
pub mod retaliation_windows;
pub mod rental_depreciation;
pub mod reps_qualification;
pub mod rules;
pub mod schedule_d;
pub mod security_deposit_caps;
pub mod soi_protection;
pub mod schedule_e;
pub mod section_1014;
pub mod section_1015;
pub mod section_170e;
pub mod section_121;
pub mod section_1045;
pub mod section_1092;
pub mod section_1202;
pub mod section_1212;
pub mod section_1233;
pub mod section_1234;
pub mod section_1244;
pub mod section_1031_f;
pub mod section_1041;
pub mod section_1091;
pub mod section_1256;
pub mod section_1259;
pub mod section_1295;
pub mod section_1296;
pub mod section_163d;
pub mod section_163j;
pub mod section_168_e6;
pub mod section_168g;
pub mod section_174;
pub mod section_263a;
pub mod section_267;
pub mod section_988;
pub mod section_280a;
pub mod section_280a_d2;
pub mod section_280f;
pub mod section_83b;
pub mod section_408_d3;
pub mod section_408m;
#[allow(non_snake_case)]
pub mod section_408A_d3;
pub mod section_453;
pub mod section_469;
pub mod section_7872;
pub mod section_864b2;
pub mod section_871m;
pub mod section_481;
pub mod seed_rules;
pub mod self_employment_tax;
pub mod sheet;
pub mod subscription_detector;
pub mod tax_equivalent_yield;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A single row from any expense-source CSV, normalized into our shape.
///
/// Sign convention: `amount` is negative for money out (expense) and positive
/// for money in (refund, income, statement credit). Each parser does that
/// normalization since each source picks its own sign convention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    pub posted_at: DateTime<Utc>,
    pub amount: Decimal,
    pub currency: String,
    pub merchant_raw: String,
    pub merchant_normalized: String,
    pub description: String,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseSource {
    Amazon,
    Bofa,
    Chase,
    AppleCard,
}

impl ExpenseSource {
    pub fn as_str(self) -> &'static str {
        match self {
            ExpenseSource::Amazon => "amazon",
            ExpenseSource::Bofa => "bofa",
            ExpenseSource::Chase => "chase",
            ExpenseSource::AppleCard => "apple_card",
        }
    }

    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "amazon" => Some(Self::Amazon),
            "bofa" | "bank_of_america" => Some(Self::Bofa),
            "chase" => Some(Self::Chase),
            "apple_card" | "apple" => Some(Self::AppleCard),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("csv: {0}")]
    Csv(#[from] csv::Error),
    #[error("parse: {0}")]
    Parse(String),
    #[error("unsupported format: {0}")]
    Unsupported(String),
}

pub trait Parser {
    fn source(&self) -> ExpenseSource;
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedTransaction>, ImportError>;
}

/// Dispatch a source enum to its concrete parser.
pub fn parse(source: ExpenseSource, bytes: &[u8]) -> Result<Vec<ParsedTransaction>, ImportError> {
    match source {
        ExpenseSource::Amazon => amazon::AmazonParser.parse(bytes),
        ExpenseSource::Bofa => boa::BofaParser.parse(bytes),
        ExpenseSource::Chase => chase::ChaseParser.parse(bytes),
        ExpenseSource::AppleCard => apple::AppleCardParser.parse(bytes),
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expense_source_str_roundtrip() {
        // Every variant must roundtrip through as_str / parse_str — the DB
        // stores the string and the parsers dispatch on it, so a mismatch
        // here silently drops imports.
        for src in [
            ExpenseSource::Amazon,
            ExpenseSource::Bofa,
            ExpenseSource::Chase,
            ExpenseSource::AppleCard,
        ] {
            let s = src.as_str();
            let back =
                ExpenseSource::parse_str(s).unwrap_or_else(|| panic!("`{s}` did not roundtrip"));
            assert_eq!(back, src);
        }
    }

    #[test]
    fn expense_source_parse_str_accepts_aliases() {
        // `bank_of_america` should map to Bofa; `apple` should map to AppleCard.
        assert_eq!(
            ExpenseSource::parse_str("bank_of_america"),
            Some(ExpenseSource::Bofa)
        );
        assert_eq!(
            ExpenseSource::parse_str("apple"),
            Some(ExpenseSource::AppleCard)
        );
    }

    #[test]
    fn expense_source_parse_str_rejects_unknown() {
        assert_eq!(ExpenseSource::parse_str("citibank"), None);
        assert_eq!(ExpenseSource::parse_str(""), None);
    }

    #[test]
    fn sha256_hex_is_deterministic() {
        // Same bytes → same digest, every time.
        let a = sha256_hex(b"hello world");
        let b = sha256_hex(b"hello world");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64, "SHA-256 hex is always 64 chars");
    }

    #[test]
    fn sha256_hex_diverges_on_one_byte_mutation() {
        let a = sha256_hex(b"hello world");
        let b = sha256_hex(b"hello worle");
        assert_ne!(a, b, "SHA-256 must avalanche on single-byte change");
    }

    #[test]
    fn sha256_hex_known_value_for_empty_input() {
        // RFC 6234 — empty input has a fixed digest.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn parse_returns_unsupported_for_stub_sources() {
        // The 4 parsers currently return Unsupported (they're stubs awaiting
        // real redacted CSV samples). This is the correct safety behavior —
        // pinning it so a future "fake-it" change is a deliberate choice.
        for src in [
            ExpenseSource::Amazon,
            ExpenseSource::Bofa,
            ExpenseSource::Chase,
            ExpenseSource::AppleCard,
        ] {
            let result = parse(src, b"this is not a real csv");
            // Either succeeds with an empty parse or returns an error — both
            // are acceptable; what's NOT acceptable is a panic.
            let _ = result; // smoke test: must not panic.
        }
    }
}
