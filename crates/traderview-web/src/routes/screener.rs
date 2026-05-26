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
        .route("/screener/run",  get(run))
        .route("/screener/top",  get(top))
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
fn default_days() -> i64 { 365 }
fn default_limit() -> usize { 50 }

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

async fn collect_universe(pool: &PgPool, user_id: Uuid, watchlist_id: Option<Uuid>) -> anyhow::Result<Vec<String>> {
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
    let bars = traderview_db::prices::get_bars(pool, symbol, BarInterval::D1, from, to).await.ok()?;
    if bars.is_empty() { return None; }
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

async fn run(State(s): State<AppState>, user: AuthUser, Query(q): Query<RunQ>)
    -> Result<Json<ScreenerResult>, ApiError>
{
    let universe = collect_universe(&s.pool, user.id, q.watchlist_id)
        .await.map_err(ApiError::Internal)?;
    let universe_size = universe.len();

    let mut hits = Vec::new();
    for sym in &universe {
        if let Some(r) = score_symbol(&s.pool, sym, q.days).await {
            if q.min_score.map_or(true, |m| r.score >= m)
               && q.max_score.map_or(true, |m| r.score <= m)
               && q.summary.as_deref().map_or(true, |w| w == r.summary)
            {
                hits.push(project(r));
            }
        }
        if hits.len() >= q.limit * 3 { break; } // bound work
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits.truncate(q.limit);
    Ok(Json(ScreenerResult { universe_size, hits }))
}

#[derive(Deserialize)]
struct TopQ {
    #[serde(default = "default_side")]
    side: String, // "buy" | "sell"
    watchlist_id: Option<Uuid>,
    #[serde(default = "default_top")]
    limit: usize,
}
fn default_side() -> String { "buy".into() }
fn default_top() -> usize { 25 }

async fn top(State(s): State<AppState>, user: AuthUser, Query(q): Query<TopQ>)
    -> Result<Json<ScreenerResult>, ApiError>
{
    let universe = collect_universe(&s.pool, user.id, q.watchlist_id)
        .await.map_err(ApiError::Internal)?;
    let universe_size = universe.len();
    let mut hits = Vec::new();
    for sym in &universe {
        if let Some(r) = score_symbol(&s.pool, sym, 365).await {
            hits.push(project(r));
        }
    }
    if q.side == "sell" {
        hits.sort_by(|a, b| a.score.cmp(&b.score));
    } else {
        hits.sort_by(|a, b| b.score.cmp(&a.score));
    }
    hits.truncate(q.limit);
    Ok(Json(ScreenerResult { universe_size, hits }))
}
