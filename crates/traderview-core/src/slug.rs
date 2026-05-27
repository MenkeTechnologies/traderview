//! Tiny slug generator — short base62 random IDs for trade shares / forum threads.

use rand::Rng;

const ALPHABET: &[u8; 62] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub fn random(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
        .collect()
}

/// Slugify a string into [a-z0-9-]+, max 64 chars.
pub fn from_title(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = true;
    for ch in s.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
        if out.len() >= 64 {
            break;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("post");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_length_and_alphabet() {
        let s = random(12);
        assert_eq!(s.len(), 12);
        for c in s.chars() {
            assert!(
                c.is_ascii_alphanumeric(),
                "random char `{}` is not base62",
                c
            );
        }
    }

    #[test]
    fn random_is_random_across_calls() {
        // Length-16 random collisions across two calls are astronomically
        // unlikely (62^16 = 4.7e28). If this ever fails, the RNG is broken.
        let a = random(16);
        let b = random(16);
        assert_ne!(a, b);
    }

    #[test]
    fn random_handles_zero_length() {
        assert_eq!(random(0), "");
    }

    #[test]
    fn from_title_lowercases_and_dashes_spaces() {
        assert_eq!(from_title("Hello World"), "hello-world");
    }

    #[test]
    fn from_title_collapses_consecutive_non_alphanumeric() {
        assert_eq!(from_title("Hello   --  World!!"), "hello-world");
    }

    #[test]
    fn from_title_strips_trailing_dash() {
        assert_eq!(from_title("trailing dash --"), "trailing-dash");
    }

    #[test]
    fn from_title_caps_at_64_chars() {
        let long = "a".repeat(200);
        let s = from_title(&long);
        assert!(s.len() <= 64);
    }

    #[test]
    fn from_title_empty_input_falls_back_to_post() {
        // Empty/non-alphanumeric input must not return "" — URL would break.
        assert_eq!(from_title(""), "post");
        assert_eq!(from_title("!!!"), "post");
        assert_eq!(from_title("   "), "post");
    }

    #[test]
    fn from_title_keeps_digits_and_strips_punctuation() {
        assert_eq!(
            from_title("AAPL Q4 2026 — earnings beat!"),
            "aapl-q4-2026-earnings-beat"
        );
    }
}
