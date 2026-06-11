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

use chrono::{DateTime, Duration, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::indicators;
use traderview_core::BarInterval;
use uuid::Uuid;

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

/// Volatility-based risk classification, stockinvest.us-style badge.
/// Computed from annualized daily-return volatility:
///   < 25%  → Low, 25–50% → Medium, > 50% → High.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
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
    /// Risk badge from annualized volatility of daily returns.
    pub risk_level: RiskLevel,
    /// Annualized volatility percent backing `risk_level` (e.g. 32.5).
    pub annualized_vol_pct: f64,
    /// Nearest support below current price — highest swing low in the
    /// last 60 bars that sits below the close. None when no swing low
    /// qualifies (e.g. price at all-time low).
    pub support: Option<Decimal>,
    /// Nearest resistance above current price — lowest swing high in
    /// the last 60 bars that sits above the close.
    pub resistance: Option<Decimal>,
    /// 3-month (66 trading days) forecast band: the score-biased drift
    /// from `target_price` extended to the longer horizon, widened by
    /// ±1σ of horizon volatility. The honest "could be anywhere in
    /// here" range, not a point estimate.
    pub forecast_3m_low: Decimal,
    pub forecast_3m_high: Decimal,
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

/// Per-component weight overrides. None for any field keeps the
/// algorithm's default. Used by `compute_with_weights` and the
/// `?trend_w=...&rsi_w=...` query parameters on the HTTP route so the
/// user can tune the blend without a redeploy. Component scores
/// themselves are unchanged — only the weighting changes.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct WeightOverrides {
    pub trend: Option<f64>,
    pub momentum: Option<f64>,
    pub macd: Option<f64>,
    pub rsi: Option<f64>,
    pub adx: Option<f64>,
    pub volume: Option<f64>,
}

impl WeightOverrides {
    pub fn any(&self) -> bool {
        self.trend.is_some()
            || self.momentum.is_some()
            || self.macd.is_some()
            || self.rsi.is_some()
            || self.adx.is_some()
            || self.volume.is_some()
    }
    fn lookup(&self, key: &str) -> Option<f64> {
        match key {
            "trend" => self.trend,
            "momentum" => self.momentum,
            "macd" => self.macd,
            "rsi" => self.rsi,
            "adx" => self.adx,
            "volume" => self.volume,
            _ => None,
        }
    }
}

pub async fn compute(
    pool: &PgPool,
    symbol: &str,
) -> Result<StockRecommendation, RecommendationError> {
    compute_with_weights(pool, symbol, None).await
}

pub async fn compute_with_weights(
    pool: &PgPool,
    symbol: &str,
    overrides: Option<&WeightOverrides>,
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

    let mut components = vec![
        score_trend(&closes),
        score_momentum(&closes),
        score_macd(&closes),
        score_rsi(&closes),
        score_adx(&highs, &lows, &closes),
        score_volume(&volumes, &closes),
    ];
    if let Some(ov) = overrides {
        if ov.any() {
            // Apply each non-None override; renormalize so the
            // resulting weights still sum to 1.0 (otherwise a small
            // override silently dilutes the score).
            for c in components.iter_mut() {
                if let Some(w) = ov.lookup(c.key) {
                    c.weight = w.max(0.0);
                }
            }
            let total: f64 = components.iter().map(|c| c.weight).sum();
            if total > 0.0 {
                for c in components.iter_mut() {
                    c.weight /= total;
                }
            }
        }
    }
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

    // Risk badge from annualized daily-return volatility.
    let annualized_vol_pct = annualized_volatility_pct(&closes);
    let risk_level = risk_from_vol(annualized_vol_pct);

    // Support/resistance: swing levels from the last 60 bars.
    let (support, resistance) = swing_levels(&highs, &lows, current_f);

    // 3-month forecast band: score-biased drift over 66 trading days,
    // widened by ±1σ of horizon volatility (vol_annual × sqrt(66/252)).
    let (forecast_3m_low, forecast_3m_high) =
        forecast_band(current_f, score, annualized_vol_pct);

    Ok(StockRecommendation {
        symbol: symbol.to_string(),
        verdict,
        score,
        stars,
        current_price: current,
        target_price,
        upside_pct,
        horizon_days: TARGET_HORIZON_DAYS,
        risk_level,
        annualized_vol_pct,
        support,
        resistance,
        forecast_3m_low,
        forecast_3m_high,
        components,
        bars_analyzed: n,
    })
}

