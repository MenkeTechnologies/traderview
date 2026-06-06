//! IRC § 6402 — Authority to make credits or refunds (Treasury
//! Offset Program). Trader-relevant when a refund is anticipated
//! but multiple federal/state agencies claim offset rights against
//! the overpayment. The statutory hierarchy controls which debt
//! gets paid first.
//!
//! Centralized administration: since January 1, 1999, the offset
//! function has been consolidated into the Treasury Offset Program
//! (TOP), administered by the Bureau of the Fiscal Service (BFS,
//! formerly FMS). The IRS handles § 6402(a) tax-liability offsets
//! directly; § 6402(c)/(d)/(e)/(f)/(g) offsets are processed by
//! BFS via TOP.
//!
//! § 6402 OFFSET HIERARCHY (in priority order):
//!
//!   1. § 6402(a) — outstanding INTERNAL REVENUE TAX liability
//!      (IRS handles directly; not routed through TOP)
//!   2. § 6402(c)(1) — past-due support ASSIGNED to a State
//!      (Soc. Sec. Act §§ 402(a)(26), 471(a)(17))
//!   3. § 6402(d) — past-due, legally enforceable debt owed to a
//!      Federal agency (student loans, agency overpayments, etc.)
//!   4. § 6402(c)(2) — past-due support NOT assigned to a State
//!   5. § 6402(e) — past-due, legally enforceable STATE INCOME TAX
//!      obligations (covered states with TOP reciprocity agreement)
//!   6. § 6402(f) — covered unemployment compensation debt owed to
//!      a State
//!   7. § 6402(g) — past-due TANF / state assistance obligations
//!
//! § 6402(n) INJURED SPOUSE RULE — for a joint return, if all or
//! part of the overpayment is attributable to one spouse but the
//! offset is for the other spouse's past-due debt, the non-debtor
//! ("injured") spouse may file Form 8379 to recover their share of
//! the refund. Generally the non-debtor's share is computed as
//! their fraction of total joint income / total joint tax.
//!
//! § 6402(k) MILITARY DISABILITY DISCHARGE — limits on offset of
//! payments to discharged service-members from military disability
//! retirement / discharge pay.
//!
//! § 6511 + § 6402 INTERACTION — claim must be timely under § 6511
//! limitations before offset analysis applies; covered in
//! `section_6511` module.
//!
//! Citations: IRC § 6402(a) (IRS tax-liability offset); § 6402(c)(1)
//! (assigned child support); § 6402(c)(2) (non-assigned child
//! support); § 6402(d) (federal agency non-tax debt); § 6402(e)
//! (state income tax); § 6402(f) (state unemployment compensation);
//! § 6402(g) (state TANF); § 6402(k) (military disability);
//! § 6402(n) (injured spouse rule + Form 8379); 26 CFR § 301.6402-6
//! (federal-agency-debt offset regulations); 31 CFR § 285.8
//! (state income tax offset regulations); Soc. Sec. Act § 402(a)(26)
//! and § 471(a)(17) (assigned child support reference); BFS TOP
//! administration.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6402Input {
    /// Overpayment / refund amount in cents (before offsets).
    pub overpayment_cents: i64,
    /// IRS outstanding internal revenue tax liability (priority 1).
    pub federal_tax_liability_cents: i64,
    /// Past-due child support assigned to a State (priority 2).
    pub child_support_state_assigned_cents: i64,
    /// Past-due federal agency non-tax debt — student loans, etc.
    /// (priority 3).
    pub federal_agency_non_tax_debt_cents: i64,
    /// Past-due child support NOT assigned to a State (priority 4).
    pub child_support_non_assigned_cents: i64,
    /// Past-due legally enforceable state income tax (priority 5).
    pub state_income_tax_debt_cents: i64,
    /// State unemployment compensation debt (priority 6).
    pub state_unemployment_debt_cents: i64,
    /// State TANF / past assistance (priority 7).
    pub state_tanf_debt_cents: i64,
    /// Whether the refund arose from a joint return where the
    /// non-debtor spouse may invoke the § 6402(n) injured spouse
    /// rule.
    pub injured_spouse_filed_form_8379: bool,
    /// Innocent spouse's share of the joint refund in basis points
    /// (e.g., 5000 = 50%). Used to compute the protected amount
    /// under § 6402(n).
    pub injured_spouse_share_bps: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct OffsetApplied {
    pub category: &'static str,
    pub statute: &'static str,
    pub amount_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6402Result {
    pub injured_spouse_protected_cents: i64,
    pub offsets_applied: Vec<OffsetApplied>,
    pub total_offset_cents: i64,
    pub remaining_refund_to_taxpayer_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section6402Input) -> Section6402Result {
    let mut notes: Vec<String> = Vec::new();
    let overpayment = input.overpayment_cents.max(0);

    let injured_spouse_protected = if input.injured_spouse_filed_form_8379 {
        let protected =
            overpayment.saturating_mul(input.injured_spouse_share_bps.min(10_000) as i64) / 10_000;
        notes.push(format!(
            "§ 6402(n) injured spouse rule — Form 8379 filed; non-debtor spouse's share of {} bps = {} cents protected from offset",
            input.injured_spouse_share_bps.min(10_000), protected
        ));
        protected
    } else {
        0
    };

    let mut available_for_offset = overpayment.saturating_sub(injured_spouse_protected);

    let priorities: [(&'static str, &'static str, i64); 7] = [
        (
            "IRS internal revenue tax liability",
            "§ 6402(a)",
            input.federal_tax_liability_cents.max(0),
        ),
        (
            "past-due support assigned to State",
            "§ 6402(c)(1)",
            input.child_support_state_assigned_cents.max(0),
        ),
        (
            "federal agency non-tax debt",
            "§ 6402(d)",
            input.federal_agency_non_tax_debt_cents.max(0),
        ),
        (
            "past-due support not assigned to State",
            "§ 6402(c)(2)",
            input.child_support_non_assigned_cents.max(0),
        ),
        (
            "state income tax debt",
            "§ 6402(e)",
            input.state_income_tax_debt_cents.max(0),
        ),
        (
            "state unemployment compensation debt",
            "§ 6402(f)",
            input.state_unemployment_debt_cents.max(0),
        ),
        (
            "state TANF / past assistance",
            "§ 6402(g)",
            input.state_tanf_debt_cents.max(0),
        ),
    ];

    let mut offsets_applied: Vec<OffsetApplied> = Vec::new();
    let mut total_offset = 0i64;

    for (category, statute, debt) in priorities {
        if available_for_offset == 0 || debt == 0 {
            continue;
        }
        let offset = debt.min(available_for_offset);
        offsets_applied.push(OffsetApplied {
            category,
            statute,
            amount_cents: offset,
        });
        total_offset = total_offset.saturating_add(offset);
        available_for_offset = available_for_offset.saturating_sub(offset);
    }

    let remaining = available_for_offset.saturating_add(injured_spouse_protected);

    if !offsets_applied.is_empty() {
        notes.push(format!(
            "{} offsets applied in § 6402 statutory priority order; total offset = {} cents",
            offsets_applied.len(),
            total_offset
        ));
    } else if overpayment > 0 {
        notes.push("no offsets applied — full refund flows to taxpayer".to_string());
    }

    notes.push(
        "Treasury Offset Program (TOP) administered by Bureau of the Fiscal Service (BFS) since 1999; IRS handles § 6402(a) tax-liability offsets directly"
            .to_string(),
    );

    Section6402Result {
        injured_spouse_protected_cents: injured_spouse_protected,
        offsets_applied,
        total_offset_cents: total_offset,
        remaining_refund_to_taxpayer_cents: remaining,
        citation: citation(),
        notes,
    }
}

fn citation() -> &'static str {
    "IRC § 6402(a)/(c)(1)/(c)(2)/(d)/(e)/(f)/(g)/(k)/(n); 26 CFR § 301.6402-6; 31 CFR § 285.8; Soc. Sec. Act §§ 402(a)(26), 471(a)(17); BFS Treasury Offset Program"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_with_overpayment(overpayment_dollars: i64) -> Section6402Input {
        Section6402Input {
            overpayment_cents: overpayment_dollars * 100,
            federal_tax_liability_cents: 0,
            child_support_state_assigned_cents: 0,
            federal_agency_non_tax_debt_cents: 0,
            child_support_non_assigned_cents: 0,
            state_income_tax_debt_cents: 0,
            state_unemployment_debt_cents: 0,
            state_tanf_debt_cents: 0,
            injured_spouse_filed_form_8379: false,
            injured_spouse_share_bps: 0,
        }
    }

    #[test]
    fn no_debts_full_refund_to_taxpayer() {
        let r = compute(&base_with_overpayment(5_000));
        assert_eq!(r.total_offset_cents, 0);
        assert_eq!(r.remaining_refund_to_taxpayer_cents, 5_000_00);
        assert!(r.offsets_applied.is_empty());
        assert!(r.notes.iter().any(|n| n.contains("no offsets applied")));
    }

    #[test]
    fn irs_tax_liability_priority_1() {
        let mut i = base_with_overpayment(5_000);
        i.federal_tax_liability_cents = 2_000_00;
        let r = compute(&i);
        assert_eq!(r.total_offset_cents, 2_000_00);
        assert_eq!(r.remaining_refund_to_taxpayer_cents, 3_000_00);
        assert_eq!(r.offsets_applied[0].statute, "§ 6402(a)");
        assert_eq!(
            r.offsets_applied[0].category,
            "IRS internal revenue tax liability"
        );
    }

    #[test]
    fn child_support_state_assigned_priority_2() {
        let mut i = base_with_overpayment(5_000);
        i.child_support_state_assigned_cents = 1_500_00;
        let r = compute(&i);
        assert_eq!(r.offsets_applied[0].statute, "§ 6402(c)(1)");
        assert_eq!(r.offsets_applied[0].amount_cents, 1_500_00);
    }

    #[test]
    fn federal_agency_debt_priority_3() {
        let mut i = base_with_overpayment(5_000);
        i.federal_agency_non_tax_debt_cents = 800_00;
        let r = compute(&i);
        assert_eq!(r.offsets_applied[0].statute, "§ 6402(d)");
    }

    #[test]
    fn state_income_tax_priority_5_after_federal_debts() {
        let mut i = base_with_overpayment(10_000);
        i.federal_tax_liability_cents = 2_000_00;
        i.federal_agency_non_tax_debt_cents = 1_500_00;
        i.state_income_tax_debt_cents = 3_000_00;
        let r = compute(&i);
        assert_eq!(r.offsets_applied.len(), 3);
        assert_eq!(r.offsets_applied[0].statute, "§ 6402(a)");
        assert_eq!(r.offsets_applied[1].statute, "§ 6402(d)");
        assert_eq!(r.offsets_applied[2].statute, "§ 6402(e)");
        assert_eq!(
            r.remaining_refund_to_taxpayer_cents,
            (10_000 - 2_000 - 1_500 - 3_000) * 100
        );
    }

    #[test]
    fn full_hierarchy_seven_levels_priority_order_preserved() {
        let mut i = base_with_overpayment(100_000);
        i.federal_tax_liability_cents = 5_000_00;
        i.child_support_state_assigned_cents = 4_000_00;
        i.federal_agency_non_tax_debt_cents = 3_000_00;
        i.child_support_non_assigned_cents = 2_500_00;
        i.state_income_tax_debt_cents = 2_000_00;
        i.state_unemployment_debt_cents = 1_500_00;
        i.state_tanf_debt_cents = 1_000_00;
        let r = compute(&i);
        assert_eq!(r.offsets_applied.len(), 7);
        let statutes: Vec<&str> = r.offsets_applied.iter().map(|o| o.statute).collect();
        assert_eq!(
            statutes,
            vec![
                "§ 6402(a)",
                "§ 6402(c)(1)",
                "§ 6402(d)",
                "§ 6402(c)(2)",
                "§ 6402(e)",
                "§ 6402(f)",
                "§ 6402(g)"
            ]
        );
    }

    #[test]
    fn offset_exhausts_refund_partial_satisfaction() {
        let mut i = base_with_overpayment(1_000);
        i.federal_tax_liability_cents = 5_000_00;
        let r = compute(&i);
        assert_eq!(r.total_offset_cents, 1_000_00);
        assert_eq!(r.remaining_refund_to_taxpayer_cents, 0);
        assert_eq!(r.offsets_applied[0].amount_cents, 1_000_00);
    }

    #[test]
    fn debts_exceeding_refund_only_consume_available_amount() {
        let mut i = base_with_overpayment(500);
        i.federal_tax_liability_cents = 300_00;
        i.federal_agency_non_tax_debt_cents = 1_000_00;
        let r = compute(&i);
        assert_eq!(r.offsets_applied[0].amount_cents, 300_00);
        assert_eq!(r.offsets_applied[1].amount_cents, 200_00);
        assert_eq!(r.remaining_refund_to_taxpayer_cents, 0);
    }

    #[test]
    fn injured_spouse_form_8379_protects_share_from_offset() {
        let mut i = base_with_overpayment(10_000);
        i.federal_tax_liability_cents = 8_000_00;
        i.injured_spouse_filed_form_8379 = true;
        i.injured_spouse_share_bps = 5_000;
        let r = compute(&i);
        assert_eq!(r.injured_spouse_protected_cents, 5_000_00);
        assert_eq!(r.total_offset_cents, 5_000_00);
        assert_eq!(r.remaining_refund_to_taxpayer_cents, 5_000_00);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6402(n) injured spouse rule") && n.contains("Form 8379")));
    }

    #[test]
    fn injured_spouse_share_100_bps_protects_all() {
        let mut i = base_with_overpayment(10_000);
        i.federal_tax_liability_cents = 10_000_00;
        i.injured_spouse_filed_form_8379 = true;
        i.injured_spouse_share_bps = 10_000;
        let r = compute(&i);
        assert_eq!(r.injured_spouse_protected_cents, 10_000_00);
        assert_eq!(r.total_offset_cents, 0);
        assert_eq!(r.remaining_refund_to_taxpayer_cents, 10_000_00);
    }

    #[test]
    fn injured_spouse_share_capped_at_10000_bps() {
        let mut i = base_with_overpayment(10_000);
        i.federal_tax_liability_cents = 10_000_00;
        i.injured_spouse_filed_form_8379 = true;
        i.injured_spouse_share_bps = 20_000;
        let r = compute(&i);
        assert_eq!(r.injured_spouse_protected_cents, 10_000_00);
    }

    #[test]
    fn no_injured_spouse_filing_no_protection() {
        let mut i = base_with_overpayment(10_000);
        i.federal_tax_liability_cents = 10_000_00;
        i.injured_spouse_filed_form_8379 = false;
        i.injured_spouse_share_bps = 5_000;
        let r = compute(&i);
        assert_eq!(r.injured_spouse_protected_cents, 0);
        assert_eq!(r.total_offset_cents, 10_000_00);
        assert_eq!(r.remaining_refund_to_taxpayer_cents, 0);
    }

    #[test]
    fn zero_overpayment_no_offsets_applied() {
        let mut i = base_with_overpayment(0);
        i.federal_tax_liability_cents = 5_000_00;
        let r = compute(&i);
        assert_eq!(r.total_offset_cents, 0);
        assert_eq!(r.remaining_refund_to_taxpayer_cents, 0);
        assert!(r.offsets_applied.is_empty());
    }

    #[test]
    fn child_support_non_assigned_priority_4_after_federal_agency() {
        let mut i = base_with_overpayment(5_000);
        i.federal_agency_non_tax_debt_cents = 1_000_00;
        i.child_support_non_assigned_cents = 2_000_00;
        let r = compute(&i);
        assert_eq!(r.offsets_applied[0].statute, "§ 6402(d)");
        assert_eq!(r.offsets_applied[1].statute, "§ 6402(c)(2)");
    }

    #[test]
    fn state_unemployment_priority_6_after_state_income_tax() {
        let mut i = base_with_overpayment(5_000);
        i.state_income_tax_debt_cents = 1_500_00;
        i.state_unemployment_debt_cents = 800_00;
        let r = compute(&i);
        assert_eq!(r.offsets_applied[0].statute, "§ 6402(e)");
        assert_eq!(r.offsets_applied[1].statute, "§ 6402(f)");
    }

    #[test]
    fn state_tanf_priority_7_lowest() {
        let mut i = base_with_overpayment(5_000);
        i.state_tanf_debt_cents = 1_000_00;
        let r = compute(&i);
        assert_eq!(r.offsets_applied[0].statute, "§ 6402(g)");
    }

    #[test]
    fn citation_pins_all_subsections_and_regulations() {
        let r = compute(&base_with_overpayment(1_000));
        assert!(r.citation.contains("§ 6402(a)"));
        assert!(r.citation.contains("(c)(1)"));
        assert!(r.citation.contains("(c)(2)"));
        assert!(r.citation.contains("(d)"));
        assert!(r.citation.contains("(e)"));
        assert!(r.citation.contains("(f)"));
        assert!(r.citation.contains("(g)"));
        assert!(r.citation.contains("(k)"));
        assert!(r.citation.contains("(n)"));
        assert!(r.citation.contains("§ 301.6402-6"));
        assert!(r.citation.contains("§ 285.8"));
        assert!(r.citation.contains("§§ 402(a)(26), 471(a)(17)"));
        assert!(r.citation.contains("BFS Treasury Offset Program"));
    }

    #[test]
    fn negative_debts_clamped_to_zero() {
        let mut i = base_with_overpayment(5_000);
        i.federal_tax_liability_cents = -1_000_00;
        i.federal_agency_non_tax_debt_cents = 500_00;
        let r = compute(&i);
        assert_eq!(r.offsets_applied.len(), 1);
        assert_eq!(r.offsets_applied[0].statute, "§ 6402(d)");
    }

    #[test]
    fn note_describes_top_administration() {
        let r = compute(&base_with_overpayment(1_000));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Treasury Offset Program (TOP)")
                && n.contains("Bureau of the Fiscal Service")));
    }

    #[test]
    fn note_describes_offset_count() {
        let mut i = base_with_overpayment(10_000);
        i.federal_tax_liability_cents = 1_000_00;
        i.federal_agency_non_tax_debt_cents = 1_000_00;
        i.state_income_tax_debt_cents = 1_000_00;
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("3 offsets applied")));
    }

    #[test]
    fn injured_spouse_protection_persists_against_all_offset_categories() {
        let mut i = base_with_overpayment(10_000);
        i.federal_tax_liability_cents = 2_000_00;
        i.child_support_state_assigned_cents = 2_000_00;
        i.federal_agency_non_tax_debt_cents = 2_000_00;
        i.state_income_tax_debt_cents = 2_000_00;
        i.state_unemployment_debt_cents = 2_000_00;
        i.injured_spouse_filed_form_8379 = true;
        i.injured_spouse_share_bps = 4_000;
        let r = compute(&i);
        assert_eq!(r.injured_spouse_protected_cents, 4_000_00);
        assert_eq!(r.total_offset_cents, 6_000_00);
        assert_eq!(r.remaining_refund_to_taxpayer_cents, 4_000_00);
    }

    #[test]
    fn injured_spouse_protected_amount_added_to_remaining_refund() {
        let mut i = base_with_overpayment(10_000);
        i.federal_tax_liability_cents = 3_000_00;
        i.injured_spouse_filed_form_8379 = true;
        i.injured_spouse_share_bps = 4_000;
        let r = compute(&i);
        let expected_remaining = (10_000 - 3_000) * 100;
        assert_eq!(r.remaining_refund_to_taxpayer_cents, expected_remaining);
    }

    #[test]
    fn offsets_consume_in_strict_priority_order_when_refund_partially_exhausted() {
        let mut i = base_with_overpayment(3_000);
        i.federal_tax_liability_cents = 2_000_00;
        i.child_support_state_assigned_cents = 2_000_00;
        i.federal_agency_non_tax_debt_cents = 5_000_00;
        let r = compute(&i);
        assert_eq!(r.offsets_applied.len(), 2);
        assert_eq!(r.offsets_applied[0].statute, "§ 6402(a)");
        assert_eq!(r.offsets_applied[0].amount_cents, 2_000_00);
        assert_eq!(r.offsets_applied[1].statute, "§ 6402(c)(1)");
        assert_eq!(r.offsets_applied[1].amount_cents, 1_000_00);
        assert_eq!(r.remaining_refund_to_taxpayer_cents, 0);
    }

    #[test]
    fn negative_overpayment_clamped_to_zero() {
        let mut i = base_with_overpayment(0);
        i.overpayment_cents = -100_000;
        i.federal_tax_liability_cents = 50_000;
        let r = compute(&i);
        assert_eq!(r.total_offset_cents, 0);
        assert_eq!(r.remaining_refund_to_taxpayer_cents, 0);
    }

    #[test]
    fn all_seven_categories_skip_when_empty() {
        let r = compute(&base_with_overpayment(5_000));
        assert!(r.offsets_applied.is_empty());
    }

    #[test]
    fn child_support_non_assigned_falls_between_federal_agency_and_state_tax() {
        let mut i = base_with_overpayment(5_000);
        i.federal_agency_non_tax_debt_cents = 1_000_00;
        i.child_support_non_assigned_cents = 1_000_00;
        i.state_income_tax_debt_cents = 1_000_00;
        let r = compute(&i);
        let statutes: Vec<&str> = r.offsets_applied.iter().map(|o| o.statute).collect();
        assert_eq!(statutes, vec!["§ 6402(d)", "§ 6402(c)(2)", "§ 6402(e)"]);
    }
}
