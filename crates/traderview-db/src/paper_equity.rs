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
    pub return_pct: f64,
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
            return_pct: account_return_pct(starting_cash, equity),
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
    let mut by_symbol: std::collections::BTreeMap<String, (Vec<traderview_core::live_vs_backtest::Fill>, f64)> =
        Default::default();
    for (symbol, side, qty, price, fee, at) in fills {
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
    for (symbol, (fills, fees)) in &by_symbol {
        let trips = traderview_core::live_vs_backtest::round_trips(fills);
        all_trips.extend(trips.iter().map(|t| (t.pnl, t.closed_ts)));
        all_holds.extend(trips.iter().map(|t| (t.pnl, t.closed_ts - t.opened_ts)));
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
    Ok(Attribution {
        symbols,
        months,
        stats,
        hold,
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
    // Gross-exposure weights: shorts contribute their SIGNED return
    // weighted by signed notional over GROSS book, so a hedged book
    // shows reduced VaR rather than inflated.
    let gross: f64 = notionals.iter().map(|n| n.abs()).sum();
    if gross <= 0.0 {
        anyhow::bail!("zero book value");
    }
    let weights: Vec<f64> = notionals.iter().map(|n| n / gross).collect();
    let port = weighted_portfolio_returns(&weights, &all_returns);
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
    let summary = summarize(&series);
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
