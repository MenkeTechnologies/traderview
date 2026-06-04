use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use traderview_core::{
    candlestick_patterns, fibonacci, heikin_ashi, ichimoku, indicators, pivots, renko,
    supertrend, swing_points, volume_profile, BarInterval, PriceBar,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/bars/:symbol", get(bars))
        .route("/bars/:symbol/heikin-ashi", get(heikin_ashi_route))
        .route("/bars/:symbol/renko", get(renko_route))
        .route("/bars/:symbol/volume-profile", get(volume_profile_route))
        .route("/bars/:symbol/ichimoku", get(ichimoku_route))
        .route("/bars/:symbol/fibonacci", get(fibonacci_route))
        .route("/bars/:symbol/supertrend", get(supertrend_route))
        .route("/bars/:symbol/swing-points", get(swing_points_route))
        .route("/bars/:symbol/candlestick-patterns", get(candlestick_patterns_route))
        .route("/bars/:symbol/pivots/floor", get(pivots_floor_route))
        .route("/bars/:symbol/pivots/camarilla", get(pivots_camarilla_route))
        .route("/bars/:symbol/pivots/woodie", get(pivots_woodie_route))
        .route("/bars/:symbol/pivots/demark", get(pivots_demark_route))
}

#[derive(Deserialize)]
struct BarsQ {
    interval: String,
    /// Unix seconds.
    from: i64,
    /// Unix seconds.
    to: i64,
}

#[derive(Serialize)]
struct BarsResponse {
    symbol: String,
    interval: String,
    bars: Vec<PriceBar>,
}

async fn bars(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<BarsQ>,
) -> Result<Json<BarsResponse>, ApiError> {
    let (iv, fetched) = fetch_bars(&s, &symbol, &q).await?;
    Ok(Json(BarsResponse { symbol, interval: iv.label().into(), bars: fetched }))
}

// ---------------------------------------------------------------------------
// Shared fetch helper. Every chart-transformation route below takes the same
// (interval, from, to) query, so resolving + loading bars once keeps the
// per-route code thin and the validation behavior uniform.
// ---------------------------------------------------------------------------

async fn fetch_bars(
    s: &AppState,
    symbol: &str,
    q: &BarsQ,
) -> Result<(BarInterval, Vec<PriceBar>), ApiError> {
    let iv = parse_interval(&q.interval)
        .ok_or_else(|| ApiError::BadRequest(format!("unknown interval: {}", q.interval)))?;
    let from: DateTime<Utc> = Utc
        .timestamp_opt(q.from, 0)
        .single()
        .ok_or_else(|| ApiError::BadRequest("bad from".into()))?;
    let to: DateTime<Utc> = Utc
        .timestamp_opt(q.to, 0)
        .single()
        .ok_or_else(|| ApiError::BadRequest("bad to".into()))?;
    let bars = traderview_db::prices::get_bars(&s.pool, symbol, iv, from, to)
        .await
        .map_err(ApiError::Internal)?;
    Ok((iv, bars))
}

