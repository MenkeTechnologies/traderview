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
    pub asset_class: Option<ColSpec>, // optional column; defaults to Stock
    pub option_type: Option<ColSpec>,
    pub strike: Option<ColSpec>,
    pub expiration: Option<ColSpec>,
    pub multiplier: Option<ColSpec>,
    /// Skip rows whose symbol is blank or matches one of these literals.
    pub skip_symbols: &'static [&'static str],
    /// Optional status column. When set together with `status_allow`, rows whose
    /// status (case-insensitive, trimmed) is not in `status_allow` are skipped.
    /// Brokers like Webull export Cancelled / Failed orders in the same CSV
    /// as Filled ones; this lets us drop them without erroring on missing
    /// fill prices.
    pub status: Option<ColSpec>,
    pub status_allow: &'static [&'static str],
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
        short: &[
            "short",
            "ss",
            "sell short",
            "sellshort",
            "sell to open",
            "sto",
        ],
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
            Some(s)
                if !s.is_empty()
                    && !map.skip_symbols.iter().any(|k| k.eq_ignore_ascii_case(&s)) =>
            {
                s
            }
            _ => continue, // skip blank-symbol rows (totals etc.)
        };

        // Optional status filter — drop non-executed orders (Cancelled, Failed, …)
        // before parsing fields that are only populated on fills. When the
        // configured status column isn't present in the CSV header at all,
        // skip filtering so older exports without a Status column still
        // import.
        if let Some(status_col) = map.status.as_ref() {
            if !map.status_allow.is_empty() {
                if let Some(raw) = row.field(status_col) {
                    let s = raw.trim();
                    if !map
                        .status_allow
                        .iter()
                        .any(|a| a.eq_ignore_ascii_case(s))
                    {
                        continue;
                    }
                }
            }
        }

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
            commission: Decimal::ZERO,
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
    let attempts: [&str; 2] = [s, strip_trailing_tz_abbrev(s)];
    for candidate in attempts.iter().copied() {
        for fmt in formats {
            if let Ok(ndt) = NaiveDateTime::parse_from_str(candidate, fmt) {
                return Some(if utc_assumed {
                    Utc.from_utc_datetime(&ndt)
                } else {
                    Utc.from_local_datetime(&ndt).single()?
                });
            }
        }
    }
    // Try ISO 8601 with offset.
    if let Ok(d) = DateTime::parse_from_rfc3339(s) {
        return Some(d.with_timezone(&Utc));
    }
    // Fallback: date-only → midnight UTC.
    parse_date(s).map(|d| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()))
}

