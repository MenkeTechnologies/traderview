//! IRS Form 8606 nondeductible IRA basis persistence.
//!
//! Multi-year ledger keyed by `(user_id, tax_year)` mirroring the
//! §1212(b) carryover module's shape. Lets the API endpoint pull
//! prior-year basis (line 14) without rerunning every prior year's
//! compute.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct IraBasisRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tax_year: i32,
    pub nondeductible_contributions: Decimal,
    pub prior_basis: Decimal,
    pub year_end_aggregate_value: Decimal,
    pub distributions_this_year: Decimal,
    pub conversions_to_roth: Decimal,
    pub line_3_total_basis_available: Decimal,
    pub line_9_proration_denominator: Decimal,
    pub line_10_proration_ratio: Decimal,
    pub line_11_nontaxable_conversion: Decimal,
    pub line_12_nontaxable_distribution: Decimal,
    pub line_13_total_nontaxable: Decimal,
    pub line_14_basis_carryover: Decimal,
    pub line_15c_taxable_distribution: Decimal,
    pub line_18_taxable_conversion: Decimal,
    pub total_taxable: Decimal,
    pub note: String,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IraBasisUpsert {
    pub tax_year: i32,
    pub nondeductible_contributions: Decimal,
    pub prior_basis: Decimal,
    pub year_end_aggregate_value: Decimal,
    pub distributions_this_year: Decimal,
    pub conversions_to_roth: Decimal,
    pub line_3_total_basis_available: Decimal,
    pub line_9_proration_denominator: Decimal,
    pub line_10_proration_ratio: Decimal,
    pub line_11_nontaxable_conversion: Decimal,
    pub line_12_nontaxable_distribution: Decimal,
    pub line_13_total_nontaxable: Decimal,
    pub line_14_basis_carryover: Decimal,
    pub line_15c_taxable_distribution: Decimal,
    pub line_18_taxable_conversion: Decimal,
    pub total_taxable: Decimal,
    pub note: String,
}

const ROW_COLS: &str = "id, user_id, tax_year,
    nondeductible_contributions, prior_basis, year_end_aggregate_value,
    distributions_this_year, conversions_to_roth,
    line_3_total_basis_available, line_9_proration_denominator,
    line_10_proration_ratio, line_11_nontaxable_conversion,
    line_12_nontaxable_distribution, line_13_total_nontaxable,
    line_14_basis_carryover, line_15c_taxable_distribution,
    line_18_taxable_conversion, total_taxable, note, computed_at";

pub async fn list(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<IraBasisRow>> {
    Ok(sqlx::query_as(&format!(
        "SELECT {ROW_COLS} FROM ira_basis_history
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
) -> anyhow::Result<Option<IraBasisRow>> {
    Ok(sqlx::query_as(&format!(
        "SELECT {ROW_COLS} FROM ira_basis_history
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
    dto: &IraBasisUpsert,
) -> anyhow::Result<IraBasisRow> {
    Ok(sqlx::query_as(&format!(
        "INSERT INTO ira_basis_history
           (user_id, tax_year, nondeductible_contributions, prior_basis,
            year_end_aggregate_value, distributions_this_year, conversions_to_roth,
            line_3_total_basis_available, line_9_proration_denominator,
            line_10_proration_ratio, line_11_nontaxable_conversion,
            line_12_nontaxable_distribution, line_13_total_nontaxable,
            line_14_basis_carryover, line_15c_taxable_distribution,
            line_18_taxable_conversion, total_taxable, note)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                 $15, $16, $17, $18)
         ON CONFLICT (user_id, tax_year) DO UPDATE SET
            nondeductible_contributions     = EXCLUDED.nondeductible_contributions,
            prior_basis                     = EXCLUDED.prior_basis,
            year_end_aggregate_value        = EXCLUDED.year_end_aggregate_value,
            distributions_this_year         = EXCLUDED.distributions_this_year,
            conversions_to_roth             = EXCLUDED.conversions_to_roth,
            line_3_total_basis_available    = EXCLUDED.line_3_total_basis_available,
            line_9_proration_denominator    = EXCLUDED.line_9_proration_denominator,
            line_10_proration_ratio         = EXCLUDED.line_10_proration_ratio,
            line_11_nontaxable_conversion   = EXCLUDED.line_11_nontaxable_conversion,
            line_12_nontaxable_distribution = EXCLUDED.line_12_nontaxable_distribution,
            line_13_total_nontaxable        = EXCLUDED.line_13_total_nontaxable,
            line_14_basis_carryover         = EXCLUDED.line_14_basis_carryover,
            line_15c_taxable_distribution   = EXCLUDED.line_15c_taxable_distribution,
            line_18_taxable_conversion      = EXCLUDED.line_18_taxable_conversion,
            total_taxable                   = EXCLUDED.total_taxable,
            note                            = EXCLUDED.note,
            computed_at                     = now()
         RETURNING {ROW_COLS}"
    ))
    .bind(user_id)
    .bind(dto.tax_year)
    .bind(dto.nondeductible_contributions)
    .bind(dto.prior_basis)
    .bind(dto.year_end_aggregate_value)
    .bind(dto.distributions_this_year)
    .bind(dto.conversions_to_roth)
    .bind(dto.line_3_total_basis_available)
    .bind(dto.line_9_proration_denominator)
    .bind(dto.line_10_proration_ratio)
    .bind(dto.line_11_nontaxable_conversion)
    .bind(dto.line_12_nontaxable_distribution)
    .bind(dto.line_13_total_nontaxable)
    .bind(dto.line_14_basis_carryover)
    .bind(dto.line_15c_taxable_distribution)
    .bind(dto.line_18_taxable_conversion)
    .bind(dto.total_taxable)
    .bind(&dto.note)
    .fetch_one(pool)
    .await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, year: i32) -> anyhow::Result<bool> {
    Ok(
        sqlx::query("DELETE FROM ira_basis_history WHERE user_id = $1 AND tax_year = $2")
            .bind(user_id)
            .bind(year)
            .execute(pool)
            .await?
            .rows_affected()
            > 0,
    )
}

/// Pull prior-year line 14 basis carryover for the chained compute.
pub async fn prior_year_basis(
    pool: &PgPool,
    user_id: Uuid,
    current_year: i32,
) -> anyhow::Result<Decimal> {
    let row: Option<(Decimal,)> = sqlx::query_as(
        "SELECT line_14_basis_carryover FROM ira_basis_history
          WHERE user_id = $1 AND tax_year = $2",
    )
    .bind(user_id)
    .bind(current_year - 1)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(v,)| v).unwrap_or(Decimal::ZERO))
}
