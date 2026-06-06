//! HTTP surface for the comprehensive Finnhub REST client.
//!
//! Three groupings:
//!   * `/symbols/:symbol/*`   — per-symbol endpoints
//!   * `/finnhub/calendar/*`  — calendar endpoints
//!   * `/finnhub/*`           — broad endpoints (forex, crypto, indices, ETF, …)
//!
//! All routes return Finnhub-shaped JSON verbatim. Plan-restricted
//! endpoints surface as **HTTP 403 Forbidden** (mapped from Finnhub's
//! own 403 via `map_fh_err` + `fh::is_premium_required`), so the
//! frontend can render a "premium required" affordance instead of a
//! generic server-error toast. All other failures still surface as
//! 500 with the underlying cause attached.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;
use traderview_db::finnhub_rest as fh;

/// Map a Finnhub-call error to the right `ApiError`. Plan-restricted
/// endpoints get 403 Forbidden; everything else falls through as a
/// generic 500 with the cause attached.
fn map_fh_err(e: anyhow::Error) -> ApiError {
    if fh::is_premium_required(&e) {
        ApiError::Forbidden
    } else {
        ApiError::Internal(e)
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        // ── per-symbol fundamentals / ownership / events ─────────────
        .route("/symbols/:symbol/profile", get(profile))
        .route("/symbols/:symbol/profile-legacy", get(profile_legacy))
        .route("/symbols/:symbol/executives", get(executives))
        .route("/symbols/:symbol/peers", get(peers))
        .route("/symbols/:symbol/upgrades", get(upgrades))
        .route(
            "/symbols/:symbol/financials-reported",
            get(financials_reported),
        )
        .route("/symbols/:symbol/financials", get(financials))
        .route("/symbols/:symbol/metric", get(metric))
        .route("/symbols/:symbol/finnhub-quote", get(finnhub_quote))
        .route("/symbols/:symbol/finnhub-news", get(per_symbol_news))
        .route("/symbols/:symbol/news-sentiment", get(news_sentiment))
        .route("/symbols/:symbol/press-releases", get(press_releases))
        .route("/symbols/:symbol/eps-surprise", get(eps_surprise))
        .route("/symbols/:symbol/revenue-estimate", get(revenue_estimate))
        .route("/symbols/:symbol/ebitda-estimate", get(ebitda_estimate))
        .route("/symbols/:symbol/ebit-estimate", get(ebit_estimate))
        .route("/symbols/:symbol/eps-estimate", get(eps_estimate))
        .route(
            "/symbols/:symbol/net-income-estimate",
            get(net_income_estimate),
        )
        .route(
            "/symbols/:symbol/pretax-income-estimate",
            get(pretax_income_estimate),
        )
        .route(
            "/symbols/:symbol/gross-income-estimate",
            get(gross_income_estimate),
        )
        .route("/symbols/:symbol/dps-estimate", get(dps_estimate))
        .route("/symbols/:symbol/price-target", get(price_target))
        .route("/symbols/:symbol/option-chain", get(option_chain))
        .route("/symbols/:symbol/fund-ownership", get(fund_ownership))
        .route("/symbols/:symbol/ownership", get(ownership))
        .route("/symbols/:symbol/company-earnings", get(company_earnings))
        .route("/symbols/:symbol/finnhub-dividends", get(stock_dividends))
        .route(
            "/symbols/:symbol/dividends-basic",
            get(stock_basic_dividends),
        )
        .route("/symbols/:symbol/splits", get(stock_splits))
        .route("/symbols/:symbol/finnhub-candles", get(stock_candles))
        .route("/symbols/:symbol/tick", get(stock_tick))
        .route("/symbols/:symbol/nbbo", get(stock_nbbo))
        .route("/symbols/:symbol/bidask", get(last_bid_ask))
        .route("/symbols/:symbol/filings", get(filings))
        .route("/symbols/:symbol/transcripts-list", get(transcripts_list))
        .route("/symbols/:symbol/similarity-index", get(similarity_index))
        // ── per-symbol alt / ESG / regulatory ────────────────────────
        .route(
            "/symbols/:symbol/finnhub-insiders",
            get(insider_transactions),
        )
        .route("/symbols/:symbol/insider-sentiment", get(insider_sentiment))
        .route("/symbols/:symbol/lobbying", get(lobbying))
        .route("/symbols/:symbol/usa-spending", get(usa_spending))
        .route("/symbols/:symbol/visa-application", get(visa_application))
        .route("/symbols/:symbol/uspto-patent", get(uspto_patent))
        .route("/symbols/:symbol/supply-chain", get(supply_chain))
        .route("/symbols/:symbol/social-sentiment", get(social_sentiment))
        .route("/symbols/:symbol/esg", get(esg_score))
        .route("/symbols/:symbol/esg-historical", get(esg_historical))
        .route(
            "/symbols/:symbol/historical-market-cap",
            get(historical_market_cap),
        )
        .route(
            "/symbols/:symbol/historical-employee-count",
            get(historical_employee_count),
        )
        .route(
            "/symbols/:symbol/earnings-quality-score",
            get(earnings_quality_score),
        )
        .route("/symbols/:symbol/revenue-breakdown", get(revenue_breakdown))
        .route(
            "/symbols/:symbol/revenue-breakdown2",
            get(revenue_breakdown2),
        )
        .route("/symbols/:symbol/presentation", get(presentation))
        .route("/symbols/:symbol/newsroom", get(newsroom))
        .route(
            "/symbols/:symbol/congressional-trading",
            get(congressional_trading),
        )
        .route("/symbols/:symbol/price-metric", get(price_metric))
        .route("/symbols/:symbol/bank-branch", get(bank_branch))
        // ── per-symbol scan ─────────────────────────────────────────
        .route("/symbols/:symbol/scan/pattern", get(scan_pattern))
        .route("/symbols/:symbol/scan/sr", get(scan_support_resistance))
        .route("/symbols/:symbol/scan/aggregate", get(scan_aggregate))
        .route("/symbols/:symbol/indicator", get(indicator))
        // ── broad calendars ─────────────────────────────────────────
        .route("/finnhub/calendar/earnings", get(earnings_calendar))
        .route("/finnhub/calendar/ipo", get(ipo_calendar))
        .route("/finnhub/calendar/economic", get(economic_calendar))
        .route("/finnhub/calendar/fda", get(fda_calendar))
        .route(
            "/finnhub/calendar/earnings-call-live",
            get(earnings_call_live),
        )
        // ── broad news ──────────────────────────────────────────────
        .route("/finnhub/news", get(general_news))
        // ── forex / crypto ──────────────────────────────────────────
        .route("/finnhub/forex/exchanges", get(forex_exchanges))
        .route("/finnhub/forex/symbols", get(forex_symbols))
        .route("/finnhub/forex/rates", get(forex_rates))
        .route("/finnhub/forex/candle", get(forex_candle))
        .route("/finnhub/crypto/exchanges", get(crypto_exchanges))
        .route("/finnhub/crypto/symbols", get(crypto_symbols))
        .route("/finnhub/crypto/candle", get(crypto_candle))
        .route("/finnhub/crypto/profile", get(crypto_profile))
        // ── indices / ETF / mutual fund / bond ──────────────────────
        .route(
            "/finnhub/index/:symbol/constituents",
            get(index_constituents),
        )
        .route(
            "/finnhub/index/:symbol/historical-constituents",
            get(index_historical),
        )
        .route("/finnhub/etf/:symbol/profile", get(etf_profile))
        .route("/finnhub/etf/:symbol/holdings", get(etf_holdings))
        .route("/finnhub/etf/:symbol/sector", get(etf_sector))
        .route("/finnhub/etf/:symbol/country", get(etf_country))
        .route("/finnhub/etf/:symbol/allocation", get(etf_allocation))
        .route("/finnhub/mutual-fund/:symbol/profile", get(mf_profile))
        .route("/finnhub/mutual-fund/:symbol/holdings", get(mf_holdings))
        .route("/finnhub/mutual-fund/:symbol/sector", get(mf_sector))
        .route("/finnhub/mutual-fund/:symbol/country", get(mf_country))
        .route("/finnhub/mutual-fund/eet/:isin", get(mf_eet))
        .route("/finnhub/bond/:isin/profile", get(bond_profile))
        .route("/finnhub/bond/:isin/price", get(bond_price))
        .route("/finnhub/bond/yield-curve", get(bond_yield_curve))
        // ── economic / market / institutional ───────────────────────
        .route("/finnhub/economic/codes", get(economic_codes))
        .route("/finnhub/economic/data", get(economic_data))
        .route("/finnhub/country-list", get(country_list))
        .route("/finnhub/market/status", get(market_status))
        .route("/finnhub/market/holiday", get(market_holiday))
        .route("/finnhub/stock-exchanges", get(stock_exchanges))
        .route("/finnhub/sector-metrics", get(sector_metrics))
        .route(
            "/finnhub/institutional/:cik/profile",
            get(institutional_profile),
        )
        .route(
            "/finnhub/institutional/:cik/portfolio",
            get(institutional_portfolio),
        )
        .route(
            "/finnhub/institutional/:symbol/ownership",
            get(institutional_ownership),
        )
        // ── meta / discovery ────────────────────────────────────────
        .route("/finnhub/search", get(symbol_lookup))
        .route("/finnhub/stock-symbols", get(stock_symbol_list))
        .route("/finnhub/symbol-change", get(symbol_change))
        .route("/finnhub/isin-change", get(isin_change))
        // ── specialty ───────────────────────────────────────────────
        .route("/finnhub/covid19", get(covid19))
        .route("/finnhub/investment-theme", get(investment_theme))
        .route("/finnhub/airline-price-index", get(airline_price_index))
}

// ──────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FromTo {
    from: Option<String>,
    to: Option<String>,
}

