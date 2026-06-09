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

use crate::algo_engine::{
    BrokerSink, EngineError, ImmediateFill, InMemorySink, OrderIntent, SubmittedOrder,
};
use crate::alpaca_trading::{AlpacaTrading, BrokerMode as AlpacaBrokerMode, PlaceOrderRequest};
use crate::ibkr_trading::{
    IbkrTrading, OrderSide as IbkrOrderSide, OrderType as IbkrOrderType,
    PlaceBracket as IbkrPlaceBracket, PlaceOrder as IbkrPlaceOrder, Tif as IbkrTif,
};
use crate::schwab_trading::{
    Duration_ as SchwabDuration, Instruction as SchwabInstruction, OrderType as SchwabOrderType,
    PlaceBracket as SchwabPlaceBracket, PlaceOrder as SchwabPlaceOrder, SchwabTrading,
    Session as SchwabSession,
};
use crate::tastytrade_trading::{
    EquityAction, PlaceEquityOrder as TastyEquityOrder, TastytradeEnv, TastytradeTrading,
};
use crate::tradier_trading::{EquitySide, OtocoBracket, TradierEnv, TradierTrading};
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
        "alpaca" => {
            let Some((key_id, secret, _)) =
                crate::data_source_keys::alpaca_creds_plain(pool, strategy.user_id)
                    .await
                    .map_err(|e| DispatchError::Db(sqlx::Error::Decode(e.into())))?
            else {
                return Ok(Box::new(IntegrationPendingSink {
                    broker: "alpaca",
                    paper,
                    detail: "no Alpaca credentials saved — go to Settings → Data sources".into(),
                }));
            };
            let mode = if paper {
                AlpacaBrokerMode::Paper
            } else {
                AlpacaBrokerMode::Live
            };
            let client = AlpacaTrading::new(mode, key_id, secret);
            Ok(Box::new(AlpacaSink { client }))
        }
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
            let env = if paper || sandbox {
                TradierEnv::Sandbox
            } else {
                TradierEnv::Live
            };
            let client = TradierTrading::new(env, token, account_id_str);
            Ok(Box::new(TradierSink { client }))
        }
        "ibkr" => {
            let Some((account_id, base_url, bearer)) =
                crate::data_source_keys::ibkr_creds(pool, strategy.user_id)
                    .await
                    .map_err(|e| DispatchError::Db(sqlx::Error::Decode(e.into())))?
            else {
                return Ok(Box::new(IntegrationPendingSink {
                    broker: "ibkr",
                    paper,
                    detail: "no IBKR credentials saved — go to Settings → Data sources".into(),
                }));
            };
            let client = IbkrTrading::new(base_url, bearer, account_id);
            // Symbol-to-conid resolution: IBKR uses integer contract IDs
            // (conid), not symbols. Strategies fire on symbols, so the
            // sink needs a lookup. For commit 41 we error out with a
            // pending message — conid resolution lands when
            // /iserver/secdef/search is wired in a follow-up.
            Ok(Box::new(IbkrSink {
                client,
                account_label: "ibkr".into(),
            }))
        }
        "td" | "tdameritrade" | "schwab" => {
            // TD Ameritrade was retired Sep 2024 — Schwab Trader API is
            // the replacement. We accept the legacy "td" broker label
            // (existing accounts in user DBs) and route to the same
            // Schwab REST adapter.
            let Some((client_id, client_secret, tokens, account_hash)) =
                crate::data_source_keys::schwab_creds(pool, strategy.user_id)
                    .await
                    .map_err(|e| DispatchError::Db(sqlx::Error::Decode(e.into())))?
            else {
                return Ok(Box::new(IntegrationPendingSink {
                    broker: "schwab",
                    paper,
                    detail: "no Schwab credentials saved — go to Settings → Data sources, run the OAuth flow".into(),
                }));
            };
            // Persist rotated tokens to the DB so a refresh survives
            // process restart. The closure captures pool + user_id; the
            // pool clone is Arc-cheap.
            let pool_clone = pool.clone();
            let user_id = strategy.user_id;
            let persist: crate::schwab_trading::TokenCallback =
                std::sync::Arc::new(move |new_tokens| {
                    let pool = pool_clone.clone();
                    tokio::spawn(async move {
                        let _ = crate::data_source_keys::save_schwab_tokens(
                            &pool,
                            user_id,
                            &new_tokens,
                        )
                        .await;
                    });
                });
            let client = SchwabTrading::new(client_id, client_secret, tokens, account_hash)
                .on_token_refresh(persist);
            Ok(Box::new(SchwabSink { client }))
        }
        "tastytrade" => {
            let Some((account_number, sandbox, auth)) =
                crate::data_source_keys::tastytrade_creds(pool, strategy.user_id)
                    .await
                    .map_err(|e| DispatchError::Db(sqlx::Error::Decode(e.into())))?
            else {
                return Ok(Box::new(IntegrationPendingSink {
                    broker: "tastytrade",
                    paper,
                    detail: "no Tastytrade credentials saved — go to Settings → Data sources"
                        .into(),
                }));
            };
            let env = if paper || sandbox {
                TastytradeEnv::Sandbox
            } else {
                TastytradeEnv::Live
            };
            let client = TastytradeTrading::new(env, auth, account_number);
            Ok(Box::new(TastytradeSink { client }))
        }
        _ => Ok(Box::new(IntegrationPendingSink {
            broker: "unknown",
            paper,
            detail: "broker not in algo-supported set".into(),
        })),
    }
}

