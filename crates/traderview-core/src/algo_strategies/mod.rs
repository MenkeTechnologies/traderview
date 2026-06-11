//! Algo strategy family — `Strategy` trait + per-kind impls + factory.
//!
//! Adding a new strategy:
//!   1. Create `algo_strategies/<name>.rs` defining a struct that
//!      implements `Strategy`.
//!   2. Add a variant to [`StrategyKind`].
//!   3. Register it in [`from_kind`].
//!
//! The engine in `traderview-db::algo_engine` builds a `Box<dyn Strategy>`
//! once per bar window via [`from_kind`], so swapping strategies is a
//! single column update in `algo_strategies.strategy_type`.

use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

pub mod bb_squeeze;
pub mod connors_rsi2;
pub mod donchian_trend;
pub mod heikin_ashi_trend;
pub mod ichimoku_cloud;
pub mod keltner_breakout;
pub mod ma_cross_adx;
pub mod macd_cross;
pub mod mean_reversion;
pub mod momentum;
pub mod orb;
pub mod order_block_sweep;
pub mod pairs;
pub mod pead;
pub mod supertrend;
pub mod ttm_squeeze;
pub mod types;
pub mod vwap_scalp;

pub use types::*;

/// Every algo strategy implements this. Pure-function evaluation — no I/O.
pub trait Strategy: Send + Sync {
    fn kind(&self) -> StrategyKind;
    /// Minimum bar count the rule stack needs before it can produce a signal.
    /// Engine no-ops shorter windows.
    fn min_bars(&self) -> usize;
    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal>;
    fn evaluate_exit(
        &self,
        bars: &[PriceBar],
        side: Side,
        anchor_high: f64,
        anchor_low: f64,
    ) -> Option<ExitSignal>;

    /// Multi-symbol strategies (pairs, stat-arb, sector rotation) return
    /// `Some(vec)` listing every symbol they need at evaluation time. The
    /// runner fetches bars for ALL of them and calls
    /// `evaluate_entry_multi`. Default: `None` — single-symbol strategy.
    fn required_symbols(&self) -> Option<Vec<String>> {
        None
    }

    /// Multi-symbol entry evaluator. Receives a map from symbol → bars
    /// for every symbol declared by `required_symbols`. Default impl
    /// returns None; only strategies that override `required_symbols`
    /// need to implement this.
    fn evaluate_entry_multi(
        &self,
        _bars_by_symbol: &std::collections::HashMap<String, Vec<PriceBar>>,
        _side_mode: SideMode,
    ) -> Option<EntrySignal> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyKind {
    Momentum,
    MeanReversion,
    Orb,
    DonchianTrend,
    BbSqueeze,
    TtmSqueeze,
    VwapScalp,
    Supertrend,
    HeikinAshiTrend,
    ConnorsRsi2,
    OrderBlockSweep,
    Pead,
    Pairs,
    MaCrossAdx,
    MacdCross,
    KeltnerBreakout,
    IchimokuCloud,
}

impl StrategyKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Momentum => "momentum",
            Self::MeanReversion => "mean_reversion",
            Self::Orb => "orb",
            Self::DonchianTrend => "donchian_trend",
            Self::BbSqueeze => "bb_squeeze",
            Self::TtmSqueeze => "ttm_squeeze",
            Self::VwapScalp => "vwap_scalp",
            Self::Supertrend => "supertrend",
            Self::HeikinAshiTrend => "heikin_ashi_trend",
            Self::ConnorsRsi2 => "connors_rsi2",
            Self::OrderBlockSweep => "order_block_sweep",
            Self::Pead => "pead",
            Self::Pairs => "pairs",
            Self::MaCrossAdx => "ma_cross_adx",
            Self::MacdCross => "macd_cross",
            Self::KeltnerBreakout => "keltner_breakout",
            Self::IchimokuCloud => "ichimoku_cloud",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Momentum,
            Self::MeanReversion,
            Self::Orb,
            Self::DonchianTrend,
            Self::BbSqueeze,
            Self::TtmSqueeze,
            Self::VwapScalp,
            Self::Supertrend,
            Self::HeikinAshiTrend,
            Self::ConnorsRsi2,
            Self::OrderBlockSweep,
            Self::Pead,
            Self::Pairs,
            Self::MaCrossAdx,
            Self::MacdCross,
            Self::KeltnerBreakout,
            Self::IchimokuCloud,
        ]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FactoryError {
    #[error("strategy_type {0} is not yet implemented in this build")]
    NotImplemented(String),
    #[error("unknown strategy_type: {0}")]
    Unknown(String),
}

/// Build a strategy from its DB row's `strategy_type` + `entry_rules` JSON.
/// Returns `Err` only for an unknown kind — invalid rule JSON falls back
/// to that strategy's defaults so a partially-populated config still runs.
pub fn from_kind(
    kind: &str,
    entry_rules: &serde_json::Value,
) -> Result<Box<dyn Strategy>, FactoryError> {
    match kind {
        "momentum" => Ok(Box::new(momentum::Momentum::from_json(entry_rules))),
        "mean_reversion" => Ok(Box::new(mean_reversion::MeanReversion::from_json(
            entry_rules,
        ))),
        "orb" => Ok(Box::new(orb::Orb::from_json(entry_rules))),
        "donchian_trend" => Ok(Box::new(donchian_trend::DonchianTrend::from_json(
            entry_rules,
        ))),
        "bb_squeeze" => Ok(Box::new(bb_squeeze::BbSqueeze::from_json(entry_rules))),
        "ttm_squeeze" => Ok(Box::new(ttm_squeeze::TtmSqueeze::from_json(entry_rules))),
        "vwap_scalp" => Ok(Box::new(vwap_scalp::VwapScalp::from_json(entry_rules))),
        "supertrend" => Ok(Box::new(supertrend::Supertrend::from_json(entry_rules))),
        "heikin_ashi_trend" => Ok(Box::new(heikin_ashi_trend::HeikinAshiTrend::from_json(
            entry_rules,
        ))),
        "connors_rsi2" => Ok(Box::new(connors_rsi2::ConnorsRsi2::from_json(entry_rules))),
        "order_block_sweep" => Ok(Box::new(order_block_sweep::OrderBlockSweep::from_json(
            entry_rules,
        ))),
        "pead" => Ok(Box::new(pead::Pead::from_json(entry_rules))),
        "pairs" => Ok(Box::new(pairs::Pairs::from_json(entry_rules))),
        "ma_cross_adx" => Ok(Box::new(ma_cross_adx::MaCrossAdx::from_json(entry_rules))),
        "macd_cross" => Ok(Box::new(macd_cross::MacdCross::from_json(entry_rules))),
        "keltner_breakout" => Ok(Box::new(keltner_breakout::KeltnerBreakout::from_json(
            entry_rules,
        ))),
        "ichimoku_cloud" => Ok(Box::new(ichimoku_cloud::IchimokuCloud::from_json(
            entry_rules,
        ))),
        other => Err(FactoryError::Unknown(other.to_string())),
    }
}
