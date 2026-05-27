//! Footprint chart — per-bar bid/ask volume + delta aggregation.
//!
//! Order-flow visualization staple in Sierra Chart, Bookmap, and Jigsaw.
//! For each price-time bar, the trade tape is bucketed into price levels;
//! within each level we accumulate the volume traded at the bid vs. at the
//! ask. The *delta* (ask − bid) reveals which side was more aggressive at
//! that level. Strong positive delta at lows = absorption; negative delta
//! at highs = rejection.
//!
//! Caller supplies an already-classified tick stream (use
//! `crate::order_flow::classify` first) plus a bar partition. The bar
//! partition is just `tick_idx → bar_id`; for time-bar charts the caller
//! resolves the bar id from the tick timestamp, for range/Renko bars from
//! the price walk, etc. This module is agnostic to the bar partitioning
//! algorithm.
//!
//! Pure compute.

use crate::order_flow::{ClassifiedTick, Side};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One tick paired with the bar it belongs to and the trade price.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BarTick {
    pub bar_id: u32,
    pub price: f64,
    pub classified: ClassifiedTick,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FootprintCell {
    pub price: f64,
    pub bid_volume: f64,
    pub ask_volume: f64,
    /// `ask_volume - bid_volume`. Positive = aggressive buyers won at this level.
    pub delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FootprintBar {
    pub bar_id: u32,
    /// Bid/ask cells indexed by price level (ascending price).
    pub cells: Vec<FootprintCell>,
    /// Total volume at all price levels in this bar.
    pub total_volume: f64,
    /// `sum(ask_volume) - sum(bid_volume)` across the bar.
    pub total_delta: f64,
    /// Point of Control = price level with the highest combined volume.
    pub poc_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FootprintReport {
    pub bars: Vec<FootprintBar>,
    /// `tick_size` used to quantize prices into levels.
    pub tick_size: f64,
}

/// Build the footprint report. `tick_size` quantizes prices into discrete
/// levels — e.g. 0.01 for stocks, 0.25 for ES futures.
pub fn build(ticks: &[BarTick], tick_size: f64) -> FootprintReport {
    if tick_size <= 0.0 || ticks.is_empty() {
        return FootprintReport { bars: vec![], tick_size };
    }
    // Bin into bar → (price_bin → (bid_vol, ask_vol, total_vol)).
    let mut by_bar: BTreeMap<u32, BTreeMap<u64, (f64, f64)>> = BTreeMap::new();
    let quantize = |p: f64| -> u64 { (p / tick_size).round() as u64 };
    for t in ticks {
        let bar = by_bar.entry(t.bar_id).or_default();
        let cell = bar.entry(quantize(t.price)).or_insert((0.0, 0.0));
        match t.classified.side {
            Side::Buy        => cell.1 += t.classified.volume,
            Side::Sell       => cell.0 += t.classified.volume,
            // Uncertain ticks split 50/50 between bid and ask so the total
            // volume stays correct but the delta doesn't lie about direction.
            Side::Uncertain  => {
                cell.0 += t.classified.volume * 0.5;
                cell.1 += t.classified.volume * 0.5;
            }
        }
    }
    let mut bars = Vec::with_capacity(by_bar.len());
    for (bar_id, levels) in by_bar {
        let mut cells = Vec::with_capacity(levels.len());
        let (mut total_vol, mut total_delta) = (0.0, 0.0);
        let (mut poc_price, mut poc_volume) = (0.0, f64::NEG_INFINITY);
        for (q_price, (bid, ask)) in levels {
            let price = q_price as f64 * tick_size;
            let vol = bid + ask;
            let delta = ask - bid;
            total_vol += vol;
            total_delta += delta;
            if vol > poc_volume { poc_volume = vol; poc_price = price; }
            cells.push(FootprintCell { price, bid_volume: bid, ask_volume: ask, delta });
        }
        bars.push(FootprintBar { bar_id, cells, total_volume: total_vol, total_delta, poc_price });
    }
    FootprintReport { bars, tick_size }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(bar_id: u32, price: f64, volume: f64, side: Side) -> BarTick {
        BarTick { bar_id, price, classified: ClassifiedTick { volume, side } }
    }

    #[test]
    fn empty_input_returns_empty_report() {
        let r = build(&[], 0.01);
        assert!(r.bars.is_empty());
        assert_eq!(r.tick_size, 0.01);
    }

    #[test]
    fn invalid_tick_size_returns_empty() {
        let ticks = vec![tick(0, 100.0, 1.0, Side::Buy)];
        let r = build(&ticks, 0.0);
        assert!(r.bars.is_empty(), "non-positive tick size is invalid");
    }

    #[test]
    fn buy_ticks_at_one_level_show_positive_delta() {
        // 3 buy ticks @ 100.0, all aggressive — total delta should be +3.
        let ticks = vec![
            tick(0, 100.0, 1.0, Side::Buy),
            tick(0, 100.0, 1.0, Side::Buy),
            tick(0, 100.0, 1.0, Side::Buy),
        ];
        let r = build(&ticks, 0.01);
        assert_eq!(r.bars.len(), 1);
        assert_eq!(r.bars[0].total_delta, 3.0);
        assert_eq!(r.bars[0].total_volume, 3.0);
        assert_eq!(r.bars[0].cells.len(), 1);
        assert_eq!(r.bars[0].cells[0].delta, 3.0);
    }

    #[test]
    fn poc_is_the_highest_volume_price_level() {
        // 1 tick @ 99.99, 5 ticks @ 100.00, 2 ticks @ 100.01 → POC = 100.00.
        let mut ticks = vec![tick(0, 99.99, 1.0, Side::Buy)];
        for _ in 0..5 { ticks.push(tick(0, 100.00, 1.0, Side::Sell)); }
        for _ in 0..2 { ticks.push(tick(0, 100.01, 1.0, Side::Buy));  }
        let r = build(&ticks, 0.01);
        assert!((r.bars[0].poc_price - 100.00).abs() < 1e-9);
    }

    #[test]
    fn uncertain_ticks_split_evenly_between_sides() {
        // 1 uncertain tick of size 10 should add 5 to each side → delta 0.
        let ticks = vec![tick(0, 100.0, 10.0, Side::Uncertain)];
        let r = build(&ticks, 0.01);
        assert_eq!(r.bars[0].total_volume, 10.0);
        assert_eq!(r.bars[0].total_delta, 0.0);
        assert_eq!(r.bars[0].cells[0].bid_volume, 5.0);
        assert_eq!(r.bars[0].cells[0].ask_volume, 5.0);
    }

    #[test]
    fn ticks_in_different_bars_stay_separate() {
        let ticks = vec![
            tick(0, 100.0, 1.0, Side::Buy),
            tick(1, 101.0, 2.0, Side::Sell),
        ];
        let r = build(&ticks, 0.01);
        assert_eq!(r.bars.len(), 2);
        assert_eq!(r.bars[0].total_delta, 1.0);
        assert_eq!(r.bars[1].total_delta, -2.0);
    }
}
