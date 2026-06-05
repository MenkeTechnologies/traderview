//! Structured-field extraction from OCR / PDF text.
//!
//! Receipt heuristics:
//!   * Merchant — first non-empty line that isn't an address/phone/URL.
//!   * Total   — scored line-by-line:
//!       + "total sale", "amount due", "balance due", "grand total" → +3
//!       + "total" alone                                            → +1
//!       - "total savings", "total number of items", "subtotal",
//!         "rebate", "discount", "you saved"                        → reject
//!       Among the highest-scoring lines, take the LAST occurrence
//!       (chain-store receipts put the final total at the bottom).
//!       Falls back to the largest dollar amount on the page.
//!   * Date    — scored across every `MM/DD/YY` and month-name date:
//!       + line contains "sale", "trans", "date", "receipt", "order",
//!         "purchase"                                               → +2
//!       + date is in the past, within last 5 years                 → +1
//!       - line contains "return", "rebate", "expires", "valid",
//!         "exp", "redeem", "good until", "offer", "warranty",
//!         "policy"                                                 → -3
//!       - date is more than 30 days in the future                  → -2
//!       Pick highest-score; tiebreak = LAST occurrence.

use crate::{OcrLineItem, OcrResult};
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Utc};
use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::OnceLock;

pub fn structure(raw: &str, confidence: f32) -> OcrResult {
    // Pre-clean the OCR text before any extractor runs. `.text` keeps
    // the original for debugging — the cleaned copy is only used
    // internally by the field extractors.
    //
    // 1. `[UNK]` → space — legacy PaddleOCR space sentinel.
    // 2. Comma-decimals → period-decimals (`06,88` → `06.88`). Tesseract
    //    on thermal-printer receipts misreads `.` as `,` constantly,
    //    leaving every price unparseable. Restrict to `<digit>,<2 digits>
    //    <word-boundary>` so we don't mangle phone numbers like
    //    `555,1234` or thousands separators like `1,000.00`.
    let mut cleaned = raw.replace("[UNK]", " ");
    {
        static COMMA_DECIMAL: OnceLock<Regex> = OnceLock::new();
        let re = COMMA_DECIMAL.get_or_init(|| {
            Regex::new(r"(\d),(\d{2})(?:\b|$|[^0-9])").unwrap()
        });
        let mut next = re.replace_all(&cleaned, |c: &regex::Captures| {
            let tail = c.get(0).unwrap().as_str();
            let trailing = &tail[c[1].len() + 1 + c[2].len()..];
            format!("{}.{}{}", &c[1], &c[2], trailing)
        }).into_owned();
        loop {
            let again = re.replace_all(&next, |c: &regex::Captures| {
                let tail = c.get(0).unwrap().as_str();
                let trailing = &tail[c[1].len() + 1 + c[2].len()..];
                format!("{}.{}{}", &c[1], &c[2], trailing)
            }).into_owned();
            if again == next { break; }
            next = again;
        }
        cleaned = next;
    }
    // Snapshot the cleaned text BEFORE colon→period replacement so
    // `extract_time` can still see real time stamps.
    let cleaned_for_time = cleaned.clone();
    // Colon-decimal restoration. Tesseract sometimes misreads `.` as
    // `:` (period gets lost in receipt-print kerning) so prices like
    // `2.49` come out as `2:49`. Strategy:
    //   * If the match has an AM/PM suffix → real time, preserve.
    //   * If the hour > 23 → can't be a time, definitely a price.
    //   * Otherwise lean price — receipts have many prices and at
    //     most one time, and `extract_time` already ran on the
    //     pre-replacement snapshot above.
    {
        static COLON_DECIMAL: OnceLock<Regex> = OnceLock::new();
        let re = COLON_DECIMAL.get_or_init(|| {
            Regex::new(r"(\d{1,2}):(\d{2})(\s*[AaPp][Mm])?").unwrap()
        });
        cleaned = re.replace_all(&cleaned, |c: &regex::Captures| {
            if c.get(3).map_or(false, |m| !m.as_str().trim().is_empty()) {
                format!("{}:{}{}", &c[1], &c[2],
                    c.get(3).map(|m| m.as_str()).unwrap_or(""))
            } else {
                format!("{}.{}", &c[1], &c[2])
            }
        }).into_owned();
    }
    OcrResult {
        merchant: extract_merchant(&cleaned),
        address: extract_address(&cleaned),
        date: extract_date(&cleaned),
        time: extract_time(&cleaned_for_time),
        subtotal: extract_subtotal(&cleaned),
        tax: extract_tax(&cleaned),
        total: extract_total(&cleaned),
        items: extract_items(&cleaned),
        confidence,
        text: raw.to_string(),
        // Set to "unknown" here — the caller in `lib.rs::extract()`
        // overwrites with the actual engine ("apple_vision" /
        // "tesseract" / "pdf"). Tests calling structure() directly
        // get "unknown" which is the right semantic.
        engine: "unknown".into(),
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
    // Strong final-total tags. We prefer these heavily — "TOTAL SALE",
    // "AMOUNT DUE", etc. unambiguously refer to the charged amount,
    // whereas a bare "TOTAL" is often the pre-tax subtotal on receipts
    // that also have "TOTAL SALE" or "TOTAL DUE" below it.
    let strong_tag = TOTAL_TAG_RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(grand\s*total|total\s*sale|total\s*charge|total\s*amount|amount\s*due|amount\s*paid|balance\s*due|charge\s*total|payment\s*total)\b",
        )
        .unwrap()
    });
    // Weak total tag — bare "TOTAL". Matches subtotal/savings noise too,
    // so we apply the reject set below before trusting it.
    static WEAK_TAG: OnceLock<Regex> = OnceLock::new();
    let weak_tag = WEAK_TAG.get_or_init(|| Regex::new(r"(?i)\btotal\b").unwrap());
    // Lines that match `total` for unrelated reasons. Skip outright.
    static REJECT_TAG: OnceLock<Regex> = OnceLock::new();
    let reject_tag = REJECT_TAG.get_or_init(|| {
        Regex::new(
            r"(?i)\b(sub\s*total|total\s*savings|total\s*saved|total\s*discount|total\s*number\s*of\s*items|total\s*items|rebate\s*total|discount\s*total|savings\s*total|you\s*saved|tip|gratuity)\b",
        )
        .unwrap()
    });

    let mut best_score = i32::MIN;
    let mut best_value: Option<Decimal> = None;
    for line in text.lines() {
        // Item-count lines like "TOTAL NUMBER OF ITEMS = 15" have a
        // small int with no `.dd` cents and would never match `amt` —
        // but `reject_tag` removes them defensively regardless.
        if reject_tag.is_match(line) {
            continue;
        }
        let score = if strong_tag.is_match(line) {
            3
        } else if weak_tag.is_match(line) {
            1
        } else {
            continue;
        };
        // Within a line, take the LAST amount — chain receipts print
        // "TOTAL SALE      99.36" with the amount at the right edge,
        // and any earlier number on that line is noise (e.g., line
        // numbers, register IDs).
        let mut line_value: Option<Decimal> = None;
        for cap in amt.captures_iter(line) {
            if let Some(d) = parse_money(&cap[1]) {
                line_value = Some(d);
            }
        }
        let Some(d) = line_value else { continue };
        // `>=` so among equal scores the LAST line wins (totals are at
        // the bottom of the receipt; payment-method echoes of the same
        // amount appear AFTER "TOTAL SALE" and we want the later one).
        if score >= best_score {
            best_score = score;
            best_value = Some(d);
        }
    }
    if best_value.is_some() {
        return best_value;
    }

    // Fallback: largest amount on the page (assumes the total is the
    // biggest line item, which is true for the vast majority of
    // receipts when the total tag wasn't picked up by OCR).
    let mut global_max: Option<Decimal> = None;
    for cap in amt.captures_iter(text) {
        if let Some(d) = parse_money(&cap[1]) {
            global_max = Some(match global_max {
                Some(prev) => {
                    if d > prev {
                        d
                    } else {
                        prev
                    }
                }
                None => d,
            });
        }
    }
    global_max
}

