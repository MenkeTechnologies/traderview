//! Microprice (Stoikov 2018) — fair-value mid that bakes in queue
//! imbalance at the inside L1 quote.
//!
//!   imbalance = bid_size / (bid_size + ask_size)
//!   microprice = bid · (1 - imbalance) + ask · imbalance
//!              = bid · (ask_size / (bid+ask)) + ask · (bid_size / (bid+ask))
//!
//! Heavy bid imbalance → microprice biases toward the ask (next trade
//! likely lifts the offer). Heavy ask imbalance → biases toward bid
//! (next trade likely hits the bid). Midpoint is the special case
//! when bid_size == ask_size.
//!
//! Per-bar output for a stream of L1 snapshots. Reports microprice
//! and the imbalance-adjusted spread bias in bps:
//!   bias_bps = (microprice - midpoint) / midpoint · 10000
//!
//! Pure compute. Companion to `weighted_midprice`, `order_flow_imbalance`,
//! `quote_imbalance`, `lee_ready`.

#[derive(Clone, Copy, Debug)]
pub struct L1Quote {
    pub bid: f64,
    pub ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
}

#[derive(Debug)]
pub struct Bar {
    pub microprice: f64,
    pub midpoint: f64,
    pub imbalance: f64,
    pub bias_bps: f64,
}

pub fn compute(quotes: &[L1Quote]) -> Vec<Option<Bar>> {
    quotes.iter().map(|q| {
        if !q.bid.is_finite() || !q.ask.is_finite()
            || !q.bid_size.is_finite() || !q.ask_size.is_finite() {
            return None;
        }
        if q.bid <= 0.0 || q.ask <= 0.0 || q.bid > q.ask { return None; }
        if q.bid_size < 0.0 || q.ask_size < 0.0 { return None; }
        let total = q.bid_size + q.ask_size;
        if total <= 0.0 { return None; }
        let imbalance = q.bid_size / total;
        let micro = q.bid * (1.0 - imbalance) + q.ask * imbalance;
        let mid = 0.5 * (q.bid + q.ask);
        let bias_bps = if mid > 0.0 { (micro - mid) / mid * 10000.0 } else { 0.0 };
        Some(Bar { microprice: micro, midpoint: mid, imbalance, bias_bps })
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(bid: f64, ask: f64, bs: f64, as_: f64) -> L1Quote {
        L1Quote { bid, ask, bid_size: bs, ask_size: as_ }
    }

    #[test]
    fn empty_input_returns_empty() {
        let r = compute(&[]);
        assert!(r.is_empty());
    }

    #[test]
    fn invalid_quote_yields_none() {
        let quotes = vec![
            q(0.0, 100.0, 100.0, 100.0),       // bid = 0
            q(-1.0, 100.0, 100.0, 100.0),      // bid < 0
            q(100.0, 99.0, 100.0, 100.0),      // crossed
            q(100.0, 101.0, 0.0, 0.0),         // zero total size
            q(f64::NAN, 100.0, 1.0, 1.0),      // NaN
        ];
        let r = compute(&quotes);
        assert!(r.iter().all(|x| x.is_none()));
    }

    #[test]
    fn balanced_sizes_yield_midpoint() {
        let r = compute(&[q(99.0, 101.0, 100.0, 100.0)]);
        let b = r[0].as_ref().unwrap();
        assert!((b.microprice - 100.0).abs() < 1e-9);
        assert!((b.bias_bps).abs() < 1e-9);
        assert!((b.imbalance - 0.5).abs() < 1e-9);
    }

    #[test]
    fn heavy_bid_size_pulls_microprice_toward_ask() {
        // bid much bigger → next trade likely lifts ask → bias toward 101.
        let r = compute(&[q(99.0, 101.0, 1000.0, 100.0)]);
        let b = r[0].as_ref().unwrap();
        assert!(b.microprice > 100.0);
        assert!(b.bias_bps > 0.0);
        assert!(b.imbalance > 0.5);
    }

    #[test]
    fn heavy_ask_size_pulls_microprice_toward_bid() {
        let r = compute(&[q(99.0, 101.0, 100.0, 1000.0)]);
        let b = r[0].as_ref().unwrap();
        assert!(b.microprice < 100.0);
        assert!(b.bias_bps < 0.0);
        assert!(b.imbalance < 0.5);
    }

    #[test]
    fn microprice_always_within_spread() {
        let r = compute(&[q(99.0, 101.0, 100.0, 50.0)]);
        let b = r[0].as_ref().unwrap();
        assert!(b.microprice >= 99.0);
        assert!(b.microprice <= 101.0);
    }

    #[test]
    fn output_length_matches_input() {
        let quotes = vec![q(99.0, 101.0, 100.0, 100.0); 5];
        let r = compute(&quotes);
        assert_eq!(r.len(), 5);
    }
}
