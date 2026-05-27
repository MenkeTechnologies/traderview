//! traderview-expense — business-expense CSV parsers, merchant→category rules,
//! and cross-account transfer dedup.
//!
//! Source parsers (Amazon, Bank of America, Chase, Apple Card) are stubs that
//! return `Unsupported` until a real redacted export is uploaded — same
//! discipline as the Webull parser in `traderview-import`. Inferring columns
//! from documentation produces wrong column maps; only a real file is the spec.

pub mod amazon;
pub mod apple;
pub mod boa;
pub mod chase;
pub mod dedup;
pub mod home_office;
pub mod mileage;
pub mod normalize;
pub mod quarterly_tax;
pub mod rules;
pub mod seed_rules;
pub mod self_employment_tax;
pub mod sheet;
pub mod subscription_detector;

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