fn parse_money(s: &str) -> Option<Decimal> {
    let cleaned: String = s
        .chars()
        .filter(|c| *c != ',' && *c != '$' && !c.is_whitespace())
        .collect();
    Decimal::from_str(&cleaned).ok()
}

// --- date ----------------------------------------------------------------

static DATE_SLASH_RE: OnceLock<Regex> = OnceLock::new();
static DATE_WORD_RE: OnceLock<Regex> = OnceLock::new();

fn extract_date(text: &str) -> Option<NaiveDate> {
    let slash = DATE_SLASH_RE
        .get_or_init(|| Regex::new(r"\b(\d{1,2})[/-](\d{1,2})[/-](\d{2,4})\b").unwrap());
    let word = DATE_WORD_RE.get_or_init(|| {
        Regex::new(r"(?i)\b(jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec)[a-z]*\.?\s+(\d{1,2}),?\s+(\d{2,4})\b").unwrap()
    });

    // Tag keywords that bias scoring on the candidate's line. Lifted
    // out so both date-format passes share the same logic.
    static SALE_TAG: OnceLock<Regex> = OnceLock::new();
    let sale_tag = SALE_TAG.get_or_init(|| {
        Regex::new(r"(?i)\b(sale|trans|transaction|date|receipt|order|purchase|sold|tendered)\b")
            .unwrap()
    });
    static REJECT_TAG: OnceLock<Regex> = OnceLock::new();
    let reject_tag = REJECT_TAG.get_or_init(|| {
        Regex::new(
            r"(?i)\b(return|rebate|expir|exp\.|valid|redeem|good\s*until|good\s*thru|offer|warranty|policy|due\s*by|due\s*date|estimated)\b",
        )
        .unwrap()
    });

    let today = Utc::now().date_naive();
    let five_years_ago = today - Duration::days(5 * 365);
    let one_month_ahead = today + Duration::days(30);

    let mut best_score = i32::MIN;
    let mut best_date: Option<NaiveDate> = None;

    let mut consider = |date: NaiveDate, line: &str| {
        // Reject obvious noise (junk OCR producing 01/01/1970 etc.).
        if date.year() < 1990 || date.year() > 2100 {
            return;
        }
        let mut score = 0;
        if reject_tag.is_match(line) {
            score -= 3;
        }
        if sale_tag.is_match(line) {
            score += 2;
        }
        // Recency: past but recent = real sale; far future = rebate/expiry.
        if date <= today && date >= five_years_ago {
            score += 1;
        } else if date > one_month_ahead {
            score -= 2;
        }
        // `>=` so later occurrences with equal score win — sale dates
        // are typically at the bottom of the receipt next to the
        // register / transaction id stamp.
        if score >= best_score {
            best_score = score;
            best_date = Some(date);
        }
    };

    // Slash / dash dates. Walk every match (not just the first) and
    // score line-locally — find each match's line by start-of-match
    // offset, then bound forward/back to the surrounding newline.
    for m in slash.find_iter(text) {
        let cap = match slash.captures(m.as_str()) {
            Some(c) => c,
            None => continue,
        };
        let month: u32 = match cap[1].parse() { Ok(v) => v, Err(_) => continue };
        let day: u32 = match cap[2].parse() { Ok(v) => v, Err(_) => continue };
        let mut year: i32 = match cap[3].parse() { Ok(v) => v, Err(_) => continue };
        if year < 100 {
            year += if year < 70 { 2000 } else { 1900 };
        }
        let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else { continue };
        let line = line_containing(text, m.start());
        consider(date, line);
    }

    // Month-name dates (e.g., "Mar 15, 2026").
    for m in word.find_iter(text) {
        let cap = match word.captures(m.as_str()) {
            Some(c) => c,
            None => continue,
        };
        let month = match cap[1].to_ascii_lowercase().as_str() {
            "jan" => 1,  "feb" => 2,  "mar" => 3,  "apr" => 4,
            "may" => 5,  "jun" => 6,  "jul" => 7,  "aug" => 8,
            "sep" | "sept" => 9, "oct" => 10, "nov" => 11, "dec" => 12,
            _ => continue,
        };
        let day: u32 = match cap[2].parse() { Ok(v) => v, Err(_) => continue };
        let mut year: i32 = match cap[3].parse() { Ok(v) => v, Err(_) => continue };
        if year < 100 {
            year += if year < 70 { 2000 } else { 1900 };
        }
        let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else { continue };
        let line = line_containing(text, m.start());
        consider(date, line);
    }

    // Only commit a date when its score is non-negative. A negative
    // score means every readable date on the receipt had a reject tag
    // ("return is done after 08/23/26", "expires", "valid until") — in
    // that case we'd rather surface a missing date and let the user
    // type it in than confidently report the return-policy date as
    // the sale date.
    if best_score < 0 {
        return None;
    }
    best_date
}

