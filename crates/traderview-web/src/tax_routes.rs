//! Tax-dashboard helpers — quarterly estimated-tax payments (CRUD) +
//! expense-category kind toggling (income vs expense).

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/estimated-payments", get(list_payments).post(create_payment))
        .route("/estimated-payments/:id", patch(update_payment).delete(delete_payment))
        .route("/categories/:id/kind", patch(set_category_kind))
        .route("/purchases", get(list_purchases))
        .route("/monthly-totals", get(monthly_totals))
        .route("/yoy", get(yoy_trend))
}

// --- estimated tax payments --------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct EstimatedPayment {
    id: Uuid,
    user_id: Uuid,
    tax_year: i32,
    quarter: i16,
    paid_at: NaiveDate,
    amount: Decimal,
    method: String,
    note: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct CreatePayment {
    tax_year: i32,
    quarter: i16,
    paid_at: NaiveDate,
    amount: Decimal,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Deserialize)]
struct UpdatePayment {
    #[serde(default)]
    tax_year: Option<i32>,
    #[serde(default)]
    quarter: Option<i16>,
    #[serde(default)]
    paid_at: Option<NaiveDate>,
    #[serde(default)]
    amount: Option<Decimal>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Deserialize)]
struct ListQ {
    #[serde(default)]
    tax_year: Option<i32>,
}

async fn list_payments(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<ListQ>,
) -> Result<Json<Vec<EstimatedPayment>>, ApiError> {
    let rows: Vec<EstimatedPayment> = sqlx::query_as(
        "SELECT id, user_id, tax_year, quarter, paid_at, amount,
                method, note, created_at
           FROM estimated_tax_payments
          WHERE user_id = $1
            AND ($2::int IS NULL OR tax_year = $2)
          ORDER BY tax_year DESC, quarter ASC, paid_at ASC",
    )
    .bind(user.id)
    .bind(q.tax_year)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_payment(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreatePayment>,
) -> Result<Json<EstimatedPayment>, ApiError> {
    if !(1..=4).contains(&body.quarter) {
        return Err(ApiError::BadRequest("quarter must be 1-4".into()));
    }
    if body.amount <= Decimal::ZERO {
        return Err(ApiError::BadRequest("amount must be positive".into()));
    }
    let row: EstimatedPayment = sqlx::query_as(
        "INSERT INTO estimated_tax_payments
            (user_id, tax_year, quarter, paid_at, amount, method, note)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, user_id, tax_year, quarter, paid_at, amount,
                   method, note, created_at",
    )
    .bind(user.id)
    .bind(body.tax_year)
    .bind(body.quarter)
    .bind(body.paid_at)
    .bind(body.amount)
    .bind(body.method.unwrap_or_default())
    .bind(body.note.unwrap_or_default())
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row))
}

async fn update_payment(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePayment>,
) -> Result<Json<EstimatedPayment>, ApiError> {
    let row: Option<EstimatedPayment> = sqlx::query_as(
        "UPDATE estimated_tax_payments SET
            tax_year = COALESCE($3, tax_year),
            quarter  = COALESCE($4, quarter),
            paid_at  = COALESCE($5, paid_at),
            amount   = COALESCE($6, amount),
            method   = COALESCE($7, method),
            note     = COALESCE($8, note)
          WHERE id = $1 AND user_id = $2
         RETURNING id, user_id, tax_year, quarter, paid_at, amount,
                   method, note, created_at",
    )
    .bind(id)
    .bind(user.id)
    .bind(body.tax_year)
    .bind(body.quarter)
    .bind(body.paid_at)
    .bind(body.amount)
    .bind(body.method)
    .bind(body.note)
    .fetch_optional(&s.pool)
    .await?;
    let row = row.ok_or(ApiError::NotFound)?;
    Ok(Json(row))
}

