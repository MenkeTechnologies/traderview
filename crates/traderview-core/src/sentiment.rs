//! Lexicon-based sentiment scorer, WSB-aware. Returns a score in [-1.0, +1.0].
//!
//! The scorer is a stripped-down VADER-style algorithm:
//!   * tokenize on whitespace + punctuation
//!   * sum lexicon weights for matching tokens
//!   * apply intensifiers ("very", "extremely") and negation flips
//!   * normalize to [-1, +1] via tanh
//!
//! WSB-specific tokens carry larger weights than general financial words —
//! "rocket", "moon", "yolo", "tendies", "diamond hands" all map to strong
//! positives; "bagholder", "rugpull", "rekt", "puts" map to negatives.
//!
//! Ticker extraction lifts `$AAPL`-style cashtags and bare-uppercase tokens
//! that pass a configurable whitelist (we use cached `symbols` from Yahoo).

use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize)]
pub struct Scored {
    pub text: String,
    pub score: f64,           // [-1.0, +1.0]
    pub tickers: Vec<String>, // unique uppercase tickers found
}

// ---- Lexicon ---------------------------------------------------------------

fn lexicon() -> HashMap<&'static str, f64> {
    let mut m = HashMap::new();
    // ---- WSB / retail-trader slang (high weight, captures the vibe) ----
    let pos = &[
        ("moon", 2.5),
        ("mooning", 2.5),
        ("moonshot", 2.5),
        ("rocket", 2.5),
        ("🚀", 2.5),
        ("yolo", 2.0),
        ("tendies", 2.0),
        ("printing", 2.0),
        ("diamond", 2.0),
        ("hodl", 2.0),
        ("squeeze", 2.0),
        ("breakout", 1.8),
        ("ripping", 2.0),
        ("ath", 1.5),
        ("rally", 1.5),
        ("pump", 1.5),
        ("pumping", 1.7),
        ("bullish", 2.0),
        ("calls", 1.4),
        ("long", 1.2),
        ("undervalued", 1.5),
        ("upgrade", 1.5),
        ("beat", 1.5),
        ("crushed", 1.6),
        ("buy", 1.0),
        ("buying", 1.0),
        ("ripped", 1.8),
        ("ripped", 1.8),
        ("green", 1.0),
        ("strong", 1.0),
        ("up", 0.6),
    ];
    let neg = &[
        ("bagholder", -2.5),
        ("bagholding", -2.5),
        ("rugpull", -2.5),
        ("rug", -2.0),
        ("rekt", -2.5),
        ("dumped", -2.0),
        ("dumping", -2.0),
        ("crash", -2.5),
        ("crashing", -2.5),
        ("tank", -2.0),
        ("tanking", -2.0),
        ("plunge", -2.0),
        ("plunging", -2.0),
        ("puts", -1.5),
        ("short", -1.2),
        ("bearish", -2.0),
        ("downgrade", -1.5),
        ("miss", -1.5),
        ("missed", -1.5),
        ("missing", -1.2),
        ("guh", -2.5),
        ("fucked", -2.0),
        ("destroyed", -2.0),
        ("loss", -1.2),
        ("losses", -1.2),
        ("losing", -1.2),
        ("red", -1.0),
        ("sell", -1.0),
        ("dump", -1.5),
        ("overvalued", -1.5),
        ("bubble", -1.5),
        ("worthless", -2.0),
        ("collapse", -2.5),
    ];
    for (w, s) in pos {
        m.insert(*w, *s);
    }
    for (w, s) in neg {
        m.insert(*w, *s);
    }
    m
}

fn intensifiers() -> HashMap<&'static str, f64> {
    let mut m = HashMap::new();
    for w in &[
        "very",
        "extremely",
        "super",
        "absolutely",
        "incredibly",
        "massively",
        "huge",
        "literally",
    ] {
        m.insert(*w, 1.5);
    }
    for w in &["slightly", "somewhat", "kinda", "kind"] {
        m.insert(*w, 0.5);
    }
    m
}

