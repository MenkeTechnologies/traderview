//! Tax-wizard backend (`#file-taxes` view).
//!
//! Endpoints:
//!   GET    /api/tax-filing/returns/:year                 — load draft
//!   PUT    /api/tax-filing/returns/:year                 — autosave
//!   POST   /api/tax-filing/returns/:year/autopopulate    — pull Sch C / E from existing data
//!   GET    /api/tax-filing/returns/:year/compute         — run engine, return TaxResult
//!   POST   /api/tax-filing/forms/upload                  — OCR a W-2/1099, persist + return extract
//!   GET    /api/tax-filing/forms/:year                   — list prior W-2/1099 imports
//!   GET    /api/tax-filing/returns/:year/pdf             — generate filled PDF (P6, in pdf module)

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Multipart, Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{Datelike, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use traderview_tax::{compute as compute_tax, TaxReturn, TaxResult};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/returns/:year", get(get_return).put(put_return))
        .route("/returns/:year/autopopulate", post(autopopulate))
        .route("/returns/:year/compute", get(compute_endpoint))
        .route("/returns/:year/pdf", get(crate::tax_pdf::generate_pdf))
        .route("/forms/upload", post(upload_form))
        .route("/forms/:year", get(list_forms))
}

// ── helpers ────────────────────────────────────────────────────────────

/// Load (or lazily create) the draft `tax_returns` row for `(user, year)`.
/// Lazy creation keeps the wizard's first-time-open path a single read.
async fn load_or_create(
    s: &AppState,
    user_id: Uuid,
    year: i32,
) -> Result<(Uuid, TaxReturn), ApiError> {
    let row: Option<(Uuid, serde_json::Value)> = sqlx::query_as(
        "SELECT id, data FROM tax_returns WHERE user_id = $1 AND tax_year = $2",
    )
    .bind(user_id)
    .bind(year)
    .fetch_optional(&s.pool)
    .await?;

    if let Some((id, data)) = row {
        let draft = serde_json::from_value::<TaxReturn>(data).unwrap_or_default();
        return Ok((id, draft));
    }

    let mut draft = TaxReturn::default();
    draft.tax_year = year;
    let payload = serde_json::to_value(&draft).unwrap();
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO tax_returns (user_id, tax_year, data) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(user_id)
    .bind(year)
    .bind(&payload)
    .fetch_one(&s.pool)
    .await?;
    Ok((id, draft))
}

async fn save_return(
    s: &AppState,
    id: Uuid,
    draft: &TaxReturn,
    result: &TaxResult,
    label: &str,
) -> Result<(), ApiError> {
    let data = serde_json::to_value(draft).unwrap();
    sqlx::query(
        "UPDATE tax_returns
            SET data = $2,
                refund_due = $3,
                tax_owed = $4,
                agi = $5,
                updated_at = NOW()
          WHERE id = $1",
    )
    .bind(id)
    .bind(&data)
    .bind(result.refund_due)
    .bind(result.tax_owed)
    .bind(result.agi)
    .execute(&s.pool)
    .await?;
    // Append revision history (append-only, manual cleanup).
    sqlx::query(
        "INSERT INTO tax_return_revisions (tax_return_id, data, change_label)
            VALUES ($1, $2, $3)",
    )
    .bind(id)
    .bind(&data)
    .bind(label)
    .execute(&s.pool)
    .await?;
    Ok(())
}

// ── routes ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ReturnView {
    id: Uuid,
    tax_year: i32,
    status: String,
    draft: TaxReturn,
    result: TaxResult,
}

async fn get_return(
    State(s): State<AppState>,
    user: AuthUser,
    Path(year): Path<i32>,
) -> Result<Json<ReturnView>, ApiError> {
    let (id, draft) = load_or_create(&s, user.id, year).await?;
    let result = compute_tax(&draft);
    let status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM tax_returns WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&s.pool)
    .await?;
    Ok(Json(ReturnView {
        id,
        tax_year: year,
        status: status.unwrap_or_else(|| "personal".into()),
        draft,
        result,
    }))
}

#[derive(Deserialize)]
struct PutReturnBody {
    draft: TaxReturn,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    change_label: Option<String>,
}

