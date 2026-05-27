//! Per-symbol technical-indicator endpoints.
//!
//! Each route fetches bars in the requested `(interval, from, to)` window
//! and pipes them through one indicator module. The response is a series
//! aligned with the input bars (one point per bar, leading `None`s or zeroes
//! for pre-warmup positions where the indicator has no value yet).
//!
//! Frontends overlay these onto the candlestick view via the same
//! `bar_time` axis returned alongside the indicator value.

use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use traderview_core::{
    anchored_vwap, aroon, awesome_oscillator, bb_squeeze, cci, chaikin_volatility, coppock,
    donchian, dpo, force_index, indicators, keltner, mass_index, mfi, parabolic_sar, roc,
    rsi_divergence, schaff_trend, swing_points, trend_channel, trix, volume_flow, vortex,
    vwap_bands, williams_r, BarInterval, PriceBar,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // f64-only series.
        .route("/bars/:symbol/sma",                  get(sma_route))
        .route("/bars/:symbol/ema",                  get(ema_route))
        .route("/bars/:symbol/rsi",                  get(rsi_route))
        .route("/bars/:symbol/macd",                 get(macd_route))
        .route("/bars/:symbol/bollinger",            get(bollinger_route))
        .route("/bars/:symbol/atr",                  get(atr_route))
        .route("/bars/:symbol/roc",                  get(roc_route))
        .route("/bars/:symbol/trix",                 get(trix_route))
        .route("/bars/:symbol/dpo",                  get(dpo_route))
        .route("/bars/:symbol/coppock",              get(coppock_route))
        .route("/bars/:symbol/schaff-trend",         get(schaff_trend_route))
        .route("/bars/:symbol/mass-index",           get(mass_index_route))
        // Bar-based indicators with dedicated modules.
        .route("/bars/:symbol/adx",                  get(adx_route))
        .route("/bars/:symbol/stochastic",           get(stochastic_route))
        .route("/bars/:symbol/williams-r",           get(williams_r_route))
        .route("/bars/:symbol/cci",                  get(cci_route))
        .route("/bars/:symbol/mfi",                  get(mfi_route))
        .route("/bars/:symbol/donchian",             get(donchian_route))
        .route("/bars/:symbol/parabolic-sar",        get(parabolic_sar_route))
        .route("/bars/:symbol/anchored-vwap",        get(anchored_vwap_route))
        .route("/bars/:symbol/aroon",                get(aroon_route))
        .route("/bars/:symbol/awesome-oscillator",   get(awesome_oscillator_route))
        .route("/bars/:symbol/vortex",               get(vortex_route))
        .route("/bars/:symbol/chaikin-volatility",   get(chaikin_volatility_route))
        // Volume-flow.
        .route("/bars/:symbol/obv",                  get(obv_route))
        .route("/bars/:symbol/accumulation-distribution", get(accumulation_distribution_route))
        .route("/bars/:symbol/force-index",          get(force_index_route))
        // Volatility bands + structural detectors.
        .route("/bars/:symbol/keltner",              get(keltner_route))
        .route("/bars/:symbol/vwap-bands",           get(vwap_bands_route))
        .route("/bars/:symbol/bb-squeeze",           get(bb_squeeze_route))
        .route("/bars/:symbol/rsi-divergence",       get(rsi_divergence_route))
        .route("/bars/:symbol/trend-channel",        get(trend_channel_route))
}

#[derive(Deserialize)]
struct WindowQ {
    interval: String,
    from: i64,
    to: i64,
}

#[derive(Deserialize)]
struct PeriodQ {
    interval: String,
    from: i64,
    to: i64,
    #[serde(default = "default_period")]
    period: usize,
}
fn default_period() -> usize { 14 }

async fn fetch_bars(
    s: &AppState,
    symbol: &str,
    interval: &str,
    from: i64,
    to: i64,
) -> Result<Vec<PriceBar>, ApiError> {
    let iv = parse_interval(interval)
        .ok_or_else(|| ApiError::BadRequest(format!("unknown interval: {interval}")))?;
    let from_ts: DateTime<Utc> = Utc.timestamp_opt(from, 0).single()
        .ok_or_else(|| ApiError::BadRequest("bad from".into()))?;
    let to_ts: DateTime<Utc> = Utc.timestamp_opt(to, 0).single()
        .ok_or_else(|| ApiError::BadRequest("bad to".into()))?;
    traderview_db::prices::get_bars(&s.pool, symbol, iv, from_ts, to_ts)
        .await
        .map_err(ApiError::Internal)
}

