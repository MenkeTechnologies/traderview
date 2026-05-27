//! Trading-halt risk classifier.
//!
//! Per-symbol historical-halt frequency feeds a risk score the
//! dashboard uses to show a "high halt risk" badge on the trade
//! entry view. Halts disproportionately affect:
//!   - Low-float micro-caps (LULD pauses)
//!   - Stocks with news-pending ("news halt")
//!   - Penny stocks (SEC-suspended for fraud risk)
//!   - Pre-IPO / post-IPO no-quote windows
//!
//! For each symbol, count halt events over a lookback window. Convert
//! to halts-per-day. Bucket into Low / Medium / High / Extreme tiers.
//!
//! Pure compute. Caller supplies historical halts; engine emits scores.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaltEvent {
    pub symbol: String,
    pub when: NaiveDate,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HaltRiskTier {
    Low,     // < 0.01 halts/trading-day (almost never)
    Medium,  // 0.01-0.05 (rare but notable)
    High,    // 0.05-0.20 (regularly)
    Extreme, // > 0.20 (multiple times per month)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolHaltRisk {
    pub symbol: String,
    pub halt_count: usize,
    pub lookback_days: i64,
    pub halts_per_day: f64,
    pub tier: HaltRiskTier,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HaltRiskReport {
    pub by_symbol: Vec<SymbolHaltRisk>,
    /// Symbols in `Extreme` or `High` tier — the dashboard's red list.
    pub watch_list: Vec<String>,
}

pub fn analyze(events: &[HaltEvent], now: NaiveDate, lookback: Duration) -> HaltRiskReport {
    let mut report = HaltRiskReport::default();
    if events.is_empty() {
        return report;
    }
    let cutoff = now - lookback;
    let lookback_days = lookback.num_days().max(1);
    let mut by_symbol: std::collections::BTreeMap<String, usize> = Default::default();
    for e in events {
        if e.when >= cutoff && e.when <= now {
            *by_symbol.entry(e.symbol.clone()).or_default() += 1;
        }
    }
    for (sym, count) in by_symbol {
        let per_day = count as f64 / lookback_days as f64;
        let tier = if per_day > 0.20 {
            HaltRiskTier::Extreme
        } else if per_day > 0.05 {
            HaltRiskTier::High
        } else if per_day > 0.01 {
            HaltRiskTier::Medium
        } else {
            HaltRiskTier::Low
        };
        if matches!(tier, HaltRiskTier::Extreme | HaltRiskTier::High) {
            report.watch_list.push(sym.clone());
        }
        report.by_symbol.push(SymbolHaltRisk {
            symbol: sym,
            halt_count: count,
            lookback_days,
            halts_per_day: per_day,
            tier,
        });
    }
    report.by_symbol.sort_by(|a, b| {
        b.halts_per_day
            .partial_cmp(&a.halts_per_day)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    report.watch_list.sort();
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }
    fn ev(sym: &str, when: NaiveDate, reason: &str) -> HaltEvent {
        HaltEvent {
            symbol: sym.into(),
            when,
            reason: reason.into(),
        }
    }

    #[test]
    fn empty_returns_empty_report() {
        let r = analyze(&[], day(2026, 5, 27), Duration::days(90));
        assert!(r.by_symbol.is_empty());
        assert!(r.watch_list.is_empty());
    }

    #[test]
    fn single_halt_in_long_window_yields_low_tier() {
        // 1 halt in 1000 days = 0.001/day → Low.
        let events = vec![ev("AAPL", day(2026, 3, 1), "LULD")];
        let r = analyze(&events, day(2026, 5, 27), Duration::days(1000));
        assert_eq!(r.by_symbol[0].halt_count, 1);
        assert_eq!(r.by_symbol[0].tier, HaltRiskTier::Low);
        assert!(r.watch_list.is_empty());
    }

    #[test]
    fn single_halt_in_90_days_yields_medium_tier() {
        // 1/90 = 0.011/day → just over 0.01 boundary → Medium.
        let events = vec![ev("AAPL", day(2026, 3, 1), "LULD")];
        let r = analyze(&events, day(2026, 5, 27), Duration::days(90));
        assert_eq!(r.by_symbol[0].tier, HaltRiskTier::Medium);
    }

    #[test]
    fn many_halts_in_90_days_yields_high_or_extreme() {
        // 20 halts in 90 days = 0.222/day → Extreme.
        let events: Vec<_> = (1..=20)
            .map(|i| ev("GME", day(2026, 3, i), "LULD"))
            .collect();
        let r = analyze(&events, day(2026, 5, 27), Duration::days(90));
        assert_eq!(r.by_symbol[0].halt_count, 20);
        assert_eq!(r.by_symbol[0].tier, HaltRiskTier::Extreme);
        assert!(r.watch_list.contains(&"GME".to_string()));
    }

    #[test]
    fn halts_outside_window_excluded() {
        let events = vec![
            ev("OLD", day(2025, 1, 1), "old"), // 1+ years ago
            ev("NEW", day(2026, 5, 1), "recent"),
        ];
        let r = analyze(&events, day(2026, 5, 27), Duration::days(90));
        assert_eq!(r.by_symbol.len(), 1);
        assert_eq!(r.by_symbol[0].symbol, "NEW");
    }

    #[test]
    fn symbols_sorted_by_halts_per_day_descending() {
        let events = vec![
            ev("A", day(2026, 5, 1), "x"),
            ev("B", day(2026, 5, 1), "x"),
            ev("B", day(2026, 5, 5), "x"),
            ev("B", day(2026, 5, 10), "x"),
        ];
        let r = analyze(&events, day(2026, 5, 27), Duration::days(90));
        assert_eq!(r.by_symbol[0].symbol, "B");
        assert_eq!(r.by_symbol[1].symbol, "A");
    }

    #[test]
    fn watch_list_only_high_and_extreme() {
        // GME: 10 halts/90d = 0.111 → High. AAPL: 1/90 = 0.011 → Medium.
        let mut events: Vec<_> = (1..=10).map(|i| ev("GME", day(2026, 3, i), "x")).collect();
        events.push(ev("AAPL", day(2026, 3, 1), "x"));
        let r = analyze(&events, day(2026, 5, 27), Duration::days(90));
        assert_eq!(r.watch_list, vec!["GME"]);
    }

    #[test]
    fn tier_threshold_boundaries() {
        // Helper: synthesize N events in 100 days, check rate threshold.
        let date = day(2026, 5, 27);
        let lookback = Duration::days(100);
        let case = |n_events: usize| -> HaltRiskTier {
            let events: Vec<_> = (1..=n_events)
                .map(|i| ev("X", date - Duration::days(i as i64), "x"))
                .collect();
            analyze(&events, date, lookback).by_symbol[0].tier
        };
        assert_eq!(case(1), HaltRiskTier::Low);
        assert_eq!(case(5), HaltRiskTier::Medium); // 0.05/day → boundary
        assert_eq!(case(6), HaltRiskTier::High);
        assert_eq!(case(21), HaltRiskTier::Extreme);
    }
}
