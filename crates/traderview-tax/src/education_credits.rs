//! Education tax credits — IRC § 25A (Form 8863).
//!
//! Two flavors, mutually exclusive *per student per year*:
//!
//! ### American Opportunity Tax Credit (AOTC) — § 25A(b)
//!
//! * First four years of post-secondary education.
//! * Per-student credit: 100% of first $2,000 + 25% of next $2,000 →
//!   **$2,500 maximum per qualifying student**.
//! * **40% refundable** — up to $1,000 of the per-student credit can
//!   reduce tax below zero (i.e., increase refund). The remaining 60%
//!   is non-refundable.
//! * MAGI phaseout (NOT inflation-adjusted, fixed by statute):
//!   * Single / HoH:  $80,000 – $90,000 (phaseout window $10,000)
//!   * MFJ:           $160,000 – $180,000 (phaseout window $20,000)
//!   * MFS:           NOT ALLOWED
//!
//! ### Lifetime Learning Credit (LLC) — § 25A(c)
//!
//! * Any post-secondary education — undergrad, grad, professional,
//!   continuing-ed. No four-year cap.
//! * **20% of up to $10,000 of qualified expenses → $2,000 maximum
//!   per RETURN** (not per student — pooled across the household).
//! * **Fully non-refundable**.
//! * MAGI phaseout (aligned with AOTC after 2020 SECURE Act):
//!   * Single / HoH:  $80,000 – $90,000
//!   * MFJ:           $160,000 – $180,000
//!   * MFS:           NOT ALLOWED
//!
//! ### Anti-double-dipping rule
//!
//! A student CANNOT be used to claim BOTH AOTC and LLC in the same
//! tax year. The taxpayer picks per-student. This module computes
//! both candidate credits independently and the caller decides which
//! to claim — typically AOTC > LLC for undergrads in their first 4
//! years (because AOTC's first $2k is 100% vs LLC's 20%).
//!
//! Sources:
//!   * IRC § 25A
//!   * IRS Pub 970 (Tax Benefits for Education)
//!   * Form 8863 instructions

use crate::engine::FilingStatus;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
struct PhaseoutWindow {
    /// MAGI at which the credit starts phasing out (full credit below this).
    start: Decimal,
    /// MAGI at which the credit is fully phased out.
    end: Decimal,
}

fn phaseout(status: FilingStatus) -> Option<PhaseoutWindow> {
    match status {
        FilingStatus::Single | FilingStatus::Hoh => Some(PhaseoutWindow {
            start: Decimal::from(80_000),
            end: Decimal::from(90_000),
        }),
        FilingStatus::Mfj => Some(PhaseoutWindow {
            start: Decimal::from(160_000),
            end: Decimal::from(180_000),
        }),
        // MFS is statutorily ineligible.
        FilingStatus::Mfs => None,
    }
}

/// Phaseout multiplier in [0.0, 1.0]. 1.0 below `start`, 0.0 at-and-above
/// `end`, linear in between.
fn phaseout_factor(magi: Decimal, w: PhaseoutWindow) -> Decimal {
    if magi <= w.start {
        return Decimal::ONE;
    }
    if magi >= w.end {
        return Decimal::ZERO;
    }
    let span = w.end - w.start;
    let above_start = magi - w.start;
    let phased_out_fraction = above_start / span;
    Decimal::ONE - phased_out_fraction
}

// ── AOTC ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AotcInput {
    /// Total qualifying education expenses across all AOTC-eligible
    /// students (tuition, fees, course materials). Capped at $4,000 per
    /// student for AOTC purposes when the caller computes.
    pub qualifying_expenses: Decimal,
    /// Number of AOTC-eligible students in the household.
    pub eligible_students: u32,
    /// Modified AGI (≈ AGI for most filers).
    pub magi: Decimal,
    pub status: FilingStatus,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct AotcResult {
    /// Total credit before refundability split. After phaseout.
    pub total: Decimal,
    /// 40% refundable portion (≤ $1,000 × eligible students).
    pub refundable_portion: Decimal,
    /// 60% non-refundable portion.
    pub nonrefundable_portion: Decimal,
    /// Phaseout factor applied (1.0 = full credit, 0.0 = none).
    pub phaseout_factor: Decimal,
    /// True when filing status forbids the credit (MFS).
    pub status_ineligible: bool,
}

