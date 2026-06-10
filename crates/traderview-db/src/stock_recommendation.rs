//! Per-symbol Buy/Sell/Hold recommendation with star rating + composite
//! score + 30-day target price.
//!
//! Mirrors the surface stockinvest.us shows on its per-ticker page: a
//! single verdict + a 1–5 star strength indicator + a numeric score so
//! the user can rank symbols at a glance. The algorithm here is OURS;
//! stockinvest.us doesn't publish theirs, so we compose a transparent
//! weighted blend of the technical indicators we already implement.
//!
//! ## Component blend (weights sum to 1.0)
//!
//! ```text
//! Trend (EMA20 vs EMA50)        0.25   — direction + stack
//! Momentum (20-day ROC)         0.20   — strength of recent move
//! MACD histogram + slope        0.15   — confirmation signal
//! RSI (14, Wilder)              0.15   — overbought / oversold
//! ADX (14)                      0.10   — trend strength multiplier
//! Volume vs 20-day avg + dir    0.15   — conviction filter
//! ```
//!
//! Each component returns a bullish-bias score in `[0, 100]` where 100
//! is maximally bullish and 0 is maximally bearish. The weighted blend
//! is the overall score. Verdict + stars are step functions of that
//! score; target_price is a heuristic 30-day projection that pushes the
//! current close toward the score's bias by up to ±15%.
//!
//! All inputs are derived from `BarInterval::D1` bars fetched via
//! `prices::get_bars`. ~100 trading days (~5 calendar months) are
//! required for ADX/MACD to seed cleanly; the function returns an
//! `EngineError::Insufficient` if fewer than 60 bars come back.