async fn put_return(
    State(s): State<AppState>,
    user: AuthUser,
    Path(year): Path<i32>,
    Json(body): Json<PutReturnBody>,
) -> Result<Json<ReturnView>, ApiError> {
    let (id, _) = load_or_create(&s, user.id, year).await?;
    let mut draft = body.draft;
    draft.tax_year = year;
    let result = compute_tax(&draft);
    let label = body.change_label.unwrap_or_else(|| "autosave".into());
    save_return(&s, id, &draft, &result, &label).await?;
    if let Some(status) = body.status.as_ref() {
        sqlx::query("UPDATE tax_returns SET status = $2 WHERE id = $1")
            .bind(id)
            .bind(status)
            .execute(&s.pool)
            .await?;
    }
    let status = body.status.unwrap_or_else(|| "personal".into());
    Ok(Json(ReturnView { id, tax_year: year, status, draft, result }))
}

async fn compute_endpoint(
    State(s): State<AppState>,
    user: AuthUser,
    Path(year): Path<i32>,
) -> Result<Json<TaxResult>, ApiError> {
    let (_, draft) = load_or_create(&s, user.id, year).await?;
    Ok(Json(compute_tax(&draft)))
}

// ── Auto-populate from existing data ───────────────────────────────────

/// Pull Schedule C totals from receipts (items with tax_bucket='business')
/// + Schedule E totals from rental_properties (or items with
/// tax_bucket='rental') for the given year. Overlays them into the
/// current draft and returns the new draft + computed result.
async fn autopopulate(
    State(s): State<AppState>,
    user: AuthUser,
    Path(year): Path<i32>,
) -> Result<Json<ReturnView>, ApiError> {
    let (id, mut draft) = load_or_create(&s, user.id, year).await?;

    // Schedule C — sum of business-tagged line totals across receipts
    // for the year. Negative items (returns) net out automatically.
    let sched_c_row: Option<(Option<Decimal>,)> = sqlx::query_as(
        "WITH items AS (
            SELECT jsonb_array_elements(ocr_extracted->'items') AS item,
                   ocr_date
              FROM receipts
             WHERE user_id = $1
               AND ocr_status = 'done'::ocr_status_t
               AND EXTRACT(YEAR FROM ocr_date) = $2
         )
         SELECT SUM((item->>'line_total')::numeric)
           FROM items
          WHERE item->>'tax_bucket' = 'business'",
    )
    .bind(user.id)
    .bind(year)
    .fetch_optional(&s.pool)
    .await?;
    let sched_c_expenses: Decimal = sched_c_row
        .and_then(|(d,)| d)
        .unwrap_or(Decimal::ZERO);

    // Gross receipts come from positive-amount transactions
    // (or a future "self-employment income" feed). For now, leave
    // the user's existing gross_receipts in place if they've already
    // entered it; otherwise zero (the wizard prompts to enter it).
    if draft.schedule_c.gross_receipts == Decimal::ZERO {
        // Pull positive-amount transactions for the year from income-kind
        // categories (placeholder — the user can refine in the wizard).
        let income_row: Option<(Option<Decimal>,)> = sqlx::query_as(
            "SELECT COALESCE(SUM(t.amount), 0)
               FROM transactions t
               JOIN financial_accounts a ON a.id = t.account_id
              WHERE a.user_id = $1
                AND EXTRACT(YEAR FROM t.posted_at) = $2
                AND t.amount > 0
                AND t.is_transfer = FALSE",
        )
        .bind(user.id)
        .bind(year)
        .fetch_optional(&s.pool)
        .await
        .unwrap_or(None);
        if let Some((Some(income),)) = income_row {
            draft.schedule_c.gross_receipts = income;
        }
    }
    draft.schedule_c.total_expenses = sched_c_expenses;
    draft.schedule_c.net_profit =
        (draft.schedule_c.gross_receipts - draft.schedule_c.total_expenses).max(Decimal::ZERO);

    // Schedule E — sum of rental-tagged expenses + gross rents from
    // rental_properties (if the rental module is populated).
    let sched_e_row: Option<(Option<Decimal>,)> = sqlx::query_as(
        "WITH items AS (
            SELECT jsonb_array_elements(ocr_extracted->'items') AS item,
                   ocr_date
              FROM receipts
             WHERE user_id = $1
               AND ocr_status = 'done'::ocr_status_t
               AND EXTRACT(YEAR FROM ocr_date) = $2
         )
         SELECT SUM((item->>'line_total')::numeric)
           FROM items
          WHERE item->>'tax_bucket' = 'rental'",
    )
    .bind(user.id)
    .bind(year)
    .fetch_optional(&s.pool)
    .await?;
    let sched_e_expenses: Decimal = sched_e_row
        .and_then(|(d,)| d)
        .unwrap_or(Decimal::ZERO);
    draft.schedule_e.total_expenses = sched_e_expenses;
    draft.schedule_e.net_income =
        draft.schedule_e.gross_rents - draft.schedule_e.total_expenses;

    // Estimated tax payments — sum across the year.
    let est_pay: Option<(Option<Decimal>,)> = sqlx::query_as(
        "SELECT COALESCE(SUM(amount), 0)
           FROM estimated_tax_payments
          WHERE user_id = $1 AND tax_year = $2",
    )
    .bind(user.id)
    .bind(year)
    .fetch_optional(&s.pool)
    .await
    .unwrap_or(None);
    if let Some((Some(p),)) = est_pay {
        draft.estimated_tax_payments = p;
    }

    let result = compute_tax(&draft);
    save_return(&s, id, &draft, &result, "autopopulate").await?;
    Ok(Json(ReturnView {
        id, tax_year: year, status: "income".into(), draft, result,
    }))
}

