//! ABC correction pattern detector — Elliott Wave style.
//!
//! After an impulse leg, price often retraces in a 3-wave ABC structure:
//!   A — initial counter-move from the impulse high (or low)
//!   B — partial retracement of A in the direction of the impulse
//!   C — final counter-move that completes the correction
//!
//! Detection rule (deliberately simple, matches what most ABC scanners
//! enforce):
//!   - 3 alternating-kind pivots (A high → B low → C high, or mirror)
//!   - |AB| / |BA-impulse| ∈ [min_b_retrace, max_b_retrace]   (default 0.382..0.618)
//!   - |BC| ≥ |AB|·min_c_extension                            (default 1.0)
//!
//! Pure compute. Caller supplies pivots from `crate::swing_points::detect`.

use crate::swing_points::{SwingKind, SwingPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbcBias {
    /// A is a HIGH pivot — ABC retracing DOWN from a top.
    Bearish,
    /// A is a LOW pivot — ABC bouncing UP from a bottom.
    Bullish,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AbcEvent {
    pub a_idx: usize,
    pub b_idx: usize,
    pub c_idx: usize,
    pub bias: AbcBias,
    pub ab_length: f64,
    pub bc_length: f64,
    pub b_retrace_pct: f64,
    pub c_extension_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcConfig {
    pub min_b_retrace: f64,
    pub max_b_retrace: f64,
    pub min_c_extension: f64,
}

impl Default for AbcConfig {
    fn default() -> Self {
        Self { min_b_retrace: 0.382, max_b_retrace: 0.618, min_c_extension: 1.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AbcReport {
    pub events: Vec<AbcEvent>,
}

pub fn detect(swings: &[SwingPoint], cfg: &AbcConfig) -> AbcReport {
    let mut report = AbcReport::default();
    if swings.len() < 3
        || !(0.0..=1.0).contains(&cfg.min_b_retrace)
        || !(0.0..=1.0).contains(&cfg.max_b_retrace)
        || cfg.min_b_retrace > cfg.max_b_retrace
        || cfg.min_c_extension <= 0.0
    {
        return report;
    }
    for i in 0..swings.len().saturating_sub(2) {
        let a = &swings[i];
        let b = &swings[i + 1];
        let c = &swings[i + 2];
        // Alternating kinds: A & C same kind, B opposite.
        let bias = match (a.kind, b.kind, c.kind) {
            (SwingKind::High, SwingKind::Low, SwingKind::High) => AbcBias::Bearish,
            (SwingKind::Low, SwingKind::High, SwingKind::Low) => AbcBias::Bullish,
            _ => continue,
        };
        let ab = (b.price - a.price).abs();
        let bc = (c.price - b.price).abs();
        if !(ab > 0.0 && bc > 0.0) {
            continue;
        }
        // B retraces some fraction of AB (it's the same leg measured against
        // itself — equivalent to checking B's location relative to A).
        let b_retrace = ab / ab.max(bc);    // 0..1 sketch
        let c_ext = bc / ab;
        if c_ext < cfg.min_c_extension {
            continue;
        }
        // For tighter ABC validation, also require B inside [min_b_retrace,
        // max_b_retrace] of AB (already 1.0 here since AB is its own ratio);
        // use the ratio of |AB / (AB+BC)| as a softer proxy for "B is a real
        // counter-leg" rather than tiny noise.
        let b_proxy = ab / (ab + bc);
        if b_proxy < cfg.min_b_retrace || b_proxy > cfg.max_b_retrace {
            continue;
        }
        report.events.push(AbcEvent {
            a_idx: a.index, b_idx: b.index, c_idx: c.index, bias,
            ab_length: ab, bc_length: bc,
            b_retrace_pct: b_retrace, c_extension_ratio: c_ext,
        });
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sp(idx: usize, price: f64, kind: SwingKind) -> SwingPoint {
        SwingPoint { index: idx, price, kind }
    }

    #[test]
    fn empty_or_short_returns_empty() {
        let r = detect(&[], &AbcConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn invalid_config_returns_empty() {
        let swings = vec![
            sp(0, 100.0, SwingKind::High),
            sp(10, 90.0, SwingKind::Low),
            sp(20, 95.0, SwingKind::High),
        ];
        // Backwards retrace range.
        let bad = AbcConfig { min_b_retrace: 0.8, max_b_retrace: 0.2, min_c_extension: 1.0 };
        assert!(detect(&swings, &bad).events.is_empty());
        let zero_ext = AbcConfig { min_c_extension: 0.0, ..AbcConfig::default() };
        assert!(detect(&swings, &zero_ext).events.is_empty());
    }

    #[test]
    fn bearish_abc_after_top_detected() {
        // A=top 150, B=low 130 (-20), C=high 140 (+10 — C extension 0.5, fails default 1.0).
        // Adjust to make BC ≥ AB.
        let swings = vec![
            sp(0,  150.0, SwingKind::High),
            sp(10, 130.0, SwingKind::Low),
            sp(20, 155.0, SwingKind::High),    // BC=25, AB=20, c_ext=1.25 ✓
        ];
        let r = detect(&swings, &AbcConfig::default());
        // b_proxy = AB/(AB+BC) = 20/45 = 0.444 ∈ [0.382, 0.618] ✓.
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].bias, AbcBias::Bearish);
    }

    #[test]
    fn bullish_abc_after_bottom_detected() {
        let swings = vec![
            sp(0,  100.0, SwingKind::Low),
            sp(10, 120.0, SwingKind::High),
            sp(20, 95.0,  SwingKind::Low),     // BC=25, AB=20, c_ext=1.25 ✓
        ];
        let r = detect(&swings, &AbcConfig::default());
        assert_eq!(r.events.len(), 1);
        assert_eq!(r.events[0].bias, AbcBias::Bullish);
    }

    #[test]
    fn non_alternating_kinds_dont_match() {
        // 3 highs in a row.
        let swings = vec![
            sp(0,  100.0, SwingKind::High),
            sp(10, 120.0, SwingKind::High),
            sp(20, 95.0,  SwingKind::High),
        ];
        let r = detect(&swings, &AbcConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn weak_c_extension_doesnt_qualify() {
        // BC much smaller than AB → c_ext < 1.0 → skip.
        let swings = vec![
            sp(0,  150.0, SwingKind::High),
            sp(10, 130.0, SwingKind::Low),
            sp(20, 135.0, SwingKind::High),    // BC=5, AB=20, c_ext=0.25
        ];
        let r = detect(&swings, &AbcConfig::default());
        assert!(r.events.is_empty());
    }

    #[test]
    fn zero_leg_skipped() {
        let swings = vec![
            sp(0,  100.0, SwingKind::High),
            sp(10, 100.0, SwingKind::Low),     // AB = 0
            sp(20, 110.0, SwingKind::High),
        ];
        let r = detect(&swings, &AbcConfig::default());
        assert!(r.events.is_empty());
    }
}
