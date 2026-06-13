//! IRMAA — Medicare Income-Related Monthly Adjustment Amount (2026).
//!
//! High-income Medicare beneficiaries pay a surcharge on top of the
//! standard Part B and Part D premiums. The surcharge is set by a **cliff**
//! schedule: cross a Modified AGI threshold by a single dollar and you owe
//! the full higher tier for the whole year — there is no phase-in. IRMAA is
//! assessed per enrolled individual, on a **two-year lookback** (2026
//! premiums use 2024 MAGI), which is why Roth conversions, capital gains,
//! and RMDs taken two years before age 65/67 quietly raise these premiums.
//!
//! Three filing schedules apply. Married-filing-separately (while living
//! with a spouse) uses a compressed schedule with only the standard tier
//! and the top two surcharge tiers. Figures below are the 2026 amounts
//! (standard Part B premium $202.90/mo); brackets are inflation-adjusted
//! each year. Pure compute.

use serde::{Deserialize, Serialize};

/// The premium year these brackets apply to. IRMAA uses the MAGI from two
/// calendar years earlier (2026 premiums are set on 2024 MAGI).
pub const PREMIUM_YEAR: u16 = 2026;
pub const MAGI_LOOKBACK_YEAR: u16 = 2024;

/// Standard (tier-0) Part B monthly premium for [`PREMIUM_YEAR`].
pub const STANDARD_PART_B_USD: f64 = 202.90;

/// Per-tier Part B total monthly premium (CMS multiples × standard).
const PART_B_MONTHLY: [f64; 6] = [202.90, 284.06, 405.80, 527.54, 649.28, 690.19];
/// Per-tier Part D monthly IRMAA surcharge (added to the plan premium).
const PART_D_SURCHARGE: [f64; 6] = [0.0, 14.50, 37.50, 60.40, 83.30, 91.00];

