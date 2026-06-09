//! Post-earnings-announcement drift (PEAD) tracker.
//!
//! PEAD is the most-replicated anomaly in equities research: stocks
//! that beat consensus EPS estimates by a meaningful margin tend to
//! drift upward for ~60 trading days after the announcement, while
//! stocks that miss drift downward. The original observation goes back
//! to Ball & Brown 1968; updated factor research (e.g. Chordia &
//! Shivakumar 2006, Hung & Hung 2024 follow-ups) still finds the
//! effect intact, just attenuated in highly-liquid mega-caps.
//!
//! Implementation:
//!
//!   1. Pull `earnings_events` rows with `surprise_pct` present and
//!      `earnings_date` in the trailing N days (default 90).
//!   2. For each, fetch daily bars over `[earnings_date - 1d,
//!      earnings_date + 65d]` from `price_bars` (the existing daily-
//!      bar cache; `prices::get_bars` populates it from Yahoo).
//!   3. Compute returns at the announcement-day close, +5d, +20d,
//!      +60d (trading days, approximated as 5/20/60 calendar days
//!      forward — close enough for ranking purposes and avoids a
//!      separate trading-day calendar).
//!   4. Score = signed-drift quality:
//!         score = sign(surprise) · (return_20d - return_day0)
//!      where the "post-day-0" piece is what PEAD actually claims.
//!      A positive score means the surprise direction was followed by
//!      drift in the same direction over the next ~month.
//!
//! Output rows are stateless — every call recomputes. Caching is
//! left to the route layer (HTTP).