async fn delete_payment(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    let n = sqlx::query(
        "DELETE FROM estimated_tax_payments WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user.id)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// --- category kind (income/expense) ------------------------------------

#[derive(Deserialize)]
struct KindBody {
    kind: String,
}

// --- unified purchases (receipt items ∪ CSV transactions) -------------
//
// One row per "purchase" — i.e., either a line item extracted from a
// receipt's OCR or a CSV/PDF-imported transaction with no attached
// receipt detail. Receipt-attached transactions are NOT emitted as
// standalone rows because their item-level rows already represent the
// same money; emitting them again would double-count.
//
// Both halves carry `receipt_id` + `transaction_id` (one or both
// nullable) so the UI can render clickable drill-downs back to either
// source.

#[derive(Deserialize)]
struct PurchasesQ {
    #[serde(default)] from: Option<NaiveDate>,
    #[serde(default)] to: Option<NaiveDate>,
    #[serde(default)] category: Option<String>,
    #[serde(default)] tax_bucket: Option<String>,
    #[serde(default)] min_total: Option<Decimal>,
    #[serde(default)] max_total: Option<Decimal>,
    #[serde(default)] search: Option<String>,
    #[serde(default)] offset: Option<i64>,
    #[serde(default)] limit: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
struct PurchaseRow {
    source: String,                 // "receipt_item" | "transaction"
    date: Option<NaiveDate>,
    name: String,
    qty: Option<Decimal>,
    unit_price: Option<Decimal>,
    total: Option<Decimal>,
    category: Option<String>,
    tax_bucket: Option<String>,
    rental_property_id: Option<String>,
    receipt_id: Option<Uuid>,
    transaction_id: Option<Uuid>,
    merchant: Option<String>,
}

#[derive(Serialize)]
struct PurchasesResponse {
    rows: Vec<PurchaseRow>,
    total: i64,
    offset: i64,
    limit: i64,
}

async fn list_purchases(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<PurchasesQ>,
) -> Result<Json<PurchasesResponse>, ApiError> {
    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let offset = q.offset.unwrap_or(0).max(0);
    let search_pat = q.search.as_ref().map(|s| format!("%{}%", s));

    // Single SQL — UNION ALL the two halves, then filter + paginate at
    // the outer level so the WHERE clauses apply to both consistently.
    // We use jsonb_array_elements_text on the items array; numeric
    // fields are stored as strings inside the JSONB per serde-decimal.
    let rows: Vec<PurchaseRow> = sqlx::query_as(
        r#"
        WITH unified AS (
            SELECT
                'receipt_item'::text AS source,
                r.ocr_date AS date,
                COALESCE(item->>'name', '') AS name,
                NULLIF(item->>'qty', '')::numeric AS qty,
                NULLIF(item->>'unit_price', '')::numeric AS unit_price,
                NULLIF(item->>'line_total', '')::numeric AS total,
                item->>'category' AS category,
                item->>'tax_bucket' AS tax_bucket,
                item->>'rental_property_id' AS rental_property_id,
                r.id AS receipt_id,
                r.transaction_id AS transaction_id,
                r.ocr_merchant AS merchant
            FROM receipts r,
                 jsonb_array_elements(
                     COALESCE(r.ocr_extracted->'items', '[]'::jsonb)
                 ) AS item
            WHERE r.user_id = $1
              AND r.ocr_status = 'done'::ocr_status_t

            UNION ALL

            SELECT
                'transaction'::text AS source,
                t.posted_at::date AS date,
                COALESCE(t.merchant_raw, t.merchant_normalized, '(unknown)') AS name,
                NULL::numeric AS qty,
                NULL::numeric AS unit_price,
                t.amount AS total,
                c.name AS category,
                CASE
                    WHEN t.is_business = TRUE  THEN 'business'
                    WHEN t.is_business = FALSE THEN 'personal'
                    ELSE NULL
                END AS tax_bucket,
                NULL::text AS rental_property_id,
                NULL::uuid AS receipt_id,
                t.id AS transaction_id,
                t.merchant_raw AS merchant
            FROM transactions t
            JOIN financial_accounts a ON a.id = t.account_id
            LEFT JOIN expense_categories c ON c.id = t.category_id
            WHERE a.user_id = $1
              AND NOT EXISTS (
                  SELECT 1 FROM receipts rr
                   WHERE rr.transaction_id = t.id
                     AND rr.ocr_status = 'done'::ocr_status_t
                     AND jsonb_array_length(
                         COALESCE(rr.ocr_extracted->'items', '[]'::jsonb)
                     ) > 0
              )
        )
        SELECT source, date, name, qty, unit_price, total,
               category, tax_bucket, rental_property_id,
               receipt_id, transaction_id, merchant
          FROM unified
         WHERE ($2::date IS NULL OR date >= $2)
           AND ($3::date IS NULL OR date <= $3)
           AND ($4::text IS NULL OR category = $4)
           AND ($5::text IS NULL OR tax_bucket = $5)
           AND ($6::numeric IS NULL OR total >= $6)
           AND ($7::numeric IS NULL OR total <= $7)
           AND ($8::text IS NULL OR name ILIKE $8 OR merchant ILIKE $8)
         ORDER BY date DESC NULLS LAST, name ASC
         LIMIT $9 OFFSET $10
        "#,
    )
    .bind(user.id)
    .bind(q.from)
    .bind(q.to)
    .bind(&q.category)
    .bind(&q.tax_bucket)
    .bind(q.min_total)
    .bind(q.max_total)
    .bind(&search_pat)
    .bind(limit)
    .bind(offset)
    .fetch_all(&s.pool)
    .await?;

    // Total count for pagination. Same CTE; just COUNT.
    let total: i64 = sqlx::query_scalar(
        r#"
        WITH unified AS (
            SELECT r.ocr_date AS date, item->>'category' AS category,
                   item->>'tax_bucket' AS tax_bucket,
                   NULLIF(item->>'line_total', '')::numeric AS total,
                   COALESCE(item->>'name', '') AS name,
                   r.ocr_merchant AS merchant
            FROM receipts r,
                 jsonb_array_elements(
                     COALESCE(r.ocr_extracted->'items', '[]'::jsonb)
                 ) AS item
            WHERE r.user_id = $1 AND r.ocr_status = 'done'::ocr_status_t
            UNION ALL
            SELECT t.posted_at::date AS date,
                   c.name AS category,
                   CASE WHEN t.is_business = TRUE THEN 'business'
                        WHEN t.is_business = FALSE THEN 'personal'
                        ELSE NULL END AS tax_bucket,
                   t.amount AS total,
                   COALESCE(t.merchant_raw, t.merchant_normalized, '(unknown)') AS name,
                   t.merchant_raw AS merchant
            FROM transactions t
            JOIN financial_accounts a ON a.id = t.account_id
            LEFT JOIN expense_categories c ON c.id = t.category_id
            WHERE a.user_id = $1
              AND NOT EXISTS (
                  SELECT 1 FROM receipts rr
                   WHERE rr.transaction_id = t.id
                     AND rr.ocr_status = 'done'::ocr_status_t
                     AND jsonb_array_length(
                         COALESCE(rr.ocr_extracted->'items', '[]'::jsonb)
                     ) > 0
              )
        )
        SELECT COUNT(*)::bigint FROM unified
         WHERE ($2::date IS NULL OR date >= $2)
           AND ($3::date IS NULL OR date <= $3)
           AND ($4::text IS NULL OR category = $4)
           AND ($5::text IS NULL OR tax_bucket = $5)
           AND ($6::numeric IS NULL OR total >= $6)
           AND ($7::numeric IS NULL OR total <= $7)
           AND ($8::text IS NULL OR name ILIKE $8 OR merchant ILIKE $8)
        "#,
    )
    .bind(user.id)
    .bind(q.from)
    .bind(q.to)
    .bind(&q.category)
    .bind(&q.tax_bucket)
    .bind(q.min_total)
    .bind(q.max_total)
    .bind(&search_pat)
    .fetch_one(&s.pool)
    .await
    .unwrap_or(0);

    Ok(Json(PurchasesResponse {
        rows,
        total,
        offset,
        limit,
    }))
}

// --- monthly totals (for the bar chart) -------------------------------
//
// Returns 12 rows for the requested year, each carrying the sum of
// receipt-item line_totals grouped by tax_bucket. Computed entirely in
// SQL via jsonb_array_elements + EXTRACT(MONTH FROM ocr_date). Months
// with no receipts return zeros so the bar chart always has all 12
// bars (visual continuity).

#[derive(Deserialize)]
struct YearQ {
    year: Option<i32>,
}

#[derive(Serialize, sqlx::FromRow)]
struct MonthlyRow {
    month: i32,
    business: Decimal,
    rental: Decimal,
    personal: Decimal,
    unclassified: Decimal,
}

async fn monthly_totals(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<YearQ>,
) -> Result<Json<Vec<MonthlyRow>>, ApiError> {
    let year = q.year.unwrap_or_else(|| chrono::Utc::now().date_naive().year() as i32);
    // Generate the 12-row series first (so months with zero spend
    // still show as bars), then LEFT JOIN the aggregated JSONB sums.
    let rows: Vec<MonthlyRow> = sqlx::query_as(
        r#"
        WITH months AS (
            SELECT generate_series(1, 12) AS month
        ),
        unnested AS (
            SELECT
                EXTRACT(MONTH FROM r.ocr_date)::int AS month,
                item->>'tax_bucket' AS bucket,
                NULLIF(item->>'line_total', '')::numeric AS total
            FROM receipts r,
                 jsonb_array_elements(
                     COALESCE(r.ocr_extracted->'items', '[]'::jsonb)
                 ) AS item
            WHERE r.user_id = $1
              AND r.ocr_status = 'done'::ocr_status_t
              AND r.ocr_date IS NOT NULL
              AND EXTRACT(YEAR FROM r.ocr_date)::int = $2
        ),
        agg AS (
            SELECT month,
                   COALESCE(SUM(total) FILTER (WHERE bucket = 'business'), 0)     AS business,
                   COALESCE(SUM(total) FILTER (WHERE bucket = 'rental'), 0)       AS rental,
                   COALESCE(SUM(total) FILTER (WHERE bucket = 'personal'), 0)     AS personal,
                   COALESCE(SUM(total) FILTER (WHERE bucket = 'unclassified'), 0) AS unclassified
              FROM unnested
             GROUP BY month
        )
        SELECT m.month,
               COALESCE(a.business, 0)     AS business,
               COALESCE(a.rental, 0)       AS rental,
               COALESCE(a.personal, 0)     AS personal,
               COALESCE(a.unclassified, 0) AS unclassified
          FROM months m
          LEFT JOIN agg a ON a.month = m.month
         ORDER BY m.month
        "#,
    )
    .bind(user.id)
    .bind(year)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

// --- year-over-year totals (for the trend line) ------------------------
//
// Returns up to N most-recent years' annual bucket totals, oldest
// first. Drives a multi-series line chart.

#[derive(Deserialize)]
struct YoyQ {
    years: Option<u8>,
}

#[derive(Serialize, sqlx::FromRow)]
struct YearRow {
    year: i32,
    business: Decimal,
    rental: Decimal,
    personal: Decimal,
    unclassified: Decimal,
}

async fn yoy_trend(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<YoyQ>,
) -> Result<Json<Vec<YearRow>>, ApiError> {
    let n = q.years.unwrap_or(5).clamp(1, 10) as i32;
    let current_year = chrono::Utc::now().date_naive().year() as i32;
    let from_year = current_year - n + 1;
    let rows: Vec<YearRow> = sqlx::query_as(
        r#"
        WITH years AS (
            SELECT generate_series($2::int, $3::int) AS year
        ),
        unnested AS (
            SELECT
                EXTRACT(YEAR FROM r.ocr_date)::int AS year,
                item->>'tax_bucket' AS bucket,
                NULLIF(item->>'line_total', '')::numeric AS total
            FROM receipts r,
                 jsonb_array_elements(
                     COALESCE(r.ocr_extracted->'items', '[]'::jsonb)
                 ) AS item
            WHERE r.user_id = $1
              AND r.ocr_status = 'done'::ocr_status_t
              AND r.ocr_date IS NOT NULL
              AND EXTRACT(YEAR FROM r.ocr_date)::int BETWEEN $2 AND $3
        ),
        agg AS (
            SELECT year,
                   COALESCE(SUM(total) FILTER (WHERE bucket = 'business'), 0)     AS business,
                   COALESCE(SUM(total) FILTER (WHERE bucket = 'rental'), 0)       AS rental,
                   COALESCE(SUM(total) FILTER (WHERE bucket = 'personal'), 0)     AS personal,
                   COALESCE(SUM(total) FILTER (WHERE bucket = 'unclassified'), 0) AS unclassified
              FROM unnested
             GROUP BY year
        )
        SELECT y.year,
               COALESCE(a.business, 0)     AS business,
               COALESCE(a.rental, 0)       AS rental,
               COALESCE(a.personal, 0)     AS personal,
               COALESCE(a.unclassified, 0) AS unclassified
          FROM years y
          LEFT JOIN agg a ON a.year = y.year
         ORDER BY y.year ASC
        "#,
    )
    .bind(user.id)
    .bind(from_year)
    .bind(current_year)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn set_category_kind(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<KindBody>,
) -> Result<axum::http::StatusCode, ApiError> {
    if !matches!(body.kind.as_str(), "income" | "expense") {
        return Err(ApiError::BadRequest(
            "kind must be 'income' or 'expense'".into(),
        ));
    }
    let n = sqlx::query(
        "UPDATE expense_categories SET kind = $3
          WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user.id)
    .bind(&body.kind)
    .execute(&s.pool)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}