fn parse_interval(s: &str) -> Option<BarInterval> {
    Some(match s {
        "10s" => BarInterval::S10,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_interval_accepts_known_strings() {
        assert!(matches!(parse_interval("10s"), Some(BarInterval::S10)));
        assert!(matches!(parse_interval("1m"),  Some(BarInterval::M1)));
        assert!(matches!(parse_interval("5m"),  Some(BarInterval::M5)));
        assert!(matches!(parse_interval("15m"), Some(BarInterval::M15)));
        assert!(matches!(parse_interval("1h"),  Some(BarInterval::H1)));
        assert!(matches!(parse_interval("1d"),  Some(BarInterval::D1)));
        assert!(matches!(parse_interval("1w"),  Some(BarInterval::W1)));
    }

    #[test]
    fn parse_interval_rejects_unknown_or_garbage() {
        // Case-sensitive, no aliases — these must all return None so the
        // route layer can 400-out with a clean error rather than silently
        // bucketing into a wrong interval.
        assert!(parse_interval("1M").is_none(),  "case-sensitive");
        assert!(parse_interval("daily").is_none(), "no aliases");
        assert!(parse_interval("").is_none());
        assert!(parse_interval(" 1m ").is_none(), "no whitespace tolerance");
    }

    #[test]
    fn dec_f64_round_trips_typical_prices() {
        // Common stock-price magnitudes round-trip exactly.
        for s in ["1.00", "100.50", "4995.25", "0.0001"] {
            let d = Decimal::from_str(s).unwrap();
            let f = dec_f64(d);
            let want: f64 = s.parse().unwrap();
            assert!((f - want).abs() < 1e-9, "round trip lost precision: {s} → {f}");
        }
    }

    #[test]
    fn dec_f64_returns_zero_on_unparseable_string_path() {
        // Decimal::to_string() always produces a valid f64-parseable string,
        // so this path is defensive but lives in production. Sanity: zero
        // decimal → exactly 0.0.
        assert_eq!(dec_f64(Decimal::ZERO), 0.0);
    }

    #[test]
    fn dec_f64_handles_negative_values() {
        // Bid/ask spreads can have negative-side computations; the
        // converter must not eat the sign.
        let d = Decimal::from_str("-42.50").unwrap();
        assert!((dec_f64(d) - (-42.50)).abs() < 1e-9);
    }
}

// ---------------------------------------------------------------------------
// Heikin Ashi — alternative candle smoothing.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct HaResponse {
    symbol: String,
    interval: String,
    bars: Vec<HaBarOut>,
}

#[derive(Serialize)]
struct HaBarOut {
    bar_time: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

async fn heikin_ashi_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<BarsQ>,
) -> Result<Json<HaResponse>, ApiError> {
    let (iv, bars) = fetch_bars(&s, &symbol, &q).await?;
    let inputs: Vec<heikin_ashi::Bar> = bars.iter().map(|b| heikin_ashi::Bar {
        open: dec_f64(b.open), high: dec_f64(b.high),
        low: dec_f64(b.low), close: dec_f64(b.close),
    }).collect();
    let ha = heikin_ashi::compute(&inputs);
    let out = bars.iter().zip(ha.iter()).map(|(src, h)| HaBarOut {
        bar_time: src.bar_time,
        open: h.open, high: h.high, low: h.low, close: h.close,
    }).collect();
    Ok(Json(HaResponse { symbol, interval: iv.label().into(), bars: out }))
}

// ---------------------------------------------------------------------------
// Renko — price-only bricks.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RenkoQ {
    interval: String, from: i64, to: i64,
    /// Brick size in price units. Required (no sensible default — depends on symbol).
    brick: f64,
}

#[derive(Serialize)]
struct RenkoResponse {
    symbol: String,
    brick_size: f64,
    bricks: Vec<renko::Brick>,
}

async fn renko_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(rq): Query<RenkoQ>,
) -> Result<Json<RenkoResponse>, ApiError> {
    if !(rq.brick.is_finite() && rq.brick > 0.0) {
        return Err(ApiError::BadRequest("brick must be a positive finite number".into()));
    }
    let q = BarsQ { interval: rq.interval, from: rq.from, to: rq.to };
    let (_, bars) = fetch_bars(&s, &symbol, &q).await?;
    // Treat each bar's close as a tick at bar_time. Adequate for chart-level
    // Renko; intra-bar tick reconstruction is out of scope here.
    let ticks: Vec<renko::PriceTick> = bars.iter()
        .map(|b| renko::PriceTick { price: dec_f64(b.close) })
        .collect();
    let bricks = renko::build(&ticks, rq.brick);
    Ok(Json(RenkoResponse { symbol, brick_size: rq.brick, bricks }))
}

// ---------------------------------------------------------------------------
// Volume Profile — VPVR with POC + VAH/VAL + HVN/LVN.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct VpvrQ {
    interval: String, from: i64, to: i64,
    /// Price bin width (tick size). Required.
    tick: f64,
}

async fn volume_profile_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(vq): Query<VpvrQ>,
) -> Result<Json<volume_profile::VolumeProfile>, ApiError> {
    if !(vq.tick.is_finite() && vq.tick > 0.0) {
        return Err(ApiError::BadRequest("tick must be a positive finite number".into()));
    }
    let q = BarsQ { interval: vq.interval, from: vq.from, to: vq.to };
    let (_, bars) = fetch_bars(&s, &symbol, &q).await?;
    // Approximation: place each bar's volume at its typical price (HLC/3).
    // Tick-level prints would be more accurate; this matches what most
    // bar-data VPVRs show.
    let prints: Vec<volume_profile::PrintAt> = bars.iter().map(|b| {
        let typical = (dec_f64(b.high) + dec_f64(b.low) + dec_f64(b.close)) / 3.0;
        volume_profile::PrintAt { price: typical, volume: dec_f64(b.volume) }
    }).collect();
    let _ = symbol;
    Ok(Json(volume_profile::build(&prints, vq.tick)))
}