use chrono::{DateTime, Duration, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

use crate::{earnings_cal, prices};

/// Minimum absolute surprise % to include a row. Below ~2% the EPS
/// surprise is in the noise of analyst estimate dispersion.
const MIN_ABS_SURPRISE_PCT: f64 = 2.0;

#[derive(Debug, Clone, Serialize)]
pub struct PeadRow {
    pub symbol: String,
    pub earnings_date: chrono::NaiveDate,
    pub surprise_pct: f64,
    /// % return from the bar BEFORE earnings to the bar ON earnings
    /// (announcement-day reaction).
    pub return_day0_pct: Option<f64>,
    /// % return from announcement bar to bar ~5 trading days later.
    pub return_5d_pct: Option<f64>,
    /// % return from announcement bar to bar ~20 trading days later.
    pub return_20d_pct: Option<f64>,
    /// % return from announcement bar to bar ~60 trading days later.
    pub return_60d_pct: Option<f64>,
    /// Composite signal: positive when surprise direction matches the
    /// 20-day post-announcement drift direction. Higher = better.
    pub score: Option<f64>,
}

/// Pure: compute the PEAD row given a surprise and a chronological
/// vector of (date, close) pairs. Caller is responsible for fetching
/// the bars and assembling the inputs.
pub fn compute_row(
    symbol: &str,
    earnings_date: chrono::NaiveDate,
    surprise_pct: f64,
    bars: &[(chrono::NaiveDate, f64)],
) -> PeadRow {
    let pre_close = bars
        .iter()
        .rfind(|(d, _)| *d < earnings_date)
        .map(|(_, c)| *c);
    let day0_close = bars
        .iter()
        .find(|(d, _)| *d >= earnings_date)
        .map(|(_, c)| *c);
    let close_at_or_after = |target: chrono::NaiveDate| -> Option<f64> {
        bars.iter().find(|(d, _)| *d >= target).map(|(_, c)| *c)
    };
    let return_day0_pct = match (pre_close, day0_close) {
        (Some(p), Some(c)) if p > 0.0 => Some((c - p) / p * 100.0),
        _ => None,
    };
    let pct_after = |target: chrono::NaiveDate| -> Option<f64> {
        match (day0_close, close_at_or_after(target)) {
            (Some(d0), Some(c)) if d0 > 0.0 => Some((c - d0) / d0 * 100.0),
            _ => None,
        }
    };
    let return_5d_pct = pct_after(earnings_date + Duration::days(7));
    let return_20d_pct = pct_after(earnings_date + Duration::days(28));
    let return_60d_pct = pct_after(earnings_date + Duration::days(84));
    let score = match return_20d_pct {
        Some(r20) => Some(surprise_pct.signum() * r20),
        None => None,
    };
    PeadRow {
        symbol: symbol.into(),
        earnings_date,
        surprise_pct,
        return_day0_pct,
        return_5d_pct,
        return_20d_pct,
        return_60d_pct,
        score,
    }
}

/// Repository function: pulls recent earnings + bar history and
/// computes PEAD rows for everything with a meaningful surprise.
/// Returns rows newest-first (most-recent earnings first).
pub async fn recent(pool: &PgPool, days: i64, limit: usize) -> anyhow::Result<Vec<PeadRow>> {
    let events = earnings_cal::surprises_recent(pool, days).await?;
    let mut out: Vec<PeadRow> = Vec::new();
    for ev in events {
        let surprise = match ev.surprise_pct {
            Some(p) => p as f64,
            None => continue,
        };
        if surprise.abs() < MIN_ABS_SURPRISE_PCT {
            continue;
        }
        let from = (ev.earnings_date - Duration::days(2))
            .and_hms_opt(0, 0, 0)
            .map(|n| DateTime::<Utc>::from_naive_utc_and_offset(n, Utc));
        let to = (ev.earnings_date + Duration::days(95))
            .and_hms_opt(23, 59, 59)
            .map(|n| DateTime::<Utc>::from_naive_utc_and_offset(n, Utc));
        let (Some(from), Some(to)) = (from, to) else {
            continue;
        };
        let bars = match prices::get_bars(pool, &ev.symbol, BarInterval::D1, from, to).await {
            Ok(b) => b,
            Err(e) => {
                tracing::debug!(?e, symbol = %ev.symbol, "pead: bar fetch failed");
                continue;
            }
        };
        let series: Vec<(chrono::NaiveDate, f64)> = bars
            .into_iter()
            .filter_map(|b| {
                let d = b.bar_time.date_naive();
                b.close.to_f64().map(|c| (d, c))
            })
            .collect();
        if series.is_empty() {
            continue;
        }
        out.push(compute_row(&ev.symbol, ev.earnings_date, surprise, &series));
        if out.len() >= limit {
            break;
        }
    }
    out.sort_by(|a, b| {
        b.earnings_date.cmp(&a.earnings_date).then_with(|| {
            b.score
                .unwrap_or(f64::NEG_INFINITY)
                .partial_cmp(&a.score.unwrap_or(f64::NEG_INFINITY))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    Ok(out)
}

/// Top-N by PEAD score (drift in surprise-aligned direction).
pub async fn top_drift(pool: &PgPool, days: i64, limit: usize) -> anyhow::Result<Vec<PeadRow>> {
    let mut rows = recent(pool, days, 200).await?;
    rows.retain(|r| r.score.is_some());
    rows.sort_by(|a, b| {
        b.score
            .unwrap_or(0.0)
            .partial_cmp(&a.score.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows.truncate(limit);
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn compute_row_returns_all_horizons_when_data_present() {
        let earnings = d(2026, 1, 15);
        let bars: Vec<(NaiveDate, f64)> = vec![
            (d(2026, 1, 14), 100.0), // day -1, pre-close
            (d(2026, 1, 15), 105.0), // day 0  (+5% day-of)
            (d(2026, 1, 22), 110.0), // day +7 → maps to +5d horizon
            (d(2026, 2, 12), 120.0), // day +28 → maps to +20d
            (d(2026, 4, 9), 130.0),  // day +84 → maps to +60d
        ];
        let r = compute_row("AAA", earnings, 8.0, &bars);
        assert!((r.return_day0_pct.unwrap() - 5.0).abs() < 1e-9);
        // (110 - 105) / 105 ≈ 4.76%
        assert!((r.return_5d_pct.unwrap() - 4.761_904_761_904_762).abs() < 1e-9);
        // (120 - 105) / 105 ≈ 14.29%
        assert!((r.return_20d_pct.unwrap() - 14.285_714_285_714_286).abs() < 1e-9);
        // (130 - 105) / 105 ≈ 23.81%
        assert!((r.return_60d_pct.unwrap() - 23.809_523_809_523_81).abs() < 1e-9);
        // surprise_pct = +8, return_20d = +14.29, score = +14.29
        assert!((r.score.unwrap() - 14.285_714_285_714_286).abs() < 1e-9);
    }

    #[test]
    fn compute_row_score_negative_when_drift_opposes_surprise() {
        // Positive surprise but stock keeps falling → PEAD failure.
        let earnings = d(2026, 1, 15);
        let bars = vec![
            (d(2026, 1, 14), 100.0),
            (d(2026, 1, 15), 95.0),
            (d(2026, 2, 12), 85.0), // -10.5% by +20d
        ];
        let r = compute_row("BBB", earnings, 8.0, &bars);
        assert!(r.return_day0_pct.unwrap() < 0.0);
        assert!(r.return_20d_pct.unwrap() < 0.0);
        // sign(+8) · (-10.5%) → negative score
        assert!(r.score.unwrap() < 0.0);
    }

    #[test]
    fn compute_row_score_positive_when_negative_surprise_drifts_down() {
        // Miss + drift down → still a valid PEAD signal (short setup).
        let earnings = d(2026, 1, 15);
        let bars = vec![
            (d(2026, 1, 14), 100.0),
            (d(2026, 1, 15), 95.0),
            (d(2026, 2, 12), 85.0),
        ];
        let r = compute_row("CCC", earnings, -8.0, &bars);
        // sign(-8) · (-10.5%) = +10.5 (PEAD working in short direction)
        assert!(r.score.unwrap() > 0.0);
    }

    #[test]
    fn compute_row_handles_missing_horizons() {
        let earnings = d(2026, 1, 15);
        // Only day -1 and day 0 — no future bars.
        let bars = vec![(d(2026, 1, 14), 100.0), (d(2026, 1, 15), 105.0)];
        let r = compute_row("DDD", earnings, 5.0, &bars);
        assert!(r.return_day0_pct.is_some());
        assert!(r.return_5d_pct.is_none());
        assert!(r.return_20d_pct.is_none());
        assert!(r.return_60d_pct.is_none());
        assert!(r.score.is_none());
    }

    #[test]
    fn compute_row_handles_no_pre_close() {
        // No bar before earnings (rare but possible for IPO-near-earnings).
        let earnings = d(2026, 1, 15);
        let bars = vec![(d(2026, 1, 15), 105.0), (d(2026, 2, 12), 120.0)];
        let r = compute_row("EEE", earnings, 5.0, &bars);
        assert!(r.return_day0_pct.is_none());
        // Day0 → +20d return still computable.
        assert!(r.return_20d_pct.is_some());
    }

    #[test]
    fn compute_row_handles_no_day0_bar() {
        // No bar on or after earnings date — entire post-event series missing.
        let earnings = d(2026, 1, 15);
        let bars = vec![(d(2026, 1, 14), 100.0)];
        let r = compute_row("FFF", earnings, 5.0, &bars);
        assert!(r.return_day0_pct.is_none());
        assert!(r.return_5d_pct.is_none());
        assert!(r.return_20d_pct.is_none());
        assert!(r.score.is_none());
    }

    #[test]
    fn compute_row_falls_through_to_next_available_bar_for_horizons() {
        // Sparse data — +20d target falls on a non-trading day; the
        // detector should pick the next available bar.
        let earnings = d(2026, 1, 15);
        let bars = vec![
            (d(2026, 1, 14), 100.0),
            (d(2026, 1, 15), 105.0),
            // Gap; target +28d = 2026-02-12 has no bar; next bar is +30d.
            (d(2026, 2, 14), 130.0),
        ];
        let r = compute_row("GGG", earnings, 5.0, &bars);
        assert!(r.return_20d_pct.is_some());
        // (130-105)/105 ≈ 23.8%
        assert!((r.return_20d_pct.unwrap() - 23.809_523_809_523_81).abs() < 1e-9);
    }
}
