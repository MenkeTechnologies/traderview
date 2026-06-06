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
use axum::extract::{Multipart, Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{Datelike, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use traderview_tax::education_credits::{self, AotcInput, AotcResult, LlcInput, LlcResult};
use traderview_tax::home_office::{self, HomeOfficeInput, HomeOfficeResult};
use traderview_tax::late_penalty::{self, LatePenaltyInput, LatePenaltyResult};
use traderview_tax::mileage::{
    self, MethodComparisonInput, MethodComparisonResult, MileageInput, MileageResult,
};
use traderview_tax::retirement_limits::{
    self, HsaInput, HsaResult, IraInput, IraResult, RothIraInput, RothIraResult,
};
use traderview_tax::safe_harbor::{self, BindingHarbor, SafeHarborInput, SafeHarborResult};
use traderview_tax::section_179::{self, Section179Input, Section179Result};
use traderview_tax::what_if::{self, Scenario, WhatIfResult};
use traderview_tax::{compute as compute_tax, TaxResult, TaxReturn};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/returns/:year", get(get_return).put(put_return))
        .route("/returns/:year/autopopulate", post(autopopulate))
        .route("/returns/:year/compute", get(compute_endpoint))
        .route("/returns/:year/pdf", get(crate::tax_pdf::generate_pdf))
        .route("/returns/:year/safe-harbor", get(safe_harbor_endpoint))
        .route("/returns/:year/what-if", post(what_if_endpoint))
        .route("/returns/:year/late-penalty", post(late_penalty_endpoint))
        .route("/planner/aotc", post(aotc_endpoint))
        .route("/planner/llc", post(llc_endpoint))
        .route("/planner/ira", post(ira_endpoint))
        .route("/planner/roth-ira", post(roth_ira_endpoint))
        .route("/planner/hsa", post(hsa_endpoint))
        .route("/planner/mileage", post(mileage_endpoint))
        .route("/planner/mileage-compare", post(mileage_compare_endpoint))
        .route("/planner/home-office", post(home_office_endpoint))
        .route("/planner/section-179", post(section_179_endpoint))
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
    let row: Option<(Uuid, serde_json::Value)> =
        sqlx::query_as("SELECT id, data FROM tax_returns WHERE user_id = $1 AND tax_year = $2")
            .bind(user_id)
            .bind(year)
            .fetch_optional(&s.pool)
            .await?;

    if let Some((id, data)) = row {
        let draft = serde_json::from_value::<TaxReturn>(data).unwrap_or_default();
        return Ok((id, draft));
    }

    let draft = TaxReturn {
        tax_year: year,
        ..Default::default()
    };
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
    let status: Option<String> = sqlx::query_scalar("SELECT status FROM tax_returns WHERE id = $1")
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
    Ok(Json(ReturnView {
        id,
        tax_year: year,
        status,
        draft,
        result,
    }))
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
/// plus Schedule E totals from rental_properties (or items with
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
    let sched_c_expenses: Decimal = sched_c_row.and_then(|(d,)| d).unwrap_or(Decimal::ZERO);

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
    let sched_e_expenses: Decimal = sched_e_row.and_then(|(d,)| d).unwrap_or(Decimal::ZERO);
    draft.schedule_e.total_expenses = sched_e_expenses;
    draft.schedule_e.net_income = draft.schedule_e.gross_rents - draft.schedule_e.total_expenses;

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
        id,
        tax_year: year,
        status: "income".into(),
        draft,
        result,
    }))
}

// ── W-2 / 1099 OCR upload ──────────────────────────────────────────────

// ── Quarterly safe-harbor calculator ─────────────────────────────────

#[derive(Deserialize)]
struct SafeHarborParams {
    /// Quarter to compute through. 1 = Q1 (Apr 15), 2 = Q2 (Jun 15),
    /// 3 = Q3 (Sep 15), 4 = Q4 (Jan 15 of next year).
    #[serde(default = "default_quarter")]
    quarter: u32,
    /// Optional override for prior-year tax — when the caller didn't
    /// file the prior year through this app and can't auto-pull. When
    /// absent, we read tax_returns for `tax_year - 1` and use that
    /// row's computed tax_owed + total_payments as the liability.
    #[serde(default)]
    prior_year_tax: Option<Decimal>,
    /// Same for prior-year AGI (determines 100% vs 110% rule).
    #[serde(default)]
    prior_year_agi: Option<Decimal>,
}

fn default_quarter() -> u32 {
    // Use the current calendar quarter so first-load lands on the
    // user's "next due date" rather than always Q1.
    let now = Utc::now();
    let month = now.month();
    match month {
        1..=3 => 1,
        4..=5 => 2, // Q2 covers Jun 15 deadline
        6..=8 => 3,
        _ => 4,
    }
}

