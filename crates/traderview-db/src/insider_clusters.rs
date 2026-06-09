//! Insider cluster scoring.
//!
//! Single insider buys carry information (Lakonishok & Lee 2001), but
//! the much sharper signal is **clusters** — multiple insiders at the
//! same issuer buying within a short window. Cohen, Malloy &
//! Pomorski (2012) document that "opportunistic" cluster buys (≥3
//! insiders within 30 days, none with a recurring trading pattern)
//! predict ~12% alpha over the next 12 months — one of the largest
//! free-data edges in the equity factor literature.
//!
//! This module rides on top of [`crate::insider_stream`]:
//!
//!   * No new data fetching — every buy/sell is already indexed by
//!     the upstream EDGAR Form 4 consumer.
//!   * Per-symbol rolling 30-day window of distinct insider buys.
//!   * Cluster score blends:
//!       - **insider_count** — distinct people buying (linear weight)
//!       - **role_mix** — officer + director + 10%-owner counts
//!         (officers weighted highest because they trade on the
//!         most-informational vantage)
//!       - **dollar_total** — sum of buy $ value across the window
//!       - **recency** — exponential decay (half-life 7 days) on
//!         each event's contribution
//!   * Output ranked descending by composite score.
//!
//! Insider sells are not folded in here; the sell side is much
//! noisier (insiders sell for diversification, tax, employment-end,
//! etc.) and including it dampens the cluster signal. The existing
//! insider_stream view surfaces sells separately.

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::insider_stream::{InsiderEvent, InsiderStore, TxKind};

const WINDOW_DAYS: i64 = 30;
const RECENCY_HALF_LIFE_DAYS: f64 = 7.0;
const MIN_INSIDER_COUNT: usize = 2;
const OFFICER_WEIGHT: f64 = 3.0;
const DIRECTOR_WEIGHT: f64 = 1.5;
const TEN_PCT_OWNER_WEIGHT: f64 = 1.0;
const RANK_DOLLAR_REFERENCE: f64 = 1_000_000.0;

#[derive(Debug, Clone, Serialize)]
pub struct InsiderCluster {
    pub symbol: String,
    pub insider_count: usize,
    pub officer_count: usize,
    pub director_count: usize,
    pub ten_pct_owner_count: usize,
    pub buy_count: usize,
    pub total_dollars: f64,
    pub recency_weighted_dollars: f64,
    /// Composite 0-N score. Useful for sorting; the absolute number
    /// has no normalised meaning — it's a relative ranking signal.
    pub score: f64,
    /// First insider buy in the window — anchors how long the cluster
    /// has been building.
    pub earliest_buy: DateTime<Utc>,
    pub latest_buy: DateTime<Utc>,
    /// Distinct insider names in the cluster, sorted alphabetically.
    pub insiders: Vec<String>,
}

/// Recency weight on a single event. Half-life of RECENCY_HALF_LIFE_DAYS
/// days; events past the rolling window get filtered before reaching
/// here, so we never produce negative ages.
pub fn recency_weight(now: DateTime<Utc>, observed: DateTime<Utc>) -> f64 {
    let age_days = (now - observed).num_seconds() as f64 / 86400.0;
    if age_days <= 0.0 {
        1.0
    } else {
        0.5_f64.powf(age_days / RECENCY_HALF_LIFE_DAYS)
    }
}