fn parse_interval(s: &str) -> Option<BarInterval> {
    Some(match s {
        "1m" => BarInterval::M1,
        "5m" => BarInterval::M5,
        "15m" => BarInterval::M15,
        "1h" => BarInterval::H1,
        "1d" => BarInterval::D1,
        "1w" => BarInterval::W1,
        _ => return None,
    })
}

fn dec_f64(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }

// Each indicator response shares this shape so the JS overlay can dispatch
// generically: parallel `t` (bar time) + `v` (optional value or struct).
#[derive(Serialize)]
struct ScalarSeries {
    t: Vec<DateTime<Utc>>,
    v: Vec<Option<f64>>,
}

#[derive(Serialize)]
struct PlainSeries {
    t: Vec<DateTime<Utc>>,
    v: Vec<f64>,
}

// ---------------------------------------------------------------------------
// Closes-only series.
// ---------------------------------------------------------------------------

async fn sma_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<ScalarSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let v = indicators::sma(&closes, q.period);
    Ok(Json(ScalarSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

async fn ema_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<ScalarSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let v = indicators::ema(&closes, q.period);
    Ok(Json(ScalarSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

async fn rsi_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<ScalarSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let v = indicators::rsi(&closes, q.period);
    Ok(Json(ScalarSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

#[derive(Deserialize)]
struct MacdQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_macd_fast")]   fast: usize,
    #[serde(default = "default_macd_slow")]   slow: usize,
    #[serde(default = "default_macd_signal")] signal: usize,
}
fn default_macd_fast() -> usize { 12 }
fn default_macd_slow() -> usize { 26 }
fn default_macd_signal() -> usize { 9 }

#[derive(Serialize)]
struct MacdResponse {
    t: Vec<DateTime<Utc>>,
    line: Vec<Option<f64>>,
    signal: Vec<Option<f64>>,
    histogram: Vec<Option<f64>>,
}

async fn macd_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<MacdQ>,
) -> Result<Json<MacdResponse>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let m = indicators::macd(&closes, q.fast, q.slow, q.signal);
    Ok(Json(MacdResponse {
        t: bars.iter().map(|b| b.bar_time).collect(),
        line: m.line, signal: m.signal, histogram: m.histogram,
    }))
}

#[derive(Deserialize)]
struct BollingerQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_bb_period")] period: usize,
    #[serde(default = "default_bb_k")]      k: f64,
}
fn default_bb_period() -> usize { 20 }
fn default_bb_k() -> f64 { 2.0 }

#[derive(Serialize)]
struct BollingerResponse {
    t: Vec<DateTime<Utc>>,
    middle: Vec<Option<f64>>,
    upper:  Vec<Option<f64>>,
    lower:  Vec<Option<f64>>,
}

async fn bollinger_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<BollingerQ>,
) -> Result<Json<BollingerResponse>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let b = indicators::bollinger(&closes, q.period, q.k);
    Ok(Json(BollingerResponse {
        t: bars.iter().map(|b| b.bar_time).collect(),
        middle: b.middle, upper: b.upper, lower: b.lower,
    }))
}

async fn atr_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<ScalarSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let highs:  Vec<f64> = bars.iter().map(|b| dec_f64(b.high)).collect();
    let lows:   Vec<f64> = bars.iter().map(|b| dec_f64(b.low)).collect();
    let closes: Vec<f64> = bars.iter().map(|b| dec_f64(b.close)).collect();
    let v = indicators::atr(&highs, &lows, &closes, q.period);
    Ok(Json(ScalarSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

async fn roc_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let v = roc::compute(&closes, q.period);
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

async fn trix_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let v = trix::compute(&closes, q.period);
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

async fn dpo_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let v = dpo::compute(&closes, q.period);
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

#[derive(Deserialize)]
struct CoppockQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_coppock_roc1")] roc1: usize,
    #[serde(default = "default_coppock_roc2")] roc2: usize,
    #[serde(default = "default_coppock_wma")]  wma: usize,
}
fn default_coppock_roc1() -> usize { 14 }
fn default_coppock_roc2() -> usize { 11 }
fn default_coppock_wma() -> usize { 10 }

async fn coppock_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<CoppockQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let v = coppock::compute(&closes, q.roc1, q.roc2, q.wma);
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

#[derive(Deserialize)]
struct SchaffQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_schaff_short")] short: usize,
    #[serde(default = "default_schaff_long")]  long: usize,
    #[serde(default = "default_schaff_cycle")] cycle: usize,
}
fn default_schaff_short() -> usize { 23 }
fn default_schaff_long() -> usize { 50 }
fn default_schaff_cycle() -> usize { 10 }

