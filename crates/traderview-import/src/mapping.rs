//! Generic column-map CSV parser.
//!
//! A `ColumnMap` declares which CSV column (by header name OR by index) maps
//! to each `ParsedExecution` field, plus how to decode side strings and parse
//! dates. Per-broker parsers are then just preset `ColumnMap`s passed to
//! [`parse_with`].

use crate::{ImportError, ParsedExecution};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use traderview_core::{AssetClass, OptionType, Side};

/// Where to find a field — by header name or by column index.
#[derive(Debug, Clone)]
pub enum ColSpec {
    Header(&'static str),
    HeaderAny(&'static [&'static str]), // first-match-wins across aliases
    Index(usize),
    Constant(String),
}

#[derive(Debug, Clone)]
pub struct ColumnMap {
    pub source: &'static str,
    pub has_header: bool,
    pub delimiter: u8,
    pub date_formats: &'static [&'static str], // tried in order
    pub utc_assumed: bool, // true = parse naive as UTC; false = parse naive as local
    pub side_lookup: SideLookup,
    pub symbol: ColSpec,
    pub side: ColSpec,
    pub qty: ColSpec,
    pub price: ColSpec,
    pub fee: Option<ColSpec>,
    pub executed_at: ColSpec,
    pub broker_order_id: Option<ColSpec>,
    pub asset_class: Option<ColSpec>,    // optional column; defaults to Stock
    pub option_type: Option<ColSpec>,
    pub strike: Option<ColSpec>,
    pub expiration: Option<ColSpec>,
    pub multiplier: Option<ColSpec>,
    /// Skip rows whose symbol is blank or matches one of these literals.
    pub skip_symbols: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub struct SideLookup {
    pub buy: &'static [&'static str],
    pub sell: &'static [&'static str],
    pub short: &'static [&'static str],
    pub cover: &'static [&'static str],
}

impl SideLookup {
    pub const DEFAULT: SideLookup = SideLookup {
        buy: &["buy", "b", "bought", "long", "buy to open", "bto"],
        sell: &["sell", "s", "sold", "sell to close", "stc"],
        short: &["short", "ss", "sell short", "sellshort", "sell to open", "sto"],
        cover: &["cover", "buy to cover", "btc", "cover short"],
    };
}

pub fn parse_with(bytes: &[u8], map: &ColumnMap) -> Result<Vec<ParsedExecution>, ImportError> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(map.delimiter)
        .has_headers(map.has_header)
        .flexible(true)
        .from_reader(bytes);

    let header_lookup: HashMap<String, usize> = if map.has_header {
        rdr.headers()?
            .iter()
            .enumerate()
            .map(|(i, h)| (normalize_header(h), i))
            .collect()
    } else {
        HashMap::new()
    };

    let mut out = Vec::new();

    for (row_idx, rec) in rdr.records().enumerate() {
        let rec = match rec {
            Ok(r) => r,
            Err(e) => return Err(ImportError::Csv(e)),
        };
        let row = Row {
            rec: &rec,
            header_lookup: &header_lookup,
        };

        let symbol = row.field(&map.symbol).map(|s| s.trim().to_string());
        let symbol = match symbol {
            Some(s) if !s.is_empty() && !map.skip_symbols.iter().any(|k| k.eq_ignore_ascii_case(&s)) => s,
            _ => continue, // skip blank-symbol rows (totals etc.)
        };

        let side_raw = row.field(&map.side).ok_or_else(|| {
            ImportError::Parse(format!("row {}: missing side column", row_idx + 1))
        })?;
        let side = decode_side(&side_raw, &map.side_lookup).ok_or_else(|| {
            ImportError::Parse(format!("row {}: unknown side {:?}", row_idx + 1, side_raw))
        })?;

        let qty = parse_decimal(&row.field(&map.qty).unwrap_or_default())
            .ok_or_else(|| ImportError::Parse(format!("row {}: bad qty", row_idx + 1)))?;
        if qty <= Decimal::ZERO {
            continue;
        }

        let price = parse_decimal(&row.field(&map.price).unwrap_or_default())
            .ok_or_else(|| ImportError::Parse(format!("row {}: bad price", row_idx + 1)))?;

        let fee = map
            .fee
            .as_ref()
            .and_then(|c| row.field(c))
            .and_then(|s| parse_decimal(&s))
            .unwrap_or(Decimal::ZERO)
            .abs();

        let executed_at_raw = row
            .field(&map.executed_at)
            .ok_or_else(|| ImportError::Parse(format!("row {}: missing date", row_idx + 1)))?;
        let executed_at = parse_datetime(&executed_at_raw, map.date_formats, map.utc_assumed)
            .ok_or_else(|| {
                ImportError::Parse(format!(
                    "row {}: unparseable date {:?}",
                    row_idx + 1,
                    executed_at_raw
                ))
            })?;

        let broker_order_id = map
            .broker_order_id
            .as_ref()
            .and_then(|c| row.field(c))
            .filter(|s| !s.is_empty());

        let asset_class = map
            .asset_class
            .as_ref()
            .and_then(|c| row.field(c))
            .map(|s| decode_asset_class(&s))
            .unwrap_or(AssetClass::Stock);

        let option_type = map
            .option_type
            .as_ref()
            .and_then(|c| row.field(c))
            .and_then(|s| decode_option_type(&s));

        let strike = map
            .strike
            .as_ref()
            .and_then(|c| row.field(c))
            .and_then(|s| parse_decimal(&s));

        let expiration = map
            .expiration
            .as_ref()
            .and_then(|c| row.field(c))
            .and_then(|s| parse_date(&s));

        let multiplier = map
            .multiplier
            .as_ref()
            .and_then(|c| row.field(c))
            .and_then(|s| parse_decimal(&s))
            .unwrap_or_else(|| match asset_class {
                AssetClass::Option => Decimal::from(100),
                _ => Decimal::ONE,
            });

        // Preserve the original row for re-parse later.
        let raw = serde_json::Value::Array(
            rec.iter()
                .map(|v| serde_json::Value::String(v.into()))
                .collect(),
        );

        out.push(ParsedExecution {
            symbol,
            side,
            qty,
            price,
            fee,
            executed_at,
            broker_order_id,
            raw,
            asset_class,
            option_type,
            strike,
            expiration,
            multiplier,
            tick_size: None,
            tick_value: None,
            base_ccy: None,
            quote_ccy: None,
            pip_size: None,
        });
    }
    Ok(out)
}

