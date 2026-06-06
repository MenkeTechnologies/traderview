//! Aggregate statistics over closed trades.
//!
//! All money values are Decimal. Pure functions — no I/O. The report-routes
//! layer in `traderview-web` adapts these into HTTP responses.

use crate::models::{AssetClass, Trade, TradeSide, TradeStatus};
use chrono::{Datelike, NaiveDate, Timelike};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

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
    pub commissions: Decimal,
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
    pub avg_scratch_hold_seconds: i64,
    pub avg_r: f64,
    pub total_volume: Decimal,
    pub total_shares: Decimal,
    pub trading_days: u32,
    pub avg_daily_pnl: Decimal,
    pub avg_daily_volume: Decimal,
    pub avg_per_share_pnl: Decimal,
    pub net_pnl_stddev: f64,
    pub avg_mae: Decimal,
    pub avg_mfe: Decimal,
    pub sqn: Option<f64>,
    pub k_ratio: Option<f64>,
    pub kelly_pct: Option<f64>,
    pub random_chance_prob: Option<f64>,
}

pub fn summary(trades: &[Trade]) -> Summary {
    let mut s = Summary::default();
    let mut win_sum = Decimal::ZERO;
    let mut loss_sum = Decimal::ZERO;
    let mut win_hold_sum: i64 = 0;
    let mut loss_hold_sum: i64 = 0;
    let mut scratch_hold_sum: i64 = 0;
    let mut scratch_hold_count: i64 = 0;
    let mut hold_sum: i64 = 0;
    let mut r_sum: f64 = 0.0;
    let mut r_count: usize = 0;
    // Welford for trade-P&L stddev (over closed trades) and r-multiple stddev.
    let mut pnl_mean = 0.0_f64;
    let mut pnl_m2 = 0.0_f64;
    let mut r_mean = 0.0_f64;
    let mut r_m2 = 0.0_f64;
    let mut mae_sum = Decimal::ZERO;
    let mut mae_count: u64 = 0;
    let mut mfe_sum = Decimal::ZERO;
    let mut mfe_count: u64 = 0;
    let mut trading_days: HashSet<NaiveDate> = HashSet::new();
    // Per-trade running net-P&L for K-Ratio.
    let mut cum_pnl_series: Vec<f64> = Vec::new();
    let mut cum_running = 0.0_f64;

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
        s.commissions += t.commissions;
        s.total_volume += t.qty * t.entry_avg * t.multiplier;
        s.total_shares += t.qty;

        if let Some(closed) = t.closed_at {
            trading_days.insert(closed.date_naive());
        }
        if let Some(mae) = t.mae {
            mae_sum += mae;
            mae_count += 1;
        }
        if let Some(mfe) = t.mfe {
            mfe_sum += mfe;
            mfe_count += 1;
        }

        // Welford for net-P&L stddev across all closed trades.
        let net_f = decimal_to_f64(net);
        let n = s.trade_count as f64;
        let delta = net_f - pnl_mean;
        pnl_mean += delta / n;
        pnl_m2 += delta * (net_f - pnl_mean);

        cum_running += net_f;
        cum_pnl_series.push(cum_running);

        if let Some(h) = t.hold_seconds() {
            hold_sum += h;
        }
        if let Some(r) = t.r_multiple() {
            let rf = decimal_to_f64(r);
            r_sum += rf;
            r_count += 1;
            let rn = r_count as f64;
            let r_delta = rf - r_mean;
            r_mean += r_delta / rn;
            r_m2 += r_delta * (rf - r_mean);
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
            if let Some(h) = t.hold_seconds() {
                scratch_hold_sum += h;
                scratch_hold_count += 1;
            }
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
    if scratch_hold_count > 0 {
        s.avg_scratch_hold_seconds = scratch_hold_sum / scratch_hold_count;
    }
    if s.trade_count > 0 {
        s.win_rate = s.win_count as f64 / s.trade_count as f64;
        s.expectancy = s.net_pnl / Decimal::from(s.trade_count as u64);
        s.avg_hold_seconds = hold_sum / s.trade_count as i64;
    }
    if r_count > 0 {
        s.avg_r = r_sum / r_count as f64;
    }
    if mae_count > 0 {
        s.avg_mae = mae_sum / Decimal::from(mae_count);
    }
    if mfe_count > 0 {
        s.avg_mfe = mfe_sum / Decimal::from(mfe_count);
    }
    s.trading_days = trading_days.len() as u32;
    if s.trading_days > 0 {
        let td = Decimal::from(s.trading_days as u64);
        s.avg_daily_pnl = s.net_pnl / td;
        s.avg_daily_volume = s.total_volume / td;
    }
    if !s.total_shares.is_zero() {
        s.avg_per_share_pnl = s.net_pnl / s.total_shares;
    }
    if s.trade_count > 1 {
        // Welford produces population M2; sample variance divides by n-1.
        let var = pnl_m2 / (s.trade_count as f64 - 1.0);
        if var.is_finite() && var >= 0.0 {
            s.net_pnl_stddev = var.sqrt();
        }
    }

    // Van Tharp SQN = avg_r / r_stddev * sqrt(n), capped at sqrt(100).
    // Capped because SQN's discriminating power saturates past n=100.
    if r_count > 1 {
        let r_var = r_m2 / (r_count as f64 - 1.0);
        if r_var > 0.0 && r_var.is_finite() {
            let r_std = r_var.sqrt();
            let n_eff = (r_count as f64).min(100.0);
            let v = s.avg_r / r_std * n_eff.sqrt();
            if v.is_finite() {
                s.sqn = Some(v);
            }
        }
    }

    s.k_ratio = k_ratio(&cum_pnl_series);

    // Kelly: f* = W - (1-W) / (avg_win / |avg_loss|).
    // Undefined when avg_loss is zero (no losers).
    if s.loss_count > 0 && !s.avg_loss.is_zero() && s.trade_count > 0 {
        let avg_w = decimal_to_f64(s.avg_win);
        let avg_l_abs = decimal_to_f64(s.avg_loss).abs();
        if avg_l_abs > 0.0 {
            let payoff = avg_w / avg_l_abs;
            if payoff > 0.0 && payoff.is_finite() {
                let f = s.win_rate - (1.0 - s.win_rate) / payoff;
                if f.is_finite() {
                    s.kelly_pct = Some(f);
                }
            }
        }
    }

    // One-tailed binomial P(X >= win_count | n, p=0.5) via normal approx
    // with continuity correction. For n=1752 this is well-approximated.
    if s.trade_count > 0 {
        let n = s.trade_count as f64;
        let mu = n * 0.5;
        let sigma = (n * 0.25).sqrt();
        if sigma > 0.0 {
            // continuity correction: use k - 0.5 as the lower bound of [k,∞)
            let z = (s.win_count as f64 - 0.5 - mu) / sigma;
            let p = 1.0 - normal_cdf(z);
            if p.is_finite() {
                s.random_chance_prob = Some(p.clamp(0.0, 1.0));
            }
        }
    }

    let loss_abs = loss_sum.abs();
    s.profit_factor = if loss_abs.is_zero() {
        if win_sum.is_zero() {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        decimal_to_f64(win_sum) / decimal_to_f64(loss_abs)
    };

    s
}

/// Kestner K-Ratio: regress cumulative P&L on trade index, return slope/SE.
/// Higher = more consistent equity-curve growth. None when n<3 or zero variance.
fn k_ratio(cum_pnl: &[f64]) -> Option<f64> {
    let n = cum_pnl.len();
    if n < 3 {
        return None;
    }
    let nf = n as f64;
    let mean_x = (nf + 1.0) / 2.0;
    let mean_y = cum_pnl.iter().sum::<f64>() / nf;
    let mut num = 0.0_f64;
    let mut denom_x = 0.0_f64;
    for (i, &y) in cum_pnl.iter().enumerate() {
        let xi = (i as f64) + 1.0;
        let dx = xi - mean_x;
        num += dx * (y - mean_y);
        denom_x += dx * dx;
    }
    if denom_x == 0.0 {
        return None;
    }
    let slope = num / denom_x;
    let intercept = mean_y - slope * mean_x;
    let mut ss_res = 0.0_f64;
    for (i, &y) in cum_pnl.iter().enumerate() {
        let xi = (i as f64) + 1.0;
        let yhat = intercept + slope * xi;
        let r = y - yhat;
        ss_res += r * r;
    }
    let sigma2 = ss_res / ((n - 2) as f64);
    if sigma2 <= 0.0 || !sigma2.is_finite() {
        return None;
    }
    let se_slope = (sigma2 / denom_x).sqrt();
    if se_slope == 0.0 || !se_slope.is_finite() {
        return None;
    }
    let v = slope / se_slope;
    if v.is_finite() {
        Some(v)
    } else {
        None
    }
}

/// Standard normal CDF via Abramowitz & Stegun 7.1.26 (max error ~1.5e-7).
/// Φ(x) = 0.5 · (1 + erf(x / √2)).
fn normal_cdf(x: f64) -> f64 {
    let z = x / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.3275911 * z.abs());
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let y = 1.0 - ((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t * (-z * z).exp();
    let erf = if z < 0.0 { -y } else { y };
    0.5 * (1.0 + erf)
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
        Some(
            match day.weekday() {
                chrono::Weekday::Mon => "1_mon",
                chrono::Weekday::Tue => "2_tue",
                chrono::Weekday::Wed => "3_wed",
                chrono::Weekday::Thu => "4_thu",
                chrono::Weekday::Fri => "5_fri",
                chrono::Weekday::Sat => "6_sat",
                chrono::Weekday::Sun => "7_sun",
            }
            .into(),
        )
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

/// Tradervue parity: 2-bucket pivot — intraday vs multiday.
/// A trade is intraday when `opened_at.date == closed_at.date` (UTC).
pub fn by_duration_coarse(trades: &[Trade]) -> Vec<Bucket> {
    let buckets = bucket_from(trades, |t| {
        let close = t.closed_at?;
        let key = if close.date_naive() == t.opened_at.date_naive() {
            "Intraday"
        } else {
            "Multiday"
        };
        Some(key.to_string())
    });
    // Stable ordering.
    let order = ["Intraday", "Multiday"];
    let mut sorted: Vec<Bucket> = order
        .iter()
        .filter_map(|k| buckets.iter().find(|b| b.key == *k).cloned())
        .collect();
    for b in &buckets {
        if !order.contains(&b.key.as_str()) {
            sorted.push(b.clone());
        }
    }
    sorted
}

/// Tradervue parity: bucket trades by R-multiple range.
/// Buckets mirror the fixed edges used by `r_distribution` so the
/// dashboard's "Performance By R" can be plotted alongside the count
/// histogram from r-dist.
pub fn by_r_bucket(trades: &[Trade]) -> Vec<Bucket> {
    let buckets = bucket_from(trades, |t| {
        let r = t.r_multiple()?;
        let rf = decimal_to_f64(r);
        Some(r_bucket_label(rf))
    });
    let order = [
        "≤ -3R",
        "-3R to -2R",
        "-2R to -1R",
        "-1R to 0R",
        "0R to 1R",
        "1R to 2R",
        "2R to 3R",
        "≥ 3R",
    ];
    let mut sorted: Vec<Bucket> = order
        .iter()
        .filter_map(|k| buckets.iter().find(|b| b.key == *k).cloned())
        .collect();
    for b in &buckets {
        if !order.contains(&b.key.as_str()) {
            sorted.push(b.clone());
        }
    }
    sorted
}

fn r_bucket_label(r: f64) -> String {
    if r <= -3.0 {
        "≤ -3R"
    } else if r <= -2.0 {
        "-3R to -2R"
    } else if r <= -1.0 {
        "-2R to -1R"
    } else if r <= 0.0 {
        "-1R to 0R"
    } else if r <= 1.0 {
        "0R to 1R"
    } else if r <= 2.0 {
        "1R to 2R"
    } else if r <= 3.0 {
        "2R to 3R"
    } else {
        "≥ 3R"
    }
    .to_string()
}

/// Tradervue parity: bucket trades by their *opening gap* — the percentage
/// move between the prior-day close and the current-day open for each
/// trade's symbol. The caller looks up the prior close from `price_bars`
/// and passes it in as `prior_close_by_trade[trade_id]`. Trades whose
/// symbol has no prior bar are skipped.
pub fn by_opening_gap(
    trades: &[Trade],
    prior_close_by_trade: &std::collections::HashMap<uuid::Uuid, Decimal>,
) -> Vec<Bucket> {
    let buckets = bucket_from(trades, |t| {
        let prior = prior_close_by_trade.get(&t.id)?;
        if prior.is_zero() {
            return None;
        }
        let entry_f = decimal_to_f64(t.entry_avg);
        let prior_f = decimal_to_f64(*prior);
        if prior_f == 0.0 || !prior_f.is_finite() {
            return None;
        }
        let gap_pct = (entry_f - prior_f) / prior_f * 100.0;
        Some(opening_gap_label(gap_pct))
    });
    let order = [
        "< -7%",
        "-7% to -2%",
        "-2% to 0%",
        "0% to +2%",
        "+2% to +7%",
        "> +7%",
    ];
    let mut sorted: Vec<Bucket> = order
        .iter()
        .filter_map(|k| buckets.iter().find(|b| b.key == *k).cloned())
        .collect();
    for b in &buckets {
        if !order.contains(&b.key.as_str()) {
            sorted.push(b.clone());
        }
    }
    sorted
}

fn opening_gap_label(g: f64) -> String {
    if g < -7.0 {
        "< -7%"
    } else if g < -2.0 {
        "-7% to -2%"
    } else if g < 0.0 {
        "-2% to 0%"
    } else if g < 2.0 {
        "0% to +2%"
    } else if g < 7.0 {
        "+2% to +7%"
    } else {
        "> +7%"
    }
    .to_string()
}

/// Bucket trades by the *symbol's average daily volume* (ADV) range.
/// Caller passes a precomputed ADV per symbol (from `price_bars` daily
/// bars). Symbols missing from the map are excluded.
pub fn by_instrument_volume(
    trades: &[Trade],
    adv_by_symbol: &std::collections::HashMap<String, Decimal>,
) -> Vec<Bucket> {
    let buckets = bucket_from(trades, |t| {
        let adv = adv_by_symbol.get(&t.symbol)?;
        Some(instrument_volume_label(decimal_to_f64(*adv)))
    });
    let order = ["< 1M", "1M - 10M", "10M - 100M", "100M - 500M", "≥ 500M"];
    let mut sorted: Vec<Bucket> = order
        .iter()
        .filter_map(|k| buckets.iter().find(|b| b.key == *k).cloned())
        .collect();
    for b in &buckets {
        if !order.contains(&b.key.as_str()) {
            sorted.push(b.clone());
        }
    }
    sorted
}

fn instrument_volume_label(v: f64) -> String {
    if v < 1_000_000.0 {
        "< 1M"
    } else if v < 10_000_000.0 {
        "1M - 10M"
    } else if v < 100_000_000.0 {
        "10M - 100M"
    } else if v < 500_000_000.0 {
        "100M - 500M"
    } else {
        "≥ 500M"
    }
    .to_string()
}

/// Bucket trades by the *symbol's average daily range* (high - low) as a
/// percent of close. Caller passes precomputed average range pct per symbol.
pub fn by_movement(
    trades: &[Trade],
    range_pct_by_symbol: &std::collections::HashMap<String, f64>,
) -> Vec<Bucket> {
    let buckets = bucket_from(trades, |t| {
        let r = range_pct_by_symbol.get(&t.symbol)?;
        Some(movement_label(*r))
    });
    let order = ["< 1%", "1% - 3%", "3% - 5%", "5% - 10%", "≥ 10%"];
    let mut sorted: Vec<Bucket> = order
        .iter()
        .filter_map(|k| buckets.iter().find(|b| b.key == *k).cloned())
        .collect();
    for b in &buckets {
        if !order.contains(&b.key.as_str()) {
            sorted.push(b.clone());
        }
    }
    sorted
}

fn movement_label(r: f64) -> String {
    if r < 1.0 {
        "< 1%"
    } else if r < 3.0 {
        "1% - 3%"
    } else if r < 5.0 {
        "3% - 5%"
    } else if r < 10.0 {
        "5% - 10%"
    } else {
        "≥ 10%"
    }
    .to_string()
}

/// Tradervue parity: bucket trades by entry-price range.
pub fn by_price_bucket(trades: &[Trade]) -> Vec<Bucket> {
    let buckets = bucket_from(trades, |t| Some(price_range(t.entry_avg)));
    let order = [
        "< $2.00",
        "$2 - $4.99",
        "$5 - $9.99",
        "$10 - $19.99",
        "$20 - $49.99",
        "$50 - $99.99",
        "$100 - $199.99",
        "$200 - $499.99",
        "$500 - $999.99",
        ">= $1000",
    ];
    let mut sorted: Vec<Bucket> = order
        .iter()
        .filter_map(|k| buckets.iter().find(|b| b.key == *k).cloned())
        .collect();
    // Append any unexpected keys at end.
    for b in &buckets {
        if !order.contains(&b.key.as_str()) {
            sorted.push(b.clone());
        }
    }
    sorted
}

fn price_range(p: Decimal) -> String {
    let f = p.to_string().parse::<f64>().unwrap_or(0.0);
    if f < 2.0 {
        "< $2.00"
    } else if f < 5.0 {
        "$2 - $4.99"
    } else if f < 10.0 {
        "$5 - $9.99"
    } else if f < 20.0 {
        "$10 - $19.99"
    } else if f < 50.0 {
        "$20 - $49.99"
    } else if f < 100.0 {
        "$50 - $99.99"
    } else if f < 200.0 {
        "$100 - $199.99"
    } else if f < 500.0 {
        "$200 - $499.99"
    } else if f < 1000.0 {
        "$500 - $999.99"
    } else {
        ">= $1000"
    }
    .to_string()
}

// ===========================================================================
// Daily volume + per-day series (Tradervue dashboard parity)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailySeriesPoint {
    pub day: NaiveDate,
    pub volume: Decimal,
    pub trades: usize,
    pub winners: usize,
    pub net_pnl: Decimal,
    pub running_avg_pnl: Decimal,
    pub running_win_rate: f64,
}

/// Per-day rollups: volume, win rate, running average trade P&L.
/// Open trades (no net_pnl) contribute volume but not P&L.
pub fn daily_series(trades: &[Trade]) -> Vec<DailySeriesPoint> {
    let mut grouped: BTreeMap<NaiveDate, Vec<&Trade>> = BTreeMap::new();
    for t in trades {
        if let Some(closed) = t.closed_at {
            grouped.entry(closed.date_naive()).or_default().push(t);
        }
    }
    let mut out = Vec::with_capacity(grouped.len());
    let mut cum_pnl = Decimal::ZERO;
    let mut cum_count: usize = 0;
    let mut cum_wins: usize = 0;
    for (day, day_trades) in grouped {
        let mut day_pnl = Decimal::ZERO;
        let mut day_vol = Decimal::ZERO;
        let mut day_wins = 0usize;
        for t in &day_trades {
            let pnl = t.net_pnl.unwrap_or(Decimal::ZERO);
            day_pnl += pnl;
            day_vol += t.qty * t.entry_avg;
            if pnl > Decimal::ZERO {
                day_wins += 1;
            }
        }
        cum_pnl += day_pnl;
        cum_count += day_trades.len();
        cum_wins += day_wins;
        let running_avg = if cum_count > 0 {
            cum_pnl / Decimal::from(cum_count as i64)
        } else {
            Decimal::ZERO
        };
        let running_win_rate = if cum_count > 0 {
            cum_wins as f64 / cum_count as f64
        } else {
            0.0
        };
        out.push(DailySeriesPoint {
            day,
            volume: day_vol,
            trades: day_trades.len(),
            winners: day_wins,
            net_pnl: day_pnl,
            running_avg_pnl: running_avg,
            running_win_rate,
        });
    }
    out
}

// ===========================================================================
// Winning days vs losing days breakdown (Tradervue "Win vs Loss Days")
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WinLossDays {
    /// Buckets keyed by day-of-week / hour / etc.; one set per cohort.
    pub by_dow: WinLossSplit,
    pub by_hour: WinLossSplit,
    pub by_hold: WinLossSplit,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WinLossSplit {
    pub winning_days: Vec<Bucket>,
    pub losing_days: Vec<Bucket>,
}

/// Split trades by whether their closed_at falls on a net-winning vs net-losing
/// day, then re-bucket each cohort. Mirrors Tradervue's "Win vs Loss Days" report.
pub fn win_loss_days(trades: &[Trade]) -> WinLossDays {
    use std::collections::HashMap;
    let mut day_pnl: HashMap<NaiveDate, Decimal> = HashMap::new();
    for t in trades {
        if let Some(closed) = t.closed_at {
            *day_pnl.entry(closed.date_naive()).or_default() += t.net_pnl.unwrap_or(Decimal::ZERO);
        }
    }
    let (winning_set, losing_set): (Vec<_>, Vec<_>) = trades
        .iter()
        .filter(|t| t.closed_at.is_some())
        .partition(|t| {
            let day = t.closed_at.unwrap().date_naive();
            day_pnl.get(&day).copied().unwrap_or(Decimal::ZERO) > Decimal::ZERO
        });
    WinLossDays {
        by_dow: WinLossSplit {
            winning_days: by_day_of_week(
                &winning_set.iter().map(|t| (*t).clone()).collect::<Vec<_>>(),
            ),
            losing_days: by_day_of_week(
                &losing_set.iter().map(|t| (*t).clone()).collect::<Vec<_>>(),
            ),
        },
        by_hour: WinLossSplit {
            winning_days: by_hour_of_day(
                &winning_set.iter().map(|t| (*t).clone()).collect::<Vec<_>>(),
            ),
            losing_days: by_hour_of_day(
                &losing_set.iter().map(|t| (*t).clone()).collect::<Vec<_>>(),
            ),
        },
        by_hold: WinLossSplit {
            winning_days: by_hold_bucket(
                &winning_set.iter().map(|t| (*t).clone()).collect::<Vec<_>>(),
            ),
            losing_days: by_hold_bucket(
                &losing_set.iter().map(|t| (*t).clone()).collect::<Vec<_>>(),
            ),
        },
    }
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
        avg_hold_seconds: if win_n > 0 {
            win_hold / win_n as i64
        } else {
            0
        },
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
// By-tag (callers supply trade_id → tag_names mapping)
// ===========================================================================

/// Group closed trades by attached tag. Trades with multiple tags contribute
/// to each tag's bucket (so totals across all buckets can exceed total trade
/// count). Untagged trades are dropped — that's intentional; the "no tag"
/// case is shown as a separate stat in the UI.
pub fn by_tag(
    trades: &[Trade],
    tags_by_trade: &std::collections::HashMap<uuid::Uuid, Vec<String>>,
) -> Vec<Bucket> {
    let mut map: BTreeMap<String, Bucket> = BTreeMap::new();
    for t in trades {
        if t.status != TradeStatus::Closed {
            continue;
        }
        let Some(names) = tags_by_trade.get(&t.id) else {
            continue;
        };
        let net = t.net_pnl.unwrap_or(Decimal::ZERO);
        let gross = t.gross_pnl.unwrap_or(Decimal::ZERO);
        for name in names {
            let b = map.entry(name.clone()).or_insert_with(|| Bucket {
                key: name.clone(),
                ..Default::default()
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
    }
    for b in map.values_mut() {
        b.win_rate = b.wins as f64 / b.trades.max(1) as f64;
        b.avg_pnl = b.net_pnl / Decimal::from(b.trades.max(1) as u64);
        b.expectancy = b.avg_pnl;
    }
    map.into_values().collect()
}

// ===========================================================================
// Advanced report — per-trade scatter + cumulative curve
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScatterPoint {
    pub trade_id: uuid::Uuid,
    pub symbol: String,
    /// closed_at as ISO date (YYYY-MM-DD); for open trades, opened_at.
    pub day: String,
    pub net_pnl: Decimal,
    pub r: Option<f64>,
    pub hold_seconds: Option<i64>,
    pub qty: Decimal,
    pub win: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Advanced {
    pub cum_curve: Vec<EquityPoint>,
    pub scatter: Vec<ScatterPoint>,
}

pub fn advanced(trades: &[Trade], starting_cash: Decimal) -> Advanced {
    let cum_curve = equity_curve(trades, starting_cash);
    let mut scatter: Vec<ScatterPoint> = trades
        .iter()
        .filter(|t| t.status == TradeStatus::Closed)
        .map(|t| {
            let net = t.net_pnl.unwrap_or(Decimal::ZERO);
            let day = t
                .closed_at
                .map(|d| d.date_naive())
                .unwrap_or_else(|| t.opened_at.date_naive())
                .format("%Y-%m-%d")
                .to_string();
            let hold = t.closed_at.map(|c| (c - t.opened_at).num_seconds());
            let r = t.risk_amount.and_then(|risk| {
                if risk.is_zero() {
                    None
                } else {
                    Some(decimal_to_f64(net / risk))
                }
            });
            let win = if net > Decimal::ZERO {
                Some(true)
            } else if net < Decimal::ZERO {
                Some(false)
            } else {
                None
            };
            ScatterPoint {
                trade_id: t.id,
                symbol: t.symbol.clone(),
                day,
                net_pnl: net,
                r,
                hold_seconds: hold,
                qty: t.qty,
                win,
            }
        })
        .collect();
    scatter.sort_by(|a, b| a.day.cmp(&b.day));
    Advanced { cum_curve, scatter }
}

// ===========================================================================
// Helpers
// ===========================================================================

fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetClass, Trade, TradeSide, TradeStatus};
    use chrono::{TimeZone, Utc};
    use std::str::FromStr;
    use uuid::Uuid;

    /// Local shorthand — `rust_decimal_macros::dec!` isn't a workspace dep,
    /// so reconstruct a Decimal from a string literal. Keeps tests readable
    /// without growing the dependency graph.
    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).expect("valid decimal literal in test")
    }

    /// Build a minimal closed trade. We force net_pnl + risk_amount directly
    /// rather than computing from entry/exit so tests assert against exact
    /// hand-rolled numbers without floating-point drift.
    fn t(symbol: &str, side: TradeSide, net_pnl: Decimal, day: u32) -> Trade {
        Trade {
            id: Uuid::new_v4(),
            account_id: Uuid::nil(),
            symbol: symbol.into(),
            side,
            status: TradeStatus::Closed,
            opened_at: Utc.with_ymd_and_hms(2026, 5, day, 9, 30, 0).unwrap(),
            closed_at: Some(Utc.with_ymd_and_hms(2026, 5, day, 15, 30, 0).unwrap()),
            qty: dec("100"),
            entry_avg: dec("10"),
            exit_avg: Some(dec("10") + net_pnl / dec("100")),
            gross_pnl: Some(net_pnl),
            fees: Decimal::ZERO,
            commissions: Decimal::ZERO,
            net_pnl: Some(net_pnl),
            asset_class: AssetClass::Stock,
            option_type: None,
            strike: None,
            expiration: None,
            multiplier: dec("1"),
            tick_size: None,
            tick_value: None,
            base_ccy: None,
            quote_ccy: None,
            pip_size: None,
            stop_loss: None,
            risk_amount: Some(dec("100")), // 1R = $100
            initial_target: None,
            mfe: None,
            mae: None,
            best_exit_pnl: None,
            exit_efficiency: None,
        }
    }

    /// Sequence-indexed trade. `seq` maps to a real calendar date by
    /// walking forward from 2026-05-01, so callers can build batches of
    /// >28 trades without hitting "invalid day of month".
    fn ts(symbol: &str, side: TradeSide, net_pnl: Decimal, seq: u32) -> Trade {
        let base = chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let dt = base + chrono::Duration::days((seq.saturating_sub(1)) as i64);
        let opened = Utc
            .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 9, 30, 0)
            .unwrap();
        let closed = Utc
            .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 15, 30, 0)
            .unwrap();
        let mut tr = t(symbol, side, net_pnl, 1);
        tr.opened_at = opened;
        tr.closed_at = Some(closed);
        tr
    }

    fn open_trade(symbol: &str) -> Trade {
        let mut tr = t(symbol, TradeSide::Long, Decimal::ZERO, 1);
        tr.status = TradeStatus::Open;
        tr.closed_at = None;
        tr.net_pnl = None;
        tr.gross_pnl = None;
        tr
    }

    // ─── summary ──────────────────────────────────────────────────────────

    #[test]
    fn summary_empty_input_returns_zeros() {
        let s = summary(&[]);
        assert_eq!(s.trade_count, 0);
        assert_eq!(s.win_count, 0);
        assert_eq!(s.loss_count, 0);
        assert_eq!(s.net_pnl, Decimal::ZERO);
        assert_eq!(s.win_rate, 0.0);
        assert_eq!(s.profit_factor, 0.0);
    }

    #[test]
    fn summary_single_winner() {
        let s = summary(&[t("AAPL", TradeSide::Long, dec("500"), 1)]);
        assert_eq!(s.trade_count, 1);
        assert_eq!(s.win_count, 1);
        assert_eq!(s.loss_count, 0);
        assert_eq!(s.net_pnl, dec("500"));
        assert_eq!(s.avg_win, dec("500"));
        assert_eq!(s.win_rate, 1.0);
        assert_eq!(s.expectancy, dec("500"));
        assert_eq!(s.largest_win, dec("500"));
    }

    #[test]
    fn summary_single_loser() {
        let s = summary(&[t("AAPL", TradeSide::Long, dec("-200"), 1)]);
        assert_eq!(s.win_count, 0);
        assert_eq!(s.loss_count, 1);
        assert_eq!(s.net_pnl, dec("-200"));
        assert_eq!(s.avg_loss, dec("-200"));
        assert_eq!(s.largest_loss, dec("-200"));
        assert_eq!(s.win_rate, 0.0);
    }

    #[test]
    fn summary_scratch_trade_excluded_from_win_loss() {
        let s = summary(&[t("AAPL", TradeSide::Long, dec("0"), 1)]);
        assert_eq!(s.trade_count, 1);
        assert_eq!(s.win_count, 0);
        assert_eq!(s.loss_count, 0);
        assert_eq!(s.scratch_count, 1);
    }

    #[test]
    fn summary_open_trades_counted_separately() {
        let s = summary(&[
            t("AAPL", TradeSide::Long, dec("100"), 1),
            open_trade("TSLA"),
        ]);
        assert_eq!(s.trade_count, 1, "open trades excluded from trade_count");
        assert_eq!(s.open_count, 1);
    }

    #[test]
    fn summary_win_rate_and_expectancy() {
        // 3 wins of $100, 1 loss of -$50 → net $250 over 4 trades.
        let s = summary(&[
            t("AAPL", TradeSide::Long, dec("100"), 1),
            t("AAPL", TradeSide::Long, dec("100"), 2),
            t("AAPL", TradeSide::Long, dec("100"), 3),
            t("AAPL", TradeSide::Long, dec("-50"), 4),
        ]);
        assert_eq!(s.trade_count, 4);
        assert_eq!(s.win_count, 3);
        assert_eq!(s.loss_count, 1);
        assert_eq!(s.net_pnl, dec("250"));
        assert!((s.win_rate - 0.75).abs() < 1e-9);
        assert_eq!(s.expectancy, dec("62.5")); // 250 / 4
        assert_eq!(s.avg_win, dec("100"));
        assert_eq!(s.avg_loss, dec("-50"));
    }

    #[test]
    fn summary_profit_factor_handles_no_losses() {
        // Pure winners → profit_factor = +Inf, not 0 or NaN.
        let s = summary(&[
            t("AAPL", TradeSide::Long, dec("100"), 1),
            t("AAPL", TradeSide::Long, dec("200"), 2),
        ]);
        assert!(s.profit_factor.is_infinite());
        assert!(s.profit_factor > 0.0);
    }

    #[test]
    fn summary_profit_factor_normal() {
        // Wins $300, losses -$100 → profit factor 3.0
        let s = summary(&[
            t("AAPL", TradeSide::Long, dec("300"), 1),
            t("AAPL", TradeSide::Long, dec("-100"), 2),
        ]);
        assert!(
            (s.profit_factor - 3.0).abs() < 1e-9,
            "expected profit_factor=3.0 got {}",
            s.profit_factor
        );
    }

    #[test]
    fn summary_max_consec_wins_and_losses() {
        // W, W, W, L, W, L, L, L, L → wins streak 3, loss streak 4.
        let pnls = [100, 100, 100, -50, 100, -50, -50, -50, -50];
        let trades: Vec<Trade> = pnls
            .iter()
            .enumerate()
            .map(|(i, p)| t("AAPL", TradeSide::Long, Decimal::from(*p), (i + 1) as u32))
            .collect();
        let s = summary(&trades);
        assert_eq!(s.max_consec_wins, 3);
        assert_eq!(s.max_consec_losses, 4);
    }

    #[test]
    fn summary_avg_r_uses_risk_amount() {
        // All trades have risk_amount=$100. P&L of 250 → R=2.5.
        let s = summary(&[
            t("AAPL", TradeSide::Long, dec("200"), 1),
            t("AAPL", TradeSide::Long, dec("300"), 2),
        ]);
        // avg R = (200/100 + 300/100) / 2 = 2.5
        assert!((s.avg_r - 2.5).abs() < 1e-9, "got avg_r={}", s.avg_r);
    }

    // ─── extended summary fields (Tradervue-parity) ───────────────────────

    #[test]
    fn summary_trading_days_counts_distinct_close_dates() {
        // Three trades over two distinct closing days.
        let s = summary(&[
            t("S", TradeSide::Long, dec("100"), 1),
            t("S", TradeSide::Long, dec("50"), 1),
            t("S", TradeSide::Long, dec("75"), 2),
        ]);
        assert_eq!(s.trading_days, 2);
        // 225 / 2 = 112.5
        assert_eq!(s.avg_daily_pnl, dec("112.5"));
    }

    #[test]
    fn summary_avg_daily_volume_divides_by_trading_days() {
        // Each trade volume = qty(100) * entry(10) * mult(1) = 1000. Two days.
        let s = summary(&[
            t("S", TradeSide::Long, dec("0"), 1),
            t("S", TradeSide::Long, dec("0"), 2),
        ]);
        assert_eq!(s.trading_days, 2);
        assert_eq!(s.total_volume, dec("2000"));
        assert_eq!(s.avg_daily_volume, dec("1000"));
    }

    #[test]
    fn summary_avg_per_share_uses_total_qty() {
        // 100 shares × 2 trades = 200 total. Net P&L = 200 → $1/share.
        let s = summary(&[
            t("S", TradeSide::Long, dec("100"), 1),
            t("S", TradeSide::Long, dec("100"), 2),
        ]);
        assert_eq!(s.total_shares, dec("200"));
        assert_eq!(s.avg_per_share_pnl, dec("1"));
    }

    #[test]
    fn summary_net_pnl_stddev_matches_sample_variance() {
        // P&Ls [100, -100, 100, -100] → mean 0, sample-var = (4 × 10000) / 3
        // → std = sqrt(40000/3) ≈ 115.47005.
        let s = summary(&[
            t("S", TradeSide::Long, dec("100"), 1),
            t("S", TradeSide::Long, dec("-100"), 2),
            t("S", TradeSide::Long, dec("100"), 3),
            t("S", TradeSide::Long, dec("-100"), 4),
        ]);
        let expected = (40000.0_f64 / 3.0).sqrt();
        assert!(
            (s.net_pnl_stddev - expected).abs() < 1e-6,
            "got stddev={} expected≈{}",
            s.net_pnl_stddev,
            expected
        );
    }

    #[test]
    fn summary_kelly_pct_classic_formula() {
        // Win rate 0.6, avg_win 200, avg_loss -100 → payoff 2.0.
        // Kelly = 0.6 - 0.4/2 = 0.4.
        let mut trades = vec![];
        for d in 1..=6 {
            trades.push(t("S", TradeSide::Long, dec("200"), d));
        }
        for d in 7..=10 {
            trades.push(t("S", TradeSide::Long, dec("-100"), d));
        }
        let s = summary(&trades);
        assert!((s.win_rate - 0.6).abs() < 1e-9);
        let k = s.kelly_pct.expect("kelly defined");
        assert!((k - 0.4).abs() < 1e-9, "got kelly={}", k);
    }

    #[test]
    fn summary_kelly_pct_none_when_no_losers() {
        let s = summary(&[t("S", TradeSide::Long, dec("100"), 1)]);
        assert!(s.kelly_pct.is_none());
    }

    #[test]
    fn summary_sqn_positive_for_winning_system() {
        // Steady positive R-multiples (200/100=2) for several trades and a
        // small spread → SQN should be positive and finite.
        let trades: Vec<Trade> = (1..=10)
            .map(|d| t("S", TradeSide::Long, dec("200"), d))
            .chain((11..=15).map(|d| t("S", TradeSide::Long, dec("150"), d)))
            .collect();
        let s = summary(&trades);
        let sqn = s.sqn.expect("sqn defined when r-spread > 0");
        assert!(sqn > 0.0 && sqn.is_finite(), "got sqn={}", sqn);
    }

    #[test]
    fn summary_sqn_none_for_constant_r() {
        // Zero variance in R → undefined SQN.
        let trades: Vec<Trade> = (1..=5)
            .map(|d| t("S", TradeSide::Long, dec("100"), d))
            .collect();
        let s = summary(&trades);
        assert!(s.sqn.is_none(), "got sqn={:?}", s.sqn);
    }

    #[test]
    fn summary_random_chance_low_for_strong_edge() {
        // 80% wins over 50 trades → binomial p(X≥40 | n=50,p=0.5) ≈ 3.3e-6.
        let mut trades = vec![];
        for d in 1..=40 {
            trades.push(ts("S", TradeSide::Long, dec("100"), d));
        }
        for d in 41..=50 {
            trades.push(ts("S", TradeSide::Long, dec("-100"), d));
        }
        let s = summary(&trades);
        let p = s.random_chance_prob.expect("p defined");
        assert!(p < 0.001, "got p={}", p);
    }

    #[test]
    fn summary_random_chance_about_half_for_50_50() {
        // 25 wins / 25 losses → continuity-corrected p ≈ 0.56.
        let mut trades = vec![];
        for d in 1..=25 {
            trades.push(ts("S", TradeSide::Long, dec("100"), d));
        }
        for d in 26..=50 {
            trades.push(ts("S", TradeSide::Long, dec("-100"), d));
        }
        let s = summary(&trades);
        let p = s.random_chance_prob.expect("p defined");
        assert!(
            (p - 0.56).abs() < 0.05,
            "got p={} (expected ≈0.56 with continuity correction)",
            p
        );
    }

    #[test]
    fn summary_k_ratio_positive_for_noisy_uptrend() {
        // Mostly $100 winners with a few $50 winners — slope is positive,
        // residuals are non-zero, so k-ratio is defined and positive.
        let pnls = [100, 100, 50, 100, 100, 50, 100, 100, 100, 50];
        let trades: Vec<Trade> = pnls
            .iter()
            .enumerate()
            .map(|(i, p)| ts("S", TradeSide::Long, Decimal::from(*p), (i + 1) as u32))
            .collect();
        let s = summary(&trades);
        let k = s.k_ratio.expect("k-ratio defined");
        assert!(k > 0.0 && k.is_finite(), "got k_ratio={}", k);
    }

    #[test]
    fn summary_k_ratio_none_for_fewer_than_three() {
        let s = summary(&[
            t("S", TradeSide::Long, dec("100"), 1),
            t("S", TradeSide::Long, dec("-50"), 2),
        ]);
        assert!(s.k_ratio.is_none());
    }

    #[test]
    fn summary_avg_mae_mfe_uses_only_set_values() {
        let mut a = t("S", TradeSide::Long, dec("100"), 1);
        a.mae = Some(dec("-30"));
        a.mfe = Some(dec("80"));
        let mut b = t("S", TradeSide::Long, dec("-50"), 2);
        b.mae = Some(dec("-90"));
        // b.mfe left None — must not pollute the mfe average.
        let mut c = t("S", TradeSide::Long, dec("20"), 3);
        c.mfe = Some(dec("40"));
        // c.mae None.
        let s = summary(&[a, b, c]);
        // mae avg over 2: (-30 + -90) / 2 = -60
        assert_eq!(s.avg_mae, dec("-60"));
        // mfe avg over 2: (80 + 40) / 2 = 60
        assert_eq!(s.avg_mfe, dec("60"));
    }

    #[test]
    fn summary_scratch_hold_isolated_from_winloss_holds() {
        // Default helper hold = 6h (09:30 → 15:30) = 21600 s.
        let s = summary(&[
            t("S", TradeSide::Long, dec("100"), 1),
            t("S", TradeSide::Long, dec("-50"), 2),
            t("S", TradeSide::Long, dec("0"), 3),
        ]);
        assert_eq!(s.avg_scratch_hold_seconds, 21600);
        assert_eq!(s.avg_win_hold_seconds, 21600);
        assert_eq!(s.avg_loss_hold_seconds, 21600);
    }

    // ─── normal_cdf ───────────────────────────────────────────────────────

    #[test]
    fn normal_cdf_known_values() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-6);
        // Φ(1.96) ≈ 0.975, Φ(-1.96) ≈ 0.025
        assert!((normal_cdf(1.96) - 0.975).abs() < 1e-3);
        assert!((normal_cdf(-1.96) - 0.025).abs() < 1e-3);
    }

    // ─── equity_curve ─────────────────────────────────────────────────────

    #[test]
    fn equity_curve_aggregates_by_day() {
        // Two trades on day 1, one trade on day 2.
        let trades = vec![
            t("AAPL", TradeSide::Long, dec("100"), 1),
            t("TSLA", TradeSide::Long, dec("50"), 1),
            t("NVDA", TradeSide::Long, dec("-30"), 2),
        ];
        let eq = equity_curve(&trades, dec("10000"));
        assert_eq!(eq.len(), 2);
        assert_eq!(eq[0].day_net_pnl, dec("150"), "day 1 = 100 + 50");
        assert_eq!(eq[0].cum_net_pnl, dec("150"));
        assert_eq!(eq[0].trades, 2);
        assert_eq!(eq[1].day_net_pnl, dec("-30"));
        assert_eq!(eq[1].cum_net_pnl, dec("120"), "150 - 30");
    }

    #[test]
    fn equity_curve_drawdown_is_negative_below_peak() {
        let trades = vec![
            t("AAPL", TradeSide::Long, dec("500"), 1),
            t("AAPL", TradeSide::Long, dec("-200"), 2),
        ];
        let eq = equity_curve(&trades, dec("10000"));
        assert_eq!(eq[0].drawdown, dec("0")); // at peak
        assert_eq!(eq[1].drawdown, dec("-200")); // 200 below peak of 500
    }

    #[test]
    fn equity_curve_skips_open_trades() {
        let eq = equity_curve(&[open_trade("AAPL")], dec("10000"));
        assert_eq!(eq.len(), 0);
    }

    // ─── max_drawdown ─────────────────────────────────────────────────────

    #[test]
    fn max_drawdown_finds_largest_peak_to_trough() {
        // Curve goes 100, 300, 100, 50, 200 → max DD = 300 - 50 = 250.
        let trades = vec![
            t("S", TradeSide::Long, dec("100"), 1),
            t("S", TradeSide::Long, dec("200"), 2),
            t("S", TradeSide::Long, dec("-200"), 3),
            t("S", TradeSide::Long, dec("-50"), 4),
            t("S", TradeSide::Long, dec("150"), 5),
        ];
        let eq = equity_curve(&trades, dec("10000"));
        let dd = max_drawdown(&eq);
        assert_eq!(
            dd.max_dd,
            dec("-250"),
            "peak was 300 on day 2, trough 50 on day 4 → -250"
        );
    }

    #[test]
    fn max_drawdown_empty_curve_is_zero() {
        let dd = max_drawdown(&[]);
        assert_eq!(dd.max_dd, Decimal::ZERO);
        assert_eq!(dd.max_dd_pct, 0.0);
    }

    // ─── by_symbol / by_side ──────────────────────────────────────────────

    #[test]
    fn by_symbol_groups_and_sums_per_ticker() {
        let trades = vec![
            t("AAPL", TradeSide::Long, dec("100"), 1),
            t("AAPL", TradeSide::Long, dec("200"), 2),
            t("TSLA", TradeSide::Long, dec("50"), 3),
        ];
        let buckets = by_symbol(&trades);
        let aapl = buckets
            .iter()
            .find(|b| b.key == "AAPL")
            .expect("AAPL bucket");
        let tsla = buckets
            .iter()
            .find(|b| b.key == "TSLA")
            .expect("TSLA bucket");
        assert_eq!(aapl.trades, 2);
        assert_eq!(aapl.net_pnl, dec("300"));
        assert_eq!(tsla.trades, 1);
        assert_eq!(tsla.net_pnl, dec("50"));
    }

    #[test]
    fn by_side_splits_long_short() {
        let trades = vec![
            t("AAPL", TradeSide::Long, dec("100"), 1),
            t("TSLA", TradeSide::Short, dec("50"), 2),
        ];
        let buckets = by_side(&trades);
        assert_eq!(buckets.len(), 2);
        let long = buckets.iter().find(|b| b.key == "long").unwrap();
        let short = buckets.iter().find(|b| b.key == "short").unwrap();
        assert_eq!(long.net_pnl, dec("100"));
        assert_eq!(short.net_pnl, dec("50"));
    }

    #[test]
    fn by_tag_buckets_only_tagged_trades_and_sums_per_tag() {
        // Three closed trades. tA is tagged "breakout" + "morning";
        // tB only "breakout"; tC untagged. Expect "breakout" bucket
        // to aggregate both A+B, "morning" only A, and tC to be skipped.
        let t1 = t("AAA", TradeSide::Long, dec("100"), 1);
        let t2 = t("BBB", TradeSide::Long, dec("-50"), 2);
        let t3 = t("CCC", TradeSide::Long, dec("999"), 3); // untagged
        let trades = vec![t1.clone(), t2.clone(), t3.clone()];
        let mut tags = std::collections::HashMap::new();
        tags.insert(t1.id, vec!["breakout".to_string(), "morning".to_string()]);
        tags.insert(t2.id, vec!["breakout".to_string()]);

        let buckets = by_tag(&trades, &tags);
        // Untagged trade contributes nothing.
        let breakout = buckets.iter().find(|b| b.key == "breakout").unwrap();
        assert_eq!(breakout.trades, 2);
        assert_eq!(breakout.wins, 1);
        assert_eq!(breakout.losses, 1);
        assert_eq!(breakout.net_pnl, dec("50"));

        let morning = buckets.iter().find(|b| b.key == "morning").unwrap();
        assert_eq!(morning.trades, 1);
        assert_eq!(morning.net_pnl, dec("100"));
        assert!((morning.win_rate - 1.0).abs() < 1e-9);

        assert!(buckets.iter().all(|b| b.key != "untagged"));
    }

    #[test]
    fn advanced_returns_curve_and_scatter_sorted_by_day() {
        // Two closed trades on day 1 and day 3. Scatter sorted by day.
        let trades = vec![
            t("AAA", TradeSide::Long, dec("100"), 3),
            t("BBB", TradeSide::Long, dec("-50"), 1),
        ];
        let adv = advanced(&trades, Decimal::ZERO);
        assert!(!adv.cum_curve.is_empty());
        assert_eq!(adv.scatter.len(), 2);
        assert!(adv.scatter[0].day < adv.scatter[1].day);
        assert_eq!(adv.scatter[0].net_pnl, dec("-50"));
        assert_eq!(adv.scatter[0].win, Some(false));
        assert_eq!(adv.scatter[1].win, Some(true));
    }
}