async fn schaff_trend_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<SchaffQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let v = schaff_trend::compute(&closes, q.short, q.long, q.cycle);
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

#[derive(Deserialize)]
struct MassIndexQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_mi_ema")] ema_period: usize,
    #[serde(default = "default_mi_sum")] sum_period: usize,
}
fn default_mi_ema() -> usize { 9 }
fn default_mi_sum() -> usize { 25 }

async fn mass_index_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<MassIndexQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let highs = indicators::highs(&bars);
    let lows  = indicators::lows(&bars);
    let v = mass_index::compute(&highs, &lows, q.ema_period, q.sum_period);
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

// ---------------------------------------------------------------------------
// Bar-based indicators (dedicated modules with classification helpers).
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct AdxResponse {
    t: Vec<DateTime<Utc>>,
    adx: Vec<Option<f64>>,
    plus_di: Vec<Option<f64>>,
    minus_di: Vec<Option<f64>>,
}

async fn adx_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<AdxResponse>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let highs = indicators::highs(&bars);
    let lows  = indicators::lows(&bars);
    let closes = indicators::closes(&bars);
    let r = indicators::adx(&highs, &lows, &closes, q.period);
    Ok(Json(AdxResponse {
        t: bars.iter().map(|b| b.bar_time).collect(),
        adx: r.adx, plus_di: r.plus_di, minus_di: r.minus_di,
    }))
}

#[derive(Deserialize)]
struct StochQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_stoch_k")] k_period: usize,
    #[serde(default = "default_stoch_d")] d_period: usize,
}
fn default_stoch_k() -> usize { 14 }
fn default_stoch_d() -> usize { 3 }

#[derive(Serialize)]
struct StochResponse {
    t: Vec<DateTime<Utc>>,
    k: Vec<Option<f64>>,
    d: Vec<Option<f64>>,
}

async fn stochastic_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<StochQ>,
) -> Result<Json<StochResponse>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let highs = indicators::highs(&bars);
    let lows  = indicators::lows(&bars);
    let closes = indicators::closes(&bars);
    let r = indicators::stochastic(&highs, &lows, &closes, q.k_period, q.d_period);
    Ok(Json(StochResponse { t: bars.iter().map(|b| b.bar_time).collect(), k: r.k, d: r.d }))
}

async fn williams_r_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let inputs: Vec<williams_r::Bar> = bars.iter().map(|b| williams_r::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low), close: dec_f64(b.close),
    }).collect();
    let v = williams_r::compute(&inputs, q.period);
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

async fn cci_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let inputs: Vec<cci::Bar> = bars.iter().map(|b| cci::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low), close: dec_f64(b.close),
    }).collect();
    let v = cci::compute(&inputs, q.period);
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

#[derive(Serialize)]
struct MfiOut {
    t: DateTime<Utc>,
    #[serde(flatten)]
    point: mfi::MfiPoint,
}

async fn mfi_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<Vec<MfiOut>>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let inputs: Vec<mfi::Bar> = bars.iter().map(|b| mfi::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low),
        close: dec_f64(b.close), volume: dec_f64(b.volume),
    }).collect();
    let pts = mfi::compute(&inputs, q.period);
    let out = bars.iter().zip(pts).map(|(b, point)| MfiOut { t: b.bar_time, point }).collect();
    Ok(Json(out))
}

#[derive(Serialize)]
struct DonchianOut {
    t: DateTime<Utc>,
    #[serde(flatten)]
    point: donchian::DonchianPoint,
}

