use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::routes::helpers::ensure_account_owner;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use traderview_core::models::{AssetClass, TradeSide};
use traderview_core::risk_gate::{evaluate, ProposedTrade, Severity};
use traderview_db::paper::{OrderRequest, PaperAccount, PaperOrder, PaperPosition};
use traderview_db::risk_rules;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/paper/accounts", get(list).post(ensure_default))
        .route("/paper/accounts/:id/reset", post(reset))
        .route("/paper/accounts/:id/orders", get(orders).post(submit))
        .route("/paper/accounts/:id/positions", get(positions))
        .route(
            "/paper/accounts/:id/parent-orders",
            post(create_parent_order),
        )
        .route("/paper/parent-orders", get(list_parent_orders))
        .route(
            "/paper/parent-orders/:id/cancel",
            post(cancel_parent_order),
        )
        .route("/paper/orders/:id/cancel", post(cancel_order))
        .route("/paper/orders/:id/replace", post(replace_order))
        .route("/paper/accounts/:id/brackets", post(submit_bracket))
        .route("/paper/accounts/:id/protect", post(protect))
        .route("/paper/accounts/:id/roll", post(roll))
        .route("/paper/accounts/:id/spreads", post(submit_spread))
        .route("/paper/accounts/:id/option-greeks", get(option_greeks))
        .route("/paper/spreads/preview", post(preview_spread))
        .route(
            "/paper/accounts/:id/recurring",
            post(create_recurring),
        )
        .route("/paper/recurring", get(list_recurring))
        .route("/paper/recurring/:id/toggle", post(toggle_recurring))
        .route("/paper/accounts/:id/drip", post(set_drip))
        .route("/paper/accounts/:id/cash-apy", post(set_cash_apy))
        .route("/paper/accounts/:id/borrow-apy", post(set_borrow_apy))
        .route("/paper/accounts/:id/interest", get(interest))
        .route("/paper/accounts/:id/statement", get(statement))
        .route(
            "/paper/recurring/:id",
            axum::routing::delete(delete_recurring),
        )
        .route("/paper/accounts/:id/equity-history", get(equity_history))
        .route("/paper/accounts/:id/attribution", get(attribution))
        .route("/paper/accounts/:id/wash-sales", get(wash_sales))
        .route("/paper/accounts/:id/correlations", get(correlations))
        .route("/paper/accounts/:id/var", get(portfolio_var))
        .route("/paper/accounts/:id/stress", get(stress))
        .route("/digest/prefs", get(get_digest_prefs).post(set_digest_prefs))
        .route("/paper/accounts/comparison", get(account_comparison))
        .route("/paper/accounts/create", post(create_account))
        .route("/paper/accounts/:id/rename", post(rename_account))
        .route("/paper/accounts/:id/delete", post(delete_account))
        .route("/paper/accounts/:id/dividends", get(dividends))
        .route("/paper/accounts/:id/splits", get(splits))
}

