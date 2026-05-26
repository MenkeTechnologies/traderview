//! Aggregate statistics over closed trades.
//!
//! All money values are Decimal. Pure functions — no I/O. The report-routes
//! layer in `traderview-web` adapts these into HTTP responses.

use crate::models::{AssetClass, Trade, TradeSide, TradeStatus};
use chrono::{Datelike, NaiveDate, Timelike};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ===========================================================================
// Overview summary
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Summary {
    pub trade_count: usize,
    pub win_count: usize,
    pub loss_count: usize,
    pub scratch_count: usize,
    pub open_count: usize,
    pub gross_pnl: Decimal,
    pub net_pnl: Decimal,
    pub fees: Decimal,
    pub avg_win: Decimal,
    pub avg_loss: Decimal,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub expectancy: Decimal,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub max_consec_wins: u32,
    pub max_consec_losses: u32,
    pub avg_hold_seconds: i64,
    pub avg_win_hold_seconds: i64,
    pub avg_loss_hold_seconds: i64,
    pub avg_r: f64,
    pub total_volume: Decimal,
}

pub fn summary(trades: &[Trade]) -> Summary {
    let mut s = Summary::default();
    let mut win_sum = Decimal::ZERO;
    let mut loss_sum = Decimal::ZERO;
    let mut win_hold_sum: i64 = 0;
    let mut loss_hold_sum: i64 = 0;
    let mut hold_sum: i64 = 0;
    let mut r_sum: f64 = 0.0;
    let mut r_count: usize = 0;

    // For streak computation, walk closed trades in chronological order.
    let mut closed_sorted: Vec<&Trade> = trades
        .iter()
        .filter(|t| t.status == TradeStatus::Closed && t.net_pnl.is_some())
        .collect();
    closed_sorted.sort_by_key(|t| t.closed_at.unwrap_or(t.opened_at));

    let mut cur_win_streak = 0u32;
    let mut cur_loss_streak = 0u32;

    for t in &closed_sorted {
        let net = t.net_pnl.unwrap_or(Decimal::ZERO);
        let gross = t.gross_pnl.unwrap_or(Decimal::ZERO);
        s.trade_count += 1;
        s.gross_pnl += gross;
        s.net_pnl += net;
        s.fees += t.fees;
        s.total_volume += t.qty * t.entry_avg * t.multiplier;

        if let Some(h) = t.hold_seconds() {
            hold_sum += h;
        }
        if let Some(r) = t.r_multiple() {
            r_sum += decimal_to_f64(r);
            r_count += 1;
        }

        if net > Decimal::ZERO {
            s.win_count += 1;
            win_sum += net;
            if let Some(h) = t.hold_seconds() {
                win_hold_sum += h;
            }
            if net > s.largest_win {
                s.largest_win = net;
            }
            cur_win_streak += 1;
            cur_loss_streak = 0;
            if cur_win_streak > s.max_consec_wins {
                s.max_consec_wins = cur_win_streak;
            }
        } else if net < Decimal::ZERO {
            s.loss_count += 1;
            loss_sum += net;
            if let Some(h) = t.hold_seconds() {
                loss_hold_sum += h;
            }
            if net < s.largest_loss {
                s.largest_loss = net;
            }
            cur_loss_streak += 1;
            cur_win_streak = 0;
            if cur_loss_streak > s.max_consec_losses {
                s.max_consec_losses = cur_loss_streak;
            }
        } else {
            s.scratch_count += 1;
            cur_win_streak = 0;
            cur_loss_streak = 0;
        }
    }

    s.open_count = trades
        .iter()
        .filter(|t| t.status == TradeStatus::Open)
        .count();

    if s.win_count > 0 {
        s.avg_win = win_sum / Decimal::from(s.win_count as u64);
        s.avg_win_hold_seconds = win_hold_sum / s.win_count as i64;
    }
    if s.loss_count > 0 {
        s.avg_loss = loss_sum / Decimal::from(s.loss_count as u64);
        s.avg_loss_hold_seconds = loss_hold_sum / s.loss_count as i64;
    }
    if s.trade_count > 0 {
        s.win_rate = s.win_count as f64 / s.trade_count as f64;
        s.expectancy = s.net_pnl / Decimal::from(s.trade_count as u64);
        s.avg_hold_seconds = hold_sum / s.trade_count as i64;
    }
    if r_count > 0 {
        s.avg_r = r_sum / r_count as f64;
    }

    let loss_abs = loss_sum.abs();
    s.profit_factor = if loss_abs.is_zero() {
        if win_sum.is_zero() { 0.0 } else { f64::INFINITY }
    } else {
        decimal_to_f64(win_sum) / decimal_to_f64(loss_abs)
    };

    s
}