struct Row<'a> {
    rec: &'a csv::StringRecord,
    header_lookup: &'a HashMap<String, usize>,
}

impl<'a> Row<'a> {
    fn field(&self, spec: &ColSpec) -> Option<String> {
        match spec {
            ColSpec::Header(h) => self
                .header_lookup
                .get(&normalize_header(h))
                .and_then(|i| self.rec.get(*i))
                .map(|s| s.to_string()),
            ColSpec::HeaderAny(hs) => hs.iter().find_map(|h| {
                self.header_lookup
                    .get(&normalize_header(h))
                    .and_then(|i| self.rec.get(*i))
                    .map(|s| s.to_string())
            }),
            ColSpec::Index(i) => self.rec.get(*i).map(|s| s.to_string()),
            ColSpec::Constant(s) => Some(s.clone()),
        }
    }
}

fn normalize_header(h: &str) -> String {
    h.trim().to_ascii_lowercase()
}

fn decode_side(raw: &str, lookup: &SideLookup) -> Option<Side> {
    let n = raw.trim().to_ascii_lowercase();
    if lookup.buy.iter().any(|s| *s == n) {
        Some(Side::Buy)
    } else if lookup.sell.iter().any(|s| *s == n) {
        Some(Side::Sell)
    } else if lookup.short.iter().any(|s| *s == n) {
        Some(Side::Short)
    } else if lookup.cover.iter().any(|s| *s == n) {
        Some(Side::Cover)
    } else {
        None
    }
}

fn decode_asset_class(raw: &str) -> AssetClass {
    let n = raw.trim().to_ascii_lowercase();
    match n.as_str() {
        "option" | "opt" | "options" => AssetClass::Option,
        "future" | "fut" | "futures" => AssetClass::Future,
        "forex" | "fx" | "cash" => AssetClass::Forex,
        _ => AssetClass::Stock,
    }
}

fn decode_option_type(raw: &str) -> Option<OptionType> {
    let n = raw.trim().to_ascii_lowercase();
    match n.as_str() {
        "c" | "call" => Some(OptionType::Call),
        "p" | "put" => Some(OptionType::Put),
        _ => None,
    }
}

fn parse_decimal(raw: &str) -> Option<Decimal> {
    let cleaned = raw.trim().replace([',', '$'], "");
    if cleaned.is_empty() {
        return None;
    }
    Decimal::from_str(&cleaned).ok()
}

fn parse_datetime(raw: &str, formats: &[&str], utc_assumed: bool) -> Option<DateTime<Utc>> {
    let s = raw.trim();
    // Try datetime formats.
    for fmt in formats {
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(if utc_assumed {
                Utc.from_utc_datetime(&ndt)
            } else {
                Utc.from_local_datetime(&ndt).single()?
            });
        }
    }
    // Try ISO 8601 with offset.
    if let Ok(d) = DateTime::parse_from_rfc3339(s) {
        return Some(d.with_timezone(&Utc));
    }
    // Fallback: date-only → midnight UTC.
    parse_date(s).map(|d| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()))
}

fn parse_date(raw: &str) -> Option<NaiveDate> {
    let s = raw.trim();
    for fmt in ["%Y-%m-%d", "%m/%d/%Y", "%d/%m/%Y", "%Y%m%d", "%m-%d-%Y"] {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
    }
    None
}
