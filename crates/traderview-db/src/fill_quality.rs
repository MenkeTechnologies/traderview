//! Fill quality analytics.
//!
//! We don't cache intraday bid/ask — only daily OHLC — so this is a
//! BAR-LEVEL approximation, not tick-perfect slippage. Two metrics per
//! fill:
//!
//!   1. `fill_in_range` ∈ [0..1]: where the fill sat between that day's
//!      low and high. For BUYs lower is better (bought near the low);
//!      for SELLs/SHORTs higher is better (sold near the high). We
//!      normalize into `fill_efficiency` so 100% always means "best
//!      possible fill within the day's range".
//!
//!   2. `slippage_bps`: deviation from the bar's typical price (HLC/3)
//!      in basis points, sign-corrected so a positive number = worse
//!      than typical for that side.
//!
//! Aggregations: by symbol, by order-size bucket, by hour-of-day (ET).

use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use traderview_core::{BarInterval, PriceBar};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct FillSample {
    pub execution_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub qty: f64,
    pub fill_price: f64,
    pub executed_at: DateTime<Utc>,
    pub bar_open: f64,
    pub bar_high: f64,
    pub bar_low: f64,
    pub bar_close: f64,
    pub typical_price: f64,         // HLC / 3
    pub fill_in_range: f64,         // 0..1
    pub fill_efficiency: f64,       // 0..1, side-adjusted
    pub slippage_bps: f64,          // positive = worse than typical for the side
    pub hour_et: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bucket {
    pub key: String,
    pub samples: usize,
    pub avg_fill_efficiency: f64,
    pub avg_slippage_bps: f64,
    pub median_slippage_bps: f64,
    pub worst_slippage_bps: f64,
    pub best_slippage_bps: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FillQualityReport {
    pub account_id: Uuid,
    pub samples: Vec<FillSample>,
    pub overall: Bucket,
    pub by_symbol:   Vec<Bucket>,
    pub by_hour_et:  Vec<Bucket>,
    pub by_size:     Vec<Bucket>,         // <100 / 100-500 / 500-2k / 2k-10k / 10k+
    pub skipped_no_bar: usize,
    pub computed_at: DateTime<Utc>,
}

pub async fn report(pool: &PgPool, _user_id: Uuid, account_id: Uuid)
    -> anyhow::Result<FillQualityReport>
{
    let execs: Vec<(Uuid, String, String, Decimal, Decimal, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, symbol, side::text, qty, price, executed_at
           FROM executions
          WHERE account_id = $1
          ORDER BY executed_at DESC
          LIMIT 1000",
    ).bind(account_id).fetch_all(pool).await?;

    if execs.is_empty() {
        return Ok(empty_report(account_id));
    }

    let mut samples: Vec<FillSample> = Vec::new();
    let mut skipped = 0usize;

    // Bar cache per symbol so we hit prices once per ticker.
    let mut bar_cache: HashMap<String, Vec<PriceBar>> = HashMap::new();
    let to = Utc::now();

    for (id, symbol, side, qty, price, executed_at) in execs {
        // Lazy-load 2 years of daily bars per distinct symbol.
        let bars = if let Some(b) = bar_cache.get(&symbol) {
            b
        } else {
            let from = to - chrono::Duration::days(730);
            let b = crate::prices::get_bars(pool, &symbol, BarInterval::D1, from, to)
                .await.unwrap_or_default();
            bar_cache.insert(symbol.clone(), b);
            bar_cache.get(&symbol).unwrap()
        };

        // Match fill to the bar dated on the same day, or fall back to the
        // most recent bar at or before fill date.
        let fill_date = executed_at.date_naive();
        let bar_idx = bars.iter().rposition(|b| b.bar_time.date_naive() <= fill_date);
        let bar = match bar_idx.and_then(|i| bars.get(i)) {
            Some(b) => b,
            None => { skipped += 1; continue; }
        };
        let open = dec(bar.open);
        let high = dec(bar.high);
        let low  = dec(bar.low);
        let close = dec(bar.close);
        if high - low <= 1e-9 { skipped += 1; continue; }
        let typical = (high + low + close) / 3.0;
        let fill = dec(price);
        let range_pos = ((fill - low) / (high - low)).clamp(0.0, 1.0);
        let is_buy = side == "buy" || side == "cover";
        let efficiency = if is_buy { 1.0 - range_pos } else { range_pos };
        // Side-adjusted slippage: positive = worse than typical.
        // For buys, paying above typical = positive slippage.
        // For sells, getting less than typical = positive slippage.
        let raw_bps = (fill - typical) / typical * 10_000.0;
        let slippage_bps = if is_buy { raw_bps } else { -raw_bps };

        // Hour of day in ET (UTC-5 winter, UTC-4 summer; cheap approx UTC-5).
        let et = chrono::FixedOffset::west_opt(5 * 3600).unwrap()
            .from_utc_datetime(&executed_at.naive_utc());
        let hour_et = et.format("%H").to_string().parse::<u32>().unwrap_or(0);

        samples.push(FillSample {
            execution_id: id, symbol: symbol.clone(), side, qty: dec(qty),
            fill_price: fill, executed_at,
            bar_open: open, bar_high: high, bar_low: low, bar_close: close,
            typical_price: typical, fill_in_range: range_pos,
            fill_efficiency: efficiency, slippage_bps,
            hour_et,
        });
    }

    let overall = bucket_stats("overall", &samples);
    let by_symbol = group_by(&samples, |s| s.symbol.clone());
    let by_hour_et = group_by(&samples, |s| format!("{:02}:00 ET", s.hour_et));
    let by_size   = group_by(&samples, |s| size_bucket(s.qty));

    Ok(FillQualityReport {
        account_id, samples, overall,
        by_symbol, by_hour_et, by_size,
        skipped_no_bar: skipped,
        computed_at: Utc::now(),
    })
}

fn empty_report(account_id: Uuid) -> FillQualityReport {
    FillQualityReport {
        account_id,
        samples: vec![],
        overall: empty_bucket("overall"),
        by_symbol: vec![], by_hour_et: vec![], by_size: vec![],
        skipped_no_bar: 0,
        computed_at: Utc::now(),
    }
}
fn empty_bucket(key: &str) -> Bucket {
    Bucket {
        key: key.into(), samples: 0,
        avg_fill_efficiency: 0.0, avg_slippage_bps: 0.0,
        median_slippage_bps: 0.0, worst_slippage_bps: 0.0, best_slippage_bps: 0.0,
    }
}

fn bucket_stats(key: &str, samples: &[FillSample]) -> Bucket {
    if samples.is_empty() { return empty_bucket(key); }
    let n = samples.len() as f64;
    let avg_eff = samples.iter().map(|s| s.fill_efficiency).sum::<f64>() / n;
    let mut slips: Vec<f64> = samples.iter().map(|s| s.slippage_bps).collect();
    let avg_slip = slips.iter().sum::<f64>() / n;
    slips.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if slips.len() % 2 == 1 { slips[slips.len() / 2] }
                 else { (slips[slips.len() / 2 - 1] + slips[slips.len() / 2]) / 2.0 };
    Bucket {
        key: key.into(), samples: samples.len(),
        avg_fill_efficiency: avg_eff, avg_slippage_bps: avg_slip,
        median_slippage_bps: median,
        worst_slippage_bps: *slips.last().unwrap_or(&0.0),
        best_slippage_bps:  *slips.first().unwrap_or(&0.0),
    }
}

fn group_by<F>(samples: &[FillSample], key_of: F) -> Vec<Bucket>
where F: Fn(&FillSample) -> String,
{
    let mut groups: HashMap<String, Vec<FillSample>> = HashMap::new();
    for s in samples { groups.entry(key_of(s)).or_default().push(s.clone()); }
    let mut out: Vec<Bucket> = groups.into_iter()
        .map(|(k, v)| bucket_stats(&k, &v))
        .collect();
    out.sort_by(|a, b| b.avg_slippage_bps.partial_cmp(&a.avg_slippage_bps)
        .unwrap_or(std::cmp::Ordering::Equal));
    out
}

fn size_bucket(qty: f64) -> String {
    if qty < 100.0       { "size < 100".into() }
    else if qty < 500.0   { "size 100-499".into() }
    else if qty < 2_000.0 { "size 500-1999".into() }
    else if qty < 10_000.0 { "size 2k-9.9k".into() }
    else                  { "size 10k+".into() }
}

fn dec(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }
