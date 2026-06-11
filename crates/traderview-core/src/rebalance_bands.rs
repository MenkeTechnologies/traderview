//! 5/25 rebalancing bands (Larry Swedroe).
//!
//! Rebalance an asset only when its weight drifts past the NARROWER of
//!   - 5 percentage points absolute, or
//!   - 25% of its target, relative
//!
//! so a 40% sleeve trades at ±5pp (absolute binds) while a 4% sleeve
//! trades at ±1pp (relative binds). Threshold-based rebalancing beats
//! calendar rebalancing on turnover for the same tracking error — the
//! bands ARE the discipline.
//!
//! Pure compute. Companion to the rebalance engine (which executes the
//! trades this screen flags).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BandAsset {
    pub name: String,
    pub target_weight_pct: f64,
    pub current_weight_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BandsInput {
    pub assets: Vec<BandAsset>,
    /// Absolute band, pp (Swedroe: 5).
    #[serde(default = "default_abs_band")]
    pub absolute_band_pp: f64,
    /// Relative band, % of target (Swedroe: 25).
    #[serde(default = "default_rel_band")]
    pub relative_band_pct: f64,
}

fn default_abs_band() -> f64 {
    5.0
}
fn default_rel_band() -> f64 {
    25.0
}

#[derive(Debug, Clone, Serialize)]
pub struct BandRow {
    pub name: String,
    pub target_weight_pct: f64,
    pub current_weight_pct: f64,
    pub drift_pp: f64,
    /// The binding band: min(absolute, relative·target), pp.
    pub band_pp: f64,
    /// "absolute" or "relative" — which rule binds.
    pub binding_rule: &'static str,
    pub breached: bool,
    /// Trade to restore target, pp of portfolio (negative = sell).
    pub trade_pp: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BandsReport {
    pub rows: Vec<BandRow>,
    pub any_breach: bool,
}

pub fn compute(inp: &BandsInput) -> Option<BandsReport> {
    if inp.assets.is_empty()
        || inp.assets.len() > 100
        || !inp.absolute_band_pp.is_finite()
        || inp.absolute_band_pp <= 0.0
        || !inp.relative_band_pct.is_finite()
        || inp.relative_band_pct <= 0.0
        || inp.assets.iter().any(|a| {
            !a.target_weight_pct.is_finite()
                || a.target_weight_pct <= 0.0
                || a.target_weight_pct > 100.0
                || !a.current_weight_pct.is_finite()
                || a.current_weight_pct < 0.0
                || a.current_weight_pct > 100.0
                || a.name.trim().is_empty()
        })
    {
        return None;
    }
    let rows: Vec<BandRow> = inp
        .assets
        .iter()
        .map(|a| {
            let rel_band = a.target_weight_pct * inp.relative_band_pct / 100.0;
            let (band, rule) = if inp.absolute_band_pp <= rel_band {
                (inp.absolute_band_pp, "absolute")
            } else {
                (rel_band, "relative")
            };
            let drift = a.current_weight_pct - a.target_weight_pct;
            BandRow {
                name: a.name.clone(),
                target_weight_pct: a.target_weight_pct,
                current_weight_pct: a.current_weight_pct,
                drift_pp: drift,
                band_pp: band,
                binding_rule: rule,
                breached: drift.abs() >= band,
                trade_pp: -drift,
            }
        })
        .collect();
    Some(BandsReport {
        any_breach: rows.iter().any(|r| r.breached),
        rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str, target: f64, current: f64) -> BandAsset {
        BandAsset {
            name: name.into(),
            target_weight_pct: target,
            current_weight_pct: current,
        }
    }

    fn input(assets: Vec<BandAsset>) -> BandsInput {
        BandsInput {
            assets,
            absolute_band_pp: 5.0,
            relative_band_pct: 25.0,
        }
    }

    #[test]
    fn large_sleeve_binds_on_the_absolute_band() {
        // 40% target: relative band would be 10pp; absolute 5pp binds.
        // 46% current = 6pp drift ⇒ breach, sell 6pp.
        let r = compute(&input(vec![asset("US equity", 40.0, 46.0)])).unwrap();
        let row = &r.rows[0];
        assert_eq!(row.binding_rule, "absolute");
        assert!((row.band_pp - 5.0).abs() < 1e-12);
        assert!(row.breached);
        assert!((row.trade_pp + 6.0).abs() < 1e-12);
    }

    #[test]
    fn small_sleeve_binds_on_the_relative_band() {
        // 4% target: relative band 1pp < absolute 5pp.
        // 5.2% = 1.2pp drift ⇒ breach; 4.8% = 0.8pp ⇒ hold.
        let r = compute(&input(vec![
            asset("EM small value", 4.0, 5.2),
            asset("REITs", 4.0, 4.8),
        ]))
        .unwrap();
        assert_eq!(r.rows[0].binding_rule, "relative");
        assert!((r.rows[0].band_pp - 1.0).abs() < 1e-12);
        assert!(r.rows[0].breached);
        assert!(!r.rows[1].breached);
        assert!(r.any_breach);
    }

    #[test]
    fn twenty_percent_target_is_the_crossover() {
        // 20% × 25% = 5pp — both rules give the same band.
        let r = compute(&input(vec![asset("Intl", 20.0, 20.0)])).unwrap();
        assert!((r.rows[0].band_pp - 5.0).abs() < 1e-12);
        assert!(!r.rows[0].breached);
        assert!(!r.any_breach);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&input(vec![])).is_none());
        assert!(compute(&input(vec![asset("X", 0.0, 5.0)])).is_none());
        assert!(compute(&input(vec![asset("X", 40.0, 150.0)])).is_none());
        let mut bad = input(vec![asset("X", 40.0, 41.0)]);
        bad.absolute_band_pp = 0.0;
        assert!(compute(&bad).is_none());
    }
}
