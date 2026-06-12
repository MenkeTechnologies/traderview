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
    /// Tick fired for this strategy but skipped without evaluating.
    /// `reason` is a short slug ("no_universe", "no_symbols",
    /// "broker_pending", "below_min_bars:AAPL=42", ...) the UI surfaces
    /// in the stdout pane so the user can see WHY nothing is happening
    /// instead of staring at an empty log.
    TickSkipped { strategy_id: Uuid, reason: String },
    /// Broker rejected an order submission. `kind` is "entry" or
    /// "exit" so the UI's stdout pane can label the failure. Without
    /// this, exit rejections silently disappeared into the trace log
    /// and the user saw a SignalFired with no follow-up.
    OrderRejected {
        strategy_id: Uuid,
        symbol: String,
        side: Side,
        kind: &'static str,
        reason: String,
    },
    /// Tick evaluated this strategy without firing a signal (the
    /// strategy returned None). Tagged with the symbol so the user
    /// sees "AAPL, MSFT, NVDA evaluated — no signal" in the stdout
    /// pane, confirming the engine IS alive.
    BarEvaluated {
        strategy_id: Uuid,
        symbol: String,
        bars: usize,
    },
    /// Per-tick heartbeat — fires every 10s from the runner regardless
    /// of timeframe boundary. Lets the user see "engine is alive,
    /// here's what's loaded" without having to wait for the next M1
    /// boundary to surface a real evaluation.
    Heartbeat {
        strategy_id: Uuid,
        universe_size: usize,
        subscribed_live: usize,
        bars_processed: i64,
        signals_emitted: i64,
        seconds_to_next_eval: i64,
    },
}

pub type EventSink = std::sync::Arc<dyn Fn(EngineEvent) + Send + Sync>;

#[derive(Debug, Clone)]
pub struct ImmediateFill {
    pub price: Decimal,
    pub qty: Decimal,
    pub fee: Decimal,
    pub executed_at: DateTime<Utc>,
    /// Per-fill unique id — the `algo_fills.broker_fill_id` UNIQUE
    /// idempotency key. For brokers that fill one order in many slices
    /// (Alpaca partial_fill events) this must be the slice id, not the
    /// order id, or every slice after the first short-circuits as a
    /// replay.
    pub broker_fill_id: Option<String>,
    /// Broker order id for the executions row. None falls back to
    /// `broker_fill_id` (the historical behaviour, correct for brokers
    /// where one fill == one order). Set it when slices share an order
    /// so executions stay queryable per broker order.
    pub broker_order_id: Option<String>,
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
    #[error("max drawdown ${cap} breached (current drawdown from peak: ${drawdown}); strategy auto-paused")]
    MaxDrawdown { cap: Decimal, drawdown: Decimal },
    #[error("position size ${notional} exceeds cap ${cap}; order rejected")]
    PositionSizeCap { notional: Decimal, cap: Decimal },
    #[error("account exposure ${open_notional} at/over cap ${cap}; entry skipped")]
    AccountExposureCap { open_notional: Decimal, cap: Decimal },
    #[error("equity-curve filter: cum PnL below its {ma_trips}-trip MA; entry skipped")]
    EquityCurveFilter { ma_trips: usize },
    #[error("earnings blackout: {symbol} reports on {date} (within {days}d window); entry skipped")]
    EarningsBlackout {
        symbol: String,
        date: chrono::NaiveDate,
        days: i64,
    },
    #[error("outside entry window {window} ET; entry skipped")]
    OutsideEntryWindow { window: String },
    #[error("not an allowed entry day ({days}); entry skipped")]
    OutsideEntryDays { days: String },
    #[error("daily entry cap reached ({count}/{cap}); entry skipped until tomorrow")]
    DailyEntryCap { cap: i64, count: i64 },
    #[error("loss cooldown: last losing trade closed {minutes_ago}m ago (cooldown {cooldown}m); entry skipped")]
    LossCooldown { minutes_ago: i64, cooldown: i64 },
    #[error("correlation gate: {symbol} vs open {other} has rho={rho:.2} (cap {cap}); entry adds concentration, skipped")]
    CorrelationGate {
        symbol: String,
        other: String,
        rho: f64,
        cap: f64,
    },
    #[error("HTF trend filter: {side} entry against the {interval} EMA{ema_period} trend; skipped")]
    HtfTrendFilter {
        side: &'static str,
        interval: String,
        ema_period: usize,
    },
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

