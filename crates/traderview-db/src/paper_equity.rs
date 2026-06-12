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
            // Option positions mark at the chain mid x 100 multiplier;
            // equities at the cached quote.
            if let Some(occ) = traderview_core::occ_symbol::parse(symbol) {
                match crate::paper::option_quote(&occ).await {
                    Ok(Some(p)) => match Decimal::try_from(p * 100.0) {
                        Ok(v) => position_value += v * qty,
                        Err(_) => all_marked = false,
                    },
                    _ => all_marked = false,
                }
                continue;
            }
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

/// Return vs the account's starting cash — the honest cross-account
/// comparison base (snapshot-series-relative return would flatter an
/// account that lost money before its first sample). None when
/// starting cash is degenerate.
pub fn account_return_pct(starting_cash: Decimal, equity: Decimal) -> Option<f64> {
    if starting_cash <= Decimal::ZERO {
        return None;
    }
    let s: f64 = starting_cash.to_string().parse().ok()?;
    let e: f64 = equity.to_string().parse().ok()?;
    Some((e / s - 1.0) * 100.0)
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountComparison {
    pub account_id: Uuid,
    pub name: String,
    pub starting_cash: Decimal,
    pub equity: Decimal,
    pub return_pct: Option<f64>,
    pub max_drawdown_pct: Option<f64>,
    pub currently_underwater: bool,
    pub snapshots: i64,
}

/// Strategy leaderboard: one row per account, ranked by return vs
/// starting cash (descending; unranked accounts last). Equity = latest
/// snapshot when one exists, else cash (fresh account, nothing held).
pub async fn compare(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<AccountComparison>> {
    let accounts: Vec<(Uuid, String, Decimal, Decimal)> = sqlx::query_as(
        "SELECT id, name, starting_cash, cash FROM paper_accounts
          WHERE user_id = $1 ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let mut rows = Vec::with_capacity(accounts.len());
    for (account_id, name, starting_cash, cash) in accounts {
        let snaps: Vec<(Decimal,)> = sqlx::query_as(
            "SELECT equity FROM paper_equity_snapshots
              WHERE paper_account_id = $1 ORDER BY taken_at",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
        let equity = snaps.last().map(|(e,)| *e).unwrap_or(cash);
        let series: Vec<f64> = snaps
            .iter()
            .filter_map(|(e,)| e.to_string().parse().ok())
            .collect();
        let summary = summarize(&series);
        rows.push(AccountComparison {
            account_id,
            name,
            starting_cash,
            equity,
            return_pct: account_return_pct(starting_cash, equity),
            max_drawdown_pct: summary.as_ref().map(|s| s.max_drawdown_pct),
            currently_underwater: summary.map(|s| s.currently_underwater).unwrap_or(false),
            snapshots: snaps.len() as i64,
        });
    }
    rows.sort_by(|a, b| {
        b.return_pct
            .unwrap_or(f64::NEG_INFINITY)
            .total_cmp(&a.return_pct.unwrap_or(f64::NEG_INFINITY))
    });
    Ok(rows)
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolAttribution {
    pub symbol: String,
    /// Realized PnL of CLOSED round trips (FIFO from the fill ledger,
    /// fees netted; options scaled by the 100× multiplier).
    pub trading_pnl: f64,
    pub closed_trips: usize,
    pub dividends: f64,
    pub fees: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Attribution {
    /// Ranked by |total contribution| descending.
    pub symbols: Vec<SymbolAttribution>,
    pub total_trading_pnl: f64,
    pub total_dividends: f64,
    pub total_fees: f64,
}

/// Where the account's P&L came from, per symbol: closed-trip trading
/// PnL + dividends − fees. Reconstructed from the fill ledger because
/// paper_positions deletes a row when it closes to zero — realized
/// PnL of closed positions lives nowhere else. Open positions'
/// unrealized PnL is deliberately NOT included: this is the realized
/// record, and the positions table already shows live unrealized.
pub async fn attribution(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
) -> anyhow::Result<Attribution> {
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let fills: Vec<(String, String, Decimal, Decimal, Decimal, chrono::DateTime<Utc>)> =
        sqlx::query_as(
            "SELECT symbol, side::text, filled_qty, filled_price, fee, filled_at
               FROM paper_orders
              WHERE paper_account_id = $1 AND status = 'filled'
                AND filled_qty IS NOT NULL AND filled_price IS NOT NULL
              ORDER BY filled_at",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
    use rust_decimal::prelude::ToPrimitive;
    let mut by_symbol: std::collections::BTreeMap<String, (Vec<traderview_core::live_vs_backtest::Fill>, f64)> =
        Default::default();
    for (symbol, side, qty, price, fee, at) in fills {
        // Options: pre-scale the per-share price by the 100× multiplier
        // so trip PnL is dollar-true while commissions (already dollars)
        // stay unscaled inside the reconstruction.
        let scale = if traderview_core::occ_symbol::is_occ(&symbol) { 100.0 } else { 1.0 };
        let entry = by_symbol.entry(symbol).or_default();
        entry.1 += fee.to_f64().unwrap_or(0.0);
        entry.0.push(traderview_core::live_vs_backtest::Fill {
            buy: side == "buy" || side == "cover",
            qty: qty.to_f64().unwrap_or(0.0),
            price: price.to_f64().unwrap_or(0.0) * scale,
            commission: fee.to_f64().unwrap_or(0.0),
            ts: at.timestamp(),
        });
    }
    let divs: Vec<(String, Decimal)> = sqlx::query_as(
        "SELECT symbol, COALESCE(SUM(cash_credited), 0)
           FROM paper_dividends WHERE paper_account_id = $1 GROUP BY symbol",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    let div_map: std::collections::BTreeMap<String, f64> = divs
        .into_iter()
        .map(|(s, c)| (s, c.to_f64().unwrap_or(0.0)))
        .collect();
    let mut symbols = Vec::new();
    let (mut tt, mut td, mut tf) = (0.0, 0.0, 0.0);
    let mut seen: std::collections::BTreeSet<String> = Default::default();
    for (symbol, (fills, fees)) in &by_symbol {
        let trips = traderview_core::live_vs_backtest::round_trips(fills);
        let trading_pnl: f64 = trips.iter().map(|t| t.pnl).sum();
        let dividends = div_map.get(symbol).copied().unwrap_or(0.0);
        seen.insert(symbol.clone());
        tt += trading_pnl;
        td += dividends;
        tf += fees;
        symbols.push(SymbolAttribution {
            symbol: symbol.clone(),
            trading_pnl,
            closed_trips: trips.len(),
            dividends,
            fees: *fees,
        });
    }
    // Dividend-only symbols (position opened elsewhere/now closed with
    // no fills in range, or credited after full exit) still appear.
    for (symbol, dividends) in &div_map {
        if !seen.contains(symbol) {
            td += dividends;
            symbols.push(SymbolAttribution {
                symbol: symbol.clone(),
                trading_pnl: 0.0,
                closed_trips: 0,
                dividends: *dividends,
                fees: 0.0,
            });
        }
    }
    symbols.sort_by(|a, b| {
        (b.trading_pnl + b.dividends)
            .abs()
            .total_cmp(&(a.trading_pnl + a.dividends).abs())
    });
    Ok(Attribution {
        symbols,
        total_trading_pnl: tt,
        total_dividends: td,
        total_fees: tf,
    })
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

    #[test]
    fn return_vs_starting_cash_pins_sign_and_zero_guard() {
        let d = |v: i64| Decimal::from(v);
        assert!((account_return_pct(d(200_000), d(220_000)).unwrap() - 10.0).abs() < 1e-9);
        assert!((account_return_pct(d(200_000), d(150_000)).unwrap() + 25.0).abs() < 1e-9);
        assert!(account_return_pct(Decimal::ZERO, d(100)).is_none());
    }
}
