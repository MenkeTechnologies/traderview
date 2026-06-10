//! Bond-tent allocation calculator (Kitces / Pfau).
//!
//! Michael Kitces and Wade Pfau's "bond tent" / "rising-equity glide
//! path" strategy: ramp UP bond allocation in the years leading into
//! retirement to dampen sequence-of-returns risk, then ramp BACK
//! DOWN in the years after — finishing retirement back at the
//! original stock allocation. The bond tent's peak is at retirement.
//!
//! Mechanism: SORR (sequence-of-returns risk) is highest in the
//! first ~10 years of retirement. Holding more bonds across that
//! window absorbs a bad early sequence without forcing equity sales
//! at the bottom; rising equity past the tent restores long-term
//! growth.
//!
//! Inputs:
//!   - current_age, retirement_age
//!   - tent_peak_bond_pct      — bond % AT retirement (peak of the tent)
//!   - pre_tent_bond_pct       — bond % today (left side)
//!   - post_tent_bond_pct      — bond % long after retirement (right side)
//!   - tent_ramp_years         — years before retirement to start ramping up
//!   - tent_descent_years      — years after retirement to ramp back down
//!   - horizon_age             — last age to include in the glide table
//!
//! Compute returns: yearly age × bond % glide table, plus identified
//! tent_start_age, tent_peak_age, tent_end_age for charting.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BondTentInput {
    pub current_age: u32,
    pub retirement_age: u32,
    pub pre_tent_bond_pct: f64,
    pub tent_peak_bond_pct: f64,
    pub post_tent_bond_pct: f64,
    pub tent_ramp_years: u32,
    pub tent_descent_years: u32,
    pub horizon_age: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlidePoint {
    pub age: u32,
    pub bond_pct: f64,
    pub stock_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BondTentReport {
    pub tent_start_age: u32,
    pub tent_peak_age: u32,
    pub tent_end_age: u32,
    pub glide: Vec<GlidePoint>,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn bond_pct_at_age(age: u32, input: &BondTentInput) -> f64 {
    let tent_start = input.retirement_age.saturating_sub(input.tent_ramp_years);
    let tent_peak = input.retirement_age;
    let tent_end = input.retirement_age.saturating_add(input.tent_descent_years);
    if age < tent_start {
        return input.pre_tent_bond_pct;
    }
    if age <= tent_peak {
        // Linear ramp up from pre to peak.
        if input.tent_ramp_years == 0 {
            return input.tent_peak_bond_pct;
        }
        let frac = (age - tent_start) as f64 / input.tent_ramp_years as f64;
        return input.pre_tent_bond_pct
            + (input.tent_peak_bond_pct - input.pre_tent_bond_pct) * frac;
    }
    if age <= tent_end {
        // Linear ramp down from peak to post.
        if input.tent_descent_years == 0 {
            return input.post_tent_bond_pct;
        }
        let frac = (age - tent_peak) as f64 / input.tent_descent_years as f64;
        return input.tent_peak_bond_pct
            + (input.post_tent_bond_pct - input.tent_peak_bond_pct) * frac;
    }
    input.post_tent_bond_pct
}

pub fn compute(input: &BondTentInput) -> BondTentReport {
    let tent_start = input.retirement_age.saturating_sub(input.tent_ramp_years);
    let tent_peak = input.retirement_age;
    let tent_end = input.retirement_age.saturating_add(input.tent_descent_years);
    let mut glide: Vec<GlidePoint> = Vec::new();
    let start = input.current_age;
    let end = input.horizon_age.max(start);
    for age in start..=end {
        let bond = bond_pct_at_age(age, input).clamp(0.0, 100.0);
        glide.push(GlidePoint {
            age,
            bond_pct: bond,
            stock_pct: 100.0 - bond,
        });
    }
    BondTentReport {
        tent_start_age: tent_start,
        tent_peak_age: tent_peak,
        tent_end_age: tent_end,
        glide,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> BondTentInput {
        BondTentInput {
            current_age: 50,
            retirement_age: 65,
            pre_tent_bond_pct: 30.0,
            tent_peak_bond_pct: 60.0,
            post_tent_bond_pct: 30.0,
            tent_ramp_years: 10,
            tent_descent_years: 10,
            horizon_age: 90,
        }
    }

    #[test]
    fn bond_pct_at_age_before_tent_is_pre() {
        let i = input();
        // tent_start = 65 − 10 = 55. At age 50 → pre.
        assert_eq!(bond_pct_at_age(50, &i), 30.0);
    }

    #[test]
    fn bond_pct_at_age_at_tent_peak_equals_peak() {
        let i = input();
        assert_eq!(bond_pct_at_age(65, &i), 60.0);
    }

    #[test]
    fn bond_pct_at_age_at_tent_end_equals_post() {
        let i = input();
        // tent_end = 65 + 10 = 75
        assert_eq!(bond_pct_at_age(75, &i), 30.0);
    }

    #[test]
    fn bond_pct_at_age_after_tent_is_post() {
        let i = input();
        assert_eq!(bond_pct_at_age(85, &i), 30.0);
    }

    #[test]
    fn bond_pct_at_age_midway_up_ramp() {
        let i = input();
        // tent_start 55, peak 65, halfway at 60 → 30 + 0.5 × (60 − 30) = 45
        assert_eq!(bond_pct_at_age(60, &i), 45.0);
    }

    #[test]
    fn bond_pct_at_age_midway_down_ramp() {
        let i = input();
        // peak 65, end 75, halfway at 70 → 60 + 0.5 × (30 − 60) = 45
        assert_eq!(bond_pct_at_age(70, &i), 45.0);
    }

    #[test]
    fn bond_pct_at_age_zero_ramp_jumps_to_peak() {
        let mut i = input();
        i.tent_ramp_years = 0;
        // tent_start = 65, so at 65 should be peak (no ramp).
        assert_eq!(bond_pct_at_age(65, &i), 60.0);
    }

    #[test]
    fn compute_glide_age_count() {
        let r = compute(&input());
        // 50..=90 inclusive = 41 entries
        assert_eq!(r.glide.len(), 41);
        assert_eq!(r.glide[0].age, 50);
        assert_eq!(r.glide.last().unwrap().age, 90);
    }

    #[test]
    fn compute_tent_landmark_ages() {
        let r = compute(&input());
        assert_eq!(r.tent_start_age, 55);
        assert_eq!(r.tent_peak_age, 65);
        assert_eq!(r.tent_end_age, 75);
    }

    #[test]
    fn compute_peak_in_glide() {
        let r = compute(&input());
        // Find age 65 in glide and check it's the peak.
        let peak = r.glide.iter().find(|p| p.age == 65).unwrap();
        assert_eq!(peak.bond_pct, 60.0);
        assert_eq!(peak.stock_pct, 40.0);
    }

    #[test]
    fn compute_glide_monotonic_up_to_peak() {
        let r = compute(&input());
        for w in r.glide.windows(2) {
            if w[1].age <= 65 && w[0].age >= 55 {
                assert!(w[1].bond_pct >= w[0].bond_pct,
                    "expected monotonic ramp up; got {} → {}",
                    w[0].bond_pct, w[1].bond_pct);
            }
        }
    }

    #[test]
    fn compute_clamps_bond_pct() {
        let mut i = input();
        i.tent_peak_bond_pct = 150.0; // intentionally out of range
        let r = compute(&i);
        for p in &r.glide {
            assert!(p.bond_pct >= 0.0 && p.bond_pct <= 100.0);
            assert!(p.stock_pct >= 0.0 && p.stock_pct <= 100.0);
        }
    }
}
