//! MFE / MAE / best-exit P&L from price bars.
//!
//! Given a trade's open window `[opened_at, closed_at]` and the relevant
//! price bars at the finest available interval, compute:
//!   - **MFE** (max favorable excursion) — the best $ unrealized profit reached
//!   - **MAE** (max adverse excursion) — the worst $ unrealized drawdown
//!   - **best_exit_pnl** — what net P&L would have been if exited at the favorable extreme

use crate::models::{AssetClass, PriceBar, Trade, TradeSide};
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

// Avoid unused-AssetClass warning for the future case where this module
// uses asset-specific code.
#[allow(dead_code)]
fn _force_use(_: AssetClass) {}
