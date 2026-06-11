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

pub async fn turn_of_month(
    pool: &PgPool,
    symbol: &str,
    years: u32,
) -> Result<TomReport, TomError> {
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
