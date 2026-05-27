//! Apple Card monthly statement parser (PDF).
//!
//! Apple Card's Wallet export is CSV, but the user's archive holds the
//! printed-style monthly PDFs — those are born-digital with a clean text
//! layer that lopdf reads without OCR.
//!
//! Layout (verified against real `Apple Card Statement - <Month> 2024.pdf`):
//!
//! ```text
//! Payments
//! Date
//! Description
//! Amount
//! 01/12/2024
//! ACH Deposit Internet transfer from account ending in 1870
//! -$802.31
//! ...
//! Total payments for this period
//! -$843.84
//!
//! Transactions
//! Date
//! Description
//! Daily Cash
//! Amount
//! 12/31/2023
//! HARBOR FREIGHT TOOLS 12399 ELLSWORTH RD YPSILANTI 48197 MI USA
//! 2%
//! $10.65
//! $532.63
//! ...
//! ```
//!
//! Each row in the Transactions section is a 5-line block; each row in the
//! Payments section is a 3-line block. We accumulate inside whichever section
//! we're currently in, terminating on the corresponding "Total ..." line.
//!
//! Sign convention on Apple Card PDFs:
//!   * Transactions section prints purchases as POSITIVE dollar amounts
//!     (because credit-card statements present "amount owed", not "money
//!     out"). We negate to align with the expense convention.
//!   * Payments section prints payments-from-bank as NEGATIVE dollar amounts
//!     (they reduce the balance). From the credit-card account's POV that's
//!     a CREDIT in (positive). We negate to flip.
//!
//! Net rule: **negate every dollar amount the PDF prints**.
//!
//! If a user later supplies a CSV export from Wallet (`Wallet > Card > ··· >
//! Export Transactions`), it can be plugged in here by detecting magic bytes
//! and dispatching to a sibling CSV path. Not implemented today since the
//! user's archive only has PDFs.

use crate::{normalize::normalize, sheet, ExpenseSource, ImportError, ParsedTransaction, Parser};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use regex::Regex;
use rust_decimal::Decimal;
use serde_json::json;
use std::str::FromStr;
use std::sync::OnceLock;

pub struct AppleCardParser;

impl Parser for AppleCardParser {
    fn source(&self) -> ExpenseSource {
        ExpenseSource::AppleCard
    }

    fn parse(&self, bytes: &[u8]) -> Result<Vec<ParsedTransaction>, ImportError> {
        // Magic-byte dispatch: PDF (%PDF) vs sheet (CSV / xlsx).
        if bytes.len() >= 4 && &bytes[0..4] == b"%PDF" {
            return parse_pdf(bytes);
        }
        // CSV / xlsx path is not implemented yet — the user's archive only has
        // PDFs. Surface a clear message so future callers know to plug it in.
        let _ = sheet::detect_kind(bytes);
        Err(ImportError::Unsupported(
            "apple card: only PDF statements are supported today. If you have a Wallet CSV \
             export, please open an issue with a sample so the CSV path can be added."
                .into(),
        ))
    }
}

fn parse_pdf(bytes: &[u8]) -> Result<Vec<ParsedTransaction>, ImportError> {
    let doc = lopdf::Document::load_mem(bytes)
        .map_err(|e| ImportError::Parse(format!("apple pdf load: {e}")))?;
    let page_count = doc.get_pages().len() as u32;
    let mut full = String::new();
    for p in 1..=page_count {
        if let Ok(t) = doc.extract_text(&[p]) {
            full.push_str(&t);
            if !t.ends_with('\n') {
                full.push('\n');
            }
        }
    }

    let lines: Vec<&str> = full.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

    let mut out = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i];
        if line.eq_ignore_ascii_case("Payments") {
            i = parse_payments_block(&lines, i + 1, &mut out);
            continue;
        }
        if line.eq_ignore_ascii_case("Transactions") {
            i = parse_transactions_block(&lines, i + 1, &mut out);
            continue;
        }
        i += 1;
    }
    Ok(out)
}

/// Returns the index AFTER the section terminator.
fn parse_payments_block(
    lines: &[&str],
    mut i: usize,
    out: &mut Vec<ParsedTransaction>,
) -> usize {
    // Skip the column-header lines that appear right after "Payments".
    while i < lines.len() && is_header_label(lines[i]) {
        i += 1;
    }
    while i + 2 < lines.len() {
        let date = lines[i];
        let desc = lines[i + 1];
        let amount = lines[i + 2];
        if is_section_end(date, desc) {
            return i;
        }
        let date_p = date_re().is_match(date);
        let amt_p = signed_amount_re().is_match(amount);
        if !date_p || !amt_p {
            // A non-block line snuck in (e.g. multi-line description).
            // Advance one and try again.
            i += 1;
            continue;
        }
        if let Some(tx) = make_tx(date, desc, amount, /*is_payment=*/ true) {
            out.push(tx);
        }
        i += 3;
    }
    i
}

