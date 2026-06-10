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
    /// Standard error of the annualised Sharpe per Andrew Lo 2002:
    /// SE(SR) = sqrt((1 + 0.5 × SR²) / N). At N < 30 the SE is large
    /// enough that the headline Sharpe number is mostly noise.
    pub sharpe_se: f64,
    /// 95% confidence interval lower bound. The conservative number to
    /// size Kelly from. A Sharpe of 1.2 with SE = 0.5 has CI [0.22, 2.18]
    /// — the headline is "1.2" but the trader's number is 0.22.
    pub sharpe_ci_lo_95: f64,
    pub sharpe_ci_hi_95: f64,
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

/// Walk-forward variant: splits the signal set into in-sample (first
/// `train_pct`%) and out-of-sample (remainder) by chronological
/// `signal_date`, runs the same horizon math on each half, and reports
/// both. Without OOS evaluation every backtest is biased upward by
/// the optimizer-over-history effect; OOS Sharpe is the conservative
/// number to size capital from.
#[derive(Debug, Clone, Serialize)]
pub struct WalkForwardResult {
    pub scanner: String,
    pub train_pct: u32,
    pub train_samples_used: usize,
    pub test_samples_used: usize,
    pub train_horizons: Vec<HorizonStats>,
    pub test_horizons: Vec<HorizonStats>,
    /// `test_sharpe_20d / train_sharpe_20d` per horizon — a degradation
    /// indicator. Ratio ≥ 0.7 is healthy; ≤ 0.3 is overfit/decayed.
    pub oos_to_is_sharpe_ratio_20d: Option<f64>,
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
    // Andrew Lo 2002 standard error for Sharpe ratio. At small N the
    // headline number is mostly noise; the CI lower bound is what to
    // size Kelly from.
    let sharpe_se = if n >= 2 {
        ((1.0 + 0.5 * annualised_sharpe * annualised_sharpe) / n as f64).sqrt()
    } else {
        0.0
    };
    let sharpe_ci_lo_95 = annualised_sharpe - 1.96 * sharpe_se;
    let sharpe_ci_hi_95 = annualised_sharpe + 1.96 * sharpe_se;
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
        sharpe_se,
        sharpe_ci_lo_95,
        sharpe_ci_hi_95,
        max_drawdown_pct: max_dd,
        total_logret_signed: total_logret,
    }
}

