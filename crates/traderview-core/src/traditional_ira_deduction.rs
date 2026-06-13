//! Traditional IRA deduction after the MAGI phase-out.
//!
//! Whether (and how much of) a traditional IRA contribution is deductible
//! depends on workplace-plan coverage and MAGI:
//!
//! * Not an active participant, and no covered spouse → fully deductible at any
//!   income.
//! * Active participant → phases out by MAGI for the filing status.
//! * Not covered but spouse is (married) → a much higher phase-out range.
//!
//! The partial deduction follows the same IRS proration as the Roth limit
//! (round the reduced figure UP to the next $10, $200 floor if any is allowed).
//! 2026 ranges: covered single $81k–$91k, covered married-joint $129k–$149k,
//! spouse-covered $242k–$252k, married-separate $0–$10k. Below the range is a
//! full deduction; above it, none (you may still contribute nondeductibly).

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
    Single,
    MarriedJoint,
    MarriedSeparate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradIraInput {
    pub magi_usd: f64,
    pub filing_status: FilingStatus,
    /// The filer is an active participant in a workplace retirement plan.
    pub covered_by_plan: bool,
    /// The filer is not covered, but their spouse is (married filers).
    #[serde(default)]
    pub spouse_covered: bool,
    pub age_50_plus: bool,
    #[serde(default = "d_base")]
    pub base_limit_usd: f64,
    #[serde(default = "d_catchup")]
    pub catch_up_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TradIraResult {
    /// Contribution limit (base + catch-up if 50+).
    pub max_contribution_usd: f64,
    /// Deductible portion at this MAGI.
    pub deductible_usd: f64,
    /// Contribution that can still be made, but nondeductibly (max − deductible).
    pub nondeductible_usd: f64,
    /// Phase-out lower bound (0 when no phase-out applies).
    pub phaseout_start_usd: f64,
    /// Phase-out upper bound (0 when no phase-out applies).
    pub phaseout_end_usd: f64,
    /// "full", "partial", or "none".
    pub status: String,
}

/// Phase-out bounds, or `None` when the contribution is fully deductible
/// regardless of income (not covered, and no covered spouse).
fn bounds(status: FilingStatus, covered: bool, spouse_covered: bool) -> Option<(f64, f64)> {
    if covered {
        Some(match status {
            FilingStatus::Single => (81_000.0, 91_000.0),
            FilingStatus::MarriedJoint => (129_000.0, 149_000.0),
            FilingStatus::MarriedSeparate => (0.0, 10_000.0),
        })
    } else if spouse_covered {
        match status {
            FilingStatus::MarriedJoint => Some((242_000.0, 252_000.0)),
            FilingStatus::MarriedSeparate => Some((0.0, 10_000.0)),
            // A single filer has no spouse — fully deductible.
            FilingStatus::Single => None,
        }
    } else {
        None
    }
}

pub fn analyze(input: &TradIraInput) -> TradIraResult {
    let max_limit = input.base_limit_usd + if input.age_50_plus { input.catch_up_usd } else { 0.0 };

    let (deductible, status, start, end) =
        match bounds(input.filing_status, input.covered_by_plan, input.spouse_covered) {
            None => (max_limit, "full", 0.0, 0.0),
            Some((start, end)) => {
                if input.magi_usd <= start {
                    (max_limit, "full", start, end)
                } else if input.magi_usd >= end {
                    (0.0, "none", start, end)
                } else {
                    let range = end - start;
                    let raw = max_limit * (1.0 - (input.magi_usd - start) / range);
                    let mut reduced = (raw / 10.0).ceil() * 10.0;
                    if reduced > 0.0 && reduced < 200.0 {
                        reduced = 200.0;
                    }
                    (reduced, "partial", start, end)
                }
            }
        };

    TradIraResult {
        max_contribution_usd: max_limit,
        deductible_usd: deductible,
        nondeductible_usd: max_limit - deductible,
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

    fn run(magi: f64, status: FilingStatus, covered: bool, spouse_covered: bool) -> TradIraResult {
        analyze(&TradIraInput {
            magi_usd: magi,
            filing_status: status,
            covered_by_plan: covered,
            spouse_covered,
            age_50_plus: false,
            base_limit_usd: 7_500.0,
            catch_up_usd: 1_100.0,
        })
    }

    #[test]
    fn not_covered_full_at_any_income() {
        let r = run(500_000.0, FilingStatus::Single, false, false);
        assert!(close(r.deductible_usd, 7_500.0));
        assert_eq!(r.status, "full");
        assert!(close(r.phaseout_start_usd, 0.0));
    }

    #[test]
    fn covered_single_below_range_full() {
        let r = run(70_000.0, FilingStatus::Single, true, false);
        assert!(close(r.deductible_usd, 7_500.0));
        assert_eq!(r.status, "full");
    }

    #[test]
    fn covered_single_above_range_none() {
        let r = run(95_000.0, FilingStatus::Single, true, false);
        assert!(close(r.deductible_usd, 0.0));
        assert_eq!(r.status, "none");
        assert!(close(r.nondeductible_usd, 7_500.0));
    }

    #[test]
    fn covered_single_midpoint() {
        // 86,000 → excess 5,000 / range 10,000 = 0.5 → 3,750.
        let r = run(86_000.0, FilingStatus::Single, true, false);
        assert!(close(r.deductible_usd, 3_750.0));
        assert_eq!(r.status, "partial");
    }

    #[test]
    fn covered_married_joint_midpoint() {
        // 139,000 → excess 10,000 / range 20,000 = 0.5 → 3,750.
        let r = run(139_000.0, FilingStatus::MarriedJoint, true, false);
        assert!(close(r.deductible_usd, 3_750.0));
        assert!(close(r.phaseout_end_usd, 149_000.0));
    }

    #[test]
    fn spouse_covered_higher_range() {
        // Filer not covered, spouse is: 247,000 → mid of 242k–252k → 3,750.
        let r = run(247_000.0, FilingStatus::MarriedJoint, false, true);
        assert!(close(r.deductible_usd, 3_750.0));
        assert!(close(r.phaseout_start_usd, 242_000.0));
    }

    #[test]
    fn catch_up_raises_limit() {
        let r = analyze(&TradIraInput {
            magi_usd: 70_000.0,
            filing_status: FilingStatus::Single,
            covered_by_plan: true,
            spouse_covered: false,
            age_50_plus: true,
            base_limit_usd: 7_500.0,
            catch_up_usd: 1_100.0,
        });
        assert!(close(r.max_contribution_usd, 8_600.0));
        assert!(close(r.deductible_usd, 8_600.0));
    }

    #[test]
    fn two_hundred_floor() {
        // Covered single, MAGI 90,800 → raw ≈ $150 → floored to $200.
        let r = run(90_800.0, FilingStatus::Single, true, false);
        assert!(close(r.deductible_usd, 200.0));
    }
}
