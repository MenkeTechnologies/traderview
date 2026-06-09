//! Confluence → paper-trade autopilot.
//!
//! Subscribes to the confluence dashboard's ranked output and, when a
//! symbol crosses a per-user score threshold AND has at least N distinct
//! independent scanners hitting, automatically submits a paper-market
//! buy for `notional_usd / quote` shares against the user's default
//! paper account.
//!
//! Why paper, not live: confluence has never been validated on out-of-
//! sample data. Paper-trade first; if Sharpe ≥ 1 over a meaningful
//! sample, promote to live with the same wiring.
//!
//! Pipeline:
//!   1. `select_candidates(rows, &cfg, &cooldown, &open_position_count)`
//!      pure-compute → `Vec<Candidate>` filtered by score, distinct-sources,
//!      cooldown, open-position cap. Fully unit-tested.
//!   2. `run_once(pool, user_id)` async wrapper: loads config, pulls
//!      confluence::global().ranked(), looks up cooldown via the
//!      `confluence_autotrade_log` table, calls `paper::ensure_default`,
//!      submits the selected candidates as paper-market orders, writes
//!      one log row per attempt (submitted or skipped).
//!
//! The actual broadcast subscriber lives in the route handler — for now,
//! `POST /confluence/autotrade/run-once` is user-triggered. A cron tick
//! lands in a follow-up commit so the user can see the wiring before it
//! starts firing autonomously.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::confluence::ConfluenceRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutotradeConfig {
    pub user_id: Uuid,
    pub enabled: bool,
    pub min_score: f64,
    pub min_distinct_sources: i32,
    pub notional_usd: f64,
    pub cooldown_minutes: i32,
    pub max_open_positions: i32,
    pub updated_at: DateTime<Utc>,
}