async fn donchian_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<Vec<DonchianOut>>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let inputs: Vec<donchian::Bar> = bars.iter().map(|b| donchian::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low), close: dec_f64(b.close),
    }).collect();
    let pts = donchian::compute(&inputs, q.period);
    let out = bars.iter().zip(pts).map(|(b, point)| DonchianOut { t: b.bar_time, point }).collect();
    Ok(Json(out))
}

#[derive(Deserialize)]
struct SarQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_sar_start")]     af_start: f64,
    #[serde(default = "default_sar_increment")] af_increment: f64,
    #[serde(default = "default_sar_max")]       af_max: f64,
}
fn default_sar_start() -> f64 { 0.02 }
fn default_sar_increment() -> f64 { 0.02 }
fn default_sar_max() -> f64 { 0.20 }

#[derive(Serialize)]
struct SarOut {
    t: DateTime<Utc>,
    #[serde(flatten)]
    point: parabolic_sar::SarPoint,
}

async fn parabolic_sar_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<SarQ>,
) -> Result<Json<Vec<SarOut>>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let inputs: Vec<parabolic_sar::Bar> = bars.iter().map(|b| parabolic_sar::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low),
    }).collect();
    let cfg = parabolic_sar::SarConfig { af_start: q.af_start, af_increment: q.af_increment, af_max: q.af_max };
    let pts = parabolic_sar::compute(&inputs, &cfg);
    let out = bars.iter().zip(pts).map(|(b, point)| SarOut { t: b.bar_time, point }).collect();
    Ok(Json(out))
}

#[derive(Deserialize)]
struct AnchoredVwapQ {
    interval: String, from: i64, to: i64,
    /// Bar index (0-based) within the returned window where the VWAP starts.
    /// 0 = anchor at the first bar in the window.
    #[serde(default)]
    anchor_index: usize,
}

#[derive(Serialize)]
struct AnchoredVwapOut {
    t: DateTime<Utc>,
    #[serde(flatten)]
    point: anchored_vwap::AnchoredPoint,
}

async fn anchored_vwap_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<AnchoredVwapQ>,
) -> Result<Json<Vec<AnchoredVwapOut>>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    if q.anchor_index >= bars.len() && !bars.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "anchor_index {} out of bounds (window has {} bars)", q.anchor_index, bars.len()
        )));
    }
    let inputs: Vec<anchored_vwap::Bar> = bars.iter().map(|b| {
        let typical = (dec_f64(b.high) + dec_f64(b.low) + dec_f64(b.close)) / 3.0;
        anchored_vwap::Bar { typical, volume: dec_f64(b.volume) }
    }).collect();
    let pts = anchored_vwap::compute(&inputs, q.anchor_index);
    let out = bars.iter().zip(pts).map(|(b, point)| AnchoredVwapOut { t: b.bar_time, point }).collect();
    Ok(Json(out))
}

#[derive(Serialize)]
struct AroonOut {
    t: DateTime<Utc>,
    #[serde(flatten)]
    point: aroon::AroonPoint,
}

async fn aroon_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<Vec<AroonOut>>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let inputs: Vec<aroon::Bar> = bars.iter().map(|b| aroon::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low),
    }).collect();
    let pts = aroon::compute(&inputs, q.period);
    let out = bars.iter().zip(pts).map(|(b, point)| AroonOut { t: b.bar_time, point }).collect();
    Ok(Json(out))
}

#[derive(Deserialize)]
struct AwesomeQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_ao_short")] short: usize,
    #[serde(default = "default_ao_long")]  long: usize,
}
fn default_ao_short() -> usize { 5 }
fn default_ao_long() -> usize { 34 }

async fn awesome_oscillator_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<AwesomeQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let inputs: Vec<awesome_oscillator::Bar> = bars.iter().map(|b| awesome_oscillator::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low),
    }).collect();
    let v = awesome_oscillator::compute(&inputs, q.short, q.long);
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

#[derive(Serialize)]
struct VortexOut {
    t: DateTime<Utc>,
    #[serde(flatten)]
    point: vortex::VortexPoint,
}

