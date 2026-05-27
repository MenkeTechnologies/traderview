//! IRC §1256 60/40 tax treatment classifier.
//!
//! §1256 contracts (regulated futures contracts, foreign currency
//! contracts, broad-based index options like SPX/NDX/RUT, dealer
//! equity options, dealer securities futures contracts) get FLAT
//! 60% long-term / 40% short-term capital-gain treatment regardless
//! of holding period. They are also marked-to-market at year-end
//! (deemed sale, similar to §475(f)).
//!
//! This module:
//!   1. Classifies a symbol as §1256-eligible (best-effort by ticker
//!      prefix — caller can override with explicit flag for edge cases).
//!   2. Splits realized + mark-to-market gain into 60/40 buckets.
//!
//! Pure compute.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Returns true if the symbol pattern matches a §1256 contract type.
/// Conservative — only matches well-known patterns. Caller can override
/// with `is_1256: Some(true)` per trade for symbols this misses.
pub fn is_section_1256_symbol(symbol: &str) -> bool {
    let s = symbol.to_uppercase();
    // Broad-based index OPTIONS (not equity options on the index ETF).
    // SPX, XSP (mini-SPX), NDX, NQX, RUT, MNX, DJX, OEX, VIX options.
    let broad_index_options = [
        "SPX", "XSP", "NDX", "NQX", "RUT", "MNX", "DJX", "OEX", "VIX",
    ];
    if broad_index_options.contains(&s.as_str()) { return true; }
    // Futures roots: /ES, /NQ, /CL, /GC, /SI, /ZB, /ZN, /6E, etc.
    // Convention: leading "/" or "@" prefix or 1-3 letter root.
    if s.starts_with('/') || s.starts_with('@') { return true; }
    // Common futures roots (heuristic — most users tag with the / prefix).
    let common_futures_roots = [
        "ES", "NQ", "RTY", "YM", "MES", "MNQ", "M2K", "MYM",
        "CL", "GC", "SI", "HG", "NG", "ZB", "ZN", "ZF", "ZT",
        "6E", "6J", "6B", "6A", "6C", "6S", "BTC", "ETH",
    ];
    if common_futures_roots.contains(&s.as_str()) { return true; }
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1256Trade {
    pub symbol: String,
    pub realized_pnl: Decimal,
    /// Override the symbol-based classifier (Some(true) forces in,
    /// Some(false) forces out, None defers to classifier).
    pub is_1256: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Section1256Report {
    pub trade_count: usize,
    pub section_1256_trade_count: usize,
    pub total_1256_pnl: Decimal,
    /// 60% of total_1256_pnl, treated as long-term cap gain/loss.
    pub long_term_portion: Decimal,
    /// 40% of total_1256_pnl, treated as short-term cap gain/loss.
    pub short_term_portion: Decimal,
    /// Trades that were excluded from the §1256 calc (equities, etc.).
    pub excluded_trade_count: usize,
    pub excluded_pnl: Decimal,
}

pub fn report(trades: &[Section1256Trade]) -> Section1256Report {
    let sixty = Decimal::from_str("0.60").unwrap();
    let forty = Decimal::from_str("0.40").unwrap();
    let mut report = Section1256Report::default();
    report.trade_count = trades.len();
    for t in trades {
        let is_1256 = t.is_1256.unwrap_or_else(|| is_section_1256_symbol(&t.symbol));
        if is_1256 {
            report.section_1256_trade_count += 1;
            report.total_1256_pnl += t.realized_pnl;
        } else {
            report.excluded_trade_count += 1;
            report.excluded_pnl += t.realized_pnl;
        }
    }
    report.long_term_portion = report.total_1256_pnl * sixty;
    report.short_term_portion = report.total_1256_pnl * forty;
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    // ─── classifier ────────────────────────────────────────────────────

    #[test]
    fn spx_options_classified_as_1256() {
        assert!(is_section_1256_symbol("SPX"));
        assert!(is_section_1256_symbol("XSP"));
        assert!(is_section_1256_symbol("NDX"));
        assert!(is_section_1256_symbol("RUT"));
        assert!(is_section_1256_symbol("VIX"));
    }

    #[test]
    fn spy_etf_options_not_1256() {
        // SPY is the S&P 500 ETF — equity options, NOT §1256.
        assert!(!is_section_1256_symbol("SPY"));
        assert!(!is_section_1256_symbol("QQQ"));
        assert!(!is_section_1256_symbol("IWM"));
    }

    #[test]
    fn slash_prefix_futures_classified() {
        assert!(is_section_1256_symbol("/ES"));
        assert!(is_section_1256_symbol("/NQ"));
        assert!(is_section_1256_symbol("/CL"));
    }

    #[test]
    fn at_prefix_futures_classified() {
        assert!(is_section_1256_symbol("@ES"));
    }

    #[test]
    fn equities_not_classified() {
        assert!(!is_section_1256_symbol("AAPL"));
        assert!(!is_section_1256_symbol("MSFT"));
        assert!(!is_section_1256_symbol("TSLA"));
    }

    #[test]
    fn case_insensitive() {
        assert!(is_section_1256_symbol("spx"));
        assert!(is_section_1256_symbol("/es"));
    }

    // ─── report ────────────────────────────────────────────────────────

    #[test]
    fn empty_returns_default() {
        let r = report(&[]);
        assert_eq!(r.trade_count, 0);
        assert_eq!(r.long_term_portion, Decimal::ZERO);
    }

    #[test]
    fn sixty_forty_split_on_gain() {
        let trades = vec![Section1256Trade {
            symbol: "SPX".into(),
            realized_pnl: d("10000"),
            is_1256: None,
        }];
        let r = report(&trades);
        assert_eq!(r.long_term_portion, d("6000.00"));
        assert_eq!(r.short_term_portion, d("4000.00"));
    }

    #[test]
    fn sixty_forty_split_on_loss() {
        // Losses also split 60/40.
        let trades = vec![Section1256Trade {
            symbol: "/ES".into(),
            realized_pnl: d("-5000"),
            is_1256: None,
        }];
        let r = report(&trades);
        assert_eq!(r.long_term_portion, d("-3000.00"));
        assert_eq!(r.short_term_portion, d("-2000.00"));
    }

    #[test]
    fn non_1256_trades_excluded_from_split() {
        let trades = vec![
            Section1256Trade { symbol: "SPX".into(),  realized_pnl: d("1000"), is_1256: None },
            Section1256Trade { symbol: "AAPL".into(), realized_pnl: d("500"),  is_1256: None },
        ];
        let r = report(&trades);
        assert_eq!(r.total_1256_pnl, d("1000"));
        assert_eq!(r.excluded_pnl, d("500"));
        assert_eq!(r.section_1256_trade_count, 1);
        assert_eq!(r.excluded_trade_count, 1);
    }

    #[test]
    fn explicit_is_1256_override_forces_classification() {
        // Some exotic symbol the classifier misses.
        let trades = vec![Section1256Trade {
            symbol: "FOOBAR".into(),
            realized_pnl: d("1000"),
            is_1256: Some(true),     // user-tagged
        }];
        let r = report(&trades);
        assert_eq!(r.section_1256_trade_count, 1);
        assert_eq!(r.total_1256_pnl, d("1000"));
    }

    #[test]
    fn explicit_false_override_excludes_otherwise_matching_symbol() {
        // SPX would auto-classify but user knows it's actually an
        // equity-option flavor — force exclude.
        let trades = vec![Section1256Trade {
            symbol: "SPX".into(),
            realized_pnl: d("1000"),
            is_1256: Some(false),
        }];
        let r = report(&trades);
        assert_eq!(r.section_1256_trade_count, 0);
        assert_eq!(r.excluded_trade_count, 1);
    }
}
