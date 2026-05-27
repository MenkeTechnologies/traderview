//! Pair / correlation matrix routes.
use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use traderview_core::correlation::{log_returns, pair_analysis, pearson, PairAnalysis};
use traderview_core::BarInterval;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/analysis/correlation", get(matrix))
        .route("/analysis/pair", get(pair))
}

#[derive(Deserialize)]
struct MQ {
    symbols: String, // CSV
    #[serde(default = "default_days")]
    days: i64,
}
fn default_days() -> i64 {
    90
}

#[derive(Serialize)]
struct MatrixResp {
    symbols: Vec<String>,
    matrix: Vec<Vec<f64>>,
    days: i64,
    samples: usize,
}

async fn matrix(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<MQ>,
) -> Result<Json<MatrixResp>, ApiError> {
    let syms: Vec<String> = q
        .symbols
        .split(',')
        .map(|x| x.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
    if syms.len() < 2 {
        return Err(ApiError::BadRequest("need ≥ 2 symbols".into()));
    }
    let to = Utc::now();
    let from = to - Duration::days(q.days);
    let mut all_closes: Vec<Vec<f64>> = Vec::with_capacity(syms.len());
    for sym in &syms {
        let bars = traderview_db::prices::get_bars(&s.pool, sym, BarInterval::D1, from, to)
            .await
            .map_err(ApiError::Internal)?;
        all_closes.push(bars.iter().map(|b| dec(b.close)).collect());
    }
    let min_n = all_closes.iter().map(|v| v.len()).min().unwrap_or(0);
    if min_n < 5 {
        return Err(ApiError::BadRequest("not enough bars".into()));
    }
    let returns: Vec<Vec<f64>> = all_closes
        .iter()
        .map(|c| log_returns(&c[c.len() - min_n..]))
        .collect();
    let n = syms.len();
    let mut mat = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            mat[i][j] = if i == j {
                1.0
            } else {
                pearson(&returns[i], &returns[j]).unwrap_or(0.0)
            };
        }
    }
    Ok(Json(MatrixResp {
        symbols: syms,
        matrix: mat,
        days: q.days,
        samples: min_n,
    }))
}

fn dec(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[derive(Deserialize)]
struct PQ {
    a: String,
    b: String,
    #[serde(default = "default_days")]
    days: i64,
}

async fn pair(
    State(s): State<AppState>,
    _u: AuthUser,
    Query(q): Query<PQ>,
) -> Result<Json<PairAnalysis>, ApiError> {
    let to = Utc::now();
    let from = to - Duration::days(q.days);
    let ba =
        traderview_db::prices::get_bars(&s.pool, &q.a.to_uppercase(), BarInterval::D1, from, to)
            .await
            .map_err(ApiError::Internal)?;
    let bb =
        traderview_db::prices::get_bars(&s.pool, &q.b.to_uppercase(), BarInterval::D1, from, to)
            .await
            .map_err(ApiError::Internal)?;
    let pa: Vec<f64> = ba.iter().map(|b| dec(b.close)).collect();
    let pb: Vec<f64> = bb.iter().map(|b| dec(b.close)).collect();
    let n = pa.len().min(pb.len());
    if n < 30 {
        return Err(ApiError::BadRequest("need ≥ 30 overlapping bars".into()));
    }
    let pa = &pa[pa.len() - n..];
    let pb = &pb[pb.len() - n..];
    pair_analysis(pa, pb)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("pair analysis failed (insufficient variance)".into()))
}
