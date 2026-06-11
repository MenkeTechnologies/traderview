//! Gap statistics per symbol.
//!
//! A gap is `open_t` vs `close_{t-1}` beyond a threshold (default
//! 0.5%). For each gap we measure whether the gap "filled" — price
//! traded back to the prior close — same-day or within the next 5
//! sessions, and how many sessions it took. Day traders use the fill
//! rate to decide whether to fade a gap or go with it; the per-symbol
//! base rates differ enormously (index ETFs fill >70% same-day; runaway
//! small caps often never fill).
//!
//! Data source: existing `prices::get_bars` daily cache — no new
//! provider dependency.

use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

const DEFAULT_GAP_THRESHOLD_PCT: f64 = 0.5;
const FILL_WINDOW_SESSIONS: usize = 5;
const LOOKBACK_DAYS: i64 = 400;
const MIN_BARS: usize = 40;

#[derive(Debug, Clone, Serialize)]
pub struct GapEvent {
    pub date: chrono::NaiveDate,
    /// Positive = gap up, negative = gap down. Percent of prior close.
    pub gap_pct: f64,
    pub prior_close: f64,
    pub open: f64,
    /// Filled the same session (price touched prior close intraday).
    pub filled_same_day: bool,
    /// Filled within FILL_WINDOW_SESSIONS sessions (incl. same day).
    pub filled_in_window: bool,
    /// Sessions until fill (0 = same day). None = never within window.
    pub sessions_to_fill: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GapSideStats {
    pub count: usize,
    pub avg_gap_pct: f64,
    pub same_day_fill_rate_pct: f64,
    pub window_fill_rate_pct: f64,
    pub avg_sessions_to_fill: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GapReport {
    pub symbol: String,
    pub threshold_pct: f64,
    pub bars_analyzed: usize,
    pub up: GapSideStats,
    pub down: GapSideStats,
    /// Most recent 30 gap events for the UI table.
    pub recent: Vec<GapEvent>,
}

#[derive(Debug, thiserror::Error)]
pub enum GapError {
    #[error("not enough bars for {symbol}: got {got}, need {need}")]
    Insufficient {
        symbol: String,
        got: usize,
        need: usize,
    },
    #[error("price fetch failed: {0}")]
    PriceFetch(anyhow::Error),
}

pub async fn compute(
    pool: &PgPool,
    symbol: &str,
    threshold_pct: Option<f64>,
) -> Result<GapReport, GapError> {
    let threshold = threshold_pct.unwrap_or(DEFAULT_GAP_THRESHOLD_PCT).max(0.05);
    let to = Utc::now();
    let from = to - Duration::days(LOOKBACK_DAYS);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .map_err(GapError::PriceFetch)?;
    let n = bars.len();
    if n < MIN_BARS {
        return Err(GapError::Insufficient {
            symbol: symbol.to_string(),
            got: n,
            need: MIN_BARS,
        });
    }
    let f = |d: rust_decimal::Decimal| -> f64 { d.to_string().parse().unwrap_or(0.0) };

    let mut events: Vec<GapEvent> = Vec::new();
    for i in 1..n {
        let prior_close = f(bars[i - 1].close);
        let open = f(bars[i].open);
        if prior_close <= 0.0 {
            continue;
        }
        let gap_pct = (open - prior_close) / prior_close * 100.0;
        if gap_pct.abs() < threshold {
            continue;
        }
        // Fill check: did price trade back to prior_close?
        // Same-day: bar i's [low, high] straddles prior_close.
        let same_day =
            f(bars[i].low) <= prior_close && prior_close <= f(bars[i].high);
        let mut sessions_to_fill: Option<u32> = if same_day { Some(0) } else { None };
        if sessions_to_fill.is_none() {
            for (k, bar) in bars
                .iter()
                .enumerate()
                .skip(i + 1)
                .take(FILL_WINDOW_SESSIONS)
            {
                if f(bar.low) <= prior_close && prior_close <= f(bar.high) {
                    sessions_to_fill = Some((k - i) as u32);
                    break;
                }
            }
        }
        events.push(GapEvent {
            date: bars[i].bar_time.date_naive(),
            gap_pct,
            prior_close,
            open,
            filled_same_day: same_day,
            filled_in_window: sessions_to_fill.is_some(),
            sessions_to_fill,
        });
    }

    let side = |up: bool| -> GapSideStats {
        let evs: Vec<&GapEvent> = events
            .iter()
            .filter(|e| (e.gap_pct > 0.0) == up)
            .collect();
        let count = evs.len();
        if count == 0 {
            return GapSideStats::default();
        }
        let avg_gap = evs.iter().map(|e| e.gap_pct.abs()).sum::<f64>() / count as f64;
        let same = evs.iter().filter(|e| e.filled_same_day).count();
        let win = evs.iter().filter(|e| e.filled_in_window).count();
        let fills: Vec<u32> = evs.iter().filter_map(|e| e.sessions_to_fill).collect();
        let avg_fill = if fills.is_empty() {
            0.0
        } else {
            fills.iter().map(|&x| x as f64).sum::<f64>() / fills.len() as f64
        };
        GapSideStats {
            count,
            avg_gap_pct: avg_gap,
            same_day_fill_rate_pct: same as f64 / count as f64 * 100.0,
            window_fill_rate_pct: win as f64 / count as f64 * 100.0,
            avg_sessions_to_fill: avg_fill,
        }
    };
    let up = side(true);
    let down = side(false);
    let recent: Vec<GapEvent> = events.iter().rev().take(30).cloned().collect();
    Ok(GapReport {
        symbol: symbol.to_string(),
        threshold_pct: threshold,
        bars_analyzed: n,
        up,
        down,
        recent,
    })
}
