//! Momentum strategy — `Strategy` trait wrapper over the existing
//! `crate::momentum_strategy` free-function evaluator.
//!
//! The rule stack (EMA(9)/EMA(21) crossover + RSI(14) band + ROC(10) +
//! RVOL(20)) lives in `momentum_strategy.rs` to keep its inline tests
//! co-located with the math. This file is the thin adapter the engine
//! talks to through the `Strategy` trait.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::models::PriceBar;
use crate::momentum_strategy::{self, Rules};

#[derive(Debug, Clone)]
pub struct Momentum {
    pub rules: Rules,
}

impl Momentum {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }

    /// Decode the strategy's `entry_rules` JSONB column; missing fields
    /// fall back to `Rules::default()`.
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone())
            .unwrap_or_default();
        Self { rules }
    }
}

impl Strategy for Momentum {
    fn kind(&self) -> StrategyKind {
        StrategyKind::Momentum
    }

    fn min_bars(&self) -> usize {
        self.rules
            .ema_slow
            .max(self.rules.rsi_period + 1)
            .max(self.rules.roc_period + 1)
            .max(self.rules.rvol_lookback + 1)
            .max(self.rules.atr_period + 1)
            + 1
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        let sig = momentum_strategy::evaluate_entry(bars, &self.rules, side_mode)?;
        Some(EntrySignal {
            side: sig.side,
            entry_price: sig.entry_price,
            // Momentum sizes off ATR × stop multiplier; expose that as the
            // shared `stop_distance` so `size_shares` works uniformly.
            stop_distance: sig.atr * self.rules.atr_stop_mult,
            trigger_index: sig.trigger_index,
            stop_price: sig.stop_price,
            take_profit_price: sig.take_profit_price,
            kind: "momentum",
            diagnostic: serde_json::json!({
                "ema_fast": sig.diagnostic.ema_fast,
                "ema_slow": sig.diagnostic.ema_slow,
                "rsi":      sig.diagnostic.rsi,
                "roc":      sig.diagnostic.roc,
                "rvol":     sig.diagnostic.rvol,
            }),
        })
    }

    fn evaluate_exit(
        &self,
        bars: &[PriceBar],
        side: Side,
        anchor_high: f64,
        anchor_low: f64,
    ) -> Option<ExitSignal> {
        momentum_strategy::evaluate_exit(bars, side, anchor_high, anchor_low, &self.rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::momentum_strategy::Rules;

    #[test]
    fn from_json_falls_back_to_defaults_on_empty_object() {
        let m = Momentum::from_json(&serde_json::json!({}));
        assert_eq!(m.rules.ema_fast, Rules::default().ema_fast);
    }

    #[test]
    fn kind_label_stable() {
        let m = Momentum::new(Rules::default());
        assert_eq!(m.kind().as_str(), "momentum");
    }

    #[test]
    fn min_bars_respects_slowest_input() {
        // Default rules: ema_slow=21, rsi=14+1=15, roc=10+1=11, rvol=20+1=21, atr=14+1=15
        // max = 21, plus 1 = 22.
        let m = Momentum::new(Rules::default());
        assert_eq!(m.min_bars(), 22);
    }
}