/// Split adjustments applied to the account by the background pass —
/// qty × ratio, avg ÷ ratio, value-preserving.
async fn splits(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Vec<traderview_db::paper_splits::PaperSplit>>, ApiError> {
    traderview_db::paper_splits::list(&s.pool, user.id, account_id, 200)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

/// Dividend cash credited to the account by the background pass —
/// longs held through an ex-date are credited, shorts debited.
async fn dividends(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Vec<traderview_db::paper_dividends::PaperDividend>>, ApiError> {
    traderview_db::paper_dividends::list(&s.pool, user.id, account_id, 500)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

/// Strategy leaderboard across the user's paper accounts.
async fn account_comparison(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<traderview_db::paper_equity::AccountComparison>>, ApiError> {
    traderview_db::paper_equity::compare(&s.pool, user.id)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

#[derive(Deserialize)]
struct CreateAccountBody {
    name: String,
    starting_cash: Decimal,
}

/// Named paper account — one per strategy is the intended use.
async fn create_account(
    State(s): State<AppState>,
    user: AuthUser,
    Json(b): Json<CreateAccountBody>,
) -> Result<Json<PaperAccount>, ApiError> {
    traderview_db::paper::create_account(&s.pool, user.id, &b.name, b.starting_cash)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

#[derive(Deserialize)]
struct RenameBody {
    name: String,
}

async fn rename_account(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<RenameBody>,
) -> Result<Json<bool>, ApiError> {
    traderview_db::paper::rename_account(&s.pool, user.id, id, &b.name)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

async fn delete_account(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    traderview_db::paper::delete_account(&s.pool, user.id, id)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

#[derive(Deserialize)]
struct CorrelationsQ {
    #[serde(default = "default_corr_lookback")]
    lookback_days: i64,
}
fn default_corr_lookback() -> i64 {
    90
}

/// Pairwise correlations of current holdings — the diversification lens.
async fn correlations(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
    Query(q): Query<CorrelationsQ>,
) -> Result<Json<traderview_db::paper_equity::PositionCorrelations>, ApiError> {
    traderview_db::paper_equity::position_correlations(&s.pool, user.id, account_id, q.lookback_days)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

#[derive(Deserialize)]
struct VarQ {
    #[serde(default = "default_var_lookback")]
    lookback_days: i64,
}
fn default_var_lookback() -> i64 {
    365
}

/// Historical-simulation VaR/ES of the current equity book.
async fn portfolio_var(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
    Query(q): Query<VarQ>,
) -> Result<Json<traderview_db::paper_equity::PortfolioVar>, ApiError> {
    traderview_db::paper_equity::portfolio_var(&s.pool, user.id, account_id, q.lookback_days)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

#[derive(Deserialize)]
struct StressQ {
    #[serde(default = "default_var_lookback")]
    lookback_days: i64,
    #[serde(default = "default_stress_benchmark")]
    benchmark: String,
}
fn default_stress_benchmark() -> String {
    "SPY".into()
}

/// Worst observed windows + beta-scaled benchmark shocks.
async fn stress(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
    Query(q): Query<StressQ>,
) -> Result<Json<traderview_db::paper_equity::StressReport>, ApiError> {
    traderview_db::paper_equity::stress(
        &s.pool,
        user.id,
        account_id,
        q.lookback_days,
        &q.benchmark.trim().to_uppercase(),
    )
    .await
    .map(Json)
    .map_err(|e| ApiError::BadRequest(e.to_string()))
}

#[derive(Serialize)]
struct DigestPrefs {
    hour_utc: u32,
}

async fn get_digest_prefs(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<DigestPrefs>, ApiError> {
    traderview_db::digest::get_hour(&s.pool, user.id)
        .await
        .map(|hour_utc| Json(DigestPrefs { hour_utc }))
        .map_err(ApiError::Internal)
}

#[derive(Deserialize)]
struct DigestPrefsBody {
    hour_utc: u32,
}

async fn set_digest_prefs(
    State(s): State<AppState>,
    user: AuthUser,
    Json(b): Json<DigestPrefsBody>,
) -> Result<Json<bool>, ApiError> {
    traderview_db::digest::set_hour(&s.pool, user.id, b.hour_utc)
        .await
        .map(|_| Json(true))
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

/// Per-symbol realized P&L decomposition: closed trips + dividends − fees.
async fn attribution(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<traderview_db::paper_equity::Attribution>, ApiError> {
    traderview_db::paper_equity::attribution(&s.pool, user.id, account_id)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

async fn wash_sales(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<Vec<traderview_db::paper_equity::SymbolWashSales>>, ApiError> {
    traderview_db::paper_equity::wash_sales(&s.pool, user.id, account_id)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

#[derive(Deserialize)]
struct EquityHistoryQ {
    #[serde(default = "default_benchmark")]
    benchmark: String,
}
fn default_benchmark() -> String {
    "SPY".into()
}

/// Background-sampled equity curve with return/drawdown summary and a
/// benchmark overlay normalized to the first snapshot's equity.
async fn equity_history(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
    Query(q): Query<EquityHistoryQ>,
) -> Result<Json<traderview_db::paper_equity::EquityHistory>, ApiError> {
    traderview_db::paper_equity::history(
        &s.pool,
        user.id,
        account_id,
        2000,
        &q.benchmark.trim().to_uppercase(),
    )
    .await
    .map(Json)
    .map_err(ApiError::Internal)
}

/// BS greeks for every OCC position, qty-and-multiplier scaled, with
/// account-level nets.
async fn option_greeks(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<traderview_db::paper::AccountOptionGreeks>, ApiError> {
    traderview_db::paper::option_greeks(&s.pool, user.id, account_id)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

/// Stateless spread quote: per-leg mids + expiry payoff profile,
/// priced by the same chain source the submit fills against.
async fn preview_spread(
    State(s): State<AppState>,
    _user: AuthUser,
    Json(req): Json<traderview_db::paper::SpreadRequest>,
) -> Result<Json<traderview_db::paper::SpreadPreview>, ApiError> {
    traderview_db::paper::preview_spread(&s.pool, &req)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

#[derive(Deserialize)]
struct DripBody {
    enabled: bool,
}

/// Toggle dividend reinvestment for the account.
async fn set_drip(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<DripBody>,
) -> Result<Json<bool>, ApiError> {
    traderview_db::paper::set_drip(&s.pool, user.id, id, b.enabled)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

#[derive(Deserialize)]
struct RecurringBody {
    #[serde(default)]
    symbol: Option<String>,
    /// Cash-flow rebalancing: buy the target's most underweight
    /// holding each run instead of a fixed symbol.
    #[serde(default)]
    target_id: Option<Uuid>,
    notional_usd: Decimal,
    cadence: String,
}

/// Auto-invest: "$N of SYMBOL" or "$N into TARGET" daily/weekly/monthly.
async fn create_recurring(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(b): Json<RecurringBody>,
) -> Result<Json<traderview_db::paper_recurring::RecurringOrder>, ApiError> {
    traderview_db::paper_recurring::create(
        &s.pool, user.id, account_id, b.symbol.as_deref(), b.target_id, b.notional_usd, &b.cadence,
    )
    .await
    .map(Json)
    .map_err(|e| ApiError::BadRequest(e.to_string()))
}

async fn list_recurring(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<traderview_db::paper_recurring::RecurringOrder>>, ApiError> {
    traderview_db::paper_recurring::list(&s.pool, user.id)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

#[derive(Deserialize)]
struct ToggleBody {
    enabled: bool,
}

async fn toggle_recurring(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<ToggleBody>,
) -> Result<Json<bool>, ApiError> {
    traderview_db::paper_recurring::set_enabled(&s.pool, user.id, id, b.enabled)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

async fn delete_recurring(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    traderview_db::paper_recurring::delete(&s.pool, user.id, id)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

/// Atomic multi-leg option spread (2-4 OCC legs, one underlying).
async fn submit_spread(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(req): Json<traderview_db::paper::SpreadRequest>,
) -> Result<Json<traderview_db::paper::SpreadResult>, ApiError> {
    traderview_db::paper::submit_spread(&s.pool, user.id, account_id, req)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

/// Bracket / OCO: entry + linked stop-loss and take-profit legs; the
/// legs activate when the entry fills and the first to fill cancels
/// the other.
async fn submit_bracket(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(req): Json<traderview_db::paper::BracketRequest>,
) -> Result<Json<traderview_db::paper::Bracket>, ApiError> {
    run_risk_gate(
        &s,
        user.id,
        account_id,
        &req.symbol,
        req.side,
        req.qty,
        // Limit entry price when known; market entries degrade to zero.
        req.limit_price.unwrap_or(Decimal::ZERO),
        // Trailing brackets have no fixed stop for the gate to see.
        req.stop_loss,
        // A bracket IS an attached plan: stop and target up front.
        true,
    )
    .await?;
    traderview_db::paper::submit_bracket(&s.pool, user.id, account_id, req)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

#[derive(serde::Deserialize)]
struct ProtectBody {
    symbol: String,
    qty: rust_decimal::Decimal,
    stop_loss: rust_decimal::Decimal,
    take_profit: rust_decimal::Decimal,
}

async fn protect(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(b): Json<ProtectBody>,
) -> Result<Json<traderview_db::paper::Protection>, ApiError> {
    traderview_db::paper::attach_protection(
        &s.pool, user.id, account_id, &b.symbol, b.qty, b.stop_loss, b.take_profit,
    )
    .await
    .map(Json)
    .map_err(|e| ApiError::BadRequest(e.to_string()))
}

/// Cancel a RESTING (pending) limit/stop order.
async fn cancel_order(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    traderview_db::paper::cancel_order(&s.pool, user.id, id)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

/// Cancel/replace: amend a resting order's qty/prices in place.
async fn replace_order(
    State(s): State<AppState>,
    user: AuthUser,
    Path(order_id): Path<Uuid>,
    Json(b): Json<traderview_db::paper::ReplaceRequest>,
) -> Result<Json<PaperOrder>, ApiError> {
    traderview_db::paper::replace_order(&s.pool, user.id, order_id, b)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

/// TWAP/VWAP parent order: child market slices submitted by the
/// background ticker through the same fill path as manual paper orders.
async fn create_parent_order(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(inp): Json<traderview_db::paper_parent_orders::ParentOrderInput>,
) -> Result<Json<traderview_db::paper_parent_orders::ParentOrder>, ApiError> {
    // Gate the FULL parent quantity up front — gating per child slice
    // would let a sized-up parent sneak past max-position rules one
    // slice at a time.
    run_risk_gate(
        &s,
        user.id,
        account_id,
        &inp.symbol,
        inp.side,
        inp.total_qty,
        Decimal::ZERO, // market children — % price rules degrade gracefully
        None,
        false,
    )
    .await?;
    traderview_db::paper_parent_orders::create(&s.pool, user.id, account_id, &inp)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

async fn list_parent_orders(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<traderview_db::paper_parent_orders::ParentOrder>>, ApiError> {
    traderview_db::paper_parent_orders::list(&s.pool, user.id)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

async fn cancel_parent_order(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    traderview_db::paper_parent_orders::cancel(&s.pool, user.id, id)
        .await
        .map(Json)
        .map_err(ApiError::Internal)
}

async fn list(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<PaperAccount>>, ApiError> {
    Ok(Json(
        traderview_db::paper::list_accounts(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn ensure_default(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<PaperAccount>, ApiError> {
    Ok(Json(
        traderview_db::paper::ensure_default(&s.pool, user.id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct ResetBody {
    starting_cash: Decimal,
}

async fn reset(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<ResetBody>,
) -> Result<Json<bool>, ApiError> {
    Ok(Json(
        traderview_db::paper::reset(&s.pool, user.id, id, b.starting_cash)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct OrdersQ {
    #[serde(default = "default_limit")]
    limit: i64,
}
fn default_limit() -> i64 {
    100
}

async fn orders(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<OrdersQ>,
) -> Result<Json<Vec<PaperOrder>>, ApiError> {
    ensure_account_owner(&s, user.id, id).await?;
    Ok(Json(
        traderview_db::paper::list_orders(&s.pool, id, q.limit)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

/// Pre-trade Risk Gate shared by EVERY paper entry point — manual
/// orders, bracket entries, and parent-order creation. Same rules the
/// live new-trade form enforces. Paper trading is the place to BUILD
/// the habit; any ungated path is a place to practice rule-breaking.
/// The paper account id is the context source so paper-specific
/// equity / today's P&L drive the % checks.
#[allow(clippy::too_many_arguments)]
async fn run_risk_gate(
    s: &AppState,
    user_id: Uuid,
    account_id: Uuid,
    symbol: &str,
    side: traderview_core::Side,
    qty: Decimal,
    entry_price: Decimal,
    stop_loss: Option<Decimal>,
    has_attached_plan: bool,
) -> Result<(), ApiError> {
    let proposed = ProposedTrade {
        symbol: symbol.to_string(),
        // Side mapping: paper Side (buy/sell/short/cover) → TradeSide.
        // Same mapping as new_trade.js.
        side: match side {
            traderview_core::Side::Buy | traderview_core::Side::Sell => TradeSide::Long,
            traderview_core::Side::Short | traderview_core::Side::Cover => TradeSide::Short,
        },
        qty,
        entry_price,
        stop_loss,
        asset_class: if traderview_core::occ_symbol::is_occ(symbol) {
            AssetClass::Option
        } else {
            AssetClass::Stock
        },
        multiplier: if traderview_core::occ_symbol::is_occ(symbol) {
            Decimal::from(100)
        } else {
            Decimal::ONE
        },
        tick_size: None,
        tick_value: None,
        has_attached_plan,
    };
    let rows = risk_rules::list(&s.pool, user_id, Some(account_id))
        .await
        .map_err(ApiError::Internal)?;
    let rules: Vec<_> = rows
        .into_iter()
        .filter(|r| r.enabled)
        .map(|r| r.rule)
        .collect();
    if !rules.is_empty() {
        let ctx = risk_rules::build_context(&s.pool, account_id)
            .await
            .map_err(ApiError::Internal)?;
        let decision = evaluate(&proposed, &ctx, &rules, Utc::now());
        if !decision.allow {
            let msg = decision
                .violations
                .iter()
                .filter(|v| v.severity == Severity::Block)
                .map(|v| format!("[{}] {}", v.rule, v.message))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(ApiError::BadRequest(format!("Risk Gate blocked: {msg}")));
        }
    }
    Ok(())
}

async fn submit(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<OrderRequest>,
) -> Result<Json<PaperOrder>, ApiError> {
    run_risk_gate(
        &s,
        user.id,
        id,
        &req.symbol,
        req.side,
        req.qty,
        // Best-effort entry price for the gate — limit price if set, else
        // stop, else zero (the % rules would just degrade gracefully).
        req.limit_price.or(req.stop_price).unwrap_or(Decimal::ZERO),
        req.stop_price,
        // A written plan note satisfies RequirePlanBeforeTrade — the
        // rule was unsatisfiable at the manual ticket before this.
        req.plan_note
            .as_deref()
            .map(str::trim)
            .is_some_and(|s| !s.is_empty()),
    )
    .await?;
    Ok(Json(
        traderview_db::paper::submit(&s.pool, user.id, id, req)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

async fn positions(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<PaperPosition>>, ApiError> {
    ensure_account_owner(&s, user.id, id).await?;
    Ok(Json(
        traderview_db::paper::positions(&s.pool, id)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct RollBody {
    from: String,
    to: String,
    qty: Decimal,
}

/// Roll an option position atomically (close old + open new through
/// the spread path — both legs quote before the book is touched).
async fn roll(
    State(s): State<AppState>,
    user: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(b): Json<RollBody>,
) -> Result<Json<traderview_db::paper::SpreadResult>, ApiError> {
    traderview_db::paper::roll_position(&s.pool, user.id, account_id, &b.from, &b.to, b.qty)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

#[derive(Deserialize)]
struct CashApyBody {
    apy_pct: Decimal,
}

/// Cash sweep APY (0 disables the daily interest credit).
async fn set_cash_apy(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<CashApyBody>,
) -> Result<Json<bool>, ApiError> {
    traderview_db::paper_interest::set_cash_apy(&s.pool, user.id, id, b.apy_pct)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

/// Interest credits posted by the background sweep pass.
async fn interest(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<traderview_db::paper_interest::InterestCredit>>, ApiError> {
    traderview_db::paper_interest::list(&s.pool, user.id, id, 200)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

/// Short borrow APY (0 disables the daily debit on equity shorts).
async fn set_borrow_apy(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(b): Json<CashApyBody>,
) -> Result<Json<bool>, ApiError> {
    traderview_db::paper_interest::set_borrow_apy(&s.pool, user.id, id, b.apy_pct)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}

#[derive(Deserialize)]
struct StatementQ {
    month: String,
}

/// Monthly brokerage-style statement composed from the existing stores.
async fn statement(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<StatementQ>,
) -> Result<Json<traderview_db::paper_equity::Statement>, ApiError> {
    traderview_db::paper_equity::statement(&s.pool, user.id, id, &q.month)
        .await
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.to_string()))
}
