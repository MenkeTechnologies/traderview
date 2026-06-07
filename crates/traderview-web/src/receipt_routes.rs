//! Receipt routes: upload, fetch, OCR matching, attach.
//!
//! Storage layout: `$DATA_DIR/receipts/<sha256[0:2]>/<sha256>.<ext>`. The DB
//! row holds the relative path so the data dir can move without rewriting
//! rows.
//!
//! Upload flow:
//!   1. POST /api/expense/receipts (multipart) → write blob, insert row with
//!      `ocr_status='pending'`, spawn a tokio task that runs OCR + persists
//!      results, then returns immediately.
//!   2. Frontend polls GET /api/expense/receipts/:id until status leaves
//!      `pending`, then GET .../matches to surface candidate transactions.
//!   3. User picks one; POST .../attach links the receipt row to the
//!      transaction. Both directions are now navigable.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Multipart, Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use traderview_ocr::matcher;
use uuid::Uuid;

/// Per-receipt upload cap. Large enough for high-DPI phone photos + PDFs;
/// small enough that a malicious or buggy client can't fill the disk on a
/// single request. Default axum body limit is 2 MB which would reject most
/// real receipts, so we override per-router.
const RECEIPT_UPLOAD_MAX_BYTES: usize = 25 * 1024 * 1024;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_receipts).post(upload_receipt))
        .route("/:id", get(get_receipt_blob))
        .route("/:id/meta", get(get_receipt_meta).patch(patch_receipt_meta))
        .route("/:id/matches", get(receipt_matches))
        .route("/:id/attach", post(attach_receipt))
        .route("/:id/retry-ocr", post(retry_receipt_ocr))
        .route(
            "/:id/items/:idx",
            axum::routing::patch(patch_receipt_item).delete(delete_receipt_item),
        )
        .route("/:id/items", post(add_receipt_item))
        .route("/tax-rollup", get(tax_rollup))
        .route("/bulk-attach", post(bulk_attach))
        .route("/bulk-delete", post(bulk_delete))
        .route("/bulk-patch-items", post(bulk_patch_items))
        .route("/bulk-reocr", post(bulk_reocr))
        .route("/bulk-reocr/progress", get(bulk_reocr_progress))
        .route("/by-merchant", get(receipts_by_merchant))
        .route("/search", get(receipts_search))
        .route("/duplicates", get(receipts_duplicates))
        .route("/recurring", get(receipts_recurring))
        .route("/spend-calendar", get(spend_calendar))
        .route("/calendar/:year/:month", get(receipts_month_calendar))
        .route("/dashboard-bundle", get(expense_dashboard_bundle))
        .route("/dow", get(receipts_dow))
        .route("/cumulative", get(receipts_cumulative))
        .route("/yoy-monthly", get(receipts_yoy_monthly))
        .route("/aging", get(receipts_aging))
        .route("/by-property", get(receipts_by_property))
        .route("/anomalies", get(receipts_anomalies))
        .route(
            "/category-distribution",
            get(receipts_category_distribution),
        )
        .route("/tax-rollup.csv", get(tax_rollup_csv))
        .route("/tax-rollup.pdf", get(tax_rollup_pdf))
        .route("/ocr-models/status", get(ocr_models_status))
        .route("/ocr-models/download", post(ocr_models_download))
        .layer(DefaultBodyLimit::max(RECEIPT_UPLOAD_MAX_BYTES))
}

// --- types ---------------------------------------------------------------

#[derive(Serialize)]
struct Receipt {
    id: Uuid,
    user_id: Uuid,
    transaction_id: Option<Uuid>,
    filename: String,
    sha256: String,
    mime: String,
    bytes_len: i64,
    ocr_status: String,
    ocr_text: Option<String>,
    ocr_merchant: Option<String>,
    ocr_total: Option<Decimal>,
    ocr_date: Option<NaiveDate>,
    ocr_confidence: Option<f32>,
    /// Structured slice — itemized list, address, time, subtotal, tax,
    /// per-item category. JSONB blob from migration 0041; shape matches
    /// `traderview_ocr::OcrResult` (minus the duplicated top-level
    /// fields). `null` when OCR hasn't run yet.
    ocr_extracted: Option<serde_json::Value>,
    match_score: Option<f32>,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct ReceiptRow {
    id: Uuid,
    user_id: Uuid,
    transaction_id: Option<Uuid>,
    filename: String,
    sha256: String,
    mime: String,
    bytes_len: i64,
    ocr_status: String,
    ocr_text: Option<String>,
    ocr_merchant: Option<String>,
    ocr_total: Option<Decimal>,
    ocr_date: Option<NaiveDate>,
    ocr_confidence: Option<f32>,
    ocr_extracted: Option<serde_json::Value>,
    match_score: Option<f32>,
    error_message: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<ReceiptRow> for Receipt {
    fn from(r: ReceiptRow) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            transaction_id: r.transaction_id,
            filename: r.filename,
            sha256: r.sha256,
            mime: r.mime,
            bytes_len: r.bytes_len,
            ocr_status: r.ocr_status,
            ocr_text: r.ocr_text,
            ocr_merchant: r.ocr_merchant,
            ocr_total: r.ocr_total,
            ocr_date: r.ocr_date,
            ocr_confidence: r.ocr_confidence,
            ocr_extracted: r.ocr_extracted,
            match_score: r.match_score,
            error_message: r.error_message,
            created_at: r.created_at,
        }
    }
}

// --- list ---------------------------------------------------------------
//
// Paginated + filterable receipts list. At 10k+ receipts the old
// `LIMIT 200` was a UX dead-end. Filters cover the actual workflow:
// find unattached `done` receipts in a date window matching a
// merchant substring within a total range.

#[derive(Deserialize)]
struct ListQ {
    /// Filter by ocr_status. `all` (default) skips the filter; any
    /// other value must be one of the ocr_status_t enum members.
    #[serde(default)]
    status: Option<String>,
    /// Substring match on ocr_merchant (case-insensitive).
    #[serde(default)]
    merchant: Option<String>,
    /// Date range over ocr_date (inclusive, ISO).
    #[serde(default)]
    from: Option<NaiveDate>,
    #[serde(default)]
    to: Option<NaiveDate>,
    /// Total range (inclusive).
    #[serde(default)]
    min_total: Option<Decimal>,
    #[serde(default)]
    max_total: Option<Decimal>,
    /// When `true`, hide receipts that already have a transaction_id.
    #[serde(default)]
    unattached: Option<bool>,
    /// Pagination. `offset` default 0, `limit` default 50 (max 500).
    #[serde(default)]
    offset: Option<i64>,
    #[serde(default)]
    limit: Option<i64>,
}

#[derive(Serialize)]
struct ListResponse {
    rows: Vec<Receipt>,
    total: i64,
    offset: i64,
    limit: i64,
}

async fn list_receipts(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<ListQ>,
) -> Result<Json<ListResponse>, ApiError> {
    let limit = q.limit.unwrap_or(50).clamp(1, 500);
    let offset = q.offset.unwrap_or(0).max(0);
    let merchant_pat = q.merchant.as_ref().map(|m| format!("%{}%", m));

    // Build the WHERE clauses dynamically. Use $N positional binds and
    // a tracker so the row count + the data query share the same args.
    // status validation: only the known enum members pass through.
    let status_valid = q.status.as_deref().map(|s| {
        matches!(
            s,
            "pending" | "matching" | "done" | "failed" | "needs_image"
        )
    });
    if let Some(false) = status_valid {
        return Err(ApiError::BadRequest(format!(
            "invalid status: {}",
            q.status.as_deref().unwrap_or("")
        )));
    }
    let want_status = q.status.as_deref().filter(|_| status_valid == Some(true));

    let rows: Vec<ReceiptRow> = sqlx::query_as(
        "SELECT id, user_id, transaction_id, filename, sha256, mime, bytes_len,
                ocr_status::text, ocr_text, ocr_merchant, ocr_total, ocr_date,
                ocr_confidence, ocr_extracted, match_score, error_message, created_at
           FROM receipts
          WHERE user_id = $1
            AND ($2::text IS NULL OR ocr_status::text = $2)
            AND ($3::text IS NULL OR ocr_merchant ILIKE $3)
            AND ($4::date IS NULL OR ocr_date >= $4)
            AND ($5::date IS NULL OR ocr_date <= $5)
            AND ($6::numeric IS NULL OR ocr_total >= $6)
            AND ($7::numeric IS NULL OR ocr_total <= $7)
            AND ($8::bool IS NULL OR ($8 = TRUE AND transaction_id IS NULL)
                                  OR ($8 = FALSE AND transaction_id IS NOT NULL))
          ORDER BY created_at DESC
          LIMIT $9 OFFSET $10",
    )
    .bind(user.id)
    .bind(want_status)
    .bind(&merchant_pat)
    .bind(q.from)
    .bind(q.to)
    .bind(q.min_total)
    .bind(q.max_total)
    .bind(q.unattached)
    .bind(limit)
    .bind(offset)
    .fetch_all(&s.pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint
           FROM receipts
          WHERE user_id = $1
            AND ($2::text IS NULL OR ocr_status::text = $2)
            AND ($3::text IS NULL OR ocr_merchant ILIKE $3)
            AND ($4::date IS NULL OR ocr_date >= $4)
            AND ($5::date IS NULL OR ocr_date <= $5)
            AND ($6::numeric IS NULL OR ocr_total >= $6)
            AND ($7::numeric IS NULL OR ocr_total <= $7)
            AND ($8::bool IS NULL OR ($8 = TRUE AND transaction_id IS NULL)
                                  OR ($8 = FALSE AND transaction_id IS NOT NULL))",
    )
    .bind(user.id)
    .bind(want_status)
    .bind(&merchant_pat)
    .bind(q.from)
    .bind(q.to)
    .bind(q.min_total)
    .bind(q.max_total)
    .bind(q.unattached)
    .fetch_one(&s.pool)
    .await
    .unwrap_or(0);

    Ok(Json(ListResponse {
        rows: rows.into_iter().map(Into::into).collect(),
        total,
        offset,
        limit,
    }))
}

// --- upload -------------------------------------------------------------

async fn upload_receipt(
    State(s): State<AppState>,
    user: AuthUser,
    mut mp: Multipart,
) -> Result<Json<Receipt>, ApiError> {
    let mut filename = String::new();
    let mut mime = String::new();
    let mut bytes: Vec<u8> = Vec::new();

    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("multipart: {e}")))?
    {
        if field.name() == Some("file") {
            filename = field.file_name().unwrap_or("receipt").to_string();
            mime = field
                .content_type()
                .map(|c| c.to_string())
                .unwrap_or_else(|| guess_mime(&filename));
            let b = field
                .bytes()
                .await
                .map_err(|e| ApiError::BadRequest(e.to_string()))?;
            bytes = b.to_vec();
        }
    }
    if bytes.is_empty() {
        return Err(ApiError::BadRequest("missing file".into()));
    }
    if !is_acceptable_mime(&mime) {
        return Err(ApiError::BadRequest(format!(
            "unsupported mime: {mime} (jpg/png/webp/pdf only)"
        )));
    }

    let mut h = Sha256::new();
    h.update(&bytes);
    let sha = hex::encode(h.finalize());

    // De-dupe per user + sha. If a row already exists, two outcomes:
    //   * Row's OCR succeeded (`done`) or is in flight (`pending` /
    //     `matching`): return the cached row, no work to do.
    //   * Row previously FAILED (`failed` / `needs_image`): the OCR engine
    //     state may have changed since (e.g. user just enabled
    //     `--features ocr-engine` or shipped paddleocr models), so reset
    //     the row to `pending` and re-queue OCR with the existing bytes
    //     on disk. The frontend's polling loop will pick the new run up
    //     transparently.
    let existing: Option<ReceiptRow> = sqlx::query_as(
        "SELECT id, user_id, transaction_id, filename, sha256, mime, bytes_len,
                ocr_status::text, ocr_text, ocr_merchant, ocr_total, ocr_date,
                ocr_confidence, ocr_extracted, match_score, error_message, created_at
           FROM receipts WHERE user_id = $1 AND sha256 = $2",
    )
    .bind(user.id)
    .bind(&sha)
    .fetch_optional(&s.pool)
    .await?;
    if let Some(r) = existing {
        let status_is_failed = matches!(r.ocr_status.as_str(), "failed" | "needs_image");
        if !status_is_failed {
            return Ok(Json(r.into()));
        }
        // Reset + re-queue. The on-disk blob is unchanged so we hand it
        // straight to `run_ocr` without re-reading from disk.
        sqlx::query(
            "UPDATE receipts SET ocr_status = 'pending'::ocr_status_t,
                                  error_message = NULL
              WHERE id = $1",
        )
        .bind(r.id)
        .execute(&s.pool)
        .await?;
        let bg_state = s.clone();
        let receipt_id = r.id;
        let mime_owned = r.mime.clone();
        tokio::spawn(async move {
            // Bound concurrent OCR jobs. At 10k uploads via the folder
            // scanner this prevents tesseract from fork-bombing the
            // host. Permit dropped automatically when the spawn ends.
            let _permit = bg_state.ocr_sem.clone().acquire_owned().await.ok();
            let result = std::panic::AssertUnwindSafe(run_ocr(
                bg_state.clone(),
                receipt_id,
                bytes,
                mime_owned,
            ));
            match futures_util::FutureExt::catch_unwind(result).await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    tracing::error!(receipt = %receipt_id, error = %e, "ocr re-queue failed");
                    mark_ocr_failed(&bg_state, receipt_id, &e.to_string()).await;
                }
                Err(panic) => {
                    let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                        (*s).to_string()
                    } else if let Some(s) = panic.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "OCR engine panicked".into()
                    };
                    tracing::error!(receipt = %receipt_id, panic = %msg, "ocr re-queue panicked");
                    mark_ocr_failed(&bg_state, receipt_id, &format!("OCR panic: {msg}")).await;
                }
            }
        });
        // Return the row with the reset status so the frontend's polling
        // loop sees `pending` and waits for the new result instead of
        // immediately rendering the stale failure.
        let refreshed: ReceiptRow = sqlx::query_as(
            "SELECT id, user_id, transaction_id, filename, sha256, mime, bytes_len,
                    ocr_status::text, ocr_text, ocr_merchant, ocr_total, ocr_date,
                    ocr_confidence, ocr_extracted, match_score, error_message, created_at
               FROM receipts WHERE id = $1",
        )
        .bind(r.id)
        .fetch_one(&s.pool)
        .await?;
        return Ok(Json(refreshed.into()));
    }

    let ext = ext_from_mime(&mime).unwrap_or_else(|| extension_of(&filename));
    let rel_path: PathBuf = PathBuf::from(&sha[0..2]).join(format!("{sha}.{ext}"));
    let abs_path = s.receipts_dir().join(&rel_path);
    if let Some(parent) = abs_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&abs_path, &bytes).await?;

    let row: ReceiptRow = sqlx::query_as(
        "INSERT INTO receipts
            (user_id, filename, sha256, mime, bytes_len, storage_path, ocr_status)
          VALUES ($1, $2, $3, $4, $5, $6, 'pending'::ocr_status_t)
         RETURNING id, user_id, transaction_id, filename, sha256, mime, bytes_len,
                   ocr_status::text, ocr_text, ocr_merchant, ocr_total, ocr_date,
                   ocr_confidence, ocr_extracted, match_score, error_message, created_at",
    )
    .bind(user.id)
    .bind(&filename)
    .bind(&sha)
    .bind(&mime)
    .bind(bytes.len() as i64)
    .bind(rel_path.to_string_lossy().to_string())
    .fetch_one(&s.pool)
    .await?;

    // Spawn OCR in the background — caller returns immediately, frontend
    // polls `/:id/meta` until `ocr_status` leaves `pending`.
    //
    // The task body runs through `AssertUnwindSafe` + `catch_unwind` so a
    // panic in the OCR engine (tract-onnx has been known to panic on
    // malformed ONNX or extreme image shapes) marks the row as failed
    // instead of leaving it stuck at `pending` forever — the frontend's
    // polling loop would otherwise spin indefinitely.
    let bg_state = s.clone();
    let receipt_id = row.id;
    let mime_owned = mime.clone();
    tokio::spawn(async move {
        // Bound concurrent OCR jobs — see semaphore docs on AppState.
        let _permit = bg_state.ocr_sem.clone().acquire_owned().await.ok();
        let result =
            std::panic::AssertUnwindSafe(run_ocr(bg_state.clone(), receipt_id, bytes, mime_owned));
        match futures_util::FutureExt::catch_unwind(result).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                tracing::error!(receipt = %receipt_id, error = %e, "ocr job failed");
                mark_ocr_failed(&bg_state, receipt_id, &e.to_string()).await;
            }
            Err(panic) => {
                let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "OCR engine panicked".into()
                };
                tracing::error!(receipt = %receipt_id, panic = %msg, "ocr task panicked");
                mark_ocr_failed(&bg_state, receipt_id, &format!("OCR panic: {msg}")).await;
            }
        }
    });

    Ok(Json(row.into()))
}

/// Best-effort status update — used by the panic path so a failed task
/// doesn't leave a receipt stuck at `pending` forever.
async fn mark_ocr_failed(s: &AppState, id: Uuid, msg: &str) {
    let _ = sqlx::query(
        "UPDATE receipts SET ocr_status = 'failed'::ocr_status_t,
                             error_message = $2
          WHERE id = $1",
    )
    .bind(id)
    .bind(msg)
    .execute(&s.pool)
    .await;
}

async fn run_ocr(
    s: AppState,
    receipt_id: Uuid,
    bytes: Vec<u8>,
    mime: String,
) -> anyhow::Result<()> {
    let model_dir = s.ocr_model_dir();
    // Off-thread OCR — pure-onnx-ocr can be CPU-bound on tract-onnx; keep the
    // tokio runtime responsive by parking on a blocking thread.
    let result = tokio::task::spawn_blocking(move || {
        traderview_ocr::extract(&bytes, &mime, Some(&model_dir))
    })
    .await?;

    match result {
        Ok(mut ocr) => {
            // Apply the user's learned merchant→category mapping
            // before persisting. The parse pipeline auto-guesses based
            // on item-name keywords; the user's past corrections are
            // strictly more authoritative. Each PATCH-time learning
            // hook UPSERTs (canonical_merchant, category, use_count) —
            // we pick the highest-use_count row and replace every
            // item's auto-guess with it.
            //
            // The user's user_id is needed for the learned-categories
            // query — look it up from the receipts row. If the lookup
            // fails for any reason (deleted receipt, RLS), just
            // proceed with the auto-guessed categories.
            if let Some(merchant_raw) = ocr.merchant.as_deref() {
                if !merchant_raw.is_empty() {
                    if let Ok(Some((user_id,))) =
                        sqlx::query_as::<_, (Uuid,)>("SELECT user_id FROM receipts WHERE id = $1")
                            .bind(receipt_id)
                            .fetch_optional(&s.pool)
                            .await
                    {
                        let aliases = crate::merchant::load_aliases(&s.pool, user_id)
                            .await
                            .unwrap_or_default();
                        let canonical = crate::merchant::canonicalize(merchant_raw, &aliases);
                        if !canonical.is_empty() {
                            let learned: Option<(String,)> = sqlx::query_as(
                                "SELECT category_code
                                   FROM learned_merchant_categories
                                  WHERE user_id = $1
                                    AND merchant_canonical = $2
                                  ORDER BY use_count DESC, last_used DESC
                                  LIMIT 1",
                            )
                            .bind(user_id)
                            .bind(&canonical)
                            .fetch_optional(&s.pool)
                            .await
                            .unwrap_or(None);
                            if let Some((learned_cat,)) = learned {
                                for item in ocr.items.iter_mut() {
                                    item.category = learned_cat.clone();
                                }
                                tracing::debug!(
                                    receipt = %receipt_id,
                                    merchant = %canonical,
                                    category = %learned_cat,
                                    items = ocr.items.len(),
                                    "applied learned merchant→category",
                                );
                            }
                        }
                    }
                }
            }

            // Persist the structured slice (address, time, subtotal, tax,
            // itemized lines + categories) into the JSONB column from
            // migration 0041. Falls back to `{}` on serialize failure so
            // a malformed item never blocks the receipt row from
            // landing.
            let extracted = serde_json::to_value(&ocr).unwrap_or_else(|_| serde_json::json!({}));
            sqlx::query(
                "UPDATE receipts SET
                    ocr_status = 'done'::ocr_status_t,
                    ocr_text = $2,
                    ocr_merchant = $3,
                    ocr_total = $4,
                    ocr_date = $5,
                    ocr_confidence = $6,
                    ocr_extracted = $7,
                    error_message = NULL
                  WHERE id = $1",
            )
            .bind(receipt_id)
            .bind(&ocr.text)
            .bind(&ocr.merchant)
            .bind(ocr.total)
            .bind(ocr.date)
            .bind(ocr.confidence)
            .bind(&extracted)
            .execute(&s.pool)
            .await?;
        }
        Err(traderview_ocr::OcrError::NeedsImage) => {
            sqlx::query(
                "UPDATE receipts SET ocr_status = 'needs_image'::ocr_status_t,
                                     error_message = $2
                  WHERE id = $1",
            )
            .bind(receipt_id)
            .bind("pdf has no text layer — re-upload as JPG/PNG")
            .execute(&s.pool)
            .await?;
        }
        Err(other) => {
            sqlx::query(
                "UPDATE receipts SET ocr_status = 'failed'::ocr_status_t,
                                     error_message = $2
                  WHERE id = $1",
            )
            .bind(receipt_id)
            .bind(other.to_string())
            .execute(&s.pool)
            .await?;
        }
    }
    Ok(())
}

