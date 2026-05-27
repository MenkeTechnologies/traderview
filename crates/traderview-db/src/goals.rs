//! Trading goals — CRUD + windowed-progress engine.
//!
//! Progress shape per goal:
//!   * actual_pnl, actual_win_rate, actual_max_drawdown_pct (computed from
//!     closed-trade history in the window)
//!   * elapsed_pct, projected_pnl (linear run-rate extrapolation to end)
//!   * pace flag for each metric: on_track | falling_short | exceeded |
//!     no_target

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Goal {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Option<Uuid>,
    pub name: String,
    pub period: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub target_pnl: Option<Decimal>,
    pub target_win_rate: Option<f32>,
    pub target_max_drawdown_pct: Option<f32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoalInput {
    pub account_id: Option<Uuid>,
    pub name: String,
    pub period: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub target_pnl: Option<f64>,
    pub target_win_rate: Option<f32>,
    pub target_max_drawdown_pct: Option<f32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Pace {
    OnTrack,
    FallingShort,
    Exceeded,
    NoTarget,
}

#[derive(Debug, Clone, Serialize)]
pub struct GoalProgress {
    pub goal: Goal,
    pub trades_in_window: usize,
    pub wins: usize,
    pub losses: usize,
    pub actual_pnl: f64,
    pub actual_win_rate: f64,
    pub actual_max_drawdown_pct: f64,
    pub days_total: i64,
    pub days_elapsed: i64,
    pub elapsed_pct: f64,
    pub projected_pnl: Option<f64>,    // linear run-rate to end
    pub pnl_pct_complete: Option<f64>, // actual / target
    pub pnl_pace: Pace,
    pub win_rate_pace: Pace,
    pub drawdown_pace: Pace, // exceeded = within cap; falling_short = over cap
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Goal>> {
    Ok(sqlx::query_as(
        "SELECT id, user_id, account_id, name, period, start_date, end_date,
                target_pnl, target_win_rate, target_max_drawdown_pct, notes,
                created_at, updated_at
           FROM trading_goals WHERE user_id = $1 ORDER BY start_date DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<Option<Goal>> {
    Ok(sqlx::query_as(
        "SELECT id, user_id, account_id, name, period, start_date, end_date,
                target_pnl, target_win_rate, target_max_drawdown_pct, notes,
                created_at, updated_at
           FROM trading_goals WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn create(pool: &PgPool, user_id: Uuid, dto: &GoalInput) -> anyhow::Result<Goal> {
    Ok(sqlx::query_as(
        "INSERT INTO trading_goals
            (user_id, account_id, name, period, start_date, end_date,
             target_pnl, target_win_rate, target_max_drawdown_pct, notes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         RETURNING id, user_id, account_id, name, period, start_date, end_date,
                   target_pnl, target_win_rate, target_max_drawdown_pct, notes,
                   created_at, updated_at",
    )
    .bind(user_id)
    .bind(dto.account_id)
    .bind(&dto.name)
    .bind(&dto.period)
    .bind(dto.start_date)
    .bind(dto.end_date)
    .bind(dto.target_pnl.and_then(|x| Decimal::try_from(x).ok()))
    .bind(dto.target_win_rate)
    .bind(dto.target_max_drawdown_pct)
    .bind(dto.notes.as_deref())
    .fetch_one(pool)
    .await?)
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    dto: &GoalInput,
) -> anyhow::Result<Option<Goal>> {
    Ok(sqlx::query_as(
        "UPDATE trading_goals SET
            account_id              = $3,
            name                    = $4,
            period                  = $5,
            start_date              = $6,
            end_date                = $7,
            target_pnl              = $8,
            target_win_rate         = $9,
            target_max_drawdown_pct = $10,
            notes                   = $11,
            updated_at              = now()
          WHERE id = $1 AND user_id = $2
          RETURNING id, user_id, account_id, name, period, start_date, end_date,
                    target_pnl, target_win_rate, target_max_drawdown_pct, notes,
                    created_at, updated_at",
    )
    .bind(id)
    .bind(user_id)
    .bind(dto.account_id)
    .bind(&dto.name)
    .bind(&dto.period)
    .bind(dto.start_date)
    .bind(dto.end_date)
    .bind(dto.target_pnl.and_then(|x| Decimal::try_from(x).ok()))
    .bind(dto.target_win_rate)
    .bind(dto.target_max_drawdown_pct)
    .bind(dto.notes.as_deref())
    .fetch_optional(pool)
    .await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    Ok(
        sqlx::query("DELETE FROM trading_goals WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?
            .rows_affected()
            > 0,
    )
}

pub async fn progress(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<GoalProgress> {
    let goal = get(pool, user_id, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("goal not found"))?;

    // Fetch closed trades in window. Account filter is optional.
    let trades: Vec<(Decimal, DateTime<Utc>)> = if let Some(aid) = goal.account_id {
        sqlx::query_as(
            "SELECT net_pnl, opened_at FROM trades
              WHERE account_id = $1
                AND status = 'closed'
                AND net_pnl IS NOT NULL
                AND DATE(opened_at AT TIME ZONE 'UTC') BETWEEN $2 AND $3
              ORDER BY opened_at ASC",
        )
        .bind(aid)
        .bind(goal.start_date)
        .bind(goal.end_date)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT t.net_pnl, t.opened_at
               FROM trades t
               JOIN accounts a ON a.id = t.account_id
              WHERE a.user_id = $1
                AND t.status = 'closed'
                AND t.net_pnl IS NOT NULL
                AND DATE(t.opened_at AT TIME ZONE 'UTC') BETWEEN $2 AND $3
              ORDER BY t.opened_at ASC",
        )
        .bind(user_id)
        .bind(goal.start_date)
        .bind(goal.end_date)
        .fetch_all(pool)
        .await?
    };

    let pnls: Vec<f64> = trades.iter().map(|(p, _)| dec(*p)).collect();
    let wins = pnls.iter().filter(|x| **x > 0.0).count();
    let losses = pnls.iter().filter(|x| **x < 0.0).count();
    let total: f64 = pnls.iter().sum();
    let win_rate = if pnls.is_empty() {
        0.0
    } else {
        wins as f64 / pnls.len() as f64
    };

    // Max drawdown across cumulative P/L curve, expressed as % of peak.
    let mut peak = 0.0f64;
    let mut cum = 0.0f64;
    let mut max_dd_pct = 0.0f64;
    for p in &pnls {
        cum += p;
        if cum > peak {
            peak = cum;
        }
        if peak > 0.0 {
            let dd = (cum - peak) / peak * 100.0;
            if dd < max_dd_pct {
                max_dd_pct = dd;
            }
        }
    }
    let actual_max_drawdown_pct = max_dd_pct.abs();

    let today = Utc::now().date_naive();
    let total_days = (goal.end_date - goal.start_date).num_days() + 1;
    let elapsed = (today.min(goal.end_date) - goal.start_date)
        .num_days()
        .max(0)
        + 1;
    let elapsed = elapsed.min(total_days);
    let elapsed_pct = if total_days > 0 {
        elapsed as f64 / total_days as f64 * 100.0
    } else {
        0.0
    };

    // Linear run-rate projection.
    let projected_pnl = if elapsed > 0 {
        Some(total / elapsed as f64 * total_days as f64)
    } else {
        None
    };

    // Per-metric pace.
    let pnl_pace = match goal.target_pnl.as_ref().map(|d| dec(*d)) {
        None => Pace::NoTarget,
        Some(t) if total >= t => Pace::Exceeded,
        Some(t) => match projected_pnl {
            Some(p) if p >= t => Pace::OnTrack,
            _ => Pace::FallingShort,
        },
    };
    let win_rate_pace = match goal.target_win_rate {
        None => Pace::NoTarget,
        Some(t) if win_rate >= t as f64 => Pace::Exceeded,
        Some(t) if pnls.is_empty() => {
            let _ = t;
            Pace::FallingShort
        }
        Some(_) => Pace::FallingShort,
    };
    let drawdown_pace = match goal.target_max_drawdown_pct {
        None => Pace::NoTarget,
        // Lower is better; "exceeded" here means under the cap (good).
        Some(cap) if actual_max_drawdown_pct <= cap as f64 => Pace::Exceeded,
        Some(_) => Pace::FallingShort,
    };

    let pnl_pct_complete = goal.target_pnl.as_ref().map(|d| {
        let t = dec(*d);
        if t.abs() < 1e-9 {
            0.0
        } else {
            total / t * 100.0
        }
    });

    Ok(GoalProgress {
        goal,
        trades_in_window: pnls.len(),
        wins,
        losses,
        actual_pnl: total,
        actual_win_rate: win_rate,
        actual_max_drawdown_pct,
        days_total: total_days,
        days_elapsed: elapsed,
        elapsed_pct,
        projected_pnl,
        pnl_pct_complete,
        pnl_pace,
        win_rate_pace,
        drawdown_pace,
    })
}

fn dec(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}
