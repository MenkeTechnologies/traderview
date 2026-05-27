//! Risk-Reward calculator.
//!
//! Given entry / stop / target prices + an account risk budget, compute:
//!   * R:R ratio (target distance / stop distance)
//:   * qty that sizes the trade to risk_budget dollars
//!   * expected return if target hits
//!   * break-even win-rate the R:R implies
//!
//! Pure compute. Frontend uses this for the position-sizing widget +
//! the new-trade form's R:R live preview.

use crate::models::TradeSide;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RrInput {
    pub side: TradeSide,
    pub entry: Decimal,
    pub stop: Decimal,
    pub target: Decimal,
    /// Dollar risk budget for this trade. Used to size the position.
    pub risk_budget: Decimal,
    /// Asset multiplier (100 for equity options, ES = 50, etc).
    pub multiplier: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RrReport {
    /// Target distance / stop distance. Higher = better trade asymmetry.
    pub r_multiple: f64,
    /// Recommended share/contract count for the risk budget.
    pub qty: Decimal,
    /// Net risk in dollars at the recommended qty (≈ risk_budget).
    pub dollar_risk: Decimal,
    /// Net reward in dollars if target hits at the recommended qty.
    pub dollar_reward: Decimal,
    /// Win-rate needed to break even at this R:R, as decimal (0..=1).
    /// Formula: 1 / (1 + r_multiple).
    pub breakeven_win_rate: f64,
    /// Recommended sub-position scale-out plan (1/3 at 1R, 1/3 at 2R,
    /// runner). Each entry is `(label, price)`.
    pub scale_outs: Vec<ScaleOut>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleOut {
    pub label: String,
    pub price: Decimal,
    /// Fraction of total qty to exit here (0..=1).
    pub fraction: f64,
}

pub fn compute(input: &RrInput) -> Result<RrReport, &'static str> {
    let stop_dist = (input.entry - input.stop).abs();
    let target_dist = (input.target - input.entry).abs();
    if stop_dist.is_zero() {
        return Err("stop equals entry — risk is zero, cannot size");
    }
    let r_multiple_dec = target_dist / stop_dist;
    let r_multiple = decimal_to_f64(r_multiple_dec);

    // Direction sanity: for a LONG, target > entry > stop. For a SHORT,
    // target < entry < stop. Otherwise the user inverted something.
    match input.side {
        TradeSide::Long if input.target <= input.entry || input.stop >= input.entry => {
            return Err("long requires target > entry > stop")
        }
        TradeSide::Short if input.target >= input.entry || input.stop <= input.entry => {
            return Err("short requires target < entry < stop")
        }
        _ => {}
    }

    let per_unit_risk = stop_dist * input.multiplier;
    if per_unit_risk.is_zero() {
        return Err("multiplier × stop distance is zero");
    }
    let qty = input.risk_budget / per_unit_risk;
    let dollar_risk = qty * per_unit_risk;
    let dollar_reward = qty * target_dist * input.multiplier;
    let breakeven_win_rate = 1.0 / (1.0 + r_multiple);

    // Scale-out at 1R + 2R (in the direction of target) then runner.
    let one_r = match input.side {
        TradeSide::Long => input.entry + stop_dist,
        TradeSide::Short => input.entry - stop_dist,
    };
    let two_r = match input.side {
        TradeSide::Long => input.entry + stop_dist + stop_dist,
        TradeSide::Short => input.entry - stop_dist - stop_dist,
    };
    let scale_outs = vec![
        ScaleOut {
            label: "1R".into(),
            price: one_r,
            fraction: 1.0 / 3.0,
        },
        ScaleOut {
            label: "2R".into(),
            price: two_r,
            fraction: 1.0 / 3.0,
        },
        ScaleOut {
            label: "target".into(),
            price: input.target,
            fraction: 1.0 / 3.0,
        },
    ];

    Ok(RrReport {
        r_multiple,
        qty,
        dollar_risk,
        dollar_reward,
        breakeven_win_rate,
        scale_outs,
    })
}

fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn long_3r() -> RrInput {
        // $1 risk, $3 reward → R:R = 3.0, 25% breakeven win-rate.
        RrInput {
            side: TradeSide::Long,
            entry: d("100"),
            stop: d("99"),
            target: d("103"),
            risk_budget: d("100"), // $100 risk → 100 shares
            multiplier: Decimal::ONE,
        }
    }

    #[test]
    fn long_3r_computes_correctly() {
        let r = compute(&long_3r()).unwrap();
        assert!((r.r_multiple - 3.0).abs() < 1e-9);
        assert_eq!(r.qty, d("100"));
        assert_eq!(r.dollar_risk, d("100"));
        assert_eq!(r.dollar_reward, d("300"));
        assert!((r.breakeven_win_rate - 0.25).abs() < 1e-9);
    }

    #[test]
    fn short_works_with_inverted_geometry() {
        // Short at 100, stop 101, target 97 → $1 risk, $3 reward.
        let i = RrInput {
            side: TradeSide::Short,
            entry: d("100"),
            stop: d("101"),
            target: d("97"),
            risk_budget: d("100"),
            multiplier: Decimal::ONE,
        };
        let r = compute(&i).unwrap();
        assert!((r.r_multiple - 3.0).abs() < 1e-9);
        assert_eq!(r.qty, d("100"));
    }

    #[test]
    fn zero_stop_distance_returns_error() {
        let mut i = long_3r();
        i.stop = i.entry;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn long_with_target_below_entry_is_geometry_error() {
        let mut i = long_3r();
        i.target = d("99");
        assert!(compute(&i).is_err(), "long with target<entry must error");
    }

    #[test]
    fn long_with_stop_above_entry_is_geometry_error() {
        let mut i = long_3r();
        i.stop = d("101");
        assert!(compute(&i).is_err(), "long with stop>entry must error");
    }

    #[test]
    fn short_with_target_above_entry_is_geometry_error() {
        let i = RrInput {
            side: TradeSide::Short,
            entry: d("100"),
            stop: d("101"),
            target: d("103"),
            risk_budget: d("100"),
            multiplier: Decimal::ONE,
        };
        assert!(compute(&i).is_err());
    }

    #[test]
    fn options_multiplier_reduces_qty_proportionally() {
        // 1 option contract × 100 mult × $1 stop = $100 per-contract risk.
        // Budget $100 → 1 contract.
        let i = RrInput {
            side: TradeSide::Long,
            entry: d("5"),
            stop: d("4"),
            target: d("8"),
            risk_budget: d("100"),
            multiplier: d("100"),
        };
        let r = compute(&i).unwrap();
        assert_eq!(r.qty, d("1"));
        assert_eq!(r.dollar_risk, d("100"));
        // Reward: 1 × $3 × 100 = $300.
        assert_eq!(r.dollar_reward, d("300"));
    }

    #[test]
    fn breakeven_win_rate_matches_1_over_one_plus_r() {
        // Hand-compute for R:R = 2.0 → 33.33% breakeven.
        let mut i = long_3r();
        i.target = d("102"); // $2 reward / $1 risk = 2R
        let r = compute(&i).unwrap();
        assert!((r.r_multiple - 2.0).abs() < 1e-9);
        assert!((r.breakeven_win_rate - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn scale_out_levels_are_at_1r_2r_target() {
        let r = compute(&long_3r()).unwrap();
        assert_eq!(r.scale_outs.len(), 3);
        // long entry 100, $1 stop dist → 1R=101, 2R=102, target=103
        assert_eq!(r.scale_outs[0].price, d("101"));
        assert_eq!(r.scale_outs[1].price, d("102"));
        assert_eq!(r.scale_outs[2].price, d("103"));
        // Fractions sum to ≈1.0.
        let total: f64 = r.scale_outs.iter().map(|s| s.fraction).sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[test]
    fn short_scale_outs_use_inverted_geometry() {
        let i = RrInput {
            side: TradeSide::Short,
            entry: d("100"),
            stop: d("101"),
            target: d("97"),
            risk_budget: d("100"),
            multiplier: Decimal::ONE,
        };
        let r = compute(&i).unwrap();
        // 1R below entry = 99, 2R = 98, target = 97.
        assert_eq!(r.scale_outs[0].price, d("99"));
        assert_eq!(r.scale_outs[1].price, d("98"));
        assert_eq!(r.scale_outs[2].price, d("97"));
    }
}
