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
use axum::extract::{Multipart, Path, State};
use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use traderview_ocr::matcher;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_receipts).post(upload_receipt))
        .route("/:id", get(get_receipt_blob))
        .route("/:id/meta", get(get_receipt_meta))
        .route("/:id/matches", get(receipt_matches))
        .route("/:id/attach", post(attach_receipt))
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
            match_score: r.match_score,
            error_message: r.error_message,
            created_at: r.created_at,
        }
    }
}

// --- list ---------------------------------------------------------------

async fn list_receipts(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<Receipt>>, ApiError> {
    let rows: Vec<ReceiptRow> = sqlx::query_as(
        "SELECT id, user_id, transaction_id, filename, sha256, mime, bytes_len,
                ocr_status::text, ocr_text, ocr_merchant, ocr_total, ocr_date,
                ocr_confidence, match_score, error_message, created_at
           FROM receipts WHERE user_id = $1
          ORDER BY created_at DESC LIMIT 200",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
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

    // De-dupe per user + sha. If it exists, return the existing row.
    let existing: Option<ReceiptRow> = sqlx::query_as(
        "SELECT id, user_id, transaction_id, filename, sha256, mime, bytes_len,
                ocr_status::text, ocr_text, ocr_merchant, ocr_total, ocr_date,
                ocr_confidence, match_score, error_message, created_at
           FROM receipts WHERE user_id = $1 AND sha256 = $2",
    )
    .bind(user.id)
    .bind(&sha)
    .fetch_optional(&s.pool)
    .await?;
    if let Some(r) = existing {
        return Ok(Json(r.into()));
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
                   ocr_confidence, match_score, error_message, created_at",
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
    let bg_state = s.clone();
    let receipt_id = row.id;
    let mime_owned = mime.clone();
    tokio::spawn(async move {
        if let Err(e) = run_ocr(bg_state, receipt_id, bytes, mime_owned).await {
            tracing::error!(receipt = %receipt_id, error = %e, "ocr job failed");
        }
    });

    Ok(Json(row.into()))
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
        Ok(ocr) => {
            sqlx::query(
                "UPDATE receipts SET
                    ocr_status = 'done'::ocr_status_t,
                    ocr_text = $2,
                    ocr_merchant = $3,
                    ocr_total = $4,
                    ocr_date = $5,
                    ocr_confidence = $6,
                    error_message = NULL
                  WHERE id = $1",
            )
            .bind(receipt_id)
            .bind(&ocr.text)
            .bind(&ocr.merchant)
            .bind(ocr.total)
            .bind(ocr.date)
            .bind(ocr.confidence)
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
    let row: Option<(Uuid, String, String, String)> = sqlx::query_as(
        "SELECT user_id, storage_path, mime, filename FROM receipts WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&s.pool)
    .await?;
    let (owner, rel_path, mime, filename) = row.ok_or(ApiError::NotFound)?;
    if owner != user.id {
        return Err(ApiError::Forbidden);
    }
    let abs = s.receipts_dir().join(&rel_path);
    let bytes = tokio::fs::read(&abs).await?;
    let disposition = format!("inline; filename=\"{}\"", filename.replace('"', "_"));
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_DISPOSITION, disposition)
        .body(Body::from(bytes))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("response build: {e}")))
}

async fn get_receipt_meta(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Receipt>, ApiError> {
    let row: Option<ReceiptRow> = sqlx::query_as(
        "SELECT id, user_id, transaction_id, filename, sha256, mime, bytes_len,
                ocr_status::text, ocr_text, ocr_merchant, ocr_total, ocr_date,
                ocr_confidence, match_score, error_message, created_at
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

async fn receipt_matches(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<CandidateMatch>>, ApiError> {
    let row: Option<(Uuid, Option<String>, Option<Decimal>, Option<NaiveDate>)> = sqlx::query_as(
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
        .filter_map(|m| by_id.get(&m.id).map(|brief| CandidateMatch {
            transaction: TxBrief {
                id: brief.id,
                account_id: brief.account_id,
                posted_at: brief.posted_at,
                amount: brief.amount,
                merchant_raw: brief.merchant_raw.clone(),
                merchant_normalized: brief.merchant_normalized.clone(),
            },
            score: m.score,
        }))
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
    let owner_check: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM receipts WHERE id = $1")
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
                   ocr_confidence, match_score, error_message, created_at",
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
    if m.starts_with("image/jpeg") { Some("jpg".into()) }
    else if m.starts_with("image/png") { Some("png".into()) }
    else if m.starts_with("image/webp") { Some("webp".into()) }
    else if m.starts_with("image/bmp") { Some("bmp".into()) }
    else if m.starts_with("application/pdf") { Some("pdf".into()) }
    else { None }
}

fn extension_of(filename: &str) -> String {
    filename
        .rsplit('.')
        .next()
        .unwrap_or("bin")
        .to_ascii_lowercase()
}
