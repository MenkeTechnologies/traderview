//! Hard-to-borrow / squeeze-pressure ranker.
//!
//! Real borrow-fee data (IB SLB / Markit Securities Finance) sits behind
//! paid feeds. Without those, "hard to borrow" still has a defensible
//! free-data proxy: the standard squeeze stack — `short_pct_float`,
//! `days_to_cover` (short_ratio), month-over-month change in shares
//! short, and float size. Each of those is monthly-updated from FINRA
//! and exposed by Finnhub's `/stock/short-interest` + `/stock/metric`
//! endpoints, which the codebase already fetches via
//! [`crate::short_interest::finnhub_short_stats`].
//!
//! The composite score blends the four signals:
//!
//!   score =  0.30 · normalise(short_pct_float)
//!          + 0.20 · normalise(days_to_cover)
//!          + 0.30 · normalise(change_pct)
//!          + 0.20 · normalise(float_inverse)
//!
//! where each `normalise(x)` clamps to a known reasonable upper bound
//! so a single extreme value doesn't dominate. The output is a
//! comparable 0–100 score; the top entries surface as candidate HTB
//! names.
//!
//! Cadence: 30-minute background refresh of the top-N most-active
//! symbols from `LiveTickStore`. FINRA short interest reports twice
//! monthly — anything more frequent is wasted budget.
//!
//! Limitations:
//!   * No real borrow-fee number is available without IB / Markit. If
//!     the user later wires a paid feed, swap the proxy out for the
//!     real `borrow_fee_bps` field.
//!   * Finnhub free tier rate-limits to 60 req/min; we pace requests
//!     at 600 ms inside a refresh round.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;

use crate::live_ticks::LiveTickStore;
use crate::short_interest;

/// How many of the most-active symbols to rank each round.
const TOP_N: usize = 30;
/// Wall-clock seconds between full refreshes.
const REFRESH_SECS: u64 = 30 * 60;
/// Pace between Finnhub requests inside a refresh round.
const PACE_MS: u64 = 600;

#[derive(Debug, Clone, Serialize)]
pub struct HtbScore {
    pub symbol: String,
    pub short_pct_float: Option<f64>,
    pub days_to_cover: Option<f64>,
    pub change_pct: Option<f64>,
    pub float: Option<f64>,
    /// Composite 0-100 score from `compute_score` — higher = more
    /// squeeze-prone / harder to borrow (by proxy).
    pub score: f64,
    /// Per-component sub-scores for UI debugging.
    pub components: HtbComponents,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HtbComponents {
    pub short_pct_float_score: f64,
    pub days_to_cover_score: f64,
    pub change_pct_score: f64,
    pub float_inverse_score: f64,
}

#[derive(Clone)]
pub struct HtbStore {
    rows: Arc<DashMap<String, HtbScore>>,
}

impl HtbStore {
    pub fn new() -> Self {
        Self {
            rows: Arc::new(DashMap::new()),
        }
    }

    pub fn upsert(&self, score: HtbScore) {
        self.rows.insert(score.symbol.clone(), score);
    }

    /// Return the top-N by score, ranked descending.
    pub fn ranked(&self, limit: usize) -> Vec<HtbScore> {
        let mut all: Vec<HtbScore> = self.rows.iter().map(|e| e.value().clone()).collect();
        all.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all.truncate(limit);
        all
    }

