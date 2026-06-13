//! Generic column-mapped spreadsheet importer.
//!
//! The fixed-source parsers (Amazon, BoA, Chase, Apple Card) each know their
//! export's exact schema. Real archives also hold *bespoke* expense sheets —
//! a handyman's job/amount/date log, a "truck expenses" workbook, a rental
//! income tab — with arbitrary column layouts no fixed parser fits. This
//! importer takes an explicit column mapping (which column is the date, the
//! amount, the description) and parses any CSV / XLSX / ODS through the
//! shared `sheet::rows`.
//!
//! Sign convention: bespoke logs usually list amounts as bare positives
//! ("amount" = what was paid). Set `negate_amount` to flip them to the
//! expense convention (negative = money out). Rows whose mapped date or
//! amount don't parse are skipped, so title rows ("Charles Scharzenwalder")
//! and blank separators fall out without failing the import. Pure compute.

use crate::{normalize::normalize, sheet, ImportError, ParsedTransaction};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenericMapping {
    /// 0-based column holding the transaction date.
    pub date_col: usize,
    /// 0-based column holding the amount.
    pub amount_col: usize,
    /// 0-based column holding the description / merchant.
    pub description_col: usize,
    /// Optional 0-based column holding a category label (kept in `raw`).
    #[serde(default)]
    pub category_col: Option<usize>,
    /// Skip the first row as a header (default true).
    #[serde(default = "default_true")]
    pub has_header: bool,
    /// Explicit date format (chrono strftime, e.g. "%m/%d/%Y"). When absent,
    /// a set of common formats and the Excel serial are tried.
    #[serde(default)]
    pub date_format: Option<String>,
    /// Negate amounts — set when the sheet lists expenses as positive numbers.
    #[serde(default)]
    pub negate_amount: bool,
    /// Currency code for every row (default USD).
    #[serde(default)]
    pub currency: Option<String>,
}

/// Common date layouts tried when no explicit `date_format` is given.
const COMMON_DATE_FORMATS: &[&str] =
    &["%m/%d/%Y", "%Y-%m-%d", "%m/%d/%y", "%m-%d-%Y", "%d/%m/%Y", "%b %d, %Y"];

pub fn parse_generic(
    bytes: &[u8],
    m: &GenericMapping,
) -> Result<Vec<ParsedTransaction>, ImportError> {
    let rows = sheet::rows(bytes)?;
    let currency = m.currency.clone().unwrap_or_else(|| "USD".into());
    let skip = if m.has_header { 1 } else { 0 };

    let mut out = Vec::new();
    for row in rows.iter().skip(skip) {
        let date_raw = row.get(m.date_col).map(|s| s.trim()).unwrap_or("");
        let amount_raw = row.get(m.amount_col).map(|s| s.trim()).unwrap_or("");
        if date_raw.is_empty() || amount_raw.is_empty() {
            continue;
        }
        let posted_at = match parse_date(date_raw, m.date_format.as_deref()) {
            Some(dt) => dt,
            None => continue,
        };
        let mut amount = match Decimal::from_str(&clean_amount(amount_raw)) {
            Ok(d) => d,
            Err(_) => continue,
        };
        if m.negate_amount {
            amount = -amount;
        }

        let desc = row.get(m.description_col).map(|s| s.trim()).unwrap_or("").to_string();
        let category = m
            .category_col
            .and_then(|c| row.get(c))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let raw = json!({
            "source": "generic",
            "raw_date": date_raw,
            "raw_amount": amount_raw,
            "category": category,
        });

        out.push(ParsedTransaction {
            posted_at,
            amount,
            currency: currency.clone(),
            merchant_raw: desc.clone(),
            merchant_normalized: normalize(&desc),
            description: category,
            raw,
        });
    }
    Ok(out)
}