fn from_to_default(q: FromTo, window_days: i64) -> (String, String) {
    let today = chrono::Utc::now().date_naive();
    let from = q
        .from
        .unwrap_or_else(|| (today - chrono::Duration::days(window_days)).to_string());
    let to = q.to.unwrap_or_else(|| today.to_string());
    (from, to)
}

#[derive(Deserialize)]
struct DaysQuery {
    days: Option<i64>,
}
#[derive(Deserialize)]
struct LimitQuery {
    limit: Option<i64>,
}
#[derive(Deserialize)]
struct FreqQuery {
    freq: Option<String>,
}
#[derive(Deserialize)]
struct ResolutionQuery {
    resolution: Option<String>,
}
#[derive(Deserialize)]
struct DateQuery {
    date: Option<String>,
}
#[derive(Deserialize)]
struct CalEarningsQuery {
    from: Option<String>,
    to: Option<String>,
    symbol: Option<String>,
}
#[derive(Deserialize)]
struct CategoryQuery {
    category: Option<String>,
}
#[derive(Deserialize)]
struct BaseQuery {
    base: Option<String>,
}
#[derive(Deserialize)]
struct SymbolCandleQuery {
    symbol: String,
    resolution: Option<String>,
    from: Option<i64>,
    to: Option<i64>,
}
#[derive(Deserialize)]
struct CodeQuery {
    code: String,
}
#[derive(Deserialize)]
struct QQuery {
    q: String,
}
#[derive(Deserialize)]
struct ThemeQuery {
    theme: String,
}
#[derive(Deserialize)]
struct FormQuery {
    from: Option<String>,
    to: Option<String>,
    form: Option<String>,
}
#[derive(Deserialize)]
struct ExchangeOnly {
    #[serde(default)]
    exchange: String,
}
#[derive(Deserialize)]
struct AirlineQuery {
    airline: String,
    from: Option<String>,
    to: Option<String>,
}
#[derive(Deserialize)]
struct IndicatorQuery {
    resolution: Option<String>,
    from: Option<i64>,
    to: Option<i64>,
    indicator: Option<String>,
}
#[derive(Deserialize)]
struct StockCandleQuery {
    resolution: Option<String>,
    from: Option<i64>,
    to: Option<i64>,
}
#[derive(Deserialize)]
struct TickQuery {
    date: String,
    limit: Option<i64>,
    skip: Option<i64>,
}
#[derive(Deserialize)]
struct HoldingsSkip {
    skip: Option<i64>,
}

