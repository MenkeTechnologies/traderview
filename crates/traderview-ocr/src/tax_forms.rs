//! Tax-form OCR extractor.
//!
//! Detects + parses the common boxed-form types a taxpayer uploads
//! into the wizard:
//!   * W-2 — Wage and Tax Statement
//!   * 1099-NEC — Nonemployee Compensation
//!   * 1099-MISC — Miscellaneous Information
//!   * 1099-INT — Interest Income
//!   * 1099-DIV — Dividends and Distributions
//!   * 1099-K — Payment Card and 3rd-Party Network Transactions
//!
//! Strategy:
//!   1. Detect kind by scanning for the form's title strings (case-
//!      insensitive). The form title appears in large type at the
//!      top of every IRS form, so OCR catches it reliably.
//!   2. For each known box label, search line-by-line; capture the
//!      first dollar amount on the same line or the next line.
//!   3. Return a structured `TaxFormExtract` with a typed `payload`
//!      keyed by box number. The wizard reads this directly into
//!      `TaxReturn.w2s[]` / `interest_income` / etc.
//!
//! Quality target: the parser is paired with the receipt OCR ensemble
//! (Vision + Tesseract + PaddleOCR), so the text input is already the
//! best multi-engine output. We can rely on box labels being
//! recognized; what's brittle is *the numeric amount being on the
//! same OCR line as its label*. We compensate with a small lookahead.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxFormKind {
    W2,
    Form1099Nec,
    Form1099Misc,
    Form1099Int,
    Form1099Div,
    Form1099K,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxFormExtract {
    pub kind: TaxFormKind,
    /// Box-number → dollar amount. Keys are stable: `"box_1"`,
    /// `"box_2"`, etc. for W-2; `"box_1"`, `"box_4"`, etc. for 1099s.
    pub payload: HashMap<String, Decimal>,
    /// Employer / payer name when extractable.
    pub party_name: Option<String>,
    /// Confidence — fraction of expected boxes recovered.
    pub confidence: f32,
}

/// Detect which tax form the OCR text appears to be from, or None.
pub fn detect(text: &str) -> Option<TaxFormKind> {
    let lower = text.to_lowercase();
    // Order matters: 1099-NEC must match before plain "1099" hits.
    if lower.contains("form w-2") || lower.contains("wage and tax statement") {
        return Some(TaxFormKind::W2);
    }
    if lower.contains("1099-nec") || lower.contains("nonemployee compensation") {
        return Some(TaxFormKind::Form1099Nec);
    }
    if lower.contains("1099-misc") || lower.contains("miscellaneous information") {
        return Some(TaxFormKind::Form1099Misc);
    }
    if lower.contains("1099-int") || lower.contains("interest income") {
        return Some(TaxFormKind::Form1099Int);
    }
    if lower.contains("1099-div") || lower.contains("dividends and distributions") {
        return Some(TaxFormKind::Form1099Div);
    }
    if lower.contains("1099-k") || lower.contains("payment card") {
        return Some(TaxFormKind::Form1099K);
    }
    None
}

/// Extract every box value we can find. Returns `None` only when the
/// kind is unknown — partial extractions are returned with reduced
/// confidence so the wizard can still pre-fill what it can.
pub fn extract(text: &str) -> Option<TaxFormExtract> {
    let kind = detect(text)?;
    let labels = box_labels_for(kind);
    let mut payload: HashMap<String, Decimal> = HashMap::new();

    for (box_key, label_patterns) in labels {
        for label in label_patterns.iter() {
            if let Some(amount) = find_amount_near_label(text, label) {
                payload.insert(box_key.to_string(), amount);
                break;
            }
        }
    }

    // Confidence = recovered / expected, clamped.
    let expected = labels.len() as f32;
    let recovered = payload.len() as f32;
    let confidence = if expected > 0.0 { recovered / expected } else { 0.0 };

    let party_name = extract_party_name(text, kind);

    Some(TaxFormExtract {
        kind,
        payload,
        party_name,
        confidence,
    })
}

