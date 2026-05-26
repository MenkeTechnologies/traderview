//! Signal detection — Golden Cross / Death Cross / Pivot Top / Pivot Bottom /
//! MACD crossover / RSI overbought-oversold / BB squeeze + composite score.
//!
//! Mirrors the StockInvest.us scoring convention: composite is an integer in
//! [-10, +10] where positive is bullish, negative is bearish, 0 = neutral.

use crate::indicators::{adx, bollinger, classic_pivots, closes, highs, lows, macd, rsi, sma, stochastic, Macd, Stoch, Pivots};
use crate::models::PriceBar;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Signal {
    pub name: &'static str,
    pub side: &'static str, // "buy" | "sell" | "hold"
    pub weight: i32,        // contribution to composite score
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalReport {
    pub symbol: String,
    pub last_close: f64,
    pub score: i32,                  // composite, clamped to [-10, +10]
    pub summary: &'static str,       // "buy" | "sell" | "hold"
    pub signals: Vec<Signal>,
    pub indicators: IndicatorSnapshot,
    pub pivots: Option<Pivots>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndicatorSnapshot {
    pub sma20:  Option<f64>,
    pub sma50:  Option<f64>,
    pub sma200: Option<f64>,
    pub ema12:  Option<f64>,
    pub ema26:  Option<f64>,
    pub macd_line:   Option<f64>,
    pub macd_signal: Option<f64>,
    pub macd_hist:   Option<f64>,
    pub rsi14:       Option<f64>,
    pub adx14:       Option<f64>,
    pub plus_di:     Option<f64>,
    pub minus_di:    Option<f64>,
    pub stoch_k:     Option<f64>,
    pub stoch_d:     Option<f64>,
    pub bb_upper:    Option<f64>,
    pub bb_middle:   Option<f64>,
    pub bb_lower:    Option<f64>,
}

pub fn analyze(symbol: &str, bars: &[PriceBar]) -> SignalReport {
    let mut signals = Vec::new();
    let c = closes(bars);
    let h = highs(bars);
    let l = lows(bars);
    let n = c.len();
    let last_close = *c.last().unwrap_or(&0.0);

    let sma20  = sma(&c, 20);
    let sma50  = sma(&c, 50);
    let sma200 = sma(&c, 200);
    let ema12  = crate::indicators::ema(&c, 12);
    let ema26  = crate::indicators::ema(&c, 26);
    let Macd { line: macd_line, signal: macd_sig, histogram: macd_hist } =
        macd(&c, 12, 26, 9);
    let rsi14 = rsi(&c, 14);
    let adx14 = adx(&h, &l, &c, 14);
    let Stoch { k: stoch_k, d: stoch_d } = stochastic(&h, &l, &c, 14, 3);
    let bb = bollinger(&c, 20, 2.0);

    let pivots = if n >= 2 {
        let pi = n - 2; // prior bar
        Some(classic_pivots(crate::indicators::highs(bars)[pi],
                            crate::indicators::lows(bars)[pi],
                            c[pi]))
    } else { None };

    let last = |v: &[Option<f64>]| -> Option<f64> { v.last().and_then(|x| *x) };

    // ---- Trend / moving averages -----------------------------------------
    if let (Some(p), Some(m20)) = (Some(last_close), last(&sma20)) {
        if p > m20 {
            signals.push(Signal { name: "Price > SMA(20)", side: "buy", weight: 1,
                detail: format!("close {:.2} above SMA20 {:.2}", p, m20) });
        } else {
            signals.push(Signal { name: "Price < SMA(20)", side: "sell", weight: -1,
                detail: format!("close {:.2} below SMA20 {:.2}", p, m20) });
        }
    }
    if let (Some(p), Some(m50)) = (Some(last_close), last(&sma50)) {
        if p > m50 {
            signals.push(Signal { name: "Price > SMA(50)", side: "buy", weight: 1,
                detail: format!("close {:.2} above SMA50 {:.2}", p, m50) });
        } else {
            signals.push(Signal { name: "Price < SMA(50)", side: "sell", weight: -1,
                detail: format!("close {:.2} below SMA50 {:.2}", p, m50) });
        }
    }
    if let (Some(p), Some(m200)) = (Some(last_close), last(&sma200)) {
        if p > m200 {
            signals.push(Signal { name: "Above 200-day SMA", side: "buy", weight: 2,
                detail: format!("close {:.2} above SMA200 {:.2}", p, m200) });
        } else {
            signals.push(Signal { name: "Below 200-day SMA", side: "sell", weight: -2,
                detail: format!("close {:.2} below SMA200 {:.2}", p, m200) });
        }
    }

    // ---- Golden / Death Cross (SMA50 vs SMA200) --------------------------
    if n >= 2 {
        let prev_50  = sma50.get(n - 2).and_then(|x| *x);
        let prev_200 = sma200.get(n - 2).and_then(|x| *x);
        if let (Some(p50), Some(p200), Some(c50), Some(c200)) =
            (prev_50, prev_200, last(&sma50), last(&sma200))
        {
            if p50 <= p200 && c50 > c200 {
                signals.push(Signal { name: "Golden Cross", side: "buy", weight: 3,
                    detail: "SMA50 crossed above SMA200".into() });
            } else if p50 >= p200 && c50 < c200 {
                signals.push(Signal { name: "Death Cross", side: "sell", weight: -3,
                    detail: "SMA50 crossed below SMA200".into() });
            }
        }
    }

    // ---- MACD crossover --------------------------------------------------
    if n >= 2 {
        let prev_l = macd_line.get(n - 2).and_then(|x| *x);
        let prev_s = macd_sig.get(n - 2).and_then(|x| *x);
        if let (Some(pl), Some(ps), Some(cl), Some(cs)) =
            (prev_l, prev_s, last(&macd_line), last(&macd_sig))
        {
            if pl <= ps && cl > cs {
                signals.push(Signal { name: "MACD bullish cross", side: "buy", weight: 2,
                    detail: format!("line {:.3} crossed above signal {:.3}", cl, cs) });
            } else if pl >= ps && cl < cs {
                signals.push(Signal { name: "MACD bearish cross", side: "sell", weight: -2,
                    detail: format!("line {:.3} crossed below signal {:.3}", cl, cs) });
            }
        }
        if let Some(h) = last(&macd_hist) {
            if h > 0.0 {
                signals.push(Signal { name: "MACD histogram >0", side: "buy", weight: 1,
                    detail: format!("hist {:.3}", h) });
            } else if h < 0.0 {
                signals.push(Signal { name: "MACD histogram <0", side: "sell", weight: -1,
                    detail: format!("hist {:.3}", h) });
            }
        }
    }

    // ---- RSI -------------------------------------------------------------
    if let Some(r) = last(&rsi14) {
        if r >= 70.0 {
            signals.push(Signal { name: "RSI overbought", side: "sell", weight: -2,
                detail: format!("RSI(14) = {:.1} ≥ 70", r) });
        } else if r <= 30.0 {
            signals.push(Signal { name: "RSI oversold", side: "buy", weight: 2,
                detail: format!("RSI(14) = {:.1} ≤ 30", r) });
        } else if r > 55.0 {
            signals.push(Signal { name: "RSI strong", side: "buy", weight: 1,
                detail: format!("RSI(14) = {:.1}", r) });
        } else if r < 45.0 {
            signals.push(Signal { name: "RSI weak", side: "sell", weight: -1,
                detail: format!("RSI(14) = {:.1}", r) });
        }
    }

    // ---- ADX trend strength + DI direction -------------------------------
    if let (Some(a), Some(pdi), Some(mdi)) = (last(&adx14.adx), last(&adx14.plus_di), last(&adx14.minus_di)) {
        if a >= 25.0 {
            if pdi > mdi {
                signals.push(Signal { name: "Strong uptrend (ADX)", side: "buy", weight: 1,
                    detail: format!("ADX {:.0}, +DI {:.0} > -DI {:.0}", a, pdi, mdi) });
            } else {
                signals.push(Signal { name: "Strong downtrend (ADX)", side: "sell", weight: -1,
                    detail: format!("ADX {:.0}, -DI {:.0} > +DI {:.0}", a, mdi, pdi) });
            }
        }
    }

    // ---- Stochastic crossover --------------------------------------------
    if n >= 2 {
        let pk = stoch_k.get(n - 2).and_then(|x| *x);
        let pd = stoch_d.get(n - 2).and_then(|x| *x);
        if let (Some(pk), Some(pd), Some(k), Some(d)) =
            (pk, pd, last(&stoch_k), last(&stoch_d))
        {
            if pk <= pd && k > d && k < 30.0 {
                signals.push(Signal { name: "Stochastic bullish (oversold)", side: "buy", weight: 1,
                    detail: format!("%K {:.0} crossed above %D {:.0} in oversold", k, d) });
            } else if pk >= pd && k < d && k > 70.0 {
                signals.push(Signal { name: "Stochastic bearish (overbought)", side: "sell", weight: -1,
                    detail: format!("%K {:.0} crossed below %D {:.0} in overbought", k, d) });
            }
        }
    }

    // ---- Bollinger position ----------------------------------------------
    if let (Some(p), Some(u), Some(l)) = (Some(last_close), last(&bb.upper), last(&bb.lower)) {
        if p > u {
            signals.push(Signal { name: "Above upper Bollinger", side: "sell", weight: -1,
                detail: format!("close {:.2} > upper {:.2}", p, u) });
        } else if p < l {
            signals.push(Signal { name: "Below lower Bollinger", side: "buy", weight: 1,
                detail: format!("close {:.2} < lower {:.2}", p, l) });
        }
    }

    // ---- Pivot Top / Bottom (local extrema with confirmation) ------------
    if n >= 5 {
        let i = n - 3;
        let hi_window = &h[i - 2..=i + 2];
        let lo_window = &l[i - 2..=i + 2];
        let center_hi = hi_window[2];
        let center_lo = lo_window[2];
        if hi_window.iter().all(|x| *x <= center_hi) && hi_window[2] > hi_window[0] && hi_window[2] > hi_window[4] {
            signals.push(Signal { name: "Pivot Top", side: "sell", weight: -2,
                detail: format!("local high {:.2}", center_hi) });
        }
        if lo_window.iter().all(|x| *x >= center_lo) && lo_window[2] < lo_window[0] && lo_window[2] < lo_window[4] {
            signals.push(Signal { name: "Pivot Bottom", side: "buy", weight: 2,
                detail: format!("local low {:.2}", center_lo) });
        }
    }

    // ---- Composite score -------------------------------------------------
    let raw: i32 = signals.iter().map(|s| s.weight).sum();
    let score = raw.clamp(-10, 10);
    let summary = if score >= 3 { "buy" } else if score <= -3 { "sell" } else { "hold" };

    SignalReport {
        symbol: symbol.to_string(),
        last_close,
        score,
        summary,
        signals,
        indicators: IndicatorSnapshot {
            sma20:   last(&sma20),
            sma50:   last(&sma50),
            sma200:  last(&sma200),
            ema12:   last(&ema12),
            ema26:   last(&ema26),
            macd_line:   last(&macd_line),
            macd_signal: last(&macd_sig),
            macd_hist:   last(&macd_hist),
            rsi14:       last(&rsi14),
            adx14:       last(&adx14.adx),
            plus_di:     last(&adx14.plus_di),
            minus_di:    last(&adx14.minus_di),
            stoch_k:     last(&stoch_k),
            stoch_d:     last(&stoch_d),
            bb_upper:    last(&bb.upper),
            bb_middle:   last(&bb.middle),
            bb_lower:    last(&bb.lower),
        },
        pivots,
    }
}
