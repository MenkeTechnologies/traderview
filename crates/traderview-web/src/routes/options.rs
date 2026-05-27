use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use traderview_core::greeks::{implied_vol, price_and_greeks, Greeks, OptKind};
use traderview_core::{gex, iv_skew, max_pain};
use traderview_db::options::Chain;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/options/:symbol",                  get(chain))
        .route("/options/:symbol/max-pain",         get(max_pain_route))
        .route("/options/:symbol/gex",              get(gex_route))
        .route("/options/:symbol/iv-skew",          get(iv_skew_route))
        .route("/greeks",                           get(greeks_calc))
}

#[derive(Deserialize)]
struct ChainQ { expiration: Option<NaiveDate> }

async fn chain(_s: State<AppState>, _u: AuthUser, Path(sym): Path<String>, Query(q): Query<ChainQ>)
    -> Result<Json<Chain>, ApiError>
{
    Ok(Json(traderview_db::options::chain(&sym.to_uppercase(), q.expiration)
        .await.map_err(ApiError::Internal)?))
}

#[derive(Deserialize)]
struct GQ {
    kind: OptKind,
    s: f64, k: f64, t: f64, sigma: f64,
    #[serde(default = "default_r")] r: f64,
    #[serde(default)] q: f64,
    #[serde(default)] market_price: Option<f64>,
}
fn default_r() -> f64 { 0.045 }

#[derive(Serialize)]
struct GResp { greeks: Greeks, implied_vol: Option<f64> }

async fn greeks_calc(Query(q): Query<GQ>) -> Json<GResp> {
    let g = price_and_greeks(q.kind, q.s, q.k, q.t, q.sigma, q.r, q.q);
    let iv = q.market_price.and_then(|m| implied_vol(q.kind, m, q.s, q.k, q.t, q.r, q.q));
    Json(GResp { greeks: g, implied_vol: iv })
}

// ---------------------------------------------------------------------------
// Options analytics — derived from the live chain.
// ---------------------------------------------------------------------------
//
// All three of max-pain, gex, and iv-skew are pure compute on top of the
// option chain Yahoo gives us.

#[derive(Deserialize)]
struct OptionsAnalyticsQ {
    expiration: Option<NaiveDate>,
}

async fn fetch_chain(symbol: &str, expiration: Option<NaiveDate>) -> Result<Chain, ApiError> {
    traderview_db::options::chain(&symbol.to_uppercase(), expiration)
        .await
        .map_err(ApiError::Internal)
}

// Merge a chain's calls + puts into one StrikeOi per strike. Yahoo's chain
// splits calls/puts but they share strike keys; OI analytics need them paired.
fn merge_oi(chain: &Chain) -> Vec<max_pain::StrikeOi> {
    let mut by_strike: BTreeMap<u64, (u64, u64)> = BTreeMap::new();
    let key = |k: f64| -> u64 { (k * 10_000.0) as u64 };
    for c in &chain.calls {
        let e = by_strike.entry(key(c.strike)).or_default();
        e.0 += c.open_interest.unwrap_or(0).max(0) as u64;
    }
    for p in &chain.puts {
        let e = by_strike.entry(key(p.strike)).or_default();
        e.1 += p.open_interest.unwrap_or(0).max(0) as u64;
    }
    by_strike.into_iter()
        .map(|(k, (call_oi, put_oi))| max_pain::StrikeOi {
            strike: k as f64 / 10_000.0,
            call_oi, put_oi,
        })
        .collect()
}

async fn max_pain_route(
    _s: State<AppState>, _u: AuthUser,
    Path(symbol): Path<String>, Query(q): Query<OptionsAnalyticsQ>,
) -> Result<Json<max_pain::MaxPainReport>, ApiError> {
    let chain = fetch_chain(&symbol, q.expiration).await?;
    Ok(Json(max_pain::compute(&merge_oi(&chain))))
}

