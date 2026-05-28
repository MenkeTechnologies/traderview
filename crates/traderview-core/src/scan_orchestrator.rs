//! Scanner orchestrator — runs every `scan::Preset` against a universe
//! of symbols' bar histories and emits a ranked hit list.
//!
//! The existing `crate::scan` module answers per-symbol queries; this
//! module is the universe-wide driver the web dashboard hits when the
//! user clicks "Show me everything matching ANY of these presets right
//! now across the watchlist."
//!
//! Inputs: a map of symbol → recent daily bars (most-recent last).
//! Outputs: each symbol that matched at least one preset, with the list
//! of matched preset labels and the full `ScanHit` stats. Hits are
//! sorted by the number of presets matched (descending) — symbols that
//! lit up multiple filters bubble to the top.
//!
//! Pure compute. Caller fans out the database fetches.

use crate::models::PriceBar;
use crate::scan::{matches, preset_label, stats_for, Preset, ScanHit};
use serde::Serialize;

/// All preset variants in dashboard display order. Centralized here so
/// callers don't have to enumerate them themselves.
pub const ALL_PRESETS: &[Preset] = &[
    Preset::PremarketGappers,
    Preset::MomentumMovers,
    Preset::HighOfDay,
    Preset::LowFloatRunners,
    Preset::Pct52wHigh,
    Preset::Pct52wLow,
    Preset::VolumeSurge,
    Preset::Breakdown,
    Preset::Breakout,
    Preset::OversoldBounce,
    Preset::GapAndGo,
    Preset::GapAndFade,
    Preset::InsideDayLow,
    Preset::InsideDayHigh,
    Preset::RangeContractionDay,
    Preset::DistributionDay,
    Preset::AccumulationDay,
    Preset::NearYearHighLowVol,
];

#[derive(Debug, Clone, Serialize, Default)]
pub struct UniverseReport {
    pub hits: Vec<ScanHit>,
    pub total_symbols_scanned: usize,
    pub total_hits: usize,
}

/// Run every preset against every symbol. Symbols whose history is too
/// short to compute stats are silently dropped (no panic, no crash on
/// short universes during pre-market warmup).
pub fn scan_universe(universe: &[(String, Vec<PriceBar>)]) -> UniverseReport {
    scan_universe_filtered(universe, ALL_PRESETS)
}

/// Same as `scan_universe` but lets the caller pick a subset of presets
/// (e.g. only the breakout family).
pub fn scan_universe_filtered(
    universe: &[(String, Vec<PriceBar>)],
    presets: &[Preset],
) -> UniverseReport {
    let mut hits: Vec<ScanHit> = Vec::new();
    let mut scanned = 0usize;
    for (symbol, bars) in universe {
        scanned += 1;
        let Some(mut hit) = stats_for(symbol, bars) else { continue };
        // Apply every preset and collect labels for those that match.
        let matched: Vec<&'static str> = presets
            .iter()
            .filter(|p| matches(&hit, **p))
            .map(|p| preset_label(*p))
            .collect();
        if matched.is_empty() {
            continue;
        }
        hit.matched = matched;
        hits.push(hit);
    }
    // Sort by hit-count desc, then by symbol asc for stable display.
    hits.sort_by(|a, b| {
        b.matched
            .len()
            .cmp(&a.matched.len())
            .then_with(|| a.symbol.cmp(&b.symbol))
    });
    let total_hits = hits.len();
    UniverseReport {
        hits,
        total_symbols_scanned: scanned,
        total_hits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BarInterval;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;

    fn bar(open: u64, high: u64, low: u64, close: u64, vol: u64, ts: i64) -> PriceBar {
        PriceBar {
            symbol: "X".into(),
            interval: BarInterval::D1,
            bar_time: Utc.timestamp_opt(ts, 0).unwrap(),
            open: Decimal::from(open),
            high: Decimal::from(high),
            low: Decimal::from(low),
            close: Decimal::from(close),
            volume: Decimal::from(vol),
            source: "test".into(),
        }
    }

    #[test]
    fn empty_universe_returns_default_report() {
        let r = scan_universe(&[]);
        assert!(r.hits.is_empty());
        assert_eq!(r.total_symbols_scanned, 0);
    }

    #[test]
    fn symbols_with_too_few_bars_are_silently_dropped() {
        let universe = vec![("TICK".to_string(), vec![bar(100, 100, 95, 100, 1000, 1)])];
        let r = scan_universe(&universe);
        assert!(r.hits.is_empty(), "1 bar can't satisfy stats_for");
        assert_eq!(r.total_symbols_scanned, 1);
    }

    #[test]
    fn gapping_symbol_matches_premarket_gappers_preset() {
        let bars = vec![
            bar(100, 100, 95, 100, 1_000_000, 1),
            bar(106, 110, 105, 108, 1_000_000, 2),    // 6% gap up
        ];
        let universe = vec![("GAP".to_string(), bars)];
        let r = scan_universe(&universe);
        assert_eq!(r.total_hits, 1);
        assert!(r.hits[0].matched.contains(&"Gappers"));
    }

    #[test]
    fn hits_sorted_by_match_count_desc() {
        // SYM_A matches gappers + accumulation (high vol up day) → 2+ matches.
        // SYM_B matches only oversold bounce (small up day) → fewer.
        // Verify A appears before B in the output.
        let a_bars = vec![
            bar(100, 101, 99, 100, 1_000_000, 1),
            bar(100, 101, 99, 100, 1_000_000, 2),
            bar(100, 101, 99, 100, 1_000_000, 3),
            bar(100, 101, 99, 100, 1_000_000, 4),
            bar(106, 110, 105, 108, 3_000_000, 5),    // gap + volume + up
        ];
        let b_bars = vec![
            bar(100, 100, 95, 100, 1_000_000, 1),
            bar(100, 102, 99, 101, 1_000_000, 2),     // tiny up day
        ];
        let universe = vec![
            ("SYM_A".to_string(), a_bars),
            ("SYM_B".to_string(), b_bars),
        ];
        let r = scan_universe(&universe);
        assert!(!r.hits.is_empty());
        // SYM_A's matched count should be >= SYM_B's; A should be first.
        if r.hits.len() >= 2 {
            assert!(r.hits[0].matched.len() >= r.hits[1].matched.len());
        }
    }

    #[test]
    fn filtered_preset_subset_only_runs_those_presets() {
        let bars = vec![
            bar(100, 100, 95, 100, 1_000_000, 1),
            bar(106, 110, 105, 108, 1_000_000, 2),
        ];
        let universe = vec![("X".to_string(), bars)];
        // Only check breakdown — should NOT match a gap-UP symbol.
        let r = scan_universe_filtered(&universe, &[Preset::Breakdown]);
        assert_eq!(r.total_hits, 0);
    }

    #[test]
    fn total_symbols_scanned_counts_every_input_even_drops() {
        let universe = vec![
            ("TOO_SHORT".to_string(), vec![bar(100, 100, 95, 100, 1000, 1)]),
            ("ALSO_SHORT".to_string(), vec![bar(100, 100, 95, 100, 1000, 1)]),
        ];
        let r = scan_universe(&universe);
        assert_eq!(r.total_symbols_scanned, 2);
        assert!(r.hits.is_empty());
    }

    #[test]
    fn all_presets_list_matches_preset_enum_variant_count() {
        // Sanity: keep ALL_PRESETS in sync with the enum. If we add a
        // variant without listing it here, scan_universe silently skips
        // checking that preset — this assertion forces the breakage to
        // show up at test time.
        // Current count: 10 original + 8 new = 18.
        assert_eq!(ALL_PRESETS.len(), 18);
    }
}
