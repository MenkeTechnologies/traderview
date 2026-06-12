//! Autotrade exit logic.
//!
//! Two rules for closing autotrade-opened paper positions:
//!
//!   * **Time-stop** — flatten after `max_holding_days` past
//!     `opened_at`. Default 20 days (the horizon Kelly defaults to).
//!   * **Signal-degradation** — if the source confluence row scores
//!     below `min_score` for `degradation_threshold_checks` consecutive
//!     evaluations, flatten. Default 3 checks.
//!
//! Manual positions are never touched — only rows in
//! `autotrade_position_tags` qualify.
//!
//! Pure compute pins the decision logic in `should_exit`, which is
//! used by `sweep_exits` to decide which positions to flatten. The
//! repository layer handles tag persistence + the paper::submit call.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct PositionTag {
    pub id: i64,
    pub paper_account_id: Uuid,
    pub symbol: String,
    pub opened_by_log_id: Option<i64>,
    pub opened_at: DateTime<Utc>,
    pub score_at_open: f64,
    pub last_observed_score: Option<f64>,
    pub consecutive_degraded_checks: i32,
    pub last_evaluated_at: Option<DateTime<Utc>>,
    pub entry_price: Option<f64>,
    pub high_water_mark_price: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    StopLoss,
    TakeProfit,
    TrailingStop,
    TimeStop,
    SignalDegraded,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExitDecision {
    pub symbol: String,
    pub reason: ExitReason,
    pub days_held: i64,
    pub last_observed_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SweepResult {
    pub considered: usize,
    pub flattened: Vec<FlattenedRow>,
    pub held: Vec<HeldRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlattenedRow {
    pub symbol: String,
    pub reason: ExitReason,
    pub days_held: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeldRow {
    pub symbol: String,
    pub days_held: i64,
    pub consecutive_degraded_checks: i32,
    pub last_observed_score: Option<f64>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Decide whether to flatten a tagged position. Time-stop takes priority
/// because it's a hard rule; signal degradation is a soft rule that only
/// fires after enough consecutive degraded checks.
///
/// `current_score` is `Some(s)` when the symbol still appears in the
/// confluence ranking, `None` when it's dropped out entirely (which
/// counts as a degraded check — no longer signaling at all).
pub fn should_exit(
    opened_at: DateTime<Utc>,
    consecutive_degraded_checks: i32,
    current_score: Option<f64>,
    min_score: f64,
    max_holding_days: i32,
    degradation_threshold_checks: i32,
    now: DateTime<Utc>,
) -> Option<ExitDecision> {
    let days_held = (now - opened_at).num_days();
    if max_holding_days > 0 && days_held >= max_holding_days as i64 {
        return Some(ExitDecision {
            symbol: String::new(),
            reason: ExitReason::TimeStop,
            days_held,
            last_observed_score: current_score,
        });
    }
    // Compute "would this check be degraded?" — current < min_score OR
    // dropped from ranking entirely (None).
    let is_degraded = match current_score {
        None => true,
        Some(s) => s < min_score,
    };
    let new_consec = if is_degraded {
        consecutive_degraded_checks + 1
    } else {
        0
    };
    if new_consec >= degradation_threshold_checks {
        return Some(ExitDecision {
            symbol: String::new(),
            reason: ExitReason::SignalDegraded,
            days_held,
            last_observed_score: current_score,
        });
    }
    None
}

/// Increment the consecutive degraded counter or reset it to zero,
/// depending on the latest observation.
pub fn next_consecutive_degraded(prior: i32, is_degraded: bool) -> i32 {
    if is_degraded {
        prior + 1
    } else {
        0
    }
}

/// Price-driven exit check. Evaluates in priority order:
///   1. Take-profit (positive — close winner)
///   2. Stop-loss / trailing-stop (negative — cut loser)
///
/// All percentages are positive numbers in [0, 100] (5.0 = 5%).
/// Returns `None` when no SL/TP rule fires or inputs are invalid.
pub fn should_exit_price_driven(
    entry_price: f64,
    current_price: f64,
    high_water_mark: f64,
    stop_loss_pct: f64,
    take_profit_pct: f64,
    trailing_stop_enabled: bool,
    trailing_stop_pct: f64,
) -> Option<ExitReason> {
    if !(entry_price > 0.0 && current_price > 0.0) {
        return None;
    }
    // Compare directly against thresholds derived from entry_price to
    // avoid floating-point precision loss in (current/entry - 1) × 100.
    if take_profit_pct > 0.0 {
        let tp_threshold = entry_price * (1.0 + take_profit_pct / 100.0);
        if current_price >= tp_threshold {
            return Some(ExitReason::TakeProfit);
        }
    }
    if trailing_stop_enabled && trailing_stop_pct > 0.0 && high_water_mark > 0.0 {
        let trail_threshold = high_water_mark * (1.0 - trailing_stop_pct / 100.0);
        if current_price <= trail_threshold {
            return Some(ExitReason::TrailingStop);
        }
    } else if stop_loss_pct > 0.0 {
        // SL is silenced when trailing is on — they're mutually exclusive
        // gates on the downside (use one or the other, not both).
        let sl_threshold = entry_price * (1.0 - stop_loss_pct / 100.0);
        if current_price <= sl_threshold {
            return Some(ExitReason::StopLoss);
        }
    }
    None
}

/// New high-water-mark price for trailing-stop tracking. Takes the max
/// of prior HWM and current; falls back to current when no prior.
pub fn next_high_water_mark(prior: Option<f64>, current: f64) -> f64 {
    if !current.is_finite() || current <= 0.0 {
        return prior.unwrap_or(0.0);
    }
    match prior {
        Some(p) if p > current => p,
        _ => current,
    }
}

// ─── Repository ────────────────────────────────────────────────────────────

pub async fn insert_tag(
    pool: &PgPool,
    paper_account_id: Uuid,
    symbol: &str,
    opened_by_log_id: Option<i64>,
    score_at_open: f64,
    entry_price: f64,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO autotrade_position_tags
            (paper_account_id, symbol, opened_by_log_id, score_at_open,
             entry_price, high_water_mark_price)
         VALUES ($1, $2, $3, $4, $5, $5)
         ON CONFLICT (paper_account_id, symbol) DO UPDATE SET
            opened_by_log_id = EXCLUDED.opened_by_log_id,
            opened_at        = now(),
            score_at_open    = EXCLUDED.score_at_open,
            entry_price      = EXCLUDED.entry_price,
            high_water_mark_price = EXCLUDED.high_water_mark_price,
            consecutive_degraded_checks = 0",
    )
    .bind(paper_account_id)
    .bind(symbol)
    .bind(opened_by_log_id)
    .bind(score_at_open)
    .bind(entry_price)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_tags(pool: &PgPool, paper_account_id: Uuid) -> anyhow::Result<Vec<PositionTag>> {
    type Row = (
        i64,
        Uuid,
        String,
        Option<i64>,
        DateTime<Utc>,
        f64,
        Option<f64>,
        i32,
        Option<DateTime<Utc>>,
        Option<f64>,
        Option<f64>,
    );
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, paper_account_id, symbol, opened_by_log_id, opened_at,
                score_at_open, last_observed_score,
                consecutive_degraded_checks, last_evaluated_at,
                entry_price, high_water_mark_price
           FROM autotrade_position_tags
          WHERE paper_account_id = $1
          ORDER BY opened_at",
    )
    .bind(paper_account_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, acc, sym, log_id, opened, score, last_score, consec, last_ev, entry, hwm)| {
                PositionTag {
                    id,
                    paper_account_id: acc,
                    symbol: sym,
                    opened_by_log_id: log_id,
                    opened_at: opened,
                    score_at_open: score,
                    last_observed_score: last_score,
                    consecutive_degraded_checks: consec,
                    last_evaluated_at: last_ev,
                    entry_price: entry,
                    high_water_mark_price: hwm,
                }
            },
        )
        .collect())
}

async fn delete_tag(pool: &PgPool, paper_account_id: Uuid, symbol: &str) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM autotrade_position_tags WHERE paper_account_id = $1 AND symbol = $2")
        .bind(paper_account_id)
        .bind(symbol)
        .execute(pool)
        .await?;
    Ok(())
}

