//! Weighted Midprice / Microprice — order-book-aware midprice.
//!
//! Stoikov (2017) microprice:
//!   microprice = (bid_price · ask_size + ask_price · bid_size) /
//!                (bid_size + ask_size)
//!
//! The microprice biases toward the side with LESS size at the top of
//! book (the side likely to trade through soonest), forecasting short-
//! horizon midprice movement better than the simple midpoint. Widely
//! used in market-making, execution algos, and HFT.
//!
//! Also reports:
//!   - Standard midpoint (bid+ask)/2
//!   - Quote imbalance: (bid_size − ask_size) / (bid_size + ask_size)
//!   - Microprice deviation from midpoint
//!
//! Pure compute. Operates on per-snapshot top-of-book quotes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Quote {
    pub bid_price: f64,
    pub bid_size: f64,
    pub ask_price: f64,
    pub ask_size: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MicropriceReport {
    pub midpoint: f64,
    pub microprice: f64,
    pub microprice_minus_midpoint: f64,
    pub quote_imbalance: f64,
    pub spread: f64,
    pub relative_spread: f64,
}

pub fn compute(q: &Quote) -> Option<MicropriceReport> {
    if !q.bid_price.is_finite() || q.bid_price <= 0.0
        || !q.ask_price.is_finite() || q.ask_price <= 0.0
        || !q.bid_size.is_finite() || q.bid_size <= 0.0
        || !q.ask_size.is_finite() || q.ask_size <= 0.0
        || q.bid_price > q.ask_price
    {
        return None;
    }
    let mid = (q.bid_price + q.ask_price) / 2.0;
    let total_size = q.bid_size + q.ask_size;
    if total_size <= 0.0 { return None; }
    // Stoikov microprice: bid_p weighted by ASK size, ask_p by BID size.
    let micro = (q.bid_price * q.ask_size + q.ask_price * q.bid_size) / total_size;
    let imbalance = (q.bid_size - q.ask_size) / total_size;
    let spread = q.ask_price - q.bid_price;
    let rel_spread = spread / mid;
    Some(MicropriceReport {
        midpoint: mid,
        microprice: micro,
        microprice_minus_midpoint: micro - mid,
        quote_imbalance: imbalance,
        spread,
        relative_spread: rel_spread,
    })
}

/// Batch convenience: compute the microprice for a series of quotes,
/// returning a vector of `Option<MicropriceReport>` aligned to input.
pub fn series(quotes: &[Quote]) -> Vec<Option<MicropriceReport>> {
    quotes.iter().map(compute).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(bp: f64, bs: f64, ap: f64, as_: f64) -> Quote {
        Quote { bid_price: bp, bid_size: bs, ask_price: ap, ask_size: as_ }
    }

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(compute(&q(bad, 100.0, 100.10, 100.0)).is_none());
            assert!(compute(&q(100.00, bad, 100.10, 100.0)).is_none());
            assert!(compute(&q(100.00, 100.0, bad, 100.0)).is_none());
            assert!(compute(&q(100.00, 100.0, 100.10, bad)).is_none());
        }
    }

    #[test]
    fn crossed_book_rejected() {
        assert!(compute(&q(100.10, 100.0, 100.00, 100.0)).is_none());
    }

    #[test]
    fn balanced_book_yields_microprice_equal_midpoint() {
        // Equal sides → microprice == midpoint, imbalance = 0.
        let r = compute(&q(100.00, 100.0, 100.10, 100.0)).unwrap();
        assert!((r.microprice - r.midpoint).abs() < 1e-12);
        assert!(r.quote_imbalance.abs() < 1e-12);
    }

    #[test]
    fn larger_bid_size_pushes_microprice_toward_ask() {
        // Bigger bid → more buyers → likely to trade through ask sooner.
        // Microprice should lean toward ask.
        let r = compute(&q(100.00, 1_000.0, 100.10, 100.0)).unwrap();
        assert!(r.microprice > r.midpoint);
        assert!(r.quote_imbalance > 0.0);
    }

    #[test]
    fn larger_ask_size_pushes_microprice_toward_bid() {
        let r = compute(&q(100.00, 100.0, 100.10, 1_000.0)).unwrap();
        assert!(r.microprice < r.midpoint);
        assert!(r.quote_imbalance < 0.0);
    }

    #[test]
    fn extreme_bid_size_caps_microprice_at_ask() {
        // bid_size → ∞ vs ask_size finite: microprice → ask_price.
        let r = compute(&q(100.00, 1e9, 100.10, 1.0)).unwrap();
        assert!((r.microprice - 100.10).abs() < 1e-6);
    }

    #[test]
    fn extreme_ask_size_caps_microprice_at_bid() {
        let r = compute(&q(100.00, 1.0, 100.10, 1e9)).unwrap();
        assert!((r.microprice - 100.00).abs() < 1e-6);
    }

    #[test]
    fn imbalance_bounded_in_minus_one_to_one() {
        // Extreme-bid book → imbalance → +1 (asymptote, never quite); ask → −1.
        // At 1e9 vs 1.0, ratio = (1e9 − 1)/(1e9 + 1) ≈ 0.999999998 →
        // natural tolerance is f64 rounding, not arbitrary 1e-9.
        let r_bid = compute(&q(100.00, 1e9, 100.10, 1.0)).unwrap();
        let r_ask = compute(&q(100.00, 1.0, 100.10, 1e9)).unwrap();
        assert!((r_bid.quote_imbalance - 1.0).abs() < 1e-6);
        assert!((r_ask.quote_imbalance + 1.0).abs() < 1e-6);
        // Bounded by definition in [-1, +1].
        assert!((-1.0..=1.0).contains(&r_bid.quote_imbalance));
        assert!((-1.0..=1.0).contains(&r_ask.quote_imbalance));
    }

    #[test]
    fn series_returns_one_entry_per_input() {
        let quotes = vec![
            q(100.00, 100.0, 100.10, 100.0),
            q(101.00, 50.0, 101.20, 100.0),
            q(99.00, 200.0, 99.05, 200.0),
        ];
        let s = series(&quotes);
        assert_eq!(s.len(), 3);
        assert!(s.iter().all(|x| x.is_some()));
    }

    #[test]
    fn relative_spread_equals_spread_over_mid() {
        let r = compute(&q(100.00, 100.0, 100.10, 100.0)).unwrap();
        let expected = 0.10 / 100.05;
        assert!((r.relative_spread - expected).abs() < 1e-9);
    }
}
