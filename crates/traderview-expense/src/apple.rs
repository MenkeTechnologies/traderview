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
//! The Wallet CSV/XLSX export (`Wallet > Card > ··· > Export Transactions`)
//! is also supported via a sibling sheet path, selected by magic bytes. Its
//! header-bearing schema is:
//!
//! ```text
//! Transaction Date,Clearing Date,Description,Merchant,Category,Type,Amount (USD),Purchased By
//! 04/28/2023,04/29/2023,"IN *RMDB ...","IN *Rmdb Handiwork L.L","Other","Purchase","250.00","Jacob Menke"
//! 04/28/2023,04/28/2023,"ACH DEPOSIT ...","Ach Deposit ...","Payment","Payment","-308.05","Jacob Menke"
//! ```
//!
//! The CSV signs amounts the same way the PDF does — purchases positive
//! ("amount owed"), payments negative — so the **negate every amount** rule
//! carries over unchanged: a $250 purchase becomes a −$250 expense, a
//! −$308.05 payment becomes a +$308.05 credit.

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
        // Magic-byte dispatch: PDF (%PDF) vs sheet (CSV / xlsx Wallet export).
        if bytes.len() >= 4 && &bytes[0..4] == b"%PDF" {
            return parse_pdf(bytes);
        }
        parse_sheet(bytes)
    }
}

/// Parse the Wallet CSV/XLSX export. The schema carries a header row, so we
/// locate it and map columns by name rather than by position (the PDF path
/// handles the printed statements). Rows without a parseable date+amount are
/// skipped silently. Returns an empty vec when the header isn't found, so a
/// non-Apple sheet doesn't error out the dispatch.
fn parse_sheet(bytes: &[u8]) -> Result<Vec<ParsedTransaction>, ImportError> {
    let rows = sheet::rows(bytes)?;

    // Find the header row: it names "Transaction Date" and an amount column.
    let hdr_idx = rows.iter().position(|row| {
        row.iter().any(|c| c.trim().eq_ignore_ascii_case("Transaction Date"))
            && row.iter().any(|c| c.to_ascii_lowercase().contains("amount"))
    });
    let hdr_idx = match hdr_idx {
        Some(i) => i,
        None => return Ok(Vec::new()),
    };
    let header = &rows[hdr_idx];
    let exact = |name: &str| header.iter().position(|h| h.trim().eq_ignore_ascii_case(name));
    let contains = |frag: &str| {
        header.iter().position(|h| h.to_ascii_lowercase().contains(frag))
    };

    let (c_date, c_amount) = match (exact("Transaction Date"), contains("amount")) {
        (Some(d), Some(a)) => (d, a),
        _ => return Ok(Vec::new()),
    };
    let c_clearing = exact("Clearing Date");
    let c_desc = exact("Description");
    let c_merchant = exact("Merchant");
    let c_category = exact("Category");
    let c_type = exact("Type");
    let c_by = exact("Purchased By");

    let cell = |row: &[String], idx: Option<usize>| -> String {
        idx.and_then(|i| row.get(i)).map(|s| s.trim().to_string()).unwrap_or_default()
    };

    let mut out = Vec::new();
    for row in &rows[hdr_idx + 1..] {
        let date_s = row.get(c_date).map(|s| s.trim()).unwrap_or("");
        let amount_s = row.get(c_amount).map(|s| s.trim()).unwrap_or("");
        if date_s.is_empty() || amount_s.is_empty() {
            continue;
        }
        let posted_at = match parse_apple_date(date_s) {
            Some(d) => d,
            None => continue,
        };
        let raw_amt = match parse_dollar_amount(amount_s) {
            Some(a) => a,
            None => continue,
        };
        // Same convention as the PDF: negate the printed "balance impact".
        let amount = -raw_amt;

        let kind = cell(row, c_type);
        let is_payment = kind.eq_ignore_ascii_case("Payment");
        let merchant_raw = {
            let m = cell(row, c_merchant);
            if m.is_empty() { cell(row, c_desc) } else { m }
        };
        let category = cell(row, c_category);
        let merchant_normalized = if is_payment {
            normalize("apple card payment")
        } else if merchant_raw.is_empty() {
            normalize("apple card")
        } else {
            normalize(&merchant_raw)
        };

        let raw = json!({
            "source": "apple_card",
            "format": "csv",
            "type": kind,
            "category": category,
            "clearing_date": cell(row, c_clearing),
            "purchased_by": cell(row, c_by),
            "raw_date": date_s,
            "raw_amount": amount_s,
        });

        out.push(ParsedTransaction {
            posted_at,
            amount,
            currency: "USD".into(),
            merchant_raw,
            merchant_normalized,
            description: if is_payment { "Apple Card payment".into() } else { category },
            raw,
        });
    }
    Ok(out)
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

    let lines: Vec<&str> = full
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

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
fn parse_payments_block(lines: &[&str], mut i: usize, out: &mut Vec<ParsedTransaction>) -> usize {
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
    if l.starts_with("total payments")
        || l.starts_with("total transactions")
        || l.starts_with("total interest")
        || l.starts_with("interest charged")
        || l.starts_with("interest charges")
        || l.starts_with("transactions")
        || l.starts_with("payments")
    {
        return true;
    }
    let l2 = desc.to_ascii_lowercase();
    l2.starts_with("total payments") || l2.starts_with("total transactions")
}

fn make_tx(
    date_s: &str,
    desc: &str,
    amount_s: &str,
    is_payment: bool,
) -> Option<ParsedTransaction> {
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
        description: if is_payment {
            "Apple Card payment".into()
        } else {
            String::new()
        },
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

    const CSV_SAMPLE: &str = "Transaction Date,Clearing Date,Description,Merchant,Category,Type,Amount (USD),Purchased By\n\
04/28/2023,04/29/2023,\"IN *RMDB HANDIWORK\",\"IN *Rmdb Handiwork L.L\",\"Other\",\"Purchase\",\"250.00\",\"Jacob Menke\"\n\
04/28/2023,04/28/2023,\"ACH DEPOSIT\",\"Ach Deposit\",\"Payment\",\"Payment\",\"-308.05\",\"Jacob Menke\"\n";

    #[test]
    fn parses_wallet_csv_export() {
        let r = AppleCardParser.parse(CSV_SAMPLE.as_bytes()).unwrap();
        assert_eq!(r.len(), 2);
        // Purchase $250 → expense −$250.
        assert_eq!(r[0].amount, Decimal::new(-25000, 2));
        assert_eq!(r[0].merchant_raw, "IN *Rmdb Handiwork L.L");
        assert_eq!(r[0].raw["type"], "Purchase");
        assert_eq!(r[0].raw["category"], "Other");
        // Payment −$308.05 → credit +$308.05.
        assert_eq!(r[1].amount, Decimal::new(30805, 2));
        assert_eq!(r[1].description, "Apple Card payment");
        assert_eq!(r[1].merchant_normalized, normalize("apple card payment"));
    }

    #[test]
    fn csv_dates_parse_to_transaction_date() {
        let r = AppleCardParser.parse(CSV_SAMPLE.as_bytes()).unwrap();
        let expected = parse_apple_date("04/28/2023").unwrap();
        assert_eq!(r[0].posted_at, expected);
    }

    #[test]
    fn non_apple_sheet_yields_empty_not_error() {
        // A sheet without the Apple header is ignored gracefully.
        let r = AppleCardParser.parse(b"foo,bar\n1,2\n").unwrap();
        assert!(r.is_empty());
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