fn parse_transactions_block(
    lines: &[&str],
    mut i: usize,
    out: &mut Vec<ParsedTransaction>,
) -> usize {
    while i < lines.len() && is_header_label(lines[i]) {
        i += 1;
    }
    while i + 4 < lines.len() {
        let date = lines[i];
        let desc = lines[i + 1];
        let cash_pct = lines[i + 2];
        let cash_amt = lines[i + 3];
        let amount = lines[i + 4];

        if is_section_end(date, desc) {
            return i;
        }
        let date_p = date_re().is_match(date);
        let amt_p = signed_amount_re().is_match(amount);
        let pct_p = cash_pct.contains('%');
        let cash_p = signed_amount_re().is_match(cash_amt);
        if !(date_p && amt_p && pct_p && cash_p) {
            i += 1;
            continue;
        }
        if let Some(mut tx) = make_tx(date, desc, amount, /*is_payment=*/ false) {
            tx.raw["daily_cash_pct"] = cash_pct.into();
            tx.raw["daily_cash_amount"] = cash_amt.into();
            out.push(tx);
        }
        i += 5;
    }
    i
}

fn is_header_label(s: &str) -> bool {
    matches!(
        s.to_ascii_lowercase().as_str(),
        "date" | "description" | "amount" | "daily cash"
    )
}

fn is_section_end(date: &str, desc: &str) -> bool {
    let l = date.to_ascii_lowercase();
    if l.starts_with("total payments") || l.starts_with("total transactions")
        || l.starts_with("total interest") || l.starts_with("interest charged")
        || l.starts_with("interest charges") || l.starts_with("transactions")
        || l.starts_with("payments")
    {
        return true;
    }
    let l2 = desc.to_ascii_lowercase();
    l2.starts_with("total payments") || l2.starts_with("total transactions")
}

fn make_tx(date_s: &str, desc: &str, amount_s: &str, is_payment: bool) -> Option<ParsedTransaction> {
    let date = parse_apple_date(date_s)?;
    let raw_amt = parse_dollar_amount(amount_s)?;
    // Negate to convert "balance impact" to "money flow" semantics.
    let amount = -raw_amt;

    let merchant_raw = desc.to_string();
    let merchant_normalized = if is_payment {
        normalize("apple card payment")
    } else {
        normalize(desc)
    };
    let raw = json!({
        "source": "apple_card",
        "section": if is_payment { "payment" } else { "transaction" },
        "raw_date": date_s,
        "raw_amount": amount_s,
    });

    Some(ParsedTransaction {
        posted_at: date,
        amount,
        currency: "USD".into(),
        merchant_raw,
        merchant_normalized,
        description: if is_payment { "Apple Card payment".into() } else { String::new() },
        raw,
    })
}

fn parse_apple_date(s: &str) -> Option<chrono::DateTime<Utc>> {
    let d = NaiveDate::parse_from_str(s.trim(), "%m/%d/%Y").ok()?;
    let naive = NaiveDateTime::new(d, NaiveTime::from_hms_opt(0, 0, 0)?);
    Some(Utc.from_utc_datetime(&naive))
}

fn parse_dollar_amount(s: &str) -> Option<Decimal> {
    // Accept "$10.65", "-$802.31", "$1,234.56", with surrounding whitespace.
    let cleaned: String = s
        .chars()
        .filter(|c| matches!(c, '0'..='9' | '.' | '-'))
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    Decimal::from_str(&cleaned).ok()
}

fn date_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\d{2}/\d{2}/\d{4}$").unwrap())
}

fn signed_amount_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^-?\$\d{1,3}(,\d{3})*\.\d{2}$").unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_pdf() {
        let err = AppleCardParser.parse(b"hello").unwrap_err();
        assert!(format!("{err}").contains("PDF"));
    }

    #[test]
    fn pdf_magic_accepted() {
        // Truncated PDF — load_mem will fail with Parse, NOT Unsupported.
        let err = AppleCardParser.parse(b"%PDF-1.4\n%bad").unwrap_err();
        match err {
            ImportError::Parse(_) => {}
            other => panic!("expected Parse, got: {other}"),
        }
    }

    #[test]
    fn date_regex_matches_real_format() {
        assert!(date_re().is_match("12/31/2023"));
        assert!(!date_re().is_match("2023-12-31"));
    }

    #[test]
    fn signed_amount_regex() {
        assert!(signed_amount_re().is_match("$532.63"));
        assert!(signed_amount_re().is_match("-$802.31"));
        assert!(signed_amount_re().is_match("$1,234.56"));
        assert!(!signed_amount_re().is_match("2%"));
    }

    #[test]
    fn negates_amount() {
        let raw = parse_dollar_amount("$532.63").unwrap();
        assert_eq!(-raw, Decimal::new(-53263, 2));
        let raw2 = parse_dollar_amount("-$802.31").unwrap();
        assert_eq!(-raw2, Decimal::new(80231, 2)); // payment received → +
    }
}
