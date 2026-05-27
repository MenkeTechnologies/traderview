use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use traderview_core::greeks::{implied_vol, price_and_greeks, Greeks, OptKind};
use traderview_db::options::Chain;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/options/:symbol",      get(chain))
        .route("/greeks",               get(greeks_calc))
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