/// Upper MAGI bound (inclusive) of each tier; the last tier is unbounded.
const SINGLE_UPPER: [f64; 5] = [109_000.0, 137_000.0, 171_000.0, 205_000.0, 500_000.0];
const JOINT_UPPER: [f64; 5] = [218_000.0, 274_000.0, 342_000.0, 410_000.0, 750_000.0];
/// MFS (living with spouse): standard up to $109k, tier 4 up to $391k, then tier 5.
const SEPARATE_STD_UPPER: f64 = 109_000.0;
const SEPARATE_T4_UPPER: f64 = 391_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedJoint,
    MarriedSeparate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IrmaaInput {
    /// Modified AGI from the lookback year (two years before the premium year).
    pub magi_usd: f64,
    pub filing_status: FilingStatus,
    /// MFJ only: are both spouses enrolled in Medicare? Doubles the household total.
    #[serde(default)]
    pub both_spouses_on_medicare: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct IrmaaResult {
    /// IRMAA tier 0 (no surcharge) through 5 (highest).
    pub tier: u8,
    /// Lower MAGI bound of the tier (exclusive above tier 0).
    pub lower_threshold_usd: f64,
    /// Upper MAGI bound of the tier; `None` for the unbounded top tier.
    pub upper_threshold_usd: Option<f64>,
    /// MAGI headroom before the next cliff; `None` at the top tier.
    pub headroom_to_next_cliff_usd: Option<f64>,
    /// Total Part B monthly premium at this tier.
    pub part_b_monthly_usd: f64,
    /// Part B surcharge over the standard premium.
    pub part_b_surcharge_usd: f64,
    /// Part D monthly IRMAA surcharge (added to the drug-plan premium).
    pub part_d_surcharge_usd: f64,
    /// Part B + Part D surcharge, per person, per month.
    pub monthly_surcharge_usd: f64,
    /// Per-person annual surcharge.
    pub annual_surcharge_usd: f64,
    /// Household annual surcharge (×2 when MFJ and both are on Medicare).
    pub household_annual_surcharge_usd: f64,
}

/// Resolve (tier, lower bound, upper bound) for a MAGI under a filing status.
fn classify(magi: f64, status: FilingStatus) -> (u8, f64, Option<f64>) {
    match status {
        FilingStatus::Single | FilingStatus::MarriedJoint => {
            let upper = if status == FilingStatus::Single { &SINGLE_UPPER } else { &JOINT_UPPER };
            for (i, &cap) in upper.iter().enumerate() {
                if magi <= cap {
                    let lo = if i == 0 { 0.0 } else { upper[i - 1] };
                    return (i as u8, lo, Some(cap));
                }
            }
            // Above the last finite cap → top tier (index 5), unbounded.
            (5, upper[4], None)
        }
        FilingStatus::MarriedSeparate => {
            if magi <= SEPARATE_STD_UPPER {
                (0, 0.0, Some(SEPARATE_STD_UPPER))
            } else if magi < SEPARATE_T4_UPPER {
                (4, SEPARATE_STD_UPPER, Some(SEPARATE_T4_UPPER))
            } else {
                (5, SEPARATE_T4_UPPER, None)
            }
        }
    }
}

pub fn compute(i: &IrmaaInput) -> IrmaaResult {
    let magi = i.magi_usd.max(0.0);
    let (tier, lower, upper) = classify(magi, i.filing_status);
    let t = tier as usize;

    let part_b_monthly = PART_B_MONTHLY[t];
    let part_b_surcharge = part_b_monthly - STANDARD_PART_B_USD;
    let part_d_surcharge = PART_D_SURCHARGE[t];
    let monthly_surcharge = part_b_surcharge + part_d_surcharge;
    let annual_surcharge = monthly_surcharge * 12.0;

    let multiplier =
        if i.filing_status == FilingStatus::MarriedJoint && i.both_spouses_on_medicare {
            2.0
        } else {
            1.0
        };

    IrmaaResult {
        tier,
        lower_threshold_usd: lower,
        upper_threshold_usd: upper,
        headroom_to_next_cliff_usd: upper.map(|u| (u - magi).max(0.0)),
        part_b_monthly_usd: part_b_monthly,
        part_b_surcharge_usd: part_b_surcharge,
        part_d_surcharge_usd: part_d_surcharge,
        monthly_surcharge_usd: monthly_surcharge,
        annual_surcharge_usd: annual_surcharge,
        household_annual_surcharge_usd: annual_surcharge * multiplier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(magi: f64, status: FilingStatus) -> IrmaaInput {
        IrmaaInput { magi_usd: magi, filing_status: status, both_spouses_on_medicare: false }
    }

    #[test]
    fn standard_tier_has_no_surcharge() {
        let r = compute(&inp(80_000.0, FilingStatus::Single));
        assert_eq!(r.tier, 0);
        assert!((r.part_b_monthly_usd - 202.90).abs() < 1e-9);
        assert!(r.monthly_surcharge_usd.abs() < 1e-9);
        assert!(r.annual_surcharge_usd.abs() < 1e-9);
    }

    #[test]
    fn single_tier_1_amounts() {
        // $120k single → tier 1: Part B 284.06 (surcharge 81.16) + Part D 14.50.
        let r = compute(&inp(120_000.0, FilingStatus::Single));
        assert_eq!(r.tier, 1);
        assert!((r.part_b_surcharge_usd - 81.16).abs() < 1e-9);
        assert!((r.part_d_surcharge_usd - 14.50).abs() < 1e-9);
        assert!((r.monthly_surcharge_usd - 95.66).abs() < 1e-9);
        assert!((r.annual_surcharge_usd - 95.66 * 12.0).abs() < 1e-9);
    }

    #[test]
    fn cliff_is_inclusive_lower_exclusive_upper() {
        // Exactly at $109k → still tier 0; one dollar over → tier 1.
        assert_eq!(compute(&inp(109_000.0, FilingStatus::Single)).tier, 0);
        assert_eq!(compute(&inp(109_000.01, FilingStatus::Single)).tier, 1);
        // Exactly at $137k → tier 1; just over → tier 2.
        assert_eq!(compute(&inp(137_000.0, FilingStatus::Single)).tier, 1);
        assert_eq!(compute(&inp(137_000.01, FilingStatus::Single)).tier, 2);
    }

    #[test]
    fn joint_thresholds_double_the_single_brackets() {
        // $200k MFJ is under the $218k standard cap → tier 0.
        assert_eq!(compute(&inp(200_000.0, FilingStatus::MarriedJoint)).tier, 0);
        // $250k MFJ → tier 1.
        assert_eq!(compute(&inp(250_000.0, FilingStatus::MarriedJoint)).tier, 1);
    }

    #[test]
    fn joint_household_doubles_when_both_enrolled() {
        let one = compute(&inp(250_000.0, FilingStatus::MarriedJoint));
        let both = compute(&IrmaaInput {
            magi_usd: 250_000.0,
            filing_status: FilingStatus::MarriedJoint,
            both_spouses_on_medicare: true,
        });
        assert!((both.household_annual_surcharge_usd - 2.0 * one.annual_surcharge_usd).abs() < 1e-9);
        // Single-enrolled household equals the per-person amount.
        assert!((one.household_annual_surcharge_usd - one.annual_surcharge_usd).abs() < 1e-9);
    }

    #[test]
    fn top_tier_is_unbounded_with_max_amounts() {
        let r = compute(&inp(600_000.0, FilingStatus::Single));
        assert_eq!(r.tier, 5);
        assert!((r.part_b_monthly_usd - 690.19).abs() < 1e-9);
        assert!((r.part_d_surcharge_usd - 91.00).abs() < 1e-9);
        assert!(r.upper_threshold_usd.is_none());
        assert!(r.headroom_to_next_cliff_usd.is_none());
    }

    #[test]
    fn married_separate_uses_compressed_schedule() {
        // MFS jumps straight from standard to tier 4, then tier 5.
        assert_eq!(compute(&inp(90_000.0, FilingStatus::MarriedSeparate)).tier, 0);
        assert_eq!(compute(&inp(150_000.0, FilingStatus::MarriedSeparate)).tier, 4);
        assert_eq!(compute(&inp(400_000.0, FilingStatus::MarriedSeparate)).tier, 5);
    }

    #[test]
    fn headroom_counts_down_to_the_next_cliff() {
        // $130k single, tier 1 cap $137k → $7k of headroom.
        let r = compute(&inp(130_000.0, FilingStatus::Single));
        assert_eq!(r.tier, 1);
        assert!((r.headroom_to_next_cliff_usd.unwrap() - 7_000.0).abs() < 1e-9);
    }
}
