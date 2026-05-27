//! Business-expense routes (Schedule C tracking).
//!
//! Mounted as a sub-router under `/api/expense`. Auth is the same `AuthUser`
//! extractor used by the trading routes.
//!
//! Receipts + OCR + Schedule C report endpoints land in subsequent files
//! (this module covers accounts, categories, transactions, imports, rules).

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{DefaultBodyLimit, Multipart, Path, Query, State};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use traderview_expense::dedup::{AccountKind as DedupKind, DedupCandidate};
use traderview_expense::rules::{CompiledRules, PatternKind as RulePatternKind, Rule};
use traderview_expense::{seed_rules, ExpenseSource};
use uuid::Uuid;

/// Cap on a single CSV upload. A year of credit-card statements is well
/// under 1 MB; we leave generous headroom but reject the malicious case
/// where a client streams gigabytes to fill disk + the in-memory `Vec<u8>`.
const CSV_UPLOAD_MAX_BYTES: usize = 50 * 1024 * 1024;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/accounts", get(list_accounts).post(create_account))
        .route("/categories", get(list_categories))
        .route("/transactions", get(list_transactions))
        .route("/transactions/:id", patch(update_transaction))
        .route("/import", post(import_csv))
        .route("/rules", get(list_rules).post(create_rule))
        .route("/rules/:id", delete(delete_rule))
        .route("/rules/seed", post(seed_default_rules))
        .route("/report/schedule_c", get(schedule_c_report))
        // Tax-workshop calculators — pure-compute endpoints. Each returns
        // the report shape exported by the corresponding traderview-expense
        // module, no DB state changes.
        .route("/calc/self-employment-tax", post(calc_self_employment_tax))
        .route("/calc/home-office",         post(calc_home_office))
        .route("/calc/mileage",             post(calc_mileage))
        .route("/calc/quarterly-tax",       post(calc_quarterly_tax))
        .route("/subscriptions/detect",     get(detect_subscriptions))
        // Bound the multipart upload on /import so import_csv can't fill
        // memory + disk on a runaway client. Receipts have their own,
        // smaller limit set in receipt_routes.
        .layer(DefaultBodyLimit::max(CSV_UPLOAD_MAX_BYTES))
        .nest("/receipts", crate::receipt_routes::router())
}

// --- accounts -------------------------------------------------------------

#[derive(Serialize)]
struct FinancialAccount {
    id: Uuid,
    user_id: Uuid,
    kind: String,
    source: String,
    name: String,
    base_currency: String,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct FinancialAccountRow {
    id: Uuid,
    user_id: Uuid,
    kind: String,
    source: String,
    name: String,
    base_currency: String,
    created_at: DateTime<Utc>,
}

impl From<FinancialAccountRow> for FinancialAccount {
    fn from(r: FinancialAccountRow) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            kind: r.kind,
            source: r.source,
            name: r.name,
            base_currency: r.base_currency,
            created_at: r.created_at,
        }
    }
}

async fn list_accounts(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<FinancialAccount>>, ApiError> {
    let rows: Vec<FinancialAccountRow> = sqlx::query_as(
        "SELECT id, user_id, kind::text, source, name, base_currency, created_at
           FROM financial_accounts WHERE user_id = $1 ORDER BY created_at ASC",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

#[derive(Deserialize)]
struct CreateAccountBody {
    kind: String,
    source: String,
    name: String,
    #[serde(default = "default_usd")]
    base_currency: String,
}

fn default_usd() -> String {
    "USD".into()
}

async fn create_account(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateAccountBody>,
) -> Result<Json<FinancialAccount>, ApiError> {
    if !matches!(body.kind.as_str(), "bank" | "credit_card" | "marketplace") {
        return Err(ApiError::BadRequest("kind must be bank|credit_card|marketplace".into()));
    }
    let row: FinancialAccountRow = sqlx::query_as(
        "INSERT INTO financial_accounts (user_id, kind, source, name, base_currency)
              VALUES ($1, $2::financial_account_kind_t, $3, $4, $5)
         RETURNING id, user_id, kind::text, source, name, base_currency, created_at",
    )
    .bind(user.id)
    .bind(&body.kind)
    .bind(&body.source)
    .bind(&body.name)
    .bind(&body.base_currency)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row.into()))
}

// --- categories ----------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct ExpenseCategory {
    code: String,
    schedule_c_line: String,
    label: String,
    deduction_pct: Decimal,
    sort_order: i32,
}

