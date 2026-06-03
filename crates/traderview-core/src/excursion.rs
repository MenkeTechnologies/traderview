//! MFE / MAE / best-exit P&L from price bars.
//!
//! Given a trade's open window `[opened_at, closed_at]` and the relevant
//! price bars at the finest available interval, compute:
//!   - **MFE** (max favorable excursion) — the best $ unrealized profit reached
//!   - **MAE** (max adverse excursion) — the worst $ unrealized drawdown
//!   - **best_exit_pnl** — what net P&L would have been if exited at the favorable extreme

use crate::models::{PriceBar, Trade, TradeSide};
use crate::pnl::{gross_pnl, PricePoint};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy)]
pub struct Excursion {
    pub mfe: Decimal,
    pub mae: Decimal,
    pub best_exit_pnl: Decimal,
}

pub fn compute_excursion(trade: &Trade, bars: &[PriceBar]) -> Excursion {
    let open = trade.opened_at;
    let close = trade.closed_at.unwrap_or(chrono::Utc::now());
    let mut mfe = Decimal::ZERO;
    let mut mae = Decimal::ZERO;
    let mut best_price = trade.entry_avg;

    for b in bars {
        if b.bar_time < open || b.bar_time > close {
            continue;
        }
        // For longs, favorable = high, adverse = low.
        // For shorts, favorable = low, adverse = high.
        let (fav, adv) = match trade.side {
            TradeSide::Long => (b.high, b.low),
            TradeSide::Short => (b.low, b.high),
        };
        let fav_pnl = gross_pnl(
            trade.asset_class,
            trade.side,
            trade.qty,
            PricePoint {
                entry: trade.entry_avg,
                exit: fav,
                multiplier: trade.multiplier,
                tick_size: trade.tick_size,
                tick_value: trade.tick_value,
            },
        );
        let adv_pnl = gross_pnl(
            trade.asset_class,
            trade.side,
            trade.qty,
            PricePoint {
                entry: trade.entry_avg,
                exit: adv,
                multiplier: trade.multiplier,
                tick_size: trade.tick_size,
                tick_value: trade.tick_value,
            },
        );
        if fav_pnl > mfe {
            mfe = fav_pnl;
            best_price = fav;
        }
        if adv_pnl < mae {
            mae = adv_pnl;
        }
    }
    let best_exit_pnl = gross_pnl(
        trade.asset_class,
        trade.side,
        trade.qty,
        PricePoint {
            entry: trade.entry_avg,
            exit: best_price,
            multiplier: trade.multiplier,
            tick_size: trade.tick_size,
            tick_value: trade.tick_value,
        },
    ) - trade.fees;
    Excursion {
        mfe,
        mae,
        best_exit_pnl,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetClass, TradeStatus};
    use chrono::{Duration, TimeZone, Utc};
    use rust_decimal::Decimal;
    use uuid::Uuid;

    // ===========================================================================
    // Fixtures
    // ===========================================================================

    fn base_trade(side: TradeSide, entry: Decimal, qty: Decimal) -> Trade {
        Trade {
            id: Uuid::nil(),
            account_id: Uuid::nil(),
            symbol: "TEST".into(),
            side,
            status: TradeStatus::Closed,
            opened_at: Utc.with_ymd_and_hms(2026, 1, 1, 9, 30, 0).unwrap(),
            closed_at: Some(Utc.with_ymd_and_hms(2026, 1, 1, 15, 30, 0).unwrap()),
            qty,
            entry_avg: entry,
            exit_avg: Some(entry),
            gross_pnl: None,
            fees: Decimal::ZERO,
            commissions: Decimal::ZERO,
            net_pnl: None,
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

    fn bar(open: i64, h: i64, l: i64, c: i64, mins_after_open: i64) -> PriceBar {
        PriceBar {
            symbol: "TEST".into(),
            interval: crate::models::BarInterval::M5,
            bar_time: Utc.with_ymd_and_hms(2026, 1, 1, 9, 30, 0).unwrap()
                + Duration::minutes(mins_after_open),
            open: Decimal::from(open),
            high: Decimal::from(h),
            low: Decimal::from(l),
            close: Decimal::from(c),
            volume: Decimal::ZERO,
            source: "test".into(),
        }
    }

    // ===========================================================================
    // No bars / no-op cases
    // ===========================================================================

    #[test]
    fn empty_bars_yields_zero_excursion() {
        let t = base_trade(TradeSide::Long, Decimal::from(100), Decimal::from(10));
        let ex = compute_excursion(&t, &[]);
        assert_eq!(ex.mfe, Decimal::ZERO);
        assert_eq!(ex.mae, Decimal::ZERO);
        // Best price stays at entry (no favorable bar seen), so best-exit P&L is
        // just -fees. fees=0 here, so best_exit_pnl == 0.
        assert_eq!(ex.best_exit_pnl, Decimal::ZERO);
    }

    #[test]
    fn bars_outside_window_are_skipped() {
        let t = base_trade(TradeSide::Long, Decimal::from(100), Decimal::from(10));
        // Bar at -60 minutes (before opened_at) — should not contribute.
        let before = PriceBar {
            bar_time: t.opened_at - Duration::minutes(60),
            ..bar(100, 200, 50, 100, 0)
        };
        let ex = compute_excursion(&t, &[before]);
        assert_eq!(ex.mfe, Decimal::ZERO);
        assert_eq!(ex.mae, Decimal::ZERO);
    }

    // ===========================================================================
    // Long-side MFE/MAE
    // ===========================================================================

    #[test]
    fn long_mfe_takes_highest_high_above_entry() {
        // entry = 100, 10 sh. Bars: high=110, high=120, high=115.
        // MFE should be (120-100)*10 = $200.
        let t = base_trade(TradeSide::Long, Decimal::from(100), Decimal::from(10));
        let bars = vec![
            bar(100, 110, 100, 105, 30),
            bar(105, 120, 100, 110, 60),
            bar(110, 115, 105, 108, 90),
        ];
        let ex = compute_excursion(&t, &bars);
        assert_eq!(ex.mfe, Decimal::from(200));
    }

    #[test]
    fn long_mae_takes_lowest_low_below_entry() {
        // entry = 100, 10 sh. Lows: 95, 90, 98. MAE = (90-100)*10 = -100.
        let t = base_trade(TradeSide::Long, Decimal::from(100), Decimal::from(10));
        let bars = vec![
            bar(100, 105, 95, 100, 30),
            bar(100, 102, 90, 95, 60),
            bar(95, 100, 98, 99, 90),
        ];
        let ex = compute_excursion(&t, &bars);
        assert_eq!(ex.mae, Decimal::from(-100));
    }

    #[test]
    fn long_mfe_zero_when_price_never_rises_above_entry() {
        // All highs ≤ entry: no favorable excursion. MFE = 0.
        let t = base_trade(TradeSide::Long, Decimal::from(100), Decimal::from(10));
        let bars = vec![bar(100, 100, 90, 95, 30), bar(95, 99, 85, 88, 60)];
        let ex = compute_excursion(&t, &bars);
        assert_eq!(ex.mfe, Decimal::ZERO);
        assert_eq!(ex.mae, Decimal::from(-150)); // (85-100)*10
    }

    // ===========================================================================
    // Short-side flips favorable/adverse
    // ===========================================================================

    #[test]
    fn short_mfe_takes_lowest_low_below_entry() {
        // Short entry = 100, 10 sh. Lows: 95, 90, 98.
        // Favorable for short = low. MFE = (95-100)*10*-1 (sign for short) = 50.
        // Actually: gross_pnl(short, qty, exit=90) = (90-100)*10*(-1) = 100.
        let t = base_trade(TradeSide::Short, Decimal::from(100), Decimal::from(10));
        let bars = vec![
            bar(100, 105, 95, 100, 30),
            bar(100, 102, 90, 95, 60),
            bar(95, 100, 98, 99, 90),
        ];
        let ex = compute_excursion(&t, &bars);
        assert_eq!(ex.mfe, Decimal::from(100));
    }

    #[test]
    fn short_mae_takes_highest_high_above_entry() {
        // Short entry = 100, 10 sh. Highs: 110, 120.
        // Adverse for short = high. MAE = (120-100)*10*-1 = -200.
        let t = base_trade(TradeSide::Short, Decimal::from(100), Decimal::from(10));
        let bars = vec![bar(100, 110, 100, 105, 30), bar(105, 120, 100, 110, 60)];
        let ex = compute_excursion(&t, &bars);
        assert_eq!(ex.mae, Decimal::from(-200));
    }

    // ===========================================================================
    // Best-exit P&L incorporates fees
    // ===========================================================================

    #[test]
    fn best_exit_pnl_subtracts_fees() {
        // Long: best high = 120. Best-exit gross = 200. Fees = 7. Net = 193.
        let mut t = base_trade(TradeSide::Long, Decimal::from(100), Decimal::from(10));
        t.fees = Decimal::from(7);
        let bars = vec![bar(100, 110, 100, 105, 30), bar(105, 120, 100, 110, 60)];
        let ex = compute_excursion(&t, &bars);
        assert_eq!(ex.best_exit_pnl, Decimal::from(193));
    }

    #[test]
    fn best_exit_pnl_negative_when_only_fees_with_no_favorable_move() {
        // No favorable move (all highs <= entry). best_price stays at entry,
        // so gross_pnl = 0, but fees = 5, net = -5.
        let mut t = base_trade(TradeSide::Long, Decimal::from(100), Decimal::from(10));
        t.fees = Decimal::from(5);
        let bars = vec![bar(100, 100, 90, 95, 30)];
        let ex = compute_excursion(&t, &bars);
        assert_eq!(ex.best_exit_pnl, Decimal::from(-5));
    }

    // ===========================================================================
    // Open trade (closed_at = None)
    // ===========================================================================

    #[test]
    fn open_trade_uses_now_as_window_end_and_still_aggregates() {
        // closed_at = None must not skip bars.
        let mut t = base_trade(TradeSide::Long, Decimal::from(100), Decimal::from(10));
        t.closed_at = None;
        t.status = TradeStatus::Open;
        let bars = vec![bar(100, 115, 95, 110, 30)];
        let ex = compute_excursion(&t, &bars);
        // Bar at 09:30 + 30m is in 2026; "now" at test runtime is past 2026-01-01
        // by construction (window end = Utc::now()), so the bar is included.
        assert_eq!(ex.mfe, Decimal::from(150)); // (115-100)*10
        assert_eq!(ex.mae, Decimal::from(-50)); // (95-100)*10
    }
}
