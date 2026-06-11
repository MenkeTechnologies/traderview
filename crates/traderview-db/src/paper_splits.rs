//! Paper split adjustment — positions held through a stock split keep
//! pre-split qty and avg_price while quotes turn post-split, so a 4:1
//! split reads as a fake −75% move (and writes a fake drawdown into
//! the equity curve). A background pass detects recent splits on held
//! symbols via the existing Yahoo split fetcher and rewrites the
//! position value-preservingly: qty × ratio, avg_price ÷ ratio. Shorts
//! scale the same way (short 100 pre-split is short 400 post-4:1).
//!
//! An account with any fill ON or AFTER the split date is skipped with
//! a warning: those fills were at post-split prices against a stale
//! pre-split book, and unscrambling that needs per-lot history the
//! paper engine doesn't keep. The 6h cadence keeps that window small.
//! Same 30-day lookback and idempotent-ledger pattern as
//! [`crate::paper_dividends`].

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PaperSplit {
    pub symbol: String,
    pub split_date: NaiveDate,
    pub numerator: Decimal,
    pub denominator: Decimal,
    pub qty_before: Decimal,
    pub qty_after: Decimal,
    pub applied_at: chrono::DateTime<Utc>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Value-preserving split transform: qty scales by ratio, avg_price by
/// its inverse, so qty × avg_price is unchanged. None when the ratio is
/// degenerate (zero/negative legs).
pub fn apply_split(
    qty: Decimal,
    avg_price: Decimal,
    numerator: Decimal,
    denominator: Decimal,
) -> Option<(Decimal, Decimal)> {
    if numerator <= Decimal::ZERO || denominator <= Decimal::ZERO {
        return None;
    }
    let new_qty = (qty * numerator / denominator).round_dp(8);
    let new_avg = (avg_price * denominator / numerator).round_dp(8);
    Some((new_qty, new_avg))
}

// ─── Repository ────────────────────────────────────────────────────────────

/// One adjustment pass over all open paper positions. Fetches each held
/// symbol's split history once, and for every split inside the lookback
/// window rewrites the position inside a per-split transaction. Returns
/// adjustments applied.
pub async fn adjust_all(pool: &PgPool) -> anyhow::Result<usize> {
    let positions: Vec<(Uuid, String, Decimal, Decimal)> = sqlx::query_as(
        "SELECT paper_account_id, symbol, qty, avg_price
           FROM paper_positions WHERE qty <> 0",
    )
    .fetch_all(pool)
    .await?;

    let today = Utc::now().date_naive();
    let mut by_symbol: std::collections::BTreeMap<String, Vec<(Uuid, Decimal, Decimal)>> =
        Default::default();
    for (account_id, symbol, qty, avg) in positions {
        by_symbol.entry(symbol).or_default().push((account_id, qty, avg));
    }

    let mut applied = 0usize;
    for (symbol, holders) in by_symbol {
        let events = crate::dividend_tracker::fetch_split_events(&symbol).await;
        let recent: Vec<_> = events
            .iter()
            .filter(|e| crate::paper_dividends::in_credit_window(e.date, today))
            .collect();
        if recent.is_empty() {
            continue; // splits are rare — almost every pass ends here
        }
        for (account_id, qty, avg) in &holders {
            for ev in &recent {
                let (Ok(num), Ok(den)) = (
                    Decimal::try_from(ev.numerator),
                    Decimal::try_from(ev.denominator),
                ) else {
                    continue;
                };
                let Some((new_qty, new_avg)) = apply_split(*qty, *avg, num, den) else {
                    continue;
                };
                // Fills on/after the split date traded post-split prices
                // against the stale book — no clean retroactive fix
                // without lot history, so leave the account alone.
                let traded_after: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM paper_orders
                      WHERE paper_account_id = $1 AND symbol = $2
                        AND status = 'filled' AND filled_at >= $3",
                )
                .bind(account_id)
                .bind(&symbol)
                .bind(ev.date.and_hms_opt(0, 0, 0).unwrap().and_utc())
                .fetch_one(pool)
                .await?;
                if traded_after.0 > 0 {
                    tracing::warn!(
                        %symbol, account = %account_id, split_date = %ev.date,
                        "split adjustment skipped: fills on/after split date"
                    );
                    continue;
                }
                let mut tx = pool.begin().await?;
                let inserted = sqlx::query(
                    "INSERT INTO paper_splits
                        (paper_account_id, symbol, split_date, numerator, denominator,
                         qty_before, qty_after)
                     VALUES ($1, $2, $3, $4, $5, $6, $7)
                     ON CONFLICT (paper_account_id, symbol, split_date) DO NOTHING",
                )
                .bind(account_id)
                .bind(&symbol)
                .bind(ev.date)
                .bind(num)
                .bind(den)
                .bind(qty)
                .bind(new_qty)
                .execute(&mut *tx)
                .await?
                .rows_affected();
                if inserted == 1 {
                    sqlx::query(
                        "UPDATE paper_positions SET qty = $1, avg_price = $2, updated_at = now()
                          WHERE paper_account_id = $3 AND symbol = $4",
                    )
                    .bind(new_qty)
                    .bind(new_avg)
                    .bind(account_id)
                    .bind(&symbol)
                    .execute(&mut *tx)
                    .await?;
                    applied += 1;
                }
                tx.commit().await?;
            }
        }
    }
    Ok(applied)
}

/// Applied split adjustments for an owned account, newest first.
pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<PaperSplit>> {
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    Ok(sqlx::query_as(
        "SELECT symbol, split_date, numerator, denominator, qty_before, qty_after, applied_at
           FROM paper_splits
          WHERE paper_account_id = $1
          ORDER BY split_date DESC, symbol LIMIT $2",
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: i64) -> Decimal {
        Decimal::from(n)
    }

    #[test]
    fn forward_split_scales_qty_up_and_avg_down() {
        // 100 @ $400 through a 4:1 → 400 @ $100, value unchanged.
        let (q, a) = apply_split(d(100), d(400), d(4), d(1)).unwrap();
        assert_eq!(q, d(400));
        assert_eq!(a, d(100));
        assert_eq!(q * a, d(100) * d(400));
    }

    #[test]
    fn reverse_split_scales_qty_down_and_avg_up() {
        // 1000 @ $2 through a 1:10 reverse → 100 @ $20.
        let (q, a) = apply_split(d(1000), d(2), d(1), d(10)).unwrap();
        assert_eq!(q, d(100));
        assert_eq!(a, d(20));
    }

    #[test]
    fn short_position_scales_the_same_way() {
        // Short 100 @ $400 through a 4:1 → short 400 @ $100.
        let (q, a) = apply_split(d(-100), d(400), d(4), d(1)).unwrap();
        assert_eq!(q, d(-400));
        assert_eq!(a, d(100));
    }

    #[test]
    fn degenerate_ratios_are_rejected() {
        assert!(apply_split(d(100), d(400), Decimal::ZERO, d(1)).is_none());
        assert!(apply_split(d(100), d(400), d(4), Decimal::ZERO).is_none());
        assert!(apply_split(d(100), d(400), d(-4), d(1)).is_none());
    }
}