// ── W-2 / 1099 OCR upload ──────────────────────────────────────────────

#[derive(Serialize)]
struct FormUploadResult {
    id: Uuid,
    kind: String,
    payload: serde_json::Value,
    party_name: Option<String>,
    confidence: f32,
    tax_year: i32,
}

async fn upload_form(
    State(s): State<AppState>,
    user: AuthUser,
    mut mp: Multipart,
) -> Result<Json<FormUploadResult>, ApiError> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut mime: String = String::new();
    let mut tax_year: i32 = (Utc::now().year() - 1) as i32; // default: last tax year

    while let Some(field) = mp.next_field().await.map_err(|e| ApiError::BadRequest(e.to_string()))? {
        match field.name() {
            Some("file") => {
                mime = field.content_type().unwrap_or("application/octet-stream").to_string();
                bytes = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("read upload: {e}")))?
                    .to_vec();
            }
            Some("tax_year") => {
                let raw = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(e.to_string()))?;
                if let Ok(y) = raw.parse::<i32>() {
                    tax_year = y;
                }
            }
            _ => {}
        }
    }
    if bytes.is_empty() {
        return Err(ApiError::BadRequest("no file uploaded".into()));
    }

    // Run the OCR ensemble + tax-form parser on a blocking thread —
    // tract/tesseract are CPU-bound.
    let model_dir = s.ocr_model_dir();
    let extract = tokio::task::spawn_blocking(move || {
        let result = traderview_ocr::extract(&bytes, &mime, Some(&model_dir));
        result.ok().and_then(|r| traderview_ocr::tax_forms::extract(&r.text))
    })
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("join: {e}")))?
    .ok_or_else(|| ApiError::BadRequest(
        "no recognizable tax form detected — upload a W-2 or 1099".into(),
    ))?;

    let kind_str = serde_json::to_string(&extract.kind)
        .ok()
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default();
    let payload = serde_json::to_value(&extract.payload).unwrap_or(serde_json::json!({}));

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO w2_imports (user_id, tax_year, kind, payload, confidence)
              VALUES ($1, $2, $3, $4, $5)
         RETURNING id",
    )
    .bind(user.id)
    .bind(tax_year)
    .bind(&kind_str)
    .bind(&payload)
    .bind(extract.confidence)
    .fetch_one(&s.pool)
    .await?;

    Ok(Json(FormUploadResult {
        id,
        kind: kind_str,
        payload,
        party_name: extract.party_name,
        confidence: extract.confidence,
        tax_year,
    }))
}

#[derive(Serialize, sqlx::FromRow)]
struct FormImportRow {
    id: Uuid,
    tax_year: i32,
    kind: String,
    payload: serde_json::Value,
    confidence: f32,
    created_at: chrono::DateTime<Utc>,
}

async fn list_forms(
    State(s): State<AppState>,
    user: AuthUser,
    Path(year): Path<i32>,
) -> Result<Json<Vec<FormImportRow>>, ApiError> {
    let rows: Vec<FormImportRow> = sqlx::query_as(
        "SELECT id, tax_year, kind, payload, confidence, created_at
           FROM w2_imports
          WHERE user_id = $1 AND tax_year = $2
       ORDER BY created_at DESC",
    )
    .bind(user.id)
    .bind(year)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}
