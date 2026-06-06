//! Premarket Gap-Percent Scanner — ranks symbols by premarket gap vs
//! prior close, filters by minimum premarket volume + gap thresholds.
//!
//! Day-trader workflow at 9:25 ET:
//!   1. Pull every symbol with premarket activity.
//!   2. Compute `gap_pct = (premarket_last − prior_close) / prior_close`.
//!   3. Filter to `gap_pct ≥ min_gap_pct` AND
//!      `premarket_volume ≥ min_volume` AND
//!      `relative_volume ≥ min_rvol` (vs N-day premarket-avg-volume).
//!   4. Rank descending by gap_pct.
//!
//! Output: ranked candidates classified as Gapper / FaderCandidate
//! (gap-and-fade pattern setup) / NewsCandidate based on `prior_news_flag`.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremarketSnapshot {
    pub symbol: String,
    pub prior_close: f64,
    pub premarket_last: f64,
    pub premarket_volume: f64,
    /// Average premarket volume over `N` prior sessions (caller supplies).
    pub avg_premarket_volume: f64,
    pub float_shares: Option<f64>,
    pub prior_news_flag: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub min_gap_pct: f64,          // 0.04 = +4%
    pub min_premarket_volume: f64, // 50_000 shares
    pub min_rvol: f64,             // 5.0 — premarket vol vs avg
    pub max_float: Option<f64>,    // optional float cap (low-float runners)
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            min_gap_pct: 0.04,
            min_premarket_volume: 50_000.0,
            min_rvol: 5.0,
            max_float: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    NewsGapper,
    LowFloatRunner,
    Gapper,
    Fader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub symbol: String,
    pub gap_pct: f64,
    pub premarket_volume: f64,
    pub rvol: f64,
    pub classification: Classification,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScannerReport {
    pub candidates: Vec<Candidate>,
    pub gappers_up: Vec<String>,
    pub gappers_down: Vec<String>,
}

