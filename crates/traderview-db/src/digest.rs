//! Morning digest — one pre-market summary composing everything the
//! background watchers know: paper equity day-change, strategy drift
//! verdicts, gate fires, held positions reporting earnings soon, and
//! rebalance targets above tolerance.
//!
//! The digest REPORTS; it never acts. Sections with nothing to say are
//! omitted entirely — a digest of empty sections trains the user to
//! stop reading.

use chrono::{Datelike, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Default)]
pub struct Digest {
    pub accounts: Vec<AccountLine>,
    pub drifting_strategies: Vec<String>,
    pub gate_fires_24h: i64,
    pub earnings_soon: Vec<EarningsLine>,
    pub rebalance_needed: Vec<String>,
    /// Strategies at ≥70% of their max-drawdown circuit breaker —
    /// the warning BEFORE the auto-pause, while there's still a cap
    /// left to act inside.
    pub breaker_proximity: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountLine {
    pub name: String,
    pub equity: f64,
    pub day_change: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EarningsLine {
    pub symbol: String,
    pub date: NaiveDate,
}

impl Digest {
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
            && self.drifting_strategies.is_empty()
            && self.gate_fires_24h == 0
            && self.earnings_soon.is_empty()
            && self.rebalance_needed.is_empty()
            && self.breaker_proximity.is_empty()
    }
}

/// Human-readable digest body; empty sections omitted.
pub fn format_digest(d: &Digest) -> String {
    let mut out = Vec::new();
    for a in &d.accounts {
        match a.day_change {
            Some(c) => out.push(format!(
                "{}: ${:.0} ({}{:.0} today)",
                a.name,
                a.equity,
                if c >= 0.0 { "+" } else { "" },
                c
            )),
            None => out.push(format!("{}: ${:.0}", a.name, a.equity)),
        }
    }
    if !d.drifting_strategies.is_empty() {
        out.push(format!("drifting: {}", d.drifting_strategies.join(", ")));
    }
    if d.gate_fires_24h > 0 {
        out.push(format!("{} gate fires in 24h", d.gate_fires_24h));
    }
    for e in &d.earnings_soon {
        out.push(format!("{} reports {}", e.symbol, e.date));
    }
    if !d.rebalance_needed.is_empty() {
        out.push(format!("rebalance: {}", d.rebalance_needed.join(", ")));
    }
    if !d.breaker_proximity.is_empty() {
        out.push(format!("near breaker: {}", d.breaker_proximity.join(", ")));
    }
    out.join(" · ")
}

/// Assemble the digest for one user. Read-only; failures in one
/// section degrade to an empty section rather than killing the digest.
pub async fn for_user(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Digest> {
    let mut d = Digest::default();
    let day_ago = Utc::now() - Duration::hours(24);

    // Paper accounts + day change from equity snapshots.
    let accounts: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, name FROM paper_accounts WHERE user_id = $1 ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    for (account_id, name) in &accounts {
        let latest: Option<(Decimal,)> = sqlx::query_as(
            "SELECT equity FROM paper_equity_snapshots
              WHERE paper_account_id = $1 ORDER BY taken_at DESC LIMIT 1",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
        let Some((equity,)) = latest else { continue };
        let prior: Option<(Decimal,)> = sqlx::query_as(
            "SELECT equity FROM paper_equity_snapshots
              WHERE paper_account_id = $1 AND taken_at <= $2
              ORDER BY taken_at DESC LIMIT 1",
        )
        .bind(account_id)
        .bind(day_ago)
        .fetch_optional(pool)
        .await?;
        use rust_decimal::prelude::ToPrimitive;
        d.accounts.push(AccountLine {
            name: name.clone(),
            equity: equity.to_f64().unwrap_or(0.0),
            day_change: prior
                .map(|(p,)| (equity - p).to_f64().unwrap_or(0.0)),
        });
    }

    // Drifting strategies (same comparison the watch uses).
    if let Ok(strategies) = crate::algo::all_active_strategy_ids(pool).await {
        for (id, uid, name) in strategies {
            if uid != user_id {
                continue;
            }
            if let Ok(Some(div)) = crate::algo::live_divergence(pool, uid, id).await {
                if matches!(div.report.verdict, "degraded" | "watch") {
                    d.drifting_strategies.push(format!("{name} ({})", div.report.verdict));
                }
            }
        }
    }

    // Circuit-breaker proximity: enabled strategies with a drawdown
    // cap whose CURRENT drawdown (same realized_drawdown the gate
    // itself uses) is at ≥70% of it. The gate fires at 100% and
    // auto-pauses; this is the morning warning while acting is still
    // possible. Already-breached (paused) strategies aren't enabled,
    // so they drop out naturally.
    let capped: Vec<(Uuid, String, f64)> = sqlx::query_as(
        "SELECT id, name, (risk_gates->>'max_drawdown_usd')::float8
           FROM algo_strategies
          WHERE user_id = $1 AND enabled = true AND deleted_at IS NULL
            AND COALESCE((risk_gates->>'max_drawdown_usd')::float8, 0) > 0",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for (sid, name, cap) in capped {
        let Ok(trips) = crate::algo::strategy_trips(pool, user_id, sid).await else {
            continue;
        };
        let pnls: Vec<f64> = trips.iter().map(|t| t.pnl).collect();
        let dd = traderview_core::live_vs_backtest::realized_drawdown(&pnls);
        if dd >= 0.7 * cap {
            d.breaker_proximity
                .push(format!("{name} (${dd:.0} of ${cap:.0} drawdown cap)"));
        }
    }

    // Gate fires over 24h, across the user's strategies.
    let fires: Option<(i64,)> = sqlx::query_as(
        "SELECT count(*) FROM algo_gate_fires f
           JOIN algo_strategies s ON s.id = f.strategy_id
          WHERE s.user_id = $1 AND f.fired_at > $2",
    )
    .bind(user_id)
    .bind(day_ago)
    .fetch_optional(pool)
    .await?;
    d.gate_fires_24h = fires.map(|(n,)| n).unwrap_or(0);

    // Held paper symbols reporting earnings within 3 days.
    let today = Utc::now().date_naive();
    let horizon = today + Duration::days(3);
    let earnings: Vec<(String, NaiveDate)> = sqlx::query_as(
        "SELECT DISTINCT p.symbol, e.earnings_date
           FROM paper_positions p
           JOIN paper_accounts a ON a.id = p.paper_account_id
           JOIN earnings_events e ON e.symbol = p.symbol
          WHERE a.user_id = $1 AND e.earnings_date BETWEEN $2 AND $3
          ORDER BY e.earnings_date",
    )
    .bind(user_id)
    .bind(today)
    .bind(horizon)
    .fetch_all(pool)
    .await?;
    d.earnings_soon = earnings
        .into_iter()
        .map(|(symbol, date)| EarningsLine { symbol, date })
        .collect();

    // Rebalance targets above tolerance (same plan the watch uses).
    if let Ok(targets) = crate::paper_rebalance::all_target_ids(pool).await {
        for (id, uid, name) in targets {
            if uid != user_id {
                continue;
            }
            if let Ok(Some(p)) = crate::paper_rebalance::plan(pool, uid, id).await {
                if p.above_threshold {
                    d.rebalance_needed
                        .push(format!("{name} ({:.1}%)", p.max_drift_pct));
                }
            }
        }
    }
    Ok(d)
}

/// Users due a digest at this UTC hour: paper-account holders whose
/// preferred hour matches (default 12 with no prefs row) and who
/// haven't been sent today's digest. last_sent_on makes delivery
/// exactly-once-per-day across restarts.
pub async fn due_users(pool: &PgPool, hour_utc: u32) -> anyhow::Result<Vec<Uuid>> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT DISTINCT pa.user_id
           FROM paper_accounts pa
           LEFT JOIN digest_prefs dp ON dp.user_id = pa.user_id
          WHERE COALESCE(dp.hour_utc, 12) = $1
            AND (dp.last_sent_on IS NULL OR dp.last_sent_on < CURRENT_DATE)",
    )
    .bind(hour_utc as i32)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(u,)| u).collect())
}