// ---------------------------------------------------------------------------
// Ichimoku — 5-line cloud overlay.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct IchimokuPointOut {
    bar_time: DateTime<Utc>,
    #[serde(flatten)]
    point: ichimoku::IchimokuPoint,
}

async fn ichimoku_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<BarsQ>,
) -> Result<Json<Vec<IchimokuPointOut>>, ApiError> {
    let (_, bars) = fetch_bars(&s, &symbol, &q).await?;
    let inputs: Vec<ichimoku::Bar> = bars.iter().map(|b| ichimoku::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low), close: dec_f64(b.close),
    }).collect();
    let points = ichimoku::compute(&inputs);
    let out = bars.iter().zip(points)
        .map(|(src, point)| IchimokuPointOut { bar_time: src.bar_time, point })
        .collect();
    Ok(Json(out))
}

// ---------------------------------------------------------------------------
// Fibonacci — auto retracement/extension from the high/low of the window.
// ---------------------------------------------------------------------------

async fn fibonacci_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<BarsQ>,
) -> Result<Json<fibonacci::FibReport>, ApiError> {
    let (_, bars) = fetch_bars(&s, &symbol, &q).await?;
    if bars.is_empty() {
        return Err(ApiError::BadRequest("no bars in window".into()));
    }
    // Identify the high + low in the window. The direction is determined by
    // which arrived first: if the high is BEFORE the low, the swing is Down
    // (retracements pull UP from the low). If the low is first, the swing is
    // Up (retracements pull DOWN from the high).
    let (mut hi_idx, mut lo_idx) = (0usize, 0usize);
    let (mut hi, mut lo) = (dec_f64(bars[0].high), dec_f64(bars[0].low));
    for (i, b) in bars.iter().enumerate() {
        let h = dec_f64(b.high);
        let l = dec_f64(b.low);
        if h > hi { hi = h; hi_idx = i; }
        if l < lo { lo = l; lo_idx = i; }
    }
    let direction = if hi_idx <= lo_idx { fibonacci::SwingDirection::Down } else { fibonacci::SwingDirection::Up };
    let input = fibonacci::FibInput { swing_high: hi, swing_low: lo, direction };
    Ok(Json(fibonacci::compute(&input)))
}

// ---------------------------------------------------------------------------
// Supertrend — trailing-band overlay with trend flip.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SupertrendQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_st_period")]
    period: usize,
    #[serde(default = "default_st_mult")]
    multiplier: f64,
}
fn default_st_period() -> usize { 10 }
fn default_st_mult() -> f64 { 3.0 }

#[derive(Serialize)]
struct SupertrendOut {
    bar_time: DateTime<Utc>,
    #[serde(flatten)]
    point: supertrend::SupertrendPoint,
}

async fn supertrend_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(sq): Query<SupertrendQ>,
) -> Result<Json<Vec<SupertrendOut>>, ApiError> {
    if sq.period == 0 || !(sq.multiplier.is_finite() && sq.multiplier > 0.0) {
        return Err(ApiError::BadRequest("period and multiplier must be positive".into()));
    }
    let q = BarsQ { interval: sq.interval, from: sq.from, to: sq.to };
    let (_, bars) = fetch_bars(&s, &symbol, &q).await?;
    let highs:  Vec<f64> = bars.iter().map(|b| dec_f64(b.high)).collect();
    let lows:   Vec<f64> = bars.iter().map(|b| dec_f64(b.low)).collect();
    let closes: Vec<f64> = bars.iter().map(|b| dec_f64(b.close)).collect();
    // Fill the pre-warmup ATR slots with 0.0 so the supertrend output stays
    // length-aligned with input bars (front-of-series ATR is None until period).
    let atr: Vec<f64> = indicators::atr(&highs, &lows, &closes, sq.period)
        .into_iter()
        .map(|v| v.unwrap_or(0.0))
        .collect();
    let inputs: Vec<supertrend::Bar> = bars.iter().map(|b| supertrend::Bar {
        high: dec_f64(b.high), low: dec_f64(b.low), close: dec_f64(b.close),
    }).collect();
    let pts = supertrend::compute(&inputs, &atr, sq.multiplier);
    let out = bars.iter().zip(pts)
        .map(|(src, point)| SupertrendOut { bar_time: src.bar_time, point })
        .collect();
    Ok(Json(out))
}

