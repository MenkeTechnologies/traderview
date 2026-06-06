//! IRC §704(c) — Pre-contribution built-in gain/loss allocation in
//! partnerships.
//!
//! Pairs with `section_704d` (outside basis loss limitation) and
//! `section_754` (§743(b) inside basis adjustment) to complete the
//! §704 / §743 / §754 partnership-allocation triangle.
//!
//! **Core rule §704(c)(1)(A)**: when a partner contributes property
//! with a tax basis ≠ fair market value at contribution, any
//! pre-contribution built-in gain (FMV − basis) or built-in loss
//! (basis − FMV) MUST be allocated back to the CONTRIBUTING PARTNER
//! on subsequent disposition by the partnership. The other partners
//! receive only the post-contribution change in value.
//!
//! **Three allocation methods** under Treas. Reg. § 1.704-3:
//!
//! 1. **Traditional** (default): allocations match book value
//!    movements but are subject to the **ceiling rule** — tax
//!    allocations to non-contributing partners cannot exceed the
//!    total tax items of the partnership. The ceiling rule can leave
//!    the contributing partner unfairly absorbing more or less than
//!    intended.
//!
//! 2. **Traditional with curative allocations**: allocates OTHER
//!    existing income or deductions (of the same character) to fix
//!    the ceiling-rule distortion and make non-contributing partners
//!    economically whole.
//!
//! 3. **Remedial**: creates NOTIONAL items of income and deduction
//!    for tax purposes (no real cash flow) to ensure all partners
//!    receive their proper share of tax allocations. Most accurate
//!    but requires the partnership to track phantom items.
//!
//! **§704(c)(1)(B) anti-mixing-bowl rule**: if the partnership
//! distributes the contributed property to ANY PARTNER OTHER THAN THE
//! ORIGINAL CONTRIBUTOR within **7 years** of contribution, the
//! contributing partner must recognize their remaining pre-contribution
//! built-in gain or loss as if the property had been sold at FMV on
//! the date of distribution.
//!
//! **§737 complementary anti-mixing-bowl**: same 7-year window;
//! triggers when the CONTRIBUTING PARTNER receives a distribution of
//! OTHER PROPERTY whose FMV exceeds their outside basis. Accelerates
//! recognition of §704(c)(1)(A) remaining gain.
//!
//! **§704(c)(1)(C) built-in LOSS restriction** (added by American
//! Jobs Creation Act of 2004, P.L. 108-357 § 833(a)): pre-contribution
//! built-in losses can ONLY be allocated to the contributing partner.
//! Other partners treat the property's basis as equal to FMV at the
//! time of contribution — effectively eliminating the built-in loss
//! from their tax calculations. Prevents the pre-2004 "loss-property
//! trafficking" strategy where loss property was contributed to share
//! losses with partners having no economic connection to them.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllocationMethod {
    Traditional,
    TraditionalWithCurativeAllocations,
    Remedial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrecontributionDirection {
    BuiltInGain,
    BuiltInLoss,
    Neither,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section704cInput {
    pub contribution_date: NaiveDate,
    pub as_of_date: NaiveDate,
    /// Pre-contribution built-in gain (FMV − basis at contribution).
    /// Use positive value; module classifies as BIG. Set to 0 if
    /// instead built-in loss is present.
    pub pre_contribution_built_in_gain: Decimal,
    /// Pre-contribution built-in loss (basis − FMV at contribution).
    /// Use positive value; module classifies as BIL.
    pub pre_contribution_built_in_loss: Decimal,
    /// Gain partnership later realizes on disposition of the
    /// contributed property (FMV at disposition − basis at
    /// disposition). Used for §704(c)(1)(A) allocation cap.
    pub disposition_gain_realized: Decimal,
    pub allocation_method: AllocationMethod,
    pub contributing_partner_normal_share_pct_bp: u32,
    /// §704(c)(1)(B) trigger: date partnership distributed the
    /// contributed property to another partner. None if no such
    /// distribution has occurred.
    pub distributed_to_other_partner_date: Option<NaiveDate>,
    /// §737 trigger: date contributing partner received distribution
    /// of OTHER property. None if no such distribution has occurred.
    pub contributor_received_other_property_date: Option<NaiveDate>,
    /// FMV of other property received by contributor for §737 gain
    /// recognition computation.
    pub other_property_received_fmv: Decimal,
    pub contributor_outside_basis: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section704cResult {
    pub direction: PrecontributionDirection,
    pub seven_year_window_expired: bool,
    /// Gain allocated to contributor under §704(c)(1)(A) on partnership
    /// disposition (capped at lesser of pre-contribution BIG or
    /// disposition gain).
    pub contributor_704c1a_allocation: Decimal,
    /// Excess disposition gain allocated PRO RATA to all partners.
    pub partnership_proportional_gain: Decimal,
    /// §704(c)(1)(B) acceleration: full remaining pre-contribution
    /// gain recognized to contributor on distribution to other partner.
    pub section_704c1b_triggered: bool,
    pub section_704c1b_gain_to_contributor: Decimal,
    /// §737 acceleration: contributor gain on receipt of other property
    /// (FMV of other property − outside basis, capped at remaining BIG).
    pub section_737_triggered: bool,
    pub section_737_gain_to_contributor: Decimal,
    /// §704(c)(1)(C) built-in loss restriction — other partners ignore
    /// built-in loss; only contributor benefits.
    pub section_704c1c_loss_restricted_to_contributor: bool,
    pub method_creates_notional_items: bool,
    pub method_subject_to_ceiling_rule: bool,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section704cInput) -> Section704cResult {
    let direction = if input.pre_contribution_built_in_gain > Decimal::ZERO {
        PrecontributionDirection::BuiltInGain
    } else if input.pre_contribution_built_in_loss > Decimal::ZERO {
        PrecontributionDirection::BuiltInLoss
    } else {
        PrecontributionDirection::Neither
    };

    let seven_years_after_contribution = input
        .contribution_date
        .checked_add_signed(chrono::Duration::days(365 * 7 + 2))
        .unwrap_or(input.contribution_date);
    let seven_year_expired = input.as_of_date > seven_years_after_contribution;

    // §704(c)(1)(A) allocation on disposition: pre-contribution gain
    // allocated to contributor up to the lesser of (pre-contribution
    // BIG) or (disposition gain realized). Any excess goes pro rata.
    let contributor_704c1a = if direction == PrecontributionDirection::BuiltInGain {
        input
            .pre_contribution_built_in_gain
            .min(input.disposition_gain_realized)
            .max(Decimal::ZERO)
    } else {
        Decimal::ZERO
    };
    let partnership_proportional =
        (input.disposition_gain_realized - contributor_704c1a).max(Decimal::ZERO);

    // §704(c)(1)(B) anti-mixing-bowl: distribution of contributed
    // property to OTHER partner within 7 years → full remaining BIG
    // recognized to contributor.
    let (sec_704c1b, sec_704c1b_gain) = match input.distributed_to_other_partner_date {
        Some(d)
            if d <= seven_years_after_contribution
                && direction == PrecontributionDirection::BuiltInGain =>
        {
            // Remaining BIG after any prior disposition allocations.
            let remaining =
                (input.pre_contribution_built_in_gain - contributor_704c1a).max(Decimal::ZERO);
            (true, remaining)
        }
        _ => (false, Decimal::ZERO),
    };

    // §737 reverse: contributing partner receives other property
    // within 7 years → gain recognition = lesser of (FMV other property
    // received − outside basis) or (remaining pre-contribution BIG).
    let (sec_737, sec_737_gain) = match input.contributor_received_other_property_date {
        Some(d)
            if d <= seven_years_after_contribution
                && direction == PrecontributionDirection::BuiltInGain =>
        {
            let excess_distribution = (input.other_property_received_fmv
                - input.contributor_outside_basis)
                .max(Decimal::ZERO);
            let remaining_big =
                (input.pre_contribution_built_in_gain - contributor_704c1a - sec_704c1b_gain)
                    .max(Decimal::ZERO);
            (true, excess_distribution.min(remaining_big))
        }
        _ => (false, Decimal::ZERO),
    };

    let bil_restricted = direction == PrecontributionDirection::BuiltInLoss;

    let (notional, ceiling) = match input.allocation_method {
        AllocationMethod::Traditional => (false, true),
        AllocationMethod::TraditionalWithCurativeAllocations => (false, false),
        AllocationMethod::Remedial => (true, false),
    };

    let mut note_parts: Vec<String> = Vec::new();
    match direction {
        PrecontributionDirection::BuiltInGain => {
            note_parts.push(format!(
                "§704(c)(1)(A): ${} pre-contribution BIG; ${} allocated to contributor on disposition, ${} pro rata to all partners",
                input.pre_contribution_built_in_gain.round_dp(2),
                contributor_704c1a.round_dp(2),
                partnership_proportional.round_dp(2),
            ));
        }
        PrecontributionDirection::BuiltInLoss => {
            note_parts.push(format!(
                "§704(c)(1)(C): ${} pre-contribution BIL restricted to contributing partner only; other partners take FMV basis (AJCA 2004 anti-loss-trafficking rule)",
                input.pre_contribution_built_in_loss.round_dp(2),
            ));
        }
        PrecontributionDirection::Neither => {
            note_parts.push("No pre-contribution gain or loss; §704(c) does not apply".to_string());
        }
    }
    if sec_704c1b {
        note_parts.push(format!(
            "§704(c)(1)(B) ANTI-MIXING-BOWL TRIGGERED: distribution to other partner within 7-year window; ${} remaining BIG accelerated to contributor",
            sec_704c1b_gain.round_dp(2),
        ));
    }
    if sec_737 {
        note_parts.push(format!(
            "§737 REVERSE MIXING-BOWL TRIGGERED: contributor received other property within 7-year window; ${} gain recognition",
            sec_737_gain.round_dp(2),
        ));
    }
    if seven_year_expired {
        note_parts.push(format!(
            "7-year window expired on {} — §704(c)(1)(B) + §737 mixing-bowl no longer applies",
            seven_years_after_contribution
        ));
    }
    note_parts.push(format!(
        "Method: {:?} ({}{})",
        input.allocation_method,
        if ceiling {
            "subject to ceiling rule"
        } else {
            "no ceiling rule"
        },
        if notional {
            "; creates notional remedial items"
        } else {
            ""
        },
    ));

    Section704cResult {
        direction,
        seven_year_window_expired: seven_year_expired,
        contributor_704c1a_allocation: contributor_704c1a,
        partnership_proportional_gain: partnership_proportional,
        section_704c1b_triggered: sec_704c1b,
        section_704c1b_gain_to_contributor: sec_704c1b_gain,
        section_737_triggered: sec_737,
        section_737_gain_to_contributor: sec_737_gain,
        section_704c1c_loss_restricted_to_contributor: bil_restricted,
        method_creates_notional_items: notional,
        method_subject_to_ceiling_rule: ceiling,
        citation:
            "IRC §704(c)(1)(A) precontribution gain allocation; §704(c)(1)(B) 7-year anti-mixing-bowl (distribution to other partner); §737 reverse anti-mixing-bowl (contributor receives other property); §704(c)(1)(C) built-in loss restriction (AJCA 2004 §833(a)); Treas. Reg. §1.704-3 traditional/curative/remedial methods"
                .to_string(),
        note: note_parts.join("; "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn base() -> Section704cInput {
        Section704cInput {
            contribution_date: d(2024, 1, 1),
            as_of_date: d(2026, 6, 1),
            pre_contribution_built_in_gain: dec!(500_000),
            pre_contribution_built_in_loss: Decimal::ZERO,
            disposition_gain_realized: dec!(700_000),
            allocation_method: AllocationMethod::Traditional,
            contributing_partner_normal_share_pct_bp: 5000, // 50%
            distributed_to_other_partner_date: None,
            contributor_received_other_property_date: None,
            other_property_received_fmv: Decimal::ZERO,
            contributor_outside_basis: dec!(1_000_000),
        }
    }

    // §704(c)(1)(A) basic allocation.

    #[test]
    fn big_disposition_allocates_500k_to_contributor_200k_to_partners() {
        // BIG $500k, disposition gain $700k → $500k to contributor,
        // $200k pro rata to all partners.
        let r = compute(&base());
        assert_eq!(r.direction, PrecontributionDirection::BuiltInGain);
        assert_eq!(r.contributor_704c1a_allocation, dec!(500_000));
        assert_eq!(r.partnership_proportional_gain, dec!(200_000));
    }

    #[test]
    fn small_disposition_gain_caps_704c1a_at_gain() {
        // BIG $500k, but disposition only $300k → only $300k to contributor.
        let mut i = base();
        i.disposition_gain_realized = dec!(300_000);
        let r = compute(&i);
        assert_eq!(r.contributor_704c1a_allocation, dec!(300_000));
        assert_eq!(r.partnership_proportional_gain, Decimal::ZERO);
    }

    #[test]
    fn exactly_pre_contribution_big_at_disposition() {
        // BIG $500k = disposition $500k → all $500k to contributor, $0 pro rata.
        let mut i = base();
        i.disposition_gain_realized = dec!(500_000);
        let r = compute(&i);
        assert_eq!(r.contributor_704c1a_allocation, dec!(500_000));
        assert_eq!(r.partnership_proportional_gain, Decimal::ZERO);
    }

    #[test]
    fn zero_pre_contribution_gain_neither_direction() {
        let mut i = base();
        i.pre_contribution_built_in_gain = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.direction, PrecontributionDirection::Neither);
        assert_eq!(r.contributor_704c1a_allocation, Decimal::ZERO);
    }

    // §704(c)(1)(B) anti-mixing-bowl.

    #[test]
    fn distribution_to_other_partner_within_7_years_triggers_704c1b() {
        let mut i = base();
        i.distributed_to_other_partner_date = Some(d(2027, 1, 1));
        i.disposition_gain_realized = Decimal::ZERO; // No prior disposition
        let r = compute(&i);
        assert!(r.section_704c1b_triggered);
        assert_eq!(r.section_704c1b_gain_to_contributor, dec!(500_000));
    }

    #[test]
    fn distribution_after_7_years_does_not_trigger_704c1b() {
        let mut i = base();
        // 7-year window from 2024-01-01 ends ~2031-01-01.
        i.distributed_to_other_partner_date = Some(d(2031, 6, 1));
        i.as_of_date = d(2031, 7, 1);
        let r = compute(&i);
        assert!(!r.section_704c1b_triggered);
    }

    #[test]
    fn seven_year_window_expired_flag_set_after_window() {
        let mut i = base();
        i.as_of_date = d(2032, 1, 1);
        let r = compute(&i);
        assert!(r.seven_year_window_expired);
    }

    #[test]
    fn seven_year_window_not_expired_within_window() {
        let r = compute(&base());
        assert!(!r.seven_year_window_expired);
    }

    #[test]
    fn sec_704c1b_only_remaining_gain_after_prior_disposition() {
        // $500k BIG, $300k allocated on prior disposition, then $200k
        // remaining accelerated on §704(c)(1)(B).
        let mut i = base();
        i.disposition_gain_realized = dec!(300_000);
        i.distributed_to_other_partner_date = Some(d(2025, 6, 1));
        let r = compute(&i);
        assert_eq!(r.contributor_704c1a_allocation, dec!(300_000));
        assert!(r.section_704c1b_triggered);
        assert_eq!(r.section_704c1b_gain_to_contributor, dec!(200_000));
    }

    // §737 reverse.

    #[test]
    fn section_737_triggered_when_other_property_received_within_7_years() {
        let mut i = base();
        i.disposition_gain_realized = Decimal::ZERO;
        i.contributor_received_other_property_date = Some(d(2026, 1, 1));
        i.other_property_received_fmv = dec!(1_200_000);
        i.contributor_outside_basis = dec!(1_000_000);
        // Excess distribution $200k, capped at remaining BIG $500k → $200k gain.
        let r = compute(&i);
        assert!(r.section_737_triggered);
        assert_eq!(r.section_737_gain_to_contributor, dec!(200_000));
    }

    #[test]
    fn section_737_gain_capped_at_remaining_big() {
        let mut i = base();
        i.disposition_gain_realized = Decimal::ZERO;
        i.contributor_received_other_property_date = Some(d(2026, 1, 1));
        i.other_property_received_fmv = dec!(10_000_000);
        i.contributor_outside_basis = dec!(1_000_000);
        // Excess distribution $9M, capped at remaining BIG $500k → $500k gain.
        let r = compute(&i);
        assert_eq!(r.section_737_gain_to_contributor, dec!(500_000));
    }

    #[test]
    fn section_737_no_excess_distribution_no_gain() {
        let mut i = base();
        i.disposition_gain_realized = Decimal::ZERO;
        i.contributor_received_other_property_date = Some(d(2026, 1, 1));
        i.other_property_received_fmv = dec!(500_000);
        i.contributor_outside_basis = dec!(1_000_000);
        let r = compute(&i);
        assert!(r.section_737_triggered);
        assert_eq!(r.section_737_gain_to_contributor, Decimal::ZERO);
    }

    #[test]
    fn section_737_after_7_years_not_triggered() {
        let mut i = base();
        i.contributor_received_other_property_date = Some(d(2032, 1, 1));
        i.as_of_date = d(2032, 1, 1);
        let r = compute(&i);
        assert!(!r.section_737_triggered);
    }

    // §704(c)(1)(C) built-in loss.

    #[test]
    fn bil_restricted_to_contributing_partner() {
        let mut i = base();
        i.pre_contribution_built_in_gain = Decimal::ZERO;
        i.pre_contribution_built_in_loss = dec!(200_000);
        let r = compute(&i);
        assert_eq!(r.direction, PrecontributionDirection::BuiltInLoss);
        assert!(r.section_704c1c_loss_restricted_to_contributor);
    }

    #[test]
    fn bil_does_not_trigger_704c1a_gain_allocation() {
        let mut i = base();
        i.pre_contribution_built_in_gain = Decimal::ZERO;
        i.pre_contribution_built_in_loss = dec!(200_000);
        i.disposition_gain_realized = dec!(100_000);
        let r = compute(&i);
        assert_eq!(r.contributor_704c1a_allocation, Decimal::ZERO);
    }

    // Allocation methods.

    #[test]
    fn traditional_method_subject_to_ceiling_rule() {
        let r = compute(&base());
        assert!(r.method_subject_to_ceiling_rule);
        assert!(!r.method_creates_notional_items);
    }

    #[test]
    fn curative_method_no_ceiling_rule_no_notional() {
        let mut i = base();
        i.allocation_method = AllocationMethod::TraditionalWithCurativeAllocations;
        let r = compute(&i);
        assert!(!r.method_subject_to_ceiling_rule);
        assert!(!r.method_creates_notional_items);
    }

    #[test]
    fn remedial_method_creates_notional_items() {
        let mut i = base();
        i.allocation_method = AllocationMethod::Remedial;
        let r = compute(&i);
        assert!(r.method_creates_notional_items);
        assert!(!r.method_subject_to_ceiling_rule);
    }

    // Notes.

    #[test]
    fn note_describes_big_allocation_path() {
        let r = compute(&base());
        assert!(r.note.contains("§704(c)(1)(A)"));
        assert!(r.note.contains("$500000"));
    }

    #[test]
    fn note_describes_bil_restriction_path() {
        let mut i = base();
        i.pre_contribution_built_in_gain = Decimal::ZERO;
        i.pre_contribution_built_in_loss = dec!(200_000);
        let r = compute(&i);
        assert!(r.note.contains("§704(c)(1)(C)"));
        assert!(r.note.contains("AJCA 2004"));
    }

    #[test]
    fn note_describes_704c1b_trigger() {
        let mut i = base();
        i.distributed_to_other_partner_date = Some(d(2025, 6, 1));
        let r = compute(&i);
        assert!(r.note.contains("ANTI-MIXING-BOWL TRIGGERED"));
    }

    #[test]
    fn note_describes_737_trigger() {
        let mut i = base();
        i.contributor_received_other_property_date = Some(d(2026, 1, 1));
        i.other_property_received_fmv = dec!(1_200_000);
        let r = compute(&i);
        assert!(r.note.contains("§737 REVERSE MIXING-BOWL TRIGGERED"));
    }

    #[test]
    fn note_describes_7_year_expiration() {
        let mut i = base();
        i.as_of_date = d(2032, 1, 1);
        let r = compute(&i);
        assert!(r.note.contains("7-year window expired"));
    }

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§704(c)(1)(A)"));
        assert!(r.citation.contains("§704(c)(1)(B)"));
        assert!(r.citation.contains("§737"));
        assert!(r.citation.contains("§704(c)(1)(C)"));
        assert!(r.citation.contains("Treas. Reg. §1.704-3"));
    }

    #[test]
    fn very_large_big_precision_path() {
        // $1B BIG, $1.5B disposition gain → $1B to contributor, $500M pro rata.
        let mut i = base();
        i.pre_contribution_built_in_gain = dec!(1_000_000_000);
        i.disposition_gain_realized = dec!(1_500_000_000);
        let r = compute(&i);
        assert_eq!(r.contributor_704c1a_allocation, dec!(1_000_000_000));
        assert_eq!(r.partnership_proportional_gain, dec!(500_000_000));
    }
}