pub async fn mark_sent(pool: &PgPool, user_id: Uuid) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO digest_prefs (user_id, last_sent_on) VALUES ($1, CURRENT_DATE)
         ON CONFLICT (user_id) DO UPDATE SET last_sent_on = CURRENT_DATE",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_hour(pool: &PgPool, user_id: Uuid) -> anyhow::Result<u32> {
    let row: Option<(i32,)> =
        sqlx::query_as("SELECT hour_utc FROM digest_prefs WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(h,)| h as u32).unwrap_or(12))
}

pub async fn set_hour(pool: &PgPool, user_id: Uuid, hour_utc: u32) -> anyhow::Result<()> {
    if hour_utc > 23 {
        anyhow::bail!("hour must be 0..=23");
    }
    sqlx::query(
        "INSERT INTO digest_prefs (user_id, hour_utc) VALUES ($1, $2)
         ON CONFLICT (user_id) DO UPDATE SET hour_utc = EXCLUDED.hour_utc",
    )
    .bind(user_id)
    .bind(hour_utc as i32)
    .execute(pool)
    .await?;
    Ok(())
}

/// Next top-of-hour strictly after `now` — the hourly scheduler tick.
pub fn next_top_of_hour(now: chrono::DateTime<Utc>) -> chrono::DateTime<Utc> {
    use chrono::Timelike;
    let truncated = now
        .date_naive()
        .and_hms_opt(now.hour(), 0, 0)
        .unwrap()
        .and_utc();
    truncated + Duration::hours(1)
}