async fn list_categories(
    State(s): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<ExpenseCategory>>, ApiError> {
    let rows: Vec<ExpenseCategory> = sqlx::query_as(
        "SELECT code, schedule_c_line, label, deduction_pct, sort_order
           FROM expense_categories ORDER BY sort_order ASC",
    )
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

// --- transactions --------------------------------------------------------

#[derive(Serialize)]
struct Transaction {
    id: Uuid,
    account_id: Uuid,
    posted_at: DateTime<Utc>,
    amount: Decimal,
    currency: String,
    merchant_raw: String,
    merchant_normalized: String,
    description: String,
    category_code: Option<String>,
    is_business: bool,
    is_transfer: bool,
    transfer_peer_id: Option<Uuid>,
    notes: String,
}

#[derive(sqlx::FromRow)]
struct TransactionRow {
    id: Uuid,
    account_id: Uuid,
    posted_at: DateTime<Utc>,
    amount: Decimal,
    currency: String,
    merchant_raw: String,
    merchant_normalized: String,
    description: String,
    category_code: Option<String>,
    is_business: bool,
    is_transfer: bool,
    transfer_peer_id: Option<Uuid>,
    notes: String,
}

impl From<TransactionRow> for Transaction {
    fn from(r: TransactionRow) -> Self {
        Self {
            id: r.id,
            account_id: r.account_id,
            posted_at: r.posted_at,
            amount: r.amount,
            currency: r.currency,
            merchant_raw: r.merchant_raw,
            merchant_normalized: r.merchant_normalized,
            description: r.description,
            category_code: r.category_code,
            is_business: r.is_business,
            is_transfer: r.is_transfer,
            transfer_peer_id: r.transfer_peer_id,
            notes: r.notes,
        }
    }
}

#[derive(Deserialize)]
struct ListTxQuery {
    account_id: Option<Uuid>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    category: Option<String>,
    is_business: Option<bool>,
    is_transfer: Option<bool>,
    search: Option<String>,
    #[serde(default = "default_tx_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_tx_limit() -> i64 {
    200
}

/// Hard cap on a single transactions-list query. Without this, a client could
/// pass `?limit=1000000` and force the server to materialize a multi-million
/// row Vec into memory.
const MAX_TX_LIMIT: i64 = 1000;

/// Escape `%`, `_`, and `\` so the ILIKE pattern from a user search treats
/// them as literal characters, not wildcards. Without this, searching "100%"
/// matches every transaction (% is the ILIKE wildcard for any sequence).
/// The bind is already parameterized so this is a UX/correctness fix, not a
/// SQL-injection fix.
pub(crate) fn escape_ilike_pattern(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Clamp the caller's `limit` to [1, MAX_TX_LIMIT]. Negative / zero would
/// error in SQL; huge values would OOM. `offset` is clamped at 0 lower bound.
pub(crate) fn clamp_pagination(limit: i64, offset: i64) -> (i64, i64) {
    let limit = limit.clamp(1, MAX_TX_LIMIT);
    let offset = offset.max(0);
    (limit, offset)
}

async fn list_transactions(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<ListTxQuery>,
) -> Result<Json<Vec<Transaction>>, ApiError> {
    let (limit, offset) = clamp_pagination(q.limit, q.offset);
    let search_pattern = q.search.as_deref().map(escape_ilike_pattern);
    // Always scope by the user's accounts; a future "ALL" account picker just
    // omits account_id and falls through to the user-wide JOIN below.
    let rows: Vec<TransactionRow> = sqlx::query_as(
        "SELECT t.id, t.account_id, t.posted_at, t.amount, t.currency,
                t.merchant_raw, t.merchant_normalized, t.description,
                t.category_code, t.is_business, t.is_transfer, t.transfer_peer_id, t.notes
           FROM transactions t
           JOIN financial_accounts a ON a.id = t.account_id
          WHERE a.user_id = $1
            AND ($2::uuid IS NULL OR t.account_id = $2)
            AND ($3::date IS NULL OR t.posted_at >= $3::date)
            AND ($4::date IS NULL OR t.posted_at < ($4::date + INTERVAL '1 day'))
            AND ($5::text IS NULL OR t.category_code = $5)
            AND ($6::bool IS NULL OR t.is_business = $6)
            AND ($7::bool IS NULL OR t.is_transfer = $7)
            AND ($8::text IS NULL OR
                  t.merchant_normalized ILIKE '%' || $8 || '%' ESCAPE '\\' OR
                  t.description         ILIKE '%' || $8 || '%' ESCAPE '\\')
          ORDER BY t.posted_at DESC
          LIMIT $9 OFFSET $10",
    )
    .bind(user.id)
    .bind(q.account_id)
    .bind(q.from)
    .bind(q.to)
    .bind(q.category)
    .bind(q.is_business)
    .bind(q.is_transfer)
    .bind(search_pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

#[derive(Deserialize)]
struct UpdateTransactionBody {
    category_code: Option<Option<String>>, // double-Option lets caller distinguish "clear" vs "unchanged"
    is_business: Option<bool>,
    is_transfer: Option<bool>,
    notes: Option<String>,
}

async fn update_transaction(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateTransactionBody>,
) -> Result<Json<Transaction>, ApiError> {
    ensure_transaction_owner(&s, user.id, id).await?;

    let row: TransactionRow = sqlx::query_as(
        "UPDATE transactions SET
            category_code = CASE WHEN $2::bool THEN $3 ELSE category_code END,
            is_business   = COALESCE($4, is_business),
            is_transfer   = COALESCE($5, is_transfer),
            notes         = COALESCE($6, notes)
          WHERE id = $1
         RETURNING id, account_id, posted_at, amount, currency,
                   merchant_raw, merchant_normalized, description,
                   category_code, is_business, is_transfer, transfer_peer_id, notes",
    )
    .bind(id)
    .bind(body.category_code.is_some())
    .bind(body.category_code.unwrap_or(None))
    .bind(body.is_business)
    .bind(body.is_transfer)
    .bind(body.notes)
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(row.into()))
}

// --- import (multipart CSV) ----------------------------------------------

#[derive(Serialize)]
struct ImportResult {
    import_id: Uuid,
    row_count: usize,
    inserted_count: usize,
    categorized_count: usize,
    transfer_pairs: usize,
    duplicate: bool,
}

async fn import_csv(
    State(s): State<AppState>,
    user: AuthUser,
    mut mp: Multipart,
) -> Result<Json<ImportResult>, ApiError> {
    let mut account_id: Option<Uuid> = None;
    let mut source: Option<ExpenseSource> = None;
    let mut filename: String = String::new();
    let mut file_bytes: Vec<u8> = Vec::new();

    while let Some(field) = mp
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("multipart: {e}")))?
    {
        match field.name().unwrap_or("") {
            "account_id" => {
                let v = field.text().await.map_err(|e| ApiError::BadRequest(e.to_string()))?;
                account_id = Some(Uuid::parse_str(&v).map_err(|e| ApiError::BadRequest(e.to_string()))?);
            }
            "source" => {
                let v = field.text().await.map_err(|e| ApiError::BadRequest(e.to_string()))?;
                source = ExpenseSource::parse_str(&v);
                if source.is_none() {
                    return Err(ApiError::BadRequest(format!("unknown source: {v}")));
                }
            }
            "file" => {
                filename = field.file_name().unwrap_or("upload.csv").to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(e.to_string()))?;
                file_bytes = bytes.to_vec();
            }
            _ => {}
        }
    }

    let account_id = account_id.ok_or_else(|| ApiError::BadRequest("missing account_id".into()))?;
    let source = source.ok_or_else(|| ApiError::BadRequest("missing source".into()))?;
    if file_bytes.is_empty() {
        return Err(ApiError::BadRequest("missing file".into()));
    }

    ensure_account_owner(&s, user.id, account_id).await?;

    let mut h = Sha256::new();
    h.update(&file_bytes);
    let sha = hex::encode(h.finalize());

    // Dedupe at the file level: same sha already imported into this account.
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM expense_imports WHERE account_id = $1 AND sha256 = $2",
    )
    .bind(account_id)
    .bind(&sha)
    .fetch_optional(&s.pool)
    .await?;
    if let Some((existing_id,)) = existing {
        return Ok(Json(ImportResult {
            import_id: existing_id,
            row_count: 0,
            inserted_count: 0,
            categorized_count: 0,
            transfer_pairs: 0,
            duplicate: true,
        }));
    }

    // Parse via the source-specific parser. All four parsers are stubs that
    // return Unsupported until a real redacted sample is provided — matches
    // the existing Webull discipline.
    let parsed = traderview_expense::parse(source, &file_bytes).map_err(|e| match e {
        traderview_expense::ImportError::Unsupported(m) => ApiError::BadRequest(m),
        traderview_expense::ImportError::Parse(m) => ApiError::BadRequest(m),
        other => ApiError::Internal(anyhow::anyhow!("{other}")),
    })?;

    let row_count = parsed.len();
    let import_id: Uuid = {
        let (id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO expense_imports (account_id, source, filename, sha256, row_count)
                  VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(account_id)
        .bind(source.as_str())
        .bind(&filename)
        .bind(&sha)
        .bind(row_count as i32)
        .fetch_one(&s.pool)
        .await?;
        id
    };

    // Insert transactions one at a time so we can ON CONFLICT DO NOTHING and
    // count how many actually landed (a batched VALUES form returns no per-row
    // status without an explicit RETURNING xmax trick). Volume per import is
    // typically <500 rows so this is fine.
    let mut inserted: Vec<Uuid> = Vec::with_capacity(row_count);
    for tx in &parsed {
        let res: Option<(Uuid,)> = sqlx::query_as(
            "INSERT INTO transactions
                (account_id, posted_at, amount, currency,
                 merchant_raw, merchant_normalized, description, raw, import_id)
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
              ON CONFLICT (account_id, posted_at, amount, merchant_raw) DO NOTHING
              RETURNING id",
        )
        .bind(account_id)
        .bind(tx.posted_at)
        .bind(tx.amount)
        .bind(&tx.currency)
        .bind(&tx.merchant_raw)
        .bind(&tx.merchant_normalized)
        .bind(&tx.description)
        .bind(&tx.raw)
        .bind(import_id)
        .fetch_optional(&s.pool)
        .await?;
        if let Some((id,)) = res {
            inserted.push(id);
        }
    }

    sqlx::query("UPDATE expense_imports SET inserted_count = $1 WHERE id = $2")
        .bind(inserted.len() as i32)
        .bind(import_id)
        .execute(&s.pool)
        .await?;

    // Apply user's merchant rules to the newly inserted transactions.
    let categorized_count = apply_rules_to_user(&s, user.id, &inserted).await?;

    // Run transfer dedup over the user's last 30 days (cheap, catches the
    // common case of credit-card payments showing up in both accounts).
    let transfer_pairs = run_transfer_dedup(&s, user.id).await?;

    Ok(Json(ImportResult {
        import_id,
        row_count,
        inserted_count: inserted.len(),
        categorized_count,
        transfer_pairs,
        duplicate: false,
    }))
}

async fn apply_rules_to_user(
    s: &AppState,
    user_id: Uuid,
    transaction_ids: &[Uuid],
) -> Result<usize, ApiError> {
    if transaction_ids.is_empty() {
        return Ok(0);
    }
    let rules: Vec<DbRule> = sqlx::query_as(
        "SELECT pattern, pattern_kind::text, category_code, is_business
           FROM merchant_rules WHERE user_id = $1 ORDER BY priority ASC, created_at ASC",
    )
    .bind(user_id)
    .fetch_all(&s.pool)
    .await?;
    if rules.is_empty() {
        return Ok(0);
    }
    let compiled = CompiledRules::compile(
        &rules
            .iter()
            .map(|r| Rule {
                pattern: r.pattern.clone(),
                pattern_kind: match r.pattern_kind.as_str() {
                    "regex" => RulePatternKind::Regex,
                    _ => RulePatternKind::Substring,
                },
                category_code: r.category_code.clone(),
                is_business: r.is_business,
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("rule regex: {e}")))?;

    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, merchant_normalized
           FROM transactions
          WHERE id = ANY($1) AND category_code IS NULL",
    )
    .bind(transaction_ids)
    .fetch_all(&s.pool)
    .await?;

    let mut hits = 0usize;
    for (tx_id, normalized) in rows {
        if let Some(m) = compiled.match_one(&normalized) {
            sqlx::query(
                "UPDATE transactions SET category_code = $1, is_business = $2 WHERE id = $3",
            )
            .bind(&m.category_code)
            .bind(m.is_business)
            .bind(tx_id)
            .execute(&s.pool)
            .await?;
            hits += 1;
        }
    }
    Ok(hits)
}

async fn run_transfer_dedup(s: &AppState, user_id: Uuid) -> Result<usize, ApiError> {
    let rows: Vec<DedupRow> = sqlx::query_as(
        "SELECT t.id, a.kind::text as kind, t.posted_at, t.amount, t.description
           FROM transactions t
           JOIN financial_accounts a ON a.id = t.account_id
          WHERE a.user_id = $1
            AND t.is_transfer = FALSE
            AND t.posted_at > now() - INTERVAL '60 days'
          ORDER BY t.posted_at ASC",
    )
    .bind(user_id)
    .fetch_all(&s.pool)
    .await?;

    let cands: Vec<DedupCandidate> = rows
        .iter()
        .map(|r| DedupCandidate {
            id: r.id,
            account_kind: match r.kind.as_str() {
                "credit_card" => DedupKind::CreditCard,
                "marketplace" => DedupKind::Marketplace,
                _ => DedupKind::Bank,
            },
            posted_at: r.posted_at,
            amount: r.amount,
            description_lower: r.description.to_lowercase(),
        })
        .collect();

    let pairs = traderview_expense::dedup::detect_pairs(&cands);
    for (a, b) in &pairs {
        // Mark both rows as transfer and link them to each other.
        sqlx::query(
            "UPDATE transactions SET is_transfer = TRUE, transfer_peer_id = $2 WHERE id = $1",
        )
        .bind(a)
        .bind(b)
        .execute(&s.pool)
        .await?;
        sqlx::query(
            "UPDATE transactions SET is_transfer = TRUE, transfer_peer_id = $2 WHERE id = $1",
        )
        .bind(b)
        .bind(a)
        .execute(&s.pool)
        .await?;
    }
    Ok(pairs.len())
}

#[derive(sqlx::FromRow)]
struct DedupRow {
    id: Uuid,
    kind: String,
    posted_at: DateTime<Utc>,
    amount: Decimal,
    description: String,
}

// --- rules ---------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
struct DbRule {
    pattern: String,
    pattern_kind: String,
    category_code: String,
    is_business: bool,
}

#[derive(Serialize, sqlx::FromRow)]
struct RuleDetail {
    id: Uuid,
    pattern: String,
    pattern_kind: String,
    category_code: String,
    is_business: bool,
    priority: i32,
    hit_count: i64,
    created_at: DateTime<Utc>,
}

async fn list_rules(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<RuleDetail>>, ApiError> {
    let rows: Vec<RuleDetail> = sqlx::query_as(
        "SELECT id, pattern, pattern_kind::text, category_code, is_business,
                priority, hit_count, created_at
           FROM merchant_rules WHERE user_id = $1
          ORDER BY priority ASC, created_at ASC",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
struct CreateRuleBody {
    pattern: String,
    #[serde(default = "default_pattern_kind")]
    pattern_kind: String,
    category_code: String,
    #[serde(default = "yes")]
    is_business: bool,
    #[serde(default = "default_priority")]
    priority: i32,
    /// When true (default), the new rule is also retroactively applied to all
    /// of the user's currently-uncategorized transactions whose normalized
    /// merchant matches. Lets the UI "categorize all STAPLES" in one click.
    #[serde(default = "yes")]
    apply_retroactively: bool,
}

fn default_pattern_kind() -> String {
    "substring".into()
}
fn default_priority() -> i32 {
    100
}
fn yes() -> bool {
    true
}

async fn create_rule(
    State(s): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateRuleBody>,
) -> Result<Json<RuleApplyResult>, ApiError> {
    if !matches!(body.pattern_kind.as_str(), "substring" | "regex") {
        return Err(ApiError::BadRequest("pattern_kind must be substring|regex".into()));
    }
    if body.pattern_kind == "regex" {
        regex::Regex::new(&body.pattern)
            .map_err(|e| ApiError::BadRequest(format!("invalid regex: {e}")))?;
    }
    let (rule_id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO merchant_rules (user_id, pattern, pattern_kind, category_code, is_business, priority)
              VALUES ($1, $2, $3::merchant_pattern_kind_t, $4, $5, $6)
         RETURNING id",
    )
    .bind(user.id)
    .bind(&body.pattern)
    .bind(&body.pattern_kind)
    .bind(&body.category_code)
    .bind(body.is_business)
    .bind(body.priority)
    .fetch_one(&s.pool)
    .await?;

    let applied = if body.apply_retroactively {
        apply_single_rule_retroactively(&s, user.id, &body).await?
    } else {
        0
    };

    Ok(Json(RuleApplyResult { rule_id, applied }))
}

#[derive(Serialize)]
struct RuleApplyResult {
    rule_id: Uuid,
    applied: usize,
}

async fn apply_single_rule_retroactively(
    s: &AppState,
    user_id: Uuid,
    body: &CreateRuleBody,
) -> Result<usize, ApiError> {
    let compiled = CompiledRules::compile(&[Rule {
        pattern: body.pattern.clone(),
        pattern_kind: if body.pattern_kind == "regex" {
            RulePatternKind::Regex
        } else {
            RulePatternKind::Substring
        },
        category_code: body.category_code.clone(),
        is_business: body.is_business,
    }])
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("rule compile: {e}")))?;

    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT t.id, t.merchant_normalized
           FROM transactions t
           JOIN financial_accounts a ON a.id = t.account_id
          WHERE a.user_id = $1 AND t.category_code IS NULL",
    )
    .bind(user_id)
    .fetch_all(&s.pool)
    .await?;

    let mut applied = 0usize;
    for (tx_id, normalized) in rows {
        if compiled.match_one(&normalized).is_some() {
            sqlx::query(
                "UPDATE transactions SET category_code = $1, is_business = $2 WHERE id = $3",
            )
            .bind(&body.category_code)
            .bind(body.is_business)
            .bind(tx_id)
            .execute(&s.pool)
            .await?;
            applied += 1;
        }
    }
    Ok(applied)
}

async fn delete_rule(
    State(s): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    let res = sqlx::query("DELETE FROM merchant_rules WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn seed_default_rules(
    State(s): State<AppState>,
    user: AuthUser,
) -> Result<Json<SeedResult>, ApiError> {
    let existing: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM merchant_rules WHERE user_id = $1")
        .bind(user.id)
        .fetch_one(&s.pool)
        .await?;
    if existing.0 > 0 {
        return Ok(Json(SeedResult { inserted: 0, skipped_existing: existing.0 as usize }));
    }
    let rules = seed_rules::seed();
    let mut inserted = 0usize;
    // Seed rules at priority 200 so user-authored rules (default 100) override.
    for (idx, r) in rules.iter().enumerate() {
        let kind_str = match r.pattern_kind {
            RulePatternKind::Substring => "substring",
            RulePatternKind::Regex => "regex",
        };
        let res = sqlx::query(
            "INSERT INTO merchant_rules (user_id, pattern, pattern_kind, category_code, is_business, priority)
                  VALUES ($1, $2, $3::merchant_pattern_kind_t, $4, $5, $6)",
        )
        .bind(user.id)
        .bind(&r.pattern)
        .bind(kind_str)
        .bind(&r.category_code)
        .bind(r.is_business)
        .bind(200 + idx as i32)
        .execute(&s.pool)
        .await?;
        inserted += res.rows_affected() as usize;
    }
    Ok(Json(SeedResult { inserted, skipped_existing: 0 }))
}

#[derive(Serialize)]
struct SeedResult {
    inserted: usize,
    skipped_existing: usize,
}

// --- Schedule C report ---------------------------------------------------

#[derive(Deserialize)]
struct ReportQuery {
    /// Calendar year for the report (e.g. 2026). Defaults to current year.
    year: Option<i32>,
}

#[derive(Serialize)]
struct ScheduleCLine {
    code: String,
    schedule_c_line: String,
    label: String,
    deduction_pct: Decimal,
    /// Sum of |amount| over expense rows (we negate to positive at the SQL
    /// layer, so this is always a positive dollar figure).
    raw_total: Decimal,
    /// `raw_total * deduction_pct` — meals halve, everything else passes through.
    deductible_total: Decimal,
    txn_count: i64,
}

#[derive(Serialize)]
struct ScheduleCReport {
    year: i32,
    from_date: NaiveDate,
    to_date: NaiveDate,
    lines: Vec<ScheduleCLine>,
    grand_total_raw: Decimal,
    grand_total_deductible: Decimal,
    uncategorized_total: Decimal,
    uncategorized_count: i64,
    excluded_transfers: i64,
    excluded_personal: i64,
}

#[derive(sqlx::FromRow)]
struct LineAggRow {
    code: Option<String>,
    schedule_c_line: Option<String>,
    label: Option<String>,
    deduction_pct: Option<Decimal>,
    raw_total: Decimal,
    txn_count: i64,
}

#[derive(sqlx::FromRow)]
struct CountRow {
    transfer_count: i64,
    personal_count: i64,
}

async fn schedule_c_report(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<ReportQuery>,
) -> Result<Json<ScheduleCReport>, ApiError> {
    let year = q.year.unwrap_or_else(|| chrono::Utc::now().date_naive().format("%Y").to_string().parse().unwrap_or(2026));
    let from = NaiveDate::from_ymd_opt(year, 1, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid year".into()))?;
    let to = NaiveDate::from_ymd_opt(year + 1, 1, 1)
        .ok_or_else(|| ApiError::BadRequest("invalid year".into()))?;

    // Per-category aggregation. We only count expense rows (amount < 0) so
    // refunds/income don't show up in the deduction column — they're a separate
    // Schedule C concern (Part I income lines, not Part II expense lines).
    let rows: Vec<LineAggRow> = sqlx::query_as(
        "SELECT ec.code, ec.schedule_c_line, ec.label, ec.deduction_pct,
                COALESCE(SUM(-t.amount), 0)::numeric(20,2) AS raw_total,
                COUNT(t.id)::bigint AS txn_count
           FROM expense_categories ec
           LEFT JOIN transactions t ON t.category_code = ec.code
                                    AND t.is_business = TRUE
                                    AND t.is_transfer = FALSE
                                    AND t.amount < 0
                                    AND t.posted_at >= $2::date
                                    AND t.posted_at <  $3::date
           LEFT JOIN financial_accounts a ON a.id = t.account_id AND a.user_id = $1
          WHERE t.id IS NULL OR a.user_id = $1
          GROUP BY ec.code, ec.schedule_c_line, ec.label, ec.deduction_pct, ec.sort_order
          ORDER BY ec.sort_order ASC",
    )
    .bind(user.id)
    .bind(from)
    .bind(to)
    .fetch_all(&s.pool)
    .await?;

    let mut lines = Vec::with_capacity(rows.len());
    let mut grand_raw = Decimal::ZERO;
    let mut grand_ded = Decimal::ZERO;
    for r in rows {
        let code = r.code.unwrap_or_default();
        let line = r.schedule_c_line.unwrap_or_default();
        let label = r.label.unwrap_or_default();
        let pct = r.deduction_pct.unwrap_or(Decimal::ONE);
        let deductible = r.raw_total * pct;
        grand_raw += r.raw_total;
        grand_ded += deductible;
        lines.push(ScheduleCLine {
            code,
            schedule_c_line: line,
            label,
            deduction_pct: pct,
            raw_total: r.raw_total,
            deductible_total: deductible,
            txn_count: r.txn_count,
        });
    }

    // Diagnostics: uncategorized business expenses + excluded rows so the
    // user can spot work-to-be-done.
    let uncat: (Decimal, i64) = sqlx::query_as(
        "SELECT COALESCE(SUM(-t.amount), 0)::numeric(20,2), COUNT(t.id)::bigint
           FROM transactions t
           JOIN financial_accounts a ON a.id = t.account_id
          WHERE a.user_id = $1 AND t.category_code IS NULL
            AND t.is_business = TRUE AND t.is_transfer = FALSE AND t.amount < 0
            AND t.posted_at >= $2::date AND t.posted_at < $3::date",
    )
    .bind(user.id)
    .bind(from)
    .bind(to)
    .fetch_one(&s.pool)
    .await?;

    let counts: CountRow = sqlx::query_as(
        "SELECT
            COUNT(*) FILTER (WHERE t.is_transfer = TRUE)::bigint AS transfer_count,
            COUNT(*) FILTER (WHERE t.is_business = FALSE)::bigint AS personal_count
           FROM transactions t
           JOIN financial_accounts a ON a.id = t.account_id
          WHERE a.user_id = $1
            AND t.posted_at >= $2::date AND t.posted_at < $3::date",
    )
    .bind(user.id)
    .bind(from)
    .bind(to)
    .fetch_one(&s.pool)
    .await?;

    Ok(Json(ScheduleCReport {
        year,
        from_date: from,
        to_date: to,
        lines,
        grand_total_raw: grand_raw,
        grand_total_deductible: grand_ded,
        uncategorized_total: uncat.0,
        uncategorized_count: uncat.1,
        excluded_transfers: counts.transfer_count,
        excluded_personal: counts.personal_count,
    }))
}

// --- helpers -------------------------------------------------------------

async fn ensure_account_owner(s: &AppState, user_id: Uuid, account_id: Uuid) -> Result<(), ApiError> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM financial_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(&s.pool)
            .await?;
    match row {
        Some((owner,)) if owner == user_id => Ok(()),
        Some(_) => Err(ApiError::Forbidden),
        None => Err(ApiError::NotFound),
    }
}

async fn ensure_transaction_owner(s: &AppState, user_id: Uuid, tx_id: Uuid) -> Result<(), ApiError> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT a.user_id FROM transactions t
           JOIN financial_accounts a ON a.id = t.account_id
          WHERE t.id = $1",
    )
    .bind(tx_id)
    .fetch_optional(&s.pool)
    .await?;
    match row {
        Some((owner,)) if owner == user_id => Ok(()),
        Some(_) => Err(ApiError::Forbidden),
        None => Err(ApiError::NotFound),
    }
}

