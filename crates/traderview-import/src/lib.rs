//! `traderview-import` — broker file parsers.
//!
//! Two layers:
//!  - [`Parser`] trait — per-broker (Webull, IBKR Flex, TD/Schwab, TOS, ...).
//!  - [`mapping`] — generic CSV mapper. Each broker is just a `ColumnMap`
//!    preset; the mapper does the actual CSV → [`ParsedExecution`] work.
//!    Unknown formats can be imported by hand-crafting a `ColumnMap`.
//!
//! Every parser returns the original row as `raw` so the importer can
//! re-parse if the schema improves later.

pub mod brokers;
pub mod mapping;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use traderview_core::{AssetClass, OptionType, Side};

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
    // Optional per-asset metadata. Defaulted to stock if absent.
    pub asset_class: AssetClass,
    pub option_type: Option<OptionType>,
    pub strike: Option<Decimal>,
    pub expiration: Option<NaiveDate>,
    pub multiplier: Decimal,
    pub tick_size: Option<Decimal>,
    pub tick_value: Option<Decimal>,
    pub base_ccy: Option<String>,
    pub quote_ccy: Option<String>,
    pub pip_size: Option<Decimal>,
}

impl ParsedExecution {
    pub fn stock(
        symbol: impl Into<String>,
        side: Side,
        qty: Decimal,
        price: Decimal,
        fee: Decimal,
        executed_at: DateTime<Utc>,
    ) -> Self {
        ParsedExecution {
            symbol: symbol.into(),
            side,
            qty,
            price,
            fee,
            executed_at,
            broker_order_id: None,
            raw: serde_json::json!({}),
            asset_class: AssetClass::Stock,
            option_type: None,
            strike: None,
            expiration: None,
            multiplier: Decimal::ONE,
            tick_size: None,
            tick_value: None,
            base_ccy: None,
            quote_ccy: None,
            pip_size: None,
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
    #[error("missing column: {0}")]
    MissingColumn(String),
    #[error("unsupported format: {0}")]
    Unsupported(String),
}

pub trait Parser: Send + Sync {
    /// Stable identifier for this broker; mirrors the `imports.source` column.
    fn source(&self) -> &'static str;
    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedExecution>, ImportError>;
}

/// SHA-256 of raw uploaded bytes; used as dedupe key in the `imports` table.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

/// Dispatch: pick a parser by source key. Returns None if unknown.
pub fn parser_for(source: &str) -> Option<Box<dyn Parser>> {
    use brokers::*;
    match source {
        "webull" => Some(Box::new(WebullParser)),
        "ibkr" | "ibkr-flex" => Some(Box::new(IbkrFlexParser)),
        "tdameritrade" | "td" => Some(Box::new(TdAmeritradeParser)),
        "schwab" => Some(Box::new(SchwabParser)),
        "tradestation" | "ts" => Some(Box::new(TradeStationParser)),
        "lightspeed" => Some(Box::new(LightspeedParser)),
        "das" | "dastrader" => Some(Box::new(DasParser)),
        "tos" | "thinkorswim" => Some(Box::new(ThinkOrSwimParser)),
        "etrade" => Some(Box::new(ETradeParser)),
        "fidelity" => Some(Box::new(FidelityParser)),
        "tradezero" => Some(Box::new(TradeZeroParser)),
        "robinhood" => Some(Box::new(RobinhoodParser)),
        "generic" | "csv" => Some(Box::new(brokers::GenericCsvParser::default())),
        _ => None,
    }
}

/// List of all supported broker source keys, for UI dropdowns.
pub fn supported_sources() -> &'static [&'static str] {
    &[
        "webull",
        "ibkr",
        "tdameritrade",
        "schwab",
        "tradestation",
        "lightspeed",
        "das",
        "tos",
        "etrade",
        "fidelity",
        "tradezero",
        "robinhood",
        "generic",
    ]
}
