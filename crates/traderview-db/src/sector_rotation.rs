//! Sector ETF rotation heatmap.
//!
//! For each of the 11 SPDR sectors and the benchmark SPY, compute the
//! cumulative % return over 5/20/60-day windows. RS = sector_return -
//! spy_return (in percentage points). Ranks 1..11 across each window. A
//! sector that's leading (top rank) across all three windows is in a
//! durable rotation; rotating leadership shows up as rank flips.
//!
//! Sparkline: per-day RS (sector minus SPY return) over the last 60 sessions.

use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

pub const SECTORS: &[(&str, &str)] = &[
    ("XLK", "Technology"),
    ("XLF", "Financials"),
    ("XLE", "Energy"),
    ("XLV", "Healthcare"),
    ("XLY", "Consumer Discretionary"),
    ("XLP", "Consumer Staples"),
    ("XLI", "Industrials"),
    ("XLB", "Materials"),
    ("XLU", "Utilities"),
    ("XLRE", "Real Estate"),
    ("XLC", "Communications"),
];

const BENCHMARK: &str = "SPY";
const WINDOWS: &[(i32, &str)] = &[(5, "5d"), (20, "20d"), (60, "60d")];

#[derive(Debug, Clone, Serialize)]
pub struct SectorRow {
    pub symbol: String,
    pub label: &'static str,
    /// length = WINDOWS.len(); each cell: { return_pct, rs_pct, rank }
    pub windows: Vec<WindowCell>,
    /// Daily RS sparkline (sector_daily_return − spy_daily_return), 60 days.
    pub rs_sparkline: Vec<f64>,
    pub bars_loaded: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowCell {
    pub label: &'static str,
    pub return_pct: Option<f64>,
    pub rs_pct: Option<f64>,
    pub rank: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotationReport {
    pub windows: Vec<&'static str>,
    pub sectors: Vec<SectorRow>,
    pub spy_returns: Vec<f64>,                  // [5d, 20d, 60d]
    pub leadership_by_window: Vec<Vec<String>>, // top-3 per window
    pub computed_at: chrono::DateTime<Utc>,
}

pub async fn report(pool: &PgPool) -> anyhow::Result<RotationReport> {
    let to = Utc::now();
    let from = to - Duration::days(120);

    // Pull SPY first so we can compute RS relative to it.
    let spy_closes = closes_for(pool, BENCHMARK, from, to).await;
    let spy_returns: Vec<f64> = WINDOWS
        .iter()
        .map(|(n, _)| return_over(&spy_closes, *n as usize))
        .map(|o| o.unwrap_or(0.0))
        .collect();
    let spy_daily = daily_returns(&spy_closes);

    // Per-sector returns + sparklines.
    let mut sectors: Vec<SectorRow> = Vec::with_capacity(SECTORS.len());
    for (sym, label) in SECTORS {
        let cls = closes_for(pool, sym, from, to).await;
        let bars_loaded = cls.len();
        let mut windows = Vec::with_capacity(WINDOWS.len());
        for (i, (n, lbl)) in WINDOWS.iter().enumerate() {
            let ret = return_over(&cls, *n as usize);
            let rs = ret.map(|r| r - spy_returns[i]);
            windows.push(WindowCell {
                label: lbl,
                return_pct: ret,
                rs_pct: rs,
                rank: None,
            });
        }
        let s_daily = daily_returns(&cls);
        let n = s_daily.len().min(spy_daily.len()).min(60);
        let rs_sparkline: Vec<f64> = (0..n)
            .map(|i| {
                let si = s_daily.len() - n + i;
                let pi = spy_daily.len() - n + i;
                s_daily[si] - spy_daily[pi]
            })
            .collect();
        sectors.push(SectorRow {
            symbol: sym.to_string(),
            label,
            windows,
            rs_sparkline,
            bars_loaded,
        });
    }

    // Compute ranks: for each window, sort sectors by rs_pct DESC and assign 1..N.
    for w_idx in 0..WINDOWS.len() {
        let mut idx_rs: Vec<(usize, f64)> = sectors
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.windows[w_idx].rs_pct.map(|v| (i, v)))
            .collect();
        idx_rs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (rank, (i, _)) in idx_rs.iter().enumerate() {
            sectors[*i].windows[w_idx].rank = Some((rank + 1) as i32);
        }
    }

    // Leadership boards per window: top 3 by rank.
    let mut leadership_by_window = Vec::with_capacity(WINDOWS.len());
    for w_idx in 0..WINDOWS.len() {
        let mut ranked: Vec<&SectorRow> = sectors
            .iter()
            .filter(|s| s.windows[w_idx].rank.is_some())
            .collect();
        ranked.sort_by_key(|s| s.windows[w_idx].rank.unwrap_or(99));
        leadership_by_window.push(ranked.iter().take(3).map(|s| s.symbol.clone()).collect());
    }

    Ok(RotationReport {
        windows: WINDOWS.iter().map(|(_, l)| *l).collect(),
        sectors,
        spy_returns,
        leadership_by_window,
        computed_at: Utc::now(),
    })
}

async fn closes_for(
    pool: &PgPool,
    sym: &str,
    from: chrono::DateTime<Utc>,
    to: chrono::DateTime<Utc>,
) -> Vec<f64> {
    crate::prices::get_bars(pool, sym, BarInterval::D1, from, to)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|b| b.close.to_string().parse().unwrap_or(0.0))
        .collect()
}