// Suppress unused-import warnings; HashMap is held for future bulk-update paths.
#[allow(dead_code)]
fn _hashmap_typecheck() -> HashMap<String, String> {
    HashMap::new()
}

// ============================================================================
// Tax-workshop calculator endpoints
// ============================================================================
//
// Each is a thin shell: deserialize a JSON body matching the calculator's
// input shape, hand to traderview-expense, return the report. No DB writes,
// no auth side-effects — the AuthUser extractor is held just to keep these
// behind the bearer wall like the rest of the expense surface.

#[derive(Deserialize)]
struct ScheduleSeRequest {
    net_profit_schedule_c: Decimal,
    w2_ss_wages: Decimal,
    filing_status: traderview_expense::self_employment_tax::FilingStatus,
    year: u16,
}

async fn calc_self_employment_tax(
    _user: AuthUser,
    Json(req): Json<ScheduleSeRequest>,
) -> Result<Json<traderview_expense::self_employment_tax::ScheduleSeReport>, ApiError> {
    use traderview_expense::self_employment_tax::{compute, ScheduleSeInput};
    let input = ScheduleSeInput {
        net_profit_schedule_c: req.net_profit_schedule_c,
        w2_ss_wages: req.w2_ss_wages,
        filing_status: req.filing_status,
        year: req.year,
    };
    Ok(Json(compute(&input)))
}