// --- fetch blob ---------------------------------------------------------

async fn get_receipt_blob(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let row: Option<(Uuid, String, String, String)> =
        sqlx::query_as("SELECT user_id, storage_path, mime, filename FROM receipts WHERE id = $1")
            .bind(id)
            .fetch_optional(&s.pool)
            .await?;
    let (owner, rel_path, mime, filename) = row.ok_or(ApiError::NotFound)?;
    if owner != user.id {
        return Err(ApiError::Forbidden);
    }
    let abs = s.receipts_dir().join(&rel_path);
    let bytes = tokio::fs::read(&abs).await?;
    let disposition = format!("inline; filename=\"{}\"", sanitize_disposition(&filename));
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_DISPOSITION, disposition)
        .body(Body::from(bytes))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("response build: {e}")))
}

/// Re-run OCR on an existing receipt using the blob already on disk.
/// Used by the "Retry OCR" affordance the frontend renders next to a
/// failed receipt — particularly after the user just downloaded missing
/// OCR models. Owner check + reset to `pending` + spawn the same
/// `run_ocr` path the upload flow uses.
async fn retry_receipt_ocr(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Receipt>, ApiError> {
    let row: Option<(Uuid, String, String)> =
        sqlx::query_as("SELECT user_id, storage_path, mime FROM receipts WHERE id = $1")
            .bind(id)
            .fetch_optional(&s.pool)
            .await?;
    let (owner, rel_path, mime) = row.ok_or(ApiError::NotFound)?;
    if owner != user.id {
        return Err(ApiError::Forbidden);
    }
    let abs = s.receipts_dir().join(&rel_path);
    let bytes = tokio::fs::read(&abs).await?;

    sqlx::query(
        "UPDATE receipts SET ocr_status = 'pending'::ocr_status_t,
                              error_message = NULL
          WHERE id = $1",
    )
    .bind(id)
    .execute(&s.pool)
    .await?;

    let bg_state = s.clone();
    let receipt_id = id;
    let mime_owned = mime.clone();
    tokio::spawn(async move {
        // Bound concurrent OCR jobs — see semaphore docs on AppState.
        let _permit = bg_state.ocr_sem.clone().acquire_owned().await.ok();
        let result =
            std::panic::AssertUnwindSafe(run_ocr(bg_state.clone(), receipt_id, bytes, mime_owned));
        match futures_util::FutureExt::catch_unwind(result).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                tracing::error!(receipt = %receipt_id, error = %e, "ocr retry failed");
                mark_ocr_failed(&bg_state, receipt_id, &e.to_string()).await;
            }
            Err(panic) => {
                let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "OCR engine panicked".into()
                };
                tracing::error!(receipt = %receipt_id, panic = %msg, "ocr retry panicked");
                mark_ocr_failed(&bg_state, receipt_id, &format!("OCR panic: {msg}")).await;
            }
        }
    });

    let refreshed: ReceiptRow = sqlx::query_as(
        "SELECT id, user_id, transaction_id, filename, sha256, mime, bytes_len,
                ocr_status::text, ocr_text, ocr_merchant, ocr_total, ocr_date,
                ocr_confidence, ocr_extracted, match_score, error_message, created_at
           FROM receipts WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(refreshed.into()))
}

/// Make a filename safe to interpolate into a Content-Disposition header.
/// A raw filename from multipart can contain CR/LF (header injection),
/// embedded quotes (breaks the header parse), or non-ASCII control bytes.
/// We strip controls + replace quotes — keeps the visible UX intact while
/// closing the injection surface.
fn sanitize_disposition(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' | '\\' => '_',
            c if c.is_control() => '_', // strips \r, \n, \t, NUL, etc.
            c => c,
        })
        .collect()
}

async fn get_receipt_meta(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Receipt>, ApiError> {
    let row: Option<ReceiptRow> = sqlx::query_as(
        "SELECT id, user_id, transaction_id, filename, sha256, mime, bytes_len,
                ocr_status::text, ocr_text, ocr_merchant, ocr_total, ocr_date,
                ocr_confidence, ocr_extracted, match_score, error_message, created_at
           FROM receipts WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&s.pool)
    .await?;
    let r = row.ok_or(ApiError::NotFound)?;
    if r.user_id != user.id {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(r.into()))
}

// --- match against transactions -----------------------------------------

#[derive(Serialize)]
struct CandidateMatch {
    transaction: TxBrief,
    score: f32,
}

#[derive(Serialize, sqlx::FromRow)]
struct TxBrief {
    id: Uuid,
    account_id: Uuid,
    posted_at: DateTime<Utc>,
    amount: Decimal,
    merchant_raw: String,
    merchant_normalized: String,
}

type ReceiptOcrRow = (Uuid, Option<String>, Option<Decimal>, Option<NaiveDate>);

async fn receipt_matches(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<CandidateMatch>>, ApiError> {
    let row: Option<ReceiptOcrRow> = sqlx::query_as(
        "SELECT user_id, ocr_merchant, ocr_total, ocr_date FROM receipts WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&s.pool)
    .await?;
    let (owner, merchant, total, date) = row.ok_or(ApiError::NotFound)?;
    if owner != user.id {
        return Err(ApiError::Forbidden);
    }

    // Pull candidates within ±7 days of the OCR date (or last 30 days if no
    // date was extracted). Single SQL hop; scoring runs in Rust.
    let candidates: Vec<TxBrief> = if let Some(d) = date {
        sqlx::query_as(
            "SELECT t.id, t.account_id, t.posted_at, t.amount,
                    t.merchant_raw, t.merchant_normalized
               FROM transactions t
               JOIN financial_accounts a ON a.id = t.account_id
              WHERE a.user_id = $1
                AND t.posted_at >= ($2::date - INTERVAL '7 days')
                AND t.posted_at <  ($2::date + INTERVAL '8 days')
                AND t.is_transfer = FALSE
              LIMIT 500",
        )
        .bind(user.id)
        .bind(d)
        .fetch_all(&s.pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT t.id, t.account_id, t.posted_at, t.amount,
                    t.merchant_raw, t.merchant_normalized
               FROM transactions t
               JOIN financial_accounts a ON a.id = t.account_id
              WHERE a.user_id = $1
                AND t.posted_at > now() - INTERVAL '30 days'
                AND t.is_transfer = FALSE
              LIMIT 500",
        )
        .bind(user.id)
        .fetch_all(&s.pool)
        .await?
    };

    let receipt = matcher::ReceiptFields {
        merchant,
        total,
        date,
    };
    let tx_cands: Vec<matcher::TxCandidate> = candidates
        .iter()
        .map(|c| matcher::TxCandidate {
            id: c.id,
            posted_date: c.posted_at.date_naive(),
            amount: c.amount,
            merchant_normalized: c.merchant_normalized.clone(),
        })
        .collect();
    let scored = matcher::score(&receipt, &tx_cands, 0.5);

    // Stitch scores back to the brief rows for the response.
    let by_id: std::collections::HashMap<Uuid, TxBrief> =
        candidates.into_iter().map(|c| (c.id, c)).collect();
    let out: Vec<CandidateMatch> = scored
        .into_iter()
        .filter_map(|m| {
            by_id.get(&m.id).map(|brief| CandidateMatch {
                transaction: TxBrief {
                    id: brief.id,
                    account_id: brief.account_id,
                    posted_at: brief.posted_at,
                    amount: brief.amount,
                    merchant_raw: brief.merchant_raw.clone(),
                    merchant_normalized: brief.merchant_normalized.clone(),
                },
                score: m.score,
            })
        })
        .take(10)
        .collect();
    Ok(Json(out))
}

// --- attach -------------------------------------------------------------

#[derive(Deserialize)]
struct AttachBody {
    transaction_id: Uuid,
}

async fn attach_receipt(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<AttachBody>,
) -> Result<Json<Receipt>, ApiError> {
    // Verify receipt + tx both belong to user.
    let owner_check: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM receipts WHERE id = $1")
        .bind(id)
        .fetch_optional(&s.pool)
        .await?;
    match owner_check {
        Some((o,)) if o == user.id => {}
        Some(_) => return Err(ApiError::Forbidden),
        None => return Err(ApiError::NotFound),
    }
    let tx_owner: Option<(Uuid,)> = sqlx::query_as(
        "SELECT a.user_id FROM transactions t
           JOIN financial_accounts a ON a.id = t.account_id WHERE t.id = $1",
    )
    .bind(body.transaction_id)
    .fetch_optional(&s.pool)
    .await?;
    match tx_owner {
        Some((o,)) if o == user.id => {}
        Some(_) => return Err(ApiError::Forbidden),
        None => return Err(ApiError::NotFound),
    }

    let row: ReceiptRow = sqlx::query_as(
        "UPDATE receipts SET transaction_id = $2 WHERE id = $1
         RETURNING id, user_id, transaction_id, filename, sha256, mime, bytes_len,
                   ocr_status::text, ocr_text, ocr_merchant, ocr_total, ocr_date,
                   ocr_confidence, ocr_extracted, match_score, error_message, created_at",
    )
    .bind(id)
    .bind(body.transaction_id)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row.into()))
}

// --- mime helpers -------------------------------------------------------

fn is_acceptable_mime(m: &str) -> bool {
    let m = m.to_ascii_lowercase();
    m.starts_with("image/jpeg")
        || m.starts_with("image/png")
        || m.starts_with("image/webp")
        || m.starts_with("image/bmp")
        || m.starts_with("application/pdf")
}

fn guess_mime(filename: &str) -> String {
    let ext = extension_of(filename);
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg".into(),
        "png" => "image/png".into(),
        "webp" => "image/webp".into(),
        "bmp" => "image/bmp".into(),
        "pdf" => "application/pdf".into(),
        _ => "application/octet-stream".into(),
    }
}

fn ext_from_mime(mime: &str) -> Option<String> {
    let m = mime.to_ascii_lowercase();
    if m.starts_with("image/jpeg") {
        Some("jpg".into())
    } else if m.starts_with("image/png") {
        Some("png".into())
    } else if m.starts_with("image/webp") {
        Some("webp".into())
    } else if m.starts_with("image/bmp") {
        Some("bmp".into())
    } else if m.starts_with("application/pdf") {
        Some("pdf".into())
    } else {
        None
    }
}

fn extension_of(filename: &str) -> String {
    // `"noext".rsplit('.').next()` returns `Some("noext")` — the whole
    // string is the first split because there's no separator. That made
    // every extensionless upload land on disk as `<sha>.noext` (BUG).
    // Guard explicitly: if there's no `.` in the name, fall back to "bin".
    match filename.rsplit_once('.') {
        Some((_, ext)) if !ext.is_empty() => ext.to_ascii_lowercase(),
        _ => "bin".into(),
    }
}

// --- receipt-meta PATCH ------------------------------------------------
//
// Lets the frontend hand-correct the OCR's top-level fields — date,
// merchant, total — without forcing the user to re-upload the receipt.
// Particularly important for dates: the parser refuses to commit a
// date when every candidate had a reject tag (return policy / rebate
// expiry), but the user can read the receipt and know the right answer.

#[derive(Deserialize)]
struct PatchMeta {
    merchant: Option<String>,
    total: Option<Decimal>,
    date: Option<NaiveDate>,
}

async fn patch_receipt_meta(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<PatchMeta>,
) -> Result<Json<Receipt>, ApiError> {
    if body.merchant.is_none() && body.total.is_none() && body.date.is_none() {
        return Err(ApiError::BadRequest("no fields to update".into()));
    }
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM receipts WHERE id = $1")
        .bind(id)
        .fetch_optional(&s.pool)
        .await?;
    let owner = owner.ok_or(ApiError::NotFound)?;
    if owner != user.id {
        return Err(ApiError::Forbidden);
    }
    // COALESCE($n, col) — when the patch supplies a field, override;
    // otherwise leave the column alone. Three independent updates in
    // one statement keep the round-trip small.
    sqlx::query(
        "UPDATE receipts SET
            ocr_merchant = COALESCE($2, ocr_merchant),
            ocr_total    = COALESCE($3, ocr_total),
            ocr_date     = COALESCE($4, ocr_date)
          WHERE id = $1",
    )
    .bind(id)
    .bind(&body.merchant)
    .bind(body.total)
    .bind(body.date)
    .execute(&s.pool)
    .await?;

    let row: ReceiptRow = sqlx::query_as(
        "SELECT id, user_id, transaction_id, filename, sha256, mime, bytes_len,
                ocr_status::text, ocr_text, ocr_merchant, ocr_total, ocr_date,
                ocr_confidence, ocr_extracted, match_score, error_message, created_at
           FROM receipts WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row.into()))
}

// --- per-item PATCH ----------------------------------------------------
//
// JSONB column is `receipts.ocr_extracted`. We deserialize the full
// `OcrResult`, mutate the indexed item, serialize back, write back.
// `Option`s on the body fields let the frontend update one attribute at
// a time (e.g., flip `tax_bucket` from `business` to `personal` without
// touching `category` or `rental_property_id`).

#[derive(Deserialize)]
struct PatchItem {
    /// Item display name. Omitted → unchanged.
    name: Option<String>,
    /// Line total. Omitted → unchanged. (Cannot clear; required field.)
    line_total: Option<Decimal>,
    /// Quantity. Use `null` to clear (`Some(None)`), omit to leave
    /// alone, send a value to set.
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    qty: Option<Option<Decimal>>,
    /// Unit price. Same Some/None/absent semantics as `qty`.
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    unit_price: Option<Option<Decimal>>,
    category: Option<String>,
    tax_bucket: Option<String>,
    /// Use `Some(None)` semantics via the inner Option to clear: send
    /// `{"rental_property_id": null}` to detach the property; omit the
    /// field entirely to leave it as-is.
    #[serde(default, deserialize_with = "deserialize_optional_uuid")]
    rental_property_id: Option<Option<Uuid>>,
}

fn deserialize_optional_decimal<'de, D>(de: D) -> Result<Option<Option<Decimal>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Option<Decimal> = Option::deserialize(de)?;
    Ok(Some(v))
}

// Distinguishes "field absent" from "field present and null" so a PATCH
// can explicitly clear the property linkage. `#[serde(default)]` on
// `rental_property_id` makes the field optional; this custom
// deserializer turns `null` into `Some(None)` and a real UUID into
// `Some(Some(uuid))`.
fn deserialize_optional_uuid<'de, D>(de: D) -> Result<Option<Option<Uuid>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Option<Uuid> = Option::deserialize(de)?;
    Ok(Some(v))
}

async fn patch_receipt_item(
    State(s): State<AppState>,
    user: AuthUser,
    Path((receipt_id, idx)): Path<(Uuid, usize)>,
    Json(body): Json<PatchItem>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if body.name.is_none()
        && body.line_total.is_none()
        && body.qty.is_none()
        && body.unit_price.is_none()
        && body.category.is_none()
        && body.tax_bucket.is_none()
        && body.rental_property_id.is_none()
    {
        return Err(ApiError::BadRequest("no fields to update".into()));
    }
    // Validate tax_bucket against the known set so a typo can't poison
    // the rollup (e.g., "buisness" silently dropping the item out of
    // the business bucket).
    if let Some(b) = body.tax_bucket.as_deref() {
        if !matches!(b, "business" | "rental" | "personal" | "unclassified") {
            return Err(ApiError::BadRequest(format!(
                "invalid tax_bucket: {b} (must be business|rental|personal|unclassified)"
            )));
        }
    }

    let row: Option<(Uuid, Option<serde_json::Value>, Option<String>)> =
        sqlx::query_as("SELECT user_id, ocr_extracted, ocr_merchant FROM receipts WHERE id = $1")
            .bind(receipt_id)
            .fetch_optional(&s.pool)
            .await?;
    let (owner, extracted, ocr_merchant) = row.ok_or(ApiError::NotFound)?;
    if owner != user.id {
        return Err(ApiError::Forbidden);
    }
    let mut extracted = extracted.unwrap_or_else(|| serde_json::json!({}));
    let items = extracted
        .get_mut("items")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| ApiError::BadRequest("receipt has no extracted items".into()))?;
    if idx >= items.len() {
        return Err(ApiError::BadRequest(format!(
            "item index {idx} out of range (have {})",
            items.len()
        )));
    }
    let item = &mut items[idx];
    let item_obj = item
        .as_object_mut()
        .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("item is not an object")))?;

    // Snapshot the existing category BEFORE applying changes so the
    // learning hook fires only on a real change. Re-saving the same
    // category should not inflate use_count.
    let old_category = item_obj
        .get("category")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if let Some(name) = body.name {
        item_obj.insert("name".into(), serde_json::Value::String(name));
    }
    if let Some(t) = body.line_total {
        item_obj.insert(
            "line_total".into(),
            serde_json::Value::String(t.to_string()),
        );
    }
    if let Some(q) = body.qty {
        match q {
            Some(v) => {
                item_obj.insert("qty".into(), serde_json::Value::String(v.to_string()));
            }
            None => {
                item_obj.insert("qty".into(), serde_json::Value::Null);
            }
        }
    }
    if let Some(u) = body.unit_price {
        match u {
            Some(v) => {
                item_obj.insert(
                    "unit_price".into(),
                    serde_json::Value::String(v.to_string()),
                );
            }
            None => {
                item_obj.insert("unit_price".into(), serde_json::Value::Null);
            }
        }
    }
    if let Some(c) = body.category {
        item_obj.insert("category".into(), serde_json::Value::String(c));
    }
    if let Some(b) = body.tax_bucket {
        item_obj.insert("tax_bucket".into(), serde_json::Value::String(b));
    }
    if let Some(prop) = body.rental_property_id {
        match prop {
            Some(uuid) => {
                item_obj.insert(
                    "rental_property_id".into(),
                    serde_json::Value::String(uuid.to_string()),
                );
            }
            None => {
                item_obj.insert("rental_property_id".into(), serde_json::Value::Null);
            }
        }
    }

    // Re-read the new category AFTER the writes so we compare against
    // what's actually being persisted, not the request body (which
    // might be None when the user only updated qty / unit_price).
    let new_category = item_obj
        .get("category")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    sqlx::query("UPDATE receipts SET ocr_extracted = $2 WHERE id = $1")
        .bind(receipt_id)
        .bind(&extracted)
        .execute(&s.pool)
        .await?;

    // Learning hook: if the user changed the category and we have a
    // merchant to attribute it to, record (canonical_merchant,
    // category) → use_count++. Fired here (not in the parse pipeline)
    // because this is the only signal that the chosen category was
    // human-confirmed, not auto-guessed.
    if let (Some(new_cat), Some(merchant_raw)) = (new_category, ocr_merchant) {
        if old_category.as_deref() != Some(new_cat.as_str()) && !merchant_raw.is_empty() {
            let aliases = crate::merchant::load_aliases(&s.pool, user.id)
                .await
                .unwrap_or_default();
            let canonical = crate::merchant::canonicalize(&merchant_raw, &aliases);
            if !canonical.is_empty() {
                let learn = sqlx::query(
                    "INSERT INTO learned_merchant_categories
                        (user_id, merchant_canonical, category_code, use_count, last_used)
                     VALUES ($1, $2, $3, 1, NOW())
                     ON CONFLICT (user_id, merchant_canonical, category_code)
                     DO UPDATE SET
                        use_count = learned_merchant_categories.use_count + 1,
                        last_used = NOW()",
                )
                .bind(user.id)
                .bind(&canonical)
                .bind(&new_cat)
                .execute(&s.pool)
                .await;
                if let Err(e) = learn {
                    // Logging only — never fail the user's PATCH because
                    // the learning side-effect choked.
                    tracing::warn!(
                        merchant = %canonical,
                        category = %new_cat,
                        err = %e,
                        "learn_merchant_category UPSERT failed",
                    );
                }
            }
        }
    }

    Ok(Json(extracted))
}

