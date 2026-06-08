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

pub mod momentum;
pub mod types;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyKind {
    Momentum,
    MeanReversion,
    Orb,
    DonchianTrend,
    BbSqueeze,
}

impl StrategyKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Momentum => "momentum",
            Self::MeanReversion => "mean_reversion",
            Self::Orb => "orb",
            Self::DonchianTrend => "donchian_trend",
            Self::BbSqueeze => "bb_squeeze",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Momentum,
            Self::MeanReversion,
            Self::Orb,
            Self::DonchianTrend,
            Self::BbSqueeze,
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
        // Slots populated in commits 6–9.
        "mean_reversion" | "orb" | "donchian_trend" | "bb_squeeze" => {
            Err(FactoryError::NotImplemented(kind.to_string()))
        }
        other => Err(FactoryError::Unknown(other.to_string())),
    }
}