fn today_str() -> String {
    chrono::Utc::now().date_naive().to_string()
}
fn ago_str(days: i64) -> String {
    (chrono::Utc::now().date_naive() - chrono::Duration::days(days)).to_string()
}

// ──────────────────────────────────────────────────────────────────────
// Per-symbol fundamentals / ownership / events
// ──────────────────────────────────────────────────────────────────────

async fn profile(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::profile2(&s).await.map_err(map_fh_err)?))
}
async fn profile_legacy(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::company_profile_legacy(&s).await.map_err(map_fh_err)?,
    ))
}
async fn executives(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::company_executive(&s).await.map_err(map_fh_err)?))
}
async fn peers(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::peers(&s).await.map_err(map_fh_err)?))
}
async fn upgrades(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::upgrade_downgrade(&s).await.map_err(map_fh_err)?))
}
async fn financials_reported(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::financials_reported(&s).await.map_err(map_fh_err)?))
}

#[derive(Deserialize)]
struct FinancialsQuery {
    statement: Option<String>,
    freq: Option<String>,
}
async fn financials(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FinancialsQuery>,
) -> Result<Json<Value>, ApiError> {
    let stmt = q.statement.unwrap_or_else(|| "ic".into());
    let freq = q.freq.unwrap_or_else(|| "annual".into());
    Ok(Json(
        fh::financials(&s, &stmt, &freq).await.map_err(map_fh_err)?,
    ))
}
async fn metric(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::metric_all(&s).await.map_err(map_fh_err)?))
}
async fn finnhub_quote(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::quote(&s).await.map_err(map_fh_err)?))
}
async fn per_symbol_news(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<DaysQuery>,
) -> Result<Json<Value>, ApiError> {
    let days = q.days.unwrap_or(7).clamp(1, 365);
    let to = today_str();
    let from = ago_str(days);
    Ok(Json(
        fh::company_news(&s, &from, &to).await.map_err(map_fh_err)?,
    ))
}
async fn news_sentiment(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::news_sentiment(&s).await.map_err(map_fh_err)?))
}
async fn press_releases(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 30);
    Ok(Json(
        fh::press_releases(&s, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn eps_surprise(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::eps_surprise(&s).await.map_err(map_fh_err)?))
}

fn freq_or(q: FreqQuery) -> String {
    q.freq.unwrap_or_else(|| "annual".into())
}

async fn revenue_estimate(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FreqQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::revenue_estimate(&s, &freq_or(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn ebitda_estimate(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FreqQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::ebitda_estimate(&s, &freq_or(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn ebit_estimate(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FreqQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::ebit_estimate(&s, &freq_or(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn eps_estimate(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FreqQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::eps_estimate(&s, &freq_or(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn net_income_estimate(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FreqQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::net_income_estimate(&s, &freq_or(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn pretax_income_estimate(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FreqQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::pretax_income_estimate(&s, &freq_or(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn gross_income_estimate(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FreqQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::gross_income_estimate(&s, &freq_or(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn dps_estimate(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FreqQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::dps_estimate(&s, &freq_or(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn price_target(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::price_target(&s).await.map_err(map_fh_err)?))
}
async fn option_chain(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::option_chain(&s).await.map_err(map_fh_err)?))
}
async fn fund_ownership(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::fund_ownership(&s, q.limit.unwrap_or(20))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn ownership(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::ownership(&s, q.limit.unwrap_or(20))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn company_earnings(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::company_earnings(&s, q.limit.unwrap_or(20))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn stock_dividends(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365);
    Ok(Json(
        fh::stock_dividends(&s, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn stock_basic_dividends(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::stock_basic_dividends(&s).await.map_err(map_fh_err)?,
    ))
}
async fn stock_splits(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365 * 5);
    Ok(Json(
        fh::stock_splits(&s, &from, &to).await.map_err(map_fh_err)?,
    ))
}
async fn stock_candles(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<StockCandleQuery>,
) -> Result<Json<Value>, ApiError> {
    let res = q.resolution.unwrap_or_else(|| "D".into());
    let from = q
        .from
        .unwrap_or_else(|| chrono::Utc::now().timestamp() - 86_400 * 30)
        .to_string();
    let to =
        q.to.unwrap_or_else(|| chrono::Utc::now().timestamp())
            .to_string();
    Ok(Json(
        fh::stock_candles(&s, &res, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn stock_tick(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<TickQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::stock_tick(&s, &q.date, q.limit.unwrap_or(500), q.skip.unwrap_or(0))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn stock_nbbo(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<TickQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::stock_nbbo(&s, &q.date, q.limit.unwrap_or(500), q.skip.unwrap_or(0))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn last_bid_ask(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::last_bid_ask(&s).await.map_err(map_fh_err)?))
}
async fn filings(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FormQuery>,
) -> Result<Json<Value>, ApiError> {
    let from = q.from.unwrap_or_else(|| ago_str(365));
    let to = q.to.unwrap_or_else(today_str);
    Ok(Json(
        fh::filings(&s, &from, &to, q.form.as_deref())
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn transcripts_list(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::transcripts_list(&s).await.map_err(map_fh_err)?))
}
async fn similarity_index(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FreqQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::similarity_index(&s, &freq_or(q))
            .await
            .map_err(map_fh_err)?,
    ))
}

// ──────────────────────────────────────────────────────────────────────
// Per-symbol alt / ESG / regulatory
// ──────────────────────────────────────────────────────────────────────

async fn insider_transactions(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 180);
    Ok(Json(
        fh::insider_transactions(&s, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn insider_sentiment(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365);
    Ok(Json(
        fh::insider_sentiment(&s, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn lobbying(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365);
    Ok(Json(
        fh::lobbying(&s, &from, &to).await.map_err(map_fh_err)?,
    ))
}
async fn usa_spending(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365);
    Ok(Json(
        fh::usa_spending(&s, &from, &to).await.map_err(map_fh_err)?,
    ))
}
async fn visa_application(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365);
    Ok(Json(
        fh::visa_application(&s, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn uspto_patent(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365);
    Ok(Json(
        fh::uspto_patent(&s, &from, &to).await.map_err(map_fh_err)?,
    ))
}
async fn supply_chain(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::supply_chain(&s).await.map_err(map_fh_err)?))
}
async fn social_sentiment(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 30);
    Ok(Json(
        fh::social_sentiment(&s, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn esg_score(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::esg_score(&s).await.map_err(map_fh_err)?))
}
async fn esg_historical(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::historical_esg(&s).await.map_err(map_fh_err)?))
}
async fn historical_market_cap(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365);
    Ok(Json(
        fh::historical_market_cap(&s, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn historical_employee_count(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365 * 5);
    Ok(Json(
        fh::historical_employee_count(&s, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn earnings_quality_score(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FreqQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::earnings_quality_score(&s, &freq_or(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn revenue_breakdown(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::revenue_breakdown(&s).await.map_err(map_fh_err)?))
}
async fn revenue_breakdown2(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::revenue_breakdown2(&s).await.map_err(map_fh_err)?))
}
async fn presentation(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::presentation(&s).await.map_err(map_fh_err)?))
}
async fn newsroom(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 30);
    Ok(Json(
        fh::newsroom(&s, &from, &to).await.map_err(map_fh_err)?,
    ))
}
async fn congressional_trading(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365);
    Ok(Json(
        fh::congressional_trading(&s, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn price_metric(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<DateQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::price_metric(&s, q.date.as_deref())
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn bank_branch(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::bank_branch(&s).await.map_err(map_fh_err)?))
}

// ──────────────────────────────────────────────────────────────────────
// Per-symbol scan
// ──────────────────────────────────────────────────────────────────────

fn res_or_d(q: ResolutionQuery) -> String {
    q.resolution.unwrap_or_else(|| "D".into())
}

async fn scan_pattern(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<ResolutionQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::pattern_recognition(&s, &res_or_d(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn scan_support_resistance(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<ResolutionQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::support_resistance(&s, &res_or_d(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn scan_aggregate(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<ResolutionQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::aggregate_indicator(&s, &res_or_d(q))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn indicator(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<IndicatorQuery>,
) -> Result<Json<Value>, ApiError> {
    let res = q.resolution.unwrap_or_else(|| "D".into());
    let from = q
        .from
        .unwrap_or_else(|| chrono::Utc::now().timestamp() - 86_400 * 30)
        .to_string();
    let to =
        q.to.unwrap_or_else(|| chrono::Utc::now().timestamp())
            .to_string();
    let ind = q.indicator.unwrap_or_else(|| "sma".into());
    Ok(Json(
        fh::technical_indicator(&s, &res, &from, &to, &ind)
            .await
            .map_err(map_fh_err)?,
    ))
}

// ──────────────────────────────────────────────────────────────────────
// Broad calendars / news
// ──────────────────────────────────────────────────────────────────────

async fn earnings_calendar(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<CalEarningsQuery>,
) -> Result<Json<Value>, ApiError> {
    let from = q.from.unwrap_or_else(|| ago_str(7));
    let to = q.to.unwrap_or_else(|| {
        (chrono::Utc::now().date_naive() + chrono::Duration::days(7)).to_string()
    });
    Ok(Json(
        fh::earnings_calendar(&from, &to, q.symbol.as_deref())
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn ipo_calendar(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let today = chrono::Utc::now().date_naive();
    let from = q.from.unwrap_or_else(|| today.to_string());
    let to =
        q.to.unwrap_or_else(|| (today + chrono::Duration::days(60)).to_string());
    Ok(Json(
        fh::ipo_calendar(&from, &to).await.map_err(map_fh_err)?,
    ))
}
async fn economic_calendar(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 7);
    Ok(Json(
        fh::calendar_economic(&from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn fda_calendar(State(_s): State<AppState>, _u: AuthUser) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::fda_calendar().await.map_err(map_fh_err)?))
}
async fn earnings_call_live(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<CalEarningsQuery>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(
        FromTo {
            from: q.from,
            to: q.to,
        },
        7,
    );
    Ok(Json(
        fh::earnings_call_live(&from, &to, q.symbol.as_deref())
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn general_news(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<CategoryQuery>,
) -> Result<Json<Value>, ApiError> {
    let cat = q.category.unwrap_or_else(|| "general".into());
    Ok(Json(fh::general_news(&cat).await.map_err(map_fh_err)?))
}

// ──────────────────────────────────────────────────────────────────────
// Forex / Crypto
// ──────────────────────────────────────────────────────────────────────

async fn forex_exchanges(
    State(_s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::forex_exchanges().await.map_err(map_fh_err)?))
}
async fn forex_symbols(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<ExchangeOnly>,
) -> Result<Json<Value>, ApiError> {
    let ex = if q.exchange.is_empty() {
        "oanda".into()
    } else {
        q.exchange
    };
    Ok(Json(fh::forex_symbols(&ex).await.map_err(map_fh_err)?))
}
async fn forex_rates(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<BaseQuery>,
) -> Result<Json<Value>, ApiError> {
    let base = q.base.unwrap_or_else(|| "USD".into());
    Ok(Json(fh::forex_rates(&base).await.map_err(map_fh_err)?))
}
async fn forex_candle(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<SymbolCandleQuery>,
) -> Result<Json<Value>, ApiError> {
    let res = q.resolution.unwrap_or_else(|| "D".into());
    let from = q
        .from
        .unwrap_or_else(|| chrono::Utc::now().timestamp() - 86_400 * 30)
        .to_string();
    let to =
        q.to.unwrap_or_else(|| chrono::Utc::now().timestamp())
            .to_string();
    Ok(Json(
        fh::forex_candles(&q.symbol, &res, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn crypto_exchanges(
    State(_s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::crypto_exchanges().await.map_err(map_fh_err)?))
}
async fn crypto_symbols(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<ExchangeOnly>,
) -> Result<Json<Value>, ApiError> {
    let ex = if q.exchange.is_empty() {
        "binance".into()
    } else {
        q.exchange
    };
    Ok(Json(fh::crypto_symbols(&ex).await.map_err(map_fh_err)?))
}
async fn crypto_candle(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<SymbolCandleQuery>,
) -> Result<Json<Value>, ApiError> {
    let res = q.resolution.unwrap_or_else(|| "D".into());
    let from = q
        .from
        .unwrap_or_else(|| chrono::Utc::now().timestamp() - 86_400 * 30)
        .to_string();
    let to =
        q.to.unwrap_or_else(|| chrono::Utc::now().timestamp())
            .to_string();
    Ok(Json(
        fh::crypto_candles(&q.symbol, &res, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
#[derive(Deserialize)]
struct SymbolOnly {
    symbol: String,
}
async fn crypto_profile(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<SymbolOnly>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::crypto_profile(&q.symbol).await.map_err(map_fh_err)?,
    ))
}

// ──────────────────────────────────────────────────────────────────────
// Indices / ETF / Mutual Fund / Bond
// ──────────────────────────────────────────────────────────────────────

async fn index_constituents(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::indices_constituents(&s).await.map_err(map_fh_err)?,
    ))
}
async fn index_historical(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::indices_hist_constituents(&s)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn etf_profile(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::etf_profile(&s).await.map_err(map_fh_err)?))
}
async fn etf_holdings(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<HoldingsSkip>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::etf_holdings(&s, q.skip.unwrap_or(0))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn etf_sector(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::etf_sector(&s).await.map_err(map_fh_err)?))
}
async fn etf_country(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::etf_country(&s).await.map_err(map_fh_err)?))
}
async fn etf_allocation(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::etf_allocation(&s).await.map_err(map_fh_err)?))
}
async fn mf_profile(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::mutual_fund_profile(&s).await.map_err(map_fh_err)?))
}
async fn mf_holdings(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<HoldingsSkip>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::mutual_fund_holdings(&s, q.skip.unwrap_or(0))
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn mf_sector(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::mutual_fund_sector(&s).await.map_err(map_fh_err)?))
}
async fn mf_country(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::mutual_fund_country(&s).await.map_err(map_fh_err)?))
}
async fn mf_eet(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(isin): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::mutual_fund_eet(&isin).await.map_err(map_fh_err)?))
}
async fn bond_profile(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(isin): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::bond_profile(&isin).await.map_err(map_fh_err)?))
}
async fn bond_price(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(isin): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365);
    Ok(Json(
        fh::bond_price(&isin, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn bond_yield_curve(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<CodeQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::bond_yield_curve(&q.code).await.map_err(map_fh_err)?,
    ))
}

// ──────────────────────────────────────────────────────────────────────
// Economic / market / institutional
// ──────────────────────────────────────────────────────────────────────

async fn economic_codes(State(_s): State<AppState>, _u: AuthUser) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::economic_codes().await.map_err(map_fh_err)?))
}
async fn economic_data(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<CodeQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::economic_data(&q.code).await.map_err(map_fh_err)?))
}
async fn country_list(State(_s): State<AppState>, _u: AuthUser) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::country_list().await.map_err(map_fh_err)?))
}
async fn market_status(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<ExchangeOnly>,
) -> Result<Json<Value>, ApiError> {
    let ex = if q.exchange.is_empty() {
        "US".into()
    } else {
        q.exchange
    };
    Ok(Json(fh::market_status(&ex).await.map_err(map_fh_err)?))
}
async fn market_holiday(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<ExchangeOnly>,
) -> Result<Json<Value>, ApiError> {
    let ex = if q.exchange.is_empty() {
        "US".into()
    } else {
        q.exchange
    };
    Ok(Json(fh::market_holiday(&ex).await.map_err(map_fh_err)?))
}
async fn stock_exchanges(
    State(_s): State<AppState>,
    _u: AuthUser,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::stock_exchanges().await.map_err(map_fh_err)?))
}
async fn sector_metrics(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<CategoryQuery>,
) -> Result<Json<Value>, ApiError> {
    let region = q.category.unwrap_or_else(|| "NA".into());
    Ok(Json(fh::sector_metrics(&region).await.map_err(map_fh_err)?))
}
async fn institutional_profile(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(cik): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::institutional_profile(&cik).await.map_err(map_fh_err)?,
    ))
}
async fn institutional_portfolio(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(cik): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365);
    Ok(Json(
        fh::institutional_portfolio(&cik, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
async fn institutional_ownership(
    State(_s): State<AppState>,
    _u: AuthUser,
    Path(s): Path<String>,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 365);
    Ok(Json(
        fh::institutional_ownership(&s, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}

// ──────────────────────────────────────────────────────────────────────
// Discovery / specialty
// ──────────────────────────────────────────────────────────────────────

async fn symbol_lookup(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<QQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::symbol_lookup(&q.q).await.map_err(map_fh_err)?))
}
async fn stock_symbol_list(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<ExchangeOnly>,
) -> Result<Json<Value>, ApiError> {
    let ex = if q.exchange.is_empty() {
        "US".into()
    } else {
        q.exchange
    };
    Ok(Json(fh::stock_symbol_list(&ex).await.map_err(map_fh_err)?))
}
async fn symbol_change(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 90);
    Ok(Json(
        fh::symbol_change(&from, &to).await.map_err(map_fh_err)?,
    ))
}
async fn isin_change(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<FromTo>,
) -> Result<Json<Value>, ApiError> {
    let (from, to) = from_to_default(q, 90);
    Ok(Json(fh::isin_change(&from, &to).await.map_err(map_fh_err)?))
}
async fn covid19(State(_s): State<AppState>, _u: AuthUser) -> Result<Json<Value>, ApiError> {
    Ok(Json(fh::covid19_us().await.map_err(map_fh_err)?))
}
async fn investment_theme(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<ThemeQuery>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        fh::investment_theme(&q.theme).await.map_err(map_fh_err)?,
    ))
}
async fn airline_price_index(
    State(_s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<AirlineQuery>,
) -> Result<Json<Value>, ApiError> {
    let from = q.from.unwrap_or_else(|| ago_str(180));
    let to = q.to.unwrap_or_else(today_str);
    Ok(Json(
        fh::airline_price_index(&q.airline, &from, &to)
            .await
            .map_err(map_fh_err)?,
    ))
}
