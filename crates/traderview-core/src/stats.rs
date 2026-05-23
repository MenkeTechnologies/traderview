//! Per-account statistics computed from closed trades.

use crate::models::Trade;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Summary {
    pub trade_count: usize,
    pub win_count: usize,
    pub loss_count: usize,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub day: NaiveDate,
    pub cum_net_pnl: Decimal,
    pub day_net_pnl: Decimal,
    pub trades: usize,
}

pub fn summary(trades: &[Trade]) -> Summary {
    let closed: Vec<&Trade> = trades.iter().filter(|t| t.net_pnl.is_some()).collect();
    if closed.is_empty() {
        return Summary::default();
    }

    let mut s = Summary::default();
    let mut win_sum = Decimal::ZERO;
    let mut loss_sum = Decimal::ZERO;

    for t in &closed {
        let net = t.net_pnl.unwrap_or(Decimal::ZERO);
        let gross = t.gross_pnl.unwrap_or(Decimal::ZERO);
        s.trade_count += 1;
        s.gross_pnl += gross;
        s.net_pnl += net;
        s.fees += t.fees;
        if net > Decimal::ZERO {
            s.win_count += 1;
            win_sum += net;
            if net > s.largest_win {
                s.largest_win = net;
            }
        } else if net < Decimal::ZERO {
            s.loss_count += 1;
            loss_sum += net;
            if net < s.largest_loss {
                s.largest_loss = net;
            }
        }
    }

    if s.win_count > 0 {
        s.avg_win = win_sum / Decimal::from(s.win_count as u64);
    }
    if s.loss_count > 0 {
        s.avg_loss = loss_sum / Decimal::from(s.loss_count as u64);
    }
    s.win_rate = s.win_count as f64 / s.trade_count as f64;

    let loss_abs = loss_sum.abs();
    s.profit_factor = if loss_abs.is_zero() {
        if win_sum.is_zero() { 0.0 } else { f64::INFINITY }
    } else {
        decimal_to_f64(win_sum) / decimal_to_f64(loss_abs)
    };

    if s.trade_count > 0 {
        s.expectancy = s.net_pnl / Decimal::from(s.trade_count as u64);
    }

    s
}

pub fn equity_curve(trades: &[Trade]) -> Vec<EquityPoint> {
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
    by_day
        .into_iter()
        .map(|(day, (day_net, n))| {
            cum += day_net;
            EquityPoint {
                day,
                cum_net_pnl: cum,
                day_net_pnl: day_net,
                trades: n,
            }
        })
        .collect()
}

fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[allow(dead_code)]
fn _force_use(_: DateTime<Utc>) {}
