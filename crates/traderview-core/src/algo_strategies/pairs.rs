//! Pairs Trading — relative-value mean reversion between two
//! cointegrated symbols.
//!
//! Treats `spread = ln(price_a) − hedge_ratio · ln(price_b)` as a
//! synthetic mean-reverting instrument. When the spread's rolling
//! z-score deviates beyond `z_entry`, the relative value is mispriced
//! and the strategy bets on reversion.
//!
//! Scope note: a true pairs trade is a SIMULTANEOUS dual-leg position
//! (long the underperformer, short the outperformer). The current
//! engine submits one order per signal, so this strategy fires an order
//! on the UNDERPERFORMING leg only — the short side of the spread is
//! implicit. For full dollar-neutral pairs, run two coupled strategies
//! (one long-only on leg A, one short-only on leg B) tied to a shared
//! account. Standalone use is still a viable directional bet on the
//! underperformer recovering.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub symbol_a: String,
    pub symbol_b: String,
    /// Hedge ratio used to build the spread. Set to 1.0 if you haven't
    /// computed cointegration explicitly — z-score reversion still
    /// works on a 1:1 spread for closely-correlated pairs.
    pub hedge_ratio: f64,
    pub lookback: usize,
    pub z_entry: f64,
    pub z_exit: f64,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            symbol_a: "PEP".into(),
            symbol_b: "KO".into(),
            hedge_ratio: 1.0,
            lookback: 60,
            z_entry: 2.0,
            z_exit: 0.5,
            atr_period: 14,
            atr_stop_mult: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pairs { pub rules: Rules }

impl Pairs {
    pub fn new(rules: Rules) -> Self { Self { rules } }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

fn ln_closes(bars: &[PriceBar]) -> Vec<f64> {
    use rust_decimal::prelude::ToPrimitive;
    bars.iter()
        .map(|b| {
            let p = b.close.to_f64().unwrap_or(0.0);
            if p > 0.0 { p.ln() } else { 0.0 }
        })
        .collect()
}

fn rolling_z(spread: &[f64], lookback: usize) -> Option<f64> {
    if spread.len() < lookback + 1 { return None; }
    let i = spread.len() - 1;
    let window = &spread[i + 1 - lookback..i + 1];
    let mean = window.iter().sum::<f64>() / lookback as f64;
    let var = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / lookback as f64;
    let sd = var.sqrt();
    if sd <= 0.0 { return None; }
    Some((spread[i] - mean) / sd)
}

impl Strategy for Pairs {
    fn kind(&self) -> StrategyKind { StrategyKind::Pairs }

    fn min_bars(&self) -> usize {
        self.rules.lookback.max(self.rules.atr_period + 1) + 2
    }

    fn required_symbols(&self) -> Option<Vec<String>> {
        Some(vec![self.rules.symbol_a.clone(), self.rules.symbol_b.clone()])
    }

    fn evaluate_entry(&self, _bars: &[PriceBar], _side_mode: SideMode) -> Option<EntrySignal> {
        // Pairs is multi-symbol — runner must use evaluate_entry_multi.
        None
    }

    fn evaluate_entry_multi(
        &self,
        bars_by_symbol: &std::collections::HashMap<String, Vec<PriceBar>>,
        side_mode: SideMode,
    ) -> Option<EntrySignal> {
        let bars_a = bars_by_symbol.get(&self.rules.symbol_a)?;
        let bars_b = bars_by_symbol.get(&self.rules.symbol_b)?;
        let n = bars_a.len().min(bars_b.len());
        if n < self.min_bars() { return None; }
        let la = &ln_closes(&bars_a[bars_a.len() - n..]);
        let lb = &ln_closes(&bars_b[bars_b.len() - n..]);
        let spread: Vec<f64> =
            la.iter().zip(lb.iter()).map(|(a, b)| a - self.rules.hedge_ratio * b).collect();

        let z = rolling_z(&spread, self.rules.lookback)?;
        let closes_a = indicators::closes(&bars_a[bars_a.len() - n..]);
        let highs_a = indicators::highs(&bars_a[bars_a.len() - n..]);
        let lows_a = indicators::lows(&bars_a[bars_a.len() - n..]);
        let atr = indicators::atr(&highs_a, &lows_a, &closes_a, self.rules.atr_period);
        let i = n - 1;
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 { return None; }
        let close_a = closes_a[i];

        // z < -z_entry → leg A is UNDERPERFORMING vs leg B → long A.
        // z > +z_entry → leg A is OVERPERFORMING → short A (or no-op if
        // SideMode::Long).
        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both)
            && z <= -self.rules.z_entry;
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
            && z >= self.rules.z_entry;