use chrono::{Duration, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::indicators;
use traderview_core::BarInterval;

const MIN_BARS: usize = 60;
const LOOKBACK_DAYS: i64 = 220;
const MOMENTUM_LOOKBACK: usize = 20;
const RSI_PERIOD: usize = 14;
const ADX_PERIOD: usize = 14;
const EMA_FAST: usize = 20;
const EMA_SLOW: usize = 50;
const VOL_AVG_WINDOW: usize = 20;
const TARGET_HORIZON_DAYS: i64 = 30;
/// Max ±swing of the target relative to current price (15% in each
/// direction at score=100 / score=0).
const TARGET_SWING_PCT: f64 = 0.15;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

impl Verdict {
    fn from_score(score: f64) -> Self {
        if score >= 75.0 {
            Verdict::StrongBuy
        } else if score >= 60.0 {
            Verdict::Buy
        } else if score >= 40.0 {
            Verdict::Hold
        } else if score >= 25.0 {
            Verdict::Sell
        } else {
            Verdict::StrongSell
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::StrongBuy => "strong_buy",
            Verdict::Buy => "buy",
            Verdict::Hold => "hold",
            Verdict::Sell => "sell",
            Verdict::StrongSell => "strong_sell",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Component {
    /// Stable identifier for the component (e.g. "trend", "rsi"). Used
    /// by the UI to pin per-component icons / colors.
    pub key: &'static str,
    /// Human label rendered in the breakdown panel.
    pub label: &'static str,
    /// Component weight in the blend, summed to 1.0 across all returned
    /// components.
    pub weight: f64,
    /// Component score in `[0, 100]`. 100 = maximally bullish.
    pub score: f64,
    /// Raw measurement that produced `score` (e.g. RSI = 64.2). Useful
    /// for power users + the breakdown tooltip.
    pub raw_value: Option<f64>,
    /// One-line explanation rendered next to the bar.
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StockRecommendation {
    pub symbol: String,
    pub verdict: Verdict,
    /// 0-100 composite score, weighted blend of components.
    pub score: f64,
    /// 1-5 stars, derived from `score`.
    pub stars: u8,
    /// Last bar's close (in symbol's quote currency).
    pub current_price: Decimal,
    /// Heuristic 30-day target. See module doc for the formula.
    pub target_price: Decimal,
    /// Upside as a percentage from `current_price` to `target_price`.
    pub upside_pct: f64,
    pub horizon_days: i64,
    pub components: Vec<Component>,
    /// Total bars used to compute the score (after filtering).
    pub bars_analyzed: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum RecommendationError {
    #[error("not enough bars for {symbol}: got {got}, need {need}")]
    Insufficient {
        symbol: String,
        got: usize,
        need: usize,
    },
    #[error("price fetch failed: {0}")]
    PriceFetch(anyhow::Error),
    #[error("current price is non-positive: {0}")]
    InvalidPrice(Decimal),
}

pub async fn compute(
    pool: &PgPool,
    symbol: &str,
) -> Result<StockRecommendation, RecommendationError> {
    let to = Utc::now();
    let from = to - Duration::days(LOOKBACK_DAYS);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .map_err(RecommendationError::PriceFetch)?;
    let n = bars.len();
    if n < MIN_BARS {
        return Err(RecommendationError::Insufficient {
            symbol: symbol.to_string(),
            got: n,
            need: MIN_BARS,
        });
    }
    let current = bars.last().map(|b| b.close).unwrap_or_default();
    if current <= Decimal::ZERO {
        return Err(RecommendationError::InvalidPrice(current));
    }
    let closes = indicators::closes(&bars);
    let highs = indicators::highs(&bars);
    let lows = indicators::lows(&bars);
    let volumes = indicators::volumes(&bars);

    let components = vec![
        score_trend(&closes),
        score_momentum(&closes),
        score_macd(&closes),
        score_rsi(&closes),
        score_adx(&highs, &lows, &closes),
        score_volume(&volumes, &closes),
    ];
    let weighted: f64 = components.iter().map(|c| c.weight * c.score).sum();
    let total_weight: f64 = components.iter().map(|c| c.weight).sum();
    let score = if total_weight > 0.0 {
        weighted / total_weight
    } else {
        50.0
    };

    let verdict = Verdict::from_score(score);
    let stars = stars_from_score(score);
    let target_price = project_target(current, score);
    let current_f = decimal_to_f64(current);
    let target_f = decimal_to_f64(target_price);
    let upside_pct = if current_f > 0.0 {
        (target_f - current_f) / current_f * 100.0
    } else {
        0.0
    };

    Ok(StockRecommendation {
        symbol: symbol.to_string(),
        verdict,
        score,
        stars,
        current_price: current,
        target_price,
        upside_pct,
        horizon_days: TARGET_HORIZON_DAYS,
        components,
        bars_analyzed: n,
    })
}

fn stars_from_score(score: f64) -> u8 {
    match score {
        s if s >= 80.0 => 5,
        s if s >= 65.0 => 4,
        s if s >= 50.0 => 3,
        s if s >= 30.0 => 2,
        _ => 1,
    }
}

fn project_target(current: Decimal, score: f64) -> Decimal {
    // Bias factor in [-1, +1] derived from score's distance from 50.
    let bias = ((score - 50.0) / 50.0).clamp(-1.0, 1.0);
    let multiplier = 1.0 + bias * TARGET_SWING_PCT;
    let cur_f = decimal_to_f64(current);
    let target = cur_f * multiplier;
    Decimal::from_f64(target).unwrap_or(current)
}

// ===========================================================================
// Component scorers
// ===========================================================================

fn score_trend(closes: &[f64]) -> Component {
    let ema_fast = indicators::ema(closes, EMA_FAST);
    let ema_slow = indicators::ema(closes, EMA_SLOW);
    let last = closes.last().copied().unwrap_or(0.0);
    let ef = ema_fast.last().and_then(|x| *x).unwrap_or(last);
    let es = ema_slow.last().and_then(|x| *x).unwrap_or(last);
    // Score: 100 if price > fast > slow (full bullish stack), 0 if
    // inverted; linear interp via two boolean checks.
    let price_over_fast = if last >= ef { 1.0 } else { 0.0 };
    let fast_over_slow = if ef >= es { 1.0 } else { 0.0 };
    // Slope of fast EMA over last 5 bars amplifies the score by ±20.
    let slope = if ema_fast.len() >= 6 {
        let prev = ema_fast[ema_fast.len() - 6].unwrap_or(ef);
        ((ef - prev) / prev.abs().max(1e-9)).clamp(-0.05, 0.05) / 0.05
    } else {
        0.0
    };
    let base = (price_over_fast + fast_over_slow) / 2.0 * 100.0;
    let score = (base + slope * 20.0).clamp(0.0, 100.0);
    let note = format!(
        "EMA{}={:.2}, EMA{}={:.2}, price={:.2}",
        EMA_FAST, ef, EMA_SLOW, es, last
    );
    Component {
        key: "trend",
        label: "Trend (EMA 20/50)",
        weight: 0.25,
        score,
        raw_value: Some(ef - es),
        note,
    }
}

fn score_momentum(closes: &[f64]) -> Component {
    let n = closes.len();
    if n <= MOMENTUM_LOOKBACK {
        return Component {
            key: "momentum",
            label: "Momentum (20-day ROC)",
            weight: 0.20,
            score: 50.0,
            raw_value: None,
            note: "not enough bars".into(),
        };
    }
    let cur = closes[n - 1];
    let ref_close = closes[n - 1 - MOMENTUM_LOOKBACK];
    let roc_pct = if ref_close.abs() > 1e-9 {
        (cur - ref_close) / ref_close * 100.0
    } else {
        0.0
    };
    // Map ±10% ROC linearly to ±50 score points around 50.
    let score = (50.0 + (roc_pct.clamp(-10.0, 10.0)) * 5.0).clamp(0.0, 100.0);
    Component {
        key: "momentum",
        label: "Momentum (20-day ROC)",
        weight: 0.20,
        score,
        raw_value: Some(roc_pct),
        note: format!("{roc_pct:+.2}% over {MOMENTUM_LOOKBACK} sessions"),
    }
}

fn score_macd(closes: &[f64]) -> Component {
    let m = indicators::macd(closes, 12, 26, 9);
    let last_hist = m.histogram.iter().rev().find_map(|x| *x).unwrap_or(0.0);
    // Look back 3 bars for slope; positive + rising = strong bullish.
    let prev_hist = if m.histogram.len() >= 4 {
        m.histogram[m.histogram.len() - 4].unwrap_or(0.0)
    } else {
        0.0
    };
    let direction = if last_hist > 0.0 { 1.0 } else { -1.0 };
    let rising = last_hist > prev_hist;
    let base: f64 = if direction > 0.0 { 70.0 } else { 30.0 };
    let bump: f64 = match (direction > 0.0, rising) {
        (true, true) => 25.0,
        (true, false) => -10.0,
        (false, true) => 20.0,
        (false, false) => -25.0,
    };
    let score = (base + bump).clamp(0.0, 100.0);
    Component {
        key: "macd",
        label: "MACD histogram",
        weight: 0.15,
        score,
        raw_value: Some(last_hist),
        note: format!("hist={last_hist:.4}, slope={}", if rising { "↑" } else { "↓" }),
    }
}

fn score_rsi(closes: &[f64]) -> Component {
    let series = indicators::rsi(closes, RSI_PERIOD);
    let rsi = series.iter().rev().find_map(|x| *x).unwrap_or(50.0);
    // Tent function: oversold (≤30) → bullish; overbought (≥70) → bearish.
    // Center of the tent at 50 = neutral hold.
    let score = if rsi <= 30.0 {
        // 30 → 70 (mild buy), 0 → 90 (strong buy)
        70.0 + (30.0 - rsi) * (20.0 / 30.0)
    } else if rsi <= 50.0 {
        // 30 → 70 (buy zone fades), 50 → 50 (neutral)
        50.0 + (50.0 - rsi) * (20.0 / 20.0)
    } else if rsi <= 70.0 {
        // 50 → 50, 70 → 35
        50.0 - (rsi - 50.0) * (15.0 / 20.0)
    } else {
        // 70 → 35, 100 → 10
        35.0 - (rsi - 70.0) * (25.0 / 30.0)
    };
    let note = if rsi <= 30.0 {
        format!("RSI {rsi:.1} — oversold")
    } else if rsi >= 70.0 {
        format!("RSI {rsi:.1} — overbought")
    } else {
        format!("RSI {rsi:.1} — neutral")
    };
    Component {
        key: "rsi",
        label: "RSI (14)",
        weight: 0.15,
        score: score.clamp(0.0, 100.0),
        raw_value: Some(rsi),
        note,
    }
}

fn score_adx(highs: &[f64], lows: &[f64], closes: &[f64]) -> Component {
    let adx_data = indicators::adx(highs, lows, closes, ADX_PERIOD);
    let adx = adx_data
        .adx
        .iter()
        .rev()
        .find_map(|x| *x)
        .unwrap_or(0.0);
    let plus = adx_data
        .plus_di
        .iter()
        .rev()
        .find_map(|x| *x)
        .unwrap_or(0.0);
    let minus = adx_data
        .minus_di
        .iter()
        .rev()
        .find_map(|x| *x)
        .unwrap_or(0.0);
    // ADX rates the STRENGTH of trend, not direction. Direction comes
    // from +DI vs -DI. Combine: strong trend (adx > 25) AND +DI > -DI
    // → strong bull; strong trend AND -DI > +DI → strong bear.
    let direction_sign = if plus > minus { 1.0 } else { -1.0 };
    let strength = (adx.clamp(0.0, 50.0)) / 50.0; // 0..1
    let score = 50.0 + direction_sign * strength * 40.0;
    let note = if adx < 20.0 {
        format!("ADX {adx:.1} — choppy / no trend")
    } else if direction_sign > 0.0 {
        format!("ADX {adx:.1} — bullish trend, +DI {plus:.1} > -DI {minus:.1}")
    } else {
        format!("ADX {adx:.1} — bearish trend, -DI {minus:.1} > +DI {plus:.1}")
    };
    Component {
        key: "adx",
        label: "ADX (14)",
        weight: 0.10,
        score: score.clamp(0.0, 100.0),
        raw_value: Some(adx),
        note,
    }
}

fn score_volume(volumes: &[f64], closes: &[f64]) -> Component {
    let n = volumes.len();
    if n < VOL_AVG_WINDOW + 1 {
        return Component {
            key: "volume",
            label: "Volume vs 20-day avg",
            weight: 0.15,
            score: 50.0,
            raw_value: None,
            note: "not enough bars".into(),
        };
    }
    let recent_vol = volumes[n - 1];
    let avg: f64 = volumes[n - 1 - VOL_AVG_WINDOW..n - 1].iter().sum::<f64>()
        / VOL_AVG_WINDOW as f64;
    let ratio = if avg > 1e-9 { recent_vol / avg } else { 1.0 };
    let price_change = if n >= 2 { closes[n - 1] - closes[n - 2] } else { 0.0 };
    let price_up = price_change > 0.0;
    // Above-average volume confirms the direction; below-average means
    // the recent move lacks conviction.
    let base = 50.0;
    let convict = ((ratio - 1.0).clamp(-1.0, 2.0)) * 20.0;
    let score = if price_up { base + convict } else { base - convict };
    let note = format!(
        "vol×{ratio:.2}, last bar {}",
        if price_up { "up" } else { "down" }
    );
    Component {
        key: "volume",
        label: "Volume vs 20-day avg",
        weight: 0.15,
        score: score.clamp(0.0, 100.0),
        raw_value: Some(ratio),
        note,
    }
}

fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_thresholds_match_doc() {
        assert_eq!(Verdict::from_score(80.0), Verdict::StrongBuy);
        assert_eq!(Verdict::from_score(75.0), Verdict::StrongBuy);
        assert_eq!(Verdict::from_score(74.9), Verdict::Buy);
        assert_eq!(Verdict::from_score(60.0), Verdict::Buy);
        assert_eq!(Verdict::from_score(59.9), Verdict::Hold);
        assert_eq!(Verdict::from_score(40.0), Verdict::Hold);
        assert_eq!(Verdict::from_score(39.9), Verdict::Sell);
        assert_eq!(Verdict::from_score(25.0), Verdict::Sell);
        assert_eq!(Verdict::from_score(24.9), Verdict::StrongSell);
        assert_eq!(Verdict::from_score(0.0), Verdict::StrongSell);
    }

    #[test]
    fn stars_match_score_buckets() {
        assert_eq!(stars_from_score(95.0), 5);
        assert_eq!(stars_from_score(80.0), 5);
        assert_eq!(stars_from_score(79.9), 4);
        assert_eq!(stars_from_score(65.0), 4);
        assert_eq!(stars_from_score(64.9), 3);
        assert_eq!(stars_from_score(50.0), 3);
        assert_eq!(stars_from_score(49.9), 2);
        assert_eq!(stars_from_score(30.0), 2);
        assert_eq!(stars_from_score(29.9), 1);
        assert_eq!(stars_from_score(0.0), 1);
    }

    #[test]
    fn target_swing_caps_at_15_pct_each_side() {
        let cur = Decimal::new(10000, 2); // $100.00
        let bull = project_target(cur, 100.0);
        let bear = project_target(cur, 0.0);
        let neutral = project_target(cur, 50.0);
        // 100 → +15%, 0 → -15%, 50 → unchanged.
        assert!((decimal_to_f64(bull) - 115.0).abs() < 0.01);
        assert!((decimal_to_f64(bear) - 85.0).abs() < 0.01);
        assert!((decimal_to_f64(neutral) - 100.0).abs() < 0.01);
    }

    #[test]
    fn trend_score_with_steady_uptrend() {
        // Synthesize 100 bars rising 0.5% per bar.
        let closes: Vec<f64> = (0..100).map(|i| 100.0 * 1.005_f64.powi(i)).collect();
        let c = score_trend(&closes);
        // Strong uptrend → price > EMA20 > EMA50 → base 100, plus slope.
        assert!(c.score >= 90.0, "expected ≥90, got {}", c.score);
    }

    #[test]
    fn trend_score_with_steady_downtrend() {
        let closes: Vec<f64> = (0..100).map(|i| 100.0 * 0.995_f64.powi(i)).collect();
        let c = score_trend(&closes);
        assert!(c.score <= 10.0, "expected ≤10, got {}", c.score);
    }

    #[test]
    fn momentum_score_caps_at_extremes() {
        let mut closes = vec![100.0; 30];
        closes[29] = 200.0; // +100% ROC over 20d clamps at +10%
        let c = score_momentum(&closes);
        assert!((c.score - 100.0).abs() < 0.01);
        let mut closes_down = vec![100.0; 30];
        closes_down[29] = 50.0;
        let c2 = score_momentum(&closes_down);
        assert!((c2.score - 0.0).abs() < 0.01);
    }

    #[test]
    fn rsi_oversold_scores_bullish() {
        // RSI ≤ 30 → high score (buy the dip).
        let mut closes = vec![100.0];
        for _ in 0..30 {
            closes.push(closes.last().unwrap() * 0.98);
        }
        let c = score_rsi(&closes);
        // Should be in the buy zone (>60).
        assert!(c.score >= 60.0, "expected ≥60, got {}", c.score);
    }

    #[test]
    fn rsi_overbought_scores_bearish() {
        let mut closes = vec![100.0];
        for _ in 0..30 {
            closes.push(closes.last().unwrap() * 1.02);
        }
        let c = score_rsi(&closes);
        assert!(c.score <= 40.0, "expected ≤40, got {}", c.score);
    }
}
