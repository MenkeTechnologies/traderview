//! Common tax credits for 2025.
//!
//! - Child Tax Credit: $2,000 per qualifying child under 17, $1,700
//!   refundable per child for 2025 (Rev. Proc. 2024-40 § 3.07).
//! - Credit for Other Dependents (ODC): $500 nonrefundable per
//!   non-CTC-qualifying dependent.
//! - CTC phase-out: $200,000 single / $250,000 MFJ; reduces by $50 per
//!   $1,000 of AGI over the threshold (IRC § 24(b)(2)).
//! - EITC: NOT implemented in v1 — table-driven phase-in/phase-out
//!   over 4 family-size tiers, requires bracket tables not yet
//!   transcribed. The wizard surfaces a "you may qualify, check IRS
//!   Pub 596" link instead of computing.

use crate::engine::FilingStatus;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy)]
pub struct CtcInput {
    pub qualifying_children_under_17: u32,
    pub other_dependents: u32,
    pub agi: Decimal,
    pub status: FilingStatus,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct CtcResult {
    pub ctc: Decimal,
    pub odc: Decimal,
    /// CTC refundable portion (Additional Child Tax Credit). Used to
    /// determine how much of the CTC survives a non-tax-owing year.
    pub refundable_portion: Decimal,
    pub total: Decimal,
}

pub fn child_tax_credit(input: CtcInput) -> CtcResult {
    let per_child = Decimal::from(2_000);
    let per_other = Decimal::from(500);
    let refundable_per_child = Decimal::from(1_700);

    let raw_ctc = per_child * Decimal::from(input.qualifying_children_under_17);
    let raw_odc = per_other * Decimal::from(input.other_dependents);
    let mut total_credit = raw_ctc + raw_odc;

    // Phase-out: $50 per $1,000 of AGI over the threshold (rounded up
    // to the next $1,000 — i.e. $50 per $1,000 or fraction thereof).
    let threshold = Decimal::from(match input.status {
        FilingStatus::Mfj => 250_000,
        _ => 200_000,
    });
    if input.agi > threshold {
        let excess = input.agi - threshold;
        // Round-up to next $1,000.
        let thousand = Decimal::from(1_000);
        let blocks = ((excess + thousand - Decimal::from(1)) / thousand).floor();
        let reduction = blocks * Decimal::from(50);
        total_credit = (total_credit - reduction).max(Decimal::ZERO);
    }

    // Split back into CTC and ODC proportionally (only matters for the
    // refundable-portion calc — IRS practice phases CTC first).
    let ctc_after = total_credit.min(raw_ctc);
    let odc_after = (total_credit - ctc_after).max(Decimal::ZERO);
    let refundable =
        (refundable_per_child * Decimal::from(input.qualifying_children_under_17)).min(ctc_after);

    CtcResult {
        ctc: ctc_after,
        odc: odc_after,
        refundable_portion: refundable,
        total: total_credit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_dependents_no_credit() {
        let r = child_tax_credit(CtcInput {
            qualifying_children_under_17: 0,
            other_dependents: 0,
            agi: Decimal::from(80_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, Decimal::ZERO);
    }

    #[test]
    fn two_kids_below_phaseout() {
        // 2 kids × $2,000 = $4,000.
        let r = child_tax_credit(CtcInput {
            qualifying_children_under_17: 2,
            other_dependents: 0,
            agi: Decimal::from(80_000),
            status: FilingStatus::Mfj,
        });
        assert_eq!(r.ctc, Decimal::from(4_000));
        assert_eq!(r.total, Decimal::from(4_000));
        assert_eq!(r.refundable_portion, Decimal::from(3_400));
    }

    #[test]
    fn odc_for_adult_dependent() {
        // 1 ODC × $500 = $500.
        let r = child_tax_credit(CtcInput {
            qualifying_children_under_17: 0,
            other_dependents: 1,
            agi: Decimal::from(80_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.odc, Decimal::from(500));
        assert_eq!(r.total, Decimal::from(500));
    }

    #[test]
    fn refundable_portion_zero_when_only_odc() {
        // ODC has no refundable portion — it's nonrefundable. A user
        // with only adult dependents (no kids) → refundable = 0.
        let r = child_tax_credit(CtcInput {
            qualifying_children_under_17: 0,
            other_dependents: 3,
            agi: Decimal::from(80_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.ctc, Decimal::ZERO);
        assert_eq!(r.odc, Decimal::from(1_500));
        assert_eq!(
            r.refundable_portion,
            Decimal::ZERO,
            "ODC has no refundable component"
        );
    }

    #[test]
    fn refundable_portion_caps_at_ctc_after_phaseout() {
        // Single AGI $215k → $15k over threshold → $750 reduction.
        // 1 kid raw $2,000 → $1,250 after phase-out.
        // Refundable per-child cap is $1,700, so min($1,700, $1,250) = $1,250.
        // Refundable_portion should DROP below the per-child max.
        let r = child_tax_credit(CtcInput {
            qualifying_children_under_17: 1,
            other_dependents: 0,
            agi: Decimal::from(215_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, Decimal::from(1_250));
        assert_eq!(
            r.refundable_portion,
            Decimal::from(1_250),
            "refundable cannot exceed the CTC remaining after phase-out"
        );
    }

    #[test]
    fn refundable_portion_zero_when_ctc_fully_phased_out() {
        // CTC fully phased out → refundable_portion must be 0
        // (refundable can't exceed remaining CTC).
        let r = child_tax_credit(CtcInput {
            qualifying_children_under_17: 1,
            other_dependents: 0,
            agi: Decimal::from(300_000), // way over → 0 total
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, Decimal::ZERO);
        assert_eq!(r.refundable_portion, Decimal::ZERO);
    }

    #[test]
    fn refundable_portion_max_per_child_1700() {
        // 3 kids well under phase-out — refundable = 3 × $1,700 = $5,100.
        // Verifies the per-child refundable cap is correctly $1,700.
        let r = child_tax_credit(CtcInput {
            qualifying_children_under_17: 3,
            other_dependents: 0,
            agi: Decimal::from(50_000),
            status: FilingStatus::Mfj,
        });
        assert_eq!(r.refundable_portion, Decimal::from(5_100));
    }

    #[test]
    fn phaseout_above_single_threshold() {
        // Single, AGI $210k → $10k over threshold → 10 × $50 = $500 reduction.
        // 1 kid raw CTC $2,000 → $1,500 after phase-out.
        let r = child_tax_credit(CtcInput {
            qualifying_children_under_17: 1,
            other_dependents: 0,
            agi: Decimal::from(210_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, Decimal::from(1_500));
    }

    #[test]
    fn agi_exactly_at_threshold_no_phaseout() {
        // Single, AGI exactly $200,000 — no excess → no phase-out.
        // 1 kid → full $2,000 CTC.
        let r = child_tax_credit(CtcInput {
            qualifying_children_under_17: 1,
            other_dependents: 0,
            agi: Decimal::from(200_000),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, Decimal::from(2_000));
    }

    #[test]
    fn agi_one_dollar_over_triggers_one_block_reduction() {
        // Single, AGI $200,001 — $1 excess rounds up to $1,000 block →
        // $50 reduction. CTC $2,000 → $1,950.
        let r = child_tax_credit(CtcInput {
            qualifying_children_under_17: 1,
            other_dependents: 0,
            agi: Decimal::from(200_001),
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, Decimal::from(1_950));
    }

    #[test]
    fn phaseout_completely_eliminates_credit() {
        // 1 kid, AGI way above threshold + phase-out window. Total reduction > credit.
        let r = child_tax_credit(CtcInput {
            qualifying_children_under_17: 1,
            other_dependents: 0,
            agi: Decimal::from(300_000), // $100k over → $5,000 reduction
            status: FilingStatus::Single,
        });
        assert_eq!(r.total, Decimal::ZERO);
    }
}