// ===========================================================================
// Equity curve + drawdown
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub day: NaiveDate,
    pub cum_net_pnl: Decimal,
    pub day_net_pnl: Decimal,
    pub trades: usize,
    pub drawdown: Decimal,
    pub drawdown_pct: f64,
}

pub fn equity_curve(trades: &[Trade], starting_cash: Decimal) -> Vec<EquityPoint> {
    let mut by_day: BTreeMap<NaiveDate, (Decimal, usize)> = BTreeMap::new();
    for t in trades {
        let Some(net) = t.net_pnl else { continue };
        let Some(closed) = t.closed_at else { continue };
        let day = closed.date_naive();
        let entry = by_day.entry(day).or_insert((Decimal::ZERO, 0));
        entry.0 += net;
        entry.1 += 1;
    }
    let mut cum = Decimal::ZERO;
    let mut peak = Decimal::ZERO;
    by_day
        .into_iter()
        .map(|(day, (day_net, n))| {
            cum += day_net;
            if cum > peak {
                peak = cum;
            }
            let drawdown = cum - peak; // ≤ 0
            let equity_at_peak = starting_cash + peak;
            let dd_pct = if equity_at_peak.is_zero() {
                0.0
            } else {
                decimal_to_f64(drawdown) / decimal_to_f64(equity_at_peak)
            };
            EquityPoint {
                day,
                cum_net_pnl: cum,
                day_net_pnl: day_net,
                trades: n,
                drawdown,
                drawdown_pct: dd_pct,
            }
        })
        .collect()
}

/// Maximum peak-to-trough drawdown in dollars and percent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaxDrawdown {
    pub max_dd: Decimal,
    pub max_dd_pct: f64,
    pub peak_day: Option<NaiveDate>,
    pub trough_day: Option<NaiveDate>,
}

pub fn max_drawdown(eq: &[EquityPoint]) -> MaxDrawdown {
    let mut peak_day = None;
    let mut trough_day = None;
    let mut max_dd = Decimal::ZERO;
    let mut max_dd_pct = 0.0_f64;
    let mut running_peak_day = None;
    let mut running_peak = Decimal::MIN;
    for p in eq {
        if p.cum_net_pnl > running_peak {
            running_peak = p.cum_net_pnl;
            running_peak_day = Some(p.day);
        }
        if p.drawdown < max_dd {
            max_dd = p.drawdown;
            max_dd_pct = p.drawdown_pct;
            peak_day = running_peak_day;
            trough_day = Some(p.day);
        }
    }
    MaxDrawdown {
        max_dd,
        max_dd_pct,
        peak_day,
        trough_day,
    }
}

// ===========================================================================
// Sharpe / Sortino (daily returns)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskAdjusted {
    pub sharpe: f64,
    pub sortino: f64,
    pub stdev_daily: f64,
    pub downside_stdev_daily: f64,
    pub mean_daily: f64,
}

