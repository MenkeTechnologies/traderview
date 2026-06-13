//! Roth IRA contribution limit after the MAGI phase-out.
//!
//! Above a modified-AGI threshold the annual Roth contribution phases down to
//! zero. The reduction follows IRS Pub 590-A Worksheet 2-2:
//!
//! ```text
//! reduced = limit × (1 − (MAGI − start) / (end − start))
//! ```
//!
//! then rounded UP to the next $10, with a $200 floor whenever any
//! contribution is allowed. 2026 defaults: a $7,500 base limit (+$1,100 catch-up
//! at age 50+), phase-out $153k–$168k single, $242k–$252k married-joint,
//! $0–$10k married-separate. The limit and catch-up are overridable for other
//! years.

use serde::{Deserialize, Serialize};

fn d_base() -> f64 {
    7_500.0
}
fn d_catchup() -> f64 {
    1_100.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    /// Single or head of household.
    Single,
    /// Married filing jointly (or qualifying widow(er)).
    MarriedJoint,
    /// Married filing separately (lived with spouse).
    MarriedSeparate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RothInput {
    pub magi_usd: f64,
    pub filing_status: FilingStatus,
    pub age_50_plus: bool,
    #[serde(default = "d_base")]
    pub base_limit_usd: f64,
    #[serde(default = "d_catchup")]
    pub catch_up_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RothResult {
    /// Limit before phase-out (base + catch-up if 50+).
    pub max_contribution_usd: f64,
    /// Contribution actually allowed at this MAGI.
    pub allowed_contribution_usd: f64,
    /// max − allowed.
    pub disallowed_usd: f64,
    /// Phase-out lower bound for the filing status.
    pub phaseout_start_usd: f64,
    /// Phase-out upper bound for the filing status.
    pub phaseout_end_usd: f64,
    /// "full", "partial", or "none".
    pub status: String,
}

fn bounds(status: FilingStatus) -> (f64, f64) {
    match status {
        FilingStatus::Single => (153_000.0, 168_000.0),
        FilingStatus::MarriedJoint => (242_000.0, 252_000.0),
        FilingStatus::MarriedSeparate => (0.0, 10_000.0),
    }
}

pub fn analyze(input: &RothInput) -> RothResult {
    let max_limit = input.base_limit_usd + if input.age_50_plus { input.catch_up_usd } else { 0.0 };
    let (start, end) = bounds(input.filing_status);

    let (allowed, status) = if input.magi_usd <= start {
        (max_limit, "full")
    } else if input.magi_usd >= end {
        (0.0, "none")
    } else {
        let range = end - start;
        let raw = max_limit * (1.0 - (input.magi_usd - start) / range);
        // IRS rounding: up to the next $10, then a $200 floor if any is allowed.
        let mut reduced = (raw / 10.0).ceil() * 10.0;
        if reduced > 0.0 && reduced < 200.0 {
            reduced = 200.0;
        }
        (reduced, "partial")
    };

    RothResult {
        max_contribution_usd: max_limit,
        allowed_contribution_usd: allowed,
        disallowed_usd: max_limit - allowed,
        phaseout_start_usd: start,
        phaseout_end_usd: end,
        status: status.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn run(magi: f64, status: FilingStatus, age50: bool) -> RothResult {
        analyze(&RothInput {
            magi_usd: magi,
            filing_status: status,
            age_50_plus: age50,
            base_limit_usd: 7_500.0,
            catch_up_usd: 1_100.0,
        })
    }

    #[test]
    fn below_threshold_full() {
        let r = run(100_000.0, FilingStatus::Single, false);
        assert!(close(r.allowed_contribution_usd, 7_500.0));
        assert_eq!(r.status, "full");
        assert!(close(r.disallowed_usd, 0.0));
    }

    #[test]
    fn above_ceiling_none() {
        let r = run(200_000.0, FilingStatus::Single, false);
        assert!(close(r.allowed_contribution_usd, 0.0));
        assert_eq!(r.status, "none");
        assert!(close(r.disallowed_usd, 7_500.0));
    }

    #[test]
    fn single_midpoint_half() {
        // MAGI 160,500 → excess 7,500 / range 15,000 = 0.5 → 3,750.
        let r = run(160_500.0, FilingStatus::Single, false);
        assert!(close(r.allowed_contribution_usd, 3_750.0));
        assert_eq!(r.status, "partial");
    }

    #[test]
    fn catch_up_raises_limit() {
        let r = run(100_000.0, FilingStatus::Single, true);
        assert!(close(r.max_contribution_usd, 8_600.0));
        assert!(close(r.allowed_contribution_usd, 8_600.0));
    }

    #[test]
    fn married_joint_midpoint() {
        // MAGI 247,000 → excess 5,000 / range 10,000 = 0.5 → 3,750.
        let r = run(247_000.0, FilingStatus::MarriedJoint, false);
        assert!(close(r.allowed_contribution_usd, 3_750.0));
        assert!(close(r.phaseout_start_usd, 242_000.0));
    }

    #[test]
    fn married_separate_tight_range() {
        let r = run(5_000.0, FilingStatus::MarriedSeparate, false);
        assert!(close(r.allowed_contribution_usd, 3_750.0));
        assert!(close(r.phaseout_end_usd, 10_000.0));
    }

    #[test]
    fn two_hundred_dollar_floor() {
        // MAGI 167,800 → raw ≈ $100, below the $200 floor → bumped to $200.
        let r = run(167_800.0, FilingStatus::Single, false);
        assert!(close(r.allowed_contribution_usd, 200.0));
    }

    #[test]
    fn rounds_up_to_next_ten() {
        // MAGI 161,001 → raw 3,499.50 → rounds UP to 3,500.
        let r = run(161_001.0, FilingStatus::Single, false);
        assert!(close(r.allowed_contribution_usd, 3_500.0));
    }
}
