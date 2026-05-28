//! Fear & Greed gauge — 7 components averaged into a 0..100 score.
//! Mirrors the CNN methodology using free data we already have plumbing for.

use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::indicators::sma;
use traderview_core::BarInterval;

#[derive(Debug, Clone, Serialize)]
pub struct Component {
    pub key: &'static str,
    pub label: &'static str,
    pub score: i32, // 0..100 (0 = extreme fear, 100 = extreme greed)
    pub interpretation: String,
    pub raw: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FearGreed {
    pub score: i32,
    pub label: &'static str, // "Extreme Fear" .. "Extreme Greed"
    pub components: Vec<Component>,
    pub fetched_at: chrono::DateTime<Utc>,
}

pub async fn snapshot(pool: &PgPool) -> anyhow::Result<FearGreed> {
    let momentum = market_momentum(pool).await;
    let strength = price_strength(pool).await;
    let breadth = price_breadth(pool).await;
    let pcr = put_call(pool).await;
    let junk = junk_demand(pool).await;
    let vix = vix_vs_ma(pool).await;
    let safe = safe_haven(pool).await;

    let components = vec![momentum, strength, breadth, pcr, junk, vix, safe];
    let n = components.len() as i32;
    let sum: i32 = components.iter().map(|c| c.score).sum();
    let avg = sum / n;
    let label = label_for(avg);
    Ok(FearGreed {
        score: avg,
        label,
        components,
        fetched_at: Utc::now(),
    })
}

fn label_for(s: i32) -> &'static str {
    match s {
        s if s <= 24 => "Extreme Fear",
        s if s <= 44 => "Fear",
        s if s <= 55 => "Neutral",
        s if s <= 74 => "Greed",
        _ => "Extreme Greed",
    }
}

fn clamp_score(v: f64) -> i32 {
    v.clamp(0.0, 100.0) as i32
}

// ---- 1. Market momentum: SPY vs 125-day SMA -----------------------------

async fn market_momentum(pool: &PgPool) -> Component {
    let to = Utc::now();
    let from = to - Duration::days(200);
    let mut score = 50;
    let mut raw = None;
    let mut interp = "no data".to_string();
    if let Ok(bars) = crate::prices::get_bars(pool, "SPY", BarInterval::D1, from, to).await {
        let closes: Vec<f64> = bars.iter().map(|b| dec(b.close)).collect();
        if closes.len() >= 125 {
            let ma = sma(&closes, 125).last().and_then(|x| *x).unwrap_or(0.0);
            let last = *closes.last().unwrap();
            if ma > 0.0 {
                let pct = (last - ma) / ma * 100.0;
                raw = Some(pct);
                // ±10% maps to 0..100, centered at 50.
                score = clamp_score(50.0 + pct * 5.0);
                interp = format!(
                    "SPY {} 125-d SMA by {:.2}%",
                    if pct >= 0.0 { "above" } else { "below" },
                    pct.abs()
                );
            }
        }
    }
    Component {
        key: "momentum",
        label: "Market momentum (SPY vs 125-d SMA)",
        score,
        interpretation: interp,
        raw,
    }
}

// ---- 2. Price strength: % of watchlist symbols within 5% of 52-w high vs low -

async fn price_strength(pool: &PgPool) -> Component {
    let to = Utc::now();
    let from = to - Duration::days(370);
    let universe: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT symbol FROM watchlist_symbols LIMIT 50")
            .fetch_all(pool)
            .await
            .unwrap_or_default();
    let mut near_high = 0usize;
    let mut near_low = 0usize;
    let mut total = 0usize;
    for sym in &universe {
        if let Ok(bars) = crate::prices::get_bars(pool, sym, BarInterval::D1, from, to).await {
            if let Some(h) = traderview_core::scan::stats_for(sym, &bars) {
                total += 1;
                if h.year_high_pct >= -5.0 {
                    near_high += 1;
                }
                if h.year_low_pct <= 5.0 {
                    near_low += 1;
                }
            }
        }
    }
    let (score, interp) = if total == 0 {
        (50, "no watchlist symbols cached".to_string())
    } else {
        let net = (near_high as f64 - near_low as f64) / total as f64; // -1..+1
        (
            clamp_score(50.0 + net * 100.0),
            format!(
                "{}/{} near 52w high, {}/{} near 52w low",
                near_high, total, near_low, total
            ),
        )
    };
    Component {
        key: "strength",
        label: "Price strength (52w-hi vs 52w-lo)",
        score,
        interpretation: interp,
        raw: None,
    }
}