#[derive(Deserialize)]
struct HomeOfficeRequest {
    business_use_sqft: Decimal,
    total_home_sqft: Decimal,
    annual_mortgage_interest: Decimal,
    annual_property_tax: Decimal,
    annual_utilities: Decimal,
    annual_insurance: Decimal,
    annual_repairs: Decimal,
    annual_depreciation: Decimal,
}

async fn calc_home_office(
    _user: AuthUser,
    Json(req): Json<HomeOfficeRequest>,
) -> Result<Json<traderview_expense::home_office::HomeOfficeReport>, ApiError> {
    use traderview_expense::home_office::{compute, HomeOfficeInput};
    let input = HomeOfficeInput {
        business_use_sqft: req.business_use_sqft,
        total_home_sqft: req.total_home_sqft,
        annual_mortgage_interest: req.annual_mortgage_interest,
        annual_property_tax: req.annual_property_tax,
        annual_utilities: req.annual_utilities,
        annual_insurance: req.annual_insurance,
        annual_repairs: req.annual_repairs,
        annual_depreciation: req.annual_depreciation,
    };
    Ok(Json(compute(&input)))
}

#[derive(Deserialize)]
struct MileageRequest {
    trips: Vec<traderview_expense::mileage::Trip>,
}

async fn calc_mileage(
    _user: AuthUser,
    Json(req): Json<MileageRequest>,
) -> Result<Json<traderview_expense::mileage::MileageReport>, ApiError> {
    Ok(Json(traderview_expense::mileage::report(&req.trips)))
}