pub fn aotc(input: AotcInput) -> AotcResult {
    let Some(window) = phaseout(input.status) else {
        return AotcResult {
            status_ineligible: true,
            ..Default::default()
        };
    };
    if input.eligible_students == 0 {
        return AotcResult::default();
    }

    let factor = phaseout_factor(input.magi, window);

    // Per-student credit: 100% of first $2,000 + 25% of next $2,000,
    // applied to the *average* per-student expense. Form 8863 treats
    // each student separately — we approximate by dividing the bulk
    // expense input evenly.
    let students = Decimal::from(input.eligible_students);
    let per_student_expense = (input.qualifying_expenses / students).max(Decimal::ZERO);
    let capped: Decimal = per_student_expense.min(Decimal::from(4_000));
    let first_2k = capped.min(Decimal::from(2_000));
    let next_2k = (capped - first_2k).max(Decimal::ZERO);
    let quarter: Decimal = "0.25".parse().unwrap();
    let per_student_credit = first_2k + next_2k * quarter;

    let pre_phaseout = (per_student_credit * students).round_dp(2);
    let total = (pre_phaseout * factor).round_dp(2);

    let refundable_rate: Decimal = "0.40".parse().unwrap();
    let refundable_portion = (total * refundable_rate).round_dp(2);
    let nonrefundable_portion = (total - refundable_portion).round_dp(2);

    AotcResult {
        total,
        refundable_portion,
        nonrefundable_portion,
        phaseout_factor: factor,
        status_ineligible: false,
    }
}

// ── LLC ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LlcInput {
    /// Total qualifying expenses across the household. Capped at
    /// $10,000 for the 20% rate (yielding the $2,000 maximum credit).
    pub qualifying_expenses: Decimal,
    pub magi: Decimal,
    pub status: FilingStatus,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct LlcResult {
    /// Total credit after phaseout. Fully non-refundable.
    pub total: Decimal,
    pub phaseout_factor: Decimal,
    pub status_ineligible: bool,
}

