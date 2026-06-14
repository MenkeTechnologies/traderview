//! SPAN-style portfolio margin — a scenario-based initial-margin estimate modeled
//! on the CME SPAN risk array. It revalues the portfolio under 16 standard risk
//! scenarios — seven price moves (−1 to +1 of the price scan range) each paired
//! with an up and down volatility shift (14), plus two extreme moves covering a
//! 35% loss fraction — and takes the worst-case loss as the scan risk. Portfolio
//! P&L under a scenario is `delta·dP + ½·gamma·dP² + vega·dV`. This is a faithful
//! simplification, not the full exchange algorithm (no inter-commodity spread
//! credits or delivery-month charges). Pure compute. Not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SpanInput {
    pub underlying_price: f64,
    /// Price scan range, percent of the underlying (e.g. 15).
    pub price_scan_pct: f64,
    /// Volatility scan range, in volatility points (the vega shock).
    #[serde(default)]
    pub vol_scan_points: f64,
    /// Net portfolio delta (per 1.0 of underlying price).
    #[serde(default)]
    pub portfolio_delta: f64,
    /// Net portfolio gamma.
    #[serde(default)]
    pub portfolio_gamma: f64,
    /// Net portfolio vega (per 1 vol point).
    #[serde(default)]
    pub portfolio_vega: f64,
    /// Extreme-move multiplier of the scan range (default 2).
    #[serde(default = "default_ext_mult")]
    pub extreme_move_multiplier: f64,
    /// Loss fraction credited on the extreme scenarios (default 0.35).
    #[serde(default = "default_ext_frac")]
    pub extreme_loss_fraction: f64,
}

fn default_ext_mult() -> f64 {
    2.0
}

fn default_ext_frac() -> f64 {
    0.35
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Scenario {
    pub index: u32,
    pub price_move: f64,
    pub vol_move: f64,
    pub pnl: f64,
    pub loss: f64,
    pub extreme: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct SpanReport {
    /// Worst-case loss across the 16 scenarios (the SPAN scan risk / margin).
    pub margin_usd: f64,
    /// Index of the worst scenario.
    pub worst_scenario: u32,
    pub scenarios: Vec<Scenario>,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &SpanInput) -> SpanReport {
    let scan = i.underlying_price * i.price_scan_pct / 100.0;
    let pnl = |dp: f64, dv: f64| -> f64 {
        i.portfolio_delta * dp + 0.5 * i.portfolio_gamma * dp * dp + i.portfolio_vega * dv
    };
    let fracs = [-1.0, -2.0 / 3.0, -1.0 / 3.0, 0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0];
    let mut scenarios = Vec::with_capacity(16);
    let mut idx = 0u32;
    // 14 price × vol scenarios.
    for &f in &fracs {
        let dp = scan * f;
        for &dv in &[i.vol_scan_points, -i.vol_scan_points] {
            let p = pnl(dp, dv);
            scenarios.push(Scenario {
                index: idx,
                price_move: round2(dp),
                vol_move: round2(dv),
                pnl: round2(p),
                loss: round2(-p),
                extreme: false,
            });
            idx += 1;
        }
    }
    // 2 extreme scenarios at the reduced loss fraction.
    for s in [1.0, -1.0] {
        let dp = scan * i.extreme_move_multiplier * s;
        let p = pnl(dp, i.vol_scan_points);
        let loss = -p * i.extreme_loss_fraction;
        scenarios.push(Scenario {
            index: idx,
            price_move: round2(dp),
            vol_move: round2(i.vol_scan_points),
            pnl: round2(p),
            loss: round2(loss),
            extreme: true,
        });
        idx += 1;
    }

    let (worst_idx, worst_loss) = scenarios
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.loss.partial_cmp(&b.1.loss).unwrap())
        .map(|(k, sc)| (k as u32, sc.loss))
        .unwrap_or((0, 0.0));

    SpanReport {
        margin_usd: round2(worst_loss.max(0.0)),
        worst_scenario: worst_idx,
        scenarios,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> SpanInput {
        SpanInput {
            underlying_price: 100.0,
            price_scan_pct: 15.0,
            vol_scan_points: 5.0,
            portfolio_delta: 10.0,
            portfolio_gamma: -2.0,
            portfolio_vega: 50.0,
            extreme_move_multiplier: 2.0,
            extreme_loss_fraction: 0.35,
        }
    }

    #[test]
    fn margin_is_worst_case_loss() {
        let d = generate(&base());
        // Verified against an independent scenario computation.
        assert!(close(d.margin_usd, 625.0));
        assert_eq!(d.scenarios.len(), 16);
        assert_eq!(d.worst_scenario, 1);
    }

    #[test]
    fn two_extreme_scenarios_flagged() {
        let d = generate(&base());
        assert_eq!(d.scenarios.iter().filter(|s| s.extreme).count(), 2);
    }

    #[test]
    fn margin_floored_at_zero() {
        // An all-cash portfolio (no Greeks) has no scenario loss.
        let d = generate(&SpanInput { portfolio_delta: 0.0, portfolio_gamma: 0.0, portfolio_vega: 0.0, ..base() });
        assert!(close(d.margin_usd, 0.0));
    }

    #[test]
    fn extreme_loss_uses_fraction() {
        let d = generate(&base());
        let ext = d.scenarios.iter().filter(|s| s.extreme).collect::<Vec<_>>();
        // Loss equals −pnl × 0.35.
        for s in ext {
            assert!(close(s.loss, -s.pnl * 0.35));
        }
    }

    #[test]
    fn larger_scan_larger_margin() {
        let d1 = generate(&base());
        let d2 = generate(&SpanInput { price_scan_pct: 30.0, ..base() });
        assert!(d2.margin_usd > d1.margin_usd);
    }
}