        if want_long {
            let stop = close_a - self.rules.atr_stop_mult * atr_now;
            let stop_distance = (close_a - stop).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_a,
                stop_distance,
                trigger_index: i,
                stop_price: stop.max(0.01),
                // Target = revert to z=0; rough proxy via ATR ceiling.
                take_profit_price: close_a + 2.0 * atr_now,
                kind: "pairs",
                diagnostic: serde_json::json!({
                    "symbol_a": self.rules.symbol_a,
                    "symbol_b": self.rules.symbol_b,
                    "spread_z": z,
                    "hedge_ratio": self.rules.hedge_ratio,
                    "atr": atr_now,
                }),
            })
        } else if want_short {
            let stop = close_a + self.rules.atr_stop_mult * atr_now;
            let stop_distance = (stop - close_a).max(0.01);
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_a,
                stop_distance,
                trigger_index: i,
                stop_price: stop,
                take_profit_price: (close_a - 2.0 * atr_now).max(0.01),
                kind: "pairs",
                diagnostic: serde_json::json!({
                    "symbol_a": self.rules.symbol_a,
                    "symbol_b": self.rules.symbol_b,
                    "spread_z": z,
                    "hedge_ratio": self.rules.hedge_ratio,
                    "atr": atr_now,
                }),
            })
        } else {
            None
        }
    }

    fn evaluate_exit(
        &self,
        _bars: &[PriceBar],
        _side: Side,
        _anchor_high: f64,
        _anchor_low: f64,
    ) -> Option<ExitSignal> {
        // Exit lives in the multi path — caller passes bars for the
        // executing leg here, but spread z-revert needs both legs. The
        // runner's exit-management for pairs lands as a follow-up; for
        // now the engine relies on the ATR trailing stop wired in
        // sizing (stop_price field).
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BarInterval;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::str::FromStr;

    fn bar(t: i64, sym: &str, c: &str) -> PriceBar {
        let cd = Decimal::from_str(c).unwrap();
        PriceBar {
            symbol: sym.into(),
            interval: BarInterval::M1,
            bar_time: Utc.timestamp_opt(t, 0).unwrap(),
            open: cd,
            high: cd + Decimal::from_str("0.1").unwrap(),
            low: cd - Decimal::from_str("0.1").unwrap(),
            close: cd,
            volume: Decimal::from(1_000_000),
            source: "test".into(),
        }
    }

    /// Build a 100-bar window where ln(A) − ln(B) hovers around a mean,
    /// then on the LAST bar the spread blows out 4σ below the mean
    /// (A underperformed). The strategy should fire a Buy on A.
    fn diverging_pair_window() -> (Vec<PriceBar>, Vec<PriceBar>) {
        let mut a = Vec::new();
        let mut b = Vec::new();
        let mut t = 1_700_000_000_i64;
        // 100 bars: both prices drift together with small noise.
        for i in 0..100 {
            let p = 100.0 + (i as f64 * 0.4).sin() * 0.4;
            a.push(bar(t, "A", &format!("{p:.3}")));
            b.push(bar(t, "B", &format!("{p:.3}")));
            t += 60;
        }
        // Last bar — A drops hard while B holds. Spread becomes deeply
        // negative (A underperformed) — should fire Buy on A.
        a.push(bar(t, "A", "95.000"));
        b.push(bar(t, "B", "100.000"));
        (a, b)
    }

    #[test]
    fn entry_fires_long_a_on_negative_spread_z() {
        let strat = Pairs::new(Rules {
            symbol_a: "A".into(),
            symbol_b: "B".into(),
            ..Default::default()
        });
        let (a, b) = diverging_pair_window();
        let mut map = HashMap::new();
        map.insert("A".into(), a);
        map.insert("B".into(), b);
        let sig = strat
            .evaluate_entry_multi(&map, SideMode::Long)
            .expect("4σ negative spread → long A");
        assert_eq!(sig.side, Side::Buy);
        let z = sig.diagnostic.get("spread_z").and_then(|v| v.as_f64()).unwrap();
        assert!(z <= -2.0, "spread_z {z} should be <= -2");
    }

    #[test]
    fn entry_blocked_when_spread_within_band() {
        let strat = Pairs::new(Rules {
            symbol_a: "A".into(),
            symbol_b: "B".into(),
            ..Default::default()
        });
        let mut a = Vec::new();
        let mut b = Vec::new();
        let mut t = 1_700_000_000_i64;
        for i in 0..100 {
            let p = 100.0 + (i as f64 * 0.4).sin() * 0.4;
            a.push(bar(t, "A", &format!("{p:.3}")));
            b.push(bar(t, "B", &format!("{p:.3}")));
            t += 60;
        }
        let mut map = HashMap::new();
        map.insert("A".into(), a);
        map.insert("B".into(), b);
        assert!(strat.evaluate_entry_multi(&map, SideMode::Long).is_none());
    }

    #[test]
    fn required_symbols_reports_both_legs() {
        let strat = Pairs::new(Rules {
            symbol_a: "PEP".into(),
            symbol_b: "KO".into(),
            ..Default::default()
        });
        let syms = strat.required_symbols().unwrap();
        assert!(syms.contains(&"PEP".to_string()));
        assert!(syms.contains(&"KO".to_string()));
    }

    #[test]
    fn single_symbol_evaluate_entry_always_returns_none() {
        // Pairs is multi-only; single-symbol path is a guard.
        let strat = Pairs::new(Rules::default());
        let bars: Vec<PriceBar> = (0..150)
            .map(|i| bar(1_700_000_000 + i * 60, "A", "100.00"))
            .collect();
        assert!(strat.evaluate_entry(&bars, SideMode::Long).is_none());
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = Pairs::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "pairs");
        // lookback (60) wins → +2 = 62.
        assert_eq!(strat.min_bars(), 62);
    }
}