    /// Read one symbol's most-recent score.
    pub fn get(&self, symbol: &str) -> Option<HtbScore> {
        self.rows
            .get(&symbol.to_ascii_uppercase())
            .map(|e| e.value().clone())
    }
}

impl Default for HtbStore {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Pure scoring ──────────────────────────────────────────────────────────

/// Clamp `x` into `[0, cap]` then scale to 0-100. NaN / negative
/// inputs return 0. Used to keep one extreme datum from dominating
/// the composite score.
fn norm(x: Option<f64>, cap: f64) -> f64 {
    match x {
        Some(v) if v.is_finite() && v > 0.0 => (v.min(cap) / cap * 100.0).clamp(0.0, 100.0),
        _ => 0.0,
    }
}

/// Inverse normaliser — lower float means harder to borrow, so a
/// smaller `float` value should produce a *larger* score. We anchor
/// against a 100M-share "very-small-cap" floor: anything under 10M
/// shares maxes the component; anything over 100M floors it at 0.
fn norm_inverse_float(float: Option<f64>) -> f64 {
    match float {
        Some(v) if v.is_finite() && v > 0.0 => {
            if v <= 10_000_000.0 {
                100.0
            } else if v >= 100_000_000.0 {
                0.0
            } else {
                // Linear between (10M, 100) and (100M, 0).
                let pct = (100_000_000.0 - v) / (100_000_000.0 - 10_000_000.0);
                (pct * 100.0).clamp(0.0, 100.0)
            }
        }
        _ => 0.0,
    }
}

/// Compute the composite HTB-pressure score for one `ShortStats` row.
/// Caps: short_pct_float 50% (0-0.5 in 0-1 input), days_to_cover 30,
/// change_pct 100% (i.e. shares short doubled MoM).
pub fn compute_score(stats: &short_interest::ShortStats) -> HtbScore {
    let short_pct_score = norm(stats.short_pct_float.map(|p| p * 100.0), 50.0);
    let dtc_score = norm(stats.short_ratio, 30.0);
    let change_score = norm(stats.change_pct, 100.0);
    let float_score = norm_inverse_float(stats.float);

    let score =
        0.30 * short_pct_score + 0.20 * dtc_score + 0.30 * change_score + 0.20 * float_score;

    HtbScore {
        symbol: stats.symbol.clone(),
        short_pct_float: stats.short_pct_float,
        days_to_cover: stats.short_ratio,
        change_pct: stats.change_pct,
        float: stats.float,
        score: (score * 100.0).round() / 100.0,
        components: HtbComponents {
            short_pct_float_score: short_pct_score,
            days_to_cover_score: dtc_score,
            change_pct_score: change_score,
            float_inverse_score: float_score,
        },
        fetched_at: stats.fetched_at,
    }
}

// ─── Background refresh ────────────────────────────────────────────────────

fn is_crypto_like(sym: &str) -> bool {
    let s = sym.to_ascii_uppercase();
    if s.contains('/') {
        return true;
    }
    const BASES: &[&str] = &[
        "BTC", "ETH", "LTC", "BCH", "DOGE", "SOL", "AVAX", "MATIC", "ADA", "DOT", "XRP", "LINK",
        "UNI", "SHIB", "AAVE", "ALGO", "ATOM", "BAT", "COMP", "CRV", "GRT", "MKR", "PAXG", "SUSHI",
        "TRX", "XLM", "XTZ", "YFI", "ZRX",
    ];
    if s.ends_with("USD") && s.len() > 3 {
        let base = &s[..s.len() - 3];
        if BASES.contains(&base) {
            return true;
        }
    }
    false
}

fn top_n_active(ticks: &LiveTickStore, n: usize) -> Vec<String> {
    let mut rows: Vec<(String, u64)> = ticks
        .snapshot()
        .into_iter()
        .filter(|s| !is_crypto_like(&s.symbol))
        .map(|s| (s.symbol, s.trade_count))
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.into_iter().take(n).map(|(s, _)| s).collect()
}

pub fn spawn_refresher(store: HtbStore, ticks: LiveTickStore) {
    tokio::spawn(async move {
        // First refresh delays a bit so live_ticks has a chance to
        // populate its symbol set.
        tokio::time::sleep(Duration::from_secs(30)).await;
        let mut interval = tokio::time::interval(Duration::from_secs(REFRESH_SECS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let symbols = top_n_active(&ticks, TOP_N);
            if symbols.is_empty() {
                continue;
            }
            for sym in symbols {
                match short_interest::finnhub_short_stats(&sym).await {
                    Ok(stats) => {
                        let s = compute_score(&stats);
                        if s.score > 0.0 {
                            store.upsert(s);
                        }
                    }
                    Err(e) => {
                        tracing::debug!(?e, symbol = %sym, "htb_ranker finnhub fetch failed");
                    }
                }
                tokio::time::sleep(Duration::from_millis(PACE_MS)).await;
            }
        }
    });
}

pub fn global() -> HtbStore {
    static STORE: once_cell::sync::OnceCell<HtbStore> = once_cell::sync::OnceCell::new();
    STORE
        .get_or_init(|| {
            let s = HtbStore::new();
            spawn_refresher(s.clone(), crate::live_ticks::global());
            s
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::short_interest::ShortStats;

    fn stats(
        sym: &str,
        pct_float: Option<f64>,
        dtc: Option<f64>,
        change: Option<f64>,
        float: Option<f64>,
    ) -> ShortStats {
        ShortStats {
            symbol: sym.into(),
            shares_short: None,
            shares_short_prior: None,
            short_ratio: dtc,
            short_pct_float: pct_float,
            short_pct_outstanding: None,
            float,
            change_pct: change,
            fetched_at: Utc::now(),
        }
    }

    #[test]
    fn norm_clamps_and_scales() {
        assert_eq!(norm(Some(0.0), 50.0), 0.0);
        assert_eq!(norm(Some(25.0), 50.0), 50.0);
        assert_eq!(norm(Some(50.0), 50.0), 100.0);
        assert_eq!(norm(Some(75.0), 50.0), 100.0); // capped
        assert_eq!(norm(None, 50.0), 0.0);
        assert_eq!(norm(Some(f64::NAN), 50.0), 0.0);
        assert_eq!(norm(Some(-5.0), 50.0), 0.0);
    }

    #[test]
    fn norm_inverse_float_inverts_size() {
        assert_eq!(norm_inverse_float(Some(5_000_000.0)), 100.0); // tiny float
        assert_eq!(norm_inverse_float(Some(100_000_000.0)), 0.0); // ceiling
        assert_eq!(norm_inverse_float(Some(200_000_000.0)), 0.0); // beyond ceiling
        assert_eq!(norm_inverse_float(None), 0.0);
        // 55M → halfway between 10M and 100M → ~50.
        let mid = norm_inverse_float(Some(55_000_000.0));
        assert!((mid - 50.0).abs() < 1.0);
    }

    #[test]
    fn compute_score_blends_components() {
        // Max-of-each → score = 30 + 20 + 30 + 20 = 100.
        let s = stats(
            "AMC",
            Some(0.50),        // → 100
            Some(30.0),        // → 100
            Some(100.0),       // → 100
            Some(5_000_000.0), // → 100
        );
        let h = compute_score(&s);
        assert!((h.score - 100.0).abs() < 1.0);
        assert!((h.components.short_pct_float_score - 100.0).abs() < 1e-9);
        assert!((h.components.days_to_cover_score - 100.0).abs() < 1e-9);
    }

    #[test]
    fn compute_score_zero_when_all_missing() {
        let s = stats("NONE", None, None, None, None);
        let h = compute_score(&s);
        assert_eq!(h.score, 0.0);
    }

    #[test]
    fn store_ranks_highest_score_first() {
        let store = HtbStore::new();
        store.upsert(compute_score(&stats(
            "LOW",
            Some(0.05),
            Some(2.0),
            Some(5.0),
            Some(500_000_000.0),
        )));
        store.upsert(compute_score(&stats(
            "HOT",
            Some(0.40),
            Some(20.0),
            Some(80.0),
            Some(8_000_000.0),
        )));
        store.upsert(compute_score(&stats(
            "MID",
            Some(0.20),
            Some(8.0),
            Some(30.0),
            Some(50_000_000.0),
        )));
        let ranked = store.ranked(5);
        assert_eq!(ranked[0].symbol, "HOT");
        assert_eq!(ranked[1].symbol, "MID");
        assert_eq!(ranked[2].symbol, "LOW");
    }

    #[test]
    fn store_get_returns_specific_symbol() {
        let store = HtbStore::new();
        store.upsert(compute_score(&stats(
            "AAPL",
            Some(0.10),
            Some(3.0),
            Some(15.0),
            Some(200_000_000.0),
        )));
        let s = store.get("AAPL").expect("present");
        assert_eq!(s.symbol, "AAPL");
        assert!(store.get("MSFT").is_none());
    }

    #[test]
    fn is_crypto_like_filters_crypto() {
        assert!(is_crypto_like("BTC/USD"));
        assert!(is_crypto_like("ETHUSD"));
        assert!(!is_crypto_like("AAPL"));
    }
}
