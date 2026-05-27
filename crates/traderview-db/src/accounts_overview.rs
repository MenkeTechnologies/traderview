//! Per-account aggregation overview.
//!
//! For each account the user owns, computes (in one SQL query each):
//!   * total_closed_pnl, mtd_pnl, ytd_pnl, today_pnl
//!   * trade_count, win_count, loss_count, win_rate
//!   * best_trade_pnl, worst_trade_pnl + symbol/id for each
//!   * open_positions_count, open_notional (from live quotes)
//!   * unrealized_pnl on open positions (reuses live_positions::snapshot)
//!
//! Useful for traders running multiple accounts per strategy / family
//! / brokerage and wanting to compare them at a glance without flipping
//! the account-select dropdown.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct AccountSummary {
    pub account_id: Uuid,
    pub broker: String,
    pub name: String,
    pub base_currency: Option<String>,
    pub trade_count: i64,
    pub win_count: i64,
    pub loss_count: i64,
    pub win_rate: f64,
    pub total_closed_pnl: f64,
    pub ytd_pnl: f64,
    pub mtd_pnl: f64,
    pub today_pnl: f64,
    pub best_trade_pnl: Option<f64>,
    pub best_trade_symbol: Option<String>,
    pub worst_trade_pnl: Option<f64>,
    pub worst_trade_symbol: Option<String>,
    pub open_positions_count: usize,
    pub open_notional: f64,
    pub open_unrealized_pnl: f64,
    pub open_day_pnl: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverviewReport {
    pub accounts: Vec<AccountSummary>,
    pub grand_total: GrandTotal,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GrandTotal {
    pub accounts: usize,
    pub trade_count: i64,
    pub win_count: i64,
    pub loss_count: i64,
    pub total_closed_pnl: f64,
    pub ytd_pnl: f64,
    pub mtd_pnl: f64,
    pub today_pnl: f64,
    pub open_positions_count: usize,
    pub open_notional: f64,
    pub open_unrealized_pnl: f64,
    pub open_day_pnl: f64,
}

pub async fn report(pool: &PgPool, user_id: Uuid) -> anyhow::Result<OverviewReport> {
    let accounts: Vec<(Uuid, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, broker, name, base_currency FROM accounts WHERE user_id = $1
          ORDER BY broker, name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut summaries = Vec::with_capacity(accounts.len());
    let mut grand = GrandTotal {
        accounts: accounts.len(),
        ..Default::default()
    };

    type AccountAggRow = (
        i64,
        i64,
        i64,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
    );
    for (id, broker, name, ccy) in accounts {
        // Aggregates for closed trades — one CTE-style query per account.
        let row: AccountAggRow = sqlx::query_as(
            "SELECT
                COUNT(*)::int8                                                      AS n,
                COUNT(*) FILTER (WHERE net_pnl > 0)::int8                           AS wins,
                COUNT(*) FILTER (WHERE net_pnl < 0)::int8                           AS losses,
                COALESCE(SUM(net_pnl), 0)::float8                                   AS total,
                COALESCE(SUM(net_pnl) FILTER (
                    WHERE DATE(opened_at AT TIME ZONE 'UTC') >= DATE_TRUNC('year', CURRENT_DATE)::date
                ), 0)::float8                                                       AS ytd,
                COALESCE(SUM(net_pnl) FILTER (
                    WHERE DATE(opened_at AT TIME ZONE 'UTC') >= DATE_TRUNC('month', CURRENT_DATE)::date
                ), 0)::float8                                                       AS mtd,
                COALESCE(SUM(net_pnl) FILTER (
                    WHERE DATE(opened_at AT TIME ZONE 'UTC') = CURRENT_DATE
                ), 0)::float8                                                       AS today
              FROM trades
             WHERE account_id = $1
               AND status = 'closed'
               AND net_pnl IS NOT NULL",
        ).bind(id).fetch_one(pool).await?;
        let (n, wins, losses, total, ytd, mtd, today) = row;

        // Best + worst (one round-trip each).
        let best: Option<(Decimal, String)> = sqlx::query_as(
            "SELECT net_pnl, symbol FROM trades
              WHERE account_id = $1 AND status = 'closed' AND net_pnl IS NOT NULL
              ORDER BY net_pnl DESC LIMIT 1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        let worst: Option<(Decimal, String)> = sqlx::query_as(
            "SELECT net_pnl, symbol FROM trades
              WHERE account_id = $1 AND status = 'closed' AND net_pnl IS NOT NULL
              ORDER BY net_pnl ASC LIMIT 1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        // Open positions — reuse the existing live snapshot. May hit Yahoo;
        // 60s cache makes repeated calls cheap.
        let live = crate::live_positions::snapshot(pool, id).await.ok();
        let open_count = live.as_ref().map(|l| l.position_count).unwrap_or(0);
        let open_not = live.as_ref().map(|l| l.total_notional).unwrap_or(0.0);
        let open_upnl = live.as_ref().map(|l| l.total_unrealized_pnl).unwrap_or(0.0);
        let open_day_pnl = live.as_ref().map(|l| l.total_day_pnl).unwrap_or(0.0);

        let total_v = total.unwrap_or(0.0);
        let ytd_v = ytd.unwrap_or(0.0);
        let mtd_v = mtd.unwrap_or(0.0);
        let today_v = today.unwrap_or(0.0);

        let win_rate = if n > 0 { wins as f64 / n as f64 } else { 0.0 };

        grand.trade_count += n;
        grand.win_count += wins;
        grand.loss_count += losses;
        grand.total_closed_pnl += total_v;
        grand.ytd_pnl += ytd_v;
        grand.mtd_pnl += mtd_v;
        grand.today_pnl += today_v;
        grand.open_positions_count += open_count;
        grand.open_notional += open_not;
        grand.open_unrealized_pnl += open_upnl;
        grand.open_day_pnl += open_day_pnl;

        summaries.push(AccountSummary {
            account_id: id,
            broker,
            name,
            base_currency: ccy,
            trade_count: n,
            win_count: wins,
            loss_count: losses,
            win_rate,
            total_closed_pnl: total_v,
            ytd_pnl: ytd_v,
            mtd_pnl: mtd_v,
            today_pnl: today_v,
            best_trade_pnl: best.as_ref().map(|(p, _)| dec(*p)),
            best_trade_symbol: best.map(|(_, s)| s),
            worst_trade_pnl: worst.as_ref().map(|(p, _)| dec(*p)),
            worst_trade_symbol: worst.map(|(_, s)| s),
            open_positions_count: open_count,
            open_notional: open_not,
            open_unrealized_pnl: open_upnl,
            open_day_pnl,
        });
    }

    Ok(OverviewReport {
        accounts: summaries,
        grand_total: grand,
        computed_at: Utc::now(),
    })
}

fn dec(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}
