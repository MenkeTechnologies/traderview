//! Setup catalog — named trading setups with computed per-setup stats.
//!
//! A "setup" is a tag the trader applies to a trade (gap-and-go, ORB,
//! ABCD, reversal-at-VWAP, etc). This module rolls trades up by setup
//! tag and computes the standard stats (count, win rate, expectancy,
//! avg R, profit factor) so the trader can compare their setups and
//! kill the losers.
//!
//! Pure compute. Inputs: a slice of trades + a tag → setup mapping.

use crate::models::{Trade, TradeStatus};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStats {
    pub setup: String,
    pub trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub scratches: usize,
    pub net_pnl: Decimal,
    pub gross_pnl: Decimal,
    pub fees: Decimal,
    pub win_rate: f64,
    pub avg_pnl: Decimal,
    pub avg_win: Decimal,
    pub avg_loss: Decimal,
    pub profit_factor: f64,
    pub expectancy: Decimal,
    pub avg_r: f64,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
}

/// Build per-setup stats. `trade_setups` maps trade_id → setup name.
/// Trades missing from the map are skipped (untagged).
pub fn stats_by_setup(
    trades: &[Trade],
    trade_setups: &HashMap<uuid::Uuid, String>,
) -> Vec<SetupStats> {
    let mut by_setup: HashMap<String, Vec<&Trade>> = HashMap::new();
    for t in trades {
        if t.status != TradeStatus::Closed { continue; }
        if let Some(setup) = trade_setups.get(&t.id) {
            by_setup.entry(setup.clone()).or_default().push(t);
        }
    }

    let mut out: Vec<SetupStats> = by_setup.into_iter()
        .map(|(setup, ts)| compute_one(setup, &ts))
        .collect();
    // Sort by net_pnl descending — winning setups float to the top.
    out.sort_by(|a, b| b.net_pnl.cmp(&a.net_pnl));
    out
}

fn compute_one(setup: String, trades: &[&Trade]) -> SetupStats {
    let mut s = SetupStats {
        setup, trades: trades.len(), wins: 0, losses: 0, scratches: 0,
        net_pnl: Decimal::ZERO, gross_pnl: Decimal::ZERO, fees: Decimal::ZERO,
        win_rate: 0.0, avg_pnl: Decimal::ZERO,
        avg_win: Decimal::ZERO, avg_loss: Decimal::ZERO,
        profit_factor: 0.0, expectancy: Decimal::ZERO, avg_r: 0.0,
        largest_win: Decimal::ZERO, largest_loss: Decimal::ZERO,
    };
    let mut win_sum = Decimal::ZERO;
    let mut loss_sum = Decimal::ZERO;
    let mut r_sum = 0.0_f64;
    let mut r_count = 0usize;
    for t in trades {
        let net = t.net_pnl.unwrap_or(Decimal::ZERO);
        let gross = t.gross_pnl.unwrap_or(Decimal::ZERO);
        s.net_pnl += net;
        s.gross_pnl += gross;
        s.fees += t.fees;
        if net > Decimal::ZERO {
            s.wins += 1;
            win_sum += net;
            if net > s.largest_win { s.largest_win = net; }
        } else if net < Decimal::ZERO {
            s.losses += 1;
            loss_sum += net;
            if net < s.largest_loss { s.largest_loss = net; }
        } else {
            s.scratches += 1;
        }
        if let Some(r) = t.r_multiple() {
            r_sum += dec_to_f64(r);
            r_count += 1;
        }
    }
    if s.trades > 0 {
        s.win_rate = s.wins as f64 / s.trades as f64;
        s.avg_pnl = s.net_pnl / Decimal::from(s.trades as u64);
        s.expectancy = s.avg_pnl;
    }
    if s.wins > 0   { s.avg_win  = win_sum  / Decimal::from(s.wins   as u64); }
    if s.losses > 0 { s.avg_loss = loss_sum / Decimal::from(s.losses as u64); }
    let loss_abs = loss_sum.abs();
    s.profit_factor = if loss_abs.is_zero() {
        if win_sum.is_zero() { 0.0 } else { f64::INFINITY }
    } else {
        dec_to_f64(win_sum) / dec_to_f64(loss_abs)
    };
    if r_count > 0 { s.avg_r = r_sum / r_count as f64; }
    s
}