pub fn scan(snapshots: &[PremarketSnapshot], cfg: &ScannerConfig) -> ScannerReport {
    let mut report = ScannerReport::default();
    if !cfg.min_gap_pct.is_finite()
        || cfg.min_gap_pct <= 0.0
        || cfg.min_premarket_volume < 0.0
        || cfg.min_rvol <= 0.0
        || !cfg.min_rvol.is_finite()
    {
        return report;
    }
    for s in snapshots {
        if !s.prior_close.is_finite()
            || !s.premarket_last.is_finite()
            || !s.premarket_volume.is_finite()
            || !s.avg_premarket_volume.is_finite()
            || s.prior_close <= 0.0
        {
            continue;
        }
        let gap_pct = (s.premarket_last - s.prior_close) / s.prior_close;
        if !gap_pct.is_finite() || gap_pct.abs() < cfg.min_gap_pct {
            continue;
        }
        if s.premarket_volume < cfg.min_premarket_volume {
            continue;
        }
        let rvol = if s.avg_premarket_volume > 0.0 {
            s.premarket_volume / s.avg_premarket_volume
        } else {
            f64::INFINITY // no prior baseline = treat as extreme
        };
        if rvol < cfg.min_rvol {
            continue;
        }
        if let (Some(cap), Some(f_)) = (cfg.max_float, s.float_shares) {
            if !f_.is_finite() || f_ > cap {
                continue;
            }
        }
        let classification = if s.prior_news_flag {
            Classification::NewsGapper
        } else if matches!(s.float_shares, Some(f_) if f_.is_finite() && f_ < 20_000_000.0) {
            Classification::LowFloatRunner
        } else if gap_pct > 0.0 {
            Classification::Gapper
        } else {
            Classification::Fader
        };
        report.candidates.push(Candidate {
            symbol: s.symbol.clone(),
            gap_pct,
            premarket_volume: s.premarket_volume,
            rvol,
            classification,
        });
    }
    // Rank descending by |gap_pct|.
    report.candidates.sort_by(|a, b| {
        b.gap_pct
            .abs()
            .partial_cmp(&a.gap_pct.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for c in &report.candidates {
        if c.gap_pct > 0.0 {
            report.gappers_up.push(c.symbol.clone());
        } else {
            report.gappers_down.push(c.symbol.clone());
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(sym: &str, close: f64, pm: f64, vol: f64, avg: f64) -> PremarketSnapshot {
        PremarketSnapshot {
            symbol: sym.into(),
            prior_close: close,
            premarket_last: pm,
            premarket_volume: vol,
            avg_premarket_volume: avg,
            float_shares: None,
            prior_news_flag: false,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = scan(&[], &ScannerConfig::default());
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let snaps = vec![snap("X", 100.0, 110.0, 100_000.0, 10_000.0)];
        for cfg in [
            ScannerConfig {
                min_gap_pct: 0.0,
                ..Default::default()
            },
            ScannerConfig {
                min_gap_pct: f64::NAN,
                ..Default::default()
            },
            ScannerConfig {
                min_rvol: 0.0,
                ..Default::default()
            },
        ] {
            assert!(scan(&snaps, &cfg).candidates.is_empty());
        }
    }

    #[test]
    fn zero_prior_close_skipped() {
        let snaps = vec![snap("X", 0.0, 110.0, 100_000.0, 10_000.0)];
        let r = scan(&snaps, &ScannerConfig::default());
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn classic_gapper_detected_and_ranked_up() {
        // +10% gap, high volume.
        let snaps = vec![snap("RUNNER", 100.0, 110.0, 500_000.0, 50_000.0)];
        let r = scan(&snaps, &ScannerConfig::default());
        assert_eq!(r.candidates.len(), 1);
        assert_eq!(r.candidates[0].classification, Classification::Gapper);
        assert!(r.gappers_up.contains(&"RUNNER".to_string()));
    }

    #[test]
    fn news_flag_promotes_to_news_gapper() {
        let mut s = snap("NEWS", 100.0, 110.0, 500_000.0, 50_000.0);
        s.prior_news_flag = true;
        let r = scan(&[s], &ScannerConfig::default());
        assert_eq!(r.candidates[0].classification, Classification::NewsGapper);
    }

    #[test]
    fn low_float_promotes_classification() {
        let mut s = snap("LOWFLOAT", 100.0, 110.0, 500_000.0, 50_000.0);
        s.float_shares = Some(5_000_000.0);
        let r = scan(&[s], &ScannerConfig::default());
        assert_eq!(
            r.candidates[0].classification,
            Classification::LowFloatRunner
        );
    }

    #[test]
    fn negative_gap_classified_as_fader() {
        let snaps = vec![snap("DOWN", 100.0, 90.0, 500_000.0, 50_000.0)];
        let r = scan(&snaps, &ScannerConfig::default());
        assert_eq!(r.candidates[0].classification, Classification::Fader);
        assert!(r.gappers_down.contains(&"DOWN".to_string()));
    }

    #[test]
    fn low_volume_filtered_out() {
        let snaps = vec![snap("THIN", 100.0, 110.0, 1_000.0, 100.0)];
        let r = scan(&snaps, &ScannerConfig::default());
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn low_rvol_filtered_out() {
        // Volume high but average is ALSO high → rvol < 5.
        let snaps = vec![snap("X", 100.0, 110.0, 100_000.0, 100_000.0)];
        let r = scan(&snaps, &ScannerConfig::default());
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn float_cap_filter_works() {
        let mut s = snap("BIG", 100.0, 110.0, 500_000.0, 50_000.0);
        s.float_shares = Some(500_000_000.0);
        let cfg = ScannerConfig {
            max_float: Some(50_000_000.0),
            ..Default::default()
        };
        let r = scan(&[s], &cfg);
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn candidates_ranked_by_absolute_gap_desc() {
        let snaps = vec![
            snap("SMALL", 100.0, 105.0, 500_000.0, 50_000.0), // +5%
            snap("HUGE", 100.0, 130.0, 500_000.0, 50_000.0),  // +30%
            snap("MID", 100.0, 110.0, 500_000.0, 50_000.0),   // +10%
        ];
        let r = scan(&snaps, &ScannerConfig::default());
        let order: Vec<&str> = r.candidates.iter().map(|c| c.symbol.as_str()).collect();
        assert_eq!(order, vec!["HUGE", "MID", "SMALL"]);
    }
}
