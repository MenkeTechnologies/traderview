//! Mood-vs-PnL analytics.
//!
//! Two correlation tracks:
//!   1. Per-trade direct: a journal entry attached to a trade (trade_id IS
//!      NOT NULL) — the mood at the time of journaling is linked to that
//!      trade's net_pnl directly. This is the cleanest signal but rare —
//!      most users journal per-day, not per-trade.
//!   2. Per-day "next-trades": for each daily journal entry with a non-null
//!      mood, find all trades opened on that day or the following session
//!      and aggregate their net_pnl. This is the predictive view: does
//!      'felt great today' actually correspond to wins?

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct MoodTradePair {
    pub source: String,                         // "per_trade" | "per_day"
    pub mood: i16,                              // -2..+2
    pub day: Option<chrono::NaiveDate>,
    pub trade_id: Uuid,
    pub symbol: String,
    pub net_pnl: f64,
    pub r_multiple: Option<f64>,
    pub opened_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoodStat {
    pub mood: i16,
    pub sample_count: i64,
    pub win_count: i64,
    pub loss_count: i64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub median_pnl: f64,
    pub avg_r: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoodReport {
    pub pairs: Vec<MoodTradePair>,
    pub stats: Vec<MoodStat>,            // one per mood bucket present
    pub overall_correlation: Option<f64>,// Pearson(mood, net_pnl) across all pairs
    pub samples_total: usize,
    pub mood_distribution: Vec<(i16, i64)>,  // (mood, count) ascending
    pub computed_at: DateTime<Utc>,
}

pub async fn report(pool: &PgPool, user_id: Uuid, account_id: Uuid)
    -> anyhow::Result<MoodReport>
{
    // Path 1 — per-trade journal entries.
    let direct: Vec<(i16, Uuid, String, Decimal, Option<Decimal>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT j.mood AS mood, t.id, t.symbol, t.net_pnl, t.risk_amount, t.opened_at
           FROM journal_entries j
           JOIN trades t ON t.id = j.trade_id
          WHERE j.user_id = $1
            AND t.account_id = $2
            AND t.status = 'closed'
            AND t.net_pnl IS NOT NULL
            AND j.mood IS NOT NULL",
    ).bind(user_id).bind(account_id).fetch_all(pool).await?;

    // Path 2 — per-day mood × trades opened that day.
    let per_day: Vec<(i16, chrono::NaiveDate, Uuid, String, Decimal, Option<Decimal>, DateTime<Utc>)> =
        sqlx::query_as(
            "SELECT j.mood, j.day, t.id, t.symbol, t.net_pnl, t.risk_amount, t.opened_at
               FROM journal_entries j
               JOIN trades t ON t.account_id = $2
                            AND DATE(t.opened_at AT TIME ZONE 'UTC') = j.day
              WHERE j.user_id = $1
                AND j.day IS NOT NULL
                AND j.mood IS NOT NULL
                AND j.trade_id IS NULL
                AND t.status = 'closed'
                AND t.net_pnl IS NOT NULL",
        ).bind(user_id).bind(account_id).fetch_all(pool).await?;

    let mut pairs: Vec<MoodTradePair> = Vec::with_capacity(direct.len() + per_day.len());
    for (mood, tid, sym, pnl, risk, opened) in direct {
        let net = dec(pnl);
        let r = risk.and_then(|r| {
            let rv = dec(r);
            if rv > 0.0 { Some(net / rv) } else { None }
        });
        pairs.push(MoodTradePair {
            source: "per_trade".into(), mood, day: None, trade_id: tid,
            symbol: sym, net_pnl: net, r_multiple: r, opened_at: opened,
        });
    }
    for (mood, day, tid, sym, pnl, risk, opened) in per_day {
        let net = dec(pnl);
        let r = risk.and_then(|r| {
            let rv = dec(r);
            if rv > 0.0 { Some(net / rv) } else { None }
        });
        pairs.push(MoodTradePair {
            source: "per_day".into(), mood, day: Some(day), trade_id: tid,
            symbol: sym, net_pnl: net, r_multiple: r, opened_at: opened,
        });
    }

    // Aggregate per-mood stats.
    use std::collections::BTreeMap;
    let mut by_mood: BTreeMap<i16, Vec<&MoodTradePair>> = BTreeMap::new();
    for p in &pairs { by_mood.entry(p.mood).or_default().push(p); }
    let mut stats: Vec<MoodStat> = Vec::new();
    let mut mood_distribution: Vec<(i16, i64)> = Vec::new();
    for (mood, ps) in &by_mood {
        let pnls: Vec<f64> = ps.iter().map(|p| p.net_pnl).collect();
        let wins  = pnls.iter().filter(|x| **x > 0.0).count() as i64;
        let losses = pnls.iter().filter(|x| **x < 0.0).count() as i64;
        let total: f64 = pnls.iter().sum();
        let n = pnls.len() as f64;
        let avg = if n > 0.0 { total / n } else { 0.0 };
        let median = {
            let mut v = pnls.clone();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            if v.is_empty() { 0.0 }
            else if v.len() % 2 == 1 { v[v.len() / 2] }
            else { (v[v.len() / 2 - 1] + v[v.len() / 2]) / 2.0 }
        };
        let rs: Vec<f64> = ps.iter().filter_map(|p| p.r_multiple).collect();
        let avg_r = if rs.is_empty() { None } else { Some(rs.iter().sum::<f64>() / rs.len() as f64) };
        stats.push(MoodStat {
            mood: *mood, sample_count: ps.len() as i64,
            win_count: wins, loss_count: losses,
            win_rate: if !pnls.is_empty() { wins as f64 / pnls.len() as f64 } else { 0.0 },
            total_pnl: total, avg_pnl: avg, median_pnl: median, avg_r,
        });
        mood_distribution.push((*mood, ps.len() as i64));
    }

    // Pearson(mood, net_pnl) across all pairs.
    let moods: Vec<f64> = pairs.iter().map(|p| p.mood as f64).collect();
    let pnls:  Vec<f64> = pairs.iter().map(|p| p.net_pnl).collect();
    let overall_correlation = traderview_core::correlation::pearson(&moods, &pnls);

    Ok(MoodReport {
        samples_total: pairs.len(),
        pairs, stats, overall_correlation, mood_distribution,
        computed_at: Utc::now(),
    })
}

fn dec(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }
