//! IRC §1212(b) capital loss carryover ledger routes.
//!
//! POST runs the pure-compute `section_1212::compute` then persists the
//! result, automatically pulling prior-year carryovers from the ledger
//! if the caller didn't supply them. GET / DELETE manage the persisted
//! rows.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use rust_decimal::Decimal;
use serde::Deserialize;
use traderview_db::carryover::{
    delete as delete_row, get_by_year, list, prior_year_carryovers, upsert, CarryoverRow,
    CarryoverUpsert,
};
use traderview_expense::section_1212::{
    compute as compute_carryover, CarryoverInput, CarryoverResult, FilingStatus,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/tax/carryover",
            get(list_route).post(compute_and_save_route),
        )
        .route("/tax/carryover/:year", get(get_route).delete(delete_route))
}

async fn list_route(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<CarryoverRow>>, ApiError> {
    Ok(Json(list(&s.pool, u.id).await.map_err(ApiError::Internal)?))
}

async fn get_route(
    State(s): State<AppState>,
    u: AuthUser,
    Path(year): Path<i32>,
) -> Result<Json<CarryoverRow>, ApiError> {
    get_by_year(&s.pool, u.id, year)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

#[derive(Deserialize)]
struct ComputeInput {
    tax_year: i32,
    st_gains: Decimal,
    st_losses: Decimal,
    lt_gains: Decimal,
    lt_losses: Decimal,
    /// Optional override of prior-year carryovers. If both omitted,
    /// the prior tax_year row is read from the ledger (zero if none).
    prior_st_carryover: Option<Decimal>,
    prior_lt_carryover: Option<Decimal>,
    filing_status: String,
}

#[derive(serde::Serialize)]
struct ComputeOutput {
    computed: CarryoverResult,
    persisted: CarryoverRow,
}

fn parse_filing_status(s: &str) -> Result<FilingStatus, ApiError> {
    Ok(match s {
        "single" => FilingStatus::Single,
        "head_of_household" => FilingStatus::HeadOfHousehold,
        "married_filing_jointly" => FilingStatus::MarriedFilingJointly,
        "married_filing_separately" => FilingStatus::MarriedFilingSeparately,
        _ => return Err(ApiError::BadRequest(format!("invalid filing_status: {s}"))),
    })
}

async fn compute_and_save_route(
    State(s): State<AppState>,
    u: AuthUser,
    Json(b): Json<ComputeInput>,
) -> Result<Json<ComputeOutput>, ApiError> {
    let fs = parse_filing_status(&b.filing_status)?;
    let (prior_st, prior_lt) = match (b.prior_st_carryover, b.prior_lt_carryover) {
        (Some(st), Some(lt)) => (st, lt),
        _ => prior_year_carryovers(&s.pool, u.id, b.tax_year)
            .await
            .map_err(ApiError::Internal)?,
    };
    if [
        b.st_gains,
        b.st_losses,
        b.lt_gains,
        b.lt_losses,
        prior_st,
        prior_lt,
    ]
    .iter()
    .any(|x| *x < Decimal::ZERO)
    {
        return Err(ApiError::BadRequest(
            "gains/losses/carryovers must be >= 0 (losses passed as positive)".into(),
        ));
    }
    let computed = compute_carryover(&CarryoverInput {
        st_gains: b.st_gains,
        st_losses: b.st_losses,
        lt_gains: b.lt_gains,
        lt_losses: b.lt_losses,
        prior_st_carryover: prior_st,
        prior_lt_carryover: prior_lt,
        filing_status: fs,
        tax_year: b.tax_year,
    });
    let persisted = upsert(
        &s.pool,
        u.id,
        &CarryoverUpsert {
            tax_year: b.tax_year,
            filing_status: b.filing_status,
            st_gains_year: b.st_gains,
            st_losses_year: b.st_losses,
            lt_gains_year: b.lt_gains,
            lt_losses_year: b.lt_losses,
            prior_st_carryover: prior_st,
            prior_lt_carryover: prior_lt,
            deductible_against_ordinary: computed.deductible_against_ordinary,
            st_absorbed_by_deduction: computed.st_absorbed_by_deduction,
            lt_absorbed_by_deduction: computed.lt_absorbed_by_deduction,
            st_carryover_next_year: computed.st_carryover_next_year,
            lt_carryover_next_year: computed.lt_carryover_next_year,
            combined_net_gain_loss: computed.combined_net_gain_loss,
            note: computed.note.clone(),
        },
    )
    .await
    .map_err(ApiError::Internal)?;
    Ok(Json(ComputeOutput {
        computed,
        persisted,
    }))
}

async fn delete_route(
    State(s): State<AppState>,
    u: AuthUser,
    Path(year): Path<i32>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !delete_row(&s.pool, u.id, year)
        .await
        .map_err(ApiError::Internal)?
    {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({"deleted": true})))
}
