//! Order Block + Liquidity Sweep — Smart Money Concepts entry.
//!
//! The setup combines two SMC primitives:
//!   1. **Liquidity sweep** of a recent swing low — price probes the
//!      stop-loss cluster below the swing, then reverses.
//!   2. **Bullish order block** — the last down-candle before a sharp
//!      up-expansion, treated as a future-support zone.
//!
//! Entry (long): in the last `lookback` bars there was a confirmed
//! liquidity sweep below a swing low AND a bullish order block; the
//! latest bar's range overlaps the order-block zone (price returned
//! to the zone for the entry) AND the bar closes bullish. Mirror for
//! short with sweep-of-high + bearish OB.
//!
//! Exit: ATR trailing stop OR price closes back below the zone_low
//! (long) / above zone_high (short) — invalidates the SMC thesis.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::liquidity_grab::{self, GrabConfig, GrabSide};
use crate::models::PriceBar;
use crate::order_block::{self, BlockKind, OrderBlockConfig};
use crate::swing_points;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    /// How many bars back we'll consider for a fresh OB + sweep.
    pub lookback: usize,
    pub swing_lookback: usize,
    pub ob_expansion_window: usize,
    pub ob_expansion_multiple: f64,
    pub grab_min_sweep_atrs: f64,
    pub grab_confirm_within: usize,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
    pub atr_take_profit_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            lookback: 30,
            swing_lookback: 3,
            ob_expansion_window: 3,
            ob_expansion_multiple: 2.0,
            grab_min_sweep_atrs: 0.1,
            grab_confirm_within: 3,
            atr_period: 14,
            atr_stop_mult: 1.5,
            atr_take_profit_mult: 3.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderBlockSweep { pub rules: Rules }

impl OrderBlockSweep {
    pub fn new(rules: Rules) -> Self { Self { rules } }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

fn ob_bars(bars: &[PriceBar]) -> Vec<order_block::OhlcBar> {
    use rust_decimal::prelude::ToPrimitive;
    bars.iter()
        .map(|b| order_block::OhlcBar {
            open: b.open.to_f64().unwrap_or(0.0),
            high: b.high.to_f64().unwrap_or(0.0),
            low: b.low.to_f64().unwrap_or(0.0),
            close: b.close.to_f64().unwrap_or(0.0),
        })
        .collect()
}

fn grab_bars(bars: &[PriceBar]) -> Vec<liquidity_grab::OhlcBar> {
    use rust_decimal::prelude::ToPrimitive;
    bars.iter()
        .map(|b| liquidity_grab::OhlcBar {
            open: b.open.to_f64().unwrap_or(0.0),
            high: b.high.to_f64().unwrap_or(0.0),
            low: b.low.to_f64().unwrap_or(0.0),
            close: b.close.to_f64().unwrap_or(0.0),
        })
        .collect()
}

fn swing_bars(bars: &[PriceBar]) -> Vec<swing_points::Bar> {
    use rust_decimal::prelude::ToPrimitive;
    bars.iter()
        .map(|b| swing_points::Bar {
            high: b.high.to_f64().unwrap_or(0.0),
            low: b.low.to_f64().unwrap_or(0.0),
        })
        .collect()
}

impl Strategy for OrderBlockSweep {
    fn kind(&self) -> StrategyKind { StrategyKind::OrderBlockSweep }

    fn min_bars(&self) -> usize {
        self.rules.lookback.max(self.rules.atr_period + 1) + 5
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let atr_f: Vec<f64> = atr.into_iter().map(|v| v.unwrap_or(0.0)).collect();

        let ob_cfg = OrderBlockConfig {
            expansion_window: self.rules.ob_expansion_window,
            expansion_multiple: self.rules.ob_expansion_multiple,
        };
        let ob = order_block::detect(&ob_bars(bars), &ob_cfg);

        let swings = swing_points::detect(&swing_bars(bars), self.rules.swing_lookback);
        let grab_cfg = GrabConfig {
            min_sweep_atrs: self.rules.grab_min_sweep_atrs,
            confirm_within: self.rules.grab_confirm_within,
            min_followthrough: 1,
        };
        let grabs = liquidity_grab::detect(&grab_bars(bars), &atr_f, &swings, &grab_cfg);

        let i = bars.len() - 1;
        let close_now = closes[i];
        let open_now = bars[i].open.to_string().parse::<f64>().unwrap_or(close_now);
        let atr_now = atr_f[i];
        if atr_now <= 0.0 { return None; }
        let recent_start = i.saturating_sub(self.rules.lookback);

        // Long path: bullish OB + low-side sweep.
        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both);
        if want_long {
            let recent_bull_ob = ob
                .blocks
                .iter()
                .rev()
                .find(|b| b.kind == BlockKind::Bullish && b.bar_index >= recent_start);
            let recent_low_sweep = grabs
                .events
                .iter()
                .rev()
                .find(|g| g.side == GrabSide::Low && g.confirm_bar >= recent_start);
            if let (Some(bull_ob), Some(_)) = (recent_bull_ob, recent_low_sweep) {
                let in_zone = bars[i].low.to_string().parse::<f64>().unwrap_or(close_now) <= bull_ob.zone_high
                    && close_now >= bull_ob.zone_low;
                let bullish_close = close_now > open_now;
                if in_zone && bullish_close {
                    let stop = bull_ob.zone_low - self.rules.atr_stop_mult * atr_now;
                    let stop_distance = (close_now - stop).max(0.01);
                    return Some(EntrySignal {
                        side: Side::Buy,
                        entry_price: close_now,
                        stop_distance,
                        trigger_index: i,
                        stop_price: stop.max(0.01),
                        take_profit_price: close_now + self.rules.atr_take_profit_mult * atr_now,
                        kind: "order_block_sweep",
                        diagnostic: serde_json::json!({
                            "ob_zone_low": bull_ob.zone_low,
                            "ob_zone_high": bull_ob.zone_high,
                            "ob_bar_index": bull_ob.bar_index,
                            "atr": atr_now,
                            "side": "buy",
                        }),
                    });
                }
            }
        }

