//! Auto-invest — recurring notional buys on paper accounts.
//!
//! "$500 of SPY weekly": the background pass submits due orders as
//! market buys through the normal paper fill path (friction, fills,
//! journaling, equity sampling all apply) with FRACTIONAL share
//! quantity = notional / price, then advances next_run_at by the
//! cadence FROM THE SCHEDULED TIME — a pass that runs late doesn't
//! push every future buy later. Transient quote failures leave
//! next_run_at untouched so the buy retries next pass.

use chrono::{DateTime, Datelike, Duration, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RecurringOrder {
    pub id: Uuid,
    pub account_id: Uuid,
    pub symbol: String,
    pub notional_usd: Decimal,
    pub cadence: String,
    pub enabled: bool,
    pub next_run_at: DateTime<Utc>,
    pub last_status: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Next occurrence strictly after `scheduled`, advancing by the
/// cadence from the SCHEDULED time. Monthly clamps to the shorter
/// month's last day (Jan 31 → Feb 28) rather than spilling into March.
pub fn next_occurrence(scheduled: DateTime<Utc>, cadence: &str) -> DateTime<Utc> {
    match cadence {
        "daily" => scheduled + Duration::days(1),
        "weekly" => scheduled + Duration::days(7),
        _ => {
            let (y, m) = if scheduled.month() == 12 {
                (scheduled.year() + 1, 1)
            } else {
                (scheduled.year(), scheduled.month() + 1)
            };
            let day = scheduled.day();
            // Walk down from the wanted day until the date exists.
            let date = (0..4)
                .filter_map(|back| {
                    chrono::NaiveDate::from_ymd_opt(y, m, day.saturating_sub(back).max(1))
                })
                .next()
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(y, m, 1).unwrap());
            DateTime::from_naive_utc_and_offset(date.and_time(scheduled.time()), Utc)
        }
    }
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    symbol: &str,
    notional_usd: Decimal,
    cadence: &str,
) -> anyhow::Result<RecurringOrder> {
    let symbol = symbol.trim().to_uppercase();
    if symbol.is_empty() || symbol.len() > 20 {
        anyhow::bail!("invalid symbol");
    }
    if notional_usd <= Decimal::ZERO || notional_usd > Decimal::from(1_000_000) {
        anyhow::bail!("notional must be in (0, 1,000,000]");
    }
    if !matches!(cadence, "daily" | "weekly" | "monthly") {
        anyhow::bail!("cadence must be daily | weekly | monthly");
    }
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    Ok(sqlx::query_as(
        "INSERT INTO paper_recurring_orders
            (user_id, account_id, symbol, notional_usd, cadence, next_run_at)
         VALUES ($1, $2, $3, $4, $5, now())
         RETURNING id, account_id, symbol, notional_usd, cadence, enabled,
                   next_run_at, last_status, created_at",
    )
    .bind(user_id)
    .bind(account_id)
    .bind(&symbol)
    .bind(notional_usd)
    .bind(cadence)
    .fetch_one(pool)
    .await?)
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<RecurringOrder>> {
    Ok(sqlx::query_as(
        "SELECT id, account_id, symbol, notional_usd, cadence, enabled,
                next_run_at, last_status, created_at
           FROM paper_recurring_orders WHERE user_id = $1 ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn set_enabled(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    enabled: bool,
) -> anyhow::Result<bool> {
    let r = sqlx::query(
        "UPDATE paper_recurring_orders SET enabled = $3 WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .bind(enabled)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let r = sqlx::query("DELETE FROM paper_recurring_orders WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

/// One pass: submit every due enabled order, advance schedules.
/// Returns orders submitted.
pub async fn tick(pool: &PgPool) -> anyhow::Result<usize> {
    #[derive(sqlx::FromRow)]
    struct Due {
        id: Uuid,
        user_id: Uuid,
        account_id: Uuid,
        symbol: String,
        notional_usd: Decimal,
        cadence: String,
        next_run_at: DateTime<Utc>,
    }
    let due: Vec<Due> = sqlx::query_as(
        "SELECT id, user_id, account_id, symbol, notional_usd, cadence, next_run_at
           FROM paper_recurring_orders
          WHERE enabled AND next_run_at <= now()
          ORDER BY next_run_at LIMIT 100",
    )
    .fetch_all(pool)
    .await?;
    let mut submitted = 0usize;
    for d in due {
        // Quote failures are transient: leave next_run_at so it retries.
        let Ok(quote) = crate::market_data::quote(pool, &d.symbol).await else {
            continue;
        };
        let Ok(price) = Decimal::try_from(quote.price) else { continue };
        if price <= Decimal::ZERO {
            continue;
        }
        // Fractional shares to 4dp — the paper book is Decimal native.
        let qty = (d.notional_usd / price).round_dp(4);
        if qty <= Decimal::ZERO {
            continue;
        }
        let status = match crate::paper::submit(
            pool,
            d.user_id,
            d.account_id,
            crate::paper::OrderRequest {
                symbol: d.symbol.clone(),
                side: traderview_core::Side::Buy,
                qty,
                order_type: "market".into(),
                limit_price: None,
                stop_price: None,
                trail_value: None,
                trail_is_pct: None,
                time_in_force: None,
                expire_at: None,
            },
        )
        .await
        {
            Ok(o) => {
                submitted += 1;
                format!("{}: {} {} @ market", o.status, qty, d.symbol)
            }
            Err(e) => format!("error: {e}"),
        };
        // Advance from the SCHEDULED time, catching up if several
        // periods were missed (laptop asleep): never schedule in the past.
        let mut next = next_occurrence(d.next_run_at, &d.cadence);
        let now = Utc::now();
        while next <= now {
            next = next_occurrence(next, &d.cadence);
        }
        sqlx::query(
            "UPDATE paper_recurring_orders
                SET next_run_at = $2, last_status = $3 WHERE id = $1",
        )
        .bind(d.id)
        .bind(next)
        .bind(status)
        .execute(pool)
        .await
        .ok();
    }
    Ok(submitted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 14, 30, 0).unwrap()
    }

    #[test]
    fn cadences_advance_from_the_scheduled_time() {
        assert_eq!(next_occurrence(t(2026, 6, 10), "daily"), t(2026, 6, 11));
        assert_eq!(next_occurrence(t(2026, 6, 10), "weekly"), t(2026, 6, 17));
        assert_eq!(next_occurrence(t(2026, 6, 10), "monthly"), t(2026, 7, 10));
        // Time-of-day is preserved.
        assert_eq!(next_occurrence(t(2026, 6, 10), "daily").time(), t(2026, 6, 10).time());
    }

    #[test]
    fn monthly_clamps_short_months_and_wraps_years() {
        // Jan 31 → Feb 28 (2026 is not a leap year).
        assert_eq!(next_occurrence(t(2026, 1, 31), "monthly"), t(2026, 2, 28));
        // Dec → Jan of the next year.
        assert_eq!(next_occurrence(t(2026, 12, 15), "monthly"), t(2027, 1, 15));
    }
}