    /// Submit a single-leg market order to close an open position. Used
    /// by the runner's exit pass — the strategy's `evaluate_exit` fired,
    /// and the engine needs to flat the position at market. `side` is
    /// the CLOSING side (opposite of the position): `Sell` for a long
    /// close, `Buy` for a short cover. `reference_price` is the
    /// strategy's expected exit price — used by simulated sinks as the
    /// fill price; live broker sinks ignore it (the broker prices at
    /// market or, in extended hours, uses it as a LIMIT). Default impl
    /// returns a Broker error so sinks that haven't been wired for
    /// exits compile — the runner logs and skips.
    fn submit_market_close(
        &self,
        _symbol: String,
        _side: Side,
        _qty: Decimal,
        _reference_price: Decimal,
        _coid: Uuid,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>,
    > {
        Box::pin(async move {
            Err(EngineError::Broker(
                "submit_market_close not implemented for this sink".into(),
            ))
        })
    }
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
                broker_order_id: None,
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

    fn submit_market_close(
        &self,
        symbol: String,
        side: Side,
        qty: Decimal,
        reference_price: Decimal,
        coid: Uuid,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>,
    > {
        let submitted = self.submitted.clone();
        Box::pin(async move {
            let id = coid.to_string();
            // Sim fills at `reference_price`. Tests that assert on close
            // P&L pass the entry signal's exit_price so realized P&L
            // matches the strategy's intent. Filling at zero (the prior
            // default) made every test trade look like a -100% loss.
            let fill = ImmediateFill {
                price: reference_price,
                qty,
                fee: Decimal::ZERO,
                executed_at: Utc::now(),
                broker_fill_id: Some(format!("sim-close-{id}")),
                broker_order_id: None,
            };
            submitted.lock().expect("lock").push(OrderIntent {
                strategy_id: Uuid::nil(),
                run_id: Uuid::nil(),
                client_order_id: coid,
                symbol,
                side,
                qty,
                entry_price: reference_price,
                stop_price: Decimal::ZERO,
                take_profit_price: Decimal::ZERO,
            });
            Ok(SubmittedOrder {
                broker_order_id: format!("sim-close-{id}"),
                status: "filled".into(),
                raw_response: None,
                immediate_fill: Some(fill),
            })
        })
    }
}

impl EngineError {
    /// The gate name when this error is a RISK-GATE fire (the thing
    /// the audit table counts), None for infra/state errors. Keep in
    /// lockstep with the variants — pinned by a test.
    pub fn gate_name(&self) -> Option<&'static str> {
        match self {
            Self::PositionCap(_) => Some("position_cap"),
            Self::DailyLossCap { .. } => Some("daily_loss_cap"),
            Self::MaxDrawdown { .. } => Some("max_drawdown"),
            Self::AccountExposureCap { .. } => Some("account_exposure"),
            Self::EquityCurveFilter { .. } => Some("equity_curve_filter"),
            Self::ConsecutiveLossesCap { .. } => Some("consecutive_losses"),
            Self::PositionSizeCap { .. } => Some("position_size_cap"),
            Self::EarningsBlackout { .. } => Some("earnings_blackout"),
            Self::OutsideEntryWindow { .. } => Some("entry_window"),
            Self::OutsideEntryDays { .. } => Some("entry_days"),
            Self::DailyEntryCap { .. } => Some("daily_entry_cap"),
            Self::LossCooldown { .. } => Some("loss_cooldown"),
            Self::CorrelationGate { .. } => Some("correlation"),
            Self::HtfTrendFilter { .. } => Some("htf_trend"),
            _ => None,
        }
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
///   "max_position_size_usd": 10000.0,
///   "earnings_blackout_days": 2,
///   "entry_window": "10:00-15:30",
///   "max_entries_per_day": 6,
///   "loss_cooldown_minutes": 30,
///   "max_entry_correlation": 0.8,
///   "correlation_lookback_days": 60,
///   "htf_interval": "1h",
///   "htf_ema_period": 50
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
    /// Circuit breaker: pause when cumulative realized PnL's CURRENT
    /// drawdown from its all-time peak reaches this dollar amount —
    /// catches the slow bleed across many days that never trips the
    /// daily cap. Stored positive USD. None = off.
    pub max_drawdown_usd: Option<Decimal>,
    /// Reject orders whose entry-price × qty exceeds this notional cap.
    pub max_position_size_usd: Option<Decimal>,
    /// ACCOUNT-level exposure ceiling: skip entries while the summed
    /// open entry notional across ALL strategies on this account is
    /// at/over the cap. Per-strategy caps bound one strategy; five
    /// strategies each inside their own cap can still stack the
    /// account — this is the aggregate brake. Portfolio-dependent,
    /// hence live-only (not backtest-replayable from one bar series).
    pub max_account_notional_usd: Option<Decimal>,
    /// Equity-curve meta-filter: skip entries while the strategy's own
    /// cumulative realized PnL sits below its N-trip SMA. Can't fix a
    /// bad system; shortens the losing streaks of a decaying one at
    /// the cost of whipsaw re-entries. ≥ 2 trips; None/0/1 = off.
    pub equity_curve_filter_trips: Option<usize>,
    /// Skip ENTRIES when the symbol's next earnings report is within
    /// this many days (forward-looking: the risk being managed is
    /// holding INTO the print). Exits are never blocked.
    pub earnings_blackout_days: Option<i64>,
    /// ENTRIES allowed only inside this US-Eastern wall-clock window,
    /// as minutes since midnight (start inclusive, end exclusive).
    /// Exits are never blocked. None = always.
    pub entry_window: Option<(u32, u32)>,
    /// ENTRIES allowed only on these US-Eastern weekdays — bitmask
    /// from core::risk_gate::parse_entry_days ("mon,tue,wed"); typos
    /// disable the gate rather than silently trading an excluded day.
    /// Exits never blocked. None = all days.
    pub entry_days: Option<u8>,
    /// Time stop: force-exit any position held at least this many
    /// minutes, regardless of the strategy's own exit opinion — the
    /// universal "no position lives forever" overlay. None = off.
    pub max_hold_minutes: Option<i64>,
    /// Overtrading gate: cap on entry orders per UTC day. Exits are
    /// never blocked. None = unlimited.
    pub max_entries_per_day: Option<i64>,
    /// Revenge-trading gate: minutes to wait after a LOSING round trip
    /// closes before the next entry. Exits never blocked. None = off.
    pub loss_cooldown_minutes: Option<i64>,
    /// Concentration gate: skip entries whose daily-return correlation
    /// with ANY open position exceeds this (|rho|, so perfect hedges
    /// gate too — a mirror is still the same trade). None = off.
    pub max_entry_correlation: Option<f64>,
    /// Daily bars of history for the correlation comparison.
    pub correlation_lookback_days: i64,
    /// Higher-timeframe trend filter: longs only above the EMA on this
    /// interval, shorts only below. Both keys required to enable.
    /// Insufficient higher-TF history SKIPS the gate (allow + log),
    /// matching the correlation gate's thin-history convention.
    pub htf_filter: Option<(traderview_core::BarInterval, usize)>,
}

/// Account-level exposure brake, shared by the single- and
/// multi-symbol paths. Entry notional (deterministic, no quotes) —
/// the same convention as the borrow-fee pass. Does NOT trip the
/// kill switch: exposure recedes as positions close, so the gate
/// self-clears, unlike drawdown which needs a human.
async fn check_account_exposure_gate(
    pool: &PgPool,
    strategy: &AlgoStrategy,
) -> Result<(), EngineError> {
    let cfg = EngineConfig::from_strategy(strategy);
    let Some(cap) = cfg.max_account_notional_usd else {
        return Ok(());
    };
    let open_notional = crate::trades::open_account_notional(pool, strategy.account_id)
        .await
        .map_err(|e| EngineError::Broker(e.to_string()))?;
    if open_notional >= cap {
        return Err(EngineError::AccountExposureCap { open_notional, cap });
    }
    Ok(())
}

/// Equity-curve meta-filter, entry-side. Shares curve_above_ma with
/// the backtest replay — one decision implementation — over the same
/// strategy_trips reconstruction the drawdown gate reads. Start 0.0:
/// the decision is start-invariant (pinned in core).
async fn check_equity_curve_gate(
    pool: &PgPool,
    strategy: &AlgoStrategy,
) -> Result<(), EngineError> {
    let cfg = EngineConfig::from_strategy(strategy);
    let Some(ma) = cfg.equity_curve_filter_trips else {
        return Ok(());
    };
    let trips = crate::algo::strategy_trips(pool, strategy.user_id, strategy.id)
        .await
        .map_err(|e| EngineError::Broker(e.to_string()))?;
    let pnls: Vec<f64> = trips.iter().map(|t| t.pnl).collect();
    if !traderview_core::equity_curve_filter::curve_above_ma(0.0, &pnls, ma) {
        return Err(EngineError::EquityCurveFilter { ma_trips: ma });
    }
    Ok(())
}

/// Max-drawdown circuit breaker, shared by the single- and
/// multi-symbol paths. Drawdown is the CURRENT distance of cumulative
/// realized PnL from its peak (core::realized_drawdown) over the
/// strategy's full trip history — a strategy that recovered to a new
/// peak is not in drawdown regardless of past dips.
async fn check_drawdown_gate(
    pool: &PgPool,
    strategy: &AlgoStrategy,
) -> Result<(), EngineError> {
    let cfg = EngineConfig::from_strategy(strategy);
    let Some(cap) = cfg.max_drawdown_usd else {
        return Ok(());
    };
    let trips = crate::algo::strategy_trips(pool, strategy.user_id, strategy.id)
        .await
        .map_err(|e| EngineError::Broker(e.to_string()))?;
    let pnls: Vec<f64> = trips.iter().map(|t| t.pnl).collect();
    let dd = traderview_core::live_vs_backtest::realized_drawdown(&pnls);
    let drawdown = Decimal::try_from(dd).unwrap_or_default();
    if drawdown >= cap {
        let reason =
            format!("auto-paused: max drawdown ${cap} breached (current drawdown: ${drawdown})");
        trip_kill_switch(pool, strategy.id, strategy.user_id, &reason).await?;
        return Err(EngineError::MaxDrawdown { cap, drawdown });
    }
    Ok(())
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
        let max_drawdown_usd = s.risk_gates.get("max_drawdown_usd").and_then(f64_to_dec);
        let max_account_notional_usd = s
            .risk_gates
            .get("max_account_notional_usd")
            .and_then(f64_to_dec);
        let equity_curve_filter_trips = s
            .risk_gates
            .get("equity_curve_filter_trips")
            .and_then(|v| v.as_u64())
            .filter(|n| *n >= 2)
            .map(|n| n as usize);
        let max_consecutive_losses = s
            .risk_gates
            .get("max_consecutive_losses")
            .and_then(|v| v.as_i64())
            .filter(|n| *n > 0);
        let max_position_size_usd = s
            .risk_gates
            .get("max_position_size_usd")
            .and_then(f64_to_dec);
        let earnings_blackout_days = s
            .risk_gates
            .get("earnings_blackout_days")
            .and_then(|v| v.as_i64())
            .filter(|n| *n > 0);
        let entry_window = s
            .risk_gates
            .get("entry_window")
            .and_then(|v| v.as_str())
            .and_then(parse_entry_window);
        let entry_days = s
            .risk_gates
            .get("entry_days")
            .and_then(|v| v.as_str())
            .and_then(traderview_core::risk_gate::parse_entry_days);
        let max_hold_minutes = s
            .risk_gates
            .get("max_hold_minutes")
            .and_then(|v| v.as_i64())
            .filter(|n| *n > 0);
        let max_entries_per_day = s
            .risk_gates
            .get("max_entries_per_day")
            .and_then(|v| v.as_i64())
            .filter(|n| *n > 0);
        let loss_cooldown_minutes = s
            .risk_gates
            .get("loss_cooldown_minutes")
            .and_then(|v| v.as_i64())
            .filter(|n| *n > 0);
        let max_entry_correlation = s
            .risk_gates
            .get("max_entry_correlation")
            .and_then(|v| v.as_f64())
            .filter(|c| (0.0..1.0).contains(c) && *c > 0.0);
        let correlation_lookback_days = s
            .risk_gates
            .get("correlation_lookback_days")
            .and_then(|v| v.as_i64())
            .filter(|n| *n >= 20)
            .unwrap_or(60);
        let htf_filter = (|| {
            let interval_str = s.risk_gates.get("htf_interval")?.as_str()?;
            let interval: traderview_core::BarInterval =
                serde_json::from_value(serde_json::Value::String(interval_str.to_string())).ok()?;
            let period = s
                .risk_gates
                .get("htf_ema_period")?
                .as_u64()
                .filter(|p| (2..=400).contains(p))? as usize;
            Some((interval, period))
        })();
        Self {
            sizing,
            side_mode,
            max_concurrent_positions,
            max_daily_loss_usd,
            max_drawdown_usd,
            max_account_notional_usd,
            equity_curve_filter_trips,
            max_consecutive_losses,
            max_position_size_usd,
            earnings_blackout_days,
            entry_window,
            entry_days,
            max_hold_minutes,
            max_entries_per_day,
            loss_cooldown_minutes,
            max_entry_correlation,
            correlation_lookback_days,
            htf_filter,
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
    // Wrap the strategy update + audit insert in a single tx so partial
    // failure (audit insert fails, strategy update succeeded) can't
    // leave divergent state: a paused strategy without a paired audit
    // row is a compliance gap. Webhook fan-out stays OUTSIDE the tx by
    // design — best-effort, must not roll back the kill.
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| EngineError::Broker(format!("trip_kill begin: {e}")))?;
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
    .execute(&mut *tx)
    .await
    .map_err(|e| EngineError::Broker(format!("trip_kill: {e}")))?;
    sqlx::query(
        "INSERT INTO algo_kill_switch_audit (strategy_id, actor_user_id, action, reason)
         VALUES ($1, $2, 'engaged', $3)",
    )
    .bind(strategy_id)
    .bind(user_id)
    .bind(reason)
    .execute(&mut *tx)
    .await
    .map_err(|e| EngineError::Broker(format!("trip_audit: {e}")))?;
    tx.commit()
        .await
        .map_err(|e| EngineError::Broker(format!("trip_kill commit: {e}")))?;

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
/// Parse "HH:MM-HH:MM" into minutes-since-midnight. Malformed input
/// or a non-increasing window (overnight wrap unsupported for US
/// equities) disables the gate by returning None.
/// Human label for an entry-days mask, for error/audit text.
pub fn entry_days_label(mask: u8) -> String {
    const NAMES: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
    NAMES
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, n)| *n)
        .collect::<Vec<_>>()
        .join(",")
}

pub fn parse_entry_window(s: &str) -> Option<(u32, u32)> {
    let (a, b) = s.split_once('-')?;
    let parse = |t: &str| -> Option<u32> {
        let (h, m) = t.trim().split_once(':')?;
        let h: u32 = h.parse().ok()?;
        let m: u32 = m.parse().ok()?;
        (h < 24 && m < 60).then_some(h * 60 + m)
    };
    let (start, end) = (parse(a)?, parse(b)?);
    (start < end).then_some((start, end))
}

/// Is `now` inside the Eastern wall-clock window? Start inclusive,
/// end exclusive; offset via the shared chrono-tz-free approximation.
/// Time-stop check: has the position been open at least cap minutes?
/// Inclusive at the boundary — a 60-minute cap fires ON minute 60.
pub fn time_stop_due(
    opened_at: chrono::DateTime<chrono::Utc>,
    now: chrono::DateTime<chrono::Utc>,
    cap_minutes: i64,
) -> bool {
    (now - opened_at).num_minutes() >= cap_minutes
}

pub fn in_entry_window(now: chrono::DateTime<chrono::Utc>, window: (u32, u32)) -> bool {
    let minutes = traderview_core::risk_gate::us_eastern_minutes(now);
    (window.0..window.1).contains(&minutes)
}

/// Higher-timeframe trend verdict: Some(true) = the entry side agrees
/// with the trend (long above EMA / short below), Some(false) = it
/// fights it, None = not enough history to compute (caller ALLOWS and
/// logs — a missing higher-TF series must not silently ban a symbol).
pub fn htf_trend_allows(is_long: bool, closes: &[f64], ema_period: usize) -> Option<bool> {
    let ema = traderview_core::indicators::ema(closes, ema_period);
    let last_ema = ema.last().copied().flatten()?;
    let last_close = *closes.last()?;
    Some(if is_long {
        last_close > last_ema
    } else {
        last_close < last_ema
    })
}

/// Revenge-trading check: true while inside the cooldown window after
/// the last losing trip. No loss on record = no cooldown.
pub fn in_loss_cooldown(last_loss_ts: Option<i64>, now_ts: i64, cooldown_minutes: i64) -> bool {
    match last_loss_ts {
        Some(t) => {
            let elapsed_min = (now_ts - t) / 60;
            elapsed_min >= 0 && elapsed_min < cooldown_minutes
        }
        None => false,
    }
}

/// Forward-looking blackout test: true when the NEXT earnings date is
/// today or within `blackout_days` days. Past earnings never block.
pub fn in_earnings_blackout(
    next_earnings: Option<chrono::NaiveDate>,
    today: chrono::NaiveDate,
    blackout_days: i64,
) -> bool {
    match next_earnings {
        Some(d) => {
            let delta = (d - today).num_days();
            (0..=blackout_days).contains(&delta)
        }
        None => false,
    }
}

/// Next scheduled earnings date for the symbol, today or later.
async fn next_earnings_date(
    pool: &PgPool,
    symbol: &str,
    today: chrono::NaiveDate,
) -> Result<Option<chrono::NaiveDate>, EngineError> {
    let row: Option<(chrono::NaiveDate,)> = sqlx::query_as(
        "SELECT earnings_date FROM earnings_events
          WHERE symbol = $1 AND earnings_date >= $2
          ORDER BY earnings_date LIMIT 1",
    )
    .bind(symbol)
    .bind(today)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(d,)| d))
}

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
    check_drawdown_gate(pool, strategy).await?;

    let strat = algo_strategies::from_kind(&strategy.strategy_type, &strategy.entry_rules)
        .map_err(|e| EngineError::Broker(e.to_string()))?;

    // Multi-symbol strategies don't use the single-bar evaluate_entry
    // path. Caller (algo_runner) calls process_bar_window_multi instead.
    if strat.required_symbols().is_some() {
        return Ok(None);
    }

    // ── Exit pass: ask the strategy whether each open position on this
    // symbol should flat at market. Runs BEFORE entry eval so we never
    // pyramid into a position the same tick the rule says to exit.
    let symbol = bars.last().map(|b| b.symbol.clone()).unwrap_or_default();
    if !symbol.is_empty() {
        let exited = run_exit_pass(
            pool,
            sink,
            strategy,
            run_id,
            strat.as_ref(),
            &symbol,
            bars,
            event_sink,
        )
        .await?;
        if exited {
            // A position closed this tick. Skip the entry eval — caller
            // will see Some(_) and increment the "evaluated" counter,
            // and the next tick can decide on a fresh entry.
            return Ok(None);
        }
    }

    let Some(sig) = strat.evaluate_entry(bars, cfg.side_mode) else {
        return Ok(None);
    };
    // Risk gate: account exposure — entries only, AFTER the exit pass
    // so positions can always flatten; the gate self-clears as closes
    // reduce the aggregate.
    check_account_exposure_gate(pool, strategy).await?;
    check_equity_curve_gate(pool, strategy).await?;
    // Risk gate: earnings blackout — entries only; the exit pass above
    // already ran, so positions can always flatten into the print.
    if let Some(days) = cfg.earnings_blackout_days {
        let today = chrono::Utc::now().date_naive();
        let next = next_earnings_date(pool, &symbol, today).await?;
        if in_earnings_blackout(next, today, days) {
            return Err(EngineError::EarningsBlackout {
                symbol: symbol.clone(),
                date: next.unwrap(),
                days,
            });
        }
    }
    // Risk gate: entry window — entries only, same placement rationale.
    if let Some(window) = cfg.entry_window {
        if !in_entry_window(chrono::Utc::now(), window) {
            return Err(EngineError::OutsideEntryWindow {
                window: format!(
                    "{:02}:{:02}-{:02}:{:02}",
                    window.0 / 60, window.0 % 60, window.1 / 60, window.1 % 60
                ),
            });
        }
    }
    // Risk gate: entry days — the "no Mondays / no Fridays" rule,
    // judged in US-Eastern wall time (the session day, not UTC's).
    if let Some(mask) = cfg.entry_days {
        if !traderview_core::risk_gate::in_entry_days(chrono::Utc::now(), mask) {
            return Err(EngineError::OutsideEntryDays {
                days: entry_days_label(mask),
            });
        }
    }
    // Risk gate: daily entry cap — overtrading discipline; exits and
    // already-open positions are untouched.
    if let Some(cap) = cfg.max_entries_per_day {
        let count = algo::entries_today(pool, strategy.id)
            .await
            .map_err(|e| EngineError::Broker(format!("entries_today: {e}")))?;
        if count >= cap {
            return Err(EngineError::DailyEntryCap { cap, count });
        }
    }
    // Risk gate: post-loss cooldown — the revenge-trading brake.
    if let Some(cooldown) = cfg.loss_cooldown_minutes {
        let last = algo::last_losing_trip_ts(pool, strategy.user_id, strategy.id)
            .await
            .map_err(|e| EngineError::Broker(format!("last_losing_trip: {e}")))?;
        let now_ts = chrono::Utc::now().timestamp();
        if in_loss_cooldown(last, now_ts, cooldown) {
            return Err(EngineError::LossCooldown {
                minutes_ago: (now_ts - last.unwrap()) / 60,
                cooldown,
            });
        }
    }
    // Risk gate: entry correlation — don't stack the same trade. The
    // candidate is compared against every OTHER open symbol on the
    // account (same-symbol pyramiding is the pending-order guard's
    // job). Symbols with thin history are skipped, not blocked.
    if let Some(cap) = cfg.max_entry_correlation {
        let open = crate::trades::open_symbols(pool, strategy.account_id)
            .await
            .map_err(|e| EngineError::Broker(format!("open_symbols: {e}")))?;
        let others: Vec<String> = open.into_iter().filter(|s| s != &symbol).collect();
        if !others.is_empty() {
            let to = chrono::Utc::now();
            let from = to - chrono::Duration::days(cfg.correlation_lookback_days);
            let cand_bars = crate::prices::get_bars(
                pool, &symbol, traderview_core::BarInterval::D1, from, to,
            )
            .await
            .map_err(|e| EngineError::Broker(format!("corr bars: {e}")))?;
            let closes = |bars: &[PriceBar]| -> Vec<f64> {
                use rust_decimal::prelude::ToPrimitive;
                bars.iter().map(|b| b.close.to_f64().unwrap_or(0.0)).collect()
            };
            let cand_returns =
                traderview_core::correlation_gate::daily_returns(&closes(&cand_bars));
            let mut open_returns = Vec::with_capacity(others.len());
            for other in &others {
                let bars = crate::prices::get_bars(
                    pool, other, traderview_core::BarInterval::D1, from, to,
                )
                .await
                .map_err(|e| EngineError::Broker(format!("corr bars {other}: {e}")))?;
                open_returns.push((
                    other.clone(),
                    traderview_core::correlation_gate::daily_returns(&closes(&bars)),
                ));
            }
            if let Some(hit) = traderview_core::correlation_gate::worst_correlation(
                &cand_returns,
                &open_returns,
                cap,
                20,
            ) {
                return Err(EngineError::CorrelationGate {
                    symbol: symbol.clone(),
                    other: hit.symbol,
                    rho: hit.rho,
                    cap,
                });
            }
        }
    }
    // Risk gate: higher-timeframe trend filter — longs only above the
    // HTF EMA, shorts only below. Insufficient HTF history allows.
    if let Some((htf_interval, ema_period)) = cfg.htf_filter {
        let to = chrono::Utc::now();
        let from = to
            - chrono::Duration::seconds(htf_interval.seconds() * (ema_period as i64) * 3);
        let htf_bars = crate::prices::get_bars(pool, &symbol, htf_interval, from, to)
            .await
            .map_err(|e| EngineError::Broker(format!("htf bars: {e}")))?;
        let closes: Vec<f64> = {
            use rust_decimal::prelude::ToPrimitive;
            htf_bars.iter().map(|b| b.close.to_f64().unwrap_or(0.0)).collect()
        };
        let is_long = matches!(sig.side, algo_strategies::Side::Buy);
        match htf_trend_allows(is_long, &closes, ema_period) {
            Some(false) => {
                return Err(EngineError::HtfTrendFilter {
                    side: if is_long { "long" } else { "short" },
                    interval: htf_interval.label().to_string(),
                    ema_period,
                });
            }
            Some(true) => {}
            None => {
                tracing::debug!(
                    strategy = %strategy.id, symbol,
                    "htf filter: insufficient higher-TF history; gate skipped"
                );
            }
        }
    }
    let qty = algo_strategies::size_shares(equity, sig.entry_price, sig.stop_distance, &cfg.sizing);
    if qty == 0 {
        return Err(EngineError::ZeroQty);
    }

    // Build the intent up-front — surfaces NaN/Inf/zero prices from the
    // strategy as a clean EngineError instead of silently shipping a
    // zero-priced bracket. Done before the position-size cap so the
    // notional check works on a real entry price.
    let intent = build_intent(strategy, run_id, &symbol, &sig, qty)?;

    // Risk gate: position-size notional cap. Hard reject — we don't
    // shrink the position because qty came from the sizing function
    // that already factored equity + per-trade risk; shrinking it
    // silently would violate the user's risk-per-trade contract.
    if let Some(cap) = cfg.max_position_size_usd {
        let notional = intent.entry_price * Decimal::from(qty);
        if notional > cap {
            return Err(EngineError::PositionSizeCap { notional, cap });
        }
    }

    // Entry-side pyramid guard. On real-Alpaca, immediate_fill is None
    // and the entry fill lands later via the trade_updates WebSocket.
    // open_position_count (passed in from algo_runner) is queried once
    // per tick BEFORE process_bar_window runs — until the WS pump
    // records the fill, the trade row doesn't exist and the count
    // stays stale. The next tick passes the position cap and submits a
    // second bracket. Skip if there's already an outstanding same-side
    // order on (strategy_id, symbol).
    let entry_side_str = side_to_str(sig.side);
    if algo::has_pending_order(pool, strategy.id, &symbol, entry_side_str)
        .await
        .map_err(|e| EngineError::Broker(format!("has_pending_order: {e}")))?
    {
        tracing::debug!(
            strategy = %strategy.id, symbol,
            "entry pass: same-side order already pending, skipping until WS settles"
        );
        return Ok(None);
    }

    algo::increment_run_counter(pool, run_id, algo::RunCounter::SignalsEmitted, 1)
        .await
        .ok();
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
            kind: "entry".into(),
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
            let reason = e.to_string();
            algo::mark_order_submitted(
                pool,
                inserted.id,
                None,
                "rejected",
                None,
                Some(reason.clone()),
            )
            .await
            .map_err(|de| EngineError::Broker(de.to_string()))?;
            if let Some(emit) = event_sink {
                emit(EngineEvent::OrderRejected {
                    strategy_id: strategy.id,
                    symbol: symbol.clone(),
                    side: intent.side,
                    kind: "entry",
                    reason,
                });
            }
            Err(e)
        }
    }
}