/// Daily Sharpe/Sortino using daily net-P&L series. Risk-free rate is 0.
/// Annualization is left to the caller (multiply Sharpe by sqrt(252)).
pub fn risk_adjusted(eq: &[EquityPoint]) -> RiskAdjusted {
    if eq.is_empty() {
        return RiskAdjusted::default();
    }
    let n = eq.len() as f64;
    let series: Vec<f64> = eq.iter().map(|p| decimal_to_f64(p.day_net_pnl)).collect();
    let mean = series.iter().sum::<f64>() / n;
    let var = series.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let stdev = var.sqrt();
    let downside_var = series
        .iter()
        .filter(|x| **x < 0.0)
        .map(|x| x.powi(2))
        .sum::<f64>()
        / n;
    let downside_stdev = downside_var.sqrt();
    RiskAdjusted {
        sharpe: if stdev == 0.0 { 0.0 } else { mean / stdev },
        sortino: if downside_stdev == 0.0 {
            0.0
        } else {
            mean / downside_stdev
        },
        stdev_daily: stdev,
        downside_stdev_daily: downside_stdev,
        mean_daily: mean,
    }
}

// ===========================================================================
// Bucketed reports — by symbol / day-of-week / time-of-day / hold-time / asset / tag / side
// ===========================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Bucket {
    pub key: String,
    pub trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub gross_pnl: Decimal,
    pub net_pnl: Decimal,
    pub win_rate: f64,
    pub avg_pnl: Decimal,
    pub expectancy: Decimal,
}

