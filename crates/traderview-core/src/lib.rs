//! `traderview-core` — domain types, FIFO roll-up, per-asset P&L, statistics,
//! risk + R-multiple, MFE/MAE excursion, liquidity, slug helpers.
//!
//! Pure-Rust, no I/O. Consumed by `traderview-db` (persistence), `traderview-web`
//! (HTTP), `traderview-import` (broker parsers), and `traderview-desktop`.

pub mod backtest;
pub mod bracket_order;
pub mod buying_power;
pub mod correlation;
pub mod discipline_score;
pub mod dow_hour_heatmap;
pub mod equity_forecast;
pub mod excursion;
pub mod greeks;
pub mod indicators;
pub mod iv_backtest;
pub mod liquidity;
pub mod models;
pub mod pnl;
pub mod position_size;
pub mod rebalance;
pub mod reconcile_1099b;
pub mod risk;
pub mod risk_gate;
pub mod risk_reward;
pub mod rollup;
pub mod scan;
pub mod sentiment;
pub mod setup_catalog;
pub mod signals;
pub mod slug;
pub mod strategy_alert;
pub mod stats;

pub use models::*;
