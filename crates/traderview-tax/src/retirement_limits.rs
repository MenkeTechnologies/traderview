//! 2025 retirement-savings contribution limits and deductibility
//! phaseouts: Traditional IRA, Roth IRA, HSA.
//!
//! All numbers are from Notice 2024-80 (IRA / 401(k) cost-of-living
//! adjustments) and Rev. Proc. 2024-25 (HSA inflation adjustments).
//!
//! ## Traditional IRA — IRC § 219
//!
//! Base limit: **$7,000** for 2025, **+$1,000 catch-up** if age ≥ 50
//! (yielding $8,000 max). The contribution itself is always allowed
//! (up to earned income); the *deductibility* of the contribution
//! phases out when the taxpayer (or spouse) is covered by a workplace
//! retirement plan.
//!
//! Deductibility phaseout (2025 — Notice 2024-80):
//!   * **Single / HoH** *covered by workplace plan*: $79,000 – $89,000
//!   * **MFJ — taxpayer covered**:                  $126,000 – $146,000
//!   * **MFJ — taxpayer NOT covered, spouse IS**:   $236,000 – $246,000
//!   * **MFS — covered**:                            $0 – $10,000 (always partial)
//!   * **Neither spouse covered**:                   no phaseout (always fully deductible)
//!
//! ## Roth IRA — IRC § 408A
//!
//! Same $7,000 base + $1,000 catch-up limit applies *jointly* with
//! traditional IRA — total IRA contributions across all flavors can't
//! exceed the limit. Roth contribution *eligibility* (not just
//! deductibility — Roth has no deduction) phases out by MAGI:
//!
//!   * **Single / HoH**: $150,000 – $165,000
//!   * **MFJ**:         $236,000 – $246,000
//!   * **MFS** (lived with spouse): $0 – $10,000
//!
//! ## HSA — IRC § 223
//!
//! Requires enrollment in a High Deductible Health Plan (HDHP).
//! 2025 limits (Rev. Proc. 2024-25):
//!   * **Self-only HDHP**:   $4,300
//!   * **Family HDHP**:      $8,550
//!   * **Catch-up (≥ 55)**:  +$1,000
//!
//! No income phaseout. Limit pro-rates by months of HDHP coverage
//! (handled via `hdhp_months` input).

use crate::engine::FilingStatus;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

const IRA_BASE_LIMIT: i64 = 7_000;
const IRA_CATCH_UP: i64 = 1_000;
const HSA_SELF: i64 = 4_300;
const HSA_FAMILY: i64 = 8_550;
const HSA_CATCH_UP: i64 = 1_000;

#[derive(Debug, Clone, Copy)]
struct Window {
    start: Decimal,
    end: Decimal,
}

fn factor(magi: Decimal, w: Window) -> Decimal {
    if magi <= w.start {
        return Decimal::ONE;
    }
    if magi >= w.end {
        return Decimal::ZERO;
    }
    let span = w.end - w.start;
    Decimal::ONE - (magi - w.start) / span
}

fn round_to_10(amount: Decimal) -> Decimal {
    // IRA partial-deduction floor: rounded to nearest $10 per
    // IRS Pub 590-A worksheet 1-2 instructions.
    let ten = Decimal::from(10);
    (amount / ten).round_dp(0) * ten
}

// ── Traditional IRA ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IraInput {
    pub magi: Decimal,
    pub status: FilingStatus,
    pub age_50_or_older: bool,
    /// Taxpayer is an active participant in a workplace retirement plan.
    pub taxpayer_covered_by_workplace_plan: bool,
    /// Spouse is an active participant (only matters for MFJ).
    pub spouse_covered_by_workplace_plan: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct IraResult {
    /// Maximum contribution limit (base + catch-up). Always available
    /// up to earned income; this is the statutory cap.
    pub contribution_limit: Decimal,
    /// Maximum *deductible* portion after MAGI phaseout. Rounded to
    /// nearest $10 per Pub 590-A.
    pub max_deductible: Decimal,
    /// Phaseout factor (1.0 = fully deductible, 0.0 = no deduction).
    pub phaseout_factor: Decimal,
    /// True when no phaseout applies (no workplace coverage either spouse).
    pub no_phaseout_applies: bool,
}

