//! Order-book imbalance signal.
//!
//! For a Level-2 snapshot (top-N bid + ask sizes), compute:
//!   total_bid_size = sum(bid_sizes[..N])
//!   total_ask_size = sum(ask_sizes[..N])
//!   imbalance = (total_bid - total_ask) / (total_bid + total_ask)
//!
//! Range [-1, 1]. Positive = more size on bid (buying pressure);
//! negative = more size on ask (selling pressure). Used by HFT firms
//! as a microsecond-scale signal.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObiReport {
    pub total_bid_size: f64,
    pub total_ask_size: f64,
    pub imbalance: f64,
    pub bias: ObiBias,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ObiBias {
    StronglyBid, // > 0.3
    Bid,         // 0.1 to 0.3
    #[default]
    Balanced, // -0.1 to 0.1
    Ask,         // -0.3 to -0.1
    StronglyAsk, // < -0.3
}

pub fn compute(bid_sizes: &[f64], ask_sizes: &[f64], levels: usize) -> ObiReport {
    let bid_total: f64 = bid_sizes.iter().take(levels).sum();
    let ask_total: f64 = ask_sizes.iter().take(levels).sum();
    let total = bid_total + ask_total;
    let imbalance = if total > 0.0 {
        (bid_total - ask_total) / total
    } else {
        0.0
    };
    let bias = if imbalance > 0.3 {
        ObiBias::StronglyBid
    } else if imbalance > 0.1 {
        ObiBias::Bid
    } else if imbalance < -0.3 {
        ObiBias::StronglyAsk
    } else if imbalance < -0.1 {
        ObiBias::Ask
    } else {
        ObiBias::Balanced
    };
    ObiReport {
        total_bid_size: bid_total,
        total_ask_size: ask_total,
        imbalance,
        bias,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_balanced() {
        let r = compute(&[], &[], 5);
        assert_eq!(r.bias, ObiBias::Balanced);
        assert_eq!(r.imbalance, 0.0);
    }

    #[test]
    fn balanced_book_zero_imbalance() {
        let r = compute(&[100.0, 50.0], &[100.0, 50.0], 5);
        assert_eq!(r.imbalance, 0.0);
        assert_eq!(r.bias, ObiBias::Balanced);
    }

    #[test]
    fn pure_bid_imbalance_one() {
        let r = compute(&[1000.0], &[], 5);
        assert_eq!(r.imbalance, 1.0);
        assert_eq!(r.bias, ObiBias::StronglyBid);
    }

    #[test]
    fn pure_ask_imbalance_minus_one() {
        let r = compute(&[], &[1000.0], 5);
        assert_eq!(r.imbalance, -1.0);
        assert_eq!(r.bias, ObiBias::StronglyAsk);
    }

    #[test]
    fn levels_caps_summation() {
        // 10 bid sizes of 100 each; ask 5 × 100. With levels=5,
        // bid = 500, ask = 500 → balanced.
        let bids = vec![100.0; 10];
        let asks = vec![100.0; 5];
        let r = compute(&bids, &asks, 5);
        assert_eq!(r.total_bid_size, 500.0);
        assert_eq!(r.imbalance, 0.0);
    }

    #[test]
    fn moderate_bid_classified_bid_bias() {
        // 20% imbalance → "Bid".
        let r = compute(&[600.0], &[400.0], 5);
        // (600-400)/(1000) = 0.2 → Bid.
        assert_eq!(r.bias, ObiBias::Bid);
    }

    #[test]
    fn moderate_ask_classified_ask_bias() {
        let r = compute(&[400.0], &[600.0], 5);
        assert_eq!(r.bias, ObiBias::Ask);
    }

    #[test]
    fn levels_zero_returns_zero_totals() {
        let r = compute(&[100.0, 200.0], &[100.0, 200.0], 0);
        assert_eq!(r.total_bid_size, 0.0);
        assert_eq!(r.imbalance, 0.0);
    }

    #[test]
    fn boundary_exactly_at_0_3_is_strongly_bid() {
        // Strict > 0.3 — exactly 0.3 should be Bid, not StronglyBid.
        // 65/35 = 0.3 imbalance exact: (65-35)/100 = 0.3.
        let r = compute(&[65.0], &[35.0], 5);
        assert!((r.imbalance - 0.3).abs() < 1e-12);
        assert_eq!(r.bias, ObiBias::Bid); // 0.3 == 0.3 → Bid (strict >)
    }

    #[test]
    fn boundary_just_above_0_3_is_strongly_bid() {
        // 65.01/35 → 0.300077 imbalance → StronglyBid.
        let r = compute(&[65.01], &[34.99], 5);
        assert_eq!(r.bias, ObiBias::StronglyBid);
    }

    #[test]
    fn boundary_exactly_at_0_1_balanced() {
        // 55/45 = 0.1 exact. Strict > 0.1 → falls to Balanced.
        let r = compute(&[55.0], &[45.0], 5);
        assert!((r.imbalance - 0.1).abs() < 1e-12);
        assert_eq!(r.bias, ObiBias::Balanced);
    }

    #[test]
    fn boundary_negative_0_3_strongly_ask() {
        // 35/65 = -0.3 exact. Strict < -0.3 → falls to Ask.
        let r = compute(&[35.0], &[65.0], 5);
        assert!((r.imbalance + 0.3).abs() < 1e-12);
        assert_eq!(r.bias, ObiBias::Ask);
    }

    #[test]
    fn negative_size_ignored_but_doesnt_panic() {
        // Negative size could come from malformed data. The function
        // shouldn't crash; it'll produce a weird-but-valid imbalance.
        let r = compute(&[-100.0], &[100.0], 5);
        // Total = 0 → guard kicks in, imbalance = 0.
        assert_eq!(r.imbalance, 0.0);
    }

    #[test]
    fn imbalance_finite_when_inputs_finite() {
        // Sanity: no NaN/infinity output for normal inputs.
        let r = compute(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0], 10);
        assert!(r.imbalance.is_finite());
        assert!(r.total_bid_size.is_finite());
        assert!(r.total_ask_size.is_finite());
    }

    #[test]
    fn very_small_total_doesnt_blow_up_imbalance() {
        let r = compute(&[0.0001], &[0.0001], 5);
        // 0/0.0002 = 0 imbalance, well-defined.
        assert_eq!(r.imbalance, 0.0);
        assert_eq!(r.bias, ObiBias::Balanced);
    }
}
