//! Commodity processing spreads — the cash margins refiners, crushers,
//! and generators trade:
//!
//! * Crack 3-2-1 — 3 bbl crude → 2 bbl gasoline + 1 bbl distillate:
//!   crack = (2·RB·42 + 1·HO·42 − 3·CL) / 3      ($/bbl)
//!   (products quoted $/gal; 42 gal per barrel)
//! * Soybean board crush — 1 bu (60 lb) beans → 44 lb meal + 11 lb oil:
//!   crush = 0.022·meal($/short ton) + 11·oil($/lb) − beans($/bu)
//!   (44 lb = 0.022 short tons — the CME yield factors)
//! * Spark spread — power − gas × heat rate; the same formula with a
//!   coal fuel cost is the dark spread.
//!
//! Pure compute. Companion to `margrabe_spread_option` (options ON
//! these spreads), `carry_roll_decomposition`.

use serde::Serialize;

const GALLONS_PER_BARREL: f64 = 42.0;
const MEAL_TONS_PER_BUSHEL: f64 = 0.022; // 44 lb / 2000 lb
const OIL_LBS_PER_BUSHEL: f64 = 11.0;

#[derive(Debug, Clone, Serialize)]
pub struct CrackReport {
    /// 3-2-1 margin, $/bbl of crude.
    pub crack_321: f64,
    /// Single-product cracks, $/bbl.
    pub gasoline_crack: f64,
    pub distillate_crack: f64,
    /// Margin as % of the crude price.
    pub margin_pct: f64,
}

/// `crude` $/bbl; `gasoline`/`distillate` $/gal.
pub fn crack_321(crude: f64, gasoline: f64, distillate: f64) -> Option<CrackReport> {
    if ![crude, gasoline, distillate].iter().all(|v| v.is_finite() && *v > 0.0) {
        return None;
    }
    let rb_bbl = gasoline * GALLONS_PER_BARREL;
    let ho_bbl = distillate * GALLONS_PER_BARREL;
    let crack = (2.0 * rb_bbl + ho_bbl - 3.0 * crude) / 3.0;
    Some(CrackReport {
        crack_321: crack,
        gasoline_crack: rb_bbl - crude,
        distillate_crack: ho_bbl - crude,
        margin_pct: crack / crude * 100.0,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct CrushReport {
    /// Board crush, $/bu.
    pub crush: f64,
    pub meal_value_per_bu: f64,
    pub oil_value_per_bu: f64,
    pub margin_pct: f64,
}

/// `beans` $/bu; `meal` $/short ton; `oil` $/lb.
pub fn soybean_crush(beans: f64, meal: f64, oil: f64) -> Option<CrushReport> {
    if ![beans, meal, oil].iter().all(|v| v.is_finite() && *v > 0.0) {
        return None;
    }
    let meal_v = MEAL_TONS_PER_BUSHEL * meal;
    let oil_v = OIL_LBS_PER_BUSHEL * oil;
    let crush = meal_v + oil_v - beans;
    Some(CrushReport {
        crush,
        meal_value_per_bu: meal_v,
        oil_value_per_bu: oil_v,
        margin_pct: crush / beans * 100.0,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct SparkReport {
    /// Margin, $/MWh.
    pub spread: f64,
    pub fuel_cost_per_mwh: f64,
    /// Power price needed to break even at this heat rate.
    pub breakeven_power: f64,
    /// Implied market heat rate power/fuel — above the plant's own
    /// heat rate means the plant is in the money.
    pub market_implied_heat_rate: f64,
}

/// `power` $/MWh; `fuel` $/MMBtu; `heat_rate` MMBtu/MWh (gas ≈ 7,
/// coal ≈ 10 — the latter makes this the dark spread).
pub fn spark_spread(power: f64, fuel: f64, heat_rate: f64) -> Option<SparkReport> {
    if ![power, fuel, heat_rate].iter().all(|v| v.is_finite() && *v > 0.0) {
        return None;
    }
    let fuel_cost = fuel * heat_rate;
    Some(SparkReport {
        spread: power - fuel_cost,
        fuel_cost_per_mwh: fuel_cost,
        breakeven_power: fuel_cost,
        market_implied_heat_rate: power / fuel,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crack_321_hand_walk() {
        // CL $80, RB $2.50/gal, HO $2.80/gal:
        // RB/bbl 105, HO/bbl 117.6; crack = (210 + 117.6 − 240)/3 = 29.2.
        let r = crack_321(80.0, 2.5, 2.8).unwrap();
        assert!((r.crack_321 - 29.2).abs() < 1e-9, "{}", r.crack_321);
        assert!((r.gasoline_crack - 25.0).abs() < 1e-9);
        assert!((r.distillate_crack - 37.6).abs() < 1e-9);
        assert!((r.margin_pct - 29.2 / 80.0 * 100.0).abs() < 1e-9);
    }

    #[test]
    fn crush_hand_walk() {
        // Beans $12/bu, meal $350/ton, oil $0.45/lb:
        // 0.022·350 = 7.70 meal + 11·0.45 = 4.95 oil − 12 = $0.65/bu.
        let r = soybean_crush(12.0, 350.0, 0.45).unwrap();
        assert!((r.meal_value_per_bu - 7.7).abs() < 1e-12);
        assert!((r.oil_value_per_bu - 4.95).abs() < 1e-12);
        assert!((r.crush - 0.65).abs() < 1e-12);
    }

    #[test]
    fn spark_and_dark_are_the_same_formula() {
        // Gas plant: $50 power, $4 gas, HR 7.5 ⇒ $20 margin, breakeven
        // $30, market heat rate 12.5 > 7.5 ⇒ in the money.
        let spark = spark_spread(50.0, 4.0, 7.5).unwrap();
        assert!((spark.spread - 20.0).abs() < 1e-12);
        assert!((spark.breakeven_power - 30.0).abs() < 1e-12);
        assert!((spark.market_implied_heat_rate - 12.5).abs() < 1e-12);
        // Coal plant at HR 10 with $2 coal: dark spread $30.
        let dark = spark_spread(50.0, 2.0, 10.0).unwrap();
        assert!((dark.spread - 30.0).abs() < 1e-12);
    }

    #[test]
    fn negative_margins_pass_through() {
        // Refining underwater: crack can go negative — report, don't
        // reject (it's a real regime).
        let r = crack_321(120.0, 2.0, 2.2).unwrap();
        assert!(r.crack_321 < 0.0);
        let s = spark_spread(20.0, 4.0, 7.5).unwrap();
        assert!(s.spread < 0.0);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(crack_321(0.0, 2.5, 2.8).is_none());
        assert!(crack_321(80.0, f64::NAN, 2.8).is_none());
        assert!(soybean_crush(12.0, -350.0, 0.45).is_none());
        assert!(spark_spread(50.0, 4.0, 0.0).is_none());
    }
}