async fn update_tag_observation(
    pool: &PgPool,
    paper_account_id: Uuid,
    symbol: &str,
    new_score: Option<f64>,
    new_consec: i32,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE autotrade_position_tags
            SET last_observed_score = $3,
                consecutive_degraded_checks = $4,
                last_evaluated_at = now()
          WHERE paper_account_id = $1 AND symbol = $2",
    )
    .bind(paper_account_id)
    .bind(symbol)
    .bind(new_score)
    .bind(new_consec)
    .execute(pool)
    .await?;
    Ok(())
}

/// Single sweep: pulls confluence ranking, evaluates each autotrade-
/// tagged position, flattens those whose time-stop or signal-degradation
/// rule has tripped. Writes per-fire audit rows via the autotrade log
/// with action 'exit_time_stop' or 'exit_degraded'.
pub async fn sweep_exits(pool: &PgPool, user_id: Uuid) -> anyhow::Result<SweepResult> {
    let cfg = crate::confluence_autotrade::get_config(pool, user_id).await?;
    let account = crate::paper::ensure_default(pool, user_id).await?;
    let tags = list_tags(pool, account.id).await?;
    if tags.is_empty() {
        return Ok(SweepResult {
            considered: 0,
            flattened: Vec::new(),
            held: Vec::new(),
        });
    }
    let confluence_rows = crate::confluence::global().ranked(Utc::now(), 500, 1);
    let now = Utc::now();
    let mut flattened: Vec<FlattenedRow> = Vec::new();
    let mut held: Vec<HeldRow> = Vec::new();

    for tag in &tags {
        let current_score = confluence_rows
            .iter()
            .find(|r| r.symbol == tag.symbol)
            .map(|r| r.score);
        // Price-driven exits take priority over time/degradation rules.
        let price_decision = match (tag.entry_price, tag.high_water_mark_price) {
            (Some(entry), prior_hwm) => {
                let quote_price = match crate::market_data::quote(pool, &tag.symbol).await {
                    Ok(q) => q.price,
                    Err(_) => 0.0,
                };
                let new_hwm = next_high_water_mark(prior_hwm, quote_price);
                // Persist the new HWM so trailing stop has accurate state
                // on the next sweep.
                let _ = sqlx::query(
                    "UPDATE autotrade_position_tags
                        SET high_water_mark_price = $3
                      WHERE paper_account_id = $1 AND symbol = $2",
                )
                .bind(account.id)
                .bind(&tag.symbol)
                .bind(new_hwm)
                .execute(pool)
                .await;
                should_exit_price_driven(
                    entry,
                    quote_price,
                    new_hwm,
                    cfg.stop_loss_pct,
                    cfg.take_profit_pct,
                    cfg.trailing_stop_enabled,
                    cfg.trailing_stop_pct,
                )
                .map(|reason| ExitDecision {
                    symbol: tag.symbol.clone(),
                    reason,
                    days_held: (now - tag.opened_at).num_days(),
                    last_observed_score: current_score,
                })
            }
            _ => None,
        };
        let decision = price_decision.or_else(|| {
            should_exit(
                tag.opened_at,
                tag.consecutive_degraded_checks,
                current_score,
                cfg.min_score,
                cfg.max_holding_days,
                cfg.degradation_threshold_checks,
                now,
            )
        });
        match decision {
            Some(d) => {
                // Look up the open position qty to flatten.
                let pos_qty: Option<(Decimal,)> = sqlx::query_as(
                    "SELECT qty FROM paper_positions WHERE paper_account_id = $1 AND symbol = $2",
                )
                .bind(account.id)
                .bind(&tag.symbol)
                .fetch_optional(pool)
                .await?;
                let Some((qty,)) = pos_qty else {
                    // Position already gone (manually closed) — just
                    // drop the tag and move on.
                    let _ = delete_tag(pool, account.id, &tag.symbol).await;
                    continue;
                };
                let qty_abs = if qty.is_sign_negative() { -qty } else { qty };
                let side = if qty.is_sign_negative() {
                    traderview_core::Side::Cover
                } else {
                    traderview_core::Side::Sell
                };
                let _ = crate::paper::submit(
                    pool,
                    user_id,
                    account.id,
                    crate::paper::OrderRequest {
                        symbol: tag.symbol.clone(),
                        side,
                        qty: qty_abs,
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
                .await?;
                let action = match d.reason {
                    ExitReason::StopLoss => "exit_stop_loss",
                    ExitReason::TakeProfit => "exit_take_profit",
                    ExitReason::TrailingStop => "exit_trailing_stop",
                    ExitReason::TimeStop => "exit_time_stop",
                    ExitReason::SignalDegraded => "exit_degraded",
                };
                // Log via the same table the autotrade pipeline uses.
                let _ = sqlx::query(
                    "INSERT INTO confluence_autotrade_log
                        (user_id, symbol, score, distinct_sources, notional_usd,
                         action, paper_order_id, reason)
                     VALUES ($1, $2, $3, 0, 0, $4, NULL, $5)",
                )
                .bind(user_id)
                .bind(&tag.symbol)
                .bind(current_score.unwrap_or(0.0))
                .bind(action)
                .bind(format!(
                    "days_held={} score_at_open={:.2} current_score={:?}",
                    d.days_held, tag.score_at_open, current_score
                ))
                .execute(pool)
                .await;
                let _ = delete_tag(pool, account.id, &tag.symbol).await;
                flattened.push(FlattenedRow {
                    symbol: tag.symbol.clone(),
                    reason: d.reason,
                    days_held: d.days_held,
                });
            }
            None => {
                let is_degraded = match current_score {
                    None => true,
                    Some(s) => s < cfg.min_score,
                };
                let new_consec =
                    next_consecutive_degraded(tag.consecutive_degraded_checks, is_degraded);
                let _ = update_tag_observation(
                    pool,
                    account.id,
                    &tag.symbol,
                    current_score,
                    new_consec,
                )
                .await;
                held.push(HeldRow {
                    symbol: tag.symbol.clone(),
                    days_held: (now - tag.opened_at).num_days(),
                    consecutive_degraded_checks: new_consec,
                    last_observed_score: current_score,
                });
            }
        }
    }

    Ok(SweepResult {
        considered: tags.len(),
        flattened,
        held,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn time_stop_fires_when_past_max_holding_days() {
        let opened = now() - Duration::days(25);
        let d = should_exit(opened, 0, Some(10.0), 8.0, 20, 3, now()).unwrap();
        assert_eq!(d.reason, ExitReason::TimeStop);
        assert!(d.days_held >= 25);
    }

    #[test]
    fn time_stop_takes_priority_over_degradation() {
        // Both rules would fire — time-stop wins.
        let opened = now() - Duration::days(25);
        let d = should_exit(opened, 5, None, 8.0, 20, 3, now()).unwrap();
        assert_eq!(d.reason, ExitReason::TimeStop);
    }

    #[test]
    fn degradation_fires_after_enough_consecutive_checks() {
        let opened = now() - Duration::days(2);
        // prior was 2, this check is degraded → 3 ≥ 3 threshold.
        let d = should_exit(opened, 2, Some(5.0), 8.0, 20, 3, now()).unwrap();
        assert_eq!(d.reason, ExitReason::SignalDegraded);
    }

    #[test]
    fn degradation_does_not_fire_below_threshold() {
        let opened = now() - Duration::days(2);
        // prior 1, this check degraded → 2 < 3 threshold → hold.
        let d = should_exit(opened, 1, Some(5.0), 8.0, 20, 3, now());
        assert!(d.is_none());
    }

    #[test]
    fn dropped_from_ranking_counts_as_degraded() {
        let opened = now() - Duration::days(2);
        let d = should_exit(opened, 2, None, 8.0, 20, 3, now()).unwrap();
        assert_eq!(d.reason, ExitReason::SignalDegraded);
        assert!(d.last_observed_score.is_none());
    }

    #[test]
    fn healthy_signal_does_not_fire() {
        let opened = now() - Duration::days(2);
        // Current 9.5 ≥ 8.0 min_score → not degraded → hold.
        let d = should_exit(opened, 5, Some(9.5), 8.0, 20, 3, now());
        assert!(d.is_none());
    }

    #[test]
    fn next_consecutive_increments_when_degraded() {
        assert_eq!(next_consecutive_degraded(2, true), 3);
        assert_eq!(next_consecutive_degraded(0, true), 1);
    }

    #[test]
    fn next_consecutive_resets_when_healthy() {
        assert_eq!(next_consecutive_degraded(5, false), 0);
        assert_eq!(next_consecutive_degraded(0, false), 0);
    }

    #[test]
    fn take_profit_fires_at_threshold() {
        // Entry 100, TP 15% → triggers at >= 115.
        let r = should_exit_price_driven(100.0, 115.0, 115.0, 5.0, 15.0, false, 8.0);
        assert_eq!(r, Some(ExitReason::TakeProfit));
    }

    #[test]
    fn take_profit_does_not_fire_below_threshold() {
        let r = should_exit_price_driven(100.0, 114.99, 114.99, 5.0, 15.0, false, 8.0);
        assert_eq!(r, None);
    }

    #[test]
    fn stop_loss_fires_at_threshold() {
        let r = should_exit_price_driven(100.0, 95.0, 100.0, 5.0, 15.0, false, 8.0);
        assert_eq!(r, Some(ExitReason::StopLoss));
    }

    #[test]
    fn stop_loss_does_not_fire_when_trailing_enabled() {
        // Even with current price below SL threshold, trailing-only mode
        // ignores SL and uses HWM-relative trail. Current 95, HWM 100,
        // trail 8% → trail threshold = 92, current 95 > 92 → no fire.
        let r = should_exit_price_driven(100.0, 95.0, 100.0, 5.0, 15.0, true, 8.0);
        assert_eq!(r, None);
    }

    #[test]
    fn trailing_stop_fires_below_hwm_threshold() {
        // HWM 120, trail 8% → threshold = 110.4. Current 110 ≤ 110.4 → fire.
        let r = should_exit_price_driven(100.0, 110.0, 120.0, 5.0, 50.0, true, 8.0);
        assert_eq!(r, Some(ExitReason::TrailingStop));
    }

    #[test]
    fn take_profit_outranks_stop_loss() {
        // Hypothetical: entry 100, current 115, but SL = 90 also triggers
        // because we set crazy SL. Should still prefer TakeProfit.
        let r = should_exit_price_driven(100.0, 115.0, 115.0, 99.0, 15.0, false, 8.0);
        assert_eq!(r, Some(ExitReason::TakeProfit));
    }

    #[test]
    fn price_driven_returns_none_on_invalid_inputs() {
        assert_eq!(
            should_exit_price_driven(0.0, 100.0, 100.0, 5.0, 15.0, false, 8.0),
            None
        );
        assert_eq!(
            should_exit_price_driven(100.0, 0.0, 100.0, 5.0, 15.0, false, 8.0),
            None
        );
        assert_eq!(
            should_exit_price_driven(-100.0, 100.0, 100.0, 5.0, 15.0, false, 8.0),
            None
        );
    }

    #[test]
    fn next_high_water_mark_advances_when_price_up() {
        assert_eq!(next_high_water_mark(Some(100.0), 110.0), 110.0);
    }

    #[test]
    fn next_high_water_mark_holds_when_price_down() {
        assert_eq!(next_high_water_mark(Some(100.0), 95.0), 100.0);
    }

    #[test]
    fn next_high_water_mark_seeds_when_no_prior() {
        assert_eq!(next_high_water_mark(None, 100.0), 100.0);
        assert_eq!(next_high_water_mark(None, 0.0), 0.0);
    }

    #[test]
    fn next_high_water_mark_falls_back_to_prior_on_invalid_current() {
        assert_eq!(next_high_water_mark(Some(100.0), f64::NAN), 100.0);
        assert_eq!(next_high_water_mark(Some(100.0), -5.0), 100.0);
    }
}
