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

/// Equity-short entry notional: Σ |qty| × avg_price over SHORT
/// (negative-qty) non-OCC positions. Options shorts are margin, not
/// stock borrow — excluded. Borrow is charged on ENTRY notional, not
/// marked value: the daily pass is deterministic from the positions
/// table, with no quote dependency to fail mid-sweep. Stated
/// simplification, not a hidden one.
pub fn short_borrow_notional(positions: &[(String, Decimal, Decimal)]) -> f64 {
    use rust_decimal::prelude::ToPrimitive;
    positions
        .iter()
        .filter(|(sym, qty, _)| *qty < Decimal::ZERO && !traderview_core::occ_symbol::is_occ(sym))
        .map(|(_, qty, avg)| {
            qty.abs().to_f64().unwrap_or(0.0) * avg.to_f64().unwrap_or(0.0)
        })
        .sum()
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

/// Set the margin-loan APY charged on negative cash (0 = off; 25%
/// covers any retail broker).
pub async fn set_margin_apy(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    apy_pct: Decimal,
) -> anyhow::Result<bool> {
    if apy_pct < Decimal::ZERO || apy_pct > Decimal::from(25) {
        anyhow::bail!("apy_pct must be in 0..=25");
    }
    let r = sqlx::query(
        "UPDATE paper_accounts SET margin_apy_pct = $3 WHERE id = $1 AND user_id = $2",
    )
    .bind(account_id)
    .bind(user_id)
    .bind(apy_pct)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

/// Set the account's margin multiplier: 1 = cash, 2 = Reg-T, up to 4.
pub async fn set_margin_multiplier(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    multiplier: Decimal,
) -> anyhow::Result<bool> {
    if multiplier < Decimal::ONE || multiplier > Decimal::from(4) {
        anyhow::bail!("margin_multiplier must be in 1..=4");
    }
    let r = sqlx::query(
        "UPDATE paper_accounts SET margin_multiplier = $3 WHERE id = $1 AND user_id = $2",
    )
    .bind(account_id)
    .bind(user_id)
    .bind(multiplier)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() > 0)
}

/// Set the account's short borrow APY (0 disables). 50% covers even
/// hard-to-borrow names.
pub async fn set_borrow_apy(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    apy_pct: Decimal,
) -> anyhow::Result<bool> {
    if apy_pct < Decimal::ZERO || apy_pct > Decimal::from(50) {
        anyhow::bail!("apy_pct must be in 0..=50");
    }
    let r = sqlx::query(
        "UPDATE paper_accounts SET borrow_apy_pct = $3 WHERE id = $1 AND user_id = $2",
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
    /// 'cash_sweep' (credit) or 'short_borrow' (debit).
    pub kind: String,
}

pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<InterestCredit>> {
    Ok(sqlx::query_as(
        "SELECT i.credited_on, i.amount, i.apy_pct, i.days, i.kind
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
    let claimed: Vec<(Uuid, Decimal, Decimal, Decimal, Decimal)> = sqlx::query_as(
        "UPDATE paper_accounts
            SET last_interest_on = $1
          WHERE (cash_apy_pct > 0 OR borrow_apy_pct > 0 OR margin_apy_pct > 0)
            AND (last_interest_on IS NULL OR last_interest_on < $1)
        RETURNING id, cash, cash_apy_pct, borrow_apy_pct, margin_apy_pct",
    )
    .bind(today)
    .fetch_all(pool)
    .await?;
    let mut credited = 0;
    for (id, cash, apy, borrow_apy, margin_apy) in &claimed {
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
        // Cash sweep credit (zero for non-positive cash or rate).
        let sweep = interest_accrual(
            cash.to_f64().unwrap_or(0.0),
            apy.to_f64().unwrap_or(0.0),
            days,
        );
        // Short borrow debit on equity-short entry notional.
        let borrow = if *borrow_apy > Decimal::ZERO {
            let positions: Vec<(String, Decimal, Decimal)> = sqlx::query_as(
                "SELECT symbol, qty, avg_price FROM paper_positions
                  WHERE paper_account_id = $1 AND qty < 0",
            )
            .bind(id)
            .fetch_all(pool)
            .await?;
            interest_accrual(
                short_borrow_notional(&positions),
                borrow_apy.to_f64().unwrap_or(0.0),
                days,
            )
        } else {
            0.0
        };
        // Margin loan interest: ACT/365 on the debit balance. The
        // accrual fn's base is the loan magnitude — interest_accrual
        // itself zeroes non-positive bases, so a positive cash
        // balance charges nothing.
        let loan = interest_accrual(
            (-*cash).to_f64().unwrap_or(0.0),
            margin_apy.to_f64().unwrap_or(0.0),
            days,
        );
        if sweep <= 0.0 && borrow <= 0.0 && loan <= 0.0 {
            continue;
        }
        let sweep = Decimal::try_from(sweep).unwrap_or_default().round_dp(2);
        let borrow = Decimal::try_from(borrow).unwrap_or_default().round_dp(2);
        let loan = Decimal::try_from(loan).unwrap_or_default().round_dp(2);
        let mut tx = pool.begin().await?;
        sqlx::query("UPDATE paper_accounts SET cash = cash + $2 WHERE id = $1")
            .bind(id)
            .bind(sweep - borrow - loan)
            .execute(&mut *tx)
            .await?;
        if sweep > Decimal::ZERO {
            sqlx::query(
                "INSERT INTO paper_interest (paper_account_id, credited_on, amount, apy_pct, days, kind)
                 VALUES ($1, $2, $3, $4, $5, 'cash_sweep')",
            )
            .bind(id).bind(today).bind(sweep).bind(apy).bind(days as i32)
            .execute(&mut *tx)
            .await?;
        }
        if borrow > Decimal::ZERO {
            sqlx::query(
                "INSERT INTO paper_interest (paper_account_id, credited_on, amount, apy_pct, days, kind)
                 VALUES ($1, $2, $3, $4, $5, 'short_borrow')",
            )
            .bind(id).bind(today).bind(-borrow).bind(borrow_apy).bind(days as i32)
            .execute(&mut *tx)
            .await?;
        }
        if loan > Decimal::ZERO {
            sqlx::query(
                "INSERT INTO paper_interest (paper_account_id, credited_on, amount, apy_pct, days, kind)
                 VALUES ($1, $2, $3, $4, $5, 'margin_interest')",
            )
            .bind(id).bind(today).bind(-loan).bind(margin_apy).bind(days as i32)
            .execute(&mut *tx)
            .await?;
        }
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
        // Credit-only: margin debits accrue nothing through the SWEEP
        // (the loan pass charges them via the negated base — same
        // ACT/365: a $20k loan at 10% for one day).
        let loan_day = interest_accrual(20_000.0, 10.0, 1);
        assert!((loan_day - 20_000.0 * 0.10 / 365.0).abs() < 1e-9);
        assert_eq!(interest_accrual(-5_000.0, 5.0, 1), 0.0);
        assert_eq!(interest_accrual(100_000.0, 0.0, 1), 0.0);
        assert_eq!(interest_accrual(100_000.0, 5.0, 0), 0.0);
        assert!((interest_accrual(100_000.0, 5.0, 400) - 30.0 * one).abs() < 1e-9);
    }

    #[test]
    fn borrow_notional_shorts_only_no_options() {
        let d = |v: i64| Decimal::from(v);
        let positions = vec![
            ("AAPL".to_string(), d(-100), d(150)),            // equity short: 15000
            ("TSLA".to_string(), d(50), d(200)),              // long: excluded
            ("AAPL260117C00190000".to_string(), d(-2), d(5)), // OCC short: margin, not borrow
            ("MSFT".to_string(), d(-10), d(300)),             // equity short: 3000
        ];
        assert!((short_borrow_notional(&positions) - 18_000.0).abs() < 1e-9);
        assert_eq!(short_borrow_notional(&[]), 0.0);
    }
}
