//! Bid/Ask Volume Ratio — cumulative trade-side imbalance over a
//! rolling window.
//!
//!   ratio_t = Σ bid_volume / Σ ask_volume   over the last `period` bars
//!
//! Where `bid_volume` is volume executed at or below the bid (sellers
//! hitting bids → bearish flow) and `ask_volume` is volume at or above
//! the ask (buyers lifting offers → bullish flow). The Lee-Ready tick
//! rule or quote-trade matching at ingest classifies each trade.
//!
//! Interpretation:
//!   ratio > 1.5 → strong sell pressure
//!   ratio < 0.67 → strong buy pressure
//!   ratio ≈ 1.0 → balanced order flow
//!
//! Pure compute. Default period = 60 (e.g. 60 1-minute bars = 1 hour).
//! Companion to `order_flow`, `tape_speed`, `cumulative_delta`,
//! `depth_imbalance`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub bid_volume: f64,
    pub ask_volume: f64,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period < 2 || n < period {
        return out;
    }
    if bars.iter().any(|b| {
        !b.bid_volume.is_finite()
            || !b.ask_volume.is_finite()
            || b.bid_volume < 0.0
            || b.ask_volume < 0.0
    }) {
        return out;
    }
    let mut bid_sum: f64 = bars[..period].iter().map(|b| b.bid_volume).sum();
    let mut ask_sum: f64 = bars[..period].iter().map(|b| b.ask_volume).sum();
    if ask_sum > 0.0 {
        out[period - 1] = Some(bid_sum / ask_sum);
    }
    for i in period..n {
        bid_sum += bars[i].bid_volume - bars[i - period].bid_volume;
        ask_sum += bars[i].ask_volume - bars[i - period].ask_volume;
        if ask_sum > 0.0 {
            out[i] = Some(bid_sum / ask_sum);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(bid: f64, ask: f64) -> Bar {
        Bar {
            bid_volume: bid,
            ask_volume: ask,
        }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(100.0, 100.0); 100];
        assert!(compute(&bars, 1).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut bars = vec![b(100.0, 100.0); 100];
        bars[5] = b(f64::NAN, 100.0);
        assert!(compute(&bars, 60).iter().all(|x| x.is_none()));
        let mut bars2 = vec![b(100.0, 100.0); 100];
        bars2[5] = b(-1.0, 100.0);
        assert!(compute(&bars2, 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn balanced_flow_yields_one() {
        let bars = vec![b(100.0, 100.0); 100];
        let r = compute(&bars, 60);
        for v in r.iter().flatten() {
            assert!((v - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn buy_pressure_yields_low_ratio() {
        // 100 ask vol, 50 bid vol → ratio 0.5 (buyers lifting offers).
        let bars = vec![b(50.0, 100.0); 100];
        let r = compute(&bars, 60);
        let last = 99;
        assert!((r[last].unwrap() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn sell_pressure_yields_high_ratio() {
        let bars = vec![b(200.0, 100.0); 100];
        let r = compute(&bars, 60);
        let last = 99;
        assert!((r[last].unwrap() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn zero_ask_yields_none() {
        let bars = vec![b(100.0, 0.0); 100];
        let r = compute(&bars, 60);
        assert!(r.iter().all(|x| x.is_none()));
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(100.0, 100.0); 100];
        assert_eq!(compute(&bars, 60).len(), 100);
    }
}
