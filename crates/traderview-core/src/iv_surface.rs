//! Implied-volatility smile surface — a 2D grid of implied volatility across
//! moneyness and expiry. Each expiry carries an at-the-money IV (the term
//! structure); the smile within an expiry is a parametric function of
//! log-moneyness `k = ln(K/F)`: `IV = atm_iv × (1 + skew·k + curvature·k²)`. A
//! negative skew makes downside strikes (puts) richer, the typical equity smirk;
//! curvature controls the smile's convexity. This composes the per-strike skew
//! and the term-structure inputs into a single interpolated surface. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ExpirySlice {
    pub label: String,
    pub years: f64,
    /// At-the-money implied volatility for this expiry, percent.
    pub atm_iv_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IvSurfaceInput {
    pub expiries: Vec<ExpirySlice>,
    /// Moneyness levels (strike ÷ forward), e.g. [0.8, 0.9, 1.0, 1.1, 1.2].
    pub moneyness_levels: Vec<f64>,
    /// Skew per unit log-moneyness (negative = equity smirk).
    #[serde(default)]
    pub skew: f64,
    /// Smile curvature (convexity).
    #[serde(default)]
    pub curvature: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IvCell {
    pub moneyness: f64,
    pub iv_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SurfaceRow {
    pub label: String,
    pub years: f64,
    pub atm_iv_pct: f64,
    pub cells: Vec<IvCell>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct IvSurfaceReport {
    pub rows: Vec<SurfaceRow>,
    pub min_iv_pct: f64,
    pub max_iv_pct: f64,
    pub ok: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &IvSurfaceInput) -> IvSurfaceReport {
    if i.expiries.is_empty() || i.moneyness_levels.is_empty() {
        return IvSurfaceReport::default();
    }
    let mut min_iv = f64::INFINITY;
    let mut max_iv = f64::NEG_INFINITY;
    let rows: Vec<SurfaceRow> = i
        .expiries
        .iter()
        .map(|e| {
            let cells: Vec<IvCell> = i
                .moneyness_levels
                .iter()
                .map(|&m| {
                    let k = if m > 0.0 { m.ln() } else { 0.0 };
                    // Floor at a small positive vol so the surface stays well-defined.
                    let iv = (e.atm_iv_pct * (1.0 + i.skew * k + i.curvature * k * k)).max(0.01);
                    min_iv = min_iv.min(iv);
                    max_iv = max_iv.max(iv);
                    IvCell { moneyness: m, iv_pct: round2(iv) }
                })
                .collect();
            SurfaceRow { label: e.label.clone(), years: e.years, atm_iv_pct: round2(e.atm_iv_pct), cells }
        })
        .collect();

    IvSurfaceReport {
        rows,
        min_iv_pct: round2(min_iv),
        max_iv_pct: round2(max_iv),
        ok: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> IvSurfaceInput {
        IvSurfaceInput {
            expiries: vec![
                ExpirySlice { label: "30d".into(), years: 0.0822, atm_iv_pct: 20.0 },
                ExpirySlice { label: "90d".into(), years: 0.2466, atm_iv_pct: 22.0 },
            ],
            moneyness_levels: vec![0.9, 1.0, 1.1],
            skew: -0.1,
            curvature: 0.5,
        }
    }

    #[test]
    fn atm_cell_equals_atm_iv() {
        let d = generate(&base());
        // moneyness 1.0 → k=0 → iv = atm.
        let atm_cell = d.rows[0].cells.iter().find(|c| close(c.moneyness, 1.0)).unwrap();
        assert!(close(atm_cell.iv_pct, 20.0));
    }

    #[test]
    fn smile_cells_match_formula() {
        let d = generate(&base());
        let r = &d.rows[0];
        assert!(close(r.cells[0].iv_pct, 20.32)); // 0.9
        assert!(close(r.cells[2].iv_pct, 19.90)); // 1.1
    }

    #[test]
    fn negative_skew_puts_richer() {
        let d = generate(&base());
        let r = &d.rows[0];
        // Downside (0.9) IV > upside (1.1) IV under negative skew.
        assert!(r.cells[0].iv_pct > r.cells[2].iv_pct);
    }

    #[test]
    fn term_structure_carries_atm() {
        let d = generate(&base());
        let atm90 = d.rows[1].cells.iter().find(|c| close(c.moneyness, 1.0)).unwrap();
        assert!(close(atm90.iv_pct, 22.0));
    }

    #[test]
    fn min_max_span_the_surface() {
        let d = generate(&base());
        assert!(d.min_iv_pct <= 20.0 && d.max_iv_pct >= 22.0);
    }

    #[test]
    fn empty_inputs_not_ok() {
        let d = generate(&IvSurfaceInput { expiries: vec![], ..base() });
        assert!(!d.ok);
    }
}
