//! COT (Commitment of Traders) report decoder + extreme-positioning
//! signal — published weekly by the CFTC for every futures contract.
//!
//! Net-position concepts:
//!   - **Commercials** = hedgers (producers, processors). When they hit
//!     a multi-year net-long extreme, they signal a *bullish* market
//!     (they buy futures to hedge inventory at low prices).
//!   - **Large speculators** (managed money, non-commercial) — trend
//!     followers. Their net-long peaks tend to mark *short-term tops*
//!     (overcrowded long).
//!   - **Small speculators** — retail-sized accounts. Contrarian
//!     indicator (small spec net-long peaks = top).
//!
//! Compute per-week net positions, then rank each over a rolling
//! `lookback_weeks` window into a 0..100 percentile. Extreme zones
//! (≤ `extreme_low_pct` or ≥ `extreme_high_pct`) flag setup conditions.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WeeklyEntry {
    pub commercial_long: i64,
    pub commercial_short: i64,
    pub large_spec_long: i64,
    pub large_spec_short: i64,
    pub small_spec_long: i64,
    pub small_spec_short: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CotConfig {
    pub lookback_weeks: usize,
    pub extreme_low_pct: f64,
    pub extreme_high_pct: f64,
}

impl Default for CotConfig {
    fn default() -> Self {
        Self { lookback_weeks: 156, extreme_low_pct: 10.0, extreme_high_pct: 90.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetPositions {
    pub commercial_net: Vec<i64>,
    pub large_spec_net: Vec<i64>,
    pub small_spec_net: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CotIndex {
    pub commercial_index: Vec<Option<f64>>,
    pub large_spec_index: Vec<Option<f64>>,
    pub small_spec_index: Vec<Option<f64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Signal {
    CommercialExtremeLong,    // bullish (smart money buying)
    CommercialExtremeShort,   // bearish (smart money distributing)
    LargeSpecExtremeLong,     // crowded long → top warning
    LargeSpecExtremeShort,    // crowded short → bottom warning
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CotReport {
    pub net_positions: NetPositions,
    pub index: CotIndex,
    pub signals_at_latest: Vec<Signal>,
}

pub fn analyze(entries: &[WeeklyEntry], cfg: &CotConfig) -> CotReport {
    let n = entries.len();
    let mut report = CotReport::default();
    if cfg.lookback_weeks == 0
        || !cfg.extreme_low_pct.is_finite()
        || !cfg.extreme_high_pct.is_finite()
        || !(0.0..=100.0).contains(&cfg.extreme_low_pct)
        || !(0.0..=100.0).contains(&cfg.extreme_high_pct)
        || cfg.extreme_low_pct >= cfg.extreme_high_pct
    {
        return report;
    }
    let mut commercial = Vec::with_capacity(n);
    let mut large = Vec::with_capacity(n);
    let mut small = Vec::with_capacity(n);
    for e in entries {
        commercial.push(e.commercial_long.saturating_sub(e.commercial_short));
        large.push(e.large_spec_long.saturating_sub(e.large_spec_short));
        small.push(e.small_spec_long.saturating_sub(e.small_spec_short));
    }
    let commercial_idx = rolling_percent_rank(&commercial, cfg.lookback_weeks);
    let large_idx = rolling_percent_rank(&large, cfg.lookback_weeks);
    let small_idx = rolling_percent_rank(&small, cfg.lookback_weeks);
    // Signals only at latest bar.
    if n > 0 {
        if let Some(v) = commercial_idx[n - 1] {
            if v >= cfg.extreme_high_pct { report.signals_at_latest.push(Signal::CommercialExtremeLong); }
            if v <= cfg.extreme_low_pct  { report.signals_at_latest.push(Signal::CommercialExtremeShort); }
        }
        if let Some(v) = large_idx[n - 1] {
            if v >= cfg.extreme_high_pct { report.signals_at_latest.push(Signal::LargeSpecExtremeLong); }
            if v <= cfg.extreme_low_pct  { report.signals_at_latest.push(Signal::LargeSpecExtremeShort); }
        }
    }
    report.net_positions = NetPositions {
        commercial_net: commercial,
        large_spec_net: large,
        small_spec_net: small,
    };
    report.index = CotIndex {
        commercial_index: commercial_idx,
        large_spec_index: large_idx,
        small_spec_index: small_idx,
    };
    report
}

/// Rolling percent-rank: at each i, "% of last `period` values that
/// are strictly less than values[i]" — returns 0..100, None for warmup.
fn rolling_percent_rank(values: &[i64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    for i in (period - 1)..n {
        let lo = i + 1 - period;
        let win = &values[lo..=i];
        let cur = values[i];
        let below = win.iter().filter(|x| **x < cur).count() as f64;
        out[i] = Some(below / period as f64 * 100.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(cl: i64, cs: i64, ll: i64, ls: i64, sl: i64, ss: i64) -> WeeklyEntry {
        WeeklyEntry {
            commercial_long: cl, commercial_short: cs,
            large_spec_long: ll, large_spec_short: ls,
            small_spec_long: sl, small_spec_short: ss,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], &CotConfig::default());
        assert!(r.signals_at_latest.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let entries = vec![e(10_000, 5_000, 5_000, 10_000, 1_000, 500); 200];
        for cfg in [
            CotConfig { lookback_weeks: 0, ..Default::default() },
            CotConfig { extreme_low_pct: -1.0, ..Default::default() },
            CotConfig { extreme_high_pct: 200.0, ..Default::default() },
            CotConfig { extreme_low_pct: 90.0, extreme_high_pct: 10.0, ..Default::default() },
        ] {
            let r = analyze(&entries, &cfg);
            assert!(r.index.commercial_index.is_empty(),
                "config {:?} should return empty", cfg.lookback_weeks);
        }
    }

    #[test]
    fn too_few_weeks_returns_no_signals_but_keeps_nets() {
        let entries = vec![e(10_000, 5_000, 5_000, 10_000, 1_000, 500); 50];
        let r = analyze(&entries, &CotConfig::default());    // lookback=156
        assert!(r.signals_at_latest.is_empty());
        assert_eq!(r.net_positions.commercial_net.len(), 50);
    }

    #[test]
    fn net_positions_compute_correctly() {
        let entries = vec![e(15_000, 5_000, 3_000, 8_000, 2_000, 1_500)];
        let r = analyze(&entries, &CotConfig { lookback_weeks: 1, ..Default::default() });
        assert_eq!(r.net_positions.commercial_net[0], 10_000);
        assert_eq!(r.net_positions.large_spec_net[0], -5_000);
        assert_eq!(r.net_positions.small_spec_net[0], 500);
    }

    #[test]
    fn extreme_commercial_long_signal_fires() {
        // 156 weeks ramping up — last week commercial net is the maximum →
        // 100th percentile → CommercialExtremeLong fires.
        let entries: Vec<WeeklyEntry> = (0..160)
            .map(|i| e(10_000 + i as i64 * 100, 5_000, 5_000, 5_000, 0, 0))
            .collect();
        let r = analyze(&entries, &CotConfig::default());
        assert!(r.signals_at_latest.contains(&Signal::CommercialExtremeLong));
    }

    #[test]
    fn extreme_large_spec_long_signal_fires() {
        let entries: Vec<WeeklyEntry> = (0..160)
            .map(|i| e(5_000, 5_000, 1_000 + i as i64 * 100, 0, 0, 0))
            .collect();
        let r = analyze(&entries, &CotConfig::default());
        assert!(r.signals_at_latest.contains(&Signal::LargeSpecExtremeLong));
    }

    #[test]
    fn saturating_subtraction_avoids_overflow() {
        let entries = vec![e(i64::MAX, -1, 0, 0, 0, 0); 1];    // long − (−1) would overflow i64
        let r = analyze(&entries, &CotConfig { lookback_weeks: 1, ..Default::default() });
        assert_eq!(r.net_positions.commercial_net[0], i64::MAX);    // saturated
    }

    #[test]
    fn percent_rank_basic() {
        let v = vec![1, 2, 3, 4, 5];
        let out = rolling_percent_rank(&v, 5);
        // index 4 is greatest → 4 below out of 5 = 80%.
        assert_eq!(out[4], Some(80.0));
    }
}
