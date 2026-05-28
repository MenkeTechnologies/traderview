//! Cross-sectional Value factor — book-to-market (B/M) ranking.
//!
//! Per-symbol B/M = (book_value_per_share) / (price_per_share).
//! Across a universe, rank into deciles:
//!   - **ValueDecile**:  top 10% B/M — cheap stocks (value tilt)
//!   - **GrowthDecile**: bottom 10% B/M — expensive stocks (growth tilt)
//!   - **Neutral**:      middle deciles
//!
//! Used by Fama-French HML construction (long value, short growth) and
//! by value-tilted portfolio managers.
//!
//! Pure compute. Filters symbols with non-finite or non-positive
//! book/price as data errors.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolFundamentals {
    pub symbol: String,
    pub book_value_per_share: f64,
    pub price_per_share: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueBucket { ValueDecile, GrowthDecile, Neutral }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueHit {
    pub symbol: String,
    pub book_to_market: f64,
    pub percentile: f64,
    pub bucket: ValueBucket,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValueFactorReport {
    pub value_decile: Vec<ValueHit>,
    pub growth_decile: Vec<ValueHit>,
    pub all_ranked: Vec<ValueHit>,
}

pub fn scan(symbols: &[SymbolFundamentals]) -> Option<ValueFactorReport> {
    if symbols.is_empty() { return None; }
    let mut ratios: Vec<(String, f64)> = Vec::new();
    for s in symbols {
        if !s.book_value_per_share.is_finite() || !s.price_per_share.is_finite()
            || s.book_value_per_share <= 0.0 || s.price_per_share <= 0.0
        {
            continue;
        }
        let bm = s.book_value_per_share / s.price_per_share;
        if bm.is_finite() {
            ratios.push((s.symbol.clone(), bm));
        }
    }
    if ratios.is_empty() { return None; }
    ratios.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let n = ratios.len();
    let mut all_ranked: Vec<ValueHit> = ratios.iter().enumerate()
        .map(|(i, (sym, bm))| {
            let pct = (i + 1) as f64 / n as f64 * 100.0;
            let bucket = if pct >= 90.0 { ValueBucket::ValueDecile }
                else if pct <= 10.0 { ValueBucket::GrowthDecile }
                else { ValueBucket::Neutral };
            ValueHit {
                symbol: sym.clone(),
                book_to_market: *bm,
                percentile: pct,
                bucket,
            }
        })
        .collect();
    all_ranked.sort_by(|a, b| b.book_to_market.partial_cmp(&a.book_to_market)
        .unwrap_or(std::cmp::Ordering::Equal));
    let value_decile: Vec<ValueHit> = all_ranked.iter()
        .filter(|h| h.bucket == ValueBucket::ValueDecile).cloned().collect();
    let growth_decile: Vec<ValueHit> = all_ranked.iter()
        .filter(|h| h.bucket == ValueBucket::GrowthDecile).cloned().collect();
    Some(ValueFactorReport { value_decile, growth_decile, all_ranked })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sym: &str, book: f64, price: f64) -> SymbolFundamentals {
        SymbolFundamentals {
            symbol: sym.into(),
            book_value_per_share: book,
            price_per_share: price,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(scan(&[]).is_none());
    }

    #[test]
    fn invalid_inputs_skipped() {
        let symbols = vec![
            s("BAD_BOOK", 0.0, 100.0),
            s("NEG_PRICE", 10.0, -1.0),
            s("NAN_BOOK", f64::NAN, 100.0),
        ];
        assert!(scan(&symbols).is_none());
    }

    #[test]
    fn deepest_value_ranked_first() {
        // Highest B/M = highest book-to-market = deepest value.
        let symbols = vec![
            s("CHEAP", 50.0, 25.0),     // B/M = 2.0
            s("FAIR", 50.0, 50.0),      // B/M = 1.0
            s("EXPENSIVE", 50.0, 100.0), // B/M = 0.5
        ];
        let r = scan(&symbols).unwrap();
        assert_eq!(r.all_ranked[0].symbol, "CHEAP");
        assert!(r.all_ranked[0].book_to_market > r.all_ranked[1].book_to_market);
    }

    #[test]
    fn top_10_pct_goes_to_value_decile() {
        // 10 symbols B/M = 1, 2, 3, ..., 10 → top 10% (the 10.0 entry).
        let symbols: Vec<_> = (1..=10).map(|i| s(&format!("S{i}"), i as f64, 1.0)).collect();
        let r = scan(&symbols).unwrap();
        assert!(!r.value_decile.is_empty());
        assert_eq!(r.value_decile[0].symbol, "S10");
    }

    #[test]
    fn bottom_10_pct_goes_to_growth_decile() {
        let symbols: Vec<_> = (1..=10).map(|i| s(&format!("S{i}"), i as f64, 1.0)).collect();
        let r = scan(&symbols).unwrap();
        assert!(!r.growth_decile.is_empty());
        assert_eq!(r.growth_decile[0].symbol, "S1");
    }

    #[test]
    fn percentiles_in_unit_range() {
        let symbols: Vec<_> = (1..=20).map(|i| s(&format!("S{i:02}"), i as f64, 1.0)).collect();
        let r = scan(&symbols).unwrap();
        for h in &r.all_ranked {
            assert!((0.0..=100.0).contains(&h.percentile));
        }
    }

    #[test]
    fn ranked_descending_by_book_to_market() {
        let symbols: Vec<_> = (1..=10).map(|i| s(&format!("S{i}"), i as f64, 1.0)).collect();
        let r = scan(&symbols).unwrap();
        for w in r.all_ranked.windows(2) {
            assert!(w[0].book_to_market >= w[1].book_to_market);
        }
    }
}
