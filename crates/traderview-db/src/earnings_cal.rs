//! Earnings calendar + EPS surprise tracking.
//!
//! Pipeline per symbol:
//!   1. Fetch Yahoo `quoteSummary` earnings + earningsHistory modules.
//!   2. Upsert next upcoming earnings event (calendarEvents.earnings).
//!   3. Upsert past quarters from earningsHistory (epsActual / epsEstimate).
//!   4. For any past event missing reactions, compute 1d/5d from `price_bars`.
//!
//! `surprise_pct` = (actual - estimate) / |estimate| * 100, clipped to None
//! when estimate is zero (avoids ±inf).

use chrono::{DateTime, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use traderview_core::BarInterval;

use crate::market_data;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct EarningsEvent {
    pub id: uuid::Uuid,
    pub symbol: String,
    pub earnings_date: NaiveDate,
    pub timing: Option<String>,
    pub eps_estimate: Option<Decimal>,
    pub eps_actual: Option<Decimal>,
    pub revenue_estimate: Option<Decimal>,
    pub revenue_actual: Option<Decimal>,
    pub surprise_pct: Option<f32>,
    pub price_close_pre: Option<Decimal>,
    pub price_close_1d: Option<Decimal>,
    pub price_close_5d: Option<Decimal>,
    pub reaction_1d_pct: Option<f32>,
    pub reaction_5d_pct: Option<f32>,
    pub fetched_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PollStats {
    pub symbols_polled: usize,
    pub events_upserted: u64,
    pub reactions_computed: u64,
}

pub async fn poll_watchlists(pool: &PgPool) -> anyhow::Result<PollStats> {
    let symbols: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT symbol FROM watchlist_symbols ORDER BY symbol LIMIT 100",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let mut events_upserted = 0u64;
    let mut reactions_computed = 0u64;
    for s in &symbols {
        if let Ok((n, r)) = poll_symbol(pool, s).await {
            events_upserted += n;
            reactions_computed += r;
        }
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    }
    Ok(PollStats {
        symbols_polled: symbols.len(),
        events_upserted,
        reactions_computed,
    })
}

pub async fn poll_symbol(pool: &PgPool, symbol: &str) -> anyhow::Result<(u64, u64)> {
    let v = market_data::earnings(symbol).await?;
    let mut upserted = 0u64;

    // Upcoming earnings from calendarEvents.earnings.
    let cal = &v["calendarEvents"]["earnings"];
    if let Some(date) = pick_date(&cal["earningsDate"][0]) {
        let timing = guess_timing(&cal["earningsDate"][0]);
        let est = f_opt(&cal["earningsAverage"]);
        let rev_est = f_opt(&cal["revenueAverage"]);
        if upsert_event(
            pool,
            EventUpsert {
                symbol,
                date,
                timing: timing.as_deref(),
                eps_est: est,
                eps_act: None,
                rev_est,
                rev_act: None,
            },
        )
        .await?
        {
            upserted += 1;
        }
    }

    // History (past quarters with actuals).
    let history = v["earningsHistory"]["history"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for h in history.iter() {
        let Some(date) = pick_date(&h["quarter"]) else {
            continue;
        };
        let est = f_opt(&h["epsEstimate"]);
        let act = f_opt(&h["epsActual"]);
        if upsert_event(
            pool,
            EventUpsert {
                symbol,
                date,
                timing: None,
                eps_est: est,
                eps_act: act,
                rev_est: None,
                rev_act: None,
            },
        )
        .await?
        {
            upserted += 1;
        }
    }

    // Compute reactions for past events that don't have them yet.
    let stale: Vec<(uuid::Uuid, NaiveDate)> = sqlx::query_as(
        "SELECT id, earnings_date FROM earnings_events
          WHERE symbol = $1 AND reaction_5d_pct IS NULL
            AND earnings_date <= CURRENT_DATE
          ORDER BY earnings_date DESC LIMIT 20",
    )
    .bind(symbol)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let mut reactions = 0u64;
    for (id, date) in stale {
        if let Some((pre, c1, c5, r1, r5)) = compute_reactions(pool, symbol, date).await {
            let _ = sqlx::query(
                "UPDATE earnings_events SET
                    price_close_pre = $2, price_close_1d = $3, price_close_5d = $4,
                    reaction_1d_pct = $5, reaction_5d_pct = $6,
                    updated_at = now()
                  WHERE id = $1",
            )
            .bind(id)
            .bind(Decimal::try_from(pre).ok())
            .bind(Decimal::try_from(c1).ok())
            .bind(Decimal::try_from(c5).ok())
            .bind(r1 as f32)
            .bind(r5 as f32)
            .execute(pool)
            .await;
            reactions += 1;
        }
    }
    Ok((upserted, reactions))
}

struct EventUpsert<'a> {
    symbol: &'a str,
    date: NaiveDate,
    timing: Option<&'a str>,
    eps_est: Option<f64>,
    eps_act: Option<f64>,
    rev_est: Option<f64>,
    rev_act: Option<f64>,
}

async fn upsert_event(pool: &PgPool, e: EventUpsert<'_>) -> anyhow::Result<bool> {
    let surprise_pct = match (e.eps_est, e.eps_act) {
        (Some(est), Some(act)) if est.abs() > 1e-9 => {
            Some(((act - est) / est.abs() * 100.0) as f32)
        }
        _ => None,
    };
    let r = sqlx::query(
        "INSERT INTO earnings_events
            (symbol, earnings_date, timing, eps_estimate, eps_actual,
             revenue_estimate, revenue_actual, surprise_pct)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (symbol, earnings_date) DO UPDATE SET
            timing           = COALESCE(EXCLUDED.timing, earnings_events.timing),
            eps_estimate     = COALESCE(EXCLUDED.eps_estimate, earnings_events.eps_estimate),
            eps_actual       = COALESCE(EXCLUDED.eps_actual, earnings_events.eps_actual),
            revenue_estimate = COALESCE(EXCLUDED.revenue_estimate, earnings_events.revenue_estimate),
            revenue_actual   = COALESCE(EXCLUDED.revenue_actual, earnings_events.revenue_actual),
            surprise_pct     = COALESCE(EXCLUDED.surprise_pct, earnings_events.surprise_pct),
            updated_at       = now()",
    )
    .bind(e.symbol).bind(e.date).bind(e.timing)
    .bind(e.eps_est.and_then(|x| Decimal::try_from(x).ok()))
    .bind(e.eps_act.and_then(|x| Decimal::try_from(x).ok()))
    .bind(e.rev_est.and_then(|x| Decimal::try_from(x).ok()))
    .bind(e.rev_act.and_then(|x| Decimal::try_from(x).ok()))
    .bind(surprise_pct)
    .execute(pool).await?;
    Ok(r.rows_affected() > 0)
}

async fn compute_reactions(
    pool: &PgPool,
    symbol: &str,
    date: NaiveDate,
) -> Option<(f64, f64, f64, f64, f64)> {
    let to = Utc::now();
    let from = to - Duration::days(45);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .ok()?;
    if bars.len() < 7 {
        return None;
    }
    // Find the bar at or just before `date` = pre-earnings close.
    let pre_idx = bars.iter().rposition(|b| b.bar_time.date_naive() <= date)?;
    let pre = dec(bars[pre_idx].close);
    let c1 = bars.get(pre_idx + 1).map(|b| dec(b.close))?;
    let c5 = bars.get(pre_idx + 5).map(|b| dec(b.close))?;
    if pre <= 0.0 {
        return None;
    }
    let r1 = (c1 - pre) / pre * 100.0;
    let r5 = (c5 - pre) / pre * 100.0;
    Some((pre, c1, c5, r1, r5))
}

pub async fn calendar_upcoming(pool: &PgPool, days: i64) -> anyhow::Result<Vec<EarningsEvent>> {
    Ok(sqlx::query_as(
        "SELECT id, symbol, earnings_date, timing, eps_estimate, eps_actual,
                revenue_estimate, revenue_actual, surprise_pct,
                price_close_pre, price_close_1d, price_close_5d,
                reaction_1d_pct, reaction_5d_pct, fetched_at, updated_at
           FROM earnings_events
          WHERE earnings_date BETWEEN CURRENT_DATE AND CURRENT_DATE + ($1::int)
          ORDER BY earnings_date, symbol",
    )
    .bind(days as i32)
    .fetch_all(pool)
    .await?)
}

pub async fn surprises_recent(pool: &PgPool, days: i64) -> anyhow::Result<Vec<EarningsEvent>> {
    Ok(sqlx::query_as(
        "SELECT id, symbol, earnings_date, timing, eps_estimate, eps_actual,
                revenue_estimate, revenue_actual, surprise_pct,
                price_close_pre, price_close_1d, price_close_5d,
                reaction_1d_pct, reaction_5d_pct, fetched_at, updated_at
           FROM earnings_events
          WHERE earnings_date BETWEEN CURRENT_DATE - ($1::int) AND CURRENT_DATE
            AND surprise_pct IS NOT NULL
          ORDER BY ABS(surprise_pct) DESC NULLS LAST
          LIMIT 100",
    )
    .bind(days as i32)
    .fetch_all(pool)
    .await?)
}

// ---- helpers --------------------------------------------------------------

fn pick_date(v: &Value) -> Option<NaiveDate> {
    if let Some(s) = v["fmt"].as_str() {
        if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Some(d);
        }
    }
    if let Some(ts) = v["raw"].as_i64() {
        if let Some(d) = chrono::DateTime::from_timestamp(ts, 0) {
            return Some(d.date_naive());
        }
    }
    None
}

fn guess_timing(v: &Value) -> Option<String> {
    // Yahoo sometimes provides earningsCallTime hint; otherwise infer
    // from the timestamp hour.
    if let Some(ts) = v["raw"].as_i64() {
        if let Some(d) = chrono::DateTime::from_timestamp(ts, 0) {
            // ET hour after market close (>= 16) → AMC, before open (< 9) → BMO.
            let h = d
                .with_timezone(&chrono::FixedOffset::west_opt(5 * 3600).unwrap())
                .hour();
            return Some(
                if h >= 16 {
                    "amc"
                } else if h < 9 {
                    "bmo"
                } else {
                    "unknown"
                }
                .into(),
            );
        }
    }
    None
}

use chrono::Timelike;

fn f_opt(v: &Value) -> Option<f64> {
    if let Some(n) = v["raw"].as_f64() {
        return Some(n);
    }
    if let Some(n) = v["raw"].as_i64() {
        return Some(n as f64);
    }
    None
}

fn dec(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ─── pick_date ─────────────────────────────────────────────────────────
    #[test]
    fn pick_date_prefers_fmt_when_present() {
        let v = json!({ "fmt": "2024-03-15", "raw": 0 });
        assert_eq!(
            pick_date(&v),
            Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap())
        );
    }

    #[test]
    fn pick_date_falls_back_to_raw_epoch_when_fmt_missing() {
        // 2024-01-02 00:00:00 UTC = 1704153600.
        let v = json!({ "raw": 1704153600i64 });
        assert_eq!(
            pick_date(&v),
            Some(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap())
        );
    }

    #[test]
    fn pick_date_returns_none_when_neither_field_usable() {
        assert!(pick_date(&json!({})).is_none());
        assert!(pick_date(&json!({ "fmt": "not-a-date" })).is_none());
        assert!(pick_date(&json!({ "raw": "stringy" })).is_none());
    }

    #[test]
    fn pick_date_falls_through_to_raw_when_fmt_unparseable() {
        let v = json!({ "fmt": "bogus", "raw": 1704153600i64 });
        assert_eq!(
            pick_date(&v),
            Some(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap())
        );
    }

    // ─── guess_timing ──────────────────────────────────────────────────────
    #[test]
    fn guess_timing_classifies_amc_when_et_hour_ge_16() {
        // 2024-01-02 21:30 UTC = 16:30 ET (winter, UTC-5) → AMC.
        let ts = 1704231000i64;
        let v = json!({ "raw": ts });
        assert_eq!(guess_timing(&v).as_deref(), Some("amc"));
    }

    #[test]
    fn guess_timing_classifies_bmo_when_et_hour_lt_9() {
        // 2024-01-02 13:00 UTC = 08:00 ET → BMO.
        let ts = 1704200400i64;
        let v = json!({ "raw": ts });
        assert_eq!(guess_timing(&v).as_deref(), Some("bmo"));
    }

    #[test]
    fn guess_timing_returns_unknown_for_midday_hours() {
        // 2024-01-02 17:00 UTC = 12:00 ET → unknown.
        let ts = 1704214800i64;
        let v = json!({ "raw": ts });
        assert_eq!(guess_timing(&v).as_deref(), Some("unknown"));
    }

    #[test]
    fn guess_timing_returns_none_without_raw_timestamp() {
        assert!(guess_timing(&json!({})).is_none());
        assert!(guess_timing(&json!({ "fmt": "2024-01-01" })).is_none());
    }

    // ─── f_opt ─────────────────────────────────────────────────────────────
    #[test]
    fn f_opt_extracts_floats_and_ints_from_raw() {
        assert_eq!(f_opt(&json!({ "raw": 1.5 })), Some(1.5));
        assert_eq!(f_opt(&json!({ "raw": 42 })), Some(42.0));
        assert_eq!(f_opt(&json!({ "raw": -3 })), Some(-3.0));
    }

    #[test]
    fn f_opt_returns_none_when_raw_missing_or_non_numeric() {
        assert!(f_opt(&json!({})).is_none());
        assert!(f_opt(&json!({ "raw": "1.5" })).is_none());
        assert!(f_opt(&json!({ "raw": null })).is_none());
    }

    // ─── dec ───────────────────────────────────────────────────────────────
    #[test]
    fn dec_roundtrips_decimal_to_f64() {
        use std::str::FromStr;
        assert_eq!(dec(Decimal::from_str("1.23").unwrap()), 1.23);
        assert_eq!(dec(Decimal::from_str("-100.5").unwrap()), -100.5);
        assert_eq!(dec(Decimal::ZERO), 0.0);
    }
}