// --- per-item POST / DELETE -------------------------------------------
//
// Lets the user fill in items the OCR missed (typical: 2 of 6 items
// extracted, 4 entered manually) and remove items the parser
// hallucinated. The POST shape matches `OcrLineItem` minus the
// auto-derived `category` / `tax_bucket` (server fills those in when
// absent, but the client can override either).

#[derive(Deserialize)]
struct NewItem {
    name: String,
    line_total: Decimal,
    #[serde(default)]
    qty: Option<Decimal>,
    #[serde(default)]
    unit_price: Option<Decimal>,
    /// Optional — derived from `name` when absent (same heuristic the
    /// parser uses on fresh OCR output).
    #[serde(default)]
    category: Option<String>,
    /// Optional — derived from `category` when absent.
    #[serde(default)]
    tax_bucket: Option<String>,
    #[serde(default)]
    rental_property_id: Option<Uuid>,
}

async fn add_receipt_item(
    State(s): State<AppState>,
    user: AuthUser,
    Path(receipt_id): Path<Uuid>,
    Json(body): Json<NewItem>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    // Derive category + bucket if the client didn't pin them.
    let category = body
        .category
        .clone()
        .unwrap_or_else(|| traderview_ocr::parse::guess_category(&body.name));
    let tax_bucket = body.tax_bucket.clone().unwrap_or_else(|| {
        traderview_ocr::parse::default_bucket_for_category(&category).to_string()
    });
    if !matches!(
        tax_bucket.as_str(),
        "business" | "rental" | "personal" | "unclassified"
    ) {
        return Err(ApiError::BadRequest(format!(
            "invalid tax_bucket: {tax_bucket}"
        )));
    }

    let row: Option<(Uuid, Option<serde_json::Value>)> =
        sqlx::query_as("SELECT user_id, ocr_extracted FROM receipts WHERE id = $1")
            .bind(receipt_id)
            .fetch_optional(&s.pool)
            .await?;
    let (owner, extracted) = row.ok_or(ApiError::NotFound)?;
    if owner != user.id {
        return Err(ApiError::Forbidden);
    }
    let mut extracted = extracted.unwrap_or_else(|| serde_json::json!({}));
    // Ensure the items array exists — older rows (pre-migration 0041)
    // may have a `{}` blob without `items`.
    let ext_obj = extracted
        .as_object_mut()
        .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("extracted is not an object")))?;
    let items = ext_obj
        .entry("items")
        .or_insert_with(|| serde_json::Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("items is not an array")))?;

    let mut new_obj = serde_json::Map::new();
    new_obj.insert("name".into(), serde_json::Value::String(body.name));
    new_obj.insert(
        "line_total".into(),
        serde_json::Value::String(body.line_total.to_string()),
    );
    new_obj.insert(
        "qty".into(),
        body.qty
            .map(|v| serde_json::Value::String(v.to_string()))
            .unwrap_or(serde_json::Value::Null),
    );
    new_obj.insert(
        "unit_price".into(),
        body.unit_price
            .map(|v| serde_json::Value::String(v.to_string()))
            .unwrap_or(serde_json::Value::Null),
    );
    new_obj.insert("category".into(), serde_json::Value::String(category));
    new_obj.insert("tax_bucket".into(), serde_json::Value::String(tax_bucket));
    new_obj.insert(
        "rental_property_id".into(),
        body.rental_property_id
            .map(|u| serde_json::Value::String(u.to_string()))
            .unwrap_or(serde_json::Value::Null),
    );
    items.push(serde_json::Value::Object(new_obj));

    sqlx::query("UPDATE receipts SET ocr_extracted = $2 WHERE id = $1")
        .bind(receipt_id)
        .bind(&extracted)
        .execute(&s.pool)
        .await?;
    Ok(Json(extracted))
}

async fn delete_receipt_item(
    State(s): State<AppState>,
    user: AuthUser,
    Path((receipt_id, idx)): Path<(Uuid, usize)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let row: Option<(Uuid, Option<serde_json::Value>)> =
        sqlx::query_as("SELECT user_id, ocr_extracted FROM receipts WHERE id = $1")
            .bind(receipt_id)
            .fetch_optional(&s.pool)
            .await?;
    let (owner, extracted) = row.ok_or(ApiError::NotFound)?;
    if owner != user.id {
        return Err(ApiError::Forbidden);
    }
    let mut extracted = extracted.unwrap_or_else(|| serde_json::json!({}));
    let items = extracted
        .get_mut("items")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| ApiError::BadRequest("receipt has no items".into()))?;
    if idx >= items.len() {
        return Err(ApiError::BadRequest(format!(
            "item index {idx} out of range (have {})",
            items.len()
        )));
    }
    items.remove(idx);

    sqlx::query("UPDATE receipts SET ocr_extracted = $2 WHERE id = $1")
        .bind(receipt_id)
        .bind(&extracted)
        .execute(&s.pool)
        .await?;
    Ok(Json(extracted))
}

// --- bulk auto-attach --------------------------------------------------
//
// Pass over every `done` receipt that's not yet attached, score it
// against transactions within ±7 days, and attach the top candidate IF
// its score meets the `threshold`. Used after a 10k-receipt import to
// auto-link the easy ones in one operation. Receipts whose best score
// falls below the threshold are left untouched for manual review.

#[derive(Deserialize)]
struct BulkAttachBody {
    /// Minimum match score (0.0..=1.0). Default 0.85 — empirically the
    /// boundary between "near-certain right answer" and "needs eyes".
    #[serde(default)]
    threshold: Option<f32>,
    /// Optional date range — same shape as the list endpoint.
    #[serde(default)]
    from: Option<NaiveDate>,
    #[serde(default)]
    to: Option<NaiveDate>,
}

#[derive(Serialize)]
struct BulkAttachResult {
    examined: u32,
    attached: u32,
    skipped_no_candidates: u32,
    skipped_low_score: u32,
}

async fn bulk_attach(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<BulkAttachBody>,
) -> Result<Json<BulkAttachResult>, ApiError> {
    let threshold = body.threshold.unwrap_or(0.85).clamp(0.0, 1.0);

    // Pull every unattached `done` receipt in scope. Streaming would be
    // nicer for 10k+ but `fetch_all` is simpler and a Receipt row is
    // small (~hundred bytes once we trim ocr_text out below).
    let receipts: Vec<(Uuid, Option<String>, Option<Decimal>, Option<NaiveDate>)> = sqlx::query_as(
        "SELECT id, ocr_merchant, ocr_total, ocr_date
               FROM receipts
              WHERE user_id = $1
                AND ocr_status = 'done'::ocr_status_t
                AND transaction_id IS NULL
                AND ($2::date IS NULL OR ocr_date >= $2)
                AND ($3::date IS NULL OR ocr_date <= $3)",
    )
    .bind(user.id)
    .bind(body.from)
    .bind(body.to)
    .fetch_all(&s.pool)
    .await?;

    let mut result = BulkAttachResult {
        examined: receipts.len() as u32,
        attached: 0,
        skipped_no_candidates: 0,
        skipped_low_score: 0,
    };

    for (rid, merchant, total, date) in receipts {
        // Date is the strongest filter; if missing, widen to last 30 days.
        let candidates: Vec<TxBrief> = if let Some(d) = date {
            sqlx::query_as(
                "SELECT t.id, t.account_id, t.posted_at, t.amount,
                        t.merchant_raw, t.merchant_normalized
                   FROM transactions t
                   JOIN financial_accounts a ON a.id = t.account_id
                  WHERE a.user_id = $1
                    AND t.posted_at >= ($2::date - INTERVAL '7 days')
                    AND t.posted_at <  ($2::date + INTERVAL '8 days')
                    AND t.is_transfer = FALSE
                  LIMIT 500",
            )
            .bind(user.id)
            .bind(d)
            .fetch_all(&s.pool)
            .await?
        } else {
            sqlx::query_as(
                "SELECT t.id, t.account_id, t.posted_at, t.amount,
                        t.merchant_raw, t.merchant_normalized
                   FROM transactions t
                   JOIN financial_accounts a ON a.id = t.account_id
                  WHERE a.user_id = $1
                    AND t.posted_at >= now() - INTERVAL '30 days'
                    AND t.is_transfer = FALSE
                  LIMIT 500",
            )
            .bind(user.id)
            .fetch_all(&s.pool)
            .await?
        };

        if candidates.is_empty() {
            result.skipped_no_candidates += 1;
            continue;
        }

        let receipt_fields = matcher::ReceiptFields {
            merchant,
            total,
            date,
        };
        let tx_cands: Vec<matcher::TxCandidate> = candidates
            .iter()
            .map(|c| matcher::TxCandidate {
                id: c.id,
                posted_date: c.posted_at.date_naive(),
                amount: c.amount,
                merchant_normalized: c.merchant_normalized.clone(),
            })
            .collect();
        let scored = matcher::score(&receipt_fields, &tx_cands, threshold);
        let best = scored.into_iter().next();
        match best {
            Some(m) if m.score >= threshold => {
                sqlx::query(
                    "UPDATE receipts
                        SET transaction_id = $2, match_score = $3
                      WHERE id = $1",
                )
                .bind(rid)
                .bind(m.id)
                .bind(m.score)
                .execute(&s.pool)
                .await?;
                result.attached += 1;
            }
            _ => result.skipped_low_score += 1,
        }
    }

    Ok(Json(result))
}

// --- bulk delete --------------------------------------------------------

#[derive(Deserialize)]
struct BulkIdsBody {
    ids: Vec<Uuid>,
}

#[derive(Serialize)]
struct BulkResult {
    affected: u32,
}

async fn bulk_delete(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<BulkIdsBody>,
) -> Result<Json<BulkResult>, ApiError> {
    if body.ids.is_empty() {
        return Ok(Json(BulkResult { affected: 0 }));
    }
    // Cap the batch so a runaway client can't issue a 10M-row DELETE in
    // one call. 500 is the same cap the list endpoint uses for read.
    if body.ids.len() > 500 {
        return Err(ApiError::BadRequest(
            "ids exceeds 500 — split into multiple requests".into(),
        ));
    }
    // ON DELETE CASCADE on the blob path doesn't exist (file lives on
    // disk). We delete the DB rows first so the cascade-safety stays
    // intact; orphan blobs are GC'd by a background sweeper. For now
    // the orphan-cleanup is intentional dead weight — the
    // dedup-by-sha layer means re-upload reclaims them anyway.
    let r = sqlx::query("DELETE FROM receipts WHERE user_id = $1 AND id = ANY($2)")
        .bind(user.id)
        .bind(&body.ids)
        .execute(&s.pool)
        .await?;
    Ok(Json(BulkResult {
        affected: r.rows_affected() as u32,
    }))
}

// --- bulk re-OCR --------------------------------------------------------
//
// Reset matching receipts to ocr_status='pending' and spawn the
// existing `run_ocr` pipeline for each. Bounded by the same
// `state.ocr_sem` semaphore as upload-time OCR so a re-OCR of 10k
// receipts can't out-compete a fresh upload for CPU.
//
// Filters:
//   * `all`              — every `done`/`failed` receipt owned by user.
//   * `non_vision`       — anything not currently `apple_vision`.
//                          Right answer after the Vision sidecar lands.
//   * `low_confidence`   — confidence < 0.80. Catches Tesseract pages
//                          where the text came out garbled.
//   * `failed`           — only receipts the engine choked on.

#[derive(Deserialize)]
struct BulkReocrBody {
    /// One of: `all` | `non_vision` | `low_confidence` | `failed`.
    /// Default `non_vision` — the most common reason to bulk-reocr is
    /// "I just installed the Vision sidecar, re-process anything that
    /// ran on Tesseract."
    #[serde(default = "default_reocr_filter")]
    filter: String,
}

fn default_reocr_filter() -> String {
    "non_vision".into()
}

#[derive(Serialize)]
struct BulkReocrResult {
    queued: u32,
    filter: String,
}

#[derive(Serialize)]
struct ReocrProgress {
    pending: u32,
    done: u32,
    failed: u32,
    needs_image: u32,
}

async fn bulk_reocr(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<BulkReocrBody>,
) -> Result<Json<BulkReocrResult>, ApiError> {
    // Translate the filter into a SQL WHERE fragment. Keeping it
    // server-side prevents the client from issuing a re-OCR of all
    // receipts when it only meant low-confidence — no surprise CPU
    // spikes.
    let where_extra = match body.filter.as_str() {
        "all" => "",
        "non_vision" => {
            // Anything that DIDN'T involve Apple Vision — catches both
            // single-engine tesseract results and old single-vision
            // results predating the ensemble. Ensemble results contain
            // `apple_vision` in the engine string so they're (correctly)
            // skipped.
            "AND COALESCE(ocr_extracted->>'engine','') NOT LIKE '%apple_vision%'"
        }
        "non_ensemble" => {
            // Anything not produced by the ensemble path — useful for
            // upgrading old single-engine receipts after the ensemble
            // rollout. Ensemble results all start with "ensemble:".
            "AND COALESCE(ocr_extracted->>'engine','') NOT LIKE 'ensemble:%'"
        }
        "low_confidence" => "AND COALESCE(ocr_confidence, 0) < 0.80",
        "failed" => "AND ocr_status = 'failed'::ocr_status_t",
        other => {
            return Err(ApiError::BadRequest(format!(
                "unknown filter '{other}' — use one of: all | non_vision | non_ensemble | low_confidence | failed"
            )));
        }
    };

    // Pull the matching ids + storage paths in one shot. We cap at
    // 10_000 so a runaway button-mash can't pin the OCR semaphore for
    // hours; the user can re-issue the call to keep draining.
    let sql = format!(
        "SELECT id, storage_path, mime
           FROM receipts
          WHERE user_id = $1
            AND ocr_status IN ('done','failed','needs_image')
            {where_extra}
          ORDER BY created_at DESC
          LIMIT 10000",
    );
    let rows: Vec<(Uuid, String, String)> = sqlx::query_as(&sql)
        .bind(user.id)
        .fetch_all(&s.pool)
        .await?;

    let queued = rows.len() as u32;

    // Flip status to 'pending' for the whole batch in one statement so
    // the UI sees the progress bar move immediately rather than per-row.
    if !rows.is_empty() {
        let ids: Vec<Uuid> = rows.iter().map(|(id, _, _)| *id).collect();
        sqlx::query(
            "UPDATE receipts
                SET ocr_status = 'pending'::ocr_status_t,
                    error_message = NULL
              WHERE id = ANY($1)",
        )
        .bind(&ids)
        .execute(&s.pool)
        .await?;
    }

    // Spawn one task per receipt. Each task acquires a semaphore permit
    // before invoking `run_ocr`, so the actual concurrency stays
    // bounded at `min(4, num_cpus)`. The spawn loop itself returns
    // immediately — the client polls /bulk-reocr/progress.
    for (id, rel_path, mime) in rows {
        let bg_state = s.clone();
        tokio::spawn(async move {
            let _permit = bg_state.ocr_sem.clone().acquire_owned().await.ok();
            let abs = bg_state.receipts_dir().join(&rel_path);
            let bytes = match tokio::fs::read(&abs).await {
                Ok(b) => b,
                Err(e) => {
                    tracing::error!(receipt = %id, error = %e, "bulk reocr: read blob failed");
                    mark_ocr_failed(&bg_state, id, &format!("read blob: {e}")).await;
                    return;
                }
            };
            let result = std::panic::AssertUnwindSafe(run_ocr(bg_state.clone(), id, bytes, mime));
            match futures_util::FutureExt::catch_unwind(result).await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    tracing::error!(receipt = %id, error = %e, "bulk reocr failed");
                    mark_ocr_failed(&bg_state, id, &e.to_string()).await;
                }
                Err(panic) => {
                    let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                        (*s).to_string()
                    } else if let Some(s) = panic.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "OCR engine panicked".into()
                    };
                    tracing::error!(receipt = %id, panic = %msg, "bulk reocr panicked");
                    mark_ocr_failed(&bg_state, id, &format!("OCR panic: {msg}")).await;
                }
            }
        });
    }

    Ok(Json(BulkReocrResult {
        queued,
        filter: body.filter,
    }))
}

async fn bulk_reocr_progress(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<ReocrProgress>, ApiError> {
    // Single aggregate query — Postgres FILTER is cheaper than four
    // round-trips. The client polls this every 2-3s during a bulk job.
    let row: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT
            COUNT(*) FILTER (WHERE ocr_status = 'pending'::ocr_status_t),
            COUNT(*) FILTER (WHERE ocr_status = 'done'::ocr_status_t),
            COUNT(*) FILTER (WHERE ocr_status = 'failed'::ocr_status_t),
            COUNT(*) FILTER (WHERE ocr_status = 'needs_image'::ocr_status_t)
           FROM receipts
          WHERE user_id = $1",
    )
    .bind(user.id)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(ReocrProgress {
        pending: row.0 as u32,
        done: row.1 as u32,
        failed: row.2 as u32,
        needs_image: row.3 as u32,
    }))
}

// --- by-merchant grouping (Track B4) ----------------------------------
//
// Powers the "Categorize by merchant" view. Walks every item across
// the user's done receipts, canonicalizes each receipt's merchant,
// groups items by canonical, returns one row per merchant with its
// receipt_ids ready to be fed back into /bulk-patch-items.
//
// Query params:
//   * `default_only=1` — restrict to items whose current category is a
//     parser default (`unclassified`, `office_supplies`). Off by
//     default — caller can also ask for the full grouping (handy for
//     duplicate-receipt detection, top-merchants reports).
//   * `min_items=N` — drop groups below N items. Default 1.

#[derive(Deserialize)]
struct ByMerchantParams {
    #[serde(default)]
    default_only: Option<u8>,
    #[serde(default)]
    business_id: Option<Uuid>,
    #[serde(default)]
    min_items: Option<u32>,
}

