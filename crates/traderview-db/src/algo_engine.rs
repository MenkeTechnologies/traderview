//! Algo engine — bridges momentum strategy signals to a broker sink,
//! applying risk gates and persisting every order + fill via the
//! `crate::algo` repository.
//!
//! Paper-only in commit 3: the only built-in `BrokerSink` is
//! `InMemorySink` (used by tests). The real Alpaca sink and the
//! internal-paper-sim sink land in commit 4 with route wiring.
//!
//! Engine flow per bar window:
//!   1. Refresh strategy + open run from DB.
//!   2. Risk gate: kill_switch engaged? → no-op.
//!   3. Strategy::evaluate_entry on the latest bars.
//!   4. Risk gate: at max_concurrent_positions? → no-op.
//!   5. Size via `momentum_strategy::size_shares` against caller-supplied equity.
//!   6. Persist `algo_orders` row in `pending_submit` status.
//!   7. Call sink.submit_bracket(intent).
//!   8. Persist broker response (broker_order_id, status, raw_response).
//!   9. Sink reports a fill async → engine writes `algo_fills`.

use crate::algo::{self, AlgoFillInsert, AlgoOrderInsert, AlgoStrategy};
use chrono::{DateTime, Utc};
use rust_decimal::prelude::{FromPrimitive, Zero};
use rust_decimal::Decimal;
use serde_json::Value as Json;
use sqlx::PgPool;
use std::sync::Arc;
use traderview_core::algo_strategies::{self, EntrySignal, Side, SideMode, Sizing};
use traderview_core::models::PriceBar;
use traderview_import::ParsedExecution;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OrderIntent {
    pub strategy_id: Uuid,
    pub run_id: Uuid,
    pub client_order_id: Uuid,
    pub symbol: String,
    pub side: Side,
    pub qty: Decimal,
    pub entry_price: Decimal,
    pub stop_price: Decimal,
    pub take_profit_price: Decimal,
}

#[derive(Debug, Clone)]
pub struct SubmittedOrder {
    pub broker_order_id: String,
    pub status: String,
    pub raw_response: Option<Json>,
    /// Set when the broker confirmed an immediate fill (paper sim,
    /// market orders against a live quote). Real-Alpaca flows leave this
    /// `None` and emit fills later via the trade_updates WebSocket; the
    /// WS handler calls `record_fill` directly with the same payload.
    pub immediate_fill: Option<ImmediateFill>,
}

/// Lightweight event emitted by the engine on every state change so the
/// web layer's realtime hub can push it to subscribed clients. Defined
/// here (no web dependency) so traderview-db can publish without
/// pulling in axum / hub plumbing.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// Strategy produced an entry signal.
    SignalFired {
        strategy_id: Uuid,
        run_id: Uuid,
        symbol: String,
        side: Side,
        entry_price: Decimal,
        kind: &'static str,
    },
    /// Broker accepted the order.
    OrderSubmitted {
        strategy_id: Uuid,
        order_id: Uuid,
        symbol: String,
        side: Side,
        qty: Decimal,
        broker_order_id: String,
    },
    /// Fill landed in algo_fills + executions.
    FillReceived {
        strategy_id: Uuid,
        order_id: Uuid,
        symbol: String,
        qty: Decimal,
        price: Decimal,
    },
}

pub type EventSink = std::sync::Arc<dyn Fn(EngineEvent) + Send + Sync>;

#[derive(Debug, Clone)]
pub struct ImmediateFill {
    pub price: Decimal,
    pub qty: Decimal,
    pub fee: Decimal,
    pub executed_at: DateTime<Utc>,
    pub broker_fill_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("kill switch engaged: {reason:?}")]
    KillSwitch { reason: Option<String> },
    #[error("max concurrent positions reached ({0})")]
    PositionCap(i64),
    #[error("paper-locked until {0}; cannot promote broker_mode to live")]
    PaperLocked(chrono::DateTime<chrono::Utc>),
    #[error("sizing produced 0 shares")]
    ZeroQty,
    #[error("daily loss cap ${cap} breached (today P&L: ${pnl}); strategy auto-paused")]
    DailyLossCap { cap: Decimal, pnl: Decimal },
    #[error("consecutive losses cap ({cap}) breached (streak: {streak}); strategy auto-paused")]
    ConsecutiveLossesCap { cap: i64, streak: i64 },
    #[error("position size ${notional} exceeds cap ${cap}; order rejected")]
    PositionSizeCap { notional: Decimal, cap: Decimal },
    #[error("broker: {0}")]
    Broker(String),
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),
}

