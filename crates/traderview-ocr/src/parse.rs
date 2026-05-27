//! Structured-field extraction from OCR / PDF text.
//!
//! Receipt heuristics:
//!   * Merchant — first non-empty line that isn't an address/phone/URL.
//!   * Total   — the largest dollar amount preceded by a "total"-like word,
//!               falling back to the largest dollar amount on the page.
//!   * Date    — first `MM/DD/YYYY`, `MM-DD-YYYY`, or month-name date.

use crate::OcrResult;
use chrono::NaiveDate;
use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::OnceLock;

pub fn structure(raw: &str, confidence: f32) -> OcrResult {
    OcrResult {
        merchant: extract_merchant(raw),
        total: extract_total(raw),
        date: extract_date(raw),
        confidence,
        text: raw.to_string(),
    }
}

// --- merchant ------------------------------------------------------------

static URL_RE: OnceLock<Regex> = OnceLock::new();
static PHONE_RE: OnceLock<Regex> = OnceLock::new();

fn extract_merchant(text: &str) -> Option<String> {
    let url = URL_RE.get_or_init(|| Regex::new(r"(?i)(https?://|www\.)").unwrap());
    let phone = PHONE_RE.get_or_init(|| Regex::new(r"\d{3}[-.\s]\d{3}[-.\s]\d{4}").unwrap());

    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if url.is_match(t) {
            continue;
        }
        if phone.is_match(t) {
            continue;
        }
        // Skip lines that look like a street address (start with a number then street word).
        if looks_like_address(t) {
            continue;
        }
        // Skip very short noise like "*" / numeric-only lines.
        if t.chars().filter(|c| c.is_alphabetic()).count() < 2 {
            continue;
        }
        return Some(t.to_string());
    }
    None
}

fn looks_like_address(line: &str) -> bool {
    static ADDR_RE: OnceLock<Regex> = OnceLock::new();
    let re = ADDR_RE.get_or_init(|| {
        Regex::new(r"(?i)^\d+\s+\S+.*\b(st|street|ave|avenue|blvd|boulevard|rd|road|dr|drive|way|ln|lane|ct|court|pl|place)\b").unwrap()
    });
    re.is_match(line)
}

// --- total ---------------------------------------------------------------

static AMOUNT_RE: OnceLock<Regex> = OnceLock::new();
static TOTAL_TAG_RE: OnceLock<Regex> = OnceLock::new();

fn extract_total(text: &str) -> Option<Decimal> {
    let amt = AMOUNT_RE.get_or_init(|| {
        // $1,234.56  or  1234.56  — require ≥1 digit before the decimal.
        Regex::new(r"\$?\s?(\d{1,3}(?:,\d{3})*\.\d{2}|\d+\.\d{2})").unwrap()
    });
    let total_tag = TOTAL_TAG_RE.get_or_init(|| {
        // "total", "amount due", "grand total", "balance"
        Regex::new(r"(?i)\b(grand\s*total|total\s*amount|amount\s*due|balance\s*due|total)\b").unwrap()
    });

    // Pass 1: look on every "total"-tagged line and take the max.
    let mut tagged_max: Option<Decimal> = None;
    for line in text.lines() {
        if !total_tag.is_match(line) {
            continue;
        }
        // Skip subtotal lines explicitly.
        if line.to_lowercase().contains("subtotal") {
            continue;
        }
        for cap in amt.captures_iter(line) {
            if let Some(d) = parse_money(&cap[1]) {
                tagged_max = Some(match tagged_max {
                    Some(prev) => if d > prev { d } else { prev },
                    None => d,
                });
            }
        }
    }
    if tagged_max.is_some() {
        return tagged_max;
    }

    // Pass 2 fallback: largest amount on the page (assumes the total is the
    // biggest line item, which is true for the vast majority of receipts).
    let mut global_max: Option<Decimal> = None;
    for cap in amt.captures_iter(text) {
        if let Some(d) = parse_money(&cap[1]) {
            global_max = Some(match global_max {
                Some(prev) => if d > prev { d } else { prev },
                None => d,
            });
        }
    }
    global_max
}