pub fn llc(input: LlcInput) -> LlcResult {
    let Some(window) = phaseout(input.status) else {
        return LlcResult {
            status_ineligible: true,
            ..Default::default()
        };
    };
    let factor = phaseout_factor(input.magi, window);

    let capped_expenses = input
        .qualifying_expenses
        .max(Decimal::ZERO)
        .min(Decimal::from(10_000));
    let rate: Decimal = "0.20".parse().unwrap();
    let pre_phaseout = (capped_expenses * rate).round_dp(2);
    let total = (pre_phaseout * factor).round_dp(2);

    LlcResult {
        total,
        phaseout_factor: factor,
        status_ineligible: false,
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

    // ── AOTC tests ──────────────────────────────────────────────────────

    #[test]
    fn aotc_single_student_max_credit() {
        // $4,000 expenses → 100% of $2k + 25% of $2k = $2,500. Single
        // filer at $50k MAGI: no phaseout. 40% refundable = $1,000.
        let r = aotc(AotcInput {
            qualifying_expenses: d(4_000),
            eligible_students: 1,
            magi: d(50_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, dc("2500"));
        assert_eq!(r.refundable_portion, dc("1000"));
        assert_eq!(r.nonrefundable_portion, dc("1500"));
        assert_eq!(r.phaseout_factor, Decimal::ONE);
    }

    #[test]
    fn aotc_two_students_doubles_credit() {
        // $8,000 expenses split across 2 students → $4,000 each →
        // $2,500 per student × 2 = $5,000.
        let r = aotc(AotcInput {
            qualifying_expenses: d(8_000),
            eligible_students: 2,
            magi: d(50_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, dc("5000"));
        assert_eq!(r.refundable_portion, dc("2000"));
    }

    #[test]
    fn aotc_expense_above_4k_per_student_capped() {
        // $10,000 / 1 student → capped at $4k per student.
        let r = aotc(AotcInput {
            qualifying_expenses: d(10_000),
            eligible_students: 1,
            magi: d(50_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, dc("2500"));
    }

    #[test]
    fn aotc_partial_phaseout_at_midpoint() {
        // Single, MAGI = $85,000 → halfway through $80k-$90k phaseout.
        // Credit reduced by 50% → $1,250.
        let r = aotc(AotcInput {
            qualifying_expenses: d(4_000),
            eligible_students: 1,
            magi: d(85_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.phaseout_factor, dc("0.5"));
        assert_eq!(r.total, dc("1250"));
        assert_eq!(r.refundable_portion, dc("500"));
    }

    #[test]
    fn aotc_fully_phased_out_at_or_above_top() {
        let r = aotc(AotcInput {
            qualifying_expenses: d(4_000),
            eligible_students: 1,
            magi: d(95_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, Decimal::ZERO);
    }

    #[test]
    fn aotc_mfs_filer_status_ineligible() {
        let r = aotc(AotcInput {
            qualifying_expenses: d(4_000),
            eligible_students: 1,
            magi: d(50_000),
            status: FilingStatus::Mfs,
        });
        assert!(r.status_ineligible);
        assert_eq!(r.total, Decimal::ZERO);
    }

    #[test]
    fn aotc_mfj_phaseout_window_is_twice_single() {
        // MFJ, MAGI = $170k → halfway through $160k-$180k. Full credit halved.
        let r = aotc(AotcInput {
            qualifying_expenses: d(4_000),
            eligible_students: 1,
            magi: d(170_000),
            status: FilingStatus::Mfj,
        });
        assert_eq!(r.phaseout_factor, dc("0.5"));
        assert_eq!(r.total, dc("1250"));
    }

    #[test]
    fn aotc_zero_students_zero_credit() {
        let r = aotc(AotcInput {
            qualifying_expenses: d(10_000),
            eligible_students: 0,
            magi: d(50_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, Decimal::ZERO);
    }

    #[test]
    fn aotc_low_expenses_still_pro_rated() {
        // $1,000 expenses → 100% of $1k = $1,000. (Doesn't hit $2k step.)
        let r = aotc(AotcInput {
            qualifying_expenses: d(1_000),
            eligible_students: 1,
            magi: d(50_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, dc("1000"));
        assert_eq!(r.refundable_portion, dc("400"));
    }

    // ── LLC tests ───────────────────────────────────────────────────────

    #[test]
    fn llc_max_credit_at_10k_expenses() {
        // 20% × $10k = $2,000.
        let r = llc(LlcInput {
            qualifying_expenses: d(10_000),
            magi: d(50_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, dc("2000"));
    }

    #[test]
    fn llc_expense_above_10k_capped() {
        let r = llc(LlcInput {
            qualifying_expenses: d(50_000),
            magi: d(50_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, dc("2000"));
    }

    #[test]
    fn llc_below_10k_is_20pct_of_actual() {
        let r = llc(LlcInput {
            qualifying_expenses: d(5_000),
            magi: d(50_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, dc("1000"));
    }

    #[test]
    fn llc_partial_phaseout_at_midpoint() {
        // Single, MAGI = $85k → factor 0.5, credit $1,000.
        let r = llc(LlcInput {
            qualifying_expenses: d(10_000),
            magi: d(85_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, dc("1000"));
    }

    #[test]
    fn llc_mfs_filer_status_ineligible() {
        let r = llc(LlcInput {
            qualifying_expenses: d(10_000),
            magi: d(50_000),
            status: FilingStatus::Mfs,
        });
        assert!(r.status_ineligible);
    }

    #[test]
    fn llc_zero_expenses_zero_credit() {
        let r = llc(LlcInput {
            qualifying_expenses: Decimal::ZERO,
            magi: d(50_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, Decimal::ZERO);
    }
}