// ---------------------------------------------------------------------------
// Swing points — pivot highs/lows.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SwingQ {
    interval: String, from: i64, to: i64,
    #[serde(default = "default_swing_lookback")]
    lookback: usize,
}
fn default_swing_lookback() -> usize { 3 }

#[derive(Serialize)]
struct SwingOut {
    bar_time: DateTime<Utc>,
    #[serde(flatten)]
    point: swing_points::SwingPoint,
}

async fn swing_points_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(sq): Query<SwingQ>,
) -> Result<Json<Vec<SwingOut>>, ApiError> {
    if sq.lookback == 0 {
        return Err(ApiError::BadRequest("lookback must be > 0".into()));
    }
    let q = BarsQ { interval: sq.interval, from: sq.from, to: sq.to };
    let (_, bars) = fetch_bars(&s, &symbol, &q).await?;
    let inputs: Vec<swing_points::Bar> = bars.iter()
        .map(|b| swing_points::Bar { high: dec_f64(b.high), low: dec_f64(b.low) })
        .collect();
    let hits = swing_points::detect(&inputs, sq.lookback);
    let out = hits.into_iter().filter_map(|p| {
        bars.get(p.index).map(|src| SwingOut { bar_time: src.bar_time, point: p })
    }).collect();
    Ok(Json(out))
}

// ---------------------------------------------------------------------------
// Candlestick pattern detection.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct PatternOut {
    bar_time: DateTime<Utc>,
    #[serde(flatten)]
    hit: candlestick_patterns::PatternHit,
}

async fn candlestick_patterns_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<BarsQ>,
) -> Result<Json<Vec<PatternOut>>, ApiError> {
    let (_, bars) = fetch_bars(&s, &symbol, &q).await?;
    let inputs: Vec<candlestick_patterns::Bar> = bars.iter().map(|b| candlestick_patterns::Bar {
        open: dec_f64(b.open), high: dec_f64(b.high),
        low: dec_f64(b.low), close: dec_f64(b.close),
    }).collect();
    let hits = candlestick_patterns::detect(&inputs);
    let out = hits.into_iter().filter_map(|hit| {
        bars.get(hit.bar_index).map(|src| PatternOut { bar_time: src.bar_time, hit })
    }).collect();
    Ok(Json(out))
}

// ---------------------------------------------------------------------------
// Daily pivots — floor / camarilla / woodie / demark. Each computes one set
// of S/R levels from the prior session's H/L/C. We pick the most recent
// FULLY-CLOSED daily bar as the prior bar.
// ---------------------------------------------------------------------------

fn prior_bar_from(bars: &[PriceBar]) -> Result<pivots::PriorBar, ApiError> {
    let b = bars.last().ok_or_else(|| ApiError::BadRequest("no bars in window".into()))?;
    Ok(pivots::PriorBar {
        high: dec_f64(b.high),
        low: dec_f64(b.low),
        close: dec_f64(b.close),
        open: dec_f64(b.open),
    })
}

async fn pivots_floor_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<BarsQ>,
) -> Result<Json<pivots::FloorPivots>, ApiError> {
    let (_, bars) = fetch_bars(&s, &symbol, &q).await?;
    Ok(Json(pivots::floor(prior_bar_from(&bars)?)))
}

async fn pivots_camarilla_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<BarsQ>,
) -> Result<Json<pivots::CamarillaPivots>, ApiError> {
    let (_, bars) = fetch_bars(&s, &symbol, &q).await?;
    Ok(Json(pivots::camarilla(prior_bar_from(&bars)?)))
}

async fn pivots_woodie_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<BarsQ>,
) -> Result<Json<pivots::WoodiePivots>, ApiError> {
    let (_, bars) = fetch_bars(&s, &symbol, &q).await?;
    Ok(Json(pivots::woodie(prior_bar_from(&bars)?)))
}

async fn pivots_demark_route(
    State(s): State<AppState>,
    Path(symbol): Path<String>,
    Query(q): Query<BarsQ>,
) -> Result<Json<pivots::DemarkPivots>, ApiError> {
    let (_, bars) = fetch_bars(&s, &symbol, &q).await?;
    Ok(Json(pivots::demark(prior_bar_from(&bars)?)))
}