/// Pure: score one symbol's buy events. Returns `None` when fewer
/// than `MIN_INSIDER_COUNT` distinct insiders are present (no cluster).
pub fn cluster_from_buys(
    symbol: &str,
    buys: &[InsiderEvent],
    now: DateTime<Utc>,
) -> Option<InsiderCluster> {
    if buys.is_empty() {
        return None;
    }
    let cutoff = now - Duration::days(WINDOW_DAYS);
    let in_window: Vec<&InsiderEvent> = buys
        .iter()
        .filter(|e| e.observed_at >= cutoff && matches!(e.kind, TxKind::Buy))
        .collect();
    if in_window.is_empty() {
        return None;
    }
    let mut insider_names: HashSet<String> = HashSet::new();
    let mut officer_names: HashSet<String> = HashSet::new();
    let mut director_names: HashSet<String> = HashSet::new();
    let mut tenpct_names: HashSet<String> = HashSet::new();
    let mut total_dollars = 0.0_f64;
    let mut recency_weighted_dollars = 0.0_f64;
    let mut earliest = in_window[0].observed_at;
    let mut latest = in_window[0].observed_at;
    for e in &in_window {
        insider_names.insert(e.insider_name.clone());
        if e.is_officer {
            officer_names.insert(e.insider_name.clone());
        }
        if e.is_director {
            director_names.insert(e.insider_name.clone());
        }
        if e.is_ten_percent_owner {
            tenpct_names.insert(e.insider_name.clone());
        }
        total_dollars += e.dollar_value;
        recency_weighted_dollars += e.dollar_value * recency_weight(now, e.observed_at);
        if e.observed_at < earliest {
            earliest = e.observed_at;
        }
        if e.observed_at > latest {
            latest = e.observed_at;
        }
    }
    if insider_names.len() < MIN_INSIDER_COUNT {
        return None;
    }
    // Composite: insider count is the primary axis (literature is
    // unambiguous), role mix adds depth, dollar value normalises so
    // small-cap clusters with strong unanimity rank close to large-
    // cap dollar-weighted clusters.
    let role_contribution = (officer_names.len() as f64) * OFFICER_WEIGHT
        + (director_names.len() as f64) * DIRECTOR_WEIGHT
        + (tenpct_names.len() as f64) * TEN_PCT_OWNER_WEIGHT;
    let dollar_contribution = (recency_weighted_dollars / RANK_DOLLAR_REFERENCE).max(0.0);
    let score = (insider_names.len() as f64) * 2.0 + role_contribution + dollar_contribution;
    let mut insiders: Vec<String> = insider_names.into_iter().collect();
    insiders.sort();
    Some(InsiderCluster {
        symbol: symbol.to_ascii_uppercase(),
        insider_count: insiders.len(),
        officer_count: officer_names.len(),
        director_count: director_names.len(),
        ten_pct_owner_count: tenpct_names.len(),
        buy_count: in_window.len(),
        total_dollars,
        recency_weighted_dollars,
        score,
        earliest_buy: earliest,
        latest_buy: latest,
        insiders,
    })
}

/// Repository: rank every symbol in the InsiderStore that has at
/// least `MIN_INSIDER_COUNT` distinct buyers in the trailing 30d.
pub fn ranked(store: &InsiderStore, limit: usize, now: DateTime<Utc>) -> Vec<InsiderCluster> {
    // Group all buys by symbol.
    let cutoff = now - Duration::days(WINDOW_DAYS);
    let snapshot = store.latest(10_000);
    let mut by_symbol: HashMap<String, Vec<InsiderEvent>> = HashMap::new();
    for e in snapshot {
        if e.observed_at < cutoff {
            continue;
        }
        if !matches!(e.kind, TxKind::Buy) {
            continue;
        }
        by_symbol.entry(e.symbol.clone()).or_default().push(e);
    }
    let mut rows: Vec<InsiderCluster> = by_symbol
        .into_iter()
        .filter_map(|(sym, buys)| cluster_from_buys(&sym, &buys, now))
        .collect();
    rows.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(limit);
    rows
}

