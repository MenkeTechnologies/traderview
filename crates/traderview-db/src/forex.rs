//! Forex (FX) spot pair support — detection + Yahoo symbol mapping.
//!
//! FX is a first-class asset class alongside equities and crypto. A
//! canonical pair is the dashless 6-letter ISO-4217 form `EURUSD`
//! (base then quote), the MT4/MT5/OANDA convention. The dash form
//! `EUR-USD` is intentionally NOT accepted: [`crate::crypto::is_crypto_pair`]
//! already claims `{BASE}-USD`, and a dashless 6-letter pair can't
//! collide with it, with OCC option symbols (which carry digits), or
//! with US equity tickers (1–5 chars).
//!
//! Quotes and bars route through the SAME Yahoo seam equities use: the
//! only FX-specific step is appending `=X` at the outbound Yahoo
//! boundary (`EURUSD` -> `EURUSD=X`). The bar and quote caches stay
//! keyed on the canonical `EURUSD`, so backtests, the optimizer, the
//! paper engine, and benchmark overlays read FX bars exactly like they
//! read equities — one seam, three asset classes.

/// ISO-4217 codes traderview quotes as FX: the eight majors plus the
/// most-traded crosses. Both halves of a pair must be in this set, so a
/// 6-letter equity ticker can't be mistaken for a currency pair.
pub const CURRENCIES: &[&str] = &[
    // The eight majors.
    "USD", "EUR", "GBP", "JPY", "CHF", "AUD", "NZD", "CAD",
    // Common crosses / EM that Yahoo quotes via `=X`.
    "CNH", "HKD", "SGD", "SEK", "NOK", "DKK", "MXN", "ZAR", "TRY", "PLN",
    "HUF", "CZK", "INR", "KRW", "THB", "ILS", "BRL",
];

/// The default pairs surfaced in the FX dashboard — the seven most
/// liquid USD majors. Order is the conventional desk order.
pub const MAJORS: &[&str] = &[
    "EURUSD", "USDJPY", "GBPUSD", "USDCHF", "AUDUSD", "USDCAD", "NZDUSD",
];

fn is_code(s: &str) -> bool {
    CURRENCIES.contains(&s)
}

/// Split a candidate into (base, quote) if it is the canonical dashless
/// 6-letter uppercase form or the `XXX/YYY` slash form. The dash form
/// is deliberately rejected — it belongs to crypto.
fn split_pair(symbol: &str) -> Option<(&str, &str)> {
    if let Some((base, quote)) = symbol.split_once('/') {
        return Some((base, quote));
    }
    if symbol.len() == 6 && symbol.bytes().all(|b| b.is_ascii_uppercase()) {
        return Some((&symbol[..3], &symbol[3..]));
    }
    None
}

/// Is this an FX spot pair the engine can quote? True only for two
/// distinct whitelisted ISO-4217 codes in canonical `EURUSD` or
/// `EUR/USD` form. Lowercase, dash form, and crypto pairs are false.
pub fn is_forex_pair(symbol: &str) -> bool {
    match split_pair(symbol) {
        Some((base, quote)) => base != quote && is_code(base) && is_code(quote),
        None => false,
    }
}

/// Normalize any accepted spelling to the canonical dashless uppercase
/// form (`eur/usd` -> `EURUSD`), or `None` if it isn't a valid pair.
pub fn normalize(symbol: &str) -> Option<String> {
    let up = symbol.trim().to_uppercase();
    let (base, quote) = split_pair(&up)?;
    (base != quote && is_code(base) && is_code(quote)).then(|| format!("{base}{quote}"))
}

/// Canonical pair -> Yahoo FX symbol (`EURUSD` -> `EURUSD=X`). Yahoo's
/// v8 chart and v7 quote endpoints both accept the 6-letter `=X` form;
/// this is the only place the suffix exists.
pub fn yahoo_symbol(canonical: &str) -> String {
    format!("{canonical}=X")
}

/// Pip size for a canonical pair: 0.01 for JPY-quoted pairs (USDJPY,
/// EURJPY, …), 0.0001 for everything else. The fourth decimal is the
/// pip for most pairs; JPY pairs quote to two decimals so the pip is
/// the second.
pub fn pip_size(canonical: &str) -> f64 {
    if canonical.ends_with("JPY") {
        0.01
    } else {
        0.0001
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn majors_are_six_letter_pairs() {
        assert!(is_forex_pair("EURUSD"));
        assert!(is_forex_pair("USDJPY"));
        assert!(is_forex_pair("GBPUSD"));
        assert!(is_forex_pair("NZDCHF"));
    }

    #[test]
    fn slash_form_accepted() {
        assert!(is_forex_pair("EUR/USD"));
        assert!(is_forex_pair("GBP/JPY"));
    }

    #[test]
    fn crypto_dash_form_is_not_forex() {
        // The dash form belongs to crypto — never claim it here, or the
        // price router would send BTC-USD to Yahoo instead of the venue.
        assert!(!is_forex_pair("BTC-USD"));
        assert!(!is_forex_pair("EUR-USD"));
        assert!(!is_forex_pair("ETH-USDT"));
    }

    #[test]
    fn equities_and_options_are_not_forex() {
        assert!(!is_forex_pair("AAPL"));
        assert!(!is_forex_pair("GOOGL"));
        assert!(!is_forex_pair("SPY"));
        // A 6-letter equity-shaped string that isn't two ISO codes.
        assert!(!is_forex_pair("ABCDEF"));
        // OCC symbols carry digits.
        assert!(!is_forex_pair("AAPL240119C00150000"));
    }

    #[test]
    fn lowercase_and_garbage_rejected() {
        assert!(!is_forex_pair("eurusd"));
        assert!(!is_forex_pair(""));
        assert!(!is_forex_pair("USD"));
        assert!(!is_forex_pair("EURUSDT")); // 7 chars, not a pair
    }

    #[test]
    fn same_currency_is_not_a_pair() {
        assert!(!is_forex_pair("USDUSD"));
        assert!(!is_forex_pair("EUR/EUR"));
    }

    #[test]
    fn normalize_canonicalizes() {
        assert_eq!(normalize("eur/usd").as_deref(), Some("EURUSD"));
        assert_eq!(normalize("  GBPusd ").as_deref(), Some("GBPUSD"));
        assert_eq!(normalize("USDJPY").as_deref(), Some("USDJPY"));
        assert_eq!(normalize("BTC-USD"), None);
        assert_eq!(normalize("AAPL"), None);
    }

    #[test]
    fn yahoo_symbol_appends_suffix() {
        assert_eq!(yahoo_symbol("EURUSD"), "EURUSD=X");
        assert_eq!(yahoo_symbol("USDJPY"), "USDJPY=X");
    }

    #[test]
    fn pip_size_is_two_decimals_for_jpy() {
        assert_eq!(pip_size("USDJPY"), 0.01);
        assert_eq!(pip_size("EURJPY"), 0.01);
        assert_eq!(pip_size("EURUSD"), 0.0001);
        assert_eq!(pip_size("GBPUSD"), 0.0001);
    }

    #[test]
    fn all_majors_detect_as_forex() {
        for m in MAJORS {
            assert!(is_forex_pair(m), "{m} should be a forex pair");
        }
    }
}
