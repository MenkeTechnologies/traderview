//! Per-user market-data provider credentials persisted in `user_settings`.
//!
//! Mirrors the get/upsert pattern in [`crate::journal_ai`] for the LLM-key
//! columns: secrets returned to the frontend are masked as `"***"`; an
//! incoming `"***"` value on save means "keep the existing column value".
//!
//! Reads from these helpers should follow the 3-tier credential resolution
//! used by the data router:
//!   1. Process-memory override (live-ticks `set_api_key` POST)
//!   2. Env var (e.g. `FINNHUB_API_KEY`, `ALPACA_KEY_ID`)
//!   3. This module (DB)

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

const MASK: &str = "***";

/// Public DTO sent to / received from the settings UI. Secret fields are
/// `Some("***")` when the column is populated, `None` when empty.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataSourceKeysDto {
    /// Finnhub REST + WebSocket API key (free tier 60 calls/min).
    #[serde(default)]
    pub finnhub_api_key: Option<String>,
    /// Alpaca trading + market-data key id (paper or live, see `alpaca_paper`).
    #[serde(default)]
    pub alpaca_key_id: Option<String>,
    /// Alpaca trading + market-data secret key (matches `alpaca_key_id`).
    #[serde(default)]
    pub alpaca_secret_key: Option<String>,
    /// When true, talk to the paper-trading API base URL instead of live.
    #[serde(default = "default_true")]
    pub alpaca_paper: bool,
}

fn default_true() -> bool {
    true
}

#[derive(sqlx::FromRow)]
struct Row {
    finnhub_api_key: Option<String>,
    alpaca_key_id: Option<String>,
    alpaca_secret_key: Option<String>,
    alpaca_paper: bool,
}

fn mask(v: Option<String>) -> Option<String> {
    v.map(|_| MASK.into())
}

/// Load the per-user data-source credentials with secret fields masked.
pub async fn get(pool: &PgPool, user_id: Uuid) -> anyhow::Result<DataSourceKeysDto> {
    // Make sure a user_settings row exists; settings::get does this lazily
    // too, but this module is callable on its own.
    sqlx::query("INSERT INTO user_settings (user_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .execute(pool)
        .await?;
    let row: Row = sqlx::query_as(
        "SELECT finnhub_api_key, alpaca_key_id, alpaca_secret_key, alpaca_paper
           FROM user_settings
          WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(DataSourceKeysDto {
        finnhub_api_key: mask(row.finnhub_api_key),
        // key_id is not a secret per se but treat consistently for UX.
        alpaca_key_id: mask(row.alpaca_key_id),
        alpaca_secret_key: mask(row.alpaca_secret_key),
        alpaca_paper: row.alpaca_paper,
    })
}

/// Upsert. Any secret field set to `Some("***")` or `Some("")` is treated
/// as "keep existing"; `None` means "leave column untouched".
pub async fn set(pool: &PgPool, user_id: Uuid, dto: &DataSourceKeysDto) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO user_settings (user_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .execute(pool)
        .await?;

    let finnhub_supplied = matches!(dto.finnhub_api_key.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let alpaca_id_supplied = matches!(dto.alpaca_key_id.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let alpaca_secret_supplied = matches!(dto.alpaca_secret_key.as_deref(), Some(k) if k != MASK && !k.is_empty());

    // Build a coalescing UPDATE so the caller can change a subset of fields
    // without re-supplying the others.
    sqlx::query(
        "UPDATE user_settings SET
             finnhub_api_key   = COALESCE($2, finnhub_api_key),
             alpaca_key_id     = COALESCE($3, alpaca_key_id),
             alpaca_secret_key = COALESCE($4, alpaca_secret_key),
             alpaca_paper      = $5,
             updated_at        = now()
           WHERE user_id = $1",
    )
    .bind(user_id)
    .bind(if finnhub_supplied { dto.finnhub_api_key.as_deref() } else { None })
    .bind(if alpaca_id_supplied { dto.alpaca_key_id.as_deref() } else { None })
    .bind(if alpaca_secret_supplied { dto.alpaca_secret_key.as_deref() } else { None })
    .bind(dto.alpaca_paper)
    .execute(pool)
    .await?;
    Ok(())
}

/// Pick any user's saved Finnhub key (newest write wins). Used at server
/// boot to warm the `live_ticks::global()` in-memory slot so REST callers
/// (e.g. `finnhub_rest`) work without the user re-saving in Settings
/// every restart. Single-user-app friendly; multi-tenant deployments
/// shouldn't use this path (per-request DB lookup keyed by user_id is
/// the right pattern there).
pub async fn any_finnhub_key(pool: &PgPool) -> anyhow::Result<Option<String>> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT finnhub_api_key FROM user_settings
           WHERE finnhub_api_key IS NOT NULL AND finnhub_api_key <> ''
           ORDER BY updated_at DESC
           LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    if let Some((Some(k),)) = row {
        if !k.is_empty() {
            return Ok(Some(k));
        }
    }
    Ok(std::env::var("FINNHUB_API_KEY").ok().filter(|s| !s.is_empty()))
}

/// Plaintext finnhub key for backend callers (the data router, the live-ticks
/// loop). Returns env-var fallback when the DB column is empty, so headless
/// / CI deployments work without going through the UI.
pub async fn finnhub_key_plain(
    pool: &PgPool,
    user_id: Uuid,
) -> anyhow::Result<Option<String>> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT finnhub_api_key FROM user_settings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    if let Some((Some(k),)) = row {
        if !k.is_empty() {
            return Ok(Some(k));
        }
    }
    Ok(std::env::var("FINNHUB_API_KEY").ok().filter(|s| !s.is_empty()))
}

/// Plaintext alpaca credentials for backend callers. Returns the
/// (key_id, secret, paper) tuple; env-var fallback for headless mode.
pub async fn alpaca_creds_plain(
    pool: &PgPool,
    user_id: Uuid,
) -> anyhow::Result<Option<(String, String, bool)>> {
    let row: Option<(Option<String>, Option<String>, bool)> = sqlx::query_as(
        "SELECT alpaca_key_id, alpaca_secret_key, alpaca_paper
           FROM user_settings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    if let Some((Some(id), Some(sec), paper)) = row {
        if !id.is_empty() && !sec.is_empty() {
            return Ok(Some((id, sec, paper)));
        }
    }
    let id = std::env::var("ALPACA_KEY_ID").ok().filter(|s| !s.is_empty());
    let sec = std::env::var("ALPACA_SECRET_KEY").ok().filter(|s| !s.is_empty());
    let paper = std::env::var("ALPACA_PAPER")
        .ok()
        .map(|s| !matches!(s.as_str(), "0" | "false" | "no"))
        .unwrap_or(true);
    if let (Some(id), Some(sec)) = (id, sec) {
        return Ok(Some((id, sec, paper)));
    }
    Ok(None)
}