/// Record a fill against (a) `algo_fills` for broker audit and (b) the
/// `executions` table tagged with the strategy's `account_id`, then
/// trigger `trades::rollup_account` so the FIFO pipeline materializes
/// the trade row. Exposed `pub(crate)` so a future WS trade_updates
/// handler can drive it with the same shape for real-Alpaca fills.
/// Ensure the "algo:<strategy name>" tag exists and attach it to the
/// most recent trade on (account, symbol) — the row the fill that just
/// rolled up belongs to. Both steps are idempotent.
async fn tag_trade_with_strategy(
    pool: &PgPool,
    strategy: &AlgoStrategy,
    account_id: Uuid,
    symbol: &str,
) -> anyhow::Result<()> {
    let Some(trade_id) = crate::trades::latest_trade_id(pool, account_id, symbol).await? else {
        return Ok(());
    };
    let tag_id = crate::tags::ensure(
        pool,
        strategy.user_id,
        &format!("algo:{}", strategy.name),
        "#7c3aed",
    )
    .await?;
    crate::tags::attach_to_trade(pool, trade_id, tag_id).await
}

pub async fn record_fill(
    pool: &PgPool,
    strategy: &AlgoStrategy,
    intent: &OrderIntent,
    algo_order_id: Uuid,
    fill: &ImmediateFill,
    event_sink: Option<&EventSink>,
) -> Result<(), EngineError> {
    let inserted_fill = algo::insert_fill(
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
    if inserted_fill.is_none() {
        // Idempotency short-circuit. Alpaca's trade_updates WS replays
        // every fill on reconnect — without this guard we'd insert
        // duplicate executions rows and double-count P&L through
        // trades::rollup_account. The broker_fill_id UNIQUE caught
        // the duplicate at the algo_fills layer; skip the rest of the
        // pipeline because the first processing already ran it.
        tracing::debug!(
            strategy = %strategy.id, symbol = %intent.symbol,
            broker_fill_id = ?fill.broker_fill_id,
            "record_fill: fill already recorded; skipping pipeline"
        );
        return Ok(());
    }
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
        broker_order_id: fill.broker_order_id.clone().or_else(|| fill.broker_fill_id.clone()),
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
    // Auto-tag the trade with its strategy so the journal's existing
    // by-tag report gives per-strategy P&L without a schema change.
    // Non-fatal: a tagging hiccup must never fail the fill pipeline.
    if let Err(e) = tag_trade_with_strategy(pool, strategy, account_id, &intent.symbol).await {
        tracing::warn!(strategy = %strategy.id, error = %e, "algo trade auto-tag failed");
    }
    // If this fill closed a trade, accumulate its net_pnl into the run's
    // pnl_realized. Without this, `today_realized_pnl` and the daily-loss
    // risk gate always see 0 and the auto-pause never fires. Window the
    // lookup at `fill.executed_at - 5s` so a clock skew between the
    // broker-side fill timestamp and our DB write doesn't miss the close.
    let since = fill
        .executed_at
        .checked_sub_signed(chrono::Duration::seconds(5))
        .unwrap_or(fill.executed_at);
    let delta = crate::trades::realized_pnl_closed_since(pool, account_id, &intent.symbol, since)
        .await
        .map_err(|e| EngineError::Broker(format!("realized_pnl_closed_since: {e}")))?;
    if let Some(d) = delta {
        algo::add_realized_pnl(pool, intent.run_id, d)
            .await
            .map_err(|e| EngineError::Broker(format!("add_realized_pnl: {e}")))?;
    }
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
    // Paper-lock check — was missing in the original multi-symbol path,
    // letting pair strategies submit live before the paper lock expired.
    if strategy.broker_mode == "live" && chrono::Utc::now() <= strategy.paper_locked_until {
        return Err(EngineError::PaperLocked(strategy.paper_locked_until));
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
    check_drawdown_gate(pool, strategy).await?;
    let strat = algo_strategies::from_kind(&strategy.strategy_type, &strategy.entry_rules)
        .map_err(|e| EngineError::Broker(e.to_string()))?;
    let primary_symbol = strat
        .required_symbols()
        .and_then(|v| v.into_iter().next())
        .unwrap_or_default();

    // Exit pass per leg. Pairs/stat-arb open one position per symbol,
    // so we walk every leg's bars and ask the strategy whether each
    // open position should flat. Without this, pair strategies leak
    // positions — entry submits but evaluate_exit never gets called.
    let mut any_exited = false;
    for (sym, bars) in bars_by_symbol.iter() {
        if bars.is_empty() {
            continue;
        }
        let exited = run_exit_pass(
            pool,
            sink,
            strategy,
            run_id,
            strat.as_ref(),
            sym,
            bars,
            event_sink,
        )
        .await?;
        if exited {
            any_exited = true;
        }
    }
    if any_exited {
        return Ok(None);
    }

    let Some(sig) = strat.evaluate_entry_multi(bars_by_symbol, cfg.side_mode) else {
        return Ok(None);
    };
    // Account exposure — entries only, after the per-leg exit pass so
    // pairs can always flatten; self-clears as closes reduce the sum.
    check_account_exposure_gate(pool, strategy).await?;
    check_equity_curve_gate(pool, strategy).await?;
    let qty = algo_strategies::size_shares(equity, sig.entry_price, sig.stop_distance, &cfg.sizing);
    if qty == 0 {
        return Err(EngineError::ZeroQty);
    }
    // Build the intent up-front so NaN/Inf/zero prices fail fast as a
    // clean EngineError rather than silently producing a zero-priced
    // bracket. The position-size cap below uses intent.entry_price.
    let intent = build_intent(strategy, run_id, &primary_symbol, &sig, qty)?;
    if let Some(cap) = cfg.max_position_size_usd {
        let notional = intent.entry_price * Decimal::from(qty);
        if notional > cap {
            return Err(EngineError::PositionSizeCap { notional, cap });
        }
    }
    // Same pyramid guard as the single-symbol path — skip if there's
    // an outstanding same-side entry on the primary leg still pending
    // the WS fill.
    let entry_side_str = side_to_str(sig.side);
    if algo::has_pending_order(pool, strategy.id, &primary_symbol, entry_side_str)
        .await
        .map_err(|e| EngineError::Broker(format!("has_pending_order: {e}")))?
    {
        tracing::debug!(
            strategy = %strategy.id, symbol = %primary_symbol,
            "entry pass (multi): same-side order already pending, skipping"
        );
        return Ok(None);
    }
    algo::increment_run_counter(pool, run_id, algo::RunCounter::SignalsEmitted, 1)
        .await
        .ok();
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
            kind: "entry".into(),
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
            let reason = e.to_string();
            algo::mark_order_submitted(
                pool,
                inserted.id,
                None,
                "rejected",
                None,
                Some(reason.clone()),
            )
            .await
            .map_err(|de| EngineError::Broker(de.to_string()))?;
            if let Some(emit) = event_sink {
                emit(EngineEvent::OrderRejected {
                    strategy_id: strategy.id,
                    symbol: primary_symbol.clone(),
                    side: intent.side,
                    kind: "entry",
                    reason,
                });
            }
            Err(e)
        }
    }
}