/// Box labels per form kind. Listed in IRS-canonical order; each entry
/// can have multiple label variants (full text or short form) so OCR
/// noise like "Wages tips other comp." still hits.
fn box_labels_for(kind: TaxFormKind) -> &'static [(&'static str, &'static [&'static str])] {
    match kind {
        TaxFormKind::W2 => &[
            ("box_1",  &["Wages, tips, other compensation", "Wages tips"]),
            ("box_2",  &["Federal income tax withheld", "Federal income tax"]),
            ("box_3",  &["Social security wages", "SS wages"]),
            ("box_4",  &["Social security tax withheld", "SS tax withheld"]),
            ("box_5",  &["Medicare wages and tips", "Medicare wages"]),
            ("box_6",  &["Medicare tax withheld"]),
            ("box_7",  &["Social security tips", "SS tips"]),
            ("box_12", &["See instructions for box 12", "Box 12"]),
            ("box_17", &["State income tax"]),
        ],
        TaxFormKind::Form1099Nec => &[
            ("box_1", &["Nonemployee compensation"]),
            ("box_4", &["Federal income tax withheld"]),
        ],
        TaxFormKind::Form1099Misc => &[
            ("box_1", &["Rents"]),
            ("box_2", &["Royalties"]),
            ("box_3", &["Other income"]),
            ("box_4", &["Federal income tax withheld"]),
            ("box_7", &["Payer made direct sales"]),
        ],
        TaxFormKind::Form1099Int => &[
            ("box_1", &["Interest income"]),
            ("box_2", &["Early withdrawal penalty"]),
            ("box_3", &["Interest on U.S. Savings Bonds", "Interest on US Savings Bonds"]),
            ("box_4", &["Federal income tax withheld"]),
            ("box_8", &["Tax-exempt interest", "Tax exempt interest"]),
        ],
        TaxFormKind::Form1099Div => &[
            ("box_1a", &["Total ordinary dividends", "Ordinary dividends"]),
            ("box_1b", &["Qualified dividends"]),
            ("box_2a", &["Total capital gain distr", "Capital gain"]),
            ("box_4",  &["Federal income tax withheld"]),
        ],
        TaxFormKind::Form1099K => &[
            ("box_1a", &["Gross amount of payment card", "Gross amount"]),
            ("box_4",  &["Federal income tax withheld"]),
        ],
    }
}

/// Find a dollar amount near a label. Scans line-by-line for the label
/// (case-insensitive substring match), then looks for an amount:
///   * later on the SAME line, OR
///   * anywhere on the NEXT line (up to 2 lines down).
///
/// Returns the first match. Amounts must look like a numeric: optional
/// `$`, digits with commas, optional `.cents`. Negative amounts in
/// parens (`(1,234.56)`) are returned as negative.
fn find_amount_near_label(text: &str, label: &str) -> Option<Decimal> {
    let lower_label = label.to_lowercase();
    let lines: Vec<&str> = text.lines().collect();
    for (i, raw_line) in lines.iter().enumerate() {
        let line_lower = raw_line.to_lowercase();
        if !line_lower.contains(&lower_label) {
            continue;
        }
        // Try the matching line first — most W-2 layouts put the
        // number to the right of the label on the same line.
        let label_end = line_lower.find(&lower_label).map(|p| p + lower_label.len()).unwrap_or(0);
        let after_label = &raw_line[label_end.min(raw_line.len())..];
        if let Some(d) = parse_amount(after_label) {
            return Some(d);
        }
        // Look up to 2 lines down for the value (some OCR engines
        // break label and value onto consecutive lines).
        for j in 1..=2 {
            if let Some(next) = lines.get(i + j) {
                if let Some(d) = parse_amount(next) {
                    return Some(d);
                }
            }
        }
    }
    None
}

/// Parse the first numeric amount in a string. Tolerates `$`, commas,
/// parens for negatives.
fn parse_amount(s: &str) -> Option<Decimal> {
    // Walk character-by-character to find the start of a number; bail
    // out the first time we hit one and try to parse.
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_digit() || c == '-' || c == '(' {
            // Slurp through the end of the numeric token.
            let start = i;
            let mut neg = false;
            if c == '-' || c == '(' {
                if c == '(' { neg = true; }
                i += 1;
            }
            let num_start = i;
            while i < bytes.len() {
                let cc = bytes[i] as char;
                if cc.is_ascii_digit() || cc == '.' || cc == ',' {
                    i += 1;
                } else {
                    break;
                }
            }
            let raw = &s[num_start..i];
            let clean: String = raw.chars().filter(|c| *c != ',').collect();
            if let Ok(mut d) = clean.parse::<Decimal>() {
                // Re-apply negative if the token was paren-wrapped.
                if neg { d = -d; }
                // Reject zero-length / pure-noise tokens.
                if !clean.is_empty() && clean != "." {
                    let _ = start;
                    return Some(d);
                }
            }
            // Couldn't parse — keep scanning the rest of the line.
            i += 1;
            continue;
        }
        i += 1;
    }
    None
}