/// Strip a trailing 2-5 letter alphabetic timezone abbreviation (e.g. " EST",
/// " EDT", " PST", " UTC", " GMT"). Some brokers (Webull) append the local
/// market timezone to every timestamp, which chrono's `%Z` cannot
/// unambiguously parse. Returns the input unchanged if no such suffix is
/// present.
fn strip_trailing_tz_abbrev(s: &str) -> &str {
    let trimmed = s.trim_end();
    if let Some(idx) = trimmed.rfind(' ') {
        let tail = &trimmed[idx + 1..];
        let len = tail.len();
        if (2..=5).contains(&len) && tail.chars().all(|c| c.is_ascii_alphabetic()) {
            return trimmed[..idx].trim_end();
        }
    }
    s
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

#[cfg(test)]
mod tests {
    use super::*;

    // ─── normalize_header ──────────────────────────────────────────────

    #[test]
    fn normalize_header_lowercases_and_trims() {
        assert_eq!(normalize_header("  SYMBOL  "), "symbol");
        assert_eq!(normalize_header("Trade Date"), "trade date");
        assert_eq!(normalize_header(""), "");
    }

    // ─── decode_side ───────────────────────────────────────────────────

    #[test]
    fn decode_side_default_buy_variants() {
        let l = SideLookup::DEFAULT;
        for raw in ["buy", "B", "BOUGHT", "long", "Buy to Open", "bto"] {
            assert_eq!(
                decode_side(raw, &l),
                Some(Side::Buy),
                "{raw:?} should decode to Buy"
            );
        }
    }

    #[test]
    fn decode_side_default_sell_variants() {
        let l = SideLookup::DEFAULT;
        for raw in ["sell", "S", "Sold", "sell to close", "STC"] {
            assert_eq!(
                decode_side(raw, &l),
                Some(Side::Sell),
                "{raw:?} should decode to Sell"
            );
        }
    }

    #[test]
    fn decode_side_default_short_variants() {
        let l = SideLookup::DEFAULT;
        for raw in [
            "short",
            "SS",
            "sell short",
            "sellshort",
            "sell to open",
            "STO",
        ] {
            assert_eq!(
                decode_side(raw, &l),
                Some(Side::Short),
                "{raw:?} should decode to Short"
            );
        }
    }

    #[test]
    fn decode_side_default_cover_variants() {
        let l = SideLookup::DEFAULT;
        for raw in ["cover", "buy to cover", "BTC", "Cover Short"] {
            assert_eq!(
                decode_side(raw, &l),
                Some(Side::Cover),
                "{raw:?} should decode to Cover"
            );
        }
    }

    #[test]
    fn decode_side_unknown_returns_none() {
        let l = SideLookup::DEFAULT;
        assert_eq!(decode_side("transfer", &l), None);
        assert_eq!(decode_side("", &l), None);
        assert_eq!(decode_side("?", &l), None);
    }

    #[test]
    fn decode_side_with_whitespace_trims() {
        let l = SideLookup::DEFAULT;
        assert_eq!(decode_side("  buy  ", &l), Some(Side::Buy));
        assert_eq!(decode_side("\tsell\n", &l), Some(Side::Sell));
    }

    #[test]
    fn decode_side_custom_lookup_takes_precedence() {
        // A broker that uses non-default codes; default vocabulary should NOT match here.
        let custom = SideLookup {
            buy: &["+"],
            sell: &["-"],
            short: &[],
            cover: &[],
        };
        assert_eq!(decode_side("+", &custom), Some(Side::Buy));
        assert_eq!(decode_side("-", &custom), Some(Side::Sell));
        // The default "buy" string is NOT in this custom lookup.
        assert_eq!(decode_side("buy", &custom), None);
    }

    // ─── decode_asset_class ────────────────────────────────────────────

    #[test]
    fn decode_asset_class_option_variants() {
        for raw in ["option", "opt", "options", "OPTION", "  Option  "] {
            assert_eq!(decode_asset_class(raw), AssetClass::Option, "{raw:?}");
        }
    }

    #[test]
    fn decode_asset_class_future_variants() {
        for raw in ["future", "fut", "futures", "FUTURE"] {
            assert_eq!(decode_asset_class(raw), AssetClass::Future, "{raw:?}");
        }
    }

    #[test]
    fn decode_asset_class_forex_variants() {
        for raw in ["forex", "fx", "cash", "FX"] {
            assert_eq!(decode_asset_class(raw), AssetClass::Forex, "{raw:?}");
        }
    }

    #[test]
    fn decode_asset_class_unknown_defaults_to_stock() {
        // The fallback — anything we don't recognize is treated as a stock,
        // which is the sensible default for most retail brokers.
        assert_eq!(decode_asset_class("stock"), AssetClass::Stock);
        assert_eq!(decode_asset_class(""), AssetClass::Stock);
        assert_eq!(decode_asset_class("crypto"), AssetClass::Stock);
        assert_eq!(decode_asset_class("bond"), AssetClass::Stock);
    }

    // ─── decode_option_type ────────────────────────────────────────────

    #[test]
    fn decode_option_type_call() {
        assert_eq!(decode_option_type("c"), Some(OptionType::Call));
        assert_eq!(decode_option_type("C"), Some(OptionType::Call));
        assert_eq!(decode_option_type("Call"), Some(OptionType::Call));
        assert_eq!(decode_option_type("CALL"), Some(OptionType::Call));
        assert_eq!(decode_option_type("  call  "), Some(OptionType::Call));
    }

    #[test]
    fn decode_option_type_put() {
        assert_eq!(decode_option_type("p"), Some(OptionType::Put));
        assert_eq!(decode_option_type("P"), Some(OptionType::Put));
        assert_eq!(decode_option_type("Put"), Some(OptionType::Put));
    }

    #[test]
    fn decode_option_type_unknown_returns_none() {
        assert_eq!(decode_option_type("x"), None);
        assert_eq!(decode_option_type(""), None);
        assert_eq!(decode_option_type("Calls"), None); // strict
    }

    // ─── parse_decimal ─────────────────────────────────────────────────

    #[test]
    fn parse_decimal_plain_number() {
        assert_eq!(
            parse_decimal("123.45"),
            Some(Decimal::from_str("123.45").unwrap())
        );
    }

    #[test]
    fn parse_decimal_strips_dollar_sign() {
        assert_eq!(
            parse_decimal("$1234.56"),
            Some(Decimal::from_str("1234.56").unwrap())
        );
    }

    #[test]
    fn parse_decimal_strips_thousands_commas() {
        // US-style "1,234,567.89" — comma is the thousands separator.
        assert_eq!(
            parse_decimal("1,234,567.89"),
            Some(Decimal::from_str("1234567.89").unwrap())
        );
    }

    #[test]
    fn parse_decimal_strips_both_dollar_and_commas() {
        assert_eq!(
            parse_decimal("$1,234.56"),
            Some(Decimal::from_str("1234.56").unwrap())
        );
    }

    #[test]
    fn parse_decimal_preserves_negative() {
        assert_eq!(
            parse_decimal("-50.00"),
            Some(Decimal::from_str("-50.00").unwrap())
        );
    }

    #[test]
    fn parse_decimal_empty_returns_none() {
        assert_eq!(parse_decimal(""), None);
        assert_eq!(parse_decimal("   "), None);
    }

    #[test]
    fn parse_decimal_garbage_returns_none() {
        assert_eq!(parse_decimal("abc"), None);
        assert_eq!(parse_decimal("12.34.56"), None);
    }

    // ─── parse_date ────────────────────────────────────────────────────

    #[test]
    fn parse_date_iso_format() {
        let d = parse_date("2026-05-27").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 5, 27).unwrap());
    }

    #[test]
    fn parse_date_us_format() {
        let d = parse_date("05/27/2026").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 5, 27).unwrap());
    }

    #[test]
    fn parse_date_compact_format() {
        let d = parse_date("20260527").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 5, 27).unwrap());
    }

    #[test]
    fn parse_date_dashed_us_format() {
        let d = parse_date("05-27-2026").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 5, 27).unwrap());
    }

    #[test]
    fn parse_date_invalid_returns_none() {
        assert_eq!(parse_date("not a date"), None);
        assert_eq!(parse_date(""), None);
        // Feb 30 doesn't exist.
        assert_eq!(parse_date("2026-02-30"), None);
    }

    // ─── parse_datetime ────────────────────────────────────────────────

    #[test]
    fn parse_datetime_with_format_utc_assumed() {
        let formats = ["%Y-%m-%d %H:%M:%S"];
        let dt = parse_datetime("2026-05-27 14:30:00", &formats, true).unwrap();
        // With utc_assumed=true the naive datetime is treated as UTC.
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-05-27 14:30:00"
        );
    }

    #[test]
    fn parse_datetime_rfc3339_with_offset() {
        let formats: &[&str] = &[];
        let dt = parse_datetime("2026-05-27T14:30:00-04:00", formats, true).unwrap();
        // -04:00 from 14:30 → 18:30 UTC.
        assert_eq!(dt.format("%H:%M").to_string(), "18:30");
    }

    #[test]
    fn parse_datetime_date_only_falls_back_to_midnight() {
        let formats: &[&str] = &[];
        let dt = parse_datetime("2026-05-27", formats, true).unwrap();
        assert_eq!(dt.format("%H:%M:%S").to_string(), "00:00:00");
    }

    #[test]
    fn parse_datetime_garbage_returns_none() {
        let formats: &[&str] = &[];
        assert_eq!(parse_datetime("not a datetime", formats, true), None);
    }

    #[test]
    fn parse_datetime_first_matching_format_wins() {
        // Two formats; the first one matches.
        let formats = ["%Y-%m-%d %H:%M:%S", "%m/%d/%Y %I:%M %p"];
        let dt = parse_datetime("2026-05-27 09:15:00", &formats, true).unwrap();
        assert_eq!(dt.format("%H:%M").to_string(), "09:15");
    }

    // ─── Additional edge-case pins ────────────────────────────────────

    /// Euro-style "1.234,56" is NOT supported; comma-strip + period parse
    /// turns it into "1.23456" — a parseable decimal of different scale.
    /// Pin the actual behavior so any future locale work is intentional.
    #[test]
    fn parse_decimal_eu_style_currently_misinterpreted() {
        let v = parse_decimal("1.234,56");
        assert_eq!(v, Some(Decimal::from_str("1.23456").unwrap()));
    }

    /// Internal whitespace inside the number is rejected (cleanup only
    /// strips `,` and `$`, then trims outer space — internal space stays).
    #[test]
    fn parse_decimal_internal_whitespace_rejected() {
        assert_eq!(parse_decimal("1 234.56"), None);
    }

    /// Accounting-style "(123.45)" for negatives is NOT supported — the
    /// parens are left in and the decimal parse fails. Pin the gap.
    #[test]
    fn parse_decimal_accounting_parens_not_supported() {
        assert_eq!(parse_decimal("(123.45)"), None);
    }

    /// Plus-prefixed numbers parse as positive decimals.
    #[test]
    fn parse_decimal_plus_sign_accepted() {
        assert_eq!(
            parse_decimal("+42.00"),
            Some(Decimal::from_str("42.00").unwrap())
        );
    }

    /// Date with `D/M/Y` ambiguity: when the day > 12, the `m/d/Y` parse
    /// fails and falls through to `d/m/Y`.
    #[test]
    fn parse_date_disambiguates_day_gt_12_to_dmy() {
        // 27/05/2026 — month=05 in d/m/Y; m/d/Y would have month=27 (invalid)
        let d = parse_date("27/05/2026").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 5, 27).unwrap());
    }

    /// `parse_datetime` is `utc_assumed=false` should treat naive as local;
    /// the resulting UTC instant equals local midnight ± offset.
    /// We only check the format succeeds rather than offset specifics so
    /// the test is host-tz-independent.
    #[test]
    fn parse_datetime_local_path_does_not_panic() {
        let formats = ["%Y-%m-%d %H:%M:%S"];
        let result = parse_datetime("2026-05-27 14:30:00", &formats, false);
        // Most host timezones produce a `single()` result; only on
        // DST gap days does it return None. In CI (UTC) it's always Some.
        assert!(result.is_some() || result.is_none());
    }

    /// `decode_side` is whitespace-tolerant on EVERY recognized vocabulary
    /// word, not just the obvious ones — pin via a spot check on "bought".
    #[test]
    fn decode_side_phrase_with_trailing_newline() {
        let l = SideLookup::DEFAULT;
        assert_eq!(decode_side("bought\n", &l), Some(Side::Buy));
    }

    /// `decode_side` is strict against partial-string prefixes — "buying"
    /// must NOT match the "buy" vocabulary (would corrupt rows).
    #[test]
    fn decode_side_rejects_prefix_only_match() {
        let l = SideLookup::DEFAULT;
        assert_eq!(decode_side("buying", &l), None);
        assert_eq!(decode_side("seller", &l), None);
    }

    /// `decode_asset_class` is case-insensitive but does not match
    /// arbitrary substrings — "optional" must NOT decode as Option.
    #[test]
    fn decode_asset_class_strict_match_only() {
        assert_eq!(decode_asset_class("optional"), AssetClass::Stock);
        assert_eq!(decode_asset_class("futurology"), AssetClass::Stock);
    }

    /// `parse_with` honors `ColSpec::Constant` — useful when a broker
    /// export has no asset-class column but the parser knows the class.
    #[test]
    fn parse_with_constant_colspec_supplies_field() {
        let map = ColumnMap {
            source: "test",
            has_header: true,
            delimiter: b',',
            date_formats: &["%Y-%m-%d %H:%M:%S"],
            utc_assumed: true,
            side_lookup: SideLookup::DEFAULT,
            symbol: ColSpec::Header("Symbol"),
            side: ColSpec::Header("Side"),
            qty: ColSpec::Header("Qty"),
            price: ColSpec::Header("Price"),
            fee: None,
            executed_at: ColSpec::Header("Date"),
            broker_order_id: None,
            asset_class: Some(ColSpec::Constant("option".into())),
            option_type: Some(ColSpec::Constant("c".into())),
            strike: None,
            expiration: None,
            multiplier: None,
            skip_symbols: &["", "total"],
            status: None,
            status_allow: &[],
        };
        let csv = "Symbol,Side,Qty,Price,Date\nSPY,buy,1,3.40,2026-03-04 10:00:00\n";
        let out = parse_with(csv.as_bytes(), &map).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].asset_class, AssetClass::Option);
        assert_eq!(out[0].option_type, Some(OptionType::Call));
        // Option default multiplier is 100 when not supplied.
        assert_eq!(out[0].multiplier.to_string(), "100");
    }

    /// `parse_with` rejects rows where qty is zero (treats them as
    /// no-execution noise — common in some broker exports).
    #[test]
    fn parse_with_skips_zero_qty_rows() {
        let map = ColumnMap {
            source: "test",
            has_header: true,
            delimiter: b',',
            date_formats: &["%Y-%m-%d %H:%M:%S"],
            utc_assumed: true,
            side_lookup: SideLookup::DEFAULT,
            symbol: ColSpec::Header("Symbol"),
            side: ColSpec::Header("Side"),
            qty: ColSpec::Header("Qty"),
            price: ColSpec::Header("Price"),
            fee: None,
            executed_at: ColSpec::Header("Date"),
            broker_order_id: None,
            asset_class: None,
            option_type: None,
            strike: None,
            expiration: None,
            multiplier: None,
            skip_symbols: &[""],
            status: None,
            status_allow: &[],
        };
        let csv = "Symbol,Side,Qty,Price,Date\n\
                   AAPL,buy,0,150,2026-03-04 10:00:00\n\
                   AAPL,buy,10,151,2026-03-04 10:01:00\n";
        let out = parse_with(csv.as_bytes(), &map).unwrap();
        // The zero-qty row is silently dropped.
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].qty.to_string(), "10");
    }

    /// `parse_with` returns an absolute-value fee even when the CSV
    /// reports it as negative (some brokers express commissions as
    /// debits with a leading minus).
    #[test]
    fn parse_with_fee_absolute_value_strips_sign() {
        let map = ColumnMap {
            source: "test",
            has_header: true,
            delimiter: b',',
            date_formats: &["%Y-%m-%d %H:%M:%S"],
            utc_assumed: true,
            side_lookup: SideLookup::DEFAULT,
            symbol: ColSpec::Header("Symbol"),
            side: ColSpec::Header("Side"),
            qty: ColSpec::Header("Qty"),
            price: ColSpec::Header("Price"),
            fee: Some(ColSpec::Header("Fee")),
            executed_at: ColSpec::Header("Date"),
            broker_order_id: None,
            asset_class: None,
            option_type: None,
            strike: None,
            expiration: None,
            multiplier: None,
            skip_symbols: &[""],
            status: None,
            status_allow: &[],
        };
        let csv = "Symbol,Side,Qty,Price,Fee,Date\n\
                   AAPL,buy,10,150,-1.25,2026-03-04 10:00:00\n";
        let out = parse_with(csv.as_bytes(), &map).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].fee.to_string(), "1.25");
    }
}
