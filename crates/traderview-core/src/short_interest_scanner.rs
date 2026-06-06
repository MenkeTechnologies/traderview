//! Short-interest squeeze candidate scanner.
//!
//! Combines two short-interest metrics into a squeeze-risk score:
//!   - **short_float_pct**: shares short / float (≥ 20% = high)
//!   - **days_to_cover**: short interest / avg daily volume (≥ 5 = high)
//!
//! Plus optional confirmation: a `recent_price_pct_change` field flags
//! symbols that have already moved up — short squeezes only run hard
//! when shorts are caught underwater.
//!
//! Output: ranked candidates with classification:
//!   - **PrimedSqueeze**: high short-float AND high days-to-cover AND recent move up
//!   - **HighShortInterest**: both metrics high but no confirming price move yet
//!   - **CrowdedShort**: high short-float OR high days-to-cover, no move
//!   - **None**: doesn't meet thresholds
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortInterestEntry {
    pub symbol: String,
    pub short_float_pct: f64, // 0..1 (0.30 = 30%)
    pub days_to_cover: f64,
    pub recent_price_pct_change: f64, // last N days, e.g. +0.15 = +15%
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqueezeRisk {
    PrimedSqueeze,
    HighShortInterest,
    CrowdedShort,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub high_short_float: f64, // default 0.20
    pub high_dtc: f64,         // default 5.0
    /// Minimum recent price move to confirm the squeeze is starting.
    pub min_squeeze_move: f64, // default 0.05 (+5%)
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            high_short_float: 0.20,
            high_dtc: 5.0,
            min_squeeze_move: 0.05,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqueezeCandidate {
    pub symbol: String,
    pub short_float_pct: f64,
    pub days_to_cover: f64,
    pub recent_price_pct_change: f64,
    pub risk: SqueezeRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScannerReport {
    pub candidates: Vec<SqueezeCandidate>,
    pub primed: Vec<String>,
    pub high_interest: Vec<String>,
}

pub fn analyze(entries: &[ShortInterestEntry], cfg: &ScannerConfig) -> ScannerReport {
    let mut report = ScannerReport::default();
    if cfg.high_short_float <= 0.0 || cfg.high_dtc <= 0.0 || !cfg.min_squeeze_move.is_finite() {
        return report;
    }
    for e in entries {
        if !e.short_float_pct.is_finite()
            || !e.days_to_cover.is_finite()
            || !e.recent_price_pct_change.is_finite()
        {
            continue;
        }
        let sf_high = e.short_float_pct >= cfg.high_short_float;
        let dtc_high = e.days_to_cover >= cfg.high_dtc;
        let moving_up = e.recent_price_pct_change >= cfg.min_squeeze_move;
        let risk = match (sf_high, dtc_high, moving_up) {
            (true, true, true) => SqueezeRisk::PrimedSqueeze,
            (true, true, false) => SqueezeRisk::HighShortInterest,
            (true, false, _) | (false, true, _) => SqueezeRisk::CrowdedShort,
            (false, false, _) => SqueezeRisk::None,
        };
        if risk == SqueezeRisk::None {
            continue;
        }
        report.candidates.push(SqueezeCandidate {
            symbol: e.symbol.clone(),
            short_float_pct: e.short_float_pct,
            days_to_cover: e.days_to_cover,
            recent_price_pct_change: e.recent_price_pct_change,
            risk,
        });
    }
    // Rank: PrimedSqueeze > HighShortInterest > CrowdedShort, then by short_float desc.
    report.candidates.sort_by(|a, b| {
        let rank = |r: SqueezeRisk| match r {
            SqueezeRisk::PrimedSqueeze => 0,
            SqueezeRisk::HighShortInterest => 1,
            SqueezeRisk::CrowdedShort => 2,
            SqueezeRisk::None => 3,
        };
        rank(a.risk).cmp(&rank(b.risk)).then_with(|| {
            b.short_float_pct
                .partial_cmp(&a.short_float_pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    for c in &report.candidates {
        match c.risk {
            SqueezeRisk::PrimedSqueeze => report.primed.push(c.symbol.clone()),
            SqueezeRisk::HighShortInterest => report.high_interest.push(c.symbol.clone()),
            _ => {}
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(sym: &str, sf: f64, dtc: f64, mv: f64) -> ShortInterestEntry {
        ShortInterestEntry {
            symbol: sym.into(),
            short_float_pct: sf,
            days_to_cover: dtc,
            recent_price_pct_change: mv,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], &ScannerConfig::default());
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let entries = vec![e("X", 0.30, 10.0, 0.10)];
        for cfg in [
            ScannerConfig {
                high_short_float: 0.0,
                ..Default::default()
            },
            ScannerConfig {
                high_dtc: 0.0,
                ..Default::default()
            },
            ScannerConfig {
                min_squeeze_move: f64::NAN,
                ..Default::default()
            },
        ] {
            assert!(analyze(&entries, &cfg).candidates.is_empty());
        }
    }

    #[test]
    fn primed_squeeze_when_all_three_hit() {
        let r = analyze(&[e("GME", 0.30, 8.0, 0.15)], &ScannerConfig::default());
        assert_eq!(r.candidates.len(), 1);
        assert_eq!(r.candidates[0].risk, SqueezeRisk::PrimedSqueeze);
        assert!(r.primed.contains(&"GME".to_string()));
    }

    #[test]
    fn high_interest_when_both_metrics_but_no_move() {
        let r = analyze(&[e("AMC", 0.30, 8.0, 0.01)], &ScannerConfig::default());
        assert_eq!(r.candidates[0].risk, SqueezeRisk::HighShortInterest);
    }

    #[test]
    fn crowded_when_only_one_metric() {
        let r = analyze(&[e("ONLY_SF", 0.25, 2.0, 0.0)], &ScannerConfig::default());
        assert_eq!(r.candidates[0].risk, SqueezeRisk::CrowdedShort);
    }

    #[test]
    fn low_short_interest_filtered() {
        let r = analyze(&[e("NORM", 0.05, 1.0, 0.0)], &ScannerConfig::default());
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn nan_inputs_skipped_safely() {
        let r = analyze(&[e("X", f64::NAN, 8.0, 0.10)], &ScannerConfig::default());
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn candidates_sorted_primed_first_then_by_short_float_desc() {
        let entries = vec![
            e("CROWD", 0.25, 2.0, 0.0),   // crowded
            e("PRIMED", 0.22, 7.0, 0.10), // primed (lower SF than CROWD2)
            e("HIGH", 0.40, 8.0, 0.01),   // high interest, no move
        ];
        let r = analyze(&entries, &ScannerConfig::default());
        // PRIMED first (highest rank), then HIGH (high_interest), then CROWD.
        assert_eq!(r.candidates[0].symbol, "PRIMED");
        assert_eq!(r.candidates[1].symbol, "HIGH");
        assert_eq!(r.candidates[2].symbol, "CROWD");
    }
}