#[derive(Serialize)]
struct ByMerchantGroup {
    canonical_merchant: String,
    receipt_ids: Vec<Uuid>,
    item_count: u32,
    receipt_count: u32,
    total: Decimal,
    /// First 6 line-item names so the UI can show a preview without
    /// fetching every receipt — `["paper towels", "milk", ...]`.
    sample_items: Vec<String>,
    /// Top learned category for this merchant, when one exists. Lets
    /// the UI pre-fill the dropdown.
    learned_category: Option<String>,
}

async fn receipts_by_merchant(
    State(s): State<AppState>,
    user: AuthUser,
    Query(params): Query<ByMerchantParams>,
) -> Result<Json<Vec<ByMerchantGroup>>, ApiError> {
    let default_only = params.default_only.unwrap_or(0) != 0;
    let min_items = params.min_items.unwrap_or(1).max(1);

    // Slurp every (receipt_id, merchant, items[]) for the user. Items
    // come back as a JSONB array — we walk it in Rust so the
    // canonicalize step can apply alias rules + match learned cats.
    let rows: Vec<(Uuid, Option<String>, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT id, ocr_merchant, ocr_extracted
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND ocr_merchant IS NOT NULL
            AND ocr_extracted IS NOT NULL",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;

    let aliases = crate::merchant::load_aliases(&s.pool, user.id)
        .await
        .unwrap_or_default();

    use std::collections::HashMap;
    struct Acc {
        receipt_ids: Vec<Uuid>,
        item_count: u32,
        total: Decimal,
        sample_items: Vec<String>,
    }
    let mut by_canon: HashMap<String, Acc> = HashMap::new();

    for (receipt_id, merchant_raw, extracted) in rows {
        let Some(merchant_raw) = merchant_raw else {
            continue;
        };
        let Some(extracted) = extracted else { continue };
        let canonical = crate::merchant::canonicalize(&merchant_raw, &aliases);
        if canonical.is_empty() {
            continue;
        }
        let items = match extracted.get("items").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => continue,
        };

        let entry = by_canon.entry(canonical).or_insert_with(|| Acc {
            receipt_ids: Vec::new(),
            item_count: 0,
            total: Decimal::ZERO,
            sample_items: Vec::new(),
        });
        let mut counted_this_receipt = false;

        for item in items {
            if !item_in_business(item, params.business_id) {
                continue;
            }
            // default_only: skip items the user has already triaged.
            if default_only {
                let cat = item.get("category").and_then(|v| v.as_str()).unwrap_or("");
                if !matches!(cat, "" | "unclassified" | "office_supplies") {
                    continue;
                }
            }
            entry.item_count += 1;

            // Line totals come back as JSON strings (Decimal does not
            // round-trip through f64 cleanly). Best-effort parse —
            // missing/garbage totals just don't contribute.
            if let Some(total_str) = item.get("line_total").and_then(|v| v.as_str()) {
                if let Ok(d) = total_str.parse::<Decimal>() {
                    entry.total += d;
                }
            }

            // Keep up to 6 sample names per group for the preview.
            if entry.sample_items.len() < 6 {
                if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                    if !name.is_empty() && !entry.sample_items.iter().any(|s| s == name) {
                        entry.sample_items.push(name.to_string());
                    }
                }
            }

            if !counted_this_receipt {
                entry.receipt_ids.push(receipt_id);
                counted_this_receipt = true;
            }
        }
    }

    // Look up learned-category top hit per merchant in one batch query.
    let merchants: Vec<String> = by_canon.keys().cloned().collect();
    let mut learned_map: HashMap<String, String> = HashMap::new();
    if !merchants.is_empty() {
        let learned: Vec<(String, String, i32)> = sqlx::query_as(
            "SELECT merchant_canonical, category_code, use_count
               FROM learned_merchant_categories
              WHERE user_id = $1
                AND merchant_canonical = ANY($2)",
        )
        .bind(user.id)
        .bind(&merchants)
        .fetch_all(&s.pool)
        .await
        .unwrap_or_default();

        // Keep only the highest-use_count entry per merchant.
        let mut best: HashMap<String, (String, i32)> = HashMap::new();
        for (m, cat, n) in learned {
            let entry = best.entry(m).or_insert_with(|| (cat.clone(), n));
            if n > entry.1 {
                *entry = (cat, n);
            }
        }
        for (m, (cat, _)) in best {
            learned_map.insert(m, cat);
        }
    }

    let mut out: Vec<ByMerchantGroup> = by_canon
        .into_iter()
        .filter(|(_, a)| a.item_count >= min_items)
        .map(|(canonical, acc)| ByMerchantGroup {
            receipt_count: acc.receipt_ids.len() as u32,
            learned_category: learned_map.get(&canonical).cloned(),
            canonical_merchant: canonical,
            receipt_ids: acc.receipt_ids,
            item_count: acc.item_count,
            total: acc.total,
            sample_items: acc.sample_items,
        })
        .collect();

    // Sort by item_count DESC — the most-impactful groups first.
    out.sort_by_key(|g| std::cmp::Reverse(g.item_count));

    Ok(Json(out))
}

// --- full-text search (Track C3) --------------------------------------
//
// Uses the `ocr_text_tsv` tsvector column added in migration 0044 and
// its GIN index. Search is `plainto_tsquery` so the user can type
// anything — no need to escape `&`, `|`, `(`, etc. ts_headline gives
// us a highlighted snippet for the UI.

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    #[serde(default)]
    limit: Option<u32>,
    #[serde(default)]
    business_id: Option<Uuid>,
}

#[derive(Serialize)]
struct SearchHit {
    id: Uuid,
    merchant: Option<String>,
    total: Option<Decimal>,
    date: Option<NaiveDate>,
    rank: f32,
    snippet: String,
    transaction_id: Option<Uuid>,
}

async fn receipts_search(
    State(s): State<AppState>,
    user: AuthUser,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchHit>>, ApiError> {
    let q = params.q.trim().to_string();
    if q.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let limit = params.limit.unwrap_or(50).clamp(1, 200) as i64;

    // ts_headline operates on the raw text; we feed the same tsquery
    // it's matching against so highlights stay aligned. StartSel /
    // StopSel use literal tokens the UI escapes itself — never inject
    // raw HTML from Postgres into innerHTML.
    let rows: Vec<(
        Uuid,
        Option<String>,
        Option<Decimal>,
        Option<NaiveDate>,
        f32,
        String,
        Option<Uuid>,
    )> = sqlx::query_as(
        "SELECT id, ocr_merchant, ocr_total, ocr_date,
                ts_rank(ocr_text_tsv, plainto_tsquery('english', $2)) AS rank,
                ts_headline(
                    'english',
                    COALESCE(ocr_text, ''),
                    plainto_tsquery('english', $2),
                    'MaxFragments=2, MaxWords=12, MinWords=5, StartSel=«, StopSel=»'
                ) AS snippet,
                transaction_id
           FROM receipts
          WHERE user_id = $1
            AND ocr_text_tsv @@ plainto_tsquery('english', $2)
            AND ($4::uuid IS NULL OR business_id = $4::uuid)
          ORDER BY rank DESC
          LIMIT $3",
    )
    .bind(user.id)
    .bind(&q)
    .bind(limit)
    .bind(params.business_id)
    .fetch_all(&s.pool)
    .await?;

    let hits = rows
        .into_iter()
        .map(
            |(id, merchant, total, date, rank, snippet, transaction_id)| SearchHit {
                id,
                merchant,
                total,
                date,
                rank,
                snippet,
                transaction_id,
            },
        )
        .collect();
    Ok(Json(hits))
}

// --- duplicate receipt detector (Track C2) ----------------------------
//
// "I uploaded the same receipt twice" is a real problem at scale —
// folder scans, retry uploads, re-photographing a previously archived
// pile. We can't dedupe by SHA (a slightly different photo of the
// same receipt has a different SHA), but we CAN dedupe by
// (canonical_merchant, total ±$0.50, date ±N days).
//
// Returns groups of ≥2 receipts that look like duplicates of each
// other. UI shows side-by-side, "Keep one, delete the rest".

#[derive(Deserialize)]
struct DuplicatesParams {
    #[serde(default)]
    within_days: Option<u32>,
    /// $ tolerance on total. Defaults to 0.50 — handles rare tax /
    /// rounding drift between an itemized photo and a Square tab.
    #[serde(default)]
    total_tolerance: Option<f32>,
    #[serde(default)]
    business_id: Option<Uuid>,
}

#[derive(Serialize)]
struct DuplicateGroup {
    canonical_merchant: String,
    total: Decimal,
    receipts: Vec<DuplicateReceipt>,
}

#[derive(Serialize)]
struct DuplicateReceipt {
    id: Uuid,
    filename: String,
    ocr_merchant: Option<String>,
    ocr_date: Option<NaiveDate>,
    ocr_total: Option<Decimal>,
    transaction_id: Option<Uuid>,
}

async fn receipts_duplicates(
    State(s): State<AppState>,
    user: AuthUser,
    Query(params): Query<DuplicatesParams>,
) -> Result<Json<Vec<DuplicateGroup>>, ApiError> {
    let within_days = params.within_days.unwrap_or(3).min(30) as i64;
    let tol_cents = (params.total_tolerance.unwrap_or(0.50) * 100.0).round() as i64;
    let tol_dollars = Decimal::new(tol_cents, 2);

    let rows: Vec<(
        Uuid,
        String,
        Option<String>,
        Option<NaiveDate>,
        Option<Decimal>,
        Option<Uuid>,
    )> = sqlx::query_as(
        "SELECT id, filename, ocr_merchant, ocr_date, ocr_total, transaction_id
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND ocr_total IS NOT NULL
            AND ocr_merchant IS NOT NULL
            AND ocr_date IS NOT NULL
            AND ($2::uuid IS NULL OR business_id = $2::uuid)",
    )
    .bind(user.id)
    .bind(params.business_id)
    .fetch_all(&s.pool)
    .await?;

    let aliases = crate::merchant::load_aliases(&s.pool, user.id)
        .await
        .unwrap_or_default();

    // Two-stage grouping: canonical_merchant → buckets keyed by
    // rounded total. Within each bucket, walk pairwise and emit
    // connected components where date diff ≤ within_days.
    use std::collections::HashMap;
    struct R {
        id: Uuid,
        filename: String,
        merchant: Option<String>,
        date: NaiveDate,
        total: Decimal,
        transaction_id: Option<Uuid>,
        canonical: String,
    }
    let mut all: Vec<R> = Vec::with_capacity(rows.len());
    for (id, filename, merchant, date, total, txn) in rows {
        let date = match date {
            Some(d) => d,
            None => continue,
        };
        let total = match total {
            Some(t) => t,
            None => continue,
        };
        let raw = merchant.clone().unwrap_or_default();
        let canonical = if raw.is_empty() {
            continue;
        } else {
            crate::merchant::canonicalize(&raw, &aliases)
        };
        if canonical.is_empty() {
            continue;
        }
        all.push(R {
            id,
            filename,
            merchant,
            date,
            total,
            transaction_id: txn,
            canonical,
        });
    }

    // Group by canonical first.
    let mut by_canon: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, r) in all.iter().enumerate() {
        by_canon.entry(r.canonical.clone()).or_default().push(i);
    }

    let mut groups: Vec<DuplicateGroup> = Vec::new();
    for (canon, idxs) in by_canon {
        if idxs.len() < 2 {
            continue;
        }
        // Naive O(n²) — fine for ≤200 receipts per merchant; the
        // user-corpus shape rarely concentrates that hard.
        let mut consumed = vec![false; idxs.len()];
        for i in 0..idxs.len() {
            if consumed[i] {
                continue;
            }
            let mut hits: Vec<usize> = vec![idxs[i]];
            consumed[i] = true;
            for j in (i + 1)..idxs.len() {
                if consumed[j] {
                    continue;
                }
                let a = &all[idxs[i]];
                let b = &all[idxs[j]];
                let date_ok = (a.date - b.date).num_days().abs() <= within_days;
                let total_ok = (a.total - b.total).abs() <= tol_dollars;
                if date_ok && total_ok {
                    hits.push(idxs[j]);
                    consumed[j] = true;
                }
            }
            if hits.len() >= 2 {
                let receipts = hits
                    .iter()
                    .map(|&k| {
                        let r = &all[k];
                        DuplicateReceipt {
                            id: r.id,
                            filename: r.filename.clone(),
                            ocr_merchant: r.merchant.clone(),
                            ocr_date: Some(r.date),
                            ocr_total: Some(r.total),
                            transaction_id: r.transaction_id,
                        }
                    })
                    .collect::<Vec<_>>();
                let total_anchor = all[hits[0]].total;
                groups.push(DuplicateGroup {
                    canonical_merchant: canon.clone(),
                    total: total_anchor,
                    receipts,
                });
            }
        }
    }

    // Largest duplicate groups first so the user attacks the highest-leverage cleanup.
    groups.sort_by_key(|g| std::cmp::Reverse(g.receipts.len()));
    Ok(Json(groups))
}

// --- recurring expense detection ---------------------------------------
//
// Finds merchants where the user has a regular cadence of receipts —
// the classic "find your subscriptions" problem. A recurring vendor is
// one with ≥ MIN_OCCURRENCES receipts whose median gap between
// consecutive receipt dates falls within a target cadence window
// (monthly ± a tolerance). Returns the merchant, predicted next charge
// date, monthly average amount, and a confidence score.
//
// This is purely on the receipts table — no LLM, no external API.

#[derive(Deserialize)]
struct RecurringParams {
    /// Minimum number of receipts per merchant to consider. Default 3.
    min_occurrences: Option<u32>,
    /// Cadence target in days. Default 30 (monthly). Common values: 7
    /// (weekly), 30 (monthly), 90 (quarterly), 365 (annual).
    cadence_days: Option<u32>,
    /// Tolerance (± days) around `cadence_days`. Default 5.
    tolerance_days: Option<u32>,
    business_id: Option<Uuid>,
}

#[derive(Serialize)]
struct RecurringMerchant {
    canonical_merchant: String,
    receipt_count: u32,
    average_amount: Decimal,
    median_gap_days: i64,
    first_seen: NaiveDate,
    last_seen: NaiveDate,
    /// `last_seen + median_gap_days`, telling the user when the next
    /// charge is expected.
    predicted_next_date: NaiveDate,
    /// 0.0–1.0, derived from gap consistency. Low standard deviation of
    /// gaps → higher confidence.
    confidence: f32,
    annualized_cost: Decimal,
}

async fn receipts_recurring(
    State(s): State<AppState>,
    user: AuthUser,
    Query(params): Query<RecurringParams>,
) -> Result<Json<Vec<RecurringMerchant>>, ApiError> {
    let min_occ = params.min_occurrences.unwrap_or(3).max(2) as usize;
    let cadence = params.cadence_days.unwrap_or(30) as i64;
    let tolerance = params.tolerance_days.unwrap_or(5) as i64;

    let rows: Vec<(Option<String>, Option<NaiveDate>, Option<Decimal>)> = sqlx::query_as(
        "SELECT ocr_merchant, ocr_date, ocr_total
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND ocr_merchant IS NOT NULL
            AND ocr_date IS NOT NULL
            AND ocr_total IS NOT NULL
            AND ($2::uuid IS NULL OR business_id = $2::uuid)",
    )
    .bind(user.id)
    .bind(params.business_id)
    .fetch_all(&s.pool)
    .await?;

    let aliases = crate::merchant::load_aliases(&s.pool, user.id)
        .await
        .unwrap_or_default();

    use std::collections::HashMap;
    let mut by_canon: HashMap<String, Vec<(NaiveDate, Decimal)>> = HashMap::new();
    for (m, d, t) in rows {
        let (Some(m), Some(d), Some(t)) = (m, d, t) else {
            continue;
        };
        let canonical = crate::merchant::canonicalize(&m, &aliases);
        if canonical.is_empty() {
            continue;
        }
        by_canon.entry(canonical).or_default().push((d, t));
    }

    let mut out = Vec::new();
    for (canonical, mut entries) in by_canon {
        if entries.len() < min_occ {
            continue;
        }
        entries.sort_by_key(|e| e.0);

        let gaps: Vec<i64> = entries
            .windows(2)
            .map(|pair| (pair[1].0 - pair[0].0).num_days())
            .filter(|d| *d > 0)
            .collect();

        if gaps.len() < min_occ - 1 {
            continue;
        }

        let median_gap = {
            let mut g = gaps.clone();
            g.sort_unstable();
            g[g.len() / 2]
        };

        if (median_gap - cadence).abs() > tolerance {
            continue;
        }

        // Confidence: lower stddev around median → higher score. Map
        // mean-absolute-deviation to [0, 1] using a soft cliff at
        // tolerance_days.
        let mad: f32 = if gaps.is_empty() {
            0.0
        } else {
            let sum: i64 = gaps.iter().map(|g| (*g - median_gap).abs()).sum();
            sum as f32 / gaps.len() as f32
        };
        let confidence = (1.0 - (mad / tolerance.max(1) as f32)).clamp(0.0, 1.0);

        let total: Decimal = entries.iter().map(|e| e.1).sum();
        let n = Decimal::from(entries.len() as i64);
        let average_amount = (total / n).round_dp(2);

        let first_seen = entries.first().unwrap().0;
        let last_seen = entries.last().unwrap().0;
        let predicted_next_date = last_seen + chrono::Duration::days(median_gap);

        // Annualized cost = average × (365 / median_gap), rounded.
        let annualized_cost = if median_gap > 0 {
            (average_amount * Decimal::from(365) / Decimal::from(median_gap)).round_dp(2)
        } else {
            Decimal::ZERO
        };

        out.push(RecurringMerchant {
            canonical_merchant: canonical,
            receipt_count: entries.len() as u32,
            average_amount,
            median_gap_days: median_gap,
            first_seen,
            last_seen,
            predicted_next_date,
            confidence,
            annualized_cost,
        });
    }

    // Highest annualized cost first — most impactful subscriptions surface at the top.
    out.sort_by_key(|m| std::cmp::Reverse(m.annualized_cost));
    Ok(Json(out))
}

// --- bulk patch items ---------------------------------------------------
//
// Applies a single category / tax_bucket / rental_property_id update
// to EVERY item across a set of receipts. Used when you've imported
// 200 Lowe's receipts and want to bulk-set every item on every one to
// rental → "Maple St duplex".

#[derive(Deserialize)]
struct BulkPatchItemsBody {
    ids: Vec<Uuid>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    tax_bucket: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_uuid")]
    rental_property_id: Option<Option<Uuid>>,
    /// Assign every item across the requested receipts to this
    /// business entity. Pass `null` (explicit) to clear.
    #[serde(default, deserialize_with = "deserialize_optional_uuid")]
    business_id: Option<Option<Uuid>>,
}

