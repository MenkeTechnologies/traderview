//! `traderview-core` — domain types, FIFO roll-up, per-asset P&L, statistics,
//! risk + R-multiple, MFE/MAE excursion, liquidity, slug helpers.
//!
//! Pure-Rust, no I/O. Consumed by `traderview-db` (persistence), `traderview-web`
//! (HTTP), `traderview-import` (broker parsers), and `traderview-desktop`.

pub mod backtest;
pub mod correlation;
pub mod excursion;
pub mod greeks;
pub mod indicators;
pub mod iv_backtest;
pub mod liquidity;
pub mod models;
pub mod pnl;
pub mod position_size;
pub mod risk;
pub mod rollup;
pub mod scan;
pub mod sentiment;
pub mod signals;
pub mod slug;
pub mod strategy_alert;
pub mod stats;

pub use models::*;