fn parse_money(s: &str) -> Option<Decimal> {
    let cleaned: String = s.chars().filter(|c| *c != ',' && *c != '$' && !c.is_whitespace()).collect();
    Decimal::from_str(&cleaned).ok()
}

// --- date ----------------------------------------------------------------

static DATE_SLASH_RE: OnceLock<Regex> = OnceLock::new();
static DATE_WORD_RE: OnceLock<Regex> = OnceLock::new();

fn extract_date(text: &str) -> Option<NaiveDate> {
    let slash = DATE_SLASH_RE
        .get_or_init(|| Regex::new(r"\b(\d{1,2})[/-](\d{1,2})[/-](\d{2,4})\b").unwrap());
    if let Some(cap) = slash.captures(text) {
        let m: u32 = cap[1].parse().ok()?;
        let d: u32 = cap[2].parse().ok()?;
        let mut y: i32 = cap[3].parse().ok()?;
        if y < 100 {
            y += if y < 70 { 2000 } else { 1900 };
        }
        if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
            return Some(date);
        }
    }
    let word = DATE_WORD_RE.get_or_init(|| {
        Regex::new(r"(?i)\b(jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec)[a-z]*\.?\s+(\d{1,2}),?\s+(\d{4})\b").unwrap()
    });
    if let Some(cap) = word.captures(text) {
        let m = match cap[1].to_ascii_lowercase().as_str() {
            "jan" => 1, "feb" => 2, "mar" => 3, "apr" => 4, "may" => 5, "jun" => 6,
            "jul" => 7, "aug" => 8, "sep" | "sept" => 9,
            "oct" => 10, "nov" => 11, "dec" => 12,
            _ => return None,
        };
        let d: u32 = cap[2].parse().ok()?;
        let y: i32 = cap[3].parse().ok()?;
        if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
            return Some(date);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merchant_skips_url() {
        let t = "https://www.example.com\nBlue Bottle Coffee\n123 Main St";
        assert_eq!(extract_merchant(t).as_deref(), Some("Blue Bottle Coffee"));
    }

    #[test]
    fn merchant_skips_address_first() {
        let t = "1234 Mission St\nBlue Bottle Coffee";
        assert_eq!(extract_merchant(t).as_deref(), Some("Blue Bottle Coffee"));
    }

    #[test]
    fn total_prefers_tagged() {
        let t = "Subtotal $40.00\nTax $3.50\nTotal $43.50\nTip $5.00";
        assert_eq!(extract_total(t), Some(Decimal::new(4350, 2)));
    }

    #[test]
    fn total_falls_back_to_max() {
        let t = "Coffee $5.50\nMuffin $4.25\nCash $20.00\nChange $10.25";
        assert_eq!(extract_total(t), Some(Decimal::new(2000, 2)));
    }

    #[test]
    fn total_skips_subtotal_line() {
        // "Subtotal" contains "total" — must be excluded.
        let t = "Subtotal: $100.00\nTotal: $108.50";
        assert_eq!(extract_total(t), Some(Decimal::new(10850, 2)));
    }

    #[test]
    fn total_with_commas() {
        let t = "Total $1,234.56";
        assert_eq!(extract_total(t), Some(Decimal::new(123456, 2)));
    }

    #[test]
    fn date_slash() {
        assert_eq!(
            extract_date("Date: 05/26/2026"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 26).unwrap())
        );
    }

    #[test]
    fn date_two_digit_year() {
        assert_eq!(
            extract_date("3/15/26 sale"),
            Some(NaiveDate::from_ymd_opt(2026, 3, 15).unwrap())
        );
    }

    #[test]
    fn date_word_month() {
        assert_eq!(
            extract_date("Receipt Mar 15, 2026"),
            Some(NaiveDate::from_ymd_opt(2026, 3, 15).unwrap())
        );
        assert_eq!(
            extract_date("Receipt September 1, 2026"),
            Some(NaiveDate::from_ymd_opt(2026, 9, 1).unwrap())
        );
    }
}
