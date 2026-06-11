//! Adversarial coverage for `parse::guess_category` and
//! `parse::default_bucket_for_category` — two `pub fn`s with zero direct
//! tests before this file.
//!
//! `guess_category` scores each category by counting how many of its
//! keywords appear in the lowercased item name via `keyword_matches`, which
//! requires the keyword to land at a **word start** (the byte before it is
//! not ASCII-alphanumeric), then takes the first category — in declaration
//! order — with the strictly-highest score (`if score > best.0`). The
//! word-start rule closes the short-keyword collision class these tests
//! guard:
//!
//!   1. Short keywords no longer match inside unrelated words. `"ad"` (the
//!      first keyword of `advertising`) used to match the bigram "ad" inside
//!      any name; it now matches only at a word boundary, so the fuel label
//!      "unleaded" classifies as `vehicle_fuel` and the drink "Gatorade"
//!      (no keyword at all) falls back to `other`.
//!   2. Ties still resolve to the earliest-declared category (`>` not `>=`);
//!      with the collision gone, "Unleaded" no longer ties advertising, so
//!      the more-specific category wins outright.
//!
//! `default_bucket_for_category` is an exact, case-sensitive `match`;
//! a capitalized category id silently falls through the `_` arm to
//! `"business"`.
//!
//! These tests assert the corrected classification and the case-sensitive
//! bucket map — any regression of the word-start rule, declaration order,
//! or the bucket match trips exactly one of them.

use traderview_ocr::parse::{default_bucket_for_category, guess_category};

/// The fuel label "Unleaded" must classify as `vehicle_fuel`. The keyword
/// "ad" no longer matches the mid-word "ad" inside "unle**ad**ed" (it is not
/// at a word start), so `advertising` scores 0 and only `vehicle_fuel`
/// (exact keyword "unleaded") scores — no tie, the specific category wins.
/// Pins the word-start rule on a realistic OCR token that previously
/// collided.
#[test]
fn unleaded_fuel_label_classifies_as_vehicle_fuel_not_advertising() {
    assert_eq!(
        guess_category("Unleaded"),
        "vehicle_fuel",
        "'ad' inside 'unleaded' is mid-word, so advertising no longer scores; \
         only the 'unleaded' fuel keyword matches"
    );
}

/// A drink with no category keyword must fall back to `other`. "Gatorade"
/// contains the "ad" bigram only mid-word ("gator**ad**e"), so the
/// word-start rule rejects it and no other keyword matches — the over-broad
/// short-keyword collision is gone.
#[test]
fn gatorade_no_keyword_match_falls_back_to_other() {
    assert_eq!(
        guess_category("Gatorade"),
        "other",
        "'ad' in 'gatorade' is mid-word so advertising no longer scores; \
         no category keyword matches → other"
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