/// Annualized volatility (%) from daily close-to-close returns over the
/// full bar window. sqrt(252) scaling. Returns 0 for degenerate input.
fn annualized_volatility_pct(closes: &[f64]) -> f64 {
    if closes.len() < 2 {
        return 0.0;
    }
    let mut rets: Vec<f64> = Vec::with_capacity(closes.len() - 1);
    for w in closes.windows(2) {
        if w[0].abs() > 1e-9 {
            rets.push((w[1] - w[0]) / w[0]);
        }
    }
    if rets.len() < 2 {
        return 0.0;
    }
    let mean = rets.iter().sum::<f64>() / rets.len() as f64;
    let var = rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (rets.len() - 1) as f64;
    var.sqrt() * (252.0_f64).sqrt() * 100.0
}

fn risk_from_vol(vol_pct: f64) -> RiskLevel {
    if vol_pct < 25.0 {
        RiskLevel::Low
    } else if vol_pct <= 50.0 {
        RiskLevel::Medium
    } else {
        RiskLevel::High
    }
}

/// Swing-level S/R from the last `SR_WINDOW` bars. A swing high is a
/// bar whose high exceeds both neighbours' highs (3-bar pivot); swing
/// low symmetric. Support = HIGHEST swing low strictly below current
/// price; resistance = LOWEST swing high strictly above. The nearest
/// meaningful levels, not the extremes.
fn swing_levels(highs: &[f64], lows: &[f64], current: f64) -> (Option<Decimal>, Option<Decimal>) {
    const SR_WINDOW: usize = 60;
    let n = highs.len();
    if n < 3 || current <= 0.0 {
        return (None, None);
    }
    let start = n.saturating_sub(SR_WINDOW).max(1);
    let mut best_support: Option<f64> = None;
    let mut best_resist: Option<f64> = None;
    for i in start..(n - 1) {
        // 3-bar pivot high.
        if highs[i] > highs[i - 1] && highs[i] > highs[i + 1] && highs[i] > current {
            best_resist = Some(match best_resist {
                Some(r) => r.min(highs[i]),
                None => highs[i],
            });
        }
        // 3-bar pivot low.
        if lows[i] < lows[i - 1] && lows[i] < lows[i + 1] && lows[i] < current {
            best_support = Some(match best_support {
                Some(s) => s.max(lows[i]),
                None => lows[i],
            });
        }
    }
    (
        best_support.and_then(Decimal::from_f64),
        best_resist.and_then(Decimal::from_f64),
    )
}

/// 3-month (66 trading-day) forecast band. Center = score-biased drift
/// extended from the 30-day swing to the longer horizon; half-width =
/// 1σ of horizon volatility. Low is floored at a cent.
fn forecast_band(current: f64, score: f64, annualized_vol_pct: f64) -> (Decimal, Decimal) {
    const HORIZON_TRADING_DAYS: f64 = 66.0;
    let bias = ((score - 50.0) / 50.0).clamp(-1.0, 1.0);
    // 30-day swing is TARGET_SWING_PCT; scale drift linearly with time.
    let drift = bias * TARGET_SWING_PCT * (HORIZON_TRADING_DAYS / 22.0);
    let center = current * (1.0 + drift);
    let sigma = annualized_vol_pct / 100.0 * (HORIZON_TRADING_DAYS / 252.0_f64).sqrt();
    let low = (center * (1.0 - sigma)).max(0.01);
    let high = center * (1.0 + sigma);
    (
        Decimal::from_f64(low).unwrap_or_default(),
        Decimal::from_f64(high).unwrap_or_default(),
    )
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

// ===========================================================================
// Persistence: stock_recommendations table
// ===========================================================================

/// One row in `stock_recommendations`. Mirrors `StockRecommendation` but
/// carries the database id + the computed_at timestamp. Used by the
/// leaderboard, the backtest tooling, and the verdict-change alerter.
#[derive(Debug, Clone, Serialize)]
pub struct StoredRecommendation {
    pub id: Uuid,
    pub symbol: String,
    pub computed_at: DateTime<Utc>,
    pub verdict: String,
    pub score: f64,
    pub stars: i16,
    pub current_price: Decimal,
    pub target_price: Decimal,
    pub upside_pct: f64,
    pub horizon_days: i32,
    pub bars_analyzed: i32,
    /// "low" | "medium" | "high" — NULL for rows written before
    /// migration 0074; the next nightly compute backfills.
    pub risk_level: Option<String>,
    pub annualized_vol_pct: f64,
    pub components: serde_json::Value,
}

fn risk_level_str(r: RiskLevel) -> &'static str {
    match r {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
    }
}

/// Persist a freshly-computed recommendation. The cron uses this after
/// every per-symbol `compute()` call. Each call inserts a new row —
/// keeping history (so the alerter can diff and the backtester can
/// replay) at the cost of unbounded growth, which a future cron
/// retention job can prune.
pub async fn save(
    pool: &PgPool,
    r: &StockRecommendation,
) -> anyhow::Result<StoredRecommendation> {
    let components = serde_json::to_value(&r.components).unwrap_or(serde_json::json!([]));
    let score_dec = Decimal::from_f64(r.score).unwrap_or_default();
    let upside_dec = Decimal::from_f64(r.upside_pct).unwrap_or_default();
    let row: (
        Uuid,
        String,
        DateTime<Utc>,
        String,
        Decimal,
        i16,
        Decimal,
        Decimal,
        Decimal,
        i32,
        i32,
        serde_json::Value,
    ) = sqlx::query_as(
        "INSERT INTO stock_recommendations
            (symbol, verdict, score, stars, current_price, target_price,
             upside_pct, horizon_days, bars_analyzed, components,
             risk_level, annualized_vol_pct, support, resistance,
             forecast_3m_low, forecast_3m_high)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                 $11, $12, $13, $14, $15, $16)
         RETURNING id, symbol, computed_at, verdict, score, stars,
                   current_price, target_price, upside_pct, horizon_days,
                   bars_analyzed, components",
    )
    .bind(&r.symbol)
    .bind(r.verdict.as_str())
    .bind(score_dec)
    .bind(r.stars as i16)
    .bind(r.current_price)
    .bind(r.target_price)
    .bind(upside_dec)
    .bind(r.horizon_days as i32)
    .bind(r.bars_analyzed as i32)
    .bind(components)
    .bind(risk_level_str(r.risk_level))
    .bind(Decimal::from_f64(r.annualized_vol_pct).unwrap_or_default())
    .bind(r.support)
    .bind(r.resistance)
    .bind(r.forecast_3m_low)
    .bind(r.forecast_3m_high)
    .fetch_one(pool)
    .await?;
    let (
        id,
        symbol,
        computed_at,
        verdict,
        score,
        stars,
        current_price,
        target_price,
        upside_pct,
        horizon_days,
        bars_analyzed,
        components,
    ) = row;
    Ok(StoredRecommendation {
        id,
        symbol,
        computed_at,
        verdict,
        score: decimal_to_f64(score),
        stars,
        current_price,
        target_price,
        upside_pct: decimal_to_f64(upside_pct),
        horizon_days,
        bars_analyzed,
        risk_level: Some(risk_level_str(r.risk_level).to_string()),
        annualized_vol_pct: r.annualized_vol_pct,
        components,
    })
}

