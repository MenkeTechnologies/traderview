//! Amazon order export parser.
//!
//! The real-world Amazon CSV/XLSX export the user supplied has no header row
//! and 23 columns. The schema is position-based (the headers were stripped
//! during a pre-filter pass) and identical between the .csv and .xlsx
//! versions:
//!
//! | idx | meaning                                  | example                            |
//! |-----|------------------------------------------|------------------------------------|
//! |  0  | order datetime (ISO 8601 UTC)            | 2024-12-30T05:16:42Z               |
//! |  1  | gift-related flag                        | Not Applicable                     |
//! |  2  | currency                                 | USD                                |
//! |  3  | item subtotal (pre-tax)                  | 143.09                             |
//! |  4  | tax                                      | 7.9                                |
//! |  5  | shipping                                 | 0                                  |
//! |  6  | discount (signed, sometimes Excel-quoted)| '-11.45'  or  -11.45  or  0        |
//! |  7  | **total charged**                        | 139.54                             |
//! |  8  | subtotal (duplicate)                     |                                    |
//! |  9  | tax (duplicate)                          |                                    |
//! | 10  | ASIN                                     | B0D6R561M9                         |
//! | 11  | condition                                | New                                |
//! | 12  | quantity                                 | 1                                  |
//! | 13  | payment method                           | Visa - 9314                        |
//! | 14  | order status                             | Closed                             |
//! | 15  | shipment status                          | Shipped                            |
//! | 16  | ship date                                | 2024-12-30T19:49:21.462Z           |
//! | 17  | carrier code                             | next-1dc                           |
//! | 18  | buyer name + address                     | Jacob A Menke 5611 ...             |
//! | 19  | ship-to name + address                   | Jacob A Menke 4847 ...             |
//! | 20  | tracking id                              | AMZN_US(TBA318400350499)           |
//! | 21  | item title                               | Kerty Garage Floor Mat ...         |
//! | 22  | user-applied tag (optional)              | rental                             |
//!
//! Sign convention: amount is set negative (expense). Refunds in the original
//! export show up with discount in col 6; we follow the simple rule "amount =
//! -col[7]" because col[7] is the actual charge after discount.
//!
//! Rows with empty col[0] or non-numeric col[7] are skipped silently — the
//! file contains blank separator rows interleaving the data.

use crate::{normalize::normalize, sheet, ExpenseSource, ImportError, ParsedTransaction, Parser};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde_json::json;
use std::str::FromStr;

const COL_DATE: usize = 0;
const COL_CURRENCY: usize = 2;
const COL_TOTAL: usize = 7;
const COL_ASIN: usize = 10;
const COL_QTY: usize = 12;
const COL_PAYMENT: usize = 13;
const COL_TRACKING: usize = 20;
const COL_TITLE: usize = 21;
const COL_TAG: usize = 22;
const MIN_COLS: usize = COL_TITLE + 1;

pub struct AmazonParser;

impl Parser for AmazonParser {
    fn source(&self) -> ExpenseSource {
        ExpenseSource::Amazon
    }

    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedTransaction>, ImportError> {
        let rows = sheet::rows(bytes)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            if row.len() < MIN_COLS {
                continue;
            }
            let date_raw = row.get(COL_DATE).map(String::as_str).unwrap_or("").trim();
            if date_raw.is_empty() {
                continue;
            }
            let posted_at = match parse_amazon_datetime(date_raw) {
                Some(dt) => dt,
                None => continue,
            };
            let total_raw = row.get(COL_TOTAL).map(String::as_str).unwrap_or("").trim();
            let charged = match Decimal::from_str(&clean_money(total_raw)) {
                Ok(d) if !d.is_zero() => d,
                _ => continue,
            };
            // Expense → negative.
            let amount = -charged;

            let title = row.get(COL_TITLE).cloned().unwrap_or_default();
            let merchant_raw = if title.is_empty() {
                "AMAZON.COM".to_string()
            } else {
                format!("AMAZON.COM — {}", truncate(&title, 80))
            };
            let merchant_normalized = normalize(if title.is_empty() {
                "amazon.com"
            } else {
                "amazon.com"
            });

            let tag = row.get(COL_TAG).cloned().unwrap_or_default();
            let description = build_description(row);
            let currency = row
                .get(COL_CURRENCY)
                .cloned()
                .unwrap_or_else(|| "USD".into());

            let raw = json!({
                "source": "amazon",
                "order_date": date_raw,
                "asin": row.get(COL_ASIN).cloned().unwrap_or_default(),
                "quantity": row.get(COL_QTY).cloned().unwrap_or_default(),
                "payment_method": row.get(COL_PAYMENT).cloned().unwrap_or_default(),
                "tracking": row.get(COL_TRACKING).cloned().unwrap_or_default(),
                "title": title,
                "user_tag": tag,
            });

            out.push(ParsedTransaction {
                posted_at,
                amount,
                currency: currency.trim().to_string(),
                merchant_raw,
                merchant_normalized,
                description,
                raw,
            });
        }
        Ok(out)
    }
}