async fn safe_harbor_endpoint(
    State(s): State<AppState>,
    user: AuthUser,
    Path(year): Path<i32>,
    Query(params): Query<SafeHarborParams>,
) -> Result<Json<SafeHarborResult>, ApiError> {
    // Current-year draft — used for projected tax + estimated YTD.
    let (_, draft) = load_or_create(&s, user.id, year).await?;
    let current_result = compute_tax(&draft);

    // Pull prior-year tax + AGI either from the override params or
    // from the user's prior-year return.
    let (prior_year_tax, prior_year_agi) = match (params.prior_year_tax, params.prior_year_agi) {
        (Some(t), Some(a)) => (t, a),
        _ => {
            let row: Option<(Decimal, Decimal)> = sqlx::query_as(
                "SELECT COALESCE(tax_owed, 0) + COALESCE(refund_due, 0), COALESCE(agi, 0)
                   FROM tax_returns WHERE user_id = $1 AND tax_year = $2",
            )
            .bind(user.id)
            .bind(year - 1)
            .fetch_optional(&s.pool)
            .await?;
            row.unwrap_or((Decimal::ZERO, Decimal::ZERO))
        }
    };

    let w2_wh: Decimal = draft
        .w2s
        .iter()
        .map(|w| w.box_2_federal_income_tax_withheld)
        .sum();

    let input = SafeHarborInput {
        prior_year_tax,
        prior_year_agi,
        filing_status: draft.status,
        current_year_projected_tax: current_result.tax_after_credits,
        w2_withholding_ytd: w2_wh,
        estimated_paid_ytd: draft.estimated_tax_payments,
        current_quarter: params.quarter,
    };
    Ok(Json(safe_harbor::compute(input)))
}

// ── "What-if" delta endpoint ──────────────────────────────────────────

#[derive(Deserialize)]
struct WhatIfBody {
    scenario: Scenario,
}

async fn what_if_endpoint(
    State(s): State<AppState>,
    user: AuthUser,
    Path(year): Path<i32>,
    Json(body): Json<WhatIfBody>,
) -> Result<Json<WhatIfResult>, ApiError> {
    let (_, draft) = load_or_create(&s, user.id, year).await?;
    let result = what_if::compute_what_if(&draft, body.scenario).ok_or_else(|| {
        ApiError::BadRequest(
            "scenario.path not recognized — see what_if::Scenario doc for valid field slugs".into(),
        )
    })?;
    Ok(Json(result))
}

// ── IRC § 6651 late-file / late-pay penalty + § 6601 interest ──────────
//
// Stateless: client posts the inputs, server returns the breakdown. No
// authentication / draft load needed beyond the auth check, but we keep
// the same `Path(year)` shape so the URL is uniform with the rest of the
// tax-wizard surface.

async fn late_penalty_endpoint(
    _state: State<AppState>,
    _user: AuthUser,
    Path(_year): Path<i32>,
    Json(body): Json<LatePenaltyInput>,
) -> Result<Json<LatePenaltyResult>, ApiError> {
    Ok(Json(late_penalty::compute(body)))
}

// ── Planner endpoints (stateless compute) ──────────────────────────────
//
// Each one takes a self-contained input struct and returns the matching
// result. No draft / database access — these are on-demand planning
// tools that don't persist anything.

async fn aotc_endpoint(
    _user: AuthUser,
    Json(body): Json<AotcInput>,
) -> Result<Json<AotcResult>, ApiError> {
    Ok(Json(education_credits::aotc(body)))
}

async fn llc_endpoint(
    _user: AuthUser,
    Json(body): Json<LlcInput>,
) -> Result<Json<LlcResult>, ApiError> {
    Ok(Json(education_credits::llc(body)))
}

async fn ira_endpoint(
    _user: AuthUser,
    Json(body): Json<IraInput>,
) -> Result<Json<IraResult>, ApiError> {
    Ok(Json(retirement_limits::ira(body)))
}

async fn roth_ira_endpoint(
    _user: AuthUser,
    Json(body): Json<RothIraInput>,
) -> Result<Json<RothIraResult>, ApiError> {
    Ok(Json(retirement_limits::roth_ira(body)))
}

async fn hsa_endpoint(
    _user: AuthUser,
    Json(body): Json<HsaInput>,
) -> Result<Json<HsaResult>, ApiError> {
    Ok(Json(retirement_limits::hsa(body)))
}

async fn mileage_endpoint(
    _user: AuthUser,
    Json(body): Json<MileageInput>,
) -> Result<Json<MileageResult>, ApiError> {
    Ok(Json(mileage::compute(body)))
}

async fn mileage_compare_endpoint(
    _user: AuthUser,
    Json(body): Json<MethodComparisonInput>,
) -> Result<Json<MethodComparisonResult>, ApiError> {
    Ok(Json(mileage::compare_methods(body)))
}

async fn home_office_endpoint(
    _user: AuthUser,
    Json(body): Json<HomeOfficeInput>,
) -> Result<Json<HomeOfficeResult>, ApiError> {
    Ok(Json(home_office::compute(body)))
}

async fn section_179_endpoint(
    _user: AuthUser,
    Json(body): Json<Section179Input>,
) -> Result<Json<Section179Result>, ApiError> {
    Ok(Json(section_179::compute(body)))
}

// Silence "BindingHarbor unused" — we re-export it from the route file's
// crate-level imports so the OpenAPI / client-side type generators can
// see the variant names. Touch via a noop assertion to anchor the
// dependency.
#[allow(dead_code)]
fn _anchor_binding_harbor() -> BindingHarbor {
    BindingHarbor::default()
}

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
    let mut tax_year: i32 = Utc::now().year() - 1; // default: last tax year

    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
    {
        match field.name() {
            Some("file") => {
                mime = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
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
        result
            .ok()
            .and_then(|r| traderview_ocr::tax_forms::extract(&r.text))
    })
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("join: {e}")))?
    .ok_or_else(|| {
        ApiError::BadRequest("no recognizable tax form detected — upload a W-2 or 1099".into())
    })?;

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
