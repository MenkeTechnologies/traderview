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

/// Plaintext Tradier credentials for the algo dispatcher. Returns
/// `(access_token, account_id, sandbox)` — sandbox=true picks the
/// sandbox.tradier.com base URL; false picks api.tradier.com.
pub async fn tradier_creds(
    pool: &PgPool,
    user_id: Uuid,
) -> anyhow::Result<Option<(String, String, bool)>> {
    let row: Option<(Option<String>, Option<String>, bool)> = sqlx::query_as(
        "SELECT tradier_access_token, tradier_account_id, tradier_sandbox
           FROM user_settings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    if let Some((Some(tok), Some(acct), sandbox)) = row {
        if !tok.is_empty() && !acct.is_empty() {
            return Ok(Some((tok, acct, sandbox)));
        }
    }
    Ok(None)
}

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
    /// Polygon.io API key — Advanced tier provides full consolidated
    /// SIP tape (CTA + UTP). Lower tiers fall back to delayed / IEX-only.
    #[serde(default)]
    pub polygon_api_key: Option<String>,
    /// Databento API key — direct CTA / UTP / OPRA SIP feeds. Paid
    /// per-gigabyte; preferred for ultra-low-latency tape replay.
    #[serde(default)]
    pub databento_api_key: Option<String>,
    /// Per-user opt-in for Alpaca's SIP feed (Live tier with SIP costs
    /// more than the default IEX-only feed; some plans support both).
    #[serde(default)]
    pub alpaca_use_sip_feed: bool,
    /// Tradier brokerage access token. Sandbox + live both use the same
    /// header format (`Authorization: Bearer <token>`); the env switch
    /// is `tradier_sandbox` below.
    #[serde(default)]
    pub tradier_access_token: Option<String>,
    /// Tradier account number — the path segment in
    /// `/v1/accounts/{account_id}/orders`. Get it from the Tradier
    /// dashboard.
    #[serde(default)]
    pub tradier_account_id: Option<String>,
    /// When true (default) routes to sandbox.tradier.com; false routes
    /// to api.tradier.com. Independent of the per-strategy broker_mode —
    /// a user can keep `tradier_sandbox=true` while running an algo
    /// strategy in `broker_mode='live'` (the sandbox env wins; strategy
    /// engine logs the discrepancy).
    #[serde(default = "default_true")]
    pub tradier_sandbox: bool,
    /// Tastytrade username (or email). Either (login + password) OR
    /// session_token is needed; the dispatcher tries token first.
    #[serde(default)]
    pub tastytrade_login: Option<String>,
    /// Tastytrade password — paired with login. Stored plaintext like
    /// every other secret in this table; UI offers a 'reveal' button.
    #[serde(default)]
    pub tastytrade_password: Option<String>,
    /// Long-lived session token from POST /sessions with
    /// `remember-me: true`. Lets the dispatcher skip the login step.
    #[serde(default)]
    pub tastytrade_session_token: Option<String>,
    /// Account number from the Tastytrade dashboard. Used as path
    /// segment in `/accounts/{n}/orders`.
    #[serde(default)]
    pub tastytrade_account_number: Option<String>,
    /// Sandbox vs prod. Sandbox = api.cert.tastyworks.com.
    #[serde(default = "default_true")]
    pub tastytrade_sandbox: bool,
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
    polygon_api_key: Option<String>,
    databento_api_key: Option<String>,
    alpaca_use_sip_feed: bool,
    tradier_access_token: Option<String>,
    tradier_account_id: Option<String>,
    tradier_sandbox: bool,
    tastytrade_login: Option<String>,
    tastytrade_password: Option<String>,
    tastytrade_session_token: Option<String>,
    tastytrade_account_number: Option<String>,
    tastytrade_sandbox: bool,
}

fn mask(v: Option<String>) -> Option<String> {
    v.map(|_| MASK.into())
}

