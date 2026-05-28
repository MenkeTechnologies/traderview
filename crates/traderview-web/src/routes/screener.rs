//! Screener — runs technical signals across the user's watchlist universe
//! and returns ranked results. Also exposes top-buy / top-sell shortcuts.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::BTreeSet;
use traderview_core::signals::{analyze, SignalReport};
use traderview_core::BarInterval;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/screener/run", get(run))
        .route("/screener/top", get(top))
}

#[derive(Deserialize)]
struct RunQ {
    /// Optional watchlist UUID. If absent, scan ALL the user's watchlist symbols.
    watchlist_id: Option<Uuid>,
    #[serde(default)]
    min_score: Option<i32>,
    #[serde(default)]
    max_score: Option<i32>,
    #[serde(default)]
    summary: Option<String>, // "buy" | "sell" | "hold"
    #[serde(default = "default_days")]
    days: i64,
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_days() -> i64 {
    365
}
fn default_limit() -> usize {
    50
}

#[derive(Serialize)]
struct ScreenerHit {
    symbol: String,
    score: i32,
    summary: &'static str,
    last_close: f64,
    rsi14: Option<f64>,
    sma50: Option<f64>,
    sma200: Option<f64>,
    macd_hist: Option<f64>,
    signal_count: usize,
}

#[derive(Serialize)]
struct ScreenerResult {
    universe_size: usize,
    hits: Vec<ScreenerHit>,
}

async fn collect_universe(
    pool: &PgPool,
    user_id: Uuid,
    watchlist_id: Option<Uuid>,
) -> anyhow::Result<Vec<String>> {
    if let Some(wid) = watchlist_id {
        if !traderview_db::watchlists::ensure_owner(pool, user_id, wid).await? {
            anyhow::bail!("forbidden");
        }
        return traderview_db::watchlists::symbols(pool, wid).await;
    }
    let lists = traderview_db::watchlists::list(pool, user_id).await?;
    let mut all: BTreeSet<String> = BTreeSet::new();
    for w in lists {
        for s in traderview_db::watchlists::symbols(pool, w.id).await? {
            all.insert(s);
        }
    }
    Ok(all.into_iter().collect())
}

async fn score_symbol(pool: &PgPool, symbol: &str, days: i64) -> Option<SignalReport> {
    let to = Utc::now();
    let from = to - Duration::days(days);
    let bars = traderview_db::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .ok()?;
    if bars.is_empty() {
        return None;
    }
    Some(analyze(symbol, &bars))
}

fn project(r: SignalReport) -> ScreenerHit {
    ScreenerHit {
        symbol: r.symbol,
        score: r.score,
        summary: r.summary,
        last_close: r.last_close,
        rsi14: r.indicators.rsi14,
        sma50: r.indicators.sma50,
        sma200: r.indicators.sma200,
        macd_hist: r.indicators.macd_hist,
        signal_count: r.signals.len(),
    }
}

async fn run(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<RunQ>,
) -> Result<Json<ScreenerResult>, ApiError> {
    let universe = collect_universe(&s.pool, user.id, q.watchlist_id)
        .await
        .map_err(ApiError::Internal)?;
    let universe_size = universe.len();

    let mut hits = Vec::new();
    for sym in &universe {
        if let Some(r) = score_symbol(&s.pool, sym, q.days).await {
            if q.min_score.is_none_or(|m| r.score >= m)
                && q.max_score.is_none_or(|m| r.score <= m)
                && q.summary.as_deref().is_none_or(|w| w == r.summary)
            {
                hits.push(project(r));
            }
        }
        if hits.len() >= q.limit * 3 {
            break;
        } // bound work
    }
    hits.sort_by_key(|a| std::cmp::Reverse(a.score));
    hits.truncate(q.limit);
    Ok(Json(ScreenerResult {
        universe_size,
        hits,
    }))
}

#[derive(Deserialize)]
struct TopQ {
    #[serde(default = "default_side")]
    side: String, // "buy" | "sell"
    watchlist_id: Option<Uuid>,
    #[serde(default = "default_top")]
    limit: usize,
}
fn default_side() -> String {
    "buy".into()
}
fn default_top() -> usize {
    25
}

async fn top(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<TopQ>,
) -> Result<Json<ScreenerResult>, ApiError> {
    let universe = collect_universe(&s.pool, user.id, q.watchlist_id)
        .await
        .map_err(ApiError::Internal)?;
    let universe_size = universe.len();
    let mut hits = Vec::new();
    for sym in &universe {
        if let Some(r) = score_symbol(&s.pool, sym, 365).await {
            hits.push(project(r));
        }
    }
    if q.side == "sell" {
        hits.sort_by_key(|a| a.score);
    } else {
        hits.sort_by_key(|a| std::cmp::Reverse(a.score));
    }
    hits.truncate(q.limit);
    Ok(Json(ScreenerResult {
        universe_size,
        hits,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use traderview_core::signals::IndicatorSnapshot;

    fn make_report(score: i32, summary: &'static str) -> SignalReport {
        SignalReport {
            symbol: "AAPL".into(),
            score,
            summary,
            last_close: 150.0,
            indicators: IndicatorSnapshot {
                sma20: Some(149.0),
                sma50: Some(148.0),
                sma200: Some(140.0),
                ema12: Some(149.5),
                ema26: Some(148.5),
                macd_line: Some(1.0),
                macd_signal: Some(0.8),
                macd_hist: Some(0.2),
                rsi14: Some(55.0),
                adx14: Some(25.0),
                plus_di: Some(20.0),
                minus_di: Some(15.0),
                stoch_k: Some(60.0),
                stoch_d: Some(55.0),
                bb_upper: Some(155.0),
                bb_middle: Some(150.0),
                bb_lower: Some(145.0),
            },
            signals: vec![],
            pivots: None,
        }
    }

    // ── project: SignalReport → ScreenerHit field plumbing ────────────────

    #[test]
    fn project_copies_score_summary_and_close() {
        let h = project(make_report(7, "buy"));
        assert_eq!(h.score, 7);
        assert_eq!(h.summary, "buy");
        assert_eq!(h.last_close, 150.0);
        assert_eq!(h.symbol, "AAPL");
    }

    #[test]
    fn project_lifts_individual_indicator_fields() {
        // The frontend reads rsi14/sma50/sma200/macd_hist directly off the hit;
        // wiring any of these to the wrong source field gives misleading
        // screener results.
        let h = project(make_report(5, "hold"));
        assert_eq!(h.rsi14, Some(55.0));
        assert_eq!(h.sma50, Some(148.0));
        assert_eq!(h.sma200, Some(140.0));
        assert_eq!(h.macd_hist, Some(0.2));
    }

    #[test]
    fn project_records_signal_count_not_signal_payload() {
        // We deliberately don't ship every individual signal into the screener
        // response — the frontend just wants a count for the badge.
        let h = project(make_report(3, "buy"));
        assert_eq!(h.signal_count, 0);
    }

    // ── default_* query parameter defaults ────────────────────────────────

    #[test]
    fn default_days_matches_365_one_year_window() {
        // The screener defaults to a one-year lookback; if this slips to 30/90
        // every indicator that needs sma200 silently goes None.
        assert_eq!(default_days(), 365);
    }

    #[test]
    fn default_limit_matches_50_screener_result_cap() {
        assert_eq!(default_limit(), 50);
    }

    #[test]
    fn default_side_matches_buy() {
        // /screener/top defaults to the buy side — the sell sort branch is
        // opt-in via ?side=sell.
        assert_eq!(default_side(), "buy");
    }

    #[test]
    fn default_top_matches_25_top_list_cap() {
        assert_eq!(default_top(), 25);
    }
}
