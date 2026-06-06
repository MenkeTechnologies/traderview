//! Effective + realized spread analysis (Lee-Ready / Bessembinder).
//!
//! Quoted, effective, and realized spreads are the three canonical
//! microstructure transaction-cost measures.
//!
//!   quoted_spread     = ask − bid
//!   effective_spread  = 2 · |trade_price − mid|     (round-trip cost)
//!   realized_spread   = 2 · D · (trade_price − mid_5min_later)
//!                       where D = +1 for buys, −1 for sells
//!   price_impact      = effective − realized
//!
//! Effective spread captures what a marketable order ACTUALLY paid vs
//! the quoted spread (often half of quoted in liquid markets).
//! Realized spread strips out adverse selection (the post-trade decay
//! of mid back through the trade price) and measures the LP's revenue
//! after the informed-flow penalty. Price impact = adverse selection cost.
//!
//! Pure compute. Caller supplies trades + paired (current_mid,
//! delayed_mid) snapshots from the quote book.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpreadObservation {
    pub trade_price: f64,
    pub current_mid: f64,
    pub delayed_mid: f64,
    pub quoted_spread: f64,
    pub direction: Direction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct SpreadReport {
    pub avg_quoted_spread: f64,
    pub avg_effective_spread: f64,
    pub avg_realized_spread: f64,
    pub avg_price_impact: f64,
    pub effective_to_quoted_ratio: f64,
    pub n_observations: usize,
}

pub fn analyze(obs: &[SpreadObservation]) -> Option<SpreadReport> {
    if obs.is_empty() {
        return None;
    }
    let mut sum_q = 0.0_f64;
    let mut sum_eff = 0.0_f64;
    let mut sum_real = 0.0_f64;
    let mut count = 0_usize;
    for o in obs {
        if !o.trade_price.is_finite()
            || o.trade_price <= 0.0
            || !o.current_mid.is_finite()
            || o.current_mid <= 0.0
            || !o.delayed_mid.is_finite()
            || o.delayed_mid <= 0.0
            || !o.quoted_spread.is_finite()
            || o.quoted_spread < 0.0
        {
            continue;
        }
        let d: f64 = match o.direction {
            Direction::Buy => 1.0,
            Direction::Sell => -1.0,
        };
        let effective = 2.0 * d * (o.trade_price - o.current_mid);
        let realized = 2.0 * d * (o.trade_price - o.delayed_mid);
        sum_q += o.quoted_spread;
        sum_eff += effective;
        sum_real += realized;
        count += 1;
    }
    if count == 0 {
        return None;
    }
    let n_f = count as f64;
    let avg_q = sum_q / n_f;
    let avg_eff = sum_eff / n_f;
    let avg_real = sum_real / n_f;
    let avg_impact = avg_eff - avg_real;
    let ratio = if avg_q > 0.0 {
        avg_eff / avg_q
    } else {
        f64::NAN
    };
    Some(SpreadReport {
        avg_quoted_spread: avg_q,
        avg_effective_spread: avg_eff,
        avg_realized_spread: avg_real,
        avg_price_impact: avg_impact,
        effective_to_quoted_ratio: ratio,
        n_observations: count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(tp: f64, mid: f64, dmid: f64, qs: f64, dir: Direction) -> SpreadObservation {
        SpreadObservation {
            trade_price: tp,
            current_mid: mid,
            delayed_mid: dmid,
            quoted_spread: qs,
            direction: dir,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(analyze(&[]).is_none());
    }

    #[test]
    fn invalid_entries_filtered() {
        let bad = vec![
            obs(0.0, 100.0, 100.0, 0.10, Direction::Buy), // bad trade price
            obs(100.10, -1.0, 100.0, 0.10, Direction::Buy),
            obs(100.10, 100.0, f64::NAN, 0.10, Direction::Buy),
        ];
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn buy_at_ask_yields_effective_eq_quoted() {
        // Trade @ ask (mid + 0.5·spread). Effective = 2 · 0.5·spread = spread.
        let observations = vec![obs(100.05, 100.00, 100.00, 0.10, Direction::Buy)];
        let r = analyze(&observations).unwrap();
        assert!((r.avg_effective_spread - 0.10).abs() < 1e-9);
    }

    #[test]
    fn sell_at_bid_yields_positive_effective() {
        // Trade @ bid (mid − 0.5·spread), direction Sell, sign flips.
        // effective = 2 · (-1) · (-0.05) = +0.10
        let observations = vec![obs(99.95, 100.00, 100.00, 0.10, Direction::Sell)];
        let r = analyze(&observations).unwrap();
        assert!((r.avg_effective_spread - 0.10).abs() < 1e-9);
    }

    #[test]
    fn no_post_trade_adverse_selection_yields_realized_eq_effective() {
        // delayed_mid == current_mid → realized = effective → impact = 0.
        let observations = vec![
            obs(100.05, 100.00, 100.00, 0.10, Direction::Buy),
            obs(99.95, 100.00, 100.00, 0.10, Direction::Sell),
        ];
        let r = analyze(&observations).unwrap();
        assert!((r.avg_realized_spread - r.avg_effective_spread).abs() < 1e-9);
        assert!(r.avg_price_impact.abs() < 1e-9);
    }

    #[test]
    fn adverse_selection_yields_positive_price_impact() {
        // Buy @ 100.05, then mid drifts to 100.10 (informed buy).
        // Realized = 2 · 1 · (100.05 - 100.10) = -0.10 (LP lost money).
        // Effective = +0.10. Impact = effective − realized = +0.20.
        let observations = vec![obs(100.05, 100.00, 100.10, 0.10, Direction::Buy)];
        let r = analyze(&observations).unwrap();
        assert!((r.avg_realized_spread + 0.10).abs() < 1e-9);
        assert!(r.avg_price_impact > 0.0);
    }

    #[test]
    fn effective_to_quoted_ratio_close_to_one_for_at_quote_trades() {
        let observations = vec![
            obs(100.05, 100.00, 100.00, 0.10, Direction::Buy),
            obs(99.95, 100.00, 100.00, 0.10, Direction::Sell),
        ];
        let r = analyze(&observations).unwrap();
        assert!((r.effective_to_quoted_ratio - 1.0).abs() < 1e-9);
    }

    #[test]
    fn inside_quote_trade_yields_effective_below_quoted() {
        // Trade INSIDE the spread (price improvement) → effective < quoted.
        let observations = vec![obs(100.02, 100.00, 100.00, 0.10, Direction::Buy)];
        let r = analyze(&observations).unwrap();
        assert!(r.effective_to_quoted_ratio < 1.0);
    }
}
