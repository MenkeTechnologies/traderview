use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use traderview_db::vol::{DollarSnapshot, VixTermStructure, YieldCurve};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/vol/vix",     get(vix))
        .route("/vol/yields",  get(yields))
        .route("/vol/dollar",  get(dollar))
}

async fn vix(State(s): State<AppState>, _u: AuthUser) -> Result<Json<VixTermStructure>, ApiError> {
    Ok(Json(traderview_db::vol::vix_term_structure(&s.pool).await.map_err(ApiError::Internal)?))
}
async fn yields(State(s): State<AppState>, _u: AuthUser) -> Result<Json<YieldCurve>, ApiError> {
    Ok(Json(traderview_db::vol::yield_curve(&s.pool).await.map_err(ApiError::Internal)?))
}
async fn dollar(State(s): State<AppState>, _u: AuthUser) -> Result<Json<DollarSnapshot>, ApiError> {
    Ok(Json(traderview_db::vol::dollar_snapshot(&s.pool).await.map_err(ApiError::Internal)?))
}