async fn bulk_patch_items(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<BulkPatchItemsBody>,
) -> Result<Json<BulkResult>, ApiError> {
    if body.ids.is_empty() {
        return Ok(Json(BulkResult { affected: 0 }));
    }
    if body.ids.len() > 500 {
        return Err(ApiError::BadRequest(
            "ids exceeds 500 — split into multiple requests".into(),
        ));
    }
    if body.category.is_none()
        && body.tax_bucket.is_none()
        && body.rental_property_id.is_none()
        && body.business_id.is_none()
    {
        return Err(ApiError::BadRequest("no fields to update".into()));
    }
    if let Some(b) = body.tax_bucket.as_deref() {
        if !matches!(b, "business" | "rental" | "personal" | "unclassified") {
            return Err(ApiError::BadRequest(format!("invalid tax_bucket: {b}")));
        }
    }

    // Fetch then update each receipt's JSONB. A single `jsonb_set` over
    // 500 rows would be elegant but iterating in Rust keeps the type-
    // checked update consistent with the per-item PATCH handler.
    let rows: Vec<(Uuid, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT id, ocr_extracted FROM receipts
          WHERE user_id = $1 AND id = ANY($2)",
    )
    .bind(user.id)
    .bind(&body.ids)
    .fetch_all(&s.pool)
    .await?;

    let mut affected: u32 = 0;
    for (rid, extracted) in rows {
        let mut ex = extracted.unwrap_or_else(|| serde_json::json!({}));
        let items = ex.get_mut("items").and_then(|v| v.as_array_mut());
        let Some(items) = items else { continue };
        let mut changed = false;
        for it in items.iter_mut() {
            let Some(obj) = it.as_object_mut() else {
                continue;
            };
            if let Some(c) = body.category.as_ref() {
                obj.insert("category".into(), serde_json::Value::String(c.clone()));
                changed = true;
            }
            if let Some(b) = body.tax_bucket.as_ref() {
                obj.insert("tax_bucket".into(), serde_json::Value::String(b.clone()));
                changed = true;
            }
            if let Some(prop) = body.rental_property_id.as_ref() {
                match prop {
                    Some(uuid) => {
                        obj.insert(
                            "rental_property_id".into(),
                            serde_json::Value::String(uuid.to_string()),
                        );
                    }
                    None => {
                        obj.insert("rental_property_id".into(), serde_json::Value::Null);
                    }
                }
                changed = true;
            }
            if let Some(biz) = body.business_id.as_ref() {
                match biz {
                    Some(uuid) => {
                        obj.insert(
                            "business_id".into(),
                            serde_json::Value::String(uuid.to_string()),
                        );
                    }
                    None => {
                        obj.insert("business_id".into(), serde_json::Value::Null);
                    }
                }
                changed = true;
            }
        }
        // Also set the receipt-level business_id when the patch provides
        // one — gives us the indexed column for fast filtering.
        if let Some(biz) = body.business_id.as_ref() {
            sqlx::query("UPDATE receipts SET business_id = $2 WHERE id = $1 AND user_id = $3")
                .bind(rid)
                .bind(*biz)
                .bind(user.id)
                .execute(&s.pool)
                .await?;
        }
        if changed {
            sqlx::query("UPDATE receipts SET ocr_extracted = $2 WHERE id = $1")
                .bind(rid)
                .bind(&ex)
                .execute(&s.pool)
                .await?;
            affected += 1;
        }
    }
    Ok(Json(BulkResult { affected }))
}

// --- tax rollup --------------------------------------------------------

#[derive(Deserialize)]
struct RollupQ {
    /// Inclusive ISO date.
    from: Option<NaiveDate>,
    /// Inclusive ISO date.
    to: Option<NaiveDate>,
    /// Optional business filter — see BundleQ for semantics.
    business_id: Option<Uuid>,
}

#[derive(Serialize)]
struct TaxRollupResponse {
    from: NaiveDate,
    to: NaiveDate,
    receipts_counted: u32,
    items_counted: u32,
    /// `business` (Schedule C) → categories → total.
    business: BucketRollup,
    /// `rental` (Schedule E) → per-property → categories → total.
    rental: RentalRollup,
    personal: BucketRollup,
    unclassified: BucketRollup,
}

#[derive(Serialize, Default)]
struct BucketRollup {
    grand_total: Decimal,
    categories: Vec<CategoryTotal>,
}

#[derive(Serialize, Default)]
struct RentalRollup {
    grand_total: Decimal,
    properties: Vec<PropertyRollup>,
}

#[derive(Serialize)]
struct PropertyRollup {
    property_id: Option<Uuid>,
    property_name: Option<String>,
    grand_total: Decimal,
    categories: Vec<CategoryTotal>,
}

#[derive(Serialize)]
struct CategoryTotal {
    category: String,
    total: Decimal,
    /// IRS Schedule C line number — present when the category maps to
    /// a Schedule C line (Business bucket). `None` for groceries / other.
    #[serde(skip_serializing_if = "Option::is_none")]
    schedule_c_line: Option<&'static str>,
    /// IRS Schedule E line number — present for rental-bucket
    /// categories that map to a specific Schedule E line.
    #[serde(skip_serializing_if = "Option::is_none")]
    schedule_e_line: Option<&'static str>,
}

/// Category id → IRS Schedule C line (sole-prop). Strings rather than
/// `u8` because some lines are `24a` / `24b` / `20a` / `20b` with letter
/// sublines. Returns `None` for groceries / other (non-deductible).
fn schedule_c_line(category: &str) -> Option<&'static str> {
    Some(match category {
        "advertising" => "8",
        "vehicle_fuel" | "vehicle_maintenance" => "9",
        "contract_labor" => "11",
        "office_equipment_software" => "13", // depreciable; line 13
        "insurance" => "15",
        "professional_services" => "17",
        "office_supplies" => "18",
        "rent_lease" => "20", // 20a vehicles / 20b other; collapsed
        "repairs_maintenance" => "21",
        "supplies_cogs" => "22",
        "taxes_licenses_dues" => "23",
        "travel_transport" | "travel_lodging" => "24a",
        "meals" => "24b", // 50% deductible — line 24b
        "utilities" => "25",
        "wages_benefits" => "26",
        "bank_fees" | "education_training" => "27a", // Other Expenses
        _ => return None,
    })
}

/// Category id → IRS Schedule E line (rental). Schedule E has fewer
/// dedicated lines than C; some categories collapse into line 19
/// (Other) when no exact match exists.
fn schedule_e_line(category: &str) -> Option<&'static str> {
    Some(match category {
        "advertising" => "5",
        "vehicle_fuel" | "vehicle_maintenance" | "travel_transport" | "travel_lodging" => "6", // Auto and travel
        "repairs_maintenance" => "14", // Repairs (or 7 Cleaning)
        "insurance" => "9",
        "professional_services" => "10",
        "rent_lease" => "11",                        // Management fees
        "supplies_cogs" | "office_supplies" => "15", // Supplies
        "taxes_licenses_dues" => "16",
        "utilities" => "17",
        "office_equipment_software" => "18", // Depreciation
        "meals" | "contract_labor" | "wages_benefits" | "bank_fees" | "education_training" => "19", // Other
        _ => return None,
    })
}

async fn tax_rollup(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<RollupQ>,
) -> Result<Json<TaxRollupResponse>, ApiError> {
    // Default window: current calendar year.
    let now = Utc::now().date_naive();
    let year = now.year();
    let from = q
        .from
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, 1, 1).unwrap());
    let to = q.to.unwrap_or(now);

    // Pull receipts in window with their extracted JSONB blob.
    let rows: Vec<(Uuid, Option<NaiveDate>, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT id, ocr_date, ocr_extracted
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND (ocr_date IS NULL OR (ocr_date >= $2 AND ocr_date <= $3))",
    )
    .bind(user.id)
    .bind(from)
    .bind(to)
    .fetch_all(&s.pool)
    .await?;

    // Side-load property names so the rental rollup labels read clean.
    let props: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT id, nickname FROM rental_properties WHERE user_id = $1")
            .bind(user.id)
            .fetch_all(&s.pool)
            .await
            .unwrap_or_default();
    let prop_name: std::collections::HashMap<Uuid, String> = props.into_iter().collect();

    // (bucket, category) → total
    let mut business: std::collections::BTreeMap<String, Decimal> = Default::default();
    let mut personal: std::collections::BTreeMap<String, Decimal> = Default::default();
    let mut unclassified: std::collections::BTreeMap<String, Decimal> = Default::default();
    // property_id → category → total
    let mut rental: std::collections::BTreeMap<
        Option<Uuid>,
        std::collections::BTreeMap<String, Decimal>,
    > = Default::default();

    let mut receipts_counted: u32 = 0;
    let mut items_counted: u32 = 0;

    for (_id, _date, extracted) in rows {
        let Some(ext) = extracted else { continue };
        let Some(items) = ext.get("items").and_then(|v| v.as_array()) else {
            continue;
        };
        if items.is_empty() {
            continue;
        }
        receipts_counted += 1;
        for it in items {
            if !item_in_business(it, q.business_id) {
                continue;
            }
            let cat = it
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("other")
                .to_string();
            let bucket = it
                .get("tax_bucket")
                .and_then(|v| v.as_str())
                .unwrap_or("unclassified");
            // line_total is serialized as a numeric string by serde-decimal.
            let total = it
                .get("line_total")
                .and_then(|v| match v {
                    serde_json::Value::String(s) => Decimal::from_str_exact(s).ok(),
                    serde_json::Value::Number(n) => {
                        n.as_f64().and_then(|f| Decimal::try_from(f).ok())
                    }
                    _ => None,
                })
                .unwrap_or(Decimal::ZERO);
            items_counted += 1;
            match bucket {
                "business" => *business.entry(cat).or_default() += total,
                "personal" => *personal.entry(cat).or_default() += total,
                "rental" => {
                    let prop_id = it
                        .get("rental_property_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| Uuid::parse_str(s).ok());
                    *rental.entry(prop_id).or_default().entry(cat).or_default() += total;
                }
                _ => *unclassified.entry(cat).or_default() += total,
            }
        }
    }

    /// Build the per-bucket rollup. `with_c` controls Schedule C line
    /// annotation (true for `business`); `with_e` controls Schedule E
    /// (true for `rental`). Personal / unclassified get neither.
    fn flatten(
        map: std::collections::BTreeMap<String, Decimal>,
        with_c: bool,
        with_e: bool,
    ) -> BucketRollup {
        let mut categories: Vec<CategoryTotal> = map
            .into_iter()
            .map(|(category, total)| CategoryTotal {
                schedule_c_line: if with_c {
                    schedule_c_line(&category)
                } else {
                    None
                },
                schedule_e_line: if with_e {
                    schedule_e_line(&category)
                } else {
                    None
                },
                category,
                total,
            })
            .collect();
        categories.sort_by_key(|c| std::cmp::Reverse(c.total));
        let grand_total = categories.iter().map(|c| c.total).sum();
        BucketRollup {
            grand_total,
            categories,
        }
    }

    let rental_properties: Vec<PropertyRollup> = rental
        .into_iter()
        .map(|(prop_id, cat_map)| {
            let mut categories: Vec<CategoryTotal> = cat_map
                .into_iter()
                .map(|(category, total)| CategoryTotal {
                    schedule_c_line: None,
                    schedule_e_line: schedule_e_line(&category),
                    category,
                    total,
                })
                .collect();
            categories.sort_by_key(|c| std::cmp::Reverse(c.total));
            let grand_total = categories.iter().map(|c| c.total).sum();
            PropertyRollup {
                property_name: prop_id.and_then(|id| prop_name.get(&id).cloned()),
                property_id: prop_id,
                grand_total,
                categories,
            }
        })
        .collect();
    let rental_grand: Decimal = rental_properties.iter().map(|p| p.grand_total).sum();

    Ok(Json(TaxRollupResponse {
        from,
        to,
        receipts_counted,
        items_counted,
        business: flatten(business, true, false),
        rental: RentalRollup {
            grand_total: rental_grand,
            properties: rental_properties,
        },
        personal: flatten(personal, false, false),
        unclassified: flatten(unclassified, false, false),
    }))
}

// ── Expense-dashboard bundle ──────────────────────────────────────────────
//
// Single endpoint that returns every slice the business-expense dashboard
// needs: KPI strip, time-series for the charts, leaderboards, distributions.
// Mirrors the architecture of `dashboard.js::loadAnalyticsBundle` on the
// trading side — one round-trip → all panels render from local data.

#[derive(Deserialize)]
struct BundleQ {
    year: Option<i32>,
    /// Filter to a specific business entity. `None` = aggregated across
    /// all businesses + personal. Item-level filter: items in the
    /// receipt's JSONB are included only when their `business_id`
    /// matches; receipt-level fields (ocr_total) fall through unchanged
    /// when no items have an explicit business_id assignment.
    business_id: Option<Uuid>,
}

/// Shared query parameter shape for endpoints that take only the
/// business filter (no date window). Used by month-calendar etc.
#[derive(Deserialize)]
struct BusinessFilterQ {
    business_id: Option<Uuid>,
}

/// Returns the receipt's effective total under a business filter.
/// When no filter is active, returns `ocr_total` unchanged. When
/// filtering, sums only items matching `wanted_business`. Items
/// without explicit business_id are treated as "personal/none".
fn receipt_total_filtered(
    total: Option<Decimal>,
    extracted: &Option<serde_json::Value>,
    wanted_business: Option<Uuid>,
) -> Decimal {
    match wanted_business {
        None => total.unwrap_or(Decimal::ZERO),
        Some(_) => extracted
            .as_ref()
            .and_then(|ext| ext.get("items"))
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter(|it| item_in_business(it, wanted_business))
                    .map(|it| {
                        it.get("total")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<Decimal>().ok())
                            .unwrap_or(Decimal::ZERO)
                    })
                    .sum()
            })
            .unwrap_or(Decimal::ZERO),
    }
}

