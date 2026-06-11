//! Adversarial coverage for `parse::guess_category` and
//! `parse::default_bucket_for_category` — two `pub fn`s with zero direct
//! tests before this file.
//!
//! `guess_category` scores each category by counting how many of its
//! keywords appear as **substrings** of the lowercased item name
//! (`parse.rs:1488` `if n.contains(kw)`), then takes the first category —
//! in declaration order — with the strictly-highest score (`parse.rs:1490`
//! `if score > best.0`). Two structural hazards fall out of that design and
//! are the bug classes these tests pin:
//!
//!   1. Short keywords match inside unrelated words. `"ad"` (the very first
//!      keyword of the first category, `advertising`, `parse.rs:1098`)
//!      matches any name containing the bigram "ad" — including the fuel
//!      label "unleaded" and the drink "Gatorade".
//!   2. Ties resolve to the earliest-declared category, not the most
//!      specific one. `advertising` is declared first, so on a tie it wins
//!      over the semantically-correct category.
//!
//! `default_bucket_for_category` is an exact, case-sensitive `match`
//! (`parse.rs:1514`); a capitalized category id silently falls through the
//! `_` arm to `"business"`.
//!
//! These tests assert the CURRENT behavior so the substring/tie/case
//! semantics are pinned — any future change to keyword length, declaration
//! order, or the bucket match will trip exactly one of them.

use traderview_ocr::parse::{default_bucket_for_category, guess_category};

/// The fuel label "Unleaded" hits TWO categories with score 1 each:
/// `advertising` via the substring "ad" inside "unle**ad**ed"
/// (`parse.rs:1098`) and `vehicle_fuel` via the exact keyword "unleaded"
/// (`parse.rs:1117`). Because `advertising` is declared first and the
/// scorer keeps the earliest category on a tie (`>` not `>=`,
/// `parse.rs:1490`), the fuel item is misclassified as advertising.
/// This pins both the "ad"-substring collision and the tie-to-first rule
/// in a single, realistic OCR token.
#[test]
fn unleaded_fuel_label_collides_with_advertising_ad_substring() {
    assert_eq!(
        guess_category("Unleaded"),
        "advertising",
        "the 'ad' substring in 'unleaded' ties advertising with vehicle_fuel; \
         advertising wins because it is declared first and the scorer keeps \
         the earliest category on a tie"
    );
}

/// A drink with no grocery keyword still lands in `advertising` purely
/// because "Gator**ad**e" contains the "ad" bigram. None of the grocery
/// keywords ("juice", "soda", "water bottle", …) are substrings of
/// "gatorade", so advertising's lone point is the global max. Pins the
/// over-broad short-keyword match in isolation (no competing category).
#[test]
fn gatorade_misclassified_as_advertising_via_ad_substring() {
    assert_eq!(
        guess_category("Gatorade"),
        "advertising",
        "'gatorade' contains 'ad'; no grocery keyword matches, so advertising \
         is the sole scorer and wins"
    );
}

/// Zero keyword hits must fall back to the sentinel `"other"`, NOT to the
/// first-declared category. The scorer seeds `best = (0, "other")` and only
/// replaces it when some category scores strictly above 0 (`parse.rs:1484`,
/// `parse.rs:1490`). Empty and pure-gibberish names exercise the
/// no-match floor — a regression that seeded `best` with a real category
/// or used `>=` would surface here.
#[test]
fn no_keyword_match_falls_back_to_other() {
    assert_eq!(guess_category(""), "other", "empty name → other");
    assert_eq!(
        guess_category("zzqxv 9981 wkjhg"),
        "other",
        "gibberish with no keyword substring → other"
    );
}

/// `default_bucket_for_category` is case-sensitive (`parse.rs:1514` matches
/// the literal lowercase ids `guess_category` emits). The three explicit
/// arms map as documented; a capitalized id — which `guess_category` never
/// produces but a caller might pass — falls through `_` to `"business"`.
/// Pins the exact bucket map AND the case-sensitivity gap so a future
/// refactor that lowercases input (or adds a `rental` auto-default) is a
/// deliberate, test-visible change.
#[test]
fn bucket_map_is_exact_and_case_sensitive() {
    // Documented explicit arms.
    assert_eq!(default_bucket_for_category("groceries"), "personal");
    assert_eq!(default_bucket_for_category("other"), "unclassified");
    // Any other lowercase category → business (Schedule C default).
    assert_eq!(default_bucket_for_category("meals"), "business");
    assert_eq!(default_bucket_for_category("vehicle_fuel"), "business");
    // rental is NEVER auto-defaulted by guess_category, but if a caller
    // passes it here it still resolves to business via the catch-all.
    assert_eq!(default_bucket_for_category("rental"), "business");
    // Case mismatch falls through `_` → business, NOT personal/unclassified.
    assert_eq!(
        default_bucket_for_category("Groceries"),
        "business",
        "capitalized 'Groceries' misses the lowercase arm and hits the \
         catch-all → business (case-sensitive match)"
    );
}
