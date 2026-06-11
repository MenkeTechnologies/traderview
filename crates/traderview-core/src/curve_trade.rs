//! Curve trade builder — DV01-neutral steepeners, flatteners, and
//! butterflies.
//!
//! Two legs ⇒ spread trade: level = y_far − y_near (bp), the far-leg
//! notional sized so both legs carry equal DV01 — P/L is then pure
//! curve, not outright duration.
//!
//! Three legs ⇒ 50:50 butterfly: level = 2·y_belly − y_near − y_far
//! (bp); each wing takes half the belly's DV01, immunizing level and
//! slope so the position prices curvature alone.
//!
//! DV01 quoted as $ per bp per $1M notional (the desk convention).
//!
//! Pure compute. Companion to `key_rate_duration`, `yield_curve`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CurveLeg {
    pub label: String,
    pub yield_pct: f64,
    /// $ per bp per $1M notional.
    pub dv01_per_million: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurveTradeInput {
    /// 2 legs (near, far) or 3 legs (near wing, belly, far wing).
    pub legs: Vec<CurveLeg>,
    /// Anchor notional, $M — on the FIRST leg for spreads, the BELLY
    /// for butterflies.
    pub anchor_notional_mm: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SizedLeg {
    pub label: String,
    pub notional_mm: f64,
    pub dv01_total: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurveTradeReport {
    /// "spread" or "butterfly".
    pub kind: &'static str,
    /// Spread or fly level, basis points.
    pub level_bp: f64,
    pub legs: Vec<SizedLeg>,
    /// $ P/L per 1bp move in the level, given DV01 neutrality.
    pub pl_per_bp: f64,
}

pub fn compute(inp: &CurveTradeInput) -> Option<CurveTradeReport> {
    let n = inp.legs.len();
    if !(n == 2 || n == 3)
        || !inp.anchor_notional_mm.is_finite()
        || inp.anchor_notional_mm <= 0.0
        || inp.legs.iter().any(|l| {
            !l.yield_pct.is_finite()
                || !l.dv01_per_million.is_finite()
                || l.dv01_per_million <= 0.0
                || l.label.trim().is_empty()
        })
    {
        return None;
    }
    let mm = inp.anchor_notional_mm;
    if n == 2 {
        let (near, far) = (&inp.legs[0], &inp.legs[1]);
        let anchor_dv01 = near.dv01_per_million * mm;
        let far_mm = anchor_dv01 / far.dv01_per_million;
        Some(CurveTradeReport {
            kind: "spread",
            level_bp: (far.yield_pct - near.yield_pct) * 100.0,
            legs: vec![
                SizedLeg {
                    label: near.label.clone(),
                    notional_mm: mm,
                    dv01_total: anchor_dv01,
                },
                SizedLeg {
                    label: far.label.clone(),
                    notional_mm: far_mm,
                    dv01_total: anchor_dv01,
                },
            ],
            pl_per_bp: anchor_dv01,
        })
    } else {
        let (near, belly, far) = (&inp.legs[0], &inp.legs[1], &inp.legs[2]);
        let belly_dv01 = belly.dv01_per_million * mm;
        let wing_dv01 = belly_dv01 / 2.0;
        Some(CurveTradeReport {
            kind: "butterfly",
            level_bp: (2.0 * belly.yield_pct - near.yield_pct - far.yield_pct) * 100.0,
            legs: vec![
                SizedLeg {
                    label: near.label.clone(),
                    notional_mm: wing_dv01 / near.dv01_per_million,
                    dv01_total: wing_dv01,
                },
                SizedLeg {
                    label: belly.label.clone(),
                    notional_mm: mm,
                    dv01_total: belly_dv01,
                },
                SizedLeg {
                    label: far.label.clone(),
                    notional_mm: wing_dv01 / far.dv01_per_million,
                    dv01_total: wing_dv01,
                },
            ],
            // The belly carries the fly: dLevel = 2·dy_belly − dy_wings,
            // and the wings each hold half the belly DV01 — $/bp of the
            // FLY equals half the belly DV01.
            pl_per_bp: belly_dv01 / 2.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leg(label: &str, y: f64, dv01: f64) -> CurveLeg {
        CurveLeg {
            label: label.into(),
            yield_pct: y,
            dv01_per_million: dv01,
        }
    }

    #[test]
    fn two_leg_spread_is_dv01_neutral() {
        // 2s10s on an inverted curve: 2y 4.60 (DV01 190/1M), 10y 4.20
        // (880/1M), 10M anchor on the 2y.
        let r = compute(&CurveTradeInput {
            legs: vec![leg("2y", 4.6, 190.0), leg("10y", 4.2, 880.0)],
            anchor_notional_mm: 10.0,
        })
        .unwrap();
        assert_eq!(r.kind, "spread");
        assert!((r.level_bp + 40.0).abs() < 1e-9); // inverted −40bp
        assert!((r.legs[0].dv01_total - 1900.0).abs() < 1e-9);
        assert!((r.legs[1].dv01_total - 1900.0).abs() < 1e-9);
        // Far notional = 1900 / 880 ≈ 2.159M.
        assert!((r.legs[1].notional_mm - 1900.0 / 880.0).abs() < 1e-9);
        assert!((r.pl_per_bp - 1900.0).abs() < 1e-9);
    }

    #[test]
    fn butterfly_splits_belly_dv01_across_wings() {
        // 2s5s10s, 10M belly at 440/1M ⇒ belly DV01 4400, wings 2200.
        let r = compute(&CurveTradeInput {
            legs: vec![
                leg("2y", 4.6, 190.0),
                leg("5y", 4.35, 440.0),
                leg("10y", 4.2, 880.0),
            ],
            anchor_notional_mm: 10.0,
        })
        .unwrap();
        assert_eq!(r.kind, "butterfly");
        // Fly = 2·4.35 − 4.6 − 4.2 = −0.10% = −10bp.
        assert!((r.level_bp + 10.0).abs() < 1e-9);
        assert!((r.legs[1].dv01_total - 4400.0).abs() < 1e-9);
        assert!((r.legs[0].dv01_total - 2200.0).abs() < 1e-9);
        assert!((r.legs[0].notional_mm - 2200.0 / 190.0).abs() < 1e-9);
        assert!((r.legs[2].notional_mm - 2.5).abs() < 1e-9); // 2200/880
        assert!((r.pl_per_bp - 2200.0).abs() < 1e-9);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&CurveTradeInput {
            legs: vec![leg("2y", 4.6, 190.0)],
            anchor_notional_mm: 10.0,
        })
        .is_none()); // one leg
        assert!(compute(&CurveTradeInput {
            legs: vec![leg("2y", 4.6, 0.0), leg("10y", 4.2, 880.0)],
            anchor_notional_mm: 10.0,
        })
        .is_none()); // zero DV01
        assert!(compute(&CurveTradeInput {
            legs: vec![leg("2y", 4.6, 190.0), leg("10y", 4.2, 880.0)],
            anchor_notional_mm: 0.0,
        })
        .is_none());
    }
}