/// Convert a strategy's f64 prices into Decimal for the broker intent.
/// Returns Err when ANY price is non-finite (NaN/Inf) or non-positive —
/// previously these silently became Decimal::ZERO, so an indicator math
/// glitch would submit a $0 stop or $0 take-profit (broker either
/// rejects, or worse on some venues accepts as a market-at-zero leg).
fn build_intent(
    strategy: &AlgoStrategy,
    run_id: Uuid,
    symbol: &str,
    sig: &EntrySignal,
    qty: u64,
) -> Result<OrderIntent, EngineError> {
    let entry_price = finite_positive_dec(sig.entry_price, "entry_price")?;
    let stop_price = finite_positive_dec(sig.stop_price, "stop_price")?;
    let take_profit_price = finite_positive_dec(sig.take_profit_price, "take_profit_price")?;
    Ok(OrderIntent {
        strategy_id: strategy.id,
        run_id,
        client_order_id: Uuid::new_v4(),
        symbol: symbol.to_string(),
        side: sig.side,
        qty: Decimal::from(qty),
        entry_price,
        stop_price,
        take_profit_price,
    })
}

/// Convert a strategy-emitted f64 price into Decimal, rejecting NaN /
/// Inf / non-positive values that would otherwise become Decimal::ZERO
/// via `unwrap_or_default` and ship a zero-priced bracket to the broker.
fn finite_positive_dec(v: f64, field: &'static str) -> Result<Decimal, EngineError> {
    if !v.is_finite() || v <= 0.0 {
        return Err(EngineError::Broker(format!(
            "signal {field} is not finite-positive: {v}"
        )));
    }
    Decimal::from_f64(v).ok_or_else(|| {
        EngineError::Broker(format!("signal {field} cannot be represented as Decimal: {v}"))
    })
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

/// Ask the strategy whether each open position on `symbol` should flat
/// at market this tick. Iterates `trades.open_positions_for_symbol`,
/// computes per-position anchor_high / anchor_low from the bars window,
/// calls `Strategy::evaluate_exit`, and submits a single-leg market
/// close via the sink on a signal. Returns `true` if at least one
/// position was closed so the caller can skip the entry pass.
async fn run_exit_pass(
    pool: &PgPool,
    sink: &dyn BrokerSink,
    strategy: &AlgoStrategy,
    run_id: Uuid,
    strat: &dyn traderview_core::algo_strategies::Strategy,
    symbol: &str,
    bars: &[PriceBar],
    event_sink: Option<&EventSink>,
) -> Result<bool, EngineError> {
    use rust_decimal::prelude::ToPrimitive;

    let positions = crate::trades::open_positions_for_symbol(pool, strategy.account_id, symbol)
        .await
        .map_err(|e| EngineError::Broker(format!("open_positions_for_symbol: {e}")))?;
    if positions.is_empty() {
        return Ok(false);
    }
    let mut closed_any = false;
    for pos in positions {
        // Position side recorded in `trades.side` is "long" or "short"
        // (the entry direction). evaluate_exit takes that same Side.
        let entry_side = match pos.side.as_str() {
            "long" => Side::Buy,
            "short" => Side::Sell,
            other => {
                tracing::debug!(
                    strategy = %strategy.id, symbol, side = other,
                    "exit pass: unknown trade.side, skipping"
                );
                continue;
            }
        };
        let entry_price = pos.entry_avg.to_f64().unwrap_or(0.0);
        // Anchor = high-water / low-water from entry through the current
        // bar window. Bars older than `opened_at` aren't part of this
        // position's lifetime so they're filtered out. If the window
        // doesn't reach the entry (entry is older than window), the
        // entry_avg itself anchors the calc — degenerate but correct
        // for momentum / trailing-stop logic that just wants "best
        // price since I'm long".
        let mut anchor_high = entry_price;
        let mut anchor_low = entry_price;
        for b in bars.iter().filter(|b| b.bar_time >= pos.opened_at) {
            let h = b.high.to_f64().unwrap_or(0.0);
            let l = b.low.to_f64().unwrap_or(0.0);
            if h > anchor_high {
                anchor_high = h;
            }
            if l < anchor_low {
                anchor_low = l;
            }
        }
        // Time stop overlays the strategy's exit opinion: a position
        // past its max hold flattens at the last close no matter what
        // evaluate_exit thinks. Exits are forced here, never blocked.
        let cfg = EngineConfig::from_strategy(strategy);
        let time_stopped = cfg
            .max_hold_minutes
            .is_some_and(|cap| time_stop_due(pos.opened_at, chrono::Utc::now(), cap));
        let exit_sig = if time_stopped {
            traderview_core::algo_strategies::ExitSignal {
                reason: "time_stop",
                exit_price: bars
                    .last()
                    .and_then(|b| b.close.to_f64())
                    .unwrap_or(entry_price),
                trigger_index: bars.len().saturating_sub(1),
            }
        } else {
            match strat.evaluate_exit(bars, entry_side, anchor_high, anchor_low) {
                Some(s) => s,
                None => continue,
            }
        };
        let close_side = match entry_side {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        };
        // Skip if a previous tick already submitted a close that hasn't
        // come back as filled/rejected/canceled yet. On real Alpaca the
        // fill lands via the trade_updates WebSocket — until that pump
        // catches up, the trade row stays 'open' and the next tick
        // would re-submit the same close, over-selling N times before
        // it settles. InMemorySink and the paper sink return immediate
        // fills so this is a no-op for them.
        let close_side_str = side_to_str(close_side);
        let pending = algo::has_pending_order(pool, strategy.id, symbol, close_side_str)
            .await
            .map_err(|e| EngineError::Broker(format!("has_pending_order: {e}")))?;
        if pending {
            tracing::debug!(
                strategy = %strategy.id, symbol, trade_id = %pos.id,
                "exit pass: close already pending, skipping until WS settles"
            );
            continue;
        }
        // Re-fetch qty right before submitting — a partial entry-fill or
        // a prior close fill landing via the WS pump can shrink the
        // position between the snapshot at the top of run_exit_pass and
        // now. Using the stale pos.qty would over-sell.
        let live_qty =
            match crate::trades::get_open_qty(pool, pos.id)
                .await
                .map_err(|e| EngineError::Broker(format!("get_open_qty: {e}")))?
            {
                Some(q) if q > Decimal::ZERO => q,
                _ => {
                    tracing::debug!(
                        strategy = %strategy.id, symbol, trade_id = %pos.id,
                        "exit pass: trade already closed or zero qty, skipping"
                    );
                    continue;
                }
            };
        // Same finite-positive guard as build_intent. Previously,
        // unwrap_or_default zeroed a NaN/Inf exit price → the close
        // ran at $0 (in sim) or sent a $0 LIMIT in extended hours.
        let exit_price_dec = match finite_positive_dec(exit_sig.exit_price, "exit_price") {
            Ok(d) => d,
            Err(e) => {
                tracing::error!(
                    strategy = %strategy.id, symbol, trade_id = %pos.id,
                    error = %e,
                    "exit pass: non-finite exit price; skipping this position"
                );
                continue;
            }
        };
        let coid = Uuid::new_v4();
        // Insert the close as a real algo_orders row first — algo_fills
        // has a FK on order_id, so record_fill needs an order_id that
        // exists before it can persist the close fill.
        let close_order = algo::insert_order(
            pool,
            run_id,
            strategy.id,
            AlgoOrderInsert {
                client_order_id: coid,
                symbol: symbol.to_string(),
                side: close_side_str.into(),
                order_type: "market".into(),
                order_class: "simple".into(),
                kind: "exit".into(),
                qty: live_qty,
                limit_price: None,
                stop_price: None,
                raw_request: Some(serde_json::json!({
                    "kind": "exit",
                    "trade_id": pos.id,
                    "reason": exit_sig.reason,
                    "exit_price": exit_sig.exit_price,
                })),
            },
        )
        .await
        .map_err(|e| EngineError::Broker(e.to_string()))?;
        let resp = match sink
            .submit_market_close(symbol.to_string(), close_side, live_qty, exit_price_dec, coid)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::info!(
                    strategy = %strategy.id, symbol, trade_id = %pos.id,
                    error = %e,
                    "submit_market_close failed; position stays open"
                );
                let reason = e.to_string();
                algo::mark_order_submitted(
                    pool,
                    close_order.id,
                    None,
                    "rejected",
                    None,
                    Some(reason.clone()),
                )
                .await
                .map_err(|e| EngineError::Broker(e.to_string()))?;
                if let Some(emit) = event_sink {
                    emit(EngineEvent::OrderRejected {
                        strategy_id: strategy.id,
                        symbol: symbol.to_string(),
                        side: close_side,
                        kind: "exit",
                        reason,
                    });
                }
                continue;
            }
        };
        algo::mark_order_submitted(
            pool,
            close_order.id,
            Some(resp.broker_order_id.clone()),
            &resp.status,
            resp.raw_response.clone(),
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
                order_id: close_order.id,
                symbol: symbol.to_string(),
                side: close_side,
                qty: live_qty,
                broker_order_id: resp.broker_order_id.clone(),
            });
        }
        if let Some(fill) = resp.immediate_fill {
            let exit_intent = OrderIntent {
                strategy_id: strategy.id,
                run_id,
                client_order_id: coid,
                symbol: symbol.to_string(),
                side: close_side,
                qty: live_qty,
                entry_price: exit_price_dec,
                stop_price: Decimal::ZERO,
                take_profit_price: Decimal::ZERO,
            };
            // record_fill inserts the closing execution; trades::rollup_account
            // matches it to the open trade and writes status='closed'.
            record_fill(pool, strategy, &exit_intent, close_order.id, &fill, event_sink).await?;
        }
        closed_any = true;
    }
    Ok(closed_any)
}

