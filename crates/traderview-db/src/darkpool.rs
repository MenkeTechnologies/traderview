//! Off-exchange / dark-pool volume estimator.
//!
//! FINRA's daily Reg SHO file (`CNMSshvolYYYYMMDD.txt`) gives the total share
//! volume that printed *off-exchange* — i.e. through the Trade Reporting
//! Facility (TRF), which captures ATS / dark-pool and internalizer prints.
//! Yahoo Finance's daily bar gives the *total* consolidated volume across
//! all venues. Dividing one by the other yields the off-exchange share %
//! per symbol per day — a free, conservative dark-pool proxy.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

#[derive(Debug, Clone, Serialize)]
pub struct DarkDay {
    pub date: NaiveDate,
    pub off_exchange_volume: u64,
    pub total_volume: u64,
    pub off_exchange_pct: f64,    // 0..1
}

#[derive(Debug, Clone, Serialize)]
pub struct DarkSeries {
    pub symbol: String,
    pub days: Vec<DarkDay>,
    pub avg_off_exchange_pct: f64,
    pub fetched_at: DateTime<Utc>,
}

pub async fn series(pool: &PgPool, symbol: &str, days_back: i64) -> anyhow::Result<DarkSeries> {
    let symbol = symbol.to_uppercase();
    // Pull FINRA short-vol (gives TRF off-exchange totals) and Yahoo bars
    // (consolidated total) over the same window.
    let finra = crate::short_interest::finra_daily(&symbol, days_back).await
        .unwrap_or_default();
    let to = Utc::now();
    let from = to - Duration::days(days_back + 5);
    let bars = crate::prices::get_bars(pool, &symbol, BarInterval::D1, from, to)
        .await.unwrap_or_default();

    let mut by_day: std::collections::BTreeMap<NaiveDate, (u64, u64)> =
        std::collections::BTreeMap::new();
    for d in finra {
        by_day.entry(d.date).or_insert((0, 0)).0 = d.total_volume;
    }
    for b in bars {
        let day = b.bar_time.date_naive();
        let vol = dec(b.volume) as u64;
        by_day.entry(day).or_insert((0, 0)).1 = vol;
    }
    let mut out = Vec::with_capacity(by_day.len());
    let mut sum_pct = 0.0;
    let mut n = 0usize;
    for (date, (off, tot)) in by_day {
        if off == 0 || tot == 0 { continue; }
        let pct = (off as f64 / tot as f64).min(1.0);
        out.push(DarkDay {
            date, off_exchange_volume: off, total_volume: tot, off_exchange_pct: pct,
        });
        sum_pct += pct;
        n += 1;
    }
    Ok(DarkSeries {
        symbol,
        avg_off_exchange_pct: if n > 0 { sum_pct / n as f64 } else { 0.0 },
        days: out,
        fetched_at: Utc::now(),
    })
}

fn dec(d: rust_decimal::Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }

#[derive(Debug, Clone, Serialize)]
pub struct DarkRanked {
    pub symbol: String,
    pub avg_off_exchange_pct: f64,
    pub samples: usize,
    pub latest_pct: Option<f64>,
}

pub async fn ranked(pool: &PgPool, symbols: &[String], days_back: i64) -> Vec<DarkRanked> {
    let mut out = Vec::new();
    for sym in symbols {
        if let Ok(s) = series(pool, sym, days_back).await {
            if !s.days.is_empty() {
                out.push(DarkRanked {
                    symbol: s.symbol,
                    avg_off_exchange_pct: s.avg_off_exchange_pct,
                    samples: s.days.len(),
                    latest_pct: s.days.last().map(|d| d.off_exchange_pct),
                });
            }
        }
    }
    out.sort_by(|a, b| b.avg_off_exchange_pct.partial_cmp(&a.avg_off_exchange_pct)
        .unwrap_or(std::cmp::Ordering::Equal));
    out
}
