//! Strategy calculators:
//!
//! * Grid trading — given a price range, grid count, and capital,
//!   computes the ladder levels (arithmetic or geometric spacing) and
//!   the per-grid round-trip profit after fees. Pure compute.
//! * Fixed-ratio position sizing (Ryan Jones) — equity thresholds at
//!   which the trader steps from N to N+1 contracts using a fixed
//!   delta per contract. Pure compute.
//! * Turn-of-month seasonality — average daily return + hit rate by
//!   trading-day offset around the month boundary (last 4 days = -4..-1,
//!   first 3 days = +1..+3) vs all remaining days, on real daily bars.

use chrono::{Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use traderview_core::BarInterval;

// ===========================================================================
// Grid trading
// ===========================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct GridInput {
    pub lower_price: f64,
    pub upper_price: f64,
    /// Number of grid intervals (the ladder has grid_count + 1 levels).
    pub grid_count: u32,
    pub total_capital: f64,
    /// Exchange fee per side, % of notional.
    pub fee_pct: f64,
    /// false = arithmetic spacing, true = geometric (equal % steps).
    pub geometric: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GridReport {
    /// Ladder prices, lowest first (grid_count + 1 entries).
    pub levels: Vec<f64>,
    pub range_pct: f64,
    pub capital_per_grid: f64,
    /// Round-trip profit per grid after both fee legs, % — constant for
    /// geometric spacing, varies by level for arithmetic.
    pub profit_per_grid_min_pct: f64,
    pub profit_per_grid_max_pct: f64,
    pub profit_per_grid_min_usd: f64,
    pub profit_per_grid_max_usd: f64,
    /// True when fees eat the whole step on at least one grid.
    pub any_grid_unprofitable: bool,
}

pub fn grid_trading(input: &GridInput) -> Result<GridReport, &'static str> {
    if input.lower_price <= 0.0 {
        return Err("lower_price must be positive");
    }
    if input.upper_price <= input.lower_price {
        return Err("upper_price must exceed lower_price");
    }
    if !(2..=200).contains(&input.grid_count) {
        return Err("grid_count must be in 2..=200");
    }
    if input.total_capital <= 0.0 {
        return Err("total_capital must be positive");
    }
    if input.fee_pct < 0.0 {
        return Err("fee_pct must be >= 0");
    }
    let n = input.grid_count as usize;
    let levels: Vec<f64> = if input.geometric {
        let r = (input.upper_price / input.lower_price).powf(1.0 / n as f64);
        (0..=n).map(|i| input.lower_price * r.powi(i as i32)).collect()
    } else {
        let step = (input.upper_price - input.lower_price) / n as f64;
        (0..=n).map(|i| input.lower_price + step * i as f64).collect()
    };
    let capital_per_grid = input.total_capital / n as f64;
    // Round trip on one grid: buy at levels[i], sell at levels[i+1];
    // fees hit both legs.
    let mut min_pct = f64::MAX;
    let mut max_pct = f64::MIN;
    for w in levels.windows(2) {
        let gross = (w[1] / w[0] - 1.0) * 100.0;
        let net = gross - 2.0 * input.fee_pct;
        min_pct = min_pct.min(net);
        max_pct = max_pct.max(net);
    }
    Ok(GridReport {
        range_pct: (input.upper_price / input.lower_price - 1.0) * 100.0,
        capital_per_grid,
        profit_per_grid_min_pct: min_pct,
        profit_per_grid_max_pct: max_pct,
        profit_per_grid_min_usd: capital_per_grid * min_pct / 100.0,
        profit_per_grid_max_usd: capital_per_grid * max_pct / 100.0,
        any_grid_unprofitable: min_pct <= 0.0,
        levels,
    })
}

// ===========================================================================
// Fixed-ratio position sizing (Ryan Jones)
// ===========================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct FixedRatioInput {
    pub starting_capital: f64,
    /// Profit per contract required to add the next contract, $.
    pub delta: f64,
    /// Table depth (clamped 1..=100).
    pub max_contracts: u32,
    /// Optional expected profit per trade per contract, $ — fills the
    /// estimated-trades column when positive.
    #[serde(default)]
    pub profit_per_trade_per_contract: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FixedRatioRow {
    pub contracts: u32,
    /// Equity at which this contract count unlocks.
    pub equity_required: f64,
    /// Gain needed from the previous level.
    pub gain_from_prev: f64,
    /// Estimated trades to clear this level from the previous one
    /// (None when no per-trade profit was supplied).
    pub est_trades_from_prev: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FixedRatioReport {
    pub rows: Vec<FixedRatioRow>,
    /// Total gain needed to reach max_contracts from start.
    pub total_gain_to_max: f64,
}

pub fn fixed_ratio(input: &FixedRatioInput) -> Result<FixedRatioReport, &'static str> {
    if input.starting_capital <= 0.0 {
        return Err("starting_capital must be positive");
    }
    if input.delta <= 0.0 {
        return Err("delta must be positive");
    }
    let max = input.max_contracts.clamp(1, 100);
    let mut rows = Vec::with_capacity(max as usize);
    for n in 1..=max {
        // Threshold to TRADE n contracts: start + delta × Σ_{k=1}^{n-1} k.
        let steps = (n - 1) as f64;
        let equity = input.starting_capital + input.delta * steps * (steps + 1.0) / 2.0;
        let gain = if n == 1 { 0.0 } else { input.delta * (n - 1) as f64 };
        let est = (input.profit_per_trade_per_contract > 0.0 && n > 1).then(|| {
            // While trading n-1 contracts, each trade earns (n-1) × per-contract.
            gain / (input.profit_per_trade_per_contract * (n - 1) as f64)
        });
        rows.push(FixedRatioRow {
            contracts: n,
            equity_required: equity,
            gain_from_prev: gain,
            est_trades_from_prev: est,
        });
    }
    let total = rows.last().map(|r| r.equity_required).unwrap_or(0.0) - input.starting_capital;
    Ok(FixedRatioReport {
        rows,
        total_gain_to_max: total,
    })
}