async fn vortex_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<PeriodQ>,
) -> Result<Json<Vec<VortexOut>>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let inputs: Vec<vortex::Bar> = bars.iter().map(|b| vortex::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low), close: dec_f64(b.close),
    }).collect();
    let pts = vortex::compute(&inputs, q.period);
    let out = bars.iter().zip(pts).map(|(b, point)| VortexOut { t: b.bar_time, point }).collect();
    Ok(Json(out))
}

#[derive(Deserialize)]
struct ChaikinVolQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_cv_ema")]    ema_period: usize,
    #[serde(default = "default_cv_change")] change_lookback: usize,
}
fn default_cv_ema() -> usize { 10 }
fn default_cv_change() -> usize { 10 }

async fn chaikin_volatility_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<ChaikinVolQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let inputs: Vec<chaikin_volatility::Bar> = bars.iter().map(|b| chaikin_volatility::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low),
    }).collect();
    let v = chaikin_volatility::compute(&inputs, q.ema_period, q.change_lookback);
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

// ---------------------------------------------------------------------------
// Volume-flow indicators.
// ---------------------------------------------------------------------------

fn vol_flow_inputs(bars: &[PriceBar]) -> Vec<volume_flow::Bar> {
    bars.iter().map(|b| volume_flow::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low),
        close: dec_f64(b.close), volume: dec_f64(b.volume),
    }).collect()
}

async fn obv_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<WindowQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let v = volume_flow::obv(&vol_flow_inputs(&bars));
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

async fn accumulation_distribution_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<WindowQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let v = volume_flow::accumulation_distribution(&vol_flow_inputs(&bars));
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

#[derive(Deserialize)]
struct ForceIndexQ {
    interval: String, from: i64, to: i64,
    /// 0 = raw force-index (no smoothing); >0 = EMA-smoothed over that period.
    #[serde(default)]
    period: usize,
}

async fn force_index_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<ForceIndexQ>,
) -> Result<Json<PlainSeries>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let inputs: Vec<force_index::Bar> = bars.iter().map(|b| force_index::Bar {
        close: dec_f64(b.close), volume: dec_f64(b.volume),
    }).collect();
    let v = if q.period == 0 {
        force_index::raw(&inputs)
    } else {
        force_index::smoothed(&inputs, q.period)
    };
    Ok(Json(PlainSeries { t: bars.iter().map(|b| b.bar_time).collect(), v }))
}

// ---------------------------------------------------------------------------
// Keltner channels — EMA midline with ATR bands.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct KeltnerQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_kelt_ema")]  ema_period: usize,
    #[serde(default = "default_kelt_atr")]  atr_period: usize,
    #[serde(default = "default_kelt_mult")] multiplier: f64,
}
fn default_kelt_ema() -> usize { 20 }
fn default_kelt_atr() -> usize { 10 }
fn default_kelt_mult() -> f64 { 2.0 }

#[derive(Serialize)]
struct KeltnerOut {
    t: DateTime<Utc>,
    #[serde(flatten)]
    point: keltner::KeltnerPoint,
}

async fn keltner_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<KeltnerQ>,
) -> Result<Json<Vec<KeltnerOut>>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let highs  = indicators::highs(&bars);
    let lows   = indicators::lows(&bars);
    // Fill pre-warmup gaps with 0 so the keltner output stays length-aligned
    // with bars; downstream consumers can read the leading zeros as "warm-up".
    let ema: Vec<f64> = indicators::ema(&closes, q.ema_period).into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let atr: Vec<f64> = indicators::atr(&highs, &lows, &closes, q.atr_period).into_iter().map(|v| v.unwrap_or(0.0)).collect();
    let pts = keltner::compute(&ema, &atr, q.multiplier);
    let out = bars.iter().zip(pts).map(|(b, point)| KeltnerOut { t: b.bar_time, point }).collect();
    Ok(Json(out))
}

// ---------------------------------------------------------------------------
// VWAP bands — single session VWAP snapshot with 1/2/3-σ bands.
// ---------------------------------------------------------------------------