#[derive(Deserialize)]
struct QuarterlyRequest {
    tax_year: i32,
    prior_year_total_tax: Decimal,
    prior_year_agi: Decimal,
    ytd_net_profit: Decimal,
    days_through_ytd: i32,
    estimated_effective_tax_rate: Decimal,
    withholding_ytd: Decimal,
}

async fn calc_quarterly_tax(
    _user: AuthUser,
    Json(req): Json<QuarterlyRequest>,
) -> Result<Json<traderview_expense::quarterly_tax::QuarterlyForecast>, ApiError> {
    use traderview_expense::quarterly_tax::{forecast, ForecastInput};
    let input = ForecastInput {
        tax_year: req.tax_year,
        prior_year_total_tax: req.prior_year_total_tax,
        prior_year_agi: req.prior_year_agi,
        ytd_net_profit: req.ytd_net_profit,
        days_through_ytd: req.days_through_ytd,
        estimated_effective_tax_rate: req.estimated_effective_tax_rate,
        withholding_ytd: req.withholding_ytd,
    };
    Ok(Json(forecast(&input)))
}

/// Scan the authenticated user's transactions for recurring subscriptions.
/// Pulls everything from `transactions` then runs detection in-process.
async fn detect_subscriptions(
    user: AuthUser,
    State(s): State<AppState>,
) -> Result<Json<Vec<traderview_expense::subscription_detector::Subscription>>, ApiError> {
    use traderview_expense::subscription_detector::{detect, DetectOptions};
    use traderview_expense::ParsedTransaction;

    // Load the user's whole transaction history (joined to their accounts
    // for owner-scope). Reasonable bound — even heavy users top out in the
    // low tens of thousands of transactions; the detector is linear.
    let rows: Vec<(DateTime<Utc>, Decimal, String, String, String)> = sqlx::query_as(
        "SELECT t.posted_at, t.amount, t.currency, t.merchant_raw, t.merchant_normalized
           FROM transactions t
           JOIN financial_accounts a ON a.id = t.account_id
          WHERE a.user_id = $1
          ORDER BY t.posted_at",
    )
    .bind(user.id)
    .fetch_all(&s.pool)
    .await?;

    let txns: Vec<ParsedTransaction> = rows.into_iter()
        .map(|(posted_at, amount, currency, merchant_raw, merchant_normalized)| {
            ParsedTransaction {
                posted_at,
                amount,
                currency,
                merchant_raw,
                merchant_normalized,
                description: String::new(),
                raw: serde_json::Value::Null,
            }
        })
        .collect();

    Ok(Json(detect(&txns, DetectOptions::default())))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── escape_ilike_pattern ─────────────────────────────────────────────

    #[test]
    fn escape_ilike_passes_through_plain_text() {
        // Normal merchant names must survive verbatim.
        assert_eq!(escape_ilike_pattern("Starbucks"), "Starbucks");
        assert_eq!(escape_ilike_pattern("café latté"), "café latté");
    }

    #[test]
    fn escape_ilike_escapes_percent_so_100_pct_is_literal() {
        // Searching "100%" must NOT match every transaction — `%` is the
        // ILIKE wildcard for "any sequence of characters".
        assert_eq!(escape_ilike_pattern("100%"), r"100\%");
    }

    #[test]
    fn escape_ilike_escapes_underscore_single_char_wildcard() {
        // `_` matches any single char under ILIKE — must be escaped.
        assert_eq!(escape_ilike_pattern("a_b"), r"a\_b");
    }

    #[test]
    fn escape_ilike_escapes_backslash() {
        // The ESCAPE '\' clause uses backslash as the escape char; if user
        // input contains a literal backslash it must itself be escaped.
        assert_eq!(escape_ilike_pattern(r"path\to"), r"path\\to");
    }

    #[test]
    fn escape_ilike_handles_all_three_simultaneously() {
        assert_eq!(escape_ilike_pattern(r"10%_foo\bar"), r"10\%\_foo\\bar");
    }

    #[test]
    fn escape_ilike_empty_string_is_empty() {
        assert_eq!(escape_ilike_pattern(""), "");
    }

    // ─── clamp_pagination ─────────────────────────────────────────────────

    #[test]
    fn clamp_pagination_passes_through_normal_values() {
        assert_eq!(clamp_pagination(200, 0),  (200, 0));
        assert_eq!(clamp_pagination(50,  100), (50, 100));
        assert_eq!(clamp_pagination(MAX_TX_LIMIT, 0), (MAX_TX_LIMIT, 0));
    }

    #[test]
    fn clamp_pagination_caps_oversize_limit() {
        // A client passing ?limit=1000000 must NOT force the server to
        // materialize a multi-million row Vec.
        assert_eq!(clamp_pagination(1_000_000, 0), (MAX_TX_LIMIT, 0));
        assert_eq!(clamp_pagination(i64::MAX, 0),   (MAX_TX_LIMIT, 0));
    }

    #[test]
    fn clamp_pagination_floors_limit_at_1() {
        // Zero / negative would yield empty/error SQL — always serve at
        // least one row.
        assert_eq!(clamp_pagination(0,  0),  (1, 0));
        assert_eq!(clamp_pagination(-1, 0),  (1, 0));
        assert_eq!(clamp_pagination(i64::MIN, 0), (1, 0));
    }

    #[test]
    fn clamp_pagination_floors_offset_at_0() {
        // Negative offset is a SQL syntax error — clamp.
        assert_eq!(clamp_pagination(100, -5), (100, 0));
        assert_eq!(clamp_pagination(100, i64::MIN), (100, 0));
    }

    #[test]
    fn clamp_pagination_allows_large_offset() {
        // Deep pagination is the user's prerogative; only the per-page
        // size is capped.
        assert_eq!(clamp_pagination(100, 1_000_000), (100, 1_000_000));
    }
}