fn return_over(closes: &[f64], n: usize) -> Option<f64> {
    if closes.len() <= n {
        return None;
    }
    let last = *closes.last()?;
    let prior = closes[closes.len() - 1 - n];
    if prior <= 0.0 {
        return None;
    }
    Some((last - prior) / prior * 100.0)
}

fn daily_returns(closes: &[f64]) -> Vec<f64> {
    if closes.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(closes.len() - 1);
    for i in 1..closes.len() {
        let prev = closes[i - 1];
        if prev > 0.0 {
            out.push((closes[i] - prev) / prev * 100.0);
        } else {
            out.push(0.0);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================================================
    // return_over — N-bar lookback percent change
    // ===========================================================================

    #[test]
    fn return_over_returns_none_when_window_exceeds_length() {
        // len = 3, n = 3 → need at least 4 closes (n+1). Returns None.
        let closes = vec![100.0, 105.0, 110.0];
        assert!(return_over(&closes, 3).is_none());
    }

    #[test]
    fn return_over_returns_none_when_window_equals_length_minus_one() {
        // closes.len() == n means closes[len-1-n] is out of range. Guard: <= n.
        let closes = vec![100.0, 110.0];
        assert!(return_over(&closes, 2).is_none());
        // n=1, len=2: 2 <= 1 is false → returns Some.
        let r = return_over(&closes, 1).unwrap();
        assert!((r - 10.0).abs() < 1e-9);
    }

    #[test]
    fn return_over_computes_pct_change_from_n_bars_ago() {
        // closes[last] = 121, closes[last-2] = 100 → return = 21%.
        let closes = vec![100.0, 110.0, 121.0];
        let r = return_over(&closes, 2).unwrap();
        assert!((r - 21.0).abs() < 1e-9);
    }

    #[test]
    fn return_over_handles_negative_returns() {
        // 100 → 90 over 1 bar = -10%.
        let closes = vec![100.0, 90.0];
        let r = return_over(&closes, 1).unwrap();
        assert!((r - (-10.0)).abs() < 1e-9);
    }

    #[test]
    fn return_over_returns_none_when_prior_is_zero_or_negative() {
        // Prior = 0 → undefined % return.
        let closes = vec![0.0, 100.0];
        assert!(return_over(&closes, 1).is_none());
        let neg = vec![-5.0, 100.0];
        assert!(return_over(&neg, 1).is_none());
    }

    #[test]
    fn return_over_empty_input_returns_none() {
        let closes: Vec<f64> = vec![];
        assert!(return_over(&closes, 0).is_none());
    }

    #[test]
    fn return_over_zero_window_returns_none() {
        // len=1, n=0 → 1 <= 0 false → Some. But last - last = 0%.
        let closes = vec![100.0];
        let r = return_over(&closes, 0).unwrap();
        assert_eq!(r, 0.0);
    }

    // ===========================================================================
    // daily_returns — bar-to-bar percent change vector
    // ===========================================================================

    #[test]
    fn daily_returns_empty_when_fewer_than_two_closes() {
        assert!(daily_returns(&[]).is_empty());
        assert!(daily_returns(&[100.0]).is_empty());
    }

    #[test]
    fn daily_returns_produces_n_minus_one_outputs() {
        let closes = vec![100.0, 110.0, 121.0, 100.0];
        let r = daily_returns(&closes);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn daily_returns_values_are_pct_change() {
        // 100→110 = +10%, 110→121 = +10%, 121→121 = 0%.
        let closes = vec![100.0, 110.0, 121.0, 121.0];
        let r = daily_returns(&closes);
        assert!((r[0] - 10.0).abs() < 1e-9);
        assert!((r[1] - 10.0).abs() < 1e-9);
        assert!((r[2] - 0.0).abs() < 1e-9);
    }

    #[test]
    fn daily_returns_negative_moves_preserved() {
        let closes = vec![100.0, 95.0, 90.25];
        let r = daily_returns(&closes);
        assert!((r[0] - (-5.0)).abs() < 1e-9);
        assert!((r[1] - (-5.0)).abs() < 1e-9);
    }

    #[test]
    fn daily_returns_zero_or_negative_prev_yields_zero() {
        // prev <= 0 short-circuits to 0.0 to avoid div-by-zero / sign flip.
        let closes = vec![0.0, 100.0, 110.0];
        let r = daily_returns(&closes);
        assert_eq!(r[0], 0.0);
        assert!((r[1] - 10.0).abs() < 1e-9);
    }

    #[test]
    fn daily_returns_handles_negative_prev() {
        let closes = vec![-1.0, 100.0];
        let r = daily_returns(&closes);
        // -1.0 <= 0 → guarded to 0.
        assert_eq!(r[0], 0.0);
    }
}