fn bucket_from<'a, F>(trades: &'a [Trade], keyf: F) -> Vec<Bucket>
where
    F: Fn(&'a Trade) -> Option<String>,
{
    let mut map: BTreeMap<String, Bucket> = BTreeMap::new();
    for t in trades {
        if t.status != TradeStatus::Closed {
            continue;
        }
        let Some(key) = keyf(t) else { continue };
        let net = t.net_pnl.unwrap_or(Decimal::ZERO);
        let gross = t.gross_pnl.unwrap_or(Decimal::ZERO);
        let b = map.entry(key.clone()).or_insert(Bucket {
            key,
            trades: 0,
            wins: 0,
            losses: 0,
            gross_pnl: Decimal::ZERO,
            net_pnl: Decimal::ZERO,
            win_rate: 0.0,
            avg_pnl: Decimal::ZERO,
            expectancy: Decimal::ZERO,
        });
        b.trades += 1;
        if net > Decimal::ZERO {
            b.wins += 1;
        } else if net < Decimal::ZERO {
            b.losses += 1;
        }
        b.gross_pnl += gross;
        b.net_pnl += net;
    }
    for b in map.values_mut() {
        b.win_rate = b.wins as f64 / b.trades.max(1) as f64;
        b.avg_pnl = b.net_pnl / Decimal::from(b.trades.max(1) as u64);
        b.expectancy = b.avg_pnl;
    }
    map.into_values().collect()
}

pub fn by_symbol(trades: &[Trade]) -> Vec<Bucket> {
    bucket_from(trades, |t| Some(t.symbol.clone()))
}

pub fn by_side(trades: &[Trade]) -> Vec<Bucket> {
    bucket_from(trades, |t| {
        Some(
            match t.side {
                TradeSide::Long => "long",
                TradeSide::Short => "short",
            }
            .into(),
        )
    })
}

pub fn by_asset_class(trades: &[Trade]) -> Vec<Bucket> {
    bucket_from(trades, |t| {
        Some(
            match t.asset_class {
                AssetClass::Stock => "stock",
                AssetClass::Option => "option",
                AssetClass::Future => "future",
                AssetClass::Forex => "forex",
            }
            .into(),
        )
    })
}

pub fn by_day_of_week(trades: &[Trade]) -> Vec<Bucket> {
    bucket_from(trades, |t| {
        let day = t.closed_at?.date_naive();
        Some(match day.weekday() {
            chrono::Weekday::Mon => "1_mon",
            chrono::Weekday::Tue => "2_tue",
            chrono::Weekday::Wed => "3_wed",
            chrono::Weekday::Thu => "4_thu",
            chrono::Weekday::Fri => "5_fri",
            chrono::Weekday::Sat => "6_sat",
            chrono::Weekday::Sun => "7_sun",
        }
        .into())
    })
}

pub fn by_hour_of_day(trades: &[Trade]) -> Vec<Bucket> {
    bucket_from(trades, |t| {
        let h = t.opened_at.hour();
        Some(format!("{:02}", h))
    })
}

pub fn by_hold_bucket(trades: &[Trade]) -> Vec<Bucket> {
    bucket_from(trades, |t| {
        let secs = t.hold_seconds()?;
        Some(match secs {
            x if x < 60 => "00_seconds".into(),
            x if x < 300 => "01_under_5m".into(),
            x if x < 900 => "02_5_to_15m".into(),
            x if x < 3_600 => "03_15m_to_1h".into(),
            x if x < 4 * 3_600 => "04_1h_to_4h".into(),
            x if x < 24 * 3_600 => "05_4h_to_1d".into(),
            x if x < 7 * 86_400 => "06_1d_to_1w".into(),
            x if x < 30 * 86_400 => "07_1w_to_1mo".into(),
            _ => "08_over_1mo".into(),
        })
    })
}

pub fn by_month(trades: &[Trade]) -> Vec<Bucket> {
    bucket_from(trades, |t| {
        let day = t.closed_at?.date_naive();
        Some(format!("{:04}-{:02}", day.year(), day.month()))
    })
}

// ===========================================================================
// R-multiple distribution
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RMultipleDistribution {
    pub bins: Vec<RBin>,
    pub avg_r: f64,
    pub median_r: f64,
    pub trades_with_r: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RBin {
    pub label: String,
    pub low: f64,
    pub high: f64,
    pub count: usize,
}

pub fn r_distribution(trades: &[Trade]) -> RMultipleDistribution {
    let mut rs: Vec<f64> = trades
        .iter()
        .filter_map(|t| t.r_multiple().map(decimal_to_f64))
        .collect();
    if rs.is_empty() {
        return RMultipleDistribution::default();
    }
    rs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let avg = rs.iter().sum::<f64>() / rs.len() as f64;
    let median = if rs.len() % 2 == 1 {
        rs[rs.len() / 2]
    } else {
        (rs[rs.len() / 2 - 1] + rs[rs.len() / 2]) / 2.0
    };

    // Fixed bins: ≤-3, -3..-2, -2..-1, -1..0, 0..1, 1..2, 2..3, ≥3
    let edges = [-3.0_f64, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0];
    let mut bins: Vec<RBin> = Vec::new();
    bins.push(RBin {
        label: "≤-3R".into(),
        low: f64::NEG_INFINITY,
        high: -3.0,
        count: 0,
    });
    for w in edges.windows(2) {
        bins.push(RBin {
            label: format!("{:+.0}R..{:+.0}R", w[0], w[1]),
            low: w[0],
            high: w[1],
            count: 0,
        });
    }
    bins.push(RBin {
        label: "≥+3R".into(),
        low: 3.0,
        high: f64::INFINITY,
        count: 0,
    });

    for r in &rs {
        for b in bins.iter_mut() {
            if *r >= b.low && *r < b.high {
                b.count += 1;
                break;
            }
        }
    }
    RMultipleDistribution {
        bins,
        avg_r: avg,
        median_r: median,
        trades_with_r: rs.len(),
    }
}

// ===========================================================================
// Calendar (daily P&L heatmap)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarCell {
    pub day: NaiveDate,
    pub net_pnl: Decimal,
    pub trades: usize,
    pub wins: usize,
    pub losses: usize,
}

pub fn calendar(trades: &[Trade]) -> Vec<CalendarCell> {
    let mut map: BTreeMap<NaiveDate, CalendarCell> = BTreeMap::new();
    for t in trades {
        let Some(closed) = t.closed_at else { continue };
        let net = t.net_pnl.unwrap_or(Decimal::ZERO);
        let day = closed.date_naive();
        let cell = map.entry(day).or_insert(CalendarCell {
            day,
            net_pnl: Decimal::ZERO,
            trades: 0,
            wins: 0,
            losses: 0,
        });
        cell.net_pnl += net;
        cell.trades += 1;
        if net > Decimal::ZERO {
            cell.wins += 1;
        } else if net < Decimal::ZERO {
            cell.losses += 1;
        }
    }
    map.into_values().collect()
}

// ===========================================================================
// Streaks
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Streak {
    pub kind: String, // "win" | "loss"
    pub length: u32,
    pub net_pnl: Decimal,
    pub start: NaiveDate,
    pub end: NaiveDate,
}

pub fn streaks(trades: &[Trade]) -> Vec<Streak> {
    let mut closed: Vec<&Trade> = trades
        .iter()
        .filter(|t| t.status == TradeStatus::Closed && t.net_pnl.is_some())
        .collect();
    closed.sort_by_key(|t| t.closed_at.unwrap_or(t.opened_at));

    let mut out = Vec::new();
    let mut cur: Option<(String, u32, Decimal, NaiveDate, NaiveDate)> = None;
    for t in closed {
        let net = t.net_pnl.unwrap_or(Decimal::ZERO);
        let day = t.closed_at.unwrap().date_naive();
        let kind = if net > Decimal::ZERO {
            "win"
        } else if net < Decimal::ZERO {
            "loss"
        } else {
            continue;
        };
        if let Some((k, n, p, start, _end)) = cur.take() {
            if k == kind {
                cur = Some((k, n + 1, p + net, start, day));
            } else {
                out.push(Streak {
                    kind: k,
                    length: n,
                    net_pnl: p,
                    start,
                    end: _end,
                });
                cur = Some((kind.into(), 1, net, day, day));
            }
        } else {
            cur = Some((kind.into(), 1, net, day, day));
        }
    }
    if let Some((k, n, p, start, end)) = cur {
        out.push(Streak {
            kind: k,
            length: n,
            net_pnl: p,
            start,
            end,
        });
    }
    out
}

// ===========================================================================
// Comparison: wins vs losses, long vs short
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Comparison {
    pub long: Bucket,
    pub short: Bucket,
    pub wins: ComparisonSide,
    pub losses: ComparisonSide,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComparisonSide {
    pub count: usize,
    pub avg_pnl: Decimal,
    pub avg_hold_seconds: i64,
    pub avg_qty: Decimal,
}

pub fn comparison(trades: &[Trade]) -> Comparison {
    let mut cmp = Comparison::default();

    let mut long_b = Bucket {
        key: "long".into(),
        trades: 0,
        wins: 0,
        losses: 0,
        gross_pnl: Decimal::ZERO,
        net_pnl: Decimal::ZERO,
        win_rate: 0.0,
        avg_pnl: Decimal::ZERO,
        expectancy: Decimal::ZERO,
    };
    let mut short_b = long_b.clone();
    short_b.key = "short".into();

    let mut win_n = 0usize;
    let mut win_pnl = Decimal::ZERO;
    let mut win_hold: i64 = 0;
    let mut win_qty = Decimal::ZERO;
    let mut loss_n = 0usize;
    let mut loss_pnl = Decimal::ZERO;
    let mut loss_hold: i64 = 0;
    let mut loss_qty = Decimal::ZERO;

    for t in trades {
        if t.status != TradeStatus::Closed {
            continue;
        }
        let net = t.net_pnl.unwrap_or(Decimal::ZERO);
        let gross = t.gross_pnl.unwrap_or(Decimal::ZERO);
        let h = t.hold_seconds().unwrap_or(0);
        let b = if t.side == TradeSide::Long {
            &mut long_b
        } else {
            &mut short_b
        };
        b.trades += 1;
        if net > Decimal::ZERO {
            b.wins += 1;
            win_n += 1;
            win_pnl += net;
            win_hold += h;
            win_qty += t.qty;
        } else if net < Decimal::ZERO {
            b.losses += 1;
            loss_n += 1;
            loss_pnl += net;
            loss_hold += h;
            loss_qty += t.qty;
        }
        b.gross_pnl += gross;
        b.net_pnl += net;
    }
    for b in [&mut long_b, &mut short_b] {
        b.win_rate = b.wins as f64 / b.trades.max(1) as f64;
        b.avg_pnl = b.net_pnl / Decimal::from(b.trades.max(1) as u64);
        b.expectancy = b.avg_pnl;
    }
    cmp.long = long_b;
    cmp.short = short_b;
    cmp.wins = ComparisonSide {
        count: win_n,
        avg_pnl: if win_n > 0 {
            win_pnl / Decimal::from(win_n as u64)
        } else {
            Decimal::ZERO
        },
        avg_hold_seconds: if win_n > 0 { win_hold / win_n as i64 } else { 0 },
        avg_qty: if win_n > 0 {
            win_qty / Decimal::from(win_n as u64)
        } else {
            Decimal::ZERO
        },
    };
    cmp.losses = ComparisonSide {
        count: loss_n,
        avg_pnl: if loss_n > 0 {
            loss_pnl / Decimal::from(loss_n as u64)
        } else {
            Decimal::ZERO
        },
        avg_hold_seconds: if loss_n > 0 {
            loss_hold / loss_n as i64
        } else {
            0
        },
        avg_qty: if loss_n > 0 {
            loss_qty / Decimal::from(loss_n as u64)
        } else {
            Decimal::ZERO
        },
    };
    cmp
}

// ===========================================================================
// Exit efficiency (uses best_exit_pnl populated by excursion module)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExitEfficiency {
    pub avg_efficiency: f64,
    pub trades_with_data: usize,
    pub missed_pnl: Decimal,
    pub by_symbol: Vec<Bucket>,
}

pub fn exit_efficiency(trades: &[Trade]) -> ExitEfficiency {
    let mut sum_eff = 0.0_f64;
    let mut n = 0;
    let mut missed = Decimal::ZERO;
    for t in trades {
        let (Some(net), Some(best)) = (t.net_pnl, t.best_exit_pnl) else {
            continue;
        };
        if best.is_zero() {
            continue;
        }
        let eff = decimal_to_f64(net) / decimal_to_f64(best);
        sum_eff += eff;
        n += 1;
        missed += best - net;
    }
    ExitEfficiency {
        avg_efficiency: if n > 0 { sum_eff / n as f64 } else { 0.0 },
        trades_with_data: n,
        missed_pnl: missed,
        by_symbol: bucket_from(trades, |t| {
            if t.best_exit_pnl.is_some() && t.net_pnl.is_some() {
                Some(t.symbol.clone())
            } else {
                None
            }
        }),
    }
}

// ===========================================================================
// Commissions & fees report
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommissionReport {
    pub total_fees: Decimal,
    pub fees_pct_of_gross: f64,
    pub avg_fee_per_trade: Decimal,
    pub avg_fee_per_unit: Decimal,
    pub by_symbol: Vec<Bucket>,
}

pub fn commissions(trades: &[Trade]) -> CommissionReport {
    let mut r = CommissionReport::default();
    let mut total_qty = Decimal::ZERO;
    let mut gross = Decimal::ZERO;
    let mut n = 0usize;
    for t in trades {
        if t.status != TradeStatus::Closed {
            continue;
        }
        r.total_fees += t.fees;
        gross += t.gross_pnl.unwrap_or(Decimal::ZERO).abs();
        total_qty += t.qty;
        n += 1;
    }
    if n > 0 {
        r.avg_fee_per_trade = r.total_fees / Decimal::from(n as u64);
    }
    if !total_qty.is_zero() {
        r.avg_fee_per_unit = r.total_fees / total_qty;
    }
    if !gross.is_zero() {
        r.fees_pct_of_gross = decimal_to_f64(r.total_fees) / decimal_to_f64(gross);
    }
    r.by_symbol = bucket_from(trades, |t| Some(t.symbol.clone()));
    r
}

// ===========================================================================
// Helpers
// ===========================================================================

fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}
