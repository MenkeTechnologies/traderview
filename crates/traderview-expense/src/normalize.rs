//! Merchant-string normalization.
//!
//! Real-world merchant strings on credit-card statements are a mess:
//!   AMZN MKTP US*RT4G81234   →  amzn mktp us
//!   SQ *BLUE BOTTLE COFFEE   →  blue bottle coffee
//!   TST* DELI WORLD #14201   →  deli world
//!   UBER   TRIP    8KX91     →  uber trip
//!
//! We strip processor prefixes, trailing IDs, multiple spaces, then lowercase.
//! The normalized form is what merchant_rules patterns match against; the raw
//! form is what we display.

use regex::Regex;
use std::sync::OnceLock;

static SQUARE_PREFIX: OnceLock<Regex> = OnceLock::new();
static TST_PREFIX: OnceLock<Regex> = OnceLock::new();
static TRAILING_IDS: OnceLock<Regex> = OnceLock::new();
static MULTI_SPACE: OnceLock<Regex> = OnceLock::new();

pub fn normalize(raw: &str) -> String {
    let s = raw.trim();

    let sq = SQUARE_PREFIX.get_or_init(|| Regex::new(r"(?i)^sq\s*\*\s*").unwrap());
    let s = sq.replace(s, "");

    let tst = TST_PREFIX.get_or_init(|| Regex::new(r"(?i)^tst\s*\*\s*").unwrap());
    let s = tst.replace(&s, "");

    // Trim trailing transaction IDs / store numbers / pound-N suffixes.
    // A trailing token qualifies as an ID if it either:
    //   * starts with `#` or `*` (eg `#14201`, `*RT4G81234`), OR
    //   * contains at least one digit (eg `8KX91`, `92127-3401`).
    // Pure-alphabetic trailing words ("COFFEE", "WORLD") are left alone.
    // Two cases:
    //   (a) `*` or `#` glued to the previous token: "AMZN MKTP US*RT4G81234"
    //       → strip everything from the `*`/`#` onward.
    //   (b) Whitespace-separated ID-shaped token: "UBER TRIP 8KX91"
    //       → strip the trailing token if it contains a digit.
    let ids = TRAILING_IDS
        .get_or_init(|| Regex::new(r"(?i)([#*]\S*$|\s+[A-Z0-9-]*\d[A-Z0-9-]*\s*$)").unwrap());
    let mut s = ids.replace(&s, "").to_string();
    // A second pass catches "STAPLES INC #14201 STORE42" patterns where the
    // inner ID is followed by another ID-shaped token.
    s = ids.replace(&s, "").to_string();

    let ms = MULTI_SPACE.get_or_init(|| Regex::new(r"\s+").unwrap());
    let s = ms.replace_all(s.trim(), " ");

    s.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_square_prefix() {
        assert_eq!(normalize("SQ *BLUE BOTTLE COFFEE"), "blue bottle coffee");
        assert_eq!(normalize("sq*Blue Bottle Coffee"), "blue bottle coffee");
    }

    #[test]
    fn strips_tst_prefix() {
        assert_eq!(normalize("TST* DELI WORLD"), "deli world");
    }

    #[test]
    fn strips_trailing_ids() {
        assert_eq!(normalize("STAPLES.COM #14201"), "staples.com");
        assert_eq!(normalize("UBER TRIP 8KX91"), "uber trip");
        assert_eq!(normalize("AMZN MKTP US*RT4G81234"), "amzn mktp us");
    }

    #[test]
    fn collapses_whitespace() {
        assert_eq!(normalize("UBER   TRIP"), "uber trip");
    }

    #[test]
    fn lowercase_pass() {
        assert_eq!(normalize("STAPLES.COM"), "staples.com");
    }
}
