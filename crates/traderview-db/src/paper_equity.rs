//! Paper account equity history — background samples of cash + marked
//! position value. Unchanged readings are skipped (a flat account adds
//! no information), so the curve compresses to its turning points plus
//! one sample per change. Summary stats delegate to the shared
//! drawdown_episodes core.

use chrono::{DateTime, Utc};
pub use traderview_core::live_vs_backtest::{hold_stats, trip_stats, HoldStats, TripStats};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct EquitySnapshot {
    pub equity: Decimal,
    pub cash: Decimal,
    pub position_value: Decimal,
    pub taken_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquitySummary {
    /// Naive first-to-last return — inflated by deposits; kept for
    /// continuity, read twr_return_pct for performance.
    pub return_pct: f64,
    /// Flow-aware time-weighted return; equals return_pct when the
    /// account has no cash flows in the window.
    pub twr_return_pct: Option<f64>,
    pub max_drawdown_pct: f64,
    pub currently_underwater: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkOverlay {
    pub symbol: String,
    /// Benchmark "equity" aligned to each snapshot's timestamp —
    /// normalized so both series start at the first snapshot's equity.
    /// None where the benchmark has no bar yet.
    pub values: Vec<Option<f64>>,
    pub summary: Option<EquitySummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquityHistory {
    pub snapshots: Vec<EquitySnapshot>,
    pub summary: Option<EquitySummary>,
    pub benchmark: Option<BenchmarkOverlay>,
}

/// Align benchmark closes to snapshot timestamps: for each snapshot
/// time, the latest bar close at-or-before it (two-pointer, both
/// inputs chronological). None before the first bar — leading
/// snapshots predate the benchmark window rather than borrowing a
/// future price.
pub fn align_benchmark(
    snap_times: &[i64],
    bar_times: &[i64],
    closes: &[f64],
) -> Vec<Option<f64>> {
    let mut out = Vec::with_capacity(snap_times.len());
    let mut j = 0usize;
    for &t in snap_times {
        while j + 1 < bar_times.len() && bar_times[j + 1] <= t {
            j += 1;
        }
        if bar_times.is_empty() || bar_times[j] > t {
            out.push(None);
        } else {
            out.push(closes.get(j).copied());
        }
    }
    out
}

/// Time-weighted return over a snapshot series with external cash
/// flows: each snapshot interval's return strips the net flow that
/// landed inside it ((e_next − flow) / e_prev), and the segments
/// compound. Deposits don't fake performance, withdrawals don't fake
/// losses — the measure of the STRATEGY, not the funding. Flows
/// outside the snapshot window are ignored (they're not in the
/// measured period); a non-positive segment base is None, not a
/// nonsense compound. Both inputs ascending by timestamp.
pub fn twr_return_pct(snapshots: &[(i64, f64)], flows: &[(i64, f64)]) -> Option<f64> {
    if snapshots.len() < 2 {
        return None;
    }
    let mut factor = 1.0_f64;
    for w in snapshots.windows(2) {
        let (t0, e0) = w[0];
        let (t1, e1) = w[1];
        if e0 <= 0.0 {
            return None;
        }
        let flow: f64 = flows
            .iter()
            .filter(|(t, _)| *t > t0 && *t <= t1)
            .map(|(_, a)| a)
            .sum();
        factor *= (e1 - flow) / e0;
    }
    Some((factor - 1.0) * 100.0)
}

/// Summary over an equity series: total return plus worst drawdown via
/// the shared drawdown_episodes core. None for fewer than 2 points.
pub fn summarize(series: &[f64]) -> Option<EquitySummary> {
    if series.len() < 2 || series[0] <= 0.0 {
        return None;
    }
    let report = traderview_core::drawdown_episodes::compute(series, 1)?;
    // The core reports depths as NEGATIVE percentages; expose the worst
    // as a positive magnitude.
    let worst = report
        .episodes
        .first()
        .map(|e| e.depth_pct)
        .unwrap_or(0.0)
        .min(report.current_drawdown_pct);
    Some(EquitySummary {
        return_pct: (series[series.len() - 1] / series[0] - 1.0) * 100.0,
        twr_return_pct: None, // db wrapper fills this with flow data
        max_drawdown_pct: -worst,
        currently_underwater: report.currently_underwater,
    })
}

/// One sampling pass over ALL paper accounts. Marks positions at the
/// cached quote (held symbols only) and inserts a snapshot when the
/// equity differs from the previous one. Returns snapshots written.
pub async fn snapshot_all(pool: &PgPool) -> anyhow::Result<usize> {
    let accounts: Vec<(Uuid, Decimal)> =
        sqlx::query_as("SELECT id, cash FROM paper_accounts").fetch_all(pool).await?;
    let mut written = 0usize;
    for (account_id, cash) in accounts {
        let positions: Vec<(String, Decimal)> = sqlx::query_as(
            "SELECT symbol, qty FROM paper_positions WHERE paper_account_id = $1",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
        let mut position_value = Decimal::ZERO;
        let mut all_marked = true;
        for (symbol, qty) in &positions {
            // Option positions mark at the chain mid x 100 multiplier;
            // equities at the cached quote.
            if let Some(occ) = traderview_core::occ_symbol::parse(symbol) {
                match crate::paper::option_quote(&occ).await {
                    Ok(Some(p)) => match Decimal::try_from(p * 100.0) {
                        Ok(v) => position_value += v * qty,
                        Err(_) => all_marked = false,
                    },
                    _ => all_marked = false,
                }
                continue;
            }
            match crate::market_data::quote(pool, symbol).await {
                Ok(q) => match Decimal::try_from(q.price) {
                    Ok(p) => position_value += p * qty,
                    Err(_) => all_marked = false,
                },
                Err(_) => all_marked = false,
            }
        }
        // A partially-marked book would write a fake dip into the
        // curve — skip the account this pass instead of lying.
        if !all_marked {
            continue;
        }
        let equity = cash + position_value;
        let last: Option<(Decimal,)> = sqlx::query_as(
            "SELECT equity FROM paper_equity_snapshots
              WHERE paper_account_id = $1
              ORDER BY taken_at DESC LIMIT 1",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
        if matches!(last, Some((e,)) if e == equity) {
            continue;
        }
        sqlx::query(
            "INSERT INTO paper_equity_snapshots
                (paper_account_id, equity, cash, position_value)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(account_id)
        .bind(equity)
        .bind(cash)
        .bind(position_value)
        .execute(pool)
        .await?;
        written += 1;
    }
    Ok(written)
}

/// Return vs the account's starting cash — the honest cross-account
/// comparison base (snapshot-series-relative return would flatter an
/// account that lost money before its first sample). None when
/// starting cash is degenerate.
pub fn account_return_pct(starting_cash: Decimal, equity: Decimal) -> Option<f64> {
    if starting_cash <= Decimal::ZERO {
        return None;
    }
    let s: f64 = starting_cash.to_string().parse().ok()?;
    let e: f64 = equity.to_string().parse().ok()?;
    Some((e / s - 1.0) * 100.0)
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountComparison {
    pub account_id: Uuid,
    pub name: String,
    pub starting_cash: Decimal,
    pub equity: Decimal,
    pub return_pct: Option<f64>,
    pub max_drawdown_pct: Option<f64>,
    pub currently_underwater: bool,
    pub snapshots: i64,
}

/// Strategy leaderboard: one row per account, ranked by return vs
/// starting cash (descending; unranked accounts last). Equity = latest
/// snapshot when one exists, else cash (fresh account, nothing held).
pub async fn compare(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<AccountComparison>> {
    let accounts: Vec<(Uuid, String, Decimal, Decimal)> = sqlx::query_as(
        "SELECT id, name, starting_cash, cash FROM paper_accounts
          WHERE user_id = $1 ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let mut rows = Vec::with_capacity(accounts.len());
    for (account_id, name, starting_cash, cash) in accounts {
        let snaps: Vec<(Decimal,)> = sqlx::query_as(
            "SELECT equity FROM paper_equity_snapshots
              WHERE paper_account_id = $1 ORDER BY taken_at",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
        let equity = snaps.last().map(|(e,)| *e).unwrap_or(cash);
        let series: Vec<f64> = snaps
            .iter()
            .filter_map(|(e,)| e.to_string().parse().ok())
            .collect();
        let summary = summarize(&series);
        rows.push(AccountComparison {
            account_id,
            name,
            starting_cash,
            equity,
            // Lifetime modified Dietz over net deposits — a funded
            // account doesn't outrank an unfunded one on deposits.
            return_pct: {
                use rust_decimal::prelude::ToPrimitive;
                let flows: Option<(Decimal,)> = sqlx::query_as(
                    "SELECT COALESCE(SUM(amount), 0) FROM paper_cash_flows
                      WHERE paper_account_id = $1",
                )
                .bind(account_id)
                .fetch_optional(pool)
                .await
                .unwrap_or(None);
                let net_flow = flows.and_then(|(d,)| d.to_f64()).unwrap_or(0.0);
                match (starting_cash.to_f64(), equity.to_f64()) {
                    (Some(s), Some(e)) if s > 0.0 => modified_dietz(s, e, net_flow),
                    _ => account_return_pct(starting_cash, equity),
                }
            },
            max_drawdown_pct: summary.as_ref().map(|s| s.max_drawdown_pct),
            currently_underwater: summary.map(|s| s.currently_underwater).unwrap_or(false),
            snapshots: snaps.len() as i64,
        });
    }
    rows.sort_by(|a, b| {
        b.return_pct
            .unwrap_or(f64::NEG_INFINITY)
            .total_cmp(&a.return_pct.unwrap_or(f64::NEG_INFINITY))
    });
    Ok(rows)
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolAttribution {
    pub symbol: String,
    /// Realized PnL of CLOSED round trips (FIFO from the fill ledger,
    /// fees netted; options scaled by the 100× multiplier).
    pub trading_pnl: f64,
    pub closed_trips: usize,
    pub dividends: f64,
    pub fees: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonthRow {
    /// "YYYY-MM".
    pub month: String,
    pub trading_pnl: f64,
    pub dividends: f64,
    pub closed_trips: usize,
}

/// Group realized trips (by close time) and dividends (by ex-date)
/// into calendar months, chronological. Pure — pinned directly.
pub fn monthly_rollup(
    trips: &[(f64, i64)],
    dividends: &[(f64, chrono::NaiveDate)],
) -> Vec<MonthRow> {
    use chrono::Datelike;
    let mut map: std::collections::BTreeMap<String, MonthRow> = Default::default();
    for (pnl, ts) in trips {
        let d = chrono::DateTime::from_timestamp(*ts, 0)
            .map(|t| t.date_naive())
            .unwrap_or_default();
        let key = format!("{:04}-{:02}", d.year(), d.month());
        let row = map.entry(key.clone()).or_insert_with(|| MonthRow {
            month: key,
            trading_pnl: 0.0,
            dividends: 0.0,
            closed_trips: 0,
        });
        row.trading_pnl += pnl;
        row.closed_trips += 1;
    }
    for (amount, date) in dividends {
        let key = format!("{:04}-{:02}", date.year(), date.month());
        let row = map.entry(key.clone()).or_insert_with(|| MonthRow {
            month: key,
            trading_pnl: 0.0,
            dividends: 0.0,
            closed_trips: 0,
        });
        row.dividends += amount;
    }
    map.into_values().collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct Attribution {
    /// Ranked by |total contribution| descending.
    pub symbols: Vec<SymbolAttribution>,
    /// Calendar months, chronological — the WHEN to the symbols' WHERE.
    pub months: Vec<MonthRow>,
    /// Trade-quality metrics over all closed trips. None until trips exist.
    pub stats: Option<TripStats>,
    /// Hold-duration discipline check (winners vs losers).
    pub hold: Option<HoldStats>,
    /// Trips whose OPENING order carried a written plan vs those
    /// without — the measurable payoff of plan-before-trade.
    pub planned_stats: Option<TripStats>,
    pub unplanned_stats: Option<TripStats>,
    pub total_trading_pnl: f64,
    pub total_dividends: f64,
    pub total_fees: f64,
}

/// Where the account's P&L came from, per symbol: closed-trip trading
/// PnL + dividends − fees. Reconstructed from the fill ledger because
/// paper_positions deletes a row when it closes to zero — realized
/// PnL of closed positions lives nowhere else. Open positions'
/// unrealized PnL is deliberately NOT included: this is the realized
/// record, and the positions table already shows live unrealized.
#[derive(Debug, serde::Serialize)]
pub struct SymbolWashSales {
    pub symbol: String,
    pub sales: Vec<traderview_core::tax::WashSale>,
    pub total_disallowed: f64,
}

/// Wash-sale scan over the account's filled orders — same fill
/// reconstruction as attribution (FIFO, fees in basis, OCC prices
/// pre-scaled 100×). Only symbols with at least one flagged sale are
/// returned. Exact-symbol matching only: a loss in shares followed by
/// a repurchase via options on the same underlying is "substantially
/// identical" to the IRS but NOT detected here.
pub async fn wash_sales(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
) -> anyhow::Result<Vec<SymbolWashSales>> {
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let fills: Vec<(String, String, Decimal, Decimal, Decimal, chrono::DateTime<Utc>)> =
        sqlx::query_as(
            "SELECT symbol, side::text, filled_qty, filled_price, fee, filled_at
               FROM paper_orders
              WHERE paper_account_id = $1 AND status = 'filled'
                AND filled_qty IS NOT NULL AND filled_price IS NOT NULL
              ORDER BY filled_at",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
    use rust_decimal::prelude::ToPrimitive;
    let mut by_symbol: std::collections::BTreeMap<String, Vec<traderview_core::live_vs_backtest::Fill>> =
        Default::default();
    for (symbol, side, qty, price, fee, at) in fills {
        let scale = if traderview_core::occ_symbol::is_occ(&symbol) { 100.0 } else { 1.0 };
        by_symbol.entry(symbol).or_default().push(traderview_core::live_vs_backtest::Fill {
            buy: side == "buy" || side == "cover",
            qty: qty.to_f64().unwrap_or(0.0),
            price: price.to_f64().unwrap_or(0.0) * scale,
            commission: fee.to_f64().unwrap_or(0.0),
            ts: at.timestamp(),
            flag: false,
        });
    }
    Ok(by_symbol
        .into_iter()
        .filter_map(|(symbol, fills)| {
            let sales = traderview_core::tax::wash_sales(&fills);
            (!sales.is_empty()).then(|| SymbolWashSales {
                total_disallowed: sales.iter().map(|w| w.disallowed).sum(),
                symbol,
                sales,
            })
        })
        .collect())
}

pub async fn attribution(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
) -> anyhow::Result<Attribution> {
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let fills: Vec<(String, String, Decimal, Decimal, Decimal, chrono::DateTime<Utc>, Option<String>)> =
        sqlx::query_as(
            "SELECT symbol, side::text, filled_qty, filled_price, fee, filled_at, plan_note
               FROM paper_orders
              WHERE paper_account_id = $1 AND status = 'filled'
                AND filled_qty IS NOT NULL AND filled_price IS NOT NULL
              ORDER BY filled_at",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
    use rust_decimal::prelude::ToPrimitive;
    let mut by_symbol: std::collections::BTreeMap<String, (Vec<traderview_core::live_vs_backtest::Fill>, f64)> =
        Default::default();
    for (symbol, side, qty, price, fee, at, plan_note) in fills {
        // Options: pre-scale the per-share price by the 100× multiplier
        // so trip PnL is dollar-true while commissions (already dollars)
        // stay unscaled inside the reconstruction.
        let scale = if traderview_core::occ_symbol::is_occ(&symbol) { 100.0 } else { 1.0 };
        let entry = by_symbol.entry(symbol).or_default();
        entry.1 += fee.to_f64().unwrap_or(0.0);
        entry.0.push(traderview_core::live_vs_backtest::Fill {
            buy: side == "buy" || side == "cover",
            qty: qty.to_f64().unwrap_or(0.0),
            price: price.to_f64().unwrap_or(0.0) * scale,
            commission: fee.to_f64().unwrap_or(0.0),
            ts: at.timestamp(),
            // The plan flag rides the fill; trips inherit it from
            // their OPENING fill only.
            flag: plan_note.as_deref().map(str::trim).is_some_and(|s| !s.is_empty()),
        });
    }
    let divs: Vec<(String, Decimal)> = sqlx::query_as(
        "SELECT symbol, COALESCE(SUM(cash_credited), 0)
           FROM paper_dividends WHERE paper_account_id = $1 GROUP BY symbol",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    let div_map: std::collections::BTreeMap<String, f64> = divs
        .into_iter()
        .map(|(s, c)| (s, c.to_f64().unwrap_or(0.0)))
        .collect();
    let mut symbols = Vec::new();
    let (mut tt, mut td, mut tf) = (0.0, 0.0, 0.0);
    let mut seen: std::collections::BTreeSet<String> = Default::default();
    let mut all_trips: Vec<(f64, i64)> = Vec::new();
    let mut all_holds: Vec<(f64, i64)> = Vec::new();
    let mut all_flagged: Vec<(f64, i64, bool)> = Vec::new();
    for (symbol, (fills, fees)) in &by_symbol {
        let trips = traderview_core::live_vs_backtest::round_trips(fills);
        all_trips.extend(trips.iter().map(|t| (t.pnl, t.closed_ts)));
        all_holds.extend(trips.iter().map(|t| (t.pnl, t.closed_ts - t.opened_ts)));
        all_flagged.extend(trips.iter().map(|t| (t.pnl, t.closed_ts, t.opened_flag)));
        let trading_pnl: f64 = trips.iter().map(|t| t.pnl).sum();
        let dividends = div_map.get(symbol).copied().unwrap_or(0.0);
        seen.insert(symbol.clone());
        tt += trading_pnl;
        td += dividends;
        tf += fees;
        symbols.push(SymbolAttribution {
            symbol: symbol.clone(),
            trading_pnl,
            closed_trips: trips.len(),
            dividends,
            fees: *fees,
        });
    }
    // Dividend-only symbols (position opened elsewhere/now closed with
    // no fills in range, or credited after full exit) still appear.
    for (symbol, dividends) in &div_map {
        if !seen.contains(symbol) {
            td += dividends;
            symbols.push(SymbolAttribution {
                symbol: symbol.clone(),
                trading_pnl: 0.0,
                closed_trips: 0,
                dividends: *dividends,
                fees: 0.0,
            });
        }
    }
    symbols.sort_by(|a, b| {
        (b.trading_pnl + b.dividends)
            .abs()
            .total_cmp(&(a.trading_pnl + a.dividends).abs())
    });
    let div_dated: Vec<(Decimal, chrono::NaiveDate)> = sqlx::query_as(
        "SELECT cash_credited, ex_date FROM paper_dividends WHERE paper_account_id = $1",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    let div_dated: Vec<(f64, chrono::NaiveDate)> = div_dated
        .into_iter()
        .map(|(c, d)| (c.to_f64().unwrap_or(0.0), d))
        .collect();
    let months = monthly_rollup(&all_trips, &div_dated);
    // Chronological for the streaks — trips were collected per symbol.
    all_trips.sort_by_key(|(_, ts)| *ts);
    let pnls: Vec<f64> = all_trips.iter().map(|(p, _)| *p).collect();
    let stats = trip_stats(&pnls);
    let hold = hold_stats(&all_holds);
    // Planned/unplanned splits, each chronological for their streaks.
    all_flagged.sort_by_key(|(_, ts, _)| *ts);
    let planned: Vec<f64> = all_flagged.iter().filter(|(_, _, f)| *f).map(|(p, _, _)| *p).collect();
    let unplanned: Vec<f64> =
        all_flagged.iter().filter(|(_, _, f)| !*f).map(|(p, _, _)| *p).collect();
    Ok(Attribution {
        symbols,
        months,
        stats,
        hold,
        planned_stats: trip_stats(&planned),
        unplanned_stats: trip_stats(&unplanned),
        total_trading_pnl: tt,
        total_dividends: td,
        total_fees: tf,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct CorrelationPair {
    pub a: String,
    pub b: String,
    pub rho: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PositionCorrelations {
    /// Held equity symbols in matrix order. OCC option positions are
    /// EXCLUDED (they correlate through their underlying; correlating
    /// option marks against stock closes would be noise) — and the
    /// exclusion is reported, never silent.
    pub symbols: Vec<String>,
    pub excluded_options: Vec<String>,
    /// Symmetric; diagonal 1.0; None where the trailing overlap is
    /// under 20 sessions.
    pub matrix: Vec<Vec<Option<f64>>>,
    /// Pairs with |ρ| > 0.7 — the redundancy warnings, worst first.
    pub redundant_pairs: Vec<CorrelationPair>,
}

/// Pairwise daily-return correlations of the account's CURRENT equity
/// holdings — "is this book five copies of one trade". Same Pearson +
/// returns math as the algo entry-correlation gate.
pub async fn position_correlations(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    lookback_days: i64,
) -> anyhow::Result<PositionCorrelations> {
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let held: Vec<(String,)> = sqlx::query_as(
        "SELECT symbol FROM paper_positions WHERE paper_account_id = $1 ORDER BY symbol",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    let mut symbols = Vec::new();
    let mut excluded_options = Vec::new();
    for (s,) in held {
        if traderview_core::occ_symbol::is_occ(&s) {
            excluded_options.push(s);
        } else {
            symbols.push(s);
        }
    }
    let to = Utc::now();
    let from = to - chrono::Duration::days(lookback_days.clamp(30, 365));
    let mut returns: Vec<Vec<f64>> = Vec::with_capacity(symbols.len());
    for s in &symbols {
        use rust_decimal::prelude::ToPrimitive;
        let bars =
            crate::prices::get_bars(pool, s, traderview_core::BarInterval::D1, from, to).await?;
        let closes: Vec<f64> = bars.iter().map(|b| b.close.to_f64().unwrap_or(0.0)).collect();
        returns.push(traderview_core::correlation_gate::daily_returns(&closes));
    }
    let n = symbols.len();
    let mut matrix = vec![vec![None; n]; n];
    let mut redundant_pairs = Vec::new();
    for i in 0..n {
        matrix[i][i] = Some(1.0);
        for j in (i + 1)..n {
            let len = returns[i].len().min(returns[j].len());
            if len < 20 {
                continue; // a correlation over days of overlap is noise
            }
            let a = &returns[i][returns[i].len() - len..];
            let b = &returns[j][returns[j].len() - len..];
            let rho = traderview_core::correlation::pearson(a, b);
            matrix[i][j] = rho;
            matrix[j][i] = rho;
            if let Some(r) = rho {
                if r.abs() > 0.7 {
                    redundant_pairs.push(CorrelationPair {
                        a: symbols[i].clone(),
                        b: symbols[j].clone(),
                        rho: r,
                    });
                }
            }
        }
    }
    redundant_pairs.sort_by(|x, y| y.rho.abs().total_cmp(&x.rho.abs()));
    Ok(PositionCorrelations {
        symbols,
        excluded_options,
        matrix,
        redundant_pairs,
    })
}

/// Weighted portfolio returns over the TRAILING common overlap of the
/// per-symbol return series — today's weights applied historically
/// (the standard historical-simulation approximation). Pure, pinned.
pub fn weighted_portfolio_returns(weights: &[f64], returns: &[Vec<f64>]) -> Vec<f64> {
    if weights.is_empty() || weights.len() != returns.len() {
        return Vec::new();
    }
    let Some(len) = returns.iter().map(|r| r.len()).min() else {
        return Vec::new();
    };
    (0..len)
        .map(|t| {
            weights
                .iter()
                .zip(returns)
                .map(|(w, r)| w * r[r.len() - len + t])
                .sum()
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioVar {
    /// Current book value the dollar figures scale against.
    pub book_value: f64,
    pub sessions: usize,
    pub var_95_usd: f64,
    pub es_95_usd: f64,
    pub var_99_usd: f64,
    pub es_99_usd: f64,
    pub var_95_pct: f64,
    pub var_99_pct: f64,
    pub excluded_options: Vec<String>,
}

/// (gross exposure, portfolio daily returns, excluded options) for
/// the current equity book — the shared input to VaR and stress.
async fn book_returns(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    lookback_days: i64,
) -> anyhow::Result<(f64, Vec<f64>, Vec<String>)> {
    use rust_decimal::prelude::ToPrimitive;
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let held: Vec<(String, Decimal)> = sqlx::query_as(
        "SELECT symbol, qty FROM paper_positions WHERE paper_account_id = $1",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    let mut excluded_options = Vec::new();
    let mut notionals: Vec<f64> = Vec::new();
    let mut all_returns: Vec<Vec<f64>> = Vec::new();
    let to = Utc::now();
    let from = to - chrono::Duration::days(lookback_days.clamp(90, 730));
    for (symbol, qty) in held {
        if traderview_core::occ_symbol::is_occ(&symbol) {
            excluded_options.push(symbol);
            continue;
        }
        let bars =
            crate::prices::get_bars(pool, &symbol, traderview_core::BarInterval::D1, from, to)
                .await?;
        let closes: Vec<f64> = bars.iter().map(|b| b.close.to_f64().unwrap_or(0.0)).collect();
        let Some(last) = closes.last().copied().filter(|p| *p > 0.0) else {
            anyhow::bail!("no price history for {symbol}");
        };
        notionals.push(qty.to_f64().unwrap_or(0.0) * last);
        all_returns.push(traderview_core::correlation_gate::daily_returns(&closes));
    }
    if notionals.is_empty() {
        anyhow::bail!("no equity positions to measure");
    }
    let gross: f64 = notionals.iter().map(|n| n.abs()).sum();
    if gross <= 0.0 {
        anyhow::bail!("zero book value");
    }
    let weights: Vec<f64> = notionals.iter().map(|n| n / gross).collect();
    let port = weighted_portfolio_returns(&weights, &all_returns);
    Ok((gross, port, excluded_options))
}

/// Historical-simulation portfolio VaR of the current equity book:
/// today's weights over the joint trailing return history, the
/// empirical 95/99 quantiles in dollars. Needs >= 60 common sessions —
/// a VaR from a few weeks of data is noise stated as a number.
pub async fn portfolio_var(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    lookback_days: i64,
) -> anyhow::Result<PortfolioVar> {
    let (gross, port, excluded_options) =
        book_returns(pool, user_id, account_id, lookback_days).await?;
    if port.len() < 60 {
        anyhow::bail!(
            "only {} common sessions across holdings — need >= 60 for a VaR that isn't noise",
            port.len()
        );
    }
    let v95 = traderview_core::value_at_risk_historical::compute(&port, 0.95)
        .ok_or_else(|| anyhow::anyhow!("VaR computation failed"))?;
    let v99 = traderview_core::value_at_risk_historical::compute(&port, 0.99)
        .ok_or_else(|| anyhow::anyhow!("VaR computation failed"))?;
    Ok(PortfolioVar {
        book_value: gross,
        sessions: port.len(),
        var_95_usd: v95.var * gross,
        es_95_usd: v95.expected_shortfall * gross,
        var_99_usd: v99.var * gross,
        es_99_usd: v99.expected_shortfall * gross,
        var_95_pct: v95.var * 100.0,
        var_99_pct: v99.var * 100.0,
        excluded_options,
    })
}

/// Worst observed k-day compounded return in the series — the
/// realized stress record. Pure, pinned.
pub fn worst_window(returns: &[f64], k: usize) -> Option<f64> {
    if k == 0 || returns.len() < k {
        return None;
    }
    returns
        .windows(k)
        .map(|w| w.iter().fold(1.0, |acc, r| acc * (1.0 + r)) - 1.0)
        .min_by(|a, b| a.total_cmp(b))
}

/// OLS beta of the portfolio vs a benchmark over the trailing common
/// overlap: cov(p, b) / var(b). Pure, pinned.
pub fn beta_vs(port: &[f64], bench: &[f64]) -> Option<f64> {
    let n = port.len().min(bench.len());
    if n < 20 {
        return None;
    }
    let p = &port[port.len() - n..];
    let b = &bench[bench.len() - n..];
    let mp = p.iter().sum::<f64>() / n as f64;
    let mb = b.iter().sum::<f64>() / n as f64;
    let cov: f64 = p.iter().zip(b).map(|(x, y)| (x - mp) * (y - mb)).sum();
    let var: f64 = b.iter().map(|y| (y - mb) * (y - mb)).sum();
    (var > 0.0).then(|| cov / var)
}

#[derive(Debug, Clone, Serialize)]
pub struct StressScenario {
    pub label: String,
    pub book_move_usd: f64,
    pub book_move_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StressReport {
    pub book_value: f64,
    pub sessions: usize,
    /// Worst OBSERVED windows in the book's own joint history.
    pub worst_day_usd: f64,
    pub worst_week_usd: f64,
    pub worst_month_usd: f64,
    /// Beta vs the benchmark; shock scenarios scale by it.
    pub beta: Option<f64>,
    pub benchmark: String,
    pub scenarios: Vec<StressScenario>,
    pub excluded_options: Vec<String>,
}

/// Stress test: the book's own worst observed 1/5/20-day windows
/// (realized history, no model) + beta-scaled benchmark shocks
/// (-5/-10/-20%). The first answers "what HAS this book done"; the
/// second "what would a market break do".
pub async fn stress(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    lookback_days: i64,
    benchmark: &str,
) -> anyhow::Result<StressReport> {
    use rust_decimal::prelude::ToPrimitive;
    let (gross, port, excluded_options) =
        book_returns(pool, user_id, account_id, lookback_days).await?;
    if port.len() < 60 {
        anyhow::bail!(
            "only {} common sessions across holdings — need >= 60",
            port.len()
        );
    }
    let to = Utc::now();
    let from = to - chrono::Duration::days(lookback_days.clamp(90, 730));
    let bars =
        crate::prices::get_bars(pool, benchmark, traderview_core::BarInterval::D1, from, to)
            .await?;
    let closes: Vec<f64> = bars.iter().map(|b| b.close.to_f64().unwrap_or(0.0)).collect();
    let bench_returns = traderview_core::correlation_gate::daily_returns(&closes);
    let beta = beta_vs(&port, &bench_returns);
    let scenarios = beta
        .map(|b| {
            [-0.05, -0.10, -0.20]
                .iter()
                .map(|shock| StressScenario {
                    label: format!("{benchmark} {:.0}%", shock * 100.0),
                    book_move_usd: b * shock * gross,
                    book_move_pct: b * shock * 100.0,
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(StressReport {
        book_value: gross,
        sessions: port.len(),
        worst_day_usd: worst_window(&port, 1).unwrap_or(0.0) * gross,
        worst_week_usd: worst_window(&port, 5).unwrap_or(0.0) * gross,
        worst_month_usd: worst_window(&port, 20).unwrap_or(0.0) * gross,
        beta,
        benchmark: benchmark.to_string(),
        scenarios,
        excluded_options,
    })
}

/// Equity history for an owned account, oldest first, with summary.
pub async fn history(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    limit: i64,
    benchmark_symbol: &str,
) -> anyhow::Result<EquityHistory> {
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let snapshots: Vec<EquitySnapshot> = sqlx::query_as(
        "SELECT equity, cash, position_value, taken_at
           FROM (SELECT equity, cash, position_value, taken_at
                   FROM paper_equity_snapshots
                  WHERE paper_account_id = $1
                  ORDER BY taken_at DESC LIMIT $2) t
          ORDER BY taken_at",
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    let series: Vec<f64> = snapshots
        .iter()
        .filter_map(|s| s.equity.to_string().parse().ok())
        .collect();
    let mut summary = summarize(&series);
    if let Some(sm) = summary.as_mut() {
        let flows: Vec<(chrono::DateTime<Utc>, Decimal)> = sqlx::query_as(
            "SELECT created_at, amount FROM paper_cash_flows
              WHERE paper_account_id = $1 ORDER BY created_at",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        use rust_decimal::prelude::ToPrimitive;
        let snap_pts: Vec<(i64, f64)> = snapshots
            .iter()
            .zip(series.iter())
            .map(|(s, e)| (s.taken_at.timestamp(), *e))
            .collect();
        let flow_pts: Vec<(i64, f64)> = flows
            .iter()
            .map(|(t, a)| (t.timestamp(), a.to_f64().unwrap_or(0.0)))
            .collect();
        sm.twr_return_pct = twr_return_pct(&snap_pts, &flow_pts);
    }
    // Benchmark overlay: SPY (or caller's symbol) normalized so both
    // series start at the FIRST SNAPSHOT's equity — same start, same
    // scale, honest comparison.
    let benchmark = if snapshots.len() >= 2 {
        build_benchmark(pool, &snapshots, &series, benchmark_symbol).await
    } else {
        None
    };
    Ok(EquityHistory {
        snapshots,
        summary,
        benchmark,
    })
}

async fn build_benchmark(
    pool: &PgPool,
    snapshots: &[EquitySnapshot],
    equity_series: &[f64],
    symbol: &str,
) -> Option<BenchmarkOverlay> {
    use rust_decimal::prelude::ToPrimitive;
    let from = snapshots.first()?.taken_at - chrono::Duration::days(5);
    let to = snapshots.last()?.taken_at;
    let bars = crate::prices::get_bars(pool, symbol, traderview_core::BarInterval::D1, from, to)
        .await
        .ok()?;
    if bars.is_empty() {
        return None;
    }
    let bar_times: Vec<i64> = bars.iter().map(|b| b.bar_time.timestamp()).collect();
    let closes: Vec<f64> = bars
        .iter()
        .map(|b| b.close.to_f64().unwrap_or(0.0))
        .collect();
    let snap_times: Vec<i64> = snapshots.iter().map(|s| s.taken_at.timestamp()).collect();
    let aligned = align_benchmark(&snap_times, &bar_times, &closes);
    // Normalize to the first snapshot's equity at the first ALIGNED
    // benchmark price.
    let first_equity = *equity_series.first()?;
    let base = aligned.iter().flatten().copied().find(|v| *v > 0.0)?;
    let values: Vec<Option<f64>> = aligned
        .iter()
        .map(|v| v.map(|p| first_equity * p / base))
        .collect();
    let bench_series: Vec<f64> = values.iter().flatten().copied().collect();
    Some(BenchmarkOverlay {
        symbol: symbol.to_string(),
        values,
        summary: summarize(&bench_series),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_pins_return_and_worst_drawdown() {
        // 100 → 120 (peak) → 90 (-25% from peak) → 110: return +10%,
        // max drawdown 25%, recovered above 90 but still under the 120
        // high-water mark → underwater.
        let s = summarize(&[100.0, 120.0, 90.0, 110.0]).unwrap();
        assert!((s.return_pct - 10.0).abs() < 1e-9);
        assert!((s.max_drawdown_pct - 25.0).abs() < 1e-9);
        assert!(s.currently_underwater);
    }

    #[test]
    fn monotonic_curve_has_zero_drawdown() {
        let s = summarize(&[100.0, 105.0, 111.0]).unwrap();
        assert!((s.return_pct - 11.0).abs() < 1e-9);
        assert_eq!(s.max_drawdown_pct, 0.0);
        assert!(!s.currently_underwater);
    }

    #[test]
    fn too_short_or_degenerate_series_is_none() {
        assert!(summarize(&[100.0]).is_none());
        assert!(summarize(&[]).is_none());
        assert!(summarize(&[0.0, 100.0]).is_none());
    }

    #[test]
    fn hold_stats_pin_the_asymmetry_flag() {
        // Winners held ~1h, losers held ~4h: flagged.
        let t = vec![(100.0, 3600), (50.0, 3000), (-80.0, 14000), (-20.0, 15000)];
        let h = hold_stats(&t).unwrap();
        assert!((h.avg_hold_secs_winners - 3300.0).abs() < 1e-9);
        assert!((h.avg_hold_secs_losers - 14500.0).abs() < 1e-9);
        assert_eq!(h.behavioral_flag, Some("cutting_winners_riding_losers"));
        // Symmetric holds: no flag.
        let t = vec![(100.0, 3600), (-80.0, 3600)];
        assert_eq!(hold_stats(&t).unwrap().behavioral_flag, None);
        // Exactly at 1.5x: flagged (>= boundary).
        let t = vec![(100.0, 1000), (-80.0, 1500)];
        assert_eq!(
            hold_stats(&t).unwrap().behavioral_flag,
            Some("cutting_winners_riding_losers")
        );
        // One-sided records can't measure asymmetry.
        assert_eq!(hold_stats(&[(100.0, 3600)]).unwrap().behavioral_flag, None);
        assert!(hold_stats(&[]).is_none());
    }

    #[test]
    fn trip_stats_pin_expectancy_and_edge_handling() {
        // 3 wins (100, 50, 250), 2 losses (-80, -120): n=5.
        let s = trip_stats(&[100.0, -80.0, 50.0, 250.0, -120.0]).unwrap();
        assert_eq!(s.trades, 5);
        assert_eq!(s.wins, 3);
        assert!((s.win_rate - 0.6).abs() < 1e-12);
        assert!((s.avg_win - 400.0 / 3.0).abs() < 1e-9);
        assert!((s.avg_loss - 100.0).abs() < 1e-9);
        assert!((s.profit_factor.unwrap() - 2.0).abs() < 1e-9); // 400/200
        assert!((s.expectancy - 40.0).abs() < 1e-9); // 200/5
        assert!((s.largest_win - 250.0).abs() < 1e-9);
        assert!((s.largest_loss + 120.0).abs() < 1e-9);
        // Kelly on this record: W=0.6, R=(400/3)/100=4/3 →
        // f* = 0.6 − 0.4/(4/3) = 0.30 exactly.
        assert!((s.kelly_fraction.unwrap() - 0.30).abs() < 1e-12);
        // Sequence +,−,+,+,−: longest win 2, longest loss 1, current −1.
        assert_eq!(s.longest_win_streak, 2);
        assert_eq!(s.longest_loss_streak, 1);
        assert_eq!(s.current_streak, -1);
        // All winners: PF None (not infinity), largest_loss 0.
        let s = trip_stats(&[10.0, 20.0]).unwrap();
        assert_eq!(s.profit_factor, None);
        assert_eq!(s.largest_loss, 0.0);
        // Kelly undefined without losses (R has no denominator) — None,
        // never "bet everything".
        assert_eq!(s.kelly_fraction, None);
        assert_eq!(s.current_streak, 2);
        // A zero-PnL trip counts as a LOSS — it paid fees for nothing.
        let s = trip_stats(&[0.0, 10.0]).unwrap();
        assert_eq!(s.wins, 1);
        assert!((s.win_rate - 0.5).abs() < 1e-12);
        assert!(trip_stats(&[]).is_none());
    }

    #[test]
    fn monthly_rollup_pins_grouping_and_order() {
        use chrono::NaiveDate;
        // Two trips in 2026-05 (one negative), one in 2026-06; a
        // dividend in 2026-04 (a month with NO trips) and one in May.
        let may1 = 1_777_000_000i64; // 2026-04-24? compute below instead
        // Use explicit timestamps: 2026-05-05 and 2026-05-20, 2026-06-02.
        let ts = |y: i32, m: u32, d: u32| {
            NaiveDate::from_ymd_opt(y, m, d)
                .unwrap()
                .and_hms_opt(15, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp()
        };
        let _ = may1;
        let trips = vec![
            (500.0, ts(2026, 5, 5)),
            (-200.0, ts(2026, 5, 20)),
            (300.0, ts(2026, 6, 2)),
        ];
        let divs = vec![
            (12.5, NaiveDate::from_ymd_opt(2026, 4, 15).unwrap()),
            (10.0, NaiveDate::from_ymd_opt(2026, 5, 11).unwrap()),
        ];
        let rows = monthly_rollup(&trips, &divs);
        assert_eq!(
            rows.iter().map(|r| r.month.as_str()).collect::<Vec<_>>(),
            vec!["2026-04", "2026-05", "2026-06"]
        );
        // April: dividend-only month still appears.
        assert_eq!(rows[0].closed_trips, 0);
        assert!((rows[0].dividends - 12.5).abs() < 1e-9);
        // May nets the winner and loser, counts both trips.
        assert!((rows[1].trading_pnl - 300.0).abs() < 1e-9);
        assert_eq!(rows[1].closed_trips, 2);
        assert!((rows[1].dividends - 10.0).abs() < 1e-9);
        assert!((rows[2].trading_pnl - 300.0).abs() < 1e-9);
    }

    #[test]
    fn worst_window_and_beta_pin_hand_math() {
        // Returns: +1%, −2%, −3%, +4%. Worst day −3%; worst 2-day
        // window compounds −2% then −3%: 0.98×0.97 − 1 = −4.94%.
        let r = [0.01, -0.02, -0.03, 0.04];
        assert!((worst_window(&r, 1).unwrap() + 0.03).abs() < 1e-12);
        assert!((worst_window(&r, 2).unwrap() + 0.0494).abs() < 1e-9);
        assert!(worst_window(&r, 5).is_none()); // window longer than data
        assert!(worst_window(&r, 0).is_none());
        // Beta: portfolio = exactly 1.5 × benchmark → beta 1.5.
        let bench: Vec<f64> = (0..40).map(|i| ((i as f64) * 0.7).sin() * 0.01).collect();
        let port: Vec<f64> = bench.iter().map(|r| 1.5 * r).collect();
        assert!((beta_vs(&port, &bench).unwrap() - 1.5).abs() < 1e-9);
        // Sub-20 overlap refuses.
        assert!(beta_vs(&port[..10], &bench[..10]).is_none());
    }

    #[test]
    fn weighted_portfolio_returns_pin_trailing_alignment() {
        // Two assets 60/40; the longer series contributes its TAIL.
        let w = [0.6, 0.4];
        let r = vec![vec![0.01, 0.02, -0.01], vec![99.0, 0.00, 0.01, 0.02]];
        let p = weighted_portfolio_returns(&w, &r);
        // Common length 3 → second series uses its last 3 (0.00, 0.01, 0.02).
        assert_eq!(p.len(), 3);
        assert!((p[0] - (0.6 * 0.01 + 0.4 * 0.00)).abs() < 1e-12);
        assert!((p[1] - (0.6 * 0.02 + 0.4 * 0.01)).abs() < 1e-12);
        assert!((p[2] - (0.6 * -0.01 + 0.4 * 0.02)).abs() < 1e-12);
        // Mismatched lengths or empty: empty result, never a panic.
        assert!(weighted_portfolio_returns(&[1.0], &[]).is_empty());
        assert!(weighted_portfolio_returns(&[], &[]).is_empty());
    }

    #[test]
    fn benchmark_alignment_pins_two_pointer_and_leading_none() {
        // Bars at t=100, 200, 300; snapshots before/at/between/after.
        let bars = [100i64, 200, 300];
        let closes = [10.0, 11.0, 12.0];
        let snaps = [50i64, 100, 250, 999];
        assert_eq!(
            align_benchmark(&snaps, &bars, &closes),
            vec![None, Some(10.0), Some(11.0), Some(12.0)]
        );
        // No bars at all: all None, no panic.
        assert_eq!(align_benchmark(&snaps, &[], &[]), vec![None; 4]);
    }

    #[test]
    fn return_vs_starting_cash_pins_sign_and_zero_guard() {
        let d = |v: i64| Decimal::from(v);
        assert!((account_return_pct(d(200_000), d(220_000)).unwrap() - 10.0).abs() < 1e-9);
        assert!((account_return_pct(d(200_000), d(150_000)).unwrap() + 25.0).abs() < 1e-9);
        assert!(account_return_pct(Decimal::ZERO, d(100)).is_none());
    }
}

/// UTC bounds of a "YYYY-MM" month: [first instant, first instant of
/// the next month). Pure; None for unparsable input.
pub fn month_bounds(month: &str) -> Option<(chrono::DateTime<Utc>, chrono::DateTime<Utc>)> {
    let (y, m) = month.split_once('-')?;
    let y: i32 = y.parse().ok()?;
    let m: u32 = m.parse().ok()?;
    let start = chrono::NaiveDate::from_ymd_opt(y, m, 1)?;
    let end = if m == 12 {
        chrono::NaiveDate::from_ymd_opt(y + 1, 1, 1)?
    } else {
        chrono::NaiveDate::from_ymd_opt(y, m + 1, 1)?
    };
    let at = |d: chrono::NaiveDate| {
        chrono::DateTime::<Utc>::from_naive_utc_and_offset(d.and_hms_opt(0, 0, 0).unwrap(), Utc)
    };
    Some((at(start), at(end)))
}

/// Modified Dietz return (half-weight convention): the flow-aware
/// period return — a mid-month deposit is capital, not gain, and a
/// withdrawal is not a loss. (close − open − flow) / (open + flow/2),
/// None when the denominator isn't positive. Half-weighting is the
/// stated approximation (flows assumed mid-period) — day-weighting
/// needs per-flow timing the statement deliberately aggregates away.
pub fn modified_dietz(opening: f64, closing: f64, net_flow: f64) -> Option<f64> {
    let denom = opening + 0.5 * net_flow;
    (denom > 0.0).then(|| (closing - opening - net_flow) / denom * 100.0)
}

#[derive(Debug, serde::Serialize)]
pub struct Statement {
    pub month: String,
    /// Last snapshot at-or-before the period start; None when the
    /// account has no history that old (a mid-month account opening
    /// shows None, not a fake zero).
    pub opening_equity: Option<f64>,
    /// Last snapshot inside the period.
    pub closing_equity: Option<f64>,
    /// Flow-aware modified-Dietz return, only when both ends exist.
    pub period_return_pct: Option<f64>,
    /// Net deposits − withdrawals inside the period.
    pub net_deposits: f64,
    /// Trips CLOSED in the period (FIFO, fees netted) — same shared
    /// reconstruction as attribution.
    pub realized_pnl: f64,
    pub trips_closed: usize,
    pub fills: i64,
    pub fees: f64,
    pub dividends: f64,
    pub interest: f64,
    pub borrow_fees: f64,
    pub margin_interest: f64,
}

/// Monthly brokerage-style statement, composed read-only from the
/// stores each subsystem already maintains.
pub async fn statement(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
    month: &str,
) -> anyhow::Result<Statement> {
    use rust_decimal::prelude::ToPrimitive;
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let (start, end) =
        month_bounds(month).ok_or_else(|| anyhow::anyhow!("month must be YYYY-MM"))?;

    let snap = |at: chrono::DateTime<Utc>| async move {
        let r: Option<(Decimal,)> = sqlx::query_as(
            "SELECT equity FROM paper_equity_snapshots
              WHERE paper_account_id = $1 AND taken_at < $2
              ORDER BY taken_at DESC LIMIT 1",
        )
        .bind(account_id)
        .bind(at)
        .fetch_optional(pool)
        .await?;
        anyhow::Ok(r.and_then(|(e,)| e.to_f64()))
    };
    let opening_equity = snap(start).await?;
    // "Closing" = last snapshot INSIDE the period — for the current
    // month that's simply the latest snapshot so far.
    let closing_inside: Option<(Decimal,)> = sqlx::query_as(
        "SELECT equity FROM paper_equity_snapshots
          WHERE paper_account_id = $1 AND taken_at >= $2 AND taken_at < $3
          ORDER BY taken_at DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(start)
    .bind(end)
    .fetch_optional(pool)
    .await?;
    let closing_equity = closing_inside.and_then(|(e,)| e.to_f64());

    let (fills, fees): (i64, f64) = {
        let r: (i64, Option<Decimal>) = sqlx::query_as(
            "SELECT count(*), COALESCE(SUM(fee), 0)
               FROM paper_orders
              WHERE paper_account_id = $1 AND status = 'filled'
                AND filled_at >= $2 AND filled_at < $3",
        )
        .bind(account_id)
        .bind(start)
        .bind(end)
        .fetch_one(pool)
        .await?;
        (r.0, r.1.and_then(|d| d.to_f64()).unwrap_or(0.0))
    };
    let dividends: Option<(Decimal,)> = sqlx::query_as(
        "SELECT COALESCE(SUM(cash_credited), 0) FROM paper_dividends
          WHERE paper_account_id = $1 AND ex_date >= $2::date AND ex_date < $3::date",
    )
    .bind(account_id)
    .bind(start.date_naive())
    .bind(end.date_naive())
    .fetch_optional(pool)
    .await?;
    let interest_rows: Vec<(String, Decimal)> = sqlx::query_as(
        "SELECT kind, COALESCE(SUM(amount), 0) FROM paper_interest
          WHERE paper_account_id = $1 AND credited_on >= $2 AND credited_on < $3
          GROUP BY kind",
    )
    .bind(account_id)
    .bind(start.date_naive())
    .bind(end.date_naive())
    .fetch_all(pool)
    .await?;
    let (mut interest, mut borrow_fees, mut margin_interest) = (0.0, 0.0, 0.0);
    for (kind, amt) in &interest_rows {
        let v = amt.to_f64().unwrap_or(0.0);
        match kind.as_str() {
            // Debits are stored negative; report them as positive costs.
            "short_borrow" => borrow_fees += -v,
            "margin_interest" => margin_interest += -v,
            _ => interest += v,
        }
    }

    let flows: Option<(Decimal,)> = sqlx::query_as(
        "SELECT COALESCE(SUM(amount), 0) FROM paper_cash_flows
          WHERE paper_account_id = $1 AND created_at >= $2 AND created_at < $3",
    )
    .bind(account_id)
    .bind(start)
    .bind(end)
    .fetch_optional(pool)
    .await?;
    let net_deposits = flows.and_then(|(d,)| d.to_f64()).unwrap_or(0.0);

    // Realized P&L: trips closed inside the period, from the same
    // full-history FIFO reconstruction attribution uses (a trip can
    // close this month on lots bought months ago).
    let fills_rows: Vec<(String, String, Decimal, Decimal, Decimal, chrono::DateTime<Utc>)> =
        sqlx::query_as(
            "SELECT symbol, side::text, filled_qty, filled_price, fee, filled_at
               FROM paper_orders
              WHERE paper_account_id = $1 AND status = 'filled'
                AND filled_qty IS NOT NULL AND filled_price IS NOT NULL
              ORDER BY filled_at",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
    let mut by_symbol: std::collections::BTreeMap<String, Vec<traderview_core::live_vs_backtest::Fill>> =
        Default::default();
    for (symbol, side, qty, price, fee, at) in fills_rows {
        let scale = if traderview_core::occ_symbol::is_occ(&symbol) { 100.0 } else { 1.0 };
        by_symbol.entry(symbol).or_default().push(traderview_core::live_vs_backtest::Fill {
            buy: side == "buy" || side == "cover",
            qty: qty.to_f64().unwrap_or(0.0),
            price: price.to_f64().unwrap_or(0.0) * scale,
            commission: fee.to_f64().unwrap_or(0.0),
            ts: at.timestamp(),
            flag: false,
        });
    }
    let (mut realized_pnl, mut trips_closed) = (0.0, 0usize);
    for fills in by_symbol.values() {
        for t in traderview_core::live_vs_backtest::round_trips(fills) {
            if t.closed_ts >= start.timestamp() && t.closed_ts < end.timestamp() {
                realized_pnl += t.pnl;
                trips_closed += 1;
            }
        }
    }

    Ok(Statement {
        month: month.to_string(),
        opening_equity,
        closing_equity,
        period_return_pct: match (opening_equity, closing_equity) {
            (Some(o), Some(c)) => modified_dietz(o, c, net_deposits),
            _ => None,
        },
        net_deposits,
        realized_pnl,
        trips_closed,
        fills,
        fees,
        dividends: dividends.and_then(|(d,)| d.to_f64()).unwrap_or(0.0),
        interest,
        borrow_fees,
        margin_interest,
    })
}

#[cfg(test)]
mod statement_tests {
    use super::*;

    #[test]
    fn twr_compounds_segments_and_strips_flows() {
        // No flows: 100k → 110k = 10%, same as naive.
        assert!((twr_return_pct(&[(0, 100_000.0), (10, 110_000.0)], &[]).unwrap() - 10.0).abs() < 1e-9);
        // Deposit inside the window with zero trading gain: 0%.
        let r = twr_return_pct(
            &[(0, 100_000.0), (10, 120_000.0)],
            &[(5, 20_000.0)],
        ).unwrap();
        assert!(r.abs() < 1e-9);
        // Deposit then a real 10% segment: (120−20)/100 × 132/120 = 1.1.
        let r = twr_return_pct(
            &[(0, 100_000.0), (10, 120_000.0), (20, 132_000.0)],
            &[(5, 20_000.0)],
        ).unwrap();
        assert!((r - 10.0).abs() < 1e-9);
        // Withdrawal isn't a loss: 100k → 85k with 20k out = +5%.
        let r = twr_return_pct(&[(0, 100_000.0), (10, 85_000.0)], &[(5, -20_000.0)]).unwrap();
        assert!((r - 5.0).abs() < 1e-9);
        // Flow exactly AT a snapshot belongs to the interval it ends
        // (t > t0 && t <= t1) — no double-count, no gap.
        let r = twr_return_pct(
            &[(0, 100_000.0), (10, 120_000.0)],
            &[(10, 20_000.0)],
        ).unwrap();
        assert!(r.abs() < 1e-9);
        // Flows outside the window are not the window's business.
        let r = twr_return_pct(&[(10, 100.0), (20, 110.0)], &[(5, 50.0), (25, 50.0)]).unwrap();
        assert!((r - 10.0).abs() < 1e-9);
        // Degenerate base / too few points.
        assert!(twr_return_pct(&[(0, 0.0), (10, 100.0)], &[]).is_none());
        assert!(twr_return_pct(&[(0, 100.0)], &[]).is_none());
    }

    #[test]
    fn dietz_flows_are_capital_not_performance() {
        // No flows: reduces to the simple return.
        assert!((modified_dietz(100_000.0, 110_000.0, 0.0).unwrap() - 10.0).abs() < 1e-9);
        // A 20k deposit with zero trading gain: return 0, not +20%.
        assert!(modified_dietz(100_000.0, 120_000.0, 20_000.0).unwrap().abs() < 1e-9);
        // A withdrawal is not a loss.
        assert!(modified_dietz(100_000.0, 80_000.0, -20_000.0).unwrap().abs() < 1e-9);
        // Gain on top of a deposit: 10k gain over (100k + half the
        // 20k flow) = 10/110.
        let r = modified_dietz(100_000.0, 130_000.0, 20_000.0).unwrap();
        assert!((r - 10_000.0 / 110_000.0 * 100.0).abs() < 1e-9);
        // Degenerate denominator (huge withdrawal): None, not a
        // nonsense percentage.
        assert!(modified_dietz(10_000.0, 0.0, -25_000.0).is_none());
    }

    #[test]
    fn month_bounds_pins_wrap_and_garbage() {
        let (s, e) = month_bounds("2026-05").unwrap();
        assert_eq!(s.to_rfc3339(), "2026-05-01T00:00:00+00:00");
        assert_eq!(e.to_rfc3339(), "2026-06-01T00:00:00+00:00");
        // December wraps the year; February of a leap year still ends
        // at March 1 (the bound is exclusive, day count irrelevant).
        let (_, e) = month_bounds("2025-12").unwrap();
        assert_eq!(e.to_rfc3339(), "2026-01-01T00:00:00+00:00");
        let (s, e) = month_bounds("2024-02").unwrap();
        assert_eq!(s.to_rfc3339(), "2024-02-01T00:00:00+00:00");
        assert_eq!(e.to_rfc3339(), "2024-03-01T00:00:00+00:00");
        assert!(month_bounds("2026-13").is_none());
        assert!(month_bounds("garbage").is_none());
        assert!(month_bounds("2026").is_none());
    }
}


#[derive(Debug, serde::Serialize)]
pub struct HoldingLeg {
    pub account: String,
    pub qty: f64,
    pub avg_price: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct ConsolidatedHolding {
    pub symbol: String,
    /// Net across accounts — a long here and a short there offset.
    pub net_qty: f64,
    /// Qty-weighted average entry. None when account legs have MIXED
    /// signs: averaging a long's basis with a short's is meaningless,
    /// and the net is a synthetic position nobody entered at any
    /// price. The legs are right there for the real numbers.
    pub weighted_avg_price: Option<f64>,
    pub legs: Vec<HoldingLeg>,
}

/// Pure consolidation of (symbol, account, qty, avg) rows.
pub fn consolidate(rows: &[(String, String, f64, f64)]) -> Vec<ConsolidatedHolding> {
    let mut by_symbol: std::collections::BTreeMap<&str, Vec<&(String, String, f64, f64)>> =
        Default::default();
    for r in rows {
        by_symbol.entry(r.0.as_str()).or_default().push(r);
    }
    by_symbol
        .into_iter()
        .map(|(symbol, legs)| {
            let net_qty: f64 = legs.iter().map(|l| l.2).sum();
            let mixed = legs.iter().any(|l| l.2 > 0.0) && legs.iter().any(|l| l.2 < 0.0);
            let weighted_avg_price = (!mixed && net_qty != 0.0).then(|| {
                legs.iter().map(|l| l.2 * l.3).sum::<f64>() / net_qty
            });
            ConsolidatedHolding {
                symbol: symbol.to_string(),
                net_qty,
                weighted_avg_price,
                legs: legs
                    .iter()
                    .map(|l| HoldingLeg { account: l.1.clone(), qty: l.2, avg_price: l.3 })
                    .collect(),
            }
        })
        .collect()
}

/// Every symbol across ALL the user's paper accounts — the household
/// view the one-account-per-strategy layout otherwise hides.
pub async fn consolidated_holdings(
    pool: &PgPool,
    user_id: Uuid,
) -> anyhow::Result<Vec<ConsolidatedHolding>> {
    use rust_decimal::prelude::ToPrimitive;
    let rows: Vec<(String, String, Decimal, Decimal)> = sqlx::query_as(
        "SELECT p.symbol, a.name, p.qty, p.avg_price
           FROM paper_positions p
           JOIN paper_accounts a ON a.id = p.paper_account_id
          WHERE a.user_id = $1
          ORDER BY p.symbol, a.created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let plain: Vec<(String, String, f64, f64)> = rows
        .into_iter()
        .map(|(s, a, q, p)| (s, a, q.to_f64().unwrap_or(0.0), p.to_f64().unwrap_or(0.0)))
        .collect();
    Ok(consolidate(&plain))
}

#[cfg(test)]
mod holdings_tests {
    use super::*;

    #[test]
    fn consolidation_weights_and_mixed_sign_honesty() {
        let rows = vec![
            ("AAPL".into(), "momo".into(), 100.0, 150.0),
            ("AAPL".into(), "value".into(), 50.0, 120.0),
            ("TSLA".into(), "momo".into(), 100.0, 200.0),
            ("TSLA".into(), "hedge".into(), -40.0, 210.0),
            ("NVDA".into(), "a".into(), 10.0, 500.0),
            ("NVDA".into(), "b".into(), -10.0, 480.0),
        ];
        let out = consolidate(&rows);
        assert_eq!(out.len(), 3); // BTreeMap: AAPL, NVDA, TSLA
        let aapl = &out[0];
        assert_eq!(aapl.net_qty, 150.0);
        // (100×150 + 50×120) / 150 = 140.
        assert!((aapl.weighted_avg_price.unwrap() - 140.0).abs() < 1e-9);
        assert_eq!(aapl.legs.len(), 2);
        // Mixed signs: net stated, average refused — a long's basis
        // averaged with a short's is meaningless.
        let nvda = &out[1];
        assert_eq!(nvda.net_qty, 0.0);
        assert!(nvda.weighted_avg_price.is_none());
        let tsla = &out[2];
        assert_eq!(tsla.net_qty, 60.0);
        assert!(tsla.weighted_avg_price.is_none());
    }
}


/// Reg-T maintenance requirement: marked equity must cover
/// maint_pct of GROSS marked exposure. The classic floor is 25%.
pub const MAINTENANCE_PCT: f64 = 0.25;

/// Pure maintenance check. Gross is Σ|marked value| (shorts count at
/// magnitude — they're exposure, not collateral); equity is cash +
/// SIGNED marks. A flat book trivially passes.
pub fn maintenance_ok(equity: f64, gross: f64, maint_pct: f64) -> bool {
    gross <= 0.0 || equity >= maint_pct * gross
}

/// Forced-liquidation plan: flatten WHOLE positions largest-gross
/// first (broker convention) until maintenance is restored. Closing
/// at the mark is equity-INVARIANT (a position becomes cash at the
/// same value), so each close only shrinks gross — the ratio can
/// only improve, and the loop provably terminates. A blown account
/// (equity ≤ 0) flattens everything: no subset restores it.
/// positions = (symbol, qty, mark_per_unit_with_multiplier_applied).
pub fn liquidation_plan(
    positions: &[(String, f64, f64)],
    cash: f64,
    maint_pct: f64,
) -> Vec<(String, f64)> {
    let mut legs: Vec<(&str, f64, f64)> = positions
        .iter()
        .map(|(s, q, m)| (s.as_str(), *q, q * m))
        .collect();
    // Largest gross exposure first.
    legs.sort_by(|a, b| b.2.abs().total_cmp(&a.2.abs()));
    let signed: f64 = legs.iter().map(|l| l.2).sum();
    let mut gross: f64 = legs.iter().map(|l| l.2.abs()).sum();
    let equity = cash + signed;
    let mut plan = Vec::new();
    for (sym, qty, value) in legs {
        if equity > 0.0 && maintenance_ok(equity, gross, maint_pct) {
            break;
        }
        plan.push((sym.to_string(), qty.abs()));
        gross -= value.abs();
    }
    plan
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MarginCall {
    pub account_id: Uuid,
    pub name: String,
    pub user_id: Uuid,
    pub equity: f64,
    pub gross_exposure: f64,
    pub required: f64,
    /// Marked legs (symbol, qty, mark per unit × multiplier) — the
    /// liquidation planner's input, carried so the watcher doesn't
    /// re-mark.
    pub legs: Vec<(String, f64, f64)>,
    pub cash: f64,
    pub auto_liquidate: bool,
}

/// Accounts below the maintenance requirement at CURRENT marks — the
/// after-the-fill counterpart to the entry-basis initial-margin check
/// in apply_fill. Same marking sources as the equity sampler (cached
/// quotes, chain mids × 100 for OCC); a partially-marked book is
/// SKIPPED, not judged — calling a margin call on stale marks is
/// worse than calling it one pass late.
/// `headroom` scales the trigger: 1.0 flags BREACHED accounts (the
/// watcher's margin call); 1.25 flags accounts within 25% of the
/// line too (the digest's early warning). `required` in the result
/// is always the true 25% figure, not the scaled trigger.
pub async fn margin_calls(pool: &PgPool, headroom: f64) -> anyhow::Result<Vec<MarginCall>> {
    use rust_decimal::prelude::ToPrimitive;
    let accounts: Vec<(Uuid, String, Uuid, Decimal, bool)> = sqlx::query_as(
        "SELECT id, name, user_id, cash, auto_liquidate FROM paper_accounts",
    )
    .fetch_all(pool)
    .await?;
    let mut out = Vec::new();
    for (account_id, name, user_id, cash, auto_liquidate) in accounts {
        let positions: Vec<(String, Decimal)> = sqlx::query_as(
            "SELECT symbol, qty FROM paper_positions WHERE paper_account_id = $1",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
        if positions.is_empty() {
            continue;
        }
        let (mut signed, mut gross) = (Decimal::ZERO, Decimal::ZERO);
        let mut all_marked = true;
        let mut legs: Vec<(String, f64, f64)> = Vec::new();
        for (symbol, qty) in &positions {
            let mark = if let Some(occ) = traderview_core::occ_symbol::parse(symbol) {
                match crate::paper::option_quote(&occ).await {
                    Ok(Some(p)) => Decimal::try_from(p * 100.0).ok(),
                    _ => None,
                }
            } else {
                match crate::market_data::quote(pool, symbol).await {
                    Ok(q) => Decimal::try_from(q.price).ok(),
                    Err(_) => None,
                }
            };
            match mark {
                Some(p) => {
                    signed += p * qty;
                    gross += (p * qty).abs();
                    legs.push((
                        symbol.clone(),
                        qty.to_f64().unwrap_or(0.0),
                        p.to_f64().unwrap_or(0.0),
                    ));
                }
                None => {
                    all_marked = false;
                    break;
                }
            }
        }
        if !all_marked {
            continue;
        }
        let equity = (cash + signed).to_f64().unwrap_or(0.0);
        let gross_f = gross.to_f64().unwrap_or(0.0);
        if !maintenance_ok(equity, gross_f, MAINTENANCE_PCT * headroom.max(1.0)) {
            out.push(MarginCall {
                account_id,
                name,
                user_id,
                equity,
                gross_exposure: gross_f,
                required: MAINTENANCE_PCT * gross_f,
                legs,
                cash: cash.to_f64().unwrap_or(0.0),
                auto_liquidate,
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod maintenance_tests {
    use super::*;

    #[test]
    fn liquidation_plan_largest_first_stops_when_ok() {
        // $5k cash, three longs marked 40k/30k/10k: equity 85k, gross
        // 80k — fine at 25%. Drop cash to −15k: equity 65k... make a
        // failing case: cash −70k → equity 10k, gross 80k, required
        // 20k: breach. Closing the 40k leg leaves gross 40k, required
        // 10k = equity: restored. Plan = exactly the largest leg.
        let pos = vec![
            ("B".to_string(), 300.0, 100.0),  // 30k
            ("A".to_string(), 400.0, 100.0),  // 40k — largest
            ("C".to_string(), 100.0, 100.0),  // 10k
        ];
        let plan = liquidation_plan(&pos, -70_000.0, 0.25);
        assert_eq!(plan, vec![("A".to_string(), 400.0)]);
        // Healthy account: empty plan.
        assert!(liquidation_plan(&pos, 0.0, 0.25).is_empty());
        // Blown account (equity ≤ 0): everything goes, largest first.
        let plan = liquidation_plan(&pos, -90_000.0, 0.25);
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].0, "A");
        // Shorts: qty negative, plan reports |qty| to cover.
        let pos = vec![("S".to_string(), -200.0, 100.0)]; // −20k short
        let plan = liquidation_plan(&pos, 4_000.0, 0.25); // equity −16k? cash 4k + signed −20k = −16k → blown → flatten
        assert_eq!(plan, vec![("S".to_string(), 200.0)]);
    }

    #[test]
    fn maintenance_pins_boundary_and_shorts() {
        // $25k equity supports exactly $100k gross at 25%.
        assert!(maintenance_ok(25_000.0, 100_000.0, 0.25));
        assert!(!maintenance_ok(24_999.0, 100_000.0, 0.25));
        // Shorts are exposure at magnitude: $50k cash, short $40k
        // marked — equity 10k (cash 50k + signed −40k), gross 40k,
        // required 10k: exactly at the line.
        assert!(maintenance_ok(10_000.0, 40_000.0, 0.25));
        assert!(!maintenance_ok(9_999.0, 40_000.0, 0.25));
        // Flat book always passes; negative equity never does with
        // exposure on.
        assert!(maintenance_ok(-5.0, 0.0, 0.25));
        assert!(!maintenance_ok(-5.0, 1.0, 0.25));
    }
}


/// PDT status for one account: trips from the same FIFO fill
/// reconstruction as attribution, equity from the latest snapshot
/// (None when the account has never been sampled — the pure layer
/// refuses to flag on unknown equity).
pub async fn pdt_status(
    pool: &PgPool,
    user_id: Uuid,
    account_id: Uuid,
) -> anyhow::Result<traderview_core::pdt_status::PdtStatus> {
    use rust_decimal::prelude::ToPrimitive;
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM paper_accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if !matches!(owner, Some((u,)) if u == user_id) {
        anyhow::bail!("forbidden");
    }
    let fills: Vec<(String, String, Decimal, Decimal, Decimal, chrono::DateTime<Utc>)> =
        sqlx::query_as(
            "SELECT symbol, side::text, filled_qty, filled_price, fee, filled_at
               FROM paper_orders
              WHERE paper_account_id = $1 AND status = 'filled'
                AND filled_qty IS NOT NULL AND filled_price IS NOT NULL
              ORDER BY filled_at",
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
    let mut by_symbol: std::collections::BTreeMap<String, Vec<traderview_core::live_vs_backtest::Fill>> =
        Default::default();
    for (symbol, side, qty, price, fee, at) in fills {
        by_symbol.entry(symbol).or_default().push(traderview_core::live_vs_backtest::Fill {
            buy: side == "buy" || side == "cover",
            qty: qty.to_f64().unwrap_or(0.0),
            price: price.to_f64().unwrap_or(0.0),
            commission: fee.to_f64().unwrap_or(0.0),
            ts: at.timestamp(),
            flag: false,
        });
    }
    let mut trips: Vec<(i64, i64)> = Vec::new();
    for fills in by_symbol.values() {
        trips.extend(
            traderview_core::live_vs_backtest::round_trips(fills)
                .iter()
                .map(|t| (t.opened_ts, t.closed_ts)),
        );
    }
    let equity: Option<(Decimal,)> = sqlx::query_as(
        "SELECT equity FROM paper_equity_snapshots
          WHERE paper_account_id = $1 ORDER BY taken_at DESC LIMIT 1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;
    Ok(traderview_core::pdt_status::pdt_status(
        &trips,
        equity.and_then(|(e,)| e.to_f64()),
        Utc::now().date_naive(),
    ))
}
