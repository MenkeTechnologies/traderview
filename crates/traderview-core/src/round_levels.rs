//! Round-number support/resistance levels.
//!
//! Psychological S/R clusters around prices ending in 00, 50, 25.
//! "Round-number resistance at $100" is a real phenomenon — order-book
//! liquidity stacks at those levels because every retail trader thinks
//! of them. Used as confluence with technical levels in chart analysis.
//!
//! Given a current price and an ATR-scaled distance, this module
//! emits the round-number levels above and below within a window,
//! grouped by their psychological "weight":
//!
//!   - **Major**: $1000s, $500s, $100s
//!   - **Medium**: $50s, $25s
//!   - **Minor**: $10s, $5s, $1s
//!
//! Pure compute. Caller picks the weights they want surfaced.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LevelWeight { Major, Medium, Minor }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundLevel {
    pub price: f64,
    pub weight: LevelWeight,
    /// Distance from current price in price units (signed: + above, - below).
    pub distance: f64,
    /// Distance as a multiple of the supplied ATR (None if atr was 0/missing).
    pub distance_atrs: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelsConfig {
    /// How many price-units above/below to scan. Caller can scale by
    /// volatility (e.g. 4× ATR for a day-trader, 20× ATR for a swing).
    pub window: f64,
    /// Only emit levels whose weight is at least this strict.
    pub min_weight: LevelWeight,
}