#[cfg(test)]
mod earnings_blackout_tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn boundary_days_pin_the_window() {
        let today = d(2026, 6, 10);
        // Earnings TODAY blocks; exactly N days out blocks; N+1 clears.
        assert!(in_earnings_blackout(Some(today), today, 2));
        assert!(in_earnings_blackout(Some(d(2026, 6, 12)), today, 2));
        assert!(!in_earnings_blackout(Some(d(2026, 6, 13)), today, 2));
    }

    #[test]
    fn past_earnings_and_no_earnings_never_block() {
        let today = d(2026, 6, 10);
        // Yesterday's print is history — the gate is forward-looking.
        assert!(!in_earnings_blackout(Some(d(2026, 6, 9)), today, 5));
        assert!(!in_earnings_blackout(None, today, 5));
    }

    #[test]
    fn time_stop_boundary_is_inclusive() {
        use chrono::TimeZone;
        let opened = chrono::Utc.with_ymd_and_hms(2026, 6, 12, 14, 0, 0).unwrap();
        // 59 minutes in: not due. Exactly 60: due — the cap means
        // "at least", not "strictly more than".
        assert!(!time_stop_due(opened, opened + chrono::Duration::minutes(59), 60));
        assert!(time_stop_due(opened, opened + chrono::Duration::minutes(60), 60));
        assert!(time_stop_due(opened, opened + chrono::Duration::days(3), 60));
    }

    #[test]
    fn entry_window_parse_pins_format_and_ordering() {
        assert_eq!(parse_entry_window("10:00-15:30"), Some((600, 930)));
        assert_eq!(parse_entry_window(" 9:35 - 11:00 "), Some((575, 660)));
        // Malformed, out-of-range, and non-increasing all disable.
        assert_eq!(parse_entry_window("10:00"), None);
        assert_eq!(parse_entry_window("25:00-26:00"), None);
        assert_eq!(parse_entry_window("10:61-11:00"), None);
        assert_eq!(parse_entry_window("15:30-10:00"), None);
        assert_eq!(parse_entry_window("10:00-10:00"), None);
    }

    #[test]
    fn entry_window_check_pins_edt_boundaries() {
        use chrono::TimeZone;
        let w = (600, 930); // 10:00-15:30 ET
        // June = EDT (UTC-4): 14:00 UTC = 10:00 ET — start is inclusive.
        let t = chrono::Utc.with_ymd_and_hms(2026, 6, 10, 14, 0, 0).unwrap();
        assert!(in_entry_window(t, w));
        // 13:59 UTC = 09:59 ET — before the window.
        let t = chrono::Utc.with_ymd_and_hms(2026, 6, 10, 13, 59, 0).unwrap();
        assert!(!in_entry_window(t, w));
        // 19:30 UTC = 15:30 ET — end is exclusive.
        let t = chrono::Utc.with_ymd_and_hms(2026, 6, 10, 19, 30, 0).unwrap();
        assert!(!in_entry_window(t, w));
        // January = EST (UTC-5): 15:00 UTC = 10:00 ET.
        let t = chrono::Utc.with_ymd_and_hms(2026, 1, 13, 15, 0, 0).unwrap();
        assert!(in_entry_window(t, w));
    }

    #[test]
    fn loss_cooldown_pins_window_and_no_loss_case() {
        // Loss closed 10 min ago, 30-min cooldown: blocked.
        assert!(in_loss_cooldown(Some(1_000_000), 1_000_600, 30));
        // Exactly at the boundary (30 min): clear — the window is [0, 30).
        assert!(!in_loss_cooldown(Some(1_000_000), 1_001_800, 30));
        // 29 min 59s: still blocked (integer minutes floor).
        assert!(in_loss_cooldown(Some(1_000_000), 1_001_799, 30));
        // No loss on record / clock skew (loss in the future): clear.
        assert!(!in_loss_cooldown(None, 1_000_000, 30));
        assert!(!in_loss_cooldown(Some(1_000_600), 1_000_000, 30));
    }

    #[test]
    fn gate_names_cover_gate_variants_and_skip_infra() {
        use rust_decimal::Decimal;
        let gates: Vec<EngineError> = vec![
            EngineError::PositionCap(5),
            EngineError::DailyLossCap { cap: Decimal::ONE, pnl: Decimal::ZERO },
            EngineError::ConsecutiveLossesCap { cap: 3, streak: 3 },
            EngineError::PositionSizeCap { notional: Decimal::ONE, cap: Decimal::ONE },
            EngineError::EarningsBlackout {
                symbol: "X".into(),
                date: chrono::NaiveDate::from_ymd_opt(2026, 6, 12).unwrap(),
                days: 2,
            },
            EngineError::OutsideEntryWindow { window: "10:00-15:30".into() },
            EngineError::DailyEntryCap { cap: 6, count: 6 },
            EngineError::LossCooldown { minutes_ago: 5, cooldown: 30 },
            EngineError::CorrelationGate {
                symbol: "X".into(), other: "Y".into(), rho: 0.9, cap: 0.8,
            },
            EngineError::HtfTrendFilter {
                side: "long", interval: "1h".into(), ema_period: 50,
            },
        ];
        let names: Vec<&str> = gates.iter().filter_map(|e| e.gate_name()).collect();
        assert_eq!(names.len(), gates.len(), "every gate variant must name itself");
        // Infra/state errors are NOT gate fires.
        assert_eq!(EngineError::ZeroQty.gate_name(), None);
        assert_eq!(EngineError::Broker("x".into()).gate_name(), None);
        assert_eq!(EngineError::KillSwitch { reason: None }.gate_name(), None);
    }

    #[test]
    fn htf_trend_pins_sides_and_insufficient_history() {
        // Rising series: last close above its EMA.
        let up: Vec<f64> = (0..60).map(|i| 100.0 + i as f64).collect();
        assert_eq!(htf_trend_allows(true, &up, 20), Some(true));
        assert_eq!(htf_trend_allows(false, &up, 20), Some(false));
        // Falling series: shorts agree, longs fight.
        let down: Vec<f64> = (0..60).map(|i| 160.0 - i as f64).collect();
        assert_eq!(htf_trend_allows(true, &down, 20), Some(false));
        assert_eq!(htf_trend_allows(false, &down, 20), Some(true));
        // Too little history to compute the EMA: None (caller allows).
        assert_eq!(htf_trend_allows(true, &up[..10], 20), None);
        assert_eq!(htf_trend_allows(true, &[], 20), None);
    }

    #[test]
    fn correlation_cap_rejects_out_of_range() {
        let get = |v: serde_json::Value| -> Option<f64> {
            v.get("max_entry_correlation")
                .and_then(|x| x.as_f64())
                .filter(|c| (0.0..1.0).contains(c) && *c > 0.0)
        };
        assert_eq!(get(serde_json::json!({"max_entry_correlation": 0.8})), Some(0.8));
        // 1.0 would never fire (|rho| <= 1); 0 and negatives are nonsense.
        assert_eq!(get(serde_json::json!({"max_entry_correlation": 1.0})), None);
        assert_eq!(get(serde_json::json!({"max_entry_correlation": 0.0})), None);
        assert_eq!(get(serde_json::json!({"max_entry_correlation": -0.5})), None);
    }

    #[test]
    fn max_entries_per_day_rejects_nonpositive() {
        let get = |v: serde_json::Value| -> Option<i64> {
            v.get("max_entries_per_day")
                .and_then(|x| x.as_i64())
                .filter(|n| *n > 0)
        };
        assert_eq!(get(serde_json::json!({"max_entries_per_day": 6})), Some(6));
        assert_eq!(get(serde_json::json!({"max_entries_per_day": 0})), None);
        assert_eq!(get(serde_json::json!({"max_entries_per_day": -2})), None);
    }

    #[test]
    fn config_parses_and_rejects_nonpositive() {
        let mk = |v: serde_json::Value| {
            serde_json::json!({ "earnings_blackout_days": v })
        };
        let get = |rg: serde_json::Value| -> Option<i64> {
            rg.get("earnings_blackout_days")
                .and_then(|v| v.as_i64())
                .filter(|n| *n > 0)
        };
        assert_eq!(get(mk(serde_json::json!(2))), Some(2));
        assert_eq!(get(mk(serde_json::json!(0))), None);
        assert_eq!(get(mk(serde_json::json!(-3))), None);
    }
}
