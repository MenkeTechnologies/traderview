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
        .route("/options/:symbol", get(chain))
        .route("/options/:symbol/max-pain", get(max_pain_route))
        .route("/options/:symbol/gex", get(gex_route))
        .route("/options/:symbol/iv-skew", get(iv_skew_route))
        .route("/greeks", get(greeks_calc))
}

#[derive(Deserialize)]
struct ChainQ {
    expiration: Option<NaiveDate>,
}

async fn chain(
    _s: State<AppState>,
    _u: AuthUser,
    Path(sym): Path<String>,
    Query(q): Query<ChainQ>,
) -> Result<Json<Chain>, ApiError> {
    Ok(Json(
        traderview_db::options::chain(&sym.to_uppercase(), q.expiration)
            .await
            .map_err(ApiError::Internal)?,
    ))
}

#[derive(Deserialize)]
struct GQ {
    kind: OptKind,
    s: f64,
    k: f64,
    t: f64,
    sigma: f64,
    #[serde(default = "default_r")]
    r: f64,
    #[serde(default)]
    q: f64,
    #[serde(default)]
    market_price: Option<f64>,
}
fn default_r() -> f64 {
    0.045
}

#[derive(Serialize)]
struct GResp {
    greeks: Greeks,
    implied_vol: Option<f64>,
}

async fn greeks_calc(Query(q): Query<GQ>) -> Json<GResp> {
    let g = price_and_greeks(q.kind, q.s, q.k, q.t, q.sigma, q.r, q.q);
    let iv = q
        .market_price
        .and_then(|m| implied_vol(q.kind, m, q.s, q.k, q.t, q.r, q.q));
    Json(GResp {
        greeks: g,
        implied_vol: iv,
    })
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

/// Quantize a strike price to a u64 bucket key. Rounds (not truncates) so that
/// floats like 100.005 — which IEEE 754 represents as 100.00499999... — and
/// 100.005 from a different source don't bucket apart. Negative / non-finite
/// strikes collapse to bucket 0 (no panic), but real option strikes are
/// always positive finite so that path is unreachable on real Yahoo data.
fn strike_key(k: f64) -> u64 {
    if !k.is_finite() || k <= 0.0 { return 0; }
    (k * 10_000.0).round() as u64
}

/// Inverse of `strike_key` — recover the canonical strike from the bucket.
fn strike_from_key(k: u64) -> f64 { k as f64 / 10_000.0 }

// Merge a chain's calls + puts into one StrikeOi per strike. Yahoo's chain
// splits calls/puts but they share strike keys; OI analytics need them paired.
fn merge_oi(chain: &Chain) -> Vec<max_pain::StrikeOi> {
    let mut by_strike: BTreeMap<u64, (u64, u64)> = BTreeMap::new();
    for c in &chain.calls {
        let e = by_strike.entry(strike_key(c.strike)).or_default();
        e.0 += c.open_interest.unwrap_or(0).max(0) as u64;
    }
    for p in &chain.puts {
        let e = by_strike.entry(strike_key(p.strike)).or_default();
        e.1 += p.open_interest.unwrap_or(0).max(0) as u64;
    }
    by_strike.into_iter()
        .map(|(k, (call_oi, put_oi))| max_pain::StrikeOi {
            strike: strike_from_key(k),
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
    for c in &chain.calls {
        if let Some(iv) = c.implied_vol {
            by_strike.entry(strike_key(c.strike)).or_default().0 = Some(iv);
        }
    }
    for p in &chain.puts {
        if let Some(iv) = p.implied_vol {
            by_strike.entry(strike_key(p.strike)).or_default().1 = Some(iv);
        }
    }
    // Skew needs IV on BOTH legs at the same strike — drop strikes that only
    // quote one side; partial chains would skew the smile calculation.
    by_strike.into_iter().filter_map(|(k, (cv, pv))| {
        Some(iv_skew::IvByStrike {
            strike: strike_from_key(k),
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
    for c in &chain.calls {
        let Some(iv) = c.implied_vol else { continue };
        if iv <= 0.0 { continue; }
        let g = price_and_greeks(OptKind::Call, chain.spot, c.strike, t_years, iv, r, div);
        let e = by_strike.entry(strike_key(c.strike)).or_insert(gex::StrikeGreeks {
            strike: c.strike, call_gamma: 0.0, call_oi: 0, put_gamma: 0.0, put_oi: 0,
        });
        e.call_gamma = g.gamma;
        e.call_oi = c.open_interest.unwrap_or(0).max(0) as u64;
    }
    for p in &chain.puts {
        let Some(iv) = p.implied_vol else { continue };
        if iv <= 0.0 { continue; }
        let g = price_and_greeks(OptKind::Put, chain.spot, p.strike, t_years, iv, r, div);
        let e = by_strike.entry(strike_key(p.strike)).or_insert(gex::StrikeGreeks {
            strike: p.strike, call_gamma: 0.0, call_oi: 0, put_gamma: 0.0, put_oi: 0,
        });
        e.put_gamma = g.gamma;
        e.put_oi = p.open_interest.unwrap_or(0).max(0) as u64;
    }
    let strikes: Vec<gex::StrikeGreeks> = by_strike.into_values().collect();
    Ok(Json(gex::compute(&strikes, chain.spot)))
}

// ──────────────────────────────────────────────────────────────────────
// Tests for the strike-key quantizer. The OI/IV/GEX routes all depend
// on identical strikes from calls vs puts mapping to identical bucket
// keys; floating-point imprecision used to cause adjacent buckets when
// the raw multiplication produced 1000049.999... instead of 1000050.
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strike_key_rounds_floating_point_drift() {
        // 100.005 is NOT exactly representable in f64 — it's stored as
        // ~100.00499999999... A naive `as u64` truncation would produce
        // 1_000_049; with `.round()` it becomes 1_000_050.
        assert_eq!(strike_key(100.005), 1_000_050);
        assert_eq!(strike_key(100.0), 1_000_000);
        assert_eq!(strike_key(0.01), 100);
    }

    #[test]
    fn strike_key_collapses_identical_strikes_from_different_sources() {
        // The call-side fetcher might receive `100.005` parsed from one JSON
        // path while the put-side fetcher computes it as `50.0025 + 50.0025`.
        // Both should bucket together.
        let from_string = 100.005_f64;
        let from_arith  = 50.0025_f64 + 50.0025_f64;
        assert_eq!(strike_key(from_string), strike_key(from_arith),
            "identical strikes from different float paths must share a bucket");
    }

    #[test]
    fn strike_key_handles_pathological_inputs() {
        // Real Yahoo data is always positive finite, but the function must
        // not panic on garbage.
        assert_eq!(strike_key(-1.0), 0);
        assert_eq!(strike_key(0.0), 0);
        assert_eq!(strike_key(f64::NAN), 0);
        assert_eq!(strike_key(f64::INFINITY), 0);
        assert_eq!(strike_key(f64::NEG_INFINITY), 0);
    }

    #[test]
    fn strike_from_key_round_trips_through_strike_key() {
        // 4-decimal precision should survive a round trip exactly.
        for strike in [1.0, 50.0, 100.5, 250.25, 4_995.50, 100.005] {
            let k = strike_key(strike);
            let back = strike_from_key(k);
            assert!((back - strike).abs() < 1e-9,
                "round-trip lost precision: {strike} → {k} → {back}");
        }
    }
}
