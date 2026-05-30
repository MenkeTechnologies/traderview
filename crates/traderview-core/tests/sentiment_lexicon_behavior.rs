//! Targeted behavior pins for `traderview_core::sentiment`.
//!
//! Complements the 4 inline tests in `src/sentiment.rs` with explicit
//! coverage of: intensifier multiplier, negation window length, exclamation
//! amplification, normalization bounds, empty/whitespace input, the `n't`
//! suffix negation path, ticker length cap, ticker case rules, dollar-prefix
//! cashtag bypass of the whitelist, and `score_post` composition.

use std::collections::HashSet;
use traderview_core::sentiment::{extract_tickers, score, score_post};

// ── Score range invariants ────────────────────────────────────────

/// `score()` output is bounded by tanh — must never escape `(-1, 1)`.
#[test]
fn score_is_bounded_by_tanh() {
    // Even a wall of positive tokens can't push past 1.0.
    let text = "moon moon moon moon moon rocket rocket rocket rocket rocket \
                yolo yolo yolo yolo tendies tendies tendies bullish bullish bullish";
    let s = score(text);
    assert!(s < 1.0, "score must be < 1.0, got {s}");
    assert!(s > 0.99, "very positive text should approach 1.0, got {s}");
}

/// Empty input yields exactly 0.0 (tanh(0) = 0).
#[test]
fn empty_text_scores_zero() {
    assert_eq!(score(""), 0.0);
}

/// Whitespace-only input yields 0.0 (no lexicon hits).
#[test]
fn whitespace_only_scores_zero() {
    assert_eq!(score("   \t  \n  "), 0.0);
}

/// Text with no lexicon-matching tokens yields 0.0.
#[test]
fn unknown_words_score_zero() {
    assert_eq!(score("the quick brown fox jumped"), 0.0);
}

// ── Intensifier multiplier (1.5x for "very" / "extremely") ────────

/// "very bullish" > "bullish" — the intensifier multiplies the next token.
#[test]
fn very_intensifier_amplifies_next_token() {
    let plain = score("bullish");
    let intensified = score("very bullish");
    assert!(
        intensified > plain,
        "'very bullish' ({intensified}) should be > 'bullish' ({plain})"
    );
}

/// "extremely" is also an intensifier per `intensifiers()` table.
#[test]
fn extremely_intensifies_next_token() {
    let plain = score("bullish");
    let intensified = score("extremely bullish");
    assert!(intensified > plain);
}

/// Diminisher ("slightly") *reduces* magnitude (multiplier 0.5).
#[test]
fn slightly_reduces_magnitude() {
    let plain = score("bullish");
    let weak = score("slightly bullish");
    assert!(weak < plain && weak > 0.0);
}

// ── Negation flips sign for the next 3 tokens ─────────────────────

/// "not bullish" flips the sign of "bullish".
#[test]
fn not_negates_next_lexicon_hit() {
    let positive = score("bullish");
    let negated = score("not bullish");
    assert!(positive > 0.0);
    assert!(
        negated < 0.0,
        "'not bullish' should be negative, got {negated}"
    );
}

/// "never moon" — same negation behavior with a different trigger word.
#[test]
fn never_negates_next_lexicon_hit() {
    let s = score("never moon");
    assert!(s < 0.0, "'never moon' should be negative, got {s}");
}

/// `n't` suffix triggers negation regardless of base word.
#[test]
fn nt_suffix_triggers_negation() {
    let plain = score("bullish");
    let neg = score("isn't bullish");
    assert!(plain > 0.0);
    assert!(
        neg < plain,
        "isn't should negate bullish (was {plain}, now {neg})"
    );
}

/// Negation window is 3 tokens — a 4th matching token escapes the flip.
#[test]
fn negation_window_is_three_tokens() {
    // "not the worst possible day, bullish" — by token 5 (bullish) the
    // window has expired and bullish is no longer negated.
    let no_neg = score("bullish");
    let after_window = score("not one two three four bullish");
    assert!(after_window > 0.0);
    assert!(
        (after_window - no_neg).abs() < 1e-9,
        "after 3-token window, score should equal plain (no_neg={no_neg}, after_window={after_window})"
    );
}

