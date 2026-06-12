//! Paper dividend crediting — the sim counterpart of the cash dividends
//! a real broker posts. A background pass walks every paper account ×
//! symbol with fill history, reconstructs the share count held going
//! INTO each recent ex-date from the filled-order ledger, and credits
//! `qty × amount_per_share` to account cash. Long positions are
//! credited; short positions are debited (a short pays the dividend to
//! the lender, same as a real margin account). The
//! UNIQUE(account, symbol, ex_date) constraint makes re-runs no-ops.
//!
//! Crediting is bounded to a 30-day ex-date lookback: the sim is
//! forward-looking from the pass that first sees an ex-date, not a
//! retroactive 5-year backfill that would inject phantom cash under an
//! already-recorded equity curve.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// Ex-dates older than this are never credited.
const CREDIT_LOOKBACK_DAYS: i64 = 30;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PaperDividend {
    pub symbol: String,
    pub ex_date: NaiveDate,
    pub amount_per_share: Decimal,
    pub qty: Decimal,
    pub cash_credited: Decimal,
    pub credited_at: DateTime<Utc>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Shares held going INTO an ex-date, from signed fills (buy/cover
/// positive, sell/short negative). Only fills dated strictly BEFORE the
/// ex-date count — buying on the ex-date itself buys without the
/// dividend. Negative result = net short on the ex-date.
pub fn qty_on_ex_date(signed_fills: &[(NaiveDate, Decimal)], ex_date: NaiveDate) -> Decimal {
    signed_fills
        .iter()
        .filter(|(d, _)| *d < ex_date)
        .map(|(_, q)| *q)
        .sum()
}

/// Whether an ex-date is creditable today: already occurred, and within
/// the lookback window.
pub fn in_credit_window(ex_date: NaiveDate, today: NaiveDate) -> bool {
    ex_date <= today && ex_date > today - chrono::Duration::days(CREDIT_LOOKBACK_DAYS)
}

// ─── Repository ────────────────────────────────────────────────────────────

/// One crediting pass over every paper account × symbol with recent or
/// current exposure. Fetches each symbol's dividend events once,
/// reconstructs the ex-date share count per account from filled orders,
/// and posts uncredited dividends inside a per-event transaction.
/// Returns credits written.
pub async fn credit_all(pool: &PgPool) -> anyhow::Result<usize> {
    // Current positions UNION recent fills — a position sold after the
    // ex-date but before this pass still gets its credit.
    let candidates: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT DISTINCT paper_account_id, symbol FROM paper_positions
         UNION
         SELECT DISTINCT paper_account_id, symbol FROM paper_orders
          WHERE status = 'filled' AND filled_at > now() - interval '60 days'",
    )
    .fetch_all(pool)
    .await?;

    let today = Utc::now().date_naive();
    let mut by_symbol: std::collections::BTreeMap<String, Vec<Uuid>> = Default::default();
    for (account_id, symbol) in candidates {
        by_symbol.entry(symbol).or_default().push(account_id);
    }

    let mut written = 0usize;
    for (symbol, accounts) in by_symbol {
        let (events, _) = crate::dividend_tracker::fetch_dividend_events(&symbol).await;
        let recent: Vec<_> = events
            .iter()
            .filter(|e| in_credit_window(e.ex_date, today))
            .collect();
        if recent.is_empty() {
            continue; // most passes: nothing in the window
        }
        for account_id in &accounts {
            // Signed fill ledger for this account × symbol, dated by the
            // UTC fill date — close enough for a sim crediting pass.
            let fills: Vec<(DateTime<Utc>, Decimal)> = sqlx::query_as(
                "SELECT filled_at,
                        CASE WHEN side IN ('buy', 'cover') THEN filled_qty
                             ELSE -filled_qty END
                   FROM paper_orders
                  WHERE paper_account_id = $1 AND symbol = $2
                    AND status = 'filled'
                    AND filled_at IS NOT NULL AND filled_qty IS NOT NULL",
            )
            .bind(account_id)
            .bind(&symbol)
            .fetch_all(pool)
            .await?;
            let signed: Vec<(NaiveDate, Decimal)> =
                fills.iter().map(|(at, q)| (at.date_naive(), *q)).collect();

            for ev in &recent {
                let qty = qty_on_ex_date(&signed, ev.ex_date);
                if qty.is_zero() {
                    continue;
                }
                let Ok(per_share) = Decimal::try_from(ev.amount_per_share) else {
                    continue;
                };
                let cash = (qty * per_share).round_dp(8);
                let mut tx = pool.begin().await?;
                let inserted = sqlx::query(
                    "INSERT INTO paper_dividends
                        (paper_account_id, symbol, ex_date, amount_per_share, qty, cash_credited)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     ON CONFLICT (paper_account_id, symbol, ex_date) DO NOTHING",
                )
                .bind(account_id)
                .bind(&symbol)
                .bind(ev.ex_date)
                .bind(per_share)
                .bind(qty)
                .bind(cash)
                .execute(&mut *tx)
                .await?
                .rows_affected();
                if inserted == 1 {
                    sqlx::query("UPDATE paper_accounts SET cash = cash + $1 WHERE id = $2")
                        .bind(cash)
                        .bind(account_id)
                        .execute(&mut *tx)
                        .await?;
                    written += 1;
                }
                tx.commit().await?;
                // DRIP: a POSITIVE credit on a drip-enabled account
                // reinvests immediately — a market buy of the credited
                // cash through the normal fill path, AFTER the credit
                // commits (a failed buy leaves the cash, honestly).
                // Short-position dividend DEBITS never reinvest.
                if inserted == 1 && cash > Decimal::ZERO {
                    let drip: Option<(bool, Uuid)> = sqlx::query_as(
                        "SELECT drip, user_id FROM paper_accounts WHERE id = $1",
                    )
                    .bind(account_id)
                    .fetch_optional(pool)
                    .await?;
                    if let Some((true, user_id)) = drip {
                        if let Ok(quote) = crate::market_data::quote(pool, &symbol).await {
                            if let Ok(price) = Decimal::try_from(quote.price) {
                                if price > Decimal::ZERO {
                                    let qty = (cash / price).round_dp(4);
                                    if qty > Decimal::ZERO {
                                        let buy = crate::paper::submit(
                                            pool,
                                            user_id,
                                            *account_id,
                                            crate::paper::OrderRequest {
                                                symbol: symbol.clone(),
                                                side: traderview_core::Side::Buy,
                                                qty,
                                                order_type: "market".into(),
                                                limit_price: None,
                                                stop_price: None,
                                                trail_value: None,
                                                trail_is_pct: None,
                                                time_in_force: None,
                                                expire_at: None,
                                                plan_note: None,
                                            },
                                        )
                                        .await;
                                        if buy.is_ok() {
                                            sqlx::query(
                                                "UPDATE paper_dividends SET reinvested = TRUE
                                                  WHERE paper_account_id = $1 AND symbol = $2 AND ex_date = $3",
                                            )
                                            .bind(account_id)
                                            .bind(&symbol)
                                            .bind(ev.ex_date)
                                            .execute(pool)
                                            .await
                                            .ok();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(written)
}

/// Credited dividends for an owned account, newest ex-date first.
pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<PaperDividend>> {
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    Ok(sqlx::query_as(
        "SELECT symbol, ex_date, amount_per_share, qty, cash_credited, credited_at
           FROM paper_dividends
          WHERE paper_account_id = $1
          ORDER BY ex_date DESC, symbol LIMIT $2",
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn fills_before_ex_date_count_fills_on_it_do_not() {
        // Buy 100 two days early, buy 50 ON the ex-date: only the 100
        // owns the dividend.
        let fills = vec![
            (d(2026, 6, 1), Decimal::from(100)),
            (d(2026, 6, 3), Decimal::from(50)),
        ];
        assert_eq!(qty_on_ex_date(&fills, d(2026, 6, 3)), Decimal::from(100));
    }

    #[test]
    fn sells_net_out_and_flat_books_get_nothing() {
        // 100 bought, 100 sold before the ex-date → flat → no credit.
        let fills = vec![
            (d(2026, 5, 1), Decimal::from(100)),
            (d(2026, 5, 20), Decimal::from(-100)),
        ];
        assert_eq!(qty_on_ex_date(&fills, d(2026, 6, 3)), Decimal::ZERO);
    }

    #[test]
    fn short_into_ex_date_is_negative() {
        // Short 40 (signed -40) held through the ex-date → the short
        // OWES the dividend: negative qty drives a cash debit.
        let fills = vec![(d(2026, 5, 28), Decimal::from(-40))];
        assert_eq!(qty_on_ex_date(&fills, d(2026, 6, 3)), Decimal::from(-40));
    }

    #[test]
    fn credit_window_excludes_future_and_stale_ex_dates() {
        let today = d(2026, 6, 11);
        assert!(in_credit_window(d(2026, 6, 11), today)); // today: creditable
        assert!(in_credit_window(d(2026, 5, 15), today)); // 27 days back: in window
        assert!(!in_credit_window(d(2026, 6, 12), today)); // future: not yet
        assert!(!in_credit_window(d(2026, 5, 12), today)); // exactly 30 days: aged out
    }
}
