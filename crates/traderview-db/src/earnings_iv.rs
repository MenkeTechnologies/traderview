//! Earnings-week IV scanner — for each symbol's NEXT earnings:
//!   1. find next earnings date (Yahoo quoteSummary calendarEvents)
//!   2. find first option expiration strictly AFTER the earnings date
//!   3. take ATM straddle on that expiration → implied move %
//!   4. pull last 8 historical earnings dates (Yahoo earningsHistory)
//!   5. for each, compute realized close→close abs move %
//!   6. run the IV backtest → edge + win rate + recommendation
//!
//! Result is cached in `implied_moves` for ~24h.

use crate::market_data;
use crate::options;
use chrono::{Duration, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::iv_backtest::{backtest, StraddleBacktest};
use traderview_core::BarInterval;

#[derive(Debug, Clone, Serialize)]
pub struct EarningsIvReport {
    pub symbol: String,
    pub earnings_date: NaiveDate,
    pub days_until: i64,
    pub spot: f64,
    pub atm_strike: f64,
    pub expiration: NaiveDate,
    pub call_mid: f64,
    pub put_mid: f64,
    pub implied_move_pct: f64,
    pub historical: Vec<HistoricalMove>,
    pub backtest: StraddleBacktest,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoricalMove {
    pub earnings_date: NaiveDate,
    pub close_before: f64,
    pub close_after: f64,
    pub abs_move_pct: f64,
    pub direction: &'static str,
}

pub async fn report(pool: &PgPool, symbol: &str) -> anyhow::Result<EarningsIvReport> {
    let symbol = symbol.to_uppercase();
    let earnings_date = next_earnings_date(&symbol).await?;
    let days_until = (earnings_date - Utc::now().date_naive()).num_days();

    let chain = options::chain(&symbol, None).await?;
    // First expiration strictly after the earnings announcement.
    let exp = chain.expirations.iter().copied()
        .find(|d| *d > earnings_date)
        .ok_or_else(|| anyhow::anyhow!("no post-earnings expiration found"))?;
    let exp_chain = options::chain(&symbol, Some(exp)).await?;
    let ((call, call_mid), (put, put_mid), atm) = options::atm_straddle(&exp_chain)
        .ok_or_else(|| anyhow::anyhow!("no ATM straddle quoted"))?;
    let implied_move_pct = (call_mid + put_mid) / exp_chain.spot * 100.0;

    // Historical earnings — Yahoo earningsHistory returns up to ~4 quarters;
    // some symbols expose more. We then derive realized moves from price_bars.
    let v = market_data::earnings(&symbol).await.unwrap_or(serde_json::Value::Null);
    let hist_entries = v["earningsHistory"]["history"]
        .as_array().cloned().unwrap_or_default();
    let mut historical = Vec::new();
    for h in hist_entries.iter().take(8) {
        let d_raw = h["quarter"]["fmt"].as_str()
            .or_else(|| h["quarter"]["raw"].as_str());
        let d = match d_raw {
            Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d").ok(),
            None => h["quarter"]["raw"].as_i64()
                .and_then(|t| chrono::DateTime::from_timestamp(t, 0))
                .map(|d| d.date_naive()),
        };
        let Some(date) = d else { continue };
        if let Ok(Some(rm)) = realized_move(pool, &symbol, date).await {
            historical.push(rm);
        }
    }

    let realized: Vec<f64> = historical.iter().map(|h| h.abs_move_pct).collect();
    let bt = backtest(implied_move_pct, &realized);

    // Persist cache.
    let _ = sqlx::query(
        "INSERT INTO implied_moves
            (symbol, earnings_date, expiration, spot_price, atm_strike,
             call_mid, put_mid, implied_move_pct,
             avg_realized_pct, median_realized_pct, realized_sample_size, edge_pct,
             long_straddle_pnl, short_straddle_pnl,
             long_straddle_winrate, short_straddle_winrate)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
         ON CONFLICT (symbol, earnings_date) DO UPDATE SET
            expiration = EXCLUDED.expiration,
            spot_price = EXCLUDED.spot_price,
            atm_strike = EXCLUDED.atm_strike,
            call_mid = EXCLUDED.call_mid, put_mid = EXCLUDED.put_mid,
            implied_move_pct = EXCLUDED.implied_move_pct,
            avg_realized_pct = EXCLUDED.avg_realized_pct,
            median_realized_pct = EXCLUDED.median_realized_pct,
            realized_sample_size = EXCLUDED.realized_sample_size,
            edge_pct = EXCLUDED.edge_pct,
            long_straddle_pnl = EXCLUDED.long_straddle_pnl,
            short_straddle_pnl = EXCLUDED.short_straddle_pnl,
            long_straddle_winrate = EXCLUDED.long_straddle_winrate,
            short_straddle_winrate = EXCLUDED.short_straddle_winrate,
            computed_at = now()",
    )
    .bind(&symbol).bind(earnings_date).bind(exp)
    .bind(rust_decimal::Decimal::try_from(exp_chain.spot)?)
    .bind(rust_decimal::Decimal::try_from(atm)?)
    .bind(rust_decimal::Decimal::try_from(call_mid)?)
    .bind(rust_decimal::Decimal::try_from(put_mid)?)
    .bind(rust_decimal::Decimal::try_from(implied_move_pct)?)
    .bind(rust_decimal::Decimal::try_from(bt.avg_realized_pct).ok())
    .bind(rust_decimal::Decimal::try_from(bt.median_realized_pct).ok())
    .bind(bt.samples as i32)
    .bind(rust_decimal::Decimal::try_from(bt.edge_pct).ok())
    .bind(rust_decimal::Decimal::try_from(bt.long_avg_pnl).ok())
    .bind(rust_decimal::Decimal::try_from(bt.short_avg_pnl).ok())
    .bind(rust_decimal::Decimal::try_from(bt.long_win_rate).ok())
    .bind(rust_decimal::Decimal::try_from(bt.short_win_rate).ok())
    .execute(pool).await;

    let _ = (call, put); // future: surface contract greeks/OI in detail page

    Ok(EarningsIvReport {
        symbol,
        earnings_date,
        days_until,
        spot: exp_chain.spot,
        atm_strike: atm,
        expiration: exp,
        call_mid,
        put_mid,
        implied_move_pct,
        historical,
        backtest: bt,
    })
}

async fn next_earnings_date(symbol: &str) -> anyhow::Result<NaiveDate> {
    let v = market_data::earnings(symbol).await?;
    // Try calendarEvents.earnings.earningsDate[0]
    if let Some(ts) = v["calendarEvents"]["earnings"]["earningsDate"][0]["raw"].as_i64() {
        if let Some(d) = chrono::DateTime::from_timestamp(ts, 0) {
            return Ok(d.date_naive());
        }
    }
    if let Some(s) = v["calendarEvents"]["earnings"]["earningsDate"][0]["fmt"].as_str() {
        if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Ok(d);
        }
    }
    anyhow::bail!("no earningsDate published for {symbol}")
}

async fn realized_move(
    pool: &PgPool,
    symbol: &str,
    earnings_date: NaiveDate,
) -> anyhow::Result<Option<HistoricalMove>> {
    // Use a ±10-day window around the earnings date to find the immediate
    // pre and post closes from price_bars.
    let to_ts   = earnings_date.and_hms_opt(0, 0, 0).unwrap().and_utc() + Duration::days(10);
    let from_ts = earnings_date.and_hms_opt(0, 0, 0).unwrap().and_utc() - Duration::days(10);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from_ts, to_ts).await?;
    if bars.len() < 2 { return Ok(None); }
    let pre  = bars.iter().filter(|b| b.bar_time.date_naive() < earnings_date).last();
    let post = bars.iter().find(|b| b.bar_time.date_naive() > earnings_date);
    if let (Some(p), Some(n)) = (pre, post) {
        let cb = dec(p.close);
        let ca = dec(n.close);
        if cb > 0.0 {
            let mv = (ca - cb) / cb * 100.0;
            let abs_pct = mv.abs();
            // Persist for fast re-use.
            let _ = sqlx::query(
                "INSERT INTO realized_earnings_moves
                    (symbol, earnings_date, close_before, close_after, abs_move_pct, direction)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (symbol, earnings_date) DO UPDATE SET
                    close_before = EXCLUDED.close_before, close_after = EXCLUDED.close_after,
                    abs_move_pct = EXCLUDED.abs_move_pct, direction = EXCLUDED.direction",
            )
            .bind(symbol).bind(earnings_date)
            .bind(rust_decimal::Decimal::try_from(cb).ok())
            .bind(rust_decimal::Decimal::try_from(ca).ok())
            .bind(rust_decimal::Decimal::try_from(abs_pct).ok())
            .bind(if mv >= 0.0 { "up" } else { "down" })
            .execute(pool).await;
            return Ok(Some(HistoricalMove {
                earnings_date,
                close_before: cb,
                close_after: ca,
                abs_move_pct: abs_pct,
                direction: if mv >= 0.0 { "up" } else { "down" },
            }));
        }
    }
    Ok(None)
}