/// Most-recent recommendation per symbol, ordered by score descending.
/// Backs the Golden Stars leaderboard. The `min_verdict` filter lets
/// callers say "buy candidates only" so the view doesn't show the
/// strong_sell list alongside the buys.
pub async fn leaderboard(
    pool: &PgPool,
    limit: i64,
    min_score: Option<f64>,
) -> anyhow::Result<Vec<StoredRecommendation>> {
    let min = min_score.unwrap_or(0.0);
    let min_dec = Decimal::from_f64(min).unwrap_or_default();
    let rows: Vec<(
        Uuid,
        String,
        DateTime<Utc>,
        String,
        Decimal,
        i16,
        Decimal,
        Decimal,
        Decimal,
        i32,
        i32,
        Option<String>,
        Decimal,
        serde_json::Value,
    )> = sqlx::query_as(
        "SELECT DISTINCT ON (symbol)
                id, symbol, computed_at, verdict, score, stars,
                current_price, target_price, upside_pct, horizon_days,
                bars_analyzed, risk_level, annualized_vol_pct, components
           FROM stock_recommendations
          WHERE score >= $1
          ORDER BY symbol, computed_at DESC",
    )
    .bind(min_dec)
    .fetch_all(pool)
    .await?;
    // Re-sort by score descending in app code (DISTINCT ON forces a
    // symbol-keyed sort; doing the score sort in SQL would need a CTE).
    let mut out: Vec<StoredRecommendation> = rows
        .into_iter()
        .map(
            |(
                id,
                symbol,
                computed_at,
                verdict,
                score,
                stars,
                current_price,
                target_price,
                upside_pct,
                horizon_days,
                bars_analyzed,
                risk_level,
                annualized_vol_pct,
                components,
            )| StoredRecommendation {
                id,
                symbol,
                computed_at,
                verdict,
                score: decimal_to_f64(score),
                stars,
                current_price,
                target_price,
                upside_pct: decimal_to_f64(upside_pct),
                horizon_days,
                bars_analyzed,
                risk_level,
                annualized_vol_pct: decimal_to_f64(annualized_vol_pct),
                components,
            },
        )
        .collect();
    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    out.truncate(limit.max(1) as usize);
    Ok(out)
}

/// Latest single recommendation for one symbol (or None if never
/// computed). Used by the verdict-change alerter to find the prior
/// verdict before comparing against the new compute.
pub async fn latest_for_symbol(
    pool: &PgPool,
    symbol: &str,
) -> anyhow::Result<Option<StoredRecommendation>> {
    let row: Option<(
        Uuid,
        String,
        DateTime<Utc>,
        String,
        Decimal,
        i16,
        Decimal,
        Decimal,
        Decimal,
        i32,
        i32,
        Option<String>,
        Decimal,
        serde_json::Value,
    )> = sqlx::query_as(
        "SELECT id, symbol, computed_at, verdict, score, stars,
                current_price, target_price, upside_pct, horizon_days,
                bars_analyzed, risk_level, annualized_vol_pct, components
           FROM stock_recommendations
          WHERE symbol = $1
          ORDER BY computed_at DESC
          LIMIT 1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(
        |(
            id,
            symbol,
            computed_at,
            verdict,
            score,
            stars,
            current_price,
            target_price,
            upside_pct,
            horizon_days,
            bars_analyzed,
            risk_level,
            annualized_vol_pct,
            components,
        )| StoredRecommendation {
            id,
            symbol,
            computed_at,
            verdict,
            score: decimal_to_f64(score),
            stars,
            current_price,
            target_price,
            upside_pct: decimal_to_f64(upside_pct),
            horizon_days,
            bars_analyzed,
            risk_level,
            annualized_vol_pct: decimal_to_f64(annualized_vol_pct),
            components,
        },
    ))
}

// ===========================================================================
// Universes
// ===========================================================================

