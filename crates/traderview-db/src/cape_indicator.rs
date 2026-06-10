//! Robert Shiller's Cyclically-Adjusted Price-to-Earnings (CAPE) ratio.
//!
//! CAPE = S&P 500 price / 10-year inflation-adjusted average earnings.
//! Shiller's seminal work (Irrational Exuberance, 2000) showed CAPE
//! correlates with subsequent 10-year returns: high CAPE = low future
//! returns, low CAPE = high future returns.
//!
//! Historical context (Shiller's S&P 500 dataset, 1881-2024):
//!   * Mean: ~17.4
//!   * Median: ~15.9
//!   * 25th percentile: ~11.5
//!   * 75th percentile: ~20.6
//!   * 95th percentile: ~30
//!   * 99th percentile: ~36
//!   * Maximum: 44.2 (Dec 1999 dot-com peak)
//!
//! The historical distribution snapshot below covers quarterly CAPE
//! values from Shiller's published data, mirrored from multpl.com.
//! Static — needs annual refresh from upstream.
//!
//! User typically wants:
//!   * Where does current CAPE sit in the historical distribution?
//!   * What CAPE value triggers "elevated" / "extreme" warnings?

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CapeScore {
    pub current_value: f64,
    pub historical_mean: f64,
    pub historical_median: f64,
    pub percentile_pct: f64,
    pub regime: &'static str, // "depressed" | "below_avg" | "near_avg" | "elevated" | "extreme"
    pub historical_25th: f64,
    pub historical_75th: f64,
    pub historical_95th: f64,
    pub historical_max: f64,
    pub interpretation: String,
    pub recent_quarterly: Vec<HistoricalCape>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct HistoricalCape {
    pub year: i32,
    pub quarter: u32,
    pub value: f64,
}

// ─── Historical reference data ─────────────────────────────────────────────

/// Quarterly CAPE snapshots from Shiller's S&P 500 dataset
/// (multpl.com/shiller-pe), 2010-2024. Embedded so the indicator works
/// without a live data feed. Refresh annually.
pub const RECENT_QUARTERLY: &[HistoricalCape] = &[
    HistoricalCape {
        year: 2010,
        quarter: 1,
        value: 20.3,
    },
    HistoricalCape {
        year: 2011,
        quarter: 1,
        value: 22.9,
    },
    HistoricalCape {
        year: 2012,
        quarter: 1,
        value: 22.1,
    },
    HistoricalCape {
        year: 2013,
        quarter: 1,
        value: 22.5,
    },
    HistoricalCape {
        year: 2014,
        quarter: 1,
        value: 25.2,
    },
    HistoricalCape {
        year: 2015,
        quarter: 1,
        value: 27.0,
    },
    HistoricalCape {
        year: 2016,
        quarter: 1,
        value: 24.4,
    },
    HistoricalCape {
        year: 2017,
        quarter: 1,
        value: 28.7,
    },
    HistoricalCape {
        year: 2018,
        quarter: 1,
        value: 33.3,
    },
    HistoricalCape {
        year: 2019,
        quarter: 1,
        value: 28.5,
    },
    HistoricalCape {
        year: 2020,
        quarter: 1,
        value: 24.8,
    },
    HistoricalCape {
        year: 2021,
        quarter: 1,
        value: 35.5,
    },
    HistoricalCape {
        year: 2022,
        quarter: 1,
        value: 37.5,
    },
    HistoricalCape {
        year: 2023,
        quarter: 1,
        value: 28.9,
    },
    HistoricalCape {
        year: 2024,
        quarter: 1,
        value: 34.2,
    },
    HistoricalCape {
        year: 2024,
        quarter: 4,
        value: 36.8,
    },
];

/// Historical distribution stats per Shiller's full 1881-2024 dataset.
const HISTORICAL_MEAN: f64 = 17.4;
const HISTORICAL_MEDIAN: f64 = 15.9;
const HISTORICAL_25TH: f64 = 11.5;
const HISTORICAL_75TH: f64 = 20.6;
const HISTORICAL_95TH: f64 = 30.0;
const HISTORICAL_MAX: f64 = 44.2;

/// CAPE values from Shiller's full 1881-2024 dataset (annual), used to
/// compute the percentile rank of a current value. About 140 yearly
/// snapshots — small enough to embed, large enough to give a stable
/// percentile estimate.
const FULL_HISTORY: &[f64] = &[
    18.5, 17.6, 17.4, 14.5, 14.5, 18.3, 18.0, 13.5, 15.1, 17.1, // 1881-1890
    16.3, 14.5, 15.7, 17.7, 16.0, 15.4, 17.1, 18.3, 21.8, 21.8, // 1891-1900
    23.2, 19.5, 17.1, 18.6, 22.6, 20.2, 17.5, 13.6, 15.1, 18.5, // 1901-1910
    16.6, 17.6, 14.7, 15.4, 13.0, 13.8, 11.9, 10.4, 10.5, 12.1, // 1911-1920
    6.4, 7.5, 8.2, 10.5, 12.7, 11.5, 13.0, 18.8, 27.1, 22.3, // 1921-1930
    19.0, 9.3, 10.2, 18.5, 14.7, 16.5, 20.6, 15.0, 17.5, 17.9, // 1931-1940
    16.4, 11.0, 12.4, 12.9, 13.6, 19.2, 13.7, 11.6, 11.0, 10.8, // 1941-1950
    13.2, 12.5, 12.7, 13.7, 17.9, 17.4, 17.7, 13.6, 16.7, 19.4, // 1951-1960
    19.6, 22.2, 21.5, 22.8, 23.3, 23.3, 21.0, 21.7, 21.0, 18.4, // 1961-1970
    16.6, 17.6, 18.7, 13.5, 8.9, 8.9, 11.2, 9.3, 9.4, 8.9, // 1971-1980
    9.2, 8.0, 7.4, 9.9, 9.7, 10.4, 14.7, 15.1, 14.7, 17.1, // 1981-1990
    15.5, 18.0, 20.0, 20.4, 20.2, 20.8, 24.7, 28.3, 32.9, 40.6, // 1991-2000
    36.6, 30.3, 21.5, 22.7, 26.5, 26.6, 27.4, 24.2, 15.5, 20.3, // 2001-2010
    22.9, 22.1, 22.5, 25.2, 27.0, 24.4, 28.7, 33.3, 28.5, 24.8, // 2011-2020
    35.5, 37.5, 28.9, 34.2, // 2021-2024
];

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Percentile rank of `value` within the historical distribution.
/// Returns 0..100 — where 50 means "half of all observations were below."
pub fn percentile_rank(value: f64, history: &[f64]) -> f64 {
    if history.is_empty() {
        return 0.0;
    }
    let below = history.iter().filter(|h| **h < value).count() as f64;
    below / history.len() as f64 * 100.0
}

/// Classify a CAPE value into a regime label.
pub fn classify(value: f64) -> &'static str {
    if value < HISTORICAL_25TH {
        "depressed"
    } else if value < HISTORICAL_MEAN - 2.0 {
        "below_avg"
    } else if value < HISTORICAL_MEAN + 2.0 {
        "near_avg"
    } else if value < 30.0 {
        "elevated"
    } else {
        "extreme"
    }
}