/// Broker abstraction. Production impls live in `traderview-db::alpaca_trading`
/// (live/paper Alpaca) and `traderview-db::paper` wrapper (internal sim) —
/// both wired in commit 4. Tests use `InMemorySink`.
pub trait BrokerSink: Send + Sync {
    fn submit_bracket(
        &self,
        intent: OrderIntent,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>,
    >;
}

/// Captures every submitted order. Used by tests + by the route layer
/// when broker_mode is internal_sim with no underlying paper account.
#[derive(Debug, Default, Clone)]
pub struct InMemorySink {
    pub submitted: Arc<std::sync::Mutex<Vec<OrderIntent>>>,
}

impl BrokerSink for InMemorySink {
    fn submit_bracket(
        &self,
        intent: OrderIntent,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>,
    > {
        let submitted = self.submitted.clone();
        Box::pin(async move {
            let id = intent.client_order_id.to_string();
            let fill = ImmediateFill {
                price: intent.entry_price,
                qty: intent.qty,
                fee: Decimal::ZERO,
                executed_at: Utc::now(),
                broker_fill_id: Some(format!("sim-fill-{id}")),
            };
            submitted.lock().expect("lock").push(intent);
            Ok(SubmittedOrder {
                broker_order_id: format!("sim-{id}"),
                status: "filled".into(),
                raw_response: None,
                immediate_fill: Some(fill),
            })
        })
    }
}

/// Configuration loaded from `algo_strategies.sizing / risk_gates / side_mode`.
/// The strategy itself is built lazily via the factory in
/// `algo_strategies::from_kind` — different `strategy_type` columns produce
/// different `Box<dyn Strategy>` impls.
///
/// `risk_gates` JSON shape (all optional; absent = unlimited):
/// ```json
/// {
///   "max_concurrent_positions": 5,
///   "max_daily_loss_usd": 500.0,
///   "max_consecutive_losses": 4,
///   "max_position_size_usd": 10000.0
/// }
/// ```
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub sizing: Sizing,
    pub side_mode: SideMode,
    pub max_concurrent_positions: i64,
    /// Pause the strategy when today's realized loss (since 00:00 UTC)
    /// hits this dollar amount. Stored as positive USD (the breach
    /// condition is `realized_pnl <= -max_daily_loss`).
    pub max_daily_loss_usd: Option<Decimal>,
    /// Pause when this many consecutive losing trades have closed.
    pub max_consecutive_losses: Option<i64>,
    /// Reject orders whose entry-price × qty exceeds this notional cap.
    pub max_position_size_usd: Option<Decimal>,
}

impl EngineConfig {
    pub fn from_strategy(s: &AlgoStrategy) -> Self {
        let sizing = serde_json::from_value::<Sizing>(s.sizing.clone())
            .unwrap_or_else(|_| Sizing::default());
        let side_mode = match s.side_mode.as_str() {
            "short" => SideMode::Short,
            "both" => SideMode::Both,
            _ => SideMode::Long,
        };
        let max_concurrent_positions = s
            .risk_gates
            .get("max_concurrent_positions")
            .and_then(|v| v.as_i64())
            .unwrap_or(5);
        let f64_to_dec = |v: &serde_json::Value| -> Option<Decimal> {
            v.as_f64()
                .and_then(|f| Decimal::try_from(f).ok())
                .filter(|d| *d > Decimal::zero())
        };
        let max_daily_loss_usd = s.risk_gates.get("max_daily_loss_usd").and_then(f64_to_dec);
        let max_consecutive_losses = s
            .risk_gates
            .get("max_consecutive_losses")
            .and_then(|v| v.as_i64())
            .filter(|n| *n > 0);
        let max_position_size_usd = s
            .risk_gates
            .get("max_position_size_usd")
            .and_then(f64_to_dec);
        Self {
            sizing,
            side_mode,
            max_concurrent_positions,
            max_daily_loss_usd,
            max_consecutive_losses,
            max_position_size_usd,
        }
    }
}

/// Realized P&L for `strategy_id` since 00:00 UTC today, summed across
/// every `algo_runs` row whose `started_at` falls in the window. Each
/// run carries its own `pnl_realized` ticker that the engine updates
/// as fills land. Positive when winning, negative when losing.
async fn today_realized_pnl(pool: &PgPool, strategy_id: Uuid) -> Result<Decimal, EngineError> {
    let row: Option<(Option<Decimal>,)> = sqlx::query_as(
        "SELECT COALESCE(SUM(pnl_realized), 0)::numeric
           FROM algo_runs
          WHERE strategy_id = $1
            AND started_at >= date_trunc('day', now() AT TIME ZONE 'UTC')",
    )
    .bind(strategy_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| EngineError::Broker(format!("today_pnl: {e}")))?;
    Ok(row.and_then(|(v,)| v).unwrap_or_else(Decimal::zero))
}

/// Count of consecutive losing runs at the tail of the strategy's run
/// history. Walks `algo_runs` newest-first; stops at the first run
/// with pnl_realized >= 0. Only counts stopped runs (in-flight runs
/// have no settled P&L yet).
async fn consecutive_losses(pool: &PgPool, strategy_id: Uuid) -> Result<i64, EngineError> {
    let rows: Vec<(Decimal,)> = sqlx::query_as(
        "SELECT pnl_realized FROM algo_runs
          WHERE strategy_id = $1 AND stopped_at IS NOT NULL
          ORDER BY stopped_at DESC
          LIMIT 50",
    )
    .bind(strategy_id)
    .fetch_all(pool)
    .await
    .map_err(|e| EngineError::Broker(format!("consec_losses: {e}")))?;
    let mut streak = 0i64;
    for (pnl,) in rows {
        if pnl < Decimal::zero() {
            streak += 1;
        } else {
            break;
        }
    }
    Ok(streak)
}

/// Auto-pause the strategy after a risk-gate breach. Sets enabled=FALSE
/// AND kill_switch=TRUE with a descriptive reason so the UI can render
/// the trip. Audit row records the strategy's owning user as the actor
/// (the engine acts on the user's behalf when a configured gate fires,
/// same as a manual trip).
async fn trip_kill_switch(
    pool: &PgPool,
    strategy_id: Uuid,
    user_id: Uuid,
    reason: &str,
) -> Result<(), EngineError> {
    sqlx::query(
        "UPDATE algo_strategies
            SET enabled = FALSE,
                kill_switch = TRUE,
                kill_reason = $2,
                last_kill_at = now()
          WHERE id = $1",
    )
    .bind(strategy_id)
    .bind(reason)
    .execute(pool)
    .await
    .map_err(|e| EngineError::Broker(format!("trip_kill: {e}")))?;
    sqlx::query(
        "INSERT INTO algo_kill_switch_audit (strategy_id, actor_user_id, action, reason)
         VALUES ($1, $2, 'engaged', $3)",
    )
    .bind(strategy_id)
    .bind(user_id)
    .bind(reason)
    .execute(pool)
    .await
    .map_err(|e| EngineError::Broker(format!("trip_audit: {e}")))?;

    // Fan out to every enabled webhook the user has configured. Done
    // AFTER the DB updates so a webhook failure can't roll back the
    // kill-switch engagement. Best-effort: errors are swallowed inside
    // `fan_out_all` (it updates last_status on each webhook row).
    let strategy_name =
        sqlx::query_scalar::<_, String>("SELECT name FROM algo_strategies WHERE id = $1")
            .bind(strategy_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "<unknown>".into());
    let payload = crate::webhooks::AlertPayload {
        title: format!("Algo strategy auto-paused: {strategy_name}"),
        message: reason.to_string(),
        symbol: None,
        kind: "algo_risk_breach".into(),
        url: None,
        fired_at: Utc::now(),
    };
    crate::webhooks::fan_out_all(pool, user_id, &payload).await;
    Ok(())
}

/// One-shot evaluator: feed it a bar window for `symbol`, the caller's
/// current account equity, and the count of currently-open positions.
/// Returns `Some(AlgoOrder.id)` when an order was submitted, `None`
/// when no signal fired (caller treats as a no-op, not an error).
pub async fn process_bar_window(
    pool: &PgPool,
    sink: &dyn BrokerSink,
    strategy: &AlgoStrategy,
    run_id: Uuid,
    bars: &[PriceBar],
    equity: f64,
    open_positions: i64,
    event_sink: Option<&EventSink>,
) -> Result<Option<Uuid>, EngineError> {
    if strategy.kill_switch {
        return Err(EngineError::KillSwitch {
            reason: strategy.kill_reason.clone(),
        });
    }
    if strategy.broker_mode == "live" && chrono::Utc::now() <= strategy.paper_locked_until {
        return Err(EngineError::PaperLocked(strategy.paper_locked_until));
    }

    let cfg = EngineConfig::from_strategy(strategy);

    if open_positions >= cfg.max_concurrent_positions {
        return Err(EngineError::PositionCap(open_positions));
    }

    // Risk gate: daily loss cap. Auto-pauses the strategy on breach so
    // the next bar processed for the same strategy will short-circuit
    // at the `kill_switch` check above.
    if let Some(cap) = cfg.max_daily_loss_usd {
        let pnl = today_realized_pnl(pool, strategy.id).await?;
        if pnl <= -cap {
            let reason = format!("auto-paused: daily loss cap ${cap} breached (today P&L: ${pnl})");
            trip_kill_switch(pool, strategy.id, strategy.user_id, &reason).await?;
            return Err(EngineError::DailyLossCap { cap, pnl });
        }
    }
    // Risk gate: consecutive-losses streak.
    if let Some(cap) = cfg.max_consecutive_losses {
        let streak = consecutive_losses(pool, strategy.id).await?;
        if streak >= cap {
            let reason = format!("auto-paused: {streak} consecutive losses (cap {cap})");
            trip_kill_switch(pool, strategy.id, strategy.user_id, &reason).await?;
            return Err(EngineError::ConsecutiveLossesCap { cap, streak });
        }
    }

    let strat = algo_strategies::from_kind(&strategy.strategy_type, &strategy.entry_rules)
        .map_err(|e| EngineError::Broker(e.to_string()))?;

    // Multi-symbol strategies don't use the single-bar evaluate_entry
    // path. Caller (algo_runner) calls process_bar_window_multi instead.
    if strat.required_symbols().is_some() {
        return Ok(None);
    }
    let Some(sig) = strat.evaluate_entry(bars, cfg.side_mode) else {
        return Ok(None);
    };
    let qty = algo_strategies::size_shares(equity, sig.entry_price, sig.stop_distance, &cfg.sizing);
    if qty == 0 {
        return Err(EngineError::ZeroQty);
    }

    // Risk gate: position-size notional cap. Hard reject — we don't
    // shrink the position because qty came from the sizing function
    // that already factored equity + per-trade risk; shrinking it
    // silently would violate the user's risk-per-trade contract.
    if let Some(cap) = cfg.max_position_size_usd {
        let entry_price = Decimal::try_from(sig.entry_price).unwrap_or_else(|_| Decimal::zero());
        let notional = entry_price * Decimal::from(qty);
        if notional > cap {
            return Err(EngineError::PositionSizeCap { notional, cap });
        }
    }

    algo::increment_run_counter(pool, run_id, algo::RunCounter::SignalsEmitted, 1)
        .await
        .ok();

    let symbol = bars.last().map(|b| b.symbol.clone()).unwrap_or_default();
    let intent = build_intent(strategy, run_id, &symbol, &sig, qty);
    if let Some(emit) = event_sink {
        emit(EngineEvent::SignalFired {
            strategy_id: strategy.id,
            run_id,
            symbol: symbol.clone(),
            side: intent.side,
            entry_price: intent.entry_price,
            kind: sig.kind,
        });
    }

    let inserted = algo::insert_order(
        pool,
        run_id,
        strategy.id,
        AlgoOrderInsert {
            client_order_id: intent.client_order_id,
            symbol: symbol.clone(),
            side: side_to_str(intent.side).into(),
            order_type: "market".into(),
            order_class: "bracket".into(),
            qty: intent.qty,
            limit_price: None,
            stop_price: Some(intent.stop_price),
            raw_request: Some(intent_to_request_json(&intent)),
        },
    )
    .await
    .map_err(|e| EngineError::Broker(e.to_string()))?;

    match sink.submit_bracket(intent.clone()).await {
        Ok(resp) => {
            algo::mark_order_submitted(
                pool,
                inserted.id,
                Some(resp.broker_order_id.clone()),
                &resp.status,
                resp.raw_response,
                None,
            )
            .await
            .map_err(|e| EngineError::Broker(e.to_string()))?;
            algo::increment_run_counter(pool, run_id, algo::RunCounter::OrdersSubmitted, 1)
                .await
                .ok();
            if let Some(emit) = event_sink {
                emit(EngineEvent::OrderSubmitted {
                    strategy_id: strategy.id,
                    order_id: inserted.id,
                    symbol: symbol.clone(),
                    side: intent.side,
                    qty: intent.qty,
                    broker_order_id: resp.broker_order_id.clone(),
                });
            }

            // Pipeline integration: any immediate fill becomes an
            // executions row tagged with the strategy's broker account,
            // then trades::rollup_account materializes the trade so
            // every dashboard / R-dist / equity curve in the app sees
            // it without needing an algo-specific code path.
            if let Some(fill) = resp.immediate_fill {
                record_fill(pool, strategy, &intent, inserted.id, &fill, event_sink).await?;
            }
            Ok(Some(inserted.id))
        }
        Err(e) => {
            algo::mark_order_submitted(
                pool,
                inserted.id,
                None,
                "rejected",
                None,
                Some(e.to_string()),
            )
            .await
            .map_err(|de| EngineError::Broker(de.to_string()))?;
            Err(e)
        }
    }
}

/// Record a fill against (a) `algo_fills` for broker audit and (b) the
/// `executions` table tagged with the strategy's `account_id`, then
/// trigger `trades::rollup_account` so the FIFO pipeline materializes
/// the trade row. Exposed `pub(crate)` so a future WS trade_updates
/// handler can drive it with the same shape for real-Alpaca fills.
pub async fn record_fill(
    pool: &PgPool,
    strategy: &AlgoStrategy,
    intent: &OrderIntent,
    algo_order_id: Uuid,
    fill: &ImmediateFill,
    event_sink: Option<&EventSink>,
) -> Result<(), EngineError> {
    algo::insert_fill(
        pool,
        AlgoFillInsert {
            order_id: algo_order_id,
            broker_fill_id: fill.broker_fill_id.clone(),
            fill_qty: fill.qty,
            fill_price: fill.price,
            commission: fill.fee,
            raw: None,
        },
    )
    .await
    .map_err(|e| EngineError::Broker(e.to_string()))?;
    algo::increment_run_counter(pool, intent.run_id, algo::RunCounter::FillsReceived, 1)
        .await
        .ok();

    // account_id is NOT NULL since migration 0056 — strategies cannot
    // exist without a bound account, so the executions pipeline always
    // gets fed.
    let account_id = strategy.account_id;

    let side = match intent.side {
        Side::Buy => traderview_core::models::Side::Buy,
        Side::Sell => traderview_core::models::Side::Sell,
    };
    let parsed = ParsedExecution {
        symbol: intent.symbol.clone(),
        side,
        qty: fill.qty,
        price: fill.price,
        fee: fill.fee,
        commission: Decimal::ZERO,
        executed_at: fill.executed_at,
        broker_order_id: fill.broker_fill_id.clone(),
        raw: serde_json::json!({
            "source": "algo_engine",
            "strategy_id": strategy.id,
            "client_order_id": intent.client_order_id,
        }),
        asset_class: traderview_core::models::AssetClass::Stock,
        option_type: None,
        strike: None,
        expiration: None,
        multiplier: Decimal::ONE,
        tick_size: None,
        tick_value: None,
        base_ccy: None,
        quote_ccy: None,
        pip_size: None,
    };
    crate::executions::insert_manual(pool, account_id, &parsed)
        .await
        .map_err(|e| EngineError::Broker(format!("executions::insert_manual: {e}")))?;
    crate::trades::rollup_account(pool, account_id)
        .await
        .map_err(|e| EngineError::Broker(format!("trades::rollup_account: {e}")))?;
    if let Some(emit) = event_sink {
        emit(EngineEvent::FillReceived {
            strategy_id: strategy.id,
            order_id: algo_order_id,
            symbol: intent.symbol.clone(),
            qty: fill.qty,
            price: fill.price,
        });
    }
    Ok(())
}

/// Multi-symbol variant of `process_bar_window`. Used by pairs / stat-arb
/// strategies whose Strategy impl exposes `required_symbols`. Bars for
/// every required symbol must be in `bars_by_symbol`. The order is
/// submitted on the strategy's "primary" symbol (whatever the strategy's
/// EntrySignal carries via the diagnostic) — for now we route through
/// the first symbol returned by required_symbols (leg A in pairs).
pub async fn process_bar_window_multi(
    pool: &PgPool,
    sink: &dyn BrokerSink,
    strategy: &AlgoStrategy,
    run_id: Uuid,
    bars_by_symbol: &std::collections::HashMap<String, Vec<PriceBar>>,
    equity: f64,
    open_positions: i64,
    event_sink: Option<&EventSink>,
) -> Result<Option<Uuid>, EngineError> {
    if strategy.kill_switch {
        return Err(EngineError::KillSwitch {
            reason: strategy.kill_reason.clone(),
        });
    }
    let cfg = EngineConfig::from_strategy(strategy);
    if open_positions >= cfg.max_concurrent_positions {
        return Err(EngineError::PositionCap(open_positions));
    }
    // Same risk gates as the single-symbol path (commit 48).
    if let Some(cap) = cfg.max_daily_loss_usd {
        let pnl = today_realized_pnl(pool, strategy.id).await?;
        if pnl <= -cap {
            let reason = format!("auto-paused: daily loss cap ${cap} breached (today P&L: ${pnl})");
            trip_kill_switch(pool, strategy.id, strategy.user_id, &reason).await?;
            return Err(EngineError::DailyLossCap { cap, pnl });
        }
    }
    if let Some(cap) = cfg.max_consecutive_losses {
        let streak = consecutive_losses(pool, strategy.id).await?;
        if streak >= cap {
            let reason = format!("auto-paused: {streak} consecutive losses (cap {cap})");
            trip_kill_switch(pool, strategy.id, strategy.user_id, &reason).await?;
            return Err(EngineError::ConsecutiveLossesCap { cap, streak });
        }
    }
    let strat = algo_strategies::from_kind(&strategy.strategy_type, &strategy.entry_rules)
        .map_err(|e| EngineError::Broker(e.to_string()))?;
    let primary_symbol = strat
        .required_symbols()
        .and_then(|v| v.into_iter().next())
        .unwrap_or_default();
    let Some(sig) = strat.evaluate_entry_multi(bars_by_symbol, cfg.side_mode) else {
        return Ok(None);
    };
    let qty = algo_strategies::size_shares(equity, sig.entry_price, sig.stop_distance, &cfg.sizing);
    if qty == 0 {
        return Err(EngineError::ZeroQty);
    }
    if let Some(cap) = cfg.max_position_size_usd {
        let entry_price = Decimal::try_from(sig.entry_price).unwrap_or_else(|_| Decimal::zero());
        let notional = entry_price * Decimal::from(qty);
        if notional > cap {
            return Err(EngineError::PositionSizeCap { notional, cap });
        }
    }
    algo::increment_run_counter(pool, run_id, algo::RunCounter::SignalsEmitted, 1)
        .await
        .ok();
    let intent = build_intent(strategy, run_id, &primary_symbol, &sig, qty);
    if let Some(emit) = event_sink {
        emit(EngineEvent::SignalFired {
            strategy_id: strategy.id,
            run_id,
            symbol: primary_symbol.clone(),
            side: intent.side,
            entry_price: intent.entry_price,
            kind: sig.kind,
        });
    }
    let inserted = algo::insert_order(
        pool,
        run_id,
        strategy.id,
        AlgoOrderInsert {
            client_order_id: intent.client_order_id,
            symbol: primary_symbol.clone(),
            side: side_to_str(intent.side).into(),
            order_type: "market".into(),
            order_class: "bracket".into(),
            qty: intent.qty,
            limit_price: None,
            stop_price: Some(intent.stop_price),
            raw_request: Some(intent_to_request_json(&intent)),
        },
    )
    .await
    .map_err(|e| EngineError::Broker(e.to_string()))?;
    match sink.submit_bracket(intent.clone()).await {
        Ok(resp) => {
            algo::mark_order_submitted(
                pool,
                inserted.id,
                Some(resp.broker_order_id.clone()),
                &resp.status,
                resp.raw_response,
                None,
            )
            .await
            .map_err(|e| EngineError::Broker(e.to_string()))?;
            algo::increment_run_counter(pool, run_id, algo::RunCounter::OrdersSubmitted, 1)
                .await
                .ok();
            if let Some(emit) = event_sink {
                emit(EngineEvent::OrderSubmitted {
                    strategy_id: strategy.id,
                    order_id: inserted.id,
                    symbol: primary_symbol.clone(),
                    side: intent.side,
                    qty: intent.qty,
                    broker_order_id: resp.broker_order_id.clone(),
                });
            }
            if let Some(fill) = resp.immediate_fill {
                record_fill(pool, strategy, &intent, inserted.id, &fill, event_sink).await?;
            }
            Ok(Some(inserted.id))
        }
        Err(e) => {
            algo::mark_order_submitted(
                pool,
                inserted.id,
                None,
                "rejected",
                None,
                Some(e.to_string()),
            )
            .await
            .map_err(|de| EngineError::Broker(de.to_string()))?;
            Err(e)
        }
    }
}

fn build_intent(
    strategy: &AlgoStrategy,
    run_id: Uuid,
    symbol: &str,
    sig: &EntrySignal,
    qty: u64,
) -> OrderIntent {
    OrderIntent {
        strategy_id: strategy.id,
        run_id,
        client_order_id: Uuid::new_v4(),
        symbol: symbol.to_string(),
        side: sig.side,
        qty: Decimal::from(qty),
        entry_price: Decimal::from_f64(sig.entry_price).unwrap_or_default(),
        stop_price: Decimal::from_f64(sig.stop_price).unwrap_or_default(),
        take_profit_price: Decimal::from_f64(sig.take_profit_price).unwrap_or_default(),
    }
}

fn side_to_str(s: Side) -> &'static str {
    match s {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}

fn intent_to_request_json(i: &OrderIntent) -> Json {
    serde_json::json!({
        "symbol": i.symbol,
        "side": side_to_str(i.side),
        "qty": i.qty.to_string(),
        "type": "market",
        "order_class": "bracket",
        "take_profit": {"limit_price": i.take_profit_price.to_string()},
        "stop_loss": {"stop_price": i.stop_price.to_string()},
        "client_order_id": i.client_order_id.to_string(),
        "time_in_force": "day",
    })
}
