//! Cash sweep interest — idle cash accrues daily at the account's
//! APY, the way a broker money-market sweep does. ACT/365 simple,
//! credit-only: negative cash (margin debit) accrues nothing rather
//! than silently charging an unconfigured borrow rate.

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// Catch-up cap: a process dead longer than this credits at most 30
/// days — the cash balance the missed days would have applied to is
/// unknowable after that long.
pub const MAX_CATCHUP_DAYS: i64 = 30;

/// Simple ACT/365 accrual over `days`. Zero for non-positive cash or
/// rate; days are clamped to [0, MAX_CATCHUP_DAYS]. No compounding
/// inside a catch-up window — one balance, one rate, N days.
pub fn interest_accrual(cash: f64, apy_pct: f64, days: i64) -> f64 {
    if cash <= 0.0 || apy_pct <= 0.0 {
        return 0.0;
    }
    let days = days.clamp(0, MAX_CATCHUP_DAYS);
    cash * (apy_pct / 100.0) / 365.0 * days as f64
}

/// Set the account's sweep APY (0 disables). Bounded to a sane range
/// — 20% is already beyond any money-market reality.
pub async fn set_cash_apy(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    apy_pct: Decimal,
) -> anyhow::Result<bool> {
    if apy_pct < Decimal::ZERO || apy_pct > Decimal::from(20) {
        anyhow::bail!("apy_pct must be in 0..=20");
    }
    let r = sqlx::query(
        "UPDATE paper_accounts SET cash_apy_pct = $3 WHERE id = $1 AND user_id = $2",
    )
    .bind(account_id)
    .bind(user_id)
    .bind(apy_pct)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct InterestCredit {
    pub credited_on: chrono::NaiveDate,
    pub amount: Decimal,
    pub apy_pct: Decimal,
    pub days: i32,
}

pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<InterestCredit>> {
    Ok(sqlx::query_as(
        "SELECT i.credited_on, i.amount, i.apy_pct, i.days
           FROM paper_interest i
           JOIN paper_accounts a ON a.id = i.paper_account_id
          WHERE i.paper_account_id = $1 AND a.user_id = $2
          ORDER BY i.credited_on DESC
          LIMIT $3",
    )
    .bind(account_id)
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

/// Daily credit pass. The claim UPDATE stamps last_interest_on = today
/// and returns the prior stamp atomically, so a concurrent pass (or a
/// restart mid-pass) credits each account at most once per day; the
/// cash credit and audit row then commit in one transaction.
pub async fn tick(pool: &PgPool) -> anyhow::Result<usize> {
    use rust_decimal::prelude::ToPrimitive;
    let today = Utc::now().date_naive();
    let claimed: Vec<(Uuid, Decimal, Decimal)> = sqlx::query_as(
        "UPDATE paper_accounts
            SET last_interest_on = $1
          WHERE cash_apy_pct > 0 AND cash > 0
            AND (last_interest_on IS NULL OR last_interest_on < $1)
        RETURNING id, cash, cash_apy_pct",
    )
    .bind(today)
    .fetch_all(pool)
    .await?;
    let mut credited = 0;
    for (id, cash, apy) in &claimed {
        // Gap days come from the AUDIT trail (last credited_on), not
        // the claim stamp — RETURNING sees the new row, and the audit
        // row is what proves a credit happened. Fresh enablement (no
        // audit rows) credits exactly one day.
        let prior: Option<(chrono::NaiveDate,)> = sqlx::query_as(
            "SELECT credited_on FROM paper_interest
              WHERE paper_account_id = $1 ORDER BY credited_on DESC LIMIT 1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        let days = prior
            .map(|(d,)| (today - d).num_days())
            .unwrap_or(1)
            .clamp(0, MAX_CATCHUP_DAYS);
        if days == 0 {
            continue;
        }
        let amount = interest_accrual(
            cash.to_f64().unwrap_or(0.0),
            apy.to_f64().unwrap_or(0.0),
            days,
        );
        if amount <= 0.0 {
            continue;
        }
        let amount = Decimal::try_from(amount).unwrap_or_default().round_dp(2);
        let mut tx = pool.begin().await?;
        sqlx::query("UPDATE paper_accounts SET cash = cash + $2 WHERE id = $1")
            .bind(id)
            .bind(amount)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO paper_interest (paper_account_id, credited_on, amount, apy_pct, days)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(today)
        .bind(amount)
        .bind(apy)
        .bind(days as i32)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        credited += 1;
    }
    Ok(credited)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accrual_pins_act365_and_edges() {
        // $100k at 5% for one day: 100000 × 0.05 / 365.
        let one = interest_accrual(100_000.0, 5.0, 1);
        assert!((one - 100_000.0 * 0.05 / 365.0).abs() < 1e-9);
        // Three missed days credit linearly (no intra-gap compounding).
        assert!((interest_accrual(100_000.0, 5.0, 3) - 3.0 * one).abs() < 1e-9);
        // Credit-only: margin debits accrue nothing. Zero rate, zero
        // days, and the 30-day catch-up cap.
        assert_eq!(interest_accrual(-5_000.0, 5.0, 1), 0.0);
        assert_eq!(interest_accrual(100_000.0, 0.0, 1), 0.0);
        assert_eq!(interest_accrual(100_000.0, 5.0, 0), 0.0);
        assert!((interest_accrual(100_000.0, 5.0, 400) - 30.0 * one).abs() < 1e-9);
    }
}