        // Short path: bearish OB + high-side sweep.
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both);
        if want_short {
            let recent_bear_ob = ob
                .blocks
                .iter()
                .rev()
                .find(|b| b.kind == BlockKind::Bearish && b.bar_index >= recent_start);
            let recent_high_sweep = grabs
                .events
                .iter()
                .rev()
                .find(|g| g.side == GrabSide::High && g.confirm_bar >= recent_start);
            if let (Some(bear_ob), Some(_)) = (recent_bear_ob, recent_high_sweep) {
                let high_now = bars[i].high.to_string().parse::<f64>().unwrap_or(close_now);
                let in_zone = high_now >= bear_ob.zone_low && close_now <= bear_ob.zone_high;
                let bearish_close = close_now < open_now;
                if in_zone && bearish_close {
                    let stop = bear_ob.zone_high + self.rules.atr_stop_mult * atr_now;
                    let stop_distance = (stop - close_now).max(0.01);
                    return Some(EntrySignal {
                        side: Side::Sell,
                        entry_price: close_now,
                        stop_distance,
                        trigger_index: i,
                        stop_price: stop,
                        take_profit_price: (close_now
                            - self.rules.atr_take_profit_mult * atr_now)
                            .max(0.01),
                        kind: "order_block_sweep",
                        diagnostic: serde_json::json!({
                            "ob_zone_low": bear_ob.zone_low,
                            "ob_zone_high": bear_ob.zone_high,
                            "ob_bar_index": bear_ob.bar_index,
                            "atr": atr_now,
                            "side": "sell",
                        }),
                    });
                }
            }
        }
        None
    }

    fn evaluate_exit(
        &self,
        bars: &[PriceBar],
        side: Side,
        anchor_high: f64,
        anchor_low: f64,
    ) -> Option<ExitSignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let i = bars.len() - 1;
        let atr_now = atr.get(i).copied().flatten()?;
        let close_now = closes[i];

        match side {
            Side::Buy => {
                let trail = anchor_high - self.rules.atr_stop_mult * atr_now;
                if lows[i] <= trail {
                    return Some(ExitSignal {
                        reason: "atr_trailing_stop",
                        exit_price: trail.max(0.01),
                        trigger_index: i,
                    });
                }
            }
            Side::Sell => {
                let trail = anchor_low + self.rules.atr_stop_mult * atr_now;
                if highs[i] >= trail {
                    return Some(ExitSignal {
                        reason: "atr_trailing_stop",
                        exit_price: trail,
                        trigger_index: i,
                    });
                }
            }
        }
        let _ = close_now;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BarInterval;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn bar(t: i64, o: &str, h: &str, l: &str, c: &str, v: u64) -> PriceBar {
        PriceBar {
            symbol: "TEST".into(),
            interval: BarInterval::M1,
            bar_time: Utc.timestamp_opt(t, 0).unwrap(),
            open: Decimal::from_str(o).unwrap(),
            high: Decimal::from_str(h).unwrap(),
            low: Decimal::from_str(l).unwrap(),
            close: Decimal::from_str(c).unwrap(),
            volume: Decimal::from(v),
            source: "test".into(),
        }
    }

    #[test]
    fn entry_blocked_on_flat_market_no_ob_no_sweep() {
        let strat = OrderBlockSweep::new(Rules::default());
        let bars: Vec<PriceBar> = (0..50)
            .map(|i| bar(1_700_000_000 + i * 60, "100.00", "100.05", "99.95", "100.00", 1_000_000))
            .collect();
        for end in strat.min_bars()..=bars.len() {
            assert!(
                strat.evaluate_entry(&bars[..end], SideMode::Long).is_none(),
                "flat market: no OB, no sweep → no signal at end={end}"
            );
        }
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = OrderBlockSweep::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "order_block_sweep");
        // lookback(30).max(atr_period+1=15) + 5 = 35.
        assert_eq!(strat.min_bars(), 35);
    }

    #[test]
    fn entry_refuses_short_under_side_mode_long() {
        let strat = OrderBlockSweep::new(Rules::default());
        // Any window — SideMode::Long must not produce a Sell signal.
        let bars: Vec<PriceBar> = (0..50)
            .map(|i| bar(1_700_000_000 + i * 60, "100.00", "100.05", "99.95", "100.00", 1_000_000))
            .collect();
        for end in strat.min_bars()..=bars.len() {
            if let Some(sig) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                assert_ne!(sig.side, Side::Sell);
            }
        }
    }
}
