//! Ichimoku Kinkō Hyō — "one-glance equilibrium chart" (Goichi Hosoda, 1969).
//!
//! Five lines:
//!   Tenkan-sen (conversion): (highest high + lowest low)/2 over 9 bars
//!   Kijun-sen  (base):       (highest high + lowest low)/2 over 26 bars
//!   Senkou A  (leading A):   (Tenkan + Kijun)/2, plotted 26 bars ahead
//!   Senkou B  (leading B):   (highest high + lowest low)/2 over 52 bars,
//!                            plotted 26 bars ahead
//!   Chikou (lagging):        close, plotted 26 bars BACK
//!
//! Cloud = the area between Senkou A and Senkou B.
//!
//! Classic long entry confluence:
//!   1. close > current cloud (above both Senkou A & B at index i).
//!   2. Tenkan > Kijun (TK cross, bullish).
//!   3. Chikou clear of price 26 bars ago (no overhead resistance).
//!   4. Senkou A > Senkou B at i (cloud bullish — "green cloud").
//!
//! Mirror these for shorts.
//!
//! Stop:   Kijun-sen line — Ichimoku traders treat it as the trailing
//!         baseline (price below kijun = bullish thesis invalid).
//! Target: cloud-thickness multiple beyond entry.
//!
//! Exit: close crosses back through Kijun OR TK opposite cross.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub tenkan_period: usize,
    pub kijun_period: usize,
    pub senkou_b_period: usize,
    pub displacement: usize,
    /// Take-profit distance as a multiple of the cloud thickness
    /// (|Senkou A - Senkou B|) at entry. Default 2.0 lets a thin
    /// cloud target a small move and a thick cloud a large one.
    pub take_profit_cloud_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            displacement: 26,
            take_profit_cloud_mult: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IchimokuCloud {
    pub rules: Rules,
}

impl IchimokuCloud {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

fn donchian_mid(bars: &[PriceBar], period: usize) -> Vec<Option<f64>> {
    let highs = indicators::highs(bars);
    let lows = indicators::lows(bars);
    let mut out = Vec::with_capacity(bars.len());
    for i in 0..bars.len() {
        if i + 1 < period {
            out.push(None);
            continue;
        }
        let start = i + 1 - period;
        let mut hi = f64::MIN;
        let mut lo = f64::MAX;
        for j in start..=i {
            if highs[j] > hi {
                hi = highs[j];
            }
            if lows[j] < lo {
                lo = lows[j];
            }
        }
        out.push(Some((hi + lo) / 2.0));
    }
    out
}

#[allow(clippy::type_complexity)]
fn ichimoku(
    bars: &[PriceBar],
    rules: &Rules,
) -> (
    Vec<Option<f64>>, // tenkan
    Vec<Option<f64>>, // kijun
    Vec<Option<f64>>, // senkou_a (at index i = drawn for THIS bar; displaced from i-26)
    Vec<Option<f64>>, // senkou_b (same convention)
) {
    let tenkan = donchian_mid(bars, rules.tenkan_period);
    let kijun = donchian_mid(bars, rules.kijun_period);
    let senkou_b_raw = donchian_mid(bars, rules.senkou_b_period);

    // Senkou A/B are PLOTTED 26 bars forward; that means the value
    // currently rendered at index i was COMPUTED at index i-26. We
    // expose senkou_a[i] = (tenkan[i-disp] + kijun[i-disp]) / 2 so that
    // a caller asking "what's the cloud at bar i?" gets the value
    // visible on a TradingView Ichimoku at that bar.
    let mut senkou_a = vec![None; bars.len()];
    let mut senkou_b = vec![None; bars.len()];
    for i in 0..bars.len() {
        if i < rules.displacement {
            continue;
        }
        let src = i - rules.displacement;
        let t = tenkan.get(src).copied().flatten();
        let k = kijun.get(src).copied().flatten();
        if let (Some(t), Some(k)) = (t, k) {
            senkou_a[i] = Some((t + k) / 2.0);
        }
        if let Some(b) = senkou_b_raw.get(src).copied().flatten() {
            senkou_b[i] = Some(b);
        }
    }
    (tenkan, kijun, senkou_a, senkou_b)
}

impl Strategy for IchimokuCloud {
    fn kind(&self) -> StrategyKind {
        StrategyKind::IchimokuCloud
    }