/// Best-effort employer/payer name extraction. W-2 puts the employer
/// name in a labeled section; 1099s use "PAYER'S name". We pick the
/// first non-empty line after the label.
fn extract_party_name(text: &str, kind: TaxFormKind) -> Option<String> {
    let needles: &[&str] = match kind {
        TaxFormKind::W2 => &["employer's name", "Employer's name"],
        _               => &["payer's name", "PAYER'S name", "Payer name"],
    };
    let lower = text.to_lowercase();
    let lines: Vec<&str> = text.lines().collect();
    for needle in needles {
        if let Some(pos) = lower.find(&needle.to_lowercase()) {
            // Convert byte-pos to line index by counting newlines.
            let line_idx = lower[..pos].matches('\n').count();
            // First non-empty line AFTER the label.
            for j in 1..=3 {
                if let Some(line) = lines.get(line_idx + j) {
                    let t = line.trim();
                    if !t.is_empty()
                        && !t.to_lowercase().contains("address")
                        && !t.to_lowercase().contains("zip")
                    {
                        return Some(t.to_string());
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_w2_by_title() {
        assert_eq!(detect("Form W-2 Wage and Tax Statement"), Some(TaxFormKind::W2));
    }

    #[test]
    fn detect_1099_nec_by_label() {
        assert_eq!(detect("Form 1099-NEC\nNonemployee Compensation"), Some(TaxFormKind::Form1099Nec));
    }

    #[test]
    fn detect_returns_none_for_random_text() {
        assert_eq!(detect("This is not a tax form"), None);
    }

    #[test]
    fn parse_amount_handles_dollar_and_commas() {
        assert_eq!(parse_amount("$1,234.56"), Some("1234.56".parse().unwrap()));
        assert_eq!(parse_amount("  $50,000.00"), Some("50000.00".parse().unwrap()));
        assert_eq!(parse_amount("Total: 9,999.99"), Some("9999.99".parse().unwrap()));
    }

    #[test]
    fn parse_amount_handles_parens_negative() {
        assert_eq!(parse_amount("(1,234.56)"), Some("-1234.56".parse().unwrap()));
    }

    #[test]
    fn parse_amount_returns_none_for_no_numeric() {
        assert_eq!(parse_amount("nothing here"), None);
    }

    #[test]
    fn extract_w2_basic_box1_box2() {
        let text = "\
Form W-2 Wage and Tax Statement
Employer's name, address, and ZIP
Acme Corp
1 Main St
Wages, tips, other compensation 50000.00
Federal income tax withheld 6000.00
Social security wages 50000.00
Social security tax withheld 3100.00
Medicare wages and tips 50000.00
Medicare tax withheld 725.00
";
        let r = extract(text).expect("should detect W-2");
        assert_eq!(r.kind, TaxFormKind::W2);
        assert_eq!(r.payload.get("box_1"), Some(&"50000.00".parse().unwrap()));
        assert_eq!(r.payload.get("box_2"), Some(&"6000.00".parse().unwrap()));
        assert_eq!(r.payload.get("box_3"), Some(&"50000.00".parse().unwrap()));
        assert_eq!(r.payload.get("box_4"), Some(&"3100.00".parse().unwrap()));
        assert_eq!(r.payload.get("box_5"), Some(&"50000.00".parse().unwrap()));
        assert_eq!(r.payload.get("box_6"), Some(&"725.00".parse().unwrap()));
        assert_eq!(r.party_name.as_deref(), Some("Acme Corp"));
        // 6 of 9 expected boxes → confidence ~0.66.
        assert!(r.confidence > 0.5);
    }

    #[test]
    fn extract_w2_label_on_one_line_value_on_next() {
        let text = "\
Form W-2 Wage and Tax Statement
Wages, tips, other compensation
42500.00
Federal income tax withheld
4250.00
";
        let r = extract(text).expect("should detect W-2");
        assert_eq!(r.payload.get("box_1"), Some(&"42500.00".parse().unwrap()));
        assert_eq!(r.payload.get("box_2"), Some(&"4250.00".parse().unwrap()));
    }

    #[test]
    fn extract_1099_int_pulls_interest_and_withholding() {
        let text = "\
Form 1099-INT
PAYER'S name:
First National Bank
Interest income 1234.56
Federal income tax withheld 0.00
Tax-exempt interest 500.00
";
        let r = extract(text).expect("should detect 1099-INT");
        assert_eq!(r.kind, TaxFormKind::Form1099Int);
        assert_eq!(r.payload.get("box_1"), Some(&"1234.56".parse().unwrap()));
        assert_eq!(r.payload.get("box_4"), Some(&"0.00".parse().unwrap()));
        assert_eq!(r.payload.get("box_8"), Some(&"500.00".parse().unwrap()));
        assert_eq!(r.party_name.as_deref(), Some("First National Bank"));
    }

    #[test]
    fn extract_1099_nec_for_freelance_income() {
        let text = "\
Form 1099-NEC Nonemployee Compensation
PAYER'S name
Big Client LLC
Nonemployee compensation 25000.00
Federal income tax withheld 0
";
        let r = extract(text).expect("should detect 1099-NEC");
        assert_eq!(r.kind, TaxFormKind::Form1099Nec);
        assert_eq!(r.payload.get("box_1"), Some(&"25000.00".parse().unwrap()));
    }

    #[test]
    fn extract_returns_none_for_non_tax_form() {
        assert!(extract("just some text without form markers").is_none());
    }

    #[test]
    fn extract_1099_div_pulls_ordinary_qualified_and_ltcg_boxes() {
        let text = "\
Form 1099-DIV Dividends and Distributions
PAYER'S name
Vanguard Brokerage
Total ordinary dividends 4523.18
Qualified dividends 3200.00
Total capital gain distr. 850.45
Federal income tax withheld 0.00
";
        let r = extract(text).expect("should detect 1099-DIV");
        assert_eq!(r.kind, TaxFormKind::Form1099Div);
        assert_eq!(r.payload.get("box_1a"), Some(&"4523.18".parse().unwrap()));
        assert_eq!(r.payload.get("box_1b"), Some(&"3200.00".parse().unwrap()));
        assert_eq!(r.payload.get("box_2a"), Some(&"850.45".parse().unwrap()));
        assert_eq!(r.payload.get("box_4"),  Some(&"0.00".parse().unwrap()));
        assert_eq!(r.party_name.as_deref(), Some("Vanguard Brokerage"));
    }

    #[test]
    fn extract_1099_misc_for_rental_landlord() {
        let text = "\
Form 1099-MISC Miscellaneous Information
PAYER'S name
Tenant Property Mgmt LLC
Rents 24000.00
Royalties 0
Other income 0
Federal income tax withheld 0.00
";
        let r = extract(text).expect("should detect 1099-MISC");
        assert_eq!(r.kind, TaxFormKind::Form1099Misc);
        assert_eq!(r.payload.get("box_1"), Some(&"24000.00".parse().unwrap()));
        assert_eq!(r.payload.get("box_4"), Some(&"0.00".parse().unwrap()));
    }

    #[test]
    fn extract_1099_k_payment_card_gross() {
        // 1099-K is the form gig-workers + small sellers receive from
        // Stripe/Square/PayPal/Etsy. Gross transaction volume goes in
        // box 1a.
        let text = "\
Form 1099-K
PAYER'S name
Stripe Inc.
Gross amount of payment card / third party network transactions 87432.19
Federal income tax withheld 0.00
";
        let r = extract(text).expect("should detect 1099-K");
        assert_eq!(r.kind, TaxFormKind::Form1099K);
        assert_eq!(r.payload.get("box_1a"), Some(&"87432.19".parse().unwrap()));
    }

    #[test]
    fn detect_precedence_1099_misc_does_not_match_int() {
        // 1099-MISC contains 'income' which 1099-INT's label
        // "interest income" overlaps with. The detector must scan for
        // form-specific tokens, not generic substrings — verify INT
        // doesn't shadow MISC.
        let text = "Form 1099-MISC Miscellaneous Information\nOther income 5000.00";
        let r = detect(text);
        assert_eq!(r, Some(TaxFormKind::Form1099Misc),
            "form-MISC must take precedence even when 'income' appears");
    }

    #[test]
    fn extract_w2_with_negative_box_value() {
        // Real-world W-2 amendments occasionally carry negative values
        // (correction filings). Parser must NOT silently drop them —
        // the wizard surfaces them for review.
        let text = "\
Form W-2 Wage and Tax Statement
Employer's name
Test Corp
Wages, tips, other compensation (1500.00)
Federal income tax withheld 0.00
";
        let r = extract(text).expect("should detect W-2");
        assert_eq!(r.payload.get("box_1"), Some(&"-1500.00".parse().unwrap()));
    }

    #[test]
    fn extract_amount_finds_value_with_dollar_sign() {
        // Some OCR engines preserve the `$` glyph; others strip it.
        // We tolerate both — verify the `$`-prefixed amount parses.
        let text = "\
Form 1099-NEC
PAYER'S name
Acme Studios
Nonemployee compensation $12,345.67
";
        let r = extract(text).expect("should detect 1099-NEC");
        assert_eq!(r.payload.get("box_1"), Some(&"12345.67".parse().unwrap()));
    }

    #[test]
    fn confidence_proportional_to_boxes_recovered() {
        // 1099-INT has 5 known box labels. A complete extraction →
        // confidence ≈ 1.0. A 1-of-5 extraction → ~0.2.
        let full = "\
Form 1099-INT
Interest income 100
Early withdrawal penalty 5
Interest on U.S. Savings Bonds 0
Federal income tax withheld 0
Tax-exempt interest 50
";
        let r_full = extract(full).unwrap();
        assert!(r_full.confidence > 0.95, "full match should be ~1.0, got {}", r_full.confidence);

        let sparse = "Form 1099-INT\nInterest income 100";
        let r_sparse = extract(sparse).unwrap();
        assert!(r_sparse.confidence < 0.5 && r_sparse.confidence > 0.0,
            "1-of-5 should be ~0.2, got {}", r_sparse.confidence);
    }
}
