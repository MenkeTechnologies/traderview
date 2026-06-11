//! Paper TWAP/VWAP parent orders — time-sliced child market orders
//! through the existing paper engine.
//!
//! Semantics (deliberately the plainest versions):
//!   - 'twap' splits total_qty into `slices` equal child orders;
//!     'vwap' weights slices by the stylized intraday volume U-curve
//!     (heavy first/last slices, light middle) via the shared
//!     optimal_execution_vwap_schedule core. Either way, integer
//!     remainders ride on the LAST slice so the total is exact.
//!   - the first slice fires immediately on the next tick, then one
//!     per `interval_seconds`.
//!   - children are MARKET orders through `paper::submit`, so fills
//!     inherit the engine's friction model (slippage + fees).
//!   - a failed child (no quote, insufficient funds…) marks the parent
//!     `error` and STOPS — a sim that silently retries forever teaches
//!     the wrong lesson about execution risk.
//!
//! The background ticker calls `tick` every few seconds.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use traderview_core::Side;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct ParentOrderInput {
    pub symbol: String,
    pub side: Side,
    pub total_qty: Decimal,
    pub slices: i32,
    pub interval_seconds: i32,
    /// 'twap' (default) or 'vwap'.
    #[serde(default)]
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ParentOrder {
    pub id: Uuid,
    pub account_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub total_qty: Decimal,
    pub slices: i32,
    pub interval_seconds: i32,
    pub style: String,
    pub slices_filled: i32,
    pub qty_filled: Decimal,
    pub status: String,
    pub last_error: Option<String>,
    pub next_slice_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Equal split with the remainder on the last slice — pure, exact.
pub fn slice_qty(total: Decimal, slices: i32, index: i32) -> Decimal {
    let n = Decimal::from(slices.max(1));
    let base = (total / n).floor();
    if index >= slices - 1 {
        total - base * Decimal::from(slices - 1)
    } else {
        base
    }
}

/// Stylized intraday volume U-curve — heavy open and close, quiet
/// middle: w(x) = 1 + 2·(2x−1)² over x ∈ [0,1] (endpoints 3×, midday
/// 1×). The standard shape VWAP schedulers assume absent a live
/// per-symbol profile.
fn u_curve(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let x = if n > 1 { i as f64 / (n - 1) as f64 } else { 0.5 };
            1.0 + 2.0 * (2.0 * x - 1.0).powi(2)
        })
        .collect()
}

/// VWAP-style split: slice i proportional to the U-curve weight,
/// floored, with the exact remainder riding the LAST slice (same
/// exactness invariant as the TWAP split). Proportions come from the
/// shared optimal_execution_vwap_schedule core.
pub fn vwap_slice_qty(total: Decimal, slices: i32, index: i32) -> Decimal {
    let n = slices.max(1) as usize;
    let total_f = total.to_string().parse::<f64>().unwrap_or(0.0);
    let Some(report) =
        traderview_core::optimal_execution_vwap_schedule::compute(total_f, &u_curve(n))
    else {
        return slice_qty(total, slices, index);
    };
    if index >= slices - 1 {
        // Exact remainder after the floored earlier slices.
        let mut consumed = Decimal::ZERO;
        for s in &report.slices[..n - 1] {
            consumed += Decimal::try_from(s.floor()).unwrap_or_default();
        }
        total - consumed
    } else {
        Decimal::try_from(report.slices[index as usize].floor()).unwrap_or_default()
    }
}