/// Every user with a paper account — the digest audience.
pub async fn audience(pool: &PgPool) -> anyhow::Result<Vec<Uuid>> {
    let rows: Vec<(Uuid,)> =
        sqlx::query_as("SELECT DISTINCT user_id FROM paper_accounts").fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(u,)| u).collect())
}

/// Next occurrence of HH:00 UTC strictly after `now`.
pub fn next_digest_time(
    now: chrono::DateTime<Utc>,
    hour: u32,
) -> chrono::DateTime<Utc> {
    let today_at = now
        .date_naive()
        .and_hms_opt(hour, 0, 0)
        .unwrap()
        .and_utc();
    if today_at > now {
        today_at
    } else {
        (now.date_naive() + Duration::days(1))
            .and_hms_opt(hour, 0, 0)
            .unwrap()
            .and_utc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn next_digest_time_pins_before_at_after() {
        let h = 12;
        let before = Utc.with_ymd_and_hms(2026, 6, 10, 9, 0, 0).unwrap();
        assert_eq!(
            next_digest_time(before, h),
            Utc.with_ymd_and_hms(2026, 6, 10, 12, 0, 0).unwrap()
        );
        // Exactly at the hour: STRICTLY after → tomorrow.
        let at = Utc.with_ymd_and_hms(2026, 6, 10, 12, 0, 0).unwrap();
        assert_eq!(
            next_digest_time(at, h),
            Utc.with_ymd_and_hms(2026, 6, 11, 12, 0, 0).unwrap()
        );
        let after = Utc.with_ymd_and_hms(2026, 6, 10, 18, 30, 0).unwrap();
        assert_eq!(
            next_digest_time(after, h),
            Utc.with_ymd_and_hms(2026, 6, 11, 12, 0, 0).unwrap()
        );
        // Month boundary rolls correctly.
        let eom = Utc.with_ymd_and_hms(2026, 6, 30, 13, 0, 0).unwrap();
        assert_eq!(next_digest_time(eom, h).day(), 1);
    }

    #[test]
    fn next_top_of_hour_pins_strictness() {
        use chrono::Timelike;
        let mid = Utc.with_ymd_and_hms(2026, 6, 10, 9, 41, 7).unwrap();
        assert_eq!(next_top_of_hour(mid), Utc.with_ymd_and_hms(2026, 6, 10, 10, 0, 0).unwrap());
        // Exactly on the hour: STRICTLY after → next hour.
        let on = Utc.with_ymd_and_hms(2026, 6, 10, 9, 0, 0).unwrap();
        assert_eq!(next_top_of_hour(on).hour(), 10);
        // Day boundary.
        let late = Utc.with_ymd_and_hms(2026, 6, 10, 23, 30, 0).unwrap();
        assert_eq!(next_top_of_hour(late), Utc.with_ymd_and_hms(2026, 6, 11, 0, 0, 0).unwrap());
    }

    #[test]
    fn format_omits_empty_sections() {
        let mut d = Digest::default();
        assert_eq!(format_digest(&d), "");
        d.accounts.push(AccountLine {
            name: "SimTrader".into(),
            equity: 201_500.0,
            day_change: Some(1_500.0),
        });
        d.gate_fires_24h = 3;
        let s = format_digest(&d);
        assert!(s.contains("SimTrader: $201500 (+1500 today)"));
        assert!(s.contains("3 gate fires in 24h"));
        assert!(!s.contains("drifting"));
        assert!(!s.contains("rebalance"));
        assert!(!s.contains("near breaker"));
    }

    #[test]
    fn breaker_proximity_formats_and_counts_nonempty() {
        let mut d = Digest::default();
        d.breaker_proximity
            .push("momo ($82 of $100 drawdown cap)".into());
        assert!(!d.is_empty());
        assert!(format_digest(&d).contains("near breaker: momo ($82 of $100 drawdown cap)"));
    }
}
