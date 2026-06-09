//! Scanner signal backtest framework.
//!
//! Twenty-five scanners exist in this stack; nobody has ranked them
//! against each other on actual P&L. This module fills that gap with
//! the simplest defensible procedure:
//!
//!   1. Each scanner adapter produces a list of `(symbol, signal_date,
//!      direction)` tuples over a historical window — direction is
//!      `Long` (the signal predicts up), `Short` (down), or `Neutral`
//!      (the signal predicts vol expansion without sign — backtested
//!      as absolute-value performance, not implemented here yet).
//!   2. For each tuple, the framework reads `price_bars` (D1 cache),
//!      computes forward log-returns at 1d / 5d / 20d / 60d from the
//!      signal date's close.
//!   3. Aggregates per horizon: `n`, `hit_rate` (% positive in the
//!      signal's direction), `mean_pct`, `median_pct`, `stdev_pct`,
//!      `sharpe` (mean / stdev × √(252 / horizon)), `max_dd_pct`.
//!
//! Pure compute is fully unit-tested. The repository adapters wrap
//! existing scanner logic + bar fetches so the framework can score
//! historical scans without re-running real-time consumers.

use chrono::{Duration, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

use crate::prices;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Long,
    Short,
}

impl Direction {
    pub fn sign(self) -> f64 {
        match self {
            Direction::Long => 1.0,
            Direction::Short => -1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalSample {
    pub symbol: String,
    pub signal_date: NaiveDate,
    pub direction: Direction,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct HorizonStats {
    pub horizon_days: u32,
    pub n: usize,
    pub hit_rate_pct: f64,
    pub mean_return_pct: f64,
    pub median_return_pct: f64,
    pub stdev_pct: f64,
    pub annualised_sharpe: f64,
    pub max_drawdown_pct: f64,
    /// Sum of per-signal direction-adjusted log returns. Useful as a
    /// rough "what would I have made putting $1 on every signal."
    pub total_logret_signed: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BacktestResult {
    pub scanner: String,
    pub samples_used: usize,
    pub horizons: Vec<HorizonStats>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Per-signal forward log-return in percent, signed for direction.
/// `None` when either the signal-day close or the forward close is
/// missing from the supplied bar series (caller's responsibility to
/// pass enough history).
pub fn signed_return_pct(
    closes: &[(NaiveDate, f64)],
    signal_date: NaiveDate,
    direction: Direction,
    horizon_days: u32,
) -> Option<f64> {
    let signal_close = closes
        .iter()
        .find(|(d, _)| *d >= signal_date)
        .map(|(_, c)| *c)?;
    if signal_close <= 0.0 {
        return None;
    }
    let target = signal_date + Duration::days(horizon_days as i64);
    let forward_close = closes.iter().find(|(d, _)| *d >= target).map(|(_, c)| *c)?;
    if forward_close <= 0.0 {
        return None;
    }
    let raw_log = (forward_close / signal_close).ln();
    Some(raw_log * 100.0 * direction.sign())
}

/// Aggregate per-signal returns into a horizon stat row. Empty input
/// produces a zero-filled row with `n = 0` so the UI can still render
/// the column.
pub fn aggregate(returns: &[f64], horizon_days: u32) -> HorizonStats {
    let n = returns.len();
    if n == 0 {
        return HorizonStats {
            horizon_days,
            ..Default::default()
        };
    }
    let hits = returns.iter().filter(|r| **r > 0.0).count();
    let hit_rate_pct = hits as f64 / n as f64 * 100.0;
    let mean = returns.iter().sum::<f64>() / n as f64;
    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };
    let var = if n > 1 {
        returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1) as f64
    } else {
        0.0
    };
    let stdev = var.sqrt();
    let annualised_sharpe = if stdev > 0.0 && horizon_days > 0 {
        mean / stdev * (252.0 / horizon_days as f64).sqrt()
    } else {
        0.0
    };
    let mut cum = 0.0_f64;
    let mut peak = f64::NEG_INFINITY;
    let mut max_dd = 0.0_f64;
    let mut total_logret = 0.0_f64;
    for r in returns {
        cum += r;
        total_logret += r / 100.0;
        if cum > peak {
            peak = cum;
        }
        let dd = peak - cum;
        if dd > max_dd {
            max_dd = dd;
        }
    }
    HorizonStats {
        horizon_days,
        n,
        hit_rate_pct,
        mean_return_pct: mean,
        median_return_pct: median,
        stdev_pct: stdev,
        annualised_sharpe,
        max_drawdown_pct: max_dd,
        total_logret_signed: total_logret,
    }
}

const DEFAULT_HORIZONS_DAYS: &[u32] = &[1, 5, 20, 60];

/// Full per-scanner backtest: takes a flat sample list + per-symbol
/// price history and produces a `BacktestResult` over the default
/// horizons. Caller is responsible for assembling the (closes per
/// symbol) lookup; the repository layer below does that from
/// `prices::get_bars`.
pub fn backtest_with_history(
    scanner: &str,
    samples: &[SignalSample],
    closes_for: &dyn Fn(&str) -> Vec<(NaiveDate, f64)>,
) -> BacktestResult {
    let mut horizon_returns: Vec<Vec<f64>> =
        vec![Vec::with_capacity(samples.len()); DEFAULT_HORIZONS_DAYS.len()];
    let mut samples_used = 0;
    for s in samples {
        let closes = closes_for(&s.symbol);
        if closes.is_empty() {
            continue;
        }
        let mut counted_this_sample = false;
        for (i, h) in DEFAULT_HORIZONS_DAYS.iter().enumerate() {
            if let Some(r) = signed_return_pct(&closes, s.signal_date, s.direction, *h) {
                horizon_returns[i].push(r);
                counted_this_sample = true;
            }
        }
        if counted_this_sample {
            samples_used += 1;
        }
    }
    let horizons: Vec<HorizonStats> = DEFAULT_HORIZONS_DAYS
        .iter()
        .enumerate()
        .map(|(i, h)| aggregate(&horizon_returns[i], *h))
        .collect();
    BacktestResult {
        scanner: scanner.into(),
        samples_used,
        horizons,
    }
}

// ─── Repository ────────────────────────────────────────────────────────────

/// PEAD backtest: every earnings event with |surprise_pct| ≥ 2% over
/// the trailing `days` days becomes a sample, direction = sign of
/// surprise. Uses the existing `earnings_events` table — the same
/// source the live PEAD tracker drives off of.
pub async fn backtest_pead(pool: &PgPool, days: i64) -> anyhow::Result<BacktestResult> {
    let events = crate::earnings_cal::surprises_recent(pool, days).await?;
    let mut samples: Vec<SignalSample> = Vec::new();
    for ev in &events {
        let Some(sur) = ev.surprise_pct else { continue };
        if (sur as f64).abs() < 2.0 {
            continue;
        }
        let direction = if (sur as f64) >= 0.0 {
            Direction::Long
        } else {
            Direction::Short
        };
        samples.push(SignalSample {
            symbol: ev.symbol.clone(),
            signal_date: ev.earnings_date,
            direction,
        });
    }
    let pool_ref = pool.clone();
    let closes_async = |symbol: &str| -> Vec<(NaiveDate, f64)> {
        let symbol = symbol.to_string();
        let pool_ref = pool_ref.clone();
        let rt = tokio::runtime::Handle::try_current();
        let bars = match rt {
            Ok(handle) => tokio::task::block_in_place(|| {
                handle.block_on(async move {
                    let to = Utc::now();
                    let from = to - Duration::days(400);
                    prices::get_bars(&pool_ref, &symbol, BarInterval::D1, from, to)
                        .await
                        .unwrap_or_default()
                })
            }),
            Err(_) => Vec::new(),
        };
        bars.into_iter()
            .filter_map(|b| b.close.to_f64().map(|c| (b.bar_time.date_naive(), c)))
            .collect()
    };
    Ok(backtest_with_history("pead", &samples, &closes_async))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn linear_closes(start: NaiveDate, days: usize, gradient_pct: f64) -> Vec<(NaiveDate, f64)> {
        // Generates a sequence where each successive close is gradient_pct
        // higher than the prior — useful for known-value tests.
        let mut out = Vec::with_capacity(days);
        let mut price = 100.0_f64;
        for i in 0..days {
            out.push((start + Duration::days(i as i64), price));
            price *= 1.0 + gradient_pct / 100.0;
        }
        out
    }

    #[test]
    fn signed_return_long_positive_when_price_rises() {
        let closes = linear_closes(d(2026, 1, 1), 100, 0.5);
        let r = signed_return_pct(&closes, d(2026, 1, 1), Direction::Long, 5).unwrap();
        // After 5 days at +0.5%/day, price ≈ 1.005^5 = 1.02525 → log ≈ 2.49%
        assert!((r - 2.494).abs() < 0.01);
    }

    #[test]
    fn signed_return_short_inverts_sign() {
        let closes = linear_closes(d(2026, 1, 1), 100, 0.5);
        let r = signed_return_pct(&closes, d(2026, 1, 1), Direction::Short, 5).unwrap();
        // Short captures the negative of the up-move.
        assert!(r < 0.0);
    }

    #[test]
    fn signed_return_none_when_forward_bar_missing() {
        let closes = linear_closes(d(2026, 1, 1), 3, 0.5); // Only 3 days.
                                                           // 30-day horizon falls past the supplied history.
        assert!(signed_return_pct(&closes, d(2026, 1, 1), Direction::Long, 30).is_none());
    }

    #[test]
    fn signed_return_walks_to_first_bar_on_or_after_target() {
        // Sparse history: only Mon/Wed/Fri prints. Forward target falls
        // on a Saturday → should land on the next available Monday.
        let closes = vec![
            (d(2026, 1, 5), 100.0), // Mon, signal
            (d(2026, 1, 7), 101.0),
            (d(2026, 1, 9), 102.0),
            (d(2026, 1, 12), 103.0), // Next Mon — captures the 5d horizon.
        ];
        let r = signed_return_pct(&closes, d(2026, 1, 5), Direction::Long, 5).unwrap();
        // log(103/100) ≈ 2.956%
        assert!((r - 2.956).abs() < 0.01);
    }

    #[test]
    fn aggregate_empty_returns_zero_filled_row() {
        let stats = aggregate(&[], 5);
        assert_eq!(stats.n, 0);
        assert_eq!(stats.hit_rate_pct, 0.0);
        assert_eq!(stats.annualised_sharpe, 0.0);
    }

    #[test]
    fn aggregate_hit_rate_counts_positives() {
        let returns = vec![1.0, -0.5, 2.0, -3.0, 0.5];
        let s = aggregate(&returns, 5);
        assert_eq!(s.n, 5);
        // 3 positive out of 5 → 60%
        assert!((s.hit_rate_pct - 60.0).abs() < 1e-9);
    }

    #[test]
    fn aggregate_median_correct_for_even_and_odd_n() {
        // Odd n
        let odd = aggregate(&[1.0, 2.0, 3.0], 1);
        assert_eq!(odd.median_return_pct, 2.0);
        // Even n
        let even = aggregate(&[1.0, 2.0, 3.0, 4.0], 1);
        assert_eq!(even.median_return_pct, 2.5);
    }

    #[test]
    fn aggregate_annualises_sharpe_by_horizon() {
        // Two return series with same mean/stdev but different horizons —
        // the 5-day horizon should annualise more than the 60-day.
        let returns = vec![1.0, -0.5, 1.5, 0.0, 2.0, -1.0, 1.0];
        let s_short = aggregate(&returns, 5);
        let s_long = aggregate(&returns, 60);
        assert!(s_short.annualised_sharpe.abs() > s_long.annualised_sharpe.abs());
    }

    #[test]
    fn aggregate_max_dd_tracks_peak_minus_trough_cumulative() {
        // Cumulative path: 5, 7, 4, 9, 3.
        // Peak hits 9 (after the +5 bar); trough after is 3 → drawdown 6.
        let returns = vec![5.0, 2.0, -3.0, 5.0, -6.0];
        let s = aggregate(&returns, 1);
        assert!((s.max_drawdown_pct - 6.0).abs() < 1e-9);
    }

    #[test]
    fn backtest_with_history_uses_only_samples_with_data() {
        let history = std::sync::Arc::new(linear_closes(d(2026, 1, 1), 100, 0.2));
        let history2 = history.clone();
        let closes_fn = move |sym: &str| -> Vec<(NaiveDate, f64)> {
            match sym {
                "AAA" => (*history2).clone(),
                _ => Vec::new(),
            }
        };
        let samples = vec![
            SignalSample {
                symbol: "AAA".into(),
                signal_date: d(2026, 1, 5),
                direction: Direction::Long,
            },
            SignalSample {
                symbol: "BBB".into(),
                signal_date: d(2026, 1, 5),
                direction: Direction::Long,
            },
        ];
        let r = backtest_with_history("test", &samples, &closes_fn);
        assert_eq!(r.scanner, "test");
        assert_eq!(r.samples_used, 1, "BBB has no history; only AAA scored");
        assert_eq!(r.horizons.len(), DEFAULT_HORIZONS_DAYS.len());
        // AAA at +0.2%/day, all horizons should be positive on the long side.
        for h in &r.horizons {
            assert_eq!(h.n, 1);
            assert!(h.mean_return_pct > 0.0);
        }
    }

    #[test]
    fn backtest_with_history_returns_zero_rows_when_no_samples() {
        let closes_fn = |_: &str| -> Vec<(NaiveDate, f64)> { Vec::new() };
        let r = backtest_with_history("empty", &[], &closes_fn);
        assert_eq!(r.samples_used, 0);
        assert_eq!(r.horizons.len(), DEFAULT_HORIZONS_DAYS.len());
        for h in &r.horizons {
            assert_eq!(h.n, 0);
        }
    }
}