#[derive(Deserialize)]
struct IvSkewQ {
    expiration: Option<NaiveDate>,
    /// OTM target distance as a fraction of spot (default 0.05 = 5% OTM each side).
    #[serde(default = "default_iv_pct_dist")]
    pct_distance: f64,
}
fn default_iv_pct_dist() -> f64 { 0.05 }

fn merge_iv(chain: &Chain) -> Vec<iv_skew::IvByStrike> {
    let mut by_strike: BTreeMap<u64, (Option<f64>, Option<f64>)> = BTreeMap::new();
    let key = |k: f64| -> u64 { (k * 10_000.0) as u64 };
    for c in &chain.calls {
        if let Some(iv) = c.implied_vol {
            by_strike.entry(key(c.strike)).or_default().0 = Some(iv);
        }
    }
    for p in &chain.puts {
        if let Some(iv) = p.implied_vol {
            by_strike.entry(key(p.strike)).or_default().1 = Some(iv);
        }
    }
    // Skew needs IV on BOTH legs at the same strike — drop strikes that only
    // quote one side; partial chains would skew the smile calculation.
    by_strike.into_iter().filter_map(|(k, (cv, pv))| {
        Some(iv_skew::IvByStrike {
            strike: k as f64 / 10_000.0,
            call_iv: cv?,
            put_iv: pv?,
        })
    }).collect()
}

async fn iv_skew_route(
    _s: State<AppState>, _u: AuthUser,
    Path(symbol): Path<String>, Query(q): Query<IvSkewQ>,
) -> Result<Json<iv_skew::SkewReport>, ApiError> {
    let chain = fetch_chain(&symbol, q.expiration).await?;
    let pairs = merge_iv(&chain);
    Ok(Json(iv_skew::analyze(&pairs, chain.spot, q.pct_distance)))
}

// GEX — Black–Scholes price each strike using the leg's IV + the chain's
// spot + days-to-expiry to derive gamma, then weight by OI.
async fn gex_route(
    _s: State<AppState>, _u: AuthUser,
    Path(symbol): Path<String>, Query(q): Query<OptionsAnalyticsQ>,
) -> Result<Json<gex::GexReport>, ApiError> {
    let chain = fetch_chain(&symbol, q.expiration).await?;
    let today = Utc::now().date_naive();
    let dte_days = (chain.expiration - today).num_days().max(0);
    let t_years = dte_days as f64 / 365.0;
    if t_years <= 0.0 {
        return Err(ApiError::BadRequest("expiration is in the past — gamma undefined".into()));
    }
    let r = 0.045_f64;    // matches the /greeks default
    let div = 0.0_f64;
    let mut by_strike: BTreeMap<u64, gex::StrikeGreeks> = BTreeMap::new();
    let key = |k: f64| -> u64 { (k * 10_000.0) as u64 };
    for c in &chain.calls {
        let Some(iv) = c.implied_vol else { continue };
        if iv <= 0.0 { continue; }
        let g = price_and_greeks(OptKind::Call, chain.spot, c.strike, t_years, iv, r, div);
        let e = by_strike.entry(key(c.strike)).or_insert(gex::StrikeGreeks {
            strike: c.strike, call_gamma: 0.0, call_oi: 0, put_gamma: 0.0, put_oi: 0,
        });
        e.call_gamma = g.gamma;
        e.call_oi = c.open_interest.unwrap_or(0).max(0) as u64;
    }
    for p in &chain.puts {
        let Some(iv) = p.implied_vol else { continue };
        if iv <= 0.0 { continue; }
        let g = price_and_greeks(OptKind::Put, chain.spot, p.strike, t_years, iv, r, div);
        let e = by_strike.entry(key(p.strike)).or_insert(gex::StrikeGreeks {
            strike: p.strike, call_gamma: 0.0, call_oi: 0, put_gamma: 0.0, put_oi: 0,
        });
        e.put_gamma = g.gamma;
        e.put_oi = p.open_interest.unwrap_or(0).max(0) as u64;
    }
    let strikes: Vec<gex::StrikeGreeks> = by_strike.into_values().collect();
    Ok(Json(gex::compute(&strikes, chain.spot)))
}