/// Returns true when this item belongs to `wanted_business` (or no
/// filter is active). Used by every analytics endpoint that walks
/// `ocr_extracted.items[]`.
fn item_in_business(it: &serde_json::Value, wanted_business: Option<Uuid>) -> bool {
    match wanted_business {
        None => true,
        Some(wanted) => it
            .get("business_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .map(|id| id == wanted)
            .unwrap_or(false),
    }
}

#[derive(Serialize)]
struct ExpenseKpis {
    total: Decimal,
    schedule_c: Decimal,
    schedule_e: Decimal,
    personal: Decimal,
    unclassified: Decimal,
    receipt_count: u32,
    item_count: u32,
    avg_ticket: Decimal,
    avg_daily: Decimal,
    biz_pct: f64,
    deductible_pct: f64,
    burn_rate_monthly: Decimal,
    biggest_receipt: Decimal,
    smallest_receipt: Decimal,
    longest_zero_streak_days: u32,
    longest_consec_spending_days: u32,
}

#[derive(Serialize)]
struct LabeledTotal {
    label: String,
    total: Decimal,
    count: u32,
}

#[derive(Serialize, Clone)]
struct DailyPoint {
    day: NaiveDate,
    total: Decimal,
    count: u32,
    cumulative: Decimal,
}

#[derive(Serialize)]
struct ExpenseBundle {
    year: i32,
    from: NaiveDate,
    to: NaiveDate,
    kpis: ExpenseKpis,
    daily: Vec<DailyPoint>,
    calendar: Vec<DailyPoint>,
    by_dow: Vec<LabeledTotal>,
    by_hour: Vec<LabeledTotal>,
    by_month: Vec<LabeledTotal>,
    by_quarter: Vec<LabeledTotal>,
    by_amount_bucket: Vec<LabeledTotal>,
    by_category: Vec<LabeledTotal>,
    by_tax_bucket: Vec<LabeledTotal>,
    top_merchants_by_total: Vec<LabeledTotal>,
    top_merchants_by_count: Vec<LabeledTotal>,
    weekday_vs_weekend: Vec<LabeledTotal>,
    uncategorized_count: u32,
    uncategorized_total: Decimal,
}

/// Single bundle endpoint feeding the expense dashboard. One DB round-trip
/// for the year's receipts + items; every slice computed in Rust.
async fn expense_dashboard_bundle(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<BundleQ>,
) -> Result<Json<ExpenseBundle>, ApiError> {
    let now = Utc::now().date_naive();
    let year = q.year.unwrap_or(now.year());
    let from = NaiveDate::from_ymd_opt(year, 1, 1)
        .ok_or_else(|| ApiError::BadRequest(format!("year {year} out of NaiveDate range")))?;
    let to = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();

    let rows: Vec<(
        NaiveDate,
        Option<String>,
        Option<Decimal>,
        Option<serde_json::Value>,
        Option<DateTime<Utc>>,
    )> = sqlx::query_as(
        "SELECT ocr_date::date, ocr_merchant, ocr_total, ocr_extracted, created_at
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND ocr_date IS NOT NULL
            AND ocr_date BETWEEN $2 AND $3",
    )
    .bind(user.id)
    .bind(from)
    .bind(to)
    .fetch_all(&s.pool)
    .await?;

    let aliases = crate::merchant::load_aliases(&s.pool, user.id)
        .await
        .unwrap_or_default();

    use std::collections::HashMap;
    let mut daily_total: HashMap<NaiveDate, (Decimal, u32)> = HashMap::new();
    let mut by_dow: [(Decimal, u32); 7] = Default::default();
    let mut by_hour: [(Decimal, u32); 24] = std::array::from_fn(|_| (Decimal::ZERO, 0));
    let mut by_month: [(Decimal, u32); 12] = Default::default();
    let mut by_quarter: [(Decimal, u32); 4] = Default::default();
    let mut by_amount_bucket: [(Decimal, u32); 7] = Default::default();
    let mut by_category: HashMap<String, (Decimal, u32)> = HashMap::new();
    let mut by_tax_bucket: HashMap<String, (Decimal, u32)> = HashMap::new();
    let mut top_merchant_total: HashMap<String, (Decimal, u32)> = HashMap::new();
    let mut weekday_total = (Decimal::ZERO, 0u32);
    let mut weekend_total = (Decimal::ZERO, 0u32);
    let mut schedule_c = Decimal::ZERO;
    let mut schedule_e = Decimal::ZERO;
    let mut personal = Decimal::ZERO;
    let mut unclassified = Decimal::ZERO;
    let mut uncategorized_count: u32 = 0;
    let mut uncategorized_total = Decimal::ZERO;
    let mut item_count: u32 = 0;
    let mut biggest = Decimal::ZERO;
    let mut smallest = Decimal::MAX;

    for (date, merchant, total, extracted, created) in &rows {
        // When a business_id filter is active, recompute the receipt's
        // effective total from filtered items only — `ocr_total` is one
        // number per receipt and would over-count when the receipt
        // mixes business + personal items.
        let receipt_total: Decimal = if q.business_id.is_some() {
            extracted
                .as_ref()
                .and_then(|ext| ext.get("items"))
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter(|it| item_in_business(it, q.business_id))
                        .map(|it| {
                            it.get("total")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<Decimal>().ok())
                                .unwrap_or(Decimal::ZERO)
                        })
                        .sum::<Decimal>()
                })
                .unwrap_or(Decimal::ZERO)
        } else {
            total.unwrap_or(Decimal::ZERO)
        };
        // Skip receipts with zero effective total under the filter so
        // they don't pollute "biggest/smallest" with $0 entries.
        if q.business_id.is_some() && receipt_total <= Decimal::ZERO {
            continue;
        }

        let entry = daily_total.entry(*date).or_insert((Decimal::ZERO, 0));
        entry.0 += receipt_total;
        entry.1 += 1;

        let dow = date.weekday().num_days_from_sunday() as usize;
        by_dow[dow].0 += receipt_total;
        by_dow[dow].1 += 1;
        if dow == 0 || dow == 6 {
            weekend_total.0 += receipt_total;
            weekend_total.1 += 1;
        } else {
            weekday_total.0 += receipt_total;
            weekday_total.1 += 1;
        }

        let mi = (date.month0()) as usize;
        by_month[mi].0 += receipt_total;
        by_month[mi].1 += 1;
        by_quarter[(mi / 3).min(3)].0 += receipt_total;
        by_quarter[(mi / 3).min(3)].1 += 1;

        let bucket = amount_bucket(receipt_total);
        by_amount_bucket[bucket].0 += receipt_total;
        by_amount_bucket[bucket].1 += 1;

        if receipt_total > biggest {
            biggest = receipt_total;
        }
        if receipt_total < smallest {
            smallest = receipt_total;
        }

        if let Some(c) = created {
            let h = c.hour() as usize;
            by_hour[h].0 += receipt_total;
            by_hour[h].1 += 1;
        }

        if let Some(m) = merchant {
            let canonical = crate::merchant::canonicalize(m, &aliases);
            if !canonical.is_empty() {
                let entry = top_merchant_total
                    .entry(canonical)
                    .or_insert((Decimal::ZERO, 0));
                entry.0 += receipt_total;
                entry.1 += 1;
            }
        }

        // Item-level aggregation for category/tax-bucket charts.
        let Some(ext) = extracted else { continue };
        let Some(items) = ext.get("items").and_then(|v| v.as_array()) else {
            continue;
        };
        for it in items {
            if !item_in_business(it, q.business_id) {
                continue;
            }
            item_count += 1;
            let cat = it
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("uncategorized")
                .to_string();
            let bucket = it
                .get("tax_bucket")
                .and_then(|v| v.as_str())
                .unwrap_or("personal")
                .to_string();
            let amount = it
                .get("total")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<Decimal>().ok())
                .unwrap_or(Decimal::ZERO);
            let cat_entry = by_category.entry(cat).or_insert((Decimal::ZERO, 0));
            cat_entry.0 += amount;
            cat_entry.1 += 1;
            let bucket_entry = by_tax_bucket
                .entry(bucket.clone())
                .or_insert((Decimal::ZERO, 0));
            bucket_entry.0 += amount;
            bucket_entry.1 += 1;
            match bucket.as_str() {
                "business" => schedule_c += amount,
                "rental" => schedule_e += amount,
                "personal" => personal += amount,
                _ => {
                    unclassified += amount;
                    uncategorized_total += amount;
                    uncategorized_count += 1;
                }
            }
        }
    }

    let total = schedule_c + schedule_e + personal + unclassified;
    // When filtered to a business, count only receipts that had ≥1
    // matching item (and therefore appeared in daily_total). Otherwise
    // the receipt count is the raw row count.
    let receipt_count = if q.business_id.is_some() {
        daily_total.values().map(|(_, c)| *c).sum::<u32>()
    } else {
        rows.len() as u32
    };
    let avg_ticket = if receipt_count > 0 {
        (total / Decimal::from(receipt_count)).round_dp(2)
    } else {
        Decimal::ZERO
    };
    let day_of_year = (now - from).num_days().max(1);
    let avg_daily = (total / Decimal::from(day_of_year)).round_dp(2);
    let burn_rate_monthly = (avg_daily * Decimal::from(30)).round_dp(2);
    let biz_pct = if total > Decimal::ZERO {
        (schedule_c + schedule_e) / total
    } else {
        Decimal::ZERO
    }
    .try_into()
    .unwrap_or(0.0);
    let deductible_pct = biz_pct * 100.0;
    if smallest == Decimal::MAX {
        smallest = Decimal::ZERO;
    }

    // Streaks: longest run of consecutive days with $0 receipts, longest run with >$0.
    let mut longest_zero = 0u32;
    let mut longest_active = 0u32;
    let mut cur_zero = 0u32;
    let mut cur_active = 0u32;
    let mut d = from;
    while d <= to.min(now) {
        if daily_total.contains_key(&d) {
            cur_active += 1;
            cur_zero = 0;
            longest_active = longest_active.max(cur_active);
        } else {
            cur_zero += 1;
            cur_active = 0;
            longest_zero = longest_zero.max(cur_zero);
        }
        d = d.succ_opt().unwrap();
    }

    // Daily series with running cumulative.
    let mut daily: Vec<DailyPoint> = Vec::new();
    let mut acc = Decimal::ZERO;
    let mut d = from;
    while d <= to {
        let (day_total, day_count) = daily_total.get(&d).cloned().unwrap_or_default();
        acc += day_total;
        daily.push(DailyPoint {
            day: d,
            total: day_total,
            count: day_count,
            cumulative: acc,
        });
        d = d.succ_opt().unwrap();
    }

    // Compact slices for tiny charts.
    fn label_arr(arr: &[(Decimal, u32)], labels: &[&str]) -> Vec<LabeledTotal> {
        arr.iter()
            .zip(labels.iter())
            .map(|((tot, cnt), l)| LabeledTotal {
                label: l.to_string(),
                total: *tot,
                count: *cnt,
            })
            .collect()
    }

    let dow_labels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let hour_labels: Vec<String> = (0..24).map(|h| format!("{h:02}")).collect();
    let hour_label_refs: Vec<&str> = hour_labels.iter().map(|s| s.as_str()).collect();
    let month_labels = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let quarter_labels = ["Q1", "Q2", "Q3", "Q4"];
    let amount_labels = [
        "$0-10", "$10-25", "$25-50", "$50-100", "$100-250", "$250-1k", "$1k+",
    ];

    let mut by_category_v: Vec<LabeledTotal> = by_category
        .into_iter()
        .map(|(label, (total, count))| LabeledTotal {
            label,
            total,
            count,
        })
        .collect();
    by_category_v.sort_by_key(|m| std::cmp::Reverse(m.total));
    by_category_v.truncate(15);

    let mut by_tax_bucket_v: Vec<LabeledTotal> = by_tax_bucket
        .into_iter()
        .map(|(label, (total, count))| LabeledTotal {
            label,
            total,
            count,
        })
        .collect();
    by_tax_bucket_v.sort_by_key(|m| std::cmp::Reverse(m.total));

    let mut by_total_v: Vec<LabeledTotal> = top_merchant_total
        .iter()
        .map(|(label, (total, count))| LabeledTotal {
            label: label.clone(),
            total: *total,
            count: *count,
        })
        .collect();
    by_total_v.sort_by_key(|m| std::cmp::Reverse(m.total));
    by_total_v.truncate(20);

    let mut by_count_v: Vec<LabeledTotal> = top_merchant_total
        .into_iter()
        .map(|(label, (total, count))| LabeledTotal {
            label,
            total,
            count,
        })
        .collect();
    by_count_v.sort_by_key(|m| std::cmp::Reverse(m.count));
    by_count_v.truncate(20);

    let weekday_v = vec![
        LabeledTotal {
            label: "weekday".into(),
            total: weekday_total.0,
            count: weekday_total.1,
        },
        LabeledTotal {
            label: "weekend".into(),
            total: weekend_total.0,
            count: weekend_total.1,
        },
    ];

    Ok(Json(ExpenseBundle {
        year,
        from,
        to,
        kpis: ExpenseKpis {
            total,
            schedule_c,
            schedule_e,
            personal,
            unclassified,
            receipt_count,
            item_count,
            avg_ticket,
            avg_daily,
            biz_pct,
            deductible_pct,
            burn_rate_monthly,
            biggest_receipt: biggest,
            smallest_receipt: smallest,
            longest_zero_streak_days: longest_zero,
            longest_consec_spending_days: longest_active,
        },
        daily: daily.clone(),
        calendar: daily,
        by_dow: label_arr(&by_dow, &dow_labels),
        by_hour: label_arr(&by_hour, &hour_label_refs),
        by_month: label_arr(&by_month, &month_labels),
        by_quarter: label_arr(&by_quarter, &quarter_labels),
        by_amount_bucket: label_arr(&by_amount_bucket, &amount_labels),
        by_category: by_category_v,
        by_tax_bucket: by_tax_bucket_v,
        top_merchants_by_total: by_total_v,
        top_merchants_by_count: by_count_v,
        weekday_vs_weekend: weekday_v,
        uncategorized_count,
        uncategorized_total,
    }))
}

fn amount_bucket(t: Decimal) -> usize {
    let t_f64: f64 = t.try_into().unwrap_or(0.0);
    match t_f64 {
        v if v < 10.0 => 0,
        v if v < 25.0 => 1,
        v if v < 50.0 => 2,
        v if v < 100.0 => 3,
        v if v < 250.0 => 4,
        v if v < 1000.0 => 5,
        _ => 6,
    }
}

// --- analytics: calendar heatmap, top merchants, day-of-week, cumulative ---
//
// Visualization-friendly aggregations of the receipt history. Each one
// is a single SQL pull + Rust roll-up — small, fast, and cache-friendly.

#[derive(Deserialize)]
struct CalendarQ {
    year: Option<i32>,
    business_id: Option<Uuid>,
}

#[derive(Serialize)]
struct CalendarDay {
    day: NaiveDate,
    total: Decimal,
    count: u32,
}

/// Per-day spend totals for the requested year — drives a GitHub-style
/// Tradervue-style monthly calendar grid. Per-day rollup includes the
/// total spend AND the tax-bucket split (business / rental / personal /
/// unclassified), so cells can be color-coded by their dominant bucket
/// — green for biz-deductible days, red for personal-only days, etc.
#[derive(Serialize)]
struct MonthCalendarDay {
    day: NaiveDate,
    total: Decimal,
    count: u32,
    business: Decimal,
    rental: Decimal,
    personal: Decimal,
    unclassified: Decimal,
    /// "business" | "rental" | "personal" | "unclassified" | "none" — the
    /// dominant tax bucket by dollar share. Drives the cell tint.
    dominant_bucket: &'static str,
}

async fn receipts_month_calendar(
    State(s): State<AppState>,
    user: AuthUser,
    Path((year, month)): Path<(i32, u32)>,
    axum::extract::Query(q): axum::extract::Query<BusinessFilterQ>,
) -> Result<Json<Vec<MonthCalendarDay>>, ApiError> {
    if !(1..=12).contains(&month) {
        return Err(ApiError::BadRequest(format!(
            "month {month} out of range 1..=12"
        )));
    }
    let from = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| ApiError::BadRequest(format!("invalid year/month {year}/{month}")))?;
    let to = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    }
    .pred_opt()
    .unwrap();

    let rows: Vec<(NaiveDate, Option<Decimal>, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT ocr_date::date, ocr_total, ocr_extracted
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND ocr_date IS NOT NULL
            AND ocr_date BETWEEN $2 AND $3",
    )
    .bind(user.id)
    .bind(from)
    .bind(to)
    .fetch_all(&s.pool)
    .await?;

    use std::collections::HashMap;
    #[derive(Default)]
    struct DayAcc {
        total: Decimal,
        count: u32,
        business: Decimal,
        rental: Decimal,
        personal: Decimal,
        unclassified: Decimal,
    }
    let mut by_day: HashMap<NaiveDate, DayAcc> = HashMap::new();

    for (day, total, extracted) in rows {
        // Under a business filter, only include receipts that have ≥1
        // matching item. Skip everything else so the calendar shows a
        // truly per-business view.
        let matched = if let Some(ext) = &extracted {
            if let Some(items) = ext.get("items").and_then(|v| v.as_array()) {
                items.iter().any(|it| item_in_business(it, q.business_id))
            } else {
                q.business_id.is_none()
            }
        } else {
            q.business_id.is_none()
        };
        if !matched {
            continue;
        }
        let acc = by_day.entry(day).or_default();
        acc.count += 1;
        // Under a filter, recompute the day's total from matching items
        // only — `ocr_total` would over-count when receipts mix buckets.
        let receipt_total: Decimal = if q.business_id.is_some() {
            extracted
                .as_ref()
                .and_then(|ext| ext.get("items"))
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter(|it| item_in_business(it, q.business_id))
                        .map(|it| {
                            it.get("total")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<Decimal>().ok())
                                .unwrap_or(Decimal::ZERO)
                        })
                        .sum()
                })
                .unwrap_or(Decimal::ZERO)
        } else {
            total.unwrap_or(Decimal::ZERO)
        };
        acc.total += receipt_total;
        // Bucket-split per item; falls back to receipt total → personal
        // if the OCR didn't extract structured items.
        if let Some(ext) = extracted {
            if let Some(items) = ext.get("items").and_then(|v| v.as_array()) {
                for it in items {
                    if !item_in_business(it, q.business_id) {
                        continue;
                    }
                    let bucket = it
                        .get("tax_bucket")
                        .and_then(|v| v.as_str())
                        .unwrap_or("personal");
                    let amount = it
                        .get("total")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<Decimal>().ok())
                        .unwrap_or(Decimal::ZERO);
                    match bucket {
                        "business" => acc.business += amount,
                        "rental" => acc.rental += amount,
                        "personal" => acc.personal += amount,
                        _ => acc.unclassified += amount,
                    }
                }
            } else if let Some(t) = total {
                acc.personal += t;
            }
        } else if let Some(t) = total {
            acc.personal += t;
        }
    }

    let days_in_month = to.day();
    let mut out: Vec<MonthCalendarDay> = Vec::with_capacity(days_in_month as usize);
    for d in 1..=days_in_month {
        let day = NaiveDate::from_ymd_opt(year, month, d).unwrap();
        let acc = by_day.remove(&day).unwrap_or_default();
        // Dominant bucket = whichever has the largest dollar share. Ties
        // resolve by declared priority (business > rental > personal > unc).
        let dominant: &'static str = if acc.business > acc.rental
            && acc.business > acc.personal
            && acc.business > acc.unclassified
            && acc.business > Decimal::ZERO
        {
            "business"
        } else if acc.rental > acc.personal
            && acc.rental > acc.unclassified
            && acc.rental > Decimal::ZERO
        {
            "rental"
        } else if acc.personal > acc.unclassified && acc.personal > Decimal::ZERO {
            "personal"
        } else if acc.unclassified > Decimal::ZERO {
            "unclassified"
        } else {
            "none"
        };
        out.push(MonthCalendarDay {
            day,
            total: acc.total,
            count: acc.count,
            business: acc.business,
            rental: acc.rental,
            personal: acc.personal,
            unclassified: acc.unclassified,
            dominant_bucket: dominant,
        });
    }
    Ok(Json(out))
}

/// year-grid heatmap on the frontend. Includes days with zero spend so
/// the frontend can render an empty cell rather than guessing.
async fn spend_calendar(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<CalendarQ>,
) -> Result<Json<Vec<CalendarDay>>, ApiError> {
    let year = q.year.unwrap_or_else(|| Utc::now().date_naive().year());
    let from = NaiveDate::from_ymd_opt(year, 1, 1)
        .ok_or_else(|| ApiError::BadRequest(format!("year {year} out of NaiveDate range")))?;
    let to = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();

    // When filtering by business we need to read items, not just
    // ocr_total — pull per-receipt rows and re-aggregate in Rust.
    let by_day: std::collections::HashMap<NaiveDate, (Decimal, i64)> = if q.business_id.is_some() {
        let rows: Vec<(NaiveDate, Option<Decimal>, Option<serde_json::Value>)> = sqlx::query_as(
            "SELECT ocr_date::date, ocr_total, ocr_extracted
               FROM receipts
              WHERE user_id = $1
                AND ocr_status = 'done'::ocr_status_t
                AND ocr_date IS NOT NULL
                AND ocr_date BETWEEN $2 AND $3",
        )
        .bind(user.id)
        .bind(from)
        .bind(to)
        .fetch_all(&s.pool)
        .await?;
        let mut m: std::collections::HashMap<NaiveDate, (Decimal, i64)> = Default::default();
        for (day, total, extracted) in rows {
            let eff = receipt_total_filtered(total, &extracted, q.business_id);
            if eff <= Decimal::ZERO {
                continue;
            }
            let entry = m.entry(day).or_insert((Decimal::ZERO, 0));
            entry.0 += eff;
            entry.1 += 1;
        }
        m
    } else {
        let rows: Vec<(NaiveDate, Decimal, i64)> = sqlx::query_as(
            "SELECT ocr_date::date AS day,
                    COALESCE(SUM(ocr_total), 0)::numeric AS total,
                    COUNT(*) AS cnt
               FROM receipts
              WHERE user_id = $1
                AND ocr_status = 'done'::ocr_status_t
                AND ocr_total IS NOT NULL
                AND ocr_date IS NOT NULL
                AND ocr_date BETWEEN $2 AND $3
              GROUP BY ocr_date::date",
        )
        .bind(user.id)
        .bind(from)
        .bind(to)
        .fetch_all(&s.pool)
        .await?;
        rows.into_iter().map(|(d, t, c)| (d, (t, c))).collect()
    };

    let days_in_year = if to.signed_duration_since(from).num_days() == 365 {
        366
    } else {
        365
    };
    let mut out = Vec::with_capacity(days_in_year);
    let mut d = from;
    while d <= to {
        let (total, count) = by_day.get(&d).cloned().unwrap_or_default();
        out.push(CalendarDay {
            day: d,
            total,
            count: count as u32,
        });
        d = d.succ_opt().unwrap();
    }
    Ok(Json(out))
}

#[derive(Deserialize)]
struct WindowQ {
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    business_id: Option<Uuid>,
}

#[derive(Serialize)]
struct DayOfWeekBucket {
    /// 0 = Sunday, 6 = Saturday (ISO 8601 calendar).
    weekday: u8,
    total: Decimal,
    count: u32,
}

/// Day-of-week spend distribution. Surfaces patterns like "groceries
/// every Sunday" or "biz meals concentrated Tue/Wed/Thu".
async fn receipts_dow(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<WindowQ>,
) -> Result<Json<Vec<DayOfWeekBucket>>, ApiError> {
    let now = Utc::now().date_naive();
    let year = now.year();
    let from = q
        .from
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, 1, 1).unwrap());
    let to = q.to.unwrap_or(now);

    let rows: Vec<(NaiveDate, Option<Decimal>, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT ocr_date::date, ocr_total, ocr_extracted
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND ocr_date IS NOT NULL
            AND ocr_date BETWEEN $2 AND $3",
    )
    .bind(user.id)
    .bind(from)
    .bind(to)
    .fetch_all(&s.pool)
    .await?;

    let mut bucket: [(Decimal, u32); 7] = [
        (Decimal::ZERO, 0),
        (Decimal::ZERO, 0),
        (Decimal::ZERO, 0),
        (Decimal::ZERO, 0),
        (Decimal::ZERO, 0),
        (Decimal::ZERO, 0),
        (Decimal::ZERO, 0),
    ];
    for (day, total, extracted) in rows {
        let eff = receipt_total_filtered(total, &extracted, q.business_id);
        if eff <= Decimal::ZERO {
            continue;
        }
        // Sunday = 0 via num_days_from_sunday for parity with JS Date.getDay().
        let weekday = day.weekday().num_days_from_sunday() as usize;
        bucket[weekday].0 += eff;
        bucket[weekday].1 += 1;
    }
    let out: Vec<DayOfWeekBucket> = (0..7)
        .map(|i| DayOfWeekBucket {
            weekday: i as u8,
            total: bucket[i].0,
            count: bucket[i].1,
        })
        .collect();
    Ok(Json(out))
}

#[derive(Serialize)]
struct CumulativePoint {
    day: NaiveDate,
    cumulative: Decimal,
}

/// Cumulative-spend curve over the requested window — the expense
/// analog of the trading equity curve. Always returns one point per
/// day so the frontend chart axis is contiguous.
async fn receipts_cumulative(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<WindowQ>,
) -> Result<Json<Vec<CumulativePoint>>, ApiError> {
    let now = Utc::now().date_naive();
    let year = now.year();
    let from = q
        .from
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, 1, 1).unwrap());
    let to = q.to.unwrap_or(now);

    let by_day: std::collections::HashMap<NaiveDate, Decimal> = if q.business_id.is_some() {
        let rows: Vec<(NaiveDate, Option<Decimal>, Option<serde_json::Value>)> = sqlx::query_as(
            "SELECT ocr_date::date, ocr_total, ocr_extracted
               FROM receipts
              WHERE user_id = $1
                AND ocr_status = 'done'::ocr_status_t
                AND ocr_date IS NOT NULL
                AND ocr_date BETWEEN $2 AND $3",
        )
        .bind(user.id)
        .bind(from)
        .bind(to)
        .fetch_all(&s.pool)
        .await?;
        let mut m: std::collections::HashMap<NaiveDate, Decimal> = Default::default();
        for (day, total, extracted) in rows {
            let eff = receipt_total_filtered(total, &extracted, q.business_id);
            if eff > Decimal::ZERO {
                *m.entry(day).or_insert(Decimal::ZERO) += eff;
            }
        }
        m
    } else {
        let rows: Vec<(NaiveDate, Decimal)> = sqlx::query_as(
            "SELECT ocr_date::date AS day,
                    COALESCE(SUM(ocr_total), 0)::numeric
               FROM receipts
              WHERE user_id = $1
                AND ocr_status = 'done'::ocr_status_t
                AND ocr_total IS NOT NULL
                AND ocr_date IS NOT NULL
                AND ocr_date BETWEEN $2 AND $3
              GROUP BY ocr_date::date
              ORDER BY day",
        )
        .bind(user.id)
        .bind(from)
        .bind(to)
        .fetch_all(&s.pool)
        .await?;
        rows.into_iter().collect()
    };
    let mut out: Vec<CumulativePoint> = Vec::new();
    let mut acc = Decimal::ZERO;
    let mut d = from;
    while d <= to {
        if let Some(t) = by_day.get(&d) {
            acc += t;
        }
        out.push(CumulativePoint {
            day: d,
            cumulative: acc,
        });
        d = d.succ_opt().unwrap();
    }
    Ok(Json(out))
}

