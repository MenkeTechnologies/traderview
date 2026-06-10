//! Social Security age route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::social_security_age;

pub fn router() -> Router<AppState> {
    Router::new().route("/social-security-age/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<social_security_age::SocialSecurityInput>,
) -> Result<Json<social_security_age::SocialSecurityReport>, ApiError> {
    if !input.fra_monthly_benefit_usd.is_finite() || input.fra_monthly_benefit_usd < 0.0 {
        return Err(ApiError::BadRequest(
            "fra_monthly_benefit_usd must be ≥ 0".into(),
        ));
    }
    if input.fra_age < 62 || input.fra_age > 70 {
        return Err(ApiError::BadRequest("fra_age must be in [62, 70]".into()));
    }
    for (n, age) in [("claim_age_a", input.claim_age_a), ("claim_age_b", input.claim_age_b)] {
        if age < 62 || age > 75 {
            return Err(ApiError::BadRequest(format!("{n} must be in [62, 75]")));
        }
    }
    if input.life_expectancy_age < 62 || input.life_expectancy_age > 120 {
        return Err(ApiError::BadRequest(
            "life_expectancy_age must be in [62, 120]".into(),
        ));
    }
    Ok(Json(social_security_age::compute(&input)))
}
