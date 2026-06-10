//! Target-Date Fund (TDF) glide path generator.
//!
//! Standard TDF glide path: bond allocation increases linearly with
//! age. Vanguard's published glide for their Target Retirement series
//! (the industry's largest TDF family):
//!
//!   - age 25 (40 years to retirement): 90% stocks / 10% bonds
//!   - age 60 (retirement target year): 50% stocks / 50% bonds
//!   - age 67 (retire +7y "landing point"): 30% stocks / 70% bonds
//!   - age 72+ (5y after landing): same 30/70 forever
//!
//! This module produces a generic two-segment linear glide:
//!   - Phase 1 (current → retirement_age): stocks go from
//!     `start_stock_pct` down to `retire_stock_pct`
//!   - Phase 2 (retirement → landing_age):  stocks continue down
//!     from `retire_stock_pct` to `landing_stock_pct`
//!   - Phase 3 (landing_age → horizon_age): flat at `landing_stock_pct`
//!
//! User can match Vanguard / Fidelity / Schwab published glides by
//! setting the four anchor percentages.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GlidePathInput {
    pub current_age: u32,
    pub retirement_age: u32,
    pub landing_age: u32,
    pub horizon_age: u32,
    pub start_stock_pct: f64,
    pub retire_stock_pct: f64,
    pub landing_stock_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlidePoint {
    pub age: u32,
    pub stock_pct: f64,
    pub bond_pct: f64,
    pub phase: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlidePathReport {
    pub glide: Vec<GlidePoint>,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn stock_pct_at(age: u32, input: &GlidePathInput) -> (f64, &'static str) {
    if age <= input.current_age {
        return (input.start_stock_pct, "pre");
    }
    if age <= input.retirement_age {
        // Linear from start to retire over the working years.
        let span = input.retirement_age.saturating_sub(input.current_age) as f64;
        if span <= 0.0 { return (input.retire_stock_pct, "working"); }
        let frac = (age - input.current_age) as f64 / span;
        let p = input.start_stock_pct
            + (input.retire_stock_pct - input.start_stock_pct) * frac;
        return (p, "working");
    }
    if age <= input.landing_age {
        // Linear from retire to landing.
        let span = input.landing_age.saturating_sub(input.retirement_age) as f64;
        if span <= 0.0 { return (input.landing_stock_pct, "to_landing"); }
        let frac = (age - input.retirement_age) as f64 / span;
        let p = input.retire_stock_pct
            + (input.landing_stock_pct - input.retire_stock_pct) * frac;
        return (p, "to_landing");
    }
    (input.landing_stock_pct, "post_landing")
}

pub fn compute(input: &GlidePathInput) -> GlidePathReport {
    let start = input.current_age;
    let end = input.horizon_age.max(start);
    let mut glide: Vec<GlidePoint> = Vec::with_capacity((end - start + 1) as usize);
    for age in start..=end {
        let (stock_raw, phase) = stock_pct_at(age, input);
        let stock = stock_raw.clamp(0.0, 100.0);
        glide.push(GlidePoint {
            age,
            stock_pct: stock,
            bond_pct: 100.0 - stock,
            phase,
        });
    }
    GlidePathReport { glide }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> GlidePathInput {
        // Vanguard Target Retirement glide (approx).
        GlidePathInput {
            current_age: 25,
            retirement_age: 65,
            landing_age: 72,
            horizon_age: 90,
            start_stock_pct: 90.0,
            retire_stock_pct: 50.0,
            landing_stock_pct: 30.0,
        }
    }

    #[test]
    fn stock_at_current_age_is_start() {
        let (s, _phase) = stock_pct_at(25, &input());
        assert_eq!(s, 90.0);
    }

    #[test]
    fn stock_at_retirement_age_is_retire() {
        let (s, _phase) = stock_pct_at(65, &input());
        assert_eq!(s, 50.0);
    }

    #[test]
    fn stock_at_landing_age_is_landing() {
        let (s, _phase) = stock_pct_at(72, &input());
        assert_eq!(s, 30.0);
    }

    #[test]
    fn stock_post_landing_flat() {
        let (s, phase) = stock_pct_at(85, &input());
        assert_eq!(s, 30.0);
        assert_eq!(phase, "post_landing");
    }

    #[test]
    fn stock_midway_working_linear() {
        let (s, _phase) = stock_pct_at(45, &input());
        // 45 is halfway between 25 and 65 → halfway between 90 and 50 = 70.
        assert_eq!(s, 70.0);
    }

    #[test]
    fn stock_midway_to_landing_linear() {
        // 65 + ~3.5 = 68.5 — but ages are integers, so 68 or 69 picks.
        // Halfway between retire 65 and landing 72: age 68.5 → (50+30)/2 = 40.
        // Pick age 68: (68 − 65) / 7 = 3/7 → 50 + 3/7 × (30 − 50) = 50 − 8.57 = 41.43
        let (s, _phase) = stock_pct_at(68, &input());
        assert!((s - 41.428571).abs() < 1e-3);
    }

    #[test]
    fn stock_phase_labels() {
        assert_eq!(stock_pct_at(25, &input()).1, "pre");
        assert_eq!(stock_pct_at(50, &input()).1, "working");
        assert_eq!(stock_pct_at(68, &input()).1, "to_landing");
        assert_eq!(stock_pct_at(85, &input()).1, "post_landing");
    }

    #[test]
    fn compute_glide_row_count() {
        let r = compute(&input());
        // 25..=90 = 66 rows
        assert_eq!(r.glide.len(), 66);
    }

    #[test]
    fn compute_glide_monotonically_decreasing_stocks() {
        let r = compute(&input());
        for w in r.glide.windows(2) {
            assert!(w[1].stock_pct <= w[0].stock_pct + 1e-6,
                "expected non-increasing; got {} → {}",
                w[0].stock_pct, w[1].stock_pct);
        }
    }

    #[test]
    fn compute_glide_bond_plus_stock_100() {
        let r = compute(&input());
        for p in &r.glide {
            assert!((p.bond_pct + p.stock_pct - 100.0).abs() < 1e-6);
        }
    }

    #[test]
    fn compute_clamps_to_0_100() {
        let mut i = input();
        i.start_stock_pct = 150.0;
        i.landing_stock_pct = -30.0;
        let r = compute(&i);
        for p in &r.glide {
            assert!(p.stock_pct >= 0.0 && p.stock_pct <= 100.0);
            assert!(p.bond_pct >= 0.0 && p.bond_pct <= 100.0);
        }
    }
}
