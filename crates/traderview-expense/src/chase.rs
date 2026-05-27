//! Chase credit card activity parser.
//!
//! Real export header (verified against the user's 2024 export):
//!
//! ```text
//! Transaction Date,Post Date,Description,Category,Type,Amount,Memo
//! 12/31/2024,12/31/2024,Amazon.com*ZP9JI1KZ2,Shopping,Sale,-1.11,
//! ```
//!
//! Sign convention already matches ours: negative = purchase (money out),
//! positive = payment/refund (money in). No flip needed.
//!
//! Chase's `Category` column is their own taxonomy ("Shopping", "Gas", "Food
//! & Drink") — useful as a hint but not a substitute for Schedule C mapping.
//! We stash it in `raw` so the UI can show it, but `category_code` stays NULL
//! until the user's merchant_rules fire.
//!
//! `Type` is `Sale`, `Payment`, `Return`, `Fee`, etc. We don't filter on it —
//! the sign on Amount already encodes the right semantics.

use crate::{normalize::normalize, sheet, ExpenseSource, ImportError, ParsedTransaction, Parser};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;

pub struct ChaseParser;

impl Parser for ChaseParser {
    fn source(&self) -> ExpenseSource {
        ExpenseSource::Chase
    }

    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedTransaction>, ImportError> {
        let rows = sheet::rows(bytes)?;
        let header_idx = rows.iter().position(|r| {
            r.iter().any(|c| c.trim().eq_ignore_ascii_case("Transaction Date"))
                && r.iter().any(|c| c.trim().eq_ignore_ascii_case("Amount"))
        });
        let header_idx = header_idx.ok_or_else(|| {
            ImportError::Parse(
                "chase: header row with 'Transaction Date' + 'Amount' not found — \
                 wrong export shape?"
                    .into(),
            )
        })?;

        let mut idx: HashMap<String, usize> = HashMap::new();
        for (i, cell) in rows[header_idx].iter().enumerate() {
            idx.insert(cell.trim().to_ascii_lowercase(), i);
        }
        let col = |name: &str| idx.get(&name.to_ascii_lowercase()).copied();
        let col_date = col("transaction date").ok_or_else(missing("transaction date"))?;
        let col_post = col("post date");
        let col_desc = col("description").ok_or_else(missing("description"))?;
        let col_cat = col("category");
        let col_type = col("type");
        let col_amount = col("amount").ok_or_else(missing("amount"))?;
        let col_memo = col("memo");

        let mut out = Vec::with_capacity(rows.len() - header_idx);
        for row in rows.iter().skip(header_idx + 1) {
            if row.len() <= col_amount.max(col_desc).max(col_date) {
                continue;
            }
            let date_raw = row[col_date].trim();
            let amount_raw = row[col_amount].trim();
            let desc = row[col_desc].trim();
            if date_raw.is_empty() || amount_raw.is_empty() {
                continue;
            }
            let posted_at = match parse_chase_date(date_raw) {
                Some(dt) => dt,
                None => continue,
            };
            let amount = match Decimal::from_str(&clean(amount_raw)) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let raw = json!({
                "source": "chase",
                "transaction_date": date_raw,
                "post_date": col_post.and_then(|i| row.get(i)).cloned().unwrap_or_default(),
                "chase_category": col_cat.and_then(|i| row.get(i)).cloned().unwrap_or_default(),
                "type": col_type.and_then(|i| row.get(i)).cloned().unwrap_or_default(),
                "memo": col_memo.and_then(|i| row.get(i)).cloned().unwrap_or_default(),
            });

            out.push(ParsedTransaction {
                posted_at,
                amount,
                currency: "USD".into(),
                merchant_raw: desc.to_string(),
                merchant_normalized: normalize(desc),
                description: col_memo
                    .and_then(|i| row.get(i))
                    .cloned()
                    .unwrap_or_default(),
                raw,
            });
        }
        Ok(out)
    }
}

fn missing(name: &'static str) -> impl Fn() -> ImportError {
    move || ImportError::Parse(format!("chase: missing required column '{name}'"))
}

fn parse_chase_date(s: &str) -> Option<chrono::DateTime<Utc>> {
    if let Ok(d) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        let naive = NaiveDateTime::new(d, NaiveTime::from_hms_opt(0, 0, 0)?);
        return Some(Utc.from_utc_datetime(&naive));
    }
    if let Ok(serial) = s.parse::<f64>() {
        let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
        let d = epoch.checked_add_signed(chrono::Duration::days(serial.trunc() as i64))?;
        let naive = NaiveDateTime::new(d, NaiveTime::from_hms_opt(0, 0, 0)?);
        return Some(Utc.from_utc_datetime(&naive));
    }
    None
}

fn clean(s: &str) -> String {
    s.chars()
        .filter(|c| matches!(c, '0'..='9' | '.' | '-'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Transaction Date,Post Date,Description,Category,Type,Amount,Memo
12/31/2024,12/31/2024,Amazon.com*ZP9JI1KZ2,Shopping,Sale,-1.11,
12/30/2024,12/31/2024,AMAZON MKTPL*ZP9W726B2,Shopping,Sale,-139.54,
12/15/2024,12/15/2024,AUTOMATIC PAYMENT - THANK,,Payment,2132.25,";

    #[test]
    fn parses_three_rows() {
        let r = ChaseParser.parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn preserves_signs() {
        let r = ChaseParser.parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(r[0].amount, Decimal::new(-111, 2));
        assert_eq!(r[2].amount, Decimal::new(213225, 2)); // payment, positive
    }

    #[test]
    fn raw_contains_chase_category() {
        let r = ChaseParser.parse(SAMPLE.as_bytes()).unwrap();
        let cat = r[0].raw.get("chase_category").and_then(|v| v.as_str()).unwrap();
        assert_eq!(cat, "Shopping");
    }

    #[test]
    fn merchant_normalized_strips_pound_id() {
        let r = ChaseParser.parse(SAMPLE.as_bytes()).unwrap();
        // "Amazon.com*ZP9JI1KZ2" → normalize strips the *ID glued suffix.
        assert_eq!(r[0].merchant_normalized, "amazon.com");
    }
}
