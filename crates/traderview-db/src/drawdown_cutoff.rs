//! Drawdown auto-cutoff.
//!
//! Reads current equity across configured brokers (alpaca + tradier
//! today; ibkr/schwab/tastytrade follow when wiring lands). Maintains a
//! per-user `high_water_mark`. When
//!
//! ```text
//! drawdown_pct = (high_water_mark - current_equity) / high_water_mark * 100
//! ```
//!
//! crosses `max_drawdown_pct`, fires the existing
//! `multi_broker::kill_all_for_user` once and pins `auto_killed_at`.
//! Subsequent evaluations are no-ops until the user explicitly resets
//! (which clears `auto_killed_at` and re-seeds `high_water_mark` from
//! the current equity).
//!
//! Pure compute lives in `should_fire_cutoff` + `new_high_water_mark` and
//! is fully unit-tested. The repository layer reads broker balances,
//! updates the cached HWM, decides, and logs.

use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{alpaca_trading, tradier_trading};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownConfig {
    pub user_id: Uuid,
    pub enabled: bool,
    pub max_drawdown_pct: f64,
    pub high_water_mark: Option<f64>,
    pub last_equity: Option<f64>,
    pub last_evaluated_at: Option<DateTime<Utc>>,
    pub auto_killed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl DrawdownConfig {
    pub fn default_for(user_id: Uuid) -> Self {
        Self {
            user_id,
            enabled: false,
            max_drawdown_pct: 5.0,
            high_water_mark: None,
            last_equity: None,
            last_evaluated_at: None,
            auto_killed_at: None,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownLogRow {
    pub id: i64,
    pub evaluated_at: DateTime<Utc>,
    pub current_equity: f64,
    pub high_water_mark: f64,
    pub drawdown_pct: f64,
    pub threshold_pct: f64,
    pub action: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvaluationResult {
    pub config: DrawdownConfig,
    pub current_equity: f64,
    pub high_water_mark: f64,
    pub drawdown_pct: f64,
    pub action: String,
    pub kill_result: Option<crate::multi_broker::KillSwitchResult>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Returns `true` iff the equity has fallen far enough below the
/// high-water mark to fire the cutoff. Uses `>=` so threshold-equality
/// fires (conservative — user said "≥ 5%" not "> 5%").
pub fn should_fire_cutoff(
    high_water_mark: f64,
    current_equity: f64,
    max_drawdown_pct: f64,
) -> bool {
    if !(high_water_mark > 0.0 && current_equity.is_finite() && max_drawdown_pct > 0.0) {
        return false;
    }
    if current_equity >= high_water_mark {
        return false;
    }
    let dd = (high_water_mark - current_equity) / high_water_mark * 100.0;
    dd >= max_drawdown_pct
}

/// New high-water mark. Takes the max of the prior mark and the current
/// equity (or just current if there's no prior mark).
pub fn new_high_water_mark(prior: Option<f64>, current_equity: f64) -> f64 {
    if !current_equity.is_finite() {
        return prior.unwrap_or(0.0);
    }
    match prior {
        Some(p) if p > current_equity => p,
        _ => current_equity.max(0.0),
    }
}

/// Drawdown % given hwm + current. Returns 0 when equity is at or above
/// the high-water mark (no drawdown) or inputs are unusable.
pub fn drawdown_pct(high_water_mark: f64, current_equity: f64) -> f64 {
    if !(high_water_mark > 0.0 && current_equity.is_finite()) {
        return 0.0;
    }
    if current_equity >= high_water_mark {
        return 0.0;
    }
    (high_water_mark - current_equity) / high_water_mark * 100.0
}

// ─── Repository ────────────────────────────────────────────────────────────

pub async fn get_config(pool: &PgPool, user_id: Uuid) -> anyhow::Result<DrawdownConfig> {
    type Row = (
        bool,
        f64,
        Option<f64>,
        Option<f64>,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
        DateTime<Utc>,
    );
    let row: Option<Row> = sqlx::query_as(
        "SELECT enabled, max_drawdown_pct, high_water_mark, last_equity,
                last_evaluated_at, auto_killed_at, updated_at
           FROM drawdown_cutoff_config WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(match row {
        Some((enabled, max_dd, hwm, last_eq, last_ev, auto_killed, updated)) => DrawdownConfig {
            user_id,
            enabled,
            max_drawdown_pct: max_dd,
            high_water_mark: hwm,
            last_equity: last_eq,
            last_evaluated_at: last_ev,
            auto_killed_at: auto_killed,
            updated_at: updated,
        },
        None => DrawdownConfig::default_for(user_id),
    })
}

pub async fn upsert_config(pool: &PgPool, cfg: &DrawdownConfig) -> anyhow::Result<DrawdownConfig> {
    sqlx::query(
        "INSERT INTO drawdown_cutoff_config
            (user_id, enabled, max_drawdown_pct, updated_at)
         VALUES ($1, $2, $3, now())
         ON CONFLICT (user_id) DO UPDATE SET
            enabled          = EXCLUDED.enabled,
            max_drawdown_pct = EXCLUDED.max_drawdown_pct,
            updated_at       = now()",
    )
    .bind(cfg.user_id)
    .bind(cfg.enabled)
    .bind(cfg.max_drawdown_pct)
    .execute(pool)
    .await?;
    get_config(pool, cfg.user_id).await
}

pub async fn reset(pool: &PgPool, user_id: Uuid) -> anyhow::Result<DrawdownConfig> {
    sqlx::query(
        "UPDATE drawdown_cutoff_config
            SET auto_killed_at = NULL, high_water_mark = NULL, last_equity = NULL,
                updated_at = now()
          WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    get_config(pool, user_id).await
}

pub async fn recent_log(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> anyhow::Result<Vec<DrawdownLogRow>> {
    type LogTup = (i64, DateTime<Utc>, f64, f64, f64, f64, String);
    let rows: Vec<LogTup> = sqlx::query_as(
        "SELECT id, evaluated_at, current_equity, high_water_mark,
                drawdown_pct, threshold_pct, action
           FROM drawdown_cutoff_log
          WHERE user_id = $1
          ORDER BY evaluated_at DESC
          LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, ts, eq, hwm, dd, thr, act)| DrawdownLogRow {
            id,
            evaluated_at: ts,
            current_equity: eq,
            high_water_mark: hwm,
            drawdown_pct: dd,
            threshold_pct: thr,
            action: act,
        })
        .collect())
}

/// Best-effort equity sum across configured brokers. Skips brokers with
/// no creds. Returns 0 when no brokers are configured.
pub async fn live_equity_usd(pool: &PgPool, user_id: Uuid) -> f64 {
    let row: Option<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<bool>,
    )> = sqlx::query_as(
        "SELECT alpaca_api_key, alpaca_api_secret, alpaca_mode,
                tradier_access_token, tradier_account_id, tradier_sandbox
           FROM user_settings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let (alpaca_key, alpaca_secret, alpaca_mode, tradier_token, tradier_acct, tradier_sandbox) =
        row.unwrap_or((None, None, None, None, None, None));

    let mut total = 0.0_f64;

    if let (Some(k), Some(s)) = (alpaca_key, alpaca_secret) {
        if !k.is_empty() && !s.is_empty() {
            let mode = match alpaca_mode.as_deref() {
                Some("live") => alpaca_trading::BrokerMode::Live,
                _ => alpaca_trading::BrokerMode::Paper,
            };
            let client = alpaca_trading::AlpacaTrading::new(mode, k, s);
            if let Ok(acct) = client.get_account().await {
                total += acct.equity.to_f64().unwrap_or(0.0);
            }
        }
    }
    if let (Some(t), Some(a)) = (tradier_token, tradier_acct) {
        if !t.is_empty() && !a.is_empty() {
            let env = if tradier_sandbox.unwrap_or(true) {
                tradier_trading::TradierEnv::Sandbox
            } else {
                tradier_trading::TradierEnv::Live
            };
            let client = tradier_trading::TradierTrading::new(env, t, a);
            if let Ok(b) = client.get_balances().await {
                if let Some(eq) = b.balances.total_equity {
                    total += eq.to_f64().unwrap_or(0.0);
                }
            }
        }
    }

    total
}

async fn write_log(
    pool: &PgPool,
    user_id: Uuid,
    eq: f64,
    hwm: f64,
    dd: f64,
    threshold: f64,
    action: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO drawdown_cutoff_log
            (user_id, current_equity, high_water_mark, drawdown_pct, threshold_pct, action)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(user_id)
    .bind(eq)
    .bind(hwm)
    .bind(dd)
    .bind(threshold)
    .bind(action)
    .execute(pool)
    .await?;
    Ok(())
}

/// Single-shot evaluation: pull live equity, update HWM, decide, fire
/// kill-switch if breached. Always writes one log row.
pub async fn evaluate(pool: &PgPool, user_id: Uuid) -> anyhow::Result<EvaluationResult> {
    let cfg = get_config(pool, user_id).await?;
    let current_equity = live_equity_usd(pool, user_id).await;
    let hwm = new_high_water_mark(cfg.high_water_mark, current_equity);
    let dd = drawdown_pct(hwm, current_equity);

    if !cfg.enabled {
        write_log(
            pool,
            user_id,
            current_equity,
            hwm,
            dd,
            cfg.max_drawdown_pct,
            "skipped_disabled",
        )
        .await?;
        sqlx::query(
            "UPDATE drawdown_cutoff_config
                SET high_water_mark = $2, last_equity = $3, last_evaluated_at = now(),
                    updated_at = now()
              WHERE user_id = $1",
        )
        .bind(user_id)
        .bind(hwm)
        .bind(current_equity)
        .execute(pool)
        .await
        .ok();
        return Ok(EvaluationResult {
            config: get_config(pool, user_id).await.unwrap_or(cfg),
            current_equity,
            high_water_mark: hwm,
            drawdown_pct: dd,
            action: "skipped_disabled".into(),
            kill_result: None,
        });
    }

    if cfg.auto_killed_at.is_some() {
        write_log(
            pool,
            user_id,
            current_equity,
            hwm,
            dd,
            cfg.max_drawdown_pct,
            "skipped_already_fired",
        )
        .await?;
        return Ok(EvaluationResult {
            config: cfg,
            current_equity,
            high_water_mark: hwm,
            drawdown_pct: dd,
            action: "skipped_already_fired".into(),
            kill_result: None,
        });
    }

    sqlx::query(
        "INSERT INTO drawdown_cutoff_config
            (user_id, enabled, max_drawdown_pct, high_water_mark, last_equity,
             last_evaluated_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, now(), now())
         ON CONFLICT (user_id) DO UPDATE SET
            high_water_mark   = EXCLUDED.high_water_mark,
            last_equity       = EXCLUDED.last_equity,
            last_evaluated_at = now(),
            updated_at        = now()",
    )
    .bind(user_id)
    .bind(cfg.enabled)
    .bind(cfg.max_drawdown_pct)
    .bind(hwm)
    .bind(current_equity)
    .execute(pool)
    .await
    .ok();

    if !should_fire_cutoff(hwm, current_equity, cfg.max_drawdown_pct) {
        write_log(
            pool,
            user_id,
            current_equity,
            hwm,
            dd,
            cfg.max_drawdown_pct,
            "evaluated",
        )
        .await?;
        let cfg_after = get_config(pool, user_id).await.unwrap_or(cfg);
        return Ok(EvaluationResult {
            config: cfg_after,
            current_equity,
            high_water_mark: hwm,
            drawdown_pct: dd,
            action: "evaluated".into(),
            kill_result: None,
        });
    }

    let kill_result = crate::multi_broker::kill_all_for_user(pool, user_id).await?;
    sqlx::query(
        "UPDATE drawdown_cutoff_config
            SET auto_killed_at = now(), updated_at = now()
          WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .ok();
    write_log(
        pool,
        user_id,
        current_equity,
        hwm,
        dd,
        cfg.max_drawdown_pct,
        "fired",
    )
    .await?;
    let cfg_after = get_config(pool, user_id).await.unwrap_or(cfg);
    Ok(EvaluationResult {
        config: cfg_after,
        current_equity,
        high_water_mark: hwm,
        drawdown_pct: dd,
        action: "fired".into(),
        kill_result: Some(kill_result),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_fire_when_drawdown_at_threshold() {
        assert!(should_fire_cutoff(100.0, 95.0, 5.0));
    }

    #[test]
    fn should_fire_when_drawdown_above_threshold() {
        assert!(should_fire_cutoff(100.0, 80.0, 5.0));
    }

    #[test]
    fn should_not_fire_when_drawdown_below_threshold() {
        assert!(!should_fire_cutoff(100.0, 96.0, 5.0));
    }

    #[test]
    fn should_not_fire_when_equity_at_or_above_hwm() {
        assert!(!should_fire_cutoff(100.0, 100.0, 5.0));
        assert!(!should_fire_cutoff(100.0, 110.0, 5.0));
    }

    #[test]
    fn should_not_fire_when_inputs_unusable() {
        assert!(!should_fire_cutoff(0.0, 95.0, 5.0));
        assert!(!should_fire_cutoff(-100.0, 95.0, 5.0));
        assert!(!should_fire_cutoff(100.0, f64::NAN, 5.0));
        assert!(!should_fire_cutoff(100.0, 95.0, 0.0));
        assert!(!should_fire_cutoff(100.0, 95.0, -1.0));
    }

    #[test]
    fn hwm_seeds_from_current_when_no_prior() {
        assert_eq!(new_high_water_mark(None, 5000.0), 5000.0);
    }

    #[test]
    fn hwm_holds_when_current_below_prior() {
        assert_eq!(new_high_water_mark(Some(10_000.0), 9500.0), 10_000.0);
    }

    #[test]
    fn hwm_advances_when_current_above_prior() {
        assert_eq!(new_high_water_mark(Some(10_000.0), 12_000.0), 12_000.0);
    }

    #[test]
    fn hwm_floors_negative_current_to_zero() {
        assert_eq!(new_high_water_mark(None, -50.0), 0.0);
    }

    #[test]
    fn hwm_falls_back_to_prior_when_current_nan() {
        assert_eq!(new_high_water_mark(Some(10_000.0), f64::NAN), 10_000.0);
    }

    #[test]
    fn drawdown_pct_zero_when_at_hwm() {
        assert_eq!(drawdown_pct(100.0, 100.0), 0.0);
        assert_eq!(drawdown_pct(100.0, 110.0), 0.0);
    }

    #[test]
    fn drawdown_pct_computes_loss_against_hwm() {
        assert!((drawdown_pct(200.0, 180.0) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn drawdown_pct_zero_when_unusable_inputs() {
        assert_eq!(drawdown_pct(0.0, 90.0), 0.0);
        assert_eq!(drawdown_pct(100.0, f64::NAN), 0.0);
    }

    #[test]
    fn default_for_disables_cutoff() {
        let c = DrawdownConfig::default_for(Uuid::nil());
        assert!(
            !c.enabled,
            "must default OFF — destructive automation is opt-in"
        );
        assert_eq!(c.max_drawdown_pct, 5.0);
    }
}
