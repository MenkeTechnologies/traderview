//! Bid/ask spread tracker — measures and classifies the cost of crossing.
//!
//! The bid/ask spread is the implicit cost of immediacy: every market order
//! pays half the spread vs. the mid. For thin / volatile names the spread
//! widens to multiple times the daily range, making "looks profitable on
//! paper" trades break even or worse net of execution.
//!
//! This module aggregates per-symbol bid/ask snapshots into spread stats:
//!   - **spread_bps**: (ask - bid) / mid × 10,000
//!   - **regime**: Tight (≤ 5 bps), Normal (5-25 bps), Wide (25-100 bps),
//!     Pathological (≥ 100 bps — 1% spread, market is broken or you're
//!     trading a wide-tick illiquid name)
//!   - **min/max/avg** spread over the sample
//!
//! Pure compute. Caller supplies a series of (bid, ask) snapshots from
//! Level 1 quotes. Implementation matches the spread-cost decomp used in
//! TCA reports (TradeStation, IBKR Best Execution).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuoteSnapshot {
    pub bid: f64,
    pub ask: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpreadRegime {
    Tight,
    #[default]
    Normal,
    Wide,
    Pathological,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpreadReport {
    pub samples: usize,
    pub avg_spread_bps: f64,
    pub min_spread_bps: f64,
    pub max_spread_bps: f64,
    pub avg_mid: f64,
    pub regime: SpreadRegime,
    /// Fraction of samples that were Pathological (> 100bps) — high values
    /// indicate a stale data feed or a truly broken book.
    pub pathological_pct: f64,
    pub note: String,
}

fn classify(bps: f64) -> SpreadRegime {
    if bps <= 5.0        { SpreadRegime::Tight }
    else if bps <= 25.0  { SpreadRegime::Normal }
    else if bps <= 100.0 { SpreadRegime::Wide }
    else                 { SpreadRegime::Pathological }
}

pub fn analyze(samples: &[QuoteSnapshot]) -> SpreadReport {
    if samples.is_empty() {
        return SpreadReport { note: "no samples".into(), ..Default::default() };
    }
    let (mut sum_bps, mut sum_mid) = (0.0_f64, 0.0_f64);
    let mut min_bps = f64::INFINITY;
    let mut max_bps = f64::NEG_INFINITY;
    let mut pathological = 0usize;
    let mut valid = 0usize;
    for s in samples {
        if !(s.bid.is_finite() && s.ask.is_finite() && s.bid > 0.0 && s.ask >= s.bid) {
            continue;
        }
        let mid = (s.bid + s.ask) / 2.0;
        if mid <= 0.0 { continue; }
        let bps = (s.ask - s.bid) / mid * 10_000.0;
        sum_bps += bps;
        sum_mid += mid;
        if bps < min_bps { min_bps = bps; }
        if bps > max_bps { max_bps = bps; }
        if bps > 100.0   { pathological += 1; }
        valid += 1;
    }
    if valid == 0 {
        return SpreadReport {
            samples: samples.len(),
            note: "no valid samples (bid <= 0, ask < bid, or non-finite)".into(),
            ..Default::default()
        };
    }
    let avg_bps = sum_bps / valid as f64;
    let avg_mid = sum_mid / valid as f64;
    let regime = classify(avg_bps);
    let pathological_pct = pathological as f64 / valid as f64;
    let note = match regime {
        SpreadRegime::Tight        => format!("tight {avg_bps:.1} bps — execute aggressively"),
        SpreadRegime::Normal       => format!("normal {avg_bps:.1} bps"),
        SpreadRegime::Wide         => format!("wide {avg_bps:.1} bps — use limit orders"),
        SpreadRegime::Pathological => format!("pathological {avg_bps:.1} bps — feed broken or illiquid name"),
    };
    SpreadReport {
        samples: valid,
        avg_spread_bps: avg_bps,
        min_spread_bps: min_bps,
        max_spread_bps: max_bps,
        avg_mid,
        regime,
        pathological_pct,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(bid: f64, ask: f64) -> QuoteSnapshot { QuoteSnapshot { bid, ask } }

    #[test]
    fn empty_input_returns_zero_report() {
        let r = analyze(&[]);
        assert_eq!(r.samples, 0);
        assert!(r.note.contains("no samples"));
    }

    #[test]
    fn tight_spread_classifies_tight() {
        // 100.00 / 100.02 → 2 bps spread.
        let r = analyze(&[q(100.00, 100.02), q(100.00, 100.02), q(100.00, 100.02)]);
        assert!(matches!(r.regime, SpreadRegime::Tight));
        assert!(r.avg_spread_bps < 5.0);
    }

    #[test]
    fn wide_spread_classifies_wide() {
        // 50 bps → Wide regime (5-100 bps boundary lands here).
        // 100 / 100.50 → 50 bps.
        let r = analyze(&[q(100.0, 100.5), q(100.0, 100.5), q(100.0, 100.5)]);
        assert!(matches!(r.regime, SpreadRegime::Wide),
            "expected Wide for ~50 bps spread, got {:?} (avg={})", r.regime, r.avg_spread_bps);
    }

    #[test]
    fn pathological_spread_flagged() {
        // 5% spread → Pathological.
        let r = analyze(&[q(100.0, 105.0)]);
        assert!(matches!(r.regime, SpreadRegime::Pathological));
        assert_eq!(r.pathological_pct, 1.0);
        assert!(r.avg_spread_bps > 100.0);
    }

    #[test]
    fn invalid_samples_skipped_but_counted() {
        // Inputs: 1 valid + 3 invalid. Only valid one is averaged.
        let r = analyze(&[
            q(100.0, 100.10),       // valid: 10 bps
            q(0.0,   100.0),        // invalid (bid=0)
            q(100.0, 99.0),         // invalid (ask < bid)
            q(f64::NAN, 100.0),     // invalid (NaN)
        ]);
        assert_eq!(r.samples, 1);
        assert!((r.avg_spread_bps - 10.0).abs() < 0.01);
    }

    #[test]
    fn min_max_track_extremes() {
        // 2-ish, ~30, ~60 bps spreads. Compute the EXACT expected values from
        // the same formula the function uses so the tolerance doesn't have
        // to absorb the bid-vs-mid drift that widens the apparent bps.
        let samples = [q(100.0, 100.02), q(100.0, 100.30), q(100.0, 100.60)];
        let expected_bps: Vec<f64> = samples.iter()
            .map(|s| (s.ask - s.bid) / ((s.bid + s.ask) / 2.0) * 10_000.0)
            .collect();
        let want_min = expected_bps.iter().copied().fold(f64::INFINITY, f64::min);
        let want_max = expected_bps.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let r = analyze(&samples);
        assert!((r.min_spread_bps - want_min).abs() < 1e-9,
            "min: expected {want_min}, got {}", r.min_spread_bps);
        assert!((r.max_spread_bps - want_max).abs() < 1e-9,
            "max: expected {want_max}, got {}", r.max_spread_bps);
    }

    #[test]
    fn all_invalid_returns_zero_with_note() {
        let r = analyze(&[q(0.0, 100.0), q(-1.0, 100.0)]);
        assert_eq!(r.samples, 2);
        assert_eq!(r.avg_spread_bps, 0.0);
        assert!(r.note.contains("no valid"));
    }
}