// ---- 3. Breadth: NYSE Advance−Decline normalized ----------------------------

async fn price_breadth(pool: &PgPool) -> Component {
    let mut score = 50;
    let mut raw = None;
    let mut interp = "no data".to_string();
    if let Ok(q) = crate::market_data::quote(pool, "^ADD").await {
        raw = Some(q.price);
        // ±2000 maps to 0..100.
        score = clamp_score(50.0 + q.price / 40.0);
        interp = format!("net A-D = {:.0}", q.price);
    }
    Component {
        key: "breadth",
        label: "Stock price breadth (A−D)",
        score,
        interpretation: interp,
        raw,
    }
}

// ---- 4. Put/call ratio (lower = greed, contrarian at extremes) -------------

async fn put_call(pool: &PgPool) -> Component {
    let mut score = 50;
    let mut raw = None;
    let mut interp = "no data".to_string();
    if let Ok(q) = crate::market_data::quote(pool, "^CPC").await {
        raw = Some(q.price);
        // 0.6 → 100 (greed), 1.2 → 0 (fear), linear between.
        let v = q.price;
        let s = ((1.2 - v) / (1.2 - 0.6)) * 100.0;
        score = clamp_score(s);
        interp = format!("PCR = {:.2}", v);
    }
    Component {
        key: "pcr",
        label: "Put-Call ratio (inverted)",
        score,
        interpretation: interp,
        raw,
    }
}

// ---- 5. Junk bond demand: HYG-LQD 20-day relative return -------------------

async fn junk_demand(pool: &PgPool) -> Component {
    let to = Utc::now();
    let from = to - Duration::days(60);
    let hyg = ret20(pool, "HYG", from, to).await;
    let lqd = ret20(pool, "LQD", from, to).await;
    let mut score = 50;
    let mut raw = None;
    let mut interp = "no data".to_string();
    if let (Some(h), Some(l)) = (hyg, lqd) {
        let spread = h - l;
        raw = Some(spread);
        // ±3% spread maps to 0..100, centered.
        score = clamp_score(50.0 + spread * 16.7);
        interp = format!("HYG−LQD 20d return = {:+.2}%", spread);
    }
    Component {
        key: "junk",
        label: "Junk-bond demand (HYG−LQD 20d)",
        score,
        interpretation: interp,
        raw,
    }
}

// ---- 6. Volatility: VIX vs 50-d MA (lower-than-MA = greed) -----------------

async fn vix_vs_ma(pool: &PgPool) -> Component {
    let to = Utc::now();
    let from = to - Duration::days(100);
    let mut score = 50;
    let mut raw = None;
    let mut interp = "no data".to_string();
    if let Ok(bars) = crate::prices::get_bars(pool, "^VIX", BarInterval::D1, from, to).await {
        let closes: Vec<f64> = bars.iter().map(|b| dec(b.close)).collect();
        if closes.len() >= 50 {
            let ma = sma(&closes, 50).last().and_then(|x| *x).unwrap_or(0.0);
            let last = *closes.last().unwrap();
            if ma > 0.0 {
                let pct = (last - ma) / ma * 100.0; // positive = VIX above MA = fear
                raw = Some(pct);
                score = clamp_score(50.0 - pct * 2.5);
                interp = format!(
                    "VIX {} 50-d MA by {:.2}%",
                    if pct >= 0.0 { "above" } else { "below" },
                    pct.abs()
                );
            }
        }
    }
    Component {
        key: "vix",
        label: "Volatility (VIX vs 50-d MA, inverted)",
        score,
        interpretation: interp,
        raw,
    }
}

// ---- 7. Safe haven demand: SPY-TLT 20-day return ---------------------------

async fn safe_haven(pool: &PgPool) -> Component {
    let to = Utc::now();
    let from = to - Duration::days(60);
    let spy = ret20(pool, "SPY", from, to).await;
    let tlt = ret20(pool, "TLT", from, to).await;
    let mut score = 50;
    let mut raw = None;
    let mut interp = "no data".to_string();
    if let (Some(s), Some(t)) = (spy, tlt) {
        let spread = s - t;
        raw = Some(spread);
        score = clamp_score(50.0 + spread * 10.0);
        interp = format!("SPY−TLT 20d return = {:+.2}%", spread);
    }
    Component {
        key: "safe_haven",
        label: "Safe-haven demand (SPY−TLT 20d)",
        score,
        interpretation: interp,
        raw,
    }
}

