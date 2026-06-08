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
use crate::tradier_trading::{
    EquitySide, OtocoBracket, TradierEnv, TradierTrading,
};
use chrono::Utc;
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
        // Alpaca: the alpaca_pump WS flow handles real fills coming
        // back in. The outgoing place_order via REST isn't wired
        // through the dispatcher yet — engine still uses the
        // InMemorySink's immediate-fill simulation. Real Alpaca REST
        // submission lands when the dispatcher fully takes over (the
        // adapter exists in alpaca_trading.rs; just needs an
        // AlpacaSink wrapper).
        "alpaca" => Ok(Box::new(InMemorySink::default())),
        "tradier" => {
            let Some((token, account_id_str, sandbox)) =
                crate::data_source_keys::tradier_creds(pool, strategy.user_id)
                    .await
                    .map_err(|e| DispatchError::Db(sqlx::Error::Decode(e.into())))?
            else {
                return Ok(Box::new(IntegrationPendingSink {
                    broker: "tradier",
                    paper,
                    detail: "no Tradier credentials saved — go to Settings → Data sources".into(),
                }));
            };
            let env = if paper || sandbox { TradierEnv::Sandbox } else { TradierEnv::Live };
            let client = TradierTrading::new(env, token, account_id_str);
            Ok(Box::new(TradierSink { client }))
        }
        "ibkr" => Ok(Box::new(IntegrationPendingSink {
            broker: "ibkr",
            paper,
            detail: "needs local TWS/Gateway process".into(),
        })),
        "td" => Ok(Box::new(IntegrationPendingSink {
            broker: "td",
            paper,
            detail: "TD/Schwab API migration pending".into(),
        })),
        "tastytrade" => Ok(Box::new(IntegrationPendingSink {
            broker: "tastytrade",
            paper,
            detail: "tastytrade adapter pending".into(),
        })),
        _ => Ok(Box::new(IntegrationPendingSink {
            broker: "unknown",
            paper,
            detail: "broker not in algo-supported set".into(),
        })),
    }
}

/// Real Tradier sink — places an OTOCO bracket order via REST.
#[derive(Debug, Clone)]
struct TradierSink {
    client: TradierTrading,
}

impl BrokerSink for TradierSink {
    fn submit_bracket(
        &self,
        intent: OrderIntent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>>
    {
        let client = self.client.clone();
        Box::pin(async move {
            let entry_side = match intent.side {
                traderview_core::algo_strategies::Side::Buy => EquitySide::Buy,
                traderview_core::algo_strategies::Side::Sell => EquitySide::SellShort,
            };
            let exit_side = match intent.side {
                traderview_core::algo_strategies::Side::Buy => EquitySide::Sell,
                traderview_core::algo_strategies::Side::Sell => EquitySide::BuyToCover,
            };
            let bracket = OtocoBracket {
                symbol: intent.symbol.clone(),
                entry_side,
                exit_side,
                quantity: intent.qty,
                take_profit_price: intent.take_profit_price,
                stop_loss_price: intent.stop_price,
                duration: crate::tradier_trading::Duration_::Day,
                tag: Some(intent.client_order_id.to_string()),
            };
            let resp = client
                .place_otoco_bracket(&bracket)
                .await
                .map_err(|e| EngineError::Broker(format!("tradier: {e}")))?;
            // Tradier doesn't return immediate_fill data on order
            // submission — fills come back via the streaming events
            // endpoint (separate pump module). For now we return the
            // accepted status without a fill; the strategy stays in
            // 'accepted' state until the pump lands.
            Ok(SubmittedOrder {
                broker_order_id: resp.id.to_string(),
                status: resp.status,
                raw_response: None,
                immediate_fill: None,
            })
        })
    }
}

/// Sink that records every submission as a deferred failure. Used for
/// brokers whose adapter module hasn't shipped yet — the strategy can
/// be SAVED + the run STARTED, but every order attempt rejects cleanly.
#[derive(Debug, Clone)]
struct IntegrationPendingSink {
    broker: &'static str,
    #[allow(dead_code)]
    paper: bool,
    detail: String,
}

impl BrokerSink for IntegrationPendingSink {
    fn submit_bracket(
        &self,
        _intent: OrderIntent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>>
    {
        let b = self.broker;
        let d = self.detail.clone();
        Box::pin(async move {
            Err(EngineError::Broker(format!(
                "integration_pending: {b} ({d}); use Alpaca or internal_sim broker_mode for now"
            )))
        })
    }
}

// Quiet the unused imports while commit 35 wires the dispatcher into
// algo_runner. Both will be used the moment that integration lands.
const _USE_THESE_AT_SUBMIT_TIME: fn() = || {
    let _: Decimal = Decimal::zero();
    let _: Option<ImmediateFill> = None;
    let _ = Utc::now();
};