impl AutotradeConfig {
    pub fn default_for(user_id: Uuid) -> Self {
        Self {
            user_id,
            enabled: false,
            min_score: 8.0,
            min_distinct_sources: 3,
            notional_usd: 1000.0,
            cooldown_minutes: 240,
            max_open_positions: 10,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub symbol: String,
    pub score: f64,
    pub distinct_sources: i32,
    pub notional_usd: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    BelowScore,
    InsufficientSources,
    Cooldown,
    CapReached,
}

#[derive(Debug, Clone, Serialize)]
pub struct Decision {
    pub symbol: String,
    pub score: f64,
    pub distinct_sources: i32,
    pub action: DecisionAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAction {
    Submit,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutotradeLogRow {
    pub id: i64,
    pub user_id: Uuid,
    pub symbol: String,
    pub score: f64,
    pub distinct_sources: i32,
    pub notional_usd: f64,
    pub action: String,
    pub paper_order_id: Option<Uuid>,
    pub reason: Option<String>,
    pub fired_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunOnceResult {
    pub config: AutotradeConfig,
    pub candidates_considered: usize,
    pub submitted: Vec<AutotradeLogRow>,
    pub skipped: Vec<AutotradeLogRow>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Filters the confluence ranking into the list of symbols that *should*
/// actually fire an order this tick. Three gates, evaluated in order:
///   1. score gate — score ≥ cfg.min_score
///   2. diversity gate — distinct_sources ≥ cfg.min_distinct_sources
///   3. cooldown gate — symbol was NOT autotraded inside the trailing
///      cfg.cooldown_minutes window (prevents the same hot symbol from
///      re-buying every poll)
///   4. cap gate — already-open autotrade-tagged positions, plus the
///      cumulative count of fires this tick, must stay ≤ max_open
///
/// All four gates are evaluated against the per-tick `now` so tests can
/// pin time. Skipped candidates come back in the second return value so
/// the caller can log *why* something didn't fire — without that, the
/// user has no way to debug why the autopilot is silent.
pub fn select_candidates(
    rows: &[ConfluenceRow],
    cfg: &AutotradeConfig,
    last_fire_by_symbol: &dyn Fn(&str) -> Option<DateTime<Utc>>,
    open_position_count: i32,
    now: DateTime<Utc>,
) -> (Vec<Candidate>, Vec<(String, f64, i32, SkipReason)>) {
    let cap = cfg.max_open_positions;
    let cooldown = Duration::minutes(cfg.cooldown_minutes as i64);

    let mut accepted: Vec<Candidate> = Vec::new();
    let mut skipped: Vec<(String, f64, i32, SkipReason)> = Vec::new();
    let mut budget_used = open_position_count;

    for row in rows {
        let sym = row.symbol.clone();
        let score = row.score;
        let distinct = row.distinct_sources as i32;

        if score < cfg.min_score {
            skipped.push((sym, score, distinct, SkipReason::BelowScore));
            continue;
        }
        if distinct < cfg.min_distinct_sources {
            skipped.push((sym, score, distinct, SkipReason::InsufficientSources));
            continue;
        }
        if let Some(last) = last_fire_by_symbol(&sym) {
            if now - last < cooldown {
                skipped.push((sym, score, distinct, SkipReason::Cooldown));
                continue;
            }
        }
        if budget_used >= cap {
            skipped.push((sym, score, distinct, SkipReason::CapReached));
            continue;
        }

        accepted.push(Candidate {
            symbol: sym,
            score,
            distinct_sources: distinct,
            notional_usd: cfg.notional_usd,
        });
        budget_used += 1;
    }

    (accepted, skipped)
}

/// Converts a per-symbol `notional_usd` budget into a share quantity at
/// the supplied last-trade price. Floor to whole shares; reject below 1.
pub fn shares_for_notional(notional_usd: f64, last_price: f64) -> Option<Decimal> {
    if !(notional_usd > 0.0 && last_price > 0.0) {
        return None;
    }
    let raw = (notional_usd / last_price).floor();
    if raw < 1.0 {
        return None;
    }
    Decimal::from_f64(raw)
}

fn reason_label(r: SkipReason) -> &'static str {
    match r {
        SkipReason::BelowScore => "skipped_score",
        SkipReason::InsufficientSources => "skipped_sources",
        SkipReason::Cooldown => "skipped_cooldown",
        SkipReason::CapReached => "skipped_cap",
    }
}

// ─── Repository ────────────────────────────────────────────────────────────

pub async fn get_config(pool: &PgPool, user_id: Uuid) -> anyhow::Result<AutotradeConfig> {
    let row: Option<(bool, f64, i32, f64, i32, i32, DateTime<Utc>)> = sqlx::query_as(
        "SELECT enabled, min_score, min_distinct_sources, notional_usd,
                cooldown_minutes, max_open_positions, updated_at
           FROM confluence_autotrade_config WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    match row {
        Some((enabled, min_score, src, notional, cd, max_open, updated)) => Ok(AutotradeConfig {
            user_id,
            enabled,
            min_score,
            min_distinct_sources: src,
            notional_usd: notional,
            cooldown_minutes: cd,
            max_open_positions: max_open,
            updated_at: updated,
        }),
        None => Ok(AutotradeConfig::default_for(user_id)),
    }
}

pub async fn upsert_config(
    pool: &PgPool,
    cfg: &AutotradeConfig,
) -> anyhow::Result<AutotradeConfig> {
    sqlx::query(
        "INSERT INTO confluence_autotrade_config
            (user_id, enabled, min_score, min_distinct_sources,
             notional_usd, cooldown_minutes, max_open_positions, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, now())
         ON CONFLICT (user_id) DO UPDATE SET
            enabled              = EXCLUDED.enabled,
            min_score            = EXCLUDED.min_score,
            min_distinct_sources = EXCLUDED.min_distinct_sources,
            notional_usd         = EXCLUDED.notional_usd,
            cooldown_minutes     = EXCLUDED.cooldown_minutes,
            max_open_positions   = EXCLUDED.max_open_positions,
            updated_at           = now()",
    )
    .bind(cfg.user_id)
    .bind(cfg.enabled)
    .bind(cfg.min_score)
    .bind(cfg.min_distinct_sources)
    .bind(cfg.notional_usd)
    .bind(cfg.cooldown_minutes)
    .bind(cfg.max_open_positions)
    .execute(pool)
    .await?;
    get_config(pool, cfg.user_id).await
}

pub async fn recent_log(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<AutotradeLogRow>> {
    let rows: Vec<(
        i64,
        Uuid,
        String,
        f64,
        i32,
        f64,
        String,
        Option<Uuid>,
        Option<String>,
        DateTime<Utc>,
    )> = sqlx::query_as(
        "SELECT id, user_id, symbol, score, distinct_sources, notional_usd,
                action, paper_order_id, reason, fired_at
           FROM confluence_autotrade_log
          WHERE user_id = $1
          ORDER BY fired_at DESC
          LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, u, sym, score, src, notional, action, oid, reason, fired)| AutotradeLogRow {
                id,
                user_id: u,
                symbol: sym,
                score,
                distinct_sources: src,
                notional_usd: notional,
                action,
                paper_order_id: oid,
                reason,
                fired_at: fired,
            },
        )
        .collect())
}

async fn last_fire_lookup(
    pool: &PgPool,
    user_id: Uuid,
    cooldown_minutes: i32,
) -> anyhow::Result<std::collections::HashMap<String, DateTime<Utc>>> {
    let since = Utc::now() - Duration::minutes(cooldown_minutes as i64);
    let rows: Vec<(String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT symbol, MAX(fired_at)
           FROM confluence_autotrade_log
          WHERE user_id = $1 AND fired_at >= $2 AND action = 'submitted'
          GROUP BY symbol",
    )
    .bind(user_id)
    .bind(since)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().collect())
}

async fn open_autotrade_positions(pool: &PgPool, account_id: Uuid) -> anyhow::Result<i64> {
    let (n,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM paper_positions WHERE paper_account_id = $1")
            .bind(account_id)
            .fetch_one(pool)
            .await?;
    Ok(n)
}

async fn write_log(
    pool: &PgPool,
    user_id: Uuid,
    cand: &Candidate,
    action: &str,
    paper_order_id: Option<Uuid>,
    reason: Option<&str>,
) -> anyhow::Result<AutotradeLogRow> {
    let (id, fired_at): (i64, DateTime<Utc>) = sqlx::query_as(
        "INSERT INTO confluence_autotrade_log
            (user_id, symbol, score, distinct_sources, notional_usd, action, paper_order_id, reason)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, fired_at",
    )
    .bind(user_id)
    .bind(&cand.symbol)
    .bind(cand.score)
    .bind(cand.distinct_sources)
    .bind(cand.notional_usd)
    .bind(action)
    .bind(paper_order_id)
    .bind(reason)
    .fetch_one(pool)
    .await?;
    Ok(AutotradeLogRow {
        id,
        user_id,
        symbol: cand.symbol.clone(),
        score: cand.score,
        distinct_sources: cand.distinct_sources,
        notional_usd: cand.notional_usd,
        action: action.into(),
        paper_order_id,
        reason: reason.map(|s| s.to_string()),
        fired_at,
    })
}

/// Single-shot pass: pull confluence::global().ranked(), filter, submit.
/// Returns a structured summary of what fired vs what got skipped.
/// When `enabled = false` returns the config with no submissions and no
/// skips — the caller's UI uses that to render the "disabled" state.
pub async fn run_once(pool: &PgPool, user_id: Uuid) -> anyhow::Result<RunOnceResult> {
    let cfg = get_config(pool, user_id).await?;
    if !cfg.enabled {
        return Ok(RunOnceResult {
            config: cfg,
            candidates_considered: 0,
            submitted: Vec::new(),
            skipped: Vec::new(),
        });
    }

    let rows =
        crate::confluence::global().ranked(Utc::now(), 200, cfg.min_distinct_sources as usize);
    let last_fire = last_fire_lookup(pool, user_id, cfg.cooldown_minutes).await?;
    let account = crate::paper::ensure_default(pool, user_id).await?;
    let open_n = open_autotrade_positions(pool, account.id).await? as i32;

    let now = Utc::now();
    let last_fire_fn = |sym: &str| -> Option<DateTime<Utc>> { last_fire.get(sym).copied() };
    let (accepted, skipped) = select_candidates(&rows, &cfg, &last_fire_fn, open_n, now);
    let candidates_considered = accepted.len() + skipped.len();

    let mut submitted_rows: Vec<AutotradeLogRow> = Vec::new();
    let mut skipped_rows: Vec<AutotradeLogRow> = Vec::new();
    for cand in &accepted {
        let quote = match crate::market_data::quote(pool, &cand.symbol).await {
            Ok(q) => q,
            Err(e) => {
                let row = write_log(
                    pool,
                    user_id,
                    cand,
                    "skipped_quote",
                    None,
                    Some(&format!("quote failed: {e}")),
                )
                .await?;
                skipped_rows.push(row);
                continue;
            }
        };
        let qty = match shares_for_notional(cand.notional_usd, quote.price) {
            Some(q) => q,
            None => {
                let row = write_log(
                    pool,
                    user_id,
                    cand,
                    "skipped_quote",
                    None,
                    Some(&format!(
                        "notional ${} at price ${} → < 1 share",
                        cand.notional_usd, quote.price
                    )),
                )
                .await?;
                skipped_rows.push(row);
                continue;
            }
        };
        let order = crate::paper::submit(
            pool,
            user_id,
            account.id,
            crate::paper::OrderRequest {
                symbol: cand.symbol.clone(),
                side: traderview_core::Side::Buy,
                qty,
                order_type: "market".into(),
                limit_price: None,
                stop_price: None,
            },
        )
        .await?;
        let row = write_log(
            pool,
            user_id,
            cand,
            "submitted",
            Some(order.id),
            Some(&format!(
                "score={:.2} sources={}",
                cand.score, cand.distinct_sources
            )),
        )
        .await?;
        submitted_rows.push(row);
    }
    for (sym, score, distinct, reason) in skipped {
        let cand = Candidate {
            symbol: sym,
            score,
            distinct_sources: distinct,
            notional_usd: cfg.notional_usd,
        };
        let row = write_log(pool, user_id, &cand, reason_label(reason), None, None).await?;
        skipped_rows.push(row);
    }

    Ok(RunOnceResult {
        config: cfg,
        candidates_considered,
        submitted: submitted_rows,
        skipped: skipped_rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::confluence::{ConfluenceEvent, ConfluenceRow, Source};
    use std::collections::HashMap;

    fn row(symbol: &str, score: f64, distinct: usize) -> ConfluenceRow {
        ConfluenceRow {
            symbol: symbol.into(),
            score,
            event_count: distinct.max(1),
            distinct_sources: distinct,
            events: Vec::<ConfluenceEvent>::new(),
            sources_hit: (0..distinct).map(|_| Source::AfterHours).collect(),
        }
    }

    fn cfg() -> AutotradeConfig {
        AutotradeConfig {
            user_id: Uuid::nil(),
            enabled: true,
            min_score: 8.0,
            min_distinct_sources: 3,
            notional_usd: 1000.0,
            cooldown_minutes: 240,
            max_open_positions: 5,
            updated_at: Utc::now(),
        }
    }

    fn no_cooldown() -> impl Fn(&str) -> Option<DateTime<Utc>> {
        |_: &str| -> Option<DateTime<Utc>> { None }
    }

    #[test]
    fn select_drops_rows_under_min_score() {
        let rows = vec![
            row("AAA", 5.0, 3), // below score
            row("BBB", 9.0, 3),
        ];
        let (acc, skip) = select_candidates(&rows, &cfg(), &no_cooldown(), 0, Utc::now());
        assert_eq!(acc.len(), 1);
        assert_eq!(acc[0].symbol, "BBB");
        assert_eq!(skip.len(), 1);
        assert_eq!(skip[0].3, SkipReason::BelowScore);
    }

    #[test]
    fn select_drops_rows_under_min_distinct_sources() {
        let rows = vec![row("CCC", 12.0, 2), row("DDD", 8.5, 3)];
        let (acc, skip) = select_candidates(&rows, &cfg(), &no_cooldown(), 0, Utc::now());
        assert_eq!(acc.len(), 1);
        assert_eq!(acc[0].symbol, "DDD");
        assert_eq!(skip[0].3, SkipReason::InsufficientSources);
    }

    #[test]
    fn select_respects_cooldown_window() {
        let now = Utc::now();
        let just_fired = now - Duration::minutes(30);
        let last_fire_map: HashMap<String, DateTime<Utc>> =
            [("EEE".to_string(), just_fired)].into_iter().collect();
        let lookup = move |sym: &str| last_fire_map.get(sym).copied();
        let rows = vec![row("EEE", 12.0, 4)];
        let (acc, skip) = select_candidates(&rows, &cfg(), &lookup, 0, now);
        assert_eq!(acc.len(), 0);
        assert_eq!(skip.len(), 1);
        assert_eq!(skip[0].3, SkipReason::Cooldown);
    }

    #[test]
    fn select_allows_through_after_cooldown_expires() {
        let now = Utc::now();
        let long_ago = now - Duration::hours(8); // cfg cooldown is 240 min = 4h
        let last_fire_map: HashMap<String, DateTime<Utc>> =
            [("FFF".to_string(), long_ago)].into_iter().collect();
        let lookup = move |sym: &str| last_fire_map.get(sym).copied();
        let rows = vec![row("FFF", 12.0, 4)];
        let (acc, _skip) = select_candidates(&rows, &cfg(), &lookup, 0, now);
        assert_eq!(acc.len(), 1);
    }

    #[test]
    fn select_caps_at_max_open_positions() {
        let mut c = cfg();
        c.max_open_positions = 3;
        let rows = vec![
            row("AAA", 12.0, 4),
            row("BBB", 11.0, 4),
            row("CCC", 10.0, 4),
            row("DDD", 9.0, 4),
        ];
        let (acc, skip) = select_candidates(&rows, &c, &no_cooldown(), 0, Utc::now());
        assert_eq!(acc.len(), 3, "exactly max_open_positions submit");
        assert_eq!(skip.len(), 1);
        assert_eq!(skip[0].0, "DDD");
        assert_eq!(skip[0].3, SkipReason::CapReached);
    }

    #[test]
    fn select_caps_account_for_already_open_positions() {
        let mut c = cfg();
        c.max_open_positions = 3;
        let rows = vec![row("AAA", 12.0, 4), row("BBB", 11.0, 4)];
        // 2 already open + 2 new = 4, cap is 3 → only 1 new submits.
        let (acc, skip) = select_candidates(&rows, &c, &no_cooldown(), 2, Utc::now());
        assert_eq!(acc.len(), 1);
        assert_eq!(skip.len(), 1);
        assert_eq!(skip[0].3, SkipReason::CapReached);
    }

    #[test]
    fn select_evaluates_gates_in_priority_order() {
        // Symbol is below score AND below sources — should land in BelowScore
        // bucket (score gate is first), NOT InsufficientSources.
        let rows = vec![row("XYZ", 3.0, 1)];
        let (_acc, skip) = select_candidates(&rows, &cfg(), &no_cooldown(), 0, Utc::now());
        assert_eq!(skip[0].3, SkipReason::BelowScore);
    }

    #[test]
    fn shares_for_notional_floors_to_whole_shares() {
        // $1000 / $99.50 = 10.05 shares → 10.
        let q = shares_for_notional(1000.0, 99.5).unwrap();
        assert_eq!(q.to_string(), "10");
    }

    #[test]
    fn shares_for_notional_rejects_when_below_one_share() {
        // $50 / $250 = 0.2 → reject.
        assert!(shares_for_notional(50.0, 250.0).is_none());
        // Zero or negative inputs reject.
        assert!(shares_for_notional(0.0, 100.0).is_none());
        assert!(shares_for_notional(100.0, 0.0).is_none());
        assert!(shares_for_notional(-100.0, 100.0).is_none());
    }

    #[test]
    fn default_for_disables_autopilot_on_first_load() {
        let c = AutotradeConfig::default_for(Uuid::nil());
        assert!(
            !c.enabled,
            "must default OFF — autopilot is opt-in only, never on by default"
        );
    }
}