pub fn for_symbol(
    store: &InsiderStore,
    symbol: &str,
    now: DateTime<Utc>,
) -> Option<InsiderCluster> {
    let buys = store.latest_for(symbol, 1_000);
    cluster_from_buys(symbol, &buys, now)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buy(
        sym: &str,
        insider: &str,
        is_officer: bool,
        is_director: bool,
        is_10p: bool,
        dollars: f64,
        days_ago: f64,
        now: DateTime<Utc>,
    ) -> InsiderEvent {
        InsiderEvent {
            symbol: sym.into(),
            insider_name: insider.into(),
            is_officer,
            is_director,
            is_ten_percent_owner: is_10p,
            officer_title: if is_officer { Some("CEO".into()) } else { None },
            transaction_code: "P".into(),
            kind: TxKind::Buy,
            shares: 1000.0,
            price_per_share: dollars / 1000.0,
            dollar_value: dollars,
            transaction_date: None,
            filing_link: None,
            observed_at: now - chrono::Duration::seconds((days_ago * 86400.0) as i64),
        }
    }

    fn sell(
        sym: &str,
        insider: &str,
        dollars: f64,
        days_ago: f64,
        now: DateTime<Utc>,
    ) -> InsiderEvent {
        InsiderEvent {
            symbol: sym.into(),
            insider_name: insider.into(),
            is_officer: false,
            is_director: false,
            is_ten_percent_owner: false,
            officer_title: None,
            transaction_code: "S".into(),
            kind: TxKind::Sell,
            shares: 1000.0,
            price_per_share: dollars / 1000.0,
            dollar_value: dollars,
            transaction_date: None,
            filing_link: None,
            observed_at: now - chrono::Duration::seconds((days_ago * 86400.0) as i64),
        }
    }

    #[test]
    fn recency_weight_half_life_matches_documented_constant() {
        let now = Utc::now();
        // 0 days ago → 1.0
        assert!((recency_weight(now, now) - 1.0).abs() < 1e-9);
        // RECENCY_HALF_LIFE_DAYS ago → 0.5
        let half = now - chrono::Duration::seconds((RECENCY_HALF_LIFE_DAYS * 86400.0) as i64);
        assert!((recency_weight(now, half) - 0.5).abs() < 1e-9);
        // Future event → 1.0
        let future = now + chrono::Duration::days(1);
        assert_eq!(recency_weight(now, future), 1.0);
    }

    #[test]
    fn cluster_requires_min_distinct_insiders() {
        let now = Utc::now();
        let one_insider = vec![
            buy("AAA", "Alice", true, false, false, 500_000.0, 1.0, now),
            buy("AAA", "Alice", true, false, false, 500_000.0, 2.0, now),
        ];
        assert!(cluster_from_buys("AAA", &one_insider, now).is_none());
    }

    #[test]
    fn cluster_emits_on_two_distinct_insiders() {
        let now = Utc::now();
        let buys = vec![
            buy("AAA", "Alice", true, false, false, 500_000.0, 1.0, now),
            buy("AAA", "Bob", false, true, false, 300_000.0, 2.0, now),
        ];
        let c = cluster_from_buys("AAA", &buys, now).expect("should emit");
        assert_eq!(c.insider_count, 2);
        assert_eq!(c.officer_count, 1);
        assert_eq!(c.director_count, 1);
        assert_eq!(c.ten_pct_owner_count, 0);
        assert_eq!(c.buy_count, 2);
        assert_eq!(c.total_dollars, 800_000.0);
        assert_eq!(c.insiders, vec!["Alice".to_string(), "Bob".to_string()]);
    }

    #[test]
    fn cluster_drops_buys_past_window() {
        let now = Utc::now();
        let buys = vec![
            buy("BBB", "Alice", true, false, false, 100_000.0, 5.0, now),
            buy("BBB", "Bob", false, true, false, 100_000.0, 7.0, now),
            // Outside the 30d window.
            buy("BBB", "Charlie", false, false, true, 1_000_000.0, 45.0, now),
        ];
        let c = cluster_from_buys("BBB", &buys, now).expect("should emit");
        assert_eq!(c.insider_count, 2);
        assert!(!c.insiders.contains(&"Charlie".to_string()));
        assert_eq!(c.ten_pct_owner_count, 0);
    }

    #[test]
    fn cluster_excludes_sells_from_buy_count() {
        let now = Utc::now();
        let mixed = vec![
            buy("CCC", "Alice", true, false, false, 200_000.0, 2.0, now),
            buy("CCC", "Bob", false, true, false, 150_000.0, 3.0, now),
            sell("CCC", "Charlie", 5_000_000.0, 1.0, now),
        ];
        let c = cluster_from_buys("CCC", &mixed, now).expect("should emit");
        assert_eq!(c.buy_count, 2);
        assert!(!c.insiders.contains(&"Charlie".to_string()));
    }

    #[test]
    fn officer_weight_dominates_role_mix() {
        let now = Utc::now();
        // Two clusters with 2 distinct insiders each. AAA's both
        // officers; BBB's both directors. AAA must score higher.
        let aaa = vec![
            buy("AAA", "A1", true, false, false, 100_000.0, 1.0, now),
            buy("AAA", "A2", true, false, false, 100_000.0, 1.0, now),
        ];
        let bbb = vec![
            buy("BBB", "B1", false, true, false, 100_000.0, 1.0, now),
            buy("BBB", "B2", false, true, false, 100_000.0, 1.0, now),
        ];
        let a = cluster_from_buys("AAA", &aaa, now).unwrap();
        let b = cluster_from_buys("BBB", &bbb, now).unwrap();
        assert!(
            a.score > b.score,
            "officer cluster {} ≤ director {}",
            a.score,
            b.score
        );
    }

    #[test]
    fn recency_weighted_dollars_favours_recent_buys() {
        let now = Utc::now();
        // Same dollar total in both, but FRESH bought today and STALE
        // bought 21 days ago (3 half-lives). Fresh ratio = 1.0, stale ≈ 0.125.
        let fresh = vec![
            buy("FRESH", "A", true, false, false, 1_000_000.0, 0.0, now),
            buy("FRESH", "B", false, true, false, 0.5e6, 0.0, now),
        ];
        let stale = vec![
            buy("STALE", "C", true, false, false, 1_000_000.0, 21.0, now),
            buy("STALE", "D", false, true, false, 0.5e6, 21.0, now),
        ];
        let f = cluster_from_buys("FRESH", &fresh, now).unwrap();
        let s = cluster_from_buys("STALE", &stale, now).unwrap();
        assert!(f.recency_weighted_dollars > s.recency_weighted_dollars);
        assert!(f.score > s.score);
    }

    #[test]
    fn empty_window_returns_none() {
        let now = Utc::now();
        let only_old = vec![
            buy("OLD", "A", true, false, false, 1_000_000.0, 60.0, now),
            buy("OLD", "B", true, false, false, 1_000_000.0, 90.0, now),
        ];
        assert!(cluster_from_buys("OLD", &only_old, now).is_none());
    }

    #[test]
    fn earliest_latest_bracket_the_cluster() {
        let now = Utc::now();
        let buys = vec![
            buy("XYZ", "A", true, false, false, 1.0, 10.0, now),
            buy("XYZ", "B", false, true, false, 1.0, 3.0, now),
            buy("XYZ", "C", true, false, false, 1.0, 7.0, now),
        ];
        let c = cluster_from_buys("XYZ", &buys, now).expect("should emit");
        // Earliest = 10 days ago (oldest), latest = 3 days ago.
        let secs_diff = (c.latest_buy - c.earliest_buy).num_seconds();
        assert!(secs_diff > 0, "latest must be after earliest");
        // Approximately 7 days = 604_800 seconds.
        assert!(
            (secs_diff - 604_800).abs() < 60,
            "expected ~7 day spread, got {} seconds",
            secs_diff
        );
    }

    #[test]
    fn insiders_list_sorted_alphabetically() {
        let now = Utc::now();
        let buys = vec![
            buy("S", "Charlie", true, false, false, 1.0, 1.0, now),
            buy("S", "Alice", true, false, false, 1.0, 1.0, now),
            buy("S", "Bob", true, false, false, 1.0, 1.0, now),
        ];
        let c = cluster_from_buys("S", &buys, now).unwrap();
        assert_eq!(c.insiders, vec!["Alice", "Bob", "Charlie"]);
    }
}