    fn min_bars(&self) -> usize {
        self.rules.senkou_b_period + self.rules.displacement + 3
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let (tenkan, kijun, sa, sb) = ichimoku(bars, &self.rules);
        let i = bars.len() - 1;
        let prev = i - 1;
        let close_now = bars[i].close.to_f64().unwrap_or(0.0);
        let t_now = tenkan[i]?;
        let k_now = kijun[i]?;
        let t_prev = tenkan[prev]?;
        let k_prev = kijun[prev]?;
        let sa_now = sa[i]?;
        let sb_now = sb[i]?;

        let cloud_top = sa_now.max(sb_now);
        let cloud_bot = sa_now.min(sb_now);
        let cloud_thickness = (sa_now - sb_now).abs();

        // Chikou-clear check: the close at i-displacement, looking back
        // ANOTHER displacement bars (i.e. compare close[i-disp] vs price
        // at i-2*disp). Standard Ichimoku "Chikou clear" test.
        let lookback = i.checked_sub(self.rules.displacement)?;
        let further_back = lookback.checked_sub(self.rules.displacement)?;
        let chikou_close = bars[lookback].close.to_f64().unwrap_or(0.0);
        let prior_price = bars[further_back].close.to_f64().unwrap_or(0.0);

        let bullish_cloud = sa_now > sb_now;
        let bearish_cloud = sb_now > sa_now;

        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both)
            && close_now > cloud_top
            && t_prev <= k_prev && t_now > k_now  // TK bull cross
            && chikou_close > prior_price
            && bullish_cloud;
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
            && close_now < cloud_bot
            && t_prev >= k_prev && t_now < k_now  // TK bear cross
            && chikou_close < prior_price
            && bearish_cloud;

        if want_long {
            let stop = k_now
                .min(close_now - 0.5 * cloud_thickness.max(0.01))
                .max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_now,
                stop_distance: (close_now - stop).max(0.01),
                trigger_index: i,
                stop_price: stop,
                take_profit_price: close_now
                    + self.rules.take_profit_cloud_mult * cloud_thickness.max(0.01),
                kind: "ichimoku_cloud",
                diagnostic: serde_json::json!({
                    "tenkan": t_now, "kijun": k_now,
                    "senkou_a": sa_now, "senkou_b": sb_now,
                    "cloud_thickness": cloud_thickness,
                }),
            })
        } else if want_short {
            let stop = k_now.max(close_now + 0.5 * cloud_thickness.max(0.01));
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_now,
                stop_distance: (stop - close_now).max(0.01),
                trigger_index: i,
                stop_price: stop,
                take_profit_price: (close_now
                    - self.rules.take_profit_cloud_mult * cloud_thickness.max(0.01))
                .max(0.01),
                kind: "ichimoku_cloud",
                diagnostic: serde_json::json!({
                    "tenkan": t_now, "kijun": k_now,
                    "senkou_a": sa_now, "senkou_b": sb_now,
                    "cloud_thickness": cloud_thickness,
                }),
            })
        } else {
            None
        }
    }

    fn evaluate_exit(
        &self,
        bars: &[PriceBar],
        side: Side,
        _anchor_high: f64,
        _anchor_low: f64,
    ) -> Option<ExitSignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let (tenkan, kijun, _, _) = ichimoku(bars, &self.rules);
        let i = bars.len() - 1;
        let prev = i - 1;
        let close_now = bars[i].close.to_f64().unwrap_or(0.0);
        let t_now = tenkan[i]?;
        let k_now = kijun[i]?;
        let t_prev = tenkan[prev]?;
        let k_prev = kijun[prev]?;

        match side {
            Side::Buy => {
                if close_now < k_now {
                    return Some(ExitSignal {
                        reason: "ichimoku_kijun_break_long",
                        exit_price: close_now,
                        trigger_index: i,
                    });
                }
                if t_prev >= k_prev && t_now < k_now {
                    return Some(ExitSignal {
                        reason: "ichimoku_tk_bear_cross",
                        exit_price: close_now,
                        trigger_index: i,
                    });
                }
            }
            Side::Sell => {
                if close_now > k_now {
                    return Some(ExitSignal {
                        reason: "ichimoku_kijun_break_short",
                        exit_price: close_now,
                        trigger_index: i,
                    });
                }
                if t_prev <= k_prev && t_now > k_now {
                    return Some(ExitSignal {
                        reason: "ichimoku_tk_bull_cross",
                        exit_price: close_now,
                        trigger_index: i,
                    });
                }
            }
        }
        None
    }
}