fn dec_to_f64(d: Decimal) -> f64 { d.to_string().parse().unwrap_or(0.0) }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetClass, TradeSide};
    use chrono::{TimeZone, Utc};
    use std::str::FromStr;
    use uuid::Uuid;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn closed_trade(net_pnl: &str, risk_amount: Option<&str>) -> Trade {
        Trade {
            id: Uuid::new_v4(),
            account_id: Uuid::nil(),
            symbol: "X".into(),
            side: TradeSide::Long,
            status: TradeStatus::Closed,
            opened_at: Utc.with_ymd_and_hms(2026, 1, 1, 9, 30, 0).unwrap(),
            closed_at: Some(Utc.with_ymd_and_hms(2026, 1, 1, 15, 30, 0).unwrap()),
            qty: d("100"),
            entry_avg: d("50"),
            exit_avg: Some(d("51")),
            gross_pnl: Some(d(net_pnl)),
            fees: Decimal::ZERO,
            net_pnl: Some(d(net_pnl)),
            asset_class: AssetClass::Stock,
            option_type: None, strike: None, expiration: None,
            multiplier: Decimal::ONE,
            tick_size: None, tick_value: None,
            base_ccy: None, quote_ccy: None, pip_size: None,
            stop_loss: None,
            risk_amount: risk_amount.map(d),
            initial_target: None,
            mfe: None, mae: None, best_exit_pnl: None, exit_efficiency: None,
        }
    }

    #[test]
    fn empty_input_returns_empty() {
        let v = stats_by_setup(&[], &HashMap::new());
        assert!(v.is_empty());
    }

    #[test]
    fn untagged_trades_skipped() {
        let t = closed_trade("100", None);
        let v = stats_by_setup(&[t], &HashMap::new());
        assert!(v.is_empty(), "no setup tag → not in the catalog");
    }

    #[test]
    fn open_trades_skipped() {
        let mut t = closed_trade("100", None);
        t.status = TradeStatus::Open;
        let mut map = HashMap::new();
        map.insert(t.id, "gap_and_go".into());
        let v = stats_by_setup(&[t], &map);
        // Open trades don't enter the catalog at all — by_setup empty.
        assert!(v.is_empty(), "open trades must not create empty setup buckets");
    }

    #[test]
    fn single_winning_setup_computes_correctly() {
        let t = closed_trade("500", Some("100"));   // +5R
        let mut map = HashMap::new();
        map.insert(t.id, "orb".into());
        let v = stats_by_setup(&[t], &map);
        assert_eq!(v.len(), 1);
        let s = &v[0];
        assert_eq!(s.setup, "orb");
        assert_eq!(s.trades, 1);
        assert_eq!(s.wins, 1);
        assert_eq!(s.net_pnl, d("500"));
        assert_eq!(s.avg_win, d("500"));
        assert!(s.profit_factor.is_infinite());
        assert!((s.avg_r - 5.0).abs() < 1e-9);
    }

    #[test]
    fn mixed_setup_aggregates_correctly() {
        let win1 = closed_trade("200", None);
        let win2 = closed_trade("300", None);
        let loss = closed_trade("-100", None);
        let mut map = HashMap::new();
        map.insert(win1.id, "abcd".into());
        map.insert(win2.id, "abcd".into());
        map.insert(loss.id, "abcd".into());
        let v = stats_by_setup(&[win1, win2, loss], &map);
        let s = &v[0];
        assert_eq!(s.trades, 3);
        assert_eq!(s.wins, 2);
        assert_eq!(s.losses, 1);
        assert_eq!(s.net_pnl, d("400"));
        assert_eq!(s.avg_win, d("250"));
        assert_eq!(s.avg_loss, d("-100"));
        assert!((s.win_rate - 2.0/3.0).abs() < 1e-9);
        // Profit factor = 500 / 100 = 5.0.
        assert!((s.profit_factor - 5.0).abs() < 1e-9);
    }

    #[test]
    fn results_sorted_by_net_pnl_descending() {
        let big_winner = closed_trade("1000", None);
        let small_winner = closed_trade("100", None);
        let loser = closed_trade("-500", None);
        let mut map = HashMap::new();
        map.insert(big_winner.id,   "winner_setup".into());
        map.insert(small_winner.id, "meh_setup".into());
        map.insert(loser.id,        "loser_setup".into());
        let v = stats_by_setup(&[big_winner, small_winner, loser], &map);
        assert_eq!(v.len(), 3);
        assert_eq!(v[0].setup, "winner_setup");
        assert_eq!(v[1].setup, "meh_setup");
        assert_eq!(v[2].setup, "loser_setup");
    }

    #[test]
    fn scratch_trades_counted_separately() {
        let t = closed_trade("0", None);
        let mut map = HashMap::new();
        map.insert(t.id, "boundary".into());
        let v = stats_by_setup(&[t], &map);
        assert_eq!(v[0].scratches, 1);
        assert_eq!(v[0].wins, 0);
        assert_eq!(v[0].losses, 0);
    }
}
