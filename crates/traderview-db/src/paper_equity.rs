//! Paper account equity history — background samples of cash + marked
//! position value. Unchanged readings are skipped (a flat account adds
//! no information), so the curve compresses to its turning points plus
//! one sample per change. Summary stats delegate to the shared
//! drawdown_episodes core.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct EquitySnapshot {
    pub equity: Decimal,
    pub cash: Decimal,
    pub position_value: Decimal,
    pub taken_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquitySummary {
    pub return_pct: f64,
    pub max_drawdown_pct: f64,
    pub currently_underwater: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquityHistory {
    pub snapshots: Vec<EquitySnapshot>,
    pub summary: Option<EquitySummary>,
}

/// Summary over an equity series: total return plus worst drawdown via
/// the shared drawdown_episodes core. None for fewer than 2 points.
pub fn summarize(series: &[f64]) -> Option<EquitySummary> {
    if series.len() < 2 || series[0] <= 0.0 {
        return None;
    }
    let report = traderview_core::drawdown_episodes::compute(series, 1)?;
    // The core reports depths as NEGATIVE percentages; expose the worst
    // as a positive magnitude.
    let worst = report
        .episodes
        .first()
        .map(|e| e.depth_pct)
        .unwrap_or(0.0)
        .min(report.current_drawdown_pct);
    Some(EquitySummary {
        return_pct: (series[series.len() - 1] / series[0] - 1.0) * 100.0,
        max_drawdown_pct: -worst,
        currently_underwater: report.currently_underwater,
    })
}

/// One sampling pass over ALL paper accounts. Marks positions at the
/// cached quote (held symbols only) and inserts a snapshot when the
/// equity differs from the previous one. Returns snapshots written.
pub async fn snapshot_all(pool: &PgPool) -> anyhow::Result<usize> {
    let accounts: Vec<(Uuid, Decimal)> =
        sqlx::query_as("SELECT id, cash FROM paper_accounts").fetch_all(pool).await?;
    let mut written = 0usize;
    for (account_id, cash) in accounts {
        let positions: Vec<(String, Decimal)> = sqlx::query_as(
            "SELECT symbol, qty FROM paper_positions WHERE paper_account_id = $1",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
        let mut position_value = Decimal::ZERO;
        let mut all_marked = true;
        for (symbol, qty) in &positions {
            match crate::market_data::quote(pool, symbol).await {
                Ok(q) => match Decimal::try_from(q.price) {
                    Ok(p) => position_value += p * qty,
                    Err(_) => all_marked = false,
                },
                Err(_) => all_marked = false,
            }
        }
        // A partially-marked book would write a fake dip into the
        // curve — skip the account this pass instead of lying.
        if !all_marked {
            continue;
        }
        let equity = cash + position_value;
        let last: Option<(Decimal,)> = sqlx::query_as(
            "SELECT equity FROM paper_equity_snapshots
              WHERE paper_account_id = $1
              ORDER BY taken_at DESC LIMIT 1",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
        if matches!(last, Some((e,)) if e == equity) {
            continue;
        }
        sqlx::query(
            "INSERT INTO paper_equity_snapshots
                (paper_account_id, equity, cash, position_value)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(account_id)
        .bind(equity)
        .bind(cash)
        .bind(position_value)
        .execute(pool)
        .await?;
        written += 1;
    }
    Ok(written)
}

/// Equity history for an owned account, oldest first, with summary.
pub async fn history(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    limit: i64,
) -> anyhow::Result<EquityHistory> {
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let snapshots: Vec<EquitySnapshot> = sqlx::query_as(
        "SELECT equity, cash, position_value, taken_at
           FROM (SELECT equity, cash, position_value, taken_at
                   FROM paper_equity_snapshots
                  WHERE paper_account_id = $1
                  ORDER BY taken_at DESC LIMIT $2) t
          ORDER BY taken_at",
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    let series: Vec<f64> = snapshots
        .iter()
        .filter_map(|s| s.equity.to_string().parse().ok())
        .collect();
    let summary = summarize(&series);
    Ok(EquityHistory { snapshots, summary })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_pins_return_and_worst_drawdown() {
        // 100 → 120 (peak) → 90 (-25% from peak) → 110: return +10%,
        // max drawdown 25%, recovered above 90 but still under the 120
        // high-water mark → underwater.
        let s = summarize(&[100.0, 120.0, 90.0, 110.0]).unwrap();
        assert!((s.return_pct - 10.0).abs() < 1e-9);
        assert!((s.max_drawdown_pct - 25.0).abs() < 1e-9);
        assert!(s.currently_underwater);
    }

    #[test]
    fn monotonic_curve_has_zero_drawdown() {
        let s = summarize(&[100.0, 105.0, 111.0]).unwrap();
        assert!((s.return_pct - 11.0).abs() < 1e-9);
        assert_eq!(s.max_drawdown_pct, 0.0);
        assert!(!s.currently_underwater);
    }

    #[test]
    fn too_short_or_degenerate_series_is_none() {
        assert!(summarize(&[100.0]).is_none());
        assert!(summarize(&[]).is_none());
        assert!(summarize(&[0.0, 100.0]).is_none());
    }
}