fn negations() -> HashSet<&'static str> {
    [
        "not", "no", "never", "n't", "without", "ain't", "isn't", "wasn't", "won't",
    ]
    .iter()
    .copied()
    .collect()
}

// ---- Tokenizer -------------------------------------------------------------

fn tokenize(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    for ch in s.chars() {
        if ch.is_alphanumeric() || ch == '\'' || ch == '$' || ch == '🚀' {
            buf.push(ch);
        } else {
            if !buf.is_empty() {
                out.push(std::mem::take(&mut buf).to_lowercase());
            }
            // Keep punctuation as its own token so "!" can intensify.
            if !ch.is_whitespace() {
                out.push(ch.to_string());
            }
        }
    }
    if !buf.is_empty() {
        out.push(buf.to_lowercase());
    }
    out
}

// ---- Scorer ----------------------------------------------------------------

pub fn score(text: &str) -> f64 {
    let lex = lexicon();
    let inten = intensifiers();
    let neg = negations();

    let tokens = tokenize(text);
    let mut sum = 0.0;
    let mut intensifier = 1.0;
    let mut negated_window = 0usize;

    for tok in &tokens {
        // Exclamation marks add to the previous token's score.
        if tok == "!" {
            sum += if sum >= 0.0 { 0.4 } else { -0.4 };
            continue;
        }
        if let Some(mult) = inten.get(tok.as_str()) {
            intensifier *= mult;
            continue;
        }
        if neg.contains(tok.as_str()) || tok.ends_with("n't") {
            negated_window = 3; // next 3 tokens flip sign
            continue;
        }
        if let Some(weight) = lex.get(tok.as_str()) {
            let mut s = weight * intensifier;
            if negated_window > 0 {
                s = -s;
            }
            sum += s;
        }
        negated_window = negated_window.saturating_sub(1);
        // Reset intensifier after applying.
        if intensifier != 1.0 {
            intensifier = 1.0;
        }
    }

    // Normalize.
    (sum / 4.0).tanh()
}

// ---- Ticker extraction ----------------------------------------------------

pub fn extract_tickers(text: &str, whitelist: &HashSet<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for raw in text.split(|c: char| !c.is_ascii_alphanumeric() && c != '$') {
        let token = raw.trim_start_matches('$').trim();
        if token.is_empty() || token.len() > 5 {
            continue;
        }
        if !token.chars().all(|c| c.is_ascii_uppercase()) {
            continue;
        }
        // Cashtag ($AAPL) always wins; bare-uppercase only if in whitelist.
        let is_cash = raw.starts_with('$');
        if (is_cash || whitelist.contains(token)) && seen.insert(token.to_string()) {
            out.push(token.to_string());
        }
    }
    out
}

pub fn score_post(text: &str, whitelist: &HashSet<String>) -> Scored {
    Scored {
        text: text.to_string(),
        score: score(text),
        tickers: extract_tickers(text, whitelist),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bullish_phrase_positive() {
        assert!(score("AAPL to the moon 🚀 yolo calls printing!") > 0.4);
    }

    #[test]
    fn bearish_phrase_negative() {
        assert!(score("TSLA puts crashing, bagholders rekt") < -0.4);
    }

    #[test]
    fn negation_flips() {
        let pos = score("strong rally");
        let neg = score("not a strong rally");
        assert!(pos > 0.0);
        assert!(neg < pos);
    }

    #[test]
    fn cashtag_extraction() {
        let wl: HashSet<String> = ["AAPL", "MSFT"].iter().map(|s| s.to_string()).collect();
        let ts = extract_tickers("Buying $TSLA and $NVDA, AAPL hodl, also msft", &wl);
        assert!(ts.contains(&"TSLA".to_string()));
        assert!(ts.contains(&"NVDA".to_string()));
        assert!(ts.contains(&"AAPL".to_string()));
        // 'msft' lowercase should not match.
        assert!(!ts.contains(&"MSFT".to_string()));
    }
}
