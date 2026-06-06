//! §1212(b) capital loss carryover persistence.
//!
//! Compute lives in `traderview-expense::section_1212`; this module
//! only persists the result row keyed by `(user_id, tax_year)` so next
//! year's compute can read prior-year ST/LT carryovers without
//! re-running every closed lot.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CarryoverRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tax_year: i32,
    pub filing_status: String,
    pub st_gains_year: Decimal,
    pub st_losses_year: Decimal,
    pub lt_gains_year: Decimal,
    pub lt_losses_year: Decimal,
    pub prior_st_carryover: Decimal,
    pub prior_lt_carryover: Decimal,
    pub deductible_against_ordinary: Decimal,
    pub st_absorbed_by_deduction: Decimal,
    pub lt_absorbed_by_deduction: Decimal,
    pub st_carryover_next_year: Decimal,
    pub lt_carryover_next_year: Decimal,
    pub combined_net_gain_loss: Decimal,
    pub note: String,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CarryoverUpsert {
    pub tax_year: i32,
    pub filing_status: String,
    pub st_gains_year: Decimal,
    pub st_losses_year: Decimal,
    pub lt_gains_year: Decimal,
    pub lt_losses_year: Decimal,
    pub prior_st_carryover: Decimal,
    pub prior_lt_carryover: Decimal,
    pub deductible_against_ordinary: Decimal,
    pub st_absorbed_by_deduction: Decimal,
    pub lt_absorbed_by_deduction: Decimal,
    pub st_carryover_next_year: Decimal,
    pub lt_carryover_next_year: Decimal,
    pub combined_net_gain_loss: Decimal,
    pub note: String,
}

const ROW_COLS: &str = "id, user_id, tax_year, filing_status,
    st_gains_year, st_losses_year, lt_gains_year, lt_losses_year,
    prior_st_carryover, prior_lt_carryover,
    deductible_against_ordinary, st_absorbed_by_deduction,
    lt_absorbed_by_deduction, st_carryover_next_year,
    lt_carryover_next_year, combined_net_gain_loss, note, computed_at";

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<CarryoverRow>> {
    Ok(sqlx::query_as(&format!(
        "SELECT {ROW_COLS} FROM capital_loss_carryovers
          WHERE user_id = $1 ORDER BY tax_year DESC"
    ))
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_by_year(
    pool: &PgPool,
    user_id: Uuid,
    year: i32,
) -> anyhow::Result<Option<CarryoverRow>> {
    Ok(sqlx::query_as(&format!(
        "SELECT {ROW_COLS} FROM capital_loss_carryovers
          WHERE user_id = $1 AND tax_year = $2"
    ))
    .bind(user_id)
    .bind(year)
    .fetch_optional(pool)
    .await?)
}

pub async fn upsert(
    pool: &PgPool,
    user_id: Uuid,
    dto: &CarryoverUpsert,
) -> anyhow::Result<CarryoverRow> {
    // ON CONFLICT (user_id, tax_year) DO UPDATE so re-running the
    // compute idempotently updates the row.
    Ok(sqlx::query_as(&format!(
        "INSERT INTO capital_loss_carryovers
           (user_id, tax_year, filing_status, st_gains_year, st_losses_year,
            lt_gains_year, lt_losses_year, prior_st_carryover, prior_lt_carryover,
            deductible_against_ordinary, st_absorbed_by_deduction,
            lt_absorbed_by_deduction, st_carryover_next_year,
            lt_carryover_next_year, combined_net_gain_loss, note)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
         ON CONFLICT (user_id, tax_year) DO UPDATE SET
            filing_status               = EXCLUDED.filing_status,
            st_gains_year               = EXCLUDED.st_gains_year,
            st_losses_year              = EXCLUDED.st_losses_year,
            lt_gains_year               = EXCLUDED.lt_gains_year,
            lt_losses_year              = EXCLUDED.lt_losses_year,
            prior_st_carryover          = EXCLUDED.prior_st_carryover,
            prior_lt_carryover          = EXCLUDED.prior_lt_carryover,
            deductible_against_ordinary = EXCLUDED.deductible_against_ordinary,
            st_absorbed_by_deduction    = EXCLUDED.st_absorbed_by_deduction,
            lt_absorbed_by_deduction    = EXCLUDED.lt_absorbed_by_deduction,
            st_carryover_next_year      = EXCLUDED.st_carryover_next_year,
            lt_carryover_next_year      = EXCLUDED.lt_carryover_next_year,
            combined_net_gain_loss      = EXCLUDED.combined_net_gain_loss,
            note                        = EXCLUDED.note,
            computed_at                 = now()
         RETURNING {ROW_COLS}"
    ))
    .bind(user_id)
    .bind(dto.tax_year)
    .bind(&dto.filing_status)
    .bind(dto.st_gains_year)
    .bind(dto.st_losses_year)
    .bind(dto.lt_gains_year)
    .bind(dto.lt_losses_year)
    .bind(dto.prior_st_carryover)
    .bind(dto.prior_lt_carryover)
    .bind(dto.deductible_against_ordinary)
    .bind(dto.st_absorbed_by_deduction)
    .bind(dto.lt_absorbed_by_deduction)
    .bind(dto.st_carryover_next_year)
    .bind(dto.lt_carryover_next_year)
    .bind(dto.combined_net_gain_loss)
    .bind(&dto.note)
    .fetch_one(pool)
    .await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, year: i32) -> anyhow::Result<bool> {
    Ok(
        sqlx::query("DELETE FROM capital_loss_carryovers WHERE user_id = $1 AND tax_year = $2")
            .bind(user_id)
            .bind(year)
            .execute(pool)
            .await?
            .rows_affected()
            > 0,
    )
}

/// Read just the prior-year ST/LT carryovers for use as inputs to next
/// year's compute. Returns (0, 0) if no prior row.
pub async fn prior_year_carryovers(
    pool: &PgPool,
    user_id: Uuid,
    current_year: i32,
) -> anyhow::Result<(Decimal, Decimal)> {
    let row: Option<(Decimal, Decimal)> = sqlx::query_as(
        "SELECT st_carryover_next_year, lt_carryover_next_year
           FROM capital_loss_carryovers
          WHERE user_id = $1 AND tax_year = $2",
    )
    .bind(user_id)
    .bind(current_year - 1)
    .fetch_optional(pool)
    .await?;
    Ok(row.unwrap_or((Decimal::ZERO, Decimal::ZERO)))
}