// ── YoY same-month overlay ────────────────────────────────────────────────

#[derive(Deserialize)]
struct YearQ {
    year: Option<i32>,
    business_id: Option<Uuid>,
}

#[derive(Serialize)]
struct YoyMonthlyRow {
    month: u32,
    current: Decimal,
    prior: Decimal,
}

async fn receipts_yoy_monthly(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<YearQ>,
) -> Result<Json<Vec<YoyMonthlyRow>>, ApiError> {
    let year = q.year.unwrap_or_else(|| Utc::now().date_naive().year());
    let prior_year = year - 1;
    let from = NaiveDate::from_ymd_opt(prior_year, 1, 1)
        .ok_or_else(|| ApiError::BadRequest(format!("year {prior_year} out of NaiveDate range")))?;
    let to = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();

    let rows: Vec<(NaiveDate, Option<Decimal>, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT ocr_date::date, ocr_total, ocr_extracted
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND ocr_date IS NOT NULL
            AND ocr_date BETWEEN $2 AND $3",
    )
    .bind(user.id)
    .bind(from)
    .bind(to)
    .fetch_all(&s.pool)
    .await?;

    let mut current = [Decimal::ZERO; 12];
    let mut prior = [Decimal::ZERO; 12];
    for (day, total, extracted) in rows {
        let eff = receipt_total_filtered(total, &extracted, q.business_id);
        if eff <= Decimal::ZERO {
            continue;
        }
        let idx = day.month0() as usize;
        if day.year() == year {
            current[idx] += eff;
        } else if day.year() == prior_year {
            prior[idx] += eff;
        }
    }
    Ok(Json(
        (0..12)
            .map(|i| YoyMonthlyRow {
                month: (i + 1) as u32,
                current: current[i],
                prior: prior[i],
            })
            .collect(),
    ))
}

// ── Receipt aging (uncategorized backlog) ─────────────────────────────────

#[derive(Serialize)]
struct AgingBucket {
    /// "0-7d" | "8-30d" | "31-90d" | "90+d"
    bucket: &'static str,
    count: u32,
    total: Decimal,
}

async fn receipts_aging(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<BusinessFilterQ>,
) -> Result<Json<Vec<AgingBucket>>, ApiError> {
    let now = Utc::now().date_naive();

    // Pull uncategorized OR unclassified items, joining receipt-level
    // total when needed. "Uncategorized" = ocr_extracted is null OR every
    // item has tax_bucket = 'personal'/unset. For simplicity here we use
    // receipts where any item has tax_bucket not in (business, rental,
    // personal) — items with category unset OR tax_bucket null. We bin
    // by age of the receipt's ocr_date.
    let rows: Vec<(NaiveDate, Option<Decimal>, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT ocr_date::date, ocr_total, ocr_extracted
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND ocr_date IS NOT NULL",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;

    let mut buckets: [(u32, Decimal); 4] = Default::default();
    for (day, total, extracted) in rows {
        // If filtering to a business, skip receipts that have no
        // items belonging to that business — they can't be its backlog.
        if let Some(_b) = q.business_id {
            let in_biz = extracted
                .as_ref()
                .and_then(|ext| ext.get("items"))
                .and_then(|v| v.as_array())
                .map(|items| items.iter().any(|it| item_in_business(it, q.business_id)))
                .unwrap_or(false);
            if !in_biz {
                continue;
            }
        }
        // Treat the receipt as "needs attention" if extracted is null or
        // any item lacks a category.
        let needs = match &extracted {
            None => true,
            Some(ext) => match ext.get("items").and_then(|v| v.as_array()) {
                None => true,
                Some(items) => items
                    .iter()
                    .filter(|it| item_in_business(it, q.business_id))
                    .any(|it| {
                        it.get("category")
                            .and_then(|v| v.as_str())
                            .map(|s| s.trim().is_empty() || s == "uncategorized")
                            .unwrap_or(true)
                    }),
            },
        };
        if !needs {
            continue;
        }
        let age_days = (now - day).num_days();
        let idx = if age_days <= 7 {
            0
        } else if age_days <= 30 {
            1
        } else if age_days <= 90 {
            2
        } else {
            3
        };
        buckets[idx].0 += 1;
        if let Some(t) = total {
            buckets[idx].1 += t;
        }
    }
    let labels = ["0-7d", "8-30d", "31-90d", "90+d"];
    Ok(Json(
        (0..4)
            .map(|i| AgingBucket {
                bucket: labels[i],
                count: buckets[i].0,
                total: buckets[i].1,
            })
            .collect(),
    ))
}

// ── Per-property Schedule E rollup ────────────────────────────────────────

#[derive(Serialize)]
struct PropertyCategoryRow {
    category: String,
    total: Decimal,
}

#[derive(Serialize)]
struct PropertyRow {
    property_id: Option<Uuid>,
    property_name: Option<String>,
    total: Decimal,
    item_count: u32,
    top_categories: Vec<PropertyCategoryRow>,
}

async fn receipts_by_property(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<YearQ>,
) -> Result<Json<Vec<PropertyRow>>, ApiError> {
    let now = Utc::now().date_naive();
    let year = q.year.unwrap_or(now.year());
    let from = NaiveDate::from_ymd_opt(year, 1, 1)
        .ok_or_else(|| ApiError::BadRequest(format!("year {year} out of NaiveDate range")))?;
    let to = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();

    let rows: Vec<(Option<serde_json::Value>,)> = sqlx::query_as(
        "SELECT ocr_extracted
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND ocr_extracted IS NOT NULL
            AND ocr_date IS NOT NULL
            AND ocr_date BETWEEN $2 AND $3",
    )
    .bind(user.id)
    .bind(from)
    .bind(to)
    .fetch_all(&s.pool)
    .await?;

    let props: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT id, nickname FROM rental_properties WHERE user_id = $1")
            .bind(user.id)
            .fetch_all(&s.pool)
            .await
            .unwrap_or_default();
    let prop_name: std::collections::HashMap<Uuid, String> = props.into_iter().collect();

    use std::collections::BTreeMap;
    // (property_id) → (total, count, category→total)
    let mut by_prop: std::collections::HashMap<
        Option<Uuid>,
        (Decimal, u32, BTreeMap<String, Decimal>),
    > = std::collections::HashMap::new();

    for (extracted,) in rows {
        let Some(ext) = extracted else { continue };
        let Some(items) = ext.get("items").and_then(|v| v.as_array()) else {
            continue;
        };
        for it in items {
            if !item_in_business(it, q.business_id) {
                continue;
            }
            let bucket = it.get("tax_bucket").and_then(|v| v.as_str()).unwrap_or("");
            if bucket != "rental" {
                continue;
            }
            let amount = it
                .get("total")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<Decimal>().ok())
                .unwrap_or(Decimal::ZERO);
            let category = it
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("uncategorized")
                .to_string();
            let property_id = it
                .get("rental_property_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());
            let entry = by_prop
                .entry(property_id)
                .or_insert_with(|| (Decimal::ZERO, 0, BTreeMap::new()));
            entry.0 += amount;
            entry.1 += 1;
            *entry.2.entry(category).or_insert(Decimal::ZERO) += amount;
        }
    }
    let mut out: Vec<PropertyRow> = by_prop
        .into_iter()
        .map(|(pid, (total, count, cats))| {
            let mut cat_v: Vec<PropertyCategoryRow> = cats
                .into_iter()
                .map(|(category, total)| PropertyCategoryRow { category, total })
                .collect();
            cat_v.sort_by_key(|c| std::cmp::Reverse(c.total));
            cat_v.truncate(5);
            PropertyRow {
                property_id: pid,
                property_name: pid.and_then(|id| prop_name.get(&id).cloned()),
                total,
                item_count: count,
                top_categories: cat_v,
            }
        })
        .collect();
    out.sort_by_key(|p| std::cmp::Reverse(p.total));
    Ok(Json(out))
}

// ── Anomalies (MoM merchant deltas + first-time merchants + outliers) ─────

#[derive(Serialize)]
struct Anomaly {
    /// "subscription_jump" | "new_merchant" | "outlier_receipt"
    kind: &'static str,
    /// Short human-readable label for the card.
    label: String,
    /// Quantitative payload (current month total, % change, receipt
    /// amount, etc.) — the frontend formats per `kind`.
    value: Decimal,
    /// Secondary value (prior month total, sigma multiple, etc.).
    secondary: Decimal,
    /// First-seen date (`new_merchant`) or receipt date (`outlier_receipt`).
    when: Option<NaiveDate>,
}

async fn receipts_anomalies(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<BusinessFilterQ>,
) -> Result<Json<Vec<Anomaly>>, ApiError> {
    let now = Utc::now().date_naive();
    let this_month_start = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap_or(now);
    let last_month_end = this_month_start.pred_opt().unwrap_or(now);
    let last_month_start =
        NaiveDate::from_ymd_opt(last_month_end.year(), last_month_end.month(), 1)
            .unwrap_or(last_month_end);
    let four_months_ago = NaiveDate::from_ymd_opt(
        if last_month_start.month() <= 3 {
            last_month_start.year() - 1
        } else {
            last_month_start.year()
        },
        ((last_month_start.month() + 8) % 12) + 1,
        1,
    )
    .unwrap_or(last_month_start);

    let raw_rows: Vec<(
        Option<String>,
        Option<Decimal>,
        NaiveDate,
        Option<serde_json::Value>,
    )> = sqlx::query_as(
        "SELECT ocr_merchant, ocr_total, ocr_date::date, ocr_extracted
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND ocr_date IS NOT NULL",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;
    // Project to (merchant, effective-total, date) after applying the
    // business filter.
    let rows: Vec<(Option<String>, Option<Decimal>, NaiveDate)> = raw_rows
        .into_iter()
        .filter_map(|(m, t, d, ext)| {
            let eff = receipt_total_filtered(t, &ext, q.business_id);
            if q.business_id.is_some() && eff <= Decimal::ZERO {
                None
            } else {
                Some((m, Some(eff), d))
            }
        })
        .collect();

    let aliases = crate::merchant::load_aliases(&s.pool, user.id)
        .await
        .unwrap_or_default();

    use std::collections::HashMap;
    let mut this_month: HashMap<String, Decimal> = HashMap::new();
    let mut last_month: HashMap<String, Decimal> = HashMap::new();
    let mut first_seen: HashMap<String, NaiveDate> = HashMap::new();
    let mut all_amounts: Vec<(Decimal, NaiveDate)> = Vec::new();

    for (m, total, day) in &rows {
        let Some(m) = m else { continue };
        let canonical = crate::merchant::canonicalize(m, &aliases);
        if canonical.is_empty() {
            continue;
        }
        first_seen
            .entry(canonical.clone())
            .and_modify(|d| {
                if *day < *d {
                    *d = *day;
                }
            })
            .or_insert(*day);

        if *day >= this_month_start {
            if let Some(t) = total {
                *this_month.entry(canonical.clone()).or_insert(Decimal::ZERO) += *t;
            }
        } else if *day >= last_month_start && *day <= last_month_end {
            if let Some(t) = total {
                *last_month.entry(canonical.clone()).or_insert(Decimal::ZERO) += *t;
            }
        }
        if let Some(t) = total {
            all_amounts.push((*t, *day));
        }
    }

    let mut anomalies: Vec<Anomaly> = Vec::new();

    // 1) Subscription jumps: MoM ≥ +30% AND ≥ $20 absolute change.
    for (canonical, current) in &this_month {
        let Some(prior) = last_month.get(canonical) else {
            continue;
        };
        if *prior <= Decimal::ZERO {
            continue;
        }
        let delta = *current - *prior;
        if delta < Decimal::from(20) {
            continue;
        }
        let pct = (delta / *prior * Decimal::from(100)).round_dp(1);
        if pct < Decimal::from(30) {
            continue;
        }
        anomalies.push(Anomaly {
            kind: "subscription_jump",
            label: canonical.clone(),
            value: *current,
            secondary: pct,
            when: None,
        });
    }

    // 2) First-time merchants this month.
    for (canonical, fs) in &first_seen {
        if *fs >= this_month_start {
            let total = this_month.get(canonical).cloned().unwrap_or(Decimal::ZERO);
            if total > Decimal::from(10) {
                anomalies.push(Anomaly {
                    kind: "new_merchant",
                    label: canonical.clone(),
                    value: total,
                    secondary: Decimal::ZERO,
                    when: Some(*fs),
                });
            }
        }
    }

    // 3) Outlier receipts: > μ + 3σ across the trailing-4-month window.
    let trailing: Vec<f64> = all_amounts
        .iter()
        .filter(|(_, day)| *day >= four_months_ago)
        .map(|(t, _)| (*t).try_into().unwrap_or(0.0))
        .collect();
    if trailing.len() >= 30 {
        let n = trailing.len() as f64;
        let mean = trailing.iter().sum::<f64>() / n;
        let var = trailing.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let stddev = var.sqrt();
        let threshold = mean + 3.0 * stddev;
        for (t, day) in &all_amounts {
            let amount: f64 = (*t).try_into().unwrap_or(0.0);
            if amount > threshold && *day >= four_months_ago {
                let sigma = if stddev > 0.0 {
                    ((amount - mean) / stddev).round() as i64
                } else {
                    0
                };
                anomalies.push(Anomaly {
                    kind: "outlier_receipt",
                    label: format!("{:.0}σ", sigma),
                    value: *t,
                    secondary: Decimal::from(sigma),
                    when: Some(*day),
                });
            }
        }
    }
    anomalies.sort_by_key(|a| std::cmp::Reverse(a.value));
    anomalies.truncate(40);
    Ok(Json(anomalies))
}

// ── Category distribution (box plot stats per category) ───────────────────

#[derive(Serialize)]
struct CategoryDist {
    category: String,
    min: Decimal,
    q1: Decimal,
    median: Decimal,
    q3: Decimal,
    max: Decimal,
    count: u32,
}

async fn receipts_category_distribution(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<YearQ>,
) -> Result<Json<Vec<CategoryDist>>, ApiError> {
    let now = Utc::now().date_naive();
    let year = q.year.unwrap_or(now.year());
    let from = NaiveDate::from_ymd_opt(year, 1, 1)
        .ok_or_else(|| ApiError::BadRequest(format!("year {year} out of NaiveDate range")))?;
    let to = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();

    let rows: Vec<(Option<serde_json::Value>,)> = sqlx::query_as(
        "SELECT ocr_extracted
           FROM receipts
          WHERE user_id = $1
            AND ocr_status = 'done'::ocr_status_t
            AND ocr_extracted IS NOT NULL
            AND ocr_date IS NOT NULL
            AND ocr_date BETWEEN $2 AND $3",
    )
    .bind(user.id)
    .bind(from)
    .bind(to)
    .fetch_all(&s.pool)
    .await?;

    use std::collections::HashMap;
    let mut by_cat: HashMap<String, Vec<Decimal>> = HashMap::new();
    for (extracted,) in rows {
        let Some(ext) = extracted else { continue };
        let Some(items) = ext.get("items").and_then(|v| v.as_array()) else {
            continue;
        };
        for it in items {
            if !item_in_business(it, q.business_id) {
                continue;
            }
            let cat = it
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("uncategorized")
                .to_string();
            let amount = it
                .get("total")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<Decimal>().ok())
                .unwrap_or(Decimal::ZERO);
            by_cat.entry(cat).or_default().push(amount);
        }
    }
    let mut out: Vec<CategoryDist> = by_cat
        .into_iter()
        .filter(|(_, v)| v.len() >= 5) // require ≥ 5 data points for stable quartiles
        .map(|(category, mut vals)| {
            vals.sort();
            let n = vals.len();
            let pct = |p: f64| {
                let idx = ((n - 1) as f64 * p).round() as usize;
                vals[idx]
            };
            CategoryDist {
                category,
                min: vals[0],
                q1: pct(0.25),
                median: pct(0.50),
                q3: pct(0.75),
                max: vals[n - 1],
                count: n as u32,
            }
        })
        .collect();
    out.sort_by_key(|c| std::cmp::Reverse(c.median));
    out.truncate(15);
    Ok(Json(out))
}

/// CSV export of the same rollup the JSON endpoint returns, in a flat
/// shape suitable for pasting into a Schedule C / Schedule E worksheet.
///
/// Columns:
///   bucket, schedule, line, category, property, total
///
/// Where `schedule` is `C` for the business bucket, `E` for rental,
/// blank for personal / unclassified. Property name is filled only for
/// rental rows; line is blank when no schedule maps the category.
async fn tax_rollup_csv(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<RollupQ>,
) -> Result<Response, ApiError> {
    // Re-use the JSON handler's body — saves duplicating the aggregation
    // loop. We rebuild the response with `tax_rollup`'s logic by calling
    // it via direct fn invocation.
    let Json(rollup) = tax_rollup(State(s.clone()), user, axum::extract::Query(q)).await?;

    let mut csv = String::new();
    csv.push_str("bucket,schedule,line,category,property,total\n");

    fn write_cat(
        csv: &mut String,
        bucket: &str,
        schedule: &str,
        ct: &CategoryTotal,
        property: &str,
    ) {
        let line = match (schedule, ct.schedule_c_line, ct.schedule_e_line) {
            ("C", Some(l), _) => l,
            ("E", _, Some(l)) => l,
            _ => "",
        };
        let property = csv_escape(property);
        let cat = csv_escape(&ct.category);
        // Decimal Display does not quote — direct push is fine.
        csv.push_str(&format!(
            "{bucket},{schedule},{line},{cat},{property},{total}\n",
            total = ct.total,
        ));
    }

    for ct in &rollup.business.categories {
        write_cat(&mut csv, "business", "C", ct, "");
    }
    csv.push_str(&format!(
        "business,C,,GRAND TOTAL,,{}\n",
        rollup.business.grand_total
    ));

    for prop in &rollup.rental.properties {
        let pname = prop
            .property_name
            .clone()
            .unwrap_or_else(|| "(unassigned)".into());
        for ct in &prop.categories {
            write_cat(&mut csv, "rental", "E", ct, &pname);
        }
        csv.push_str(&format!(
            "rental,E,,GRAND TOTAL,{pname},{total}\n",
            pname = csv_escape(&pname),
            total = prop.grand_total,
        ));
    }
    csv.push_str(&format!(
        "rental,E,,GRAND TOTAL,ALL PROPERTIES,{}\n",
        rollup.rental.grand_total
    ));

    for ct in &rollup.personal.categories {
        write_cat(&mut csv, "personal", "", ct, "");
    }
    csv.push_str(&format!(
        "personal,,,GRAND TOTAL,,{}\n",
        rollup.personal.grand_total
    ));

    for ct in &rollup.unclassified.categories {
        write_cat(&mut csv, "unclassified", "", ct, "");
    }
    csv.push_str(&format!(
        "unclassified,,,GRAND TOTAL,,{}\n",
        rollup.unclassified.grand_total
    ));

    let filename = format!("tax-rollup-{}-to-{}.csv", rollup.from, rollup.to);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(csv))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("response build: {e}")))
}

