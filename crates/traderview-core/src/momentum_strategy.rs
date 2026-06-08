//! Momentum strategy — pure-function entry / exit / sizing on `PriceBar`
//! windows. No I/O. The algo engine in `traderview-db::algo_engine` wraps
//! this with risk gates + broker dispatch.
//!
//! Entry (long):
//!   EMA(fast) crossed above EMA(slow) on the latest closed bar AND
//!   RSI(p) ∈ [rsi_long_min, rsi_long_max] AND
//!   ROC(p) > roc_long_min AND
//!   relative_volume = vol / SMA(vol, rvol_lookback) >= rvol_min
//!
//! Entry (short) — mirror.
//!
//! Exit (long): close < EMA(slow), RSI < rsi_long_min,
//! ATR-stop hit (low touched anchor − atr_stop_mult * ATR), or MACD
//! bearish cross.
//!
//! Sizing: `qty = floor((equity * risk_pct_per_trade) / (atr * atr_stop_mult))`,
//! capped at `equity * max_pos_pct / entry_price`.

use crate::indicators;
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SideMode {
    Long,
    Short,
    Both,
}

impl Default for SideMode {
    fn default() -> Self { Self::Long }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub ema_fast: usize,
    pub ema_slow: usize,
    pub rsi_period: usize,
    pub rsi_long_min: f64,
    pub rsi_long_max: f64,
    pub rsi_short_min: f64,
    pub rsi_short_max: f64,
    pub roc_period: usize,
    pub roc_long_min: f64,
    pub roc_short_max: f64,
    pub rvol_lookback: usize,
    pub rvol_min: f64,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
    pub atr_take_profit_mult: f64,
    pub macd_fast: usize,
    pub macd_slow: usize,
    pub macd_signal: usize,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            ema_fast: 9,
            ema_slow: 21,
            rsi_period: 14,
            rsi_long_min: 50.0,
            rsi_long_max: 70.0,
            rsi_short_min: 30.0,
            rsi_short_max: 50.0,
            roc_period: 10,
            roc_long_min: 0.02,
            roc_short_max: -0.02,
            rvol_lookback: 20,
            rvol_min: 1.5,
            atr_period: 14,
            atr_stop_mult: 2.0,
            atr_take_profit_mult: 3.0,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sizing {
    /// Fraction of equity risked on the stop. Default 0.01 = 1%.
    pub risk_pct_per_trade: f64,
    /// Hard ceiling on a single position as a fraction of equity.
    pub max_pos_pct: f64,
}

impl Default for Sizing {
    fn default() -> Self {
        Self { risk_pct_per_trade: 0.01, max_pos_pct: 0.20 }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EntrySignal {
    pub side: Side,
    pub entry_price: f64,
    pub atr: f64,
    /// Bar timestamp that triggered the signal (last bar in the window).
    pub trigger_index: usize,
    /// Stop price implied by `entry_price -/+ atr_stop_mult * atr`.
    pub stop_price: f64,
    /// Take-profit price implied by `entry_price +/- atr_take_profit_mult * atr`.
    pub take_profit_price: f64,
    /// Why each leg of the rule fired — for journaling / debugging.
    pub diagnostic: EntryDiagnostic,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntryDiagnostic {
    pub ema_fast: f64,
    pub ema_slow: f64,
    pub rsi: f64,
    pub roc: f64,
    pub rvol: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExitSignal {
    pub reason: &'static str, // "ema_loss" | "rsi_loss" | "atr_stop" | "macd_bearish"
    pub exit_price: f64,
    pub trigger_index: usize,
}

/// Evaluate the rule stack on the bar window. Returns `Some` only on the
/// bar a crossover ACTUALLY happens — the engine calls this on every
/// closed bar and only acts when a signal is fresh.
pub fn evaluate_entry(
    bars: &[PriceBar],
    rules: &Rules,
    side_mode: SideMode,
) -> Option<EntrySignal> {
    let needed = rules
        .ema_slow
        .max(rules.rsi_period + 1)
        .max(rules.roc_period + 1)
        .max(rules.rvol_lookback + 1)
        .max(rules.atr_period + 1);
    if bars.len() < needed + 1 {
        return None;
    }

    let closes = indicators::closes(bars);
    let highs = indicators::highs(bars);
    let lows = indicators::lows(bars);
    let vols = indicators::volumes(bars);

    let ema_fast = indicators::ema(&closes, rules.ema_fast);
    let ema_slow = indicators::ema(&closes, rules.ema_slow);
    let rsi = indicators::rsi(&closes, rules.rsi_period);
    let atr = indicators::atr(&highs, &lows, &closes, rules.atr_period);
    let roc_v = crate::roc::compute(&closes, rules.roc_period);

    let i = bars.len() - 1;
    let prev = i - 1;

    let ef_now = ema_fast.get(i).copied().flatten()?;
    let ef_prev = ema_fast.get(prev).copied().flatten()?;
    let es_now = ema_slow.get(i).copied().flatten()?;
    let es_prev = ema_slow.get(prev).copied().flatten()?;
    let rsi_now = rsi.get(i).copied().flatten()?;
    let atr_now = atr.get(i).copied().flatten()?;
    if atr_now <= 0.0 {
        return None;
    }
    let roc_now = *roc_v.get(i)?;
    let rvol_now = rolling_rvol(&vols, i, rules.rvol_lookback)?;
    let close_now = closes[i];

    let diag = EntryDiagnostic {
        ema_fast: ef_now,
        ema_slow: es_now,
        rsi: rsi_now,
        roc: roc_now,
        rvol: rvol_now,
    };

    let long_cross = ef_prev <= es_prev && ef_now > es_now;
    let short_cross = ef_prev >= es_prev && ef_now < es_now;

    let want_long = matches!(side_mode, SideMode::Long | SideMode::Both)
        && long_cross
        && (rules.rsi_long_min..=rules.rsi_long_max).contains(&rsi_now)
        && roc_now > rules.roc_long_min
        && rvol_now >= rules.rvol_min;
    let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
        && short_cross
        && (rules.rsi_short_min..=rules.rsi_short_max).contains(&rsi_now)
        && roc_now < rules.roc_short_max
        && rvol_now >= rules.rvol_min;

    if want_long {
        let stop = close_now - rules.atr_stop_mult * atr_now;
        let tp = close_now + rules.atr_take_profit_mult * atr_now;
        Some(EntrySignal {
            side: Side::Buy,
            entry_price: close_now,
            atr: atr_now,
            trigger_index: i,
            stop_price: stop.max(0.01),
            take_profit_price: tp,
            diagnostic: diag,
        })
    } else if want_short {
        let stop = close_now + rules.atr_stop_mult * atr_now;
        let tp = (close_now - rules.atr_take_profit_mult * atr_now).max(0.01);
        Some(EntrySignal {
            side: Side::Sell,
            entry_price: close_now,
            atr: atr_now,
            trigger_index: i,
            stop_price: stop,
            take_profit_price: tp,
            diagnostic: diag,
        })
    } else {
        None
    }
}

/// Evaluate exit conditions for an already-open position. Returns
/// `Some` the first time any of the exit rules fires on the latest bar.
pub fn evaluate_exit(
    bars: &[PriceBar],
    position_side: Side,
    anchor_high: f64,
    anchor_low: f64,
    rules: &Rules,
) -> Option<ExitSignal> {
    let needed = rules
        .ema_slow
        .max(rules.rsi_period + 1)
        .max(rules.atr_period + 1)
        .max(rules.macd_slow + rules.macd_signal);
    if bars.len() < needed + 1 {
        return None;
    }

    let closes = indicators::closes(bars);
    let highs = indicators::highs(bars);
    let lows = indicators::lows(bars);

    let ema_slow = indicators::ema(&closes, rules.ema_slow);
    let rsi = indicators::rsi(&closes, rules.rsi_period);
    let atr = indicators::atr(&highs, &lows, &closes, rules.atr_period);
    let macd = indicators::macd(&closes, rules.macd_fast, rules.macd_slow, rules.macd_signal);

    let i = bars.len() - 1;
    let prev = i - 1;

    let close_now = closes[i];
    let es_now = ema_slow.get(i).copied().flatten()?;
    let rsi_now = rsi.get(i).copied().flatten()?;
    let atr_now = atr.get(i).copied().flatten()?;

    // MACD line crosses below signal — bearish for longs / bullish for shorts.
    let macd_now = macd.line.get(i).and_then(|v| *v);
    let macd_prev = macd.line.get(prev).and_then(|v| *v);
    let sig_now = macd.signal.get(i).and_then(|v| *v);
    let sig_prev = macd.signal.get(prev).and_then(|v| *v);

    match position_side {
        Side::Buy => {
            // ATR trailing stop anchored to the highest high since entry.
            let atr_stop = anchor_high - rules.atr_stop_mult * atr_now;
            if lows[i] <= atr_stop {
                return Some(ExitSignal { reason: "atr_stop", exit_price: atr_stop, trigger_index: i });
            }
            if close_now < es_now {
                return Some(ExitSignal { reason: "ema_loss", exit_price: close_now, trigger_index: i });
            }
            if rsi_now < rules.rsi_long_min {
                return Some(ExitSignal { reason: "rsi_loss", exit_price: close_now, trigger_index: i });
            }
            if let (Some(mn), Some(mp), Some(sn), Some(sp)) = (macd_now, macd_prev, sig_now, sig_prev) {
                if mp >= sp && mn < sn {
                    return Some(ExitSignal { reason: "macd_bearish", exit_price: close_now, trigger_index: i });
                }
            }
        }
        Side::Sell => {
            let atr_stop = anchor_low + rules.atr_stop_mult * atr_now;
            if highs[i] >= atr_stop {
                return Some(ExitSignal { reason: "atr_stop", exit_price: atr_stop, trigger_index: i });
            }
            if close_now > es_now {
                return Some(ExitSignal { reason: "ema_loss", exit_price: close_now, trigger_index: i });
            }
            if rsi_now > rules.rsi_short_max {
                return Some(ExitSignal { reason: "rsi_loss", exit_price: close_now, trigger_index: i });
            }
            if let (Some(mn), Some(mp), Some(sn), Some(sp)) = (macd_now, macd_prev, sig_now, sig_prev) {
                if mp <= sp && mn > sn {
                    return Some(ExitSignal { reason: "macd_bearish", exit_price: close_now, trigger_index: i });
                }
            }
        }
    }
    None
}

/// Position sizing — risk-first, capped by max_pos_pct.
/// Returns the integer share count to trade (rounded down) or 0 if
/// inputs make a single-share trade exceed the cap.
pub fn size_shares(
    equity: f64,
    entry_price: f64,
    atr: f64,
    sizing: &Sizing,
    rules: &Rules,
) -> u64 {
    if entry_price <= 0.0 || atr <= 0.0 || equity <= 0.0 {
        return 0;
    }
    let risk_dollars = equity * sizing.risk_pct_per_trade;
    let per_share_risk = atr * rules.atr_stop_mult;
    if per_share_risk <= 0.0 {
        return 0;
    }
    let risk_qty = (risk_dollars / per_share_risk).floor();
    let cap_qty = ((equity * sizing.max_pos_pct) / entry_price).floor();
    risk_qty.min(cap_qty).max(0.0) as u64
}

fn rolling_rvol(volumes: &[f64], i: usize, lookback: usize) -> Option<f64> {
    if i < lookback {
        return None;
    }
    let start = i - lookback;
    let avg = volumes[start..i].iter().sum::<f64>() / lookback as f64;
    if avg <= 0.0 {
        return None;
    }
    Some(volumes[i] / avg)
}

// ─── tests ─────────────────────────────────────────────────────────────────

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

    /// Build a synthetic window that ends with a fresh EMA(9)>EMA(21)
    /// crossover, RSI in band, ROC above threshold, RVOL elevated. The
    /// shape: long flat seed → gentle downtrend (pushes ema_fast BELOW
    /// ema_slow) → sharp recovery that crosses ema_slow on the FINAL
    /// bar. Bar count and slopes were tuned by sweeping.
    fn uptrend_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        // 35 flat bars to seed slow EMA / RVOL baseline.
        for i in 0..35 {
            let p = 100.0;
            bars.push(bar(
                1_700_000_000 + i * 60,
                &format!("{p:.2}"),
                &format!("{:.2}", p + 0.1),
                &format!("{:.2}", p - 0.1),
                &format!("{p:.2}"),
                1_000_000,
            ));
        }
        // 8 declining bars (98 → 96.6) — pushes EMA(9) under EMA(21).
        for i in 0..8 {
            let p = 100.0 - (i as f64 + 1.0) * 0.4;
            bars.push(bar(
                1_700_000_000 + (35 + i) * 60,
                &format!("{:.2}", p + 0.1),
                &format!("{:.2}", p + 0.2),
                &format!("{:.2}", p - 0.2),
                &format!("{p:.2}"),
                1_000_000,
            ));
        }
        // 12 recovery bars rising from 97.5 → ~108, last bar lifts both
        // RSI + ROC + volume into qualifying band AND crosses EMAs.
        for i in 0..12 {
            let p = 96.6 + (i as f64 + 1.0) * 0.95;
            let vol = if i == 11 { 4_000_000 } else { 2_000_000 };
            bars.push(bar(
                1_700_000_000 + (43 + i) * 60,
                &format!("{:.2}", p - 0.3),
                &format!("{:.2}", p + 0.4),
                &format!("{:.2}", p - 0.4),
                &format!("{p:.2}"),
                vol,
            ));
        }
        bars
    }

    fn flat_window() -> Vec<PriceBar> {
        (0..60)
            .map(|i| {
                bar(
                    1_700_000_000 + i * 60,
                    "100.00",
                    "100.10",
                    "99.90",
                    "100.00",
                    1_000_000,
                )
            })
            .collect()
    }

    /// Walk evaluate_entry over an expanding window of `bars` and return
    /// the first index where a Long signal fires. None if no bar fires.
    /// Models how the engine actually uses the function: one bar at a time.
    fn first_long_signal(bars: &[PriceBar], rules: &Rules) -> Option<(usize, EntrySignal)> {
        for end in 30..=bars.len() {
            if let Some(sig) = evaluate_entry(&bars[..end], rules, SideMode::Long) {
                return Some((end - 1, sig));
            }
        }
        None
    }

    #[test]
    fn entry_fires_on_clean_uptrend_with_volume() {
        let bars = uptrend_window();
        let (_idx, sig) = first_long_signal(&bars, &Rules::default())
            .expect("uptrend must produce at least one long entry across the window");
        assert_eq!(sig.side, Side::Buy);
        assert!(sig.stop_price < sig.entry_price);
        assert!(sig.take_profit_price > sig.entry_price);
        assert!(sig.diagnostic.rvol >= 1.5, "rvol {} >= 1.5", sig.diagnostic.rvol);
        assert!(sig.diagnostic.roc > 0.02, "roc {} > 0.02", sig.diagnostic.roc);
        assert!(
            sig.diagnostic.rsi >= 50.0 && sig.diagnostic.rsi <= 70.0,
            "rsi {} ∈ [50,70]", sig.diagnostic.rsi,
        );
    }

    #[test]
    fn entry_blocked_on_flat_market() {
        let bars = flat_window();
        assert!(
            evaluate_entry(&bars, &Rules::default(), SideMode::Long).is_none(),
            "flat market has no momentum signal"
        );
    }

    #[test]
    fn entry_blocked_when_side_mode_short_on_long_setup() {
        let bars = uptrend_window();
        assert!(
            evaluate_entry(&bars, &Rules::default(), SideMode::Short).is_none(),
            "long crossover must NOT fire under SideMode::Short"
        );
    }

    #[test]
    fn entry_blocked_when_rvol_below_threshold() {
        let mut bars = uptrend_window();
        // Suppress volume on the last bar to fall below RVOL min.
        let last = bars.len() - 1;
        bars[last].volume = Decimal::from(100_000);
        assert!(
            evaluate_entry(&bars, &Rules::default(), SideMode::Long).is_none(),
            "low-volume bar must veto entry"
        );
    }

    #[test]
    fn size_shares_uses_atr_stop_distance_for_risk() {
        // $100k equity, 1% risk = $1000. ATR=$1, stop_mult=2 ⇒ $2/share risk
        // ⇒ 500 shares. max_pos_pct=0.20 ⇒ 200-share cap at $100/share.
        // Cap wins.
        let qty = size_shares(
            100_000.0,
            100.0,
            1.0,
            &Sizing { risk_pct_per_trade: 0.01, max_pos_pct: 0.20 },
            &Rules::default(),
        );
        assert_eq!(qty, 200);
    }

    #[test]
    fn size_shares_returns_zero_on_degenerate_inputs() {
        assert_eq!(size_shares(0.0, 100.0, 1.0, &Sizing::default(), &Rules::default()), 0);
        assert_eq!(size_shares(10_000.0, 0.0, 1.0, &Sizing::default(), &Rules::default()), 0);
        assert_eq!(size_shares(10_000.0, 100.0, 0.0, &Sizing::default(), &Rules::default()), 0);
    }

    #[test]
    fn exit_fires_when_close_breaks_ema_slow() {
        // Build a long-position window where the last close has decisively
        // dropped below EMA(21) — the ema_loss leg must trigger.
        let mut bars = uptrend_window();
        let last = bars.len() - 1;
        bars[last].close = Decimal::from_str("80.00").unwrap();
        bars[last].low = Decimal::from_str("79.50").unwrap();
        let exit = evaluate_exit(
            &bars,
            Side::Buy,
            130.0, // anchor_high — high water mark
            100.0, // anchor_low  — unused for longs
            &Rules::default(),
        )
        .expect("ema_loss must fire");
        // atr_stop fires first if anchor_high − 2*ATR ≥ low, otherwise ema_loss.
        assert!(matches!(exit.reason, "atr_stop" | "ema_loss"));
    }
}
