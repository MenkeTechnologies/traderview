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
        "generic" | "csv" => Some(Box::new(brokers::GenericCsvParser)),
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Every key in `supported_sources()` must resolve through `parser_for`
    /// to a parser whose `source()` matches the dispatch key. Aliases are
    /// also checked explicitly because they don't appear in `supported_sources`.
    #[test]
    fn parser_for_dispatches_every_supported_source() {
        for key in supported_sources() {
            let p = parser_for(key)
                .unwrap_or_else(|| panic!("supported source {key:?} did not dispatch"));
            // `parser_for("csv")` maps to GenericCsvParser whose source() is "generic" —
            // accept both canonical key and its alias targets.
            let s = p.source();
            assert!(
                s == *key
                    || (*key == "ibkr" && s == "ibkr")
                    || (*key == "tdameritrade" && s == "tdameritrade")
                    || (*key == "tos" && s == "tos")
                    || (*key == "tradestation" && s == "tradestation"),
                "key {key:?} dispatched to parser whose source is {s:?}"
            );
        }
    }

    #[test]
    fn parser_for_resolves_aliases() {
        assert_eq!(parser_for("ibkr-flex").unwrap().source(), "ibkr");
        assert_eq!(parser_for("td").unwrap().source(), "tdameritrade");
        assert_eq!(parser_for("ts").unwrap().source(), "tradestation");
        assert_eq!(parser_for("dastrader").unwrap().source(), "das");
        assert_eq!(parser_for("thinkorswim").unwrap().source(), "tos");
        assert_eq!(parser_for("csv").unwrap().source(), "generic");
    }

    #[test]
    fn parser_for_unknown_returns_none() {
        assert!(parser_for("not-a-broker").is_none());
        assert!(parser_for("").is_none());
    }

    /// `sha256_hex` is the dedupe primitive for the `imports` table.
    /// Identical bytes must produce identical hashes; one different byte
    /// must produce a different hash. This pins the dedupe contract.
    #[test]
    fn sha256_dedupe_key_is_stable_for_identical_bytes() {
        let csv = b"Symbol,Side,Qty,Price,Fee,Date\nAAPL,buy,100,150.50,1.00,2026-01-15 09:30:00\n";
        let a = sha256_hex(csv);
        let b = sha256_hex(csv);
        assert_eq!(a, b, "identical bytes must hash to identical keys");
        assert_eq!(a.len(), 64, "sha256 hex digest must be 64 chars");

        let mutated = b"Symbol,Side,Qty,Price,Fee,Date\nAAPL,buy,101,150.50,1.00,2026-01-15 09:30:00\n";
        assert_ne!(
            a,
            sha256_hex(mutated),
            "one-byte change must produce a different dedupe key"
        );

        // Empty input is still a valid (and stable) digest.
        assert_eq!(sha256_hex(b"").len(), 64);
    }
}