fn parse_amazon_datetime(s: &str) -> Option<DateTime<Utc>> {
    // Real-world export uses two shapes: "2024-12-30T05:16:42Z" and
    // "2024-12-30T19:49:21.462Z". chrono parses both via DateTime::parse_from_rfc3339.
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // calamine may have surfaced an Excel serial date as a float string —
    // try parsing as f64 days-since-1900.
    if let Ok(serial) = s.parse::<f64>() {
        return excel_serial_to_utc(serial);
    }
    // Final fallback: "YYYY-MM-DD".
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let naive = NaiveDateTime::new(d, NaiveTime::from_hms_opt(0, 0, 0)?);
        return Some(Utc.from_utc_datetime(&naive));
    }
    None
}

fn excel_serial_to_utc(serial: f64) -> Option<DateTime<Utc>> {
    // Excel epoch is 1899-12-30 (after accounting for the 1900-leap-year bug).
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    let days = serial.trunc() as i64;
    let frac = serial.fract();
    let secs_of_day = (frac * 86_400.0).round() as i64;
    let date = epoch.checked_add_signed(chrono::Duration::days(days))?;
    let naive = date.and_hms_opt(0, 0, 0)?;
    let naive = naive.checked_add_signed(chrono::Duration::seconds(secs_of_day))?;
    Some(Utc.from_utc_datetime(&naive))
}

fn clean_money(s: &str) -> String {
    // Amazon CSV escapes negative discounts as Excel-friendly `'-11.45'`. Strip
    // any surrounding apostrophes/quotes and currency symbols before parsing.
    s.chars()
        .filter(|c| matches!(c, '0'..='9' | '.' | '-'))
        .collect()
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max - 1).collect();
    out.push('…');
    out
}

fn build_description(row: &[String]) -> String {
    let mut parts = Vec::new();
    if let Some(asin) = row.get(COL_ASIN) {
        if !asin.trim().is_empty() {
            parts.push(format!("ASIN {}", asin.trim()));
        }
    }
    if let Some(qty) = row.get(COL_QTY) {
        if !qty.trim().is_empty() {
            parts.push(format!("qty {}", qty.trim()));
        }
    }
    if let Some(tag) = row.get(COL_TAG) {
        if !tag.trim().is_empty() {
            parts.push(format!("tag {}", tag.trim()));
        }
    }
    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 23-column row exactly matching the real-world export shape.
    fn synthetic_row() -> String {
        // Use a printable line that survives CSV escaping.
        "2024-12-30T05:16:42Z,Not Applicable,USD,143.09,7.9,0,'-11.45',139.54,143.09,7.9,B0D6R561M9,New,1,Visa - 9314,Closed,Shipped,2024-12-30T19:49:21.462Z,next-1dc,Buyer,ShipTo,AMZN_US(TBA),Garage Mat,rental".into()
    }

    #[test]
    fn parses_one_row() {
        let csv = format!(",,,,,,,,,,,,,,,,,,,,,,\n,,,,,,,,,,,,,,,,,,,,,,\n{}\n", synthetic_row());
        let r = AmazonParser.parse(csv.as_bytes()).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].amount, Decimal::new(-13954, 2));
        assert_eq!(r[0].currency, "USD");
        assert!(r[0].merchant_raw.starts_with("AMAZON.COM"));
        assert_eq!(r[0].merchant_normalized, "amazon.com");
        assert!(r[0].description.contains("ASIN B0D6R561M9"));
        assert!(r[0].description.contains("tag rental"));
    }

    #[test]
    fn skips_blank_separators() {
        let csv = ",,,,,,,,,,,,,,,,,,,,,,\n,,,,,,,,,,,,,,,,,,,,,,";
        let r = AmazonParser.parse(csv.as_bytes()).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn keeps_negative_amount_for_expense() {
        let csv = format!("{}\n", synthetic_row());
        let r = AmazonParser.parse(csv.as_bytes()).unwrap();
        assert!(r[0].amount.is_sign_negative());
    }

    #[test]
    fn handles_quoted_negative_discount() {
        // Discount column has the Excel-escape form `'-11.45'`. We don't parse
        // the discount column directly, but verify the clean_money helper.
        assert_eq!(clean_money("'-11.45'"), "-11.45");
        assert_eq!(clean_money("$1,234.56"), "1234.56");
    }
}
