//! Tax bracket optimizer route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::tax_bracket_optimizer;

pub fn router() -> Router<AppState> {
    Router::new().route("/tax-bracket-optimizer/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<tax_bracket_optimizer::TaxBracketInput>,
) -> Result<Json<tax_bracket_optimizer::TaxBracketReport>, ApiError> {
    if !["single", "mfj", "hoh"].contains(&input.filing_status.as_str()) {
        return Err(ApiError::BadRequest(
            "filing_status must be single | mfj | hoh".into(),
        ));
    }
    if !input.taxable_ordinary_income_usd.is_finite()
        || input.taxable_ordinary_income_usd < 0.0
    {
        return Err(ApiError::BadRequest(
            "taxable_ordinary_income_usd must be ≥ 0".into(),
        ));
    }
    Ok(Json(tax_bracket_optimizer::compute(&input)))
}
