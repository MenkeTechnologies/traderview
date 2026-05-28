//! Post-Earnings Announcement Drift (PEAD) scanner.
//!
//! After an earnings release, stocks that beat expectations TEND to
//! drift further upward over the following weeks; stocks that miss
//! tend to drift further downward. The PEAD anomaly is one of the
//! most robust documented in finance (Bernard & Thomas 1989, replicated
//! through 2024).
//!
//! This scanner takes a list of recent earnings events with
//!   - days_since_announcement
//!   - announcement-day surprise % (consensus vs actual)
//!   - announcement-day price reaction %
//!   - post-announcement performance %
//!
//! and classifies each for drift continuation:
//!   - **Beat** (positive surprise): drift target is positive; flag
//!     symbols within `drift_window` days where post-perf is positive
//!     and surprise ≥ `min_surprise_pct`.
//!   - **Miss** (negative surprise): mirror.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsEvent {
    pub symbol: String,
    pub days_since_announcement: i64,
    /// Surprise as a fraction (0.05 = +5% above consensus).
    pub surprise_pct: f64,
    /// Price reaction on the announcement day (open → close).
    pub announcement_day_return_pct: f64,
    /// Cumulative price return since the announcement.
    pub post_announcement_return_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeadConfig {
    /// Days post-announcement to keep tracking drift (typical 45–60).
    pub drift_window: i64,
    pub min_surprise_pct: f64,
    /// Minimum announcement-day reaction (e.g. 0.03 = +/-3%) to confirm
    /// the market caught the surprise.
    pub min_announcement_reaction: f64,
}

impl Default for PeadConfig {
    fn default() -> Self {
        Self { drift_window: 60, min_surprise_pct: 0.05, min_announcement_reaction: 0.03 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftBias {
    /// Beat + still positive drift expected.
    BeatContinuation,
    /// Miss + still negative drift expected.
    MissContinuation,
    /// Beat but drift has already faded — exit-the-trade flag.
    BeatFaded,
    /// Miss but stock has recovered — short-squeeze warning.
    MissFaded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftCandidate {
    pub symbol: String,
    pub bias: DriftBias,
    pub days_since: i64,
    pub surprise_pct: f64,
    pub post_announcement_return_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeadReport {
    pub candidates: Vec<DriftCandidate>,
    pub n_beats: usize,
    pub n_misses: usize,
}

pub fn analyze(events: &[EarningsEvent], cfg: &PeadConfig) -> PeadReport {
    let mut report = PeadReport::default();
    if cfg.drift_window <= 0
        || !cfg.min_surprise_pct.is_finite()
        || cfg.min_surprise_pct <= 0.0
        || !cfg.min_announcement_reaction.is_finite()
        || cfg.min_announcement_reaction <= 0.0
    {
        return report;
    }
    for e in events {
        // Skip events outside the drift window or with non-finite metrics.
        if e.days_since_announcement < 0
            || e.days_since_announcement > cfg.drift_window
            || !e.surprise_pct.is_finite()
            || !e.announcement_day_return_pct.is_finite()
            || !e.post_announcement_return_pct.is_finite()
        {
            continue;
        }
        let abs_surprise = e.surprise_pct.abs();
        if abs_surprise < cfg.min_surprise_pct {
            continue;
        }
        let abs_reaction = e.announcement_day_return_pct.abs();
        if abs_reaction < cfg.min_announcement_reaction {
            continue;
        }
        let bias = if e.surprise_pct > 0.0 {
            report.n_beats += 1;
            if e.post_announcement_return_pct > 0.0 {
                DriftBias::BeatContinuation
            } else {
                DriftBias::BeatFaded
            }
        } else {
            report.n_misses += 1;
            if e.post_announcement_return_pct < 0.0 {
                DriftBias::MissContinuation
            } else {
                DriftBias::MissFaded
            }
        };
        report.candidates.push(DriftCandidate {
            symbol: e.symbol.clone(),
            bias,
            days_since: e.days_since_announcement,
            surprise_pct: e.surprise_pct,
            post_announcement_return_pct: e.post_announcement_return_pct,
        });
    }
    // Sort: beats with biggest drift first, then misses.
    report.candidates.sort_by(|a, b| {
        b.surprise_pct
            .abs()
            .partial_cmp(&a.surprise_pct.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(sym: &str, days: i64, surprise: f64, react: f64, post: f64) -> EarningsEvent {
        EarningsEvent {
            symbol: sym.into(),
            days_since_announcement: days,
            surprise_pct: surprise,
            announcement_day_return_pct: react,
            post_announcement_return_pct: post,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], &PeadConfig::default());
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let events = vec![ev("X", 10, 0.10, 0.05, 0.08)];
        for cfg in [
            PeadConfig { drift_window: 0, ..Default::default() },
            PeadConfig { drift_window: -1, ..Default::default() },
            PeadConfig { min_surprise_pct: 0.0, ..Default::default() },
            PeadConfig { min_announcement_reaction: -1.0, ..Default::default() },
        ] {
            assert!(analyze(&events, &cfg).candidates.is_empty());
        }
    }

    #[test]
    fn beat_with_positive_post_drift_classified_as_continuation() {
        let r = analyze(
            &[ev("BEAT", 5, 0.10, 0.06, 0.04)],
            &PeadConfig::default(),
        );
        assert_eq!(r.candidates.len(), 1);
        assert_eq!(r.candidates[0].bias, DriftBias::BeatContinuation);
        assert_eq!(r.n_beats, 1);
    }

    #[test]
    fn miss_with_negative_post_drift_classified_as_continuation() {
        let r = analyze(
            &[ev("MISS", 5, -0.10, -0.06, -0.04)],
            &PeadConfig::default(),
        );
        assert_eq!(r.candidates[0].bias, DriftBias::MissContinuation);
        assert_eq!(r.n_misses, 1);
    }

    #[test]
    fn beat_that_faded_classified_as_beat_faded() {
        let r = analyze(
            &[ev("FADER", 30, 0.10, 0.06, -0.05)],
            &PeadConfig::default(),
        );
        assert_eq!(r.candidates[0].bias, DriftBias::BeatFaded);
    }

    #[test]
    fn small_surprise_filtered_out() {
        let r = analyze(
            &[ev("NOISE", 5, 0.02, 0.06, 0.04)],
            &PeadConfig::default(),
        );
        assert!(r.candidates.is_empty(), "surprise below 5% threshold should be filtered");
    }

    #[test]
    fn small_announcement_reaction_filtered_out() {
        let r = analyze(
            &[ev("WEAK", 5, 0.10, 0.005, 0.04)],
            &PeadConfig::default(),
        );
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn outside_drift_window_filtered() {
        let r = analyze(
            &[ev("OLD", 90, 0.10, 0.06, 0.04)],
            &PeadConfig::default(),    // 60-day window
        );
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn candidates_sorted_by_absolute_surprise() {
        let events = vec![
            ev("SMALL", 5, 0.06, 0.04, 0.02),
            ev("BIG",   5, 0.25, 0.10, 0.05),
            ev("MID",   5, 0.10, 0.05, 0.03),
        ];
        let r = analyze(&events, &PeadConfig::default());
        let order: Vec<&str> = r.candidates.iter().map(|c| c.symbol.as_str()).collect();
        assert_eq!(order, vec!["BIG", "MID", "SMALL"]);
    }
}
