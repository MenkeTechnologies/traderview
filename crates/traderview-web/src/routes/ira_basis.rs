//! Form 8606 IRA basis + pro-rata rule routes.
//!
//! POST runs the pure-compute `form_8606::compute` then persists the
//! result, auto-pulling prior-year basis from the ledger if the caller
//! didn't supply it. GET/DELETE manage the persisted rows. Mirrors the
//! `/api/tax/carryover` pattern from iter 6.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use rust_decimal::Decimal;
use serde::Deserialize;
use traderview_db::ira_basis::{
    delete as delete_row, get_by_year, list, prior_year_basis, upsert, IraBasisRow, IraBasisUpsert,
};
use traderview_expense::form_8606::{compute as compute_form_8606, Form8606Input, Form8606Result};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/tax/ira-basis",
            get(list_route).post(compute_and_save_route),
        )
        .route("/tax/ira-basis/:year", get(get_route).delete(delete_route))
}

async fn list_route(
    State(s): State<AppState>,
    u: AuthUser,
) -> Result<Json<Vec<IraBasisRow>>, ApiError> {
    Ok(Json(list(&s.pool, u.id).await.map_err(ApiError::Internal)?))
}

async fn get_route(
    State(s): State<AppState>,
    u: AuthUser,
    Path(year): Path<i32>,
) -> Result<Json<IraBasisRow>, ApiError> {
    get_by_year(&s.pool, u.id, year)
        .await
        .map_err(ApiError::Internal)?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

#[derive(Deserialize)]
struct ComputeInput {
    tax_year: i32,
    nondeductible_contributions: Decimal,
    /// If omitted, pulled from the ledger row for tax_year - 1.
    prior_basis: Option<Decimal>,
    year_end_aggregate_value: Decimal,
    distributions_this_year: Decimal,
    conversions_to_roth: Decimal,
}

#[derive(serde::Serialize)]
struct ComputeOutput {
    computed: Form8606Result,
    persisted: IraBasisRow,
}

async fn compute_and_save_route(
    State(s): State<AppState>,
    u: AuthUser,
    Json(b): Json<ComputeInput>,
) -> Result<Json<ComputeOutput>, ApiError> {
    if [
        b.nondeductible_contributions,
        b.year_end_aggregate_value,
        b.distributions_this_year,
        b.conversions_to_roth,
    ]
    .iter()
    .chain(b.prior_basis.iter())
    .any(|x| *x < Decimal::ZERO)
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    let prior = match b.prior_basis {
        Some(v) => v,
        None => prior_year_basis(&s.pool, u.id, b.tax_year)
            .await
            .map_err(ApiError::Internal)?,
    };
    let computed = compute_form_8606(&Form8606Input {
        tax_year: b.tax_year,
        nondeductible_contributions: b.nondeductible_contributions,
        prior_basis: prior,
        year_end_aggregate_value: b.year_end_aggregate_value,
        distributions_this_year: b.distributions_this_year,
        conversions_to_roth: b.conversions_to_roth,
    });
    let persisted = upsert(
        &s.pool,
        u.id,
        &IraBasisUpsert {
            tax_year: b.tax_year,
            nondeductible_contributions: b.nondeductible_contributions,
            prior_basis: prior,
            year_end_aggregate_value: b.year_end_aggregate_value,
            distributions_this_year: b.distributions_this_year,
            conversions_to_roth: b.conversions_to_roth,
            line_3_total_basis_available: computed.line_3_total_basis_available,
            line_9_proration_denominator: computed.line_9_proration_denominator,
            line_10_proration_ratio: computed.line_10_proration_ratio,
            line_11_nontaxable_conversion: computed.line_11_nontaxable_conversion,
            line_12_nontaxable_distribution: computed.line_12_nontaxable_distribution,
            line_13_total_nontaxable: computed.line_13_total_nontaxable,
            line_14_basis_carryover: computed.line_14_basis_carryover,
            line_15c_taxable_distribution: computed.line_15c_taxable_distribution,
            line_18_taxable_conversion: computed.line_18_taxable_conversion,
            total_taxable: computed.total_taxable,
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
