//! Impermanent loss — the constant-product AMM LP's cost of motion.
//!
//! For a 50/50 x·y=k pool and price ratio r = P_end / P_entry:
//!
//!   IL = 2√r / (1 + r) − 1        (≤ 0, symmetric in r and 1/r)
//!
//! The famous waypoints: ±2× costs 5.72%, ±4× costs 20%. The report
//! adds the fee APR the pool must earn over the holding period to
//! offset the IL — the only number that decides whether LPing beats
//! holding.
//!
//! Pure compute. Companion to `processing_spreads` (other people's
//! margins), `variance_risk_premium` (LP fees are short-vol income in
//! disguise).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct IlInput {
    /// Expected price ratio P_end / P_entry (e.g. 2.0 = doubles).
    pub price_ratio: f64,
    /// Holding period for the fee-breakeven row, days.
    #[serde(default = "default_days")]
    pub holding_days: f64,
}

fn default_days() -> f64 {
    365.0
}

#[derive(Debug, Clone, Serialize)]
pub struct IlPoint {
    pub price_ratio: f64,
    pub il_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct IlReport {
    /// IL at the requested ratio, % (negative).
    pub il_pct: f64,
    /// LP value as % of HODL value.
    pub lp_vs_hodl_pct: f64,
    /// Fee APR needed to offset the IL over the holding period.
    pub breakeven_fee_apr_pct: f64,
    /// The standard reference curve.
    pub curve: Vec<IlPoint>,
}

fn il(r: f64) -> f64 {
    2.0 * r.sqrt() / (1.0 + r) - 1.0
}

pub fn compute(inp: &IlInput) -> Option<IlReport> {
    if !inp.price_ratio.is_finite()
        || inp.price_ratio <= 0.0
        || !inp.holding_days.is_finite()
        || inp.holding_days <= 0.0
    {
        return None;
    }
    let loss = il(inp.price_ratio);
    let curve = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0, 5.0]
        .iter()
        .map(|&r| IlPoint {
            price_ratio: r,
            il_pct: il(r) * 100.0,
        })
        .collect();
    Some(IlReport {
        il_pct: loss * 100.0,
        lp_vs_hodl_pct: (1.0 + loss) * 100.0,
        breakeven_fee_apr_pct: -loss * 100.0 * 365.0 / inp.holding_days,
        curve,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_waypoints() {
        // No move ⇒ zero IL; 4× ⇒ exactly −20% (2·2/5 − 1).
        assert!(compute(&IlInput { price_ratio: 1.0, holding_days: 365.0 })
            .unwrap()
            .il_pct
            .abs()
            < 1e-12);
        let four_x = compute(&IlInput { price_ratio: 4.0, holding_days: 365.0 }).unwrap();
        assert!((four_x.il_pct + 20.0).abs() < 1e-9, "{}", four_x.il_pct);
        assert!((four_x.lp_vs_hodl_pct - 80.0).abs() < 1e-9);
        // 2× ⇒ 2√2/3 − 1 ≈ −5.719%.
        let two_x = compute(&IlInput { price_ratio: 2.0, holding_days: 365.0 }).unwrap();
        assert!((two_x.il_pct - (2.0 * 2.0_f64.sqrt() / 3.0 - 1.0) * 100.0).abs() < 1e-9);
    }

    #[test]
    fn symmetric_in_ratio_and_inverse() {
        let up = compute(&IlInput { price_ratio: 4.0, holding_days: 365.0 }).unwrap();
        let down = compute(&IlInput { price_ratio: 0.25, holding_days: 365.0 }).unwrap();
        assert!((up.il_pct - down.il_pct).abs() < 1e-9);
    }

    #[test]
    fn shorter_holding_needs_higher_fee_apr() {
        // 5.72% IL over a year needs ~5.72% APR; over 30 days ×365/30.
        let year = compute(&IlInput { price_ratio: 2.0, holding_days: 365.0 }).unwrap();
        let month = compute(&IlInput { price_ratio: 2.0, holding_days: 30.0 }).unwrap();
        assert!((month.breakeven_fee_apr_pct - year.breakeven_fee_apr_pct * 365.0 / 30.0).abs() < 1e-9);
        assert!(month.breakeven_fee_apr_pct > year.breakeven_fee_apr_pct);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&IlInput { price_ratio: 0.0, holding_days: 365.0 }).is_none());
        assert!(compute(&IlInput { price_ratio: f64::NAN, holding_days: 365.0 }).is_none());
        assert!(compute(&IlInput { price_ratio: 2.0, holding_days: 0.0 }).is_none());
    }
}