/// The 11 SPDR sector ETFs. Used by the sector-aggregation view and by
/// the default Golden Stars universe when the user hasn't bound the
/// symbols catalog.
pub const SECTOR_ETFS: &[(&str, &str)] = &[
    ("XLK", "Technology"),
    ("XLF", "Financials"),
    ("XLE", "Energy"),
    ("XLV", "Health Care"),
    ("XLI", "Industrials"),
    ("XLY", "Consumer Discretionary"),
    ("XLP", "Consumer Staples"),
    ("XLU", "Utilities"),
    ("XLRE", "Real Estate"),
    ("XLB", "Materials"),
    ("XLC", "Communication Services"),
];

/// Default Golden Stars universe — anchors of major indexes + each
/// sector ETF + the highest-liquidity names from each sector. Kept
/// hardcoded so the leaderboard ships without depending on a
/// user-curated watchlist. The cron iterates this; if a name fails
/// (no bars), the cron logs + continues.
pub const DEFAULT_UNIVERSE: &[&str] = &[
    // Index / leadership
    "SPY", "QQQ", "DIA", "IWM",
    // Sector SPDRs
    "XLK", "XLF", "XLE", "XLV", "XLI", "XLY", "XLP", "XLU", "XLRE", "XLB", "XLC",
    // Mega-cap tech
    "AAPL", "MSFT", "GOOGL", "AMZN", "META", "NVDA", "TSLA", "AVGO", "ORCL", "CRM",
    "ADBE", "AMD", "INTC", "QCOM", "TXN", "MU", "PYPL", "NFLX",
    // Financials
    "JPM", "BAC", "WFC", "GS", "MS", "C", "V", "MA", "AXP", "BLK",
    // Energy
    "XOM", "CVX", "COP", "SLB", "EOG",
    // Health care
    "UNH", "JNJ", "LLY", "PFE", "ABBV", "MRK", "TMO", "ABT",
    // Consumer
    "WMT", "HD", "COST", "MCD", "NKE", "SBUX", "TGT", "LOW",
    // Industrials / defense
    "BA", "CAT", "GE", "LMT", "RTX", "HON", "UPS", "FDX",
    // Communications
    "DIS", "CMCSA", "T", "VZ",
];

// ===========================================================================
// Cron: compute + persist for a universe
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct CronResult {
    pub attempted: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub failures: Vec<(String, String)>,
}

/// Run `compute()` for every symbol in `universe`, persist each
/// successful result, and report a per-symbol roll-up. Failures
/// (insufficient history, price-fetch errors) are logged + skipped so
/// the cron doesn't abort mid-batch. Returns the universe size and
/// success/failure counts.
pub async fn cron_compute_universe(
    pool: &PgPool,
    universe: &[&str],
) -> CronResult {
    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut failures: Vec<(String, String)> = Vec::new();
    for sym in universe {
        match compute(pool, sym).await {
            Ok(r) => match save(pool, &r).await {
                Ok(_) => {
                    succeeded += 1;
                }
                Err(e) => {
                    failed += 1;
                    failures.push((sym.to_string(), format!("save: {e}")));
                    tracing::warn!(symbol = %sym, error = %e, "stock_rec: save failed");
                }
            },
            Err(e) => {
                failed += 1;
                failures.push((sym.to_string(), format!("compute: {e}")));
                tracing::debug!(symbol = %sym, error = %e, "stock_rec: compute failed");
            }
        }
    }
    CronResult {
        attempted: universe.len(),
        succeeded,
        failed,
        failures,
    }
}

// ===========================================================================
// Historical accuracy backtest
// ===========================================================================

/// One historical "what would the algorithm have said" event.
#[derive(Debug, Clone, Serialize)]
pub struct BacktestSignal {
    pub at: DateTime<Utc>,
    pub verdict: Verdict,
    pub score: f64,
    pub close: f64,
    /// Forward 30-bar return in percent (None if the bar window doesn't
    /// extend `horizon_bars` past `at`).
    pub forward_return_pct: Option<f64>,
    /// Was the verdict's directional bias correct? Buy/StrongBuy require
    /// forward_return_pct > 0; Sell/StrongSell require < 0; Hold is
    /// neutral (always None).
    pub correct: Option<bool>,
}

