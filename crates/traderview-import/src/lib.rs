//! traderview-import — broker file parsers.
//!
//! Add a new broker by implementing `Parser`. Each parser returns a vec of
//! `ParsedExecution`s with the broker-specific raw row preserved as JSON so
//! the original can be re-parsed if the schema evolves.

pub mod webull;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use traderview_core::Side;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedExecution {
    pub symbol: String,
    pub side: Side,
    pub qty: Decimal,
    pub price: Decimal,
    pub fee: Decimal,
    pub executed_at: DateTime<Utc>,
    pub broker_order_id: Option<String>,
    pub raw: serde_json::Value,
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
    fn source(&self) -> &'static str;
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError>;
}

/// SHA-256 of the raw uploaded bytes — used as the dedupe key in `imports`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}