/// Dispatch by stored style.
pub fn styled_slice_qty(style: &str, total: Decimal, slices: i32, index: i32) -> Decimal {
    match style {
        "vwap" => vwap_slice_qty(total, slices, index),
        _ => slice_qty(total, slices, index),
    }
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    inp: &ParentOrderInput,
) -> anyhow::Result<ParentOrder> {
    let sym = inp.symbol.trim().to_uppercase();
    if sym.is_empty() || sym.len() > 20 {
        anyhow::bail!("invalid symbol");
    }
    if inp.total_qty <= Decimal::ZERO {
        anyhow::bail!("total_qty must be positive");
    }
    if !(2..=100).contains(&inp.slices) {
        anyhow::bail!("slices must be in 2..=100");
    }
    if !(5..=3600).contains(&inp.interval_seconds) {
        anyhow::bail!("interval_seconds must be in 5..=3600");
    }
    if Decimal::from(inp.slices) > inp.total_qty {
        anyhow::bail!("more slices than units to fill");
    }
    let style = inp.style.as_deref().unwrap_or("twap");
    if !matches!(style, "twap" | "vwap") {
        anyhow::bail!("style must be 'twap' or 'vwap'");
    }
    // VWAP floors each slice toward the U-curve weight; a midday slice
    // of a thin order can floor to zero. Require ≥1 unit per slice at
    // the LIGHTEST weight (midday 1× of total weight) so every child
    // is a real order.
    if style == "vwap" {
        let zero_slice = (0..inp.slices)
            .any(|i| vwap_slice_qty(inp.total_qty, inp.slices, i) < Decimal::ONE);
        if zero_slice {
            anyhow::bail!("total_qty too small for a vwap split into this many slices");
        }
    }
    // Ownership check (submit re-checks per child as well).
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let side = serde_json::to_value(inp.side)?
        .as_str()
        .unwrap_or("buy")
        .to_string();
    let row: ParentOrder = sqlx::query_as(
        "INSERT INTO paper_parent_orders
            (user_id, account_id, symbol, side, total_qty, slices, interval_seconds, style, next_slice_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
         RETURNING id, account_id, symbol, side, total_qty, slices, interval_seconds, style,
                   slices_filled, qty_filled, status, last_error, next_slice_at, created_at",
    )
    .bind(user_id)
    .bind(account_id)
    .bind(&sym)
    .bind(&side)
    .bind(inp.total_qty)
    .bind(inp.slices)
    .bind(inp.interval_seconds)
    .bind(style)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<ParentOrder>> {
    Ok(sqlx::query_as(
        "SELECT id, account_id, symbol, side, total_qty, slices, interval_seconds, style,
                slices_filled, qty_filled, status, last_error, next_slice_at, created_at
           FROM paper_parent_orders
          WHERE user_id = $1
          ORDER BY created_at DESC
          LIMIT 100",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn cancel(pool: &PgPool, user_id: Uuid, id: Uuid) -> anyhow::Result<bool> {
    let res = sqlx::query(
        "UPDATE paper_parent_orders SET status = 'cancelled'
          WHERE id = $1 AND user_id = $2 AND status = 'working'",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// One ticker pass: submit a child for every due working parent.
/// Returns the number of children submitted.
pub async fn tick(pool: &PgPool) -> anyhow::Result<usize> {
    #[derive(sqlx::FromRow)]
    struct Due {
        id: Uuid,
        user_id: Uuid,
        account_id: Uuid,
        symbol: String,
        side: String,
        total_qty: Decimal,
        slices: i32,
        interval_seconds: i32,
        style: String,
        slices_filled: i32,
        qty_filled: Decimal,
    }
    let due: Vec<Due> = sqlx::query_as(
        "SELECT id, user_id, account_id, symbol, side, total_qty, slices,
                interval_seconds, style, slices_filled, qty_filled
           FROM paper_parent_orders
          WHERE status = 'working' AND next_slice_at <= now()
          ORDER BY next_slice_at
          LIMIT 50",
    )
    .fetch_all(pool)
    .await?;
    let mut submitted = 0usize;
    for d in due {
        let side: Side = match serde_json::from_value(serde_json::Value::String(d.side.clone())) {
            Ok(s) => s,
            Err(_) => {
                mark_error(pool, d.id, "unparseable side").await;
                continue;
            }
        };
        let qty = styled_slice_qty(&d.style, d.total_qty, d.slices, d.slices_filled);
        let req = crate::paper::OrderRequest {
            symbol: d.symbol.clone(),
            side,
            qty,
            order_type: "market".into(),
            limit_price: None,
            stop_price: None,
        };
        match crate::paper::submit(pool, d.user_id, d.account_id, req).await {
            Ok(_) => {
                submitted += 1;
                let filled = d.slices_filled + 1;
                let done = filled >= d.slices;
                sqlx::query(
                    "UPDATE paper_parent_orders
                        SET slices_filled = $2,
                            qty_filled = $3,
                            status = CASE WHEN $4 THEN 'done' ELSE status END,
                            next_slice_at = now() + ($5 || ' seconds')::interval
                      WHERE id = $1",
                )
                .bind(d.id)
                .bind(filled)
                .bind(d.qty_filled + qty)
                .bind(done)
                .bind(d.interval_seconds.to_string())
                .execute(pool)
                .await
                .ok();
            }
            Err(e) => mark_error(pool, d.id, &e.to_string()).await,
        }
    }
    Ok(submitted)
}

async fn mark_error(pool: &PgPool, id: Uuid, msg: &str) {
    sqlx::query(
        "UPDATE paper_parent_orders SET status = 'error', last_error = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(msg)
    .execute(pool)
    .await
    .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_slices_with_remainder_on_the_last() {
        // 1000 over 3: 333, 333, 334 — exact total.
        let total = Decimal::from(1000);
        let parts: Vec<Decimal> = (0..3).map(|i| slice_qty(total, 3, i)).collect();
        assert_eq!(parts[0], Decimal::from(333));
        assert_eq!(parts[1], Decimal::from(333));
        assert_eq!(parts[2], Decimal::from(334));
        assert_eq!(parts.iter().copied().sum::<Decimal>(), total);
    }

    #[test]
    fn even_division_has_no_remainder() {
        let total = Decimal::from(100);
        let parts: Vec<Decimal> = (0..4).map(|i| slice_qty(total, 4, i)).collect();
        assert!(parts.iter().all(|p| *p == Decimal::from(25)));
    }

    #[test]
    fn single_unit_slices_never_zero_out_early() {
        // 5 units over 4 slices: 1,1,1,2.
        let total = Decimal::from(5);
        let parts: Vec<Decimal> = (0..4).map(|i| slice_qty(total, 4, i)).collect();
        assert_eq!(parts, vec![1.into(), 1.into(), 1.into(), 2.into()]);
        assert_eq!(parts.iter().copied().sum::<Decimal>(), total);
    }

    #[test]
    fn vwap_slices_sum_exactly_to_total() {
        let total = Decimal::from(1000);
        let parts: Vec<Decimal> = (0..7).map(|i| vwap_slice_qty(total, 7, i)).collect();
        assert_eq!(parts.iter().copied().sum::<Decimal>(), total);
    }

    #[test]
    fn vwap_slices_follow_the_u_shape() {
        // n=5 U-curve weights: 3, 1.5, 1, 1.5, 3 (sum 10). With total
        // 900 every slice is exact: 270, 135, 90, 135, 270.
        let total = Decimal::from(900);
        let parts: Vec<Decimal> = (0..5).map(|i| vwap_slice_qty(total, 5, i)).collect();
        let expect: Vec<Decimal> =
            [270, 135, 90, 135, 270].iter().map(|&v| Decimal::from(v)).collect();
        assert_eq!(parts, expect);
    }

    #[test]
    fn styled_dispatch_twap_matches_plain_split() {
        let total = Decimal::from(1000);
        for i in 0..3 {
            assert_eq!(
                styled_slice_qty("twap", total, 3, i),
                slice_qty(total, 3, i)
            );
        }
    }
}