// ===========================================================================
// Anti-martingale streak sizing
// ===========================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct AntiMartingaleInput {
    pub starting_capital: f64,
    /// Risk on the first trade, % of current equity.
    pub base_risk_pct: f64,
    /// Multiplier applied to risk after a WIN (>1 presses winners).
    pub win_factor: f64,
    /// Multiplier applied to risk after a LOSS (<1 cuts back).
    pub loss_factor: f64,
    /// Hard cap on risk per trade, % of equity.
    pub max_risk_pct: f64,
    /// Reward-to-risk per winning trade (R multiple, e.g. 1.5).
    pub win_payoff_r: f64,
    /// Trade outcomes oldest-first, e.g. "WWLWLLW".
    pub sequence: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AntiMartingaleRow {
    pub trade: usize,
    pub outcome: char,
    pub risk_pct: f64,
    pub pnl: f64,
    pub equity: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AntiMartingaleReport {
    pub rows: Vec<AntiMartingaleRow>,
    pub final_equity: f64,
    pub total_return_pct: f64,
    /// Same sequence traded at flat base_risk_pct — the control arm.
    pub fixed_risk_final_equity: f64,
    pub fixed_risk_return_pct: f64,
    pub max_drawdown_pct: f64,
}

pub fn anti_martingale(
    input: &AntiMartingaleInput,
) -> Result<AntiMartingaleReport, &'static str> {
    if input.starting_capital <= 0.0 {
        return Err("starting_capital must be positive");
    }
    if input.base_risk_pct <= 0.0 || input.base_risk_pct > 100.0 {
        return Err("base_risk_pct must be in (0, 100]");
    }
    if input.win_factor <= 0.0 || input.loss_factor <= 0.0 {
        return Err("win_factor and loss_factor must be positive");
    }
    if input.max_risk_pct < input.base_risk_pct {
        return Err("max_risk_pct must be >= base_risk_pct");
    }
    if input.win_payoff_r <= 0.0 {
        return Err("win_payoff_r must be positive");
    }
    let seq: Vec<char> = input
        .sequence
        .trim()
        .to_uppercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if seq.is_empty() || seq.len() > 500 {
        return Err("sequence must have 1..=500 outcomes");
    }
    if seq.iter().any(|c| *c != 'W' && *c != 'L') {
        return Err("sequence may only contain W and L");
    }
    let mut equity = input.starting_capital;
    let mut fixed = input.starting_capital;
    let mut risk_pct = input.base_risk_pct;
    let mut peak = equity;
    let mut max_dd = 0.0_f64;
    let mut rows = Vec::with_capacity(seq.len());
    for (i, &c) in seq.iter().enumerate() {
        let risked = equity * risk_pct / 100.0;
        let fixed_risked = fixed * input.base_risk_pct / 100.0;
        let pnl = if c == 'W' {
            risked * input.win_payoff_r
        } else {
            -risked
        };
        equity += pnl;
        fixed += if c == 'W' {
            fixed_risked * input.win_payoff_r
        } else {
            -fixed_risked
        };
        peak = peak.max(equity);
        if peak > 0.0 {
            max_dd = max_dd.max((peak - equity) / peak * 100.0);
        }
        rows.push(AntiMartingaleRow {
            trade: i + 1,
            outcome: c,
            risk_pct,
            pnl,
            equity,
        });
        // Anti-martingale: press after wins, cut after losses,
        // clamped to [base, max].
        risk_pct = if c == 'W' {
            (risk_pct * input.win_factor).min(input.max_risk_pct)
        } else {
            (risk_pct * input.loss_factor).max(input.base_risk_pct).min(input.max_risk_pct)
        };
    }
    Ok(AntiMartingaleReport {
        final_equity: equity,
        total_return_pct: (equity / input.starting_capital - 1.0) * 100.0,
        fixed_risk_final_equity: fixed,
        fixed_risk_return_pct: (fixed / input.starting_capital - 1.0) * 100.0,
        max_drawdown_pct: max_dd,
        rows,
    })
}

// ===========================================================================
// Turn-of-month seasonality
// ===========================================================================

const TOM_LAST_DAYS: usize = 4; // offsets -4..-1
const TOM_FIRST_DAYS: usize = 3; // offsets +1..+3
const MIN_DAYS: usize = 252;

