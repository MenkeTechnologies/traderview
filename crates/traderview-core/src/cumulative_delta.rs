//! Cumulative delta — running net aggressor volume.
//!
//! Per-bar delta = ask_volume − bid_volume (computed from a classified
//! tick stream). Cumulative delta sums that across the session and is
//! one of the most-watched order-flow primitives: when CD diverges from
//! price (price up, CD flat or down), buyers have lost conviction and
//! the move is fragile. CD breaking a key low alongside price breaking
//! a swing low is double confirmation of weakness.
//!
//! Caller supplies a series of classified ticks (use `crate::order_flow`
//! to classify raw prints first). We emit:
//!   - per-tick cumulative delta value
//!   - the running high-water/low-water marks of CD
//!   - the divergence-vs-price flag: did price make a new high while CD
//!     failed to, or vice versa?
//!
//! Pure compute. Distinct from `footprint::FootprintReport` (per-bar
//! cells) — this module emits a single time-series.

use crate::order_flow::{ClassifiedTick, Side};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TickWithPrice {
    pub price: f64,
    pub classified: ClassifiedTick,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct CdPoint {
    pub price: f64,
    /// Signed running net: + = net aggressive buying, − = net selling.
    pub cumulative_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CdReport {
    pub points: Vec<CdPoint>,
    pub max_cd: f64,
    pub min_cd: f64,
    pub final_cd: f64,
    /// True when price reached a new HIGH but cumulative delta did NOT —
    /// classic bearish divergence (buyers absent at the high).
    pub bearish_divergence: bool,
    /// Mirror: price made new LOW but CD did not — bullish divergence.
    pub bullish_divergence: bool,
}

pub fn compute(ticks: &[TickWithPrice]) -> CdReport {
    if ticks.is_empty() {
        return CdReport::default();
    }
    let mut points: Vec<CdPoint> = Vec::with_capacity(ticks.len());
    let mut cd = 0.0_f64;
    for t in ticks {
        match t.classified.side {
            Side::Buy => cd += t.classified.volume,
            Side::Sell => cd -= t.classified.volume,
            // Uncertain ticks contribute 0 net delta — they're noise
            // unless caller pre-classified differently.
            Side::Uncertain => {}
        }
        points.push(CdPoint {
            price: t.price,
            cumulative_delta: cd,
        });
    }
    let max_cd = points
        .iter()
        .map(|p| p.cumulative_delta)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_cd = points
        .iter()
        .map(|p| p.cumulative_delta)
        .fold(f64::INFINITY, f64::min);
    let final_cd = points.last().map(|p| p.cumulative_delta).unwrap_or(0.0);
    // Divergence: did the LAST point's price make a new extreme while CD didn't?
    let last = *points.last().expect("non-empty");
    let prior_max_price = points[..points.len() - 1]
        .iter()
        .map(|p| p.price)
        .fold(f64::NEG_INFINITY, f64::max);
    let prior_min_price = points[..points.len() - 1]
        .iter()
        .map(|p| p.price)
        .fold(f64::INFINITY, f64::min);
    let bearish_divergence = last.price > prior_max_price && last.cumulative_delta < max_cd;
    let bullish_divergence = last.price < prior_min_price && last.cumulative_delta > min_cd;
    CdReport {
        points,
        max_cd,
        min_cd,
        final_cd,
        bearish_divergence,
        bullish_divergence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(price: f64, volume: f64, side: Side) -> TickWithPrice {
        TickWithPrice {
            price,
            classified: ClassifiedTick { volume, side },
        }
    }

    #[test]
    fn empty_input_returns_default() {
        let r = compute(&[]);
        assert!(r.points.is_empty());
        assert_eq!(r.final_cd, 0.0);
    }

    #[test]
    fn all_buys_produces_monotonic_positive_cd() {
        let r = compute(&[
            tick(100.0, 1.0, Side::Buy),
            tick(101.0, 2.0, Side::Buy),
            tick(102.0, 3.0, Side::Buy),
        ]);
        assert_eq!(r.points.len(), 3);
        assert_eq!(r.points[2].cumulative_delta, 6.0);
        assert_eq!(r.final_cd, 6.0);
        assert_eq!(r.max_cd, 6.0);
        assert_eq!(r.min_cd, 1.0);
    }

    #[test]
    fn all_sells_produces_monotonic_negative_cd() {
        let r = compute(&[tick(100.0, 1.0, Side::Sell), tick(99.0, 2.0, Side::Sell)]);
        assert_eq!(r.final_cd, -3.0);
        assert_eq!(r.min_cd, -3.0);
    }

    #[test]
    fn uncertain_ticks_dont_move_cd() {
        let r = compute(&[
            tick(100.0, 100.0, Side::Uncertain),
            tick(101.0, 5.0, Side::Buy),
        ]);
        assert_eq!(r.final_cd, 5.0, "uncertain shouldn't add to delta");
    }

    #[test]
    fn bearish_divergence_when_price_new_high_but_cd_not() {
        // Setup: price goes up 100 → 102 → 104, but during the rally
        // there's heavy selling so CD goes 5 → 4 → 3.
        let r = compute(&[
            tick(100.0, 5.0, Side::Buy),  // CD = 5
            tick(102.0, 1.0, Side::Sell), // CD = 4 (price up but CD down)
            tick(104.0, 1.0, Side::Sell), // price NEW HIGH (104), CD = 3 < max_cd 5 = divergence
        ]);
        assert!(
            r.bearish_divergence,
            "price 104 > prior max 102; CD 3 < max_cd 5 → bearish div"
        );
        assert!(!r.bullish_divergence);
    }

    #[test]
    fn bullish_divergence_when_price_new_low_but_cd_not() {
        let r = compute(&[
            tick(100.0, 5.0, Side::Sell), // CD = -5
            tick(98.0, 1.0, Side::Buy),   // CD = -4
            tick(96.0, 1.0, Side::Buy),   // new low 96, CD -3 > min -5 → bullish div
        ]);
        assert!(r.bullish_divergence);
    }

    #[test]
    fn no_divergence_when_price_makes_no_new_extreme() {
        // Price never goes outside of its prior range → no divergence.
        let r = compute(&[
            tick(100.0, 5.0, Side::Buy),
            tick(101.0, 5.0, Side::Buy),
            tick(100.5, 1.0, Side::Sell),
        ]);
        assert!(!r.bearish_divergence);
        assert!(!r.bullish_divergence);
    }
}