// ---- helpers ---------------------------------------------------------------

async fn ret20(
    pool: &PgPool,
    sym: &str,
    from: chrono::DateTime<Utc>,
    to: chrono::DateTime<Utc>,
) -> Option<f64> {
    let bars = crate::prices::get_bars(pool, sym, BarInterval::D1, from, to)
        .await
        .ok()?;
    if bars.len() < 21 {
        return None;
    }
    let n = bars.len();
    let last = dec(bars[n - 1].close);
    let prior = dec(bars[n - 21].close);
    if prior <= 0.0 {
        return None;
    }
    Some((last - prior) / prior * 100.0)
}

fn dec(d: rust_decimal::Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================================================
    // label_for — score band boundaries
    // ===========================================================================

    #[test]
    fn label_extreme_fear_at_low_end_inclusive_of_24() {
        assert_eq!(label_for(0), "Extreme Fear");
        assert_eq!(label_for(10), "Extreme Fear");
        assert_eq!(label_for(24), "Extreme Fear");
    }

    #[test]
    fn label_fear_band_25_to_44() {
        assert_eq!(label_for(25), "Fear");
        assert_eq!(label_for(35), "Fear");
        assert_eq!(label_for(44), "Fear");
    }

    #[test]
    fn label_neutral_band_45_to_55() {
        assert_eq!(label_for(45), "Neutral");
        assert_eq!(label_for(50), "Neutral");
        assert_eq!(label_for(55), "Neutral");
    }

    #[test]
    fn label_greed_band_56_to_74() {
        assert_eq!(label_for(56), "Greed");
        assert_eq!(label_for(65), "Greed");
        assert_eq!(label_for(74), "Greed");
    }

    #[test]
    fn label_extreme_greed_at_75_and_above() {
        assert_eq!(label_for(75), "Extreme Greed");
        assert_eq!(label_for(100), "Extreme Greed");
        // Sanity: even malformed >100 still classifies as Extreme Greed.
        assert_eq!(label_for(150), "Extreme Greed");
    }

    #[test]
    fn label_negative_score_falls_into_extreme_fear() {
        // Defensive: negative caller bug should land in Extreme Fear, not panic.
        assert_eq!(label_for(-10), "Extreme Fear");
    }

    // ===========================================================================
    // clamp_score — 0..100 range, integer truncation
    // ===========================================================================

    #[test]
    fn clamp_score_clips_below_zero_to_zero() {
        assert_eq!(clamp_score(-50.0), 0);
        assert_eq!(clamp_score(-0.5), 0);
    }

    #[test]
    fn clamp_score_clips_above_100_to_100() {
        assert_eq!(clamp_score(150.0), 100);
        assert_eq!(clamp_score(100.5), 100);
    }

    #[test]
    fn clamp_score_passes_through_in_range_values() {
        assert_eq!(clamp_score(0.0), 0);
        assert_eq!(clamp_score(50.0), 50);
        assert_eq!(clamp_score(100.0), 100);
    }

    #[test]
    fn clamp_score_truncates_fractional_part_toward_zero() {
        // `as i32` truncates toward zero (Rust spec for f64 → i32).
        assert_eq!(clamp_score(49.9), 49);
        assert_eq!(clamp_score(99.99), 99);
    }

    #[test]
    fn clamp_score_nan_clamps_to_zero() {
        // f64::NAN goes through clamp which returns NAN; `as i32` of NAN is 0
        // (Rust spec: saturating cast).
        assert_eq!(clamp_score(f64::NAN), 0);
    }

    #[test]
    fn clamp_score_infinity_clipped_to_100() {
        assert_eq!(clamp_score(f64::INFINITY), 100);
        assert_eq!(clamp_score(f64::NEG_INFINITY), 0);
    }

    // ===========================================================================
    // dec — Decimal → f64
    // ===========================================================================

    #[test]
    fn dec_handles_zero_and_negatives() {
        use rust_decimal::Decimal;
        assert_eq!(dec(Decimal::ZERO), 0.0);
        assert_eq!(dec(Decimal::from(-42)), -42.0);
        assert!((dec(Decimal::new(255, 2)) - 2.55).abs() < 1e-9);
    }
}
