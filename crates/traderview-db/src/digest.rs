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
            if let Ok(Some((report, _, _))) = crate::algo::live_divergence(pool, uid, id).await {
                if matches!(report.verdict, "degraded" | "watch") {
                    d.drifting_strategies.push(format!("{name} ({})", report.verdict));
                }
            }
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
    }
}