/// Load the per-user data-source credentials WITHOUT masking. Used by
/// the Settings → Data Sources "reveal" button so the user can read
/// their own keys back. Single-user desktop app — no privilege check
/// beyond the standard `AuthUser` extractor on the route.
pub async fn get_unmasked(pool: &PgPool, user_id: Uuid) -> anyhow::Result<DataSourceKeysDto> {
    sqlx::query("INSERT INTO user_settings (user_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .execute(pool)
        .await?;
    let row: Row = sqlx::query_as(
        "SELECT finnhub_api_key, alpaca_key_id, alpaca_secret_key, alpaca_paper,
                polygon_api_key, databento_api_key, alpaca_use_sip_feed,
                tradier_access_token, tradier_account_id, tradier_sandbox,
                tastytrade_login, tastytrade_password, tastytrade_session_token,
                tastytrade_account_number, tastytrade_sandbox
           FROM user_settings
          WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(DataSourceKeysDto {
        finnhub_api_key: row.finnhub_api_key,
        alpaca_key_id: row.alpaca_key_id,
        alpaca_secret_key: row.alpaca_secret_key,
        alpaca_paper: row.alpaca_paper,
        polygon_api_key: row.polygon_api_key,
        databento_api_key: row.databento_api_key,
        alpaca_use_sip_feed: row.alpaca_use_sip_feed,
        tradier_access_token: row.tradier_access_token,
        tradier_account_id: row.tradier_account_id,
        tradier_sandbox: row.tradier_sandbox,
        tastytrade_login: row.tastytrade_login,
        tastytrade_password: row.tastytrade_password,
        tastytrade_session_token: row.tastytrade_session_token,
        tastytrade_account_number: row.tastytrade_account_number,
        tastytrade_sandbox: row.tastytrade_sandbox,
    })
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
        "SELECT finnhub_api_key, alpaca_key_id, alpaca_secret_key, alpaca_paper,
                polygon_api_key, databento_api_key, alpaca_use_sip_feed,
                tradier_access_token, tradier_account_id, tradier_sandbox,
                tastytrade_login, tastytrade_password, tastytrade_session_token,
                tastytrade_account_number, tastytrade_sandbox
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
        polygon_api_key: mask(row.polygon_api_key),
        databento_api_key: mask(row.databento_api_key),
        alpaca_use_sip_feed: row.alpaca_use_sip_feed,
        tradier_access_token: mask(row.tradier_access_token),
        // Account ID isn't a secret (it appears in error messages, URLs)
        // but mask consistently so the reveal-button UX stays uniform.
        tradier_account_id: mask(row.tradier_account_id),
        tradier_sandbox: row.tradier_sandbox,
        tastytrade_login: mask(row.tastytrade_login),
        tastytrade_password: mask(row.tastytrade_password),
        tastytrade_session_token: mask(row.tastytrade_session_token),
        tastytrade_account_number: mask(row.tastytrade_account_number),
        tastytrade_sandbox: row.tastytrade_sandbox,
    })
}