async fn vwap_bands_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<WindowQ>,
) -> Result<Json<vwap_bands::VwapSnapshot>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    // VWAP wants (typical_price, volume) pairs across the window.
    let pairs: Vec<(f64, f64)> = bars.iter().map(|b| {
        let typical = (dec_f64(b.high) + dec_f64(b.low) + dec_f64(b.close)) / 3.0;
        (typical, dec_f64(b.volume))
    }).collect();
    Ok(Json(vwap_bands::final_snapshot(&pairs)))
}

// ---------------------------------------------------------------------------
// Bollinger-Band squeeze — detects when BB is tighter than KC (low-volatility
// coil) and the subsequent expansion direction. Needs precomputed SMA, σ,
// EMA, ATR per bar — we build those inline.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SqueezeQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_sq_sma")] sma_period: usize,
    #[serde(default = "default_sq_ema")] ema_period: usize,
    #[serde(default = "default_sq_atr")] atr_period: usize,
}
fn default_sq_sma() -> usize { 20 }
fn default_sq_ema() -> usize { 20 }
fn default_sq_atr() -> usize { 20 }

#[derive(Serialize)]
struct SqueezeOut {
    t: DateTime<Utc>,
    #[serde(flatten)]
    point: bb_squeeze::SqueezeBar,
}

async fn bb_squeeze_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<SqueezeQ>,
) -> Result<Json<Vec<SqueezeOut>>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let highs  = indicators::highs(&bars);
    let lows   = indicators::lows(&bars);
    let sma  = indicators::sma(&closes, q.sma_period);
    let ema  = indicators::ema(&closes, q.ema_period);
    let atr  = indicators::atr(&highs, &lows, &closes, q.atr_period);
    // Rolling standard deviation of closes over sma_period.
    let stdev = rolling_stdev(&closes, q.sma_period);
    // Build squeeze inputs only for bars where ALL four components are valid;
    // leading positions become "passive" inputs (close + zeros) and the
    // detector treats them as non-squeezed.
    let inputs: Vec<bb_squeeze::SqueezeInput> = (0..bars.len()).map(|i| bb_squeeze::SqueezeInput {
        close: closes[i],
        sma_20:   sma[i].unwrap_or(0.0),
        stdev_20: stdev[i].unwrap_or(0.0),
        ema_20:   ema[i].unwrap_or(0.0),
        atr_20:   atr[i].unwrap_or(0.0),
    }).collect();
    let pts = bb_squeeze::analyze(&inputs);
    let out = bars.iter().zip(pts).map(|(b, point)| SqueezeOut { t: b.bar_time, point }).collect();
    Ok(Json(out))
}

fn rolling_stdev(values: &[f64], window: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if window == 0 || n < window { return out; }
    for i in (window - 1)..n {
        let slice = &values[(i + 1 - window)..=i];
        let m = slice.iter().sum::<f64>() / window as f64;
        let var = slice.iter().map(|v| (v - m).powi(2)).sum::<f64>() / window as f64;
        out[i] = Some(var.sqrt());
    }
    out
}

// ---------------------------------------------------------------------------
// RSI divergence — bullish/bearish divergence on swing points.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct DivergenceQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_div_lookback")] swing_lookback: usize,
    #[serde(default = "default_div_period")]   rsi_period: usize,
}
fn default_div_lookback() -> usize { 3 }
fn default_div_period() -> usize { 14 }

async fn rsi_divergence_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<DivergenceQ>,
) -> Result<Json<Vec<rsi_divergence::Divergence>>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    let closes = indicators::closes(&bars);
    let rsi = indicators::rsi(&closes, q.rsi_period);
    let series: Vec<rsi_divergence::PriceRsiPoint> = (0..bars.len()).filter_map(|i| {
        rsi[i].map(|r| rsi_divergence::PriceRsiPoint { bar_index: i, price: closes[i], rsi: r })
    }).collect();
    let swing_bars: Vec<swing_points::Bar> = bars.iter()
        .map(|b| swing_points::Bar { high: dec_f64(b.high), low: dec_f64(b.low) })
        .collect();
    let swings = swing_points::detect(&swing_bars, q.swing_lookback);
    Ok(Json(rsi_divergence::detect(&swings, &series)))
}

// ---------------------------------------------------------------------------
// Auto trend channel — fits an upper+lower trendline from swing points.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ChannelQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_channel_lookback")] swing_lookback: usize,
}
fn default_channel_lookback() -> usize { 3 }

