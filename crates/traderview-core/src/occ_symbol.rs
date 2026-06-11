//! OCC option symbology — parse/format the 21-char compact form:
//! `AAPL260117C00190000` = root + YYMMDD + C/P + strike×1000 (8 digits).
//!
//! Parsing walks from the END (the tail is fixed-width: 8-digit price,
//! 1 side char, 6-digit date) so variable-length roots (SPXW, BRKB)
//! need no padding convention.

use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OccContract {
    pub underlying: String,
    pub expiry: NaiveDate,
    pub call: bool,
    pub strike: f64,
}

/// Cheap shape test: tail matches digits+side+digits and the root is
/// 1..=6 uppercase alphanumerics. Equities never look like this.
pub fn is_occ(symbol: &str) -> bool {
    parse(symbol).is_some()
}

pub fn parse(symbol: &str) -> Option<OccContract> {
    let s = symbol.trim();
    if s.len() < 16 || !s.is_ascii() {
        return None;
    }
    let (head, price) = s.split_at(s.len() - 8);
    if !price.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (head, side) = head.split_at(head.len() - 1);
    let call = match side {
        "C" => true,
        "P" => false,
        _ => return None,
    };
    if head.len() < 7 {
        return None;
    }
    let (root, date) = head.split_at(head.len() - 6);
    if !date.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if root.is_empty()
        || root.len() > 6
        || !root.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    {
        return None;
    }
    let yy: i32 = date[..2].parse().ok()?;
    let mm: u32 = date[2..4].parse().ok()?;
    let dd: u32 = date[4..6].parse().ok()?;
    let expiry = NaiveDate::from_ymd_opt(2000 + yy, mm, dd)?;
    let strike = price.parse::<u64>().ok()? as f64 / 1000.0;
    if strike <= 0.0 {
        return None;
    }
    Some(OccContract {
        underlying: root.to_string(),
        expiry,
        call,
        strike,
    })
}

/// Mid when both sides quote, last as fallback, None when the contract
/// has no usable price — a zero-bid deep OTM never fills at 0.
pub fn fill_price(bid: Option<f64>, ask: Option<f64>, last: Option<f64>) -> Option<f64> {
    match (bid, ask) {
        (Some(b), Some(a)) if b > 0.0 && a > 0.0 && a >= b => Some((a + b) / 2.0),
        _ => last.filter(|l| *l > 0.0),
    }
}

/// Expiration intrinsic value: what cash settlement pays per share.
pub fn intrinsic(call: bool, strike: f64, spot: f64) -> f64 {
    if call {
        (spot - strike).max(0.0)
    } else {
        (strike - spot).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_and_weekly_roots() {
        let c = parse("AAPL260117C00190000").unwrap();
        assert_eq!(c.underlying, "AAPL");
        assert_eq!(c.expiry, NaiveDate::from_ymd_opt(2026, 1, 17).unwrap());
        assert!(c.call);
        assert!((c.strike - 190.0).abs() < 1e-9);
        // Weekly root + fractional strike.
        let p = parse("SPXW241220P05900500").unwrap();
        assert_eq!(p.underlying, "SPXW");
        assert!(!p.call);
        assert!((p.strike - 5900.5).abs() < 1e-9);
    }

    #[test]
    fn rejects_equities_and_garbage() {
        assert!(parse("AAPL").is_none());
        assert!(parse("BRK.B").is_none());
        assert!(parse("AAPL260117X00190000").is_none()); // bad side char
        assert!(parse("AAPL26011C700190000").is_none()); // non-digit date
        assert!(parse("AAPL260117C00000000").is_none()); // zero strike
        assert!(!is_occ("TSLA"));
        assert!(is_occ("TSLA270115P00200000"));
    }

    #[test]
    fn intrinsic_pins_itm_otm_atm() {
        assert!((intrinsic(true, 190.0, 200.0) - 10.0).abs() < 1e-12); // ITM call
        assert_eq!(intrinsic(true, 190.0, 180.0), 0.0); // OTM call
        assert!((intrinsic(false, 190.0, 180.0) - 10.0).abs() < 1e-12); // ITM put
        assert_eq!(intrinsic(false, 190.0, 200.0), 0.0); // OTM put
        assert_eq!(intrinsic(true, 190.0, 190.0), 0.0); // ATM expires worthless
    }

    #[test]
    fn fill_price_prefers_mid_and_never_fills_at_zero() {
        assert_eq!(fill_price(Some(1.0), Some(1.2), Some(5.0)), Some(1.1));
        // One-sided or crossed books fall back to last.
        assert_eq!(fill_price(None, Some(1.2), Some(1.05)), Some(1.05));
        assert_eq!(fill_price(Some(0.0), Some(1.2), Some(1.05)), Some(1.05));
        // No usable price at all: None, not zero.
        assert_eq!(fill_price(None, None, None), None);
        assert_eq!(fill_price(None, None, Some(0.0)), None);
    }
}