fn parse_date(s: &str, explicit: Option<&str>) -> Option<chrono::DateTime<Utc>> {
    let to_utc = |d: NaiveDate| -> Option<chrono::DateTime<Utc>> {
        let naive = NaiveDateTime::new(d, NaiveTime::from_hms_opt(0, 0, 0)?);
        Some(Utc.from_utc_datetime(&naive))
    };
    if let Some(fmt) = explicit {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return to_utc(d);
        }
    }
    for fmt in COMMON_DATE_FORMATS {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            // chrono's %Y greedily accepts a 2-digit string ("08/01/23" →
            // year 23), shadowing the %y format that would read it as 2023.
            // Reject the sub-millennium year so the loop falls through to %y.
            if d.year() >= 1000 {
                return to_utc(d);
            }
        }
    }
    // Excel serial date (cells that surfaced as a number through calamine).
    if let Ok(serial) = s.parse::<f64>() {
        let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
        let d = epoch.checked_add_signed(chrono::Duration::days(serial.trunc() as i64))?;
        return to_utc(d);
    }
    None
}

fn clean_amount(s: &str) -> String {
    // Keep digits, dot, leading minus; tolerate "$1,234.56", "(50.00)" parens.
    let neg = s.contains('(') && s.contains(')');
    let mut cleaned: String = s.chars().filter(|c| matches!(c, '0'..='9' | '.' | '-')).collect();
    if neg && !cleaned.starts_with('-') {
        cleaned.insert(0, '-');
    }
    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map() -> GenericMapping {
        GenericMapping {
            date_col: 0,
            amount_col: 1,
            description_col: 2,
            category_col: None,
            has_header: true,
            date_format: None,
            negate_amount: false,
            currency: None,
        }
    }

    #[test]
    fn parses_basic_three_column_sheet() {
        let csv = "date,amount,description\n10/18/2024,200,444 repair\n10/25/2024,410,land clearing\n";
        let r = parse_generic(csv.as_bytes(), &map()).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].amount, Decimal::new(20000, 2));
        assert_eq!(r[0].merchant_raw, "444 repair");
        assert_eq!(r[0].currency, "USD");
    }

    #[test]
    fn negate_flips_positive_expenses_to_negative() {
        let csv = "date,amount,description\n10/18/2024,200,repair\n";
        let r = parse_generic(csv.as_bytes(), &GenericMapping { negate_amount: true, ..map() }).unwrap();
        assert_eq!(r[0].amount, Decimal::new(-20000, 2));
    }

    #[test]
    fn no_header_parses_first_row() {
        let csv = "10/18/2024,200,repair\n";
        let r = parse_generic(csv.as_bytes(), &GenericMapping { has_header: false, ..map() }).unwrap();
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn skips_rows_that_dont_parse() {
        // Title row + blank row + a good row. The bespoke handyman log shape.
        let csv = "Charles Scharzenwalder,,\njob,amount,date\n10/18/2024,200,444 repair\n,,\n";
        // date in col 0, amount col 1, desc col 2; header skip drops the title.
        let r = parse_generic(csv.as_bytes(), &map()).unwrap();
        // "job,amount,date" row: date_col0="job" → no date → skipped; blank skipped.
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].amount, Decimal::new(20000, 2));
    }

    #[test]
    fn two_digit_year_resolves_to_correct_century() {
        // Real rental sheet uses "08/01/23" — must parse as 2023, not 0023.
        let csv = "date,amount,description\n08/01/23,900,rent\n";
        let r = parse_generic(csv.as_bytes(), &map()).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].posted_at.date_naive().to_string(), "2023-08-01");
    }

    #[test]
    fn explicit_date_format_honored() {
        let csv = "date,amount,description\n2024-10-18,200,repair\n";
        let r = parse_generic(
            csv.as_bytes(),
            &GenericMapping { date_format: Some("%Y-%m-%d".into()), ..map() },
        )
        .unwrap();
        assert_eq!(r.len(), 1);
        let expected = parse_date("2024-10-18", Some("%Y-%m-%d")).unwrap();
        assert_eq!(r[0].posted_at, expected);
    }

    #[test]
    fn category_column_captured_in_raw_and_description() {
        let csv = "date,amount,description,category\n10/18/2024,200,repair,Repairs\n";
        let r = parse_generic(
            csv.as_bytes(),
            &GenericMapping { category_col: Some(3), ..map() },
        )
        .unwrap();
        assert_eq!(r[0].raw["category"], "Repairs");
        assert_eq!(r[0].description, "Repairs");
    }

    #[test]
    fn parens_and_currency_symbols_parse() {
        assert_eq!(clean_amount("$1,234.56"), "1234.56");
        assert_eq!(clean_amount("(50.00)"), "-50.00");
        assert_eq!(clean_amount("-12.00"), "-12.00");
    }
}