fn dec(d: rust_decimal::Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }

// ===========================================================================
// Scanner: rank all symbols in a universe by edge.
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct EarningsIvHit {
    pub symbol: String,
    pub earnings_date: NaiveDate,
    pub days_until: i64,
    pub implied_move_pct: f64,
    pub median_realized_pct: f64,
    pub edge_pct: f64,
    pub samples: usize,
    pub recommendation: &'static str,
    pub long_avg_pnl: f64,
    pub short_avg_pnl: f64,
}

pub async fn scan(
    pool: &PgPool,
    user_id: uuid::Uuid,
    watchlist_id: Option<uuid::Uuid>,
    horizon_days: i64,
    limit: usize,
) -> anyhow::Result<Vec<EarningsIvHit>> {
    use std::collections::BTreeSet;
    let universe: Vec<String> = if let Some(wid) = watchlist_id {
        if !crate::watchlists::ensure_owner(pool, user_id, wid).await? {
            anyhow::bail!("forbidden");
        }
        crate::watchlists::symbols(pool, wid).await?
    } else {
        let mut all = BTreeSet::new();
        for w in crate::watchlists::list(pool, user_id).await? {
            for s in crate::watchlists::symbols(pool, w.id).await? { all.insert(s); }
        }
        all.into_iter().collect()
    };

    let today = Utc::now().date_naive();
    let mut hits = Vec::new();
    for sym in universe {
        match report(pool, &sym).await {
            Ok(r) => {
                let du = (r.earnings_date - today).num_days();
                if du < 0 || du > horizon_days { continue; }
                hits.push(EarningsIvHit {
                    symbol: r.symbol,
                    earnings_date: r.earnings_date,
                    days_until: du,
                    implied_move_pct: r.implied_move_pct,
                    median_realized_pct: r.backtest.median_realized_pct,
                    edge_pct: r.backtest.edge_pct,
                    samples: r.backtest.samples,
                    recommendation: r.backtest.recommendation,
                    long_avg_pnl: r.backtest.long_avg_pnl,
                    short_avg_pnl: r.backtest.short_avg_pnl,
                });
            }
            Err(e) => tracing::debug!(symbol = %sym, error = %e, "iv-scan skip"),
        }
    }
    hits.sort_by(|a, b| b.edge_pct.abs().partial_cmp(&a.edge_pct.abs())
        .unwrap_or(std::cmp::Ordering::Equal));
    hits.truncate(limit);
    Ok(hits)
}
