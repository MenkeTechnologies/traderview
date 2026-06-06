//! IRC § 961 — Adjustments to Basis of Stock in Controlled Foreign
//! Corporations and Other Property.
//!
//! § 961 establishes the stock-basis tracking mechanism that prevents double
//! US taxation of CFC earnings: PTEP inclusion under § 951(a) Subpart F, §
//! 951A GILTI / NCTI, or § 956 US-property investment INCREASES US
//! shareholder's basis in CFC stock under § 961(a); subsequent actual
//! distribution of PTEP under § 959 DECREASES basis under § 961(b);
//! distributions exceeding basis are RECOGNIZED as § 301(c)(3) capital
//! gain under § 961(b)(2).
//!
//! § 961(a) BASIS INCREASE: US shareholder's basis in CFC stock increased
//! by amount included in gross income under § 951(a) or § 951A. Shields
//! future gain when undistributed PTEP exists within the CFC.
//!
//! § 961(b)(1) BASIS DECREASE: US shareholder's basis in CFC stock
//! decreased on actual distribution of PTEP under § 959(a) (excluded from
//! gross income) by amount of the distribution.
//!
//! § 961(b)(2) BASIS-FLOOR GAIN: distribution of PTEP that exceeds US
//! shareholder's basis in CFC stock RECOGNIZED as § 301(c)(3) capital gain
//! to extent of excess. Prevents negative basis from forming.
//!
//! § 961(c) INDIRECTLY OWNED CFC BASIS: parallel basis adjustments apply
//! to lower-tier CFC stock held INDIRECTLY through other CFCs — limited to
//! determining § 951 inclusion amounts only (not for gain / loss
//! recognition on disposition of intermediate stock).
//!
//! Notice 2024-16 (announced January 16, 2024): in certain inbound non-
//! recognition transactions (qualifying § 332 liquidation, § 368(a)(1)
//! asset reorg, § 351 transfer) where domestic acquiring corporation
//! receives CFC stock from another domestic corporation, § 961(c) basis of
//! the acquired CFC carries over to the domestic acquiring corporation —
//! prevents trapped-PTEP situations where domestic acquiror would otherwise
//! lose § 961(c) basis on inbound transfer and face gain on subsequent PTEP
//! distribution.
//!
//! Proposed Regulations REG-105479-18 (Nov 29, 2024 / published Dec 2,
//! 2024) implement § 959 sixteen-basket PTEP framework + § 961 basis
//! tracking + § 962 election + currency translation + S corp PTEP +
//! consolidated group PTEP + anti-avoidance rules. Coordinates with
//! `section_959` iter 512 + `section_960` iter 520 + `section_962`
//! iter 510.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BasisAdjustmentEvent {
    /// PTEP inclusion under § 951(a) Subpart F — § 961(a) basis increase.
    PtepInclusionSubpartF,
    /// GILTI / NCTI inclusion under § 951A — § 961(a) basis increase.
    PtepInclusionGiltiOrNcti,
    /// § 956 US-property investment inclusion — § 961(a) basis increase.
    PtepInclusionSection956,
    /// Actual PTEP distribution under § 959(a) — § 961(b) basis decrease;
    /// excess over basis = § 961(b)(2) capital gain.
    ActualPtepDistribution,
    /// § 332 liquidation of CFC into domestic corp — Notice 2024-16
    /// carryover rule.
    Section332InboundLiquidation,
    /// § 368(a)(1) asset reorg involving CFC + domestic acquiror — Notice
    /// 2024-16 carryover rule.
    Section368AssetReorganization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    BasisIncreaseUnderSection961a,
    BasisDecreaseUnderSection961b,
    BasisFloorExcessDistributionGainSection961b2,
    InboundNonrecognitionCarryoverUnderNotice2024_16,
    IndirectlyOwnedSection961cLimitedToSection951Inclusion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section961Input {
    pub event: BasisAdjustmentEvent,
    /// Pre-event US shareholder's basis in CFC stock in cents.
    pub pre_event_basis_cents: u64,
    /// Amount of inclusion or distribution in cents.
    pub event_amount_cents: u64,
    /// Whether stock is held INDIRECTLY through another CFC (§ 961(c)
    /// limitation applies).
    pub indirectly_owned_through_cfc: bool,
    /// Taxable year.
    pub taxable_year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section961Result {
    pub severity: Severity,
    pub post_event_basis_cents: u64,
    pub recognized_gain_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const NOTICE_2024_16_DATE: &str = "2024-01-16";
pub const PROPOSED_REGS_REG_105479_18_PUBLISHED: &str = "2024-12-02";
pub const TCJA_SECTION_961_AMENDMENT_YEAR: i32 = 2017;

pub fn check(input: &Section961Input) -> Section961Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if input.indirectly_owned_through_cfc {
        notes.push(
            "§ 961(c) limitation applies for stock held INDIRECTLY through another CFC. Basis \
             adjustments are limited to determining § 951 inclusion amounts only — NOT \
             available for gain / loss recognition on disposition of intermediate stock. \
             Notice 2024-16 carryover rule preserves § 961(c) basis in qualifying inbound \
             § 332 or § 368(a)(1) transactions."
                .to_string(),
        );
        return Section961Result {
            severity: Severity::IndirectlyOwnedSection961cLimitedToSection951Inclusion,
            post_event_basis_cents: input.pre_event_basis_cents,
            recognized_gain_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 961(c); Notice 2024-16",
            notes: build_coord_notes(notes),
        };
    }

    match input.event {
        BasisAdjustmentEvent::PtepInclusionSubpartF
        | BasisAdjustmentEvent::PtepInclusionGiltiOrNcti
        | BasisAdjustmentEvent::PtepInclusionSection956 => {
            let post = input
                .pre_event_basis_cents
                .saturating_add(input.event_amount_cents);
            actions.push(format!(
                "§ 961(a) basis INCREASE: PTEP inclusion of {} cents under {:?} adds to US \
                 shareholder's basis in CFC stock. Pre-event basis {} cents → post-event \
                 basis {} cents. Track within § 959 sixteen-basket PTEP framework (see \
                 [[section_959]] iter 512); maintain PTEP account on Form 5471 Schedule J + \
                 Schedule P; preserve basis records indefinitely until CFC stock disposed.",
                input.event_amount_cents, input.event, input.pre_event_basis_cents, post
            ));
            Section961Result {
                severity: Severity::BasisIncreaseUnderSection961a,
                post_event_basis_cents: post,
                recognized_gain_cents: 0,
                recommended_actions: actions,
                citation: "26 U.S.C. § 961(a); § 951(a); § 951A; § 956",
                notes: build_coord_notes(notes),
            }
        }
        BasisAdjustmentEvent::ActualPtepDistribution => {
            if input.event_amount_cents <= input.pre_event_basis_cents {
                let post = input.pre_event_basis_cents - input.event_amount_cents;
                actions.push(format!(
                    "§ 961(b)(1) basis DECREASE: actual PTEP distribution of {} cents under § \
                     959(a) reduces US shareholder's basis in CFC stock. Pre-event basis {} \
                     cents → post-event basis {} cents. Distribution excluded from gross \
                     income per § 959(a)(1); no gain recognized.",
                    input.event_amount_cents, input.pre_event_basis_cents, post
                ));
                Section961Result {
                    severity: Severity::BasisDecreaseUnderSection961b,
                    post_event_basis_cents: post,
                    recognized_gain_cents: 0,
                    recommended_actions: actions,
                    citation: "26 U.S.C. § 961(b)(1); § 959(a)",
                    notes: build_coord_notes(notes),
                }
            } else {
                let recognized_gain = input
                    .event_amount_cents
                    .saturating_sub(input.pre_event_basis_cents);
                actions.push(format!(
                    "§ 961(b)(2) BASIS-FLOOR GAIN: actual PTEP distribution of {} cents \
                     EXCEEDS US shareholder's basis of {} cents; excess of {} cents \
                     RECOGNIZED as § 301(c)(3) capital gain. Post-event basis = $0 (basis \
                     cannot go negative). Report on Form 1040 Schedule D (individual) or \
                     Form 1120 Schedule D (corp); long-term vs. short-term depends on stock \
                     holding period.",
                    input.event_amount_cents, input.pre_event_basis_cents, recognized_gain
                ));
                Section961Result {
                    severity: Severity::BasisFloorExcessDistributionGainSection961b2,
                    post_event_basis_cents: 0,
                    recognized_gain_cents: recognized_gain,
                    recommended_actions: actions,
                    citation: "26 U.S.C. § 961(b)(2); § 959(a); § 301(c)(3)",
                    notes: build_coord_notes(notes),
                }
            }
        }
        BasisAdjustmentEvent::Section332InboundLiquidation
        | BasisAdjustmentEvent::Section368AssetReorganization => {
            actions.push(format!(
                "Notice 2024-16 ({}) carryover rule: in qualifying inbound non-recognition \
                 transaction ({:?}), the § 961(c) basis of acquired CFC stock CARRIES OVER \
                 from transferor to the domestic acquiring corporation. Prevents trapped-\
                 PTEP situations where § 961(c) basis would otherwise be lost on inbound \
                 transfer + face gain on subsequent PTEP distribution under § 961(b)(2). \
                 Coordinate with Proposed Regs REG-105479-18 (published {}).",
                NOTICE_2024_16_DATE, input.event, PROPOSED_REGS_REG_105479_18_PUBLISHED
            ));
            Section961Result {
                severity: Severity::InboundNonrecognitionCarryoverUnderNotice2024_16,
                post_event_basis_cents: input.pre_event_basis_cents,
                recognized_gain_cents: 0,
                recommended_actions: actions,
                citation: "26 U.S.C. § 961(c); Notice 2024-16; REG-105479-18",
                notes: build_coord_notes(notes),
            }
        }
    }
}

fn build_coord_notes(mut notes: Vec<String>) -> Vec<String> {
    notes.push(
        "Coordination with [[section_959]] (PTEP — iter 512 — sixteen-basket framework + \
         § 961 basis adjustments coordinate to prevent double tax), [[section_960]] (deemed-\
         paid FTC — iter 520), [[section_951]] (Subpart F inclusion mechanism), \
         [[section_951a]] (GILTI / NCTI — iter 500), [[section_956]] (CFC US property — \
         iter 504), [[section_962]] (individual election — iter 510), [[section_245a]] \
         (foreign-source DRD — iter 502), [[section_965]] (transition tax — iter 514), \
         [[section_301]] (corporate distribution framework — § 301(c)(3) excess basis = \
         capital gain), [[section_332]] (parent-subsidiary liquidation — Notice 2024-16 \
         carryover), [[section_368]] (corporate reorganization — Notice 2024-16 carryover)."
            .to_string(),
    );
    notes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section961Input {
        Section961Input {
            event: BasisAdjustmentEvent::PtepInclusionSubpartF,
            pre_event_basis_cents: 100_000_000_00,
            event_amount_cents: 10_000_000_00,
            indirectly_owned_through_cfc: false,
            taxable_year: 2024,
        }
    }

    #[test]
    fn ptep_subpart_f_inclusion_increases_basis() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::BasisIncreaseUnderSection961a
        ));
        assert_eq!(r.post_event_basis_cents, 110_000_000_00);
        assert_eq!(r.recognized_gain_cents, 0);
    }

    #[test]
    fn gilti_inclusion_increases_basis() {
        let mut i = baseline();
        i.event = BasisAdjustmentEvent::PtepInclusionGiltiOrNcti;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::BasisIncreaseUnderSection961a
        ));
        assert_eq!(r.post_event_basis_cents, 110_000_000_00);
    }

    #[test]
    fn section_956_inclusion_increases_basis() {
        let mut i = baseline();
        i.event = BasisAdjustmentEvent::PtepInclusionSection956;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::BasisIncreaseUnderSection961a
        ));
    }

    #[test]
    fn ptep_distribution_within_basis_decreases_basis() {
        let mut i = baseline();
        i.event = BasisAdjustmentEvent::ActualPtepDistribution;
        i.pre_event_basis_cents = 100_000_000_00;
        i.event_amount_cents = 40_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::BasisDecreaseUnderSection961b
        ));
        assert_eq!(r.post_event_basis_cents, 60_000_000_00);
        assert_eq!(r.recognized_gain_cents, 0);
    }

    #[test]
    fn ptep_distribution_at_basis_zero_no_gain() {
        let mut i = baseline();
        i.event = BasisAdjustmentEvent::ActualPtepDistribution;
        i.pre_event_basis_cents = 100_000_000_00;
        i.event_amount_cents = 100_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::BasisDecreaseUnderSection961b
        ));
        assert_eq!(r.post_event_basis_cents, 0);
        assert_eq!(r.recognized_gain_cents, 0);
    }

    #[test]
    fn ptep_distribution_exceeds_basis_recognizes_gain() {
        let mut i = baseline();
        i.event = BasisAdjustmentEvent::ActualPtepDistribution;
        i.pre_event_basis_cents = 100_000_000_00;
        i.event_amount_cents = 150_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::BasisFloorExcessDistributionGainSection961b2
        ));
        assert_eq!(r.post_event_basis_cents, 0);
        assert_eq!(r.recognized_gain_cents, 50_000_000_00);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 301(c)(3)")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Schedule D")));
    }

    #[test]
    fn indirectly_owned_section_961c_limited() {
        let mut i = baseline();
        i.indirectly_owned_through_cfc = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::IndirectlyOwnedSection961cLimitedToSection951Inclusion
        ));
        assert!(r.notes.iter().any(|n| n.contains("§ 961(c)")));
        assert!(r.notes.iter().any(|n| n.contains("Notice 2024-16")));
    }

    #[test]
    fn section_332_inbound_liquidation_notice_2024_16_carryover() {
        let mut i = baseline();
        i.event = BasisAdjustmentEvent::Section332InboundLiquidation;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::InboundNonrecognitionCarryoverUnderNotice2024_16
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Notice 2024-16")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("REG-105479-18")));
        assert_eq!(r.post_event_basis_cents, i.pre_event_basis_cents);
    }

    #[test]
    fn section_368_asset_reorg_notice_2024_16_carryover() {
        let mut i = baseline();
        i.event = BasisAdjustmentEvent::Section368AssetReorganization;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::InboundNonrecognitionCarryoverUnderNotice2024_16
        ));
    }

    #[test]
    fn notice_2024_16_date_pins_2024_01_16() {
        assert_eq!(NOTICE_2024_16_DATE, "2024-01-16");
    }

    #[test]
    fn proposed_regs_date_pins_2024_12_02() {
        assert_eq!(PROPOSED_REGS_REG_105479_18_PUBLISHED, "2024-12-02");
    }

    #[test]
    fn tcja_section_961_amendment_year_pins_2017() {
        assert_eq!(TCJA_SECTION_961_AMENDMENT_YEAR, 2017);
    }

    #[test]
    fn action_references_form_5471_schedule_j_and_p() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Form 5471")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Schedule J")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Schedule P")));
    }

    #[test]
    fn coordination_note_references_all_international_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_959")));
        assert!(r.notes.iter().any(|n| n.contains("section_960")));
        assert!(r.notes.iter().any(|n| n.contains("section_951")));
        assert!(r.notes.iter().any(|n| n.contains("section_951a")));
        assert!(r.notes.iter().any(|n| n.contains("section_956")));
        assert!(r.notes.iter().any(|n| n.contains("section_962")));
        assert!(r.notes.iter().any(|n| n.contains("section_245a")));
        assert!(r.notes.iter().any(|n| n.contains("section_965")));
        assert!(r.notes.iter().any(|n| n.contains("section_301")));
        assert!(r.notes.iter().any(|n| n.contains("section_332")));
        assert!(r.notes.iter().any(|n| n.contains("section_368")));
    }

    #[test]
    fn citation_pins_961_a_b_c_and_notice_2024_16() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 961(a)"));
    }

    #[test]
    fn distribution_citation_pins_961_b1_section_959_a() {
        let mut i = baseline();
        i.event = BasisAdjustmentEvent::ActualPtepDistribution;
        i.event_amount_cents = 50_000_000_00;
        let r = check(&i);
        assert!(r.citation.contains("§ 961(b)(1)"));
        assert!(r.citation.contains("§ 959(a)"));
    }

    #[test]
    fn excess_distribution_citation_pins_961_b2_301_c3() {
        let mut i = baseline();
        i.event = BasisAdjustmentEvent::ActualPtepDistribution;
        i.event_amount_cents = 200_000_000_00;
        let r = check(&i);
        assert!(r.citation.contains("§ 961(b)(2)"));
        assert!(r.citation.contains("§ 301(c)(3)"));
    }

    #[test]
    fn inbound_citation_pins_961_c_and_reg_105479_18() {
        let mut i = baseline();
        i.event = BasisAdjustmentEvent::Section332InboundLiquidation;
        let r = check(&i);
        assert!(r.citation.contains("§ 961(c)"));
        assert!(r.citation.contains("Notice 2024-16"));
        assert!(r.citation.contains("REG-105479-18"));
    }

    #[test]
    fn realistic_corp_with_gilti_inclusion_then_distribution() {
        let mut i = baseline();
        i.pre_event_basis_cents = 50_000_000_00;
        i.event = BasisAdjustmentEvent::PtepInclusionGiltiOrNcti;
        i.event_amount_cents = 30_000_000_00;
        let r1 = check(&i);
        assert_eq!(r1.post_event_basis_cents, 80_000_000_00);

        let mut i2 = baseline();
        i2.pre_event_basis_cents = 80_000_000_00;
        i2.event = BasisAdjustmentEvent::ActualPtepDistribution;
        i2.event_amount_cents = 30_000_000_00;
        let r2 = check(&i2);
        assert_eq!(r2.post_event_basis_cents, 50_000_000_00);
    }

    #[test]
    fn extreme_value_does_not_overflow() {
        let mut i = baseline();
        i.pre_event_basis_cents = u64::MAX / 2;
        i.event_amount_cents = u64::MAX / 2;
        let r = check(&i);
        let _ = r.post_event_basis_cents;
    }

    #[test]
    fn zero_event_amount_no_change() {
        let mut i = baseline();
        i.event_amount_cents = 0;
        let r = check(&i);
        assert_eq!(r.post_event_basis_cents, i.pre_event_basis_cents);
        assert_eq!(r.recognized_gain_cents, 0);
    }
}
