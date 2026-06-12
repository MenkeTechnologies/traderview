//! Paper TWAP/VWAP/POV parent orders — time-sliced child market orders
//! through the existing paper engine.
//!
//! Semantics (deliberately the plainest versions):
//!   - 'twap' splits total_qty into `slices` equal child orders;
//!     'vwap' weights slices by the stylized intraday volume U-curve
//!     (heavy first/last slices, light middle) via the shared
//!     optimal_execution_vwap_schedule core. Either way, integer
//!     remainders ride on the LAST slice so the total is exact.
//!   - 'pov' sizes each child as participation_rate x the cumulative
//!     day-volume delta observed between ticks (quote cache is 60s, so
//!     pov intervals are >= 60s). No volume movement = no child, just a
//!     reschedule — when the tape is silent a POV algo does not trade.
//!     For pov, `slices` is the child-count safety cap; hitting it with
//!     quantity unfilled marks the parent 'capped', never a fake done.
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
    /// 'twap' (default), 'vwap', or 'pov'.
    #[serde(default)]
    pub style: Option<String>,
    /// Required for 'pov': fraction of observed volume to take, (0, 0.5].
    #[serde(default)]
    pub participation_rate: Option<Decimal>,
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
    pub participation_rate: Option<Decimal>,
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

/// Dispatch by stored style ('pov' sizes off the tape, not here).
pub fn styled_slice_qty(style: &str, total: Decimal, slices: i32, index: i32) -> Decimal {
    match style {
        "vwap" => vwap_slice_qty(total, slices, index),
        _ => slice_qty(total, slices, index),
    }
}

