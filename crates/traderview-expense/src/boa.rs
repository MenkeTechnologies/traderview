//! Bank of America checking/savings statement parser.
//!
//! The real export (BoA `Download transactions` from checking) has two sections:
//!
//! ```text
//! Description,,Summary Amt.
//! Beginning balance as of 01/01/2024,,"4,523.69"
//! Total credits,,"185,585.58"
//! Total debits,,"-184,924.97"
//! Ending balance as of 12/31/2024,,"5,184.30"
//!
//! Date,Description,Amount,Running Bal.
//! 01/01/2024,Beginning balance as of 01/01/2024,,"4,523.69"
//! 01/02/2024,"Transfer Robinhood Securities","689.50","5,213.19"
//! ...
//! ```
//!
//! We skip until we hit a row whose first cell is `Date` and second cell is
//! `Description` — that's the transactions header. Everything after gets
//! parsed positionally.
//!
//! Amount column already has the correct sign (negative = debit / money out,
//! positive = credit / money in), matching our schema convention exactly.
//! Rows with empty `Amount` (the "Beginning balance" marker) are dropped.
//!
//! Same logic for the .xlsx export — sheet helper produces the same `Vec<Vec<String>>`.

use crate::{normalize::normalize, sheet, ExpenseSource, ImportError, ParsedTransaction, Parser};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde_json::json;
use std::str::FromStr;

pub struct BofaParser;

impl Parser for BofaParser {
    fn source(&self) -> ExpenseSource {
        ExpenseSource::Bofa
    }

    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedTransaction>, ImportError> {
        let rows = sheet::rows(bytes)?;
        let header_idx = rows
            .iter()
            .position(|r| {
                r.len() >= 3
                    && r[0].trim().eq_ignore_ascii_case("Date")
                    && r[1].trim().eq_ignore_ascii_case("Description")
                    && r[2].trim().eq_ignore_ascii_case("Amount")
            })
            .ok_or_else(|| {
                ImportError::Parse(
                    "bofa: couldn't find 'Date,Description,Amount,Running Bal.' header — \
                     wrong export format?"
                        .into(),
                )
            })?;

        let mut out = Vec::with_capacity(rows.len() - header_idx);
        for row in rows.iter().skip(header_idx + 1) {
            if row.len() < 3 {
                continue;
            }
            let date_raw = row[0].trim();
            let desc = row[1].trim();
            let amount_raw = row[2].trim();
            if date_raw.is_empty() || amount_raw.is_empty() {
                continue; // beginning-balance marker, blank padding row
            }
            let posted_at = match parse_bofa_date(date_raw) {
                Some(dt) => dt,
                None => continue,
            };
            let amount = match Decimal::from_str(&clean_amount(amount_raw)) {
                Ok(d) => d,
                Err(_) => continue,
            };
            let merchant_raw = desc.to_string();
            let merchant_normalized = normalize(desc);
            let raw = json!({
                "source": "bofa",
                "raw_date": date_raw,
                "running_balance": row.get(3).cloned().unwrap_or_default(),
            });
            out.push(ParsedTransaction {
                posted_at,
                amount,
                currency: "USD".into(),
                merchant_raw,
                merchant_normalized,
                description: String::new(),
                raw,
            });
        }
        Ok(out)
    }
}

fn parse_bofa_date(s: &str) -> Option<chrono::DateTime<Utc>> {
    // BoA uses MM/DD/YYYY in both CSV and xlsx (xlsx surfaces it as a string
    // not a serial — the cells were saved as text by the export).
    if let Ok(d) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        let naive = NaiveDateTime::new(d, NaiveTime::from_hms_opt(0, 0, 0)?);
        return Some(Utc.from_utc_datetime(&naive));
    }
    // Excel serial fallback for cells that came through as numeric.
    if let Ok(serial) = s.parse::<f64>() {
        let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
        let d = epoch.checked_add_signed(chrono::Duration::days(serial.trunc() as i64))?;
        let naive = NaiveDateTime::new(d, NaiveTime::from_hms_opt(0, 0, 0)?);
        return Some(Utc.from_utc_datetime(&naive));
    }
    None
}

fn clean_amount(s: &str) -> String {
    s.chars()
        .filter(|c| matches!(c, '0'..='9' | '.' | '-'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Description,,Summary Amt.
Beginning balance as of 01/01/2024,,\"4,523.69\"
Total credits,,\"185,585.58\"
Total debits,,\"-184,924.97\"
Ending balance as of 12/31/2024,,\"5,184.30\"

Date,Description,Amount,Running Bal.
01/01/2024,Beginning balance as of 01/01/2024,,\"4,523.69\"
01/02/2024,\"Transfer Robinhood Securities\",\"689.50\",\"5,213.19\"
01/02/2024,\"Zelle payment to TAUREAN COLLINS Conf# o0nqkedse\",\"-60.00\",\"4,991.19\"";

    #[test]
    fn skips_summary_and_balance_row() {
        let r = BofaParser.parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn preserves_signed_amount() {
        let r = BofaParser.parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(r[0].amount, Decimal::new(68950, 2)); // credit, positive
        assert_eq!(r[1].amount, Decimal::new(-6000, 2)); // debit, negative
    }

    #[test]
    fn merchant_normalized_lowercased() {
        let r = BofaParser.parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(r[0].merchant_normalized, "transfer robinhood securities");
    }

    #[test]
    fn errors_without_transactions_header() {
        let bad = "Description,,Summary Amt.\nBeginning balance,,1000";
        let err = BofaParser.parse(bad.as_bytes()).unwrap_err();
        assert!(format!("{err}").contains("Date,Description,Amount"));
    }
}