/// CSV-escape a cell — wrap in quotes when it contains a comma, quote,
/// or newline; double-up any embedded quotes (RFC 4180).
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// --- tax rollup PDF ----------------------------------------------------
//
// Server-rendered PDF via `printpdf`. Produces a Letter-size document
// with: header (year + period), 4-stat strip (Income / Sched C /
// Sched E / Net), Schedule C category breakdown, Schedule E per-
// property breakdown. Multi-page; tables continue with running headers.
//
// We delegate JSON aggregation to the existing `tax_rollup` handler
// and just render its output here — keeps the math single-sourced.

#[derive(Deserialize)]
struct PdfRollupQ {
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    /// When `?detail=1`, append the full transaction listing. Default 0.
    #[serde(default)]
    detail: Option<u8>,
}

async fn tax_rollup_pdf(
    State(s): State<AppState>,
    user: AuthUser,
    axum::extract::Query(q): axum::extract::Query<PdfRollupQ>,
) -> Result<Response, ApiError> {
    let rollup_query = RollupQ {
        from: q.from,
        to: q.to,
        business_id: None,
    };
    let Json(rollup) =
        tax_rollup(State(s.clone()), user, axum::extract::Query(rollup_query)).await?;
    let want_detail = q.detail.unwrap_or(0) != 0;
    let detail = if want_detail {
        // Pull transactions for the same window. Capped at 5000 rows
        // for the PDF detail listing — anyone needing the full audit
        // trail beyond that can grab the CSV.
        sqlx::query_as::<_, (DateTime<Utc>, String, Decimal, Option<String>)>(
            "SELECT t.posted_at,
                    COALESCE(t.merchant_raw, '') AS merchant,
                    t.amount,
                    c.name AS category
               FROM transactions t
               JOIN financial_accounts a ON a.id = t.account_id
               LEFT JOIN expense_categories c ON c.id = t.category_id
              WHERE a.user_id = $1
                AND (t.posted_at >= $2::date OR $2 IS NULL)
                AND (t.posted_at <  ($3::date + INTERVAL '1 day') OR $3 IS NULL)
              ORDER BY t.posted_at
              LIMIT 5000",
        )
        .bind(user.id)
        .bind(q.from)
        .bind(q.to)
        .fetch_all(&s.pool)
        .await
        .unwrap_or_default()
    } else {
        Vec::new()
    };

    let bytes = render_tax_pdf(&rollup, want_detail.then_some(&detail));
    let filename = format!(
        "tax-rollup-{}-to-{}{}.pdf",
        rollup.from,
        rollup.to,
        if want_detail { "-detail" } else { "" },
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(bytes))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("pdf response: {e}")))
}

fn render_tax_pdf(
    rollup: &TaxRollupResponse,
    detail: Option<&Vec<(DateTime<Utc>, String, Decimal, Option<String>)>>,
) -> Vec<u8> {
    use printpdf::*;
    let (doc, page1, layer1) =
        PdfDocument::new("TraderView Tax Rollup", Mm(215.9), Mm(279.4), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let font_b = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let mono = doc.add_builtin_font(BuiltinFont::Courier).unwrap();

    let mut layer = doc.get_page(page1).get_layer(layer1);
    let mut y = 260.0_f32;
    let left = 15.0_f32;
    let _ = &mut y; // silence "value never read" — overwritten on add_page paths

    // ── Header ───────────────────────────────────────────────────────
    layer.use_text(
        format!("TraderView Tax Rollup — {} → {}", rollup.from, rollup.to),
        16.0,
        Mm(left),
        Mm(y),
        &font_b,
    );
    y -= 6.0;
    layer.use_text(
        format!(
            "{} receipts · {} items examined",
            rollup.receipts_counted, rollup.items_counted
        ),
        9.0,
        Mm(left),
        Mm(y),
        &font,
    );
    y -= 10.0;

    // ── 4-stat strip ──────────────────────────────────────────────────
    let stats = [
        ("Schedule C (Business)", rollup.business.grand_total),
        ("Schedule E (Rental)", rollup.rental.grand_total),
        ("Personal", rollup.personal.grand_total),
        ("Unclassified", rollup.unclassified.grand_total),
    ];
    for (i, (label, total)) in stats.iter().enumerate() {
        let x = left + (i as f32) * 46.0;
        layer.use_text(*label, 8.0, Mm(x), Mm(y), &font);
        layer.use_text(format!("${total:.2}"), 12.0, Mm(x), Mm(y - 5.0), &mono);
    }
    y -= 14.0;

    // ── Schedule C section ───────────────────────────────────────────
    layer.use_text(
        "SCHEDULE C — BUSINESS BREAKDOWN",
        10.0,
        Mm(left),
        Mm(y),
        &font_b,
    );
    y -= 6.0;
    if rollup.business.categories.is_empty() {
        layer.use_text("(none in range)", 9.0, Mm(left), Mm(y), &font);
        y -= 6.0;
    } else {
        // Column headers
        layer.use_text("Line", 8.0, Mm(left), Mm(y), &font_b);
        layer.use_text("Category", 8.0, Mm(left + 16.0), Mm(y), &font_b);
        layer.use_text("Total", 8.0, Mm(left + 130.0), Mm(y), &font_b);
        y -= 4.0;
        for c in &rollup.business.categories {
            if y < 25.0 {
                let (next_page, next_layer) = doc.add_page(Mm(215.9), Mm(279.4), "next");
                layer = doc.get_page(next_page).get_layer(next_layer);
                y = 265.0;
            }
            let line = c
                .schedule_c_line
                .map(|l| format!("C{l}"))
                .unwrap_or_default();
            layer.use_text(line, 9.0, Mm(left), Mm(y), &mono);
            layer.use_text(&c.category, 9.0, Mm(left + 16.0), Mm(y), &font);
            layer.use_text(
                format!("${:.2}", c.total),
                9.0,
                Mm(left + 130.0),
                Mm(y),
                &mono,
            );
            y -= 5.0;
        }
    }
    y -= 6.0;

    // ── Schedule E section ───────────────────────────────────────────
    if y < 60.0 {
        let (np, nl) = doc.add_page(Mm(215.9), Mm(279.4), "next");
        layer = doc.get_page(np).get_layer(nl);
        y = 265.0;
    }
    layer.use_text(
        "SCHEDULE E — RENTAL BREAKDOWN",
        10.0,
        Mm(left),
        Mm(y),
        &font_b,
    );
    y -= 6.0;
    if rollup.rental.properties.is_empty() {
        layer.use_text("(none in range)", 9.0, Mm(left), Mm(y), &font);
    } else {
        for prop in &rollup.rental.properties {
            if y < 25.0 {
                let (np, nl) = doc.add_page(Mm(215.9), Mm(279.4), "next");
                layer = doc.get_page(np).get_layer(nl);
                y = 265.0;
            }
            let pname = prop
                .property_name
                .clone()
                .unwrap_or_else(|| "(unassigned)".into());
            layer.use_text(
                format!("{pname} — ${:.2}", prop.grand_total),
                9.5,
                Mm(left),
                Mm(y),
                &font_b,
            );
            y -= 5.0;
            for c in &prop.categories {
                if y < 25.0 {
                    let (np, nl) = doc.add_page(Mm(215.9), Mm(279.4), "next");
                    layer = doc.get_page(np).get_layer(nl);
                    y = 265.0;
                }
                let line = c
                    .schedule_e_line
                    .map(|l| format!("E{l}"))
                    .unwrap_or_default();
                layer.use_text(line, 9.0, Mm(left + 4.0), Mm(y), &mono);
                layer.use_text(&c.category, 9.0, Mm(left + 20.0), Mm(y), &font);
                layer.use_text(
                    format!("${:.2}", c.total),
                    9.0,
                    Mm(left + 130.0),
                    Mm(y),
                    &mono,
                );
                y -= 5.0;
            }
            y -= 3.0;
        }
    }

    // ── Detail listing (optional) ────────────────────────────────────
    if let Some(rows) = detail {
        let (np, nl) = doc.add_page(Mm(215.9), Mm(279.4), "detail");
        layer = doc.get_page(np).get_layer(nl);
        y = 265.0;
        layer.use_text("TRANSACTION DETAIL", 10.0, Mm(left), Mm(y), &font_b);
        y -= 6.0;
        layer.use_text("Date", 8.0, Mm(left), Mm(y), &font_b);
        layer.use_text("Merchant", 8.0, Mm(left + 24.0), Mm(y), &font_b);
        layer.use_text("Category", 8.0, Mm(left + 110.0), Mm(y), &font_b);
        layer.use_text("Amount", 8.0, Mm(left + 165.0), Mm(y), &font_b);
        y -= 4.0;
        for (posted, merchant, amount, category) in rows {
            if y < 18.0 {
                let (np, nl) = doc.add_page(Mm(215.9), Mm(279.4), "detail");
                layer = doc.get_page(np).get_layer(nl);
                y = 265.0;
            }
            let date_str = posted.date_naive().to_string();
            let mname: String = merchant.chars().take(40).collect();
            let cname: String = category
                .clone()
                .unwrap_or_default()
                .chars()
                .take(22)
                .collect();
            layer.use_text(date_str, 8.0, Mm(left), Mm(y), &mono);
            layer.use_text(mname, 8.0, Mm(left + 24.0), Mm(y), &font);
            layer.use_text(cname, 8.0, Mm(left + 110.0), Mm(y), &font);
            layer.use_text(
                format!("${:.2}", amount),
                8.0,
                Mm(left + 165.0),
                Mm(y),
                &mono,
            );
            y -= 4.0;
        }
    }

    let mut buf = Vec::with_capacity(64 * 1024);
    // Save failure → return an empty PDF rather than a 500. The browser
    // renders nothing; the rollup JSON endpoint stays the source of truth.
    let _ = doc.save(&mut std::io::BufWriter::new(&mut buf));
    buf
}

// --- OCR engine status -------------------------------------------------
//
// OCR delegates to the system `tesseract` CLI binary (see
// `engine.rs`). No model files need downloading — `brew install
// tesseract` (macOS) / `apt install tesseract-ocr` (Linux) ships
// both the binary and `eng.traineddata`.
//
// `status` reports whether tesseract is on PATH, its version, and
// where its tessdata lives.
// `download` is preserved for API back-compat with the previous
// PaddleOCR pipeline — it now just probes tesseract and returns a
// readiness report.

// `monkt/paddleocr-onnx` doesn't ship a "multilingual" recognition
// bundle — verified the actual `languages/` directory contents on
// the HuggingFace repo: only `english`, `arabic`, `chinese`, `eslav`,
// `greek`, `hindi`, `korean`, `latin`, `tamil`, `telugu`, `thai` exist.
// Sticking with English. The `[UNK]` artefacts on thermal-printer
// receipts come from the recognition model's CTC decoder, not from
// dictionary gaps — switching to a different language model wouldn't
// help. The parse layer compensates with `[UNK]` → space
// preprocessing in `traderview_ocr::parse::structure`.
#[derive(Serialize)]
struct OcrModelsStatus {
    /// Reported as `model_dir` for back-compat with the legacy
    /// frontend; with tesseract this is the path where a user-supplied
    /// override `eng.traineddata` would be picked up.
    model_dir: String,
    /// Always true when `tesseract` is on PATH. The frontend uses this
    /// to decide whether to surface the "install tesseract" affordance.
    ready: bool,
    /// `tesseract --version` first line (e.g. `tesseract 5.5.2`), or
    /// `"not installed"` when the binary isn't found.
    tesseract_version: String,
    /// Whether `eng.traineddata` is reachable. Tesseract uses
    /// `TESSDATA_PREFIX` if set, otherwise the compile-time default.
    eng_traineddata_present: bool,
}

async fn ocr_models_status(
    State(s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<OcrModelsStatus>, ApiError> {
    let dir = s.ocr_model_dir();

    let version = tokio::process::Command::new("tesseract")
        .arg("--version")
        .output()
        .await
        .ok()
        .and_then(|o| {
            if o.status.success() {
                // `tesseract --version` writes the version line to
                // stderr on some builds, stdout on others — take
                // whichever's non-empty.
                let s1 = String::from_utf8_lossy(&o.stdout);
                let s2 = String::from_utf8_lossy(&o.stderr);
                let first = s1
                    .lines()
                    .next()
                    .or_else(|| s2.lines().next())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                if !first.is_empty() {
                    Some(first)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| "not installed".to_string());
    let ready = version != "not installed";

    // `eng.traineddata` lookup: honour user-supplied override in
    // the receipts model dir, then try common system paths.
    let candidates: Vec<std::path::PathBuf> = vec![
        dir.join("eng.traineddata"),
        std::path::PathBuf::from("/opt/homebrew/share/tessdata/eng.traineddata"),
        std::path::PathBuf::from("/usr/local/share/tessdata/eng.traineddata"),
        std::path::PathBuf::from("/usr/share/tessdata/eng.traineddata"),
        std::path::PathBuf::from("/usr/share/tesseract-ocr/5/tessdata/eng.traineddata"),
        std::path::PathBuf::from("/usr/share/tesseract-ocr/4.00/tessdata/eng.traineddata"),
    ];
    let mut eng_present = false;
    for c in candidates {
        if tokio::fs::metadata(&c).await.is_ok() {
            eng_present = true;
            break;
        }
    }

    Ok(Json(OcrModelsStatus {
        model_dir: dir.display().to_string(),
        ready: ready && eng_present,
        tesseract_version: version,
        eng_traineddata_present: eng_present,
    }))
}

#[derive(Serialize)]
struct OcrModelsDownloadResult {
    /// Always 0 — tesseract uses the system install, nothing to fetch.
    bytes_total: u64,
    /// Legacy: empty for tesseract.
    downloaded: Vec<String>,
    /// Legacy: empty for tesseract.
    skipped: Vec<String>,
    /// Human-readable status message — points the user at `brew
    /// install tesseract` when the binary is missing, otherwise a
    /// "ready" string.
    message: String,
    model_dir: String,
}

#[derive(Deserialize)]
struct OcrModelsDownloadQ {
    /// Preserved for API back-compat with the previous PaddleOCR
    /// pipeline. Has no effect with the tesseract backend (nothing to
    /// re-download); kept so older frontend builds continue parsing.
    #[serde(default)]
    #[allow(dead_code)]
    force: Option<u8>,
}

async fn ocr_models_download(
    State(s): State<AppState>,
    _user: AuthUser,
    axum::extract::Query(_q): axum::extract::Query<OcrModelsDownloadQ>,
) -> Result<Json<OcrModelsDownloadResult>, ApiError> {
    let dir = s.ocr_model_dir();
    tokio::fs::create_dir_all(&dir).await.ok();
    let tess_present = tokio::process::Command::new("tesseract")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false);
    let message = if tess_present {
        "tesseract is installed and ready — no download required".to_string()
    } else {
        "tesseract not on PATH — install with `brew install tesseract` (macOS) or \
         `apt install tesseract-ocr tesseract-ocr-eng` (Linux)"
            .to_string()
    };
    Ok(Json(OcrModelsDownloadResult {
        bytes_total: 0,
        downloaded: Vec::new(),
        skipped: Vec::new(),
        message,
        model_dir: dir.display().to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── extension_of ─────────────────────────────────────────────────────

    #[test]
    fn extension_of_picks_last_dot_segment() {
        assert_eq!(extension_of("photo.jpg"), "jpg");
        assert_eq!(extension_of("scan.tar.gz"), "gz");
    }

    #[test]
    fn extension_of_lowercases() {
        assert_eq!(extension_of("RECEIPT.PDF"), "pdf");
        assert_eq!(extension_of("phone.JPEG"), "jpeg");
    }

    #[test]
    fn extension_of_returns_bin_when_no_dot() {
        // The bug we just fixed: pre-fix this returned the full filename
        // because rsplit('.').next() returns the whole string when no
        // separator is present.
        assert_eq!(
            extension_of("noext"),
            "bin",
            "extensionless filenames must fall back to bin"
        );
        assert_eq!(extension_of("scan"), "bin");
    }

    #[test]
    fn extension_of_returns_bin_when_filename_ends_with_dot() {
        // "scan." → empty extension → bin (not "").
        assert_eq!(extension_of("scan."), "bin");
    }

    #[test]
    fn extension_of_handles_empty_string() {
        assert_eq!(extension_of(""), "bin");
    }

    // ─── is_acceptable_mime ───────────────────────────────────────────────

    #[test]
    fn accepts_image_jpeg_png_webp_bmp_and_pdf() {
        for m in [
            "image/jpeg",
            "image/png",
            "image/webp",
            "image/bmp",
            "application/pdf",
        ] {
            assert!(is_acceptable_mime(m), "must accept {m}");
        }
    }

    #[test]
    fn accepts_mime_with_charset_suffix() {
        // Browser sometimes appends `; charset=...`.
        assert!(is_acceptable_mime("image/jpeg; charset=binary"));
    }

    #[test]
    fn accepts_case_insensitive_mime() {
        assert!(is_acceptable_mime("IMAGE/JPEG"));
        assert!(is_acceptable_mime("Application/PDF"));
    }

    #[test]
    fn rejects_unsupported_mimes() {
        for m in [
            "text/plain",
            "image/gif",
            "image/tiff",
            "application/zip",
            "",
            "video/mp4",
        ] {
            assert!(!is_acceptable_mime(m), "must reject {m}");
        }
    }

    // ─── guess_mime ───────────────────────────────────────────────────────

    #[test]
    fn guess_mime_handles_known_extensions() {
        assert_eq!(guess_mime("a.jpg"), "image/jpeg");
        assert_eq!(guess_mime("a.jpeg"), "image/jpeg");
        assert_eq!(guess_mime("a.png"), "image/png");
        assert_eq!(guess_mime("a.webp"), "image/webp");
        assert_eq!(guess_mime("a.bmp"), "image/bmp");
        assert_eq!(guess_mime("a.pdf"), "application/pdf");
    }

    #[test]
    fn guess_mime_unknown_falls_back_to_octet_stream() {
        assert_eq!(guess_mime("notes.txt"), "application/octet-stream");
        assert_eq!(guess_mime("noext"), "application/octet-stream");
    }

    // ─── sanitize_disposition ─────────────────────────────────────────────

    #[test]
    fn sanitize_disposition_replaces_quotes() {
        // Embedded `"` would close the header value early.
        assert_eq!(sanitize_disposition(r#"sneaky".jpg"#), "sneaky_.jpg");
    }

    #[test]
    fn sanitize_disposition_strips_crlf_injection() {
        // CR/LF would inject a new HTTP header.
        let bad = "evil.jpg\r\nSet-Cookie: pwned=1";
        let safe = sanitize_disposition(bad);
        assert!(!safe.contains('\r'));
        assert!(!safe.contains('\n'));
    }

    #[test]
    fn sanitize_disposition_strips_nul_and_other_controls() {
        let bad = "weird\x00name\x07.png";
        let safe = sanitize_disposition(bad);
        for c in safe.chars() {
            assert!(!c.is_control(), "leftover control char {c:?}");
        }
    }

    #[test]
    fn sanitize_disposition_preserves_normal_filenames() {
        // Don't mangle legitimate filenames.
        assert_eq!(
            sanitize_disposition("receipt-2026-05-27.jpg"),
            "receipt-2026-05-27.jpg"
        );
        assert_eq!(
            sanitize_disposition("Café Latté.pdf"),
            "Café Latté.pdf",
            "Unicode letters/spaces must survive — only controls + quotes are sanitized"
        );
    }
}