impl Default for LevelsConfig {
    fn default() -> Self { Self { window: 50.0, min_weight: LevelWeight::Minor } }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LevelsReport {
    /// Levels in ascending price order.
    pub levels: Vec<RoundLevel>,
    /// Nearest level above the current price.
    pub nearest_above: Option<RoundLevel>,
    /// Nearest level below the current price.
    pub nearest_below: Option<RoundLevel>,
}

fn classify(price: f64) -> Option<LevelWeight> {
    // Only round-number levels qualify — caller's `detect` walks integer
    // candidates, so a non-integer input means we're not at a level.
    // Largest-divisor checks first so a $1000 price is Major, not Minor.
    if !price.is_finite() { return None; }
    if (price - price.round()).abs() > 1e-9 { return None; }
    let p = price.round();
    if p % 1000.0 == 0.0 || p % 500.0 == 0.0 || p % 100.0 == 0.0 {
        Some(LevelWeight::Major)
    } else if p % 50.0 == 0.0 || p % 25.0 == 0.0 {
        Some(LevelWeight::Medium)
    } else if p % 10.0 == 0.0 || p % 5.0 == 0.0 || p % 1.0 == 0.0 {
        Some(LevelWeight::Minor)
    } else {
        None
    }
}

/// Weight ordering for the `min_weight` filter.
fn weight_rank(w: LevelWeight) -> u8 {
    match w {
        LevelWeight::Major  => 3,
        LevelWeight::Medium => 2,
        LevelWeight::Minor  => 1,
    }
}

pub fn detect(current_price: f64, atr: Option<f64>, cfg: &LevelsConfig) -> LevelsReport {
    if !current_price.is_finite() || current_price <= 0.0 || cfg.window <= 0.0 {
        return LevelsReport::default();
    }
    let min_rank = weight_rank(cfg.min_weight);
    let lo = (current_price - cfg.window).max(0.0).floor() as i64;
    let hi = (current_price + cfg.window).ceil() as i64;
    if hi <= lo { return LevelsReport::default(); }
    // Guard against window-too-wide blow-up.
    if (hi - lo) > 100_000 {
        return LevelsReport::default();
    }
    let mut levels: Vec<RoundLevel> = Vec::new();
    for p in lo..=hi {
        let price = p as f64;
        let Some(w) = classify(price) else { continue };
        if weight_rank(w) < min_rank { continue; }
        let distance = price - current_price;
        if distance.abs() > cfg.window { continue; }
        let distance_atrs = atr.filter(|a| *a > 0.0).map(|a| distance / a);
        levels.push(RoundLevel { price, weight: w, distance, distance_atrs });
    }
    let nearest_above = levels.iter()
        .filter(|l| l.distance > 0.0)
        .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal))
        .cloned();
    let nearest_below = levels.iter()
        .filter(|l| l.distance < 0.0)
        .max_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal))
        .cloned();
    LevelsReport { levels, nearest_above, nearest_below }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_recognizes_each_weight_tier() {
        assert_eq!(classify(1000.0), Some(LevelWeight::Major));
        assert_eq!(classify(500.0),  Some(LevelWeight::Major));
        assert_eq!(classify(100.0),  Some(LevelWeight::Major));
        assert_eq!(classify(50.0),   Some(LevelWeight::Medium));
        assert_eq!(classify(25.0),   Some(LevelWeight::Medium));
        assert_eq!(classify(10.0),   Some(LevelWeight::Minor));
        assert_eq!(classify(5.0),    Some(LevelWeight::Minor));
        assert_eq!(classify(7.0),    Some(LevelWeight::Minor));    // mod 1
        // Non-integer prices return None.
        assert_eq!(classify(100.5),  None);
    }

    #[test]
    fn near_50_emits_levels_above_and_below() {
        // current=48.0, window=5 — should pick up 45 (Minor) and 50 (Medium).
        let cfg = LevelsConfig { window: 5.0, min_weight: LevelWeight::Minor };
        let r = detect(48.0, None, &cfg);
        let has_45 = r.levels.iter().any(|l| (l.price - 45.0).abs() < 1e-9);
        let has_50 = r.levels.iter().any(|l| (l.price - 50.0).abs() < 1e-9);
        assert!(has_45 && has_50,
            "expected 45 and 50 in window, got {:?}",
            r.levels.iter().map(|l| l.price).collect::<Vec<_>>());
        // Nearest below should be 47 or 46 (or 45 — any minor).
        assert!(r.nearest_below.is_some());
        assert!(r.nearest_above.is_some());
    }

    #[test]
    fn min_weight_major_filters_out_minor_and_medium() {
        let cfg = LevelsConfig { window: 100.0, min_weight: LevelWeight::Major };
        let r = detect(125.0, None, &cfg);
        // Only $100 and $200 (no $25/$50/$75 medium, no random integers minor).
        for l in &r.levels {
            assert!(matches!(l.weight, LevelWeight::Major),
                "minor/medium leaked through: {l:?}");
        }
        // $100 should be present (it's in window).
        assert!(r.levels.iter().any(|l| (l.price - 100.0).abs() < 1e-9));
    }

    #[test]
    fn atr_scaling_populates_distance_atrs() {
        let cfg = LevelsConfig { window: 5.0, min_weight: LevelWeight::Major };
        let r = detect(101.0, Some(1.0), &cfg);
        // The 100 level is 1.0 away, ATR 1.0 → 1.0 ATR.
        let l100 = r.levels.iter().find(|l| (l.price - 100.0).abs() < 1e-9).unwrap();
        assert!((l100.distance_atrs.unwrap() - (-1.0)).abs() < 1e-9);
    }

    #[test]
    fn invalid_inputs_return_empty() {
        assert!(detect(-1.0, None, &LevelsConfig::default()).levels.is_empty());
        assert!(detect(f64::NAN, None, &LevelsConfig::default()).levels.is_empty());
        // Window zero → empty.
        let cfg = LevelsConfig { window: 0.0, min_weight: LevelWeight::Minor };
        assert!(detect(100.0, None, &cfg).levels.is_empty());
    }

    #[test]
    fn enormous_window_doesnt_blow_up_memory() {
        // 200_000 integers > 100_000 cap → returns empty rather than allocating.
        let cfg = LevelsConfig { window: 100_001.0, min_weight: LevelWeight::Minor };
        let r = detect(50_000.0, None, &cfg);
        assert!(r.levels.is_empty(), "huge window must short-circuit");
    }
}