async fn trend_channel_route(
    State(s): State<AppState>, Path(sym): Path<String>, Query(q): Query<ChannelQ>,
) -> Result<Json<trend_channel::ChannelReport>, ApiError> {
    let bars = fetch_bars(&s, &sym, &q.interval, q.from, q.to).await?;
    if bars.is_empty() {
        return Err(ApiError::BadRequest("no bars in window".into()));
    }
    let swing_bars: Vec<swing_points::Bar> = bars.iter()
        .map(|b| swing_points::Bar { high: dec_f64(b.high), low: dec_f64(b.low) })
        .collect();
    let swings = swing_points::detect(&swing_bars, q.swing_lookback);
    // Split swing highs vs lows. trend_channel uses its own SwingPoint shape
    // (bar_index, price) — translate here.
    let (highs_only, lows_only): (Vec<_>, Vec<_>) = swings.iter().partition(|s| {
        matches!(s.kind, swing_points::SwingKind::High)
    });
    let to_local = |s: &swing_points::SwingPoint| trend_channel::SwingPoint {
        bar_index: s.index, price: s.price,
    };
    let swings_high: Vec<trend_channel::SwingPoint> = highs_only.iter().map(|s| to_local(s)).collect();
    let swings_low:  Vec<trend_channel::SwingPoint> = lows_only.iter().map(|s| to_local(s)).collect();
    let last_bar = bars.len() - 1;
    let report = trend_channel::fit(&swings_high, &swings_low, last_bar)
        .ok_or_else(|| ApiError::BadRequest(
            "not enough swing points to fit a channel — try widening the window or shrinking swing_lookback".into()
        ))?;
    Ok(Json(report))
}

// ──────────────────────────────────────────────────────────────────────
// Tests for inline helpers. The `rolling_stdev` helper feeds the
// bb-squeeze route's SqueezeInput — wrong stdev there silently breaks
// every BB-squeeze signal across every chart that uses it.
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolling_stdev_pre_warmup_slots_are_none() {
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let s = rolling_stdev(&v, 3);
        assert!(s[0].is_none() && s[1].is_none(), "first window-1 slots must be None");
        assert!(s[2].is_some(), "slot at window-1 must be Some");
    }

    #[test]
    fn rolling_stdev_constant_window_is_zero() {
        // Flat 5.0 for any window → stdev = 0 in every populated slot.
        let v = vec![5.0; 10];
        let s = rolling_stdev(&v, 4);
        for slot in s.iter().skip(3) {
            let val = slot.expect("populated after warmup");
            assert!(val.abs() < 1e-12, "flat series stdev should be 0, got {val}");
        }
    }

    #[test]
    fn rolling_stdev_window_zero_or_too_big_returns_all_none() {
        let v = vec![1.0, 2.0, 3.0];
        // window=0 — degenerate, return all-None rather than divide by zero.
        let s0 = rolling_stdev(&v, 0);
        assert!(s0.iter().all(|x| x.is_none()));
        // window > len — no slot has enough data, return all-None.
        let s_big = rolling_stdev(&v, 10);
        assert!(s_big.iter().all(|x| x.is_none()));
    }

    #[test]
    fn rolling_stdev_population_formula_matches_hand_calc() {
        // Window [1, 2, 3]: mean=2, variance=(1+0+1)/3 = 0.6666..., stdev=√(2/3).
        let v = vec![1.0, 2.0, 3.0];
        let s = rolling_stdev(&v, 3);
        let want = (2.0_f64 / 3.0_f64).sqrt();
        let got = s[2].expect("populated");
        assert!((got - want).abs() < 1e-12,
            "expected stdev {want} for [1,2,3], got {got}");
    }

    #[test]
    fn rolling_stdev_window_advances_with_input() {
        // Window [1,2,3] then [2,3,4] — should both produce the same stdev
        // (arithmetic progression with same step preserves variance).
        let v = vec![1.0, 2.0, 3.0, 4.0];
        let s = rolling_stdev(&v, 3);
        let a = s[2].expect("populated");
        let b = s[3].expect("populated");
        assert!((a - b).abs() < 1e-12,
            "stdev of constant-step windows must match: {a} vs {b}");
    }
}