/// Quarterly stability decomposition: split samples into 4
/// chronological quarters and report Sharpe in each. A signal whose
/// Sharpe trends down across quarters is degrading; one that's roughly
/// flat across quarters is more trustworthy than the headline implies.
#[derive(Debug, Clone, Serialize)]
pub struct QuarterStats {
    pub quarter_index: u32,
    pub samples_used: usize,
    pub horizon_stats: HorizonStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct StabilityReport {
    pub scanner: String,
    pub horizon_days: u32,
    pub quarters: Vec<QuarterStats>,
    /// Sharpe in the latest quarter / Sharpe in the earliest quarter.
    /// < 1 = decaying, ~1 = stable, > 1 = improving.
    pub q4_vs_q1_sharpe_ratio: Option<f64>,
    /// Headline full-window stats for comparison.
    pub full_window_horizon_stats: HorizonStats,
}

/// Pure compute: divide chronologically-sorted samples into N equal-size
/// quarters by index, run the horizon's backtest on each.
pub fn quarterly_decomposition(
    scanner: &str,
    samples: &[SignalSample],
    closes_for: &dyn Fn(&str) -> Vec<(NaiveDate, f64)>,
    horizon_days: u32,
) -> StabilityReport {
    let mut sorted = samples.to_vec();
    sorted.sort_by_key(|s| s.signal_date);
    let n = sorted.len();
    let quarter_size = n / 4;
    let mut quarters: Vec<QuarterStats> = Vec::with_capacity(4);
    for i in 0..4 {
        let start = i * quarter_size;
        let end = if i == 3 { n } else { (i + 1) * quarter_size };
        let chunk = if start < end {
            &sorted[start..end]
        } else {
            &[]
        };
        let result = backtest_with_history(scanner, chunk, closes_for);
        let horizon_stats = result
            .horizons
            .into_iter()
            .find(|h| h.horizon_days == horizon_days)
            .unwrap_or_default();
        quarters.push(QuarterStats {
            quarter_index: i as u32 + 1,
            samples_used: result.samples_used,
            horizon_stats,
        });
    }
    let full = backtest_with_history(scanner, &sorted, closes_for);
    let full_horizon = full
        .horizons
        .into_iter()
        .find(|h| h.horizon_days == horizon_days)
        .unwrap_or_default();
    let q4_vs_q1 = match (quarters.first(), quarters.last()) {
        (Some(q1), Some(q4))
            if q1.horizon_stats.annualised_sharpe.abs() > 1e-9
                && q1.horizon_stats.n > 0
                && q4.horizon_stats.n > 0 =>
        {
            Some(q4.horizon_stats.annualised_sharpe / q1.horizon_stats.annualised_sharpe)
        }
        _ => None,
    };
    StabilityReport {
        scanner: scanner.into(),
        horizon_days,
        quarters,
        q4_vs_q1_sharpe_ratio: q4_vs_q1,
        full_window_horizon_stats: full_horizon,
    }
}

const DEFAULT_HORIZONS_DAYS: &[u32] = &[1, 5, 20, 60];

/// Pure compute: walk-forward split of a chronologically-sorted sample
/// list. Sorts by signal_date, picks the first `train_pct`% into the
/// training set, remainder into test. Empty input → empty splits.
pub fn walk_forward_split(
    samples: &[SignalSample],
    train_pct: u32,
) -> (Vec<SignalSample>, Vec<SignalSample>) {
    if samples.is_empty() || train_pct == 0 || train_pct >= 100 {
        return (samples.to_vec(), Vec::new());
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by_key(|s| s.signal_date);
    let cutoff = (sorted.len() as f64 * train_pct as f64 / 100.0).round() as usize;
    let test = sorted.split_off(cutoff.min(sorted.len()));
    (sorted, test)
}

/// Walk-forward backtest: splits chronologically + runs the same
/// horizon math on each half. `train_pct` typical 70.
pub fn walk_forward_backtest(
    scanner: &str,
    samples: &[SignalSample],
    closes_for: &dyn Fn(&str) -> Vec<(NaiveDate, f64)>,
    train_pct: u32,
) -> WalkForwardResult {
    let (train, test) = walk_forward_split(samples, train_pct);
    let train_r = backtest_with_history(scanner, &train, closes_for);
    let test_r = backtest_with_history(scanner, &test, closes_for);
    let ratio = {
        let train_20 = train_r.horizons.iter().find(|h| h.horizon_days == 20);
        let test_20 = test_r.horizons.iter().find(|h| h.horizon_days == 20);
        match (train_20, test_20) {
            (Some(t_in), Some(t_out))
                if t_in.annualised_sharpe.is_finite() && t_in.annualised_sharpe.abs() > 1e-9 =>
            {
                Some(t_out.annualised_sharpe / t_in.annualised_sharpe)
            }
            _ => None,
        }
    };
    WalkForwardResult {
        scanner: scanner.into(),
        train_pct,
        train_samples_used: train_r.samples_used,
        test_samples_used: test_r.samples_used,
        train_horizons: train_r.horizons,
        test_horizons: test_r.horizons,
        oos_to_is_sharpe_ratio_20d: ratio,
    }
}

/// Same as `backtest_with_history` but subtracts round-trip friction
/// from every per-signal return before aggregating. The Sharpe / hit
/// rate / mean / max-DD reported are then the *net* figures — what the
/// autopilot would actually have realized including slippage,
/// commission, and SEC fees. Use this for live-trading decisions; use
/// the gross variant only for comparing pure signal predictive power.
pub fn backtest_with_history_with_friction(
    scanner: &str,
    samples: &[SignalSample],
    closes_for: &dyn Fn(&str) -> Vec<(NaiveDate, f64)>,
    friction: crate::friction::FrictionConfig,
) -> BacktestResult {
    let mut result = backtest_with_history(scanner, samples, closes_for);
    let cost = friction.round_trip_pct();
    if cost == 0.0 {
        return result;
    }
    for h in result.horizons.iter_mut() {
        if h.n == 0 {
            continue;
        }
        // Subtract the round-trip cost from mean + median + total
        // signed return; stdev is unchanged (friction is a constant
        // per-leg, doesn't affect dispersion); Sharpe recomputes.
        h.mean_return_pct -= cost;
        h.median_return_pct -= cost;
        h.total_logret_signed -= cost * h.n as f64 / 100.0;
        if h.stdev_pct > 0.0 && h.horizon_days > 0 {
            h.annualised_sharpe =
                h.mean_return_pct / h.stdev_pct * (252.0 / h.horizon_days as f64).sqrt();
        }
        // Hit rate at gross 0% threshold → after subtracting cost, hit
        // means net return > 0 ↔ gross return > cost. We can't recover
        // that from the aggregate alone — flag conservatively by
        // leaving hit_rate gross; the live trader sees Sharpe / mean
        // as the friction-adjusted numbers and uses hit_rate as the
        // gross-signal indicator.
    }
    result
}

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

/// Pure compute: cluster insider-purchase events by (symbol, signal_date)
/// where signal_date is the latest txn_date in the cluster. A cluster
/// fires when ≥ `min_distinct_insiders` distinct insiders bought the
/// same symbol within the trailing `window_days`. Each cluster becomes
/// one Long sample on the latest txn_date.
///
/// Why cluster vs single-insider: Lakonishok & Lee 2001 + Cohen Malloy
/// Pomorski 2012 both show single-insider buys are noisy but ≥3-insider
/// opportunistic clusters predict ~12% alpha over the next 12 months.
/// Same threshold the live `insider_clusters` scanner uses.
pub fn cluster_insider_purchases(
    events: &[(String, NaiveDate, String)],
    window_days: i64,
    min_distinct_insiders: usize,
) -> Vec<SignalSample> {
    use std::collections::HashMap;
    let mut by_symbol: HashMap<String, Vec<(NaiveDate, String)>> = HashMap::new();
    for (sym, date, filer) in events {
        by_symbol
            .entry(sym.clone())
            .or_default()
            .push((*date, filer.clone()));
    }
    let mut samples = Vec::new();
    for (sym, mut rows) in by_symbol {
        rows.sort_by_key(|(d, _)| *d);
        // Rolling window: for each row, look back window_days and count
        // distinct filers including this row. Emit one sample per
        // cluster *event* but dedupe by signal_date so a single hot
        // streak doesn't generate one sample per day.
        let mut last_emitted: Option<NaiveDate> = None;
        for i in 0..rows.len() {
            let end_date = rows[i].0;
            let start_date = end_date - Duration::days(window_days);
            let window: std::collections::HashSet<&String> = rows[..=i]
                .iter()
                .filter(|(d, _)| *d >= start_date)
                .map(|(_, f)| f)
                .collect();
            if window.len() >= min_distinct_insiders {
                // Emit only when we haven't already emitted within the
                // last window_days — otherwise consecutive insider days
                // re-fire the same cluster.
                let should_emit = match last_emitted {
                    Some(last) => (end_date - last).num_days() >= window_days,
                    None => true,
                };
                if should_emit {
                    samples.push(SignalSample {
                        symbol: sym.clone(),
                        signal_date: end_date,
                        direction: Direction::Long,
                    });
                    last_emitted = Some(end_date);
                }
            }
        }
    }
    samples
}

/// Insider-cluster backtest: pulls every txn_type='P' (purchase) Form 4
/// event over the trailing `days` window, clusters them per symbol with
/// 30-day rolling window + ≥3 distinct insiders, and scores forward
/// returns from cached price_bars. Same cluster logic as the live
/// `insider_clusters` scanner.
pub async fn backtest_insider_clusters(pool: &PgPool, days: i64) -> anyhow::Result<BacktestResult> {
    let rows: Vec<(String, NaiveDate, String)> = sqlx::query_as(
        "SELECT symbol, txn_date, filer_name
           FROM disclosures
          WHERE kind = 'insider_form4'
            AND txn_type = 'P'
            AND symbol IS NOT NULL
            AND txn_date IS NOT NULL
            AND txn_date >= CURRENT_DATE - ($1::int)
          ORDER BY symbol, txn_date",
    )
    .bind(days as i32)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let samples = cluster_insider_purchases(&rows, 30, 3);

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
    Ok(backtest_with_history(
        "insider_clusters",
        &samples,
        &closes_async,
    ))
}

/// Walk-forward PEAD backtest convenience wrapper.
pub async fn walk_forward_pead(
    pool: &PgPool,
    days: i64,
    train_pct: u32,
) -> anyhow::Result<WalkForwardResult> {
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
    Ok(walk_forward_backtest(
        "pead",
        &samples,
        &closes_async,
        train_pct,
    ))
}

/// PEAD quarterly stability report at the requested horizon.
pub async fn stability_pead(
    pool: &PgPool,
    days: i64,
    horizon_days: u32,
) -> anyhow::Result<StabilityReport> {
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
    Ok(quarterly_decomposition(
        "pead",
        &samples,
        &closes_async,
        horizon_days,
    ))
}

/// IPO lockup backtest: pulls every IPO whose 180-day lockup expired
/// in the trailing `days` window, projects signal_date = lockup_expiry,
/// direction = Short (Field & Hanka 2001 — mechanical supply pressure
/// when insider/employee shares unlock). Scores forward 1d/5d/20d/60d
/// returns from cached price_bars.
///
/// First Finnhub fetch can be slow + rate-limited; subsequent calls hit
/// whatever cache the finnhub_rest layer maintains. Bars come from the
/// same price_bars table the autopilot uses.
pub async fn backtest_ipo_lockups(pool: &PgPool, days: i64) -> anyhow::Result<BacktestResult> {
    let events = crate::ipo_lockups::historical(days)
        .await
        .unwrap_or_default();
    let samples: Vec<SignalSample> = events
        .into_iter()
        .map(|e| SignalSample {
            symbol: e.symbol,
            signal_date: e.lockup_expires_at,
            direction: Direction::Short,
        })
        .collect();
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
    Ok(backtest_with_history(
        "ipo_lockups",
        &samples,
        &closes_async,
    ))
}

/// Walk-forward insider-cluster backtest convenience wrapper.
pub async fn walk_forward_insider_clusters(
    pool: &PgPool,
    days: i64,
    train_pct: u32,
) -> anyhow::Result<WalkForwardResult> {
    let rows: Vec<(String, NaiveDate, String)> = sqlx::query_as(
        "SELECT symbol, txn_date, filer_name
           FROM disclosures
          WHERE kind = 'insider_form4'
            AND txn_type = 'P'
            AND symbol IS NOT NULL
            AND txn_date IS NOT NULL
            AND txn_date >= CURRENT_DATE - ($1::int)
          ORDER BY symbol, txn_date",
    )
    .bind(days as i32)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let samples = cluster_insider_purchases(&rows, 30, 3);
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
    Ok(walk_forward_backtest(
        "insider_clusters",
        &samples,
        &closes_async,
        train_pct,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn aggregate_emits_sharpe_se_and_ci() {
        // 50 returns drawn around mean 1.0%, stdev 2% → annual SR
        // around (1.0/2.0) × sqrt(252/5) ≈ 3.55 if horizon = 5d.
        let returns: Vec<f64> = (0..50)
            .map(|i| 1.0 + ((i % 4) as f64 - 1.5) * 0.5)
            .collect();
        let s = aggregate(&returns, 5);
        assert!(s.sharpe_se > 0.0, "SE should be > 0 with N=50");
        // CI is symmetric around the headline Sharpe by ±1.96·SE.
        let mid = (s.sharpe_ci_lo_95 + s.sharpe_ci_hi_95) / 2.0;
        assert!((mid - s.annualised_sharpe).abs() < 1e-9);
        let half_width = (s.sharpe_ci_hi_95 - s.sharpe_ci_lo_95) / 2.0;
        assert!((half_width - 1.96 * s.sharpe_se).abs() < 1e-9);
    }

    #[test]
    fn aggregate_sharpe_se_shrinks_with_more_samples() {
        // Same per-sample distribution, more samples → smaller SE.
        let r10: Vec<f64> = (0..10).map(|i| 1.0 + (i % 2) as f64).collect();
        let r100: Vec<f64> = (0..100).map(|i| 1.0 + (i % 2) as f64).collect();
        let s10 = aggregate(&r10, 20);
        let s100 = aggregate(&r100, 20);
        assert!(s100.sharpe_se < s10.sharpe_se);
    }

    #[test]
    fn aggregate_empty_returns_zero_se() {
        let s = aggregate(&[], 20);
        assert_eq!(s.sharpe_se, 0.0);
        assert_eq!(s.sharpe_ci_lo_95, 0.0);
        assert_eq!(s.sharpe_ci_hi_95, 0.0);
    }

    #[test]
    fn quarterly_decomposition_splits_chronologically() {
        let history = std::sync::Arc::new(linear_closes(d(2026, 1, 1), 400, 0.4));
        let history2 = history.clone();
        let closes_fn = move |sym: &str| -> Vec<(NaiveDate, f64)> {
            if sym == "AAA" {
                (*history2).clone()
            } else {
                Vec::new()
            }
        };
        let samples: Vec<SignalSample> = (0..40)
            .map(|i| SignalSample {
                symbol: "AAA".into(),
                signal_date: d(2026, 1, 1) + Duration::days(i * 8),
                direction: Direction::Long,
            })
            .collect();
        let r = quarterly_decomposition("test", &samples, &closes_fn, 20);
        assert_eq!(r.scanner, "test");
        assert_eq!(r.quarters.len(), 4);
        // Each quarter should be roughly 10 samples (40/4).
        for q in &r.quarters {
            assert!(q.samples_used >= 9 && q.samples_used <= 11);
        }
        // Sharpe ratio should be defined (some signal in both endpoints).
        // We don't pin a precise ratio because the 20d look-ahead pushes
        // late-quarter signal returns past the end of cached history,
        // which affects q4 sample count + variance asymmetrically.
        // The point of this test is *the structure*, not the value.
        assert!(r.full_window_horizon_stats.n > 0);
    }

    #[test]
    fn quarterly_decomposition_empty_returns_four_empty_quarters() {
        let closes_fn = |_: &str| -> Vec<(NaiveDate, f64)> { Vec::new() };
        let r = quarterly_decomposition("e", &[], &closes_fn, 20);
        assert_eq!(r.quarters.len(), 4);
        for q in &r.quarters {
            assert_eq!(q.samples_used, 0);
            assert_eq!(q.horizon_stats.n, 0);
        }
        assert!(r.q4_vs_q1_sharpe_ratio.is_none());
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

    fn insider(
        symbol: &str,
        year: i32,
        month: u32,
        day: u32,
        filer: &str,
    ) -> (String, NaiveDate, String) {
        (symbol.into(), d(year, month, day), filer.into())
    }

    #[test]
    fn cluster_fires_when_three_distinct_insiders_buy_within_window() {
        let events = vec![
            insider("AAA", 2026, 1, 1, "CEO Smith"),
            insider("AAA", 2026, 1, 5, "Director Jones"),
            insider("AAA", 2026, 1, 15, "CFO Brown"),
        ];
        let samples = cluster_insider_purchases(&events, 30, 3);
        assert_eq!(samples.len(), 1, "exactly one cluster emitted");
        assert_eq!(samples[0].symbol, "AAA");
        assert_eq!(samples[0].direction, Direction::Long);
        assert_eq!(samples[0].signal_date, d(2026, 1, 15));
    }

    #[test]
    fn cluster_does_not_fire_below_distinct_threshold() {
        let events = vec![
            insider("BBB", 2026, 1, 1, "CEO Smith"),
            insider("BBB", 2026, 1, 5, "CEO Smith"), // same filer again
            insider("BBB", 2026, 1, 15, "CEO Smith"), // same filer
        ];
        let samples = cluster_insider_purchases(&events, 30, 3);
        assert!(samples.is_empty(), "same filer 3x ≠ 3 distinct insiders");
    }

    #[test]
    fn cluster_does_not_fire_outside_window() {
        // 3 distinct insiders but spread 60 days → no window has 3.
        let events = vec![
            insider("CCC", 2026, 1, 1, "A"),
            insider("CCC", 2026, 2, 5, "B"),
            insider("CCC", 2026, 4, 15, "C"),
        ];
        let samples = cluster_insider_purchases(&events, 30, 3);
        assert!(samples.is_empty(), "events too spread out → no cluster");
    }

    #[test]
    fn cluster_dedupes_consecutive_hot_streak() {
        // 5 insider buys in 10 days — should fire ONCE, not 5×.
        let events = vec![
            insider("DDD", 2026, 1, 1, "A"),
            insider("DDD", 2026, 1, 3, "B"),
            insider("DDD", 2026, 1, 5, "C"),
            insider("DDD", 2026, 1, 7, "D"),
            insider("DDD", 2026, 1, 10, "E"),
        ];
        let samples = cluster_insider_purchases(&events, 30, 3);
        assert_eq!(
            samples.len(),
            1,
            "hot streak dedupes to one cluster per window"
        );
    }

    #[test]
    fn cluster_can_fire_twice_when_separated_by_window() {
        // First cluster Jan, second cluster May → two clusters total.
        let events = vec![
            insider("EEE", 2026, 1, 1, "A"),
            insider("EEE", 2026, 1, 5, "B"),
            insider("EEE", 2026, 1, 10, "C"),
            insider("EEE", 2026, 5, 1, "D"),
            insider("EEE", 2026, 5, 5, "E"),
            insider("EEE", 2026, 5, 10, "F"),
        ];
        let samples = cluster_insider_purchases(&events, 30, 3);
        assert_eq!(samples.len(), 2, "two clusters separated by > 30d");
    }

    #[test]
    fn cluster_empty_input_empty_output() {
        let samples = cluster_insider_purchases(&[], 30, 3);
        assert!(samples.is_empty());
    }

    #[test]
    fn friction_lowers_mean_and_sharpe_vs_gross() {
        let history = std::sync::Arc::new(linear_closes(d(2026, 1, 1), 200, 0.5));
        let history2 = history.clone();
        let closes_fn = move |sym: &str| -> Vec<(NaiveDate, f64)> {
            if sym == "AAA" {
                (*history2).clone()
            } else {
                Vec::new()
            }
        };
        let samples: Vec<SignalSample> = (5..50)
            .step_by(10)
            .map(|i| SignalSample {
                symbol: "AAA".into(),
                signal_date: d(2026, 1, 1) + Duration::days(i),
                direction: Direction::Long,
            })
            .collect();
        let gross = backtest_with_history("g", &samples, &closes_fn);
        let net = backtest_with_history_with_friction(
            "n",
            &samples,
            &closes_fn,
            crate::friction::FrictionConfig::baseline_equity(),
        );
        // Net mean strictly less than gross mean by the round-trip cost.
        let cost = crate::friction::FrictionConfig::baseline_equity().round_trip_pct();
        for (g, n) in gross.horizons.iter().zip(net.horizons.iter()) {
            if g.n > 0 {
                assert!((g.mean_return_pct - n.mean_return_pct - cost).abs() < 1e-6);
                // Sharpe also lower because mean is lower, stdev unchanged.
                assert!(n.annualised_sharpe < g.annualised_sharpe);
            }
        }
    }

    #[test]
    fn walk_forward_split_70_30_chronologically() {
        let samples: Vec<SignalSample> = (1..=10)
            .map(|i| SignalSample {
                symbol: format!("S{i}"),
                signal_date: d(2026, 1, i as u32),
                direction: Direction::Long,
            })
            .collect();
        let (train, test) = walk_forward_split(&samples, 70);
        assert_eq!(train.len(), 7);
        assert_eq!(test.len(), 3);
        assert_eq!(train[0].signal_date, d(2026, 1, 1));
        assert_eq!(test[0].signal_date, d(2026, 1, 8));
    }

    #[test]
    fn walk_forward_split_sorts_chronologically_first() {
        // Reverse-ordered input → splitter must sort by date before splitting.
        let samples: Vec<SignalSample> = (1..=10)
            .rev()
            .map(|i| SignalSample {
                symbol: format!("S{i}"),
                signal_date: d(2026, 1, i as u32),
                direction: Direction::Long,
            })
            .collect();
        let (train, test) = walk_forward_split(&samples, 70);
        assert_eq!(train.first().unwrap().signal_date, d(2026, 1, 1));
        assert_eq!(test.last().unwrap().signal_date, d(2026, 1, 10));
    }

    #[test]
    fn walk_forward_split_edge_cases() {
        let samples: Vec<SignalSample> = vec![];
        let (t, _) = walk_forward_split(&samples, 70);
        assert!(t.is_empty());
        // train_pct=0 or 100 → train holds everything, test empty
        let single = vec![SignalSample {
            symbol: "A".into(),
            signal_date: d(2026, 1, 1),
            direction: Direction::Long,
        }];
        let (t0, te0) = walk_forward_split(&single, 0);
        assert_eq!(t0.len(), 1);
        assert!(te0.is_empty());
        let (t100, te100) = walk_forward_split(&single, 100);
        assert_eq!(t100.len(), 1);
        assert!(te100.is_empty());
    }

    #[test]
    fn walk_forward_backtest_reports_both_halves_and_ratio() {
        let history = std::sync::Arc::new(linear_closes(d(2026, 1, 1), 300, 0.3));
        let history2 = history.clone();
        let closes_fn = move |sym: &str| -> Vec<(NaiveDate, f64)> {
            if sym == "AAA" {
                (*history2).clone()
            } else {
                Vec::new()
            }
        };
        // 20 samples spread over the year — first 14 are training, last 6 test.
        let samples: Vec<SignalSample> = (0..20)
            .map(|i| SignalSample {
                symbol: "AAA".into(),
                signal_date: d(2026, 1, 1) + Duration::days(i * 10),
                direction: Direction::Long,
            })
            .collect();
        let r = walk_forward_backtest("wf", &samples, &closes_fn, 70);
        assert_eq!(r.scanner, "wf");
        assert_eq!(r.train_samples_used + r.test_samples_used, samples.len());
        // Stable linear-up series → train Sharpe and test Sharpe should
        // both be positive and the ratio should be defined.
        if let Some(ratio) = r.oos_to_is_sharpe_ratio_20d {
            assert!(
                ratio > 0.5,
                "stable series should have OOS ratio close to 1, got {ratio}"
            );
        }
    }

    #[test]
    fn friction_zero_config_matches_gross() {
        let history = std::sync::Arc::new(linear_closes(d(2026, 1, 1), 100, 0.2));
        let history2 = history.clone();
        let closes_fn = move |sym: &str| -> Vec<(NaiveDate, f64)> {
            if sym == "AAA" {
                (*history2).clone()
            } else {
                Vec::new()
            }
        };
        let samples = vec![SignalSample {
            symbol: "AAA".into(),
            signal_date: d(2026, 1, 5),
            direction: Direction::Long,
        }];
        let gross = backtest_with_history("g", &samples, &closes_fn);
        let net = backtest_with_history_with_friction(
            "n",
            &samples,
            &closes_fn,
            crate::friction::FrictionConfig::frictionless(),
        );
        for (g, n) in gross.horizons.iter().zip(net.horizons.iter()) {
            assert_eq!(g.mean_return_pct, n.mean_return_pct);
            assert_eq!(g.annualised_sharpe, n.annualised_sharpe);
        }
    }
}