#[derive(Debug, Clone, Serialize)]
pub struct TomOffsetRow {
    /// Trading-day offset: -1 = last day of month, +1 = first day.
    pub offset: i32,
    pub avg_return_pct: f64,
    pub hit_rate_pct: f64,
    pub n: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TomReport {
    pub symbol: String,
    pub days_analyzed: usize,
    pub rows: Vec<TomOffsetRow>,
    /// Mean daily return inside the TOM window vs all other days.
    pub tom_avg_return_pct: f64,
    pub rest_avg_return_pct: f64,
    pub edge_pct: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum TomError {
    #[error("not enough daily closes for {symbol}: got {got}, need {need}")]
    Insufficient {
        symbol: String,
        got: usize,
        need: usize,
    },
    #[error("price fetch failed: {0}")]
    PriceFetch(anyhow::Error),
}

/// Pure core over (date, close) pairs, oldest→newest — unit-testable
/// without a database.
pub fn tom_stats(symbol: &str, closes: &[(chrono::NaiveDate, f64)]) -> TomReport {
    // Group indices by calendar month so each day gets a within-month
    // trading-day position.
    let mut months: Vec<Vec<usize>> = Vec::new();
    let mut cur_key = (0, 0);
    for (i, (d, _)) in closes.iter().enumerate() {
        let key = (d.year(), d.month());
        if key != cur_key {
            months.push(Vec::new());
            cur_key = key;
        }
        months.last_mut().expect("pushed above").push(i);
    }
    // offset for bar index i (None = "rest").
    let mut offsets: std::collections::HashMap<usize, i32> = std::collections::HashMap::new();
    for m in &months {
        let len = m.len();
        for (pos, &i) in m.iter().enumerate() {
            if pos < TOM_FIRST_DAYS {
                offsets.insert(i, pos as i32 + 1);
            } else if len - pos <= TOM_LAST_DAYS {
                offsets.insert(i, -((len - pos) as i32));
            }
        }
    }
    let mut by_offset: std::collections::BTreeMap<i32, Vec<f64>> = std::collections::BTreeMap::new();
    let mut tom: Vec<f64> = Vec::new();
    let mut rest: Vec<f64> = Vec::new();
    for i in 1..closes.len() {
        let (p0, p1) = (closes[i - 1].1, closes[i].1);
        if p0 <= 0.0 {
            continue;
        }
        let r = (p1 / p0 - 1.0) * 100.0;
        match offsets.get(&i) {
            Some(&off) => {
                by_offset.entry(off).or_default().push(r);
                tom.push(r);
            }
            None => rest.push(r),
        }
    }
    let mean = |v: &[f64]| {
        if v.is_empty() {
            0.0
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    };
    let rows = by_offset
        .into_iter()
        .map(|(offset, rets)| TomOffsetRow {
            offset,
            avg_return_pct: mean(&rets),
            hit_rate_pct: rets.iter().filter(|r| **r > 0.0).count() as f64 / rets.len() as f64
                * 100.0,
            n: rets.len(),
        })
        .collect();
    let tom_avg = mean(&tom);
    let rest_avg = mean(&rest);
    TomReport {
        symbol: symbol.to_string(),
        days_analyzed: closes.len(),
        rows,
        tom_avg_return_pct: tom_avg,
        rest_avg_return_pct: rest_avg,
        edge_pct: tom_avg - rest_avg,
    }
}

// ===========================================================================
// Day-of-week seasonality (data wrapper around
// traderview_core::day_of_week_seasonality — the stats live there)
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct DowReport {
    pub symbol: String,
    pub days_analyzed: usize,
    #[serde(flatten)]
    pub stats: traderview_core::day_of_week_seasonality::DayOfWeekSeasonalityReport,
}

pub async fn day_of_week(
    pool: &PgPool,
    symbol: &str,
    years: u32,
) -> Result<DowReport, TomError> {
    use traderview_core::day_of_week_seasonality::{self as dow, DailyClose};
    let closes = daily_closes(pool, symbol, years).await?;
    let tagged: Vec<DailyClose> = closes
        .iter()
        .filter_map(|(d, c)| {
            let wd = d.weekday().num_days_from_monday() as u8 + 1; // 1=Mon
            (wd <= 5).then_some(DailyClose {
                day_of_week: wd,
                close: *c,
            })
        })
        .collect();
    let stats = dow::compute(&tagged).ok_or_else(|| TomError::Insufficient {
        symbol: symbol.to_string(),
        got: tagged.len(),
        need: MIN_DAYS,
    })?;
    Ok(DowReport {
        symbol: symbol.to_string(),
        days_analyzed: tagged.len(),
        stats,
    })
}

// ===========================================================================
// Santa Claus rally (data wrapper around
// traderview_core::holiday_seasonality — per-offset stats live there)
// ===========================================================================

/// Last 4 trading days of December + the anchor (last Dec session) +
/// first 2 of January — Hirsch's classic 7-session window.
const SANTA_BEFORE: u32 = 4;
const SANTA_AFTER: u32 = 2;

#[derive(Debug, Clone, Serialize)]
pub struct SantaYearRow {
    pub year: i32,
    pub window_return_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SantaReport {
    pub symbol: String,
    pub years_analyzed: usize,
    /// Mean cumulative return over the 7-session window across years.
    pub rally_avg_return_pct: f64,
    pub rally_hit_rate_pct: f64,
    pub yearly: Vec<SantaYearRow>,
    /// Per-offset stats from holiday_seasonality (offset 0 = last
    /// December session).
    pub offsets: traderview_core::holiday_seasonality::HolidaySeasonalityReport,
}

/// Pure core over (date, close) pairs, oldest→newest.
pub fn santa_stats(symbol: &str, closes: &[(chrono::NaiveDate, f64)]) -> Option<SantaReport> {
    use traderview_core::holiday_seasonality::{self, TradingDay};
    // Anchor = index of the LAST trading day of each December.
    let mut anchors: Vec<usize> = Vec::new();
    for (i, (d, _)) in closes.iter().enumerate() {
        if d.month() == 12 {
            let next_is_new_month = closes
                .get(i + 1)
                .map(|(nd, _)| nd.month() != 12)
                .unwrap_or(false);
            if next_is_new_month {
                anchors.push(i);
            }
        }
    }
    // Per-year cumulative window return — needs the full window on
    // both sides; partial years are skipped, not approximated.
    let before = SANTA_BEFORE as usize;
    let after = SANTA_AFTER as usize;
    let mut yearly: Vec<SantaYearRow> = Vec::new();
    for &i in &anchors {
        if i < before + 1 || i + after >= closes.len() {
            continue;
        }
        let base = closes[i - before - 1].1;
        let end = closes[i + after].1;
        if base <= 0.0 {
            continue;
        }
        yearly.push(SantaYearRow {
            year: closes[i].0.year(),
            window_return_pct: (end / base - 1.0) * 100.0,
        });
    }
    if yearly.is_empty() {
        return None;
    }
    let avg = yearly.iter().map(|y| y.window_return_pct).sum::<f64>() / yearly.len() as f64;
    let hits = yearly.iter().filter(|y| y.window_return_pct > 0.0).count();
    // Per-offset stats via the existing holiday_seasonality core.
    let days: Vec<TradingDay> = closes
        .iter()
        .enumerate()
        .map(|(i, (_, c))| TradingDay {
            trading_day_index: i as u32,
            close: *c,
        })
        .collect();
    let anchor_indices: Vec<u32> = anchors.iter().map(|i| *i as u32).collect();
    let offsets =
        holiday_seasonality::compute(&days, &anchor_indices, SANTA_BEFORE, SANTA_AFTER)?;
    Some(SantaReport {
        symbol: symbol.to_string(),
        years_analyzed: yearly.len(),
        rally_avg_return_pct: avg,
        rally_hit_rate_pct: hits as f64 / yearly.len() as f64 * 100.0,
        yearly,
        offsets,
    })
}

pub async fn santa_rally(
    pool: &PgPool,
    symbol: &str,
    years: u32,
) -> Result<SantaReport, TomError> {
    let closes = daily_closes(pool, symbol, years).await?;
    santa_stats(symbol, &closes).ok_or_else(|| TomError::Insufficient {
        symbol: symbol.to_string(),
        got: closes.len(),
        need: MIN_DAYS,
    })
}

// ===========================================================================
// Correlation regime (data wrapper around
// traderview_core::correlation_regime)
// ===========================================================================

pub async fn correlation_regime(
    pool: &PgPool,
    symbol_a: &str,
    symbol_b: &str,
    window: usize,
    years: u32,
) -> Result<traderview_core::correlation_regime::CorrelationRegimeReport, TomError> {
    let ca = daily_closes(pool, symbol_a, years).await?;
    let cb = daily_closes(pool, symbol_b, years).await?;
    // Inner-join on date so holidays/listing gaps drop everywhere.
    let by_date: std::collections::BTreeMap<chrono::NaiveDate, f64> = cb.into_iter().collect();
    let mut a = Vec::new();
    let mut b = Vec::new();
    for (d, c) in &ca {
        if let Some(x) = by_date.get(d) {
            a.push(*c);
            b.push(*x);
        }
    }
    traderview_core::correlation_regime::compute(&a, &b, window).ok_or_else(|| {
        TomError::Insufficient {
            symbol: format!("{symbol_a}/{symbol_b}"),
            got: a.len(),
            need: window + 1,
        }
    })
}

// ===========================================================================
// Overnight vs intraday split (data wrapper around
// traderview_core::overnight_intraday)
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct OvernightSplitReport {
    pub symbol: String,
    #[serde(flatten)]
    pub stats: traderview_core::overnight_intraday::OvernightReport,
}

pub async fn overnight_split(
    pool: &PgPool,
    symbol: &str,
    years: u32,
) -> Result<OvernightSplitReport, TomError> {
    let years = years.clamp(1, 20);
    let to = Utc::now();
    let from = to - Duration::days(366 * years as i64);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .map_err(TomError::PriceFetch)?;
    let pairs: Vec<(f64, f64)> = bars
        .iter()
        .filter_map(|b| {
            let open: f64 = b.open.to_string().parse().unwrap_or(0.0);
            let close: f64 = b.close.to_string().parse().unwrap_or(0.0);
            (open > 0.0 && close > 0.0).then_some((open, close))
        })
        .collect();
    if pairs.len() < MIN_DAYS {
        return Err(TomError::Insufficient {
            symbol: symbol.to_string(),
            got: pairs.len(),
            need: MIN_DAYS,
        });
    }
    traderview_core::overnight_intraday::compute(&pairs)
        .map(|stats| OvernightSplitReport {
            symbol: symbol.to_string(),
            stats,
        })
        .ok_or_else(|| TomError::Insufficient {
            symbol: symbol.to_string(),
            got: pairs.len(),
            need: MIN_DAYS,
        })
}

// ===========================================================================
// Best/worst-days concentration + drawdown episodes (data wrappers)
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct ConcentrationSymbolReport {
    pub symbol: String,
    #[serde(flatten)]
    pub stats: traderview_core::best_worst_days::ConcentrationReport,
}

pub async fn best_worst_days(
    pool: &PgPool,
    symbol: &str,
    years: u32,
    n: usize,
) -> Result<ConcentrationSymbolReport, TomError> {
    let closes = daily_closes(pool, symbol, years).await?;
    let series: Vec<f64> = closes.iter().map(|(_, c)| *c).collect();
    traderview_core::best_worst_days::compute(&series, n.clamp(1, 50))
        .map(|stats| ConcentrationSymbolReport {
            symbol: symbol.to_string(),
            stats,
        })
        .ok_or_else(|| TomError::Insufficient {
            symbol: symbol.to_string(),
            got: series.len(),
            need: MIN_DAYS,
        })
}

#[derive(Debug, Clone, Serialize)]
pub struct EpisodesSymbolReport {
    pub symbol: String,
    /// Dates resolved from episode indices, worst-first.
    pub rows: Vec<EpisodeRow>,
    pub currently_underwater: bool,
    pub current_drawdown_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EpisodeRow {
    pub peak_date: chrono::NaiveDate,
    pub trough_date: chrono::NaiveDate,
    pub depth_pct: f64,
    pub decline_bars: usize,
    pub recovery_bars: Option<usize>,
}

pub async fn drawdown_episodes(
    pool: &PgPool,
    symbol: &str,
    years: u32,
    top_n: usize,
) -> Result<EpisodesSymbolReport, TomError> {
    let closes = daily_closes(pool, symbol, years).await?;
    let series: Vec<f64> = closes.iter().map(|(_, c)| *c).collect();
    let report = traderview_core::drawdown_episodes::compute(&series, top_n.clamp(1, 25))
        .ok_or_else(|| TomError::Insufficient {
            symbol: symbol.to_string(),
            got: series.len(),
            need: MIN_DAYS,
        })?;
    let rows = report
        .episodes
        .iter()
        .map(|e| EpisodeRow {
            peak_date: closes[e.peak_index].0,
            trough_date: closes[e.trough_index].0,
            depth_pct: e.depth_pct,
            decline_bars: e.decline_bars,
            recovery_bars: e.recovery_bars,
        })
        .collect();
    Ok(EpisodesSymbolReport {
        symbol: symbol.to_string(),
        rows,
        currently_underwater: report.currently_underwater,
        current_drawdown_pct: report.current_drawdown_pct,
    })
}

// ===========================================================================
// Event-day studies (FOMC/CPI via caller dates, OpEx via computed
// third Fridays) — per-offset stats delegate to holiday_seasonality
// ===========================================================================

/// Third Friday of a month — the standard monthly equity/index option
/// expiration anchor. Pure calendar math, no hardcoded dates.
pub fn third_friday(year: i32, month: u32) -> Option<chrono::NaiveDate> {
    let first = chrono::NaiveDate::from_ymd_opt(year, month, 1)?;
    let days_to_friday =
        (chrono::Weekday::Fri.num_days_from_monday() + 7 - first.weekday().num_days_from_monday())
            % 7;
    first.checked_add_days(chrono::Days::new(days_to_friday as u64 + 14))
}

/// Index of the last trading day at or before `target` (None when the
/// target precedes the sample).
fn index_at_or_before(closes: &[(chrono::NaiveDate, f64)], target: chrono::NaiveDate) -> Option<usize> {
    match closes.binary_search_by(|(d, _)| d.cmp(&target)) {
        Ok(i) => Some(i),
        Err(0) => None,
        Err(i) => Some(i - 1),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventStudyReport {
    pub symbol: String,
    pub events_used: usize,
    pub events_supplied: usize,
    /// Mean return and hit rate ON the event day itself, %.
    pub event_day_avg_pct: f64,
    pub event_day_hit_rate_pct: f64,
    /// Per-offset stats from holiday_seasonality (offset 0 = event day).
    pub offsets: traderview_core::holiday_seasonality::HolidaySeasonalityReport,
}

pub async fn event_study(
    pool: &PgPool,
    symbol: &str,
    years: u32,
    event_dates: &[chrono::NaiveDate],
    window_before: u32,
    window_after: u32,
) -> Result<EventStudyReport, TomError> {
    use traderview_core::holiday_seasonality::{self, TradingDay};
    let closes = daily_closes(pool, symbol, years).await?;
    let anchors: Vec<u32> = event_dates
        .iter()
        .filter_map(|d| index_at_or_before(&closes, *d))
        .filter(|i| *i > 0)
        .map(|i| i as u32)
        .collect();
    if anchors.is_empty() {
        return Err(TomError::Insufficient {
            symbol: symbol.to_string(),
            got: 0,
            need: 1,
        });
    }
    let days: Vec<TradingDay> = closes
        .iter()
        .enumerate()
        .map(|(i, (_, c))| TradingDay {
            trading_day_index: i as u32,
            close: *c,
        })
        .collect();
    let offsets = holiday_seasonality::compute(&days, &anchors, window_before, window_after)
        .ok_or_else(|| TomError::Insufficient {
            symbol: symbol.to_string(),
            got: closes.len(),
            need: MIN_DAYS,
        })?;
    // Event-day simple returns (offset 0 in simple terms, not log).
    let rets: Vec<f64> = anchors
        .iter()
        .filter_map(|&i| {
            let i = i as usize;
            let p0 = closes[i - 1].1;
            (p0 > 0.0).then(|| (closes[i].1 / p0 - 1.0) * 100.0)
        })
        .collect();
    let n = rets.len().max(1) as f64;
    Ok(EventStudyReport {
        symbol: symbol.to_string(),
        events_used: anchors.len(),
        events_supplied: event_dates.len(),
        event_day_avg_pct: rets.iter().sum::<f64>() / n,
        event_day_hit_rate_pct: rets.iter().filter(|r| **r > 0.0).count() as f64 / n * 100.0,
        offsets,
    })
}

/// Pre-holiday drift study — anchors at each market holiday (the
/// event-study snap lands on the last session BEFORE it). The embedded
/// calendar covers 2024+, so events_used reflects the covered slice of
/// the lookback, not all of it.
pub async fn pre_holiday(
    pool: &PgPool,
    symbol: &str,
    years: u32,
) -> Result<EventStudyReport, TomError> {
    let years = years.clamp(1, 20);
    let now = Utc::now().date_naive();
    let from = now - chrono::Duration::days(366 * years as i64);
    let anchors = traderview_core::holiday_calendar::holidays_in_range(from, now);
    event_study(pool, symbol, years, &anchors, 3, 2).await
}

/// Monthly OpEx study; `quarterly` restricts to the Mar/Jun/Sep/Dec
/// triple-witching expirations.
pub async fn opex_week(
    pool: &PgPool,
    symbol: &str,
    years: u32,
    quarterly: bool,
) -> Result<EventStudyReport, TomError> {
    let years = years.clamp(1, 20);
    let now = Utc::now().date_naive();
    let mut anchors = Vec::new();
    for back in 0..=(years * 12) {
        let months_total = now.year() * 12 + now.month() as i32 - 1 - back as i32;
        let (y, m) = (months_total.div_euclid(12), months_total.rem_euclid(12) as u32 + 1);
        if quarterly && m % 3 != 0 {
            continue;
        }
        if let Some(d) = third_friday(y, m) {
            if d <= now {
                anchors.push(d);
            }
        }
    }
    event_study(pool, symbol, years, &anchors, 4, 2).await
}

/// Ex-dividend behavior study — anchors at each historical ex-date
/// (Yahoo dividend events), answering the dividend-capture question:
/// does the price recover the drop, and how fast?
pub async fn ex_div_study(
    pool: &PgPool,
    symbol: &str,
    years: u32,
) -> Result<EventStudyReport, TomError> {
    let years = years.clamp(1, 20);
    let now = Utc::now().date_naive();
    let from = now - chrono::Duration::days(366 * years as i64);
    let (events, _) = crate::dividend_tracker::fetch_dividend_events(symbol).await;
    let anchors: Vec<chrono::NaiveDate> = events
        .iter()
        .map(|e| e.ex_date)
        .filter(|d| *d >= from && *d <= now)
        .collect();
    if anchors.is_empty() {
        return Err(TomError::Insufficient {
            symbol: symbol.to_string(),
            got: 0,
            need: 1,
        });
    }
    event_study(pool, symbol, years, &anchors, 3, 3).await
}

// ===========================================================================
// Symbol character sheet — ONE bar fetch, every pure seasonal/vol/
// drawdown analysis in this module run over it
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct CharacterSheet {
    pub symbol: String,
    pub days_analyzed: usize,
    pub turn_of_month: Option<TomReport>,
    pub day_of_week: Option<traderview_core::day_of_week_seasonality::DayOfWeekSeasonalityReport>,
    pub santa: Option<SantaReport>,
    pub overnight: Option<traderview_core::overnight_intraday::OvernightReport>,
    pub vol_cone: Vec<traderview_core::vol_cone::VolConeRow>,
    pub drawdowns: Option<traderview_core::drawdown_episodes::EpisodesReport>,
    pub concentration: Option<traderview_core::best_worst_days::ConcentrationReport>,
}

/// Every leg is Option — a symbol with enough history for the cone but
/// not for Santa gets partial results, never errors or fakes.
pub async fn character_sheet(
    pool: &PgPool,
    symbol: &str,
    years: u32,
) -> Result<CharacterSheet, TomError> {
    use traderview_core::day_of_week_seasonality::{self as dow, DailyClose};
    let years = years.clamp(1, 20);
    let to = Utc::now();
    let from = to - Duration::days(366 * years as i64);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .map_err(TomError::PriceFetch)?;
    let dated: Vec<(chrono::NaiveDate, f64)> = bars
        .iter()
        .filter_map(|b| {
            let close: f64 = b.close.to_string().parse().unwrap_or(0.0);
            (close > 0.0).then(|| (b.bar_time.date_naive(), close))
        })
        .collect();
    if dated.len() < MIN_DAYS {
        return Err(TomError::Insufficient {
            symbol: symbol.to_string(),
            got: dated.len(),
            need: MIN_DAYS,
        });
    }
    let closes: Vec<f64> = dated.iter().map(|(_, c)| *c).collect();
    let oc: Vec<(f64, f64)> = bars
        .iter()
        .filter_map(|b| {
            let open: f64 = b.open.to_string().parse().unwrap_or(0.0);
            let close: f64 = b.close.to_string().parse().unwrap_or(0.0);
            (open > 0.0 && close > 0.0).then_some((open, close))
        })
        .collect();
    let tagged: Vec<DailyClose> = dated
        .iter()
        .filter_map(|(d, c)| {
            let wd = d.weekday().num_days_from_monday() as u8 + 1;
            (wd <= 5).then_some(DailyClose {
                day_of_week: wd,
                close: *c,
            })
        })
        .collect();
    Ok(CharacterSheet {
        symbol: symbol.to_string(),
        days_analyzed: dated.len(),
        turn_of_month: Some(tom_stats(symbol, &dated)),
        day_of_week: dow::compute(&tagged),
        santa: santa_stats(symbol, &dated),
        overnight: traderview_core::overnight_intraday::compute(&oc),
        vol_cone: traderview_core::vol_cone::compute(&closes, VOL_CONE_HORIZONS),
        drawdowns: traderview_core::drawdown_episodes::compute(&closes, 3),
        concentration: traderview_core::best_worst_days::compute(&closes, 10),
    })
}

// ===========================================================================
// Vol rich/cheap — per-horizon IV vs realized (vol_cone ×
// variance_risk_premium composition)
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct VolRichCheapRow {
    pub days: usize,
    pub iv_pct: f64,
    pub realized_pct: f64,
    /// Where current realized sits in its own history at this horizon.
    pub realized_rank_pct: f64,
    pub vrp_variance_points: f64,
    pub vol_spread_pct: f64,
    pub premium_regime: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct VolRichCheapReport {
    pub symbol: String,
    pub days_analyzed: usize,
    pub rows: Vec<VolRichCheapRow>,
}

/// `term` = (horizon trading days, implied vol %) pairs from the
/// live chain; realized legs come from the cone over real closes.
pub async fn vol_rich_cheap(
    pool: &PgPool,
    symbol: &str,
    years: u32,
    term: &[(usize, f64)],
) -> Result<VolRichCheapReport, TomError> {
    let closes = daily_closes(pool, symbol, years).await?;
    let series: Vec<f64> = closes.iter().map(|(_, c)| *c).collect();
    let horizons: Vec<usize> = term.iter().map(|(d, _)| *d).collect();
    let cone = traderview_core::vol_cone::compute(&series, &horizons);
    let mut rows = Vec::with_capacity(term.len());
    for &(days, iv) in term {
        let Some(cone_row) = cone.iter().find(|r| r.horizon_days == days) else {
            continue; // horizon longer than the sample — skip, don't fake
        };
        let Some(vrp) =
            traderview_core::variance_risk_premium::compute(iv, cone_row.current_pct)
        else {
            continue;
        };
        rows.push(VolRichCheapRow {
            days,
            iv_pct: iv,
            realized_pct: cone_row.current_pct,
            realized_rank_pct: cone_row.current_rank_pct,
            vrp_variance_points: vrp.vrp_variance_points,
            vol_spread_pct: vrp.vol_spread_pct,
            premium_regime: vrp.premium_regime,
        });
    }
    if rows.is_empty() {
        return Err(TomError::Insufficient {
            symbol: symbol.to_string(),
            got: series.len(),
            need: MIN_DAYS,
        });
    }
    Ok(VolRichCheapReport {
        symbol: symbol.to_string(),
        days_analyzed: series.len(),
        rows,
    })
}

// ===========================================================================
// Volatility cone (data wrapper around traderview_core::vol_cone)
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct VolConeReport {
    pub symbol: String,
    pub days_analyzed: usize,
    pub rows: Vec<traderview_core::vol_cone::VolConeRow>,
}

/// Standard option-desk horizons: 1w / 2w / 1m / 2m / 3m / 6m.
const VOL_CONE_HORIZONS: &[usize] = &[5, 10, 21, 42, 63, 126];

pub async fn vol_cone(pool: &PgPool, symbol: &str, years: u32) -> Result<VolConeReport, TomError> {
    let closes = daily_closes(pool, symbol, years).await?;
    let series: Vec<f64> = closes.iter().map(|(_, c)| *c).collect();
    Ok(VolConeReport {
        symbol: symbol.to_string(),
        days_analyzed: series.len(),
        rows: traderview_core::vol_cone::compute(&series, VOL_CONE_HORIZONS),
    })
}

/// Daily (date, close) pairs shared by turn_of_month and vol_cone.
async fn daily_closes(
    pool: &PgPool,
    symbol: &str,
    years: u32,
) -> Result<Vec<(chrono::NaiveDate, f64)>, TomError> {
    let years = years.clamp(1, 20);
    let to = Utc::now();
    let from = to - Duration::days(366 * years as i64);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .map_err(TomError::PriceFetch)?;
    let closes: Vec<(chrono::NaiveDate, f64)> = bars
        .iter()
        .filter_map(|b| {
            let close: f64 = b.close.to_string().parse().unwrap_or(0.0);
            (close > 0.0).then(|| (b.bar_time.date_naive(), close))
        })
        .collect();
    if closes.len() < MIN_DAYS {
        return Err(TomError::Insufficient {
            symbol: symbol.to_string(),
            got: closes.len(),
            need: MIN_DAYS,
        });
    }
    Ok(closes)
}

pub async fn turn_of_month(
    pool: &PgPool,
    symbol: &str,
    years: u32,
) -> Result<TomReport, TomError> {
    let closes = daily_closes(pool, symbol, years).await?;
    Ok(tom_stats(symbol, &closes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    // ── grid trading ──────────────────────────────────────────────────────

    #[test]
    fn grid_arithmetic_levels_are_evenly_spaced() {
        let r = grid_trading(&GridInput {
            lower_price: 100.0,
            upper_price: 200.0,
            grid_count: 10,
            total_capital: 10_000.0,
            fee_pct: 0.0,
            geometric: false,
        })
        .unwrap();
        assert_eq!(r.levels.len(), 11);
        assert!((r.levels[0] - 100.0).abs() < 1e-9);
        assert!((r.levels[10] - 200.0).abs() < 1e-9);
        for w in r.levels.windows(2) {
            assert!((w[1] - w[0] - 10.0).abs() < 1e-9);
        }
        // Lowest grid has the largest % step (10/100), highest the
        // smallest (10/190).
        assert!((r.profit_per_grid_max_pct - 10.0).abs() < 1e-9);
        assert!((r.profit_per_grid_min_pct - 10.0 / 190.0 * 100.0).abs() < 1e-9);
        assert!((r.capital_per_grid - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn grid_geometric_steps_have_constant_percentage() {
        let r = grid_trading(&GridInput {
            lower_price: 100.0,
            upper_price: 400.0,
            grid_count: 4,
            total_capital: 8_000.0,
            fee_pct: 0.0,
            geometric: true,
        })
        .unwrap();
        // (400/100)^(1/4) = √2 per step ⇒ every grid yields the same %.
        assert!((r.profit_per_grid_min_pct - r.profit_per_grid_max_pct).abs() < 1e-9);
        let step = 2f64.sqrt();
        assert!((r.profit_per_grid_min_pct - (step - 1.0) * 100.0).abs() < 1e-9);
        assert!((r.levels[2] - 200.0).abs() < 1e-9); // 100·(√2)² = 200
    }

    #[test]
    fn grid_fees_subtract_both_legs_and_flag_unprofitable() {
        // 1% gross step, 0.6%/side fee ⇒ net = 1 − 1.2 = −0.2% on the
        // tightest grid → flagged.
        let r = grid_trading(&GridInput {
            lower_price: 100.0,
            upper_price: 110.0,
            grid_count: 10,
            total_capital: 1_000.0,
            fee_pct: 0.6,
            geometric: false,
        })
        .unwrap();
        assert!(r.any_grid_unprofitable);
        assert!(r.profit_per_grid_min_pct < 0.0);
        // The widest (lowest) grid: 1/100 = 1% gross − 1.2% = −0.2%.
        assert!((r.profit_per_grid_max_pct - (-0.2)).abs() < 1e-9);
    }

    #[test]
    fn grid_rejects_bad_inputs() {
        let base = GridInput {
            lower_price: 100.0,
            upper_price: 200.0,
            grid_count: 10,
            total_capital: 10_000.0,
            fee_pct: 0.1,
            geometric: false,
        };
        assert!(grid_trading(&GridInput { lower_price: 0.0, ..base.clone() }).is_err());
        assert!(grid_trading(&GridInput { upper_price: 99.0, ..base.clone() }).is_err());
        assert!(grid_trading(&GridInput { grid_count: 1, ..base.clone() }).is_err());
        assert!(grid_trading(&GridInput { grid_count: 201, ..base.clone() }).is_err());
        assert!(grid_trading(&GridInput { total_capital: 0.0, ..base.clone() }).is_err());
        assert!(grid_trading(&GridInput { fee_pct: -0.1, ..base }).is_err());
    }

    // ── fixed ratio ───────────────────────────────────────────────────────

    #[test]
    fn fixed_ratio_thresholds_follow_triangular_formula() {
        let r = fixed_ratio(&FixedRatioInput {
            starting_capital: 10_000.0,
            delta: 5_000.0,
            max_contracts: 5,
            profit_per_trade_per_contract: 0.0,
        })
        .unwrap();
        // E(n) = start + delta · (n−1)n/2 evaluated at n−1 steps:
        // 1→10k, 2→15k, 3→25k, 4→40k, 5→60k.
        let want = [10_000.0, 15_000.0, 25_000.0, 40_000.0, 60_000.0];
        assert_eq!(r.rows.len(), 5);
        for (row, w) in r.rows.iter().zip(want) {
            assert!((row.equity_required - w).abs() < 1e-9, "{row:?}");
        }
        // Gain from prev level n−1→n is delta × (n−1).
        assert!((r.rows[4].gain_from_prev - 20_000.0).abs() < 1e-9);
        assert!((r.total_gain_to_max - 50_000.0).abs() < 1e-9);
        assert!(r.rows[1].est_trades_from_prev.is_none());
    }

    #[test]
    fn fixed_ratio_estimates_trades_per_level() {
        let r = fixed_ratio(&FixedRatioInput {
            starting_capital: 10_000.0,
            delta: 5_000.0,
            max_contracts: 3,
            profit_per_trade_per_contract: 250.0,
        })
        .unwrap();
        // Level 2: gain 5k while trading 1 contract at $250 → 20 trades.
        assert!((r.rows[1].est_trades_from_prev.unwrap() - 20.0).abs() < 1e-9);
        // Level 3: gain 10k while trading 2 contracts at $500 → 20 trades.
        assert!((r.rows[2].est_trades_from_prev.unwrap() - 20.0).abs() < 1e-9);
    }

    #[test]
    fn fixed_ratio_rejects_bad_inputs() {
        assert!(fixed_ratio(&FixedRatioInput {
            starting_capital: 0.0,
            delta: 1.0,
            max_contracts: 5,
            profit_per_trade_per_contract: 0.0,
        })
        .is_err());
        assert!(fixed_ratio(&FixedRatioInput {
            starting_capital: 1.0,
            delta: 0.0,
            max_contracts: 5,
            profit_per_trade_per_contract: 0.0,
        })
        .is_err());
    }

    // ── anti-martingale ───────────────────────────────────────────────────

    #[test]
    fn anti_martingale_presses_wins_and_resets_on_loss() {
        // base 1%, ×2 on win, ×0.5 on loss (floored at base), 1R payoff.
        let r = anti_martingale(&AntiMartingaleInput {
            starting_capital: 10_000.0,
            base_risk_pct: 1.0,
            win_factor: 2.0,
            loss_factor: 0.5,
            max_risk_pct: 4.0,
            win_payoff_r: 1.0,
            sequence: "WWWL".into(),
        })
        .unwrap();
        // Risk schedule: 1%, 2%, 4% (capped), 4%.
        let risks: Vec<f64> = r.rows.iter().map(|row| row.risk_pct).collect();
        assert_eq!(risks, vec![1.0, 2.0, 4.0, 4.0]);
        // Hand-walked equity: 10000 →+100→ 10100 →+202→ 10302 →+412.08→
        // 10714.08 →−428.5632→ 10285.5168.
        assert!((r.final_equity - 10_285.5168).abs() < 1e-6, "{}", r.final_equity);
        // Fixed 1% control: ×1.01³ ×0.99.
        let fixed_want = 10_000.0 * 1.01_f64.powi(3) * 0.99;
        assert!((r.fixed_risk_final_equity - fixed_want).abs() < 1e-6);
        // The press grew faster on this win-streak sequence.
        assert!(r.final_equity > r.fixed_risk_final_equity);
    }

    #[test]
    fn anti_martingale_loss_floor_is_base_risk() {
        let r = anti_martingale(&AntiMartingaleInput {
            starting_capital: 10_000.0,
            base_risk_pct: 2.0,
            win_factor: 1.5,
            loss_factor: 0.25,
            max_risk_pct: 6.0,
            win_payoff_r: 1.0,
            sequence: "LLW".into(),
        })
        .unwrap();
        // Losses never push risk below base.
        let risks: Vec<f64> = r.rows.iter().map(|row| row.risk_pct).collect();
        assert_eq!(risks, vec![2.0, 2.0, 2.0]);
        assert!(r.max_drawdown_pct > 0.0);
    }

    #[test]
    fn anti_martingale_rejects_bad_inputs() {
        let base = AntiMartingaleInput {
            starting_capital: 10_000.0,
            base_risk_pct: 1.0,
            win_factor: 2.0,
            loss_factor: 0.5,
            max_risk_pct: 4.0,
            win_payoff_r: 1.5,
            sequence: "WL".into(),
        };
        assert!(anti_martingale(&AntiMartingaleInput { starting_capital: 0.0, ..base.clone() }).is_err());
        assert!(anti_martingale(&AntiMartingaleInput { base_risk_pct: 0.0, ..base.clone() }).is_err());
        assert!(anti_martingale(&AntiMartingaleInput { max_risk_pct: 0.5, ..base.clone() }).is_err());
        assert!(anti_martingale(&AntiMartingaleInput { sequence: "WXL".into(), ..base.clone() }).is_err());
        assert!(anti_martingale(&AntiMartingaleInput { sequence: "".into(), ..base }).is_err());
    }

    // ── santa rally ───────────────────────────────────────────────────────

    /// Weekday-only closes across a date range, all at `flat` price.
    fn weekdays(
        from: NaiveDate,
        to: NaiveDate,
        flat: f64,
    ) -> Vec<(NaiveDate, f64)> {
        let mut out = Vec::new();
        let mut d = from;
        while d <= to {
            if d.weekday().num_days_from_monday() < 5 {
                out.push((d, flat));
            }
            d = d.succ_opt().expect("valid date");
        }
        out
    }

    /// Two full Dec/Jan turns; +1% per session inside each 7-session
    /// Santa window, flat elsewhere.
    fn santa_closes() -> Vec<(NaiveDate, f64)> {
        let mut days = weekdays(
            NaiveDate::from_ymd_opt(2022, 11, 1).expect("valid"),
            NaiveDate::from_ymd_opt(2024, 1, 31).expect("valid"),
            0.0,
        );
        // Anchor per year = last December session index.
        let mut anchors = Vec::new();
        for (i, (d, _)) in days.iter().enumerate() {
            if d.month() == 12 && days.get(i + 1).map(|(n, _)| n.month() != 12).unwrap_or(false) {
                anchors.push(i);
            }
        }
        assert_eq!(anchors.len(), 2, "fixture needs two Decembers");
        let mut price = 100.0;
        for i in 0..days.len() {
            let in_window = anchors
                .iter()
                .any(|&a| i + 4 >= a && i <= a + 2 && a >= 4);
            if in_window {
                price *= 1.01;
            }
            days[i].1 = price;
        }
        days
    }

    #[test]
    fn santa_rally_measures_the_seven_session_window() {
        let closes = santa_closes();
        let r = santa_stats("TEST", &closes).expect("report");
        assert_eq!(r.years_analyzed, 2);
        // Each year compounds 1.01^7 − 1 ≈ 7.214% over the window.
        let want = (1.01_f64.powi(7) - 1.0) * 100.0;
        for y in &r.yearly {
            assert!((y.window_return_pct - want).abs() < 1e-9, "{y:?}");
        }
        assert!((r.rally_avg_return_pct - want).abs() < 1e-9);
        assert!((r.rally_hit_rate_pct - 100.0).abs() < 1e-12);
        // Per-offset stats (from holiday_seasonality) see ln(1.01) at
        // every offset in the window.
        for o in &r.offsets.by_offset {
            assert!((o.mean_return - 1.01_f64.ln()).abs() < 1e-9, "{o:?}");
            assert!((o.hit_rate - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn santa_rally_skips_partial_years() {
        // Truncate before the +2 January sessions of the second year:
        // only the first year has a complete window.
        let closes = santa_closes();
        let cutoff = NaiveDate::from_ymd_opt(2023, 12, 29).expect("valid");
        let truncated: Vec<_> = closes.into_iter().filter(|(d, _)| *d <= cutoff).collect();
        let r = santa_stats("TEST", &truncated).expect("report");
        assert_eq!(r.years_analyzed, 1);
        assert_eq!(r.yearly[0].year, 2022);
    }

    #[test]
    fn santa_rally_requires_at_least_one_complete_window() {
        let flat = weekdays(
            NaiveDate::from_ymd_opt(2024, 3, 1).expect("valid"),
            NaiveDate::from_ymd_opt(2024, 6, 1).expect("valid"),
            100.0,
        );
        assert!(santa_stats("TEST", &flat).is_none());
    }

    // ── event studies ─────────────────────────────────────────────────────

    #[test]
    fn third_friday_hand_checked_dates() {
        // June 2026 starts Monday ⇒ Fridays 5/12/19 ⇒ 19th.
        assert_eq!(
            third_friday(2026, 6),
            NaiveDate::from_ymd_opt(2026, 6, 19)
        );
        // January 2026 starts Thursday ⇒ Fridays 2/9/16 ⇒ 16th.
        assert_eq!(
            third_friday(2026, 1),
            NaiveDate::from_ymd_opt(2026, 1, 16)
        );
        // A month starting ON Friday: August 2025 ⇒ 1/8/15 ⇒ 15th.
        assert_eq!(
            third_friday(2025, 8),
            NaiveDate::from_ymd_opt(2025, 8, 15)
        );
        // Every result is a Friday.
        for m in 1..=12 {
            assert_eq!(
                third_friday(2026, m).expect("valid").weekday(),
                chrono::Weekday::Fri
            );
        }
    }

    #[test]
    fn index_lookup_snaps_to_prior_trading_day() {
        let closes: Vec<(NaiveDate, f64)> = [3, 4, 5, 10, 11]
            .iter()
            .map(|d| (NaiveDate::from_ymd_opt(2026, 6, *d).expect("valid"), 100.0))
            .collect();
        // Exact hit.
        let hit = NaiveDate::from_ymd_opt(2026, 6, 5).expect("valid");
        assert_eq!(index_at_or_before(&closes, hit), Some(2));
        // Weekend date (June 7, Sunday) snaps back to June 5.
        let weekend = NaiveDate::from_ymd_opt(2026, 6, 7).expect("valid");
        assert_eq!(index_at_or_before(&closes, weekend), Some(2));
        // Before the sample → None.
        let early = NaiveDate::from_ymd_opt(2026, 6, 1).expect("valid");
        assert_eq!(index_at_or_before(&closes, early), None);
    }

    // ── turn of month ─────────────────────────────────────────────────────

    /// Two synthetic months of 10 trading days each. Price jumps +1%
    /// only on the LAST trading day of each month, flat otherwise.
    fn synthetic_closes() -> Vec<(NaiveDate, f64)> {
        let mut out = Vec::new();
        let mut price = 100.0;
        for (year, month, days) in [(2024, 1, 10), (2024, 2, 10)] {
            for d in 1..=days {
                if d == days {
                    price *= 1.01;
                }
                out.push((
                    NaiveDate::from_ymd_opt(year, month, d).expect("valid date"),
                    price,
                ));
            }
        }
        out
    }

    #[test]
    fn tom_assigns_last_day_gain_to_offset_minus_one() {
        let closes = synthetic_closes();
        let r = tom_stats("TEST", &closes);
        let minus_one = r
            .rows
            .iter()
            .find(|row| row.offset == -1)
            .expect("offset -1 present");
        assert!((minus_one.avg_return_pct - 1.0).abs() < 1e-6, "{minus_one:?}");
        assert!((minus_one.hit_rate_pct - 100.0).abs() < 1e-9);
        // Every other offset bucket is flat.
        for row in &r.rows {
            if row.offset != -1 {
                assert!(row.avg_return_pct.abs() < 1e-9, "{row:?}");
            }
        }
        assert!(r.edge_pct > 0.0);
        assert!(r.rest_avg_return_pct.abs() < 1e-9);
    }

    #[test]
    fn tom_offsets_cover_expected_window() {
        let r = tom_stats("TEST", &synthetic_closes());
        let offsets: Vec<i32> = r.rows.iter().map(|row| row.offset).collect();
        assert_eq!(offsets, vec![-4, -3, -2, -1, 1, 2, 3]);
    }
}