pub fn interpretation_for(value: f64) -> String {
    let regime = classify(value);
    match regime {
        "depressed" => "Historically very low — multi-year forward returns tend to be high. Aggressive accumulation can be rewarded.".into(),
        "below_avg" => "Below historical average — forward returns tend to be above average.".into(),
        "near_avg" => "Near historical average — expected forward returns roughly typical (~6-7% real).".into(),
        "elevated" => "Above historical average — expected forward returns below average (~3-5% real). Lower-risk allocations may be prudent.".into(),
        "extreme" => "Top decile historically — Shiller's regression suggests <2% real forward returns over the next 10 years. Defensive posture warranted.".into(),
        _ => "Unknown regime.".into(),
    }
}

/// Build the full score for a given CAPE value.
pub fn score(value: f64) -> CapeScore {
    let percentile = percentile_rank(value, FULL_HISTORY);
    CapeScore {
        current_value: value,
        historical_mean: HISTORICAL_MEAN,
        historical_median: HISTORICAL_MEDIAN,
        percentile_pct: percentile,
        regime: classify(value),
        historical_25th: HISTORICAL_25TH,
        historical_75th: HISTORICAL_75TH,
        historical_95th: HISTORICAL_95TH,
        historical_max: HISTORICAL_MAX,
        interpretation: interpretation_for(value),
        recent_quarterly: RECENT_QUARTERLY.to_vec(),
    }
}

/// Most recent embedded CAPE value — what to default the UI to when no
/// user input is provided.
pub fn latest_known_value() -> f64 {
    RECENT_QUARTERLY.last().map(|h| h.value).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_known_regimes() {
        assert_eq!(classify(8.0), "depressed");
        assert_eq!(classify(13.0), "below_avg");
        assert_eq!(classify(17.0), "near_avg");
        assert_eq!(classify(22.0), "elevated");
        assert_eq!(classify(35.0), "extreme");
        assert_eq!(classify(45.0), "extreme");
    }

    #[test]
    fn percentile_rank_extremes() {
        // Value below all history → 0
        let p = percentile_rank(0.0, FULL_HISTORY);
        assert!(p == 0.0);
        // Value above all history → 100
        let p = percentile_rank(100.0, FULL_HISTORY);
        assert!(p == 100.0);
    }

    #[test]
    fn percentile_rank_mid_value() {
        // Value near median → percentile near 50.
        let p = percentile_rank(15.9, FULL_HISTORY);
        assert!(
            (p - 50.0).abs() < 30.0,
            "median percentile = {p}, expected near 50"
        );
    }

    #[test]
    fn percentile_rank_empty_history() {
        assert_eq!(percentile_rank(20.0, &[]), 0.0);
    }

    #[test]
    fn score_returns_consistent_structure() {
        let s = score(33.0);
        assert_eq!(s.current_value, 33.0);
        assert_eq!(s.historical_mean, HISTORICAL_MEAN);
        assert!(
            s.percentile_pct > 75.0,
            "33 should be high percentile, got {}",
            s.percentile_pct
        );
        assert_eq!(s.regime, "extreme");
        assert!(!s.recent_quarterly.is_empty());
    }

    #[test]
    fn score_at_historical_mean_classifies_near_avg() {
        let s = score(HISTORICAL_MEAN);
        assert_eq!(s.regime, "near_avg");
    }

    #[test]
    fn latest_known_value_returns_last_embedded() {
        let v = latest_known_value();
        assert!(v > 0.0);
        assert!(v < 100.0);
    }

    #[test]
    fn historical_distribution_covers_century_plus() {
        // Sanity: at least ~140 annual snapshots from 1881 through 2024.
        assert!(FULL_HISTORY.len() >= 140);
    }

    #[test]
    fn interpretation_non_empty_for_all_regimes() {
        for v in &[8.0, 13.0, 17.0, 22.0, 35.0, 45.0] {
            assert!(!interpretation_for(*v).is_empty());
        }
    }
}