pub fn ira(input: IraInput) -> IraResult {
    let contribution_limit = Decimal::from(if input.age_50_or_older {
        IRA_BASE_LIMIT + IRA_CATCH_UP
    } else {
        IRA_BASE_LIMIT
    });

    let neither_covered =
        !input.taxpayer_covered_by_workplace_plan && !input.spouse_covered_by_workplace_plan;

    if neither_covered {
        return IraResult {
            contribution_limit,
            max_deductible: contribution_limit,
            phaseout_factor: Decimal::ONE,
            no_phaseout_applies: true,
        };
    }

    let window = match (input.status, input.taxpayer_covered_by_workplace_plan) {
        (FilingStatus::Single | FilingStatus::Hoh, true) => Window {
            start: Decimal::from(79_000),
            end: Decimal::from(89_000),
        },
        (FilingStatus::Mfj, true) => Window {
            start: Decimal::from(126_000),
            end: Decimal::from(146_000),
        },
        // MFJ, taxpayer NOT covered but spouse IS.
        (FilingStatus::Mfj, false) => Window {
            start: Decimal::from(236_000),
            end: Decimal::from(246_000),
        },
        (FilingStatus::Mfs, _) => Window {
            start: Decimal::ZERO,
            end: Decimal::from(10_000),
        },
        // Single / HoH with no workplace coverage was already handled above.
        (FilingStatus::Single | FilingStatus::Hoh, false) => {
            return IraResult {
                contribution_limit,
                max_deductible: contribution_limit,
                phaseout_factor: Decimal::ONE,
                no_phaseout_applies: true,
            };
        }
    };

    let f = factor(input.magi, window);
    let pre_round = contribution_limit * f;
    // Floor at $200 when there's any partial deduction (Pub 590-A
    // safe harbor: if the phaseout result is between $0 and $200, round
    // up to $200).
    let max_deductible = if f > Decimal::ZERO && f < Decimal::ONE {
        let rounded = round_to_10(pre_round);
        rounded.max(Decimal::from(200)).min(contribution_limit)
    } else {
        (contribution_limit * f).round_dp(2)
    };

    IraResult {
        contribution_limit,
        max_deductible,
        phaseout_factor: f,
        no_phaseout_applies: false,
    }
}

// ── Roth IRA ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RothIraInput {
    pub magi: Decimal,
    pub status: FilingStatus,
    pub age_50_or_older: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct RothIraResult {
    /// Maximum allowed Roth contribution after MAGI phaseout.
    pub max_contribution: Decimal,
    /// The unphased statutory cap, for comparison.
    pub contribution_cap_if_no_phaseout: Decimal,
    pub phaseout_factor: Decimal,
}

pub fn roth_ira(input: RothIraInput) -> RothIraResult {
    let cap = Decimal::from(if input.age_50_or_older {
        IRA_BASE_LIMIT + IRA_CATCH_UP
    } else {
        IRA_BASE_LIMIT
    });

    let window = match input.status {
        FilingStatus::Single | FilingStatus::Hoh => Window {
            start: Decimal::from(150_000),
            end: Decimal::from(165_000),
        },
        FilingStatus::Mfj => Window {
            start: Decimal::from(236_000),
            end: Decimal::from(246_000),
        },
        // MFS who lived with spouse: $0-$10k phaseout.
        FilingStatus::Mfs => Window {
            start: Decimal::ZERO,
            end: Decimal::from(10_000),
        },
    };

    let f = factor(input.magi, window);
    let pre_round = cap * f;
    let max_contribution = if f > Decimal::ZERO && f < Decimal::ONE {
        let rounded = round_to_10(pre_round);
        rounded.max(Decimal::from(200)).min(cap)
    } else {
        (cap * f).round_dp(2)
    };

    RothIraResult {
        max_contribution,
        contribution_cap_if_no_phaseout: cap,
        phaseout_factor: f,
    }
}