/// Per-verdict roll-up: hit rate + sample count + average forward
/// return. Renders alongside the recommendation panel so the user can
/// see "strong_buy hit 72% over 31 prior fires" before acting on
/// today's signal.
#[derive(Debug, Clone, Serialize, Default)]
pub struct BacktestVerdictStats {
    pub verdict: Option<String>,
    pub sample_count: usize,
    pub hit_count: usize,
    pub hit_rate_pct: f64,
    pub avg_forward_return_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BacktestReport {
    pub symbol: String,
    pub bars_used: usize,
    pub horizon_bars: usize,
    /// One stats row per verdict bucket that had ≥1 sample.
    pub by_verdict: Vec<BacktestVerdictStats>,
    /// Most recent 50 signals so the UI can plot them along the chart.
    pub recent_signals: Vec<BacktestSignal>,
}

/// Replay the algorithm across every day in the bar window. For each
/// day with ≥`MIN_BARS` of trailing data, run the same component blend
/// as `compute()` then observe the return `horizon_bars` ahead. Returns
/// per-verdict hit rates so the panel can show "buys correct 67% over
/// 31 prior fires."
///
/// `horizon_bars` defaults to 22 (≈1 trading month) when None — same
/// horizon the `target_price` heuristic projects to.
pub async fn backtest(
    pool: &PgPool,
    symbol: &str,
    horizon_bars: Option<usize>,
) -> Result<BacktestReport, RecommendationError> {
    let horizon = horizon_bars.unwrap_or(22);
    // Lookback enough days to fit MIN_BARS + horizon + a year of replays.
    let to = Utc::now();
    let from = to - Duration::days(LOOKBACK_DAYS * 3 + horizon as i64 * 2);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .map_err(RecommendationError::PriceFetch)?;
    let n = bars.len();
    if n < MIN_BARS + horizon {
        return Err(RecommendationError::Insufficient {
            symbol: symbol.to_string(),
            got: n,
            need: MIN_BARS + horizon,
        });
    }
    let closes_all = indicators::closes(&bars);
    let highs_all = indicators::highs(&bars);
    let lows_all = indicators::lows(&bars);
    let volumes_all = indicators::volumes(&bars);
    let mut signals: Vec<BacktestSignal> = Vec::new();
    for i in MIN_BARS..(n - horizon) {
        let closes = &closes_all[..=i];
        let highs = &highs_all[..=i];
        let lows = &lows_all[..=i];
        let volumes = &volumes_all[..=i];
        let components = vec![
            score_trend(closes),
            score_momentum(closes),
            score_macd(closes),
            score_rsi(closes),
            score_adx(highs, lows, closes),
            score_volume(volumes, closes),
        ];
        let weighted: f64 = components.iter().map(|c| c.weight * c.score).sum();
        let total_weight: f64 = components.iter().map(|c| c.weight).sum();
        let score = if total_weight > 0.0 {
            weighted / total_weight
        } else {
            50.0
        };
        let verdict = Verdict::from_score(score);
        let close_now = closes[i];
        let close_future = closes_all[i + horizon];
        let forward_return_pct = if close_now.abs() > 1e-9 {
            Some((close_future - close_now) / close_now * 100.0)
        } else {
            None
        };
        let correct = match verdict {
            Verdict::StrongBuy | Verdict::Buy => forward_return_pct.map(|r| r > 0.0),
            Verdict::Sell | Verdict::StrongSell => forward_return_pct.map(|r| r < 0.0),
            Verdict::Hold => None,
        };
        signals.push(BacktestSignal {
            at: bars[i].bar_time,
            verdict,
            score,
            close: close_now,
            forward_return_pct,
            correct,
        });
    }
    // Per-verdict roll-up.
    let mut by_verdict_map: std::collections::HashMap<&'static str, (usize, usize, f64)> =
        std::collections::HashMap::new();
    for s in &signals {
        let key = s.verdict.as_str();
        let entry = by_verdict_map.entry(key).or_insert((0, 0, 0.0));
        entry.0 += 1;
        if matches!(s.correct, Some(true)) {
            entry.1 += 1;
        }
        if let Some(r) = s.forward_return_pct {
            entry.2 += r;
        }
    }
    let mut by_verdict: Vec<BacktestVerdictStats> = by_verdict_map
        .into_iter()
        .map(|(verdict, (count, hits, sum_ret))| BacktestVerdictStats {
            verdict: Some(verdict.to_string()),
            sample_count: count,
            hit_count: hits,
            hit_rate_pct: if count > 0 {
                hits as f64 / count as f64 * 100.0
            } else {
                0.0
            },
            avg_forward_return_pct: if count > 0 { sum_ret / count as f64 } else { 0.0 },
        })
        .collect();
    by_verdict.sort_by(|a, b| {
        b.hit_rate_pct
            .partial_cmp(&a.hit_rate_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    // Keep the last 50 signals — the UI plots them as small dots over
    // the price chart so the user sees where the algorithm flipped.
    let recent_signals = signals
        .iter()
        .rev()
        .take(50)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    Ok(BacktestReport {
        symbol: symbol.to_string(),
        bars_used: n,
        horizon_bars: horizon,
        by_verdict,
        recent_signals,
    })
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
    fn risk_levels_at_boundaries() {
        assert_eq!(risk_from_vol(10.0), RiskLevel::Low);
        assert_eq!(risk_from_vol(24.99), RiskLevel::Low);
        assert_eq!(risk_from_vol(25.0), RiskLevel::Medium);
        assert_eq!(risk_from_vol(50.0), RiskLevel::Medium);
        assert_eq!(risk_from_vol(50.01), RiskLevel::High);
        assert_eq!(risk_from_vol(120.0), RiskLevel::High);
    }

    #[test]
    fn annualized_vol_zero_for_flat_series() {
        let closes = vec![100.0; 50];
        assert!((annualized_volatility_pct(&closes) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn annualized_vol_positive_for_noisy_series() {
        // Alternate ±1% daily — definitely positive volatility.
        let mut closes = vec![100.0];
        for i in 1..100 {
            let prev: f64 = *closes.last().unwrap();
            closes.push(if i % 2 == 0 { prev * 1.01 } else { prev * 0.99 });
        }
        let v = annualized_volatility_pct(&closes);
        assert!(v > 5.0, "expected meaningful vol, got {v}");
    }

    #[test]
    fn swing_levels_find_nearest_pivots() {
        // Build a series with a clear pivot low at 90 and pivot high
        // at 110 around a current price of 100.
        let mut highs = vec![100.0; 20];
        let mut lows = vec![95.0; 20];
        highs[10] = 110.0; // pivot high (neighbors are 100)
        lows[5] = 90.0; // pivot low (neighbors are 95)
        let (support, resistance) = swing_levels(&highs, &lows, 100.0);
        let s = support.map(decimal_to_f64).unwrap_or(0.0);
        let r = resistance.map(decimal_to_f64).unwrap_or(0.0);
        assert!((s - 90.0).abs() < 1e-9, "support {s}");
        assert!((r - 110.0).abs() < 1e-9, "resistance {r}");
    }

    #[test]
    fn forecast_band_straddles_drift_center() {
        // Neutral score (50) → no drift; band must straddle current.
        let (lo, hi) = forecast_band(100.0, 50.0, 30.0);
        let lo_f = decimal_to_f64(lo);
        let hi_f = decimal_to_f64(hi);
        assert!(lo_f < 100.0 && hi_f > 100.0, "band [{lo_f}, {hi_f}]");
        // Bullish score → center above current.
        let (lo_b, hi_b) = forecast_band(100.0, 100.0, 30.0);
        let mid_b = (decimal_to_f64(lo_b) + decimal_to_f64(hi_b)) / 2.0;
        assert!(mid_b > 100.0, "bullish mid {mid_b}");
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