/// POV child size: floor(rate x volume delta), clamped to what's left.
/// Negative deltas (day rollover resets cumulative volume) count as no
/// observable volume.
pub fn pov_child_qty(rate: Decimal, delta_volume: i64, remaining: Decimal) -> Decimal {
    if delta_volume <= 0 || rate <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    (rate * Decimal::from(delta_volume)).floor().min(remaining)
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
    let style = inp.style.as_deref().unwrap_or("twap");
    if !matches!(style, "twap" | "vwap" | "pov") {
        anyhow::bail!("style must be 'twap', 'vwap' or 'pov'");
    }
    // For pov, `slices` is only a child cap — small totals are fine.
    if style != "pov" && Decimal::from(inp.slices) > inp.total_qty {
        anyhow::bail!("more slices than units to fill");
    }
    let rate = if style == "pov" {
        let r = inp
            .participation_rate
            .ok_or_else(|| anyhow::anyhow!("pov requires participation_rate"))?;
        if r <= Decimal::ZERO || r > Decimal::new(5, 1) {
            anyhow::bail!("participation_rate must be in (0, 0.5]");
        }
        // The quote cache refreshes every 60s — faster pov ticks would
        // read the same cumulative volume and trade nothing.
        if inp.interval_seconds < 60 {
            anyhow::bail!("pov interval_seconds must be >= 60 (quote cache granularity)");
        }
        Some(r)
    } else {
        None
    };
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
            (user_id, account_id, symbol, side, total_qty, slices, interval_seconds, style, participation_rate, next_slice_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
         RETURNING id, account_id, symbol, side, total_qty, slices, interval_seconds, style, participation_rate,
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
    .bind(rate)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<ParentOrder>> {
    Ok(sqlx::query_as(
        "SELECT id, account_id, symbol, side, total_qty, slices, interval_seconds, style, participation_rate,
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
        participation_rate: Option<Decimal>,
        last_market_volume: Option<i64>,
        slices_filled: i32,
        qty_filled: Decimal,
    }
    let due: Vec<Due> = sqlx::query_as(
        "SELECT id, user_id, account_id, symbol, side, total_qty, slices,
                interval_seconds, style, participation_rate, last_market_volume,
                slices_filled, qty_filled
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
        // Size this child. POV reads the tape and may legitimately have
        // nothing to do this tick (no volume yet / silent tape).
        let (qty, observed_volume) = if d.style == "pov" {
            let vol = match crate::market_data::quote(pool, &d.symbol).await {
                Ok(q) => q.volume,
                Err(e) => {
                    mark_error(pool, d.id, &e.to_string()).await;
                    continue;
                }
            };
            let Some(vol) = vol else {
                // No volume on the quote (e.g. some non-equity feeds) —
                // POV cannot pace without it.
                mark_error(pool, d.id, "quote has no volume; pov needs a volume feed").await;
                continue;
            };
            let Some(prev) = d.last_market_volume else {
                // First tick only sets the baseline.
                reschedule(pool, d.id, d.interval_seconds, Some(vol)).await;
                continue;
            };
            let rate = d.participation_rate.unwrap_or_default();
            let qty = pov_child_qty(rate, vol - prev, d.total_qty - d.qty_filled);
            if qty < Decimal::ONE {
                reschedule(pool, d.id, d.interval_seconds, Some(vol)).await;
                continue;
            }
            (qty, Some(vol))
        } else {
            (
                styled_slice_qty(&d.style, d.total_qty, d.slices, d.slices_filled),
                None,
            )
        };
        let req = crate::paper::OrderRequest {
            symbol: d.symbol.clone(),
            side,
            qty,
            order_type: "market".into(),
            limit_price: None,
            stop_price: None,
            trail_value: None,
            trail_is_pct: None,
            time_in_force: None,
            expire_at: None,
            plan_note: None,
        };
        match crate::paper::submit(pool, d.user_id, d.account_id, req).await {
            Ok(_) => {
                submitted += 1;
                let filled = d.slices_filled + 1;
                let done = if d.style == "pov" {
                    d.qty_filled + qty >= d.total_qty
                } else {
                    filled >= d.slices
                };
                // POV child cap hit with quantity unfilled: 'capped',
                // never a fake 'done'.
                let capped = !done && d.style == "pov" && filled >= d.slices;
                let unfilled = d.total_qty - d.qty_filled - qty;
                sqlx::query(
                    "UPDATE paper_parent_orders
                        SET slices_filled = $2,
                            qty_filled = $3,
                            status = CASE WHEN $4 THEN 'done' WHEN $6 THEN 'capped' ELSE status END,
                            last_error = CASE WHEN $6 THEN 'child cap reached with ' || $7 || ' unfilled' ELSE last_error END,
                            last_market_volume = COALESCE($8, last_market_volume),
                            next_slice_at = now() + ($5 || ' seconds')::interval
                      WHERE id = $1",
                )
                .bind(d.id)
                .bind(filled)
                .bind(d.qty_filled + qty)
                .bind(done)
                .bind(d.interval_seconds.to_string())
                .bind(capped)
                .bind(unfilled.to_string())
                .bind(observed_volume)
                .execute(pool)
                .await
                .ok();
            }
            Err(e) => mark_error(pool, d.id, &e.to_string()).await,
        }
    }
    Ok(submitted)
}

/// Push the next tick out one interval, optionally storing the freshly
/// observed cumulative volume as the POV baseline.
async fn reschedule(pool: &PgPool, id: Uuid, interval_seconds: i32, volume: Option<i64>) {
    sqlx::query(
        "UPDATE paper_parent_orders
            SET next_slice_at = now() + ($2 || ' seconds')::interval,
                last_market_volume = COALESCE($3, last_market_volume)
          WHERE id = $1",
    )
    .bind(id)
    .bind(interval_seconds.to_string())
    .bind(volume)
    .execute(pool)
    .await
    .ok();
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
    fn pov_child_takes_rate_times_delta_floored() {
        // 10% of 5,432 traded shares -> 543 (floored), well under remaining.
        let q = pov_child_qty(Decimal::new(1, 1), 5432, Decimal::from(10_000));
        assert_eq!(q, Decimal::from(543));
    }

    #[test]
    fn pov_child_clamps_to_remaining() {
        // 25% of 1M shares wants 250k but only 800 remain.
        let q = pov_child_qty(Decimal::new(25, 2), 1_000_000, Decimal::from(800));
        assert_eq!(q, Decimal::from(800));
    }

    #[test]
    fn pov_silent_tape_and_day_rollover_trade_nothing() {
        let rate = Decimal::new(1, 1);
        assert_eq!(pov_child_qty(rate, 0, Decimal::from(100)), Decimal::ZERO);
        // Cumulative volume reset (new session) shows as a negative delta.
        assert_eq!(pov_child_qty(rate, -50_000, Decimal::from(100)), Decimal::ZERO);
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