// ── HSA ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HdhpCoverage {
    SelfOnly,
    Family,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HsaInput {
    pub coverage: HdhpCoverage,
    pub age_55_or_older: bool,
    /// Number of full months in 2025 the taxpayer was HDHP-covered (0-12).
    /// Limit pro-rates by months/12.
    pub hdhp_months: u32,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct HsaResult {
    /// Annual HDHP-tier limit (self-only or family).
    pub tier_annual_limit: Decimal,
    /// Catch-up addition (55+).
    pub catch_up: Decimal,
    /// Final allowed contribution, pro-rated by hdhp_months/12.
    pub max_contribution: Decimal,
}

pub fn hsa(input: HsaInput) -> HsaResult {
    let tier = Decimal::from(match input.coverage {
        HdhpCoverage::SelfOnly => HSA_SELF,
        HdhpCoverage::Family => HSA_FAMILY,
    });
    let catch_up = if input.age_55_or_older {
        Decimal::from(HSA_CATCH_UP)
    } else {
        Decimal::ZERO
    };
    let annual = tier + catch_up;
    let months = Decimal::from(input.hdhp_months.min(12));
    let twelve = Decimal::from(12);
    let max_contribution = (annual * months / twelve).round_dp(2);
    HsaResult {
        tier_annual_limit: tier,
        catch_up,
        max_contribution,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: i64) -> Decimal {
        Decimal::from(n)
    }
    fn dc(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    // ── Traditional IRA ─────────────────────────────────────────────────

    #[test]
    fn ira_uncovered_filer_fully_deductible_at_any_magi() {
        let r = ira(IraInput {
            magi: d(500_000),
            status: FilingStatus::Single,
            age_50_or_older: false,
            taxpayer_covered_by_workplace_plan: false,
            spouse_covered_by_workplace_plan: false,
        });
        assert!(r.no_phaseout_applies);
        assert_eq!(r.max_deductible, d(7_000));
    }

    #[test]
    fn ira_age_50_plus_gets_8k_limit() {
        let r = ira(IraInput {
            magi: d(50_000),
            status: FilingStatus::Single,
            age_50_or_older: true,
            taxpayer_covered_by_workplace_plan: false,
            spouse_covered_by_workplace_plan: false,
        });
        assert_eq!(r.contribution_limit, d(8_000));
        assert_eq!(r.max_deductible, d(8_000));
    }

    #[test]
    fn ira_single_covered_phases_out_79k_to_89k() {
        // MAGI $84,000 → halfway through window → factor 0.5.
        // $7,000 × 0.5 = $3,500. Rounded to nearest $10 = $3,500.
        let r = ira(IraInput {
            magi: d(84_000),
            status: FilingStatus::Single,
            age_50_or_older: false,
            taxpayer_covered_by_workplace_plan: true,
            spouse_covered_by_workplace_plan: false,
        });
        assert_eq!(r.phaseout_factor, dc("0.5"));
        assert_eq!(r.max_deductible, d(3_500));
    }

    #[test]
    fn ira_single_covered_at_top_of_window_zero_deductible() {
        let r = ira(IraInput {
            magi: d(89_000),
            status: FilingStatus::Single,
            age_50_or_older: false,
            taxpayer_covered_by_workplace_plan: true,
            spouse_covered_by_workplace_plan: false,
        });
        assert_eq!(r.max_deductible, Decimal::ZERO);
    }

    #[test]
    fn ira_mfj_taxpayer_covered_uses_126k_146k() {
        // MFJ, taxpayer covered, MAGI $136k → midpoint → factor 0.5.
        let r = ira(IraInput {
            magi: d(136_000),
            status: FilingStatus::Mfj,
            age_50_or_older: false,
            taxpayer_covered_by_workplace_plan: true,
            spouse_covered_by_workplace_plan: false,
        });
        assert_eq!(r.phaseout_factor, dc("0.5"));
    }

    #[test]
    fn ira_mfj_only_spouse_covered_uses_higher_236k_window() {
        let r = ira(IraInput {
            magi: d(241_000),
            status: FilingStatus::Mfj,
            age_50_or_older: false,
            taxpayer_covered_by_workplace_plan: false,
            spouse_covered_by_workplace_plan: true,
        });
        assert_eq!(r.phaseout_factor, dc("0.5"));
    }

    #[test]
    fn ira_partial_deduction_floors_at_200_when_near_top() {
        // MAGI very close to top → phased almost completely, but the
        // Pub 590-A worksheet rounds up to $200 when any partial
        // deduction exists.
        let r = ira(IraInput {
            magi: d(88_900),
            status: FilingStatus::Single,
            age_50_or_older: false,
            taxpayer_covered_by_workplace_plan: true,
            spouse_covered_by_workplace_plan: false,
        });
        // $7000 × 0.01 = $70 → rounded to $70 → bumped up to $200.
        assert_eq!(r.max_deductible, d(200));
    }

    #[test]
    fn ira_mfs_covered_has_narrow_window() {
        // MFS, MAGI $5,000 → halfway through $0-$10k.
        let r = ira(IraInput {
            magi: d(5_000),
            status: FilingStatus::Mfs,
            age_50_or_older: false,
            taxpayer_covered_by_workplace_plan: true,
            spouse_covered_by_workplace_plan: false,
        });
        assert_eq!(r.phaseout_factor, dc("0.5"));
    }

    // ── Roth IRA ────────────────────────────────────────────────────────

    #[test]
    fn roth_single_under_150k_full_limit() {
        let r = roth_ira(RothIraInput {
            magi: d(100_000),
            status: FilingStatus::Single,
            age_50_or_older: false,
        });
        assert_eq!(r.max_contribution, d(7_000));
        assert_eq!(r.phaseout_factor, Decimal::ONE);
    }

    #[test]
    fn roth_single_above_165k_zero() {
        let r = roth_ira(RothIraInput {
            magi: d(180_000),
            status: FilingStatus::Single,
            age_50_or_older: false,
        });
        assert_eq!(r.max_contribution, Decimal::ZERO);
    }

    #[test]
    fn roth_single_at_157500_half_phase_out() {
        // Midpoint $150k-$165k → factor 0.5.
        let r = roth_ira(RothIraInput {
            magi: d(157_500),
            status: FilingStatus::Single,
            age_50_or_older: false,
        });
        assert_eq!(r.phaseout_factor, dc("0.5"));
        assert_eq!(r.max_contribution, d(3_500));
    }

    #[test]
    fn roth_mfj_window_is_236k_to_246k() {
        let r = roth_ira(RothIraInput {
            magi: d(241_000),
            status: FilingStatus::Mfj,
            age_50_or_older: false,
        });
        assert_eq!(r.phaseout_factor, dc("0.5"));
    }

    #[test]
    fn roth_age_50_plus_gets_8k_cap() {
        let r = roth_ira(RothIraInput {
            magi: d(100_000),
            status: FilingStatus::Single,
            age_50_or_older: true,
        });
        assert_eq!(r.contribution_cap_if_no_phaseout, d(8_000));
        assert_eq!(r.max_contribution, d(8_000));
    }

    // ── HSA ─────────────────────────────────────────────────────────────

    #[test]
    fn hsa_self_full_year() {
        let r = hsa(HsaInput {
            coverage: HdhpCoverage::SelfOnly,
            age_55_or_older: false,
            hdhp_months: 12,
        });
        assert_eq!(r.max_contribution, d(4_300));
    }

    #[test]
    fn hsa_family_full_year() {
        let r = hsa(HsaInput {
            coverage: HdhpCoverage::Family,
            age_55_or_older: false,
            hdhp_months: 12,
        });
        assert_eq!(r.max_contribution, d(8_550));
    }

    #[test]
    fn hsa_55_plus_adds_1000_catch_up() {
        let r = hsa(HsaInput {
            coverage: HdhpCoverage::SelfOnly,
            age_55_or_older: true,
            hdhp_months: 12,
        });
        assert_eq!(r.max_contribution, d(5_300));
    }

    #[test]
    fn hsa_partial_year_prorates_to_month() {
        // 6 months of self-only HDHP → $4,300 × 6 / 12 = $2,150.
        let r = hsa(HsaInput {
            coverage: HdhpCoverage::SelfOnly,
            age_55_or_older: false,
            hdhp_months: 6,
        });
        assert_eq!(r.max_contribution, d(2_150));
    }

    #[test]
    fn hsa_zero_months_zero_contribution() {
        let r = hsa(HsaInput {
            coverage: HdhpCoverage::Family,
            age_55_or_older: false,
            hdhp_months: 0,
        });
        assert_eq!(r.max_contribution, Decimal::ZERO);
    }

    #[test]
    fn hsa_months_clamped_to_12() {
        // Defensive: input of 99 months → use 12.
        let r = hsa(HsaInput {
            coverage: HdhpCoverage::SelfOnly,
            age_55_or_older: false,
            hdhp_months: 99,
        });
        assert_eq!(r.max_contribution, d(4_300));
    }
}
