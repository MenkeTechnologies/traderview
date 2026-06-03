//! 2D heatmap: day-of-week × hour-of-day P&L.
//!
//! `stats::by_day_of_week` and `stats::by_hour_of_day` exist but lose
//! information when one axis is collapsed. The classic example: "I'm
//! great Mon-Fri average, but terrible specifically at 09:30 ET on
//! Mondays." Only a 2D cut shows that.
//!
//! Output: a fixed 7×24 grid. Each cell carries trades + net P&L +
//! win-rate. Empty cells stay zero so the UI can render the full grid
//! without conditional gaps.

use crate::models::{Trade, TradeStatus};
use chrono::{Datelike, Timelike};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct HeatCell {
    pub trades: u32,
    pub wins: u32,
    pub net_pnl: Decimal,
}

impl HeatCell {
    pub fn win_rate(&self) -> f64 {
        if self.trades == 0 {
            0.0
        } else {
            self.wins as f64 / self.trades as f64
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DowHourHeatmap {
    /// `cells[dow][hour]` where dow is 0=Sun..6=Sat, hour is 0..=23.
    pub cells: Vec<Vec<HeatCell>>,
    pub total_trades: u32,
    pub total_pnl: Decimal,
}

impl Default for DowHourHeatmap {
    fn default() -> Self {
        Self {
            cells: vec![vec![HeatCell::default(); 24]; 7],
            total_trades: 0,
            total_pnl: Decimal::ZERO,
        }
    }
}

pub fn build(trades: &[Trade]) -> DowHourHeatmap {
    let mut h = DowHourHeatmap::default();
    for t in trades {
        if t.status != TradeStatus::Closed {
            continue;
        }
        let Some(net) = t.net_pnl else { continue };
        // Use opened_at — entries determine when the user was at the desk.
        let dow = t.opened_at.weekday().num_days_from_sunday() as usize; // 0..6
        let hour = t.opened_at.hour() as usize; // 0..23
        if dow >= 7 || hour >= 24 {
            continue;
        } // belt + braces
        let cell = &mut h.cells[dow][hour];
        cell.trades += 1;
        cell.net_pnl += net;
        if net > Decimal::ZERO {
            cell.wins += 1;
        }
        h.total_trades += 1;
        h.total_pnl += net;
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetClass, TradeSide};
    use chrono::{TimeZone, Utc};
    use std::str::FromStr;
    use uuid::Uuid;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn trade_at(year: i32, month: u32, day: u32, hour: u32, net_pnl: &str) -> Trade {
        Trade {
            id: Uuid::new_v4(),
            account_id: Uuid::nil(),
            symbol: "X".into(),
            side: TradeSide::Long,
            status: TradeStatus::Closed,
            opened_at: Utc.with_ymd_and_hms(year, month, day, hour, 30, 0).unwrap(),
            closed_at: Some(Utc.with_ymd_and_hms(year, month, day, hour, 45, 0).unwrap()),
            qty: d("100"),
            entry_avg: d("50"),
            exit_avg: Some(d("51")),
            gross_pnl: Some(d(net_pnl)),
            fees: Decimal::ZERO,
            commissions: Decimal::ZERO,
            net_pnl: Some(d(net_pnl)),
            asset_class: AssetClass::Stock,
            option_type: None,
            strike: None,
            expiration: None,
            multiplier: Decimal::ONE,
            tick_size: None,
            tick_value: None,
            base_ccy: None,
            quote_ccy: None,
            pip_size: None,
            stop_loss: None,
            risk_amount: None,
            initial_target: None,
            mfe: None,
            mae: None,
            best_exit_pnl: None,
            exit_efficiency: None,
        }
    }

    #[test]
    fn empty_input_returns_zero_filled_grid() {
        let h = build(&[]);
        assert_eq!(h.cells.len(), 7);
        assert!(h.cells.iter().all(|row| row.len() == 24));
        assert_eq!(h.total_trades, 0);
        assert!(h.cells.iter().all(|row| row.iter().all(|c| c.trades == 0)));
    }

    #[test]
    fn single_trade_lands_in_correct_dow_hour_cell() {
        // 2026-05-26 is a Tuesday (weekday 2 from Sunday).
        let t = trade_at(2026, 5, 26, 10, "100");
        let h = build(&[t]);
        let cell = h.cells[2][10];
        assert_eq!(cell.trades, 1);
        assert_eq!(cell.wins, 1);
        assert_eq!(cell.net_pnl, d("100"));
        // Surrounding cells must stay zero.
        assert_eq!(h.cells[2][11].trades, 0);
        assert_eq!(h.cells[1][10].trades, 0);
    }

    #[test]
    fn multiple_trades_in_same_cell_aggregate() {
        let trades = vec![
            trade_at(2026, 5, 26, 10, "100"),
            trade_at(2026, 5, 26, 10, "-50"),
            trade_at(2026, 5, 26, 10, "200"),
        ];
        let h = build(&trades);
        let cell = h.cells[2][10];
        assert_eq!(cell.trades, 3);
        assert_eq!(cell.wins, 2);
        assert_eq!(cell.net_pnl, d("250"));
        assert!((cell.win_rate() - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn open_trades_skipped() {
        let mut t = trade_at(2026, 5, 26, 10, "100");
        t.status = TradeStatus::Open;
        let h = build(&[t]);
        assert_eq!(h.total_trades, 0);
    }

    #[test]
    fn trade_without_net_pnl_skipped() {
        let mut t = trade_at(2026, 5, 26, 10, "0");
        t.net_pnl = None;
        let h = build(&[t]);
        assert_eq!(h.total_trades, 0);
    }

    #[test]
    fn cells_across_different_days_dont_bleed() {
        // Tue + Wed at 10am — separate cells.
        let trades = vec![
            trade_at(2026, 5, 26, 10, "100"), // Tue
            trade_at(2026, 5, 27, 10, "-50"), // Wed
        ];
        let h = build(&trades);
        assert_eq!(h.cells[2][10].net_pnl, d("100"));
        assert_eq!(h.cells[3][10].net_pnl, d("-50"));
        assert_eq!(h.total_trades, 2);
        assert_eq!(h.total_pnl, d("50"));
    }

    #[test]
    fn weekend_trades_land_in_dow_0_and_6() {
        // 2026-05-30 = Saturday, 2026-05-31 = Sunday.
        let trades = vec![
            trade_at(2026, 5, 30, 10, "100"),
            trade_at(2026, 5, 31, 10, "100"),
        ];
        let h = build(&trades);
        assert_eq!(h.cells[6][10].trades, 1); // Saturday = weekday 6 (Sun=0)
        assert_eq!(h.cells[0][10].trades, 1); // Sunday   = weekday 0
    }
}