/// Upsert. Any secret field set to `Some("***")` or `Some("")` is treated
/// as "keep existing"; `None` means "leave column untouched".
pub async fn set(pool: &PgPool, user_id: Uuid, dto: &DataSourceKeysDto) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO user_settings (user_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .execute(pool)
        .await?;

    let finnhub_supplied =
        matches!(dto.finnhub_api_key.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let alpaca_id_supplied =
        matches!(dto.alpaca_key_id.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let alpaca_secret_supplied =
        matches!(dto.alpaca_secret_key.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let polygon_supplied =
        matches!(dto.polygon_api_key.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let databento_supplied =
        matches!(dto.databento_api_key.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let tradier_token_supplied =
        matches!(dto.tradier_access_token.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let tradier_acct_supplied =
        matches!(dto.tradier_account_id.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let tt_login_supplied =
        matches!(dto.tastytrade_login.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let tt_password_supplied =
        matches!(dto.tastytrade_password.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let tt_token_supplied =
        matches!(dto.tastytrade_session_token.as_deref(), Some(k) if k != MASK && !k.is_empty());
    let tt_acct_supplied =
        matches!(dto.tastytrade_account_number.as_deref(), Some(k) if k != MASK && !k.is_empty());

    // Build a coalescing UPDATE so the caller can change a subset of fields
    // without re-supplying the others.
    sqlx::query(
        "UPDATE user_settings SET
             finnhub_api_key           = COALESCE($2, finnhub_api_key),
             alpaca_key_id             = COALESCE($3, alpaca_key_id),
             alpaca_secret_key         = COALESCE($4, alpaca_secret_key),
             alpaca_paper              = $5,
             polygon_api_key           = COALESCE($6, polygon_api_key),
             databento_api_key         = COALESCE($7, databento_api_key),
             alpaca_use_sip_feed       = $8,
             tradier_access_token      = COALESCE($9, tradier_access_token),
             tradier_account_id        = COALESCE($10, tradier_account_id),
             tradier_sandbox           = $11,
             tastytrade_login          = COALESCE($12, tastytrade_login),
             tastytrade_password       = COALESCE($13, tastytrade_password),
             tastytrade_session_token  = COALESCE($14, tastytrade_session_token),
             tastytrade_account_number = COALESCE($15, tastytrade_account_number),
             tastytrade_sandbox        = $16,
             updated_at                = now()
           WHERE user_id = $1",
    )
    .bind(user_id)
    .bind(if finnhub_supplied { dto.finnhub_api_key.as_deref() } else { None })
    .bind(if alpaca_id_supplied { dto.alpaca_key_id.as_deref() } else { None })
    .bind(if alpaca_secret_supplied { dto.alpaca_secret_key.as_deref() } else { None })
    .bind(dto.alpaca_paper)
    .bind(if polygon_supplied { dto.polygon_api_key.as_deref() } else { None })
    .bind(if databento_supplied { dto.databento_api_key.as_deref() } else { None })
    .bind(dto.alpaca_use_sip_feed)
    .bind(if tradier_token_supplied { dto.tradier_access_token.as_deref() } else { None })
    .bind(if tradier_acct_supplied { dto.tradier_account_id.as_deref() } else { None })
    .bind(dto.tradier_sandbox)
    .bind(if tt_login_supplied { dto.tastytrade_login.as_deref() } else { None })
    .bind(if tt_password_supplied { dto.tastytrade_password.as_deref() } else { None })
    .bind(if tt_token_supplied { dto.tastytrade_session_token.as_deref() } else { None })
    .bind(if tt_acct_supplied { dto.tastytrade_account_number.as_deref() } else { None })
    .bind(dto.tastytrade_sandbox)
    .execute(pool)
    .await?;
    Ok(())
}

/// Plaintext Tastytrade credentials for backend callers. Returns
/// `(account_number, sandbox, Auth)` where Auth is either a long-lived
/// session_token or a (login, password) pair to mint one on demand.
pub async fn tastytrade_creds(
    pool: &PgPool,
    user_id: Uuid,
) -> anyhow::Result<Option<(String, bool, crate::tastytrade_trading::Auth)>> {
    let row: Option<(Option<String>, Option<String>, Option<String>, Option<String>, bool)> =
        sqlx::query_as(
            "SELECT tastytrade_login, tastytrade_password, tastytrade_session_token,
                    tastytrade_account_number, tastytrade_sandbox
               FROM user_settings WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    let Some((login, password, token, account_number, sandbox)) = row else {
        return Ok(None);
    };
    let account_number = match account_number {
        Some(a) if !a.is_empty() => a,
        _ => return Ok(None),
    };
    if let Some(t) = token.filter(|t| !t.is_empty()) {
        return Ok(Some((
            account_number,
            sandbox,
            crate::tastytrade_trading::Auth::SessionToken(t),
        )));
    }
    if let (Some(l), Some(p)) = (login, password) {
        if !l.is_empty() && !p.is_empty() {
            return Ok(Some((
                account_number,
                sandbox,
                crate::tastytrade_trading::Auth::UserPass {
                    login: l,
                    password: p,
                    remember_me: true,
                },
            )));
        }
    }
    Ok(None)
}

/// Plaintext Polygon key for backend callers — env-var fallback for
/// headless. Symmetric with [`finnhub_key_plain`].
pub async fn polygon_key_plain(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Option<String>> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT polygon_api_key FROM user_settings WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    if let Some((Some(k),)) = row {
        if !k.is_empty() {
            return Ok(Some(k));
        }
    }
    Ok(std::env::var("POLYGON_API_KEY")
        .ok()
        .filter(|s| !s.is_empty()))
}

/// Plaintext Databento key for backend callers.
pub async fn databento_key_plain(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Option<String>> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT databento_api_key FROM user_settings WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    if let Some((Some(k),)) = row {
        if !k.is_empty() {
            return Ok(Some(k));
        }
    }
    Ok(std::env::var("DATABENTO_API_KEY")
        .ok()
        .filter(|s| !s.is_empty()))
}

/// Boot-time scoop for any user's saved Alpaca creds + SIP toggle.
/// `(key_id, secret, use_sip_feed)`. Newest write wins. Env-var
/// fallback (`ALPACA_KEY_ID`, `ALPACA_SECRET_KEY`, `ALPACA_USE_SIP`)
/// for headless / CI deployments.
pub async fn any_alpaca_creds(pool: &PgPool) -> anyhow::Result<Option<(String, String, bool)>> {
    let row: Option<(Option<String>, Option<String>, bool)> = sqlx::query_as(
        "SELECT alpaca_key_id, alpaca_secret_key, alpaca_use_sip_feed
           FROM user_settings
          WHERE alpaca_key_id IS NOT NULL AND alpaca_key_id <> ''
            AND alpaca_secret_key IS NOT NULL AND alpaca_secret_key <> ''
          ORDER BY updated_at DESC
          LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    if let Some((Some(id), Some(sec), use_sip)) = row {
        if !id.is_empty() && !sec.is_empty() {
            return Ok(Some((id, sec, use_sip)));
        }
    }
    let id = std::env::var("ALPACA_KEY_ID")
        .ok()
        .filter(|s| !s.is_empty());
    let sec = std::env::var("ALPACA_SECRET_KEY")
        .ok()
        .filter(|s| !s.is_empty());
    let use_sip = std::env::var("ALPACA_USE_SIP")
        .ok()
        .map(|s| matches!(s.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    if let (Some(id), Some(sec)) = (id, sec) {
        return Ok(Some((id, sec, use_sip)));
    }
    Ok(None)
}

/// Boot-time scoop for the Polygon SIP key. Mirrors [`any_finnhub_key`].
pub async fn any_polygon_key(pool: &PgPool) -> anyhow::Result<Option<String>> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT polygon_api_key FROM user_settings
           WHERE polygon_api_key IS NOT NULL AND polygon_api_key <> ''
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
    Ok(std::env::var("POLYGON_API_KEY")
        .ok()
        .filter(|s| !s.is_empty()))
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
    Ok(std::env::var("FINNHUB_API_KEY")
        .ok()
        .filter(|s| !s.is_empty()))
}

/// Plaintext finnhub key for backend callers (the data router, the live-ticks
/// loop). Returns env-var fallback when the DB column is empty, so headless
/// / CI deployments work without going through the UI.
pub async fn finnhub_key_plain(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Option<String>> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT finnhub_api_key FROM user_settings WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    if let Some((Some(k),)) = row {
        if !k.is_empty() {
            return Ok(Some(k));
        }
    }
    Ok(std::env::var("FINNHUB_API_KEY")
        .ok()
        .filter(|s| !s.is_empty()))
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
    let id = std::env::var("ALPACA_KEY_ID")
        .ok()
        .filter(|s| !s.is_empty());
    let sec = std::env::var("ALPACA_SECRET_KEY")
        .ok()
        .filter(|s| !s.is_empty());
    let paper = std::env::var("ALPACA_PAPER")
        .ok()
        .map(|s| !matches!(s.as_str(), "0" | "false" | "no"))
        .unwrap_or(true);
    if let (Some(id), Some(sec)) = (id, sec) {
        return Ok(Some((id, sec, paper)));
    }
    Ok(None)
}
