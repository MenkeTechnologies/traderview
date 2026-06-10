//! Roth vs Traditional 401(k) route.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use traderview_db::roth_vs_trad_401k;

pub fn router() -> Router<AppState> {
    Router::new().route("/roth-vs-trad-401k/compute", post(compute))
}

async fn compute(
    State(_s): State<AppState>,
    _user: AuthUser,
    Json(input): Json<roth_vs_trad_401k::RothVsTradInput>,
) -> Result<Json<roth_vs_trad_401k::RothVsTradReport>, ApiError> {
    if !input.annual_pretax_contribution_usd.is_finite()
        || input.annual_pretax_contribution_usd < 0.0
    {
        return Err(ApiError::BadRequest(
            "annual_pretax_contribution_usd must be ≥ 0".into(),
        ));
    }
    let pcts = [
        ("current_marginal_tax_rate_pct", input.current_marginal_tax_rate_pct),
        ("retirement_marginal_tax_rate_pct", input.retirement_marginal_tax_rate_pct),
        ("ltcg_rate_pct", input.ltcg_rate_pct),
    ];
    for (n, v) in pcts {
        if !v.is_finite() || v < 0.0 || v > 60.0 {
            return Err(ApiError::BadRequest(format!("{n} must be in [0, 60]")));
        }
    }
    if !input.expected_annual_return_pct.is_finite()
        || input.expected_annual_return_pct < -20.0
        || input.expected_annual_return_pct > 30.0
    {
        return Err(ApiError::BadRequest(
            "expected_annual_return_pct must be in [-20, 30]".into(),
        ));
    }
    if input.years_to_retirement == 0 || input.years_to_retirement > 70 {
        return Err(ApiError::BadRequest(
            "years_to_retirement must be in [1, 70]".into(),
        ));
    }
    Ok(Json(roth_vs_trad_401k::compute(&input)))
}