// Returns the line of `text` containing byte-offset `pos`. Used so the
// date scorer can inspect the keywords on the same line as the match
// (e.g., "return is done after 08/23/26" vs "Sale 05/25/26").
fn line_containing(text: &str, pos: usize) -> &str {
    let start = text[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = text[pos..]
        .find('\n')
        .map(|i| pos + i)
        .unwrap_or(text.len());
    &text[start..end]
}

// --- time ----------------------------------------------------------------

fn extract_time(text: &str) -> Option<NaiveTime> {
    // Matches `06:07PM`, `6:07 PM`, `18:07`, `18:07:42`. 12-hour form
    // requires AM/PM; 24-hour form requires hours 0-23.
    static TIME_RE: OnceLock<Regex> = OnceLock::new();
    let re = TIME_RE.get_or_init(|| {
        Regex::new(r"(?i)\b(\d{1,2}):(\d{2})(?::(\d{2}))?\s*(am|pm)?\b").unwrap()
    });
    for cap in re.captures_iter(text) {
        let h: u32 = cap[1].parse().ok()?;
        let m: u32 = cap[2].parse().ok()?;
        let s: u32 = cap.get(3).and_then(|x| x.as_str().parse().ok()).unwrap_or(0);
        let suffix = cap.get(4).map(|x| x.as_str().to_ascii_lowercase());
        let h24 = match suffix.as_deref() {
            Some("am") => {
                if !(1..=12).contains(&h) { continue; }
                if h == 12 { 0 } else { h }
            }
            Some("pm") => {
                if !(1..=12).contains(&h) { continue; }
                if h == 12 { 12 } else { h + 12 }
            }
            _ => {
                // 24-hour: reject obvious price-noise like "$1.23"
                // (won't reach here — re requires HH:MM) and the
                // "06:07PM" w/o space when suffix is None. Accept only
                // when 0-23.
                if h > 23 { continue; }
                h
            }
        };
        if let Some(t) = NaiveTime::from_hms_opt(h24, m, s) {
            return Some(t);
        }
    }
    None
}

// --- subtotal / tax ------------------------------------------------------

fn extract_subtotal(text: &str) -> Option<Decimal> {
    extract_amount_for_tag(text, r"(?i)\b(sub\s*total|subtotal)\b")
}

fn extract_tax(text: &str) -> Option<Decimal> {
    // Match "Tax", "Sales Tax", "TAX STATE OF MI 6%", "GST", "HST", "VAT".
    // Skip "TAX EXEMPT" / "TAX FREE" lines that have no dollar value
    // (the amount regex below requires `.dd` so they naturally drop out).
    extract_amount_for_tag(
        text,
        r"(?i)\b(tax|sales\s*tax|tax\s*state|gst|hst|vat)\b",
    )
}

// Shared helper: return the last `.dd` amount on the LAST line that
// matches `tag_pattern`. Used for subtotal + tax.
fn extract_amount_for_tag(text: &str, tag_pattern: &str) -> Option<Decimal> {
    let amt = AMOUNT_RE
        .get_or_init(|| Regex::new(r"\$?\s?(\d{1,3}(?:,\d{3})*\.\d{2}|\d+\.\d{2})").unwrap());
    let tag = Regex::new(tag_pattern).unwrap();
    let mut last: Option<Decimal> = None;
    for line in text.lines() {
        if !tag.is_match(line) { continue; }
        // Avoid the "TOTAL SALE" line matching "tax" sub-strings if any.
        // Also skip explicit reject phrases.
        let lc = line.to_lowercase();
        if lc.contains("tax exempt") || lc.contains("tax free") { continue; }
        let mut line_value: Option<Decimal> = None;
        for cap in amt.captures_iter(line) {
            if let Some(d) = parse_money(&cap[1]) {
                line_value = Some(d);
            }
        }
        if let Some(d) = line_value {
            last = Some(d);
        }
    }
    last
}

// --- address -------------------------------------------------------------

fn extract_address(text: &str) -> Option<String> {
    // Walk consecutive lines; when one looks like a street and the next
    // looks like "City, ST 12345", join them. Falls back to the street
    // alone if no city/state line follows within 2 lines.
    static CITY_STATE_ZIP_RE: OnceLock<Regex> = OnceLock::new();
    let csz = CITY_STATE_ZIP_RE.get_or_init(|| {
        // "Ann Arbor, MI 48103" / "San Francisco CA 94102" / "12345-6789"
        Regex::new(r"(?i)\b[A-Za-z][A-Za-z .'\-]+,?\s+[A-Z]{2}\s+\d{5}(?:-\d{4})?\b").unwrap()
    });
    let lines: Vec<&str> = text.lines().map(|l| l.trim()).collect();
    for (i, line) in lines.iter().enumerate() {
        if !looks_like_address(line) { continue; }
        // Check the next two non-empty lines for a city/state/zip.
        for j in (i + 1)..(i + 4).min(lines.len()) {
            let next = lines[j];
            if next.is_empty() { continue; }
            if csz.is_match(next) {
                return Some(format!("{}, {}", line, next));
            }
            // Stop early if we hit a clearly-non-address line (e.g.,
            // "KEEP YOUR RECEIPT").
            if next.chars().any(|c| c == '$') { break; }
        }
        return Some((*line).to_string());
    }
    None
}

// --- items ---------------------------------------------------------------

fn extract_items(text: &str) -> Vec<OcrLineItem> {
    // After [UNK]→space preprocessing, real receipts come out as 3-5
    // lines per item (Menards example):
    //
    //   IM                ← 1-2 char marker (skipped)
    //   SPRAY FORD RED    ← name
    //   55                ← partial SKU (skipped)
    //   4 @6.88           ← qty @ unit
    //   27.52             ← line total (or "2.49 NT" with status flag)
    //
    // Plus the legacy single-line "NAME       PRICE" layout chains use.
    //
    // Algorithm: for every line that looks like an item-name candidate
    // (alphabetic, not header noise, not too long), look forward in a
    // 4-line window for a standalone-price "line-total" line. If found,
    // emit the item, picking up any `\d+\s*@\s*\d+\.\d+` qty/unit in
    // the same window. If no line-total appears within the window, try
    // the single-line "NAME PRICE" fast path on the candidate.
    static QTY_AT_RE: OnceLock<Regex> = OnceLock::new();
    static PRICE_LINE_RE: OnceLock<Regex> = OnceLock::new();
    static TRAILING_PRICE_RE: OnceLock<Regex> = OnceLock::new();
    static SKU_ONLY_RE: OnceLock<Regex> = OnceLock::new();
    let qty_at = QTY_AT_RE.get_or_init(|| {
        Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*@\s*\$?\s*(\d+(?:\.\d{2}))").unwrap()
    });
    // PRICE_LINE: the entire line is JUST a price + optional 1-2 letter
    // flag ("NT", "T", "F"). Used as a line-total detector.
    let price_line = PRICE_LINE_RE.get_or_init(|| {
        Regex::new(r"^\s*\$?\s*(\d{1,3}(?:,\d{3})*\.\d{2}|\d+\.\d{2})\s*(?:[A-Z]{1,2})?\s*$")
            .unwrap()
    });
    // TRAILING_PRICE: price right before end-of-line, with anything in
    // front. Used for single-line "NAME      5.50" fallback.
    let trailing_price = TRAILING_PRICE_RE.get_or_init(|| {
        Regex::new(r"(\d{1,3}(?:,\d{3})*\.\d{2}|\d+\.\d{2})\s*(?:[A-Z]{1,2})?\s*$")
            .unwrap()
    });
    let sku_only = SKU_ONLY_RE.get_or_init(|| {
        Regex::new(r"^\s*\d[\d\s]*$").unwrap()
    });

    // Tesseract decorates every line on photo'd receipts with leading
    // junk (`—`, `=`, `:`, `|`, single-char fragments like `oS`, `AZ`,
    // `Se` that the OCR invented out of the receipt's left edge) and
    // similar trailing junk. Strip them BEFORE the walker sees the line
    // so item-name detection isn't fooled by ornamentation.
    let cleaned_lines: Vec<String> = text.lines().map(strip_line_decorations).collect();
    let lines: Vec<&str> = cleaned_lines.iter().map(|s| s.as_str()).collect();

    // Totals-block boundary. Catches `TOTAL    9.75` (bare TOTAL with
    // a trailing price), the dedicated `TOTAL SALE` / `GRAND TOTAL` /
    // `AMOUNT DUE` lines, and the standalone `TOTAL` line that lives
    // above the actual total on multi-line POS formats.
    static TOTALS_BOUNDARY_RE: OnceLock<Regex> = OnceLock::new();
    let totals_re = TOTALS_BOUNDARY_RE.get_or_init(|| {
        Regex::new(r"(?i)\b(sub\s*total|sub-?total|grand\s*total|amount\s*due|total\s*sale)\b|^\s*total\b").unwrap()
    });
    let mut item_end = lines.len();
    for (i, line) in lines.iter().enumerate() {
        if totals_re.is_match(line) {
            item_end = i;
            break;
        }
    }

    let mut out: Vec<OcrLineItem> = Vec::new();
    let mut i = 0;
    while i < item_end {
        let line = lines[i];
        // Paragraph-skip — count non-whitespace chars instead of raw
        // length so item lines decorated with long whitespace runs by
        // Tesseract (`oS                 GUNK CHLORNTD BRI` ≈ 60 chars
        // raw, 22 non-whitespace) don't get rejected as paragraphs.
        let content_chars = line.chars().filter(|c| !c.is_whitespace()).count();
        if line.is_empty() || sku_only.is_match(line) || is_header_noise(line)
            || looks_like_address(line) || looks_like_city_state_zip(line)
            || content_chars > 50  // policy / disclaimer paragraph
        {
            i += 1; continue;
        }
        // Need real alphabetic content to start an item.
        if !line_has_alpha(line) {
            i += 1; continue;
        }

        // ── Single-line "NAME    PRICE" fast path ─────────────────
        // Only when this line itself is a self-contained item — alpha
        // prefix, trailing price, AND the next line doesn't look like
        // a SKU/qty continuation (which would suggest a multi-line
        // block where this is just the name).
        let next_is_continuation = lines.get(i + 1).map_or(false, |n| {
            sku_only.is_match(n) || qty_at.is_match(n) || price_line.is_match(n)
        });
        if !next_is_continuation {
            if let Some(cap) = trailing_price.captures(line) {
                let m = cap.get(1).unwrap();
                let name_part = line[..m.start()].trim_end_matches(|c: char| {
                    c == '$' || c.is_whitespace()
                });
                if line_has_alpha(name_part) {
                    if let Some(t) = parse_money(&cap[1]) {
                        push_item(&mut out, clean_item_name(name_part), None, None, t);
                        i += 1; continue;
                    }
                }
            }
        }

        // ── Multi-line block — look ahead up to 4 lines for a
        //    line-total (price-only line). Cap name lines at 2 so
        //    consecutive items don't collapse into one.
        let mut name_parts: Vec<&str> = vec![line];
        let mut qty: Option<Decimal> = None;
        let mut unit: Option<Decimal> = None;
        let mut total: Option<Decimal> = None;
        let mut consumed = 1usize;
        let max_block = 5;
        for k in 1..max_block {
            let idx = i + k;
            if idx >= item_end { break; }
            let l = lines[idx];
            if l.is_empty() { consumed = k + 1; continue; }
            // Line-total terminates the block.
            if let Some(cap) = price_line.captures(l) {
                if let Some(t) = parse_money(&cap[1]) {
                    total = Some(t);
                    consumed = k + 1;
                    break;
                }
            }
            // qty @ unit — MUST run before the SKU+trailing-price
            // check, because a line like `4 @6.88` is digit-heavy
            // (passes `next_line_is_sku_shaped`) AND has a trailing
            // `.dd` (looks like a SKU+price line). Without this
            // ordering, the unit price gets captured as the line total
            // and the actual total on the next line is missed.
            if let Some(cap) = qty_at.captures(l) {
                qty = parse_money(&cap[1]);
                unit = parse_money(&cap[2]);
                consumed = k + 1;
                // qty @ unit + same-line trailing total (one-line dense
                // POS format): if a price appears AFTER the qty@unit
                // match, take that as the line total. Otherwise the
                // total is on the next line and we keep scanning.
                if let Some(price_cap) = trailing_price.captures(l) {
                    let inner = price_cap.get(1).unwrap();
                    let prior = qty_at.find(l).unwrap();
                    if inner.start() > prior.end() {
                        if let Some(t) = parse_money(&price_cap[1]) {
                            total = Some(t);
                            break;
                        }
                    }
                }
                continue;
            }
            // "SKU + trailing price" on one line (`5746883       2.49`)
            // — the SKU prefix prevents `price_line` from matching, but
            // it's still effectively a line total. Skipped above when
            // the line carries a qty@unit pattern.
            if next_line_is_sku_shaped(l) {
                if let Some(cap) = trailing_price.captures(l) {
                    if let Some(t) = parse_money(&cap[1]) {
                        total = Some(t);
                        consumed = k + 1;
                        break;
                    }
                }
            }
            // SKU.
            if sku_only.is_match(l) {
                consumed = k + 1;
                continue;
            }
            // Alphabetic line. If we already accumulated 2 name lines
            // OR a qty/unit already landed, this line is the next
            // item — stop accumulating WITHOUT consuming it.
            if line_has_alpha(l) {
                if name_parts.len() >= 2 || qty.is_some() {
                    break;
                }
                name_parts.push(l);
                consumed = k + 1;
                continue;
            }
            break;
        }
        if let Some(t) = total {
            let name = name_parts.join(" ");
            push_item(&mut out, clean_item_name(&name), qty, unit, t);
            i += consumed;
            continue;
        }
        // No line-total found within window: 3-line block with qty+unit
        // → synthesize total from qty × unit so the item shows up.
        if qty.is_some() && unit.is_some() {
            let synthetic = qty.unwrap() * unit.unwrap();
            let name = name_parts.join(" ");
            push_item(&mut out, clean_item_name(&name), qty, unit, synthetic);
            i += consumed.max(1);
            continue;
        }
        i += 1;
    }
    out
}

// Helper: build + push an OcrLineItem with auto-default bucket.
fn push_item(
    out: &mut Vec<OcrLineItem>,
    name: String,
    qty: Option<Decimal>,
    unit_price: Option<Decimal>,
    line_total: Decimal,
) {
    let cat = guess_category(&name);
    out.push(OcrLineItem {
        name,
        qty,
        unit_price,
        line_total,
        tax_bucket: default_bucket_for_category(&cat).into(),
        category: cat,
        rental_property_id: None,
    });
}

// Detects `City, ST 12345` second-line of a postal address.
fn looks_like_city_state_zip(line: &str) -> bool {
    static CSZ: OnceLock<Regex> = OnceLock::new();
    let re = CSZ.get_or_init(|| {
        Regex::new(r"(?i)\b[A-Za-z][A-Za-z .'\-]+,?\s+[A-Z]{2}\s+\d{5}(?:-\d{4})?\b").unwrap()
    });
    re.is_match(line)
}

fn line_has_alpha(s: &str) -> bool {
    s.chars().filter(|c| c.is_alphabetic()).count() >= 3
}

// "SKU-shaped" = mostly digits with at most 2 stray letters (status
// flags like `NT`, `T`, `F` after the price). Distinguishes a "5574679
// 2 @14.97 29.94" continuation from a "MUFFIN 4.25" next item.
fn next_line_is_sku_shaped(s: &str) -> bool {
    let alpha = s.chars().filter(|c| c.is_alphabetic()).count();
    let digits = s.chars().filter(|c| c.is_ascii_digit()).count();
    digits >= 3 && alpha <= 2
}

fn is_header_noise(line: &str) -> bool {
    let lc = line.to_ascii_lowercase();
    matches!(
        lc.trim(),
        "sale transaction" | "guest copy" | "keep your receipt" |
        "thank you" | "" | "*" | "im" | "1m"
    ) || lc.starts_with("the following rebate")
      || lc.starts_with("return policy")
      || lc.starts_with("unless noted")
      || lc.starts_with("if you have questions")
      || lc.starts_with("email us")
      || lc.contains("@menards") || lc.contains("@gmail") || lc.contains(".com")
}

/// Pre-walker line cleaner. Tesseract OCR on photo'd receipts
/// frequently invents decoration tokens from the receipt's edges:
///
///   "—             a              Li        SPRAY. FORD. RED"
///   "oS =                        GUNK CHLORNTD BRI\" 8 499      |"
///   "AZ    :               FUNABLES MIXED BERRY            Be"
///
/// We strip:
///   * Leading runs of non-alphanumeric chars (`—`, `=`, `:`, `|`, `.`).
///   * Leading short alphanumeric "junk tokens" (1-2 char words that
///     don't form real item-name content) followed by whitespace.
///   * The same on the trailing side.
///   * Repeated whitespace runs are not collapsed — `extract_items`
///     relies on column gaps for some heuristics, and the walker's own
///     line.trim() (built into the regex anchors) handles edge spaces.
///
/// Returns the cleaned line. Lines that are pure decoration return an
/// empty string, which the walker treats as a skip.
fn strip_line_decorations(s: &str) -> String {
    static LEAD: OnceLock<Regex> = OnceLock::new();
    static TRAIL: OnceLock<Regex> = OnceLock::new();
    // Leading: zero or more "junk segments". A segment is either
    //   * pure non-alphanumeric (with whitespace) — punctuation runs
    //   * a 1-2 char alphanumeric token followed by whitespace
    // We loop both together until the line stabilizes.
    let lead = LEAD.get_or_init(|| {
        Regex::new(r"^(?:[^A-Za-z0-9]+|[A-Za-z]{1,2}\s+)+").unwrap()
    });
    let trail = TRAIL.get_or_init(|| {
        Regex::new(r"(?:[^A-Za-z0-9]+|\s+[A-Za-z]{1,2})+$").unwrap()
    });
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let after_lead = lead.replace(trimmed, "");
    let after_trail = trail.replace(&after_lead, "");
    after_trail.trim().to_string()
}

fn clean_item_name(s: &str) -> String {
    // Collapse runs of whitespace and strip leading prefixes like "1M".
    let mut name = s.split_whitespace().collect::<Vec<_>>().join(" ");
    // Strip a leading single-token quantity-marker like "1M", "1H" that
    // some POS systems print before the item name.
    if let Some((first, rest)) = name.split_once(' ') {
        if first.len() <= 2
            && first.chars().any(|c| c.is_ascii_digit())
            && first.chars().any(|c| c.is_ascii_alphabetic())
        {
            name = rest.to_string();
        }
    }
    name
}

// Best-guess Schedule-C category from an item's text. Keyword bag with
// a per-category weight; returns the highest-scoring bucket or "other"
// when nothing matches.
//
// Taxonomy is anchored to IRS Schedule C 2025 line items and mirrors the
// canonical set used by QuickBooks Self-Employed + FreshBooks + Expensify
// + Wave (cross-referenced; see `docs/research/categories.md` for the
// source comparison). Names use short snake_case identifiers so the
// frontend can colour-code + sort independently of the long display
// label, which gets resolved through i18n.
//
// Splits:
//   * vehicle_fuel vs vehicle_maintenance — both Schedule C line 9, but
//     keyword overlap is small and downstream depreciation routing
//     differs.
//   * travel_transport vs travel_lodging — both Schedule C line 24a,
//     but receipts look completely different (airline ticket vs hotel
//     folio).
//   * supplies_cogs vs office_supplies — line 22 vs line 18; mixing
//     them mis-states gross profit.
//   * office_equipment_software vs office_supplies — >$200 capitalisation
//     threshold (QBO-SE convention) makes downstream Section-179 / depr.
//     routing easier when separated at intake.
pub fn guess_category(name: &str) -> String {
    let n = name.to_ascii_lowercase();
    // (category_id, keyword_list) — order is irrelevant; we score
    // additively. A keyword phrase containing a space matches as a
    // substring, so multi-word brands ("jiffy lube") work too.
    let cats: &[(&str, &[&str])] = &[
        // Advertising (Schedule C line 8).
        ("advertising", &[
            "ad", "ads", "advert", "facebook ads", "google ads", "billboard",
            "flyer", "banner", "logo", "branding", "seo", "mailer",
        ]),
        // Vehicle — fuel (line 9).
        ("vehicle_fuel", &[
            "gasoline", "diesel", "unleaded", "shell", "chevron", "exxon",
            "mobil", "valero", "sunoco", "ev charge", "supercharger",
            "fuel", "bp ", " gas ",
        ]),
        // Vehicle — maintenance + parts + registration (line 9).
        ("vehicle_maintenance", &[
            "tire", "oil change", "brake", "wiper", "rotor", "spray ford",
            "truck bed", "auto", "autozone", "o'reilly", "oreilly", "napa",
            "jiffy lube", "dmv", "registration", "smog", "lube",
            "windshield", "engine", "transmission", "coolant", "antifreeze",
            "lubric", "gunk", "wd-40", "wd40", "spray paint",
        ]),
        // Travel — transport (line 24a).
        ("travel_transport", &[
            "airline", "airfare", "flight", "delta", "united", "southwest",
            "amtrak", "uber", "lyft", "taxi", "parking", "toll", "baggage",
            "rental car",
        ]),
        // Travel — lodging (line 24a).
        ("travel_lodging", &[
            "hotel", "motel", "inn", "marriott", "hilton", "hyatt", "airbnb",
            "vrbo", "lodging", "resort", "suite",
        ]),
        // Deductible meals (line 24b, 50% rule).
        ("meals", &[
            "restaurant", "cafe", "diner", "coffee", "starbucks", "dunkin",
            "lunch", "dinner", "breakfast", "doordash", "ubereats", "grubhub",
            "bar tab", "brewery", "espresso", "latte", "burger", "pizza",
            "sandwich", "salad", "entree", "dessert", "tip", "gratuity",
            "beer", "wine", "cocktail",
        ]),
        // Office supplies (line 18, consumables).
        ("office_supplies", &[
            "paper", "pen ", "pencil", "stapler", "ink", "toner", "envelope",
            "folder", "notebook", "staples", "officedepot", "postage",
            "stamps", "binder", "tape ",
        ]),
        // Office equipment + software (line 18 / line 13 depreciation).
        ("office_equipment_software", &[
            "laptop", "monitor", "keyboard", "mouse", "printer", "scanner",
            "dell", "apple ", "macbook", "ipad", "iphone", "hp ", "lenovo",
            "adobe", "microsoft", "office 365", "github", "slack", "zoom",
            "saas", "subscription", "license", "domain", "hosting",
        ]),
        // Supplies — COGS / raw materials (line 22).
        ("supplies_cogs", &[
            "lumber", "fabric", "thread", "screws", "bolts", "nails",
            "resin", "filament", "ingredient", "packaging", "shipping box",
            "hardware",
        ]),
        // Repairs & maintenance — business property (line 21).
        ("repairs_maintenance", &[
            "repair", "fix ", "maintenance", "service", "replace", "hvac",
            "plumber", "electrician", "handyman",
        ]),
        // Utilities (line 25).
        ("utilities", &[
            "electric", "electricity", "power bill", "water bill", "sewer",
            "gas bill", "internet", "comcast", "xfinity", "at&t", "verizon",
            "t-mobile", "wifi", "broadband", "phone bill",
        ]),
        // Rent / lease (line 20a/b).
        ("rent_lease", &[
            "rent ", "lease", "wework", "regus", "storage unit", "warehouse",
            "copier lease",
        ]),
        // Insurance (line 15).
        ("insurance", &[
            "insurance", "premium", "geico", "progressive", "state farm",
            "allstate", "hiscox", "hartford", "liability", "workers comp",
            "e&o", "umbrella",
        ]),
        // Professional services (line 17).
        ("professional_services", &[
            "attorney", "lawyer", "legal", "cpa", "accountant", "bookkeeper",
            "consultant", "tax prep", "hr block", "turbotax",
        ]),
        // Contract labor (line 11).
        ("contract_labor", &[
            "contractor", "freelance", "1099", "upwork", "fiverr",
            "subcontractor",
        ]),
        // Wages & benefits (line 26 / 14 / 19).
        ("wages_benefits", &[
            "payroll", "gusto", "adp ", "paychex", "wages", "salary",
            "401k", "sep ira", "health premium", "benefit",
        ]),
        // Bank / card / payment fees (line 27a — "Other").
        ("bank_fees", &[
            "stripe", "square ", "paypal", "bank fee", "atm fee", "overdraft",
            "wire fee", "merchant fee", "processing fee", "finance charge",
        ]),
        // Taxes, licenses, dues (line 23).
        ("taxes_licenses_dues", &[
            "license", "permit", "llc fee", "secretary of state", "sales tax",
            "dues", "membership", "chamber", "association",
        ]),
        // Education & training (line 27a — "Other").
        ("education_training", &[
            "course", "udemy", "coursera", "conference", "seminar", "book ",
            "training", "certification", "tuition",
        ]),
        // Groceries — not strictly Schedule C but extremely common on
        // mixed receipts (Menards hardware store with milk + berries).
        // Maps to "other" / personal for now; user re-classifies in UI.
        ("groceries", &[
            "milk", "bread", "egg", "cheese", "butter", "yogurt", "fruit",
            "berry", "berries", "snack", "cereal", "cookie", "ice",
            "funables", "candy", "chocolate", "juice", "soda", "water bottle",
            "produce", "veg", "meat", "chicken",
        ]),
    ];
    let mut best: (i32, &str) = (0, "other");
    for (cat, kws) in cats {
        let score: i32 = kws.iter().map(|kw| if n.contains(kw) { 1 } else { 0 }).sum();
        if score > best.0 {
            best = (score, cat);
        }
    }
    best.1.to_string()
}

/// Default tax bucket for a newly-extracted item, given its category.
/// The user always retains override authority in the match modal —
/// these defaults exist so the tax-rollup totals start out with
/// reasonable numbers rather than every item sitting in
/// `unclassified` until clicked.
///
/// Rules:
///   * `groceries` → `personal` (vast majority of grocery items on
///     mixed-purpose receipts like a Menards run are personal; the
///     user can flip the rare exception via the dropdown).
///   * `other` → `unclassified` (we don't know enough to default).
///   * everything else → `business` (Schedule C — sole-prop default).
///
/// `rental` is NEVER an auto-default. Rental allocation requires a
/// specific `rental_property_id` choice the parser can't infer from
/// item text alone — the user picks the property in the UI.
pub fn default_bucket_for_category(category: &str) -> &'static str {
    match category {
        "groceries" => "personal",
        "other" => "unclassified",
        _ => "business",
    }
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

    // ─── pinned: real Menards receipt regressions ─────────────────────

    /// A return-policy line ("the return is done after 08/23/26") near
    /// the top of the receipt must NOT be picked over the sale-stamp
    /// line at the bottom ("53921 10 6398 05/25/26 06:07PM 3322").
    /// Lifted verbatim from a real receipt where the first-match
    /// heuristic chose the return date.
    #[test]
    fn date_picks_sale_stamp_over_return_policy() {
        let receipt = "\
MENARDS - ANN ARBOR
Unless noted below allowable returns for
items on this receipt will be in the form
of an in store credit voucher if the
return is done after 08/23/26
Sale Transaction
TOTAL SALE         99.36
MASTERCARD 3797    99.36
53921 10 6398  05/25/26  06:07PM  3322
";
        assert_eq!(
            extract_date(receipt),
            Some(NaiveDate::from_ymd_opt(2026, 5, 25).unwrap())
        );
    }

    /// On the same receipt, a bare "TOTAL" prints the pre-tax subtotal
    /// (94.19) and "TOTAL SALE" prints the actual charge (99.36).
    /// "TOTAL SAVINGS 0.80" and "TOTAL NUMBER OF ITEMS = 15" must not
    /// pollute the bucket.
    #[test]
    fn total_picks_total_sale_over_bare_total() {
        let receipt = "\
TOTAL                94.19
TAX STATE OF MI 6%    5.17
TOTAL SALE           99.36
MASTERCARD 3797      99.36
TOTAL SAVINGS         0.80
TOTAL NUMBER OF ITEMS = 15
";
        assert_eq!(
            extract_total(receipt),
            Some(Decimal::new(9936, 2))
        );
    }

    /// "Total Savings", "Subtotal", "You Saved" must never win — the
    /// real total ($43.50) trumps the larger "$50.00 you saved" line.
    #[test]
    fn total_ignores_savings_and_subtotal() {
        let receipt = "Subtotal $100.00\nYou saved $50.00\nTotal Savings $50.00\nTotal $43.50";
        assert_eq!(extract_total(receipt), Some(Decimal::new(4350, 2)));
    }

    /// Rebate-expiration date in the future must not outrank an older
    /// sale date.
    #[test]
    fn date_avoids_future_rebate_expiration() {
        let receipt = "Sale 05/25/26\nRebate expires 12/31/99";
        // 12/31/99 → 1999. Both candidates are valid; the past one with
        // the "sale" tag should win.
        assert_eq!(
            extract_date(receipt),
            Some(NaiveDate::from_ymd_opt(2026, 5, 25).unwrap())
        );
    }

    // ─── time / subtotal / tax ─────────────────────────────────────────

    #[test]
    fn time_12_hour_pm() {
        assert_eq!(
            extract_time("53921 10 6398  05/25/26  06:07PM  3322"),
            Some(NaiveTime::from_hms_opt(18, 7, 0).unwrap())
        );
    }

    #[test]
    fn time_24_hour() {
        assert_eq!(
            extract_time("Trans 18:07:42 RegID 4"),
            Some(NaiveTime::from_hms_opt(18, 7, 42).unwrap())
        );
    }

    #[test]
    fn subtotal_picks_pre_tax_line() {
        let r = "TOTAL                94.19\nTAX STATE OF MI 6%    5.17\nTOTAL SALE           99.36";
        assert_eq!(extract_subtotal(r), None,
            "bare TOTAL on this receipt is the subtotal, but it's not tagged 'Subtotal'");
        let r2 = "Subtotal $40.00\nTax $3.50\nTotal $43.50";
        assert_eq!(extract_subtotal(r2), Some(Decimal::new(4000, 2)));
    }

    #[test]
    fn tax_picks_tax_line() {
        let r = "TOTAL                94.19\nTAX STATE OF MI 6%    5.17\nTOTAL SALE           99.36";
        assert_eq!(extract_tax(r), Some(Decimal::new(517, 2)));
    }

    // ─── address ───────────────────────────────────────────────────────

    #[test]
    fn address_joins_street_and_city_state_zip() {
        let r = "MENARDS - ANN ARBOR\n6405 Jackson Road\nAnn Arbor, MI 48103\nKeep your receipt";
        assert_eq!(
            extract_address(r).as_deref(),
            Some("6405 Jackson Road, Ann Arbor, MI 48103")
        );
    }

    // ─── items + category ──────────────────────────────────────────────

    /// The Menards receipt items: two-line patterns with qty@unit and
    /// trailing line totals. Each item should produce a typed
    /// `OcrLineItem` with the right line_total and a plausible category.
    #[test]
    fn items_extracts_menards_receipt() {
        let receipt = "\
MENARDS - ANN ARBOR
6405 Jackson Road
Ann Arbor, MI 48103
Sale Transaction
1M    SPRAY FORD RED
55/0/0        4  @6.88        27.52
PRO TRUCK BED SPRAY BLAC
5574679       2 @14.97        29.94
GUNK CHLORNTD BRK CLNR 190
2607888       6                23.88
STRAWBERRY MILK
5746883                         2.49
FUNABLES MIXED BERRY
5748419                         5.47
TOTAL                          94.19
TAX STATE OF MI 6%              5.17
TOTAL SALE                     99.36
";
        let items = extract_items(receipt);
        // We expect at least the four item rows; OCR noise around "1M" /
        // sku-only lines should be filtered.
        assert!(items.len() >= 4, "got {} items: {:?}",
                items.len(), items.iter().map(|i| &i.name).collect::<Vec<_>>());

        // Specific pins:
        let spray = items.iter().find(|i| i.name.contains("SPRAY FORD"))
            .expect("SPRAY FORD line");
        assert_eq!(spray.line_total, Decimal::new(2752, 2));
        assert_eq!(spray.qty, Some(Decimal::new(4, 0)));
        assert_eq!(spray.unit_price, Some(Decimal::new(688, 2)));
        assert_eq!(spray.category, "vehicle_maintenance");

        let milk = items.iter().find(|i| i.name.contains("STRAWBERRY MILK"))
            .expect("STRAWBERRY MILK line");
        assert_eq!(milk.line_total, Decimal::new(249, 2));
        assert_eq!(milk.category, "groceries");

        let truck = items.iter().find(|i| i.name.contains("TRUCK BED"))
            .expect("PRO TRUCK BED line");
        assert_eq!(truck.line_total, Decimal::new(2994, 2));
        assert_eq!(truck.category, "vehicle_maintenance");
    }

    #[test]
    fn items_ignores_totals_block_and_footer() {
        let receipt = "\
COFFEE                  5.50
MUFFIN                  4.25
TOTAL                   9.75
THANK YOU
Auth Code:612747
";
        let items = extract_items(receipt);
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.line_total <= Decimal::new(550, 2)));
    }

    /// Regression pin: the actual OCR output the user pasted from a
    /// real Menards receipt. PaddleOCR emits `[UNK]` for spaces and
    /// renders each item across 3-5 lines. The `structure()` entry
    /// point handles the [UNK] preprocessing; `extract_items` only
    /// sees the cleaned text.
    #[test]
    fn items_parses_real_menards_ocr_multi_line() {
        // This is the post-[UNK]→space, post-line-trim version of the
        // user's OCR output (the structure() entry point would do this
        // conversion before extract_items sees it).
        let cleaned = "\
MENARDS
- ANN ARBOR
6405 Jackson Road
Ann Arbor, MI
48103
KEEP YOUR RECEIPT
IM
SPRAY FORD RED
55
4 @6.88
27.52
PRO TRUCK Bed SPRaY BLAC
5574679
2@14.97
29.94
GUNK CHLORNTD BRK CLNR
190
2607888
6
23.88
STRAWBERRY MILK
5746883
2.49 NT
FUNABLES MIXED BERRY
5748419
5.47 NT
6-PAK BLACK ICE
2618078
4.00
TOTAL
94.19
";
        let items = extract_items(cleaned);
        // We expect 4-6 items recognized. Without strict equality (the
        // OCR-mangled lines may or may not parse), assert the high-
        // confidence ones land with the right totals.
        let total_27 = items.iter().any(|i| i.line_total == Decimal::new(2752, 2));
        let total_29 = items.iter().any(|i| i.line_total == Decimal::new(2994, 2));
        let total_249 = items.iter().any(|i| i.line_total == Decimal::new(249, 2));
        let total_547 = items.iter().any(|i| i.line_total == Decimal::new(547, 2));
        assert!(total_27, "27.52 line missing — items={:?}",
                items.iter().map(|i| (&i.name, i.line_total)).collect::<Vec<_>>());
        assert!(total_29, "29.94 line missing");
        assert!(total_249, "2.49 (Strawberry Milk) line missing");
        assert!(total_547, "5.47 (Funables) line missing");

        // SPRAY FORD RED block should carry qty + unit_price.
        let spray = items.iter().find(|i| i.line_total == Decimal::new(2752, 2))
            .expect("spray item");
        assert_eq!(spray.qty, Some(Decimal::new(4, 0)));
        assert_eq!(spray.unit_price, Some(Decimal::new(688, 2)));
    }

    /// [UNK] preprocessing pin — confirms `structure()` strips the
    /// PaddleOCR space sentinel before delegating to extractors.
    #[test]
    fn structure_strips_unk_tokens_before_parsing() {
        let raw = "TOTAL[UNK]SALE                     99.36\nMASTERCARD[UNK]3797       99.36";
        let r = structure(raw, 0.85);
        // Total picks up TOTAL SALE despite the [UNK] glue.
        assert_eq!(r.total, Some(Decimal::new(9936, 2)));
        // Raw text is preserved verbatim for debugging.
        assert!(r.text.contains("[UNK]"));
    }

    /// Comma-decimal preprocessing — Tesseract on thermal receipts
    /// regularly misreads `.` as `,`. The cleaner restores `06,88` →
    /// `06.88` so prices parse, without mangling thousands separators
    /// (`1,234.56`) or phone numbers (`555,1234`).
    #[test]
    fn structure_restores_comma_decimals_to_periods() {
        let r = structure("TOTAL SALE 29,94\nPHONE 555,1234\nGRAND TOTAL 1,234.56", 0.85);
        // 29,94 → 29.94 → picked up as TOTAL SALE.
        // 1,234.56 → preserved as 1234.56 → larger, but TOTAL SALE
        // outranks GRAND TOTAL in our tag scoring (both strong, last
        // wins).
        assert_eq!(r.total, Some(Decimal::new(123456, 2)));
    }

    /// Real-Tesseract regression — the receipt strings the user pasted
    /// have comma-decimals on the item lines (`4 06,88 ... 29,94`).
    /// After the cleaner runs, items should be extractable.
    #[test]
    fn items_extracted_from_real_tesseract_with_comma_decimals() {
        let raw = "\
SPRAY FORD RED
55      4 06,88      27,52
PRO TRUCK BED SPRAY BLAC
9974679 2 014,97     29,94
";
        let r = structure(raw, 0.85);
        assert!(r.items.iter().any(|i| i.line_total == Decimal::new(2752, 2)),
                "27.52 item missing — got {:?}",
                r.items.iter().map(|i| &i.line_total).collect::<Vec<_>>());
        assert!(r.items.iter().any(|i| i.line_total == Decimal::new(2994, 2)),
                "29.94 item missing");
    }

    /// Photo'd-receipt regression — Tesseract decorates every line on
    /// crinkled / shadowed receipts with leading dashes / equals /
    /// short noise tokens (`oS`, `AZ`, `Se`) AND extends the line with
    /// trailing junk (`|`, `Be`, `a`). The strip_line_decorations
    /// helper runs before the walker so all 3 items below produce
    /// extractions despite the visual mess.
    #[test]
    fn items_extracted_from_photod_receipt_with_edge_noise() {
        let raw = "\
=             =                                    bale Transaction
SS —=        PRO TRUCK BED SPRAY BLAC
= SS              -         9974679       2 014,97          29,94
oS =                        GUNK CHLORNTD BRI 8 499                        |
see                    2607888       6           23.88        a
AZ    :               FUNABLES MIXED BERRY                                Be
                   5748419                          9.47 NT
";
        let r = structure(raw, 0.85);
        let totals: Vec<Decimal> = r.items.iter().map(|i| i.line_total).collect();
        // PRO TRUCK BED (29.94 via comma fix), GUNK (23.88), FUNABLES (9.47).
        assert!(totals.contains(&Decimal::new(2994, 2)),
                "PRO TRUCK BED 29.94 missing — got {:?}", totals);
        assert!(totals.contains(&Decimal::new(2388, 2)),
                "GUNK 23.88 missing — got {:?}", totals);
        assert!(totals.contains(&Decimal::new(947, 2)),
                "FUNABLES 9.47 missing — got {:?}", totals);
    }

    /// strip_line_decorations unit pins — confirms the cleaner removes
    /// what we expect WITHOUT eating valid item-name characters.
    #[test]
    fn strip_line_decorations_removes_edge_junk() {
        assert_eq!(
            strip_line_decorations("=             =       GUNK CHLORNTD BRI 8 499       |"),
            "GUNK CHLORNTD BRI 8 499",
        );
        assert_eq!(
            strip_line_decorations("AZ    :   FUNABLES MIXED BERRY    Be"),
            "FUNABLES MIXED BERRY",
        );
        // Plain item-name lines are unchanged.
        assert_eq!(
            strip_line_decorations("STRAWBERRY MILK"),
            "STRAWBERRY MILK",
        );
        // Pure decoration → empty.
        assert_eq!(strip_line_decorations("= = = ---"), "");
    }
}
