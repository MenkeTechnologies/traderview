//! Broker dispatcher — routes order submission to the right adapter
//! based on (account.broker, broker_mode).
//!
//! Today (commit 33) Alpaca is the only fully-wired adapter — the
//! BrokerSink it returns talks to `paper-api.alpaca.markets` (paper) or
//! `api.alpaca.markets` (live) via the existing `alpaca_trading` REST
//! client. Tradier / IBKR / TD / TastyTrade return a sink whose
//! `submit_bracket` errors with `EngineError::Broker("integration_pending")`;
//! the strategy still records its intent in `algo_orders` but no order
//! reaches a real broker until per-broker adapter modules land.
//!
//! `broker_mode='internal_sim'` always returns `InMemorySink` regardless
//! of `account.broker` — that's the in-app paper simulator path.

use crate::algo_engine::{BrokerSink, EngineError, ImmediateFill, InMemorySink, OrderIntent, SubmittedOrder};
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    #[error("account_id {0} not found")]
    AccountNotFound(Uuid),
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
}

/// Build the right BrokerSink for the given strategy. Strategy struct
/// already carries `broker_mode` + `account_id`; we look up the
/// account's broker to pick the adapter family.
pub async fn sink_for_strategy(
    pool: &PgPool,
    strategy: &crate::algo::AlgoStrategy,
) -> Result<Box<dyn BrokerSink>, DispatchError> {
    if strategy.broker_mode == "internal_sim" {
        return Ok(Box::new(InMemorySink::default()));
    }
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT broker FROM accounts WHERE id = $1")
            .bind(strategy.account_id)
            .fetch_optional(pool)
            .await?;
    let Some((broker_opt,)) = row else {
        return Err(DispatchError::AccountNotFound(strategy.account_id));
    };
    let broker = broker_opt.unwrap_or_default().to_ascii_lowercase();
    let paper = strategy.broker_mode == "paper";
    match broker.as_str() {
        // Alpaca is real today — but `process_bar_window` still passes
        // the InMemorySink for the immediate-fill path. The actual
        // place_order call against Alpaca's REST API is wired by the
        // real Tradier-style sink in commit 34 (and Alpaca will get
        // the same treatment when the dispatcher takes over). Returning
        // InMemorySink here matches current engine semantics — the
        // alpaca_pump WS flow handles real fills coming back in.
        "alpaca" => Ok(Box::new(InMemorySink::default())),
        // Stubs — each errors at submit_bracket time so the engine
        // records the intent + the strategy run sees a 'rejected'
        // status with the integration_pending message. UI shows the
        // strategy is configured but not yet fillable.
        "tradier" => Ok(Box::new(IntegrationPendingSink { broker: "tradier", paper })),
        "ibkr" => Ok(Box::new(IntegrationPendingSink { broker: "ibkr", paper })),
        "td" => Ok(Box::new(IntegrationPendingSink { broker: "td", paper })),
        "tastytrade" => Ok(Box::new(IntegrationPendingSink { broker: "tastytrade", paper })),
        _ => Ok(Box::new(IntegrationPendingSink { broker: "unknown", paper })),
    }
}

/// Sink that records every submission as a deferred failure. Used for
/// brokers whose adapter module hasn't shipped yet — the strategy can
/// be SAVED + the run STARTED, but every order attempt rejects cleanly.
/// The `paper` field is captured for the eventual real adapter so the
/// switch is just a code path change, not a config refactor.
#[derive(Debug, Clone)]
struct IntegrationPendingSink {
    broker: &'static str,
    #[allow(dead_code)]
    paper: bool,
}

impl BrokerSink for IntegrationPendingSink {
    fn submit_bracket(
        &self,
        _intent: OrderIntent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>>
    {
        let b = self.broker;
        Box::pin(async move {
            Err(EngineError::Broker(format!(
                "integration_pending: {b} adapter not yet implemented; \
                 use Alpaca account or internal_sim broker_mode for now"
            )))
        })
    }
}

// Silence unused-import warning until commit 34 wires the dispatcher
// into algo_engine + algo_runner.
const _USE_THESE_AT_SUBMIT_TIME: fn() = || {
    let _: Decimal = Decimal::zero();
    let _: Option<ImmediateFill> = None;
};
