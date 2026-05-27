//! Bracket / OCO order modeling.
//!
//! A bracket order is an entry + a stop-loss + a take-profit, where the
//! stop and target are wired OCO ("one cancels the other") — fill one,
//! cancel the other. Real brokers handle this natively; the paper-trade
//! sim doesn't, so we model the resolution here.
//!
//! Given the entry fill price + the post-entry tick stream (high/low/
//! close per bar), determine which leg fires first and at what price.
//! Output the exit + which leg won.
//!
//! Pure compute. Used by paper trading + by the new-trade form's
//! "what if" simulator.

use crate::models::TradeSide;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BracketOrder {
    pub side: TradeSide,
    pub entry: Decimal,
    pub stop: Decimal,
    pub target: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolvedLeg {
    /// Stop hit first.
    Stopped,
    /// Take-profit hit first.
    TargetHit,
    /// Neither leg triggered within the bar window.
    StillOpen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedBracket {
    pub leg: ResolvedLeg,
    /// Exit price (= stop or target). None when StillOpen.
    pub exit_price: Option<Decimal>,
    /// Index into the input bar slice where the leg fired. None when
    /// StillOpen.
    pub bar_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriceBar {
    pub high: Decimal,
    pub low: Decimal,
}

/// Walk bars in order. For a LONG, the stop fires when low <= stop and
/// target fires when high >= target. For a SHORT, mirror.
///
/// Within a single bar, if BOTH levels are touched, the convention is
/// "stop wins" (pessimistic — assumes the stop fills before the target
/// from the same intra-bar movement). Real brokers behave similarly when
/// they can't disambiguate which tick came first.
pub fn resolve(order: &BracketOrder, bars: &[PriceBar]) -> ResolvedBracket {
    for (i, b) in bars.iter().enumerate() {
        let (stop_hit, target_hit) = match order.side {
            TradeSide::Long => (b.low <= order.stop, b.high >= order.target),
            TradeSide::Short => (b.high >= order.stop, b.low <= order.target),
        };
        match (stop_hit, target_hit) {
            (true, _) => {
                return ResolvedBracket {
                    leg: ResolvedLeg::Stopped,
                    exit_price: Some(order.stop),
                    bar_index: Some(i),
                }
            }
            (false, true) => {
                return ResolvedBracket {
                    leg: ResolvedLeg::TargetHit,
                    exit_price: Some(order.target),
                    bar_index: Some(i),
                }
            }
            (false, false) => continue,
        }
    }
    ResolvedBracket {
        leg: ResolvedLeg::StillOpen,
        exit_price: None,
        bar_index: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }
    fn bar(low: &str, high: &str) -> PriceBar {
        PriceBar {
            low: d(low),
            high: d(high),
        }
    }

    fn long_bracket() -> BracketOrder {
        BracketOrder {
            side: TradeSide::Long,
            entry: d("100"),
            stop: d("99"),
            target: d("102"),
        }
    }

    #[test]
    fn long_target_hit_first() {
        let bars = vec![
            bar("99.50", "101.00"),  // touched neither
            bar("100.50", "102.50"), // target hit
        ];
        let r = resolve(&long_bracket(), &bars);
        assert_eq!(r.leg, ResolvedLeg::TargetHit);
        assert_eq!(r.exit_price, Some(d("102")));
        assert_eq!(r.bar_index, Some(1));
    }

    #[test]
    fn long_stop_hit_first() {
        let bars = vec![
            bar("98.50", "100.50"), // stop hit
        ];
        let r = resolve(&long_bracket(), &bars);
        assert_eq!(r.leg, ResolvedLeg::Stopped);
        assert_eq!(r.exit_price, Some(d("99")));
        assert_eq!(r.bar_index, Some(0));
    }

    #[test]
    fn pessimistic_when_both_levels_touched_same_bar() {
        // Bar low = 98 (stop touched), high = 103 (target touched).
        let bars = vec![bar("98", "103")];
        let r = resolve(&long_bracket(), &bars);
        assert_eq!(
            r.leg,
            ResolvedLeg::Stopped,
            "ambiguous intra-bar — must pessimistically assume stop"
        );
    }

    #[test]
    fn long_still_open_when_neither_touched() {
        let bars = vec![bar("99.50", "101.50"), bar("99.10", "101.90")];
        let r = resolve(&long_bracket(), &bars);
        assert_eq!(r.leg, ResolvedLeg::StillOpen);
        assert_eq!(r.exit_price, None);
        assert_eq!(r.bar_index, None);
    }

    #[test]
    fn empty_bars_returns_still_open() {
        let r = resolve(&long_bracket(), &[]);
        assert_eq!(r.leg, ResolvedLeg::StillOpen);
    }

    #[test]
    fn short_uses_inverted_geometry() {
        // Short entry 100, stop 101, target 98.
        let short = BracketOrder {
            side: TradeSide::Short,
            entry: d("100"),
            stop: d("101"),
            target: d("98"),
        };
        // First bar: high 102 → stop hit (101 reached).
        let r = resolve(&short, &[bar("99", "102")]);
        assert_eq!(r.leg, ResolvedLeg::Stopped);
        // Different bars: only low 97 → target hit.
        let r2 = resolve(&short, &[bar("97", "100.5")]);
        assert_eq!(r2.leg, ResolvedLeg::TargetHit);
        assert_eq!(r2.exit_price, Some(d("98")));
    }

    #[test]
    fn long_stop_exactly_at_low_triggers_inclusive() {
        let bars = vec![bar("99.00", "101.00")]; // low == stop
        let r = resolve(&long_bracket(), &bars);
        assert_eq!(
            r.leg,
            ResolvedLeg::Stopped,
            "stop must trigger at equality (touch)"
        );
    }

    #[test]
    fn long_target_exactly_at_high_triggers_inclusive() {
        let bars = vec![bar("99.50", "102.00")]; // high == target
        let r = resolve(&long_bracket(), &bars);
        assert_eq!(r.leg, ResolvedLeg::TargetHit);
    }

    #[test]
    fn first_bar_with_resolution_short_circuits() {
        // 5 bars; bar #2 stops out. Bars 3-4 should never be examined,
        // but more importantly we should return at index 2, not 4.
        let bars = vec![
            bar("99.50", "101.50"),
            bar("99.20", "101.80"),
            bar("98.50", "100.50"), // stop hit
            bar("99.50", "102.50"), // target would hit here if not stopped
            bar("99.10", "101.50"),
        ];
        let r = resolve(&long_bracket(), &bars);
        assert_eq!(r.bar_index, Some(2));
        assert_eq!(r.leg, ResolvedLeg::Stopped);
    }
}
