//! Wyckoff phase classifier — accumulation / markup / distribution / markdown.
//!
//! Richard Wyckoff's market-cycle model: smart money accumulates in a
//! sideways range at the lows, marks the price up, distributes at the
//! highs, then marks down. The four phases are identified by two
//! features over a lookback window:
//!
//!   1. **Trend direction** — linear slope of the price series.
//!   2. **Range tightness** — range as a fraction of the mean, indicating
//!      consolidation vs. expansion.
//!
//! Phase classification:
//!   - **Accumulation**: flat slope + tight range AT or near multi-bar low.
//!   - **Markup**: positive slope above the recent base.
//!   - **Distribution**: flat slope + tight range at multi-bar high.
//!   - **Markdown**: negative slope below the recent top.
//!
//! Pure compute. Heuristic — Wyckoff phase detection is fundamentally
//! visual, but this approximation catches the most common transitions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WyckoffConfig {
    pub lookback: usize,
    /// Absolute slope below this fraction of mean price → "flat".
    pub flat_slope_pct: f64,
    /// Range / mean below this → "tight" (in consolidation).
    pub tight_range_pct: f64,
}

impl Default for WyckoffConfig {
    fn default() -> Self {
        Self { lookback: 40, flat_slope_pct: 0.001, tight_range_pct: 0.05 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WyckoffPhase {
    Accumulation,
    Markup,
    Distribution,
    Markdown,
    #[default]
    Indeterminate,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WyckoffReport {
    pub phase: WyckoffPhase,
    pub slope_pct: f64,
    pub range_pct: f64,
    pub price_position_in_range: f64,
    pub note: String,
}

pub fn classify(closes: &[f64], cfg: &WyckoffConfig) -> WyckoffReport {
    let n = closes.len();
    // Floor the lookback at 3 — same minimum used by the guard below.
    // Without this binding the early-exit checks against `lookback.max(3)`
    // but then slices with raw `cfg.lookback`, so `lookback == 0` panics
    // at `slice.last().expect("non-empty slice")` on any n ≥ 3.
    let lookback = cfg.lookback.max(3);
    if n < lookback {
        return WyckoffReport {
            note: format!("need ≥ {lookback} closes, got {n}"),
            ..Default::default()
        };
    }
    let slice = &closes[n - lookback..];
    let mean: f64 = slice.iter().sum::<f64>() / lookback as f64;
    if mean <= 0.0 {
        return WyckoffReport { note: "non-positive mean price".into(), ..Default::default() };
    }
    // Linear-regression slope of price vs index.
    let mean_x = (lookback as f64 - 1.0) / 2.0;
    let (mut num, mut den) = (0.0_f64, 0.0_f64);
    for (i, &p) in slice.iter().enumerate() {
        let dx = i as f64 - mean_x;
        num += dx * (p - mean);
        den += dx * dx;
    }
    let slope = if den > 0.0 { num / den } else { 0.0 };
    let slope_pct = slope / mean;
    let high = slice.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let low  = slice.iter().copied().fold(f64::INFINITY, f64::min);
    let range_pct = if mean > 0.0 { (high - low) / mean } else { 0.0 };
    let latest = *slice.last().expect("non-empty slice");
    let position = if (high - low) > 0.0 { (latest - low) / (high - low) } else { 0.5 };

    let flat = slope_pct.abs() < cfg.flat_slope_pct;
    let tight = range_pct < cfg.tight_range_pct;
    // "Near low" = bottom 30% of the range; "near high" = top 30%.
    let near_low  = position < 0.30;
    let near_high = position > 0.70;
    let phase = if flat && tight && near_low {
        WyckoffPhase::Accumulation
    } else if flat && tight && near_high {
        WyckoffPhase::Distribution
    } else if slope_pct >= cfg.flat_slope_pct {
        WyckoffPhase::Markup
    } else if slope_pct <= -cfg.flat_slope_pct {
        WyckoffPhase::Markdown
    } else {
        WyckoffPhase::Indeterminate
    };
    let note = match phase {
        WyckoffPhase::Accumulation  => format!("flat tight range near low (position {:.0}%) — smart money likely buying", position * 100.0),
        WyckoffPhase::Markup        => format!("positive slope {:.3}%/bar — trend up", slope_pct * 100.0),
        WyckoffPhase::Distribution  => format!("flat tight range near high (position {:.0}%) — smart money likely selling", position * 100.0),
        WyckoffPhase::Markdown      => format!("negative slope {:.3}%/bar — trend down", slope_pct * 100.0),
        WyckoffPhase::Indeterminate => "no clear phase signal".into(),
    };
    WyckoffReport {
        phase,
        slope_pct,
        range_pct,
        price_position_in_range: position,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_few_closes_returns_default_with_note() {
        let r = classify(&[1.0, 2.0], &WyckoffConfig::default());
        assert!(matches!(r.phase, WyckoffPhase::Indeterminate));
        assert!(r.note.contains("need"));
    }

    #[test]
    fn monotonic_uptrend_is_markup() {
        let v: Vec<f64> = (1..=50).map(|i| 100.0 + i as f64 * 0.5).collect();
        let r = classify(&v, &WyckoffConfig::default());
        assert!(matches!(r.phase, WyckoffPhase::Markup),
            "expected Markup, got {:?}", r.phase);
        assert!(r.slope_pct > 0.0);
    }

    #[test]
    fn monotonic_downtrend_is_markdown() {
        let v: Vec<f64> = (1..=50).map(|i| 100.0 - i as f64 * 0.3).collect();
        let r = classify(&v, &WyckoffConfig::default());
        assert!(matches!(r.phase, WyckoffPhase::Markdown));
    }

    #[test]
    fn tight_flat_near_low_is_accumulation() {
        // 40 bars between 100 and 100.5 (tight range), latest at 100.05 (near low).
        let mut v: Vec<f64> = (0..40).map(|i| {
            if i % 2 == 0 { 100.0 } else { 100.5 }
        }).collect();
        v[39] = 100.05;
        let r = classify(&v, &WyckoffConfig::default());
        assert!(matches!(r.phase, WyckoffPhase::Accumulation),
            "expected Accumulation, got {:?} pos={:.2} range={:.4}",
            r.phase, r.price_position_in_range, r.range_pct);
    }

    #[test]
    fn tight_flat_near_high_is_distribution() {
        let mut v: Vec<f64> = (0..40).map(|i| {
            if i % 2 == 0 { 100.0 } else { 100.5 }
        }).collect();
        v[39] = 100.45;
        let r = classify(&v, &WyckoffConfig::default());
        assert!(matches!(r.phase, WyckoffPhase::Distribution));
    }

    #[test]
    fn mid_range_flat_is_indeterminate() {
        // Flat oscillation centered on mid → no Accumulation or Distribution.
        let mut v: Vec<f64> = (0..40).map(|i| {
            if i % 2 == 0 { 100.0 } else { 100.5 }
        }).collect();
        v[39] = 100.25;    // dead center of [100, 100.5]
        let r = classify(&v, &WyckoffConfig::default());
        assert!(matches!(r.phase, WyckoffPhase::Indeterminate));
    }

    #[test]
    fn negative_mean_doesnt_panic() {
        // Negative prices shouldn't happen but test defense.
        let v: Vec<f64> = (0..40).map(|i| -100.0 + i as f64 * 0.1).collect();
        let r = classify(&v, &WyckoffConfig::default());
        // Mean is non-positive → defaults out.
        assert!(matches!(r.phase, WyckoffPhase::Indeterminate));
    }
}