// ── Exclamation amplification ─────────────────────────────────────

/// `!` after a positive context adds positive momentum (+0.4 in implementation).
#[test]
fn exclamation_amplifies_existing_positive() {
    let plain = score("bullish");
    let amplified = score("bullish!");
    assert!(amplified > plain);
}

/// `!` after a negative context drives the score MORE negative.
#[test]
fn exclamation_amplifies_existing_negative() {
    let plain = score("bagholder");
    let amplified = score("bagholder!");
    assert!(amplified < plain);
}

// ── Lexicon mixed-sign cancellation ───────────────────────────────

/// Equal-weight positive and negative tokens roughly cancel.
#[test]
fn pos_and_neg_tokens_cancel_roughly() {
    let s = score("bullish bearish");
    assert!(s.abs() < 0.1, "bullish + bearish should ~cancel, got {s}");
}

// ── Ticker extraction edge cases ──────────────────────────────────

/// Cashtag (`$`-prefixed) bypasses the whitelist for unknown tickers
/// as long as the symbol is <= 5 chars (real US ticker limit).
#[test]
fn cashtag_overrides_whitelist_for_unknown_ticker() {
    let wl: HashSet<String> = ["AAPL"].iter().map(|s| s.to_string()).collect();
    // 4 chars, not in whitelist — cashtag still extracts it.
    let ts = extract_tickers("Buying $RIVN today", &wl);
    assert!(ts.contains(&"RIVN".to_string()));
}

/// Bare uppercase not in whitelist is NOT picked up.
#[test]
fn bare_uppercase_not_in_whitelist_skipped() {
    let wl: HashSet<String> = ["AAPL"].iter().map(|s| s.to_string()).collect();
    let ts = extract_tickers("RANDO and AAPL", &wl);
    assert!(ts.contains(&"AAPL".to_string()));
    assert!(!ts.contains(&"RANDO".to_string()));
}

/// Tickers longer than 5 chars are skipped (real US tickers max 5).
#[test]
fn ticker_longer_than_five_chars_skipped() {
    let wl: HashSet<String> = HashSet::new();
    let ts = extract_tickers("$TOOLONG should be skipped", &wl);
    assert!(!ts.contains(&"TOOLONG".to_string()));
}

/// Lowercase tokens — even matching a whitelist member literally — must
/// NOT match (whitelist is uppercase, and bare-mixed-case is filtered).
#[test]
fn lowercase_ticker_not_extracted() {
    let wl: HashSet<String> = ["AAPL"].iter().map(|s| s.to_string()).collect();
    let ts = extract_tickers("aapl is undervalued", &wl);
    assert!(!ts.contains(&"AAPL".to_string()));
}

/// Duplicate tickers are deduplicated.
#[test]
fn duplicate_tickers_deduplicated() {
    let wl: HashSet<String> = ["AAPL"].iter().map(|s| s.to_string()).collect();
    let ts = extract_tickers("$AAPL $AAPL AAPL", &wl);
    assert_eq!(ts.iter().filter(|t| *t == "AAPL").count(), 1);
}

// ── `score_post` composition ──────────────────────────────────────

/// `score_post` carries text verbatim, score from `score()`, and tickers
/// from `extract_tickers()`.
#[test]
fn score_post_combines_text_score_tickers() {
    let wl: HashSet<String> = ["AAPL"].iter().map(|s| s.to_string()).collect();
    let post = score_post("AAPL to the moon 🚀", &wl);
    assert_eq!(post.text, "AAPL to the moon 🚀");
    assert!(post.score > 0.0);
    assert!(post.tickers.contains(&"AAPL".to_string()));
}

/// `score_post` preserves the original case of the text but lowercases
/// only internally during tokenization.
#[test]
fn score_post_preserves_original_text_case() {
    let wl: HashSet<String> = HashSet::new();
    let post = score_post("BULLISH RALLY", &wl);
    assert_eq!(post.text, "BULLISH RALLY");
}
