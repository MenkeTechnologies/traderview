//! Order-flow imbalance estimator (tick-based).
//!
//! For each trade tick, classify as aggressive-buy or aggressive-sell
//! using the tick rule:
//!   - Trade at ask (or above prior trade) → aggressive BUY
//!   - Trade at bid (or below prior trade) → aggressive SELL
//!   - Trade between bid/ask (no prior comparison) → uncertain
//!
//! Aggregate over a window to expose net flow imbalance — institutional
//! traders' #1 intraday signal. Positive imbalance = aggressive buying;
//! large negative = institutional dumping.
//!
//! Pure compute. Caller supplies the tick stream + bid/ask quotes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Tick {
    pub price: f64,
    pub volume: f64,
    pub bid: f64,
    pub ask: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side { Buy, Sell, Uncertain }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClassifiedTick {
    pub volume: f64,
    pub side: Side,
}

pub fn classify(ticks: &[Tick]) -> Vec<ClassifiedTick> {
    let mut out = Vec::with_capacity(ticks.len());
    let mut prev_price: Option<f64> = None;
    for t in ticks {
        let side = if t.price >= t.ask {
            Side::Buy
        } else if t.price <= t.bid {
            Side::Sell
        } else if let Some(prev) = prev_price {
            // Inside spread — use tick rule against previous trade.
            if t.price > prev      { Side::Buy }
            else if t.price < prev { Side::Sell }
            else                    { Side::Uncertain }
        } else {
            Side::Uncertain
        };
        out.push(ClassifiedTick { volume: t.volume, side });
        prev_price = Some(t.price);
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImbalanceReport {
    pub buy_volume: f64,
    pub sell_volume: f64,
    pub uncertain_volume: f64,
    pub net_volume: f64,
    /// (buy - sell) / (buy + sell). Range -1..=+1.
    pub imbalance_ratio: f64,
}

pub fn aggregate(classified: &[ClassifiedTick]) -> ImbalanceReport {
    let mut report = ImbalanceReport::default();
    for c in classified {
        match c.side {
            Side::Buy       => report.buy_volume += c.volume,
            Side::Sell      => report.sell_volume += c.volume,
            Side::Uncertain => report.uncertain_volume += c.volume,
        }
    }
    report.net_volume = report.buy_volume - report.sell_volume;
    let directional = report.buy_volume + report.sell_volume;
    report.imbalance_ratio = if directional > 0.0 {
        report.net_volume / directional
    } else { 0.0 };
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(price: f64, vol: f64, bid: f64, ask: f64) -> Tick {
        Tick { price, volume: vol, bid, ask }
    }

    #[test]
    fn empty_returns_empty_classified() {
        assert!(classify(&[]).is_empty());
    }

    #[test]
    fn trade_at_ask_classified_buy() {
        let out = classify(&[t(100.01, 100.0, 100.00, 100.01)]);
        assert_eq!(out[0].side, Side::Buy);
    }

    #[test]
    fn trade_above_ask_classified_buy() {
        let out = classify(&[t(100.05, 100.0, 100.00, 100.01)]);
        assert_eq!(out[0].side, Side::Buy);
    }

    #[test]
    fn trade_at_bid_classified_sell() {
        let out = classify(&[t(100.00, 100.0, 100.00, 100.01)]);
        assert_eq!(out[0].side, Side::Sell);
    }

    #[test]
    fn trade_below_bid_classified_sell() {
        let out = classify(&[t(99.95, 100.0, 100.00, 100.01)]);
        assert_eq!(out[0].side, Side::Sell);
    }

    #[test]
    fn first_inside_spread_trade_uncertain() {
        // No prior trade → tick rule has nothing to compare to.
        let out = classify(&[t(100.005, 100.0, 100.00, 100.01)]);
        assert_eq!(out[0].side, Side::Uncertain);
    }

    #[test]
    fn inside_spread_uses_uptick_rule_for_buy() {
        let ticks = vec![
            t(100.00, 100.0, 100.00, 100.01),    // sell (at bid)
            t(100.005, 100.0, 100.00, 100.01),    // uptick from .00 → buy
        ];
        let out = classify(&ticks);
        assert_eq!(out[0].side, Side::Sell);
        assert_eq!(out[1].side, Side::Buy);
    }

    #[test]
    fn inside_spread_uses_downtick_rule_for_sell() {
        let ticks = vec![
            t(100.01, 100.0, 100.00, 100.01),    // buy (at ask)
            t(100.005, 100.0, 100.00, 100.01),    // downtick → sell
        ];
        let out = classify(&ticks);
        assert_eq!(out[0].side, Side::Buy);
        assert_eq!(out[1].side, Side::Sell);
    }

    #[test]
    fn aggregate_empty_zero_report() {
        let r = aggregate(&[]);
        assert_eq!(r.buy_volume, 0.0);
        assert_eq!(r.imbalance_ratio, 0.0);
    }

    #[test]
    fn aggregate_balanced_zero_imbalance() {
        let cl = vec![
            ClassifiedTick { volume: 100.0, side: Side::Buy },
            ClassifiedTick { volume: 100.0, side: Side::Sell },
        ];
        let r = aggregate(&cl);
        assert_eq!(r.net_volume, 0.0);
        assert_eq!(r.imbalance_ratio, 0.0);
    }

    #[test]
    fn aggregate_pure_buy_yields_imbalance_one() {
        let cl = vec![ClassifiedTick { volume: 500.0, side: Side::Buy }];
        let r = aggregate(&cl);
        assert_eq!(r.imbalance_ratio, 1.0);
        assert_eq!(r.buy_volume, 500.0);
    }

    #[test]
    fn aggregate_uncertain_excluded_from_imbalance_ratio() {
        let cl = vec![
            ClassifiedTick { volume: 100.0, side: Side::Buy },
            ClassifiedTick { volume: 999.0, side: Side::Uncertain },
        ];
        let r = aggregate(&cl);
        // Uncertain doesn't go in numerator or denominator.
        assert_eq!(r.imbalance_ratio, 1.0);
        assert_eq!(r.uncertain_volume, 999.0);
    }
}
