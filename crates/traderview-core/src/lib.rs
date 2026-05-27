//! `traderview-core` — domain types, FIFO roll-up, per-asset P&L, statistics,
//! risk + R-multiple, MFE/MAE excursion, liquidity, slug helpers.
//!
//! Pure-Rust, no I/O. Consumed by `traderview-db` (persistence), `traderview-web`
//! (HTTP), `traderview-import` (broker parsers), and `traderview-desktop`.

pub mod backtest;
pub mod bracket_order;
pub mod buying_power;
pub mod commission_optimizer;
pub mod concentration;
pub mod correlation;
pub mod correlation_clusters;
pub mod discipline_score;
pub mod dow_hour_heatmap;
pub mod drawdown_throttle;
pub mod equity_forecast;
pub mod excursion;
pub mod exit_timing;
pub mod greeks;
pub mod halt_risk;
pub mod hedge_ratio;
pub mod high_water_mark;
pub mod indicators;
pub mod kelly;
pub mod margin_call;
pub mod iv_backtest;
pub mod liquidity;
pub mod mae_stop_tuning;
pub mod market_impact;
pub mod models;
pub mod monte_carlo;
pub mod optimal_f;
pub mod options_margin;
pub mod overtrading;
pub mod pnl;
pub mod position_size;
pub mod per_symbol_slippage;
pub mod position_aging;
pub mod pyramid;
pub mod rebalance;
pub mod reconcile_1099b;
pub mod risk;
pub mod risk_gate;
pub mod risk_metrics;
pub mod risk_reward;
pub mod rollup;
pub mod scan;
pub mod sector_exposure;
pub mod sentiment;
pub mod setup_catalog;
pub mod sharpe_by_window;
pub mod signals;
pub mod slug;
pub mod sortino;
pub mod stops;
pub mod strategy_alert;
pub mod strategy_correlation;
pub mod streaks;
pub mod trade_quality;
pub mod stats;
pub mod tax_loss_harvest;
pub mod tilt_detector;
pub mod twap;
pub mod vwap_slippage;
pub mod wash_sale;

pub use models::*;