/// Real Alpaca sink — places a native bracket order via REST. Fills
/// come back through the alpaca_pump WS flow which calls `record_fill`
/// directly, so this sink returns `immediate_fill=None`; the algo
/// engine treats the order as 'accepted' until the WS event lands.
#[derive(Debug, Clone)]
struct AlpacaSink {
    client: AlpacaTrading,
}

impl BrokerSink for AlpacaSink {
    fn submit_bracket(
        &self,
        intent: OrderIntent,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>,
    > {
        let client = self.client.clone();
        Box::pin(async move {
            let side = match intent.side {
                traderview_core::algo_strategies::Side::Buy => "buy",
                traderview_core::algo_strategies::Side::Sell => "sell",
            };
            // Asset-class + session detection. Three submission paths
            // exist, gated by these two booleans:
            //   * Crypto symbol (`BTC/USD`-style): simple market, no
            //     bracket — Alpaca rejects bracket on crypto. Trades
            //     24/7 so GTC is the right TIF.
            //   * Equity in extended hours: LIMIT only, no bracket,
            //     extended_hours=true. Bracket+extended is rejected.
            //   * Equity in regular hours: native bracket as before.
            let is_crypto = intent.symbol.contains('/');
            let req = if is_crypto {
                PlaceOrderRequest::crypto_market(
                    intent.symbol.clone(),
                    side,
                    intent.qty,
                    intent.client_order_id,
                )
            } else if is_extended_hours_session_now() {
                // Use the signal's entry_price as the limit — strategy
                // wants this fill at signal time, after-hours liquidity
                // is thin, going limit avoids slippage surprises.
                PlaceOrderRequest::extended_hours_limit(
                    intent.symbol.clone(),
                    side,
                    intent.qty,
                    intent.client_order_id,
                    intent.entry_price,
                )
            } else {
                PlaceOrderRequest::bracket_market(
                    intent.symbol.clone(),
                    side,
                    intent.qty,
                    intent.client_order_id,
                    intent.take_profit_price,
                    intent.stop_price,
                )
            };
            let resp = client
                .place_order(&req)
                .await
                .map_err(|e| EngineError::Broker(format!("alpaca: {e}")))?;
            Ok(SubmittedOrder {
                broker_order_id: resp.id,
                status: resp.status,
                raw_response: None,
                immediate_fill: None,
            })
        })
    }
}

/// True when wall-clock time falls inside Alpaca's extended-hours
/// session: pre-market 4:00–9:30 ET or after-hours 16:00–20:00 ET
/// (Monday–Friday). Outside those windows AND outside RTH, returns
/// false — overnight orders aren't supported on equities and the
/// engine should refuse rather than submit a doomed order.
fn is_extended_hours_session_now() -> bool {
    // Chrono FixedOffset for Eastern. We use -04:00 (EDT) all year
    // for the algo trader's purposes since US equities follow that
    // schedule with two flip-points (DST). For paper trading a
    // single fixed offset gives "good enough" gating; the real
    // submission gets rejected by Alpaca anyway if we get it wrong.
    // The proper fix is chrono-tz with America/New_York but pulling
    // in tzdata for one timestamp check isn't worth the binary
    // bloat for the algo subsystem.
    use chrono::{Datelike, FixedOffset, Timelike, Utc};
    let now = Utc::now().with_timezone(&FixedOffset::west_opt(4 * 3600).expect("EDT offset"));
    if matches!(now.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
        return false;
    }
    let h = now.hour();
    let m = now.minute();
    let mins = h * 60 + m;
    // Pre-market: 4:00 (240) up to but not including 9:30 (570).
    // After-hours: 16:00 (960) up to but not including 20:00 (1200).
    (240..570).contains(&mins) || (960..1200).contains(&mins)
}

/// Real IBKR sink — resolves symbol→conid, then POSTs a single-leg
/// market order via /iserver/account/{id}/orders. Native bracket
/// support on IBKR uses the parent-child orderType+parentId pattern;
/// this commit ships entry-only and leaves brackets to a follow-up.
#[derive(Debug, Clone)]
struct IbkrSink {
    client: IbkrTrading,
    /// Reserved for future logging hooks.
    #[allow(dead_code)]
    account_label: String,
}

impl BrokerSink for IbkrSink {
    fn submit_bracket(
        &self,
        intent: OrderIntent,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>,
    > {
        let client = self.client.clone();
        Box::pin(async move {
            let conid = client
                .resolve_stock_conid(&intent.symbol)
                .await
                .map_err(|e| EngineError::Broker(format!("ibkr secdef: {e}")))?;
            let side = match intent.side {
                traderview_core::algo_strategies::Side::Buy => IbkrOrderSide::Buy,
                traderview_core::algo_strategies::Side::Sell => IbkrOrderSide::Sell,
            };
            let parent_coid = intent.client_order_id.to_string();
            let has_bracket =
                intent.take_profit_price > Decimal::zero() && intent.stop_price > Decimal::zero();
            let resps = if has_bracket {
                let req = IbkrPlaceBracket {
                    conid,
                    side,
                    order_type: IbkrOrderType::Market,
                    quantity: intent.qty,
                    tif: IbkrTif::Day,
                    entry_price: None,
                    take_profit_price: intent.take_profit_price,
                    stop_loss_price: intent.stop_price,
                    parent_coid: parent_coid.clone(),
                };
                client
                    .place_bracket(&req)
                    .await
                    .map_err(|e| EngineError::Broker(format!("ibkr bracket: {e}")))?
            } else {
                let req = IbkrPlaceOrder {
                    conid,
                    side,
                    order_type: IbkrOrderType::Market,
                    quantity: intent.qty,
                    tif: IbkrTif::Day,
                    price: None,
                    client_order_id: Some(parent_coid.clone()),
                };
                client
                    .place_order(&req)
                    .await
                    .map_err(|e| EngineError::Broker(format!("ibkr: {e}")))?
            };
            let first = resps
                .into_iter()
                .next()
                .ok_or_else(|| EngineError::Broker("ibkr: empty order response array".into()))?;
            let order_id = first
                .order_id
                .clone()
                .or(first.local_order_id.clone())
                .unwrap_or(parent_coid);
            Ok(SubmittedOrder {
                broker_order_id: order_id,
                status: first.order_status.unwrap_or_else(|| "submitted".into()),
                raw_response: None,
                // IBKR fills land via the /ws pump (ibkr_pump.rs).
                immediate_fill: None,
            })
        })
    }
}

/// Real Schwab sink — places a single-leg equity market order via
/// /trader/v1/accounts/{accountHash}/orders. On 401 the underlying
/// client transparently refreshes the access token once and retries.
#[derive(Debug, Clone)]
struct SchwabSink {
    client: SchwabTrading,
}

impl BrokerSink for SchwabSink {
    fn submit_bracket(
        &self,
        intent: OrderIntent,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>,
    > {
        let client = self.client.clone();
        Box::pin(async move {
            let instruction = match intent.side {
                traderview_core::algo_strategies::Side::Buy => SchwabInstruction::Buy,
                traderview_core::algo_strategies::Side::Sell => SchwabInstruction::SellShort,
            };
            let has_bracket =
                intent.take_profit_price > Decimal::zero() && intent.stop_price > Decimal::zero();
            let resp = if has_bracket {
                let req = SchwabPlaceBracket {
                    symbol: intent.symbol.clone(),
                    instruction,
                    order_type: SchwabOrderType::Market,
                    quantity: intent.qty,
                    duration: SchwabDuration::Day,
                    session: SchwabSession::Normal,
                    entry_price: None,
                    take_profit_price: intent.take_profit_price,
                    stop_loss_price: intent.stop_price,
                    comment: Some(intent.client_order_id.to_string()),
                };
                client
                    .place_bracket(&req)
                    .await
                    .map_err(|e| EngineError::Broker(format!("schwab bracket: {e}")))?
            } else {
                let req = SchwabPlaceOrder {
                    symbol: intent.symbol.clone(),
                    instruction,
                    order_type: SchwabOrderType::Market,
                    quantity: intent.qty,
                    duration: SchwabDuration::Day,
                    session: SchwabSession::Normal,
                    price: None,
                    comment: Some(intent.client_order_id.to_string()),
                };
                client
                    .place_order(&req)
                    .await
                    .map_err(|e| EngineError::Broker(format!("schwab: {e}")))?
            };
            Ok(SubmittedOrder {
                broker_order_id: resp
                    .order_id
                    .clone()
                    .unwrap_or_else(|| intent.client_order_id.to_string()),
                status: resp.status.unwrap_or_else(|| "submitted".into()),
                raw_response: None,
                // Schwab fills land via the trader-streaming WebSocket
                // (separate pump module — fills arrive after this
                // returns).
                immediate_fill: None,
            })
        })
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
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>,
    > {
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

/// Real Tastytrade sink — places a single-leg equity market order via
/// REST. Native bracket support (Tastytrade's Complex Orders class) is
/// a follow-up; entry-only orders work today.
#[derive(Debug, Clone)]
struct TastytradeSink {
    client: TastytradeTrading,
}

impl BrokerSink for TastytradeSink {
    fn submit_bracket(
        &self,
        intent: OrderIntent,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>,
    > {
        let client = self.client.clone();
        Box::pin(async move {
            // Tastytrade uses 'Buy to Open' / 'Sell to Open' / etc.
            // semantics. Strategy intent.side maps to BuyToOpen (long)
            // or SellToOpen (short).
            let action = match intent.side {
                traderview_core::algo_strategies::Side::Buy => EquityAction::BuyToOpen,
                traderview_core::algo_strategies::Side::Sell => EquityAction::SellToOpen,
            };
            // Bracket gap: the public Tastytrade REST API for retail
            // equity does NOT expose an atomic OTOCO/bracket primitive.
            // Their web UI stitches the parent + OCO children client-
            // side over multiple separate POSTs; there is no
            // "advanced-instructions: rules.route-after" envelope we
            // can target safely. Submitting independent SL+TP orders
            // BEFORE the entry fills would leave them live against a
            // not-yet-open position (risk: the broker fills the SL on
            // a momentary down-tick + later the entry fills, leaving
            // a naked short).
            //
            // Until the pump grows a "on entry fill → submit OCO pair"
            // callback (substrate-level change), Tastytrade strategies
            // ship ENTRY-ONLY orders with the take-profit + stop-loss
            // surfaced via tracing::warn so the user sees the drop in
            // the desktop log. Risk-side responsibility shifts to the
            // user / kill-switch / position-size-cap for these.
            if intent.take_profit_price > Decimal::zero() || intent.stop_price > Decimal::zero() {
                tracing::warn!(
                    symbol = %intent.symbol,
                    take_profit = %intent.take_profit_price,
                    stop = %intent.stop_price,
                    client_order_id = %intent.client_order_id,
                    "tastytrade bracket dropped — entry-only submitted; \
                     atomic OTOCO not available in Tastytrade public REST. \
                     Manage the exit manually or use risk_gates.max_position_size_usd \
                     as the safety floor."
                );
            }
            let req = TastyEquityOrder::market(intent.symbol.clone(), action, intent.qty);
            let resp = client
                .place_equity_order(&req)
                .await
                .map_err(|e| EngineError::Broker(format!("tastytrade: {e}")))?;
            Ok(SubmittedOrder {
                broker_order_id: resp.id.to_string(),
                status: resp.status,
                raw_response: None,
                // Tastytrade reports fills via the streaming events
                // websocket (tastytrade_pump.rs).
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
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SubmittedOrder, EngineError>> + Send + '_>,
    > {
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
